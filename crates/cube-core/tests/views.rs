//! Export/import views and the content hash (CORE-11), through the public API.

use cube_core::{
    AssetHost, AssetId, AssetView, BrickBounds, BrickCoord, CellChange, CellPayload, CellStore,
    CellView, CellWrite, Channel, ChannelId, ChannelSet, CommitBatch, CoreError, ExpectedRevision,
    GeometryAsset, GeometryTransform, Layer, LayerData, LayerId, LayerKind, LocalPos,
    MaterialIndex, OperationId, Revision, Rotation, WriteSet,
};
use cube_testkit::{DetRng, for_each_seed};

fn host() -> AssetHost {
    let bounds = BrickBounds::new(BrickCoord::new(-4, -4, -4), BrickCoord::new(4, 4, 4)).unwrap();
    let layers = vec![
        Layer::new(
            LayerId::new(1),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Voxel),
        ),
        Layer::new(
            LayerId::new(2),
            GeometryTransform::new([3, -4, 5], Rotation::from_index(7).unwrap(), 1).unwrap(),
            LayerData::Voxel(CellStore::with_bounds(bounds)),
        ),
        Layer::new(
            LayerId::new(3),
            GeometryTransform::IDENTITY,
            LayerData::empty(LayerKind::Mesh),
        ),
    ];
    AssetHost::new(GeometryAsset::new(AssetId::new(5), layers).unwrap())
}
fn mat(m: u16) -> CellChange {
    CellChange::Put(CellPayload::Channels(ChannelSet::uniform_material(
        MaterialIndex::from_raw(m),
    )))
}
fn put(
    layer: u64,
    level: i8,
    x: i64,
    expected: ExpectedRevision,
    change: CellChange,
) -> CommitBatch {
    CommitBatch {
        sets: vec![WriteSet {
            layer: LayerId::new(layer),
            writes: vec![CellWrite {
                level,
                brick: BrickCoord::new(x, 0, 0),
                expected,
                change,
            }],
        }],
    }
}
fn op(n: u64) -> OperationId {
    OperationId::new(1, n)
}

fn populated() -> (cube_core::SharedAsset, cube_core::UserCommit) {
    let h = host();
    let r = h.reader();
    let user = h.into_handles().user;
    user.commit(op(1), &put(1, 0, 0, ExpectedRevision::Absent, mat(3)))
        .unwrap();
    user.commit(
        op(2),
        &put(1, 0, 1, ExpectedRevision::Absent, CellChange::KnownEmpty),
    )
    .unwrap();
    user.commit(op(3), &put(2, 2, -1, ExpectedRevision::Absent, mat(9)))
        .unwrap();
    user.commit(
        op(4),
        &put(1, 0, 0, ExpectedRevision::At(Revision::new(1)), mat(4)),
    )
    .unwrap();
    (r, user)
}

#[test]
fn export_import_round_trip_keeps_content_revisions_and_hash() {
    let (r, _user) = populated();
    let view = r.snapshot().export_view();
    assert_eq!(view.revision, Revision::new(4));
    assert_eq!(view.layers[0].cells.len(), 2);
    let restored = AssetHost::from_view(&view).unwrap();
    let view2 = restored.reader().snapshot().export_view();
    assert_eq!(view, view2);
    assert_eq!(view.content_hash(), view2.content_hash());
    // The restored asset continues the revision sequence.
    let user = restored.into_handles().user;
    let out = user
        .commit(
            op(10),
            &put(1, 0, 0, ExpectedRevision::At(Revision::new(2)), mat(5)),
        )
        .unwrap();
    assert_eq!(
        (out.asset_token.value(), out.cell_tokens[0].value()),
        (Revision::new(5), Revision::new(3))
    );
}

#[test]
fn hash_ignores_revision_history_and_channel_encoding() {
    // Same final content reached by different histories.
    let direct = host();
    let dr = direct.reader();
    let du = direct.into_handles().user;
    du.commit(op(1), &put(1, 0, 0, ExpectedRevision::Absent, mat(4)))
        .unwrap();
    let rounded = host();
    let rr = rounded.reader();
    let ru = rounded.into_handles().user;
    ru.commit(op(1), &put(1, 0, 0, ExpectedRevision::Absent, mat(1)))
        .unwrap();
    ru.commit(
        op(2),
        &put(1, 0, 0, ExpectedRevision::At(Revision::new(1)), mat(2)),
    )
    .unwrap();
    ru.commit(
        op(3),
        &put(1, 0, 0, ExpectedRevision::At(Revision::new(2)), mat(4)),
    )
    .unwrap();
    assert_ne!(
        dr.snapshot().asset_token().value(),
        rr.snapshot().asset_token().value()
    );
    assert_eq!(
        dr.snapshot().export_view().content_hash(),
        rr.snapshot().export_view().content_hash()
    );
    // Dense and constant encodings of equal values hash equally.
    let mut dense = Channel::constant(7);
    dense.set(LocalPos::new(1, 1, 1).unwrap(), 8);
    dense.set(LocalPos::new(1, 1, 1).unwrap(), 7);
    let hash_of = |ch: Channel| {
        let h = host();
        let r = h.reader();
        let u = h.into_handles().user;
        let set = ChannelSet::new().with(ChannelId::Material, ch);
        u.commit(
            op(1),
            &put(
                1,
                0,
                0,
                ExpectedRevision::Absent,
                CellChange::Put(CellPayload::Channels(set)),
            ),
        )
        .unwrap();
        r.snapshot().export_view().content_hash()
    };
    assert!(dense.is_dense());
    assert_eq!(hash_of(dense), hash_of(Channel::constant(7)));
}

