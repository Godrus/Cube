# Mobile Validation Report (spike)

**Branch:** `spike/rust-godot`. **Scope:** the existing minimal slice only (no new systems; Core ↔ adapter architecture, persistence format, TCP protocol and undo semantics unchanged).

## Summary

| Platform | Build | Ran | Verdict |
|---|---|---|---|
| **Android ARM64** (`aarch64-linux-android`), release | **Built**, APK exported and signed | **Ran only on an emulator** (x86_64 Android 17 emulator using ARM translation). **No physical ARM64 device was available** | Functionally validated on emulator. **Not validated on real ARM64 hardware**; performance and memory numbers are not representative |
| **Android x86_64**, release (supplementary, same code) | Built, APK exported | Ran natively on the emulator | Functionally validated; shows what the emulator numbers look like without translation |
| **iOS ARM64** | **Not built** | Not run | **Blocked: no macOS/Xcode in this environment.** Commands below, untested |

Nothing here is a claim of mobile readiness. The accurate statement is: *the same Rust library compiles for Android ARM64, loads in a Godot 4.7.2 release export, and the whole Fill/Undo/Redo/Save/Load scenario passes on an Android emulator; real-device behaviour is unknown.*

## Environment

| Item | Value |
|---|---|
| Host | Windows 11, Intel Core i5-13600KF, 32 GB RAM, RTX 3070 Ti |
| Rust | 1.97.1; targets installed for this task: `aarch64-linux-android`, `x86_64-linux-android` (official `rustup`) |
| Android | SDK at `%LOCALAPPDATA%\Android\Sdk`, NDK 28.1.13356709 (16 KB page alignment by default), build-tools 36.0.0, API level 24 minimum linked |
| Godot | 4.7.2 stable with its own Android export templates (non-Gradle export, prebuilt template APK) |
| Device | AVD `Medium_Phone`: `sdk_gphone16k_x86_64`, Android 17 (API 37), x86_64 with ARM64 translation (`ro.product.cpu.abilist=x86_64,arm64-v8a`), 2 GB RAM, **16 KB memory page size**, 1080×2400 at 420 dpi, GLES via host GPU translator |
| Signing | The APKs are signed with the **Android debug keystore** that Godot installed on this machine (release export, not a production key). Credentials passed by environment variables at export time; none stored in the repository |

## What was built

```bash
# libraries (needs ANDROID_NDK_HOME; the script sets the NDK linker per target)
ANDROID_NDK_HOME="$LOCALAPPDATA/Android/Sdk/ndk/28.1.13356709" scripts/build-android.sh arm64 x86_64

# APK export (Godot 4.7.2, preset names live in godot-client/export_presets.cfg)
export GODOT_ANDROID_KEYSTORE_RELEASE_PATH=... GODOT_ANDROID_KEYSTORE_RELEASE_USER=... GODOT_ANDROID_KEYSTORE_RELEASE_PASSWORD=...
godot --headless --path godot-client --import
godot --headless --path godot-client --export-release "Android ARM64 Validation"          build/cube-spike-arm64.apk
godot --headless --path godot-client --export-release "Android x86_64 Emulator Validation" build/cube-spike-x86_64.apk
```

| Artifact | Size |
|---|---|
| `libcube_godot_adapter.so` arm64 (release, stripped by Rust release profile) | 3.43 MB |
| `libcube_godot_adapter.so` x86_64 (release) | 3.41 MB |
| `cube-spike-arm64.apk` (Godot engine 71 MB uncompressed inside) | **29.9 MB** |
| `cube-spike-x86_64.apk` | 31.2 MB |

ELF checks (`llvm-readelf`): both libraries export `gdext_rust_init`, depend only on `libc.so`/`libdl.so`, and have LOAD segment alignment `0x4000` (16 KB), which Android 15+ requires on 16 KB devices. Build time for the two libraries from a clean target: about 4 min 9 s.

Export-only changes made for this validation (no change to core logic): `scripts/build-android.sh`, `godot-client/export_presets.cfg` (two validation presets that set the custom feature `autotest`), Android library paths inside `res://bin/` in `cube.gdextension`, `import_etc2_astc=true`, a placeholder `icon.svg`, and an `autotest` hook in `main.gd` that runs the existing UI test at start and keeps the app alive. Libraries and APKs are git-ignored.

## What was checked on Android

Method: install the APK with `adb`, cold-start it (`am start -W` after `force-stop`), the app runs the same 19-check UI scenario used on desktop (it injects mouse events into the real buttons), results are read from `logcat`; then **real touch taps** are sent with `adb input tap` to each button.

| Item | ARM64 release (translated) | x86_64 release (native) |
|---|---|---|
| Install and launch | ok | ok |
| GDExtension loads (`CubeRuntime` class available) | ok | ok |
| Fill, Undo, Redo, Save, Clear, Load by injected click | ok (19/19 checks, 3 runs) | ok (19/19, 2 runs) |
| Display: MultiMesh count equals Rust result (32, 80, 0, 80; 16 384 in the large case), camera framing | ok | ok |
| Real finger-style taps via `adb input tap` | Fill, Save, Clear, Load, Undo, Redo all reached the handlers; Undo/Redo with empty history returned the typed errors `EDIT-0002/0003` (history is dropped on Load by design); after a new Fill, Undo and Redo succeeded | not repeated |
| Crash / `FATAL EXCEPTION` / `SIGSEGV` / `SIGABRT` in logcat | none (all runs, including the touch session) | none |
| Engine errors in the app log | none | none |

