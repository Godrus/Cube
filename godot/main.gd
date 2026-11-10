extends Node3D
## UI and glue only (ADP-02): reads input, calls the Rust CubeWorld node, copies the camera pose.
## No geometry, no world rules and no file writes are done here (except test pictures and test
## labels when a command line option asks for them).
##
## Controls: click to capture the mouse, WASD move, Space / Shift up and down, mouse looks,
## left click removes the block under the crosshair, right click places a block, Esc releases the mouse.
##
## Command line after `--`:
##   --smoke                          headless self check
##   --screenshot <png>               save one picture and quit
##   --edit-shots <before> <after>    picture, remove the block under the crosshair, picture
##   --world <dir> --seed <n>         world directory and seed
##   --yaw <rad> --pitch <rad>        extra camera turn at the start
##   --down <frames>                  descend for that many frames first (edit pictures from close up)
##   --truth <json>                   write the ground-truth label of this picture (written at
##                                    capture time, before any model sees the picture)
##   --fault <kind>                   QA only: break the SCENE (never the renderer or the world) in a
##                                    controlled way: missing_brick, hidden_area, wrong_material,
##                                    partial_load (the world stops delivering batches after one
##                                    frame), stale (with --edit-shots: no update after the edit)
##   --scenario <json>                QA only (Visual QA v3): run a scripted list of steps (wait, move,
##                                    turn, shot, edit, probe, fault, freeze, occlude) with a fixed
##                                    time step, save pictures and write telemetry.json next to them.
##                                    Never active in a normal game start.

@onready var world = $World
@onready var cam: Camera3D = $Camera3D

var look := Vector2.ZERO
var init_look := Vector2.ZERO
var smoke := false
var shot_path := ""
var before_path := ""
var after_path := ""
var truth_path := ""
var fault := ""
var seed_value := 7
var frames := 0
var edited := false
var base_bricks := 0
var freeze_world := false
var fault_bbox = null
var edit_hit := false
var down_frames := 0
var scen_active := false
var scen_steps := []
var scen_out := ""
var scen_i := 0
var scen_cmd = null
var scen_events := []
var scen_changed := 0
var scen_edit_event = null


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var dir := ProjectSettings.globalize_path("user://world")
	var i := 0
	while i < args.size():
		match args[i]:
			"--smoke":
				smoke = true
			"--screenshot":
				i += 1
				shot_path = args[i]
			"--edit-shots":
				before_path = args[i + 1]
				after_path = args[i + 2]
				i += 2
			"--world":
				i += 1
				dir = args[i]
			"--seed":
				i += 1
				seed_value = int(args[i])
			"--yaw":
				i += 1
				init_look.x = float(args[i])
			"--pitch":
				i += 1
				init_look.y = float(args[i])
			"--down":
				i += 1
				down_frames = int(args[i])
			"--truth":
				i += 1
				truth_path = args[i]
			"--fault":
				i += 1
				fault = args[i]
			"--scenario":
				i += 1
				_load_scenario(args[i])
		i += 1
	var create := not FileAccess.file_exists(dir.path_join("manifest"))
	if not world.open_world(dir, seed_value, create):
		push_error("could not open the world")
		get_tree().quit(2)
		return
	print("CUBE world opened: dir=", dir, " create=", create)


func _input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed:
		if Input.mouse_mode != Input.MOUSE_MODE_CAPTURED:
			Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
		elif event.button_index == MOUSE_BUTTON_LEFT:
			world.edit(0)
		elif event.button_index == MOUSE_BUTTON_RIGHT:
			world.edit(5)
	elif event is InputEventMouseMotion and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		look += Vector2(-event.relative.x, -event.relative.y) * 0.003
	elif event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		Input.mouse_mode = Input.MOUSE_MODE_VISIBLE


func _process(delta: float) -> void:
	if scen_active:
		_scenario_frame()
		return
	frames += 1
	if frames == 1:
		look += init_look
	if not freeze_world:
		var fwd := int(Input.is_key_pressed(KEY_W)) - int(Input.is_key_pressed(KEY_S))
		var right := int(Input.is_key_pressed(KEY_D)) - int(Input.is_key_pressed(KEY_A))
		var up := int(Input.is_key_pressed(KEY_SPACE)) - int(Input.is_key_pressed(KEY_SHIFT))
		if frames <= down_frames:
			up = -1
		world.frame(fwd, right, up, look.x, look.y, delta)
		look = Vector2.ZERO
	cam.position = world.camera_position()
	cam.rotation = world.camera_rotation()
	if fault == "partial_load" and frames == 1:
		freeze_world = true  # the world delivered one batch; the rest never reaches the scene
	if smoke:
		_smoke_step()
	elif before_path != "":
		_edit_shots_step()
	elif shot_path != "":
		_screenshot_step()


func _smoke_step() -> void:
	if frames == 30:
		base_bricks = world.brick_count()
		print("CUBE smoke: bricks after 30 frames = ", base_bricks)
		if base_bricks <= 0:
			push_error("no geometry reached the scene")
			get_tree().quit(1)
	elif frames == 31:
		edited = world.edit(0)
		print("CUBE smoke: edit hit = ", edited)
	elif frames == 60:
		print("CUBE smoke: bricks after the edit = ", world.brick_count(), " edit applied = ", edited)
		world.close()
		get_tree().quit(0 if edited else 1)


