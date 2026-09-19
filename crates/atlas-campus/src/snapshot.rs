//! The campus catalog snapshot, as sw-campus publishes it.
//!
//! The file is a struct of parallel arrays with pipe-separated
//! multi-values, which is how moe-microscope's docent corpus reads it.
//! This crate reads that shape rather than asking sw-campus to change it.

use serde::Deserialize;

/// One published snapshot of the campus catalog.
#[derive(Debug, Clone, Deserialize)]
pub struct Snapshot {
    /// Which snapshot this is: "A", "B", and so on.
    pub snapshot: String,
    /// Where the catalog was read from, and when.
    pub source: String,
    /// Base URL every place hangs off.
    pub url_base: String,
    /// What was deliberately left out, and why.
    #[serde(default)]
    pub excluded: String,
    /// Place identifiers.
    pub ids: Vec<String>,
    /// Place kinds: campus, building, wing, exhibit.
    pub kinds: Vec<String>,
    /// Each place's parent, empty for the campus itself.
    pub parents: Vec<String>,
    /// Place titles.
    pub titles: Vec<String>,
    /// One-line descriptions.
    pub taglines: Vec<String>,
    /// Pipe-separated `label=url` pairs.
    pub links: Vec<String>,
    /// Pipe-separated alternative names.
    pub aliases: Vec<String>,
    /// Pipe-separated concepts.
    pub concepts: Vec<String>,
    /// Story identifiers.
    pub story_ids: Vec<String>,
    /// The place each story belongs to.
    pub story_places: Vec<String>,
    /// arrival, detail, feature or anecdote.
    pub story_kinds: Vec<String>,
    /// Story titles.
    pub story_titles: Vec<String>,
    /// The stories themselves.
    pub story_texts: Vec<String>,
}

/// Read a snapshot from disk.
///
/// # Errors
///
/// Fails if the file cannot be read or is not a snapshot of this shape. A
/// catalog that has grown a field is fine; one that has changed the
/// meaning of a field fails here, which is the intended behaviour.
pub fn load(path: &std::path::Path) -> Result<Snapshot, crate::Error> {
    let text = std::fs::read_to_string(path).map_err(crate::Error::Io)?;
    serde_json::from_str(&text).map_err(|e| crate::Error::Snapshot {
        path: path.display().to_string(),
        cause: e.to_string(),
    })
}

/// Split one of the pipe-separated fields.
pub fn split(field: &str) -> Vec<String> {
    field
        .split('|')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}
