# M3 Vertical Slice: scope review

**Status:** **Proposed** / **Pending Owner Approval**. Reviews `docs/production/VERTICAL_SLICE_SCOPE.md` (Proposed, commit `362a124`). No official milestone changes. Counts are design volumes, not performance budgets; no effort is estimated (none is known).

## 1. Is the first Vertical Slice overloaded?

**Yes, as written.** Three observations:

1. **Seven new gameplay systems arrive at once** on top of M2: crafting, quests, a boss encounter with its own behaviour, a progression choice, rarity, a third equipment slot, and points-of-interest placement. Each needs rules, data, UI and tests. M2 will have exercised none of them.
2. **Breadth works against the purpose.** A vertical slice is judged by how finished one piece feels. 20-30 items, 3 environments, 4-5 creatures and a boss multiply the art and design content that has no owner today, while the *quality of the loop* needs a few things done well.
3. **Co-op as "optional" is hidden scope**: it needs sessions, authentication, reconnect, an external transport and its owners (`SRV-02`, `SRV-05`, `NET-04`; none assigned, Later in the ADG). It is a different kind of risk (architecture and network), better validated by a separate spike than by overloading the slice.

The reduction below is justified item by item by its effect on the slice's quality and completeness, not by arbitrary cutting. The questions Q-1...Q-8 of the original scope (full session, variety, progression shape, combat variety, building as gameplay, persistence at scale, direction, scale of technology) must still be answerable.

## 2. Item-by-item: Minimum, Recommended, Deferred

"Minimum" = the smallest slice that still answers Q-1...Q-8. "Recommended" = a good slice if the content owners exist. "Deferred" = valuable, not needed for the slice to be complete.

| Item | Original | **Minimum M3** | **Recommended M3** | **Deferred** | Justification (effect on quality and completeness) |
|---|---|---|---|---|---|
| Points of interest | 5-8 | 3 (distinct kinds: a resource site, a camp with creatures, a boss site) | 5 | 6+ | destinations give exploration a purpose (Q-2); three different kinds show variety, more is repetition of the same work |
| Environments | 3 | the 2 of M2, polished, plus transitions and one water-like obstacle | 3 | more | two coherent, polished environments read better than three rough ones; the third mainly adds art cost |
| Creature kinds | 4-5 | 3 (two hostile with different behaviours, one passive) | 4 | 5+ | behaviour variety matters more than count; two hostile behaviours already test combat variety (Q-4) |
| Boss | 1 | 1 **elite encounter** at the boss site (one special attack) | 1 boss with two phases | more bosses | the encounter is the emotional peak of a slice; a two-phase boss needs scripting beyond the M2 AI |
| Items | 20-30 | 12-15 | 20 | 30+ | enough to show tiers and variety; each item is content with icon and balance |
| Rarity levels | 3 | 2 | 3 | more | rarity is a small data change once the item system exists |
| Equipment slots | 3 | 2 (weapon, body) | 3 | sets, more slots | two slots show the system; a third is data |
| Progression | levels + 1 choice | levels and stat growth, no choice | levels + **one choice** (two paths) | talent trees, classes | the choice tests whether progression is meaningful (Q-3); without it Q-3 is only partly answered, so it belongs in Recommended, not Minimum |
| Gathering | yes | yes (dig/harvest) | yes | tools, depletion | it feeds building and crafting |
| Crafting | 5-10 recipes | 3 recipes at a fixed station | 8 | advanced crafting | crafting closes "gather -> improve"; without it resources serve only building (hypothesis H-2 untested) |
| Building and destruction | structure type + consequences + undo | place/remove with gathered materials plus **one structure type**; no undo | consequences (cover, access) + undo | composite assets, advanced tools | building is the identity pillar P-3; one structure type proves it; undo needs History (`PS-7`), which is Later |
| Quests / activities | short chain + boss site | one 3-step chain acting as the guided path | chain + one side activity | quest system generalisation | gives the session shape (Q-1); a general quest system is not needed for one chain |
| Persistence | world, character, inventory, quests | same | same | cloud saves, multiple characters | non-negotiable for Q-6 |
| Visual direction | style guide + set | style guide and **one finished region** | plus lighting/sky/water polish | wide asset library | direction is judged on one region (Q-7) |
| Audio direction | music + SFX set | SFX for combat/UI/ambience, one music loop | adds variety | adaptive music | |
| Platforms | desktop (+ device checkpoint) | desktop + MAC-2 device baseline | + a second device class | iOS | AOD-08/11: device evidence; see `M2_SCOPE_REVIEW.md` section 5 |
| Co-op | optional | **not in M3**; a technical co-op spike (two clients, one host, edits visible) runs separately | co-op included only if owners and `SRV-02/05`, `NET-04` exist | multiplayer features | removes network scope from the quality question; the spike de-risks M4 |

## 3. Proposed split

* **M3-min (Minimum):** everything in the Minimum column. Answers Q-1...Q-7 and Q-8 for single-player.
* **M3-rec (Recommended):** Minimum plus the Recommended column. Adds the progression choice, a third environment and a fuller boss.
* The owner chooses at M3 planning, with measured M2 velocity and content costs in hand; both are consistent with the same technology additions.

## 4. Technology added for M3 (not for M2)

| Need | Where | Condition |
|---|---|---|
| Richer generation: features, points of interest, third environment, creature spawn rules | `WGEN` beyond `WGEN-01..04`, `TGAP-07/08` | generator version bump rehearsal |
| Item data scale, crafting, quests, boss scripting, abilities | Game Rules | module exists from M2 |
| Building structure type | Game Rules + Editing API | no History in M3-min |
| Undo for building | History (`PS-7`, `HIST-*`) | only in M3-rec if the owner wants it |
| Greedy mesher, LOD | `VOX-03`, `CLI-03` | only if measurements on the target hardware require it |
| Device baseline | `DEV-01`, `QA-06`, `DG-*` | hardware |
| Full persistence crash matrix, journal at scale | `QA-P2` complete | |
| Persistence format approval | `GOV-10` | needs the Voxel Architecture or a recorded alternative; M2-T saves are disposable, M3 saves should not be |
| Content pipeline and data format | Import/Export (O-4), ODM-13 | before content grows |

## 5. What M3 must prove (kept, reordered)

Q-1 session engagement; Q-2 purposeful variety; Q-3 progression shape; Q-4 combat variety; Q-5 building as gameplay; Q-6 persistence with real state; Q-7 direction; Q-8 scale of technology on the target hardware. Q-9 (co-op) moves out of M3 to the spike and M4.

## 6. Entry conditions

* M2 accepted (or content pre-production started earlier on the owner's decision; design and asset work do not touch production code).
* Owners for content, art/audio and Game Rules exist.
* The Processing and persistence contracts are no longer only Provisional, or the owner records why they stay so (TC-01, TC-02, TC-08 expire at M3 start or earlier).
* MAC-1 (device smoke) done if mobile is part of M3.

## 7. Risks

Content volume without owners is the main risk; the Minimum column exists to keep the slice finishable. No schedule is given: velocity from M2-T and M2 and the content costs are the missing inputs.
