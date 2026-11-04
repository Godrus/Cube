extends Node3D
## World-generation spike client. Terrain lives in Rust (CubeWorld); this script only streams chunk
## nodes around the player, draws each chunk as one MultiMesh of cubes and shows a HUD.
##   godot --path godot-client world.tscn                          interactive (WASD / arrows, on-screen buttons)
##   godot --headless --path godot-client world.tscn -- --world-smoke   scripted check, exits 0/1
##   godot --path godot-client world.tscn -- --world-smoke              same check with a real renderer (timings, screenshot)
## Renderer: "cubes" (control: one MultiMesh cube per voxel) or "faces" (experimental: one mesh of the visible faces
## per section) or "greedy" (experimental: the visible faces of a section merged into maximal same-material
## rectangles). Choose with --renderer=<mode>, the custom feature "faces"/"greedy" or, on Android,
## `setprop debug.cube.renderer <mode>`; the Renderer button cycles cubes -> faces -> greedy at run time.
## A chunk (16x32x16) has two mesh sections, one per 16^3 brick. After an edit the faces renderer rebuilds, by
## --rebuild=<policy> (Android: `setprop debug.cube.rebuild <policy>`):
##   partial (default)  only the dirty sections reported by Rust (edited voxel + its six face neighbours)
##   chunk              every section of the edited chunks and of neighbours across an edited border
##   all                every section of the edited chunks and of all four neighbour chunks (full rebuild fallback)

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
var rebuild_policy := "partial"        # faces renderer after an edit: "partial", "chunk" or "all" (see header)
var last_rebuilt := 0                  # mesh sections (faces) or chunk nodes (cubes) rebuilt by the last _apply_delta
var last_rebuilt_chunks := 0           # distinct chunks among them
var mesh_stats := {}                   # Vector3i section -> Vector3i(vertices, indices, drawn elements)
var sec_nodes := {}                    # Vector3i(chunk x, brick, chunk z) -> MeshInstance3D (faces renderer)
var rust_us := 0                       # microseconds inside world.section_mesh (Rust meshing + ArrayMesh creation)
var node_us := 0                       # microseconds spent replacing Godot nodes

func _ready() -> void:
	if not ClassDB.class_exists("CubeWorld"):
		print("WORLDTEST FAIL CubeWorld class missing (GDExtension not loaded)")
		if _smoke_mode():
			get_tree().quit(1)
		return
	world = ClassDB.instantiate("CubeWorld")
	var rmode := _renderer_arg()
	if rmode in ["cubes", "faces", "greedy"]:
		renderer = rmode
	elif OS.has_feature("greedy"):
		renderer = "greedy"
	elif OS.has_feature("faces"):
		renderer = "faces"
	var arg := _rebuild_arg()
	if arg in ["partial", "chunk", "all"]:
		rebuild_policy = arg
	_build_scene()
	world.create(SEED, 1)
	_place_player(player.x, player.z)
	if _smoke_mode():
		smoke_running = true
		await _smoke()
		smoke_running = false
	else:
		_stream()

## Renderer requested from outside: --renderer=<m>, or on Android the system property debug.cube.renderer.
## The property is read with OS.execute("getprop"), which forks a child process. On the ARM64 emulator (instruction
## translation) forking while the render thread runs crashed the app (report section 26), so the validation presets
## marked "noexec" never fork and choose the renderer by custom feature ("faces" / "greedy") instead.
func _renderer_arg() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--renderer="):
			return a.substr(11)
	if OS.get_name() == "Android" and not OS.has_feature("noexec"):   # OS.execute forks: see report, ARM64 translation
		var out := []
		OS.execute("getprop", ["debug.cube.renderer"], out)
		if out.size() > 0:
			return String(out[0]).strip_edges()
	return ""

func _is_mesh() -> bool:
	return renderer != "cubes"

