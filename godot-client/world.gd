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
const DEFAULT_RADIUS := 1

# P-WORLD stage C: voxel picking (mouse cursor ray), editing, save file.
const AIM_RANGE := 80.0                # how far the camera ray looks (voxels)
const REACH := 10.0                    # how far from the character a voxel can be removed or placed (voxels)
var aim_valid := false                 # the ray hit a voxel
var aim_in_reach := false              # ... within REACH of the character
var aim_voxel := Vector3i.ZERO
var aim_normal := Vector3i.ZERO        # face that was hit; the new block goes at aim_voxel + aim_normal
var aim_override: Variant = null       # a Vector2 screen position instead of the mouse (tests); null = the mouse
var highlight := MeshInstance3D.new()
var highlight_mat := StandardMaterial3D.new()
var rmb_travel := 0.0                  # mouse travel while the right button is down: a click places, a drag orbits
var edits_unsaved := 0
var persist_enabled := false           # autosave on close / load on start: only the real client turns this on
var save_path := ""
var edit_log: Array = []               # (x, y, z, material after) of every edit by the player, for tests
var walk_mode := true                  # stage B: a physical character (gravity, collision); false = old flying marker
var jump_pressed := false
# P-WORLD stage A: `--seed=<n>` and `--radius=<0..8>` (Godot user arguments after `--`), orbit camera, frame times.
var seed_value := SEED
var radius_value := DEFAULT_RADIUS
var cam_yaw := 0.0                     # radians around the player, 0 = camera behind (+z) the player
var cam_pitch := atan2(16.0, 20.0)     # radians above the horizon (default view = offset (0, 16, 20))
var cam_dist := sqrt(16.0 * 16.0 + 20.0 * 20.0)
var orbiting := false
var frame_acc := 0.0                   # seconds accumulated in the current one-second window
var frame_count := 0
var frame_max := 0.0
var frame_avg_ms := 0.0                # published every second: average and worst frame time of the last window
var frame_max_ms := 0.0

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
	persist_enabled = true
	if _init_world():
		if persist_enabled and FileAccess.file_exists(save_path) and not _has_flag("--fresh") and _load_world():
			return
		_stream()
		if walk_mode:
			_spawn_character()

## `--name` among the Godot user arguments.
func _has_flag(flag: String) -> bool:
	return OS.get_cmdline_user_args().has(flag)

func _str_arg(prefix: String) -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with(prefix):
			return a.substr(prefix.length())
	return ""

## Loads the save file (at start: replaces the fresh world). A damaged file is kept as `<file>.bad` and the fresh
## world is used. The character is not saved: Rust puts it above the middle of the saved centre chunk and it falls.
func _load_world() -> bool:
	var ok: bool = world.load(save_path)
	if not ok:
		var err: String = world.last_error()
		print("WORLD load failed: %s" % err)
		var bad := save_path + ".bad"
		DirAccess.rename_absolute(save_path, bad)
		hud.text = "save file unreadable (%s), kept as %s" % [err, bad.get_file()]
		return false
	_clear_chunks()
	seed_value = world.seed()
	radius_value = world.radius()
	player = world.player_pos()
	marker.position = player + Vector3(0, 0.9, 0)
	_update_camera()
	_apply_delta()
	edits_unsaved = 0
	print("WORLD loaded %s seed=%d radius=%d chunks=%d hash=%s" % [save_path, seed_value, radius_value, world.active_count(), world.world_hash()])
	return true

func _save_world() -> bool:
	var ok: bool = world.save(save_path)
	if ok:
		edits_unsaved = 0
	print("WORLD save ok=%s path=%s hash=%s" % [ok, save_path, world.world_hash()])
	return ok

func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST and persist_enabled and world != null:
		_save_world()

## P-WORLD stage B: the character stands on the surface at (8, 8) once its chunk is loaded.
func _spawn_character() -> void:
	if world.player_spawn(8, 8):
		player = world.player_pos()
		marker.position = player + Vector3(0, 0.9, 0)
		_update_camera()

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
	var s: Variant = _int_arg("--seed=")
	if s != null:
		seed_value = s
	var r: Variant = _int_arg("--radius=")
	if r != null:
		radius_value = clampi(r, 0, 8)
	_build_scene()
	var sp := _str_arg("--save=")
	save_path = sp if sp != "" else ProjectSettings.globalize_path("user://world.wst0")
	world.create(seed_value, radius_value)
	_place_player(player.x, player.z)
	print("WORLD seed=%d radius=%d renderer=%s" % [seed_value, radius_value, renderer])
	return true

