# Owner Decision Record (draft for review): Extensibility and Modifications, No. 1

**Status of this document:** **Draft for owner review.** Section 2 records what the owner confirmed in chat on 2026-11-09 (**CONFIRMED**). Everything else is the agent's proposal (**PROPOSED**), deliberately postponed (**DEFERRED**), or needs a decision (**OPEN**). Nothing here approves a contract, changes an existing AOD, edits AU-1.0 or ACRs (AOD-19), or starts implementation.
**Proposed registry ids:** AOD-25 and AOD-26 (next free numbers after AOD-24); they are proposed ids only, assigned when the owner confirms this record (change control: `owner-architecture-decisions-v1.md` section 4).
**Scope:** extensibility of Cube's built-in content and systems, future modifications, and saves of modified worlds. **Out of scope by owner's instruction:** building, world generation design and multiplayer (separate topics); they are mentioned only where a save or identity rule touches them.

Status marks: **CONFIRMED** (owner), **PROPOSED** (agent), **DEFERRED** (consciously postponed), **OPEN** (needs a decision).

---

## 1. Summary of the decision

| # | Statement | Status |
|---|---|---|
| S-1 | Cube is built first. A full modding ecosystem (tools, SDK, scripting API, WASM runtime, distribution platform) comes in later, separate stages. | CONFIRMED |
| S-2 | The architecture is designed from the start so that a full modding system can be added later **without a fundamental rework of the core**. | CONFIRMED |
| S-3 | Built-in mechanics and future mods use **compatible extension mechanisms** wherever a capability is deliberately opened by the developer: no separate architecture for built-in content and for mods, no two independent registration systems for the same kind of game object. | CONFIRMED |
| S-4 | Not every internal interface becomes public. Cube's developer decides which capabilities are open to mods; internal implementations may stay closed; mods get no automatic access to internals. | CONFIRMED |
| S-5 | A modified world must restore with all buildings, objects and other data created with mods. Normal loading must protect the save from data loss; objects are never silently removed because a mod is missing. | CONFIRMED (requirement); the handling policy is PROPOSED (section 5) |
| S-6 | The execution technology for mod code is not chosen. WASM is a candidate only. | CONFIRMED |
| S-7 | Three conceptual extension kinds: Content Packs, Game Extensions, Runtime Plugins. A preliminary classification, not three mechanisms to build now. | CONFIRMED (as a classification) |
| S-8 | **Built-in content is itself packaged as a pack** (a `cube` base pack, possibly split into several) that goes through the same registration, identity and save-reference path a mod would use. This is the concrete form of S-3. | PROPOSED |

**Proposed short form for the registry:**
* **AOD-25 Extensibility principle** = S-1...S-4, S-6, S-7 (+ S-8 if the owner accepts it).
* **AOD-26 Modified-world saves** = S-5 + the mandatory save requirements R-10...R-17 (section 2).

### 1.1 Priorities (owner's table)

| Direction | Decision | Status |
|---|---|---|
| Developing Cube itself | Main priority | CONFIRMED |
| Architectural extensibility | Designed in from the start | CONFIRMED |
| Content mods | Provided for | CONFIRMED |
| New game mechanics through mods | Provided for | CONFIRMED |
| New game systems | Architectural possibility provided for | CONFIRMED |
| Tools for mod authors | Later | CONFIRMED (DEFERRED) |
| Rust SDK | Later | CONFIRMED (DEFERRED) |
| Scripting API | Later | CONFIRMED (DEFERRED) |
| WASM runtime | Not approved; a possible direction | CONFIRMED (OPEN as a choice) |
| Mod publishing platform | A separate future stage | CONFIRMED (DEFERRED) |

---

## 2. Mandatory architecture requirements

"Mandatory" means: the architecture must not contradict them from now on, and the listed items must exist **before the first save format is approved** (they are cheap if designed in and expensive to retrofit). It does not mean implementing a mod system in First Playable.

### 2.1 Shared contracts and boundaries

