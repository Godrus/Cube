# Work packages: epics and tasks for M1, M2, M3 (and later epics)

**Status:** proposed decomposition. Hierarchy: Vision -> pillar -> milestone -> epic -> feature (`docs/features.json`) -> work package -> task -> acceptance test. Each task names its acceptance; a task is done only by the Definition of Done in [`docs/handoff/NEW-CHAT-START.md`](../handoff/NEW-CHAT-START.md) section 6. Ids here are work ids (`E-`, `M1-A1`, `CMB-01`), not feature ids; a feature id is created only when a real new function appears (rule 5 of [`docs/archive/AGENT-HANDOFF.md`](../archive/AGENT-HANDOFF.md)). Roles are engineering areas, not owner assignments (the ownership matrix and DEC-072 stand; nobody is assigned by this file).

## Epic E-M1: persistent world and gameplay integrity (M1)

| Task | Action | Start state | Acceptance | Status |
|---|---|---|---|---|
| **M1-A2** | Inventory and world reconciliation | `game.state` and the journal were two stores | write-ahead intent in `game.state`; at open settle against the journal; Rust tests crash between each write pair (`crates/cube-app/tests/reconcile.rs`) | **TESTED** on branch `prod/m1-reconcile` (integrated, see [`.agent/LOG.md`](../../.agent/LOG.md) and git) |
| **M1-A1** | Unload edited bricks and bring them back | `evict_far` never unloads bricks in `Streamed.edited` (pinned) | build over many bricks, walk away, resident memory bounded, come back and restart: identical content; edit journal and snapshots intact | NOT_STARTED, design below |
| **M1-A3** | Measure generation cost on the main thread | `ensure_loaded` generates up to 48 bricks per frame in the frame call | a measured frame-time table (debug and release) under heavier worlds; only if it exceeds a budget, move generation to a worker with version checks, determinism and budgets | MEASURED 2026-11-12: the budget is exceeded, see the M1-A3 measurement below; follow-ups M1-A3b, M1-A3c |
| **M1-A4** | Character data ready for a server | player and game state are client files | an engine-neutral `CharacterRecord` with a stable text/binary form separate from the world (AI-15) | NOT_STARTED |

### M1-A1 design notes (so nobody rediscovers the trap)

Why edited bricks are pinned: the journal records `expected` revisions of the cells an edit touched, and **every unload and reload of a brick raises its revision by one** (never resets). A restart replays the journal onto a regenerated base; `bring_to_revision` reproduces the unload/reload steps of the *first* expectation of each brick only. If an edited brick were unloaded and reloaded between two of its edits, the second edit would expect a revision that a replay from scratch does not reach, and replay would fail (or silently drop the edit as superseded). So unloading edited bricks needs one of:

1. **Journal the reload** of an edited brick (a system record the replay understands), so replay reproduces the revision. Touches the journal contract (PS-10), needs design and probably owner review.
2. **An edited-brick store**: on eviction write the brick's current content to a per-brick file (checksummed, atomic) with its revision; reload from it, with replay skipping edits already contained (revision comparison). Needs a rule for what replay does on top of a stored brick.
3. **Fold edits into a snapshot** (compaction exists and is crash-tested) and reload bricks from that snapshot by key. Needs per-brick loads from the snapshot (the store loads whole worlds today).

Recommended first step: a short spike on option 2 or 3 in a test only (no API change) to see if replay-from-seed-plus-journal equals the world after eviction cycles, with the failing case (option "just evict") reproduced first as a test. Do not change the replay semantics (`replay_confirmed`) or the journal format without the persistence owner path; both are experimental until GOV-10.

### M1-A3 measurement (2026-11-12, claude-2)

Method: `cargo run -p cube-app --example gen_cost [--release] [-- quick]` drives `LocalHost::ensure_loaded` the way a walking player does in `Session::frame`: seed 7, the centre moves one brick per step for 40 steps (a new border every frame, the worst case), at most 48 bricks per call (`StreamBudget::default`), call time = streaming cost of that frame. Load radius = view radius + 1 (margin). The machine is one Windows desktop; mobile ARM will be slower, not faster. Working budget for streaming (an engineering value, not an owner decision): at most **4 ms** of a 16.7 ms frame at 60 FPS.

