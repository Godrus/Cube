extends "res://world.gd"
## P-WORLD stage A, test-only entry point (scene `world_seams.tscn`): for several seeds, builds a world twice
## (determinism of the content hash), checks that no section is stale, and with a real renderer writes
## screenshots of chunk corners and a measurement of frame times per radius. Exit code 0/1.
##   godot --headless --path godot-client world_seams.tscn -- --shot=<dir>
##   godot --path godot-client world_seams.tscn -- --shot=<dir> [--perf]
const SEEDS := [1, 7, 12345, 424242]

var failures := 0

func _ready() -> void:
	if not _init_world():
		print("SEAMS FAIL CubeWorld class missing")
		get_tree().quit(1)
		return
	auto_stream = false
	await _run()
	get_tree().quit(0 if failures == 0 else 1)

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("SEAMS %s %s %s" % ["PASS" if cond else "FAIL", name, detail])
	if not cond:
		failures += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _fill(seed: int, radius: int, at: Vector2) -> void:
	world.create(seed, radius)
	_clear_chunks()
	player = Vector3(at.x, 0, at.y)
	_place_player(at.x, at.y)
	world.set_player(player.x, player.y, player.z)
	world.pump_all()
	_apply_delta()

func _run() -> void:
	var shot_dir := ""
	var perf := false
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--shot="):
			shot_dir = a.substr(7)
		if a == "--perf":
			perf = true
	var headless := DisplayServer.get_name() == "headless"
	print("SEAMS ENV godot=%s gpu=%s" % [Engine.get_version_info().string, RenderingServer.get_video_adapter_name()])
	var hashes := {}
	for s in SEEDS:
		_fill(s, 2, Vector2(0.0, 0.0))      # centred on a chunk corner: chunks with negative and positive coordinates
		var h1: String = world.world_hash()
		var n: int = world.active_count()
		_fill(s, 2, Vector2(0.0, 0.0))
		var h2: String = world.world_hash()
		_check("seed %d: same seed, same world hash" % s, h1 == h2 and n == 25, "%s chunks=%d" % [h1, n])
		hashes[s] = h1
		_check("seed %d: one node per active chunk" % s, chunks.size() == n)
		if not headless and shot_dir != "":
			# close views along the chunk borders x = 0 and z = 0 (the four chunks around the corner meet there)
			var views := [Vector3(10, 12, 10), Vector3(-10, 12, 10), Vector3(0, 30, 28)]
			for i in views.size():
				camera.look_at_from_position(player + views[i], Vector3(0, 10, 0))
				await _frames(3)
				await RenderingServer.frame_post_draw
				var img := get_viewport().get_texture().get_image()
				print("SEAMS SHOT seed=%d view=%d err=%d" % [s, i, img.save_png("%s/seam-seed%d-view%d.png" % [shot_dir, s, i])])
	var uniq := {}
	for s in hashes:
		uniq[hashes[s]] = true
	_check("different seeds give different worlds", uniq.size() == SEEDS.size())
	if perf:
		await _perf()
	print("SEAMS SUMMARY failures=%d" % failures)

## Frame times while the player walks 400 voxels along +x at the client's speed, per radius (needs a renderer).
func _perf() -> void:
	for radius in [1, 2, 4, 6, 8]:
		world.create(12345, radius)
		_clear_chunks()
		player = Vector3(8.5, 0, 8.5)
		_place_player(player.x, player.z)
		_stream(1000)
		var worst := 0.0
		var total := 0.0
		var n := 0
		var t_end := player.x + 400.0
		var last := Time.get_ticks_usec()
		while player.x < t_end:
			await get_tree().process_frame
			var now := Time.get_ticks_usec()
			var dt := (now - last) / 1000000.0
			last = now
			_place_player(player.x + SPEED * 1.0 / 60.0, player.z)      # fixed step: comparable between runs
			_stream(PUMP_PER_FRAME)
			if n > 5:
				worst = maxf(worst, dt)
				total += dt
			n += 1
		print("SEAMS PERF radius=%d frames=%d avg_ms=%.2f worst_ms=%.2f chunks=%d queue=%d" % [radius, n, total / maxi(n - 6, 1) * 1000.0, worst * 1000.0, world.active_count(), world.queue_len()])
