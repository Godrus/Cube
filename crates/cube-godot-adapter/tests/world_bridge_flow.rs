//! Integration test of the pure-Rust world bridge (what `CubeWorld` calls). Does not start Godot.

use cube_godot_adapter::world_bridge::{WorldBridge, FLOATS_PER_INSTANCE};

#[test]
fn create_stream_move_edit_save_load() {
    let mut b = WorldBridge::new(12345, 1);
    assert_eq!((b.seed(), b.radius()), (12345, 1));

    assert_eq!(b.set_player(0.5, 20.0, 0.5).queued, 9);
    assert_eq!(b.queue_len(), 9);
    assert_eq!(b.pump_all().generated, 9);
    assert_eq!(b.active_count(), 9);
    let added = b.take_added();
    assert_eq!(added.len(), 9);
    assert!(b.take_removed().is_empty());

    // Voxel data of a ready chunk and the renderer buffer describe the same cubes.
    let voxels = b.chunk_voxels(0, 0).unwrap();
    let n = voxels.len() / 4;
    assert!(n > 256 * 6);
    let buf = b.chunk_instance_buffer(0, 0).unwrap();
    assert_eq!(buf.len(), n * FLOATS_PER_INSTANCE);
    assert_eq!(&buf[3], &(voxels[0] as f32 + 0.5));
    assert_eq!(b.chunk_voxel_count(0, 0), Some(n));
    assert!(
        b.chunk_voxels(40, 40).is_none(),
        "unloaded chunk has no data"
    );

    // Move 4 chunks east: far chunks are removed, new ones queued.
    b.take_added();
    assert!(b.set_player(4.0 * 16.0 + 1.0, 20.0, 0.0).queued > 0);
    let removed = b.take_removed();
    assert!(removed.contains(&[0, 0]) && removed.contains(&[-1, -1]));
    b.pump_all();
    assert_eq!(b.active_count(), 9);
    assert!(b.active_chunks().iter().all(|c| c[0] >= 3 && c[0] <= 5));

    // Edit, save, change the world, load.
    let (px, pz) = (4 * 16 + 1, 1);
    let y = b.ground_height(px, pz) + 4;
    assert_eq!(b.edit_around(px, y, pz), 27);
    assert_eq!(b.take_modified(), vec![[4, 0]]);
    let voxels_before = b.active_voxels();
    let mut p = std::env::temp_dir();
    p.push(format!("cube_world_bridge_{}.wst0", std::process::id()));
    b.save(&p).unwrap();

    b.set_player(-500.0, 0.0, -500.0);
    b.pump_all();
    assert_ne!(b.active_voxels(), 0);
    let s = b.load(&p).unwrap();
    assert_eq!(
        (
            s.saved_active,
            s.restored_active,
            s.missing,
            s.edits_restored
        ),
        (9, 9, 0, 27)
    );
    assert_eq!(b.active_voxels(), voxels_before);
    assert_eq!(
        b.take_added().len(),
        9,
        "after load all active chunks are reported as added"
    );
    let _ = std::fs::remove_file(p);
}

#[test]
fn bad_input_is_handled() {
    let mut b = WorldBridge::new(1, 99); // radius is clamped
    assert_eq!(b.radius(), 8);
    b.set_player(f32::NAN, 0.0, f32::INFINITY);
    assert!(b.load(std::path::Path::new("/no/such/file.wst0")).is_err());
    assert_eq!(b.edit_around(0, 5, 0), 0, "nothing is loaded yet");
}
