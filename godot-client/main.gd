extends Node3D
## Spike client: a thin view over the Rust runtime (CubeRuntime from the GDExtension).
## It keeps only a display mirror of voxels; the Rust side owns the geometry.
##   godot --headless --path godot-client -- --smoke      headless smoke run, exits 0/1
##   godot --path godot-client -- --ui-test [--shot=PATH]  windowed run that clicks the real buttons, exits 0/1

var runtime: RefCounted
var mirror := {}                      # Vector3i -> material (display copy only)
var multi := MultiMesh.new()
var status := Label.new()
var camera: Camera3D
var buttons := {}                     # name -> Button
var clicks := {}                      # name -> times the handler ran
var timings := {}                     # name -> {"total_us": int, "rust_us": int}
var step := 0

func _ready() -> void:
	if not ClassDB.class_exists("CubeRuntime"):
		push_error("CubeRuntime not found: build the Rust extension first (cargo build -p cube-godot-adapter).")
		print("UITEST FAIL CubeRuntime class missing (GDExtension not loaded)")
		if _has_arg("--smoke") or _has_arg("--ui-test"):
			get_tree().quit(1)
		return
	runtime = ClassDB.instantiate("CubeRuntime")
	_build_scene()
	if _has_arg("--smoke"):
		get_tree().quit(0 if _smoke() else 1)
	elif _has_arg("--ui-test") or OS.has_feature("autotest"):
		_ui_test()

func _has_arg(a: String) -> bool:
	return a in OS.get_cmdline_user_args()

func _build_scene() -> void:
	camera = Camera3D.new()
	add_child(camera)
	camera.position = Vector3(14, 12, 18)
	camera.look_at_from_position(camera.position, Vector3(0, 2, 0))
	var light := DirectionalLight3D.new()
	add_child(light)
	light.rotation_degrees = Vector3(-50, 30, 0)

	var box := BoxMesh.new()
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	box.material = mat
	multi.transform_format = MultiMesh.TRANSFORM_3D
	multi.use_colors = true
	multi.mesh = box
	var mmi := MultiMeshInstance3D.new()
	mmi.multimesh = multi
	add_child(mmi)

	var ui := CanvasLayer.new()
	add_child(ui)
	var row := HBoxContainer.new()
	ui.add_child(row)
	for spec in [["Fill", _on_fill], ["Clear", _on_clear], ["Undo", _on_undo], ["Redo", _on_redo], ["Save", _on_save], ["Load", _on_load]]:
		var b := Button.new()
		b.text = spec[0]
		b.pressed.connect(spec[1])
		row.add_child(b)
		buttons[spec[0]] = b
		clicks[spec[0]] = 0
	status.position = Vector2(8, 56)
	ui.add_child(status)
	_set_status("ready")

func _on_fill() -> void:
	clicks["Fill"] += 1
	step += 1
	var x := (step * 3) % 12 - 6
	var z := (step * 5) % 12 - 6
	_timed("Fill", func(): return runtime.fill_region(x, 0, z, x + 4, 1 + step % 4, z + 4, 1 + step % 6))

func _on_clear() -> void:
	clicks["Clear"] += 1
	_timed("Clear", func(): return runtime.fill_region(-8, 0, -8, 8, 8, 8, 0))

func _on_undo() -> void:
	clicks["Undo"] += 1
	_timed("Undo", func(): return runtime.undo())

func _on_redo() -> void:
	clicks["Redo"] += 1
	_timed("Redo", func(): return runtime.redo())

func _on_save() -> void:
	clicks["Save"] += 1
	var t0 := Time.get_ticks_usec()
	var ok: bool = runtime.save(ProjectSettings.globalize_path("user://spike.cube"))
	timings["Save"] = {"total_us": Time.get_ticks_usec() - t0, "rust_us": Time.get_ticks_usec() - t0}
	_set_status("saved" if ok else runtime.last_error())

func _on_load() -> void:
	clicks["Load"] += 1
	_timed("Load", func(): return runtime.load(ProjectSettings.globalize_path("user://spike.cube")))

## Runs a runtime call and applies its result; records how long the Rust call and the whole handler took.
func _timed(name: String, call: Callable) -> void:
	var t0 := Time.get_ticks_usec()
	var ok: bool = call.call()
	var rust_us := Time.get_ticks_usec() - t0
	_apply(ok)
	timings[name] = {"total_us": Time.get_ticks_usec() - t0, "rust_us": rust_us}

func _apply(ok: bool) -> void:
	if not ok:
		_set_status(runtime.last_error())
		return
	if runtime.last_reset():
		mirror.clear()
	var d: PackedInt32Array = runtime.last_dirty()
	for i in range(0, d.size(), 6):                       # clear each dirty region, then re-add
		for x in range(d[i], d[i + 3]):
			for y in range(d[i + 1], d[i + 4]):
				for z in range(d[i + 2], d[i + 5]):
					mirror.erase(Vector3i(x, y, z))
	var c: PackedInt32Array = runtime.last_cells()
	for i in range(0, c.size(), 4):
		mirror[Vector3i(c[i], c[i + 1], c[i + 2])] = c[i + 3]
	_rebuild()
	_set_status("epoch %d  asset rev %d  layer rev %d  voxels %d" % [runtime.epoch(), runtime.asset_revision(), runtime.layer_revision(), mirror.size()])

