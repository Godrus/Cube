extends Node3D
## UI and glue only (ADP-02): reads input, calls the Rust CubeWorld node, copies the camera pose.
## No geometry, no world rules and no file writes are done here (except test pictures and test
## labels when a command line option asks for them).
##
## Controls: click to capture the mouse, WASD move, mouse looks, Space jumps (walking) or rises
## (flying), Shift descends (flying), F switches walking / flying, V switches first / third person,
## 1-9 or the mouse wheel choose the hotbar slot, left click mines the block under the crosshair
## (it goes into the inventory), right click places the selected block (it is used up), I opens
## the inventory (click two slots to swap them), C switches creative / survival, E swings the sword at
## the nearest creature (only with --creatures), Esc releases the mouse. The world and the player are saved when the
## window is closed and are loaded again on the next start (same --world directory).
##
## Command line after `--`:
##   --smoke                          headless self check
##   --screenshot <png>               save one picture and quit
##   --edit-shots <before> <after>    picture, remove the block under the crosshair, picture
##   --fly                            start flying instead of walking (QA options always fly)
##   --creatures                      switch creatures on (internal, prototype combat: slimes chase and
##                                    hurt you, E attacks; they are a Rust-side feature, drawn here)
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

const Rig = preload("res://character_rig.gd")

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
var start_flying := false
var third_person := false
var player_body: MeshInstance3D
var rig  # CubeCharacterRig: the voxel traveller (CHAR-01), drawn from the authoritative state
var char_res := 0
var char_lod := -1
var hud_layer: CanvasLayer
var cs_view := ""      # --char-shot <view> <png>: developer preview in the real world scene
var cs_path := ""
var cs_feet := Vector3.ZERO
var cs_have := false
var bench_n := -1      # --char-bench <n>: n instances, frame-time and draw-call measurement
var bench_times := []
var bench_rigs := []
var creatures_on := false
var creature_nodes := []
var creature_prev := []
var health_bar: ProgressBar
var health_label: Label
var hurt_overlay: ColorRect
var prev_health := -1
var hud: Label
var objective_label: Label
var message_label: Label
var hotbar: HBoxContainer
var inventory_panel: PanelContainer
var slot_nodes := []
var panel_slot_nodes := []
var picked_slot := -1
var target_box: MeshInstance3D
var ghost_box: MeshInstance3D
var toast := ""
var toast_left := 0.0
var scen_active := false
var scen_hud := false
var scen_steps := []
var scen_out := ""
var scen_i := 0
var scen_cmd = null
var scen_events := []
var scen_changed := 0
var scen_edit_event = null
var scen_hits := 0
var fight_result := ""
var fight_trace := []
var fight_best := 1e9
var fight_stuck := 0
var fight_last := ""
var fight_prev_living := -1
var fight_hits_at_start := 0
var fight_start_frame := 0
var fight_states := []
var scen_last_hit_pos := []


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
			"--char-shot":
				cs_view = args[i + 1]
				cs_path = args[i + 2]
				i += 2
			"--char-bench":
				i += 1
				bench_n = int(args[i])
			"--char-res":
				i += 1
				char_res = int(args[i])
			"--lod":
				i += 1
				char_lod = int(args[i])
			"--fly":
				start_flying = true
			"--creatures":
				OS.set_environment("CUBE_CREATURES", "1")
				creatures_on = true
		i += 1
	var create := not FileAccess.file_exists(dir.path_join("manifest"))
	if not world.open_world(dir, seed_value, create):
		push_error("could not open the world")
		get_tree().quit(2)
		return
	print("CUBE world opened: dir=", dir, " create=", create)
	var qa_mode := smoke or shot_path != "" or before_path != "" or scen_active
	if cs_view != "" or bench_n >= 0:
		world.set_walk(true)
		third_person = true
		world.set_third_person(true)
		_build_player_and_hud()
		for ch in get_children():
			if ch is CanvasLayer:
				hud_layer = ch
		if cs_view != "" and cs_view != "game" and cs_view != "move" and cs_view != "attack":
			hud_layer.visible = false
			$Sun.shadow_enabled = true
		if bench_n >= 0:
			hud_layer.visible = false
			DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
		return
	if not qa_mode:
		if not start_flying and not world.is_walking():
			world.set_walk(true)
		_build_player_and_hud()
	elif scen_hud:
		_build_player_and_hud()  # a scenario that wants the interface in its pictures


