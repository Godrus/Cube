//! Transaction lifecycle, rollback and the editing service (ORCH-01, E-04, E-11).
//!
//! Path of a request: authorization, shape validation, retry lookup, backend dispatch by
//! capabilities, read-only overlay, write sets from the backend, write-set checks, staging in the
//! private overlay (`execute`); then at `commit`: expected-revision batch, write-ahead journal
//! intent (when a journal is configured), one atomic Core commit through the single user commit
//! path, journal confirmation, dirty propagation, observer notification.
//!
//! Atomicity: nothing authoritative changes before the Core commit, and Core applies a batch
//! entirely or not at all; a transaction that fails, conflicts, is rolled back or cancelled
//! leaves the world unchanged. There are no nested transactions and no partial commits.
//!
//! Journal (DEC-086, blocker CB-1): Core commit and the durable journal are not one atomic
//! action, so the journal is written ahead (`Intent`), confirmed after the commit, or aborted.
//! If the confirmation cannot be made durable the result is `AppliedUnconfirmed`, never
//! `Success`; a retry with the same `OperationId` completes the confirmation without applying
//! twice. Crash durability is not claimed here (PS-10).
//!
//! Lock order: transaction, then commit section, then the small tables. Observers run inside
//! the commit section (that is what makes their order equal to the commit order) and must not
//! call back into the service.

use std::collections::{BTreeMap, HashMap};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use cube_core::{
    AssetId, AssetState, BrickCoord, CellChange, CellKey, CellState, CellWrite, CommitBatch,
    CommitError, CommitOutcome, ExpectedRevision, LayerId, OperationId, ResourceId, RevisionToken,
    SharedAsset,
};
use cube_editing_api::{
    ActorId, AuthorizationContext, ConversionPolicy, DeltaId, EditingError, EditingOperation,
    EditingResult, EditingService, JournalEntry, JournalPhase, JournalSeq, OperationJournal,
    OperationStatus, Origin, OverlayRead, Success, TransactionHandle, TransactionId,
    TransactionObserver, TransactionSpec, TransactionState, TransactionSummary,
};

use crate::auth::Authorizer;
use crate::commit_path::CommitPath;
use crate::dirty::{DirtySink, Publisher};
use crate::registry::BackendRegistry;
use crate::validate::{
    Replay, RetryTable, check_expected_tokens, check_write_sets, conflict_result, expected_of,
};

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Everything the service needs.
pub struct Config {
    /// The single user commit path.
    pub path: CommitPath,
    /// Backends by layer kind.
    pub registry: BackendRegistry,
    /// Authorization port.
    pub authorizer: Arc<dyn Authorizer>,
    /// The durable operation journal; `None` = operations are not journalled (no durability
    /// claim is made for them).
    pub journal: Option<Arc<dyn OperationJournal>>,
    /// Receives the dirty regions of every commit (Processing invalidation).
    pub dirty: Option<Arc<dyn DirtySink>>,
    /// Observers, notified in this order after every commit or rollback.
    pub observers: Vec<Arc<dyn TransactionObserver>>,
    /// How many committed operation ids are remembered for idempotent retry.
    pub retry_capacity: usize,
}

struct Staged {
    change: CellChange,
    expected: ExpectedRevision,
}

struct Txn {
    id: TransactionId,
    state: TransactionState,
    context: AuthorizationContext,
    spec: TransactionSpec,
    base: Arc<AssetState>,
    staged: BTreeMap<(LayerId, i8, BrickCoord), Staged>,
    ops: Vec<EditingOperation>,
    replayed: Vec<EditingResult>,
}

/// The transaction as its backends see it: the base state plus what this transaction staged.
struct Overlay<'a>(&'a Txn);

impl OverlayRead for Overlay<'_> {
    fn cell(&self, key: CellKey) -> Result<CellState, EditingError> {
        if let Some(s) = self.0.staged.get(&(key.layer(), key.level(), key.brick())) {
            return Ok(match &s.change {
                CellChange::Put(p) => CellState::Present(Arc::new(p.clone())),
                CellChange::KnownEmpty => CellState::KnownEmpty,
            });
        }
        match self.0.base.cell(key.layer(), key.level(), key.brick()) {
            Some(Ok(c)) => Ok(c),
            Some(Err(_)) => Err(EditingError::InvalidResolution),
            None => Err(EditingError::InvalidOperation("layer has no cell store")),
        }
    }
}

