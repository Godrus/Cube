extends "res://world.gd"
## P-WORLD stage C, test-only entry point (scene `world_edit.tscn`): picking, editing and saving through the REAL
## client script in a real window. Mouse and keys are injected with `Input.parse_input_event` (synthetic input,
## not a person). Three phases, each its own OS process, driven by `scripts/test-world-persistence.ps1`:
##   --phase=write    edit the world (pick, remove, place, border voxels, dig under the character, wall), record
##                    what is expected in <dir>/expected.json, then wait; the driver closes the window (WM_CLOSE),
##                    which triggers the client's autosave
##   --phase=verify   start the real client (it loads the save), check voxels, hash, geometry, collision
##   --phase=corrupt  start with a damaged save: the client must keep it as .bad and start a fresh world
## Options: --dir=<dir> --save=<file> [--shot=<dir>] [--self-close] (close itself: for headless runs).
## Every phase writes <dir>/<phase>.result (PASS/FAIL lines and SUMMARY) and exits 0/1.

var phase := ""
var out_dir := ""
var shot_dir := ""
var failures := 0
var lines: Array = []

func _check(name: String, cond: bool, detail: String = "") -> void:
	var l := "%s %s %s" % ["PASS" if cond else "FAIL", name, detail]
	print("EDIT " + l)
	lines.append(l)
	if not cond:
		failures += 1

func _ready() -> void:
	phase = _str_arg("--phase=")
	out_dir = _str_arg("--dir=")
	shot_dir = _str_arg("--shot=")
	super._ready()
	if world == null:
		_finish(1)
		return
	await _frames(20)
	match phase:
		"write":
			await _write()
		"verify":
			await _verify()
		"corrupt":
			await _corrupt()
		_:
			_check("known phase", false, phase)
	_finish(0)

func _finish(_x: int) -> void:
	lines.append("SUMMARY failures=%d" % failures)
	print("EDIT SUMMARY phase=%s failures=%d" % [phase, failures])
	if out_dir != "":
		var f := FileAccess.open("%s/%s.result" % [out_dir, phase], FileAccess.WRITE)
		if f != null:
			f.store_string("\n".join(lines) + "\n")
			f.close()
	if phase == "write" and failures == 0:
		# wait to be closed (autosave on close), or close ourselves
		var rf := FileAccess.open("%s/ready.txt" % out_dir, FileAccess.WRITE)
		rf.store_string("ready\n")
		rf.close()
		if _has_flag("--self-close"):
			await _frames(5)
			get_tree().root.propagate_notification(NOTIFICATION_WM_CLOSE_REQUEST)
			get_tree().quit(0)
		return                                # the driver closes the window
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

var _mouse := Vector2.ZERO

func _move_mouse(p: Vector2) -> void:
	_mouse = p
	var e := InputEventMouseMotion.new()
	e.position = p
	e.global_position = p
	e.relative = Vector2.ZERO
	Input.parse_input_event(e)

func _click(button: MouseButton) -> void:
	for down in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = button
		e.pressed = down
		e.position = _mouse
		e.global_position = _mouse
		Input.parse_input_event(e)
		await get_tree().process_frame

## Moves the real mouse (synthetic) to where a world point appears on screen and lets a frame pass.
## In a real window a synthetic motion event does not move the viewport's cursor (that stays where the user's real
## mouse is), so the aim point is set through `aim_override`; clicks still arrive as real input events.
func _aim_at(world_point: Vector3) -> void:
	var sp := camera.unproject_position(world_point)
	aim_override = sp
	_move_mouse(sp)
	await _frames(3)

func _consistent() -> String:
	for k in chunks.keys():
		for b in 2:
			var s := Vector3i(k.x, b, k.y)
			var want: int = world.section_face_count(k.x, b, k.y)
			var have: int = drawn.get(s, 0)
			if want != have:
				return "section %s drawn %d, Rust %d" % [s, have, want]
	if chunks.size() != world.active_count():
		return "nodes %d, active chunks %d" % [chunks.size(), world.active_count()]
	return ""

func _shot(name: String) -> void:
	if DisplayServer.get_name() == "headless" or shot_dir == "":
		return
	await _frames(2)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	print("EDIT SHOT %s err=%d" % [name, img.save_png("%s/%s.png" % [shot_dir, name])])

