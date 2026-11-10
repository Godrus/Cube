# Cube first world in Godot 4

The scene `main.tscn` shows a world that the Rust library generates, edits and saves. GDScript here is UI and glue only (ADP-02); every voxel, every face and every edit is in Rust.

## Run

```bash
cargo build -p cube-godot-adapter
godot --path godot
```

Click to capture the mouse. W A S D move, Space and Shift go up and down, the mouse looks, the left button removes the block under the crosshair, the right button places one, Esc releases the mouse. The world is stored in Godot's user data folder (`user://world`): the second launch loads it and your edits are back.

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

A game: there is no character, inventory or gameplay, no streaming (the world is a static bounded area of 11 x 11 x 11 bricks around the origin), no textures (vertex colours by material) and the Godot run is not part of CI.
