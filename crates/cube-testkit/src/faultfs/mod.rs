//! Fault-injecting in-memory filesystem and crash-point runner (TEST-04).
//!
//! The model is deliberately the *weakest* POSIX-like contract a durable store must survive:
//!
//! * file data written through `write`/`append` is volatile until `fsync(path)`;
//! * names are volatile until `fsync_dir()` (create, rename and remove all live in the
//!   namespace);
//! * `rename` is atomic in the volatile namespace;
//! * on a crash everything volatile is dropped, and optionally a **torn** tail of unsynced data
//!   survives (a prefix of the bytes written since the last `fsync`).
//!
//! Every mutating call is a numbered *step*. [`explore`] runs a scenario once to count its steps
//! and then crashes it after every step (and before the first), under every torn-write mode,
//! calling the checker on the recovered state: "can crash after any numbered I/O step".
//!
//! It makes no durability claim about any real platform; platform behaviour is a device-gate
//! matter (DG-2). It is a harness for proving that a protocol is correct *under this model*.

use std::collections::BTreeMap;

/// What survives of unsynced data at a crash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Torn {
    /// Nothing unsynced survives.
    Nothing,
    /// Half of the bytes written since the last `fsync` survive.
    HalfOfUnsynced,
    /// All unsynced bytes survive (the lucky case: the disk happened to write them).
    AllOfUnsynced,
}

/// Errors of the simulated filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsError {
    /// The simulated crash already happened; no further call has any effect.
    Crashed,
    /// The path does not exist.
    NotFound(String),
}

#[derive(Debug, Clone, Default)]
struct Inode {
    volatile: Vec<u8>,
    durable: Vec<u8>,
}

/// The simulated filesystem.
#[derive(Debug, Clone, Default)]
pub struct FaultFs {
    inodes: Vec<Inode>,
    volatile_ns: BTreeMap<String, usize>,
    durable_ns: BTreeMap<String, usize>,
    steps: u64,
    crash_after: Option<u64>,
    crashed: bool,
}

impl FaultFs {
    /// A healthy filesystem.
    pub fn new() -> Self {
        Self::default()
    }

    /// A filesystem that crashes right after step number `step` completes (`0` = before any
    /// step has an effect).
    pub fn crashing_after(step: u64) -> Self {
        Self {
            crash_after: Some(step),
            crashed: step == 0,
            ..Self::default()
        }
    }

    /// Number of mutating steps performed so far.
    pub fn steps(&self) -> u64 {
        self.steps
    }

    /// `true` once the simulated crash has happened.
    pub fn has_crashed(&self) -> bool {
        self.crashed
    }

    fn step(&mut self) -> Result<(), FsError> {
        if self.crashed {
            return Err(FsError::Crashed);
        }
        self.steps += 1;
        if self.crash_after == Some(self.steps) {
            // The step takes effect; the next call finds the machine dead.
            self.crashed = true;
            self.crash_after = None;
            return Ok(());
        }
        Ok(())
    }

    fn inode_of(&mut self, path: &str) -> usize {
        if let Some(i) = self.volatile_ns.get(path) {
            return *i;
        }
        self.inodes.push(Inode::default());
        let i = self.inodes.len() - 1;
        self.volatile_ns.insert(path.to_owned(), i);
        i
    }

    /// Creates or truncates `path` and writes `data` (volatile).
    pub fn write(&mut self, path: &str, data: &[u8]) -> Result<(), FsError> {
        if self.crashed {
            return Err(FsError::Crashed);
        }
        let i = self.inode_of(path);
        // Truncate and rewrite: the old durable content stays durable until the next fsync.
        self.inodes[i].volatile = data.to_vec();
        self.step()
    }

    /// Appends `data` to `path`, creating it if needed (volatile).
    pub fn append(&mut self, path: &str, data: &[u8]) -> Result<(), FsError> {
        if self.crashed {
            return Err(FsError::Crashed);
        }
        let i = self.inode_of(path);
        self.inodes[i].volatile.extend_from_slice(data);
        self.step()
    }

    /// Shortens `path` to `len` bytes (volatile, like a write).
    pub fn truncate(&mut self, path: &str, len: usize) -> Result<(), FsError> {
        if self.crashed {
            return Err(FsError::Crashed);
        }
        let i = *self
            .volatile_ns
            .get(path)
            .ok_or_else(|| FsError::NotFound(path.to_owned()))?;
        self.inodes[i].volatile.truncate(len);
        self.step()
    }

    /// Makes the current content of `path` durable (not its name).
    pub fn fsync(&mut self, path: &str) -> Result<(), FsError> {
        if self.crashed {
            return Err(FsError::Crashed);
        }
        let i = *self
            .volatile_ns
            .get(path)
            .ok_or_else(|| FsError::NotFound(path.to_owned()))?;
        self.inodes[i].durable = self.inodes[i].volatile.clone();
        self.step()
    }

    /// Makes the current namespace (names of created, renamed and removed files) durable.
    pub fn fsync_dir(&mut self) -> Result<(), FsError> {
        if self.crashed {
            return Err(FsError::Crashed);
        }
        self.durable_ns = self.volatile_ns.clone();
        self.step()
    }