## A z near 8 where the top face of column (x, z) is not hidden from the default camera (which looks along -z from
## +z) by higher columns on its camera side; the cursor then picks exactly that voxel.
func _visible_z(x: int) -> int:
	for z in [8, 7, 9, 6, 10, 5, 11, 4, 12]:
		var top: int = world.column_top(x, z)
		if world.column_top(x, z + 1) <= top and world.column_top(x, z + 2) <= top and world.column_top(x, z + 3) <= top + 1:
			return z
	return 8

## Aims at the top face of column (x, z), checks the pick, returns the picked voxel.
func _pick_top(x: int, z: int, label: String) -> bool:
	var top: int = world.column_top(x, z)
	await _aim_at(Vector3(x + 0.5, top + 1.0, z + 0.5))
	var ok := aim_valid and aim_voxel == Vector3i(x, top, z) and aim_normal == Vector3i(0, 1, 0)
	_check("%s: the cursor picks voxel (%d, %d, %d), face up" % [label, x, top, z], ok, "picked valid=%s %s face %s" % [aim_valid, aim_voxel, aim_normal])
	_check("%s: highlight sits on the picked voxel" % label, highlight.visible and highlight.position.is_equal_approx(Vector3(x + 0.5, top + 0.5, z + 0.5)))
	return ok

func _write() -> void:
	_check("character stands before editing", world.player_on_ground())
	var orig_top: int = world.column_top(8, 8)
	var hash0: String = world.world_hash()
	_check("hash can be read", hash0.length() == 16)
	await _shot("edit-1-start")

	# 1. dig under the feet: the character falls one voxel (collision follows the edit at once)
	var y0 := player.y
	await _pick_top(8, 8, "under feet")
	await _click(MOUSE_BUTTON_LEFT)
	_check("left click removed the voxel under the feet", world.voxel_at(8, orig_top, 8) == 0 and world.column_top(8, 8) == orig_top - 1)
	_check("geometry updated at once", _consistent() == "", _consistent())
	await _secs(0.8)
	_check("the character fell into the hole", y0 - player.y > 0.99 and world.player_on_ground(), "y %.2f -> %.2f" % [y0, player.y])
	await _frames(5)

	# 2. a removal inside the chunk, and on both sides of the x border 15|16 (the neighbour chunk changes too)
	for x in [11, 15, 16]:
		var zz := _visible_z(x)
		var top_before: int = world.column_top(x, zz)
		if await _pick_top(x, zz, "remove x=%d z=%d" % [x, zz]):
			await _click(MOUSE_BUTTON_LEFT)
		_check("remove x=%d changed the stored voxel" % x, world.voxel_at(x, top_before, zz) == 0 and edit_log.size() > 0 and edit_log.back()[0] == x and edit_log.back()[1] == top_before)
		_check("remove x=%d: geometry of both sides is consistent" % x, _consistent() == "", _consistent())
	# the voxel left of the hole at x=16 is visible now through the x=15 hole: both chunks must show the new faces
	await _shot("edit-2-removed")

	# 3. place on the face under the cursor: right click without a drag
	var tx := 13
	await _pick_top(tx, 8, "place on top of x=13")
	var top13: int = world.column_top(tx, 8)
	await _click(MOUSE_BUTTON_RIGHT)
	_check("right click placed a block on the picked face", world.voxel_at(tx, top13 + 1, 8) == 4, "voxel=%d" % world.voxel_at(tx, top13 + 1, 8))
	_check("geometry after placing is consistent", _consistent() == "", _consistent())
	# a right-button DRAG must orbit, not place
	var edits_before := edit_log.size()
	var e := InputEventMouseButton.new()
	e.button_index = MOUSE_BUTTON_RIGHT
	e.pressed = true
	e.position = _mouse
	Input.parse_input_event(e)
	await _frames(2)
	var m := InputEventMouseMotion.new()
	m.position = _mouse + Vector2(80, 0)
	m.relative = Vector2(80, 0)
	Input.parse_input_event(m)
	await _frames(2)
	e = InputEventMouseButton.new()
	e.button_index = MOUSE_BUTTON_RIGHT
	e.pressed = false
	e.position = _mouse + Vector2(80, 0)
	Input.parse_input_event(e)
	await _frames(3)
	_check("a right-button drag orbits and does not place", edit_log.size() == edits_before and absf(cam_yaw) > 0.1, "yaw %.2f" % cam_yaw)
	cam_yaw = 0.0
	_update_camera()
	await _frames(3)

	# 4. refused actions: out of reach, inside the character, unloaded
	await _aim_at(Vector3(8.5, 14.0, 8.5 - 60.0))
	var far_ok := true
	if aim_valid:
		far_ok = not aim_in_reach
	_check("a voxel more than 10 voxels away is out of reach", far_ok)
	var n_before := edit_log.size()
	if aim_valid and not aim_in_reach:
		await _click(MOUSE_BUTTON_LEFT)
		_check("clicking out of reach edits nothing", edit_log.size() == n_before)

	# 5. a two-high wall of placed blocks: walking into it is blocked
	cam_yaw = 0.0
	var wx := 10
	var top10: int = world.column_top(wx, 8)
	await _pick_top(wx, 8, "wall base")
	await _click(MOUSE_BUTTON_RIGHT)
	await _aim_at(Vector3(wx + 0.5, top10 + 2.0, 8.5))
	await _click(MOUSE_BUTTON_RIGHT)
	_check("two blocks placed", world.voxel_at(wx, top10 + 1, 8) == 4 and world.voxel_at(wx, top10 + 2, 8) == 4, "%d %d" % [world.voxel_at(wx, top10 + 1, 8), world.voxel_at(wx, top10 + 2, 8)])
	aim_override = Vector2(10, 300)                    # park the aim away from the world
	var x0 := player.x
	_key(KEY_D, true)                                  # D = +x with the camera at yaw 0
	await _secs(2.0)
	_key(KEY_D, false)
	_check("the wall blocks the character", player.x < wx - 0.3 + 0.02 or player.y > top10 + 2.0 - 0.01, "x %.2f -> %.2f y %.2f wall top %d" % [x0, player.x, player.y, top10 + 2])
	await _shot("edit-3-placed")
	_check("final geometry consistent", _consistent() == "", _consistent())

	# what must come back after a restart
	var expected := {"hash": world.world_hash(), "edits": edit_log, "orig_top": orig_top, "seed": world.seed()}
	var f := FileAccess.open("%s/expected.json" % out_dir, FileAccess.WRITE)
	f.store_string(JSON.stringify(expected))
	f.close()
	print("EDIT expected: %d edits, hash %s, unsaved edits %d" % [edit_log.size(), expected["hash"], edits_unsaved])