Release, per call (ms) and per brick (microseconds):

| gen | view radius | region bricks | first region ms | call ms mean | call ms p95 | call ms max | us per brick |
|---|---|---|---|---|---|---|---|
| v1 | 1 | 125 | 3.4 | 1.84 | 3.30 | 4.15 | 60 |
| v1 | 3 | 729 | 26.7 | 4.01 | 7.26 | 8.95 | 101 |
| v1 | 8 | 6859 | 760.6 | 22.04 | 35.50 | 40.39 | 484 |
| v6 (default) | 1 | 125 | 33.5 | 9.37 | 11.77 | 22.36 | 324 |
| v6 (default) | 2 | 343 | 110.7 | 10.05 | 21.36 | 26.60 | 356 |
| v6 (default) | 3 | 729 | 228.7 | 15.07 | 22.95 | 24.61 | 371 |
| v6 (default) | 5 | 2197 | 591.9 | 18.76 | 28.97 | 31.78 | 436 |
| v6 (default) | 8 | 6859 | 1948.1 | 29.25 | 46.83 | 55.43 | 644 |

Versions v3 to v6 cost the same within 3 percent (v3 added caves and ores; v4 to v6 add content that is cheap to generate). Debug, view radii 1, 2, 3: v1 434 to 687 us per brick, call mean 13 to 27 ms; v6 about 3.3 to 3.5 ms per brick, call mean 94 to 143 ms, first region 0.3 to 2.3 s. Debug is for development only.

Findings:

1. **The budget is exceeded.** The default generator (v6) costs about 0.32 to 0.37 ms per brick in release; a full call of 48 bricks is 9 to 15 ms at the device-class radii 1 to 3 (p95 up to 23 ms, max up to 27 ms), that is 2 to 6 times the 4 ms working budget and more than a whole 16.7 ms frame at radius 3. Radii 5 and 8 are worse (per-call scan of the whole cube adds to it: 436 and 644 us per brick).
2. **The hitch is real but only at borders.** The worst case moves a brick every frame, which no walking speed does (a sprint is about one brick per second). In practice each border crossing generates a slab of 25 to 81 bricks (radius 1 to 3), that is two frames of 9 to 15 ms about every second or two of walking: a visible hitch, not a steady load.
3. **The first region blocks.** Radius 3: 229 ms (v6); radius 8: 1.9 s. Acceptable behind a loading screen on desktop, to be re-measured on a phone (AND-01).
4. **Per brick, v6 is 3.5 times v1.** The cost is generation plus materialisation into Core through the system-commit port; the split between the two was not measured.
5. This measurement covers generation only. The question of DEC-095 (computed neighbours or halo cells) concerns meshing and is not answered by it: see M1-A3b.

Decision per the work package: the budget is exceeded, so generation must not stay unbounded on the main thread. Cheapest first (M1-A3c): a time-sliced per-frame budget (stop when about 4 ms are spent, a few bricks per call, the rest next frame); a worker thread (with version checks, determinism and budgets) only if time slicing still shows pop-in in a playtest (QA-01).

### M1-A3c result (2026-11-12, claude-2)

`Session::frame` now generates in chunks of `StreamBudget::load_chunk` (8) and stops after the first chunk that ends past `StreamBudget::load_time_us` (4000), within the old cap `load_per_frame` (48); the first region of a world is still generated in full, `load_time_us: 0` turns it off. Generation stays deterministic: only how many frames a load takes changes. Measured with the same example (release, desktop view radius 3, generator v6, flying east at 12 voxels per second, 1800 frames of 1/60 s, `Session::frame` wall time, includes client geometry):

| budget | frame ms mean | p95 | max | frames over 16.7 ms | bricks loaded |
|---|---|---|---|---|---|
| time-sliced 4 ms (default) | 1.65 | 6.66 | 21.89 | 9 | 2511 |
| unsliced, 48 bricks (before) | 1.64 | 1.32 | 38.25 | 36 | 2511 |

