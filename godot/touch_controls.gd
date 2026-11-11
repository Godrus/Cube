extends Control
## Touch input for phones and tablets (glue only, like the rest of the GDScript here): it turns
## fingers into the same intents the keyboard and mouse give (move, look, jump and the action
## keys). The world rules stay in Rust; nothing here decides anything about the game.
##
## Layout (landscape): the left side is a floating joystick (it appears where the thumb lands),
## the rest of the screen is a look pad (drag to turn, a short tap mines the block under the
## crosshair), and a cluster of large round buttons sits at the bottom right. Every finger is
## tracked by its own index, so moving, looking and pressing buttons work together.
##
## Canon points: touch targets are at least about 11 mm / 48 dp (sized from the shorter screen
## side, then clamped), the layout keeps out of the display safe area (notches, rounded
## corners), the joystick has a dead zone and the movement is quantised to the 8 directions the
## Rust controls accept (the controls API is digital: -1, 0, 1 per axis), a light vibration
## confirms a button press, and the overlay is drawn semi-transparent so the world stays visible.

## Radians to turn (already in the sign of the mouse path: add it to the look vector).
signal look_moved(delta: Vector2)
## A button or a tap asked for an action: "mine", "place", "attack", "craft", "interact",
## "inventory", "view", "fly".
signal action(name: String)

const DEAD_ZONE := 0.28
const AXIS_THRESHOLD := 0.38
const LOOK_SENSITIVITY := 0.0045  # radians per pixel of a 1280 px wide screen
const TAP_MAX_SECONDS := 0.25
const TAP_MAX_MOVE := 14.0
const MIN_BUTTON_RADIUS := 34.0

## Movement as the joystick gives it: x to the right, y forward, each -1..1 (before quantising).
var move := Vector2.ZERO
## The jump / ascend button is held.
var jump_held := false

var _joy_id := -1
var _joy_origin := Vector2.ZERO
var _joy_pos := Vector2.ZERO
var _look_id := -1
var _look_start := Vector2.ZERO
var _look_time := 0.0
var _look_travel := 0.0
var _button_ids := {}  # finger index -> button name
var _buttons := []  # [{name, label, center, radius}]
var _joy_radius := 80.0
var _scale := 1.0


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	get_viewport().size_changed.connect(_layout)
	_layout()


## The 8-way direction (-1, 0, 1 per axis) of a joystick vector: nothing inside the dead zone.
static func quantize(v: Vector2) -> Vector2i:
	if v.length() < DEAD_ZONE:
		return Vector2i.ZERO
	return Vector2i(
		(1 if v.x > AXIS_THRESHOLD else (-1 if v.x < -AXIS_THRESHOLD else 0)),
		(1 if v.y > AXIS_THRESHOLD else (-1 if v.y < -AXIS_THRESHOLD else 0)))


## The joystick vector (length 0..1) of a drag from the origin, with the radius of full tilt.
static func joystick_vector(drag: Vector2, radius: float) -> Vector2:
	if radius <= 0.0:
		return Vector2.ZERO
	var v := drag / radius
	return v.limit_length(1.0)


## A tap is a short touch that hardly moved.
static func is_tap(travel: float, seconds: float) -> bool:
	return travel <= TAP_MAX_MOVE and seconds <= TAP_MAX_SECONDS


## The part of the viewport that is safe to draw buttons in (not under a notch or the corners).
func _safe_rect() -> Rect2:
	var vp := get_viewport().get_visible_rect()
	var area := DisplayServer.get_display_safe_area()
	var screen := DisplayServer.screen_get_size()
	if area.size.x <= 0 or screen.x <= 0 or screen.y <= 0:
		return vp
	var sx := vp.size.x / float(screen.x)
	var sy := vp.size.y / float(screen.y)
	var r := Rect2(Vector2(area.position) * Vector2(sx, sy), Vector2(area.size) * Vector2(sx, sy))
	return r.intersection(vp) if r.has_area() else vp


