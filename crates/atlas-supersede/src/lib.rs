//! Which copy of a repository is the real one, when two exist.
//!
//! Its own crate because superseding is a fifth concern: `atlas-repos` reads
//! the cache and `atlas-corpus` validates, and neither is the place to decide
//! that two resources are one piece of work. The house rule in CLAUDE.md says
//! to make the sibling crate before the fifth module, so here it is.
//!
//! GitHub's fork flag records which button was pressed, not who wrote the
//! code. Work moved into an organisation by forking is therefore marked as
//! somebody else's, permanently, and the abandoned original is marked as the
//! source -- so a rule that keeps forks only when a post links to them keeps
//! the copy nobody pushes to. `sources/repo-canonical.ron` says which copy is
//! real, per pair, in the owner's words.
//!
//! The superseded copy is not deleted from the world, only from the corpus:
//! its name becomes an alias on the canonical resource, so a visitor asking
//! for `proact` reaches `sw-init` rather than nothing.

pub mod apply;

pub use apply::supersede;

use serde::Deserialize;
use std::path::Path;

/// One pair: the copy that is real, and the copy it replaces.
#[derive(Debug, Clone, Deserialize)]
pub struct Pair {
    /// `owner/name` of the copy to keep, fork flag notwithstanding.
    pub canonical: String,
    /// `owner/name` of the copy to leave out.
    pub superseded: String,
    /// Why, in the owner's words.
    pub why: String,
    /// The canonical copy is currently behind the one it supersedes, and the
    /// owner intends to merge the difference forward. Recorded rather than
    /// resolved: a push date is evidence, not intent.
    #[serde(default)]
    pub pending_sync: bool,
}

/// `sources/repo-canonical.ron`.
#[derive(Debug, Default, Deserialize)]
pub struct Canonical {
    /// Every declared pair.
    #[serde(default)]
    pub pairs: Vec<Pair>,
}

impl Canonical {
    /// Read the declarations.
    ///
    /// # Errors
    ///
    /// The file cannot be read, or is not a `Canonical` in RON.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Whether this repository is declared canonical, and so is kept even
    /// though GitHub calls it a fork.
    pub fn keeps(&self, full_name: &str) -> bool {
        self.pairs.iter().any(|p| p.canonical == full_name)
    }

    /// Whether this repository is superseded, and so is left out.
    pub fn drops(&self, full_name: &str) -> bool {
        self.pairs.iter().any(|p| p.superseded == full_name)
    }

    /// The names a superseded copy was known by, to carry onto the canonical
    /// resource: the bare repository name, and its spoken form where those
    /// differ. `softwarewrighter/proact` yields `proact`.
    pub fn aliases(&self, full_name: &str) -> Vec<String> {
        let mut out: Vec<String> = self
            .pairs
            .iter()
            .filter(|p| p.canonical == full_name)
            .filter_map(|p| p.superseded.rsplit('/').next())
            .flat_map(|name| [name.to_string(), name.replace(['-', '_'], " ")])
            .collect();
        out.sort();
        out.dedup();
        out
    }
}
