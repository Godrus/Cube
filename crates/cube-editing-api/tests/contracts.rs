//! Mock-based contract tests of the Editing API (API-01..API-04). The mocks here are the
//! smallest implementations that can show the contracts are implementable and what they must
//! guarantee; real implementations live in Orchestration, backends, History and Persistence.

use std::collections::BTreeSet;
use std::sync::Mutex;

use cube_core::{
    AssetId, BrickCoord, CellChange, CellKey, CellState, Epoch, GeneratorVersion, LayerId,
    LayerKind, OperationId, Region, ResourceId, Revision, RevisionToken, VoxelPos, WorldSeed,
    WriteSet,
};
use cube_editing_api::*;
use cube_processing::Snapshot;

fn region() -> Region {
    Region::new(0, VoxelPos::new(0, 0, 0), VoxelPos::new(16, 16, 16)).unwrap()
}

fn op(n: u64, kind: KindId) -> EditingOperation {
    EditingOperation {
        id: OperationId::new(1, n),
        kind,
        asset: AssetId::new(1),
        layers: vec![LayerId::new(1)],
        parameters: Parameters::default(),
        read_region: None,
        write_region: region(),
        conversion: None,
        mode: EditMode::Destructive,
    }
}

fn ctx(caps: &[u32], origin: Origin) -> AuthorizationContext {
    AuthorizationContext {
        actor: ActorId(9),
        session: SessionId(1),
        origin,
        capabilities: caps.iter().map(|c| Capability(*c)).collect(),
        scope: Scope::default(),
    }
}

fn token(layer: u64, rev: u64) -> RevisionToken {
    RevisionToken::new(
        ResourceId::Layer(LayerId::new(layer)),
        Epoch::INITIAL,
        Revision::new(rev),
    )
}

const EDIT: Capability = Capability(1);

#[derive(Default)]
struct State {
    next: u64,
    txs: Vec<(TransactionId, TransactionState, Vec<OperationId>)>,
    current_rev: u64,
}

/// A mock that implements the facade contract in memory.
struct MockService {
    state: Mutex<State>,
    observer: Option<Box<dyn TransactionObserver>>,
}

impl MockService {
    fn new(observer: Option<Box<dyn TransactionObserver>>) -> Self {
        Self {
            state: Mutex::new(State::default()),
            observer,
        }
    }
    fn transactions(&self) -> usize {
        self.state.lock().unwrap().txs.len()
    }
    fn summary(
        &self,
        id: TransactionId,
        ops: Vec<OperationId>,
        tokens: Vec<RevisionToken>,
    ) -> TransactionSummary {
        TransactionSummary {
            transaction: id,
            operations: ops,
            asset: AssetId::new(1),
            actor: 9,
            new_tokens: tokens,
            dirty: vec![],
        }
    }
}

impl EditingService for MockService {
    fn begin(
        &self,
        context: AuthorizationContext,
        spec: TransactionSpec,
    ) -> Result<TransactionHandle, EditingError> {
        // Authorization first, before anything else, and a denial creates nothing.
        if !context.has(EDIT) {
            return Err(EditingError::PermissionDenied);
        }
        let mut s = self.state.lock().unwrap();
        if let Some(e) = spec.expected.first()
            && e.value().raw() != s.current_rev
        {
            return Err(EditingError::RevisionConflict);
        }
        s.next += 1;
        let id = TransactionId(s.next);
        s.txs.push((id, TransactionState::Open, vec![]));
        Ok(TransactionHandle { id })
    }