func _exit_tree() -> void:
	# A normal quit (window closed, Alt+F4, quit-after) must stop the world so that the journal is
	# flushed and the world can be reopened; close() is a no-op when the world is already closed.
	if world != null:
		world.close()


func _build_player_and_hud() -> void:
	player_body = MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(0.6, 1.8, 0.6)
	player_body.mesh = box
	var m := StandardMaterial3D.new()
	m.albedo_color = Color(0.9, 0.55, 0.2)
	player_body.material_override = m
	player_body.visible = false
	add_child(player_body)
	rig = Rig.new()
	if not rig.setup(char_res):
		push_error("voxel character failed to build; falling back to the box")
		rig = null
	else:
		if char_lod >= 0:
			rig.forced_lod = char_lod
		add_child(rig)
		print("CUBE character: ", character_stats_text())
	# Target outline and placement ghost (cubes of one voxel, drawn slightly larger).
	target_box = _make_marker(Color(1, 1, 1, 0.22))
	ghost_box = _make_marker(Color(0.2, 1, 0.2, 0.45))
	var layer := CanvasLayer.new()
	add_child(layer)
	# Crosshair: a dark outline under a light cross, readable on any background.
	var cross := Label.new()
	cross.text = "+"
	cross.add_theme_font_size_override("font_size", 28)
	cross.add_theme_color_override("font_color", Color(1, 1, 1))
	cross.add_theme_color_override("font_outline_color", Color(0, 0, 0))
	cross.add_theme_constant_override("outline_size", 6)
	cross.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	cross.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	cross.set_anchors_preset(Control.PRESET_CENTER)
	cross.grow_horizontal = Control.GROW_DIRECTION_BOTH
	cross.grow_vertical = Control.GROW_DIRECTION_BOTH
	layer.add_child(cross)
	hud = Label.new()
	hud.position = Vector2(10, 8)
	hud.add_theme_color_override("font_outline_color", Color(0, 0, 0))
	hud.add_theme_constant_override("outline_size", 4)
	layer.add_child(hud)
	objective_label = Label.new()
	objective_label.position = Vector2(10, 32)
	objective_label.add_theme_color_override("font_color", Color(1, 0.95, 0.6))
	objective_label.add_theme_color_override("font_outline_color", Color(0, 0, 0))
	objective_label.add_theme_constant_override("outline_size", 4)
	layer.add_child(objective_label)
	health_bar = ProgressBar.new()
	health_bar.position = Vector2(10, 58)
	health_bar.custom_minimum_size = Vector2(200, 16)
	health_bar.size = Vector2(200, 16)
	health_bar.show_percentage = false
	health_bar.visible = false
	layer.add_child(health_bar)
	health_label = Label.new()
	health_label.position = Vector2(214, 54)
	health_label.add_theme_color_override("font_outline_color", Color(0, 0, 0))
	health_label.add_theme_constant_override("outline_size", 4)
	layer.add_child(health_label)
	hurt_overlay = ColorRect.new()
	hurt_overlay.color = Color(1, 0, 0, 0)
	hurt_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	hurt_overlay.mouse_filter = Control.MOUSE_FILTER_IGNORE
	layer.add_child(hurt_overlay)
	message_label = Label.new()
	message_label.set_anchors_preset(Control.PRESET_CENTER_TOP)
	message_label.grow_horizontal = Control.GROW_DIRECTION_BOTH
	message_label.position = Vector2(0, 70)
	message_label.add_theme_font_size_override("font_size", 22)
	message_label.add_theme_color_override("font_outline_color", Color(0, 0, 0))
	message_label.add_theme_constant_override("outline_size", 6)
	layer.add_child(message_label)
	# Hotbar (always visible) and the inventory panel (key I).
	hotbar = HBoxContainer.new()
	hotbar.set_anchors_preset(Control.PRESET_CENTER_BOTTOM)
	hotbar.grow_horizontal = Control.GROW_DIRECTION_BOTH
	hotbar.grow_vertical = Control.GROW_DIRECTION_BEGIN
	hotbar.position = Vector2(0, -12)
	hotbar.add_theme_constant_override("separation", 4)
	layer.add_child(hotbar)
	for i in range(9):
		var slot := _make_slot(i)
		hotbar.add_child(slot)
		slot_nodes.append(slot)
	inventory_panel = PanelContainer.new()
	inventory_panel.set_anchors_preset(Control.PRESET_CENTER)
	inventory_panel.grow_horizontal = Control.GROW_DIRECTION_BOTH
	inventory_panel.grow_vertical = Control.GROW_DIRECTION_BOTH
	inventory_panel.visible = false
	var grid := GridContainer.new()
	grid.columns = 9
	grid.add_theme_constant_override("h_separation", 4)
	grid.add_theme_constant_override("v_separation", 4)
	inventory_panel.add_child(grid)
	for i in range(27):
		var slot := _make_slot(i)
		grid.add_child(slot)
		panel_slot_nodes.append(slot)
	layer.add_child(inventory_panel)


