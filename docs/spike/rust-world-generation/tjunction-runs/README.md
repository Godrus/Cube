# Android noexec runs of the T-junction stage

One line per launch in `*.runs.txt` (verdict, SIGSEGV/ANR flags, process alive at the end, seconds), `memory.txt` (peak RSS / PSS), `logs/` (two complete filtered logcat files of passing greedy smoke runs), `run-android-noexec.sh` (the script that booted the emulator, ran every APK and shut the emulator down). All APKs are the `noexec` presets, so no `OS.execute` call is made. Interpretation: `../report.md`, sections 33-35. Emulator results only; ARM64 validation remains inconclusive.
