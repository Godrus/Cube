extends Node3D
## World-generation spike client. Terrain lives in Rust (CubeWorld); this script only streams chunk
## nodes around the player, draws each chunk as one MultiMesh of cubes and shows a HUD.
##   godot --path godot-client world.tscn                          interactive (WASD / arrows, on-screen buttons)
##   godot --headless --path godot-client world.tscn -- --world-smoke   scripted check, exits 0/1
##   godot --path godot-client world.tscn -- --world-smoke              same check with a real renderer (timings, screenshot)
## Renderer: "cubes" (control: one MultiMesh cube per voxel) or "faces" (experimental: one mesh of the visible faces
## per chunk). Choose with --renderer=faces or the custom feature "faces"; the Renderer button switches at run time.

const SEED := 12345
const SPEED := 24.0
const PUMP_PER_FRAME := 2

var world: RefCounted
var player := Vector3(8.5, 0, 8.5)
var chunks := {}                       # Vector2i -> MultiMeshInstance3D
var box := BoxMesh.new()
var hud := Label.new()
var camera := Camera3D.new()
var marker := MeshInstance3D.new()
var hold := {"fwd": false, "back": false, "left": false, "right": false}
var smoke_running := false             # true only while the scripted smoke test drives the world
var update_us := []                    # microseconds spent building chunk MultiMeshes / meshes
var renderer := "cubes"
var drawn := {}                        # Vector2i -> drawn elements of the chunk (cubes or visible faces)
var face_mat := StandardMaterial3D.new()
var mode_button := Button.new()

func _ready() -> void:
	if not ClassDB.class_exists("CubeWorld"):
		print("WORLDTEST FAIL CubeWorld class missing (GDExtension not loaded)")
		if _smoke_mode():
			get_tree().quit(1)
		return
	world = ClassDB.instantiate("CubeWorld")
	if OS.has_feature("faces") or "--renderer=faces" in OS.get_cmdline_user_args():
		renderer = "faces"
	_build_scene()
	world.create(SEED, 1)
	_place_player(player.x, player.z)
	if _smoke_mode():
		smoke_running = true
		await _smoke()
		smoke_running = false
	else:
		_stream()

func _smoke_mode() -> bool:
	return "--world-smoke" in OS.get_cmdline_user_args() or OS.has_feature("worldtest")

func _build_scene() -> void:
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	box.material = mat
	face_mat.vertex_color_use_as_albedo = true
	var env := Environment.new()
	env.background_mode = Environment.BG_COLOR
	env.background_color = Color(0.3, 0.3, 0.3)
	env.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	env.ambient_light_color = Color(0.55, 0.55, 0.55)
	var we := WorldEnvironment.new()
	we.environment = env
	add_child(we)
	add_child(camera)
	var light := DirectionalLight3D.new()
	light.rotation_degrees = Vector3(-55, 35, 0)
	add_child(light)
	var m := BoxMesh.new()
	m.size = Vector3(1.2, 2.4, 1.2)
	var yellow := StandardMaterial3D.new()
	yellow.albedo_color = Color(1, 0.9, 0.1)
	m.material = yellow
	marker.mesh = m
	add_child(marker)

	var ui := CanvasLayer.new()
	add_child(ui)
	hud.position = Vector2(8, 4)
	ui.add_child(hud)
	var row := HBoxContainer.new()
	row.position = Vector2(0, 90)
	ui.add_child(row)
	for spec in [["Fwd", "fwd"], ["Back", "back"], ["Left", "left"], ["Right", "right"]]:
		var b := Button.new()
		b.text = spec[0]
		b.button_down.connect(func(): hold[spec[1]] = true)
		b.button_up.connect(func(): hold[spec[1]] = false)
		row.add_child(b)
	mode_button.pressed.connect(_on_mode)
	row.add_child(mode_button)
	_update_mode_button()
	for spec in [["Edit", _on_edit], ["Save", _on_save], ["Load", _on_load], ["Jump 64", _on_jump]]:
		var b := Button.new()
		b.text = spec[0]
		b.pressed.connect(spec[1])
		row.add_child(b)

func _place_player(x: float, z: float) -> void:
	player.x = x
	player.z = z
	player.y = world.ground_height(floori(x), floori(z)) + 1.0
	marker.position = player + Vector3(0, 0.2, 0)
	camera.look_at_from_position(player + Vector3(0, 16, 20), player)