func _make_marker(color: Color) -> MeshInstance3D:
	var mi := MeshInstance3D.new()
	var b := BoxMesh.new()
	b.size = Vector3(1.01, 1.01, 1.01)
	mi.mesh = b
	var mat := StandardMaterial3D.new()
	mat.albedo_color = color
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mi.material_override = mat
	mi.visible = false
	add_child(mi)
	return mi


## One inventory slot: a coloured swatch, a count and a frame that is bright when selected.
func _make_slot(index: int) -> Control:
	var panel := Panel.new()
	panel.custom_minimum_size = Vector2(54, 54)
	panel.set_meta("index", index)
	var swatch := ColorRect.new()
	swatch.name = "swatch"
	swatch.position = Vector2(9, 9)
	swatch.size = Vector2(36, 36)
	swatch.mouse_filter = Control.MOUSE_FILTER_IGNORE
	panel.add_child(swatch)
	var count := Label.new()
	count.name = "count"
	count.position = Vector2(30, 32)
	count.add_theme_color_override("font_outline_color", Color(0, 0, 0))
	count.add_theme_constant_override("outline_size", 5)
	count.mouse_filter = Control.MOUSE_FILTER_IGNORE
	panel.add_child(count)
	var key := Label.new()
	key.name = "key"
	key.text = str(index + 1) if index < 9 else ""
	key.position = Vector2(4, 0)
	key.add_theme_font_size_override("font_size", 12)
	key.mouse_filter = Control.MOUSE_FILTER_IGNORE
	panel.add_child(key)
	panel.gui_input.connect(_on_slot_input.bind(index))
	return panel


func _on_slot_input(event: InputEvent, index: int) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		if picked_slot < 0:
			picked_slot = index
		else:
			world.swap_slots(picked_slot, index)
			picked_slot = -1


func _style_slot(panel: Control, selected: bool, picked: bool) -> void:
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color(0.1, 0.1, 0.12, 0.8)
	sb.set_border_width_all(4 if selected else 2)
	sb.border_color = Color(1, 0.9, 0.3) if selected else (Color(0.4, 0.8, 1) if picked else Color(0.5, 0.5, 0.55))
	panel.add_theme_stylebox_override("panel", sb)


func _fill_slot(panel: Control, item: int, count: int, creative: bool) -> void:
	var swatch: ColorRect = panel.get_node("swatch")
	var label: Label = panel.get_node("count")
	swatch.visible = item != 0
	if item != 0:
		swatch.color = world.item_colour(item)
	label.text = "" if (item == 0 or creative) else str(count)


