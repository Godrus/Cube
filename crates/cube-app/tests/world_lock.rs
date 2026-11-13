//! M1-A5, concurrent writers: a world directory is open for writing in one host at a time.
use std::path::PathBuf;

use cube_core::{GeneratorVersion, WorldSeed};
use cube_persistence::WorldRecord;
use cube_server::{HostError, LocalHost, StaticArea};

fn dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("cube-wlock-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn record() -> WorldRecord {
    WorldRecord {
        world_id: 1,
        seed: WorldSeed::new(7),
        generator_version: GeneratorVersion::new(1),
        structure_version: cube_persistence::STRUCTURE_VERSION,
    }
}

#[test]
fn a_world_is_open_for_writing_in_one_host_at_a_time() {
    let d = dir("one");
    let rec = record();
    let host = LocalHost::create(&d, rec, StaticArea::around_origin(&rec, 1)).unwrap();
    // A second host on the same directory is refused at once, whatever it wants to do.
    match LocalHost::open(&d, StaticArea::around_origin(&rec, 1)) {
        Err(HostError::Persistence(m)) => assert!(m.contains("already open"), "{m}"),
        Err(other) => panic!("wrong error: {other:?}"),
        Ok(_) => panic!("two hosts wrote to one world"),
    }
    // The first host is unharmed by the refused attempt.
    assert!(host.loaded_bricks() > 0);
    drop(host);
    // Once the writer is gone the world opens again, with its content.
    let again = LocalHost::open(&d, StaticArea::around_origin(&rec, 1)).unwrap();
    assert!(again.loaded_bricks() > 0);
    drop(again);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn another_directory_is_not_blocked_by_an_open_world() {
    let (a, b) = (dir("a"), dir("b"));
    let rec = record();
    let _first = LocalHost::create(&a, rec, StaticArea::around_origin(&rec, 1)).unwrap();
    let second = LocalHost::create(&b, rec, StaticArea::around_origin(&rec, 1));
    assert!(second.is_ok());
    let _ = std::fs::remove_dir_all(&a);
    let _ = std::fs::remove_dir_all(&b);
}
