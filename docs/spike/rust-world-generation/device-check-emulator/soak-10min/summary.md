# Device validation summary (EMULATOR - NOT A DEVICE RESULT)

- device: sdk_gphone16k_x86_64, serial emulator-5554, ABI x86_64
- fingerprint: google/sdk_gphone16k_x86_64/emu64xa16k:17/CP31.260623.009/15923651:user/dev-keys
- android: 17 (SDK 37)
- date: 2026-11-04T15:17:30Z, runs per mode: 1, soak: 10 min

## Mode faces: scripted smoke, 1 launches
### faces run 1: OK, SUMMARY failures=0, WORLDTEST FAIL lines 0, SIGSEGV/abort: no, ANR: no, logcat error lines 0
- start (am start -W TotalTime): 453 ms; peak RSS VmHWM 388004 kB, VmRSS 284956 kB, PSS 140863 kB
- battery temperature: 25 C -> 25 C; thermal: Thermal Status: 0 -> Thermal Status: 0
```
  TIME edit mode=faces total_ms=2.24
  TIME load mode=faces total_ms=6.19
  TIME first_5x5 mode=faces total_ms=19.57 build_ms_per_chunk=0.35 voxels=87098 drawn=12146 triangles=24292
  GEOM mode=faces elements=12146 triangles=24292 mesh_bytes=2234864 voxels=87098 nodes=33
  MEM mode=faces static_mb=0.0 video_mb=25.2 draw_calls=48 primitives=18562
  EDIT mode=faces policy=partial scenario=inside_section avg_ms=0.86 rust_ms=0.79 node_ms=0.04 sections=1.0 chunks=1.0 faces=12146 stale=0
  EDIT mode=faces policy=partial scenario=chunk_border avg_ms=1.06 rust_ms=0.98 node_ms=0.05 sections=2.0 chunks=2.0 faces=12152 stale=0
  EDIT mode=faces policy=partial scenario=corner avg_ms=1.51 rust_ms=1.38 node_ms=0.07 sections=3.0 chunks=3.0 faces=12152 stale=0
  EDIT mode=faces policy=partial scenario=corner_brick avg_ms=1.87 rust_ms=1.72 node_ms=0.10 sections=4.0 chunks=3.0 faces=12158 stale=0
  FILL mode=faces policy=partial fill_ms=1.76 voxels=243 sections=4 chunks=4 rust_ms=1.38 node_ms=0.11
  UNDO mode=faces policy=partial undo_avg_ms=0.70 redo_avg_ms=0.77 undo_sections=5 redo_sections=5 rust_ms=1.89 node_ms=0.12 faces_before=12212 edited=12272 undone=12212
  IMAGE faces vs greedy(after switch): of 1977600 px 0 differ, 0 holes; faces vs greedy(session): 0 differ, 0 holes
  VIEWS total of 9888000 px: 0 differ, 0 background holes
  SUMMARY failures=0
```
### faces: real touch events (adb input tap) on the live app
- taps sent: 8; app log lines produced by the taps (empty = the taps did not reach the buttons, check faces.after-touch.png):
```
```
- **faces smoke: 1 of 1 launches completed with no failures, 0 did not**

