# INDEX: допустимые документы проекта

Единственный белый список `.md`. Новый файл допустим только со строкой здесь (путь и зачем) или по слову владельца. Проверка: `cargo xtask docscheck` (в CI): падает на `.md` вне списка, на путях из списка, которых нет, на битых относительных ссылках, на файлах без единой входящей ссылки, на упоминании другого `.md` обычным кодом вместо ссылки (граф связей должен оставаться связным) и на раздувшихся [`AGENTS.md`](../AGENTS.md) (больше 60 строк) и [`LOG.md`](LOG.md) (больше 40 строк). Путь с `/` на конце покрывает папку целиком. Не читать «на всякий случай»: открывать по нужде.

## Рабочие (читать и обновлять)

| Файл | Зачем |
|---|---|
| [`AGENTS.md`](../AGENTS.md), [`CLAUDE.md`](../CLAUDE.md) | вход для агентов (CLAUDE.md только ссылка) |
| [`.agent/BOARD.md`](BOARD.md) | задачи: в работе, дальше, заблокировано, сделано |
| [`.agent/LOG.md`](LOG.md) | краткая история работ (старое уходит в `.agent/archive/`) |
| `.agent/INDEX.md` | этот список |
| `.agent/live/CHAT.md` | канал агентов, не в git, только через `cargo xtask say` и `chat` |
| [`docs/handoff/NEW-CHAT-START.md`](../docs/handoff/NEW-CHAT-START.md) | правила работы: автономия, статусы, готовность функции, реестр функций, git и CI |
| [`docs/roadmap/PRODUCT-ROADMAP.md`](../docs/roadmap/PRODUCT-ROADMAP.md), [`docs/roadmap/WORK-PACKAGES.md`](../docs/roadmap/WORK-PACKAGES.md) | вехи M0..M12 и разбивка задач |
| `docs/features.json` (источник), [`docs/FEATURE-REGISTRY.md`](../docs/FEATURE-REGISTRY.md) (генерируется), [`docs/FUNCTIONAL-ROADMAP.md`](../docs/FUNCTIONAL-ROADMAP.md) | реестр функций |
| [`docs/decisions/decision-registry.md`](../docs/decisions/decision-registry.md) | решения владельца, решает только он |

## Справочные (открывать по задаче)

| Файл | Зачем |
|---|---|
| `docs/vision/` | видение игры и визуальный стиль |
| `docs/product-vision/` | подтверждённые владельцем требования к продукту (CPR), которые остаются в силе; решения и сверка планов |
| `docs/production/` | инженерный план, ADG ([`module-boundaries.md`](../docs/production/module-boundaries.md)), ворота, контракты Processing и Editing, CI, защита веток ([`github-branch-protection.md`](../docs/production/github-branch-protection.md)) |
| `docs/api-contracts/` | контракты (все Proposed или Provisional), в том числе временные контракты TC-01..TC-11 |
| `docs/architecture/`, `docs/specifications/`, `docs/agent-tasks/` | спецификации геометрии и границы системы |
| `docs/decisions/` | пакеты решений владельца (кроме реестра, который выше) |
| `docs/research/` | заметки по референсам и спайкам |
| [`docs/RELEASE_CRITERIA.md`](../docs/RELEASE_CRITERIA.md) | критерии вех к релизу 1.0 (Proposed) |
| [`docs/GODOT-OBSERVATION-PLAN.md`](../docs/GODOT-OBSERVATION-PLAN.md) | план наблюдения физики и камеры в Godot (не реализован) |
| [`godot/README.md`](../godot/README.md) | запуск Godot-демо и сборка Android |
| [`tools/visual-qa/README.md`](../tools/visual-qa/README.md) и `tools/visual-qa/**` | Visual QA: README, версии, результаты прогонов (эксперимент, не ворота) |

## Архив

`docs/archive/` покрыт целиком, читать только при поиске истории. Внутри: пакеты GOV-01, M2 delivery review, owner-decision M2T, аудиты, governance; старые файлы handoff и статуса; планы Game Track (First Playable, Vertical Slice); отчёты ревизий плана v2 и v2.1; прежний индекс документации. Реестр решений цитирует архивные файлы как первоисточники решений: не удалять, не править и не переименовывать их без записи в реестре. Точки входа в архив: [`docs/archive/m2-delivery-review/README.md`](../docs/archive/m2-delivery-review/README.md), [`docs/archive/owner-decision-m2t-optimization/README.md`](../docs/archive/owner-decision-m2t-optimization/README.md), [`docs/archive/README.md`](../docs/archive/README.md) (прежний индекс), [`docs/archive/handoff/AGENT-BOOTSTRAP-PROMPT.md`](../docs/archive/handoff/AGENT-BOOTSTRAP-PROMPT.md) (прежний промпт для нового чата). Скрипты внутри архива (аудиторские `*.py`) не поддерживаются.

## Как добавить файл

1. Проверить, нельзя ли дописать в существующий.
2. Добавить строку в нужную таблицу (путь и зачем) в том же коммите.
3. `cargo xtask docscheck` должен пройти.
