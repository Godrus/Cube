//! Core errors (draft C-19). Stable codes; no memory references in the payload.

use crate::coord::BrickCoord;
use crate::ids::LayerId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreError {
    InvalidLayer(LayerId),
    /// Expected brick state did not match; lists the conflicting bricks.
    RevisionConflict {
        bricks: Vec<BrickCoord>,
    },
    InvalidWriteSet(&'static str),
    InvalidImport(&'static str),
}

impl CoreError {
    pub fn code(&self) -> &'static str {
        match self {
            CoreError::InvalidLayer(_) => "CORE-0001",
            CoreError::RevisionConflict { .. } => "CORE-0002",
            CoreError::InvalidWriteSet(_) => "CORE-0003",
            CoreError::InvalidImport(_) => "CORE-0004",
        }
    }
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::InvalidLayer(l) => write!(f, "{}: invalid layer {}", self.code(), l.0),
            CoreError::RevisionConflict { bricks } => {
                write!(
                    f,
                    "{}: revision conflict on {} brick(s)",
                    self.code(),
                    bricks.len()
                )
            }
            CoreError::InvalidWriteSet(m) => write!(f, "{}: invalid write set: {}", self.code(), m),
            CoreError::InvalidImport(m) => write!(f, "{}: invalid import: {}", self.code(), m),
        }
    }
}

impl std::error::Error for CoreError {}