func _rebuild() -> void:
	multi.instance_count = mirror.size()
	var i := 0
	for p in mirror:
		multi.set_instance_transform(i, Transform3D(Basis(), Vector3(p) + Vector3(0.5, 0.5, 0.5)))
		multi.set_instance_color(i, Color.from_hsv(fmod(mirror[p] * 0.13, 1.0), 0.6, 0.9))
		i += 1
	_frame_camera()

## Keeps the whole mirror in view: camera sits outside the bounding box, looking at its center.
func _frame_camera() -> void:
	if mirror.is_empty():
		return
	var lo := Vector3(INF, INF, INF)
	var hi := Vector3(-INF, -INF, -INF)
	for p in mirror:
		lo = lo.min(Vector3(p))
		hi = hi.max(Vector3(p) + Vector3.ONE)
	var center := (lo + hi) * 0.5
	var radius := maxf(((hi - lo) * 0.5).length(), 2.0)
	var dir := Vector3(0.55, 0.5, 0.7).normalized()
	var dist := radius / sin(deg_to_rad(camera.fov * 0.5)) * 1.15
	camera.look_at_from_position(center + dir * dist, center)

func _set_status(t: String) -> void:
	status.text = t
	print(t)

func _smoke() -> bool:
	_apply(runtime.fill_region(-2, 0, -2, 2, 2, 2, 4))
	if mirror.size() != 32: return false
	_apply(runtime.undo())
	if mirror.size() != 0: return false
	_apply(runtime.redo())
	if mirror.size() != 32: return false
	var p := ProjectSettings.globalize_path("user://smoke.cube")
	if not runtime.save(p): return false
	_apply(runtime.fill_region(-2, 0, -2, 2, 2, 2, 0))
	_apply(runtime.load(p))
	return mirror.size() == 32

# ---------------------------------------------------------------- windowed UI test

var _failures := 0

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("UITEST %s %s %s" % ["PASS" if cond else "FAIL", name, detail])
	if not cond:
		_failures += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## Clicks with real mouse events; falls back to emitting the signal and says so.
func _click(name: String) -> void:
	var b: Button = buttons[name]
	var before: int = clicks[name]
	var pos := b.get_global_rect().get_center()
	# Leave and re-enter the button so hover state is fresh on every click.
	for target in [pos + Vector2(300, 300), pos]:
		var mv := InputEventMouseMotion.new()
		mv.position = target
		mv.global_position = target
		Input.parse_input_event(mv)
		await _frames(3)
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.button_mask = MOUSE_BUTTON_MASK_LEFT
	down.pressed = true
	down.position = pos
	down.global_position = pos
	Input.parse_input_event(down)
	await _frames(2)
	var up := down.duplicate()
	up.pressed = false
	up.button_mask = 0
	Input.parse_input_event(up)
	await _frames(2)
	if clicks[name] == before:
		print("UITEST NOTE %s: real mouse click did not reach the button, using pressed.emit()" % name)
		b.pressed.emit()
		await _frames(2)
	await _frames(2)

## Wall-clock FPS over 300 rendered frames after a 30-frame warm-up. Engine.get_frames_per_second()
## is not used: it only updates once per second and reads ~1 in a short test.
func _close(a: Color, b: Color) -> bool:
	return absf(a.r - b.r) < 0.02 and absf(a.g - b.g) < 0.02 and absf(a.b - b.b) < 0.02

func _fps(label: String) -> float:
	await _frames(30)
	var t0 := Time.get_ticks_usec()
	await _frames(300)
	var dt := (Time.get_ticks_usec() - t0) / 1e6
	var f := 300.0 / dt
	print("UITEST FPS %s fps=%.0f frame_ms=%.2f voxels=%d vsync=%d" % [label, f, 1000.0 / f, multi.instance_count, DisplayServer.window_get_vsync_mode()])
	return f

func _timing(name: String) -> void:
	var t: Dictionary = timings.get(name, {})
	print("UITEST TIME %s total_ms=%.2f rust_ms=%.2f" % [name, t.get("total_us", -1) / 1000.0, t.get("rust_us", -1) / 1000.0])

func _shot(path: String) -> void:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	print("UITEST SHOT %s %dx%d err=%d" % [path, img.get_width(), img.get_height(), img.save_png(path)])

