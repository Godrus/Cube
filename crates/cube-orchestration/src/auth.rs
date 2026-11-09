//! The authorization port of Orchestration (DEC-083).
//!
//! Server/Authorization owns the permission schema and region ACLs; Orchestration only asks.
//! Authorization is checked before revisions are validated, and a denial leaves no state. The
//! origin never adds rights: a local client is evaluated exactly like a server request.

use cube_editing_api::{
    AuthorizationContext, Capability, EditingError, EditingOperation, TransactionSpec,
};

/// Decides whether a context may start a transaction and run an operation.
pub trait Authorizer: Send + Sync {
    /// May this context begin a transaction with this spec?
    fn authorize_begin(
        &self,
        context: &AuthorizationContext,
        spec: &TransactionSpec,
    ) -> Result<(), EditingError>;

    /// May this context run this operation?
    fn authorize_operation(
        &self,
        context: &AuthorizationContext,
        operation: &EditingOperation,
    ) -> Result<(), EditingError>;
}

/// The minimal policy for PS-4: the context must hold one capability and name the asset in its
/// scope. No region ACLs (not part of PS-4); nothing is granted by the origin.
#[derive(Debug, Clone, Copy)]
pub struct CapabilityAuthorizer {
    /// The capability that allows editing.
    pub edit: Capability,
}

impl CapabilityAuthorizer {
    fn allows(&self, c: &AuthorizationContext, asset: cube_core::AssetId) -> bool {
        c.has(self.edit) && c.scope.assets.contains(&asset)
    }
}

impl Authorizer for CapabilityAuthorizer {
    fn authorize_begin(
        &self,
        context: &AuthorizationContext,
        spec: &TransactionSpec,
    ) -> Result<(), EditingError> {
        if self.allows(context, spec.asset) {
            Ok(())
        } else {
            Err(EditingError::PermissionDenied)
        }
    }

    fn authorize_operation(
        &self,
        context: &AuthorizationContext,
        operation: &EditingOperation,
    ) -> Result<(), EditingError> {
        if self.allows(context, operation.asset) {
            Ok(())
        } else {
            Err(EditingError::PermissionDenied)
        }
    }
}

/// Denies everything: the safe default when no policy is configured.
#[derive(Debug, Clone, Copy, Default)]
pub struct DenyAll;

impl Authorizer for DenyAll {
    fn authorize_begin(
        &self,
        _: &AuthorizationContext,
        _: &TransactionSpec,
    ) -> Result<(), EditingError> {
        Err(EditingError::PermissionDenied)
    }
    fn authorize_operation(
        &self,
        _: &AuthorizationContext,
        _: &EditingOperation,
    ) -> Result<(), EditingError> {
        Err(EditingError::PermissionDenied)
    }
}
