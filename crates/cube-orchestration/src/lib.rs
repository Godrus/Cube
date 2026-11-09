//! Cube Editing Orchestration: the single user commit path (PS-4).
//!
//! Implements the editing service of `cube-editing-api` over Core: authorization, backend
//! dispatch by capabilities, private overlays, expected-revision validation, atomic commit
//! through the user commit handle, write-ahead journal boundary, dirty propagation and observer
//! notification. It holds no representation-specific logic (backends are ports) and no game
//! rules (authorization is a port).
//!
//! Only `commit_path.rs` may name the user commit handle (`cargo xtask archcheck`), and nothing
//! in this crate offers a way to obtain a system or recovery commit.

pub mod auth;
pub mod commit_path;
pub mod dirty;
pub mod registry;
pub mod txn;
pub mod validate;

pub use auth::{Authorizer, CapabilityAuthorizer, DenyAll};
pub use commit_path::CommitPath;
pub use dirty::DirtySink;
pub use registry::{BackendRegistry, RegistryError};
pub use txn::{Config, OrchestrationService, RecoveryReport};
pub use validate::{check_expected_tokens, check_write_sets, conflict_result};