func _update_creatures_and_health(g: Dictionary) -> void:
	# Drawing only: the creatures, their health and the player's health come from Rust.
	var hp: int = int(g["health"])
	var mx: int = int(g["max_health"])
	health_bar.visible = mx > 0
	health_bar.max_value = max(mx, 1)
	health_bar.value = hp
	health_label.text = "HP %d/%d" % [hp, mx] if mx > 0 else ""
	if prev_health >= 0 and hp < prev_health:
		hurt_overlay.color = Color(1, 0, 0, 0.35)
		if rig != null:
			rig.trigger_hit()
	hurt_overlay.color.a = max(0.0, hurt_overlay.color.a - 1.5 * get_process_delta_time())
	prev_health = hp
	if not creatures_on:
		return
	var list: Array = world.creatures()
	while creature_nodes.size() < list.size():
		var n := MeshInstance3D.new()
		var b := BoxMesh.new()
		b.size = Vector3(0.9, 0.8, 0.9)
		n.mesh = b
		n.material_override = StandardMaterial3D.new()
		var tag := Label3D.new()
		tag.billboard = BaseMaterial3D.BILLBOARD_ENABLED
		tag.position = Vector3(0, 0.9, 0)
		tag.pixel_size = 0.01
		tag.no_depth_test = true
		n.add_child(tag)
		add_child(n)
		creature_nodes.append(n)
		creature_prev.append(-1)
	for i in range(creature_nodes.size()):
		var node: MeshInstance3D = creature_nodes[i]
		if i >= list.size():
			node.visible = false
			creature_prev[i] = -1
			continue
		var c: Dictionary = list[i]
		node.visible = true
		node.position = (c["pos"] as Vector3) + Vector3(0, 0.4, 0)
		node.rotation = Vector3(0, float(c["yaw"]), 0)
		var hit_now: bool = creature_prev[i] >= 0 and int(c["health"]) < int(creature_prev[i])
		creature_prev[i] = int(c["health"])
		var col := Color(0.3, 0.8, 0.35)
		if int(c["state"]) == 2:
			col = Color(0.9, 0.35, 0.3)
		elif int(c["state"]) == 1:
			col = Color(0.75, 0.85, 0.3)
		if hit_now:
			col = Color(1, 1, 1)
		(node.material_override as StandardMaterial3D).albedo_color = col
		(node.get_child(0) as Label3D).text = "%d/%d" % [int(c["health"]), int(c["max_health"])]


func _update_player_and_hud() -> void:
	if player_body == null:
		return
	_update_creatures_and_health(world.game_view())
	var feet: Vector3 = world.player_feet()
	var show_body: bool = third_person and world.is_walking()
	if rig != null:
		rig.visible = show_body
		player_body.visible = false
		rig.update_state(feet, cam.rotation.y, world.is_grounded(), get_process_delta_time())
		rig.auto_lod(cam.position)
	else:
		player_body.visible = show_body
		player_body.position = feet + Vector3(0, 0.9, 0)
		player_body.rotation = Vector3(0, cam.rotation.y, 0)
	var p: Vector3 = cam.position
	var st: Dictionary = world.stream_stats()
	hud.text = "%s | %s | %s | %s | x %.0f y %.0f z %.0f | bricks %d" % [
		"walking" if world.is_walking() else "flying",
		"third person" if third_person else "first person",
		"creative" if world.game_view()["creative"] else "survival",
		world.game_view()["biome"],
		p.x, p.y, p.z, st["resident"]]
	var g: Dictionary = world.game_view()
	var slots: PackedInt32Array = g["slots"]
	var creative: bool = g["creative"]
	for i in range(9):
		_fill_slot(slot_nodes[i], slots[2 * i], slots[2 * i + 1], creative)
		_style_slot(slot_nodes[i], i == int(g["selected"]), false)
	if inventory_panel.visible:
		for i in range(27):
			_fill_slot(panel_slot_nodes[i], slots[2 * i], slots[2 * i + 1], creative)
			_style_slot(panel_slot_nodes[i], i == int(g["selected"]), i == picked_slot)
	objective_label.text = "%s: %s%s" % [g["objective_title"], g["objective_text"], "  [done]" if g["objective_done"] else ""]
	var msg: String = g["message"]
	if toast != "":
		msg = toast
		toast_left -= get_process_delta_time()
		if toast_left <= 0.0:
			toast = ""
	message_label.text = msg
	# Target outline and placement ghost.
	var t: Dictionary = world.target_info()
	if t.is_empty() or inventory_panel.visible:
		target_box.visible = false
		ghost_box.visible = false
		return
	var vox: Vector3 = t["voxel"]
	var front: Vector3 = t["front"]
	target_box.position = vox + Vector3(0.5, 0.5, 0.5)
	target_box.visible = true
	ghost_box.position = front + Vector3(0.5, 0.5, 0.5)
	ghost_box.visible = true
	var gm: StandardMaterial3D = ghost_box.material_override
	if t["can_place"]:
		var sel: int = int(g["selected"])
		var item: int = slots[2 * sel]
		var c: Color = world.item_colour(item)
		gm.albedo_color = Color(c.r, c.g, c.b, 0.5)
	else:
		gm.albedo_color = Color(1, 0.15, 0.15, 0.5)
	var tm: StandardMaterial3D = target_box.material_override
	tm.albedo_color = Color(1, 1, 1, 0.22) if t["can_remove"] else Color(1, 0.3, 0.3, 0.3)


