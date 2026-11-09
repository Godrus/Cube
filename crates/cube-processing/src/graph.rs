//! Provider registry and dependency graph (PROC-02, PR-02/PR-07).
//!
//! Registration is explicit. [`ProviderRegistry::freeze`] validates the whole set at once and
//! reports *all* problems (duplicate kinds, missing dependencies, cycles, oversized aprons), so a
//! misconfiguration is diagnosed in one run. A frozen [`DependencyGraph`] is immutable, acyclic,
//! and has one deterministic order: topological, ties broken by the smaller `DerivedKind`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use cube_core::{BrickCoord, CellKey, DirtyRegion};

use crate::provider::{DerivedKind, DerivedTarget, MAX_APRON, Provider};

/// A problem found while building the graph.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegistryError {
    /// Two providers claim the same kind.
    DuplicateKind {
        /// The contested kind.
        kind: DerivedKind,
        /// Name of the provider registered first.
        first: String,
        /// Name of the provider rejected.
        second: String,
    },
    /// A provider depends on a kind that nobody provides.
    MissingDependency {
        /// The dependent kind.
        kind: DerivedKind,
        /// The kind that is not registered.
        missing: DerivedKind,
    },
    /// A provider lists its own kind or the same dependency twice.
    InvalidDependencyList {
        /// The offending provider kind.
        kind: DerivedKind,
    },
    /// The dependencies form a cycle; `path` lists it with the first kind repeated at the end.
    Cycle {
        /// The cycle.
        path: Vec<DerivedKind>,
    },
    /// A provider asks for an apron larger than [`MAX_APRON`].
    ApronTooLarge {
        /// The offending provider kind.
        kind: DerivedKind,
        /// The requested apron.
        apron: u32,
    },
    /// A provider reacts to no cell change and has no derived inputs: it could never be invalidated.
    NeverInvalidated {
        /// The offending provider kind.
        kind: DerivedKind,
    },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateKind {
                kind,
                first,
                second,
            } => write!(
                f,
                "kind {} is provided by both `{first}` and `{second}`",
                kind.0
            ),
            Self::MissingDependency { kind, missing } => {
                write!(f, "kind {} needs unregistered kind {}", kind.0, missing.0)
            }
            Self::InvalidDependencyList { kind } => {
                write!(f, "kind {} lists itself or a dependency twice", kind.0)
            }
            Self::Cycle { path } => {
                let p: Vec<String> = path.iter().map(|k| k.0.to_string()).collect();
                write!(f, "dependency cycle: {}", p.join(" -> "))
            }
            Self::ApronTooLarge { kind, apron } => {
                write!(
                    f,
                    "kind {} asks for apron {apron} (max {MAX_APRON})",
                    kind.0
                )
            }
            Self::NeverInvalidated { kind } => {
                write!(f, "kind {} reacts to no change and has no inputs", kind.0)
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// Collects providers; nothing is active until [`ProviderRegistry::freeze`] succeeds.
#[derive(Default)]
pub struct ProviderRegistry {
    providers: BTreeMap<DerivedKind, Arc<dyn Provider>>,
}

impl std::fmt::Debug for ProviderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.providers.keys()).finish()
    }
}