## Rebuild policy requested from outside: --rebuild=<p>, or on Android the system property debug.cube.rebuild.
func _rebuild_arg() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--rebuild="):
			return a.substr(10)
	if OS.get_name() == "Android" and not OS.has_feature("noexec"):
		var out := []
		OS.execute("getprop", ["debug.cube.rebuild"], out)
		if out.size() > 0:
			return String(out[0]).strip_edges()
	return ""

func _smoke_mode() -> bool:
	return "--world-smoke" in OS.get_cmdline_user_args() or OS.has_feature("worldtest")

func _build_scene() -> void:
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	box.material = mat
	face_mat.vertex_color_use_as_albedo = true
	# Diffuse only: lighting then depends on the (constant) normal alone. With specular the view vector comes from the
	# interpolated fragment position, which differs by float noise between two triangulations of the same surface.
	face_mat.specular_mode = BaseMaterial3D.SPECULAR_DISABLED
	face_mat.metallic_specular = 0.0
	face_mat.diffuse_mode = BaseMaterial3D.DIFFUSE_LAMBERT       # Burley diffuse also depends on the view vector
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
	for spec in [["Edit", _on_edit], ["Undo", _on_undo], ["Redo", _on_redo], ["Save", _on_save], ["Load", _on_load], ["Jump 64", _on_jump]]:
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
	var dirty := {}                        # faces: Vector3i sections; cubes: Vector2i chunks
	var removed: PackedInt32Array = world.take_removed()
	for i in range(0, removed.size(), 2):
		var k := Vector2i(removed[i], removed[i + 1])
		_drop_chunk(k)
		_mark_neighbours(dirty, k)
	var added: PackedInt32Array = world.take_added()
	for i in range(0, added.size(), 2):
		var k := Vector2i(added[i], added[i + 1])
		if _is_mesh():
			_ensure_container(k)
		_mark_chunk(dirty, k)
		_mark_neighbours(dirty, k)
	var modified: PackedInt32Array = world.take_modified()
	var touched: PackedInt32Array = world.take_border_touched()
	var secs: PackedInt32Array = world.take_dirty_sections()
	if _is_mesh() and rebuild_policy == "partial":
		# Rust reports exactly the sections whose faces can change (voxel + six face neighbours).
		for i in range(0, secs.size(), 3):
			if chunks.has(Vector2i(secs[i], secs[i + 2])):
				dirty[Vector3i(secs[i], secs[i + 1], secs[i + 2])] = true
	elif _is_mesh() and rebuild_policy == "chunk":
		for i in range(0, modified.size(), 2):
			_mark_chunk(dirty, Vector2i(modified[i], modified[i + 1]))
		for i in range(0, touched.size(), 2):
			var k := Vector2i(touched[i], touched[i + 1])
			if chunks.has(k):
				_mark_chunk(dirty, k)
	else:
		for i in range(0, modified.size(), 2):
			var k := Vector2i(modified[i], modified[i + 1])
			_mark_chunk(dirty, k)
			if _is_mesh():                       # rebuild policy "all"
				_mark_neighbours(dirty, k)
	last_rebuilt = dirty.size()
	var chunk_set := {}
	for k in dirty:
		chunk_set[Vector2i(k.x, k.z) if k is Vector3i else k] = true
	last_rebuilt_chunks = chunk_set.size()
	for k in dirty:
		if _is_mesh():
			_build_section(k)
		else:
			_build_chunk(k)

func _mark_chunk(dirty: Dictionary, k: Vector2i) -> void:
	if _is_mesh():
		dirty[Vector3i(k.x, 0, k.y)] = true
		dirty[Vector3i(k.x, 1, k.y)] = true
	else:
		dirty[k] = true