    /// Atomically renames `from` to `to`, replacing `to`.
    pub fn rename(&mut self, from: &str, to: &str) -> Result<(), FsError> {
        if self.crashed {
            return Err(FsError::Crashed);
        }
        let i = self
            .volatile_ns
            .remove(from)
            .ok_or_else(|| FsError::NotFound(from.to_owned()))?;
        self.volatile_ns.insert(to.to_owned(), i);
        self.step()
    }

    /// Removes `path`.
    pub fn remove(&mut self, path: &str) -> Result<(), FsError> {
        if self.crashed {
            return Err(FsError::Crashed);
        }
        self.volatile_ns
            .remove(path)
            .ok_or_else(|| FsError::NotFound(path.to_owned()))?;
        self.step()
    }

    /// Reads the current (volatile) content; after [`FaultFs::recover`] this is the recovered one.
    pub fn read(&self, path: &str) -> Option<Vec<u8>> {
        self.volatile_ns
            .get(path)
            .map(|i| self.inodes[*i].volatile.clone())
    }

    /// Names present, ascending.
    pub fn list(&self) -> Vec<String> {
        self.volatile_ns.keys().cloned().collect()
    }

    /// The machine comes back after a crash: volatile state is dropped according to `torn` and
    /// the filesystem works again with exactly what survived.
    pub fn recover(&mut self, torn: Torn) {
        let mut inodes = std::mem::take(&mut self.inodes);
        for n in &mut inodes {
            let keep = match torn {
                Torn::Nothing => 0,
                Torn::HalfOfUnsynced => unsynced(n) / 2,
                Torn::AllOfUnsynced => unsynced(n),
            };
            let common = n
                .durable
                .iter()
                .zip(&n.volatile)
                .take_while(|(a, b)| a == b)
                .count();
            // Unsynced bytes are those appended past the common durable prefix.
            let mut survived = n.durable.clone();
            if keep > 0 && n.volatile.len() >= common {
                survived.truncate(common);
                survived.extend_from_slice(
                    &n.volatile[common..common + keep.min(n.volatile.len() - common)],
                );
            }
            n.durable = survived.clone();
            n.volatile = survived;
        }
        self.inodes = inodes;
        self.volatile_ns = self.durable_ns.clone();
        self.crashed = false;
        self.crash_after = None;
    }
}

fn unsynced(n: &Inode) -> usize {
    let common = n
        .durable
        .iter()
        .zip(&n.volatile)
        .take_while(|(a, b)| a == b)
        .count();
    n.volatile.len().saturating_sub(common)
}

/// A point where a checker failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashFailure {
    /// The scenario was crashed after this step (0 = before the first).
    pub after_step: u64,
    /// The torn-write mode in force.
    pub torn: Torn,
    /// What the checker said.
    pub message: String,
}

