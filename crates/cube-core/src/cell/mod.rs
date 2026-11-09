//! Cell model (CORE-03): channels, payloads, the five cell states and sparse brick storage.

mod channel;
mod state;
mod store;

pub use channel::{Channel, ChannelId, ChannelSet};
pub use state::{CellPayload, CellState, LoadFailureReason, MeshPage};
pub use store::{BrickBounds, CellStore};
