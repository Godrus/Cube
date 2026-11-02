> **Repository provenance notice — status: PROPOSED / DRAFT.**
> This file is a copy reconstructed from a chat message. The original approved document was not available to the repository, and the copy has not been verified byte-for-byte. The decisions below are treated as the owner's approved decisions only to the extent the owner confirms this copy. Resolution: owner supplies the original file or confirms this one. The text below the line is unchanged from the message.

---

# Cube — Geometry Architecture Decisions & Audit Resolution
## Architecture Update v1.0

**Статус:** Architecture Decisions Approved  
**Назначение:** Ответ на Architecture Audit главного агента и дополнение существующей технической документации Cube.  
**Область:** Geometry Core, Geometry Editing, Processing, Storage, Concurrency, Persistence, Networking и Integration.

---

# 1. Назначение документа

Этот документ фиксирует согласованные архитектурные решения, принятые после аудита репозитория главным агентом.

Документ должен использоваться совместно с существующими материалами:

1. Geometry Editing Specification v1.0.
2. Cube Agent Development Specification v1.0.
3. Geometry Editing Agent Task Pack v1.0.
4. Geometry Editing Integration & QA Pack v1.0.
5. Voxel Architecture v1.0.
6. GDD/TDD v0.3.

Цели:

- Закрыть архитектурные вопросы из первоначального аудита.
- Устранить противоречия между модулями.
- Уточнить владельцев модулей.
- Установить однонаправленную архитектуру зависимостей.
- Формализовать принципы Geometry Core и Processing.
- Определить требования к будущим API Contracts.
- Подготовить основу для распределённой разработки агентами.

**Важно:** документ утверждает архитектурные принципы и ограничения, но не утверждает окончательные language-specific API signatures.

# 2. Module Ownership

Каждый модуль имеет одного ответственного владельца.

| Module | Owner |
|---|---|
| Geometry Core | Core Agent |
| Processing | Processing Agent |
| Editing API & Contracts | Main Agent / Integration Owner |
| Editing Orchestration | Agent A |
| Voxel Backend | Agent B |
| SDF Backend | Agent C |
| Mesh Backend | Agent D |
| Sculpting Tools | Agent E |
| Boolean Operations | Agent F |
| History System | Agent G |
| Persistence Integration | Agent H |
| QA & Benchmarks | Agent I |
| Composite Geometry | Composite Agent |
| Resampling & Resolution Refinement | Resolution Agent |

## 2.1 Ownership Rules

1. Каждый модуль имеет одного владельца.
2. Владелец отвечает за реализацию, тесты и документацию своего модуля.
3. Integration Owner отвечает за архитектурную согласованность.
4. Изменение публичных контрактов требует согласования.
5. Агенты не изменяют внутреннюю архитектуру соседних модулей самостоятельно.
6. Каждый модуль интегрируется отдельно после прохождения тестов.
7. QA не является runtime dependency.
8. Отсутствие реализации модуля не разрешает другим агентам создавать его скрытую замену.

# 3. Dependency Architecture

Устанавливается строго однонаправленная архитектура зависимостей.

Основные правила:

- Нижние архитектурные слои не зависят от верхних.
- Циклические зависимости запрещены.
- Межмодульное взаимодействие выполняется через API Contracts.
- Geometry Core не зависит от движка, UI, сети и Persistence.
- Processing не содержит игровой или редакторской бизнес-логики.
- Persistence не управляет редактированием.
- History не реализует собственное Brick Storage.
- Geometry Backends не вызывают Editing Tools.
- Composite Geometry не объединяет части автоматически.
- QA не подключается к runtime.

## 3.1 Основные архитектурные слои

1. Geometry Core.
2. Geometry Processing.
3. Editing API & Contracts.
4. Editing Orchestration.
5. Geometry Backends.
6. Editing Tools.
7. History System.
8. Composite Geometry.
9. Persistence Integration.
10. Networking Adapter.
11. Engine Integration Adapters.
12. QA & Benchmarks.

