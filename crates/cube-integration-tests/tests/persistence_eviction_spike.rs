//! M1-A1 spike (test only, no API or journal change): can an edited brick be unloaded?
//!
//! The trap (docs/roadmap/WORK-PACKAGES.md, M1-A1 design notes): every unload and reload of a
//! brick raises its revision by one, the journal records the revision each edit expected, and a
//! restart replays the journal onto a regenerated base. If an edited brick is unloaded and
//! reloaded between two of its edits, the second edit expects a revision that a replay from
//! scratch does not reach.
//!
//! Test 1 reproduces the failure of "just unload it". Test 2 spikes option 2 of the notes (an
//! edited-brick store: the content and revision at unload are kept, a restart restores the stored
//! content, catches the revision up with the same unload and reload steps, then replays the
//! journal) and shows that the existing `replay_confirmed` reaches the same world. Test 3 shows
//! the result with repeated eviction cycles. Nothing here is a design decision: the result and
//! the open points are in WORK-PACKAGES, section "M1-A1 spike result".
#![cfg(not(loom))]

use std::collections::BTreeMap;
use std::sync::Arc;

use cube_backend_voxel::{VoxelBackend, set_material};
use cube_core::{
    BrickCoord, CellChange, CellPayload, CellState, ChannelSet, LayerId, MaterialIndex,
    OperationId, Region, VoxelPos,
};
use cube_editing_api::{EditMode, EditingOperation, kinds};
use cube_integration_tests::fakes::*;
use cube_persistence::{Limits, replay_confirmed};

const B: i64 = 0; // the brick under test is (0, 0, 0)

fn brick() -> BrickCoord {
    bc(B, 0, 0)
}

/// The generator's base content of the brick: material 1 everywhere.
fn base() -> CellChange {
    CellChange::Put(CellPayload::Channels(ChannelSet::uniform_material(
        MaterialIndex::from_raw(1),
    )))
}

/// A user edit that sets the whole brick to `material` (operation number `n`).
fn edit(n: u64, material: u16) -> EditingOperation {
    EditingOperation {
        id: OperationId::new(71, n),
        kind: kinds::SET_MATERIAL,
        asset: cube_core::AssetId::new(ASSET),
        layers: vec![LayerId::new(L_VOXEL)],
        parameters: set_material(material, None),
        read_region: None,
        write_region: Region::new(0, VoxelPos::new(0, 0, 0), VoxelPos::new(16, 16, 16)).unwrap(),
        conversion: None,
        mode: EditMode::Destructive,
    }
}

fn boot(journal: &Arc<FaultyJournal>) -> Fixture {
    fixture_with(Options {
        backends: vec![VoxelBackend::arc()],
        with_payload: true,
        shared_journal: Some(Arc::clone(journal)),
        ..Options::default()
    })
}

fn revision_of(f: &Fixture) -> u64 {
    f.reader
        .snapshot()
        .cell_token(LayerId::new(L_VOXEL), 0, brick())
        .map_or(0, |t| t.value().raw())
}

fn materialise(f: &Fixture, n: u64, change: CellChange) {
    f.system
        .materialise(
            OperationId::new(0, n),
            LayerId::new(L_VOXEL),
            0,
            brick(),
            change,
        )
        .expect("materialise");
}

fn evict(f: &Fixture) {
    assert!(
        f.system
            .evict(LayerId::new(L_VOXEL), 0, brick())
            .expect("evict")
    );
}

/// What the brick holds now, as a change that would store it again.
fn current_content(f: &Fixture) -> CellChange {
    let snap = f.reader.snapshot();
    match snap
        .cell(LayerId::new(L_VOXEL), 0, brick())
        .unwrap()
        .unwrap()
    {
        CellState::Present(p) => CellChange::Put((*p).clone()),
        other => panic!("the brick is not present: {other:?}"),
    }
}

fn hash(f: &Fixture) -> cube_core::ContentHash {
    f.reader.snapshot().export_view().content_hash()
}

#[test]
fn just_unloading_an_edited_brick_makes_replay_lose_the_edit_made_after_the_reload() {
    let journal = Arc::new(FaultyJournal::default());
    // Life 1: load, edit, unload, reload (the base comes back), edit again.
    let f = boot(&journal);
    materialise(&f, 1, base());
    assert_eq!(revision_of(&f), 1);
    assert!(f.run(vec![edit(1, 9)]).is_success());
    assert_eq!(revision_of(&f), 2);
    evict(&f);
    materialise(&f, 2, base());
    assert_eq!(revision_of(&f), 3, "a reload is one revision");
    assert_eq!(
        f.material(brick()),
        Some(1),
        "the first edit is gone from memory: the base was regenerated"
    );
    assert!(f.run(vec![edit(2, 7)]).is_success());
    assert_eq!(f.material(brick()), Some(7));
    let truth = hash(&f);
    drop(f);

    // Life 2: regenerate the base and replay the journal.
    let g = boot(&journal);
    materialise(&g, 1, base());
    let report = replay_confirmed(&journal.entries(), &g.recovery, &Limits::default());
    // The first edit applies; the second expected revision 3 and the restored brick is at 2.
    assert_eq!(report.replayed, vec![OperationId::new(71, 1)]);
    assert_eq!(
        g.material(brick()),
        Some(9),
        "the world after the restart has the first edit only"
    );
    assert_ne!(
        hash(&g),
        truth,
        "the restart does not reach the world before it"
    );
    assert!(
        report
            .already_applied_or_superseded
            .contains(&OperationId::new(71, 2))
            || report
                .failed
                .iter()
                .any(|(id, _)| *id == OperationId::new(71, 2)),
        "the second edit is reported, not hidden: {report:?}"
    );
}

