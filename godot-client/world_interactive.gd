extends "res://world.gd"
## P-WORLD stage B, test-only entry point (scene `world_interactive.tscn`): an *interactive smoke test with
## synthetic input*. The real client script runs unchanged in a real window (`_process`, `_unhandled_input`,
## streaming, the character in Rust); this script only injects key and mouse events with
## `Input.parse_input_event`, exactly where a person's keyboard and mouse would enter, and checks the outcome.
## It is NOT a person at the keyboard. Exit code 0/1.
##   godot --path godot-client world_interactive.tscn -- [--shot=<dir>] [--radius=<n>] [--seed=<n>]
## Headless runs work too (no screenshots, frame times are then meaningless).

var failures := 0
var frame_ms: Array = []                # every real frame time while the test runs
var held_frames := 0
var below_surface := 0
var shot_dir := ""
var recording := false

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("WALK %s %s %s" % ["PASS" if cond else "FAIL", name, detail])
	if not cond:
		failures += 1

func _process(delta: float) -> void:
	super._process(delta)
	if not recording or world == null:
		return
	frame_ms.append(delta * 1000.0)
	if world.player_held():
		held_frames += 1
	elif player.y + 0.01 < float(world.column_top(floori(player.x), floori(player.z)) + 1) - 1.0:
		below_surface += 1                  # more than one voxel below the column under the centre: fell through

func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--shot="):
			shot_dir = a.substr(7)
	if not _init_world():
		print("WALK FAIL CubeWorld class missing")
		get_tree().quit(1)
		return
	_stream()
	_spawn_character()
	await _run()
	print("WALK SUMMARY failures=%d" % failures)
	get_tree().quit(0 if failures == 0 else 1)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _secs(s: float) -> void:
	var t := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t < int(s * 1000.0):
		await get_tree().process_frame

func _key(code: Key, down: bool) -> void:
	var e := InputEventKey.new()
	e.keycode = code
	e.physical_keycode = code
	e.pressed = down
	Input.parse_input_event(e)

func _mouse_button(idx: MouseButton, down: bool) -> void:
	var e := InputEventMouseButton.new()
	e.button_index = idx
	e.pressed = down
	e.position = Vector2(500, 300)
	Input.parse_input_event(e)

func _mouse_move(rel: Vector2) -> void:
	var e := InputEventMouseMotion.new()
	e.position = Vector2(500, 300) + rel
	e.relative = rel
	Input.parse_input_event(e)

func _shot(name: String) -> void:
	if DisplayServer.get_name() == "headless" or shot_dir == "":
		return
	await _frames(2)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	print("WALK SHOT %s err=%d" % [name, img.save_png("%s/%s.png" % [shot_dir, name])])

func _chunk_of(p: Vector3) -> Vector2i:
	return Vector2i(floori(p.x / 16.0), floori(p.z / 16.0))