    fn execute(&self, h: &TransactionHandle, operation: EditingOperation) -> EditingResult {
        if let Err(e) = operation.validate() {
            return EditingResult::ValidationFailure(e);
        }
        let mut s = self.state.lock().unwrap();
        let Some(tx) = s.txs.iter_mut().find(|t| t.0 == h.id) else {
            return EditingResult::ValidationFailure(EditingError::UnknownTransaction(h.id));
        };
        if tx.1 != TransactionState::Open {
            return EditingResult::ValidationFailure(EditingError::InvalidTransactionState {
                transaction: h.id,
                state: tx.1,
            });
        }
        if operation.kind == kinds::BOOLEAN {
            return EditingResult::Unsupported(EditingError::UnsupportedGeometryOperation(
                operation.kind,
            ));
        }
        if operation.kind == kinds::RESAMPLE
            && operation.conversion_policy() == ConversionPolicy::Forbid
        {
            return EditingResult::ConversionRequired(EditingError::ConversionRequired);
        }
        tx.2.push(operation.id);
        EditingResult::Success(Success {
            operation: operation.id,
            new_tokens: vec![],
            dirty: vec![],
            produced_ids: vec![],
            delta: None,
        })
    }

    fn commit(&self, h: &TransactionHandle) -> EditingResult {
        let (ops, rev) = {
            let mut s = self.state.lock().unwrap();
            let rev = s.current_rev;
            let Some(tx) = s.txs.iter_mut().find(|t| t.0 == h.id) else {
                return EditingResult::ValidationFailure(EditingError::UnknownTransaction(h.id));
            };
            if !tx.1.can_become(TransactionState::Committing) {
                return EditingResult::ValidationFailure(EditingError::InvalidTransactionState {
                    transaction: h.id,
                    state: tx.1,
                });
            }
            tx.1 = TransactionState::Committed;
            let ops = tx.2.clone();
            s.current_rev += 1;
            (ops, rev + 1)
        };
        let tokens = vec![token(1, rev)];
        if let Some(o) = &self.observer {
            let delta = GeometryDelta {
                id: DeltaId(rev),
                operation: ops.first().copied().unwrap_or(OperationId::new(1, 0)),
                asset: AssetId::new(1),
                layer: LayerId::new(1),
                pre_tokens: vec![token(1, rev - 1)],
                post_tokens: tokens.clone(),
                changed_bricks: vec![],
                non_brick: vec![],
                dirty: vec![],
            };
            o.on_committed(&self.summary(h.id, ops.clone(), tokens.clone()), &delta);
        }
        EditingResult::Success(Success {
            operation: ops.first().copied().unwrap_or(OperationId::new(1, 0)),
            new_tokens: tokens,
            dirty: vec![],
            produced_ids: vec![],
            delta: Some(DeltaId(rev)),
        })
    }

    fn rollback(&self, h: &TransactionHandle) -> Result<(), EditingError> {
        let mut s = self.state.lock().unwrap();
        let tx = s
            .txs
            .iter_mut()
            .find(|t| t.0 == h.id)
            .ok_or(EditingError::UnknownTransaction(h.id))?;
        if !tx.1.can_become(TransactionState::RolledBack) {
            return Err(EditingError::InvalidTransactionState {
                transaction: h.id,
                state: tx.1,
            });
        }
        tx.1 = TransactionState::RolledBack;
        let ops = tx.2.clone();
        drop(s);
        if let Some(o) = &self.observer {
            o.on_rolled_back(&self.summary(h.id, ops, vec![]));
        }
        Ok(())
    }

    fn cancel(&self, operation: OperationId) -> Result<(), EditingError> {
        let s = self.state.lock().unwrap();
        if s.txs.iter().any(|t| t.2.contains(&operation)) {
            Ok(())
        } else {
            Err(EditingError::NotFound)
        }
    }

    fn status(&self, operation: OperationId) -> Result<OperationStatus, EditingError> {
        let s = self.state.lock().unwrap();
        s.txs
            .iter()
            .find(|t| t.2.contains(&operation))
            .map(|t| match t.1 {
                TransactionState::Open => OperationStatus::Pending,
                TransactionState::Committing => OperationStatus::Running,
                TransactionState::Committed => OperationStatus::Succeeded,
                TransactionState::Cancelled => OperationStatus::Cancelled,
                _ => OperationStatus::Failed,
            })
            .ok_or(EditingError::NotFound)
    }
}

