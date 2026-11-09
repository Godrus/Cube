//! TEST-03a: the generic backend conformance suite, self-checked. A conforming fake and the
//! reference backend pass; every deliberately broken fake is caught by its specific rule.
#![cfg(not(loom))]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use cube_core::{CellKey, CellState, Region, WriteSet};
use cube_editing_api::{
    BackendCapabilities, EditingError, EditingOperation, GeometryBackend, OverlayRead,
    ValidationReport, kinds,
};
use cube_integration_tests::conformance::backend::{Case, Violation, run};
use cube_integration_tests::fakes::*;
use cube_processing::Snapshot;

struct EmptyOverlay;

impl OverlayRead for EmptyOverlay {
    fn cell(&self, _: CellKey) -> Result<CellState, EditingError> {
        Ok(CellState::NotLoaded)
    }
}

fn cases() -> Vec<Case> {
    [
        kinds::SET_MATERIAL,
        kinds::ADD_GEOMETRY,
        kinds::REMOVE_GEOMETRY,
    ]
    .into_iter()
    .enumerate()
    .map(|(i, k)| Case {
        operation: op(i as u64 + 1, k, L_VOXEL, bc(-1, 0, 0), bc(0, 1, 0), 3),
    })
    .collect()
}

fn rules(v: &[Violation]) -> Vec<&'static str> {
    let mut r: Vec<_> = v.iter().map(|x| x.rule).collect();
    r.sort_unstable();
    r.dedup();
    r
}

#[test]
fn the_reference_backend_conforms() {
    let b = SimpleBackend::arc();
    assert_eq!(run(&*b, &cases(), &EmptyOverlay), vec![]);
}

/// A configurable fake that conforms unless told otherwise.
#[derive(Clone, Copy, Default)]
struct Fake {
    outside_region: bool,
    wrong_layer: bool,
    duplicate: bool,
    wrong_level: bool,
    panics: bool,
    nondeterministic: bool,
    wrong_unsupported_error: bool,
    advertised_but_fails: bool,
    advertise_nothing: bool,
    validate_fails: bool,
}

struct FakeBackend {
    f: Fake,
    caps: BackendCapabilities,
    calls: AtomicU64,
}

fn fake(f: Fake) -> Arc<FakeBackend> {
    let mut caps = SimpleBackend::arc().capabilities().clone();
    if f.advertise_nothing {
        caps.kinds.clear();
    }
    Arc::new(FakeBackend {
        f,
        caps,
        calls: AtomicU64::new(0),
    })
}

impl GeometryBackend for FakeBackend {
    fn capabilities(&self) -> &BackendCapabilities {
        &self.caps
    }
    fn apply(
        &self,
        o: &EditingOperation,
        overlay: &dyn OverlayRead,
        snap: Option<&Snapshot>,
    ) -> Result<Vec<WriteSet>, EditingError> {
        if self.f.panics {
            panic!("fake backend bug");
        }
        if !self.caps.supports(o.kind) {
            return Err(if self.f.wrong_unsupported_error {
                EditingError::ProcessingFailed
            } else {
                EditingError::UnsupportedGeometryOperation(o.kind)
            });
        }
        if self.f.advertised_but_fails && o.kind == kinds::REMOVE_GEOMETRY {
            return Err(EditingError::UnsupportedGeometryOperation(o.kind));
        }
        let mut sets = SimpleBackend::arc().apply(o, overlay, snap)?;
        let n = self.calls.fetch_add(1, Ordering::SeqCst);
        if self.f.outside_region {
            sets[0]
                .writes
                .push(cell_write(L_VOXEL, bc(99, 99, 99), 1).writes.remove(0));
        }
        if self.f.wrong_layer {
            sets[0].layer = cube_core::LayerId::new(L_BOUNDED);
        }
        if self.f.duplicate {
            let w = sets[0].writes[0].clone();
            sets[0].writes.push(w);
        }
        if self.f.wrong_level {
            sets[0].writes[0].level = 2;
        }
        if self.f.nondeterministic {
            sets[0].writes.truncate(1 + (n as usize % 2));
        }
        Ok(sets)
    }
    fn validate(&self, _: &dyn OverlayRead, _: &Region) -> Result<ValidationReport, EditingError> {
        if self.f.validate_fails {
            Err(EditingError::ProcessingFailed)
        } else {
            Ok(ValidationReport::default())
        }
    }
}

