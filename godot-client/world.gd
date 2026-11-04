extends Node3D
## World-generation spike client (runtime). Terrain lives in Rust (CubeWorld); this script streams chunk nodes
## around the player, draws each chunk as meshes or cubes and shows a HUD. It starts no child process, reads no
## system properties and contains no test code: the scripted validation is a separate entry point
## (`world_test.tscn` / `world_smoke.gd`, excluded from the client export presets).
##   godot --path godot-client world.tscn                      interactive (WASD / arrows, on-screen buttons)
##   godot --path godot-client world.tscn -- --renderer=greedy   choose the renderer on the command line
## Renderer, in this order: `--renderer=<mode>` (Godot user argument), custom feature `cubes` / `faces` / `greedy`
## (export presets), default `faces`. Modes: "cubes" (control: one MultiMesh cube per voxel), "faces" (reference:
## one mesh of the visible faces per section), "greedy" (experimental: the visible faces of a section merged into
## same-material rectangles without T-junctions). The Renderer button cycles cubes -> faces -> greedy at run time.
## A chunk (16x32x16) has two mesh sections, one per 16^3 brick. After an edit the mesh renderers rebuild, by
## `--rebuild=<policy>`:
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
var auto_stream := true               # false while a test drives the world itself
var update_us := []                    # microseconds spent building chunk MultiMeshes / meshes
var renderer := "faces"
var drawn := {}                        # Vector2i -> drawn elements of the chunk (cubes or visible faces)
var face_mat := StandardMaterial3D.new()
var mode_button := Button.new()
var buttons := {}                      # name -> Button (positions are read by the test entry point)
var rebuild_policy := "partial"        # faces renderer after an edit: "partial", "chunk" or "all" (see header)
var last_rebuilt := 0                  # mesh sections (faces) or chunk nodes (cubes) rebuilt by the last _apply_delta
var last_rebuilt_chunks := 0           # distinct chunks among them
var mesh_stats := {}                   # Vector3i section -> Vector3i(vertices, indices, drawn elements)
var sec_nodes := {}                    # Vector3i(chunk x, brick, chunk z) -> MeshInstance3D (faces renderer)
var rust_us := 0                       # microseconds inside world.section_mesh (Rust meshing + ArrayMesh creation)
var node_us := 0                       # microseconds spent replacing Godot nodes

func _ready() -> void:
	if _init_world():
		_stream()

## Creates the Rust world and the scene; false (with an error) if the GDExtension is not loaded.
func _init_world() -> bool:
	if not ClassDB.class_exists("CubeWorld"):
		push_error("CubeWorld class missing: the GDExtension is not loaded (cargo build -p cube-godot-adapter).")
		return false
	world = ClassDB.instantiate("CubeWorld")
	var rmode := _renderer_arg()
	if rmode in ["cubes", "faces", "greedy"]:
		renderer = rmode
	elif OS.has_feature("greedy"):
		renderer = "greedy"
	elif OS.has_feature("cubes"):
		renderer = "cubes"
	elif OS.has_feature("faces"):
		renderer = "faces"
	var arg := _rebuild_arg()
	if arg in ["partial", "chunk", "all"]:
		rebuild_policy = arg
	_build_scene()
	world.create(SEED, 1)
	_place_player(player.x, player.z)
	return true

## `--renderer=<m>` among the Godot user arguments (after `--`); empty if absent.
func _renderer_arg() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--renderer="):
			return a.substr(11)
	return ""

func _is_mesh() -> bool:
	return renderer != "cubes"

## `--rebuild=<policy>` among the Godot user arguments; empty if absent.
func _rebuild_arg() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--rebuild="):
			return a.substr(10)
	return ""

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
		buttons[spec[0]] = b
		b.button_down.connect(func(): hold[spec[1]] = true)
		b.button_up.connect(func(): hold[spec[1]] = false)
		row.add_child(b)
	mode_button.pressed.connect(_on_mode)
	buttons["Renderer"] = mode_button
	row.add_child(mode_button)
	_update_mode_button()
	for spec in [["Edit", _on_edit], ["Undo", _on_undo], ["Redo", _on_redo], ["Save", _on_save], ["Load", _on_load], ["Jump 64", _on_jump]]:
		var b := Button.new()
		b.text = spec[0]
		b.pressed.connect(spec[1])
		buttons[spec[0]] = b
		row.add_child(b)

func _place_player(x: float, z: float) -> void:
	player.x = x
	player.z = z
	player.y = world.ground_height(floori(x), floori(z)) + 1.0
	marker.position = player + Vector3(0, 0.2, 0)
	camera.look_at_from_position(player + Vector3(0, 16, 20), player)

func _process(delta: float) -> void:
	if world == null or not auto_stream:
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
	print("save ok=%s" % ok)

func _on_load() -> void:
	var ok: bool = world.load(ProjectSettings.globalize_path("user://world.wst0"))
	print("load ok=%s" % ok)
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
	print("jump to (%.0f, %.0f)" % [player.x, player.z])
