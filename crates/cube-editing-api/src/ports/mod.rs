//! Ports of the Editing API (API-02, contracts E-07, E-11, E-12; ACR-002).
//!
//! Traits only. The editing service is implemented by Orchestration, backends by the Voxel, SDF
//! and Mesh crates, observers by History. Nothing here has behaviour; backends compile against
//! it without Orchestration.

mod backend;
mod observer;
mod service;

pub use backend::{BackendCapabilities, GeometryBackend, OverlayRead, ValidationReport};
pub use observer::TransactionObserver;
pub use service::EditingService;