struct Txs {
    next: u64,
    map: BTreeMap<TransactionId, Arc<Mutex<Txn>>>,
    by_op: HashMap<OperationId, TransactionId>,
}

struct PendingConfirm {
    operation: OperationId,
    actor: ActorId,
    asset: AssetId,
    post_tokens: Vec<RevisionToken>,
    members: Vec<(OperationId, u64)>,
}

/// What [`OrchestrationService::recover_idempotency`] found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecoveryReport {
    /// Confirmed transactions remembered again (retries are answered, never re-applied).
    pub recorded: usize,
    /// Confirmed transactions the restored asset does not contain: changes lost since the
    /// snapshot. Their ids can be applied again.
    pub lost_since_snapshot: Vec<OperationId>,
    /// Transactions with an `Intent` and no later record: undecided, retries are refused.
    pub blocked: Vec<OperationId>,
    /// Aborted transactions (nothing to do).
    pub aborted: usize,
}

/// The editing service of one asset (E-11).
pub struct OrchestrationService {
    path: CommitPath,
    reader: SharedAsset,
    registry: BackendRegistry,
    authorizer: Arc<dyn Authorizer>,
    journal: Option<Arc<dyn OperationJournal>>,
    publisher: Publisher,
    txs: Mutex<Txs>,
    retry: Mutex<RetryTable>,
    pending: Mutex<Vec<PendingConfirm>>,
    undecided: Mutex<HashMap<OperationId, Vec<(OperationId, u64)>>>,
    section: Mutex<()>,
    next_delta: AtomicU64,
}

impl std::fmt::Debug for OrchestrationService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrchestrationService")
            .field("asset", &self.reader.id())
            .finish_non_exhaustive()
    }
}

/// Maps a controlled error to the result variant that names it (E-05).
fn classify(e: EditingError) -> EditingResult {
    match e {
        EditingError::UnsupportedGeometryOperation(_) | EditingError::UnknownKind(_) => {
            EditingResult::Unsupported(e)
        }
        EditingError::ConversionRequired => EditingResult::ConversionRequired(e),
        EditingError::ProcessingFailed
        | EditingError::ProcessingCancelled
        | EditingError::PersistenceFailure => EditingResult::ResourceFailure(e),
        other => EditingResult::ValidationFailure(other),
    }
}

fn core_failure(e: CommitError) -> EditingResult {
    match e {
        CommitError::UnknownLayer(_) => {
            EditingResult::ValidationFailure(EditingError::InvalidOperation("unknown layer"))
        }
        CommitError::PayloadKindMismatch(_) => EditingResult::ValidationFailure(
            EditingError::InvalidOperation("payload does not fit the layer"),
        ),
        CommitError::OutsideBounds { .. } => EditingResult::ValidationFailure(
            EditingError::InvalidOperation("write outside the layer bounds"),
        ),
        CommitError::DuplicateWrite { .. } => {
            EditingResult::ValidationFailure(EditingError::InvalidOperation("cell written twice"))
        }
        CommitError::Core(cube_core::CoreError::InvalidLevel(_)) => {
            EditingResult::ValidationFailure(EditingError::InvalidResolution)
        }
        // A conflict is handled by the caller; anything else is a limit of Core itself.
        _ => EditingResult::ResourceFailure(EditingError::ProcessingFailed),
    }
}

impl OrchestrationService {
    /// Creates the service of the asset behind `config.path`.
    pub fn new(config: Config) -> Self {
        let reader = config.path.reader();
        Self {
            path: config.path,
            reader,
            registry: config.registry,
            authorizer: config.authorizer,
            journal: config.journal,
            publisher: Publisher::new(config.dirty, config.observers),
            txs: Mutex::new(Txs {
                next: 0,
                map: BTreeMap::new(),
                by_op: HashMap::new(),
            }),
            retry: Mutex::new(RetryTable::new(config.retry_capacity)),
            pending: Mutex::new(Vec::new()),
            undecided: Mutex::new(HashMap::new()),
            section: Mutex::new(()),
            next_delta: AtomicU64::new(1),
        }
    }

    /// Read access to the asset this service edits.
    pub fn reader(&self) -> SharedAsset {
        self.reader.clone()
    }

