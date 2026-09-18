//! Stable identifiers for the two things the corpus names.
//!
//! Identifiers are opaque strings with a kind prefix, chosen so that a
//! human reading a training row or a relation edge can tell what it points
//! at without a lookup: `blog:2026-09-17-tbt-apl-360-revisited`,
//! `campus:apl`, `repo:sw-ml-study/moe-microscope`.

use serde::{Deserialize, Serialize};

/// Identifier of one resource: a post, exhibit, repository, video, demo or
/// paper.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceId(String);

impl ResourceId {
    /// Wrap an already-formed identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrow the identifier as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Identifier of one concept: `mixture-of-experts`, `array-languages`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConceptId(String);

impl ConceptId {
    /// Wrap an already-formed identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrow the identifier as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
