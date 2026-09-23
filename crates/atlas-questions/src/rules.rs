//! What a drafter cannot check by eye.
//!
//! Two rules, both mechanical. Every expected identifier must name a
//! resource that exists, and -- in the paraphrase set -- a question must not
//! contain an alias or the title of its own answer. The second is the one
//! that matters: it is the difference between measuring whether a system
//! understands a question and measuring whether it can find a string, and
//! no drafter holds every alias of 643 resources in mind.

use crate::{QuestionSet, Row, leakage};
use atlas_core::Resource;
use atlas_corpus::Corpus;

/// Everything wrong with this set, in the order a drafter should fix it.
///
/// An expectation that names no resource, and -- for the paraphrase set
/// -- a question containing an alias or the title of its own answer,
/// which would make it a lookup wearing a paraphrase's clothes.
pub fn problems(set: &QuestionSet, corpus: &Corpus) -> Vec<String> {
    let paraphrase = set.name == "paraphrase";
    let mut out = Vec::new();
    for row in &set.rows {
        for id in &row.expect {
            match corpus.resources.iter().find(|r| r.id.as_str() == id) {
                None => out.push(format!("{}: no such resource {id}", row.text)),
                Some(answer) if paraphrase => out.extend(given_away(row, answer)),
                Some(_) => {}
            }
        }
    }
    out
}

/// The answer's own words, where a paraphrase used them.
fn given_away(row: &Row, answer: &Resource) -> Option<String> {
    let asked = row.text.to_lowercase();
    let mut terms = answer.aliases.iter().chain(std::iter::once(&answer.title));
    let found = terms.find(|term| leakage::verbatim(&asked, term))?;
    Some(format!(
        "{}: contains its answer's own words: {found:?}",
        row.text
    ))
}