## A face-culled mesh depends on the four neighbouring chunks (border faces): for added or removed chunks and
## for the "all" policy every loaded neighbour is rebuilt.
func _mark_neighbours(dirty: Dictionary, k: Vector2i) -> void:
	if not _is_mesh():
		return
	for d in [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
		if chunks.has(k + d):
			_mark_chunk(dirty, k + d)

func _ensure_container(k: Vector2i) -> void:
	if chunks.has(k):
		return
	var c := Node3D.new()
	c.name = "chunk_%d_%d" % [k.x, k.y]
	add_child(c)
	chunks[k] = c

func _drop_chunk(k: Vector2i) -> void:
	if chunks.has(k):
		chunks[k].queue_free()
		chunks.erase(k)
	drawn.erase(k)
	for bi in 2:
		sec_nodes.erase(Vector3i(k.x, bi, k.y))
		drawn.erase(Vector3i(k.x, bi, k.y))
		mesh_stats.erase(Vector3i(k.x, bi, k.y))

## Faces renderer: replaces the mesh of one section (brick) of a chunk.
func _build_section(s: Vector3i) -> void:
	var t0 := Time.get_ticks_usec()
	var mesh: ArrayMesh = world.section_mesh_greedy(s.x, s.y, s.z) if renderer == "greedy" else world.section_mesh(s.x, s.y, s.z)
	var t1 := Time.get_ticks_usec()
	var key := Vector2i(s.x, s.z)
	if sec_nodes.has(s):
		sec_nodes[s].free()
		sec_nodes.erase(s)
		drawn.erase(s)
		mesh_stats.erase(s)
	if mesh != null and chunks.has(key):
		mesh.surface_set_material(0, face_mat)
		var mi := MeshInstance3D.new()
		mi.mesh = mesh
		mi.name = "section_%d" % s.y
		chunks[key].add_child(mi)
		sec_nodes[s] = mi
		var idx_len: int = mesh.surface_get_array_index_len(0)
		var elements: int = mesh.get_meta("elements", idx_len / 6)
		drawn[s] = elements
		mesh_stats[s] = Vector3i(mesh.surface_get_array_len(0), idx_len, elements)
	var t2 := Time.get_ticks_usec()
	rust_us += t1 - t0
	node_us += t2 - t1
	update_us.append(t2 - t0)

func _build_chunk(k: Vector2i) -> void:
	var t0 := Time.get_ticks_usec()
	var inst: Node3D = null
	var count := 0
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
	if renderer == "cubes":
		return _drawn_elements() * 12
	var n := 0
	for k in mesh_stats:
		n += mesh_stats[k].y / 3
	return n

func _on_mode() -> void:
	_set_renderer({"cubes": "faces", "faces": "greedy", "greedy": "cubes"}[renderer])

## Switches the renderer in the running app: every chunk is rebuilt in the new mode.
func _set_renderer(mode: String) -> void:
	renderer = mode
	_update_mode_button()
	_clear_chunks()
	var a: PackedInt32Array = world.active_chunks()
	var dirty := {}
	for i in range(0, a.size(), 2):
		var k := Vector2i(a[i], a[i + 1])
		if _is_mesh():
			_ensure_container(k)
		_mark_chunk(dirty, k)
	for k in dirty:
		if _is_mesh():
			_build_section(k)
		else:
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
	sec_nodes.clear()
	mesh_stats.clear()

func _on_undo() -> void:
	var n: int = world.undo()
	_apply_delta()
	_update_hud()
	print("undo restored %d voxels" % n)

func _on_redo() -> void:
	var n: int = world.redo()
	_apply_delta()
	_update_hud()
	print("redo applied %d voxels" % n)

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
	print("WORLDTEST GEOM mode=%s elements=%d triangles=%d mesh_bytes=%d voxels=%d nodes=%d" % [renderer, _drawn_elements(), _drawn_triangles(), _mesh_bytes(), world.active_voxels(), _node_count()])
	if _is_mesh():
		_check("drawn elements are far fewer than cube faces", _drawn_elements() * 10 < world.active_voxels() * 6, "elements=%d cube_faces=%d" % [_drawn_elements(), world.active_voxels() * 6])
		var sum := 0
		for k in chunks:
			sum += world.chunk_face_count(k.x, k.y)
		var totals := _geom_totals()
		_check("covered unit faces from Rust equal the visible faces", totals["area"] == sum, "%d vs %d" % [totals["area"], sum])
		_check("no stale section after the first fill", totals["stale"] == 0)
	else:
		_check("one cube instance per voxel", _drawn_elements() == world.active_voxels())
	await _frames(60)
	var t1 := Time.get_ticks_usec()
	await _frames(300)
	var fps := 300.0 / ((Time.get_ticks_usec() - t1) / 1e6)
	print("WORLDTEST FPS 5x5 mode=%s fps=%.0f (headless/dummy renderer fps is not a rendering result)" % [renderer, fps])
	print("WORLDTEST MEM mode=%s static_mb=%.1f video_mb=%.1f draw_calls=%d primitives=%d" % [renderer, OS.get_static_memory_usage() / 1048576.0, RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_VIDEO_MEM_USED) / 1048576.0, RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_TOTAL_DRAW_CALLS_IN_FRAME), RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_TOTAL_PRIMITIVES_IN_FRAME)])
	await _shot(shot_dir + "/world-%s-3-5x5.png" % renderer, Vector3(0, 70, 80))
	if _is_mesh():
		await _edit_bench(shot_dir)
		await _renderer_matrix(shot_dir)
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

