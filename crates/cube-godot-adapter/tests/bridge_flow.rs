//! Integration test of the adapter's pure-Rust bridge (what the Godot adapter calls).
//! It does not start Godot; that is a manual step documented in docs/spike/rust-godot.

use cube_godot_adapter::bridge::RuntimeBridge;

fn cell_count(cells: &[i32]) -> usize {
    cells.len() / 4
}

#[test]
fn edit_undo_redo_save_load_through_bridge() {
    let mut b = RuntimeBridge::new();

    let first = b.fill_region([-2, 0, -2, 2, 2, 2], 4).unwrap();
    assert!(!first.reset);
    assert_eq!(first.dirty, vec![[-2, 0, -2, 2, 2, 2]]);
    assert_eq!(cell_count(&first.cells), 4 * 2 * 4);
    assert_eq!(
        (first.asset_revision, first.layer_revision, first.epoch),
        (2, 1, 0)
    );
    assert_eq!(first.cells.chunks(4).next().unwrap()[3], 4);

    let undone = b.undo().unwrap();
    assert_eq!(undone.dirty, first.dirty);
    assert_eq!(cell_count(&undone.cells), 0); // client clears the region, nothing to add
    assert!(undone.layer_revision > first.layer_revision);

    let redone = b.redo().unwrap();
    assert_eq!(cell_count(&redone.cells), 32);

    let mut p = std::env::temp_dir();
    p.push(format!("cube_bridge_{}.cube", std::process::id()));
    b.save(&p).unwrap();
    b.fill_region([0, 0, 0, 1, 1, 1], 0).unwrap();
    let loaded = b.load(&p).unwrap();
    assert!(loaded.reset);
    assert_eq!(cell_count(&loaded.cells), 32);
    assert_eq!(loaded.epoch, 1);
    assert!(!b.can_undo()); // history dropped on load
    let _ = std::fs::remove_file(p);
}

#[test]
fn errors_are_coded_and_leave_state_alone() {
    let mut b = RuntimeBridge::new();
    let too_big = b.fill_region([0, 0, 0, 1000, 1000, 1000], 1).unwrap_err();
    assert_eq!(too_big.code, "ADPT-0001");
    assert_eq!(b.undo().unwrap_err().code, "EDIT-0002");
    assert_eq!(
        b.fill_region([1, 1, 1, 1, 4, 4], 1).unwrap_err().code,
        "EDIT-0001"
    );
    let missing = b
        .load(std::path::Path::new("/no/such/file.cube"))
        .unwrap_err();
    assert_eq!(missing.code, "PERS-0001");
    // Nothing was ever committed.
    let ok = b.fill_region([0, 0, 0, 1, 1, 1], 1).unwrap();
    assert_eq!((ok.asset_revision, ok.layer_revision), (2, 1));
}
