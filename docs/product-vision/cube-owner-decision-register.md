# Cube — Owner Decision Register (консолидирующий слой, Proposed)

> **Erratum 2026-11-09 (decision registry now exists: `docs/decisions/decision-registry.md`).** (1) Statements here that unsigned commits mean 'no decision can be confirmed' are withdrawn: DEC-002 accepts an explicit owner chat statement as a source and makes signatures optional. (2) CPR-PRN-05…08 are not confirmed decisions: the owner stated them as context; they are `RECORDED` (DEC-052). CPR-PRN-01…04 repeat AOD-01…07 (`RECORDED`). (3) AOD-25/AOD-26 are **not assigned** as bundles; the content is split into DEC-030…DEC-035 (CONCEPT, CONFIRMED from the owner's message) and DEC-053/DEC-054 (RECORDED / PROPOSED). (4) 'M2' in CPR-SAV-01 is read as 'the first approved save format'; working terms: **M2-T** = technical slice (walk, edit, save/load; the old M2 of DEVELOPMENT_ROADMAP), **M2** = First Playable with the gameplay loop (Game Track); ODM-06 still fixes their acceptance. (5) Priority, stage, module and 'related decisions' columns are agent annotations, not confirmations; only the `statement` of a registry entry is the owner's wording. (6) 'owner-signed' means an explicit owner statement, PR approval or owner-written text; no cryptographic signature is required (DEC-002).

**Статус:** **Proposed.** Это **не** `decision-registry.md` из плана §3.3 (тот файл не существует). Документ только классифицирует и сопоставляет; он не создаёт, не изменяет и не подтверждает решений. Существующие записи (AOD, OAM, ACR, OD/ODM, D-xx, GOV-01) **не перенумерованы и не изменены**; история утверждений не переписывалась.
**База:** ветка `docs/architecture-roadmap-v1`, HEAD `16db379`, 2026-11-09.

---

## 0. Классы записей

| Класс | Метка | Смысл | Кто может присвоить |
|---|---|---|---|
| A | **OWNER-CONCEPT** | концептуальное подтверждение владельца в чате (OC-2026-11-09); не утверждает API, алгоритмы, форматы, ACR | только владелец (агент записывает) |
| B | **AGENT-PROPOSAL** | предложение агента (VP-xx ниже); не имеет силы | агент |
| C | **OWNER-TECHNICAL** | техническое решение, утверждённое владельцем: AOD-01…23 (Approved), AOD-24 (Conditionally Approved), OAM-01/02/03/07 (Approved) | владелец |
| D | **PENDING** | ожидает решения/подтверждения (ACR-001…004, OAM-04/05/06/09/10, GOV-01 №1…15, ODM-xx, ST-1/2, X-1…X-4) | владелец |
| E | **SUPERSEDED / ANNOTATED** | устаревшее или частично заменённое | только владелец; здесь только предлагаемые пометки |

### 0.1 Оговорка о подтверждении (проверено)

* `docs/decisions/decision-registry.md` отсутствует; `.github` отсутствует; все **43** коммита ветки имеют `%G? = N` (не подписаны), автор один (`Godrus`).
* Следовательно, по правилу плана §3.3 **ни одна запись пока не может быть `CONFIRMED` реестром**. Записи класса C — «Approved по инструкции владельца в чате, ожидают контрподписи» (формулировка самого `owner-architecture-decisions-v1.md`). Класс A — тем более концепция из чата.
* Решение ODM-10 (механизм подтверждения) — предпосылка для превращения любого класса A/C в запись реестра.

---

## 1. Класс A: концептуальные подтверждения OC-2026-11-09

Одна строка = одна область сообщения владельца. Детализация по отдельным требованиям — `cube-product-vision.md` §2.

| ID | Область | Требования | Относится к существующим решениям | Нужна ли техническая ратификация |
|---|---|---|---|---|
| OCC-01 | Принципы (раздел 1) | CPR-PRN-01…08 | AOD-01…07, 18, 24 (дублирует Approved, кроме PRN-05/06/07) | PRN-05, 06, 07 — новые формулировки уровня продукта; AOD для них нет |
| OCC-02 | Контент и моды 2.1 | CPR-CON-01…11 | чернов. AOD-25 (S-1…S-8, R-01…R-09); AOD-04, 06, 07, 12 | да: AOD-25 (не присвоен), OPN-01…05 |
| OCC-03 | Сохранения 2.2 | CPR-SAV-01…09 | чернов. AOD-26 (R-10…R-17); AOD-17, 21; WP-01, WP-08 (Draft) | да: AOD-26, OPN-03, OPN-06 |
| OCC-04 | Строительство и мир 2.3 | CPR-BLD-01…14 | AOD-15, 16, 17, 20, 21; ACR-003; C-07…C-12; D-K, D-T, D-C; E-02 | частично: OPN-07…OPN-15 |
| OCC-05 | Мир и симуляция 2.4 | CPR-SIM-01…07 | AOD-13, 17, 21; WP-01 | нет немедленной; OPN-09, 16 |
| OCC-06 | Механизмы 2.5 | CPR-MEC-01…08 | S-1, S-6 | нет (всё отложено) |
| OCC-07 | RPG 2.6 | CPR-RPG-01…11 | ODM-05 (Game Rules module), REF-13 | нет (вне technology-track до M3) |
| OCC-08 | Мультиплеер и экономика 2.7 | CPR-MP-01…08 | AOD-13, 14, 22; OAM-05; «account/MMO model» (не решена) | OPN-13 (свойства журнала) |

**Что НЕ утверждено в классе A (явно):** ширина числовых ID; грамматика `namespace:name`; формат manifest/id map; схема версий; число уровней микровокселей; класс входа симуляции; любой алгоритм, формат, API, ACR.

---

## 2. Соответствие черновику `owner-decision-extensibility-v1.md`

Черновик помечал большинство R-xx как PROPOSED. После OC-2026-11-09 прогресс статусов (текст черновика **не изменялся**; это предложение пометок для владельца):

| Пункт черновика | Прежний статус | Что даёт OC-2026-11-09 | Предлагаемая отметка |
|---|---|---|---|
| S-1…S-7 | CONFIRMED | подтверждено повторно (2.1, 2.5) | без изменений |
| S-8 `cube` как пакет | PROPOSED | CPR-CON-01 | CONCEPT-CONFIRMED |
| R-01 единая модель реестров | PROPOSED | CPR-CON-02 | CONCEPT-CONFIRMED |
| R-02 `namespace:name` | PROPOSED | CPR-CON-03, 05 | CONCEPT-CONFIRMED (грамматика — OPEN) |
| R-03 runtime ID не идентичность | PROPOSED | CPR-CON-04, SAV-03 | CONCEPT-CONFIRMED (ширина — OPEN) |
| R-04 явная поверхность расширений | PROPOSED | CPR-CON-11 | CONCEPT-CONFIRMED |
| R-05 мод-изменения мира только через авторитетные пути | PROPOSED | CPR-CON-11 (частично) | остаётся PROPOSED |
| R-06 engine neutrality | PROPOSED | CPR-PRN-02/03 | CONCEPT-CONFIRMED |
| R-07 пакеты — данные, без исполнения кода | PROPOSED | не упомянуто | остаётся PROPOSED |
| R-08 манифест пакета | PROPOSED | CPR-CON-07 (зависимости, версии) | CONCEPT-CONFIRMED (поля — OPEN) |
| R-09 детерминированный порядок загрузки | PROPOSED | CPR-CON-08 | CONCEPT-CONFIRMED |
| R-10, R-12 | CONFIRMED | CPR-SAV-02, 04 | без изменений |
| R-11 таблицы id в мире | PROPOSED | CPR-SAV-03 | CONCEPT-CONFIRMED |
| R-13 «без молчаливой потери» | CONFIRMED (механизм PROPOSED) | CPR-SAV-06, 09 | без изменений |
| R-14 блокировка обычной загрузки | PROPOSED | CPR-SAV-04 | CONCEPT-CONFIRMED |
| R-15 восстановление только явное, backup | PROPOSED | CPR-SAV-05 (режим), backup не упомянут | режим — CONCEPT-CONFIRMED; backup остаётся PROPOSED |
| R-16 миграции при обновлении мода | PROPOSED | CPR-SAV-08 (мягко) | остаётся PROPOSED |
| R-17 идентичность генерации включает пакеты | PROPOSED | не упомянуто | остаётся PROPOSED |
| Q-1 (S-8) | OPEN | CPR-CON-01 | отвечено концептуально |
| Q-2 (`ns:name`, id maps) | OPEN | CPR-CON-03/04, SAV-03 | отвечено концептуально |
| Q-3 (блокировка при отсутствии) | OPEN | CPR-SAV-04 | отвечено |
| Q-4 (игра с заглушками) | OPEN | CPR-SAV-05 (только в режиме восстановления) | отвечено частично (после backup — не подтверждено) |
| Q-5 (замены чужих id) | OPEN | — | остаётся OPEN |
| Q-6 (semver, диапазоны) | OPEN | CPR-SAV-07 (диапазоны) | частично; semver — OPEN |
| Q-7 (правки статов без подтверждения) | OPEN | — | OPEN |
| Q-8 (формально как AOD-25/26?) | OPEN | — | OPEN |
| Q-9 (REF-16) | OPEN | — | OPEN → OPN-17 |
| Q-10 (формат M2 несёт PackSet) | OPEN | CPR-SAV-01 | отвечено концептуально («да») |

---

## 3. Класс C: существующие технические решения, затронутые новыми требованиями

Состояние **не изменено**. Колонка «Влияние» — только анализ.

| ID | Статус по источнику | Влияние OC-2026-11-09 |
|---|---|---|
| AOD-04 Core pure Rust, без Godot | Approved | подтверждается; AI-01: Core контент-агностичен |
| AOD-06/07 Godot заменяем, GDScript — glue | Approved | CPR-CON-11, R-06: пакеты не зависят от Godot |
| AOD-12 нет child processes | Approved | ограничивает будущий код-хост модов |
| AOD-13/14 сервер авторитетен | Approved | основа CPR-MP-05/08 |
| AOD-15 пользовательские изменения — через Editing API | Approved | AI-09: изменения от симуляции вне формулировки «user-originated» |
| AOD-16/20 WorldGen — отдельная граница (C3) | Approved (ACR-004 Pending) | AI-09: ровно один System-вход, у WorldGen |
| AOD-17/21 seed + GeneratorVersion + журнал + снимки | Approved (форматы не утверждены) | CPR-SAV-*: PackSet/IdMaps/GenerationIdentity входят в запись мира; AI-04 порядок долговечности |
| AOD-18 P0/P1 контракты остаются Draft | Approved | все AI/VP — Draft-уровень |
| AOD-19 AU-1.0, ACR-001/002/003 не переписываются | Approved | ничего не переписано |
| AOD-22 одинаковый runtime embedded/dedicated | Approved | локальный игрок — тоже сессия (SR-06/08); записи персонажа (AI-15) |
| AOD-23 источники/лицензии | Approved | паки сторонних модов — отдельная тема лицензирования (DF-18) |
| AOD-24 отсутствующие источники | Conditionally Approved | PC-1 (C-07/08/09) остаётся Provisional; микровоксели опираются на Provisional-базу |
| OAM-09 C1/C2 | Recommended / Requires Approval | AI-11: временные изменения без C2 — аргумент за C1 |
| OAM-10 модель ревизий system commits | Proposed | OPN-09: добавить `Simulation`/`World` к набору источников — тот же вопрос |

---

## 4. Класс D: ожидающие решения, на которые новые требования влияют

| ID | Ожидает | Влияние OC-2026-11-09 | Усиливает срочность? |
|---|---|---|---|
| GOV-01 №10 D-K (ключ, ширины, edge 16, уровень i8) | owner | CPR-BLD-11 «нет произвольного предела»: ширина `i32` ограничивает мир (ODM-11); `level: i8` — задел микровокселей | нет (уже блокер GOV-01) |
| GOV-01 №11 D-T (трансформы) | owner | AI-13: непрерывная поза — вне `Layer.transform` | нет; нужна одна фраза-уточнение |
| GOV-01 №12 D-C (каналы, `Material(u16)`) | owner | AI-01/OPN-01: объявить индекс непрозрачным и заявить ширину в формате | **да, как уточнение** |
| GOV-01 №5 revision principle (OAM-10, ACR-004 D) | owner | OPN-09 | **да, как уточнение** |
| ACR-001 foundation types in Core | owner | без влияния | нет |
| ACR-002 ports in L2 | owner | без влияния | нет |
| ACR-003 composite data in Core | owner | CPR-BLD-14 (подвижные конструкции) использует Composite-данные: ACR-003 — желателен | нет |
| ACR-004 A–F | owner | AI-09: часть A (Core prerequisites, `ActorKind`) | уточнение, не новое решение |
| OAM-05 сеть | spike | без влияния на PS-1 | нет |
| ODM-05 Game Rules module | owner | CPR-RPG-*, MP-*, MEC-* целиком в Game Rules | нет (нужно до `GM-10`) |
| ODM-10 механизм подтверждения | owner | без него OC-2026-11-09 не попадает в реестр | уже блокер |
| ODM-11 граница мира/VoxelPos | owner | CPR-BLD-11, PRN «бесконечный мир» | уже блокер для `CORE-02` |
| ODM-13 формат данных контента | DEFERRED | CPR-CON-*: минимум манифеста нужен раньше `GM-20` — **только** при утверждении первого формата сохранения | да, граничное условие |
| ODM-16 локальная авторитетная модель | PROPOSED | AI-15: локальный игрок и запись персонажа | нет |
| ODM-03 PC-1 и экспериментальный формат | owner | CPR-SAV-01: экспериментальный формат M2-T (TC-08) одноразов; формат **M2** обязан нести PackSet | граничное условие |

---

## 5. Класс B: предложения агента (VP-xx) — все **Proposed**, ни одно не утверждено

| ID | Предложение | Мотив (реальная потребность) | Цена, если отложить | Необходимое утверждение |
|---|---|---|---|---|
| VP-01 | В D-C записать: значение материала в Core — непрозрачный `MaterialIndex`, ширина объявляется в заголовке формата; идентичность живёт выше Core | иначе `u16` станет «идентичностью» (спайк уже так делает) | перепись формата сохранения | owner (GOV-01 №12) |
| VP-02 | Объявить набор источников коммита (`ActorKind`) расширяемым и зарезервировать значение для «симуляции мира» | CPR-SIM-*, MEC-01, BLD-08 | изменение сохраняемого enum журнала | owner (GOV-01 №5 / ACR-004 A, D; OAM-10) |
| VP-03 | Одна фраза в D-T: непрерывная поза подвижной конструкции — состояние сущности/физики, не `Layer.transform` | CPR-BLD-14 | спор о float в Core позже | owner (GOV-01 №11) |
| VP-04 | Новый Draft `content-registry-contracts.md` (CR-01…CR-05) — см. alignment §3.10 | CPR-CON/SAV | формат сохранения без места для PackSet | Integration Owner → owner при регистрации Provisional |
| VP-05 | Расширить WP-01 полями PackSet / IdMaps / GameVersion / GenerationIdentity; исходы `PackMissing`, `PackIncompatible`, `UnknownDefinition` | CPR-SAV-* | то же | Draft-ревизия (AOD-18: без утверждения) |
| VP-06 | Правило порядка: запись id-map durable не позже первой journal-записи, использующей её | AI-04 | потеря соответствия id ↔ индекс при аварии | owner с утверждением WP-07/08 |
| VP-07 | Запись персонажа — отдельный домен сохранения (не внутри журнала мира) | CPR-MP-02/03/04 | сложный перенос/несколько персонажей | owner (при PERS-запись игрока) |
| VP-08 | Атомарная группа записей в журнале (несколько доменов под один OperationId) как свойство, не реализация | CPR-MP-08, BLD-10 | перепись журнала при экономике/блок-сущностях | owner при утверждении формата журнала |
| VP-09 | Решение об объёме SDF/Mesh/Sculpt/Boolean — после First Playable, Core-enum `Representation` без изменений | CPR-BLD-04 | время/деньги на PS-5/PS-6 | owner (OPN-07) |
| VP-10 | Минимальный микровоксель для PSL-3: фиксированные уровни; клетка либо грубая, либо разбита; без resampling-перехода | CPR-BLD-02/03 | M3 ждёт PS-9 | owner (OPN-08) |
| VP-11 | Минимум манифеста пакета (id, версия, зависимости, диапазоны) при утверждении формата сохранения; остальное — `GM-20` | CPR-CON-07 | формат без идентичности | owner (OPN-02/03) |
| VP-12 | Тест «ничего не создаётся вне загрузчика» (встроенный контент идёт тем же путём) | S-8, риск «две архитектуры» | дрейф к двум системам | Integration Owner |

---

## 6. Класс E: устаревшее, заменённое, аннотируемое

| Запись | Что | Предложение (не применено) |
|---|---|---|
| REF-16 «Rejected (out of scope)» (`reference-comparison.md:40`, `veloren-notes.md:86`) | основание «моды вне scope» заменено OC-2026-11-09 (моды — архитектурное требование); объект (WASM-плагины сейчас) остаётся не принятым (S-6) | вариант (a) черновика X-1: «Rejected for the current roadmap; rationale superseded; WASM = candidate» — решение владельца |
| `AGENTS.md` ссылается на `.agent/` и `.cursor/rules`, которых нет; файл в `.gitignore` | расхождение «источник состояния» (ST-1, PB-7) | решение владельца по ST-1 |
| `ps-0-exit-checklist.md` §5 «READY FOR OWNER APPROVAL» | ярлык вердикта расходится (F-3 PS-0-REMAINING-BLOCKERS) | косметика |
| ODM-13 «content data format — DEFERRED» | остаётся, но с граничным условием из §4 | уточнение при регистрации |

**Ничего не объявлено устаревшим автоматически.**

---

## 7. Нумерация

* Новые ID этого слоя: `OCC-nn`, `VP-nn`, `CPR-/AI-/DF-/OPN-` (в `cube-product-vision.md`). Конфликтов с существующими префиксами нет (проверено: `OD-R*`, `ODM-*`, `D-*`, `AOD-*`, `OAM-*`, `ACR-*`, `GOV-*` не использованы).
* AOD-25 и AOD-26 **не присвоены** (предложены в черновике extensibility; присваиваются при подтверждении владельцем, правило §4 `owner-architecture-decisions-v1.md`).
* Если владелец решит зарегистрировать OCC-02/OCC-03 как AOD-25/26 — это отдельное действие владельца; ничего здесь не нужно переписывать.