| # | Requirement | Status |
|---|---|---|
| R-01 | **One registry model** for every kind of game definition (block/material, item, creature, biome, structure, recipe, ...). Built-in content and mod content register into the same registries through the same path (S-3, S-8). | PROPOSED (follows from CONFIRMED S-3) |
| R-02 | **Stable, namespaced identifiers** for every registered definition: `namespace:name` (for example `cube:stone`, `castlemod:oak_beam`). The namespace belongs to exactly one pack. Identifiers are never reused for a different meaning. | PROPOSED |
| R-03 | **Runtime numeric ids are not persistent identity.** Compact numbers (for example the `u16` material value in Core bricks today) are assigned per world/session and mapped to stable ids through a **per-world id map** (palette) that is saved with the world. This matches the existing specification rule "each Geometry Asset uses a local material palette; Material IDs reference the global material library" (`geometry-editing-specification-v1.0.md` 3.5). | PROPOSED |
| R-04 | **Explicit public extension surface.** Each extension point is declared deliberately, with a status (internal / experimental / public-stable) and a version. Anything not declared is internal. Mods cannot reach Core internals, the commit primitive, the system-commit port, persistence internals or the network layer. | PROPOSED (follows from CONFIRMED S-4) |
| R-05 | **All mod-originated world changes go through the same authoritative paths as built-in ones**: user-originated geometry changes through the Editing API (AOD-15), generation through the WorldGen/system-commit boundary (AOD-16, AOD-20), state decided by the authoritative runtime (AOD-13, AOD-22). No extension gets a private write path. | PROPOSED (derived from Approved AODs) |
| R-06 | **Engine neutrality.** Pack data and contracts are engine-neutral (Rust-side definitions). Godot-specific presentation (scenes, shaders, `.pck`) may exist only as an optional client presentation layer keyed by stable ids; a pack never requires Godot to be valid (AOD-06, AOD-07). | PROPOSED (derived from Approved AODs) |
| R-07 | **Packs are data first.** Content Packs are declarative and validated against schemas on load; a Content Pack never executes code. | PROPOSED |
| R-08 | **Pack manifest** for built-in and future mod packs: id (namespace), version (semantic), declared dependencies with version ranges, declared extension-point/API versions it targets, which save-relevant data it contributes. | PROPOSED |
| R-09 | **Deterministic load order** of packs (dependency order, then a stable tie-break), so that the same pack set always yields the same registries and the same per-world id assignment. | PROPOSED |

### 2.2 Saves of modified worlds

| # | Requirement | Status |
|---|---|---|
| R-10 | Each world records the **set of packs it depends on**, with id and exact version used (including the built-in `cube` pack and game version). | CONFIRMED (owner listed it) |
| R-11 | Each world stores the **per-world id map** (R-03) for every registry whose ids appear in saved data (voxels, items, entities, ...). Saved data stores numbers only through this map, or stable ids directly. | PROPOSED (implements CONFIRMED "store references to mod-added objects") |
| R-12 | On load, the runtime **checks presence and version compatibility** of every required pack before any world data is applied. | CONFIRMED (owner listed it) |
| R-13 | **No silent loss:** a reference to a definition that is missing (pack absent, or id removed) is never dropped, replaced or rewritten during normal load. Unknown data is **preserved as opaque data** ("placeholder") and written back unchanged on save. | CONFIRMED (requirement); placeholder mechanism PROPOSED |
| R-14 | **Normal load is blocked** (or the world is opened read-only) when required packs are missing or incompatible; the player is told exactly what is missing and which version is needed. | PROPOSED (owner listed it as an option) |
| R-15 | **Recovery is explicit:** any operation that changes or removes data because of a missing/changed pack (replacement, deletion, migration) happens only in a dedicated recovery/migration action chosen by the player, after an automatic **backup** of the world. | PROPOSED |
| R-16 | **Mod updates carry migrations:** a pack version that renames or removes ids must declare a mapping/migration from earlier versions; a version change without one is treated as incompatible for worlds that use the affected ids. | PROPOSED |
| R-17 | **Generation identity includes packs.** If a pack contributes to generation (biomes, structures), the world's generation identity is `GeneratorVersion` **plus** the generation-relevant pack set and versions; the AOD-21 rule "a generator change never silently changes an existing world" applies to them too (pin, bake or explicit migration). | PROPOSED (extends Approved AOD-21 to packs) |

---

## 3. What can be deferred

