extends Node3D
## UI and glue only (ADP-02): reads input, calls the Rust CubeWorld node, copies the camera pose.
## No geometry, no world rules and no file writes are done here.
##
## Controls: click to capture the mouse, WASD move, Space / Shift up and down, mouse looks,
## left click removes the block under the crosshair, right click places a block, Esc releases the mouse.
## Command line after `--`: --smoke (headless self check), --screenshot <path> (save a picture and quit),
## --edit-shots <before.png> <after.png> (picture, remove the block under the crosshair, picture),
## --world <dir> (world directory), --seed <n>.

@onready var world = $World
@onready var cam: Camera3D = $Camera3D

var look := Vector2.ZERO
var smoke := false
var shot_path := ""
var before_path := ""
var after_path := ""
var frames := 0
var edited := false
var base_bricks := 0


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var dir := ProjectSettings.globalize_path("user://world")
	var seed_value := 7
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
	var fwd := int(Input.is_key_pressed(KEY_W)) - int(Input.is_key_pressed(KEY_S))
	var right := int(Input.is_key_pressed(KEY_D)) - int(Input.is_key_pressed(KEY_A))
	var up := int(Input.is_key_pressed(KEY_SPACE)) - int(Input.is_key_pressed(KEY_SHIFT))
	var changed: int = world.frame(fwd, right, up, look.x, look.y, delta)
	look = Vector2.ZERO
	cam.position = world.camera_position()
	cam.rotation = world.camera_rotation()
	frames += 1
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
	if frames == 40:
		var img := get_viewport().get_texture().get_image()
		img.save_png(shot_path)
		print("CUBE screenshot saved: ", shot_path, " bricks=", world.brick_count())
		world.close()
		get_tree().quit(0)


func _edit_shots_step() -> void:
	if frames == 40:
		get_viewport().get_texture().get_image().save_png(before_path)
	elif frames == 41:
		print("CUBE edit hit = ", world.edit(0))
	elif frames == 70:
		get_viewport().get_texture().get_image().save_png(after_path)
		print("CUBE edit shots saved: ", before_path, " ", after_path)
		world.close()
		get_tree().quit(0)
