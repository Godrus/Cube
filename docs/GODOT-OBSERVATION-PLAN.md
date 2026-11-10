# Plan: read-only observation of physics and camera in Godot (PLAN, NOT IMPLEMENTED)

Status: proposal for Visual QA. Nothing below exists yet except what is marked "exists". It changes no authoritative Core code and no gameplay rule; every query is read-only and only active with the QA scenario option of `godot/main.gd`.

## What exists today (implemented, tested)

* The walking player's collision is **Rust** (`cube-client/src/walk.rs`): an axis-aligned body swept through the voxel grid against the derived faces the client already holds. Rust tests prove: lands on the floor, jump height, cannot enter a wall, slides along it, steps up one voxel but not two, head stops at a ceiling, never ends inside a solid voxel, third-person camera boom stops in front of a wall (10 tests), and the session tests walk on generated terrain.
* `godot/main.gd` (QA scenario mode) reports camera position/rotation, `walking`, `grounded` and `feet` per event, and `tools/visual-qa/walk_smoke.py` drives the real Godot scene: fall and land, walk, jump, third-person boom, dig, far walk (streaming), quit and continue.
* So "the camera does not enter geometry in walking mode" is covered in Rust by construction (the eye is inside the body box, the boom is clamped). What is **not** observed from Godot yet: that the camera position the engine shows is outside every solid voxel, and that the engine-side mesh agrees with the collision data.

## Observations that need no new Godot physics (cheapest, recommended first)

Three read-only `#[func]` on `CubeWorld` (adapter, about 60 lines), all answered from the data gameplay already uses:

1. `qa_ray(origin, direction, max) -> Dictionary`: first solid face hit (voxel, entry distance, material) via `cube_client::pick_distance` on the near-player face index. Replaces "raycast" for the scene's real collision data.
2. `qa_solid_at(voxel) -> bool`: `FaceIndex::is_solid` (a voxel with a visible face; interior voxels report false, documented).
3. `qa_body_overlap(feet) -> bool` and `qa_camera_inside_solid() -> bool`: `Walker::collides_at` for the body, and the voxel containing the camera position for the camera.

Checks this gives (as Visual QA scenario steps, all provable, not screenshots): camera never inside a solid voxel over a scripted walk; body never overlaps; a ray straight down from the feet hits at the expected height; after an edit the ray result changes; the first ray along the view agrees with the block the edit would remove. Cost: small, no engine physics, no new dependency. Needs: an `observe` step in the scenario runner and telemetry fields.

## Observations through Godot physics (only if a mesh-versus-collision cross-check is wanted)

Godot 4.2.2 can answer the same questions on the **rendered mesh** instead of the Rust data, which is a different check (do the two agree?):

| Need | Godot facility | What it requires |
|---|---|---|
| Ray hits | `PhysicsDirectSpaceState3D.intersect_ray(PhysicsRayQueryParameters3D)` | a `StaticBody3D` with a `CollisionShape3D` per brick (`ArrayMesh.create_trimesh_shape()`), created only in QA mode; many bricks mean many shapes (about 80 in the current scene) |
| Camera inside geometry | `intersect_point` or `intersect_shape` with a small `SphereShape3D` (near-plane radius) at the camera | same bodies |
| Body overlap | `PhysicsDirectSpaceState3D.collide_shape` with the body box | same bodies |
| State of obstacles | `get_rest_info`, or listing the bodies the shape touches | same bodies |
| Frame-synchronous queries | must run in `_physics_process`; the direct space state is only valid there | a physics-step hook in the scenario runner (today scenarios run in `_process`) |

Risks: a second collision representation must never be used by gameplay (it would duplicate Rust logic); trimesh shapes are rebuilt at every brick update (cost, and a one-frame lag against the Rust data); `RayCast3D` nodes add nothing over `intersect_ray`. If this is built, the QA bodies are created and freed by the QA hook, not by the adapter, so the normal game has none.

## Recommended order

1. The three Rust-backed observation functions and an `observe` scenario step (camera inside solid, body overlap, ray down, ray along the view) with generated scenarios `camera_in_solid`, `walk_into_wall`, `ceiling_and_jump` as real tests of the Rust collision from inside Godot. This turns the "collision" rows of Visual QA from UNSUPPORTED into supported ones.
2. Only then, if a rendering-versus-collision mismatch is ever suspected, the trimesh cross-check.
3. Keep Visual QA's TypeSafe layer out of collision entirely: a screenshot cannot prove penetration.