func _screenshot_step() -> void:
	var shoot := 45 if fault != "partial_load" else 10
	if frames == 40 and fault != "" and fault != "partial_load":
		_apply_scene_fault()
	elif frames == shoot:
		get_viewport().get_texture().get_image().save_png(shot_path)
		_write_truth("scene")
		print("CUBE screenshot saved: ", shot_path, " bricks=", world.brick_count())
		world.close()
		get_tree().quit(0)


func _edit_shots_step() -> void:
	var o := down_frames
	if frames == 40 + o:
		get_viewport().get_texture().get_image().save_png(before_path)
	elif frames == 41 + o:
		edit_hit = world.edit(0)
		print("CUBE edit hit = ", edit_hit)
		if fault == "stale":
			freeze_world = true  # the world changed but the scene never receives the update
	elif frames == 70 + o:
		get_viewport().get_texture().get_image().save_png(after_path)
		_write_truth("edit")
		print("CUBE edit shots saved: ", before_path, " ", after_path)
		world.close()
		get_tree().quit(0)


## Brick nodes of the scene, as (node, screen-projected centre in 0..1 or null).
func _bricks() -> Array:
	var out := []
	var size := get_viewport().get_visible_rect().size
	for n in world.get_children():
		if n is MeshInstance3D:
			var centre: Vector3 = n.position + Vector3(8, 8, 8)
			if cam.is_position_behind(centre):
				continue
			var d := cam.global_position.distance_to(centre)
			if d < 8.0 or d > 70.0:
				continue
			var p := cam.unproject_position(centre)
			out.append({"node": n, "p": Vector2(p.x / size.x, p.y / size.y), "d": d})
	return out


## Normalised screen rectangle [left, top, right, bottom] of the 16-voxel cube at `origin`.
func _bbox_of(origin: Vector3):
	var size := get_viewport().get_visible_rect().size
	var lo := Vector2(1e9, 1e9)
	var hi := Vector2(-1e9, -1e9)
	for dx in [0, 16]:
		for dy in [0, 16]:
			for dz in [0, 16]:
				var c := origin + Vector3(dx, dy, dz)
				if cam.is_position_behind(c):
					return null
				var p := cam.unproject_position(c)
				lo = Vector2(min(lo.x, p.x), min(lo.y, p.y))
				hi = Vector2(max(hi.x, p.x), max(hi.y, p.y))
	var box := [clamp(lo.x / size.x, 0.0, 1.0), clamp(lo.y / size.y, 0.0, 1.0),
		clamp(hi.x / size.x, 0.0, 1.0), clamp(hi.y / size.y, 0.0, 1.0)]
	return box


func _nearest_to(target: Vector2, count: int) -> Array:
	var all := _bricks()
	all.sort_custom(func(a, b): return a.p.distance_to(target) < b.p.distance_to(target))
	return all.slice(0, count)


func _apply_scene_fault() -> void:
	var picks := _nearest_to(Vector2(0.5, 0.62), 1 if fault != "hidden_area" else 4)
	if picks.is_empty():
		push_error("no brick to break")
		return
	var union = null
	for pick in picks:
		var node: MeshInstance3D = pick.node
		var box = _bbox_of(node.position)
		if box != null:
			if union == null:
				union = box.duplicate()
			else:
				union = [min(union[0], box[0]), min(union[1], box[1]), max(union[2], box[2]), max(union[3], box[3])]
		match fault:
			"missing_brick":
				world.remove_child(node)  # the brick is simply not in the scene; the adapter still owns it
			"hidden_area":
				node.visible = false
			"wrong_material":
				var m := StandardMaterial3D.new()
				m.albedo_color = Color(0.2, 0.3, 0.9)
				node.material_override = m
	fault_bbox = union
	print("CUBE fault applied: ", fault, " bbox=", fault_bbox)


func _write_truth(kind: String) -> void:
	if truth_path == "":
		return
	var size := get_viewport().get_visible_rect().size
	var t := {
		"kind": kind,
		"seed": seed_value,
		"fault": fault if fault != "" else "none",
		"label": "good" if fault == "" and not (kind == "edit") else "defective",
		"viewport": [size.x, size.y],
		"bricks_in_scene": world.brick_count(),
		"camera_position": [cam.position.x, cam.position.y, cam.position.z],
		"camera_rotation": [cam.rotation.x, cam.rotation.y, cam.rotation.z],
		"expected_bbox": fault_bbox,
		"frames": frames,
	}
	if kind == "edit":
		# The edit targets the block under the crosshair: the centre of the picture.
		t["edit_hit"] = edit_hit
		t["edit_region"] = [0.45, 0.4, 0.55, 0.6]
		t["expected_changed"] = fault != "stale" and edit_hit
		t["label"] = "changed" if fault != "stale" and edit_hit else "unchanged"
	var f := FileAccess.open(truth_path, FileAccess.WRITE)
	f.store_string(JSON.stringify(t, "  "))
	f.close()


