//! Edges between resources, and where each edge came from.

use crate::id::ResourceId;
use serde::{Deserialize, Serialize};

/// What one resource is to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RelationKind {
    /// A post discusses a subject resource.
    Discusses,
    /// A repository implements what a post describes.
    Implements,
    /// A demo or video shows a thing working.
    Demos,
    /// A post cites an outside work.
    Cites,
    /// A place sits inside another place.
    PartOf,
    /// A weaker association than the kinds above.
    RelatedTo,
    /// The next instalment of a series.
    SeriesNext,
}

/// Where an edge came from, and therefore how far it can be trusted.
///
/// This is load-bearing. A `Declared` edge was written by the author in
/// front matter and is not second-guessed. A `Teacher` edge came from a
/// model and is the only kind that can be wrong in an interesting way, so
/// every published metric must be recomputable with `Teacher` excluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Provenance {
    /// Written by the author, in front matter or a catalog field.
    Declared,
    /// Produced by deterministic analysis of declared data.
    Derived,
    /// Produced by a model.
    Teacher,
}

/// One directed edge in the corpus graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relation {
    /// The resource the edge leaves.
    pub from: ResourceId,
    /// What the edge asserts.
    pub kind: RelationKind,
    /// The resource the edge arrives at.
    pub to: ResourceId,
    /// Strength, for ranking. Declared edges are 1.0 unless stated.
    pub weight: f32,
    /// Where this edge came from.
    pub provenance: Provenance,
}