## Soak faces: 10 minutes (in-app loop)
- process alive at the end: yes; SOAK DONE reported: 1; Fatal signal/ANR lines: 0; peak RSS VmHWM 377188 kB
```
  SOAK START seconds=600 mode=faces
  SOAK t=30 fps_avg=60.0 fps_min=7.0 chunks=30 nodes=41 elements=14742 draw_calls=55 primitives=24154 video_mb=25.5 edits=9 edit_avg_ms=0.91 edit_max_ms=2.38 stale=0
  SOAK t=60 fps_avg=60.0 fps_min=60.0 chunks=30 nodes=39 elements=14102 draw_calls=57 primitives=22808 video_mb=25.4 edits=19 edit_avg_ms=0.82 edit_max_ms=2.58 stale=0
  SOAK t=90 fps_avg=60.0 fps_min=31.5 chunks=30 nodes=48 elements=14430 draw_calls=61 primitives=22512 video_mb=25.4 edits=29 edit_avg_ms=0.85 edit_max_ms=2.58 stale=0
  SOAK t=120 fps_avg=53.8 fps_min=7.6 chunks=30 nodes=40 elements=14238 draw_calls=58 primitives=23346 video_mb=25.4 edits=39 edit_avg_ms=0.98 edit_max_ms=4.92 stale=0
  SOAK t=150 fps_avg=51.6 fps_min=11.5 chunks=30 nodes=42 elements=14082 draw_calls=59 primitives=23150 video_mb=25.4 edits=49 edit_avg_ms=1.29 edit_max_ms=11.36 stale=0
  SOAK t=180 fps_avg=51.8 fps_min=14.8 chunks=30 nodes=43 elements=14592 draw_calls=60 primitives=23246 video_mb=27.5 edits=59 edit_avg_ms=1.30 edit_max_ms=11.36 stale=0
  SOAK t=210 fps_avg=50.4 fps_min=12.3 chunks=30 nodes=43 elements=15180 draw_calls=58 primitives=23424 video_mb=27.5 edits=69 edit_avg_ms=1.40 edit_max_ms=11.36 stale=0
  SOAK t=240 fps_avg=51.3 fps_min=11.1 chunks=30 nodes=41 elements=14276 draw_calls=57 primitives=23038 video_mb=27.4 edits=79 edit_avg_ms=1.31 edit_max_ms=11.36 stale=0
  SOAK t=270 fps_avg=59.9 fps_min=21.1 chunks=30 nodes=54 elements=14726 draw_calls=72 primitives=24422 video_mb=27.5 edits=89 edit_avg_ms=1.29 edit_max_ms=11.36 stale=0
  SOAK t=300 fps_avg=60.0 fps_min=40.6 chunks=30 nodes=38 elements=14036 draw_calls=54 primitives=21684 video_mb=27.4 edits=99 edit_avg_ms=1.21 edit_max_ms=11.36 stale=0
  SOAK t=330 fps_avg=60.0 fps_min=60.0 chunks=30 nodes=48 elements=14838 draw_calls=61 primitives=23350 video_mb=27.5 edits=109 edit_avg_ms=1.18 edit_max_ms=11.36 stale=0
  SOAK t=360 fps_avg=60.0 fps_min=60.0 chunks=30 nodes=49 elements=14506 draw_calls=64 primitives=23654 video_mb=27.5 edits=119 edit_avg_ms=1.14 edit_max_ms=11.36 stale=0
  SOAK t=390 fps_avg=60.0 fps_min=60.0 chunks=30 nodes=38 elements=14232 draw_calls=55 primitives=23060 video_mb=27.4 edits=129 edit_avg_ms=1.11 edit_max_ms=11.36 stale=0
  SOAK t=420 fps_avg=60.0 fps_min=60.0 chunks=30 nodes=38 elements=14428 draw_calls=48 primitives=22052 video_mb=27.5 edits=139 edit_avg_ms=1.07 edit_max_ms=11.36 stale=0
  SOAK t=450 fps_avg=60.0 fps_min=60.0 chunks=30 nodes=34 elements=14324 draw_calls=52 primitives=22622 video_mb=27.4 edits=149 edit_avg_ms=1.04 edit_max_ms=11.36 stale=0
  SOAK t=480 fps_avg=60.0 fps_min=60.0 chunks=30 nodes=32 elements=13838 draw_calls=49 primitives=22666 video_mb=27.4 edits=159 edit_avg_ms=1.01 edit_max_ms=11.36 stale=0
  SOAK t=510 fps_avg=60.0 fps_min=60.0 chunks=30 nodes=37 elements=13704 draw_calls=56 primitives=22402 video_mb=27.4 edits=169 edit_avg_ms=0.99 edit_max_ms=11.36 stale=0
  SOAK t=540 fps_avg=60.0 fps_min=60.0 chunks=30 nodes=47 elements=14966 draw_calls=59 primitives=21942 video_mb=27.5 edits=179 edit_avg_ms=0.95 edit_max_ms=11.36 stale=0
  SOAK t=570 fps_avg=60.0 fps_min=60.0 chunks=30 nodes=34 elements=14248 draw_calls=50 primitives=22014 video_mb=27.4 edits=189 edit_avg_ms=0.95 edit_max_ms=11.36 stale=0
  SOAK DONE seconds=600 edits=199
```
(memory/temperature samples: soak-faces.samples.csv)

