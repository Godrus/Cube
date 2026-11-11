# Cube Product Roadmap (M0 to M12), engineering crosswalk, critical path

**Status:** proposed execution order from the owner's directive of 2026-11-10. The strategic direction is the owner's; the milestone cut and order are an agent proposal that stays reversible. It changes **no** engineering gate, ADG edge or contract (those stay in `docs/production/`). Status words follow [`docs/archive/AGENT-HANDOFF.md`](../archive/AGENT-HANDOFF.md) rule list: implementation NOT_STARTED / IN_PROGRESS / IMPLEMENTED / TESTED / INTEGRATED, availability NOT_AVAILABLE / INTERNAL_ONLY / DEMO_AVAILABLE / PLAYER_AVAILABLE. Feature ids refer to `docs/features.json` (never a second registry); statuses are copied from it, not restated here. Vision: [`docs/vision/GAME-VISION.md`](../vision/GAME-VISION.md). Tasks: [`WORK-PACKAGES.md`](WORK-PACKAGES.md).

**Numbering.** `M0..M12` here are *product* milestones. The older Game Track uses `M2-T` (technical slice: walk, edit, save/load, done in the demo), `M2` (First Playable with a gameplay loop) and `M3` (Vertical Slice) in `docs/production/`. Proposed mapping: Game Track M2 = product M1 + M2 (+ minimal M3); Game Track M3 = product M3..M5 slice. The historical Phase 0..8 of GDD/TDD v0.3 are kept in section 2; PS-0..PS-14 are not renamed.

## 1. Milestones

Status = what is true in the repository on `dev` 679e8d5 plus later commits (current tasks: [`.agent/BOARD.md`](../../.agent/BOARD.md)). "Demo" means the Godot developer demo, run by hand on the developer machine, not in CI.