## `--name=<int>` among the Godot user arguments; null if absent or not an integer.
func _int_arg(prefix: String):
	for a in OS.get_cmdline_user_args():
		if a.begins_with(prefix):
			var v := a.substr(prefix.length())
			if v.is_valid_int():
				return v.to_int()
	return null

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
	m.size = Vector3(0.6, 1.8, 0.6)
	var yellow := StandardMaterial3D.new()
	yellow.albedo_color = Color(1, 0.9, 0.1)
	m.material = yellow
	marker.mesh = m
	add_child(marker)
	var hb := BoxMesh.new()
	hb.size = Vector3(1.03, 1.03, 1.03)
	highlight_mat.albedo_color = Color(1, 0.95, 0.2, 0.35)
	highlight_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	highlight_mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	hb.material = highlight_mat
	highlight.mesh = hb
	highlight.visible = false
	add_child(highlight)

	var ui := CanvasLayer.new()
	add_child(ui)
	hud.position = Vector2(8, 4)
	ui.add_child(hud)
	var row := HBoxContainer.new()
	row.position = Vector2(0, 118)
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
	marker.position = player + Vector3(0, 0.9, 0)
	_update_camera()

## Orbit camera: yaw and pitch around the player at distance `cam_dist` (defaults equal the old fixed offset).
func _update_camera() -> void:
	var off := Vector3(sin(cam_yaw) * cos(cam_pitch), sin(cam_pitch), cos(cam_yaw) * cos(cam_pitch)) * cam_dist
	var target := player + Vector3(0, 1.0, 0)
	camera.look_at_from_position(target + off, target)

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_E:
		_do_place()
	elif event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
			_do_remove()
		elif event.button_index == MOUSE_BUTTON_MIDDLE and event.pressed:
			_do_place()
		elif event.button_index == MOUSE_BUTTON_RIGHT:
			orbiting = event.pressed
			if event.pressed:
				rmb_travel = 0.0
			elif rmb_travel < 6.0:
				_do_place()                       # a right click without a drag places a block
		elif event.pressed and event.button_index == MOUSE_BUTTON_WHEEL_UP:
			cam_dist = maxf(6.0, cam_dist * 0.9)
			_update_camera()
		elif event.pressed and event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			cam_dist = minf(140.0, cam_dist * 1.1)
			_update_camera()
	elif event is InputEventMouseMotion and orbiting:
		rmb_travel += event.relative.length()
		cam_yaw -= event.relative.x * 0.005
		cam_pitch = clampf(cam_pitch + event.relative.y * 0.005, 0.1, 1.5)
		_update_camera()

func _track_frame(delta: float) -> void:
	frame_acc += delta
	frame_count += 1
	frame_max = maxf(frame_max, delta)
	if frame_acc >= 1.0:
		frame_avg_ms = frame_acc / frame_count * 1000.0
		frame_max_ms = frame_max * 1000.0
		frame_acc = 0.0
		frame_count = 0
		frame_max = 0.0

func _process(delta: float) -> void:
	if world == null or not auto_stream:
		return
	_track_frame(delta)
	if walk_mode:
		_walk(delta)
		return
	var d := Vector2.ZERO
	if Input.is_key_pressed(KEY_W) or Input.is_key_pressed(KEY_UP) or hold["fwd"]: d.y -= 1
	if Input.is_key_pressed(KEY_S) or Input.is_key_pressed(KEY_DOWN) or hold["back"]: d.y += 1
	if Input.is_key_pressed(KEY_A) or Input.is_key_pressed(KEY_LEFT) or hold["left"]: d.x -= 1
	if Input.is_key_pressed(KEY_D) or Input.is_key_pressed(KEY_RIGHT) or hold["right"]: d.x += 1
	if d != Vector2.ZERO:
		d = d.normalized().rotated(-cam_yaw) * SPEED * delta      # movement is relative to the view direction
		_place_player(player.x + d.x, player.z + d.y)
	_stream(PUMP_PER_FRAME)

## Picks the voxel under the mouse cursor (or `aim_override`): ray from the camera through the cursor, in Rust.
func _update_aim() -> void:
	var sp: Vector2 = aim_override if aim_override != null else get_viewport().get_mouse_position()
	var origin := camera.project_ray_origin(sp)
	var dir := camera.project_ray_normal(sp)
	var r: PackedInt32Array = world.raycast(origin, dir, AIM_RANGE)
	aim_valid = r[0] == 1
	aim_in_reach = false
	if aim_valid:
		aim_voxel = Vector3i(r[1], r[2], r[3])
		aim_normal = Vector3i(r[4], r[5], r[6])
		var centre := Vector3(aim_voxel) + Vector3(0.5, 0.5, 0.5)
		aim_in_reach = centre.distance_to(player + Vector3(0, 0.9, 0)) <= REACH
		highlight.position = centre
		highlight_mat.albedo_color = Color(1, 0.95, 0.2, 0.35) if aim_in_reach else Color(1, 0.2, 0.2, 0.30)
	highlight.visible = aim_valid

