# ODM-10 Governance Review

> **Update 2026-11-09:** the owner adopted the model of this proposal in simplified form (DEC-002 in [`docs/decisions/decision-registry.md`](../../decisions/decision-registry.md)): statuses `PROPOSED/RECORDED/CONFIRMED/DEFERRED/SUPERSEDED/REJECTED`, levels `CONCEPT/TECHNICAL`, chat confirmation as a permitted source, no mandatory signatures, no second confirmation cycle. ODM-10 is closed. Where this document differs (E1/E2/E3/E5 evidence ladder, G-GOV gate, ratification batch as a gate), the registry rules win. The historical analysis is kept unchanged.

**Статус:** **Proposed** (анализ агента). Документ ничего не утверждает, не закрывает ODM-10, не меняет статусов GOV-01, не меняет существующих документов. Все правила, которые он предлагает изменить, помечены **ПРЕДЛОЖЕНИЕ**.
**Дата проверки:** 2026-11-09. Все прежние сведения о состоянии репозитория — исторические; ниже — результат повторной read-only проверки.

---

## 1. Актуальное состояние репозитория (повторная проверка)

| Что | Результат | Команда |
|---|---|---|
| Ветка / HEAD | `docs/architecture-roadmap-v1`, `8854a44574febc73fdcc82574307557f500d338a` | `git status -sb`, `rev-parse HEAD` |
| Рабочее дерево | `M .gitignore`; `?? docs/decisions/owner-decision-extensibility-v1.md`. Больше ничего | `git status --short` |
| `master` / `origin/master` | `bf60534f…` (docs-only, 0 файлов `crates/`) | `rev-parse`, `ls-tree` |
| `origin/docs/architecture-roadmap-v1` | `8854a44…` (подтверждено `git ls-remote origin`; reflog: «update by push») | `ls-remote`, `reflog` |
| Подписи | `%G?` = `N` у всех коммитов (теперь 44); автор один (`Godrus`) | `git log --format=%G?` |
| `commit.gpgsign`, `user.signingkey` | **не заданы** | `git config --get` |
| [`decision-registry.md`](../../decisions/decision-registry.md) | **отсутствует**; `.github/`, `CODEOWNERS` отсутствуют | `ls` |
| `gh` | **не установлен** — настройки GitHub (защита веток, правила) прочитать нельзя; сведения о них (`protected: false`, 0 workflows, 2026-11-07/08) исторические, не перепроверены | `which gh` |

### 1.1 Коммит `8854a44574febc73fdcc82574307557f500d338a` (проверка)

* **Существует** (`git cat-file -t` → `commit`).
* **Родитель:** один, `16db379d1704c06e8f71da7452eff3fff73b0fcb`.
* **Автор:** `Godrus`; дата 2026-11-09 13:31:30 +0300; подпись `N`.
* **Тема:** «docs: product vision consolidation, owner decision register, Production Plan alignment report, roadmap alignment (proposal)».
* **Содержимое** (`git show --stat`, `git diff --name-only 16db379 8854a44`): ровно **4 файла**, 721 добавление, все в `docs/product-vision/`: [`cube-product-vision.md`](../../product-vision/cube-product-vision.md), [`cube-owner-decision-register.md`](../../product-vision/cube-owner-decision-register.md), [`production-plan-alignment-report.md`](../../product-vision/production-plan-alignment-report.md), [`implementation-roadmap-alignment.md`](../../product-vision/implementation-roadmap-alignment.md). Других файлов нет. Тем самым коммит **действительно содержит** четыре подготовленных документа; это проверено содержимым, а не предположением.
* **Принадлежность веткам:** локальная `docs/architecture-roadmap-v1` и удалённая `origin/docs/architecture-roadmap-v1` (на сервере проверено `ls-remote`). Не содержится в `master` и в `spike/*`.
* **Примечание:** коммит создан в прошлом этапе по вашей просьбе; в удалённую ветку он попал не из этой сессии (push здесь не выполнялся; reflog фиксирует push на вашей стороне).

---

## 2. Как ODM-10 сформулирован сейчас

**ODM-10 — метка, созданная предыдущим пакетом ревью, а не первичный источник** ([`OWNER_DECISION_REGISTER.md`](../owner-decision-m2t-optimization/OWNER_DECISION_REGISTER.md) §0.1; `m2-delivery-review/OWNER_DECISIONS_REQUIRED.md`). Первичные правила — план §3.3 и AOD §4.

