//! Operation identity and the origin of a change (CORE-07, contract C-17, DEC-059, DEC-063).

/// Identifier of one submitted operation: `(scope, sequence)` assigned by the authoritative side,
/// never by a client. Equality only: it is not a revision and its order carries no meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperationId {
    scope: u64,
    sequence: u64,
}

impl OperationId {
    /// Schema version of the persisted form (DEC-063: stored separately from the id).
    pub const SCHEMA_VERSION: u8 = 1;

    /// Creates an id.
    pub const fn new(scope: u64, sequence: u64) -> Self {
        Self { scope, sequence }
    }
    /// The scope (for example an actor or a session).
    pub const fn scope(self) -> u64 {
        self.scope
    }
    /// The per-scope sequence number.
    pub const fn sequence(self) -> u64 {
        self.sequence
    }
}

/// Who caused a change. Persisted, so the set is extensible: new variants are additions to a
/// versioned format (DEC-059). `Simulation` is reserved for changes originated by world rules
/// (renewal, collapse, mechanisms); its policy is not decided yet and no handle issues it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ActorKind {
    /// A user-originated edit (through the Editing API).
    User,
    /// System generation or eviction (WorldGen boundary).
    System,
    /// Replay of persisted history during recovery.
    Recovery,
    /// Reserved: world-simulation originated change.
    Simulation,
}

impl ActorKind {
    /// Stable numeric code for persisted forms. Codes are never reused.
    pub const fn code(self) -> u8 {
        match self {
            Self::User => 1,
            Self::System => 2,
            Self::Recovery => 3,
            Self::Simulation => 4,
        }
    }

    /// The actor for a stored code, `None` for an unknown code.
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::User),
            2 => Some(Self::System),
            3 => Some(Self::Recovery),
            4 => Some(Self::Simulation),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_ids_compare_by_both_parts() {
        assert_eq!(OperationId::new(1, 2), OperationId::new(1, 2));
        assert_ne!(OperationId::new(1, 2), OperationId::new(2, 1));
        assert_eq!(
            (
                OperationId::new(7, 9).scope(),
                OperationId::new(7, 9).sequence()
            ),
            (7, 9)
        );
    }

    #[test]
    fn actor_codes_are_stable_and_round_trip() {
        let all = [
            ActorKind::User,
            ActorKind::System,
            ActorKind::Recovery,
            ActorKind::Simulation,
        ];
        assert_eq!(all.map(ActorKind::code), [1, 2, 3, 4]);
        for a in all {
            assert_eq!(ActorKind::from_code(a.code()), Some(a));
        }
        assert_eq!(ActorKind::from_code(0), None);
        assert_eq!(ActorKind::from_code(5), None);
    }
}