func _say(text: String) -> void:
	toast = text
	toast_left = 2.5


func _input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and player_body != null:
		if event.keycode == KEY_F:
			world.set_walk(not world.is_walking())
			if not world.is_walking():
				third_person = false
				world.set_third_person(false)
		elif event.keycode == KEY_E and creatures_on:
			var ra: String = world.attack()
			if rig != null:
				rig.trigger_attack()
			if ra != "ok" and ra != "none":
				_say(ra)
		elif event.keycode == KEY_V and world.is_walking():
			third_person = not third_person
			world.set_third_person(third_person)
		elif event.keycode >= KEY_1 and event.keycode <= KEY_9:
			world.select_slot(event.keycode - KEY_1)
		elif event.keycode == KEY_C:
			var g: Dictionary = world.game_view()
			world.set_creative(not g["creative"])
			_say("Creative mode" if not g["creative"] else "Survival mode")
		elif event.keycode == KEY_I:
			inventory_panel.visible = not inventory_panel.visible
			picked_slot = -1
			Input.mouse_mode = Input.MOUSE_MODE_VISIBLE if inventory_panel.visible else Input.MOUSE_MODE_CAPTURED
	if event is InputEventMouseButton and event.pressed:
		if inventory_panel != null and inventory_panel.visible:
			return
		if Input.mouse_mode != Input.MOUSE_MODE_CAPTURED:
			Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
		elif event.button_index == MOUSE_BUTTON_LEFT:
			var r: String = world.mine()
			if r != "ok" and r != "none":
				_say(r)
		elif event.button_index == MOUSE_BUTTON_RIGHT:
			var r2: String = world.place()
			if r2 != "ok" and r2 != "none":
				_say(r2)
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
			world.scroll_slot(-1)
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			world.scroll_slot(1)
	elif event is InputEventMouseMotion and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		look += Vector2(-event.relative.x, -event.relative.y) * 0.003
	elif event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		if inventory_panel != null and inventory_panel.visible:
			inventory_panel.visible = false
		Input.mouse_mode = Input.MOUSE_MODE_VISIBLE


func character_stats_text() -> String:
	var st: Dictionary = rig.asset.stats(0)
	var l1: Dictionary = rig.asset.stats(1)
	var l2: Dictionary = rig.asset.stats(2)
	return "res=%d/unit voxels=%d/%d/%d tris=%d/%d/%d mesh_bytes=%d/%d/%d build_ms=%.1f rig_setup_ms=%.1f id=%x hash=%s height=%.2f" % [
		int(st["res"]), st["voxels"], l1["voxels"], l2["voxels"], st["triangles"], l1["triangles"],
		l2["triangles"], st["mesh_bytes"], l1["mesh_bytes"], l2["mesh_bytes"], st["build_ms"],
		rig.setup_ms, st["asset_id"], st["content_hash"], st["height"]]


func _process(delta: float) -> void:
	if cs_view != "":
		_charshot_frame(delta)
		return
	if bench_n >= 0:
		_bench_frame(delta)
		return
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
	_update_player_and_hud()
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
	scen_hud = bool(data.get("hud", false))
	if bool(data.get("creatures", false)):
		OS.set_environment("CUBE_CREATURES", "1")
		creatures_on = true
	scen_active = true


func _event(op: String, extra: Dictionary = {}) -> Dictionary:
	var e := {"op": op, "frame": frames, "bricks": world.brick_count(),
		"camera_position": [cam.position.x, cam.position.y, cam.position.z],
		"camera_rotation": [cam.rotation.x, cam.rotation.y, cam.rotation.z],
		"walking": world.is_walking(), "grounded": world.is_grounded(),
		"feet": [world.player_feet().x, world.player_feet().y, world.player_feet().z],
		"stats": world.stream_stats(), "ms": Time.get_ticks_msec(), "game": world.game_view(),
		"creatures": _creatures_plain(), "target_material": int(world.target_info().get("material", -1)), "nodes": _nodes_plain(), "hp_label": "" if health_label == null else health_label.text, "hp_bar": 0 if health_bar == null else health_bar.value}
	for k in extra:
		e[k] = extra[k]
	scen_events.append(e)
	return e


