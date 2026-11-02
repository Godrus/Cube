extends Node3D
## Spike client: a thin view over the Rust runtime (CubeRuntime from the GDExtension).
## It keeps only a display mirror of voxels; the Rust side owns the geometry.
##   godot --headless --path godot-client -- --smoke   (headless smoke run, exits 0/1)

var runtime: RefCounted
var mirror := {}                      # Vector3i -> material (display copy only)
var multi := MultiMesh.new()
var status := Label.new()
var step := 0

func _ready() -> void:
	if not ClassDB.class_exists("CubeRuntime"):
		push_error("CubeRuntime not found: build the Rust extension first (cargo build -p cube-godot-adapter).")
		if "--smoke" in OS.get_cmdline_user_args():
			get_tree().quit(1)
		return
	runtime = ClassDB.instantiate("CubeRuntime")
	_build_scene()
	if "--smoke" in OS.get_cmdline_user_args():
		get_tree().quit(0 if _smoke() else 1)

func _build_scene() -> void:
	var cam := Camera3D.new()
	add_child(cam)
	cam.position = Vector3(14, 12, 18)
	cam.look_at_from_position(cam.position, Vector3(0, 2, 0))
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
	status.position = Vector2(8, 56)
	ui.add_child(status)
	_set_status("ready")

func _on_fill() -> void:
	step += 1
	var x := (step * 3) % 12 - 6
	var z := (step * 5) % 12 - 6
	_apply(runtime.fill_region(x, 0, z, x + 4, 1 + step % 4, z + 4, 1 + step % 6))

func _on_clear() -> void:
	_apply(runtime.fill_region(-8, 0, -8, 8, 8, 8, 0))

func _on_undo() -> void:
	_apply(runtime.undo())

func _on_redo() -> void:
	_apply(runtime.redo())

func _on_save() -> void:
	var ok: bool = runtime.save(ProjectSettings.globalize_path("user://spike.cube"))
	_set_status("saved" if ok else runtime.last_error())

func _on_load() -> void:
	_apply(runtime.load(ProjectSettings.globalize_path("user://spike.cube")))

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
