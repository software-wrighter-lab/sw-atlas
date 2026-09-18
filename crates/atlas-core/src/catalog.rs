//! The two things the catalog contains: artifacts, and what they are about.
//!
//! Kept in one module because `sw-checklist` allows four modules to a
//! crate and these are one concern: a resource is a thing, a concept is
//! what a thing is about, and neither is meaningful without the other.

use crate::id::{ConceptId, ResourceId};
use serde::{Deserialize, Serialize};

/// What kind of artifact a resource is.
///
/// The kind is part of what the model predicts: "have you written about
/// X" wants a `Post`, "is there a demo" wants a `Demo`, and a question may
/// legitimately want several kinds ranked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResourceKind {
    /// A blog post.
    Post,
    /// A place on the campus: campus, building, wing or exhibit.
    Campus,
    /// A public git repository.
    Repo,
    /// A video.
    Video,
    /// A live, runnable demonstration.
    Demo,
    /// An outside work that a post cites.
    Paper,
}

/// How far along the thing a resource describes actually is.
///
/// Derived from the catalog where a place does not say: a live demo or
/// scene means `Working`, repositories only means `Early`, a placeholder
/// means `Planned`. A hand-written note overrides the derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Maturity {
    /// Complete and stable.
    Finished,
    /// Runs today, still improving.
    Working,
    /// Source exists, nothing runnable.
    Early,
    /// On the map, nothing built.
    Planned,
}

/// One artifact in the corpus.
///
/// Everything a visitor is ever told comes from a field here or from a
/// story attached to one. The model never produces any of it; it produces
/// concepts and kinds, and deterministic code finds the resources that
/// match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// Stable identifier.
    pub id: ResourceId,
    /// What kind of artifact this is.
    pub kind: ResourceKind,
    /// Human title, shown as written.
    pub title: String,
    /// Where it lives. Empty for a resource with no page of its own.
    pub url: String,
    /// Publication or last-change date, ISO 8601, empty when unknown.
    pub date: String,
    /// Short summary, taken from an author's own words wherever one exists.
    pub summary: String,
    /// Authored long-form text for this resource: a campus story, a video
    /// script. Empty when there is none yet; a video's script arrives here
    /// when the repositories holding the scripts are available, with no
    /// change to this type. Never training text; always answer material.
    pub body: String,
    /// Concepts this resource is about.
    pub concepts: Vec<ConceptId>,
    /// Other ways a visitor might name this thing.
    pub aliases: Vec<String>,
    /// How far along it is, where that is known.
    pub maturity: Option<Maturity>,
    /// Hash of the source this was derived from, so a rebuild can tell
    /// whether the upstream artifact actually changed.
    pub source_hash: String,
}

/// One idea that resources can be about.
///
/// Concepts are the model's output vocabulary, which is why they are
/// normalised and curated rather than taken raw from tags: `moe`,
/// `mixture-of-experts` and `mixture of experts` have to be one concept
/// or the model is asked to distinguish things a human would not.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Concept {
    /// Stable identifier.
    pub id: ConceptId,
    /// Human label.
    pub label: String,
    /// Other ways this concept is written or said.
    pub aliases: Vec<String>,
    /// Broader concepts this one sits under.
    pub parents: Vec<ConceptId>,
    /// Resources tagged with this concept. Derived, and kept here so the
    /// browser does not have to build the reverse index at startup.
    pub resources: Vec<ResourceId>,
}