func fight_reset() -> void:
	fight_result = ""
	fight_trace = []
	fight_best = 1e9
	fight_stuck = 0
	fight_last = ""
	fight_prev_living = -1
	fight_hits_at_start = scen_hits
	fight_start_frame = frames
	fight_states = []


func _write_progress() -> void:
	# Written while a scenario runs so that a stuck run can say where it stopped.
	var f := FileAccess.open(scen_out.path_join("progress.txt"), FileAccess.WRITE)
	if f == null:
		return
	var op := "" if scen_cmd == null else str(scen_cmd["op"])
	f.store_string("step %d/%d op=%s frame=%d hp=%s creatures=%d hits=%d fight=%s last=%s
" % [
		scen_i, scen_steps.size(), op, frames, str(world.game_view()["health"]), world.creatures().size(), scen_hits, fight_result, fight_last])
	f.close()


func _creatures_plain() -> Array:
	var out := []
	for c in world.creatures():
		var p: Vector3 = c["pos"]
		out.append({"pos": [p.x, p.y, p.z], "health": c["health"], "max_health": c["max_health"], "state": c["state"]})
	return out


func _nodes_plain() -> Array:
	var out := []
	for n in creature_nodes:
		if (n as MeshInstance3D).visible:
			var p: Vector3 = (n as MeshInstance3D).position
			out.append([p.x, p.y, p.z])
	return out


func _scenario_frame() -> void:
	frames += 1
	if frames % 60 == 0:
		_write_progress()
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
	elif op == "hover":
		# QA (scenario mode only): fly forward while keeping a fixed height above the ground below,
		# the way a person flying over the land would; `height` voxels above the surface.
		fwd = 1
		var hp: Vector3 = cam.position
		var want_y: float = float(world.ground_height(int(floor(hp.x)), int(floor(hp.z)))) + float(scen_cmd.get("height", 30))
		up = 1 if hp.y < want_y - 2.0 else (-1 if hp.y > want_y + 2.0 else 0)
		if hp.x <= float(scen_cmd.get("until_x", -1e9)):
			scen_cmd["_left"] = 1
		if hp.z >= float(scen_cmd.get("until_z", 1e9)):
			scen_cmd["_left"] = 1
	elif op == "fight":
		# QA bot (scenario mode only): walk to the nearest living creature and swing at it, the way a
		# person would by hand. It ends itself with an explicit result: killed, stuck (no progress
		# for 4 s), no_target, or timeout (the step's frame budget).
		if int(scen_cmd["_left"]) == int(scen_cmd["_total"]):
			fight_reset()
		var list: Array = world.creatures()
		var feet: Vector3 = world.player_feet()
		var best = null
		var bd := 1e9
		var living := 0
		for c in list:
			if int(c["state"]) == 3:
				continue
			living += 1
			var d: float = feet.distance_to(c["pos"])
			if d < bd:
				bd = d
				best = c
		# Killed = the creature that was hit is gone (the population may already have spawned another).
		if scen_hits > fight_hits_at_start and scen_last_hit_pos.size() == 3:
			var lp := Vector3(scen_last_hit_pos[0], scen_last_hit_pos[1], scen_last_hit_pos[2])
			var still := false
			for c2 in list:
				if int(c2["state"]) != 3 and (c2["pos"] as Vector3).distance_to(lp) < 1.5:
					still = true
			if not still:
				fight_result = "killed"
		if best == null or bd > 90.0:
			if fight_result == "":
				fight_result = "no_target"
		elif fight_result == "":
			var bp: Vector3 = best["pos"]
			if not fight_states.has(int(best["state"])):
				fight_states.append(int(best["state"]))
			var want := atan2(-(bp.x - feet.x), -(bp.z - feet.z))
			dyaw = wrapf(want - cam.rotation.y, -PI, PI) * 0.5
			if bd < fight_best - 0.05:
				fight_best = bd
				fight_stuck = 0
			else:
				fight_stuck += 1
			if fight_stuck > 240 and bd > 2.0:
				fight_result = "stuck"
			elif bd > 2.0:
				fwd = 1
				up = 1 if frames % 40 == 0 else 0
			else:
				fight_last = world.attack()
				if rig != null:
					rig.trigger_attack()
				if fight_last == "ok":
					scen_hits += 1
					scen_last_hit_pos = [bp.x, bp.y, bp.z]
			if int(scen_cmd["_total"]) - int(scen_cmd["_left"]) > 0 and frames % 30 == 0:
				fight_trace.append({"frame": frames, "dist": snappedf(bd, 0.01), "state": int(best["state"]), "hp": int(world.game_view()["health"]),
					"hits": scen_hits, "last": fight_last, "living": living, "target_hp": int(best["health"])})
		if fight_result != "":
			scen_cmd["_left"] = 1
	if not freeze_world:
		var changed: int = world.frame(fwd, right, up, dyaw, dpitch, 1.0 / 60.0)
		if changed > 0:
			scen_changed += changed
			if scen_edit_event != null:
				scen_edit_event["bricks_changed"] = int(scen_edit_event.get("bricks_changed", 0)) + changed
	cam.position = world.camera_position()
	cam.rotation = world.camera_rotation()
	_update_player_and_hud()
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
		"mine":
			var r: String = world.mine()
			scen_edit_event = _event("mine", {"result": r, "bricks_changed": 0})
		"place":
			var r2: String = world.place()
			scen_edit_event = _event("place", {"result": r2, "bricks_changed": 0})
		"select":
			world.select_slot(int(scen_cmd.get("slot", 0)))
			_event("select")
		"creative":
			world.set_creative(bool(scen_cmd.get("on", true)))
			_event("creative")
		"inventory":
			if inventory_panel != null:
				inventory_panel.visible = bool(scen_cmd.get("open", true))
			_event("inventory")
		"teleport":
			world.qa_teleport(float(scen_cmd["x"]), float(scen_cmd["y"]), float(scen_cmd["z"]), float(scen_cmd.get("yaw", 0.0)), float(scen_cmd.get("pitch", 0.0)))
			cam.position = world.camera_position()
			cam.rotation = world.camera_rotation()
			_event("teleport")
		"fight":
			_event("fight", {"hits": scen_hits, "last_hit_pos": scen_last_hit_pos, "result": fight_result if fight_result != "" else "timeout", "trace": fight_trace, "states": fight_states, "frames_used": frames - fight_start_frame + 1})
		"attack":
			if rig != null:
				rig.trigger_attack()
			_event("attack", {"result": world.attack()})
		"mode":
			world.set_walk(bool(scen_cmd.get("walk", true)))
			world.set_third_person(bool(scen_cmd.get("third", false)))
			_event("mode", {"walk": bool(scen_cmd.get("walk", true))})
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