Эта последовательность задаёт концептуальную архитектуру, а не окончательную dependency matrix.

Integration Owner обязан подготовить отдельную матрицу зависимостей, где явно указаны:

- Provider.
- Consumer.
- Interface.
- Direction.
- Ownership.
- Data Lifetime.
- Revision Requirements.
- Threading Requirements.

## 3.2 History / Persistence Boundary

Циклическая зависимость History и Persistence должна быть устранена.

History отвечает за:

- Commands.
- Deltas.
- Undo/Redo.
- Checkpoints.
- Историю операций.

Persistence отвечает за:

- Сохранение.
- Загрузку.
- Manifest.
- Brick Storage.
- Incremental Deltas.
- Durable Snapshots.

Взаимодействие организуется через независимые абстракции хранения и сериализации.

History не импортирует внутренние классы Persistence.

Persistence не импортирует внутренние классы History.

Общие структуры данных и контракты размещаются в подходящем нижнем слое.

# 4. Coordinate Architecture

## 4.1 Unified Coordinate System

Устанавливается единая система координат:

- World Coordinates.
- Asset Coordinates.
- Layer Coordinates.
- Brick Coordinates.
- Local Voxel Coordinates.

Все Geometry Layers одного Asset используют общий пространственный базис.

Resolution не является частью Transform.

Поддерживаются отрицательные координаты и Floating Origin.

## 4.2 Precision

Используется Integer + Float Hybrid.

- World/Brick/Local Voxel addressing — integer.
- Local Geometry — floating point.
- Transform — float или double в зависимости от требований.
- Вычисления используют точность, соответствующую задаче.
- Serialization сохраняет точные integer coordinates.

Конкретные типы i32, i64, f32 и f64 требуют технической оценки.

## 4.3 Coordinate Handedness

Используется Right-Handed Coordinate System:

- X — вправо.
- Y — вверх.
- Z — вперёд.
- Rotation соответствует правилу правой руки.

Импорт внешних моделей с другой системой координат выполняет необходимое преобразование winding и normals.

## 4.4 World Units

1 World Unit = 1 meter.

Physics, Collision, Geometry и внешние модели используют согласованные единицы.

Численные допуски определяются с учётом масштаба и precision.

# 5. Transform Architecture

## 5.1 Transform Representation

Утверждён Hybrid TRS + Matrix.

Основное представление:

- Translation — Vector3.
- Rotation — Quaternion.
- Scale — Vector3.

Matrix 4×4 используется как производное представление и кэш.

Дополнительные Affine Transformations разрешены для специальных случаев.

Обязательные ограничения:

- Quaternion нормализуется по установленной политике.
- Singular Matrix контролируется явно.
- Shear не должен теряться при неявной декомпозиции в TRS.
- Negative Scale требует корректной обработки winding и normals.
- Unsupported Transform возвращает типизированную ошибку или требует явного Bake.

Transform хранится отдельно от Geometry Data.

Изменение Transform не должно перестраивать исходные Brick.

## 5.2 Geometry Bounds

Утверждена Hybrid Bounds Architecture.

Поддерживаемые типы:

- Local AABB.
- World AABB.
- OBB.
- Bounding Sphere.

Local AABB является основной локальной границей Geometry Layer.

Остальные представления являются производными.

Предусмотрено состояние Empty Bounds.

## 5.3 Bounds Ownership

Geometry Core:

- Определяет Bounds Types.
- Предоставляет публичные контракты.
- Выполняет базовые синхронные операции.
- Проверяет корректность Bounds.

Processing:

- Вычисляет производные Bounds асинхронно.
- Обновляет World AABB, OBB и Bounding Sphere.
- Обновляет BVH и пространственные структуры.
- Проверяет Revision.
- Отбрасывает устаревшие результаты.

Geometry Core не зависит от Processing.

# 6. Geometry Resolution

Утверждён Hybrid Resolution Profile.