Frames over the frame budget fell from 36 to 9 of 1800 and the worst frame from 38 to 22 ms; the mean is the same and p95 rises because the same work is spread over more frames. The 9 remaining slow frames are not generation (one chunk is about 3 ms): the next suspects are the client's derived geometry after a region move and `evict_far`; profile them in M1-A3d before any worker thread. Residency is trimmed only when the region moves, so a device that cannot keep up lets the resident set grow until it catches up (pre-existing, now reachable): watch `resident` in QA-01. Tests that simulate fast flight with long simulated frames turn slicing off (`load_time_us: 0`).

### M1-A3b result (2026-11-13, claude-2)

`cargo run -p cube-app --example mesh_cost --release` meshes every present brick of a generated region (generator v6, seed 7, view radius 3: 240 present bricks of 343) two ways and checks that both give the same faces: the current faces provider, which reads the neighbours voxel by voxel through the snapshot (computed on demand), and a temporary halo (one pass over the 27 bricks copies the 18x18x18 voxels around the brick into an array, then the faces come from the array). Release, median of five passes, microseconds per brick:

| step | us per brick |
|---|---|
| snapshot capture (same for both) | 5.0 |
| on demand: provider compute | 296.8 |
| halo: copy 18^3 voxels | 8.8 |
| halo: faces from the array | 9.2 |
| halo total | 17.9 |
| on demand / halo | 16.5x |

About 240 faces per brick. The on-demand cost is 5832 snapshot lookups per brick (72 us for the lookups alone) plus six neighbour lookups per solid voxel; the halo reads each neighbour brick once. The halo here is temporary (a local array that lives for one compute), not stored ghost cells: it needs no format change and no extra memory beyond 11.4 KiB per compute.