/// The stored brick of the spike: content and revision at the moment of unloading, kept outside
/// the asset (a map here, a checksummed file per brick in a real design).
struct Stored {
    content: CellChange,
    revision_after_reload: u64,
}

/// Unloads the brick keeping its content, and reloads it from that content: the edit stays in
/// memory and the reload is one revision, exactly as in the failing case.
fn unload_and_reload_from_store(f: &Fixture, n: u64, store: &mut BTreeMap<i64, Stored>) {
    let content = current_content(f);
    let before = revision_of(f);
    evict(f);
    materialise(f, n, content.clone());
    store.insert(
        B,
        Stored {
            content,
            revision_after_reload: revision_of(f),
        },
    );
    assert_eq!(revision_of(f), before + 1);
}

/// Restores the stored brick in a fresh world and brings its revision to what it was after the
/// last reload of the earlier life, with the same unload and reload steps `LocalHost` already
/// uses for unedited bricks (`bring_to_revision`).
fn restore_from_store(g: &Fixture, store: &BTreeMap<i64, Stored>) {
    let s = &store[&B];
    materialise(g, 1, s.content.clone());
    let mut n = 1;
    while revision_of(g) < s.revision_after_reload {
        n += 1;
        evict(g);
        materialise(g, n, s.content.clone());
    }
    assert_eq!(revision_of(g), s.revision_after_reload);
}

#[test]
fn an_edited_brick_store_with_revision_catch_up_lets_the_existing_replay_reach_the_same_world() {
    let journal = Arc::new(FaultyJournal::default());
    let mut store = BTreeMap::new();
    // Life 1: load, edit, unload to the store and reload from it, edit again.
    let f = boot(&journal);
    materialise(&f, 1, base());
    assert!(f.run(vec![edit(1, 9)]).is_success());
    unload_and_reload_from_store(&f, 2, &mut store);
    assert_eq!(
        f.material(brick()),
        Some(9),
        "the first edit survives the cycle"
    );
    assert!(f.run(vec![edit(2, 7)]).is_success());
    let (truth, truth_rev) = (hash(&f), revision_of(&f));
    assert_eq!(truth_rev, 4);
    drop(f);

    // Life 2: restore the stored brick, catch the revision up, replay the journal.
    let g = boot(&journal);
    restore_from_store(&g, &store);
    let report = replay_confirmed(&journal.entries(), &g.recovery, &Limits::default());
    assert!(report.failed.is_empty(), "nothing is lost: {report:?}");
    assert_eq!(
        report.replayed,
        vec![OperationId::new(71, 2)],
        "the edit before the cycle is already in the stored brick, the one after is replayed"
    );
    assert!(
        report
            .already_applied_or_superseded
            .contains(&OperationId::new(71, 1)),
        "{report:?}"
    );
    assert_eq!(g.material(brick()), Some(7));
    assert_eq!(
        hash(&g),
        truth,
        "the world after the restart is the world before it"
    );
    assert_eq!(revision_of(&g), truth_rev, "and so is the revision");
}

#[test]
fn repeated_cycles_keep_the_world_and_the_revision_reachable() {
    let journal = Arc::new(FaultyJournal::default());
    let mut store = BTreeMap::new();
    let f = boot(&journal);
    materialise(&f, 1, base());
    // Edit, unload and reload, edit, unload and reload, edit: three edits, two cycles.
    assert!(f.run(vec![edit(1, 3)]).is_success());
    unload_and_reload_from_store(&f, 2, &mut store);
    assert!(f.run(vec![edit(2, 4)]).is_success());
    unload_and_reload_from_store(&f, 3, &mut store);
    assert!(f.run(vec![edit(3, 5)]).is_success());
    let (truth, truth_rev) = (hash(&f), revision_of(&f));
    drop(f);

    let g = boot(&journal);
    restore_from_store(&g, &store);
    let report = replay_confirmed(&journal.entries(), &g.recovery, &Limits::default());
    assert!(report.failed.is_empty(), "{report:?}");
    assert_eq!(g.material(brick()), Some(5));
    assert_eq!(hash(&g), truth);
    assert_eq!(revision_of(&g), truth_rev);
    // The journal holds the three user operations and nothing for the cycles: system unloads and
    // reloads are not journaled in this spike.
    let mut ops: Vec<OperationId> = journal.entries().iter().map(|e| e.operation).collect();
    ops.dedup();
    assert_eq!(ops.len(), 3, "{ops:?}");
}