    /// Number of observer or dirty-sink failures that were isolated so far.
    pub fn isolated_failures(&self) -> u64 {
        self.publisher.failures()
    }

    /// Number of applied operations whose journal confirmation is still outstanding.
    pub fn unconfirmed(&self) -> usize {
        lock(&self.pending).len()
    }

    fn tx(&self, h: &TransactionHandle) -> Result<Arc<Mutex<Txn>>, EditingError> {
        lock(&self.txs)
            .map
            .get(&h.id)
            .cloned()
            .ok_or(EditingError::UnknownTransaction(h.id))
    }

    fn finish(&self, tx: &Txn) {
        let mut t = lock(&self.txs);
        t.map.remove(&tx.id);
        for op in &tx.ops {
            t.by_op.remove(&op.id);
        }
    }

    fn summary(
        tx: &Txn,
        tokens: Vec<RevisionToken>,
        dirty: Vec<cube_core::DirtyRegion>,
    ) -> TransactionSummary {
        TransactionSummary {
            transaction: tx.id,
            operations: tx.ops.iter().map(|o| o.id).collect(),
            asset: tx.spec.asset,
            actor: tx.context.actor.0,
            new_tokens: tokens,
            dirty,
        }
    }

    fn journal_append(
        &self,
        phase: JournalPhase,
        operation: OperationId,
        actor: ActorId,
        asset: AssetId,
        post_tokens: Vec<RevisionToken>,
        members: Vec<(OperationId, u64)>,
    ) -> Result<JournalSeq, EditingError> {
        let Some(j) = &self.journal else {
            return Ok(JournalSeq(0));
        };
        j.append(JournalEntry {
            seq: JournalSeq(0),
            phase,
            operation,
            actor,
            asset,
            post_tokens,
            delta: None,
            members,
        })
    }

    /// Tries to make every outstanding journal confirmation durable. Returns how many succeeded;
    /// the retry table is upgraded from `AppliedUnconfirmed` to `Success` for them.
    pub fn reconcile(&self) -> usize {
        let todo = std::mem::take(&mut *lock(&self.pending));
        let mut still = Vec::new();
        let mut done = 0;
        for p in todo {
            match self.journal_append(
                JournalPhase::Confirmed,
                p.operation,
                p.actor,
                p.asset,
                p.post_tokens.clone(),
                p.members.clone(),
            ) {
                Ok(_) => {
                    done += 1;
                    let mut r = lock(&self.retry);
                    if let Some(EditingResult::AppliedUnconfirmed(s)) =
                        r.result_of(p.operation).cloned()
                    {
                        r.update_result(p.operation, EditingResult::Success(s));
                    }
                }
                Err(_) => still.push(p),
            }
        }
        lock(&self.pending).extend(still);
        done
    }

    /// Number of operation ids the retry table currently remembers (bounded).
    pub fn remembered(&self) -> usize {
        lock(&self.retry).len()
    }

