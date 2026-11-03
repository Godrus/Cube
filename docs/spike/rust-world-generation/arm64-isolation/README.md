# ARM64 emulator crash isolation (raw data)

Status of ARM64 validation: **inconclusive** (emulator with instruction translation only; no physical ARM64 device). Interpretation and tables: `../report.md`, section 26.

* `runs/<label>.runs.txt` - one line per launch (verdict, SIGSEGV/ANR flags, whether the process was alive at the end, seconds, `am start` result) and a summary line. Every launch is listed; none was retried silently. `arm64_minimal_exec4` run 10 was discarded because a second harness overlapped; this is noted in the file.
* `runs/<label>.runN.tombstone.txt` - crash header and backtrace lines (`DEBUG` tag) of each failed launch.
* `runs/native-rust-tests.txt` - the Rust test binaries run natively on the ARM64 (translated) and x86_64 emulator, 3 rounds, exit codes and test result lines. `dependency_rules` fails on both ABIs because it reads repository sources that do not exist on the device.
* `runs/earlier-arm64-attempts-with-retries.out`, `runs/earlier-x86_64-runs-exec-apk.out` - output of the earlier batches (greedy commit APK that calls `OS.execute`).
* `runs/memory.txt` - peak RSS / PSS of the noexec APKs.
* `logs/` - selected filtered logcat files (a crashed launch of the app, of the minimal project with four `OS.execute` calls and with one, and complete passing smoke runs on ARM64 and x86_64).
* `minimal-godot/` - the Godot project without any GDExtension used for steps 3a-3d (`noexec` feature skips `OS.execute`; mode via `setprop debug.cube.min`).
* `scripts/iso_run.sh` - the harness (`iso_run.sh <label> <apk> <package> <ready-marker> <N> [prop=value ...]`); `scripts/summarize.py` - counts per label.

Not possible: running Godot itself headless on Android (the release export ignores command-line intent extras), so the graphics-free check is the native Rust test binaries.