- Основное значение — Voxel Size в метрах.
- Voxels Per Unit — производная величина.
- Каждый Layer имеет собственный Resolution Profile.
- Refinement Levels определяют иерархию уточнения.
- Refinement Regions / Bricks обеспечивают локальное повышение разрешения.
- Базовый Brick остаётся 16³.
- Refinement не меняет базовый размер Brick.

Voxel, SDF и Mesh Backends могут иметь собственные параметры дискретизации.

Общая пространственная система координат сохраняется.

Resampling требует явной политики сохранения геометрии, материалов и атрибутов.

# 7. Revision Architecture

Утверждена Hierarchical Revision System.

Поддерживаются:

1. Asset Revision.
2. Layer Revision.
3. Brick Revision.
4. Region Revision.
5. Dependency Revision.
6. Operation ID.

Operation ID идентифицирует операцию, но не заменяет Revision.

## 7.1 Revision Rules

- Revision увеличивается при фактическом изменении соответствующих данных.
- Изменение одного Brick не увеличивает Revision всех остальных Brick без необходимости.
- Snapshot фиксирует согласованный набор Revision.
- Processing проверяет актуальность всех зависимостей.
- Устаревшие асинхронные результаты отбрасываются.
- Revision Validation применяется при интеграции, сохранении и сетевой синхронизации.

## 7.2 Atomicity

Используется Hybrid Concurrency Model:

- Optimistic Concurrency по Revision.
- Атомарные Editing Transactions.
- Временные блокировки для тяжёлых конфликтных операций.
- Region-based concurrency.
- Проверка областей чтения и записи.

Точные правила Revision Token, compare-and-swap, commit и conflict resolution должны быть разработаны в API Contracts.

# 8. Geometry Region Model

Утверждена Hybrid Region Architecture.

Region представляет логическую область операции.

Brick остаётся единицей хранения.

Processing определяет фактически затронутые Brick и дополнительные зависимости.

Region не обязан совпадать с границами одного Brick.

Операции должны учитывать:

- Read Region.
- Write Region.
- Dependency Region.
- Processing Region.

Расширение Processing Region по радиусу зависимости алгоритма не должно автоматически расширять Dirty Region.

# 9. Geometry Snapshot Architecture

Утверждён Hybrid Immutable Snapshot + Copy-on-Write.

Snapshot содержит:

- AssetId.
- LayerId.
- Source Revision.
- Brick References / Snapshots.
- Dependency Revisions.
- Operation Context при необходимости.

Основной механизм — immutable snapshots с Copy-on-Write.

Для MVP разрешается копирование небольших затронутых областей, если это упрощает корректную реализацию.

Snapshot должен обеспечивать согласованное состояние данных на момент создания.

Фоновые задачи не должны читать изменяющиеся данные без гарантии стабильности.

# 10. Dirty Tracking

Утверждён Hybrid Dirty Tracking.

- Dirty Regions отслеживаются внутри Brick.
- Используется Region-level tracking.
- Изменения классифицируются по ChangeType.
- Processing Region может расширяться для зависимых вычислений.

ChangeType включает:

- Geometry.
- Material.
- Attribute.
- Transform.
- Topology.
- Resolution.

Инвалидация применяется к соответствующим производным данным:

- Surface Mesh.
- Collision.
- Gameplay Hitbox.
- LOD.
- Bounds.

Не все изменения обязаны инвалидировать все кэши.

# 11. Dependency Graph

Утверждён Hybrid Dependency Graph.

Используются:

1. Статические зависимости между модулями.
2. Динамический граф задач и производных данных.

Граф должен поддерживать:

- Dependency Registration.
- Dependency Invalidation.
- Revision Validation.
- Async Scheduling.
- Cancellation.
- Stale Result Rejection.

Циклы в runtime processing dependencies должны контролироваться.

Processing отвечает за выполнение графа вычислений, но не содержит бизнес-логику редактирования.

# 12. Threading & Job Scheduling

Утверждён Hybrid Task-based Scheduler.

Требования:

- Priority Queue.
- Dependency-aware Scheduling.
- Cancellation.
- Resource Limits.
- Main-thread Budget.
- Platform-specific Scheduling Policies.
- Starvation Prevention.
- Асинхронная обработка тяжёлых операций.

