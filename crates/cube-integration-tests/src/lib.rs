//! Leaf crate for cross-crate suites. Suites live in `tests/`; the library holds the shared
//! fixtures (`fakes`) and the generic conformance suites (`conformance`).

pub mod conformance;
pub mod fakes;