Screenshots: `screenshots/mobile-arm64-emulator.png`, `screenshots/mobile-arm64-emulator-touch.png` (emulator, ARM64 APK; 16 384 cubes shown, buttons visible).

## Measurements

**Read this first:** the ARM64 numbers come from an x86_64 emulator that **translates ARM64 code**. That makes the ARM64 start time and memory far worse than on a phone, and the x86_64 numbers (native on the emulator) are not phone numbers either. Neither is a mobile performance result.

| Metric | ARM64 release (translated), 3 cold starts | x86_64 release (native on emulator), 2 cold starts |
|---|---|---|
| Time to launched activity (`am start -W` TotalTime) | 2.84 / 2.97 / 2.88 s | 0.42 / 0.48 s |
| Engine ticks when the test starts | 3.47–3.54 s | 0.83–1.01 s |
| Peak memory (`VmHWM`, after the full scenario incl. 16 384 cubes) | 722–724 MB | 288–290 MB |
| PSS at end | 516–521 MB | 135–141 MB |
| FPS | 58–60 (**vsync-capped at 60; not a performance measure**) | 60 (capped) |

Operation time, total (Rust call + GDScript mirror + MultiMesh rebuild), median:

| Operation | ARM64 translated | x86_64 native emulator | Windows desktop (for scale) |
|---|---|---|---|
| Fill (80-cube scenario) | 19.3 ms (Rust 16.1) | 0.3 ms | 1.5 ms |
| Undo | 3.9 ms | 0.5 ms | 0.9 ms |
| Redo | 2.4 ms | 0.4 ms | 0.9 ms |
| Save | 9.4 ms | 2.2 ms | 4.9 ms |
| Load | 16.3 ms | 1.1 ms | 4.8 ms |
| Fill 16 384 | 45.9 ms | 10.0 ms | 12.2 ms |
| Undo / Redo / Load at 16 384 | 26.1 / 34.0 / 48.8 ms | 2.8 / 7.4 / 6.8 ms | 3.1 / 10.9 / 11.1 ms |

The first Fill is slower on ARM64 because of translation warm-up; vsync could not be disabled on Android (`vsync=1` in the log), so FPS cannot show headroom.

## Problems found

1. **No real device.** The only way to run the ARM64 build here was a translating emulator; the memory and start-time numbers for ARM64 are dominated by that translation. Real ARM64 measurements are still needed.
2. **Peak memory looks high** (722 MB translated, 290 MB native) for 16 384 cubes. Part of this is the Godot engine (71 MB library) and emulator GL translation; it was not broken down. With a 2 GB emulator this is a risk flag for low-end phones, not a conclusion.
3. **Release export signed with a debug key** (see Environment); not a distributable build.
4. **Export message fixed:** Godot reported "No project icon specified"; a placeholder icon was added, no export errors remain.
5. **UI is not mobile-sized:** the button row is ~50 px wide per button on a 1080×2400 screen and the window is landscape 2400×954; taps worked at the coordinates logged by the app, but a user-facing layout was not designed (out of scope).
6. **Godot launcher activity** is `GodotAppLauncher`; launching `GodotApp` directly is not exported (documentation note for scripts).
7. **Persistence path** on Android is `user://` (app-private storage); save/load worked there. Behaviour across app updates and storage permissions was not examined.

## iOS: not built

Blocked because the environment is Windows with no macOS or Xcode. Required: a Mac with Xcode and command-line tools, `rustup target add aarch64-apple-ios`, and a decision on packaging (GDExtension for iOS needs a framework/xcframework or static library as described by the godot-rust and Godot iOS documentation for your versions, and an Apple developer team for signing). Untested starting commands:

```bash
rustup target add aarch64-apple-ios
cargo build --release -p cube-godot-adapter --target aarch64-apple-ios     # produces the library; packaging into a framework is not scripted
# then add it to the Godot iOS export (Xcode project), set the ios.* entries in godot-client/cube.gdextension,
# export with the Godot iOS template (ios.zip is present among the 4.7.2 templates) and run from Xcode on a device or simulator
```

The `.gdextension` file still carries only a placeholder iOS path. Not run, not verified.

## Not verified

Physical Android device (any ABI), ARM64 native performance and memory, thermal behaviour, long sessions, battery, Android API levels below 37 / devices with 4 KB pages, 32-bit ARM, Play-Store-style signing and `aab` export, Gradle-based export, iOS in any form, touch gestures beyond single taps, UI scaling for small screens.

## Reproduction (emulator)

```bash
emulator -avd Medium_Phone -no-snapshot -no-audio          # boot, then:
adb install build/cube-spike-arm64.apk
adb shell am start -W -n org.cube.spike/com.godot.game.GodotAppLauncher
adb logcat -s godot | grep UITEST                          # 19 PASS, "UITEST DONE staying alive"
adb shell input tap 112 79                                 # Undo (coordinates are logged as "UITEST BTN ...")
```

## Not to move to `master` without an owner decision

Export presets, build script, `res://bin` library layout, the `autotest` hook, the debug-key signing approach, and anything about persistence format, TCP protocol, undo semantics or repository layout.