    /// Rebuilds the idempotency state from the durable journal after a restart (TD-4). Call it
    /// once, before serving, on a service whose asset was restored from a snapshot.
    ///
    /// * `Confirmed` records whose post tokens the restored asset already contains (same epoch,
    ///   revision not beyond the restored one) make every member operation a known, applied
    ///   operation: a retry gets the recorded result and nothing is applied again; a retry with
    ///   other content is refused.
    /// * `Confirmed` records *beyond* the restored state are changes that were lost since the
    ///   snapshot (the journal carries no write payload yet): they are reported, not
    ///   remembered, so a retry applies them again instead of claiming a success that the world
    ///   does not contain.
    /// * An `Intent` with no later record is **undecided** (it may or may not have reached
    ///   Core): its operations are blocked, retries are refused, until
    ///   [`OrchestrationService::resolve_pending`] decides. It is never treated as aborted just
    ///   because memory is empty.
    /// * `Aborted` records need nothing: the id may be retried.
    pub fn recover_idempotency(&self) -> Result<RecoveryReport, EditingError> {
        let Some(j) = &self.journal else {
            return Err(EditingError::InvalidOperation("no journal configured"));
        };
        let mut entries = Vec::new();
        let mut from = 1u64;
        loop {
            let page = j.read_from(JournalSeq(from), 1024)?;
            let Some(last) = page.last() else { break };
            from = last.seq.0 + 1;
            entries.extend(page);
        }
        let mut order: Vec<OperationId> = Vec::new();
        let mut last: HashMap<OperationId, JournalEntry> = HashMap::new();
        for e in entries {
            if last.insert(e.operation, e.clone()).is_none() {
                order.push(e.operation);
            }
        }
        let now = self.reader.snapshot().asset_token();
        let mut report = RecoveryReport::default();
        for id in order {
            let e = &last[&id];
            match e.phase {
                JournalPhase::Aborted => report.aborted += 1,
                JournalPhase::Intent => {
                    let mut r = lock(&self.retry);
                    r.block(id);
                    for (m, _) in &e.members {
                        r.block(*m);
                    }
                    drop(r);
                    lock(&self.undecided).insert(id, e.members.clone());
                    report.blocked.push(id);
                }
                JournalPhase::Confirmed => {
                    let asset_tokens: Vec<_> = e
                        .post_tokens
                        .iter()
                        .filter(|t| matches!(t.resource(), ResourceId::Asset(_)))
                        .collect();
                    let contained = !asset_tokens.is_empty()
                        && !e.members.is_empty()
                        && asset_tokens
                            .iter()
                            .all(|t| t.epoch() == now.epoch() && t.value() <= now.value());
                    if contained {
                        let success = Success {
                            operation: id,
                            new_tokens: e.post_tokens.clone(),
                            dirty: vec![],
                            produced_ids: vec![],
                            delta: e.delta,
                        };
                        let mut r = lock(&self.retry);
                        for (m, fp) in &e.members {
                            r.record_fingerprint(*m, *fp, EditingResult::Success(success.clone()));
                        }
                        report.recorded += 1;
                    } else {
                        report.lost_since_snapshot.push(id);
                    }
                }
            }
        }
        Ok(report)
    }

    /// Decides an operation that [`OrchestrationService::recover_idempotency`] left undecided:
    /// `applied = true` records it as confirmed (the caller established that the restored world
    /// contains its effect), `false` closes it as aborted so that it can be retried. The
    /// decision is journalled.
    pub fn resolve_pending(
        &self,
        commit_id: OperationId,
        applied: bool,
    ) -> Result<(), EditingError> {
        let members = lock(&self.undecided)
            .get(&commit_id)
            .cloned()
            .ok_or(EditingError::NotFound)?;
        let asset = self.reader.id();
        let token = self.reader.snapshot().asset_token();
        if applied {
            self.journal_append(
                JournalPhase::Confirmed,
                commit_id,
                ActorId(0),
                asset,
                vec![token],
                members.clone(),
            )?;
            let success = Success {
                operation: commit_id,
                new_tokens: vec![token],
                dirty: vec![],
                produced_ids: vec![],
                delta: None,
            };
            let mut r = lock(&self.retry);
            for (m, fp) in &members {
                r.unblock(*m);
                r.record_fingerprint(*m, *fp, EditingResult::Success(success.clone()));
            }
            r.unblock(commit_id);
        } else {
            self.journal_append(
                JournalPhase::Aborted,
                commit_id,
                ActorId(0),
                asset,
                vec![],
                members.clone(),
            )?;
            let mut r = lock(&self.retry);
            for (m, _) in &members {
                r.unblock(*m);
            }
            r.unblock(commit_id);
        }
        lock(&self.undecided).remove(&commit_id);
        Ok(())
    }

    fn replayed(&self, op: &EditingOperation, recorded: EditingResult) -> EditingResult {
        if matches!(recorded, EditingResult::AppliedUnconfirmed(_)) {
            self.reconcile();
            if let Some(r) = lock(&self.retry).result_of(op.id) {
                return r.clone();
            }
        }
        recorded
    }
}

impl EditingService for OrchestrationService {
    fn begin(
        &self,
        context: AuthorizationContext,
        spec: TransactionSpec,
    ) -> Result<TransactionHandle, EditingError> {
        // Authorization first; a denial creates nothing. System origin never edits through the
        // user path: system writes use their own port.
        if context.origin == Origin::System {
            return Err(EditingError::PermissionDenied);
        }
        self.authorizer.authorize_begin(&context, &spec)?;
        if spec.asset != self.reader.id() {
            return Err(EditingError::InvalidOperation("unknown asset"));
        }
        let base = self.reader.snapshot();
        for l in &spec.layers {
            if base.asset().layer(*l).is_none() {
                return Err(EditingError::InvalidOperation("unknown layer"));
            }
        }
        check_expected_tokens(&base, &spec.expected)?;
        let mut t = lock(&self.txs);
        t.next += 1;
        let id = TransactionId(t.next);
        t.map.insert(
            id,
            Arc::new(Mutex::new(Txn {
                id,
                state: TransactionState::Open,
                context,
                spec,
                base,
                staged: BTreeMap::new(),
                ops: Vec::new(),
                replayed: Vec::new(),
            })),
        );
        Ok(TransactionHandle { id })
    }

