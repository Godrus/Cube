class_name CubeCharacterRig
extends Node3D
## Visual rig of the voxel traveller (CHAR-01). Glue only: the geometry comes from the Rust
## CubeCharacter, the position and facing come from the authoritative player state (the caller
## passes them in). Animation rotates part nodes; no geometry is rebuilt after setup().
## Model space: feet at the node origin, 1 unit = 1 world voxel, front = -Z.

const LOD_DISTANCES := [7.0, 18.0]  # metres: LOD0 up to 7, LOD1 up to 18, LOD2 beyond
const SWORD_REST := -2.6  # blade down and a little forward (radians about X)
const SWORD_SPLAY := 0.32  # blade turned outward so it reads from the front

# One asset and one set of meshes is shared by every rig instance (same resolution): instances
# only own nodes, never geometry.
static var _shared_asset: CubeCharacter
static var _shared_meshes: Array = []
static var _shared_res := -1
static var _shared_material: StandardMaterial3D

var asset: CubeCharacter
var part_nodes: Array[MeshInstance3D] = []
var part_names: Array[String] = []
var part_pivots: Array[Vector3] = []
var lod_meshes: Array = []  # [part][lod] -> ArrayMesh
var material: StandardMaterial3D
var lod := 0
var forced_lod := -1
var phase := 0.0
var speed := 0.0
var _last_pos := Vector3.ZERO
var _have_last := false
var _vy := 0.0
var attack_t := -1.0
var hit_t := -1.0
var grounded := true
var time := 0.0
var cape_angle := 0.0
var setup_ms := 0.0
var fill: DirectionalLight3D


func setup(res: int = 0) -> bool:
	var t0 := Time.get_ticks_usec()
	if _shared_asset == null or _shared_res != res:
		var a := CubeCharacter.new()
		if not a.build(res):
			return false
		_shared_asset = a
		_shared_res = res
		_shared_meshes = []
		for i in range(a.part_count()):
			_shared_meshes.append([a.part_mesh(i, 0), a.part_mesh(i, 1), a.part_mesh(i, 2)])
	asset = _shared_asset
	# the material is per instance (hit tint), the meshes are shared
	material = StandardMaterial3D.new()
	material.vertex_color_use_as_albedo = true
	material.vertex_color_is_srgb = true
	material.roughness = 0.92
	var n := asset.part_count()
	for i in range(n):
		part_names.append(asset.part_name(i))
		part_pivots.append(asset.part_pivot(i))
		lod_meshes.append(_shared_meshes[i])
		var node := MeshInstance3D.new()
		node.name = part_names[i]
		node.material_override = material
		node.mesh = lod_meshes[i][0]
		node.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_ON
		node.layers = 1 | 2  # layer 2 is lit by the rig's own fill light
		part_nodes.append(node)
	for i in range(n):
		var parent := asset.part_parent(i)
		if parent < 0:
			add_child(part_nodes[i])
			part_nodes[i].position = part_pivots[i]
		else:
			part_nodes[parent].add_child(part_nodes[i])
			part_nodes[i].position = part_pivots[i] - part_pivots[parent]
	# Fill light: affects only this rig (cull mask = layer 2) so the character reads in shade
	# without changing the world's lighting; it is turned to come from the camera side.
	fill = DirectionalLight3D.new()
	fill.light_cull_mask = 2
	fill.light_energy = 0.22
	fill.shadow_enabled = false
	fill.top_level = true
	add_child(fill)
	setup_ms = (Time.get_ticks_usec() - t0) / 1000.0
	_pose_rest()
	return true


func part(name: String) -> MeshInstance3D:
	return part_nodes[part_names.find(name)]


func set_lod(l: int) -> void:
	l = clampi(l, 0, 2)
	if l == lod and part_nodes[0].mesh == lod_meshes[0][l]:
		return
	lod = l
	for i in range(part_nodes.size()):
		part_nodes[i].mesh = lod_meshes[i][l]


## Picks the LOD from the camera distance unless one is forced (mobile / preview).
func auto_lod(cam_pos: Vector3) -> void:
	var to_rig := global_position + Vector3(0, 1.0, 0) - (cam_pos + Vector3(0, 0.6, 0))
	if to_rig.length() > 0.01 and absf(to_rig.normalized().y) < 0.99:
		fill.global_transform = Transform3D(Basis.looking_at(to_rig.normalized(), Vector3.UP), global_position)
	if forced_lod >= 0:
		set_lod(forced_lod)
		return
	var d := cam_pos.distance_to(global_position + Vector3(0, 0.9, 0))
	set_lod(0 if d < LOD_DISTANCES[0] else (1 if d < LOD_DISTANCES[1] else 2))


func trigger_attack() -> void:
	if attack_t < 0.0:
		attack_t = 0.0