fn spec() -> TransactionSpec {
    TransactionSpec::new(AssetId::new(1))
}

#[test]
fn a_denied_begin_creates_no_state_and_is_checked_before_revisions() {
    let svc = MockService::new(None);
    let mut s = spec();
    s.expected = vec![token(1, 999)]; // would also be a revision conflict
    let denied = svc.begin(ctx(&[], Origin::LocalClient), s);
    assert_eq!(denied.unwrap_err(), EditingError::PermissionDenied);
    assert_eq!(svc.transactions(), 0);
    assert!(svc.begin(ctx(&[1], Origin::LocalClient), spec()).is_ok());
}

#[test]
fn the_local_client_origin_gets_no_rights_the_server_origin_lacks() {
    let svc = MockService::new(None);
    for origin in [Origin::LocalClient, Origin::Server] {
        assert_eq!(
            svc.begin(ctx(&[], origin), spec()).unwrap_err(),
            EditingError::PermissionDenied
        );
    }
    assert_eq!(svc.transactions(), 0);
}

#[test]
fn transaction_life_cycle_follows_the_state_table() {
    let svc = MockService::new(None);
    let h = svc.begin(ctx(&[1], Origin::Server), spec()).unwrap();
    assert!(svc.execute(&h, op(1, kinds::SET_MATERIAL)).is_success());
    assert_eq!(
        svc.status(OperationId::new(1, 1)),
        Ok(OperationStatus::Pending)
    );
    let committed = svc.commit(&h);
    assert!(committed.is_success());
    assert_eq!(
        svc.status(OperationId::new(1, 1)),
        Ok(OperationStatus::Succeeded)
    );
    // Terminal means final: no second commit, no rollback, no more operations.
    for r in [svc.commit(&h), svc.execute(&h, op(2, kinds::SET_MATERIAL))] {
        assert!(matches!(
            r,
            EditingResult::ValidationFailure(EditingError::InvalidTransactionState {
                state: TransactionState::Committed,
                ..
            })
        ));
    }
    assert!(matches!(
        svc.rollback(&h),
        Err(EditingError::InvalidTransactionState { .. })
    ));
    assert_eq!(
        svc.rollback(&TransactionHandle {
            id: TransactionId(99)
        }),
        Err(EditingError::UnknownTransaction(TransactionId(99)))
    );
}

#[test]
fn every_result_variant_can_be_produced_and_failures_never_report_success() {
    let svc = MockService::new(None);
    let h = svc.begin(ctx(&[1], Origin::Server), spec()).unwrap();
    let mut bad = op(1, kinds::SET_MATERIAL);
    bad.layers.clear();
    let rs = [
        svc.execute(&h, op(2, kinds::SET_MATERIAL)),
        svc.execute(&h, bad),
        svc.execute(&h, op(3, kinds::BOOLEAN)),
        svc.execute(&h, op(4, kinds::RESAMPLE)),
        EditingResult::Conflict(ConflictResult {
            scope: ConflictScope::Layer,
            conflicting: vec![ResourceId::Layer(LayerId::new(1))],
            expected: vec![token(1, 1)],
            current: vec![token(1, 2)],
            holder: None,
            suggestion: Suggestion::Retry,
        }),
        EditingResult::ResourceFailure(EditingError::ProcessingFailed),
        EditingResult::Cancelled,
    ];
    assert!(matches!(rs[0], EditingResult::Success(_)));
    assert!(matches!(rs[1], EditingResult::ValidationFailure(_)));
    assert!(matches!(rs[2], EditingResult::Unsupported(_)));
    assert!(matches!(rs[3], EditingResult::ConversionRequired(_)));
    assert!(rs[1..].iter().all(EditingResult::left_state_unchanged));
}