impl ProviderRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a provider. A second provider for the same kind is rejected and the first stays.
    pub fn register(&mut self, provider: Arc<dyn Provider>) -> Result<(), RegistryError> {
        let kind = provider.kind();
        if let Some(first) = self.providers.get(&kind) {
            return Err(RegistryError::DuplicateKind {
                kind,
                first: first.name().to_owned(),
                second: provider.name().to_owned(),
            });
        }
        self.providers.insert(kind, provider);
        Ok(())
    }

    /// Number of registered providers.
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// `true` if nothing is registered.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// Validates the set and builds the immutable graph, or returns every problem found.
    pub fn freeze(self) -> Result<DependencyGraph, Vec<RegistryError>> {
        let mut errors = Vec::new();
        for (kind, p) in &self.providers {
            let deps = p.depends_on();
            let unique: BTreeSet<_> = deps.iter().collect();
            if unique.len() != deps.len() || deps.contains(kind) {
                errors.push(RegistryError::InvalidDependencyList { kind: *kind });
            }
            for d in deps {
                if d != kind && !self.providers.contains_key(d) {
                    errors.push(RegistryError::MissingDependency {
                        kind: *kind,
                        missing: *d,
                    });
                }
            }
            if p.apron() > MAX_APRON {
                errors.push(RegistryError::ApronTooLarge {
                    kind: *kind,
                    apron: p.apron(),
                });
            }
            if p.consumes().is_none() && deps.is_empty() {
                errors.push(RegistryError::NeverInvalidated { kind: *kind });
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        let order = match topological_order(&self.providers) {
            Ok(o) => o,
            Err(path) => return Err(vec![RegistryError::Cycle { path }]),
        };
        Ok(DependencyGraph {
            providers: self.providers,
            order,
        })
    }
}

/// Kahn's algorithm with a sorted ready set (deterministic). On a cycle returns one concrete cycle.
fn topological_order(
    providers: &BTreeMap<DerivedKind, Arc<dyn Provider>>,
) -> Result<Vec<DerivedKind>, Vec<DerivedKind>> {
    let mut indegree: BTreeMap<DerivedKind, usize> = BTreeMap::new();
    let mut dependents: BTreeMap<DerivedKind, Vec<DerivedKind>> = BTreeMap::new();
    for (k, p) in providers {
        indegree.insert(*k, p.depends_on().len());
        for d in p.depends_on() {
            dependents.entry(*d).or_default().push(*k);
        }
    }
    let mut ready: BTreeSet<DerivedKind> = indegree
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(k, _)| *k)
        .collect();
    let mut order = Vec::with_capacity(providers.len());
    while let Some(k) = ready.pop_first() {
        order.push(k);
        for dep in dependents.get(&k).into_iter().flatten() {
            let n = indegree.get_mut(dep).expect("known kind");
            *n -= 1;
            if *n == 0 {
                ready.insert(*dep);
            }
        }
    }
    if order.len() == providers.len() {
        return Ok(order);
    }
    // Every kind left over has an unsatisfied dependency that is itself left over: walk
    // dependencies until a kind repeats.
    let left: BTreeSet<DerivedKind> = indegree
        .iter()
        .filter(|(_, n)| **n > 0)
        .map(|(k, _)| *k)
        .collect();
    let mut path = vec![*left.iter().next().expect("a cycle exists")];
    loop {
        let cur = *path.last().expect("non-empty");
        let next = providers[&cur]
            .depends_on()
            .iter()
            .filter(|d| left.contains(d))
            .min()
            .copied()
            .expect("a leftover kind has a leftover dependency");
        if let Some(pos) = path.iter().position(|k| *k == next) {
            let mut cycle = path.split_off(pos);
            cycle.push(next);
            return Err(cycle);
        }
        path.push(next);
    }
}

/// The frozen, validated set of providers and their ordering.
pub struct DependencyGraph {
    providers: BTreeMap<DerivedKind, Arc<dyn Provider>>,
    order: Vec<DerivedKind>,
}

impl std::fmt::Debug for DependencyGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DependencyGraph")
            .field("order", &self.order)
            .finish()
    }
}

impl DependencyGraph {
    /// The provider of `kind`.
    pub fn provider(&self, kind: DerivedKind) -> Option<&Arc<dyn Provider>> {
        self.providers.get(&kind)
    }

    /// All kinds in execution order: every kind comes after the kinds it depends on.
    pub fn order(&self) -> &[DerivedKind] {
        &self.order
    }