Работы распределяются с учётом близости, важности, видимости и срочности.

Длительные задачи не блокируют игровой поток.

Численные Performance Budgets определяются на прототипах и целевом оборудовании.

# 13. Geometry API & Ownership

Утверждён Hybrid Strict Public API.

- Внешние модули используют публичные API Contracts.
- Доступ к внутренним данным допускается только через ограниченные доверенные backend interfaces.
- Ownership и lifetime данных должны быть определены явно.
- Изменение данных проходит через контролируемый Editing Pipeline.
- Прямой доступ не должен обходить Dirty Tracking, Revision и Dependency Graph.

Публичные контракты должны включать:

- Inputs.
- Outputs.
- Ownership.
- Lifetime.
- Error Model.
- Revision Semantics.
- Threading.
- Cancellation.
- Compatibility.
- Data Validation.

# 14. Error Handling

Утверждён Hybrid Error Handling.

Основной механизм ожидаемых ошибок — Result / Error.

Исключения допускаются для непредвиденных критических ситуаций.

Ошибки должны содержать:

- Stable Error Code.
- Context.
- Module.
- Operation ID, если применимо.
- Recovery Policy.
- Severity.

Ошибки должны быть типизированными.

Неподдерживаемая операция и некорректные входные данные не должны приводить к неопределённому поведению.

# 15. Geometry Serialization

Утверждён Hybrid Persistence Architecture.

Структура:

1. Asset Manifest.
2. Chunked Brick Storage.
3. Incremental Deltas.
4. Snapshots / Checkpoints.
5. Content Hash.
6. Deduplication.
7. Atomic Save.
8. Recovery.

Runtime Storage и Serialized Storage разделены.

Manifest хранит метаданные Asset, Layers, Resolution Profiles, Material References и Dependencies.

Brick сохраняются независимо.

Composite Geometry сохраняет структуру Parts и Assembly Graph.

Manifest не должен ссылаться на отсутствующие или некорректные Brick после прерванного сохранения.

Сохранение выполняется асинхронно.

# 16. Versioning & Migration

Утверждён Hybrid Versioning.

- Schema Versioning.
- Sequential Migrations.
- Compatibility Validation.
- Optional Feature Preservation.
- Required Feature Validation.

Неизвестные Optional Features должны сохраняться без интерпретации, если формат позволяет.

Неизвестные Required Features приводят к контролируемой ошибке.

Миграции должны быть тестируемыми и воспроизводимыми.

Автоматическое преобразование не должно молча терять данные.

# 17. Import / Export

Утверждён Hybrid Import / Export Pipeline.

Native Geometry Format остаётся основным внутренним форматом.

Импорт и экспорт реализуются отдельными адаптерами.

Pipeline должен включать:

- Coordinate Conversion.
- Unit Conversion.
- Transform Conversion.
- Material Mapping.
- Attribute Mapping.
- Geometry Validation.
- Capability Validation.
- Loss Reporting.

Потери данных должны обозначаться явно.

Не поддерживаемые форматы или возможности должны приводить к контролируемой ошибке.

# 18. Geometry Validation & Integrity

Утверждён Hybrid Validation Architecture.

Уровни:

1. Fast Validation при операциях.
2. Deep Validation асинхронно.
3. Integrity Checks при сохранении.
4. Integrity Checks при загрузке.

Validation должен проверять:

- Geometry Data.
- Brick Integrity.
- Channel Integrity.
- Material References.
- Transform Validity.
- Bounds Validity.
- Revision Consistency.
- Manifest References.
- Dependency Consistency.
- Serialization Integrity.

Предусматриваются Severity Levels и Recovery Policies.

# 19. Engine Independence

Утверждена Hybrid Engine Integration Architecture.

Geometry Core полностью независим от игрового движка.

Внешние адаптеры обеспечивают интеграцию с:

- Rendering.
- Physics.
- Memory.
- Job Systems.
- Platform-specific facilities.

