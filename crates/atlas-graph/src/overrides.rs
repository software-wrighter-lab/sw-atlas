//! The corrections a person makes to the normalizer, committed.
//!
//! The normalizer only unifies spellings. Every other relationship between
//! two labels is a judgement: `moe` and `mixture of experts` are one idea,
//! `llm` and `large language monkeys` are not, and no rule over the strings
//! can tell those two cases apart. So they are written down here, applied
//! after normalization, and survive every rebuild.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// `sources/concept-overrides.ron`.
#[derive(Debug, Default, Deserialize)]
pub struct Overrides {
    /// A concept key, and the keys that mean the same thing and fold into
    /// it. The key on the left survives.
    #[serde(default)]
    pub merge: BTreeMap<String, Vec<String>>,
    /// A concept key, and the broader concepts it sits under.
    #[serde(default)]
    pub parents: BTreeMap<String, Vec<String>>,
    /// A concept key, and the spelling to show a visitor instead of the
    /// one the corpus used most.
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    /// Pairs the normalizer must not merge, however alike they look.
    #[serde(default)]
    pub keep_apart: Vec<(String, String)>,
}

impl Overrides {
    /// Read the file.
    ///
    /// # Errors
    ///
    /// The file cannot be read, or is not an `Overrides` in RON.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Where a key ends up after the merges: itself, unless it folds.
    pub fn canonical(&self, key: &str) -> String {
        let target = self
            .merge
            .iter()
            .find(|(_, folded)| folded.iter().any(|f| f == key));
        target.map_or_else(|| key.to_string(), |(into, _)| into.clone())
    }

    /// Keys this file names that the corpus does not have.
    ///
    /// A correction for a concept nobody writes about is a typo or a
    /// leftover, and either way it is silently doing nothing, so the build
    /// says so instead.
    pub fn unknown(&self, keys: &BTreeSet<String>) -> Vec<String> {
        let named = self
            .merge
            .iter()
            .flat_map(|(into, folded)| std::iter::once(into).chain(folded))
            .chain(
                self.parents
                    .iter()
                    .flat_map(|(k, ps)| std::iter::once(k).chain(ps)),
            )
            .chain(self.labels.keys());
        let mut missing: Vec<String> = named
            .filter(|key| !keys.contains(key.as_str()))
            .cloned()
            .collect();
        missing.sort();
        missing.dedup();
        missing
    }
}
