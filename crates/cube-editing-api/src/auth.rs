//! E-02 AuthorizationContext (API-03).
//!
//! Who is acting and with what rights. It is produced by the networking/session layer, evaluated
//! server-side, and treated by the Editing API as an opaque immutable input. This module only
//! carries the data and the documented mapping from origin to the Core actor kind; it decides
//! nothing: the permission model and region-level ACLs are open (E-02 Open) and belong to
//! gameplay and the server.
//!
//! **No extra rights from the origin:** rights come only from `capabilities` and `scope`. A
//! `LocalClient` context with the same capabilities and scope as a `Server` context has exactly
//! the same rights. The origin only decides how the change is attributed (`ActorKind`).

use cube_core::{ActorKind, AssetId, LayerId, Region};

/// Identifier of an acting user or system component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActorId(pub u64);

/// Identifier of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionId(pub u64);

/// A named right, defined by the module that owns the permission schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Capability(pub u32);

/// Where the request came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origin {
    /// A client of the local process (single player, editor). Gives no extra rights.
    LocalClient,
    /// The server on behalf of a connected user.
    Server,
    /// A system component (generation, maintenance).
    System,
}

impl Origin {
    /// The Core actor kind a change from this origin is attributed to: user-originated edits are
    /// `User` whether they come from a local client or through the server; system components
    /// are `System`. Recovery replay never comes through an authorization context.
    pub const fn actor_kind(self) -> ActorKind {
        match self {
            Self::LocalClient | Self::Server => ActorKind::User,
            Self::System => ActorKind::System,
        }
    }
}

/// A region a context may touch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopedRegion {
    /// Layer of the region.
    pub layer: LayerId,
    /// The region.
    pub region: Region,
}

/// What an actor may reach. An empty scope allows nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scope {
    /// Assets the actor may edit as a whole.
    pub assets: Vec<AssetId>,
    /// Regions the actor may edit inside other assets.
    pub regions: Vec<ScopedRegion>,
}

/// The acting identity of a transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationContext {
    /// Who acts.
    pub actor: ActorId,
    /// Session of the request.
    pub session: SessionId,
    /// Origin of the request.
    pub origin: Origin,
    /// Rights held.
    pub capabilities: Vec<Capability>,
    /// Reach of the rights.
    pub scope: Scope,
}

impl AuthorizationContext {
    /// `true` if the context holds `capability`. The origin is deliberately not consulted.
    pub fn has(&self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }

    /// The Core actor kind the changes of this context are attributed to.
    pub const fn actor_kind(&self) -> ActorKind {
        self.origin.actor_kind()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(origin: Origin) -> AuthorizationContext {
        AuthorizationContext {
            actor: ActorId(7),
            session: SessionId(1),
            origin,
            capabilities: vec![Capability(2)],
            scope: Scope::default(),
        }
    }

    #[test]
    fn origin_maps_to_the_documented_actor_kind() {
        assert_eq!(Origin::LocalClient.actor_kind(), ActorKind::User);
        assert_eq!(Origin::Server.actor_kind(), ActorKind::User);
        assert_eq!(Origin::System.actor_kind(), ActorKind::System);
        assert_eq!(ctx(Origin::System).actor_kind(), ActorKind::System);
    }

    #[test]
    fn the_local_client_origin_gives_no_extra_rights() {
        let (local, server) = (ctx(Origin::LocalClient), ctx(Origin::Server));
        for c in [Capability(1), Capability(2), Capability(3)] {
            assert_eq!(local.has(c), server.has(c));
        }
        assert!(local.has(Capability(2)) && !local.has(Capability(1)));
        assert_eq!(local.scope, server.scope);
    }
}