## Edit cost of the faces renderer under each rebuild policy, on a fresh 5x5 world each time.
## Scenarios (single-voxel toggles between a placed block and air) with the sections the "partial" policy must rebuild:
##   inside_section (1), brick_boundary (2), chunk_border (2), corner (3), corner_brick (4); plus a 3x3x3 block,
## then three edits undone and redone. Every step checks that no drawn mesh is stale and that the pictures of all
## policies are identical.
func _stale_sections() -> int:
	var n := 0
	for k in chunks:
		for bi in 2:
			var want: int = world.section_face_count(k.x, bi, k.y)
			if renderer == "greedy":
				want = world.section_greedy_stats(k.x, bi, k.y)[0]
			if drawn.get(Vector3i(k.x, bi, k.y), 0) != maxi(want, 0):
				n += 1
	return n

## Visible unit faces covered by the current meshes (greedy: sum of quad areas), drawn elements, stale sections.
func _geom_totals() -> Dictionary:
	var area := 0
	for k in chunks:
		for bi in 2:
			if renderer == "greedy":
				area += maxi(world.section_greedy_stats(k.x, bi, k.y)[1], 0)
			else:
				area += maxi(world.section_face_count(k.x, bi, k.y), 0)
	return {"area": area, "elements": _drawn_elements(), "stale": _stale_sections()}

func _mesh_bytes() -> int:
	if renderer == "cubes":
		return _drawn_elements() * 64
	var n := 0
	for k in mesh_stats:                                # vertices (position, normal, colour = 40 B) + 32-bit indices
		n += mesh_stats[k].x * 40 + mesh_stats[k].y * 4
	return n

func _node_count() -> int:
	return sec_nodes.size() if _is_mesh() else chunks.size()

## Compares the 3D area (below the HUD strip, which shows a live fps number): returns Vector3i(pixels of the 3D
## area, pixels that differ at all, pixels where the background colour shows in exactly one of the two pictures
## = holes). -1 for different sizes.
func _diff_images(x: Image, y: Image) -> Vector3i:
	var w: int = x.get_width()
	var a: PackedByteArray = x.get_data()
	var b: PackedByteArray = y.get_data()
	var stride: int = a.size() / (w * x.get_height())
	var start := 130 * w * stride
	var area := w * (x.get_height() - 130)
	if a.size() != b.size():
		return Vector3i(area, -1, -1)
	if a.slice(start) == b.slice(start):
		return Vector3i(area, 0, 0)
	var diff := 0
	var holes := 0
	for i in range(start, a.size() - 3, stride):
		if a[i] != b[i] or a[i + 1] != b[i + 1] or a[i + 2] != b[i + 2]:
			diff += 1
			var bg_a: bool = a[i] == 76 and a[i + 1] == 76 and a[i + 2] == 76    # Environment background (0.3 grey)
			var bg_b: bool = b[i] == 76 and b[i + 1] == 76 and b[i + 2] == 76
			if bg_a != bg_b:
				holes += 1
	return Vector3i(area, diff, holes)

