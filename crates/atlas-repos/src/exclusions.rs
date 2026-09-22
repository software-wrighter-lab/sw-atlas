//! Repositories that are public but are not publishable artifacts.
//!
//! A corpus that quietly drops what it cannot describe reports a coverage
//! number that means nothing, so every exclusion is written down with its
//! reason and the coverage report states how many there are. The bar is
//! high: `softwarewrighter/images` holds static images for a Pages site and
//! `softwarewrighter/placeholder` is "repo for future use". Thinness is not
//! a reason -- a real project with a description belongs in the corpus even
//! if nobody has written about it yet.

use serde::Deserialize;
use std::collections::BTreeMap;
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
