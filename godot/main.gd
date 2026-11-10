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
