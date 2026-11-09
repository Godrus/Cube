//! SPK-PERS durability spike (spike branch only; never merged into dev). Real-filesystem
//! experiments with the primitives the persistence design relies on. Each experiment asserts
//! what the design needs and prints what it observed (`cargo test --test spk_pers -- --nocapture`).
//! What a process-kill test proves is limited: killing a process does not lose data that is
//! already in the OS cache; only a power cut or kernel crash does, and that cannot be injected
//! here. Those guarantees belong to physical-device gate DG-2.
#![cfg(not(loom))]

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("spk-pers-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn fsync_file(p: &Path) {
    // Write access is needed to flush on Windows.
    std::fs::OpenOptions::new()
        .write(true)
        .open(p)
        .unwrap()
        .sync_all()
        .unwrap();
}

fn fsync_dir(d: &Path) -> &'static str {
    #[cfg(unix)]
    {
        std::fs::File::open(d).unwrap().sync_all().unwrap();
        "performed"
    }
    #[cfg(not(unix))]
    {
        let _ = d;
        "not available on this platform (no-op)"
    }
}

#[test]
fn e1_temp_file_fsync_rename_dir_sync_replaces_a_file_whole() {
    let d = dir("e1");
    let target = d.join("manifest");
    std::fs::write(&target, b"old-old-old").unwrap();
    fsync_file(&target);
    let tmp = d.join("manifest.tmp");
    std::fs::write(&tmp, b"new-new-new").unwrap();
    fsync_file(&tmp);
    std::fs::rename(&tmp, &target).unwrap(); // must replace an existing file
    let dirsync = fsync_dir(&d);
    assert_eq!(std::fs::read(&target).unwrap(), b"new-new-new");
    assert!(!tmp.exists());
    println!("E1 rename over an existing file: ok; directory fsync: {dirsync}");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn e2_append_fsync_and_truncate_behave_as_the_journal_assumes() {
    let d = dir("e2");
    let f = d.join("log");
    for chunk in [&b"AAAA"[..], b"BBBB", b"CCCC"] {
        let mut h = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&f)
            .unwrap();
        h.write_all(chunk).unwrap();
        h.sync_all().unwrap();
    }
    assert_eq!(std::fs::read(&f).unwrap(), b"AAAABBBBCCCC");
    std::fs::OpenOptions::new()
        .write(true)
        .open(&f)
        .unwrap()
        .set_len(8)
        .unwrap();
    assert_eq!(std::fs::read(&f).unwrap(), b"AAAABBBB");
    // An append after a truncate lands at the new end (append mode), not at the old offset.
    let mut h = std::fs::OpenOptions::new().append(true).open(&f).unwrap();
    h.write_all(b"DDDD").unwrap();
    assert_eq!(std::fs::read(&f).unwrap(), b"AAAABBBBDDDD");
    println!("E2 append, fsync, truncate, append after truncate: ok");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn e3_fsync_cost_is_measured_not_assumed() {
    let d = dir("e3");
    let f = d.join("log");
    let n = 50;
    let t = Instant::now();
    for _ in 0..n {
        let mut h = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&f)
            .unwrap();
        h.write_all(&[7u8; 512]).unwrap();
        h.sync_all().unwrap();
    }
    let per = t.elapsed() / n;
    println!("E3 append+fsync of 512 bytes: {per:?} each (this host; informational, not a gate)");
    assert!(
        per.as_secs() < 5,
        "an fsync taking seconds would make per-record durability unusable"
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// Child: replaces `state` atomically with ever newer complete versions, forever.
#[test]
fn e4_child() {
    let Ok(d) = std::env::var("SPK_CHILD_DIR") else {
        return;
    };
    let d = PathBuf::from(d);
    for v in 1u32.. {
        let body = vec![(v % 251) as u8; 4096 + (v as usize % 7) * 1000];
        let mut payload = v.to_le_bytes().to_vec();
        payload.extend_from_slice(&(body.len() as u32).to_le_bytes());
        payload.extend_from_slice(&body);
        let tmp = d.join("state.tmp");
        std::fs::write(&tmp, &payload).unwrap();
        fsync_file(&tmp);
        std::fs::rename(&tmp, d.join("state")).unwrap();
        println!("V {v}");
    }
}

#[test]
fn e4_a_killed_process_never_leaves_a_partial_replaced_file() {
    let d = dir("e4");
    let mut torn_tmp = 0;
    for round in 0..6 {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["e4_child", "--exact", "--nocapture", "--test-threads=1"])
            .env("SPK_CHILD_DIR", &d)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let out = child.stdout.take().unwrap();
        let mut last = 0u32;
        for line in std::io::BufReader::new(out).lines() {
            if let Some(v) = line.unwrap().strip_prefix("V ") {
                last = v.parse().unwrap();
                if last >= 5 + round * 3 {
                    break;
                }
            }
        }
        child.kill().unwrap();
        let _ = child.wait();
        let bytes = std::fs::read(d.join("state")).unwrap();
        let v = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        let len = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        assert_eq!(
            bytes.len(),
            8 + len,
            "round {round}: the published file is whole"
        );
        assert!(
            bytes[8..].iter().all(|b| *b == (v % 251) as u8),
            "round {round}: content matches its version"
        );
        assert!(
            v >= last,
            "round {round}: everything the child reported is published (v {v}, reported {last})"
        );
        if d.join("state.tmp").exists() {
            torn_tmp += 1; // a leftover temporary file is expected and harmless
        }
    }
    println!(
        "E4 six killed-process rounds: the published file was always whole; leftover temp files in {torn_tmp} rounds"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn e5_a_short_or_zeroed_file_after_a_crash_is_detectable_by_length_and_checksum() {
    // Simulates what a crash without a protocol can leave: a file extended to full length with
    // zeros (delayed allocation) or cut short. The codec's frame (length + CRC) must detect both.
    let d = dir("e5");
    let bytes = {
        use cube_persistence::{Limits, decode_asset, encode_asset};
        let view = cube_core::AssetView {
            id: cube_core::AssetId::new(1),
            epoch: cube_core::Epoch::INITIAL,
            revision: cube_core::Revision::new(3),
            layers: vec![],
        };
        let b = encode_asset(&view);
        assert!(decode_asset(&b, &Limits::default()).is_ok());
        let mut zeroed = b.clone();
        for x in zeroed[11..].iter_mut() {
            *x = 0;
        }
        assert!(decode_asset(&zeroed, &Limits::default()).is_err());
        assert!(decode_asset(&b[..b.len() - 1], &Limits::default()).is_err());
        b
    };
    std::fs::write(d.join("f"), &bytes).unwrap();
    println!("E5 zero-filled and truncated frames are rejected by length and CRC: ok");
    let _ = std::fs::remove_dir_all(&d);
}