| M | Goal and player outcome | Required features | Engineering stages | Main risks | Acceptance (playable) | Status |
|---|---|---|---|---|---|---|
| **M0** Current playable foundation | Launch, walk, edit blocks, inventory, save/continue | F-CLI-001/002/003, F-GPL-002/003, F-SRV-003/004, F-WGN-002 | PS-1..5, 10 (part), 11 (part), 12 (demo) | Godot not in CI; steep terrain blocks walking | `python tools/visual-qa/walk_smoke.py --play` in real Godot | IMPLEMENTED, DEMO_AVAILABLE |
| **M1** Persistent world and gameplay integrity | Travel and build for long without losing progress or filling memory | evict edited bricks; inventory and world consistent (done: reconcile); worker-thread generation if measured; character data ready for a server | PS-10, PS-4, PS-12 | edited-brick revisions in replay (see WP M1-A1); CB-1 | build, leave, unload, restart, return: intact; kill between stores: no dupes | IN_PROGRESS (inventory consistency TESTED; eviction NOT_STARTED) |
| **M2** First action RPG combat loop | See an enemy, fight, win, loot, repeat (prototype loop exists) | health, damage, death/respawn, one weapon, one hostile creature with chase and attack AI, loot into inventory, combat feedback, persistence | gameplay (no module yet, see below), PS-12; PS-13 contracts later | no gameplay owner or module (OI-8/ODM-05); balance undecided | end-to-end scripted combat in real Godot | IMPLEMENTED prototype, DEMO_AVAILABLE with `--creatures` (F-GPL-004/005, F-CRT-001/002, F-CLI-005); not played by a person |
| **M3** Interesting procedural world | Exploring shows new things | 2-3 biomes with transitions, caves, ores, trees, simple structures, POIs, generator versioning, old worlds still load | PS-11 | WGEN-02..04 gated (GOV-21, GOV-11); noise cost on main thread | a person walks and sees different areas; old save regenerates identically | IMPLEMENTED prototype, DEMO_AVAILABLE: biomes, caves, ores, trees, ruins (F-WGN-003..006); not explored by a person |
| **M4** RPG progression | The character grows and plays differently | XP, levels, stats, first specialisation, several abilities, weapon types, armour, loot tiers, persistence | gameplay | class design is an owner decision (ODM-05) | level up, new ability, new weapon changes play | IN_PROGRESS: XP and levels done (slice 1, F-GPL-006, `cube-app/src/progression.rs`); first weapon type and recipe done (iron sword, G, F-GPL-007); attack bonus by level and iron armour (second recipe) done (F-GPL-008); class stats screen, abilities, equipment slots, more recipes not started |
| **M5** Crafting and deep building | Building is more than unit cubes | recipes, stations, tools, sculpt, boolean, undo/redo, composite, micro-voxel editing (limited) | PS-6, 7, 8, 9 | G5/RG-1 open; OPN-07 (smooth surfaces) | craft a tool, sculpt, undo, place a composite object | NOT_STARTED (engine parts TESTED, not player tools) |
| **M6** Living world | The world feels alive | several creatures, behaviour states, spawn/despawn, simple ecosystem, NPCs, settlements, day/night, weather | gameplay, PS-11 | simulation cost; owner for creatures | watch creatures act on their own | NOT_STARTED |
| **M7** Adventure content | Exploration leads to adventures | dungeons, local bosses, rewards, events, quests, unique items, collections, repeatables | gameplay, PS-11 | content volume; pipeline | clear a dungeon, beat a boss | NOT_STARTED |
| **M8** Multiplayer vertical slice | Two players travel, fight and build together | dedicated server, two clients, auth (minimal), character and world sync, shared building, combat sync, server inventory validation, reconnect, interest management | PS-13 (+10) | no Server/Networking owner (DEC-072); transport undecided | two clients in one world | NOT_STARTED |
| **M9** Android and cross-play MVP | PC and Android in one world | Android build, touch input, adaptive UI, mobile profile, compatible content, physical-device checks | PS-12, 14 | no physical devices (EXT-DEV); thermal and memory | a PC and an Android player together | NOT_STARTED (early Android spike due before M9) |
| **M10** Expanded RPG and social | Depth and community | several classes, skill growth, weapon variety, deeper crafting, trade, groups, guilds, collections, social spaces | gameplay, PS-13 | economy integrity (AI-20) | group activity with trade | NOT_STARTED |
| **M11** Large-scale infrastructure | Toward 500 players | distributed simulation, regional servers, entity handoff, load balancing, monitoring, admin, accounts, stress tests | PS-13, 14 | only after M8 works | measured load tests | NOT_STARTED |
| **M12** Production quality | Release quality | art, animation, sound, music, UX, accessibility, content, PC and mobile optimisation, mod support, user content editor, release prep | PS-14 | scope | release candidate checks | NOT_STARTED |

**Visual direction (owner, 2026-11-11):** Cube World Alpha / Veloren style; micro-voxel character work is deferred and changes no milestone. See [`docs/vision/VISUAL-STYLE-GUIDE.md`](../vision/VISUAL-STYLE-GUIDE.md). Focus: mechanics, adventures, RPG progression, exploration, Android compatibility.

**Do not** build the M11 infrastructure before M8 runs; **do not** wait for M8 to design the network contracts: gameplay state in M2 must be engine-neutral and server-authoritative in shape (intent in, validated result out).

## 2. Historical Phase 0..8 of GDD/TDD v0.3 mapped to today

The v0.3 text is not in the repository; this list is the owner's directive's own summary. Order of execution does not follow the old numbering.

| Phase | Content | Where it lives now |
|---|---|---|
| 0 Technical research | engines, mobile rendering, chunks, meshing, networking, memory | engine chosen (Rust + Godot, AOD-01/02); spike done; **Android spike and memory profiling on devices not done** (DG-1..3) |
| 1 Voxel engine prototype | chunks, terrain, place/break, persistence, streaming, PC and Android builds | M0 (PC); Android in M9 |
| 2 Gameplay prototype | controller, camera, inventory, items, basic combat, creatures, AI, equipment | controller, camera, inventory: M0; the rest M2, M4 |
| 3 World generation | biomes, caves, structures, resources, vegetation, weather, POIs | M3, M6 |
| 4 Multiplayer prototype | server, auth, sessions, cross-play, sync, shared building | M8, M9 |
| 5 Persistence and infrastructure | accounts, characters, world, regions, private server, local world, recovery | M1 and M8/M11; world and recovery exist at engine level (PS-10) |
| 6 RPG and content | classes, skills, progression, crafting, dungeons, bosses, collections, social | M4, M5, M7, M10 |
| 7 Scaling | regions, load balancing, handoff, 500-player tests, DB, monitoring | M11 |
| 8 Production | content, UX, mobile and PC optimisation, QA, certification, release | M12 |

