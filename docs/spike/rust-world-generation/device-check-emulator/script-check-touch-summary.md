# Device validation summary (EMULATOR - NOT A DEVICE RESULT)

- device: sdk_gphone16k_x86_64, serial emulator-5554, ABI x86_64
- fingerprint: google/sdk_gphone16k_x86_64/emu64xa16k:17/CP31.260623.009/15923651:user/dev-keys
- android: 17 (SDK 37)
- date: 2026-11-04T15:38:52Z, runs per mode: 1, soak: skipped

## Mode faces: scripted smoke, 1 launches
### faces run 1: OK, SUMMARY failures=0, WORLDTEST FAIL lines 0, SIGSEGV/abort: no, ANR: no, logcat error lines 0
- start (am start -W TotalTime): 576 ms; peak RSS VmHWM 385392 kB, VmRSS 276172 kB, PSS 139176 kB
- battery temperature: 25 C -> 25 C; thermal: Thermal Status: 0 -> Thermal Status: 0
```
  TIME edit mode=faces total_ms=3.24
  TIME load mode=faces total_ms=5.93
  TIME first_5x5 mode=faces total_ms=27.81 build_ms_per_chunk=0.51 voxels=87098 drawn=12146 triangles=24292
  GEOM mode=faces elements=12146 triangles=24292 mesh_bytes=2234864 voxels=87098 nodes=33
  MEM mode=faces static_mb=0.0 video_mb=25.2 draw_calls=48 primitives=18562
  EDIT mode=faces policy=partial scenario=inside_section avg_ms=1.50 rust_ms=1.31 node_ms=0.15 sections=1.0 chunks=1.0 faces=12146 stale=0
  EDIT mode=faces policy=partial scenario=chunk_border avg_ms=1.73 rust_ms=1.57 node_ms=0.13 sections=2.0 chunks=2.0 faces=12152 stale=0
  EDIT mode=faces policy=partial scenario=corner avg_ms=3.08 rust_ms=2.78 node_ms=0.25 sections=3.0 chunks=3.0 faces=12152 stale=0
  EDIT mode=faces policy=partial scenario=corner_brick avg_ms=4.31 rust_ms=3.93 node_ms=0.33 sections=4.0 chunks=3.0 faces=12158 stale=0
  FILL mode=faces policy=partial fill_ms=4.87 voxels=243 sections=4 chunks=4 rust_ms=3.88 node_ms=0.51
  UNDO mode=faces policy=partial undo_avg_ms=1.62 redo_avg_ms=1.22 undo_sections=5 redo_sections=5 rust_ms=4.36 node_ms=0.37 faces_before=12212 edited=12272 undone=12212
  IMAGE faces vs greedy(after switch): of 1977600 px 0 differ, 0 holes; faces vs greedy(session): 0 differ, 0 holes
  VIEWS total of 9888000 px: 0 differ, 0 background holes
  SUMMARY failures=0
```
### faces: real touch events (adb input tap) on the live app
- screen 2400x1080, viewport 2400x954, tap offset (0, 63)
- taps sent: 8; app log lines produced by the taps (empty = the taps did not reach the buttons, check faces.after-touch.png):
```
edit changed 26 voxels
undo restored 26 voxels
redo applied 26 voxels
save ok=true
load ok=true
jump to (72, 8)
edit changed 27 voxels
```
- **faces smoke: 1 of 1 launches completed with no failures, 0 did not**

## Mode greedy: scripted smoke, 1 launches
### greedy run 1: OK, SUMMARY failures=0, WORLDTEST FAIL lines 0, SIGSEGV/abort: no, ANR: no, logcat error lines 0
- start (am start -W TotalTime): 482 ms; peak RSS VmHWM 386204 kB, VmRSS 283148 kB, PSS 151847 kB
- battery temperature: 25 C -> 25 C; thermal: Thermal Status: 0 -> Thermal Status: 0
```
  TIME edit mode=greedy total_ms=2.29
  TIME load mode=greedy total_ms=5.67
  TIME first_5x5 mode=greedy total_ms=20.09 build_ms_per_chunk=0.37 voxels=87098 drawn=1710 triangles=10858
  GEOM mode=greedy elements=1710 triangles=10858 mesh_bytes=701416 voxels=87098 nodes=33
  MEM mode=greedy static_mb=0.0 video_mb=24.3 draw_calls=48 primitives=9074
  EDIT mode=greedy policy=partial scenario=inside_section avg_ms=1.22 rust_ms=1.16 node_ms=0.04 sections=1.0 chunks=1.0 faces=1710 stale=0
  EDIT mode=greedy policy=partial scenario=chunk_border avg_ms=1.06 rust_ms=1.01 node_ms=0.04 sections=2.0 chunks=2.0 faces=1716 stale=0
  EDIT mode=greedy policy=partial scenario=corner avg_ms=1.66 rust_ms=1.56 node_ms=0.08 sections=3.0 chunks=3.0 faces=1716 stale=0
  EDIT mode=greedy policy=partial scenario=corner_brick avg_ms=2.17 rust_ms=2.03 node_ms=0.11 sections=4.0 chunks=3.0 faces=1722 stale=0
  FILL mode=greedy policy=partial fill_ms=2.20 voxels=243 sections=4 chunks=4 rust_ms=1.78 node_ms=0.17
  UNDO mode=greedy policy=partial undo_avg_ms=0.63 redo_avg_ms=0.95 undo_sections=5 redo_sections=5 rust_ms=1.79 node_ms=0.07 faces_before=1732 edited=1746 undone=1732
  IMAGE faces vs greedy(after switch): of 1977600 px 0 differ, 0 holes; faces vs greedy(session): 0 differ, 0 holes
  VIEWS total of 9888000 px: 0 differ, 0 background holes
  SUMMARY failures=0
```
### greedy: real touch events (adb input tap) on the live app
- screen 2400x1080, viewport 2400x954, tap offset (0, 63)
- taps sent: 8; app log lines produced by the taps (empty = the taps did not reach the buttons, check greedy.after-touch.png):
```
edit changed 26 voxels
undo restored 26 voxels
redo applied 26 voxels
save ok=true
load ok=true
jump to (72, 8)
edit changed 27 voxels
```
- **greedy smoke: 1 of 1 launches completed with no failures, 0 did not**

