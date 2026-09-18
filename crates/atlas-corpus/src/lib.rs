//! The corpus collection, its validator, and its canonical form.
//!
//! Everything here is deterministic: the same facts produce the same bytes
//! and the same hash on any machine, which is what the nightly pipeline and
//! the snapshot manifest are built on.

pub mod canon;
pub mod corpus;
pub mod validate;

pub use canon::{canonical, content_hash, to_ron};
pub use corpus::Corpus;
pub use validate::{Problem, validate};
