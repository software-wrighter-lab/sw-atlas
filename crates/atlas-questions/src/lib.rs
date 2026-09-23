//! The frozen evaluation sets, and the checks that keep them honest.
//!
//! These rows decide whether anything later in this project may claim to be
//! an improvement, so the rules about them are enforced here rather than
//! trusted:
//!
//! - **A paraphrase contains no alias of its own answer.** That is what
//!   makes it a paraphrase rather than a lookup, and it is checked
//!   mechanically against the corpus, because a person drafting three
//!   hundred questions cannot hold every alias of every resource in mind.
//! - **Every expected identifier resolves** to a resource that exists.
//! - **A drafted row is not a frozen row.** Rows written by a model are
//!   `Teacher` and `Unconfirmed` until the repository owner passes over
//!   them; a set's confirmed rows and its drafted rows are counted
//!   separately everywhere.
//! - **A frozen row never appears in training.** [`leakage`] checks that by
//!   hash and by shared eight-word runs.

pub mod leakage;
pub mod rules;

use atlas_corpus::content_hash;
use serde::Deserialize;
use std::path::Path;

/// What the visitor wants, mirroring `atlas_decide::Intent` without
/// depending on it: these files are written by hand and read by the
/// harness, and the harness should not have to load a model's vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Intent {
    /// Take me somewhere.
    Navigate,
    /// Tell me what this is.
    Explain,
    /// Suggest something.
    Recommend,
    /// Tell me the story.
    Story,
    /// How far along is it.
    Status,
    /// Find the artifact that covers this.
    FindResource,
    /// Set two things side by side.
    Compare,
    /// About the index itself, not the corpus.
    Meta,
    /// Nothing here answers this.
    Unsupported,
}

/// Who wrote a row, and therefore how far it can be trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Origin {
    /// Written by a person.
    Authored,
    /// Drafted by a model, for a person to accept or reject.
    Teacher,
}

/// Whether the repository owner has passed over a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Status {
    /// Accepted. Usable in a frozen set.
    Confirmed,
    /// Awaiting review. Counted, reported, and not usable as a yardstick.
    Unconfirmed,
}

/// One evaluation question.
#[derive(Debug, Clone, Deserialize)]
pub struct Row {
    /// What a visitor would type.
    pub text: String,
    /// Every resource that is a correct answer. Empty means the corpus
    /// cannot answer it, which is itself the expected answer.
    #[serde(default)]
    pub expect: Vec<String>,
    /// What the visitor wants.
    pub intent: Intent,
    /// Who wrote it.
    pub origin: Origin,
    /// Whether it has been reviewed.
    pub status: Status,
    /// Anything a reviewer should know.
    #[serde(default)]
    pub note: String,
}

/// One set: a file of rows with a name and the reason it exists.
#[derive(Debug, Clone, Deserialize)]
pub struct QuestionSet {
    /// `paraphrase`, `off-topic`, `ambiguous`, `meta`, `follow-up`.
    pub name: String,
    /// What this set is for, and what would make it invalid.
    pub note: String,
    /// The questions.
    pub rows: Vec<Row>,
}

impl QuestionSet {
    /// Read a set.
    ///
    /// # Errors
    ///
    /// The file cannot be read, or is not a `QuestionSet` in RON.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// The hash that freezes this set: its rows' texts and expectations, in
    /// file order. Adding a row changes it, and so does quietly editing one.
    pub fn digest(&self) -> String {
        let rows: Vec<String> = self
            .rows
            .iter()
            .map(|r| format!("{}\u{1f}{}", r.text, r.expect.join(",")))
            .collect();
        content_hash(rows.join("\u{1e}").as_bytes())
    }

    /// How many rows the owner has confirmed, and how many await review.
    pub fn counted(&self) -> (usize, usize) {
        let confirmed = self
            .rows
            .iter()
            .filter(|r| r.status == Status::Confirmed)
            .count();
        (confirmed, self.rows.len() - confirmed)
    }
}