Формулировка карточки ([`OWNER_DECISION_REGISTER.md`](../owner-decision-m2t-optimization/OWNER_DECISION_REGISTER.md), ODM-10): *«Какой акт считается подтверждением решения владельца в смысле плана §3.3, если ни один коммит не подписан и агентские сессии коммитят под идентичностью владельца?»* Варианты: (a) E2 — владелец одобряет PR с записями; (b) E3 — владелец делает подтверждающий коммит через веб-интерфейс GitHub; (c) E1 — коммиты, подписанные ключом, недоступным агентам; (d) владелец пишет записи сам. Рекомендация: (a) или (b). Статус `PENDING_OWNER_DECISION`; «блокирует: да (каждая запись зависит)».

Подробный протокол — [`docs/archive/owner-decision-m2t-optimization/OWNER_CONFIRMATION_PROTOCOL.md`](../owner-decision-m2t-optimization/OWNER_CONFIRMATION_PROTOCOL.md) (Proposed): поля записи, акты E1–E4, процесс, `registry-lint`.

---

## 3. Что реально требуют действующие документы

| Источник | Правило | Замечание |
|---|---|---|
| [`production-architecture-plan.md`](../../production/production-architecture-plan.md) §3.3 | единый [`docs/decisions/decision-registry.md`](../../decisions/decision-registry.md); запись: id, дата, решение, статус, scope, **источник подтверждения**, заменённые id. Принимаемые источники: **(a) owner-signed commit или pull-request approval в репозитории; (b) текст решения, написанный владельцем в репозитории.** «Chat instruction alone is not a confirmation source»; агент может записать её как `recorded, awaiting confirmation`; действует после контрподписи по (a) или (b) | **Криптографическая подпись нигде не определена как обязательная.** Слово «signed» в (a) не расшифровано (подпись ключом или «коммит, сделанный владельцем»). Источники «PR approval» и (b) подписи вообще не требуют |
| План §3.1 | статусы Proposed → Provisional → Owner-Approved; Rejected/Superseded; вторая ось Implemented → Frozen; «агент никогда не переводит контракт из Proposed в Provisional/Owner-Approved»; Owner-Approved — «project owner only» | — |
| [`owner-architecture-decisions-v1.md`](../../decisions/owner-architecture-decisions-v1.md) §4 | изменение AOD — «явное одобрение владельца **в чате** или в подписанном PR»; агент может предложить, но не применять | **расходится с §3.3** («чат — не источник»): CR-01 корректирующего ревью |
| [`owner-approval-matrix-v1.md`](../../decisions/owner-approval-matrix-v1.md) §12 | решение записывается в [`owner-architecture-decisions-v1.md`](../../decisions/owner-architecture-decisions-v1.md) (или преемник) → только тогда статус строки Approved | третий механизм записи |
| [`ps-0-exit-checklist.md`](../../production/ps-0-exit-checklist.md) §1, строка GOV-01 | записи с источником подтверждения («owner-signed commit or PR approval»); «ни одна запись не сделана агентом» | — |
| GOV-01 Decision Register §0.2 (CR-01) | правило пакета: решает и утверждает Project Owner; Integration Owner только готовит, проверяет и записывает *по записи владельца* | старые документы (Task Pack 6, 710; audit-resolution 211, 244) приписывают approval контрактов Integration Owner — спор отнесён к X-2/GOV-02 |
| [`OWNER_CONFIRMATION_PROTOCOL.md`](../owner-decision-m2t-optimization/OWNER_CONFIRMATION_PROTOCOL.md) | различает authorship / cryptographic verification / owner decision confirmation / historical traceability | Proposed, не принят |

**Вывод по п.4 задания (нужны ли криптографические подписи коммитов):** ни один действующий источник не требует их однозначно. План допускает PR approval и написанный владельцем текст. Протокол ODM-10 называет подписи «optional … insufficient by itself … not necessary if E2 or E3 is used». Если владелец прочтёт «owner-signed» как «обязательная криптографическая подпись», это изменение трактовки, и оно должно быть явно им выбрано (вопрос Q-2 ниже).

---

## 4. Корректировка прежнего вывода

Прежние отчёты ([`cube-owner-decision-register.md`](../../product-vision/cube-owner-decision-register.md) §0.1, [`cube-product-vision.md`](../../product-vision/cube-product-vision.md) §0.1, [`production-plan-alignment-report.md`](../../product-vision/production-plan-alignment-report.md) §1 п.6; а также [`OWNER_DECISION_REGISTER.md`](../owner-decision-m2t-optimization/OWNER_DECISION_REGISTER.md) §0.3: «nothing can currently be APPROVED/CONFIRMED») формулировали: *«все коммиты не подписаны, поэтому ни одно решение пока нельзя записать в реестр как подтверждённое»*. Этот вывод **неверен в части причинности**:

* Отсутствие подписи — факт о коммитах, не о решениях. Владелец мог принять любое из решений; подпись этого не доказывает и не опровергает.
* План §3.3 имеет как минимум три независимых пути, не требующих подписи: PR approval, текст владельца в репозитории и (в зависимости от прочтения) «owner-signed commit».
* Верный вывод: **«ни одно решение пока не зарегистрировано в реестре с источником подтверждения по §3.3, потому что реестра нет и ни один из актов (a)/(b) для этих решений не совершён»**. Это вопрос регистрации, а не существования решения.
* Что остаётся верным: так как все коммиты (в том числе агентские) идут под одной Git-идентичностью, **метаданные автора не доказывают, что решение принял владелец** ([`OWNER_CONFIRMATION_PROTOCOL.md`](../owner-decision-m2t-optimization/OWNER_CONFIRMATION_PROTOCOL.md) §1–2). Это проблема доказательства происхождения, а не наличия решения; она решается выбором акта (PR approval на платформе, веб-коммит владельца), а не подписью как таковой.

Пять понятий, которые теперь различаются:

| Понятие | Вопрос | Чем подтверждается | Статус в Cube сейчас |
|---|---|---|---|
| Фактическое решение владельца | принял ли владелец решение | действие владельца | для AOD-01…24, OAM-01/02/03/07/08, OC-2026-11-09 — **по сообщениям агентских записей**; первичный источник (чат) в репозитории отсутствует |
| Доказательство / источник | что можно предъявить | акт E1…E3, E5 (см. процедуру) или «нет» | для перечисленных решений — «нет» (только транскрипция агента) |
| Регистрация | запись в реестре | запись с полями | реестра нет |
| Соответствие governance | выполнены ли условия §3.3 | регистрация + источник | не выполнено |
| Авторство коммита (подпись) | чей ключ | GPG/SSH | подписей нет; к решению не относится |

Ошибочные утверждения в закоммиченных документах **не правятся задним числом** (запрет менять существующие документы). Предлагается erratum-запись (см. [`owner-approval-procedure-proposal.md`](owner-approval-procedure-proposal.md) §9) при следующей правке владельцем-разрешённой ревизии.

---

## 5. Ответы на десять вопросов ODM-10

| # | Вопрос | Ответ (с источником) |
|---|---|---|
| 1 | Как ODM-10 сформулирован | §2 выше |
| 2 | Какие проблемы должен решить | (i) нет реестра и единого места записи; (ii) три механизма записи (план §3.3, AOD §4, OAM §12); (iii) чат vs репозиторий как источник (CR-01); (iv) одна Git-идентичность у владельца и агентов; (v) AOD-01…24 агентом транскрибированы из чата («recorded, awaiting owner confirmation», план §3.3 Fact); (vi) разнобой путей реестра (F-2) и словарей статусов |
| 3 | Какие документы регулируют | §3 |
| 4 | Нужны ли подписи | нет однозначного требования; §3, вывод; решает владелец (Q-2) |
| 5 | Как фиксировать источник подтверждения | в поле `confirmation_evidence` записи: тип акта + ссылка (PR, review id, merge commit / web-коммит / файл владельца); если первоисточник недоступен — явное `UNAVAILABLE` |
| 6 | Концептуальное vs техническое | **ПРЕДЛОЖЕНИЕ:** отдельное поле `level` (CONCEPT/TECHNICAL) и обязательное поле `not_decided`; см. процедура §3 |
| 7 | Решения до формального процесса | **ПРЕДЛОЖЕНИЕ:** «пакет ратификации»: записи со статусом RECORDED, историческая дата в `decided_on` с пометкой основания, дата регистрации и дата подтверждения — раздельно; без задним числом; см. процедура §6 |
| 8 | Пересмотр и отмена | план §3.1/§3.3 и AOD §4: новая запись, заменяющая старую (`supersedes`/`superseded_by`), ничего не удаляется; REJECTED/SUPERSEDED только по акту владельца |
| 9 | Кто меняет статус | **Project Owner** (AOD-18; план §3.1/§3.3; CR-01-правило). Integration Owner — механически вносит запись *по акту владельца*; агент — только PROPOSED/RECORDED |
| 10 | Что реально блокирует PS-0 и PS-1 | §6 |

---

## 6. Что блокирует PS-0 и PS-1 (по действующим документам)