## 3. Engineering crosswalk

`Product capability -> original phase -> engineering stage -> modules -> status -> availability -> evidence`. Verified against `docs/features.json` on `dev` 679e8d5. Dependencies are the ADG's ([`production/module-boundaries.md`](../production/module-boundaries.md)), not new ones.

| Capability | Phase | Stage | Modules | Status | Availability | Evidence / feature ids |
|---|---|---|---|---|---|---|
| Walk, camera, place/mine blocks | 1, 2 | PS-12 (+4, 5) | client, app, server, orchestration, voxel backend | INTEGRATED | DEMO_AVAILABLE | F-CLI-002/003 |
| Streaming with unloading (unedited bricks) | 1 | PS-12 | server, core | INTEGRATED | DEMO_AVAILABLE | F-SRV-004 |
| Unloading of edited bricks | 1, 5 | PS-10 + PS-12 | server, persistence | NOT_STARTED | NOT_AVAILABLE | limit in F-SRV-004 |
| Inventory, objective (prototype rules) | 2 | PS-12 (gameplay: no module) | app | INTEGRATED | DEMO_AVAILABLE | F-GPL-002/003 |
| Inventory/world consistency | 5 | PS-10 design (CB-1) | app, server | TESTED (reconcile, Rust tests) | DEMO_AVAILABLE | F-GPL-002, `tests/reconcile.rs` |
| Combat, creatures, AI | 2, 6 | gameplay; PS-12/13 for views and authority | `cube-app` (no gameplay module yet) | prototype INTEGRATED | DEMO_AVAILABLE (`--creatures`) | F-GPL-004/005, F-CRT-001/002, F-CLI-005 |
| Biomes, caves, structures, POIs | 3 | PS-11 | worldgen | NOT_STARTED (terrain-v1 only) | NOT_AVAILABLE | F-WGN-002 |
| Save/load, recovery, world record | 5 | PS-10 | persistence | INTEGRATED (experimental format) | DEMO_AVAILABLE | F-PER-003/007/008/009 |
| Sculpt, boolean | 6 | PS-6 | tools over editing | NOT_STARTED | NOT_AVAILABLE | F-TLS-001/002 |
| Undo/redo | 6 | PS-7 | history | NOT_STARTED, blocked GOV-25 | NOT_AVAILABLE | F-HST-001 |
| Micro-voxel editing | 6 | PS-9 | resolution, voxel backend | level-aware ops INTEGRATED, selective detail NOT_STARTED | NOT_AVAILABLE | F-RES-001/002 |
| Composite objects (modular gear) | 6 | PS-8 | composite | NOT_STARTED, blocked GOV-19/23 | NOT_AVAILABLE | F-CMP-001 |
| Shared building, multiplayer, cross-play | 4 | PS-13, 14 (+3, 4, 10) | server, net | NOT_STARTED, owners unassigned | NOT_AVAILABLE | F-SRV-001/002 |
| Android | 1, 8 | PS-12, 14 | adapter, client | NOT_STARTED | NOT_AVAILABLE | F-PLT-002 |
| Content packages and mods | 6, 8 | PS-14 / content | content (proposed) | NOT_STARTED | NOT_AVAILABLE | F-MOD-001 |