Core не использует напрямую Unity, Godot, Unreal или другие движковые API.

Выбор движка остаётся открытым.

# 20. Networking Boundary

Утверждён Hybrid Networking Adapter.

Networking располагается над Editing API.

Geometry Core не зависит от сетевого транспорта.

Требования:

- Server-authoritative Editing.
- Authorization Context.
- Operation ID.
- Revision Validation.
- Region-based Concurrency.
- Optimistic Client Preview.
- Reconciliation.
- Confirmed Operation Journal.
- Reconnect Recovery.
- Data-based Synchronization.

Сеть передаёт изменения геометрических данных и операций, а не готовые render meshes в качестве источника истины.

Сервер обязан проверять авторизацию и актуальность Revision.

Конкретный сетевой протокол и транспорт пока не утверждены.

# 21. Existing Architecture Decisions

Все ранее согласованные решения остаются действующими, если прямо не противоречат настоящему документу.

В частности:

- Hybrid Voxel + SDF + Polygon Mesh.
- Geometry Asset с независимыми Geometry Layers.
- Flat Layers + Composite Hierarchy.
- Built-in Layer Types и Registered Extension Types.
- Adaptive Attribute Channels.
- Global Material Library + Asset Palette + Layer Mapping.
- Typed Attribute Registry.
- Sparse Brick Storage.
- Brick Dimensions 16³.
- Hash Map для прямого доступа.
- Sparse Octree для пространственной организации.
- BVH для запросов пересечения и collision.
- SoA + AoS Hybrid.
- Adaptive Material ID Width.
- Adaptive Occupancy.
- Adaptive SDF Precision.
- Surface Nets для SDF.
- Face Culling + Greedy Meshing для Cubic Voxel.
- Explicit Geometry Junctions.
- Hybrid Collision Architecture.
- Hybrid Editing API.
- Hybrid History.
- Hybrid Composite Geometry.
- Hybrid Boolean Operations.
- Hybrid Resampling.
- Layered Modular Architecture.
- Central Integration.
- Comprehensive Testing Architecture.

# 22. Resolution of Initial Audit Findings

| Audit Finding | Required Resolution |
|---|---|
| Missing Geometry Core owner | Core Agent назначен |
| Missing Processing owner | Processing Agent назначен |
| Missing Composite owner | Composite Agent назначен |
| Missing Resampling owner | Resolution Agent назначен |
| Missing Processing responsibilities | Зафиксированы |
| Processing stage placement | Пересмотреть dependency matrix с учётом его базовой роли |
| History ↔ Persistence cycle | Устранить через независимые абстракции |
| Orchestration dependencies | Уточнить в контрактах |
| Task Pack assumes existing Core | Исправить порядок разработки |
| Missing Transform contract | Добавить |
| Missing Bounds contract | Добавить |
| Missing Region contract | Добавить |
| Missing ModifierStack contract | Добавить |
| Missing Junction contract | Добавить |
| Missing RefinementRegion contract | Добавить |
| Missing BrickStorage contract | Добавить |
| Missing Delta contract | Добавить |
| Missing Checkpoint contract | Добавить |
| Missing AuthorizationContext | Добавить |
| Missing OperationId | Добавить |
| RevisionToken semantics undefined | Разработать и согласовать |
| Engine not selected | Оставить открытым решением |
| Language not selected | Оставить открытым решением |
| Networking not specified | Подготовить отдельные networking contracts |

# 23. Required API Contracts

Главный агент должен подготовить перечень и спецификации следующих контрактов:

## Geometry Core

- GeometryAsset.
- GeometryLayer.
- GeometryLayerType.
- GeometryTransform.
- GeometryBounds.
- GeometryRegion.
- BrickKey.
- BrickStorage.
- GeometryChannel.
- Material Mapping.
- ResolutionProfile.
- RefinementRegion.
- Junction.
- Composite Part.
- Assembly Graph.

## Processing