func _layout() -> void:
	var safe := _safe_rect()
	var short_side := minf(safe.size.x, safe.size.y)
	var radius := maxf(MIN_BUTTON_RADIUS, short_side * 0.075)
	_scale = radius / 48.0
	_joy_radius = maxf(60.0, short_side * 0.16)
	var gap := radius * 0.35
	var right := safe.end.x - radius - gap
	var bottom := safe.end.y - radius - gap - radius * 0.2
	_buttons = [
		{"name": "jump", "label": "JUMP", "center": Vector2(right, bottom), "radius": radius * 1.25},
		{"name": "attack", "label": "HIT", "center": Vector2(right - radius * 2.7, bottom + radius * 0.2), "radius": radius},
		{"name": "place", "label": "PLACE", "center": Vector2(right - radius * 0.3, bottom - radius * 2.5), "radius": radius * 0.9},
		{"name": "interact", "label": "USE", "center": Vector2(right - radius * 2.6, bottom - radius * 2.2), "radius": radius * 0.8},
		{"name": "craft", "label": "CRAFT", "center": Vector2(right - radius * 4.4, bottom - radius * 1.0), "radius": radius * 0.7},
		{"name": "inventory", "label": "BAG", "center": Vector2(safe.end.x - radius * 1.0 - gap, safe.position.y + radius * 1.0 + gap), "radius": radius * 0.7},
		{"name": "view", "label": "VIEW", "center": Vector2(safe.end.x - radius * 2.6 - gap, safe.position.y + radius * 1.0 + gap), "radius": radius * 0.7},
		{"name": "fly", "label": "FLY", "center": Vector2(safe.end.x - radius * 4.2 - gap, safe.position.y + radius * 1.0 + gap), "radius": radius * 0.7},
	]
	queue_redraw()


func _button_at(p: Vector2) -> String:
	var best := ""
	var best_d := INF
	for b in _buttons:
		var d: float = p.distance_to(b["center"])
		# A small extra margin: a thumb is not a pixel.
		if d <= b["radius"] * 1.12 and d < best_d:
			best = b["name"]
			best_d = d
	return best


func _input(event: InputEvent) -> void:
	if not visible:
		return
	if event is InputEventScreenTouch:
		_on_touch(event.index, event.position, event.pressed)
	elif event is InputEventScreenDrag:
		_on_drag(event.index, event.position, event.relative)


func _on_touch(index: int, pos: Vector2, pressed: bool) -> void:
	if pressed:
		var name := _button_at(pos)
		if name != "":
			_button_ids[index] = name
			if name == "jump":
				jump_held = true
			else:
				action.emit(name)
			Input.vibrate_handheld(18)
			queue_redraw()
			return
		var safe := _safe_rect()
		var left_zone := pos.x < safe.position.x + safe.size.x * 0.42 and pos.y > safe.position.y + safe.size.y * 0.25
		if left_zone and _joy_id == -1:
			_joy_id = index
			_joy_origin = pos
			_joy_pos = pos
			move = Vector2.ZERO
		elif _look_id == -1:
			_look_id = index
			_look_start = pos
			_look_time = Time.get_ticks_msec() / 1000.0
			_look_travel = 0.0
		queue_redraw()
	else:
		if _button_ids.has(index):
			if _button_ids[index] == "jump":
				jump_held = false
			_button_ids.erase(index)
		elif index == _joy_id:
			_joy_id = -1
			move = Vector2.ZERO
		elif index == _look_id:
			var secs := Time.get_ticks_msec() / 1000.0 - _look_time
			if is_tap(_look_travel, secs):
				action.emit("mine")
			_look_id = -1
		queue_redraw()


func _on_drag(index: int, pos: Vector2, relative: Vector2) -> void:
	if index == _joy_id:
		_joy_pos = pos
		var v := joystick_vector(pos - _joy_origin, _joy_radius)
		# Screen y grows downwards; forward is up.
		move = Vector2(v.x, -v.y)
		queue_redraw()
	elif index == _look_id:
		_look_travel = maxf(_look_travel, pos.distance_to(_look_start))
		var width := get_viewport().get_visible_rect().size.x if is_inside_tree() else 1280.0
		var k := LOOK_SENSITIVITY * (1280.0 / maxf(width, 1.0))
		look_moved.emit(Vector2(-relative.x, -relative.y) * k)