    fn execute(&self, handle: &TransactionHandle, operation: EditingOperation) -> EditingResult {
        let tx = match self.tx(handle) {
            Ok(t) => t,
            Err(e) => return EditingResult::ValidationFailure(e),
        };
        let mut tx = lock(&tx);
        if tx.state != TransactionState::Open {
            return EditingResult::ValidationFailure(EditingError::InvalidTransactionState {
                transaction: tx.id,
                state: tx.state,
            });
        }
        if let Err(e) = self.authorizer.authorize_operation(&tx.context, &operation) {
            return classify(e);
        }
        if let Err(e) = operation.validate() {
            return classify(e);
        }
        if operation.asset != tx.spec.asset {
            return EditingResult::ValidationFailure(EditingError::InvalidOperation(
                "operation targets another asset",
            ));
        }
        let looked = lock(&self.retry).lookup(&operation);
        match looked {
            Replay::Mismatch => {
                return EditingResult::ValidationFailure(EditingError::InvalidOperation(
                    "operation id reused with different content",
                ));
            }
            Replay::Same(r) => {
                if !tx.ops.is_empty() {
                    return EditingResult::ValidationFailure(EditingError::InvalidOperation(
                        "mixing replayed and new operations",
                    ));
                }
                let r = self.replayed(&operation, r);
                tx.replayed.push(r.clone());
                return r;
            }
            Replay::Expired => {
                return EditingResult::ValidationFailure(EditingError::InvalidOperation(
                    "operation id is older than the retry window and unknown",
                ));
            }
            Replay::Blocked => {
                return EditingResult::ValidationFailure(EditingError::InvalidOperation(
                    "outcome of an earlier attempt is undecided after recovery",
                ));
            }
            Replay::Fresh => {}
        }
        if !tx.replayed.is_empty() {
            return EditingResult::ValidationFailure(EditingError::InvalidOperation(
                "mixing replayed and new operations",
            ));
        }
        if tx.ops.iter().any(|o| o.id == operation.id) {
            return EditingResult::ValidationFailure(EditingError::InvalidOperation(
                "duplicate operation in transaction",
            ));
        }
        // All target layers must share one representation: no implicit conversion.
        let mut kinds = Vec::new();
        for l in &operation.layers {
            match tx.base.asset().layer(*l) {
                Some(layer) => kinds.push(layer.kind()),
                None => {
                    return EditingResult::ValidationFailure(EditingError::InvalidOperation(
                        "unknown layer",
                    ));
                }
            }
        }
        if kinds.windows(2).any(|w| w[0] != w[1]) {
            return if operation.conversion_policy() == ConversionPolicy::Forbid {
                EditingResult::ValidationFailure(EditingError::IncompatibleGeometryTypes)
            } else {
                EditingResult::Unsupported(EditingError::UnsupportedGeometryOperation(
                    operation.kind,
                ))
            };
        }
        let backend = match self.registry.dispatch(kinds[0], operation.kind) {
            Ok(b) => b,
            Err(e) => return classify(e),
        };
        let applied = catch_unwind(AssertUnwindSafe(|| {
            backend.apply(&operation, &Overlay(&tx), None)
        }));
        let sets = match applied {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => return classify(e),
            Err(_) => return EditingResult::ResourceFailure(EditingError::ProcessingFailed),
        };
        if let Err(e) = check_write_sets(&operation, &sets) {
            return classify(e);
        }
        // Stage atomically: nothing above has changed the transaction.
        for set in sets {
            for w in set.writes {
                let key = (set.layer, w.level, w.brick);
                // The expectation is the base revision, whatever the backend wrote into it.
                let expected = match tx.staged.get(&key) {
                    Some(s) => s.expected,
                    None => expected_of(&tx.base, set.layer, w.level, w.brick),
                };
                tx.staged.insert(
                    key,
                    Staged {
                        change: w.change,
                        expected,
                    },
                );
            }
        }
        let id = tx.id;
        lock(&self.txs).by_op.insert(operation.id, id);
        let op_id = operation.id;
        tx.ops.push(operation);
        EditingResult::Success(Success {
            operation: op_id,
            new_tokens: vec![],
            dirty: vec![],
            produced_ids: vec![],
            delta: None,
        })
    }

