//! What the docent says, and why it is allowed to say it.
//!
//! A0 plus ordinary code, no model. The policy turns the matcher's evidence
//! into one of five outcomes, and every string a visitor sees is a frame with
//! catalog text quoted into it -- nothing is composed, which is the same
//! guarantee the model tier will inherit.
//!
//! The shape of the policy is a measurement, not a taste. Fitted on the 397
//! confirmed questions that name a destination: at no combination of score and
//! margin does answering with a single resource beat 0.54 precision, and only a
//! margin of 8 or more reaches 0.86 -- on 2% of questions. So the default
//! outcome is a short list of the closest matches, which is right 0.435 of the
//! time at three and honest about being a list. A docent that confidently named
//! one resource would be wrong about half the time, and a visitor cannot tell
//! which half.

pub mod frames;
pub mod policy;
pub mod suggest;

pub use policy::{Outcome, Reply, decide};

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// The thresholds, from `sources/answer-policy.ron` with their measurements.
#[derive(Debug, Clone, Deserialize)]
pub struct Policy {
    /// A margin this large is decisive: precision 0.86 on 2% of questions.
    pub one_margin: f32,
    /// How many candidates to offer otherwise.
    pub several: usize,
    /// The strongest field that fired, below which the evidence is prose.
    pub weak_best: f32,
    /// The total score below which weak evidence means "rephrase".
    pub weak_top: f32,
    /// Below this a hit is noise.
    pub floor: f32,
}

/// Subjects the corpus knowingly does not hold, and what to say about each.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Absent {
    /// Words to watch for, and the sentence to quote when they appear.
    pub subjects: BTreeMap<String, String>,
}

impl Policy {
    /// Read the fitted thresholds.
    ///
    /// # Errors
    ///
    /// The file cannot be read or is not a `Policy` in RON.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }
}

impl Absent {
    /// Read the list of knowingly absent subjects.
    ///
    /// # Errors
    ///
    /// The file cannot be read or is not an `Absent` in RON.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// The sentence to quote when the query names an absent subject.
    /// The whole reply for a subject on the list, or nothing.
    pub fn reply(&self, query: &str) -> Option<policy::Reply> {
        Some(policy::Reply {
            outcome: policy::Outcome::NotYet(self.covering(query)?.clone()),
            offer: Vec::new(),
            because: "a subject the corpus knowingly does not hold yet".into(),
        })
    }

    /// The sentence for the first listed subject the query names.
    pub fn covering(&self, query: &str) -> Option<&String> {
        let asked = query.to_lowercase();
        self.subjects
            .iter()
            .find(|(subject, _)| atlas_questions::leakage::verbatim(&asked, subject))
            .map(|(_, reply)| reply)
    }
}