func _process(delta: float) -> void:
	if world == null or smoke_running:
		return
	var d := Vector2.ZERO
	if Input.is_key_pressed(KEY_W) or Input.is_key_pressed(KEY_UP) or hold["fwd"]: d.y -= 1
	if Input.is_key_pressed(KEY_S) or Input.is_key_pressed(KEY_DOWN) or hold["back"]: d.y += 1
	if Input.is_key_pressed(KEY_A) or Input.is_key_pressed(KEY_LEFT) or hold["left"]: d.x -= 1
	if Input.is_key_pressed(KEY_D) or Input.is_key_pressed(KEY_RIGHT) or hold["right"]: d.x += 1
	if d != Vector2.ZERO:
		d = d.normalized() * SPEED * delta
		_place_player(player.x + d.x, player.z + d.y)
	_stream(PUMP_PER_FRAME)

## One streaming step: tell Rust where the player is, generate a few chunks, mirror the delta.
func _stream(budget: int = 1000) -> void:
	world.set_player(player.x, player.y, player.z)
	world.pump(budget)
	_apply_delta()
	_update_hud()

func _apply_delta() -> void:
	var dirty := {}                        # chunks whose node must be (re)built
	var removed: PackedInt32Array = world.take_removed()
	for i in range(0, removed.size(), 2):
		var k := Vector2i(removed[i], removed[i + 1])
		_drop_chunk(k)
		_mark_neighbours(dirty, k)
	var added: PackedInt32Array = world.take_added()
	for i in range(0, added.size(), 2):
		var k := Vector2i(added[i], added[i + 1])
		dirty[k] = true
		_mark_neighbours(dirty, k)
	var modified: PackedInt32Array = world.take_modified()
	for i in range(0, modified.size(), 2):
		var k := Vector2i(modified[i], modified[i + 1])
		dirty[k] = true
		_mark_neighbours(dirty, k)
	for k in dirty:
		_build_chunk(k)

