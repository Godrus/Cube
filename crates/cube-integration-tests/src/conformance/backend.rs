//! Generic backend conformance suite (TEST-03a, E-07, ACR-002).
//!
//! A backend owner supplies cases (an operation per advertised kind that the backend can run on
//! the given overlay); the suite checks everything the contract requires of a backend and
//! returns the violations. An empty result means the backend conforms.

use std::collections::BTreeSet;
use std::panic::{AssertUnwindSafe, catch_unwind};

use cube_core::Region;
use cube_editing_api::{EditingError, EditingOperation, GeometryBackend, KindId, OverlayRead};

/// One conformance rule that a backend broke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Stable rule id.
    pub rule: &'static str,
    /// What happened.
    pub detail: String,
}

fn v(rule: &'static str, detail: impl Into<String>) -> Violation {
    Violation {
        rule,
        detail: detail.into(),
    }
}

/// An operation the backend must be able to apply on the overlay of the suite.
pub struct Case {
    /// The operation; its kind must be one the backend advertises.
    pub operation: EditingOperation,
}

/// A kind id that no backend advertises.
pub const UNSUPPORTED_KIND: KindId = KindId(u32::MAX);

/// Runs the suite against `backend` and returns the violations found.
pub fn run(
    backend: &dyn GeometryBackend,
    cases: &[Case],
    overlay: &dyn OverlayRead,
) -> Vec<Violation> {
    let mut out = Vec::new();
    let caps = backend.capabilities();

    // C1: a backend must advertise something and cases must cover every advertised kind.
    if caps.kinds.is_empty() {
        out.push(v("CAPS-EMPTY", "no operation kind advertised"));
    }
    let covered: BTreeSet<KindId> = cases.iter().map(|c| c.operation.kind).collect();
    for k in &caps.kinds {
        if !covered.contains(k) {
            out.push(v(
                "CASE-MISSING",
                format!("no case for advertised kind {}", k.0),
            ));
        }
    }

    for case in cases {
        let o = &case.operation;
        let first = catch_unwind(AssertUnwindSafe(|| backend.apply(o, overlay, None)));
        let Ok(first) = first else {
            out.push(v(
                "NO-PANIC",
                format!("apply panicked for kind {}", o.kind.0),
            ));
            continue;
        };
        if !caps.supports(o.kind) {
            // C2: a kind that is not advertised must be refused with the controlled error.
            if first != Err(EditingError::UnsupportedGeometryOperation(o.kind)) {
                out.push(v(
                    "UNSUPPORTED-ERROR",
                    format!("kind {} not advertised but {first:?}", o.kind.0),
                ));
            }
            continue;
        }
        let sets = match first {
            Ok(s) => s,
            Err(e) => {
                out.push(v(
                    "ADVERTISED-KIND-FAILS",
                    format!("kind {} advertised but apply returned {e}", o.kind.0),
                ));
                continue;
            }
        };
        // C3: writes stay inside targets, level and region, no cell twice.
        let mut seen = BTreeSet::new();
        let Some((lo, hi)) = o.write_region.brick_span() else {
            continue;
        };
        for set in &sets {
            if !o.layers.contains(&set.layer) {
                out.push(v(
                    "WRITE-LAYER",
                    format!("write set for non-target layer {}", set.layer.raw()),
                ));
            }
            for w in &set.writes {
                let b = w.brick;
                let inside = (lo.x..=hi.x).contains(&b.x)
                    && (lo.y..=hi.y).contains(&b.y)
                    && (lo.z..=hi.z).contains(&b.z);
                if w.level != o.write_region.level() {
                    out.push(v("WRITE-LEVEL", format!("write at level {}", w.level)));
                }
                if !inside {
                    out.push(v(
                        "WRITE-REGION",
                        format!("write outside the write region at {b:?}"),
                    ));
                }
                if !seen.insert((set.layer, w.level, b)) {
                    out.push(v("WRITE-DUPLICATE", format!("cell {b:?} written twice")));
                }
            }
        }
        // C4: determinism.
        match catch_unwind(AssertUnwindSafe(|| backend.apply(o, overlay, None))) {
            Ok(Ok(again)) if again == sets => {}
            _ => out.push(v(
                "DETERMINISM",
                format!("kind {} gave different results on a second run", o.kind.0),
            )),
        }
    }

    // C2 again on a kind nobody advertises, with a well-formed operation borrowed from a case.
    if let Some(case) = cases.first() {
        let mut o = case.operation.clone();
        o.kind = UNSUPPORTED_KIND;
        let r = catch_unwind(AssertUnwindSafe(|| backend.apply(&o, overlay, None)));
        match r {
            Ok(Err(EditingError::UnsupportedGeometryOperation(k))) if k == UNSUPPORTED_KIND => {}
            Ok(other) => out.push(v(
                "UNSUPPORTED-ERROR",
                format!("unknown kind gave {other:?}"),
            )),
            Err(_) => out.push(v("NO-PANIC", "apply panicked for an unknown kind")),
        }
    }

    // C5: validation must not panic and must be consistent with the capability flag.
    if let Some(case) = cases.first() {
        let region: Region = case.operation.write_region;
        match catch_unwind(AssertUnwindSafe(|| backend.validate(overlay, &region))) {
            Err(_) => out.push(v("NO-PANIC", "validate panicked")),
            Ok(Err(e)) if caps.validate => {
                out.push(v(
                    "VALIDATE",
                    format!("validate advertised but failed: {e}"),
                ));
            }
            Ok(_) => {}
        }
    }
    out
}