## Removes the aimed voxel; the mesh and the collision change at once (Rust reads the stored voxels).
func _do_remove() -> bool:
	_update_aim()
	if not (aim_valid and aim_in_reach):
		return false
	var v := aim_voxel
	if not world.remove_block(v.x, v.y, v.z):
		return false
	edit_log.append([v.x, v.y, v.z, 0])
	edits_unsaved += 1
	_apply_delta()
	_update_aim()
	return true

## Places a block on the aimed face (next to the aimed voxel); refused inside the character or in unloaded chunks.
func _do_place() -> bool:
	_update_aim()
	if not (aim_valid and aim_in_reach) or aim_normal == Vector3i.ZERO:
		return false
	var t := aim_voxel + aim_normal
	if not world.place_block(t.x, t.y, t.z):
		return false
	edit_log.append([t.x, t.y, t.z, 4])
	edits_unsaved += 1
	_apply_delta()
	_update_aim()
	return true

## Stage B frame: input -> character step in Rust -> camera -> streaming around the character.
func _walk(delta: float) -> void:
	var d := Vector2.ZERO
	if Input.is_key_pressed(KEY_W) or Input.is_key_pressed(KEY_UP) or hold["fwd"]: d.y -= 1
	if Input.is_key_pressed(KEY_S) or Input.is_key_pressed(KEY_DOWN) or hold["back"]: d.y += 1
	if Input.is_key_pressed(KEY_A) or Input.is_key_pressed(KEY_LEFT) or hold["left"]: d.x -= 1
	if Input.is_key_pressed(KEY_D) or Input.is_key_pressed(KEY_RIGHT) or hold["right"]: d.x += 1
	d = d.normalized().rotated(-cam_yaw)            # movement is relative to the view direction
	jump_pressed = Input.is_key_pressed(KEY_SPACE)
	world.player_step(delta, d.x, d.y, jump_pressed)
	player = world.player_pos()
	marker.position = player + Vector3(0, 0.9, 0)
	_update_camera()
	_update_aim()
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
	hud.text = "seed %d   %s   active chunks %d (nodes %d)   voxels %d   drawn %d   queue %d   player (%.0f, %.0f, %.0f)
frame avg %.1f ms max %.1f ms   radius %d   %s\n%s   unsaved edits %d" % [world.seed(), renderer, world.active_count(), chunks.size(), world.active_voxels(), _drawn_elements(), world.queue_len(), player.x, player.y, player.z, frame_avg_ms, frame_max_ms, world.radius(), _body_state(), _aim_state(), edits_unsaved]

func _aim_state() -> String:
	if not aim_valid:
		return "no target"
	return "target (%d, %d, %d) face (%d, %d, %d)%s" % [aim_voxel.x, aim_voxel.y, aim_voxel.z, aim_normal.x, aim_normal.y, aim_normal.z, "" if aim_in_reach else " (out of reach)"]

func _body_state() -> String:
	if not walk_mode:
		return "fly"
	return "waiting for chunk" if world.player_held() else ("on ground" if world.player_on_ground() else "air")

func _on_edit() -> void:
	var n: int = world.edit_around(floori(player.x), floori(player.y) + 2, floori(player.z))
	_apply_delta()
	_update_hud()
	print("edit changed %d voxels" % n)

func _on_save() -> void:
	var ok := _save_world()
	hud.text = "saved" if ok else world.last_error()

func _on_load() -> void:
	var ok: bool = world.load(save_path)
	print("load ok=%s" % ok)
	if not ok:
		hud.text = world.last_error()
		return
	_clear_chunks()
	if walk_mode:
		player = world.player_pos()          # Rust put the character above the saved centre chunk
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
	if walk_mode:
		world.player_teleport(player.x + 64.0, player.z)      # waits in the air until the chunks there are loaded
		player = world.player_pos()
		marker.position = player + Vector3(0, 0.9, 0)
		_update_camera()
	else:
		_place_player(player.x + 64.0, player.z)
	print("jump to (%.0f, %.0f)" % [player.x, player.z])