#[test]
fn a_conforming_fake_passes() {
    assert_eq!(
        run(&*fake(Fake::default()), &cases(), &EmptyOverlay),
        vec![]
    );
}

#[test]
fn every_broken_fake_is_caught_by_its_own_rule() {
    let broken: [(&str, Fake, &str); 9] = [
        (
            "writes outside the region",
            Fake {
                outside_region: true,
                ..Fake::default()
            },
            "WRITE-REGION",
        ),
        (
            "writes to a non-target layer",
            Fake {
                wrong_layer: true,
                ..Fake::default()
            },
            "WRITE-LAYER",
        ),
        (
            "writes a cell twice",
            Fake {
                duplicate: true,
                ..Fake::default()
            },
            "WRITE-DUPLICATE",
        ),
        (
            "writes at another level",
            Fake {
                wrong_level: true,
                ..Fake::default()
            },
            "WRITE-LEVEL",
        ),
        (
            "panics",
            Fake {
                panics: true,
                ..Fake::default()
            },
            "NO-PANIC",
        ),
        (
            "is nondeterministic",
            Fake {
                nondeterministic: true,
                ..Fake::default()
            },
            "DETERMINISM",
        ),
        (
            "wrong error for unknown kinds",
            Fake {
                wrong_unsupported_error: true,
                ..Fake::default()
            },
            "UNSUPPORTED-ERROR",
        ),
        (
            "advertises a kind it cannot run",
            Fake {
                advertised_but_fails: true,
                ..Fake::default()
            },
            "ADVERTISED-KIND-FAILS",
        ),
        (
            "advertises nothing",
            Fake {
                advertise_nothing: true,
                ..Fake::default()
            },
            "CAPS-EMPTY",
        ),
    ];
    for (name, f, rule) in broken {
        let v = run(&*fake(f), &cases(), &EmptyOverlay);
        assert!(
            rules(&v).contains(&rule),
            "{name}: expected {rule}, got {v:?}"
        );
    }
    let v = run(
        &*fake(Fake {
            validate_fails: true,
            ..Fake::default()
        }),
        &cases(),
        &EmptyOverlay,
    );
    assert_eq!(rules(&v), vec!["VALIDATE"]);
}

#[test]
fn missing_cases_for_advertised_kinds_are_reported() {
    let one = vec![cases().remove(0)];
    let v = run(&*SimpleBackend::arc(), &one, &EmptyOverlay);
    assert_eq!(rules(&v), vec!["CASE-MISSING"]);
    assert_eq!(v.len(), 2);
}

#[test]
fn any_registered_backend_can_run_the_suite_through_the_registry() {
    // The suite needs nothing but the port, so a backend taken out of a registry runs it too.
    let mut r = cube_orchestration::BackendRegistry::new();
    r.register(SimpleBackend::arc()).unwrap();
    let b = r
        .dispatch(cube_core::LayerKind::Voxel, kinds::SET_MATERIAL)
        .unwrap();
    assert!(run(&**b, &cases(), &EmptyOverlay).is_empty());
    assert!(
        r.dispatch(cube_core::LayerKind::Sdf, kinds::SET_MATERIAL)
            .is_err()
    );
    assert_eq!(
        r.register(SimpleBackend::arc()),
        Err(cube_orchestration::RegistryError::DuplicateLayerKind(
            cube_core::LayerKind::Voxel
        ))
    );
    let empty = fake(Fake {
        advertise_nothing: true,
        ..Fake::default()
    });
    assert_eq!(
        cube_orchestration::BackendRegistry::new().register(empty),
        Err(cube_orchestration::RegistryError::NoCapabilities)
    );
}