#[test]
fn a_revision_conflict_on_begin_is_a_controlled_error() {
    let svc = MockService::new(None);
    let mut s = spec();
    s.expected = vec![token(1, 5)];
    assert_eq!(
        svc.begin(ctx(&[1], Origin::Server), s).unwrap_err(),
        EditingError::RevisionConflict
    );
}

#[test]
fn a_conflict_report_names_exactly_the_conflicting_resources_and_pairs_tokens() {
    let a = CellKey::new(LayerId::new(1), 0, BrickCoord::new(0, 0, 0)).unwrap();
    let b = CellKey::new(LayerId::new(1), 0, BrickCoord::new(-1, 0, 0)).unwrap();
    let c = ConflictResult {
        scope: ConflictScope::Brick,
        conflicting: vec![ResourceId::Cell(b), ResourceId::Cell(a)],
        expected: vec![
            RevisionToken::new(ResourceId::Cell(b), Epoch::INITIAL, Revision::new(1)),
            RevisionToken::new(ResourceId::Cell(a), Epoch::INITIAL, Revision::new(1)),
        ],
        current: vec![
            RevisionToken::new(ResourceId::Cell(b), Epoch::INITIAL, Revision::new(2)),
            RevisionToken::new(ResourceId::Cell(a), Epoch::INITIAL, Revision::new(3)),
        ],
        holder: None,
        suggestion: Suggestion::Rebase,
    };
    assert!(c.tokens_pair_up());
    assert_eq!(c.current.len(), c.conflicting.len());
    for (res, t) in c.conflicting.iter().zip(&c.current) {
        assert_eq!(*res, t.resource());
    }
    let mut skewed = c.clone();
    skewed.current.pop();
    assert!(!skewed.tokens_pair_up());
}

#[derive(Default)]
struct Recorder {
    log: Mutex<Vec<String>>,
}

struct Rec(std::sync::Arc<Recorder>);

impl TransactionObserver for Rec {
    fn on_committed(&self, s: &TransactionSummary, d: &GeometryDelta) {
        self.0
            .log
            .lock()
            .unwrap()
            .push(format!("commit {} {}", s.transaction.0, d.id.0));
    }
    fn on_rolled_back(&self, s: &TransactionSummary) {
        self.0
            .log
            .lock()
            .unwrap()
            .push(format!("rollback {}", s.transaction.0));
    }
}

#[test]
fn observers_are_told_exactly_once_and_in_order() {
    let rec = std::sync::Arc::new(Recorder::default());
    let svc = MockService::new(Some(Box::new(Rec(std::sync::Arc::clone(&rec)))));
    let h1 = svc.begin(ctx(&[1], Origin::Server), spec()).unwrap();
    let h2 = svc.begin(ctx(&[1], Origin::Server), spec()).unwrap();
    svc.execute(&h1, op(1, kinds::SET_MATERIAL));
    assert!(svc.commit(&h1).is_success());
    svc.rollback(&h2).unwrap();
    let _ = svc.commit(&h1); // illegal: must not notify again
    assert_eq!(*rec.log.lock().unwrap(), vec!["commit 1 1", "rollback 2"]);
}

struct Mock {
    caps: BackendCapabilities,
}

struct NoOverlay;

impl OverlayRead for NoOverlay {
    fn cell(&self, _: CellKey) -> Result<CellState, EditingError> {
        Ok(CellState::NotLoaded)
    }
}

impl GeometryBackend for Mock {
    fn capabilities(&self) -> &BackendCapabilities {
        &self.caps
    }
    fn apply(
        &self,
        o: &EditingOperation,
        _: &dyn OverlayRead,
        _: Option<&Snapshot>,
    ) -> Result<Vec<WriteSet>, EditingError> {
        if !self.caps.supports(o.kind) {
            return Err(EditingError::UnsupportedGeometryOperation(o.kind));
        }
        Ok(vec![])
    }
    fn validate(&self, _: &dyn OverlayRead, _: &Region) -> Result<ValidationReport, EditingError> {
        Ok(ValidationReport::default())
    }
}

