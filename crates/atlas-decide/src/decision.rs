//! What the model is allowed to say.

use atlas_core::{ConceptId, ResourceKind};
use serde::{Deserialize, Serialize};

/// What a visitor is asking for.
///
/// The campus docent had six intents. `FindResource` and `Compare` are
/// what a question spanning the whole corpus needs and a single-site
/// docent never did: "where did you write about X" and "how does X differ
/// from Y" are not navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Intent {
    /// Take me somewhere.
    Navigate,
    /// Tell me what this is.
    Explain,
    /// Suggest something I would like.
    Recommend,
    /// Tell me the story behind it.
    Story,
    /// Tell me how far along it is.
    Status,
    /// Find me the artifact, of whatever kind, that covers this.
    FindResource,
    /// Set two things side by side.
    Compare,
    /// Nothing here answers this.
    Unsupported,
}

/// A weighted concept, as the model scores it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoredConcept {
    /// The concept.
    pub id: ConceptId,
    /// Its probability under the model.
    pub score: f32,
}

/// A weighted resource kind, as the model scores it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoredKind {
    /// The kind.
    pub kind: ResourceKind,
    /// Its probability under the model.
    pub score: f32,
}

/// The model's entire output.
///
/// Note what is absent: no prose, no URL, no resource identifier. Ordinary
/// code resolves an intent and a set of concepts against the catalog, so a
/// stale model cannot invent an artifact or a dead link. The worst it can
/// do is rank badly, and `confidence` is where it says so.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    /// Intent distribution, one entry per intent the model scored.
    pub intents: Vec<(Intent, f32)>,
    /// Concepts the question is about.
    pub concepts: Vec<ScoredConcept>,
    /// Kinds of artifact the answer should prefer.
    pub kinds: Vec<ScoredKind>,
    /// How sure the model is overall, calibrated so that 0.9 means right
    /// about nine times in ten.
    pub confidence: f32,
    /// How close the runners-up are. High ambiguity with high confidence
    /// is a correct answer of the form "I have two good matches".
    pub ambiguity: f32,
}
