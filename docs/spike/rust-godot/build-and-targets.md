# Build and Targets (spike)

Measured on the development machine described below. Everything not listed under "Verified" is **not verified**.

## Environment used

| Item | Value |
|---|---|
| OS | Windows 11 Pro 10.0.26200 |
| Rust | 1.97.1, host `x86_64-pc-windows-gnu` (MinGW-w64 from WinLibs); only this target installed |
| Godot | 4.7.2 stable, `D:\Program Files\Godot_v4.7.2-stable_win64.exe\` (the owner supplied the path after the first report) |
| Android SDK/NDK | not installed (`ANDROID_NDK_HOME`/`ANDROID_HOME` unset) |
| macOS / Xcode | not available |
| Free disk on `D:` | about 9.5 GB at start |

## Commands

Everything that does not involve Godot:

```bash
cargo build --release -p cube-server
cargo test -p cube-core -p cube-processing -p cube-editing -p cube-persistence -p cube-server
cargo run -p cube-server -- --selftest        # scripted run, exits 0/1
cargo run -p cube-server -- --listen 127.0.0.1:7878   # headless, loopback, no authentication
```

Godot adapter (needs the `godot` crate from crates.io):

```bash
cargo build -p cube-godot-adapter             # debug dll in target/debug
cargo build --release -p cube-godot-adapter   # release dll in target/release
cargo test -p cube-godot-adapter              # tests the pure-Rust bridge, does not start Godot
```

Run the client (manual, needs Godot 4.2+; `res://` paths in `godot-client/cube.gdextension` point to `../target/...`):

```bash
godot --path godot-client                              # interactive
godot --headless --path godot-client -- --smoke        # headless smoke run: fill, undo, redo, save, load
```

## Targets

| Target | Rust triple | State |
|---|---|---|
| Windows x86_64 (GNU) | `x86_64-pc-windows-gnu` | **Built and tested.** Library exports `gdext_rust_init` (checked with `objdump -p`). **Loads in Godot 4.7.2 and passes the headless smoke run** (see below) |
| Windows x86_64 (MSVC) | `x86_64-pc-windows-msvc` | Not built. Official Godot Windows builds are MSVC-built; a GNU-built GDExtension DLL normally loads but this is unverified |
| Android ARM64 | `aarch64-linux-android` | **Not built, not verified.** Needs: Android NDK, `rustup target add aarch64-linux-android`, a linker configured through `.cargo/config.toml` or `cargo-ndk`, then copy `libcube_godot_adapter.so` into the Godot Android export. The `.gdextension` file already lists the expected path |
| iOS ARM64 | `aarch64-apple-ios` | **Not built, not verified.** Needs macOS with Xcode, `rustup target add aarch64-apple-ios`, and a decision on how the library is packaged for iOS export (dynamic library vs xcframework). The `.gdextension` file lists a placeholder path only |

Not done on purpose: no `.cargo/config.toml` with NDK paths, no `rustup target add`, no downloads of toolchains. Those need the owner's environment or permission.

## Verified in Godot (Windows, 4.7.2)

```bash
godot --headless --path godot-client --import          # extension recognised: "Initialize godot-rust (API v4.6, runtime v4.7.2)"
godot --headless --path godot-client -- --smoke        # exit code 0
```

The smoke run goes through the real GDExtension: fill (32 voxels), undo (0), redo (32), save, clear, load (new epoch 1, 32 voxels). Not checked: the interactive scene, rendering output, buttons, the editor UI, Windows release build inside Godot, export.

## Measured (Windows GNU host)

| Measurement | Value |
|---|---|
| Release build of `cube-server` from scratch (no Godot, clean target dir) | about 4 s |
| Tests of the five non-Godot crates from scratch (debug) | about 5 s |
| Debug build of `cube-godot-adapter` including the `godot` crate (first time) | about 55 s |
| Full release workspace build | about 2 min 7 s |
| `cube-server.exe` (release) | 0.56 MB |
| `cube_godot_adapter.dll` (release) | 14.6 MB |
| `cube_godot_adapter.dll` (debug) | 158.7 MB |
| `target/` after debug + release builds | about 2.7 GB (ignored by git) |

Mobile library sizes and startup times are unknown.

## Export notes (not tested)

`res://../target/...` works only in the editor. For an exported project the native libraries must be copied inside the Godot project and referenced from `cube.gdextension`.
