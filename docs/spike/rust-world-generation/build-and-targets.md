# Build and targets: world-generation spike

Branch `spike/rust-world-generation`. Extends `../rust-godot/build-and-targets.md` (same toolchain and machine: Windows 11, Rust host `x86_64-pc-windows-gnu`, Godot 4.7.2, Android SDK with NDK 28.1.13356709). Results: `report.md`.

## Status of the targets

| Target | State |
|---|---|
| Windows x86_64 desktop | Built, tested, smoke-tested in Godot (headless and windowed) |
| Android x86_64 emulator (native on the emulator) | Built, exported, ran: full smoke, real `adb input tap` touches, a 10-minute soak loop (report sections 37 and 38) |
| Android ARM64 on an emulator (x86_64 host, instruction translation) | Built, exported, ran (final APKs: test 3 of 3 for faces and 3 of 3 for greedy, client 3 of 3 each, no crash); **emulator only, not a device result** |
| **Android ARM64 physical device** | **Blocked: no physical ARM64 device.** `adb devices` lists no device on the development machine. Nothing in this branch claims device results. The APKs and the command to run on a device are below |
| **iOS ARM64** | **iOS validation: blocked — no macOS/Xcode.** No iOS build exists and none is claimed |

## Build

```bash
cargo test --workspace                                   # 131 tests
cargo clippy --workspace --all-targets
cargo build --release -p cube-godot-adapter              # Windows DLL used by the desktop client (target/release)
ANDROID_NDK_HOME="$LOCALAPPDATA/Android/Sdk/ndk/28.1.13356709" scripts/build-android.sh arm64 x86_64   # godot-client/bin/android/<abi>/
```

Export (Godot 4.7.2 with the Android export templates; the Android debug keystore is enough for the spike, credentials are passed through the environment and not stored):

```bash
export GODOT_ANDROID_KEYSTORE_RELEASE_PATH=~/.android/debug.keystore GODOT_ANDROID_KEYSTORE_RELEASE_USER=androiddebugkey GODOT_ANDROID_KEYSTORE_RELEASE_PASSWORD=android
cd godot-client
godot --headless --path . --export-release "Test arm64 faces" build/test-faces-arm64.apk
```

Export presets (`godot-client/export_presets.cfg`, preset 0 and 1 are the older UI-test presets). All world presets: package `org.cube.spike.world`, release, scripts exported as text so the packages can be inspected, 16 KB aligned libraries:

| Preset name | Custom features | Starts | Package contents |
|---|---|---|---|
| `Test <abi> <mode>` | `worldtest,<mode>` | `world_test.tscn` (scripted smoke test, then stays alive) | client + test scripts |
| `Soak <abi> <mode>` | `worldtest,soak,<mode>` | the same, plus a 10-minute soak loop | client + test scripts |
| `Client <abi> <mode>` | `worldclient,<mode>` | `world.tscn` only | client only: `world_smoke.gd` and `world_test.tscn` are excluded |

`<abi>` = `arm64` or `x86_64`, `<mode>` = `cubes`, `faces`, `greedy` (client and soak: `faces`, `greedy`). No preset starts a child process.

## Choosing the renderer without child processes

`world.gd` (the runtime client) picks the renderer in this order, using only what Godot already has: (1) the Godot user argument `--renderer=<cubes|faces|greedy>` (desktop: `godot --path godot-client world.tscn -- --renderer=greedy`), (2) the export preset's custom feature `greedy` / `cubes` / `faces`, (3) the default `faces`. The rebuild policy is `--rebuild=<partial|chunk|all>` (default `partial`). The Renderer button switches modes at run time. There is no shell, no `getprop`, no system property. On Android the mode is therefore fixed by the preset (a different APK per mode).

## Desktop

```bash
godot --path godot-client world.tscn                                       # the client, default renderer faces
godot --headless --path godot-client world_test.tscn -- --world-smoke --renderer=greedy   # scripted test, exit code 0/1
godot --path godot-client world_test.tscn -- --world-smoke --renderer=greedy --shot=<dir> # with a GPU: pictures, faces-vs-greedy comparison
godot --path godot-client world_test.tscn -- --world-smoke --renderer=faces --soak-seconds=60
cargo run --release -p cube-godot-adapter --example render_bench
```

