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

    /// One corpus from several, with each resource, concept and relation
    /// appearing once.
    ///
    /// Sources describe each other: a post declares a video, and the video
    /// ingester describes that same video. Both produce the identifier, so
    /// the described resource wins over the stub that only named it, and a
    /// stub carrying a title wins over one that does not.
    pub fn merge(corpora: &[Self]) -> Self {
        let mut out = Self::new();
        for corpus in corpora {
            out.resources.extend(corpus.resources.iter().cloned());
            out.concepts.extend(corpus.concepts.iter().cloned());
            out.relations.extend(corpus.relations.iter().cloned());
        }
        let known = |r: &Resource| {
            usize::from(!r.source_hash.is_empty()) * 2 + usize::from(!r.title.is_empty())
        };
        out.resources
            .sort_by(|a, b| (&a.id, known(b)).cmp(&(&b.id, known(a))));
        out.resources.dedup_by(|a, b| a.id == b.id);
        out.concepts.sort_by(|a, b| a.id.cmp(&b.id));
        out.concepts.dedup_by(|a, b| a.id == b.id);
        out.relations
            .sort_by(|a, b| (&a.from, &a.kind, &a.to).cmp(&(&b.from, &b.kind, &b.to)));
        out.relations.dedup();
        out
    }
}