| Item | Why it can wait | Until | Status |
|---|---|---|---|
| Choice of code-execution technology for Runtime Plugins (WASM, embedded scripting, native Rust dylibs) | no mod code runs before Cube exists | a dedicated decision after First Playable | DEFERRED (WASM = candidate, CONFIRMED not approved) |
| Scripting API, Rust SDK, mod-author tools, documentation for modders | owner priorities | later stages | DEFERRED (CONFIRMED) |
| Mod publishing/distribution platform, signing, update channel | owner priorities | separate stage | DEFERRED (CONFIRMED) |
| Sandboxing, permission model and resource limits for mod code | depends on the technology choice | with the technology decision | DEFERRED |
| Loading of third-party packs at all (user-installed mods) | not needed for First Playable | after First Playable | DEFERRED; but the built-in pack uses the path from day one (S-8, PROPOSED) |
| Which systems become public extension points (combat rules, AI, UI, crafting, ...) | the systems do not exist yet | per system, when designed | DEFERRED, case by case |
| Concrete pack file format, schema language, manifest fields | no format is approved anywhere yet | with the first data-format decision (FIRST_PLAYABLE_SCOPE item "data-driven definitions from day one, format not decided") | DEFERRED |
| Mods in multiplayer (server/client pack sync, mismatch rules) | owner excluded multiplayer from this topic | multiplayer topic | DEFERRED |
| Mods adding new geometry representations | representations are a closed Core enum by design (production-architecture-plan); internal | not planned | DEFERRED / probably never public |
| Licensing of mod content and redistribution of packs with worlds | AOD-23 covers only third-party sources | distribution stage | DEFERRED |

**Needed before the first approved save format (cannot be deferred past it):** R-02, R-03, R-08 (minimal), R-10, R-11, R-12, R-13 and the place for R-17 in the world record. They cost little now and are a rewrite of the save format later.

---

## 4. Conceptual scheme: built-in systems and mods (PROPOSED)

```
                  +-----------------------------------------------+
                  |               Pack Loader (Rust)              |
                  |  manifests -> dependency resolution -> order  |
                  |  schema validation -> registration            |
                  +-------+--------------------+------------------+
                          |                    |
      built-in packs      |                    |   (later) third-party packs
      cube:base, ...  ----+                    +---- castlemod, furnimod ...
                          v                    v
  +-----------------------------------------------------------------------+
  |             Registries (one per definition kind; shared)              |
  |  materials/blocks | items | creatures | biomes | structures | recipes |
  |  key = stable id `ns:name` ; value = validated definition (data)      |
  +----------+------------------------+-------------------------+---------+
             | read-only views         | extension points        |
             v                         v (declared, versioned)   v
  +-------------------+   +------------------------------+   +---------------------+
  | Game systems      |   | Public extension surface     |   | Per-world id map    |
  | (built-in Rust):  |<--| - Content kinds (data)       |   | stable id <-> u16 / |
  | combat, items,    |   | - Game Extension hooks       |   | u32 runtime ids     |
  | creatures, ...    |   |   (events, rule tables)      |   | saved with world    |
  +---------+---------+   | - Runtime Plugin host (later,|   +----------+----------+
            |             |   technology OPEN)           |              |
            |             +------------------------------+              |
            v  intents / operations only                                v
  +-----------------------------------------------------------------------+
  | Authoritative runtime (AOD-13, AOD-22): Editing API (AOD-15),         |
  | WorldGen / system commit (AOD-16, AOD-20), persistence (AOD-21)       |
  +-----------------------------------------------------------------------+
            |                                                   (internal: Core,
            v                                                    commit primitive,
  Client (Godot today, replaceable): presentation keyed by stable ids   storage, network)
```

How the three kinds share contracts (PROPOSED):

| Kind | Uses | Never gets |
|---|---|---|
| **Content Pack** | manifest, registries (data definitions), presentation assets keyed by id | code execution |
| **Game Extension** | everything a Content Pack uses + **declared hooks** of built-in systems (event subscriptions, rule/parameter tables, new definition kinds registered through a registry API that Cube exposes) | internal state, private write paths |
| **Runtime Plugin** | everything above + a code host with a capability-limited API (technology OPEN) | Core internals, commit primitive, file system/network/process access (AOD-12) unless a capability grants it |