## ---- Visual QA v3 scenario runner (QA only; inert unless --scenario is given) ----

func _load_scenario(path: String) -> void:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		push_error("cannot read scenario " + path)
		get_tree().quit(2)
		return
	var data = JSON.parse_string(f.get_as_text())
	f.close()
	scen_steps = data["steps"]
	scen_out = str(data["out_dir"])
	scen_active = true


func _event(op: String, extra: Dictionary = {}) -> Dictionary:
	var e := {"op": op, "frame": frames, "bricks": world.brick_count(),
		"camera_position": [cam.position.x, cam.position.y, cam.position.z],
		"camera_rotation": [cam.rotation.x, cam.rotation.y, cam.rotation.z]}
	for k in extra:
		e[k] = extra[k]
	scen_events.append(e)
	return e


func _scenario_frame() -> void:
	frames += 1
	if scen_cmd == null:
		if scen_i >= scen_steps.size():
			_scenario_finish(0)
			return
		scen_cmd = scen_steps[scen_i].duplicate()
		scen_i += 1
		scen_cmd["_left"] = int(scen_cmd.get("frames", 1))
		scen_cmd["_total"] = scen_cmd["_left"]
	var op: String = scen_cmd["op"]
	var fwd := 0
	var right := 0
	var up := 0
	var dyaw := 0.0
	var dpitch := 0.0
	if op == "move":
		fwd = int(scen_cmd.get("fwd", 0))
		right = int(scen_cmd.get("right", 0))
		up = int(scen_cmd.get("up", 0))
	elif op == "turn":
		dyaw = float(scen_cmd.get("yaw", 0.0)) / float(scen_cmd["_total"])
		dpitch = float(scen_cmd.get("pitch", 0.0)) / float(scen_cmd["_total"])
	if not freeze_world:
		var changed: int = world.frame(fwd, right, up, dyaw, dpitch, 1.0 / 60.0)
		if changed > 0:
			scen_changed += changed
			if scen_edit_event != null:
				scen_edit_event["bricks_changed"] = int(scen_edit_event.get("bricks_changed", 0)) + changed
	cam.position = world.camera_position()
	cam.rotation = world.camera_rotation()
	scen_cmd["_left"] -= 1
	if scen_cmd["_left"] > 0:
		return
	_scenario_action(op)
	scen_cmd = null


func _scenario_action(op: String) -> void:
	match op:
		"shot":
			var path: String = scen_out.path_join(str(scen_cmd["name"]) + ".png")
			get_viewport().get_texture().get_image().save_png(path)
			scen_edit_event = null
			_event("shot", {"name": scen_cmd["name"], "file": path})
		"probe":
			_event("probe", {"name": scen_cmd.get("name", "")})
		"edit":
			var hit: bool = world.edit(int(scen_cmd.get("material", 0)))
			scen_edit_event = _event("edit", {"hit": hit, "material": int(scen_cmd.get("material", 0)), "bricks_changed": 0})
		"fault":
			fault = str(scen_cmd["kind"])
			_apply_scene_fault()
			_event("fault", {"kind": fault, "bbox": fault_bbox})
		"freeze":
			freeze_world = true
			_event("freeze")
		"occlude":
			_occlude(scen_cmd)
			_event("occlude", {"kind": scen_cmd.get("kind", "panel")})


func _occlude(c: Dictionary) -> void:
	var col: Array = c.get("color", [0.1, 0.1, 0.1, 1.0])
	var color := Color(col[0], col[1], col[2], col[3])
	if str(c.get("kind", "panel")) == "object":
		# an opaque box right in front of the camera (a stray 3D object)
		var m := MeshInstance3D.new()
		var b := BoxMesh.new()
		b.size = Vector3(float(c.get("w", 1.2)), float(c.get("h", 0.8)), 0.2)
		m.mesh = b
		var mat := StandardMaterial3D.new()
		mat.albedo_color = color
		m.material_override = mat
		cam.add_child(m)
		m.position = Vector3(0, 0, -float(c.get("distance", 1.5)))
	else:
		var layer := CanvasLayer.new()
		var r := ColorRect.new()
		var size := get_viewport().get_visible_rect().size
		var rect: Array = c["rect"]
		r.position = Vector2(rect[0] * size.x, rect[1] * size.y)
		r.size = Vector2((rect[2] - rect[0]) * size.x, (rect[3] - rect[1]) * size.y)
		r.color = color
		layer.add_child(r)
		add_child(layer)


func _scenario_finish(code: int) -> void:
	var size := get_viewport().get_visible_rect().size
	var t := {"viewport": [size.x, size.y], "frames": frames, "bricks_final": world.brick_count(),
		"changed_total": scen_changed, "events": scen_events}
	var f := FileAccess.open(scen_out.path_join("telemetry.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify(t, "  "))
	f.close()
	print("CUBE scenario done: events=", scen_events.size(), " bricks=", world.brick_count())
	world.close()
	get_tree().quit(code)