## faces and greedy cover the same surface with different triangulations. The greedy mesh has no T-junctions and
## subdivides every edge that borders another surface like the one-face-per-voxel mesh (report section 29), so
## the pictures must be identical in the 3D area, in every view.
func _images_equivalent(d: Vector3i) -> bool:
	return d.y == 0

## The same user session in the given renderer: Fill, Edit inside a section, on a brick boundary, on a chunk border,
## 3x3x3 Edit, Undo, Redo, Jump, Save, wander off, Load. Checks no stale section after every step.
func _session(mode: String, tag: String) -> Dictionary:
	renderer = mode
	rebuild_policy = "partial"
	world.create(SEED, 2)
	_clear_chunks()
	_place_player(8.5, 8.5)
	_stream()
	var steps := []
	var tt := Time.get_ticks_usec()
	var filled: int = world.fill_box(Vector3i(12, 20, 12), Vector3i(20, 22, 20), 4)            # Fill across two chunk borders
	_apply_delta()
	steps.append(["fill", filled, last_rebuilt])
	for e in [[8, 10, 8], [8, 15, 8], [15, 10, 8], [15, 15, 15]]:         # section / brick / chunk border / corner
		world.edit_voxel(e[0], e[1], e[2], 4)
		_apply_delta()
		steps.append(["edit_%d_%d_%d" % [e[0], e[1], e[2]], 1, last_rebuilt])
	world.edit_around(-8, 14, -8)
	_apply_delta()
	steps.append(["edit_block", 27, last_rebuilt])
	world.undo()
	_apply_delta()
	steps.append(["undo", 0, last_rebuilt])
	world.undo()
	_apply_delta()
	world.redo()
	_apply_delta()
	steps.append(["redo", 0, last_rebuilt])
	var path := ProjectSettings.globalize_path("user://world_%s.wst0" % tag)
	_check("%s: save" % mode, world.save(path), world.last_error())
	var before_totals := _geom_totals()
	_place_player(8.5 + 64.0, 8.5)                                         # Jump
	_stream()
	steps.append(["jump", 0, last_rebuilt])
	_check("%s: no stale section after jump" % mode, _stale_sections() == 0)
	_check("%s: load" % mode, world.load(path), world.last_error())
	_clear_chunks()
	_apply_delta()
	_place_player(8.5, 8.5)
	_stream()
	var after_totals := _geom_totals()
	for st in steps:
		print("WORLDTEST STEP mode=%s %s changed=%d rebuilt_sections=%d" % [mode, st[0], st[1], st[2]])
	_check("%s: no stale section after load" % mode, after_totals["stale"] == 0, "%s" % [after_totals])
	_check("%s: load restores the saved geometry" % mode, after_totals["area"] == before_totals["area"], "%d vs %d" % [after_totals["area"], before_totals["area"]])
	print("WORLDTEST SESSION mode=%s total_ms=%.1f area=%d elements=%d" % [mode, (Time.get_ticks_usec() - tt) / 1000.0, after_totals["area"], after_totals["elements"]])
	return after_totals

