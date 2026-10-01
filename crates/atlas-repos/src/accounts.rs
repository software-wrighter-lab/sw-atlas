//! The accounts whose work is the owner's own.
//!
//! GitHub's `fork` field records which button was pressed, not who wrote the
//! code, so it cannot tell work moved between the owner's accounts from
//! somebody else's code kept for reference. The parent's owner can, and this
//! is the list it is checked against. Owner rule, 2026-09-26: a fork of my own
//! work is my work; a fork of somebody else's is not.

use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

/// `sources/accounts.ron`.
#[derive(Debug, Default, Deserialize)]
pub struct Accounts {
    /// Every account the owner writes under, including earlier ones that are
    /// no longer indexed but whose work was carried forward.
    #[serde(default)]
    pub owned: BTreeSet<String>,
}

impl Accounts {
    /// Read the list.
    ///
    /// # Errors
    ///
    /// The file cannot be read, or is not an `Accounts` in RON.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Whether a fork of this parent is the owner's own work.
    pub fn owns(&self, parent: Option<&String>) -> bool {
        parent
            .and_then(|full| full.split('/').next())
            .is_some_and(|owner| self.owned.contains(owner))
    }
}
