//! Backend registry and dispatch by capabilities (ORCH-02, ACR-002).
//!
//! Orchestration reaches representation-specific logic only through registered
//! [`GeometryBackend`] values; it imports no backend crate. One backend serves one layer kind.

use std::collections::BTreeMap;
use std::sync::Arc;

use cube_core::LayerKind;
use cube_editing_api::{EditingError, GeometryBackend, KindId};

/// Why a backend could not be registered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryError {
    /// A backend for this layer kind is already registered.
    DuplicateLayerKind(LayerKind),
    /// The backend advertises no operation kind and could never be used.
    NoCapabilities,
}

/// Backends by layer kind.
#[derive(Default)]
pub struct BackendRegistry {
    backends: BTreeMap<u8, Arc<dyn GeometryBackend>>,
}

impl std::fmt::Debug for BackendRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendRegistry")
            .field("kinds", &self.backends.keys().collect::<Vec<_>>())
            .finish()
    }
}

fn key(kind: LayerKind) -> u8 {
    match kind {
        LayerKind::Voxel => 1,
        LayerKind::Sdf => 2,
        LayerKind::Mesh => 3,
    }
}

impl BackendRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a backend for the layer kind in its capabilities.
    pub fn register(&mut self, backend: Arc<dyn GeometryBackend>) -> Result<(), RegistryError> {
        let caps = backend.capabilities();
        if caps.kinds.is_empty() {
            return Err(RegistryError::NoCapabilities);
        }
        let k = key(caps.layer_kind);
        if self.backends.contains_key(&k) {
            return Err(RegistryError::DuplicateLayerKind(caps.layer_kind));
        }
        self.backends.insert(k, backend);
        Ok(())
    }

    /// The backend for `layer` that can apply `operation`. A missing backend and an unsupported
    /// operation kind are the same controlled error: `UnsupportedGeometryOperation`.
    pub fn dispatch(
        &self,
        layer: LayerKind,
        operation: KindId,
    ) -> Result<&Arc<dyn GeometryBackend>, EditingError> {
        let backend = self
            .backends
            .get(&key(layer))
            .ok_or(EditingError::UnsupportedGeometryOperation(operation))?;
        if backend.capabilities().supports(operation) {
            Ok(backend)
        } else {
            Err(EditingError::UnsupportedGeometryOperation(operation))
        }
    }

    /// Number of registered backends.
    pub fn len(&self) -> usize {
        self.backends.len()
    }

    /// `true` if nothing is registered.
    pub fn is_empty(&self) -> bool {
        self.backends.is_empty()
    }
}