What this means for DEC-095 (the owner chose computed neighbours on 2026-11-02, against the assistant's recommendation of temporary halo cells, and deferred the final choice to this measurement): the measured gap is 16 times, and it comes from the per-voxel reads, so the current provider is the slow path. If neighbours stay computed on demand, the cheap way to do it is a halo. Waiting for the owner: keep the current provider, or switch the faces provider (and later the greedy and SDF meshers) to the temporary halo (task M1-A3e). Meshing is not yet shown to be the cause of the remaining slow frames: see M1-A3d.

### M1-A3d result (2026-11-13, claude-2)

The `gen_cost` example now lists the slow frames of the Session run with what happened in each (bricks loaded, bricks evicted, change of the derived cache and of the active set). After M1-A3c all 7 remaining slow frames (18 to 22 ms) were frames that evicted 81 bricks (a slab behind the player) and loaded one: not generation, not meshing, not the client region. Cause: `SharedAsset::evict` copies the layer's whole cell store for every evicted brick (`current.store(layer).cloned()`), so evicting 81 bricks copied a store of up to 1500 bricks 81 times. Fix: `SystemCommit::evict_many` (one copy for the whole list, no revision, token or log change, unloaded cells skipped, duplicates counted once) used by `LocalHost::evict_far`. Result in release (desktop radius 3, v6, flying east, 1800 frames): frames over 16.7 ms 7 -> 0, worst frame 22.1 -> 16.3 ms, mean unchanged (1.6 ms). The unsliced run (the old budget) still shows 37 slow frames, so M1-A3c is still needed.

Same pattern, not fixed (changes revision semantics, needs its own task, M1-A3f): `materialise` also copies the store once per brick (each load is one commit); the per-brick cost in the M1-A3 table grows with the resident set (v1: 60 us at 125 bricks, 484 us at 6859), which is that copy. A batch load would be one commit for many bricks, and the asset token, operation ids and journal replay (`bring_to_revision`) all count commits, so it must be designed together with M1-A1.

Decision per the work package: no worker thread is needed for generation at the device-class radii; revisit after M1-A3f and a measurement on a phone (AND-01).

### M1-A5 failure matrix (2026-11-13, claude-2)

Storage failures against the stores of a world, and the test that covers each cell. Model results (the crash-point filesystem of `cube-testkit`) are not claims about a physical drive or a phone (DG-2). New in M1-A5 are marked NEW.

| Failure | Journal | Snapshot and world files | Tests |
|---|---|---|---|
| Torn tail (a crash inside a write) | repaired on open, acknowledged records stay; a damaged last acknowledged record is not mistaken for a torn tail | truncation and extension of any length detected | `a_torn_tail_is_repaired_and_damage_in_the_middle_is_refused`, `a_damaged_last_acknowledged_record_is_never_mistaken_for_a_torn_tail`, `every_truncation_and_extension_is_detected` |
| Corrupt bytes | damage in the middle refused, not hidden; any single flipped byte detected | detected, files named precisely | `damage_in_the_middle_lost_marks_and_a_foreign_schema_are_refused_not_hidden`, `every_single_byte_corruption_is_detected`, `missing_corrupt_and_foreign_files_are_reported_precisely_and_leftovers_are_harmless`, `hostile_counts_with_a_valid_checksum_do_not_allocate_or_panic` |
| Version mismatch | foreign schema refused | unknown schema, kind or limits refused before the body; wrong generator, structure or content version refused, never adapted | `headers_schema_kind_and_limits_are_checked_before_the_body`, `records_with_the_wrong_generator_structure_or_content_are_refused_never_adapted` |
| Failure at any step of a write or sync | the journal is poisoned, nothing is acknowledged afterwards, restart repairs to a consistent prefix | the previous snapshot is kept | `a_failed_write_or_fsync_poisons_the_journal_and_nothing_is_acknowledged`, `an_injected_failure_at_any_step_fails_the_save_and_keeps_the_previous_snapshot` |
| Crash at every point | no acknowledged record is lost | one whole generation survives, also through compaction | `every_crash_point_keeps_every_acknowledged_change_and_replay_restores_it`, `every_crash_point_leaves_either_a_complete_old_or_a_complete_new_snapshot`, `every_crash_point_of_a_compaction_leaves_one_whole_generation_and_loses_no_acknowledged_record` |
| Killed process | repairable file, no acknowledged record lost | n/a | `a_killed_process_loses_no_acknowledged_record_and_leaves_a_repairable_file` |
| Disk full (a write that fails half way) | NEW: half a record on disk, journal poisoned, restart cuts the tail, sequence continues without a gap | covered by failure at any step | NEW `a_write_that_fails_half_way_leaves_a_torn_tail_and_the_acknowledged_records_stay` |
| Unreadable file (permission, device) | NEW: an error, never an empty journal; a failed open changes nothing | NEW: `Store` error, not `Missing` | NEW `an_unreadable_file_is_an_error_never_an_empty_or_missing_one` |
| Read-only file (permission) | NEW: reading works, append is a controlled error and poisons the journal, content intact after the permission returns | n/a | NEW `a_read_only_journal_file_on_a_real_directory_is_a_controlled_error` |
| Repeated failures | the retry table and the floors survive restarts | NEW: three rounds with every step of a save failing in turn: the file is always a whole snapshot (old, or the offered one when only the directory sync failed), no leftovers but temporary files | NEW `repeated_failed_saves_never_touch_the_last_good_snapshot`, `a_small_retry_table_evicts_safely_and_the_floors_survive_a_restart_that_is_smaller_still` |
| Concurrent writers | NEW: a world is open for writing in one host at a time (`WorldLock`, an OS file lock on `.world.lock`, released at drop or process death, so a crash leaves no stale lock); a second `LocalHost::open` fails at once | same lock | NEW `store::lock_tests` in `cube-persistence`, NEW `cube-app/tests/world_lock.rs` |
| Migration | no migration exists: old or foreign versions are refused with a typed error and never adapted | same | the version tests above; the final format (F-PER-006) and a migration path are an owner decision, task M1-A5b |

Found by the matrix: nothing in the journal, snapshot or manifest code was wrong; the missing cell was the writer lock (two hosts could append to one journal and break the gapless sequence). The leftover `.world.lock` file in a world directory is harmless and is not removed.

## Epic E-CMB: basic combat (M2)

Goal: one scripted loop. See an enemy, it chases and attacks, the player attacks it, Rust computes damage, it dies, loot enters the inventory, the state is saved, the loop repeats. Out of scope: classes, skills, balance, networking (shape only), art.

Placement: engine-neutral Rust in `cube-app` (`combat.rs`, `creature.rs`) until the owner decides a gameplay crate (see [`PRODUCT-ROADMAP.md`](PRODUCT-ROADMAP.md) section 3). No Godot types, no wall clock, no randomness without a seeded source; fixed time step input; results are plain data the adapter draws.

Shape (so that it can become server-authoritative): `Input -> client intent (Attack{target?}) -> host validates and resolves (reach, cooldown, state) -> CombatEvent list -> game state -> client feedback`. The client never decides damage.

| Task | Action | Public data | Acceptance (Rust test unless noted) |
|---|---|---|---|
| CMB-01 | `Health` component and `Damage` event; death state; respawn rule (position, health) | `Health{cur,max}`, `Life{Alive,Dead}` | damage clamps at zero, death once, respawn restores, overflow-safe integers |
| CMB-02 | Attack resolution: cooldown, reach, one melee weapon definition (data) | `WeaponDef`, `AttackIntent`, `CombatEvent` | out-of-reach and on-cooldown refused with no state change; deterministic for equal inputs |
| CMB-03 | Persist player health and weapon in the game state (versioned text, like `game.state`, reading old files) | `GameState` extension | old `cube-game 1` files still load; round trip; damaged file rejected |
| CMB-04 | One hostile creature definition (data) and its state machine: idle, chase, attack, dead | `CreatureDef`, `CreatureState` | chases within sight, attacks in reach, stops when dead; no walking through solid voxels (uses the same collision query as the player) |
| CMB-05 | Creature spawn in the loaded world (deterministic from seed + brick, not saved while untouched) and despawn when far | spawn rule | same seed same spawns; far creatures removed; resident creatures bounded |
| CMB-06 | Loot: a drop table (data) rolled with a seeded source; the item enters the inventory by the existing inventory operation, refused when full without losing it | `LootTable` | drop equals table for a seeded roll; full inventory leaves the loot on the ground (or defers) per a stated rule |
| CMB-07 | Combat through the reconcile pattern: loot and damage taken are effects of confirmed host events, not of client claims | event ids | restart after a kill does not drop loot twice (same intent/confirm pattern as M1-A2) |
| CMB-08 | Adapter view: `GameView`/`SessionPort` carry health, target, hit feedback (no logic) | port fields | headless: fields equal the model |
| CMB-09 | Godot: attack input, health bar, creature mesh (simple), hit flash, sound hooks | GDScript glue only | real Godot smoke (`walk_smoke.py --combat`, to be written): kill one creature, loot appears, restart keeps health |
| CMB-10 | End-to-end combat test headless and in Godot | | the loop of section above passes twice in a row; features registry updated (new ids F-GPL-xxx, F-CRT-xxx created at that time, availability DEMO_AVAILABLE at most) |

Dependencies: CMB-02 needs CMB-01; CMB-04 needs CMB-01 and the collision query (exists in `cube-client::walk`); CMB-06 needs the inventory (exists); CMB-07 needs M1-A2 (done); CMB-09 needs CMB-08. CMB-01..03 start now.

### Путь от локального рантайма к мультиплееру (из прежнего разбора критического пути)

Локальный игрок это один сеанс единого авторитетного рантайма с одним писателем в такте; клиент обращается к нему только через типизированный интерфейс World Operations (команды внутрь: запрос правки, намерение игрока, открытие сеанса; представления и события наружу; обычные данные без ссылок и обратных вызовов на состояние рантайма), с локальным `AuthorizationContext`. Сообщения несут `OperationId` и `ActorKind`, клиент держит кэш и не меняет состояние мира. Малый сериализующий loopback-адаптер доказывает, что сообщения сериализуются; набор соответствия серверных контрактов позже идёт на обоих транспортах. Порядок наращивания, ни один шаг не меняет интерфейс: проводной протокол и версии (NET-02), затем эксперимент по транспорту SPK-NET и выбор GOV-12, внешний транспорт NET-04, интеграция аутентификации SRV-02, повторное подключение и права SRV-05 и SRV-06, выделенный хост SRV-08, предсказание и согласование на клиенте CLI-05. Не тянуть это в ближайшие вехи без решения владельца и владельцев сети и сервера.

### Модуль правил игры (предложение, ждёт решения владельца ODM-05 и изменения ADG)

Сейчас игровая логика живёт в `cube-app`. Минимальный отдельный модуль, не ломающий слои: ответственность авторитетное состояние и правила на стороне сервера (игрок и контроллер, запросы взаимодействия с миром, существа и ИИ, бой и здоровье, предметы, инвентарь, экипировка, добыча, смерть и возрождение, запись игрока); не отвечает за геометрию, генерацию, механику хранения, сеть, отрисовку, интерфейс, ассеты. Положение: выше портов Editing API и чтений Processing, ниже серверного корня композиции; зависит от Core (типы чтения), Processing (снимки, запросы), Editing API (`IEditingService`, происхождение и авторизация); собирается серверным хостом. Потребляет: чтение мира, запрос правки, тик, детерминированные потоки случайных чисел, хранилище записей, кандидатов спавна. Запрещено: godot, ввод-вывод, часы, глобальная случайность, внутренности хранилища, сеть. Мир меняется только через `IEditingService` с происхождением Player. Открытый вопрос: модель сущностей (ECS или явная доменная модель), решает короткий эксперимент с письменным выводом до скелета модуля. Последствия: новая строка и столбец в ADG, правило archcheck, владелец и орган приёмки.

## Epic E-WGN2: first biomes and caves (M3)

| Task | Action | Acceptance |
|---|---|---|
| WGN2-04 (DONE as terrain-v5 ruins with a den, F-WGN-006) | places of interest | see the registry |
| WGN2-03 (DONE as terrain-v4, F-WGN-005) | trees and forests | see the registry |
| WGN2-02 (DONE as terrain-v3, F-WGN-004) | caves and ores | see the registry |
| WGN2-01 (DONE as terrain-v2, F-WGN-003; later slices are versions 3, 4, 5, never edits of v2) | generator version 2 in the existing `cube-worldgen` registry (never edit v1 in place): climate fields (integer noise), 2-3 biomes with blended borders | pinned hashes; v1 worlds still open and regenerate identically; order and thread independence tests like v1 |
| WGN2-02 | caves (3D integer noise carve), ores | same as above; ground stays continuous across brick borders; no open holes at the surface by accident |
| WGN2-03 | trees and simple vegetation as deterministic features crossing brick borders | oracle equality across borders |
| WGN2-04 | POIs: camp or ruin as a structure placed deterministically | same |
| WGN2-05 | new worlds use v2, existing worlds keep their version (record) | a v1 world opens unchanged |
| WGN2-06 | Godot: biome colours for materials, look at it | person or picture check; TypeSafe optional, advisory only |

WGEN-02..04 (service, materialiser) remain gated by GOV-21/GOV-11; this epic needs neither because it only adds a generator version to the registry.

## Later epics (epic level only; product decisions pending)

E-RPG (M4): XP and levels, stats, one specialisation, ability data, weapon types, armour slots, loot tiers. Needs ODM-05.
E-CRAFT-BUILD (M5): recipes and stations; sculpt/boolean tools over the Editing API (PS-6), history (PS-7, GOV-25), composite (PS-8), micro-voxel editing (PS-9).
E-LIVING (M6): creature set, behaviour, spawn rules, NPCs, settlements, day/night, weather.
E-ADVENTURE (M7): dungeons, bosses, quests, collections.
E-NET (M8): protocol and authority, dedicated host, replication, reconnect, interest management; start with the intent/validated-result shape already used by E-CMB.
E-MOBILE (M9): Android build and spike, touch input, adaptive UI, budgets; start the spike as soon as a device exists.
E-SOCIAL, E-SCALE, E-POLISH (M10..M12).

## Parallel workstreams and areas of ownership

| Stream | Area | Next tasks | Conflict rule |
|---|---|---|---|
| World and persistence | `cube-server`, `cube-persistence` | M1-A1 spike, M1-A3 measurement | touches journal contract only with the persistence path |
| Gameplay and combat | `cube-app/src/{game,combat,creature}.rs` | CMB-01..03 | does not touch `cube-server` internals |
| WorldGen content | `cube-worldgen` | WGN2-01.. | adds versions, never edits a pinned one |
| Client and Godot | `cube-client`, `cube-godot-adapter`, `godot/` | CMB-08/09 after the model | GDScript glue only, no logic |
| Geometry and building | backends, editing | none before M5 planning decisions | Editing API only |
| Networking, mobile | none yet | contracts, spike when a device exists | |