func _draw() -> void:
	var line := Color(1, 1, 1, 0.55)
	var fill := Color(0.1, 0.12, 0.16, 0.38)
	var font := ThemeDB.fallback_font
	for b in _buttons:
		var held: bool = _button_ids.values().has(b["name"])
		draw_circle(b["center"], b["radius"], Color(0.9, 0.8, 0.3, 0.5) if held else fill)
		draw_arc(b["center"], b["radius"], 0.0, TAU, 40, line, 2.0)
		var size := int(clampf(b["radius"] * 0.42, 11.0, 28.0))
		var w := font.get_string_size(b["label"], HORIZONTAL_ALIGNMENT_LEFT, -1, size).x
		draw_string(font, b["center"] + Vector2(-w * 0.5, size * 0.35), b["label"], HORIZONTAL_ALIGNMENT_LEFT, -1, size, Color(1, 1, 1, 0.85))
	if _joy_id != -1:
		draw_circle(_joy_origin, _joy_radius, fill)
		draw_arc(_joy_origin, _joy_radius, 0.0, TAU, 48, line, 2.0)
		draw_circle(_joy_origin + (_joy_pos - _joy_origin).limit_length(_joy_radius), _joy_radius * 0.38, Color(1, 1, 1, 0.45))
	else:
		# The resting hint: where a thumb can start moving.
		var safe := _safe_rect()
		var c := Vector2(safe.position.x + safe.size.x * 0.17, safe.end.y - safe.size.y * 0.28)
		draw_arc(c, _joy_radius * 0.8, 0.0, TAU, 40, Color(1, 1, 1, 0.18), 2.0)


## A check of the pure logic and of the event handling with synthetic touches. Returns the list
## of failures (empty: all passed). Run with `godot --headless --path godot -- --touch-test`.
static func self_test() -> Array:
	var bad := []
	if quantize(Vector2(0.1, 0.1)) != Vector2i.ZERO:
		bad.append("dead zone")
	if quantize(Vector2(0.0, 1.0)) != Vector2i(0, 1):
		bad.append("forward")
	if quantize(Vector2(-1.0, -1.0)) != Vector2i(-1, -1):
		bad.append("diagonal back left")
	if quantize(Vector2(0.9, 0.2)) != Vector2i(1, 0):
		bad.append("a small cross axis is dropped")
	if joystick_vector(Vector2(300, 0), 100.0).length() > 1.0001:
		bad.append("joystick vector is clamped")
	if joystick_vector(Vector2(30, 0), 100.0).x < 0.29 or joystick_vector(Vector2(30, 0), 100.0).x > 0.31:
		bad.append("joystick vector is linear")
	if not is_tap(5.0, 0.1) or is_tap(40.0, 0.1) or is_tap(5.0, 0.9):
		bad.append("tap classification")
	# Events on a node of a fixed layout.
	var t = (load("res://touch_controls.gd") as GDScript).new()
	t._buttons = [{"name": "jump", "label": "JUMP", "center": Vector2(1000, 600), "radius": 60.0},
		{"name": "attack", "label": "HIT", "center": Vector2(850, 620), "radius": 50.0}]
	t._joy_radius = 100.0
	var seen := []
	t.action.connect(func(n): seen.append(n))
	var looks := []
	t.look_moved.connect(func(d): looks.append(d))
	t._on_touch(1, Vector2(1000, 600), true)
	if not t.jump_held:
		bad.append("jump held while the finger is down")
	t._on_touch(2, Vector2(850, 620), true)
	if seen != ["attack"]:
		bad.append("attack press: %s" % [seen])
	t._on_touch(1, Vector2(1000, 600), false)
	if t.jump_held:
		bad.append("jump released")
	# The joystick (the point is in the left zone only when a real viewport exists; set it directly).
	t._joy_id = 7
	t._joy_origin = Vector2(200, 500)
	t._on_drag(7, Vector2(200, 400), Vector2(0, -100))
	if quantize(t.move) != Vector2i(0, 1):
		bad.append("dragging up moves forward: %s" % [t.move])
	t._on_touch(7, Vector2(200, 400), false)
	if t.move != Vector2.ZERO:
		bad.append("joystick released")
	# Look: a drag turns, a tap mines.
	t._look_id = 9
	t._look_start = Vector2(600, 300)
	t._on_drag(9, Vector2(640, 300), Vector2(40, 0))
	if looks.size() != 1 or looks[0].x >= 0.0:
		bad.append("drag right turns right (negative x in the look sign): %s" % [looks])
	t._on_touch(9, Vector2(640, 300), false)
	if seen.has("mine"):
		bad.append("a drag is not a tap")
	t._look_id = 10
	t._look_start = Vector2(600, 300)
	t._look_time = Time.get_ticks_msec() / 1000.0
	t._look_travel = 2.0
	t._on_touch(10, Vector2(601, 300), false)
	if not seen.has("mine"):
		bad.append("a short still touch mines")
	t.free()
	return bad
