# SPK-PERS — persistence durability spike report

Status: **report accepted for the experimental protocol; no format approved (GOV-10 open); no device durability claimed (DG-2 open).**
Task text (ADG [`agent-task-breakdown.md`](../production/agent-task-breakdown.md)): "Persistence durability spike (fsync, directory fsync, atomic rename, crash points) on a spike branch; NO format approval. Acceptance: report states results per platform; never merged."

Before this report no SPK-PERS report existed in the repository, in `docs/` or in any branch (searched `docs/`, `.agent`, all remote branches, decision registry, exit checklist). The spike code is therefore new.

## Where the code is

Spike branch `spike/pers-durability`, commit `03d1ff6` (file `crates/cube-integration-tests/tests/spk_pers.rs`). CI on that commit: success (run 42719662199, Linux and Windows). **The spike code is not merged** (acceptance rule). Everything the spike learned that production needs is implemented separately in `cube-persistence` and covered by its own tests (see "What production relies on").

## Experiments, results, limits

| # | Question | Method | Result (Windows 11, NTFS, local dev machine) | Result (CI Linux) | Limit |
|---|---|---|---|---|---|
| E1 | Does write temp, fsync temp, rename over existing, directory sync leave a whole old or new file? | repeated replace of an existing file with verification after each step | whole file every time; rename over an existing file works; **directory fsync is a no-op on Windows** (`DirStorage` skips it) | assertions pass (ext4/overlay of the runner, not inspected) | a clean run proves nothing about power loss; the model (FaultFs) covers the ordering, not the hardware |
| E2 | Append, fsync, truncate behaviour of a journal file | append records, fsync, truncate the tail, reopen | exact lengths after each step; truncate plus fsync is observed durable in-process | pass | observed in one process; OS caches not dropped |
| E3 | Cost of one acknowledged append | append 512 B plus fsync, repeated | about 0.7 ms per acknowledged record on this machine (SSD, write cache state unknown) | not measured (CI timings are not device measurements) | not a device figure; mobile storage will differ (DG-2) |
| E4 | Process kill during replacement | child process replaces the file in a loop, parent kills it at random moments, six rounds | every observed state was a whole old or whole new file; leftover temp files were harmless | not run on CI | a process kill does not lose OS-cached data, so it tests ordering and atomicity of the visible name only, not power loss |
| E5 | Corruption and truncation detection | zero-filled and truncated frames, flipped bytes | every case rejected by the CRC or the length check, never accepted as data | pass | CRC-32 detects accidental damage, not tampering |

Mobile (Android, iOS): **not tested** (no devices, DEV-01 / EXT-DEV open).

## Guarantee boundaries (what may be claimed)

* **Claimed:** the *protocol* is ordered correctly: data is written and `fsync`ed before it is acknowledged or published; a name is published only by an atomic rename; the directory is synced after rename and after the first creation of a file where the platform allows. Under the FaultFs model (data volatile until fsync, names volatile until directory sync, atomic rename, torn writes) every crash point of the protocols in `journal.rs`, `manifest.rs`, `compact.rs` recovers to a valid state. Observed on Windows local and in CI.
* **Not claimed:** that data survives power loss on any real device. `fsync` is as strong as the OS and the drive make it (drive write caches, `F_FULLFSYNC` on Apple platforms, Android file systems and vendor firmware are outside this spike). The directory sync is **a no-op on Windows**; there NTFS metadata journalling is relied on and *not* verified.
* **Not claimed:** crash-atomicity of the Core commit together with the journal write (CB-1 stays open; the write-ahead phases make the gap detectable and recoverable, not impossible).

## Verdict

* The spike question "is the planned write/fsync/rename/directory-sync protocol implementable and testable with deterministic crash exploration on desktop CI?" is **answered yes**.
* Acceptance criterion "report states results per platform": Windows (observed), Linux (CI assertions only, no timings), mobile (not tested, stated). The report meets the criterion as worded; it does **not** answer DG-2 and does not try to.
* Consequence: the start-after dependency SPK-PERS of PERS-02, PERS-03, PERS-05 is considered satisfied for *experimental-format* work. GOV-10 still needs the RG-1 and PERS-05 evidence and owner approval; DG-2 still needs devices.

## What production relies on

| Finding | Where it landed |
|---|---|
| Windows `sync_all` needs a handle opened with write access | `DirStorage::fsync` |
| Directory sync is unavailable on Windows | `DirStorage::fsync_dir` (no-op, documented) |
| Truncated or zero-filled records must never be taken for data | CRC and length checks in `codec.rs`; fail-closed journal marks (`journal.rs`) |
| Leftover temporary files are harmless but must not accumulate | `collect_garbage` in `manifest.rs`, called by `open_world` and `compact` |
| A killed process leaves whole files at the visible names | `tests/persistence_journal.rs` (killed child writer) |