func _run() -> void:
	print("WALK ENV godot=%s gpu=%s display=%s" % [Engine.get_version_info().string, RenderingServer.get_video_adapter_name(), DisplayServer.get_name()])
	await _frames(30)
	_check("character stands on the surface after spawn", world.player_on_ground() and not world.player_held(), "pos=%s" % player)
	var ground_y: float = world.column_top(floori(player.x), floori(player.z)) + 1.0
	_check("feet are on top of the column", absf(player.y - ground_y) < 0.05, "y=%.3f ground=%.0f" % [player.y, ground_y])
	await _shot("walk-1-spawn")

	# --- mouse camera (right button drag orbits, wheel zooms) ---
	var yaw0 := cam_yaw
	var pitch0 := cam_pitch
	var dist0 := cam_dist
	var cam_pos0 := camera.global_position
	_mouse_button(MOUSE_BUTTON_RIGHT, true)
	await _frames(2)
	_mouse_move(Vector2(100, 20))
	await _frames(2)
	_mouse_button(MOUSE_BUTTON_RIGHT, false)
	await _frames(2)
	_mouse_move(Vector2(60, 0))            # button released: must not orbit
	await _frames(2)
	_check("right-drag changes yaw and pitch", absf(cam_yaw - yaw0) > 0.3 and absf(cam_pitch - pitch0) > 0.05, "yaw %.2f->%.2f pitch %.2f->%.2f" % [yaw0, cam_yaw, pitch0, cam_pitch])
	var yaw1 := cam_yaw
	_check("mouse motion without the button does not orbit", is_equal_approx(yaw1, cam_yaw))
	_check("camera really moved", camera.global_position.distance_to(cam_pos0) > 1.0)
	_mouse_button(MOUSE_BUTTON_WHEEL_UP, true)
	await _frames(2)
	_check("wheel zooms in", cam_dist < dist0 - 1.0, "%.1f->%.1f" % [dist0, cam_dist])
	await _shot("walk-2-orbit")

	# --- movement relative to the view direction ---
	var fwd := -camera.global_transform.basis.z
	var flat := Vector2(fwd.x, fwd.z).normalized()
	var p0 := player
	_key(KEY_W, true)
	await _secs(1.0)
	_key(KEY_W, false)
	await _frames(5)
	var moved := Vector2(player.x - p0.x, player.z - p0.z)
	_check("W walks along the camera direction", moved.length() > 3.0 and moved.normalized().dot(flat) > 0.9, "moved %s want %s" % [moved, flat * 6.0])
	_check("walking speed is the character's (about 6 voxels/s)", moved.length() > 3.0 and moved.length() < 7.5, "%.2f" % moved.length())

	# --- jump ---
	await _frames(20)
	var y_rest := player.y
	var y_max := y_rest
	_key(KEY_SPACE, true)
	var tj := Time.get_ticks_msec()
	while Time.get_ticks_msec() - tj < 400:         # wall-clock time, so the test does not depend on the frame rate
		await get_tree().process_frame
		y_max = maxf(y_max, player.y)
	_key(KEY_SPACE, false)
	await _secs(1.0)
	_check("jump leaves the ground and lands again", y_max > y_rest + 0.9 and absf(player.y - y_rest) < 1.1 and world.player_on_ground(), "rest %.2f peak %.2f now %.2f" % [y_rest, y_max, player.y])

	# --- long walk across >= 5 chunk borders, in the real frame loop ---
	cam_yaw = -PI / 2.0                    # forward is now +x
	_update_camera()
	recording = true
	frame_ms.clear()
	var start := player
	var chunk := _chunk_of(player)
	var borders := 0
	var last_x := player.x
	var stuck_s := 0.0
	var t_start := Time.get_ticks_msec()
	_key(KEY_W, true)
	while player.x - start.x < 100.0 and Time.get_ticks_msec() - t_start < 60000:
		await get_tree().process_frame
		var c := _chunk_of(player)
		if c != chunk:
			borders += 1
			chunk = c
		if player.x > last_x + 0.001:
			last_x = player.x
			stuck_s = 0.0
			_key(KEY_SPACE, false)
		else:
			stuck_s += get_process_delta_time()
			if stuck_s > 0.4:
				_key(KEY_SPACE, true)       # blocked by more than the step-up: jump
		if borders == 3:
			await _shot("walk-3-mid")
			borders += 0
	_key(KEY_W, false)
	_key(KEY_SPACE, false)
	await _frames(10)
	recording = false
	var walked := player.x - start.x
	_check("walked 100 voxels in the real frame loop", walked >= 100.0, "walked %.1f in %d ms" % [walked, Time.get_ticks_msec() - t_start])
	_check("crossed at least five chunk borders", borders >= 5, "borders=%d" % borders)
	_check("never fell through the surface", below_surface == 0, "frames below=%d" % below_surface)
	_check("never waited for a chunk while walking", held_frames == 0, "held frames=%d" % held_frames)
	_print_perf("walk", frame_ms)
	await _shot("walk-4-end")

	# --- teleport onto unloaded ground: the character waits, then lands ---
	var before_teleport := player
	buttons["Jump 64"].pressed.emit()
	await _frames(1)
	var saw_held: bool = world.player_held()
	var t0 := Time.get_ticks_msec()
	recording = true
	frame_ms.clear()
	held_frames = 0
	while (world.player_held() or not world.player_on_ground()) and Time.get_ticks_msec() - t0 < 10000:
		await get_tree().process_frame
	recording = false
	_check("teleport onto unloaded chunks waits (held) then lands", world.player_on_ground() and not world.player_held() and player.x > before_teleport.x + 60.0,
		"saw_held=%s landed after %d ms at %s" % [saw_held, Time.get_ticks_msec() - t0, player])
	var gy: float = world.column_top(floori(player.x), floori(player.z)) + 1.0
	_check("lands on the surface of the new chunk", absf(player.y - gy) < 0.05, "y=%.3f ground=%.0f" % [player.y, gy])
	_print_perf("teleport", frame_ms)
	await _shot("walk-5-teleport")

func _print_perf(label: String, a: Array) -> void:
	if a.is_empty():
		return
	var s := a.duplicate()
	s.sort()
	var sum := 0.0
	for v in s:
		sum += v
	print("WALK PERF %s frames=%d avg_ms=%.2f p50=%.2f p95=%.2f p99=%.2f max=%.2f over16.7=%d" % [label, s.size(), sum / s.size(), s[s.size() / 2], s[int(s.size() * 0.95)], s[int(s.size() * 0.99)], s[s.size() - 1], s.filter(func(v): return v > 16.7).size()])