#[test]
fn hash_reacts_to_every_kind_of_content_change() {
    let (r, _u) = populated();
    let base = r.snapshot().export_view();
    let h0 = base.content_hash();
    let mut changed: Vec<AssetView> = Vec::new();
    let mut v = base.clone();
    v.layers[0].cells[0].payload = Some(CellPayload::Channels(ChannelSet::uniform_material(
        MaterialIndex::from_raw(99),
    )));
    changed.push(v);
    let mut v = base.clone();
    v.layers[0].cells.remove(1);
    changed.push(v);
    let mut v = base.clone();
    v.layers[0].cells[0].level = 1;
    changed.push(v);
    let mut v = base.clone();
    v.layers[1].transform = GeometryTransform::IDENTITY;
    changed.push(v);
    let mut v = base.clone();
    v.layers[1].bounds = None;
    changed.push(v);
    let mut v = base.clone();
    v.layers.swap(0, 2);
    changed.push(v);
    let mut v = base.clone();
    v.id = AssetId::new(6);
    changed.push(v);
    for (i, c) in changed.iter().enumerate() {
        assert_ne!(c.content_hash(), h0, "change {i}");
    }
    // Revisions and epochs are not content.
    let mut v = base.clone();
    v.revision = Revision::new(999);
    v.layers[0].revision = Revision::new(77);
    v.layers[0].cells[0].revision = Revision::new(55);
    assert_eq!(v.content_hash(), h0);
}

#[test]
fn golden_hash_pins_algorithm_ch1() {
    // If this fails the canonical byte stream or the algorithm changed: that is a new algorithm id.
    let view = AssetView {
        id: AssetId::new(1),
        epoch: cube_core::Epoch::INITIAL,
        revision: Revision::INITIAL,
        layers: vec![cube_core::LayerView {
            id: LayerId::new(1),
            kind: LayerKind::Voxel,
            transform: GeometryTransform::IDENTITY,
            bounds: None,
            revision: Revision::INITIAL,
            cells: vec![CellView {
                level: 0,
                brick: BrickCoord::new(-1, 0, 2),
                revision: Revision::new(1),
                payload: Some(CellPayload::Channels(ChannelSet::uniform_material(
                    MaterialIndex::from_raw(3),
                ))),
            }],
        }],
    };
    cube_testkit::golden::Golden::new("content-hash CH1 of the reference asset", GOLDEN_CH1)
        .check(view.content_hash().value());
    assert_eq!(cube_core::ContentHash::ALGORITHM, "CH1");
}

const GOLDEN_CH1: u128 = 227_353_646_010_888_218_910_847_508_926_292_005_005;

#[test]
fn invalid_views_are_rejected() {
    let (r, _u) = populated();
    let base = r.snapshot().export_view();
    let mut dup = base.clone();
    let first_cell = dup.layers[0].cells[0].clone();
    dup.layers[0].cells.push(first_cell);
    assert_eq!(
        AssetHost::from_view(&dup).err(),
        Some(CoreError::DuplicateKey)
    );
    let mut outside = base.clone();
    outside.layers[1].cells.push(CellView {
        level: 0,
        brick: BrickCoord::new(9, 0, 0),
        revision: Revision::new(1),
        payload: None,
    });
    assert_eq!(
        AssetHost::from_view(&outside).err(),
        Some(CoreError::InvalidView)
    );
    let mut on_mesh = base.clone();
    on_mesh.layers[2].cells.push(CellView {
        level: 0,
        brick: BrickCoord::new(0, 0, 0),
        revision: Revision::new(1),
        payload: None,
    });
    assert_eq!(
        AssetHost::from_view(&on_mesh).err(),
        Some(CoreError::InvalidView)
    );
    let mut neg = base.clone();
    neg.layers[0].cells[0].level = -1;
    assert_eq!(
        AssetHost::from_view(&neg).err(),
        Some(CoreError::InvalidLevel(-1))
    );
    let mut page = base.clone();
    page.layers[0].cells[0].payload = Some(CellPayload::MeshPage(Default::default()));
    assert_eq!(
        AssetHost::from_view(&page).err(),
        Some(CoreError::InvalidView)
    );
    let mut dup_layer = base;
    let first = dup_layer.layers[0].clone();
    dup_layer.layers.push(first);
    assert!(matches!(
        AssetHost::from_view(&dup_layer).err(),
        Some(CoreError::DuplicateLayer(_))
    ));
}

#[test]
fn random_assets_round_trip_with_equal_hashes() {
    for_each_seed(51, 40, |seed| {
        let mut rng = DetRng::new(seed);
        let h = host();
        let r = h.reader();
        let u = h.into_handles().user;
        for n in 0..rng.below(25) {
            let layer = 1 + rng.below(2);
            let x = rng.range_i64(-3, 3);
            let cur = r
                .snapshot()
                .cell_token(LayerId::new(layer), 0, BrickCoord::new(x, 0, 0));
            let expected = cur.map_or(ExpectedRevision::Absent, |t| {
                ExpectedRevision::At(t.value())
            });
            let change = if rng.below(4) == 0 {
                CellChange::KnownEmpty
            } else {
                mat(rng.below(5) as u16)
            };
            u.commit(op(n), &put(layer, 0, x, expected, change))
                .unwrap();
        }
        let view = r.snapshot().export_view();
        let again = AssetHost::from_view(&view)
            .unwrap()
            .reader()
            .snapshot()
            .export_view();
        assert_eq!(view, again);
        assert_eq!(view.content_hash(), again.content_hash());
    });
}