    fn commit(&self, handle: &TransactionHandle) -> EditingResult {
        let tx = match self.tx(handle) {
            Ok(t) => t,
            Err(e) => return EditingResult::ValidationFailure(e),
        };
        let mut tx = lock(&tx);
        if !tx.state.can_become(TransactionState::Committing) {
            return EditingResult::ValidationFailure(EditingError::InvalidTransactionState {
                transaction: tx.id,
                state: tx.state,
            });
        }
        // A transaction made only of retried operations answers with the recorded result.
        if tx.ops.is_empty() {
            tx.state = TransactionState::Committed;
            let r = match tx.replayed.first() {
                Some(r) => self.replayed_latest(r),
                None => EditingResult::Success(Success {
                    operation: OperationId::new(0, 0),
                    new_tokens: vec![self.reader.snapshot().asset_token()],
                    dirty: vec![],
                    produced_ids: vec![],
                    delta: None,
                }),
            };
            self.finish(&tx);
            return r;
        }
        tx.state = TransactionState::Committing;
        let first = tx.ops[0].id;
        let actor = tx.context.actor;
        let asset = tx.spec.asset;
        let batch = CommitBatch {
            sets: {
                let mut by_layer: BTreeMap<LayerId, Vec<CellWrite>> = BTreeMap::new();
                for ((layer, level, brick), s) in &tx.staged {
                    by_layer.entry(*layer).or_default().push(CellWrite {
                        level: *level,
                        brick: *brick,
                        expected: s.expected,
                        change: s.change.clone(),
                    });
                }
                by_layer
                    .into_iter()
                    .map(|(layer, writes)| cube_core::WriteSet { layer, writes })
                    .collect()
            },
        };

        let members: Vec<(OperationId, u64)> =
            tx.ops.iter().map(|o| (o.id, o.fingerprint())).collect();
        let _section = lock(&self.section);
        if self
            .journal_append(
                JournalPhase::Intent,
                first,
                actor,
                asset,
                vec![],
                members.clone(),
            )
            .is_err()
        {
            return self.fail(
                &mut tx,
                EditingResult::ResourceFailure(EditingError::PersistenceFailure),
            );
        }
        let outcome = match self.path.commit(first, &batch) {
            Ok(o) => o,
            Err(CommitError::Conflict(cs)) => {
                let report = conflict_result(&self.reader.snapshot(), &cs);
                let _ = self.journal_append(
                    JournalPhase::Aborted,
                    first,
                    actor,
                    asset,
                    vec![],
                    members.clone(),
                );
                return self.fail(&mut tx, EditingResult::Conflict(report));
            }
            Err(other) => {
                let _ = self.journal_append(
                    JournalPhase::Aborted,
                    first,
                    actor,
                    asset,
                    vec![],
                    members.clone(),
                );
                return self.fail(&mut tx, core_failure(other));
            }
        };
        tx.state = TransactionState::Committed;
        let success = self.success_of(&tx, first, &outcome);
        let result = match self.journal_append(
            JournalPhase::Confirmed,
            first,
            actor,
            asset,
            vec![outcome.asset_token],
            members.clone(),
        ) {
            Ok(_) => EditingResult::Success(success.clone()),
            Err(_) => {
                lock(&self.pending).push(PendingConfirm {
                    operation: first,
                    actor,
                    asset,
                    post_tokens: vec![outcome.asset_token],
                    members: members.clone(),
                });
                EditingResult::AppliedUnconfirmed(success.clone())
            }
        };
        {
            let mut r = lock(&self.retry);
            for op in &tx.ops {
                r.record(op.clone(), result.clone());
            }
        }
        let summary = Self::summary(&tx, success.new_tokens.clone(), outcome.dirty.clone());
        let delta = self.delta_of(&tx, first, &success, outcome.dirty);
        self.publisher.committed(&summary, &delta);
        self.finish(&tx);
        result
    }

