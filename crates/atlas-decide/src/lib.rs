//! What a model may say about the corpus, and the rows it is judged on.
//!
//! Separated from `atlas-core` because the corpus outlives any model: the
//! catalog is published and read by a browser that may be running no model
//! at all, while everything here exists only to train one and to score it.

pub mod decision;
pub mod question;

pub use decision::{Decision, Intent, ScoredConcept, ScoredKind};
pub use question::{Origin, Question, Split};