#[test]
fn a_backend_compiles_against_the_api_alone_and_rejects_unsupported_kinds() {
    let b: Box<dyn GeometryBackend> = Box::new(Mock {
        caps: BackendCapabilities {
            layer_kind: LayerKind::Voxel,
            kinds: [kinds::SET_MATERIAL, kinds::ADD_GEOMETRY]
                .into_iter()
                .collect(),
            native_boolean: false,
            resample: false,
            validate: true,
        },
    });
    assert!(
        b.apply(&op(1, kinds::SET_MATERIAL), &NoOverlay, None)
            .is_ok()
    );
    assert_eq!(
        b.apply(&op(2, kinds::SCULPT), &NoOverlay, None),
        Err(EditingError::UnsupportedGeometryOperation(kinds::SCULPT))
    );
    assert!(b.validate(&NoOverlay, &region()).unwrap().is_valid());
    assert!(!b.capabilities().native_boolean);
}

/// In-memory storage mocks: History and Persistence need no mutual dependency, and recovery
/// needs no WorldGen: it only sees `BaseProvider`.
#[derive(Default)]
struct MemStore {
    payloads: Mutex<Vec<Vec<u8>>>,
    deltas: Mutex<Vec<GeometryDelta>>,
    journal: Mutex<Vec<JournalEntry>>,
}

impl DeltaStore for MemStore {
    fn put_payload(&self, bytes: &[u8]) -> Result<PayloadRef, EditingError> {
        let mut p = self.payloads.lock().unwrap();
        p.push(bytes.to_vec());
        Ok(PayloadRef(p.len() as u64))
    }
    fn payload(&self, r: PayloadRef) -> Result<Vec<u8>, EditingError> {
        self.payloads
            .lock()
            .unwrap()
            .get((r.0 as usize).wrapping_sub(1))
            .cloned()
            .ok_or(EditingError::NotFound)
    }
    fn put_delta(&self, d: &GeometryDelta) -> Result<(), EditingError> {
        self.deltas.lock().unwrap().push(d.clone());
        Ok(())
    }
    fn delta(&self, id: DeltaId) -> Result<GeometryDelta, EditingError> {
        self.deltas
            .lock()
            .unwrap()
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .ok_or(EditingError::NotFound)
    }
}

impl OperationJournal for MemStore {
    fn append(&self, mut e: JournalEntry) -> Result<JournalSeq, EditingError> {
        let mut j = self.journal.lock().unwrap();
        e.seq = JournalSeq(j.len() as u64 + 1);
        let s = e.seq;
        j.push(e);
        Ok(s)
    }
    fn read_from(&self, from: JournalSeq, limit: usize) -> Result<Vec<JournalEntry>, EditingError> {
        Ok(self
            .journal
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.seq >= from)
            .take(limit)
            .cloned()
            .collect())
    }
}

fn delta(n: u64, p: PayloadRef) -> GeometryDelta {
    GeometryDelta {
        id: DeltaId(n),
        operation: OperationId::new(1, n),
        asset: AssetId::new(1),
        layer: LayerId::new(1),
        pre_tokens: vec![token(1, n - 1)],
        post_tokens: vec![token(1, n)],
        changed_bricks: vec![ChangedBrick {
            key: CellKey::new(LayerId::new(1), 0, BrickCoord::new(-1, 0, 0)).unwrap(),
            change: BrickChange::Patch(p),
        }],
        non_brick: vec![],
        dirty: vec![],
    }
}