## faces and greedy must show the same geometry and the same picture, also after switching in a running app.
func _renderer_matrix(shot_dir: String) -> void:
	var fa := _session("faces", "m_faces")
	var img_f: Image = await _capture(Vector3(0, 40, 48))
	_set_renderer("greedy")                                                  # switch in the running app
	var sw := _geom_totals()
	_check("switch faces -> greedy keeps the covered geometry", sw["area"] == fa["area"] and sw["stale"] == 0, "%s vs %s" % [sw, fa])
	_check("greedy draws fewer elements than faces", sw["elements"] * 3 < fa["elements"], "%d vs %d" % [sw["elements"], fa["elements"]])
	var img_s: Image = await _capture(Vector3(0, 40, 48))
	var ga := _session("greedy", "m_greedy")
	var img_g: Image = await _capture(Vector3(0, 40, 48))
	_check("greedy session covers the same geometry as faces", ga["area"] == fa["area"], "%d vs %d" % [ga["area"], fa["area"]])
	if img_f != null:
		var d1 := _diff_images(img_f, img_s)
		var d2 := _diff_images(img_f, img_g)
		print("WORLDTEST IMAGE faces vs greedy(after switch): of %d px %d differ, %d holes; faces vs greedy(session): %d differ, %d holes" % [d1.x, d1.y, d1.z, d2.y, d2.z])
		_check("faces and greedy pictures equivalent after switching", _images_equivalent(d1))
		_check("faces and greedy pictures equivalent after the same session", _images_equivalent(d2))
		img_f.save_png(shot_dir + "/world-session-faces.png")
		img_g.save_png(shot_dir + "/world-session-greedy.png")
	# Several camera views of the same final state: count the pixels where faces and greedy differ.
	var views := [Vector3(0, 40, 48), Vector3(30, 25, 30), Vector3(-40, 60, 10), Vector3(0, 90, 5), Vector3(-25, 12, -30)]
	_set_renderer("faces")
	var fimgs := []
	for v in views:
		fimgs.append(await _capture(v))
	_set_renderer("greedy")
	var tot_px := 0
	var tot_diff := 0
	var tot_big := 0
	for i in views.size():
		var gi: Image = await _capture(views[i])
		if gi == null or fimgs[i] == null:
			continue
		fimgs[i].save_png(shot_dir + "/world-view%d-faces.png" % i)
		gi.save_png(shot_dir + "/world-view%d-greedy.png" % i)
		var dd := _diff_images(fimgs[i], gi)
		tot_px += dd.x
		tot_diff += dd.y
		tot_big += dd.z
		print("WORLDTEST VIEW %d of %d px: %d differ, %d of them background holes" % [i, dd.x, dd.y, dd.z])
	if tot_px > 0:
		print("WORLDTEST VIEWS total of %d px: %d differ, %d background holes" % [tot_px, tot_diff, tot_big])
		_check("no background shows through the greedy mesh in any view", tot_big == 0, "%d holes" % tot_big)
		_check("faces and greedy pictures identical in all views", tot_diff == 0, "%d of %d pixels differ" % [tot_diff, tot_px])
	_set_renderer("cubes")                                                   # cycle back through all modes
	_check("cubes mode after switching has one node per chunk", chunks.size() == world.active_count() and _drawn_elements() == world.active_voxels())
	_set_renderer("faces")
	var back := _geom_totals()
	_check("switch back to faces keeps the geometry", back["area"] == fa["area"] and back["stale"] == 0)

func _capture(offset: Vector3) -> Image:
	if DisplayServer.get_name() == "headless":
		return null
	camera.look_at_from_position(player + offset, player)
	await _frames(3)
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()

func _reset_timers() -> void:
	rust_us = 0
	node_us = 0