/// Runs `scenario` once to count its steps, then once per crash point and torn mode, and calls
/// `check` on the recovered filesystem. Returns every failure (empty = the protocol survives
/// every crash point under this model). The scenario must tolerate `FsError::Crashed` by simply
/// returning (use `?`).
pub fn explore(
    scenario: impl Fn(&mut FaultFs) -> Result<(), FsError>,
    check: impl Fn(&FaultFs, u64) -> Result<(), String>,
) -> Vec<CrashFailure> {
    let mut healthy = FaultFs::new();
    let _ = scenario(&mut healthy);
    let total = healthy.steps();
    let mut failures = Vec::new();
    for k in 0..=total {
        for torn in [Torn::Nothing, Torn::HalfOfUnsynced, Torn::AllOfUnsynced] {
            let mut fs = FaultFs::crashing_after(k);
            let _ = scenario(&mut fs);
            fs.recover(torn);
            if let Err(message) = check(&fs, k) {
                failures.push(CrashFailure {
                    after_step: k,
                    torn,
                    message,
                });
            }
        }
    }
    failures
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_is_volatile_until_fsync_and_names_until_fsync_dir() {
        let mut fs = FaultFs::new();
        fs.write("a", b"hello").unwrap();
        assert_eq!(fs.read("a"), Some(b"hello".to_vec()));
        fs.recover(Torn::Nothing);
        assert_eq!(fs.read("a"), None, "the name was never made durable");

        let mut fs = FaultFs::new();
        fs.write("a", b"hello").unwrap();
        fs.fsync_dir().unwrap();
        fs.recover(Torn::Nothing);
        assert_eq!(
            fs.read("a"),
            Some(vec![]),
            "the name survived, the unsynced data did not"
        );

        let mut fs = FaultFs::new();
        fs.write("a", b"hello").unwrap();
        fs.fsync("a").unwrap();
        fs.fsync_dir().unwrap();
        fs.recover(Torn::Nothing);
        assert_eq!(fs.read("a"), Some(b"hello".to_vec()));
    }

    #[test]
    fn torn_writes_keep_a_prefix_of_unsynced_bytes() {
        let mut fs = FaultFs::new();
        fs.append("log", b"AAAA").unwrap();
        fs.fsync("log").unwrap();
        fs.fsync_dir().unwrap();
        fs.append("log", b"BBBBBBBB").unwrap();
        let mut half = fs.clone();
        half.recover(Torn::HalfOfUnsynced);
        assert_eq!(half.read("log"), Some(b"AAAABBBB".to_vec()));
        let mut all = fs.clone();
        all.recover(Torn::AllOfUnsynced);
        assert_eq!(all.read("log"), Some(b"AAAABBBBBBBB".to_vec()));
        fs.recover(Torn::Nothing);
        assert_eq!(fs.read("log"), Some(b"AAAA".to_vec()));
    }

    #[test]
    fn rename_replaces_atomically_and_remove_works() {
        let mut fs = FaultFs::new();
        fs.write("x", b"1").unwrap();
        fs.write("y", b"2").unwrap();
        fs.rename("x", "y").unwrap();
        assert_eq!((fs.read("x"), fs.read("y")), (None, Some(b"1".to_vec())));
        assert_eq!(
            fs.rename("nope", "z"),
            Err(FsError::NotFound("nope".into()))
        );
        fs.remove("y").unwrap();
        assert!(fs.list().is_empty());
        assert_eq!(fs.fsync("y"), Err(FsError::NotFound("y".into())));
    }

    #[test]
    fn a_crash_after_step_k_stops_the_scenario_there() {
        let mut fs = FaultFs::crashing_after(2);
        assert!(fs.write("a", b"1").is_ok());
        assert!(fs.write("b", b"2").is_ok());
        assert!(fs.has_crashed());
        assert_eq!(fs.write("c", b"3"), Err(FsError::Crashed));
        assert_eq!(fs.steps(), 2);
        assert_eq!(
            FaultFs::crashing_after(0).write("a", b""),
            Err(FsError::Crashed)
        );
    }

    /// The textbook replace-a-file protocol: write a temp file, fsync it, rename over the target,
    /// fsync the directory. Before the scenario the old version is durable.
    fn atomic_replace(fs: &mut FaultFs) -> Result<(), FsError> {
        fs.write("data", b"old-old-old")?;
        fs.fsync("data")?;
        fs.fsync_dir()?;
        fs.write("data.tmp", b"new-new-new")?;
        fs.fsync("data.tmp")?;
        fs.rename("data.tmp", "data")?;
        fs.fsync_dir()
    }

    fn old_or_new(fs: &FaultFs, k: u64) -> Result<(), String> {
        // Before the baseline (steps 1..=3) is complete anything goes; afterwards the file must
        // be exactly the old or exactly the new content, never partial or missing.
        if k < 3 {
            return Ok(());
        }
        match fs.read("data").as_deref() {
            Some(b"old-old-old") | Some(b"new-new-new") => Ok(()),
            other => Err(format!("data is {other:?}")),
        }
    }

    #[test]
    fn the_correct_protocol_survives_every_crash_point_and_torn_mode() {
        assert_eq!(explore(atomic_replace, old_or_new), vec![]);
    }

    #[test]
    fn dropping_the_fsync_is_caught_at_a_concrete_crash_point() {
        let buggy = |fs: &mut FaultFs| -> Result<(), FsError> {
            fs.write("data", b"old-old-old")?;
            fs.fsync("data")?;
            fs.fsync_dir()?;
            fs.write("data.tmp", b"new-new-new")?;
            // missing fsync of the temp file
            fs.rename("data.tmp", "data")?;
            fs.fsync_dir()
        };
        let failures = explore(buggy, old_or_new);
        assert!(!failures.is_empty(), "the missing fsync must be found");
        assert!(failures.iter().any(|f| f.message.contains("data is")));
        // Every failure is at or after the rename became durable, in a mode that dropped data.
        assert!(failures.iter().all(|f| f.after_step >= 5));
    }

    #[test]
    fn writing_in_place_without_any_protocol_is_caught() {
        let in_place = |fs: &mut FaultFs| -> Result<(), FsError> {
            fs.write("data", b"old-old-old")?;
            fs.fsync("data")?;
            fs.fsync_dir()?;
            fs.write("data", b"new-new-new")?;
            fs.fsync("data")
        };
        let failures = explore(in_place, old_or_new);
        assert!(
            failures
                .iter()
                .any(|f| f.torn == Torn::HalfOfUnsynced || f.torn == Torn::Nothing)
        );
    }

    #[test]
    fn every_numbered_step_is_a_crash_point() {
        let mut healthy = FaultFs::new();
        atomic_replace(&mut healthy).unwrap();
        let seen = std::cell::RefCell::new(Vec::new());
        let _ = explore(atomic_replace, |_, k| {
            seen.borrow_mut().push(k);
            Ok(())
        });
        let seen = seen.into_inner();
        for k in 0..=healthy.steps() {
            assert_eq!(
                seen.iter().filter(|s| **s == k).count(),
                3,
                "step {k} under three torn modes"
            );
        }
    }
}