    /// Kinds that (transitively) depend on `kind`, in execution order, excluding `kind`.
    pub fn downstream(&self, kind: DerivedKind) -> Vec<DerivedKind> {
        let mut hit: BTreeSet<DerivedKind> = BTreeSet::new();
        hit.insert(kind);
        let mut out = Vec::new();
        for k in &self.order {
            if *k != kind
                && self.providers[k]
                    .depends_on()
                    .iter()
                    .any(|d| hit.contains(d))
            {
                hit.insert(*k);
                out.push(*k);
            }
        }
        out
    }

    /// The derived items that must be recomputed after `dirty`: for every dirty brick and every
    /// provider that reacts to the change type, every brick within the provider's apron (their
    /// snapshots contain the dirty brick), plus the kinds downstream of each such provider for the
    /// same bricks. Sorted and without duplicates, so the result is deterministic. A brick whose
    /// neighbours would lie beyond the `i64` grid is clipped to the grid.
    pub fn invalidated_by(&self, dirty: &[DirtyRegion]) -> Vec<DerivedTarget> {
        let mut out: BTreeSet<DerivedTarget> = BTreeSet::new();
        // Per provider: its apron, the changes it reacts to and the kinds to recompute with it.
        let chains: Vec<(i64, crate::provider::ChangeMask, Vec<DerivedKind>)> = self
            .providers
            .iter()
            .map(|(kind, p)| {
                let mut kinds = vec![*kind];
                kinds.extend(self.downstream(*kind));
                (i64::from(p.apron()), p.consumes(), kinds)
            })
            .collect();
        for d in dirty {
            let level = d.region.level();
            for brick in d.region.bricks() {
                for (a, mask, kinds) in &chains {
                    if !mask.contains(d.change) {
                        continue;
                    }
                    let lo = |v: i64| v.saturating_sub(*a);
                    let hi = |v: i64| v.saturating_add(*a);
                    for x in lo(brick.x)..=hi(brick.x) {
                        for y in lo(brick.y)..=hi(brick.y) {
                            for z in lo(brick.z)..=hi(brick.z) {
                                let Ok(key) =
                                    CellKey::new(d.layer, level, BrickCoord::new(x, y, z))
                                else {
                                    continue;
                                };
                                for k in kinds {
                                    out.insert(DerivedTarget { kind: *k, key });
                                }
                            }
                        }
                    }
                }
            }
        }
        out.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{ChangeMask, ComputeContext, Payload, ProviderFailure};

    struct P {
        kind: u32,
        name: &'static str,
        deps: Vec<DerivedKind>,
        mask: ChangeMask,
        apron: u32,
    }

    impl P {
        fn chain(kind: u32, deps: &[u32]) -> Arc<dyn Provider> {
            Arc::new(Self {
                kind,
                name: "p",
                deps: deps.iter().map(|d| DerivedKind(*d)).collect(),
                mask: if deps.is_empty() {
                    ChangeMask::ALL
                } else {
                    ChangeMask::NONE
                },
                apron: 0,
            })
        }
        fn with(kind: u32, mask: ChangeMask, apron: u32) -> Arc<dyn Provider> {
            Arc::new(Self {
                kind,
                name: "w",
                deps: vec![],
                mask,
                apron,
            })
        }
    }

    impl Provider for P {
        fn kind(&self) -> DerivedKind {
            DerivedKind(self.kind)
        }
        fn name(&self) -> &str {
            self.name
        }
        fn version(&self) -> u32 {
            1
        }
        fn consumes(&self) -> ChangeMask {
            self.mask
        }
        fn depends_on(&self) -> &[DerivedKind] {
            &self.deps
        }
        fn apron(&self) -> u32 {
            self.apron
        }
        fn compute(&self, _: &ComputeContext<'_>) -> Result<Payload, ProviderFailure> {
            Ok(Arc::new(()))
        }
    }

    fn registry(ps: Vec<Arc<dyn Provider>>) -> ProviderRegistry {
        let mut r = ProviderRegistry::new();
        for p in ps {
            r.register(p).unwrap();
        }
        r
    }

    #[test]
    fn order_is_topological_and_deterministic_regardless_of_registration_order() {
        let a = vec![
            P::chain(3, &[2]),
            P::chain(2, &[1]),
            P::chain(1, &[]),
            P::chain(9, &[]),
        ];
        let mut b = vec![
            P::chain(9, &[]),
            P::chain(1, &[]),
            P::chain(2, &[1]),
            P::chain(3, &[2]),
        ];
        let g1 = registry(a).freeze().unwrap();
        b.reverse();
        let g2 = registry(b).freeze().unwrap();
        let want: Vec<DerivedKind> = [1, 2, 3, 9].iter().map(|k| DerivedKind(*k)).collect();
        // Ready kinds are taken smallest first: 1, then 2 and 3 (both smaller than 9), then 9.
        assert_eq!(g1.order(), want.as_slice());
        assert_eq!(g2.order(), g1.order());
    }

    #[test]
    fn duplicate_kinds_are_rejected_and_the_first_provider_stays() {
        let mut r = ProviderRegistry::new();
        r.register(P::chain(1, &[])).unwrap();
        let err = r.register(P::chain(1, &[])).unwrap_err();
        assert!(matches!(err, RegistryError::DuplicateKind { kind, .. } if kind == DerivedKind(1)));
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn missing_dependencies_and_all_other_problems_are_reported_together() {
        let r = registry(vec![
            P::chain(1, &[7]),
            P::with(2, ChangeMask::ALL, MAX_APRON + 1),
            P::with(3, ChangeMask::NONE, 0),
            P::chain(4, &[4]),
        ]);
        let errs = r.freeze().unwrap_err();
        assert!(errs.contains(&RegistryError::MissingDependency {
            kind: DerivedKind(1),
            missing: DerivedKind(7)
        }));
        assert!(errs.contains(&RegistryError::ApronTooLarge {
            kind: DerivedKind(2),
            apron: MAX_APRON + 1
        }));
        assert!(errs.contains(&RegistryError::NeverInvalidated {
            kind: DerivedKind(3)
        }));
        assert!(errs.contains(&RegistryError::InvalidDependencyList {
            kind: DerivedKind(4)
        }));
    }

    #[test]
    fn cycles_are_found_and_reported_as_a_concrete_path() {
        let r = registry(vec![
            P::chain(1, &[3]),
            P::chain(2, &[1]),
            P::chain(3, &[2]),
            P::chain(8, &[]),
        ]);
        let errs = r.freeze().unwrap_err();
        let [RegistryError::Cycle { path }] = errs.as_slice() else {
            panic!("expected one cycle, got {errs:?}");
        };
        assert_eq!(path.first(), path.last());
        assert_eq!(path.len(), 4);
        let set: BTreeSet<_> = path.iter().collect();
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn a_two_node_cycle_is_detected() {
        let r = registry(vec![P::chain(1, &[2]), P::chain(2, &[1])]);
        assert!(matches!(
            r.freeze().unwrap_err().as_slice(),
            [RegistryError::Cycle { .. }]
        ));
    }

    #[test]
    fn downstream_lists_transitive_dependents_in_order() {
        let g = registry(vec![
            P::chain(1, &[]),
            P::chain(2, &[1]),
            P::chain(3, &[2]),
            P::chain(4, &[1]),
            P::chain(5, &[]),
        ])
        .freeze()
        .unwrap();
        assert_eq!(
            g.downstream(DerivedKind(1)),
            vec![DerivedKind(2), DerivedKind(3), DerivedKind(4)]
        );
        assert!(g.downstream(DerivedKind(5)).is_empty());
        assert!(g.provider(DerivedKind(9)).is_none());
    }

    #[test]
    fn empty_registry_freezes_to_an_empty_graph() {
        let g = ProviderRegistry::new().freeze().unwrap();
        assert!(g.order().is_empty());
    }
}