func trigger_hit() -> void:
	hit_t = 0.0


func _pose_rest() -> void:
	for n in part_nodes:
		n.rotation = Vector3.ZERO
	part("sword").rotation = Vector3(SWORD_REST, 0, SWORD_SPLAY)


## Called every frame with the authoritative state: world position of the feet, yaw, grounded.
func update_state(feet: Vector3, yaw: float, is_grounded: bool, delta: float) -> void:
	position = feet
	rotation = Vector3(0, yaw, 0)
	if _have_last and delta > 0.0:
		var d := feet - _last_pos
		_vy = d.y / delta
		var hs := Vector2(d.x, d.z).length() / delta
		speed = lerpf(speed, hs, clampf(delta * 12.0, 0.0, 1.0))
	_last_pos = feet
	_have_last = true
	grounded = is_grounded
	_animate(delta)


func _animate(delta: float) -> void:
	time += delta
	var moving := clampf(speed / 3.0, 0.0, 1.0)
	phase += delta * (4.0 + speed * 1.6) * (1.0 if moving > 0.05 else 0.0)
	var s := sin(phase)
	var c := cos(phase)
	var breathe := sin(time * 2.2)
	var torso := part("torso")
	var head := part("head")
	var arm_l := part("arm_l")
	var arm_r := part("arm_r")
	var leg_l := part("leg_l")
	var leg_r := part("leg_r")
	var cape := part("cape")
	var sword := part("sword")
	# walk / run cycle and idle
	var swing := 0.75 * moving * (1.0 if speed < 6.5 else 1.25)
	leg_l.rotation = Vector3(s * swing, 0, 0)
	leg_r.rotation = Vector3(-s * swing, 0, 0)
	arm_l.rotation = Vector3(-s * swing * 0.7 + 0.03 * breathe, 0, 0.04 + 0.02 * breathe)
	arm_r.rotation = Vector3(s * swing * 0.35 + 0.05, 0, -0.04 - 0.02 * breathe)
	torso.position.y = part_pivots[0].y + absf(c) * 0.03 * moving + 0.004 * breathe
	torso.rotation = Vector3(0.05 * moving, s * 0.08 * moving, 0)
	head.rotation = Vector3(-0.05 * moving + 0.015 * breathe, -s * 0.05 * moving, 0)
	var cape_target := 0.10 + 0.55 * moving + 0.12 * absf(s) * moving
	# jump / fall: arms up, legs split, cape lifted
	if not grounded:
		leg_l.rotation = Vector3(0.55, 0, 0.05)
		leg_r.rotation = Vector3(-0.35, 0, -0.05)
		arm_l.rotation = Vector3(1.05, 0, 0.25)
		arm_r.rotation = Vector3(0.85, 0, -0.25)
		torso.rotation = Vector3(0.0, 0, 0)
		cape_target = 0.9 if _vy < 0.0 else 0.35
	cape_angle = lerpf(cape_angle, cape_target, clampf(delta * 6.0, 0.0, 1.0))
	cape.rotation = Vector3(cape_angle, 0, 0.03 * s * moving)
	sword.rotation = Vector3(SWORD_REST, 0, SWORD_SPLAY)
	# attack: wind-up, overhead slash, recovery (0.45 s); torso twists into the swing
	if attack_t >= 0.0:
		attack_t += delta
		var p := attack_t / 0.45
		if p >= 1.0:
			attack_t = -1.0
		else:
			var a: float
			if p < 0.28:
				a = lerpf(0.0, -0.9, p / 0.28)  # arm goes back
			elif p < 0.55:
				var q := (p - 0.28) / 0.27
				a = lerpf(-0.9, 2.5, q * q * (3.0 - 2.0 * q))  # fast slash up and over
			else:
				a = lerpf(2.5, 0.05, (p - 0.55) / 0.45)  # recover
			arm_r.rotation = Vector3(a, 0, -0.05)
			arm_l.rotation = Vector3(-0.4 * sin(p * PI), 0, 0.2)
			torso.rotation.y = lerpf(0.45, -0.35, clampf(p * 1.6, 0.0, 1.0))
			if p > 0.28 and p < 0.55:
				sword.rotation = Vector3(SWORD_REST + 0.9, 0, 0)
	# hit reaction: flinch back and flash red for 0.25 s
	var tint := Color(0.86, 0.86, 0.86)  # keeps pale cloth and skin from clipping under the sun
	if hit_t >= 0.0:
		hit_t += delta
		if hit_t > 0.25:
			hit_t = -1.0
		else:
			var k := 1.0 - hit_t / 0.25
			torso.rotation.x -= 0.35 * k
			head.rotation.x -= 0.25 * k
			tint = Color(0.86 + 0.14 * k, 0.86 - 0.5 * k, 0.86 - 0.5 * k)
	material.albedo_color = tint
