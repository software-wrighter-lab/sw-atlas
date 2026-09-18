//! The collection, and the indexes a validator and a matcher both need.

use atlas_core::{Concept, ConceptId, Relation, Resource, ResourceId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Every resource, concept and relation the ingesters produced.
///
/// Ordering is not meaningful here; [`crate::canon`] imposes one before
/// anything is written or hashed.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Corpus {
    /// Schema version the corpus was written against.
    pub schema_version: String,
    /// Every artifact.
    pub resources: Vec<Resource>,
    /// Every concept.
    pub concepts: Vec<Concept>,
    /// Every edge.
    pub relations: Vec<Relation>,
}

impl Corpus {
    /// An empty corpus stamped with the schema version of this build.
    pub fn new() -> Self {
        Self {
            schema_version: atlas_core::SCHEMA_VERSION.to_string(),
            ..Self::default()
        }
    }

    /// The set of resource identifiers, for membership tests.
    pub fn resource_ids(&self) -> BTreeSet<&ResourceId> {
        self.resources.iter().map(|r| &r.id).collect()
    }

    /// The set of concept identifiers, for membership tests.
    pub fn concept_ids(&self) -> BTreeSet<&ConceptId> {
        self.concepts.iter().map(|c| &c.id).collect()
    }
}