The three kinds are **one manifest + one registry model + increasing permissions**, not three mechanisms. Built-in systems are written against the same registries and the same hooks they would expose; whether a hook is *published* is a separate, per-hook decision (S-4).

Public vs internal (PROPOSED rule): a type or function becomes part of the extension surface only by being listed in an **extension API manifest** with a version and a stability level. Internal crates (Core storage, commit primitive, persistence internals, transport) never appear in it. A dependency test (like `crates/cube-core/tests/dependency_rules.rs` today) can enforce that the extension-surface crate depends only on the allowed crates.

---

## 5. Lifecycle of a modified-world save (PROPOSED)

### 5.1 What a world save records (additions to the Draft world record WP-01)

```
WorldRecord (WP-01, Draft) + proposed additions:
  WorldSeed, GeneratorVersion, SchemaVersion, WorldConfig, ...
  PackSet:      [ {pack_id, exact_version, content_hash, role: content|generation|rules} ... ]
  GameVersion:  version of the built-in cube pack(s)
  IdMaps:       per registry kind: [ runtime_id -> "ns:name" ]   (append-only within a world)
  GenerationIdentity = GeneratorVersion + generation-relevant packs (R-17)
Journal / snapshot records reference definitions only through IdMaps or stable ids.
```

### 5.2 Save

```
gameplay ---> authoritative runtime ---> journal records (stable ids or mapped ids)
                                    \---> IdMaps grow append-only when a new definition is first used
on save/compaction (AOD-21 protocol): snapshot + manifest include PackSet, IdMaps, GenerationIdentity
```

### 5.3 Load (normal)

```
read manifest
  -> read PackSet, IdMaps, GenerationIdentity
  -> resolve installed packs
       |-- all present, versions compatible ----------------------> NORMAL LOAD
       |      build registries, remap IdMaps to current runtime ids,
       |      base generation -> snapshot -> journal tail (WP-08)
       |
       |-- newer pack version with declared migration -----------> ASK PLAYER: "update world?"
       |      yes: backup -> migrate (explicit) -> NORMAL LOAD
       |      no:  stop (keep world unchanged)
       |
       |-- missing pack / incompatible version / removed ids -----> BLOCKED
              show: missing pack ids, required versions, what in the world uses them (counts)
              options:
                a) install/restore the required pack version   -> retry load (world untouched)
                b) open READ-ONLY with placeholders           -> nothing is written
                c) RECOVERY MODE (explicit, after automatic backup):
                     - map missing ids to compatible replacements (declared by packs or chosen by player)
                     - or keep placeholders and continue playing (placeholders preserved on save)
                     - or remove listed objects (explicit, shown, reversible through the backup)
```

### 5.4 Placeholders (PROPOSED)

* A voxel, item or entity whose definition is missing loads as a **placeholder** that keeps the original stable id and its raw saved payload.
* Placeholders are visible (a marked block/object), inert (no behaviour), and are saved back byte-for-byte.
* When the pack returns, placeholders resolve to the real definition with no loss.
* Generation-relevant missing packs cannot be placeholdered for **ungenerated** chunks (the generator is missing): such worlds stay BLOCKED for generation (read-only or recovery), consistent with the AOD-21/WP-09 rule "no silent change".

### 5.5 Pack update and removal