# ---- developer preview of the voxel character (real world scene, real renderer) -------------

const CS_LAND := 220  # frames until the walking player has landed and the terrain is resident


func _flat_spot() -> Vector2i:
	# a 9 x 9 patch of equal ground height (real terrain heights from Rust) near the origin
	for r in range(0, 60, 2):
		for ang in range(0, 360, 20):
			var cx := int(r * cos(deg_to_rad(ang)))
			var cz := int(r * sin(deg_to_rad(ang)))
			var h0: int = world.ground_height(cx, cz)
			var ok := true
			for dx in range(-4, 5):
				for dz in range(-4, 5):
					if world.ground_height(cx + dx, cz + dz) != h0:
						ok = false
						break
				if not ok:
					break
			if ok:
				return Vector2i(cx, cz)
	return Vector2i.ZERO


func _charshot_frame(delta: float) -> void:
	frames += 1
	if frames == 2 and cs_view != "game" and cs_view != "move" and cs_view != "attack":
		var fs := _flat_spot()
		var gh: int = world.ground_height(fs.x, fs.y)
		world.qa_teleport(fs.x + 0.5, gh + 3.0, fs.y + 0.5, 0.0, 0.0)
		print("CUBE char-shot flat spot ", fs, " height ", gh)
	var fwd := 0
	var game_view: bool = cs_view == "game" or cs_view == "move" or cs_view == "attack"
	if (cs_view == "move" or cs_view == "attack") and frames > CS_LAND and frames < CS_LAND + 26:
		fwd = 1
	world.frame(fwd, 0, 0, 0.0, 0.0, delta)
	cam.position = world.camera_position()
	cam.rotation = world.camera_rotation()
	_update_player_and_hud()
	if frames < CS_LAND:
		return
	if not cs_have:
		cs_feet = world.player_feet()
		cs_have = true
	if cs_view == "attack" and frames == CS_LAND + 8:
		rig.trigger_attack()
	if not game_view:
		# studio views: the rig stands on the real terrain, facing -Z, camera placed around it
		rig.update_state(cs_feet, 0.0, true, delta)
		target_box.visible = false
		ghost_box.visible = false
		rig.visible = true
		var look_at_p := cs_feet + Vector3(0, 0.9, 0)
		var cp := cs_feet
		cam.fov = 38.0
		match cs_view:
			"front", "lod0", "lod1", "lod2":
				cp += Vector3(0.0, 1.15, -3.6)
			"side":
				cp += Vector3(3.6, 1.15, 0.0)
			"back":
				cp += Vector3(0.0, 1.15, 3.6)
			"face":
				cp += Vector3(0.5, 1.72, -0.95)
				look_at_p = cs_feet + Vector3(0, 1.64, 0)
				cam.fov = 28.0
			"armor":
				cp += Vector3(-1.05, 1.55, -1.1)
				look_at_p = cs_feet + Vector3(-0.1, 1.30, 0)
				cam.fov = 34.0
			"torso":
				cp += Vector3(0.9, 1.3, -1.9)
				look_at_p = cs_feet + Vector3(0, 1.1, 0)
		match cs_view:
			"lod0": rig.forced_lod = 0
			"lod1": rig.forced_lod = 1
			"lod2": rig.forced_lod = 2
		rig.auto_lod(cp)
		cam.position = cp
		cam.look_at(look_at_p, Vector3.UP)
	var shoot := CS_LAND + 30
	if cs_view == "attack":
		shoot = CS_LAND + 8 + 8
	if cs_view == "game":
		shoot = CS_LAND + 12
	if frames == shoot:
		get_viewport().get_texture().get_image().save_png(cs_path)
		print("CUBE char-shot saved: ", cs_path, " view=", cs_view, " lod=", rig.lod)
		world.close()
		get_tree().quit(0)