#[test]
fn delta_store_and_journal_round_trip_and_the_journal_is_gapless() {
    let store = MemStore::default();
    let p = store.put_payload(b"patch").unwrap();
    let d = delta(1, p);
    store.put_delta(&d).unwrap();
    assert_eq!(store.delta(DeltaId(1)).unwrap(), d);
    assert_eq!(store.payload(p).unwrap(), b"patch");
    assert_eq!(store.delta(DeltaId(2)), Err(EditingError::NotFound));
    assert_eq!(store.payload(PayloadRef(0)), Err(EditingError::NotFound));

    for n in 1..=5 {
        let seq = store
            .append(JournalEntry {
                seq: JournalSeq(0),
                phase: JournalPhase::Confirmed,
                operation: OperationId::new(1, n),
                actor: ActorId(9),
                asset: AssetId::new(1),
                post_tokens: vec![token(1, n)],
                delta: Some(DeltaId(n)),
            })
            .unwrap();
        assert_eq!(seq, JournalSeq(n));
    }
    let tail = store.read_from(JournalSeq(3), 10).unwrap();
    assert_eq!(
        tail.iter().map(|e| e.seq.0).collect::<Vec<_>>(),
        vec![3, 4, 5]
    );
    assert_eq!(store.read_from(JournalSeq(1), 2).unwrap().len(), 2);
}

struct FlatBase;

impl BaseProvider for FlatBase {
    fn base_brick(
        &self,
        seed: WorldSeed,
        version: GeneratorVersion,
        _: AssetId,
        _: LayerId,
        _: i8,
        brick: BrickCoord,
    ) -> Result<CellChange, EditingError> {
        // Deterministic in all arguments.
        if (seed.raw() ^ u64::from(version.raw()) ^ brick.x as u64).is_multiple_of(2) {
            Ok(CellChange::KnownEmpty)
        } else {
            Err(EditingError::NotFound)
        }
    }
}

#[test]
fn recovery_sees_the_base_only_through_the_port_and_it_is_deterministic() {
    let p: &dyn BaseProvider = &FlatBase;
    let call = || {
        p.base_brick(
            WorldSeed::new(2),
            GeneratorVersion::new(2),
            AssetId::new(1),
            LayerId::new(1),
            0,
            BrickCoord::new(-4, 0, 0),
        )
    };
    assert_eq!(call(), call());
    assert!(call().is_ok());
}

#[test]
fn checkpoint_data_describes_a_state_without_holding_it() {
    let c = Checkpoint {
        id: CheckpointId(1),
        asset_token: RevisionToken::new(
            ResourceId::Asset(AssetId::new(1)),
            Epoch::INITIAL,
            Revision::new(7),
        ),
        layer_tokens: vec![token(1, 7)],
        snapshot: SnapshotRef::Durable(PayloadRef(3)),
        last_delta: Some(DeltaId(7)),
        epoch: Epoch::INITIAL,
        reason: CheckpointReason::Compaction,
    };
    assert_eq!(c.clone(), c);
}

#[test]
fn error_codes_are_pinned_unique_and_in_the_editing_range() {
    let k = KindId(1);
    let t = TransactionId(1);
    let all = [
        (EditingError::RevisionConflict, 4001),
        (EditingError::TransactionConflict, 4002),
        (EditingError::PermissionDenied, 4003),
        (EditingError::ProcessingCancelled, 4004),
        (EditingError::ProcessingFailed, 4005),
        (EditingError::UnsupportedGeometryOperation(k), 4006),
        (EditingError::IncompatibleGeometryTypes, 4007),
        (EditingError::ConversionRequired, 4008),
        (EditingError::InvalidResolution, 4009),
        (EditingError::UnknownKind(k), 4010),
        (EditingError::InvalidOperation("x"), 4011),
        (EditingError::UnknownTransaction(t), 4012),
        (
            EditingError::InvalidTransactionState {
                transaction: t,
                state: TransactionState::Open,
            },
            4013,
        ),
        (EditingError::PersistenceFailure, 4014),
        (EditingError::NotFound, 4015),
    ];
    let mut seen = BTreeSet::new();
    for (e, code) in &all {
        assert_eq!(e.code().0, *code, "{e}");
        assert!(seen.insert(*code) && (4000..5000).contains(code));
        assert!(!e.to_string().is_empty());
    }
    assert_eq!(
        EditingError::PersistenceFailure.severity(),
        cube_core::Severity::Critical
    );
}