- Processing Task.
- Processing Dependency.
- Dependency Revision.
- Processing Snapshot.
- Dirty Region.
- Processing Result.
- Cancellation Token.
- Job Priority.
- Spatial Hierarchy.
- BVH Update.
- Bounds Update.

## Editing

- Editing Operation.
- Editing Transaction.
- Operation ID.
- Revision Token.
- Conflict Result.
- Authorization Context.
- Editing Result.
- Modifier Stack.
- Geometry Delta.
- Checkpoint.

## Persistence

- Asset Manifest.
- Brick Serialization.
- Delta Serialization.
- Snapshot Serialization.
- Storage Transaction.
- Integrity Report.
- Migration Descriptor.

Для каждого контракта требуется определить:

1. Responsibility.
2. Public Data Model.
3. Ownership.
4. Lifetime.
5. Read/Write Rules.
6. Revision Semantics.
7. Dirty Tracking.
8. Error Model.
9. Threading.
10. Cancellation.
11. Serialization.
12. Compatibility.
13. Tests.

# 24. Remaining Open Technical Decisions

Следующие вопросы не считаются автоматически решёнными:

1. Выбор языка программирования.
2. Выбор игрового движка.
3. Точные integer и floating-point типы.
4. Формальная семантика RevisionToken.
5. Точный алгоритм атомарного commit.
6. Точные API signatures.
7. Конкретная реализация Snapshot и COW.
8. Конкретный алгоритм conflict resolution.
9. Сетевой протокол и транспорт.
10. Формат сериализации.
11. Конкретный компрессор и параметры.
12. Performance Budgets на целевых устройствах.

Решения принимаются на основе прототипов, benchmark results и согласованных API Contracts.

# 25. Required Documentation Updates

Главный агент должен обновить существующий комплект документов, а не создавать параллельную конкурирующую архитектуру.

Обязательные обновления:

1. Geometry Editing Specification.
2. Voxel Architecture.
3. Cube Agent Development Specification.
4. Geometry Editing Agent Task Pack.
5. Geometry Editing Integration & QA Pack.
6. GDD/TDD — только разделы, затрагиваемые технической архитектурой.

Необходимо синхронизировать:

- Module Ownership.
- Dependency Matrix.
- API Contract Matrix.
- Agent Task Dependencies.
- Integration Order.
- QA Requirements.
- Acceptance Criteria.
- Open Technical Decisions.

Документы должны ссылаться друг на друга и не содержать противоречащих определений.

# 26. Implementation Order

Предлагаемый порядок разработки:

1. Geometry Core.
2. Processing Foundations.
3. Editing API & Contracts.
4. Editing Orchestration.
5. Voxel Backend.
6. SDF Backend.
7. Mesh Backend.
8. Sculpting Tools.
9. Boolean Operations.
10. History System.
11. Composite Geometry.
12. Resampling & Resolution Refinement.
13. Persistence Integration.
14. Networking Adapter.
15. Engine Integration.
16. Full Integration & QA.

Порядок должен быть уточнён после формирования Dependency Matrix.

Отсутствующие базовые модули нельзя считать готовыми только потому, что они описаны в документации.

# 27. Final Acceptance Criteria

Архитектурный пакет считается обновлённым после выполнения следующих требований:

- Все владельцы модулей назначены.
- Dependency Matrix не содержит циклических зависимостей.
- Processing имеет чётко определённые границы ответственности.
- History и Persistence разделены.
- Все обязательные контракты перечислены.
- Revision Semantics формализована.
- Snapshot и Dirty Tracking согласованы.
- Transform и Bounds согласованы.
- Serialization и Migration согласованы.
- Networking Boundary определена.
- Engine Independence соблюдена.
- Agent Task Pack соответствует новой архитектуре.
- Integration & QA Pack отражает новые зависимости.
- Открытые решения явно перечислены.
- Нет неподтверждённых API signatures, объявленных окончательными.

**Финальное правило:** утверждённая архитектура является источником истины. Реализация должна следовать ей. Изменение архитектуры требует согласования с Integration Owner.

---

**Document Status: Approved Architecture Decisions / Pending Contract Formalization**