func _ui_test() -> void:
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)   # measure uncapped
	var shot_dir := "user://"
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--shot="):
			shot_dir = a.substr(7)
	print("UITEST STARTUP engine_ticks_ms=%d" % Time.get_ticks_msec())
	await _frames(10)
	print("UITEST ENV godot=%s renderer=%s adapter=%s" % [Engine.get_version_info().string, RenderingServer.get_video_adapter_name(), ProjectSettings.get_setting("rendering/renderer/rendering_method")])
	_check("camera is current", get_viewport().get_camera_3d() == camera)
	_check("scene has MultiMeshInstance3D", get_children().any(func(n): return n is MultiMeshInstance3D))
	_check("six buttons visible", buttons.size() == 6 and buttons.values().all(func(b): return b.is_visible_in_tree() and b.size.x > 0))
	_check("starts empty", multi.instance_count == 0)
	print("UITEST WINDOW size=%s" % get_window().size)
	for n in buttons:
		print("UITEST BTN %s center=%s" % [n, buttons[n].get_global_rect().get_center()])

	# ---- minimal volume: the real Fill button
	await _click("Fill")
	_check("Fill ran by click", clicks["Fill"] == 1)
	var after_fill := multi.instance_count
	_check("Fill shows cubes", after_fill > 0 and after_fill == mirror.size(), "voxels=%d" % after_fill)
	_timing("Fill")
	var fps_min := await _fps("minimal")
	await _shot(shot_dir + "/interactive-1-fill.png")

	await _click("Fill")
	var after_fill2 := multi.instance_count
	_check("second Fill adds or replaces cubes", after_fill2 >= after_fill, "voxels=%d" % after_fill2)

	await _click("Undo")
	_check("Undo reverts to previous count", multi.instance_count == after_fill, "voxels=%d" % multi.instance_count)
	_timing("Undo")
	await _click("Redo")
	_check("Redo restores count", multi.instance_count == after_fill2, "voxels=%d" % multi.instance_count)
	_timing("Redo")

	await _click("Save")
	_check("Save ok", runtime.last_error() == "" and FileAccess.file_exists("user://spike.cube"))
	_timing("Save")
	var saved_count := multi.instance_count
	await _click("Clear")
	_check("Clear empties the view", multi.instance_count == 0, "voxels=%d" % multi.instance_count)
	_timing("Clear")
	await _shot(shot_dir + "/interactive-2-cleared.png")
	await _click("Load")
	_check("Load restores saved count", multi.instance_count == saved_count, "voxels=%d" % multi.instance_count)
	_check("Load starts new epoch", runtime.epoch() >= 1, "epoch=%d" % runtime.epoch())
	_timing("Load")
	await _shot(shot_dir + "/interactive-3-loaded.png")

	# ---- larger volume through the same apply path (not via button: the button box is small)
	var t0 := Time.get_ticks_usec()
	_apply(runtime.fill_region(-16, 0, -16, 16, 16, 16, 3))        # 16384 voxels
	var big_total := Time.get_ticks_usec() - t0
	print("UITEST TIME FillBig16384 total_ms=%.2f" % (big_total / 1000.0))
	_check("big fill shows 16384 cubes", multi.instance_count == 16384, "voxels=%d" % multi.instance_count)
	var fps_big := await _fps("16384")
	await _shot(shot_dir + "/interactive-4-big.png")
	var img := get_viewport().get_texture().get_image()
	var clear := RenderingServer.get_default_clear_color()
	_check("big view leaves background visible (camera outside volume)", _close(img.get_pixel(img.get_width() - 4, img.get_height() - 4), clear), "corner=%s clear=%s" % [img.get_pixel(img.get_width() - 4, img.get_height() - 4), clear])
	var half := Vector3(16, 8, 16)
	_check("camera is outside the filled volume", (camera.global_position - Vector3(0, 8, 0)).abs().max_axis_index() >= 0 and (camera.global_position - Vector3(0, 8, 0)).length() > half.length())
	t0 = Time.get_ticks_usec()
	_apply(runtime.undo())
	print("UITEST TIME UndoBig total_ms=%.2f" % ((Time.get_ticks_usec() - t0) / 1000.0))
	t0 = Time.get_ticks_usec()
	_apply(runtime.redo())
	print("UITEST TIME RedoBig total_ms=%.2f" % ((Time.get_ticks_usec() - t0) / 1000.0))
	_check("big redo shows 16384 cubes", multi.instance_count == 16384)
	runtime.save(ProjectSettings.globalize_path("user://big.cube"))
	_apply(runtime.fill_region(-16, 0, -16, 16, 16, 16, 0))
	t0 = Time.get_ticks_usec()
	_apply(runtime.load(ProjectSettings.globalize_path("user://big.cube")))
	print("UITEST TIME LoadBig total_ms=%.2f" % ((Time.get_ticks_usec() - t0) / 1000.0))
	_check("big load shows 16384 cubes", multi.instance_count == 16384)

	_check("no runtime error left", runtime.last_error() == "", runtime.last_error())
	print("UITEST SUMMARY failures=%d fps_min=%.0f fps_big=%.0f" % [_failures, fps_min, fps_big])
	# Export builds with the custom feature "autotest" (mobile validation) keep running so memory and
	# touch input can be inspected from outside; command-line runs exit with the result code.
	if OS.has_feature("autotest") and not _has_arg("--ui-test"):
		print("UITEST DONE staying alive")
		return
	get_tree().quit(0 if _failures == 0 else 1)
