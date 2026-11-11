# INDEX: допустимые документы проекта

Единственный белый список `.md`. Новый файл допустим только со строкой здесь (путь и зачем) или по слову владельца. Проверка в CI появится в DOC-02; до этого правило действует по договорённости. Не читать «на всякий случай»: открывать по нужде.

## Рабочие (читать и обновлять)

| Файл | Зачем |
|---|---|
| `AGENTS.md`, `CLAUDE.md` | вход для агентов (CLAUDE.md только ссылка) |
| `.agent/BOARD.md` | задачи: в работе, дальше, заблокировано, сделано |
| `.agent/LOG.md` | краткая история работ |
| `.agent/live/CHAT.md` | канал агентов, не в git, только через `cargo xtask say/chat` |
| `docs/handoff/NEW-CHAT-START.md` | правила работы, автономия, готовность функции, git и CI |
| `docs/roadmap/PRODUCT-ROADMAP.md`, `docs/roadmap/WORK-PACKAGES.md` | вехи M0..M12 и разбивка задач |
| `docs/features.json` (источник), `docs/FEATURE-REGISTRY.md` (генерируется), `docs/FUNCTIONAL-ROADMAP.md` | реестр функций |
| `docs/decisions/decision-registry.md` | решения владельца, решает только он |

## Справочные (открывать по задаче)

| Файл | Зачем |
|---|---|
| `docs/vision/GAME-VISION.md`, `docs/vision/VISUAL-STYLE-GUIDE.md` | видение и стиль |
| `docs/production/module-boundaries.md` | граф crate (ADG) |
| `docs/production/` (остальное), `docs/api-contracts/`, `docs/architecture/`, `docs/specifications/`, `docs/agent-tasks/`, `docs/research/` | инженерный план, контракты, спецификации |
| `docs/decisions/` (кроме реестра) | отчёты и пакеты решений |
| `docs/AGENT-HANDOFF.md` | действуют только разделы Hard constraints и Rules for features |
| `godot/README.md`, `tools/visual-qa/**` | запуск Godot, Visual QA (эксперимент) |

## Исторические (не читать без причины)

`docs/gov01/`, `docs/m2-delivery-review/`, `docs/owner-decision-m2t-optimization/`, `docs/audit/`, `docs/governance/`, `docs/product-vision/`, `docs/AGENT_HANDOFF.md`, `docs/PROJECT_STATUS.md`, `docs/handoff/NEXT-EXECUTION.md`, `docs/handoff/AGENT-BOOTSTRAP-PROMPT.md`, `docs/README.md`, `docs/DEVELOPMENT_ROADMAP.md`, `docs/RELEASE_CRITERIA.md`, `docs/GODOT-OBSERVATION-PLAN.md`. Перенос в `docs/archive/` запланирован в DOC-02.

## Как добавить файл

1. Проверить, нельзя ли дописать в существующий.
2. Добавить строку в нужную таблицу (путь и зачем) в том же коммите.
