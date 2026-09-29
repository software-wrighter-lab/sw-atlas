//! What belongs in the corpus, and what does not.
//!
//! Repositories that are public but are not publishable artifacts, and the
//! decision that reads that list alongside the declared forks and the
//! canonical-copy declarations.
//!
//! A corpus that quietly drops what it cannot describe reports a coverage
//! number that means nothing, so every exclusion is written down with its
//! reason and the coverage report states how many there are. The bar is
//! high: `softwarewrighter/images` holds static images for a Pages site and
//! `softwarewrighter/placeholder` is "repo for future use". Thinness is not
//! a reason -- a real project with a description belongs in the corpus even
//! if nobody has written about it yet.

use crate::record::Record;
use atlas_core::ResourceId;
use atlas_supersede::Canonical;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

/// `sources/repo-exclusions.ron`.
#[derive(Debug, Default, Deserialize)]
pub struct Exclusions {
    /// `owner/name`, and why it is not an artifact.
    #[serde(default)]
    pub excluded: BTreeMap<String, String>,
}

impl Exclusions {
    /// Read the list.
    ///
    /// # Errors
    ///
    /// The file cannot be read, or is not an `Exclusions` in RON.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Whether this repository is excluded.
    pub fn excludes(&self, full_name: &str) -> bool {
        self.excluded.contains_key(full_name)
    }
}

/// Whether this repository belongs in the corpus.
///
/// A fork is kept when a post links to it, or when a declaration calls it the
/// real copy of work that exists twice. Anything superseded, or listed as not
/// an artifact at all, is out.
pub fn wanted(
    record: &Record,
    id: &ResourceId,
    declared: &BTreeSet<ResourceId>,
    rules: (&Exclusions, &Canonical),
) -> bool {
    let (skip, canonical) = rules;
    let known = declared.contains(id) || canonical.keeps(&record.full_name);
    let out = skip.excludes(&record.full_name) || canonical.drops(&record.full_name);
    (known || !record.fork) && !out
}