**Gameplay has no module in the ADG.** `cube-app/src/game.rs` holds the prototype gameplay in the composition root. A new crate (for example `cube-game`, engine-neutral) needs an ADG Part A/B edit, which `cargo xtask archcheck` enforces equal to the code graph: that is an owner/architecture decision (ODM-05, OI-8). Until then new engine-neutral gameplay code lives in `cube-app` as a pure module (no Godot, no I/O in the rule code), shaped so that it can be moved without change.

## 4. Dependencies and critical path

Hard technical (T), product (P), owner decision (O), external (E):

```
M1 (T) eviction of edited bricks ......................... independent of M2
M2 combat: needs (T) inventory ops (exist), (O) module location, no persistence blocker
   -> M4 progression (O: class design)  -> M10
M3 world: (T) PS-11 service WGEN-02..04 (gated O: GOV-21, GOV-11) or a version-2 generator inside the existing registry
M5 tools: (T) G5/RG-1 for micro-voxel and composite (O); sculpt/boolean/history PS-6/7 after G4 (done)
M8 multiplayer: (T) M2 intent/authority shape, PS-10 recovery, (O) Server/Networking owners, transport; (E) none
M9 Android: (E) physical devices, (T) M8 for cross-play; feasibility spike has no dependency but a device
M11: after M8 only
```

**Critical path to the MVP (section 6a of the vision):** M2 (combat) -> M3 (world) -> M4 (progression) -> M8 (two clients) -> M9 (Android). M1 integrity runs in parallel and must finish before any claim that edits persist for long play. M5 and M6 enter the MVP in minimal form.

**Independent work available now** (no owner decision needed): M1-A1 design and implementation; M1-A3 measurement of main-thread generation; M2 engine-neutral combat in `cube-app`; M3 caves and a second biome as a new generator version in `cube-worldgen` (registry-based, pinned hashes); Godot HUD and animation for M2.

**Blocked, with who unblocks:** gameplay module and owner (owner: ODM-05/OI-8); Server, Networking, Client Runtime, Engine Adapter, WorldGen owners (owner: DEC-072 deferral); final save format GOV-10/11 and Frozen contracts (owner, after RG-1); Android validation (owner: devices).

## 5. Owner decisions that really block

| Id | Question | Blocks | Meanwhile |
|---|---|---|---|
| OI-8 / ODM-05 | gameplay owner and module location | clean M2/M4 structure | pure module in `cube-app` |
| GOV-10/11 | final save format | stable saves, device claims | experimental formats, no promises |
| OPN-19 | Game Track pillars as written | M2 acceptance wording | this document's pillars as working text |
| DEC-072 | owners of Server/Networking/Client/Adapter/WorldGen | M8 start | contracts and shapes only |
| rules | inventory, death, balance numbers | final M2/M4 | prototype values, configurable |
| OPN-07 | smooth surfaces in the building scope | M5 planning (SDF/mesh tools) | engine parts stay as built |

## Путь к релизу и пробелы планирования (перенесено из прежнего DEVELOPMENT_ROADMAP)

**Готовность к релизу** меряется отдельной шкалой: Internal Alpha, Closed Alpha, Beta, Release Candidate, 1.0. Критерии и тесты: [`docs/RELEASE_CRITERIA.md`](../RELEASE_CRITERIA.md) (её номера M0..M8 это вехи релиза, а не продуктовые M0..M12 выше). Суть шкалы:

- **Internal Alpha:** основные циклы проходят сценариями, нет открытых Critical, серверная авторитетность подтверждена на встроенном и выделенном хосте, в CI есть регрессионный набор.
- **Closed Alpha:** сборки стоят у тестеров на каждой заявленной платформе, краш-отчёты и метрики приходят, обновление N на N+1 сохраняет миры (тест миграции), повреждённый мир восстанавливается или отвергается.
- **Beta:** нагрузочный тест достигает согласованного числа игроков, cross-play зелёный, открытых Critical нет.
- **Release Candidate:** объём заморожен, PC-1 Frozen через GOV-15, все форматы Owner-Approved, после заморозки нет feature-коммитов.
- **1.0:** операционные инструкции отработаны, откат проверен.

