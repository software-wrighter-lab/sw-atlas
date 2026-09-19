//! Edges between resources, and where each edge came from.

use crate::catalog::{Resource, ResourceKind};
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

/// A link one resource declares to another, before either is a resource.
///
/// Both the blog's front matter and the campus catalog declare links this
/// way, and both must resolve to the *same* identifier when they name the
/// same thing -- that shared identity is what lets a visitor on the campus
/// find the post about what they are looking at.
pub struct Link {
    /// What sort of thing it points at.
    pub kind: ResourceKind,
    /// Where it points.
    pub url: String,
    /// What the declarer called it, where they said.
    pub title: Option<String>,
}

/// The identifier a link's target gets, derived from its URL.
///
/// Two sources naming one repository, video or paper produce one resource
/// rather than two, however differently they wrote the URL.
pub fn link_id(link: &Link) -> ResourceId {
    let trimmed = link
        .url
        .trim_end_matches('/')
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");
    let (prefix, tail) = match link.kind {
        ResourceKind::Repo => ("repo", trimmed.trim_start_matches("github.com/")),
        ResourceKind::Video => (
            "video",
            trimmed.rsplit(['/', '=']).next().unwrap_or(trimmed),
        ),
        ResourceKind::Demo => ("demo", trimmed),
        _ => ("paper", trimmed),
    };
    ResourceId::new(format!("{prefix}:{tail}"))
}

/// The resource a link names and the edge that reaches it.
pub fn edge(from: &ResourceId, link: &Link) -> (Resource, Relation) {
    let kind = match link.kind {
        ResourceKind::Repo => RelationKind::Implements,
        ResourceKind::Video | ResourceKind::Demo => RelationKind::Demos,
        ResourceKind::Paper => RelationKind::Cites,
        _ => RelationKind::RelatedTo,
    };
    let to = link_id(link);
    let title = link.title.clone().unwrap_or_default();
    let target = Resource::stub(to.clone(), link.kind, title, link.url.clone());
    let relation = Relation {
        from: from.clone(),
        kind,
        to,
        weight: 1.0,
        provenance: Provenance::Declared,
    };
    (target, relation)
}
