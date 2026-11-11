# Cube first world in Godot 4

The scene `main.tscn` shows a world that the Rust library generates, edits and saves. GDScript here is UI and glue only (ADP-02); every voxel, every face and every edit is in Rust.

## Run

```bash
cargo build -p cube-godot-adapter
godot --path godot
```

Click to capture the mouse. You start as a **walking player**: you fall onto the ground, W A S D walk, the mouse looks, Space jumps (one-voxel steps are climbed automatically; the body is 0.6 wide and 1.8 tall), V switches between first and third person (the third-person camera stops in front of walls), F switches between walking and free flight (Space up, Shift down), Esc releases the mouse.

**Building and items (prototype rules):** the left button **mines** the block under the crosshair (within 6 voxels; in survival it goes into your inventory), the right button **places** the selected hotbar block (in survival one item is used up; a block is never placed into your own body). Keys 1-9 or the mouse wheel choose the hotbar slot, I opens the inventory (click two slots to swap them), C switches between survival and creative (creative offers the five materials without limit). The block under the crosshair is outlined and a ghost shows where the selected block would go (red when it cannot be placed). The top-left text shows the objective: mine 8 blocks and place 12.

The world is generated as you move and far bricks are unloaded (`--fly` starts in flight). The world, your edits, your position, inventory and objective progress are saved when you close the window and loaded on the next start (`user://world`, or `--world <dir>`): you continue where you left off.

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

Not yet a full game: no health, creatures, crafting, item drops or tools, one hard-coded objective, no textures (vertex colours by material), no player model (a plain box in third person), no sound. The world is finite (48 bricks, about 770 voxels, from the origin in every horizontal direction); bricks you edited are never unloaded. Terrain is only walkable where slopes are one voxel per step or you jump; steep hills stop you. The Godot run is not part of CI; `python tools/visual-qa/walk_smoke.py` drives the real scene (fall, walk, jump, third person, dig, walk 130 voxels, quit, continue); `--journey` walks 150 voxels out and back and checks that resident bricks and memory stay flat; `--play` mines, builds, uses the hotbar and inventory, completes the objective and restarts.

## Combat (prototype, internal)

`godot --path godot -- --creatures` switches creatures on: slimes spawn near you, chase and hurt you, `E` swings the sword. Scripted check in real Godot: `python tools/visual-qa/walk_smoke.py --combat`. A fresh checkout or git worktree needs one import run first (`godot --headless --path godot --import`); the smoke runner does it by itself. Without that cache the scene shows a frozen picture because the native extension is not loaded.


## Android (early spike, not a supported platform yet)

Status (2026-11-11): the x86_64 build was exported and **ran in the Android Studio emulator** (Android 16, Vulkan Forward Mobile): the native Rust core loaded, the world was created in `user://world`, terrain, HUD and hotbar drew (`docs/screenshots/android/`). Not run on a physical phone, no arm64 build was run, no touch controls, performance not measured. The Rust half builds the same GDExtension for Android; Godot packs it into an APK.

```bash
rustup target add aarch64-linux-android x86_64-linux-android      # once
ANDROID_NDK_HOME=<sdk>/ndk/28.1.13356709 scripts/build-android.sh x86_64 arm64
godot --headless --path godot --export-debug "Android x86_64 (emulator)" build/cube-x86_64.apk
```

Needs: the Android SDK and NDK (r27 or newer), Godot export templates for the same Godot version, the Android debug keystore in the Godot editor settings. The mobile renderer is `mobile` (Vulkan, `project.godot`). The compatibility (GLES3) renderer was tried first and failed in the emulator (both of its GPU modes): shaders did not link, "active uniforms exceed GL_MAX_FRAGMENT_UNIFORM_VECTORS (261)", grey screen; whether real phones behave the same is unknown. The emulator needs `-gpu swiftshader_indirect` or `auto`; `-gpu host` did not boot. The world is stored in `user://world` (no command line on a phone). There are no touch controls yet (mouse and keyboard only), so a phone cannot play it; that is M9 work.