Числовые пороги не придумывать: бенчмарки B-1..B-11 описаны в [`docs/RELEASE_CRITERIA.md`](../RELEASE_CRITERIA.md), значения задаёт владелец по результатам замеров.

**Работы без плана** (нет ни в M1..M12, ни в инженерных стадиях; нужны до Closed Alpha и Beta, требуют решения владельца и отдельного плана): телеметрия и краш-отчёты; юридические вопросы, приватность, соответствие требованиям магазинов; локализация; античит и безопасность эксплуатации; онлайн-сервисы (аккаунты, подбор игроков, список серверов, региональное распределение).

**Известные пробелы:**

- Распределение одного логического мира на 500 игроков по регионам и серверам не спроектировано: сначала исследование и прототип-замер, потом детальный дизайн PS-13 и M11.
- Сроки дать нельзя: для оценки нужны замеры B-1..B-9, оценки задач от владельцев областей, измеренная скорость работы и владельцы неназначенных областей.

**Справка в архиве** (Proposed, по нужде): [`docs/archive/production/VERTICAL_SLICE_SCOPE.md`](../archive/production/VERTICAL_SLICE_SCOPE.md) (состав и критерии вертикального среза), [`docs/archive/production/FIRST_PLAYABLE_SCOPE.md`](../archive/production/FIRST_PLAYABLE_SCOPE.md) (требования к первой играбельной версии), [`docs/archive/production/GAME_TRACK_ROADMAP.md`](../archive/production/GAME_TRACK_ROADMAP.md) (прежний трек игры), [`docs/archive/DEVELOPMENT_ROADMAP.md`](../archive/DEVELOPMENT_ROADMAP.md) (спецификации вех по двенадцати полям).

## Вертикальный срез и контроль прогресса (перенесено из архивных планов Game Track)

**Вертикальный срез** (первый маленький законченный кусок игры; Proposed, состав выбирает владелец при планировании) должен ответить на вопросы, которые не снимает прототип:

1. Держит ли цикл интерес весь сеанс (час и больше), а не первые двадцать минут?
2. Мир разнообразен и целенаправлен (несколько сред, точки интереса, причины идти)?
3. Есть ли у прогрессии форма (начало, середина, конец) и важны ли выборы?
4. Выдерживают ли бой и разнообразие существ (разные угрозы, элитный враг или босс, умения)?
5. Становятся ли строительство и разрушение игровой системой, а не просто постановкой и ломанием блоков?
6. Справляется ли сохранение и восстановление с реальным объёмом состояния (мир, персонаж, инвентарь, прогресс, квесты)?
7. Есть ли убедительное визуальное и звуковое направление и конвейер контента?
8. Масштабируется ли технология до этого контента на выбранном железе (и на физическом Android, если мобильное внутри)?
9. (по желанию) Работает ли кооператив вдвоём на одном авторитетном хосте?

| Область | Минимум | Рекомендуемо |
|---|---|---|
| Точки интереса | 3 разных вида (ресурсный объект, лагерь с существами, место босса) | 5 |
| Среды | 2 отполированные плюс переходы и водоподобное препятствие | 3 |
| Существа | 3 (двое враждебных с разным поведением, одно мирное) | 4 |
| Босс | элитный бой с одной особой атакой | двухфазный босс |
| Предметы и редкость | 12-15 предметов, 2 уровня редкости | 20 предметов, 3 уровня |
| Экипировка | 2 слота (оружие, тело) | 3 |
| Прогрессия | уровни и рост статов | плюс один значимый выбор (две ветки) |
| Крафт | 3 рецепта на станции | 8 |
| Строительство | материалы из добычи и один тип сооружения, без отмены | последствия разрушения (укрытие, доступ) и отмена |
| Активности | цепочка квеста из 3 шагов как ведущий путь | цепочка плюс побочная активность |
| Платформы | десктоп и базовое устройство Android | плюс второй класс устройств |
| Кооператив | не в срезе; отдельный технический спайк (два клиента, один хост, правки видны обоим) | только если есть владельцы и серверные задачи |

