# Production Plan Alignment Report

> **Erratum 2026-11-09 (decision registry now exists: `docs/decisions/decision-registry.md`).** (1) Statements here that unsigned commits mean 'no decision can be confirmed' are withdrawn: DEC-002 accepts an explicit owner chat statement as a source and makes signatures optional. (2) CPR-PRN-05…08 are not confirmed decisions: the owner stated them as context; they are `RECORDED` (DEC-052). CPR-PRN-01…04 repeat AOD-01…07 (`RECORDED`). (3) AOD-25/AOD-26 are **not assigned** as bundles; the content is split into DEC-030…DEC-035 (CONCEPT, CONFIRMED from the owner's message) and DEC-053/DEC-054 (RECORDED / PROPOSED). (4) 'M2' in CPR-SAV-01 is read as 'the first approved save format'; working terms: **M2-T** = technical slice (walk, edit, save/load; the old M2 of DEVELOPMENT_ROADMAP), **M2** = First Playable with the gameplay loop (Game Track); ODM-06 still fixes their acceptance. (5) Priority, stage, module and 'related decisions' columns are agent annotations, not confirmations; only the `statement` of a registry entry is the owner's wording. (6) 'owner-signed' means an explicit owner statement, PR approval or owner-written text; no cryptographic signature is required (DEC-002).

**Статус:** **Proposed.** Анализ совместимости подтверждённых концептуальных требований (`cube-product-vision.md`) с Production Plan v2.1. Ничего не утверждается; код, документы репозитория, Git-топология не изменялись.
**Дата:** 2026-11-09. Ссылки `AI-/CPR-/OPN-` — `cube-product-vision.md`; `VP-/OCC-` — `cube-owner-decision-register.md`.

---

## 1. Проверка актуального состояния (read-only, факты)

| # | Что проверялось | Результат | Как проверено |
|---|---|---|---|
| 1 | Ветка / HEAD | `docs/architecture-roadmap-v1`, `16db379` (= `origin/docs/architecture-roadmap-v1`); 40 коммитов впереди `master` (`bf60534`); всего 43 коммита, все с `%G?=N`, один автор | `git status -sb`, `rev-parse`, `rev-list`, `log --format=%G?` |
| 2 | Рабочее дерево | `M .gitignore` (добавлены `AGENTS.md` и 20 путей `_build/*`); `?? docs/decisions/owner-decision-extensibility-v1.md` (черновик). Больше ничего. (Мой запуск `adg_check.py` создал `docs/archive/audit/__pycache__`; он **удалён**, дерево восстановлено) | `git status --short`, `git diff` |
| 3 | Комплект Production Plan | **v2.1**, статус **Proposed**; `docs/production/`: plan, module-boundaries (ADG), implementation-roadmap, agent-task-breakdown, architecture-blocker-decisions, integration-and-qa-plan, ps-0-exit-checklist, два отчёта ревизии; плюс Game Track: GAME_TRACK_ROADMAP, FIRST_PLAYABLE_SCOPE/DEPENDENCIES, VERTICAL_SLICE_SCOPE, TECHNOLOGY-GAME-TRACK-INTEGRATION (все Proposed) | чтение заголовков |
| 4 | PS-0 / ACR / AOD / OAM / OD | присутствуют: `ps-0-exit-checklist.md`, `PS-0-REMAINING-BLOCKERS.md`; ACR: `audit-resolution-report.md` + `acr-004-owner-review.md`; AOD: `owner-architecture-decisions-v1.md` (AOD-01…24); OAM: `owner-approval-matrix-v1.md` (OAM-01…10); OD: `OWNER_DECISION_REGISTER.md` (ODM-01…17), `GOV-01-DECISION-REGISTER.md` (15 решений), `OWNER-DECISION-REQUIRED.md`, `m2-delivery-review/OWNER_DECISIONS_REQUIRED.md` | `ls`, чтение |
| 5 | Статусы контрактов | все Draft 0.1.0 / Proposed; P0-alpha не утверждён; PC-1 не зарегистрирован Provisional; ACR-001…003 Pending, ACR-004 Pending; OAM-09/10 не утверждены | README контрактов, plan §3–4, OAM |
| 6 | Реестр / CI | `decision-registry.md` **нет**; `.github` **нет**; CI **нет** | `ls` |
| 7 | PS-0 | **0 из 8** gate-задач выполнено (GOV-01…06, TEST-01, XT-01); **0 из 15** решений GOV-01 подтверждено | `ps-0-exit-checklist.md` §5; `PS-0-REMAINING-BLOCKERS.md` (на 2026-11-05; новых записей реестра с тех пор не появилось — проверено отсутствием файла) |
| 8 | Условия допуска к PS-1 | plan §14: 6 критериев (подтверждённые решения ACR-001, ACR-003, ACR-004 A и D, OD-R1, OD-R2, OD-R6, D-R8, D-K, D-T, D-C, D-0; P0-alpha Approved + PC-1 Provisional; named owners; docs-only baseline; CI + required checks; ADG adopted). **Ни один не выполнен** | чек-лист §3 |
| 9 | Структурная проверка ADG | запуск `python -I docs/archive/audit/adg_check.py` (read-only): 147 задач, 15 gates; **PASS**: нет циклов, все узлы существуют, нет обратных ссылок по стадиям, согласованность gate; **FAIL (policy)**: G14 closure, G1 как предшественник G2…G14, G1 merge policy (102 задачи) — это известные дефекты D-01/D-02 (`docs/archive/gov01/ADG-D01-D02-ANALYSIS.md`), патч — Proposed, не применён | фактический запуск |
| 10 | Актуальность аудитов | `Independent-Production-Audit.md` и `PROJECT_STATUS.md` датированы 2026-11-05 на базе `2c0c1f3`; с тех пор 7 коммитов (GOV-01 corrective review, Game Track, M2 delivery review, M2-T optimisation). Аудит **не перезапускался**: считать устаревшим по состоянию репозитория, не по выводам | `git rev-list --count 2c0c1f3..HEAD` = 7 |
| 11 | Код vs документы | на `master` — только документация (0 файлов `crates/`: GOV-04 по факту ближе всего выполнен там); на ветке — 42 отслеживаемых файла `crates/` (спайк, 7 crate: `cube-core, -processing, -editing, -persistence, -worldgen, -server, -godot-adapter`). Имена crate отличаются от плана (`cube-editing` vs `cube-editing-api` + `cube-orchestration`); `cube-godot-adapter` зависит напрямую от `worldgen` и `persistence` (план: adapter → core, processing, api, client). Core: `Material` = голый `u16` (`brick.rs`). Всё это **спайк (AOD-18, AOD-24), не production**; расхождение ожидаемо, ничего не переносится. Тесты и бенчмарки **не запускались** | чтение `Cargo.toml`, `brick.rs`, `git ls-tree` |
| 12 | `AGENTS.md` | в `.gitignore` (не отслеживается), ссылается на `.agent/…` и `.cursor/rules`; **их нет**; `.agent/state/CURRENT_CONTEXT.md` отсутствует. Условия обязательного обновления CURRENT_CONTEXT (завершён этап/рефакторинг) не наступили: файлы репозитория не менялись; правило не применено | `ls -a` |

**Вывод §1:** состояние — «PS-0 открыт, PS-1 не начат, реализации нет»; новые требования не меняют этого и не открывают новых блокеров PS-1 сверх перечисленных в `implementation-roadmap-alignment.md` §8.

---

## 2. Метод

Для каждого модуля/вопроса: (1) что говорит существующий контракт или решение (с ссылкой); (2) требование из vision; (3) вердикт: **OK** (совместимо), **OK+** (совместимо с уточнением формулировки), **GAP** (в плане нет места), **CONFLICT** (прямое противоречие). Предложения ограничены тем, что подтверждено реальной потребностью Cube и становится дорогим при откладывании.

---

## 3. Модуль за модулем

### 3.1 Geometry Core (PS-1) — **OK+**
* Модель: `Layer{id, representation, transform, resolution, data}`, `CellKey{layer, level:i8, coord:[i32;3]}`, `Material(u16)`, `CommitBatch`, `ActorKind{User,System,Recovery}` (plan §5).
* Контентная независимость: Core хранит число; ни имён, ни пакетов не знает — **совместимо** (AOD-04). Условие: число — *индекс палитры*, а не «идентичность» (AI-01, VP-01).
* Выборочная детализация — см. §5 вопрос 1.
* Размер сооружений: `CellKey` с `i32` ограничен D-K (ODM-11); отдельного предела «размера сооружения» нет — **совместимо**; операция не ограничена размером, только ресурсами (AI-10).
* Конфликтов нет.

### 3.2 Processing (PS-2) — **OK**
Производные данные (меши, collision, LOD) — провайдеры; уровни детализации входят как зависимость ввода. Переход между уровнями на границах — открытый C-12 (AI-07). Provisional из-за отсутствия Geometry Processing Specification (ODM-02/TC-01).

### 3.3 Editing API и Orchestration (PS-3/PS-4) — **OK+ / GAP (мелкий)**
* AOD-15: все пользовательские изменения — операции; `expected`-ревизия даёт определённый конфликт при совместном строительстве (AI-17) — OK.
* **GAP:** операции огромного размера (CPR-BLD-11) — контракт E-xx должен допускать разбиение на CommitBatch-и под одним OperationId, бюджет/отмену и определённую семантику частичной неудачи; сейчас `CommitBatch` — «один asset, атомарно», история хранит post-state (AI-10). Не блокирует PS-1; проверяется при approval E-02…E-12 (GOV-09).
* **GAP:** изменения от симуляции (CPR-SIM-*, MEC-01) — нет класса входа (AI-09).

### 3.4 Geometry Backends (PS-5) — **OK+ / решение об объёме**
Voxel-бэкенд обязателен. SDF/Mesh-бэкенды и Sculpt/Boolean (PS-6) обоснованы столпом «Universal Geometry» (PROJECT_STATUS 5.1), а владелец теперь говорит «плавные поверхности — не требование» (CPR-BLD-04). Это не конфликт с Core (enum закрыт, voxel — один вариант), но меняет приоритет PS-5/PS-6 (OPN-07, VP-09).

### 3.5 Resolution и Composite (PS-8/PS-9) — **GAP по времени**
* `RefinementRegion` — данные в Core (C-12), логика — Resolution (PS-9, после History и Composite). Требование микровокселей (CPR-BLD-02) идёт в срез PSL-3 (≈M3), а PS-9 стоит поздно в ADG → нужен минимальный путь (VP-10) либо перенос приоритета. Подробно — §4 строка R-3.
* Composite: подвижные конструкции (CPR-BLD-14) используют `Composite Part`/`Assembly Graph` (ACR-003) — OK по данным; динамика — вне Composite (§4, R-9).

### 3.6 History (PS-7) — **OK+**
Undo/redo и «серверная история» (CPR-BLD-10). History и Persistence не импортируют друг друга (Part B note 1). Не определено, **персистентна ли** история между сессиями. Если да, она ссылается на материал через id-таблицы и должна переживать отсутствующие пакеты (§5 вопрос 6). Не блокирует.

### 3.7 Persistence (PS-10) — **GAP (формат) — главное расхождение**
* AOD-21 утверждает архитектуру (seed+GeneratorVersion, журнал, снимки, порядок загрузки), форматы — нет. WP-01 (Draft) знает `WorldSeed, GeneratorVersion, SchemaVersion, WorldConfig`, **не** знает PackSet/IdMaps/GenerationIdentity; нет исходов `PackMissing/PackIncompatible/UnknownDefinition` (черновик X-3).
* Нет домена «запись персонажа» в PERS-01…09 (FIRST_PLAYABLE_DEPENDENCIES: «a player record … needs a contract and a place in PS-10») (AI-15).
* Нет свойства «атомарная группа записей в журнале» (AI-15/20).
* Эти пункты **дёшевы сейчас, дороги после первого утверждённого формата** (§4 R-4…R-7).

### 3.8 WorldGen (PS-11) — **OK+**
Чистая детерминированная функция (AOD-09). Требование: биомы/структуры/материалы через реестр (read-only view) и включение пакетов, влияющих на генерацию, в идентичность (AI-06). Новое ребро `worldgen → content` (§6). До PS-11 не критично.

### 3.9 Networking и Server Runtime (PS-13) — **OK**
Server — composition root; один авторитетный runtime (AOD-22). Экономика/торговля (CPR-MP-05/08) использует однопоточный тик (SRV-03) — OK; требование к журналу — §3.7. Серверы с отдельными персонажами/экономиками — политика данных, не транспорта. Моды в multiplayer (синхронизация пакетов) — исключено владельцем (DF-17).

### 3.10 Content Registry и будущие игровые системы — **GAP (в ADG нет модуля)**
* В ADG Part A/B нет crate для реестров/идентичности; Game Rules (ODM-05) не утверждён.
* Предлагается (Proposed) **минимальный** набор контрактов `content-registry-contracts.md` (Draft, не утверждать): 
  * **CR-01 Identity:** `ContentId = namespace:name`, правила грамматики (OPN-05).
  * **CR-02 Registry:** на вид определений: регистрация, поиск, **дубликат = ошибка диагностики**, а не panic.
  * **CR-03 PackManifest (минимум):** id, версия, зависимости+диапазоны.
  * **CR-04 WorldContentRecord:** `PackSet`, `IdMap` (append-only), `GameVersion`, `GenerationIdentity` — **простые данные**, используются WP-01.
  * **CR-05 Diagnostics:** структурированные события (дубликат, конфликт, нехватка зависимости) + порт логирования.
  Этого достаточно для CPR-CON-01…11, CPR-SAV-01…09 на этапах M2-T/M2. Ничего про код-хост, SDK, скрипты, схемы типов определений (ODM-13 — до `GM-20`).

### 3.11 Engine Adapters (PS-12) — **OK**
Presentation по стабильным id; пакеты не требуют Godot (R-06). Adapter → content только через client; прямой зависимости в Part B нет и не добавляется.

---

## 4. Таблица несоответствий

Формат: `Requirement ID → Existing Contract → Conflict → Impact → Proposed Correction → Required Approval → Target Stage`. Все корректировки — **предложения**.

| # | Requirement | Existing Contract | Conflict | Impact | Proposed Correction | Required Approval | Target |
|---|---|---|---|---|---|---|---|
| R-1 | CPR-CON-03/04, SAV-03 | D-C (GOV-01 №12): `Material(u16)`; spec 3.5 «локальная палитра → глобальная библиотека»; `brick.rs` `Constant(u16)` | не конфликт, но спайк трактует число как идентичность; ширина `u16` не обоснована для модов | если число — идентичность → перепись формата сохранения и Core brick при первом моде | VP-01: в D-C записать «непрозрачный `MaterialIndex`, ширина объявляется в заголовке формата; идентичность выше Core» | owner (GOV-01 №12) — уточнение, не новый пункт | до GOV-02 |
| R-2 | CPR-CON-01…11 | ADG Part A/B: модуль контента отсутствует | **GAP** | реестры окажутся в Core, server или game-rules — вероятно в Core (нарушает AOD-04 «контент-агностичен») | VP-04: Draft `content-registry-contracts.md` + строка `cube-content` в ADG Part A/B (OPN-04) | Integration Owner → owner (изменение ADG и модульной сетки; AOD-19: AU-1.0 не правится) | до M2-T (не PS-1) |
| R-3 | CPR-BLD-02/03 (микровоксели) | C-11/C-12 (Core данные + Resolution логика), PS-9 в ADG после History/Composite; C-12 Open: переход, макс. уровень | время: M3 нуждается в микровокселях, PS-9 поздно; PC-1 Provisional | либо M3 без микровокселей, либо ломка порядка стадий | VP-10: минимальный режим «клетка либо крупная, либо разбита на 2³ на фиксированном уровне; без resampling-перехода и без гладкого шва» реализуется в `cube-backend-voxel` (PS-5) + Processing meshing; PS-9 остаётся полной версией | owner (OPN-08) | PSL-3 |
| R-4 | CPR-SAV-01…04, 06 | WP-01…WP-10 (Draft); AOD-21 | **GAP:** нет PackSet/IdMaps/исходов | первый утверждённый формат без места под моды = переформат | VP-05: расширить WP-01 (+ исходы), Draft | Draft-ревизия; формат — owner позже | до утверждения формата M2 |
| R-5 | CPR-SAV-03/06/09 | WP-07/WP-08 (протокол компакции Approved как архитектура) | **GAP:** порядок долговечности id-map vs. journal не определён; снимок «проверяет hash, epoch, revision, generator version» — не id-map | авария между записью journal и записью id-map → индекс без идентичности | VP-06: id-map append-only, записывается не позже первой использующей записи; проверка id-map — часть recovery-check | owner (при утверждении WP-07/08) | PS-10 дизайн |
| R-6 | CPR-MP-02/03/04, RPG-03 | FIRST_PLAYABLE_DEPENDENCIES §«player record» (место не определено) | **GAP:** персонаж должен жить отдельно от мира | позже — тяжёлый перенос между серверами, мультиперсонаж | VP-07: запись персонажа — отдельный домен с собственным id; мир хранит ссылки | owner | M2 (структура) |
| R-7 | CPR-MP-08, BLD-10, MEC-01 | AOD-21 журнал: гранулярность **не утверждена**; E-серия: `IOperationJournal` для committed user operations | **GAP:** журнал ориентирован на геометрию; атомарная операция «геометрия + инвентарь/блок-сущность/торговая запись» не выражена | экономика и блок-сущности потребуют переписать журнал | VP-08: свойство «атомарная группа под одним OperationId на несколько доменов» записать как требование к формату журнала (не реализация) | owner (с гранулярностью журнала) | PS-10 дизайн |
| R-8 | CPR-SIM-*, MEC-01, BLD-08 | MB-1: User/System(WorldGen)/Recovery/Core-internal; AOD-15, 20; OAM-10 (Proposed) | **GAP:** изменений мира, порождённых правилами сервера (рост, обрушение, механизмы, откат временных), нет входа; допускать им `User` нельзя (авторизация, история), `System` зарезервирован за WorldGen | позже — расширение сохраняемого enum журнала = migration | VP-02: объявить набор источников расширяемым, зарезервировать значение для «симуляции мира»; политику (история, права) — позже | owner (GOV-01 №5 / ACR-004 A, D / OAM-10) | к GOV-02 (одна строка) |
| R-9 | CPR-BLD-13/14 | D-T: авторитетное состояние без float; спека Composite | **OK+:** непрерывная поза тела (вращение) — не `Layer.transform` (целые + 24 вращения + степень двойки) | иначе придётся ослаблять D-T | VP-03: уточнение в D-T; поза — состояние сущности/физики | owner (GOV-01 №11) | к GOV-02 |
| R-10 | CPR-BLD-13 | ADG: «collision» — только провайдер (O-1); модуля физики нет | **GAP:** гибридная физика построек (динамические конструкции) без модуля/владельца | M4+; для M2 достаточно kinematic controller | зафиксировать как OPN-12, ничего не строить сейчас | owner | после M3 |
| R-11 | CPR-BLD-04 | PROJECT_STATUS 5.1 столп «Universal Geometry (voxels, micro-voxels, SDF, polygon mesh)»; PS-5/PS-6 | **CONFLICT по смыслу** (не по контракту) | время/усилия PS-5 (SDF, Mesh), PS-6 (Sculpt, Boolean) | VP-09/OPN-07: после First Playable; Core-enum не менять | owner | до PS-5 |
| R-12 | CPR-BLD-11/12 | E-серия; CommitBatch — один asset атомарно | **GAP:** нет ограниченного по ресурсам механизма для сверхкрупных операций | защита сервера и истории | требование к E-02…E-12: разбиение, бюджет, отмена; числа — только измеряемые | owner при approval E-серии | PS-3/PS-4 |
| R-13 | CPR-BLD-09 | P-09 Import/Export — UNASSIGNED (O-4); спека 3.5 палитра | **GAP (малый)** | чертежи переносятся между мирами с разными id-таблицами | чертёж = asset с палитрой стабильных id; формат/владелец — OPN-15 | owner | M3/M4 |
| R-14 | CPR-SIM-05/06 | AOD-13/21 | **GAP (малый)** | состояние региона «когда симулирован» — метаданные вне геометрии | AI-14: метаданные региона в персистентном домене; догоняющая симуляция — операции | owner | M4 |
| R-15 | CPR-CON-10 | нет порта логирования | **GAP (малый)** | диагностика модов/зависимостей | AI-21/CR-05: порт логирования вне Core; D-E dependencies (GOV-01 №14) не блокируются | Integration Owner | M2 |

**Не конфликтуют** и не требуют изменений: AOD-04, 06, 07, 12, 13, 14, 15 (кроме R-8), 16, 17, 20, 21 (кроме R-5/R-7), 22, 23, 24; ACR-001, ACR-002; OAM-03/04/05/06 (OAM-09 — аргумент за C1, см. AI-11).

---

## 5. Отдельные проверки

**1. Может ли модель Core поддерживать выборочную микровоксельную детализацию?**
**Да, по модели; нет, по текущему расписанию.** `CellKey` включает `level: i8` (plan §5.3), `Layer.resolution`, «микровоксели — Voxel-слой с более тонким resolution» (§5.1), `RefinementRegion` — данные Core (C-12), brick edge 16, хранение разреженное (`CellState` различает `Present / KnownEmpty / NotLoaded…`). Выборочная детализация означает, что в области есть bricks на level>0, в остальной — нет. Требования «фиксированные базовые уровни с расширением» совместимы с `i8` (диапазон шире любого практического набора). Оговорки: (а) C-07/08/09 — Provisional (PC-1, нет Voxel Architecture); (б) переход между уровнями и максимум уровня — открыты (C-12); (в) полноценное Resolution — PS-9, поздно (R-3). Вывод: Core менять не надо; решать нужно расписание и минимальный режим.

**2. Не зависит ли Core от конкретного игрового контента?**
Нет. Core хранит `u16` и не знает ни одного материала; тест `dependency_rules.rs` запрещает engine-типы и лишние рёбра. Риск один — трактовка `u16` как идентичности (R-1).

**3. Где находятся контракты Content Registry?**
Предложение: Draft `docs/api-contracts/content-registry-contracts.md` (CR-01…CR-05, §3.10) и новый crate `cube-content`: без зависимостей от других crate; от него зависят worldgen, server, client (презентация), game-rules; **Core от него не зависит**, persistence — **не обязательно** (PackSet/IdMap как простые данные в WP-01, склейка в server). Решение о crate — OPN-04.

**4. Как совместить M2 с постоянными ID и отсутствующими пакетами?**
В M2 существует один пакет `cube` + тестовая фикстура. Формат M2 несёт: `GameVersion`, `PackSet` (`cube`, версия), `IdMaps` (материал: индекс → `cube:stone`…), опционально резерв для остальных видов. Обычная загрузка: проверка PackSet до применения данных; отсутствие/несовместимость → блок. Неизвестный материал: запись палитры сохраняется, индекс получает заглушку; запись обратно побайтно. Для не-воксельных доменов — непрозрачный payload. Тесты (проверяемые без модов): (a) мир с пакетами A+B загружается без изменений; (b) без B — отказ, файл побайтно не изменён; (c) round-trip заглушки; (d) детерминизм: тот же набор → те же реестры и id-карты. Режим восстановления M2 — минимум (заглушки + backup); замены/миграции — после M2. Для M2-T допустим экспериментальный формат (TC-08) без PackSet, но дешёвый резервный заголовок рекомендуется (ODM-03).

**5. Не нарушает ли система модов границы модулей?**
Не нарушает при условиях: (1) мод-эффекты на мир — только операции авторитетного runtime (Editing API, system-commit WorldGen, серверная симуляция по VP-02) — **мод-пути записи нет** (R-05); (2) публичная поверхность — только объявленные точки; внутренности Core, commit, persistence, network недоступны; проверка dependency-тестом/archcheck (по образцу `dependency_rules.rs`); (3) пакеты engine-neutral, Godot — слой презентации; (4) кода модов до технологического решения нет (AOD-12 запрещает процессы). Требует отдельного approval при реализации.

**6. Как редактирование, History и Persistence совместно обеспечивают целостность?**
Цепочка: `UserCommit` (единственный вход пользователя, MB-1) → CommitBatch с `expected` → журнал `IOperationJournal` (AOD-21) → снимки (протокол компакции WP-07) ← History читает через `ITransactionObserver`, Persistence — через `IDeltaStore`; они не импортируют друг друга. Для новых требований дополнительно нужно: (a) id-map append-only и durable не позже первой использующей записи (R-5); (b) операция, затрагивающая несколько доменов (геометрия + блок-сущность + экономика), — **одна атомарная группа** (R-7); (c) откат/Undo сохраняет ссылки по тем же id-картам; (d) история системных операций исключена из undo (AOD-20 (5)); для симуляции политика — OPN-09. Проверка: журнал-replay = живое состояние; `снимок+хвост = полный replay` (критерий AOD-21 уже есть) + тест «replay с отсутствующим пакетом даёт заглушки, не потерю».

**7. Не создают ли новые требования циклических зависимостей?**
Проверено скриптом сессии на графе Part B + предложенные рёбра `worldgen→content`, `server→content`, `client→content`, `game-rules→{core, processing, editing-api, content}`, `server→game-rules`: **новых циклов нет**; `content` не зависит ни от одного crate, `core→content` отсутствует. Единственный цикл в проверенной сетке — `orchestration ↔ history` — присутствует в самой Part B как пара **A**-рёбер (абстракция через L2-порт, note 1) и существует независимо от этого анализа; задачный граф ADG (147 задач) acyclic по `adg_check.py`. Оговорка: проверена только Part B как таблица, а не реальный граф crate (его пока нет; archcheck XT-01 не написан).

**8. Сохраняется ли независимость Rust Core от игрового клиента?**
Да. Предложения не добавляют зависимостей Core на что-либо; AOD-04/06 остаются. Клиент получает контент через presentation-слой, ключи — стабильные id. `cube-godot-adapter` в спайке нарушает план (зависимость на worldgen/persistence) — спайк, не production.

---

## 6. Зависимости (предлагаемые рёбра)

| Consumer | Provider | Тип | Обоснование |
|---|---|---|---|
| worldgen | content | D (read-only views) | биомы/структуры/материалы через реестр (AI-06) |
| server (composition root) | content, game-rules | D | загрузка пакетов, wiring |
| client | content | D (presentation keyed by id) | ассеты по стабильным id |
| game-rules (ODM-05, не утверждён) | core, processing, editing-api, content | D | правила, предметы, прогрессия |
| persistence | content | **нет** (по умолчанию) | PackSet/IdMap — простые данные WP-01 |
| core | content | **запрещено** | AOD-04 |

---

## 7. Минимальные изменения Production Plan (предложения, не применены)

| # | Изменение | Куда | Размер | Утверждает |
|---|---|---|---|---|
| M-1 | В D-C (GOV-01 №12): «`MaterialIndex` непрозрачен, ширина объявляется в формате» | `architecture-blocker-decisions.md` D-C | 1 фраза | owner |
| M-2 | В GOV-01 №5/ACR-004 A, D: набор источников коммита расширяем, резерв значения «симуляция мира» | там же | 1–2 строки | owner |
| M-3 | В D-T: поза подвижной конструкции вне `Layer.transform` | там же | 1 фраза | owner |
| M-4 | Строка `cube-content` в ADG Part A/B и задачи CONT-01…CONT-03 (Draft-контракты CR-01…05, базовая реализация, тест «нет определений вне загрузчика») в M2-T | `module-boundaries.md` | ≈3 задачи | Integration Owner → owner |
| M-5 | Draft-ревизия WP-01/WP-02/WP-08 (PackSet, IdMaps, GenerationIdentity, исходы, порядок id-map) | `world-persistence-contracts.md` | Draft | Integration Owner |
| M-6 | PERS: запись персонажа отдельным доменом (`PERS-0x`, место отсутствует) | ADG, FIRST_PLAYABLE_DEPENDENCIES | ≈1 задача | owner |
| M-7 | Требования к журналу: атомарная группа домена; требования к E-серии: крупные операции | `world-persistence-contracts.md`, `editing-contracts.md` | текст | owner при approval |
| M-8 | Решение об объёме PS-5/PS-6 (SDF/Mesh/Sculpt/Boolean) и о микровоксельном режиме для PSL-3 | `implementation-roadmap.md` | порядок/scope | owner |
| M-9 | Аннотация REF-16 (вариант a) | `docs/research/*` | 1 строка | owner |

**Не добавляется сознательно:** интерфейсы Mod SDK, script/WASM-хост, схемы типов контента, модуль физики, модуль энергии/логики, экономика, мультиклассы, плагины для representations (X-2 черновика: Core enum остаётся закрытым).

---

## 8. Расхождения документов (без исправления)

| # | Расхождение | Где |
|---|---|---|
| X-A | `AGENTS.md` ссылается на несуществующие `.agent/` и `.cursor/rules` и находится в `.gitignore` (ST-1/PB-7) | `AGENTS.md`, `.gitignore` |
| X-B | «Registry path»: план `docs/decisions/decision-registry.md`; другие документы иначе (F-2) | `PS-0-REMAINING-BLOCKERS.md` |
| X-C | `ps-0-exit-checklist.md` §5 — «READY FOR OWNER APPROVAL» vs метки пакета (F-3) | чек-лист |
| X-D | `PROJECT_STATUS.md` и аудит — на базе `2c0c1f3` (7 коммитов назад) | статус/аудит |
| X-E | Черновик extensibility не в git; `AGENTS.md`/`_build` — в незакоммиченном `.gitignore` | рабочее дерево |

---

## 9. Ограничения этого анализа

* Не запускались: `cargo build/test`, бенчмарки, CI (его нет). Результаты §1 п.9 — запуск существующего проверочного скрипта ADG; §5.7 — скрипт этой сессии по таблице Part B.
* Документы «Voxel Architecture», «Geometry Processing Specification», GDD/TDD отсутствуют (B-1) — выводы по C-07…C-12 и PR-01…08 остаются при условии AOD-24.
* «Блокер» здесь — только то, без чего этап нельзя начать; список — в `implementation-roadmap-alignment.md` §8.