func _verify() -> void:
	var txt := FileAccess.get_file_as_string("%s/expected.json" % out_dir)
	var ex: Dictionary = JSON.parse_string(txt)
	_check("the client loaded the save at start", world.seed() == int(ex["seed"]) and world.active_count() > 0, "seed %d chunks %d" % [world.seed(), world.active_count()])
	await _secs(3.0)
	_check("world hash equals the hash before the restart", world.world_hash() == ex["hash"], "%s vs %s" % [world.world_hash(), ex["hash"]])
	var bad := 0
	for e in ex["edits"]:
		var v: int = world.voxel_at(int(e[0]), int(e[1]), int(e[2]))
		if v != int(e[3]):
			bad += 1
			print("EDIT voxel mismatch %s now %d" % [e, v])
	_check("all %d edited voxels came back" % ex["edits"].size(), bad == 0, "mismatches=%d" % bad)
	_check("geometry matches the stored voxels", _consistent() == "", _consistent())
	_check("the character stands (collision of the restored world)", world.player_on_ground() and not world.player_held(), "pos %s" % player)
	var top: int = world.column_top(floori(player.x), floori(player.z))
	_check("the character stands on the edited surface", absf(player.y - (top + 1.0)) < 0.05, "y %.3f column top %d" % [player.y, top])
	var dug: bool = floori(player.x) == 8 and floori(player.z) == 8
	_check("it landed in the hole dug under the old spawn point", (not dug) or absf(player.y - float(int(ex["orig_top"]))) < 0.05, "y %.3f original top %d" % [player.y, int(ex["orig_top"])])
	await _shot("edit-4-after-restart")

func _corrupt() -> void:
	_check("a damaged save was kept as .bad", FileAccess.file_exists(save_path + ".bad") and not FileAccess.file_exists(save_path))
	_check("a fresh world started instead", world.active_count() > 0 and world.seed() == 12345 and edit_log.is_empty())
	var txt := FileAccess.get_file_as_string("%s/expected.json" % out_dir)
	var ex: Dictionary = JSON.parse_string(txt)
	await _secs(1.0)
	_check("the fresh world is not the edited one", world.world_hash() != ex["hash"])
