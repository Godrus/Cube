extends Node3D
## Isolation scene without GDExtension. Mode comes from the Android property debug.cube.min:
##   label   : only a Label (no 3D content)
##   scene3d : camera, light, 25 chunk-like ArrayMeshes with vertex colours + a MultiMesh of cubes (the same
##             rendering features as the Cube client, but built from GDScript, no Rust)
func _ready() -> void:
	var mode := "scene3d"
	var execs := 0
	if OS.get_name() == "Android" and not OS.has_feature("noexec"):
		var out := []
		OS.execute("getprop", ["debug.cube.min"], out)
		execs = 1
		if out.size() > 0 and String(out[0]).strip_edges() != "":
			mode = String(out[0]).strip_edges()
		if mode == "exec4":                       # three more child processes, like the Cube client does
			for i in 3:
				OS.execute("getprop", ["debug.cube.min"], [])
				execs += 1
			mode = "scene3d"
	print("MINIMAL start mode=%s os_execute_calls=%d godot=%s" % [mode, execs, Engine.get_version_info().string])
	var ui := CanvasLayer.new()
	add_child(ui)
	var l := Label.new()
	l.text = "minimal isolation: " + mode
	l.position = Vector2(20, 20)
	ui.add_child(l)
	if mode == "scene3d":
		_scene3d()
	for i in 120:
		await get_tree().process_frame
	print("MINIMAL frames_done mode=%s" % mode)
	print("MINIMAL DONE staying alive")

func _scene3d() -> void:
	var env := Environment.new()
	env.background_mode = Environment.BG_COLOR
	env.background_color = Color(0.3, 0.3, 0.3)
	env.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	env.ambient_light_color = Color(0.55, 0.55, 0.55)
	var we := WorldEnvironment.new()
	we.environment = env
	add_child(we)
	var cam := Camera3D.new()
	add_child(cam)
	cam.look_at_from_position(Vector3(40, 60, 90), Vector3(40, 5, 40))
	var light := DirectionalLight3D.new()
	light.rotation_degrees = Vector3(-55, 35, 0)
	add_child(light)
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	for cz in 5:
		for cx in 5:
			var verts := PackedVector3Array()
			var norms := PackedVector3Array()
			var cols := PackedColorArray()
			var idx := PackedInt32Array()
			for z in 16:
				for x in 16:
					var h := 6.0 + float((cx * 7 + cz * 13 + x + z) % 5)
					var b := verts.size()
					var wx := float(cx * 16 + x)
					var wz := float(cz * 16 + z)
					verts.append_array([Vector3(wx, h, wz), Vector3(wx, h, wz + 1), Vector3(wx + 1, h, wz + 1), Vector3(wx + 1, h, wz)])
					for k in 4:
						norms.append(Vector3.UP)
						cols.append(Color(0.35, 0.75, 0.25))
					idx.append_array([b, b + 1, b + 2, b, b + 2, b + 3])
			var arrays := []
			arrays.resize(Mesh.ARRAY_MAX)
			arrays[Mesh.ARRAY_VERTEX] = verts
			arrays[Mesh.ARRAY_NORMAL] = norms
			arrays[Mesh.ARRAY_COLOR] = cols
			arrays[Mesh.ARRAY_INDEX] = idx
			var mesh := ArrayMesh.new()
			mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
			mesh.surface_set_material(0, mat)
			var mi := MeshInstance3D.new()
			mi.mesh = mesh
			add_child(mi)
	var box := BoxMesh.new()
	box.material = mat
	var mm := MultiMesh.new()
	mm.transform_format = MultiMesh.TRANSFORM_3D
	mm.use_colors = true
	mm.mesh = box
	mm.instance_count = 2000
	for i in 2000:
		mm.set_instance_transform(i, Transform3D(Basis(), Vector3(float(i % 40), 12.0 + float(i / 400), float((i / 40) % 10))))
		mm.set_instance_color(i, Color(0.9, 0.5, 0.1))
	var mmi := MultiMeshInstance3D.new()
	mmi.multimesh = mm
	add_child(mmi)
