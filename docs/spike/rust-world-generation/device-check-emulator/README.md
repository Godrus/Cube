# Emulator checks of the device-validation tooling (NOT device results)

No physical Android ARM64 device was available (`Blocked: no physical ARM64 device`). These files come from emulators and only show that the APKs, `scripts/device-validation.sh` and the in-app test entry point work.

* `script-check-touch-summary.md` - `scripts/device-validation.sh --allow-emulator --no-soak` on the x86_64 emulator: scripted smoke, then real `adb input tap` touches (Edit, Undo, Redo, Save, Load, Jump, Renderer, Edit) and the log lines they produced.
* `soak-10min/` - the same script with the 10-minute soak of `soak-faces-x86_64.apk` on the x86_64 emulator (this run used the APK build just before the last rebuild, which only changed how the button coordinates are printed); `soak-faces.samples.csv` has the memory / temperature samples.
* `runs/*.runs.txt`, `runs/run-output.txt` - ARM64 translated emulator launches of the final `test-faces/greedy-arm64.apk` (3 + 3, every launch listed) and the client APKs (3 + 3, 25 s alive check).