## A face-culled mesh depends on the four neighbouring chunks (border faces), so they are rebuilt too.
func _mark_neighbours(dirty: Dictionary, k: Vector2i) -> void:
	if renderer != "faces":
		return
	for d in [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
		if chunks.has(k + d):
			dirty[k + d] = true

func _drop_chunk(k: Vector2i) -> void:
	if chunks.has(k):
		chunks[k].queue_free()
		chunks.erase(k)
	drawn.erase(k)

func _build_chunk(k: Vector2i) -> void:
	var t0 := Time.get_ticks_usec()
	var inst: Node3D = null
	var count := 0
	if renderer == "faces":
		var mesh: ArrayMesh = world.chunk_mesh(k.x, k.y)
		if mesh != null:
			mesh.surface_set_material(0, face_mat)
			var mi := MeshInstance3D.new()
			mi.mesh = mesh
			inst = mi
			count = world.chunk_face_count(k.x, k.y)
	else:
		var buf: PackedFloat32Array = world.chunk_instance_buffer(k.x, k.y)
		if not buf.is_empty():
			var mm := MultiMesh.new()
			mm.transform_format = MultiMesh.TRANSFORM_3D
			mm.use_colors = true
			mm.mesh = box
			mm.instance_count = buf.size() / 16
			mm.buffer = buf
			var mmi := MultiMeshInstance3D.new()
			mmi.multimesh = mm
			inst = mmi
			count = mm.instance_count
	_drop_chunk(k)
	if inst == null:
		return
	inst.name = "chunk_%d_%d" % [k.x, k.y]
	add_child(inst)
	chunks[k] = inst
	drawn[k] = count
	update_us.append(Time.get_ticks_usec() - t0)

## Drawable elements: cube instances (cubes) or visible faces (faces). A cube is 6 faces / 12 triangles.
func _drawn_elements() -> int:
	var n := 0
	for k in drawn:
		n += drawn[k]
	return n

func _drawn_triangles() -> int:
	return _drawn_elements() * (12 if renderer == "cubes" else 2)

func _on_mode() -> void:
	renderer = "faces" if renderer == "cubes" else "cubes"
	_update_mode_button()
	for k in chunks.keys():
		_build_chunk(k)

func _update_mode_button() -> void:
	mode_button.text = "Renderer: " + renderer

func _update_hud() -> void:
	hud.text = "seed %d   %s   active chunks %d (nodes %d)   voxels %d   drawn %d   queue %d   player (%.0f, %.0f, %.0f)   fps %d" % [world.seed(), renderer, world.active_count(), chunks.size(), world.active_voxels(), _drawn_elements(), world.queue_len(), player.x, player.y, player.z, Engine.get_frames_per_second()]

func _on_edit() -> void:
	var n: int = world.edit_around(floori(player.x), floori(player.y) + 2, floori(player.z))
	_apply_delta()
	_update_hud()
	print("edit changed %d voxels" % n)

func _on_save() -> void:
	var ok: bool = world.save(ProjectSettings.globalize_path("user://world.wst0"))
	hud.text = "saved" if ok else world.last_error()

func _on_load() -> void:
	var ok: bool = world.load(ProjectSettings.globalize_path("user://world.wst0"))
	if not ok:
		hud.text = world.last_error()
		return
	_clear_chunks()
	player = player          # the saved centre is restored on the Rust side; the camera stays where it is
	_apply_delta()
	_update_hud()

func _clear_chunks() -> void:
	for k in chunks.keys():
		chunks[k].queue_free()
	chunks.clear()
	drawn.clear()

func _on_jump() -> void:
	_place_player(player.x + 64.0, player.z)

# ---------------------------------------------------------------- smoke test

var _failures := 0

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("WORLDTEST %s %s %s" % ["PASS" if cond else "FAIL", name, detail])
	if not cond:
		_failures += 1

func _node_keys_match_active() -> bool:
	var a: PackedInt32Array = world.active_chunks()
	if a.size() / 2 != chunks.size():
		return false
	for i in range(0, a.size(), 2):
		if not chunks.has(Vector2i(a[i], a[i + 1])):
			return false
	return true

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _smoke() -> void:
	print("WORLDTEST ENV godot=%s gpu=%s mode=%s" % [Engine.get_version_info().string, RenderingServer.get_video_adapter_name(), renderer])
	_check("class available and seed set", world.seed() == SEED)
	var shot_dir := "user://"
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--shot="):
			shot_dir = a.substr(7)

	var t0 := Time.get_ticks_usec()
	_stream()
	var first_ms := (Time.get_ticks_usec() - t0) / 1000.0
	_check("3x3 chunks active", world.active_count() == 9, "active=%d" % world.active_count())
	_check("3x3 chunk nodes", chunks.size() == 9 and _node_keys_match_active())
	print("WORLDTEST TIME first_3x3 mode=%s total_ms=%.2f build_ms_per_chunk=%.2f voxels=%d drawn=%d" % [renderer, first_ms, _avg(update_us) / 1000.0, world.active_voxels(), _drawn_elements()])
	await _shot(shot_dir + "/world-%s-1-3x3.png" % renderer, Vector3(0, 40, 48))
	var data: PackedInt32Array = world.chunk_voxels(0, 0)
	_check("voxel data of ready chunk", data.size() > 0 and data.size() % 4 == 0, "voxels=%d" % (data.size() / 4))
	_check("unloaded chunk has no data", world.chunk_voxels(99, 99).is_empty() and world.chunk_voxel_count(99, 99) == -1)

	# Move the camera/player five chunks east (beyond radius + margin): far chunks must go, new ones must come.
	var before_keys := chunks.keys()
	_place_player(player.x + 80.0, player.z)
	_stream()
	_check("still 9 active after moving", world.active_count() == 9 and chunks.size() == 9 and _node_keys_match_active())
	var far_gone := true
	for k in before_keys:
		if k.x <= 0 and chunks.has(k):
			far_gone = false
	_check("every chunk left of x=1 was unloaded", far_gone)
	await _shot(shot_dir + "/world-%s-2-moved.png" % renderer, Vector3(0, 40, 48))
	_check("new neighbours loaded", chunks.has(Vector2i(4, 0)) and chunks.has(Vector2i(5, 0)) and chunks.has(Vector2i(6, 0)))

	# Edit near the player, then save, wander off, load.
	var te := Time.get_ticks_usec()
	var edited: int = world.edit_around(floori(player.x), floori(player.y) + 2, floori(player.z))
	_apply_delta()
	print("WORLDTEST TIME edit mode=%s total_ms=%.2f" % [renderer, (Time.get_ticks_usec() - te) / 1000.0])
	_check("edit changed 27 voxels", edited == 27 and world.edit_count() == 27, "edited=%d" % edited)
	var voxels_before: int = world.active_voxels()
	var path := ProjectSettings.globalize_path("user://world_smoke.wst0")
	_check("save", world.save(path), world.last_error())
	_place_player(-400.0, 300.0)
	_stream()
	_check("wandered to a different area", world.active_count() == 9 and not chunks.has(Vector2i(5, 0)))
	var tl := Time.get_ticks_usec()
	_check("load", world.load(path), world.last_error())
	_clear_chunks()
	_apply_delta()
	print("WORLDTEST TIME load mode=%s total_ms=%.2f" % [renderer, (Time.get_ticks_usec() - tl) / 1000.0])
	_check("loaded world has 9 active chunks and nodes", world.active_count() == 9 and chunks.size() == 9 and _node_keys_match_active())
	_check("voxel count restored", world.active_voxels() == voxels_before, "%d vs %d" % [world.active_voxels(), voxels_before])
	_check("edits restored", world.edit_count() == 27)

	# 5x5 and timings.
	world.create(SEED, 2)
	_clear_chunks()
	update_us.clear()
	_place_player(8.5, 8.5)
	t0 = Time.get_ticks_usec()
	_stream()
	var ms5 := (Time.get_ticks_usec() - t0) / 1000.0
	_check("5x5 chunks active", world.active_count() == 25 and chunks.size() == 25)
	print("WORLDTEST TIME first_5x5 mode=%s total_ms=%.2f build_ms_per_chunk=%.2f voxels=%d drawn=%d triangles=%d" % [renderer, ms5, _avg(update_us) / 1000.0, world.active_voxels(), _drawn_elements(), _drawn_triangles()])
	if renderer == "faces":
		_check("visible faces are far fewer than cube faces", _drawn_elements() * 10 < world.active_voxels() * 6, "faces=%d cube_faces=%d" % [_drawn_elements(), world.active_voxels() * 6])
		var sum := 0
		for k in chunks:
			sum += world.chunk_face_count(k.x, k.y)
		_check("face count from Rust equals drawn", sum == _drawn_elements())
	else:
		_check("one cube instance per voxel", _drawn_elements() == world.active_voxels())
	await _frames(60)
	var t1 := Time.get_ticks_usec()
	await _frames(300)
	var fps := 300.0 / ((Time.get_ticks_usec() - t1) / 1e6)
	print("WORLDTEST FPS 5x5 mode=%s fps=%.0f (headless/dummy renderer fps is not a rendering result)" % [renderer, fps])
	print("WORLDTEST MEM mode=%s static_mb=%.1f video_mb=%.1f draw_calls=%d primitives=%d" % [renderer, OS.get_static_memory_usage() / 1048576.0, RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_VIDEO_MEM_USED) / 1048576.0, RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_TOTAL_DRAW_CALLS_IN_FRAME), RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_TOTAL_PRIMITIVES_IN_FRAME)])
	await _shot(shot_dir + "/world-%s-3-5x5.png" % renderer, Vector3(0, 70, 80))
	print("WORLDTEST SUMMARY failures=%d" % _failures)
	# Mobile validation exports (custom feature "worldtest") stay alive so memory can be read from outside.
	if OS.has_feature("worldtest") and not ("--world-smoke" in OS.get_cmdline_user_args()):
		print("WORLDTEST DONE staying alive")
		return
	get_tree().quit(0 if _failures == 0 else 1)

## Saves a screenshot when a real renderer is present (skipped headless); camera is posed behind the player.
func _shot(path: String, offset: Vector3) -> void:
	if DisplayServer.get_name() == "headless":
		return
	camera.look_at_from_position(player + offset, player)
	await _frames(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	print("WORLDTEST SHOT %s err=%d" % [path, img.save_png(path)])

func _avg(a: Array) -> float:
	if a.is_empty():
		return 0.0
	var s := 0.0
	for v in a:
		s += v
	return s / a.size()