## The "no child process" guard

* `crates/cube-godot-adapter/tests/no_child_processes.rs` (part of `cargo test`) fails if any `.gd` file uses `OS.execute`, `OS.create_process`, `OS.create_instance`, `OS.shell_open`, `getprop` (comments are ignored), if `world.gd` contains test hooks, or if a client preset would ship the test scripts.
* `python scripts/check-apk-no-exec.py godot-client/build/*.apk` inspects exported packages (scripts are exported as text): no packaged script may contain `execute` / `getprop` / `create_process` / `shell_open`, and a `client-*` APK must not contain `world_smoke` or `world_test`. It was verified to fail on a package built from a `world.gd` with an `OS.execute` call added.

(The engine library and the Rust binding library contain the *names* of OS API methods; only GDScript calls matter.)

## Running on a physical Android ARM64 phone (owner)

1. Enable USB debugging, connect one phone, check `adb devices` shows exactly one `device`.
2. Build the APKs as above (or use the ones in `godot-client/build/`, not committed; SHA-256 of the files tested in the last emulator run, ARM64 builds):
   ```
   b5c3fb9a92f31b54c5704ddd211c8a67c7616ad28b7943ff4d56dec6725fb867  client-faces-arm64.apk
   976be067c7e41b271023998be63508324d035f1a4412772670cecc4a65a737a8  client-greedy-arm64.apk
   f3ec3188ba13a221c3054a7861d76494f5b34dfd7774b7439ac0dd34bd0e693e  test-faces-arm64.apk
   feab52db61aea116821e0ee4403da5f52fa3478d8ec2982fdcd26685cdfbc212  test-greedy-arm64.apk
   298fdad22bb73d5c44ed18623b3cc06df42db6fd4586081d51f5ad7401bb7326  test-cubes-arm64.apk
   c5f37fce47eb2ff88f5248700020f0ecade7b39c7c59266c7b3c99548e6fd9dc  soak-faces-arm64.apk
   9588be083cd185ba4bdbdc10ac9866834905e3e08c19358b8eb10f2ef04cb589  soak-greedy-arm64.apk
   ```
3. Run the automatic procedure (3 launches per mode with the scripted Fill/Edit/Undo/Redo/Jump/Save/Load test, real `adb input tap` touches, a 10-minute soak per mode, memory, battery temperature and thermal status samples, logcat, crash/ANR detection):
   ```bash
   ADB=/path/to/adb scripts/device-validation.sh --runs 3 --modes "faces greedy"
   ```
   It refuses to run (exit 2, `Blocked: no physical ARM64 device`) when no physical ARM64 device is attached; `--allow-emulator` exists only to test the script itself and labels the output `EMULATOR - NOT A DEVICE RESULT`. Output: `device-results-<date>/summary.md` plus raw logcat files and `soak-<mode>.samples.csv`.
4. Manual checks to add by hand: install `client-faces-arm64.apk` and `client-greedy-arm64.apk`, drag/touch the on-screen buttons (Fwd/Back/Left/Right are hold buttons), watch for visual differences between modes (the Renderer button cycles modes), and let the soak APKs run with the screen on and the phone not on a charger for thermal behaviour. Useful commands: `adb shell dumpsys thermalservice`, `adb shell dumpsys battery`, `adb shell dumpsys meminfo org.cube.spike.world`, `adb logcat -d | grep -E "godot|Fatal signal|ANR"`.
5. Send back `summary.md` and the raw files. Until then the status stays **Blocked: no physical ARM64 device**.

## iOS

**iOS validation: blocked — no macOS/Xcode.** Rust target `aarch64-apple-ios` and a Godot iOS export need macOS with Xcode; neither is available. No iOS commands were run and no iOS result exists.

## Notes for the Windows/Git Bash host

`adb` arguments that look like POSIX paths are rewritten by MSYS; export `MSYS_NO_PATHCONV=1` before calling `adb push`/`shell` with `/data/...` paths. Stopping the emulator: `adb emu kill` followed by `taskkill //F //IM qemu-system-x86_64.exe` / `emulator.exe` if the window stays open (`pkill` does not exist in Git Bash).
