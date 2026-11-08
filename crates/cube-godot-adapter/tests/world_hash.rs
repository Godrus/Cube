//! P-WORLD stage A: the world hash used by the client to compare runs (experimental, not a format).

use cube_godot_adapter::world_bridge::WorldBridge;

fn hash(seed: u64, radius: i32) -> u64 {
    let mut b = WorldBridge::new(seed, radius);
    b.set_player(8.0, 10.0, 8.0);
    b.pump_all();
    b.world_hash()
}

#[test]
fn same_seed_and_radius_give_the_same_world_hash() {
    assert_eq!(hash(12345, 2), hash(12345, 2));
}

#[test]
fn different_seeds_and_radii_give_different_world_hashes() {
    let seeds = [1u64, 7, 12345, 424242, u64::MAX];
    let mut all: Vec<u64> = seeds.iter().map(|s| hash(*s, 2)).collect();
    all.push(hash(12345, 3));
    let n = all.len();
    all.sort_unstable();
    all.dedup();
    assert_eq!(all.len(), n);
}

#[test]
fn an_edit_changes_the_world_hash() {
    let mut b = WorldBridge::new(12345, 1);
    b.set_player(8.0, 10.0, 8.0);
    b.pump_all();
    let before = b.world_hash();
    assert!(b.edit_voxel(8, 25, 8, 4));
    assert_ne!(before, b.world_hash());
}