    fn rollback(&self, handle: &TransactionHandle) -> Result<(), EditingError> {
        let tx = self.tx(handle)?;
        let mut tx = lock(&tx);
        if !tx.state.can_become(TransactionState::RolledBack) {
            return Err(EditingError::InvalidTransactionState {
                transaction: tx.id,
                state: tx.state,
            });
        }
        tx.state = TransactionState::RolledBack;
        self.close_without_change(&tx);
        Ok(())
    }

    fn cancel(&self, operation: OperationId) -> Result<(), EditingError> {
        let id = lock(&self.txs)
            .by_op
            .get(&operation)
            .copied()
            .ok_or(EditingError::NotFound)?;
        let tx = self.tx(&TransactionHandle { id })?;
        let mut tx = lock(&tx);
        if !tx.state.can_become(TransactionState::Cancelled) {
            return Err(EditingError::InvalidTransactionState {
                transaction: tx.id,
                state: tx.state,
            });
        }
        tx.state = TransactionState::Cancelled;
        self.close_without_change(&tx);
        Ok(())
    }

    fn status(&self, operation: OperationId) -> Result<OperationStatus, EditingError> {
        let live = lock(&self.txs).by_op.get(&operation).copied();
        if let Some(id) = live
            && let Ok(tx) = self.tx(&TransactionHandle { id })
        {
            let state = lock(&tx).state;
            return Ok(match state {
                TransactionState::Open => OperationStatus::Pending,
                TransactionState::Committing => OperationStatus::Running,
                TransactionState::Committed => OperationStatus::Succeeded,
                TransactionState::Cancelled => OperationStatus::Cancelled,
                _ => OperationStatus::Failed,
            });
        }
        match lock(&self.retry).result_of(operation) {
            Some(EditingResult::Success(_)) => Ok(OperationStatus::Succeeded),
            // Applied, confirmation outstanding: not finished, not a success yet.
            Some(EditingResult::AppliedUnconfirmed(_)) => Ok(OperationStatus::Running),
            _ => Err(EditingError::NotFound),
        }
    }
}

impl OrchestrationService {
    fn replayed_latest(&self, r: &EditingResult) -> EditingResult {
        if let EditingResult::AppliedUnconfirmed(s) = r {
            self.reconcile();
            if let Some(now) = lock(&self.retry).result_of(s.operation) {
                return now.clone();
            }
        }
        r.clone()
    }

    fn fail(&self, tx: &mut Txn, result: EditingResult) -> EditingResult {
        tx.state = TransactionState::Failed;
        self.close_without_change(tx);
        result
    }

    /// Observers are told that nothing changed; the transaction is released.
    fn close_without_change(&self, tx: &Txn) {
        let summary = Self::summary(tx, vec![], vec![]);
        self.publisher.rolled_back(&summary);
        self.finish(tx);
    }

    fn success_of(&self, tx: &Txn, first: OperationId, outcome: &CommitOutcome) -> Success {
        let mut tokens = vec![outcome.asset_token];
        tokens.extend(outcome.layer_tokens.iter().copied());
        tokens.extend(outcome.cell_tokens.iter().copied());
        let _ = tx;
        Success {
            operation: first,
            new_tokens: tokens,
            dirty: outcome.dirty.clone(),
            produced_ids: vec![],
            delta: None,
        }
    }

    /// The delta handed to observers. Brick payload capture (`changed_bricks`) belongs to the
    /// History and Persistence stages; here the delta carries tokens and dirty regions.
    fn delta_of(
        &self,
        tx: &Txn,
        first: OperationId,
        success: &Success,
        dirty: Vec<cube_core::DirtyRegion>,
    ) -> cube_editing_api::GeometryDelta {
        let layer = tx.ops[0].layers[0];
        let mut pre = vec![tx.base.asset_token()];
        pre.extend(tx.base.layer_token(layer));
        let _ = ResourceId::Layer(layer);
        cube_editing_api::GeometryDelta {
            id: DeltaId(self.next_delta.fetch_add(1, Ordering::SeqCst)),
            operation: first,
            asset: tx.spec.asset,
            layer,
            pre_tokens: pre,
            post_tokens: success.new_tokens.clone(),
            changed_bricks: vec![],
            non_brick: vec![],
            dirty,
        }
    }
}