| Event | Rule |
|---|---|
| Pack updated, ids unchanged, data changed (for example a block's hardness) | compatible; definitions change, saved ids stay valid; flagged as "content changed" in the load summary (OPEN: whether stat-level changes need player confirmation) |
| Pack updated, ids renamed/removed **with** declared migration | explicit update with backup (5.3) |
| Pack updated, ids removed **without** migration | treated as missing for those ids -> placeholders / recovery |
| Pack removed by the player | world still lists it; next load = BLOCKED path; nothing is deleted automatically |
| Pack contributes to generation and changes its generation logic | follows AOD-21/WP-09: pin (keep old version available), bake (write generated chunks), or explicit migration; never silent |
| Built-in `cube` pack changes between game versions | same rules: the game itself ships migrations for its own ids (S-8) |

---

## 6. Contradictions and interactions with existing documents (nothing was changed)

| # | Existing statement | Where | Conflict with this record | Proposed resolution (owner chooses) |
|---|---|---|---|---|
| X-1 | "**REF-16 Rejected for the roadmap** (modding is out of scope until the GDD says otherwise)" | `docs/research/veloren-notes.md:86`, `docs/research/reference-comparison.md:40` (REF-16 "Rejected (out of scope)") | The owner now puts extensibility and future modding in scope as an architectural requirement. The rejection's *reason* is superseded; its *object* (adopting Veloren-style WASM plugins now) is still consistent with S-6. | (a) Keep REF-16 "Rejected for the current roadmap" and add a note "rationale superseded by AOD-25; WASM remains a candidate, not chosen" (recommended); or (b) change REF-16 to "Deferred / candidate". Both are edits the owner must approve. |
| X-2 | "No plugin registry for representations"; `Representation` is a **closed enum**, a new one needs an ACR | `docs/production/production-architecture-plan.md` lines 139, 179 | No real conflict: geometry representations are Core internals and should **not** be an extension point (S-4). Possible misreading: "no plugin registry" could be read as "no registries at all". | State in AOD-25 that content registries (R-01) are a game-layer concept and do not open Core representations; no change to the plan text needed. |
| X-3 | `WorldRecord` = seed, GeneratorVersion, SchemaVersion, WorldConfig ...; recovery refuses unreconcilable `GeneratorVersionMismatch` | `docs/api-contracts/world-persistence-contracts.md` WP-01, WP-02, WP-08, WP-09 (Draft) | Gap, not conflict: no PackSet, no IdMaps, no generation identity beyond GeneratorVersion, no missing-pack outcome code. | Add, when the Draft is next revised: PackSet, IdMaps, GenerationIdentity (R-17), outcome codes such as `PackMissing`, `PackIncompatible`, `UnknownDefinition`. Not edited now. |
| X-4 | `GeneratorVersion` registry with pinned reference outputs | `docs/api-contracts/worldgen-contracts.md` WG-02 (Draft) | Gap: generation-affecting packs are not part of the version identity. | Extend WG-02 later with R-17; Draft only. |
| X-5 | Core bricks store a raw `u16` material; spike materials 1-4 are hard-coded | `crates/cube-core/src/brick.rs`, spike code | Not a contract conflict (spike, AOD-18); confirms R-03 is needed: the `u16` must become a per-world palette index, not an identity. | Record R-03 as an input to the Core material-palette design (CORE-03/CORE-12, "palette deferred to PS-5"). |
| X-6 | `EditingOperation.Kind` "extensible"; "**Open:** Kind extension registry" | `docs/api-contracts/editing-contracts.md` E-03 | Consistent; the open item is where mod-defined edit kinds would go. | Keep open; decide per R-04 (probably internal until the Editing API is approved). |
| X-7 | "data-driven definitions from day one (format not decided here)" | `docs/production/FIRST_PLAYABLE_SCOPE.md` section 5 table | Consistent and supportive: S-8 makes this concrete (built-in content as a pack using registries). | Reference AOD-25 from the scope when it is next revised. |
| X-8 | GDD/TDD missing (B-1); BL-2 (AODs as product constraints) only a candidate | `docs/README.md`, AOD-24 | REF-16 deferred modding to the GDD; the owner's chat decision now acts as the product requirement. | Record AOD-25/26 as owner decisions so they count under BL-2 if approved. |
| X-9 | Entity model is open (REF-13, ECS vs domain model) | `GAME_TRACK_ROADMAP.md` | Affects how Game Extensions attach behaviour to creatures/items. | Add "extension hooks for entities" as an input to the REF-13 spike; no decision now. |
| X-10 | AOD-12 forbids child processes in the runtime | owner decisions | Consistent: Runtime Plugins may not spawn processes; restates a constraint for the future host. | No change. |

---

## 7. Risks and owner questions

### 7.1 Risks

| Risk | Effect | Mitigation (PROPOSED) |
|---|---|---|
| Over-engineering: building a mod platform instead of the game | First Playable delayed | Only R-01...R-03, R-08 (minimal) and the save items in section 3 are done before First Playable; everything else DEFERRED |
| Under-engineering: hard-coded numeric ids in the first save format | every later mod breaks old saves, or a save-format rewrite | R-02/R-03/R-11 before the first approved save format |
| Built-in content using shortcuts mods cannot use | two architectures in practice (contradicts S-3) | S-8: built-in content registers through the pack path; a test that no definition is created outside the loader |
| Public surface grows by accident | every internal change breaks mods | explicit extension API manifest with stability levels; dependency test |
| Placeholder data grows or blocks systems | odd worlds, performance | placeholders inert and counted; recovery mode shows them |
| Generation packs make old worlds unreproducible | silent terrain change | R-17 + AOD-21 pin/bake |
| Mod code security (later) | malicious mods | deferred to the technology decision; no code execution before it |

### 7.2 Questions for the owner

| # | Question | Agent recommendation | Status |
|---|---|---|---|
| Q-1 | Accept S-8: built-in content is a pack going through the same loader and registries? | Yes | OPEN |
| Q-2 | Accept namespaced stable ids `ns:name` + per-world id maps (R-02, R-03, R-11)? | Yes | OPEN |
| Q-3 | Missing/incompatible pack on load: block normal load with read-only and recovery options (5.3)? | Yes, block by default | OPEN |
| Q-4 | Allow continued play with placeholders (in recovery mode), or only read-only until the pack returns? | Allow, after explicit choice and backup | OPEN |
| Q-5 | May packs declare replacement mappings for other packs' ids (for example a successor mod), or only for their own? | Own ids only at first | OPEN |
| Q-6 | Version scheme for packs and for the extension API (semantic versioning, compatibility ranges)? | Semantic versions, exact version recorded per world, ranges for dependencies | OPEN |
| Q-7 | Are rule/stat changes in an updated pack (same ids) acceptable without confirmation? | Yes, but listed in the load summary | OPEN |
| Q-8 | Record this as AOD-25/AOD-26 in `owner-architecture-decisions-v1.md`, or keep it as this separate record referenced from there? | This separate file + one line per AOD in the main table when confirmed | OPEN |
| Q-9 | Resolution of X-1 (REF-16) | option (a) | OPEN |
| Q-10 | Is the first save format (M2) required to carry PackSet and IdMaps even though only the built-in pack exists? | Yes (cheap now, a rewrite later) | OPEN |

---

## 8. Recommendations for the future Rust Core architecture (PROPOSED)

1. **Keep Core content-agnostic.** Core stores compact per-asset palette indices (`u16` or wider, OPEN) and knows nothing about pack ids, names or definitions. The mapping lives above Core (AOD-04 stays intact; matches spec 3.5 "local palette -> global library").
2. **A separate content layer** (crate name OPEN, for example `cube-content`) owns: manifest parsing, dependency resolution, schema validation, registries, id maps. Pure Rust, no Godot, no I/O beyond reading pack data through an abstract source. Built-in content is loaded through it from day one (S-8).
3. **Game systems read definitions through registry views**, never through hard-coded tables or constants.
4. **Persistence stores ids through id maps** and carries PackSet and GenerationIdentity in the world record; recovery outcome codes include the pack cases (X-3).
5. **WorldGen consumes registries** for biomes/structures/materials and includes generation-relevant packs in its version identity (R-17), consistent with AOD-09 determinism.
6. **Extension surface crate** (later) exposes only declared, versioned interfaces; enforced by dependency rules like `cube-core/tests/dependency_rules.rs`.
7. **Every mod-originated effect is an operation of the authoritative runtime** (AOD-13/15/20/22); there is no mod write path.
8. **No code execution host before a technology decision**; when chosen, it runs inside the runtime with capability-limited APIs and no process spawning (AOD-12).
9. **Tests that keep the promise**: a save written with packs A+B loads unchanged with A+B; with B missing, normal load is refused and the file is byte-identical afterwards; placeholders round-trip; a migration rehearsal for a renamed id; a determinism test that the same pack set yields the same registries and id maps.

---

## 9. Proposed placement

* This record: `docs/decisions/owner-decision-extensibility-v1.md` (this file), next to the other owner decision records.
* After confirmation: one row each for AOD-25 and AOD-26 in `owner-architecture-decisions-v1.md` section 1 pointing here; REF-16 annotation (X-1); later Draft revisions of WP-01/02/08/09, WG-02 and the material-palette design (X-3...X-5). None of these edits is made now.