Критерии приёмки среза (показ записывается): VSG-01 посторонний тестер проходит весь сеанс без помощи разработчика; VSG-02..08 каждая из областей выше показана в игре (среды, существа и бой, предметы и экипировка, прогрессия, сбор-крафт-постройка, квест и точка интереса); VSG-09 сохранение, выход, загрузка посреди квеста, сбой при записи восстанавливается, сохранение прототипа мигрирует; VSG-10 есть стиль и представительные ассеты и звук; VSG-11 правила на стороне сервера, клиент не хранит авторитетного состояния; VSG-12 замеры на названном железе, пороги только после одобрения владельцем; VSG-14 перечислено, чего в срезе нет.

**Контроль прогресса.** Прогресс не измеряется числом задач, тестов, контрактов или документов. Он измеряется показанным игроку: вход и выход из мира; прогулка по маршруту через границы блоков и биомов без дыр и швов; бой, дроп и респаун; предмет меняет исход следующего боя; изменение мира переживает перезапуск; возврат и продолжение с сохранёнными инвентарём, позицией и статами; посторонний тестер понимает и повторяет цикл. Каждое приращение заканчивается демо-сборкой и одностраничной заметкой плейтеста (что игрок смог, что сломалось, что сказал); по каждому вопросу честно отмечать: не демонстрируется, показано разработчиками, показано посторонним тестером.

**Гипотезы для проверки играми (не требования):** H-1 рельеф как тактический инструмент в бою (копать, заграждать, высота); H-2 ресурсы из исследования питают и строительство, и снаряжение (один экономический контур); H-3 короткий быстрый цикл мотивирует в первый час сильнее глубокого дерева умений.

## Критерии приёмки первой играбельной версии (перенесено из архивного FIRST_PLAYABLE_SCOPE)

Определение: игрок запускает игру, создаёт мир, бегает и прыгает по процедурному ландшафту с двумя видами местности, встречает враждебное существо, дерётся оружием, берёт добычу, надевает лучшее оружие и чувствует разницу в следующем бою, копает и строит, сохраняется, выходит, возвращается и находит мир, персонажа и вещи как оставил. Каждая проверка с записанным показом (S скрипт, M человек, O посторонний тестер); числа производительности не критерии, их измеряют и сообщают на названном железе, пороги задаёт владелец.

FPS-01 запуск и меню. FPS-02 новый мир по seed (одинаковые хеши блоков при одном seed, разные при другом). FPS-03 управление (ходьба, бег, прыжок, ступенька в один воксель, камера не входит в рельеф). FPS-04 исследование (маршрут через границы блоков и двух видов местности без дыр и швов). FPS-05 встреча враждебного и мирного существа. FPS-06 бой (победа; поражение и возрождение). FPS-07 добыча и инвентарь. FPS-08 эффект экипировки (тот же бой заметно короче). FPS-09 изменение мира (каждая правка через Editing API, в журнале `UserCommit`). FPS-10 сохранение через меню. FPS-11 возобновление (правки, позиция, инвентарь, экипировка, статы как были). FPS-12 сбой при сохранении (убийство процесса на каждом шаге протокола уплотнения, следующий запуск восстанавливает проверенное состояние). FPS-13 авторитет (правил игры нет в GDScript, клиент не меняет состояние мира и инвентаря локально). FPS-14 понятный и повторяемый цикл (посторонний тестер играет 20+ минут без подсказок; отмечено, нашёл ли управление, дрался ли, надевал ли, строил ли, начал ли второй бой сам). FPS-15 заявленные отсутствия. FPS-16 замеры (кадр, задержка правки, время сохранения и загрузки, память). FPS-17 письменное заявление, что мешало бы мобильной сборке.

Ещё не показаны (по состоянию на 2026-11-11): FPS-14 (человек не играл), FPS-12 на полном цикле «мир, журнал, инвентарь» (CB-1 открыт), FPS-16 замеры, FPS-17 как документ. Они входят в QA-01 на [`.agent/BOARD.md`](../../.agent/BOARD.md).