* **PS-0 (G0)** закрывается, когда выполнены 8 задач: GOV-01…06, TEST-01, XT-01 ([`ps-0-exit-checklist.md`](../../production/ps-0-exit-checklist.md) §1). Сейчас 0/8. ODM-10 **не входит в список** G0-задач и критериев PS-1 буквально. Он блокирует **косвенно**: GOV-01 требует записей «с источником подтверждения», а источник определяется механизмом, который ODM-10 выбирает. Поэтому ODM-10 — предпосылка GOV-01/GOV-02, но не самостоятельная G0-задача (различие, которое предыдущие документы стирали).
* **Не блокирует PS-0/PS-1:** выбор статусной модели для необязательных деталей, регистр исторических концептуальных подтверждений OC-2026-11-09 (они не входят в 15 решений GOV-01), `registry-lint` (опционален, нужен CI), отдельный аккаунт агента (желателен для E2, не обязателен при E3).
* **PS-1 entry criteria** (план §14 / чек-лист §3): записи для ACR-001, ACR-003, ACR-004 A, D, OD-R1, OD-R2, OD-R6, D-R8, D-K, D-T, D-C, D-0; P0-alpha Owner-Approved; PC-1 Provisional; named owners; docs-only baseline; CI + required checks; ADG принят. Ничего не выполнено.
* Кратчайший путь по документам: выбрать акт подтверждения (ODM-10) → создать реестр → GOV-01 (15 решений + пакет ратификации X-1) → GOV-02…06 → G0.

---

## 7. Найденные противоречия (с вариантами разрешения для владельца)

| ID | Противоречие | Варианты | Рекомендация |
|---|---|---|---|
| C-1 | План §3.3 («чат не источник») vs AOD §4 («одобрение в чате или в подписанном PR» для изменения AOD) | (a) правило §3.3 везде; AOD §4 приводится в соответствие; (b) чат — достаточный источник для CONCEPT-уровня, не для TECHNICAL; (c) чат достаточен везде | (a) для TECHNICAL; (b) допустимо как явный выбор владельца. Любой вариант = изменение правила, нужен ваш акт |
| C-2 | «owner-signed commit» не определён (подпись ключом или коммит владельца) | (a) определить: «подпись не требуется; допустимы PR approval, веб-коммит владельца, owner-written file»; (b) требовать подпись ключом, недоступным агентам; (c) оставить как есть | (a). Не меняет безопасность: никакие проверки не отключаются |
| C-3 | Прежний вывод «unsigned ⇒ ничего не подтверждено» (и его эхо в [`OWNER_DECISION_REGISTER.md`](../owner-decision-m2t-optimization/OWNER_DECISION_REGISTER.md) §0.3) | erratum (см. §4) | принять формулировку §4 |
| C-4 | Три места записи: реестр (§3.3), [`owner-architecture-decisions-v1.md`](../../decisions/owner-architecture-decisions-v1.md) (AOD §4/OAM §12), протокол | единый реестр `DEC-nnn` с алиасами AOD/OAM/ACR/GOV; AOD-файлы остаются как документы-основания | единый реестр |
| C-5 | «Approved Owner Decisions» (заголовок AOD-файла) vs план §3.3 Fact («recorded, awaiting owner confirmation») | считать слово «Approved» в AOD-файлах = статусу `RECORDED` до пакета ратификации | принять; это уже зафиксировано планом |
| C-6 | Три словаря статусов: контрактный (Proposed/Provisional/Owner-Approved/…), реестра протокола (PENDING_OWNER_DECISION/…/CONFIRMED), предложенный в задании (PROPOSED/OWNER_CONFIRMED_CONCEPT/OWNER_APPROVED_TECHNICAL/…) | сведение в одну модель (см. процедура §2) | процедура §2 |
| C-7 | Путь реестра: [`docs/decisions/decision-registry.md`](../../decisions/decision-registry.md) (план) vs `docs/production/…` (другие документы) (F-2) | путь плана | путь плана |
| C-8 | E2 (PR approval) невозможен, пока агенты пушат под учётной записью владельца (GitHub не даёт одобрить свой PR); `gh` в этой среде нет | (a) E3 веб-коммит владельца; (b) отдельная учётка/токен агента; (c) E5 owner-written file (слабее) | (a), позже (b) |
| C-9 | Дата в прежних документах «OC-2026-11-09» и в черновике extensibility («confirmed in chat on 2026-11-09») ссылаются на чат, не доступный в репозитории | запись `source: UNAVAILABLE in repository` | принять |

---

## 8. Общая оценка

Механизм ODM-10 сформулирован корректно как вопрос, но (1) не имеет единой нормативной формулировки в плане, (2) опирается на прежний чрезмерно жёсткий вывод про подписи и (3) смешан с вопросом словаря статусов. Достаточно одной скромной процедуры ([`owner-approval-procedure-proposal.md`](owner-approval-procedure-proposal.md)) и одного акта владельца на пакет, чтобы закрыть ODM-10 без отключения проверок и без правки существующих документов задним числом.

Закрытие ODM-10 — **только решением владельца**; этот документ его не закрывает.
