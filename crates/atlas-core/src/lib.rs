//! Canonical corpus schema for sw-atlas.
//!
//! The corpus is the authority for facts; a model only ever maps language
//! onto identifiers defined here. Nothing in this crate knows about
//! inference, browsers or weights.
//!
//! Three crates hold the schema, split because `sw-checklist` allows four
//! modules to a crate and the split is meaningful anyway: `atlas-core` is
//! what the corpus *is*, `atlas-decide` is what a model says about it, and
//! `atlas-corpus` is the collection with its validator and canonical form.

pub mod catalog;
pub mod id;
pub mod relation;

pub use catalog::{Concept, Maturity, Resource, ResourceKind};
pub use id::{ConceptId, ResourceId};
pub use relation::{Provenance, Relation, RelationKind};

/// Version of the corpus schema this build reads and writes.
///
/// A published snapshot records this in its manifest. A runtime that finds
/// a snapshot declaring a different major version refuses it rather than
/// guessing at the difference.
pub const SCHEMA_VERSION: &str = "0.1";

/// Producer name recorded in every snapshot manifest.
pub const PRODUCER: &str = "sw-atlas";