func _bench_frame(delta: float) -> void:
	frames += 1
	world.frame(0, 0, 0, 0.0, 0.0, delta)
	cam.position = world.camera_position()
	cam.rotation = world.camera_rotation()
	_update_player_and_hud()
	if frames == 100:
		cs_feet = world.player_feet()
		for i in range(bench_n):
			var r = Rig.new()
			r.setup(char_res)
			r.forced_lod = char_lod
			add_child(r)
			var gx := (i % 8) - 4
			var gz := int(i / 8)
			r.position = cs_feet + Vector3(gx * 1.4, 0, -3.0 - gz * 1.4)
			r.set_lod(maxi(char_lod, 0))
			bench_rigs.append(r)
	if frames > 130:
		bench_times.append(delta * 1000.0)
		for r in bench_rigs:
			r.update_state(r.position, 0.0, true, delta)
			r.speed = 3.0
			r.phase += delta * 8.0
	if frames == 330:
		var sum := 0.0
		var worst := 0.0
		for t in bench_times:
			sum += t
			worst = maxf(worst, t)
		var avg: float = sum / bench_times.size()
		var draws := RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_TOTAL_DRAW_CALLS_IN_FRAME)
		var prims := RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_TOTAL_PRIMITIVES_IN_FRAME)
		var objs := RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_TOTAL_OBJECTS_IN_FRAME)
		var vmem := RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_VIDEO_MEM_USED)
		var t0 := Time.get_ticks_usec()
		for k in range(30):
			for r in bench_rigs:
				r.set_lod(k % 3)
		var sw: float = (Time.get_ticks_usec() - t0) / 1000.0 / maxf(30.0 * bench_rigs.size(), 1.0)
		print("CUBE bench: instances=%d(+player) lod=%d avg_frame_ms=%.2f worst_ms=%.2f fps=%.0f draw_calls=%d primitives=%d objects=%d vram_bytes=%d lod_switch_ms_per_instance=%.4f" % [
			bench_n, char_lod, avg, worst, 1000.0 / avg, draws, prims, objs, vmem, sw])
		world.close()
		get_tree().quit(0)
