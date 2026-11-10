# Cube first world in Godot 4

The scene `main.tscn` shows a world that the Rust library generates, edits and saves. GDScript here is UI and glue only (ADP-02); every voxel, every face and every edit is in Rust.

## Run

```bash
cargo build -p cube-godot-adapter
godot --path godot
```

Click to capture the mouse. You start as a **walking player**: you fall onto the ground, W A S D walk, the mouse looks, Space jumps (one-voxel steps are climbed automatically; a body is 0.6 wide and 1.8 tall), the left button removes the block under the crosshair, the right button places the chosen block (keys 1 to 5 choose it; a block is never placed into your own body), V switches between first and third person (the third-person camera stops in front of walls), F switches between walking and free flight (Space up, Shift down), Esc releases the mouse. The world around you is generated as you move (`--fly` starts in flight). The world, your edits and your position are saved when you close the window and loaded on the next start (`user://world`, or `--world <dir>`): you continue where you left off.

Tested with Godot 4.2.2 (the binding is built for API 4.2; later 4.x versions load it too). The library path in `cube.gdextension` is `../target/debug/...`; build in release mode and change the path for a release build.

## Self check (no window)

```bash
cargo build -p cube-godot-adapter
godot --headless --path godot -- --smoke --world <empty directory>
```

Prints `CUBE smoke: ...` lines and exits 0 when the native library loaded, geometry reached the scene and an edit was applied. In headless mode Godot's dummy renderer logs `mesh_get_surface_count` errors for every mesh; they are a property of the dummy renderer, not of the world.

## Picture

```bash
godot --path godot --rendering-driver opengl3 -- --screenshot <file.png> --world <empty directory>
```

Saves one frame and exits. `docs/screenshots/first-world.png` was made this way.

## What this is not

Not yet a game: no inventory, health, creatures or goals, no textures (vertex colours by material), no player model (a plain box in third person), no sound. The world is finite (48 bricks, about 770 voxels, from the origin in every horizontal direction) and bricks that were generated are never unloaded. Terrain is only walkable where slopes are one voxel per step or you jump; steep hills stop you. The Godot run is not part of CI; `python tools/visual-qa/walk_smoke.py` drives the real scene (fall, walk, jump, third person, dig, walk 130 voxels, quit, continue).