func _edit_bench(shot_dir: String) -> void:
	var policies := ["all", "chunk", "partial"]
	var only := _rebuild_arg()
	if only in policies:
		policies = [only]               # one policy per process, so process memory can be compared
	var images := {}
	var finals := {}
	for policy in policies:
		rebuild_policy = policy
		world.create(SEED, 2)
		_clear_chunks()
		update_us.clear()
		_place_player(8.5, 8.5)
		_stream()
		await _frames(2)
		var base_img: Image = await _capture(Vector3(0, 40, 48))
		var scenarios := {
			"inside_section": [8, 10, 8],
			"brick_boundary": [8, 15, 8],
			"chunk_border": [15, 10, 8],
			"corner": [15, 10, 15],
			"corner_brick": [15, 15, 15],
		}
		# Sections rebuilt per edit and chunks touched, by policy: partial / chunk / all.
		var want_sections := {
			"partial": {"inside_section": 1, "brick_boundary": 2, "chunk_border": 2, "corner": 3, "corner_brick": 4},
			"chunk": {"inside_section": 2, "brick_boundary": 2, "chunk_border": 4, "corner": 6, "corner_brick": 6},
			"all": {"inside_section": 10, "brick_boundary": 10, "chunk_border": 10, "corner": 10, "corner_brick": 10},
		}
		for name in scenarios:
			var v: Array = scenarios[name]
			var total_us := 0
			var rebuilt := 0
			var rebuilt_chunks := 0
			var reps := 11
			_reset_timers()
			for i in reps:
				var t := Time.get_ticks_usec()
				var changed: bool = world.edit_voxel(v[0], v[1], v[2], 4 if i % 2 == 0 else 0)
				_apply_delta()
				total_us += Time.get_ticks_usec() - t
				rebuilt += last_rebuilt
				rebuilt_chunks += last_rebuilt_chunks
				if not changed:
					_check("edit %s toggle %d changed" % [name, i], false)
			var stale := _stale_sections()
			print("WORLDTEST EDIT mode=" + renderer + " policy=%s scenario=%s avg_ms=%.2f rust_ms=%.2f node_ms=%.2f sections=%.1f chunks=%.1f faces=%d stale=%d" % [policy, name, total_us / 1000.0 / reps, rust_us / 1000.0 / reps, node_us / 1000.0 / reps, float(rebuilt) / reps, float(rebuilt_chunks) / reps, _drawn_elements(), stale])
			_check("no stale mesh after %s edits (%s)" % [name, policy], stale == 0)
			_check("%s edit rebuilds %d section(s) (%s)" % [name, want_sections[policy][name], policy], rebuilt == want_sections[policy][name] * reps, "rebuilt=%d" % rebuilt)
		_reset_timers()
		var tf := Time.get_ticks_usec()
		var nf: int = world.fill_box(Vector3i(12, 20, 12), Vector3i(20, 22, 20), 4)
		_apply_delta()
		print("WORLDTEST FILL mode=%s policy=%s fill_ms=%.2f voxels=%d sections=%d chunks=%d rust_ms=%.2f node_ms=%.2f" % [renderer, policy, (Time.get_ticks_usec() - tf) / 1000.0, nf, last_rebuilt, last_rebuilt_chunks, rust_us / 1000.0, node_us / 1000.0])
		_check("fill leaves no stale mesh (%s %s)" % [renderer, policy], _stale_sections() == 0)
		world.undo()
		_apply_delta()
		_check("undo of the fill leaves no stale mesh (%s %s)" % [renderer, policy], _stale_sections() == 0)
		_reset_timers()
		var tb := Time.get_ticks_usec()
		var n: int = world.edit_around(24, world.ground_height(24, 24) + 3, 24)
		_apply_delta()
		print("WORLDTEST EDIT mode=" + renderer + " policy=%s scenario=block3x3x3 avg_ms=%.2f rust_ms=%.2f node_ms=%.2f sections=%d chunks=%d faces=%d stale=%d voxels=%d" % [policy, (Time.get_ticks_usec() - tb) / 1000.0, rust_us / 1000.0, node_us / 1000.0, last_rebuilt, last_rebuilt_chunks, _drawn_elements(), _stale_sections(), n])
		_check("3x3x3 block leaves no stale mesh (%s)" % policy, _stale_sections() == 0 and n == 27)

		# Undo / redo of three edits (interior voxel, chunk border voxel, a 3x3x3 block in chunk -1,-1).
		var faces_before := _drawn_elements()
		world.edit_voxel(6, 20, 6, 4)
		world.edit_voxel(15, 12, 9, 4)
		_apply_delta()
		world.edit_around(-8, 14, -8)
		_apply_delta()
		var faces_edited := _drawn_elements()
		var undo_us := 0
		var undo_sections := 0
		var undone := 0
		_reset_timers()
		for i in 3:
			var tu := Time.get_ticks_usec()
			undone += world.undo()
			_apply_delta()
			undo_us += Time.get_ticks_usec() - tu
			undo_sections += last_rebuilt
		var undo_rust := rust_us
		var undo_node := node_us
		var stale_u := _stale_sections()
		_check("undo x3 leaves no stale mesh (%s)" % policy, stale_u == 0, "stale=%d" % stale_u)
		_check("undo x3 restores the earlier face count (%s)" % policy, _drawn_elements() == faces_before, "%d vs %d" % [_drawn_elements(), faces_before])
		var faces_undone := _drawn_elements()
		var redo_us := 0
		var redo_sections := 0
		var redone := 0
		_reset_timers()
		for i in 3:
			var tr := Time.get_ticks_usec()
			redone += world.redo()
			_apply_delta()
			redo_us += Time.get_ticks_usec() - tr
			redo_sections += last_rebuilt
		var stale_r := _stale_sections()
		_check("redo x3 leaves no stale mesh (%s)" % policy, stale_r == 0)
		_check("redo x3 restores the edited face count (%s)" % policy, _drawn_elements() == faces_edited, "%d vs %d" % [_drawn_elements(), faces_edited])
		_check("undo then redo move the same number of voxels (%s)" % policy, undone == redone and undone > 0, "%d vs %d" % [undone, redone])
		print("WORLDTEST UNDO mode=" + renderer + " policy=%s undo_avg_ms=%.2f redo_avg_ms=%.2f undo_sections=%d redo_sections=%d rust_ms=%.2f node_ms=%.2f faces_before=%d edited=%d undone=%d" % [policy, undo_us / 3000.0, redo_us / 3000.0, undo_sections, redo_sections, undo_rust / 1000.0, undo_node / 1000.0, faces_before, faces_edited, faces_undone])
		finals[policy] = _drawn_elements()
		print("WORLDTEST MEM edit policy=%s static_mb=%.1f" % [policy, OS.get_static_memory_usage() / 1048576.0])
		var img: Image = await _capture(Vector3(0, 40, 48))
		if img != null:
			images[policy] = img
			if base_img != null:
				_check("edits are visible in the picture (%s)" % policy, base_img.get_data() != img.get_data())
			img.save_png(shot_dir + "/world-%s-edit-%s.png" % [renderer, policy])
	if finals.size() > 1:
		var ref: int = finals["partial"]
		for p in finals:
			_check("%s ends with the same number of faces as partial" % p, finals[p] == ref, "%d vs %d" % [finals[p], ref])
	if images.size() > 1:
		# The HUD strip (top 130 px) shows a live fps number, so only the 3D area below it is compared.
		var ref_img: Image = images["partial"]
		for p in images:
			var w: int = images[p].get_width()
			var a: PackedByteArray = images[p].get_data()
			var b: PackedByteArray = ref_img.get_data()
			var stride: int = a.size() / (w * images[p].get_height())
			var start := 130 * w * stride
			var diff := 0
			if a.slice(start) != b.slice(start):
				for i in range(start, mini(a.size(), b.size()) - 3, stride):
					if a[i] != b[i] or a[i + 1] != b[i + 1] or a[i + 2] != b[i + 2]:
						diff += 1
			print("WORLDTEST IMAGE policy=%s vs partial differing_pixels=%d of %d (3D area)" % [p, diff, w * (images[p].get_height() - 130)])
			_check("picture after edits and undo/redo identical to partial (%s)" % p, diff == 0)
