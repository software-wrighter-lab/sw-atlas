//! Which of the matcher's hits count as evidence for an answer.
//!
//! Separate from the policy on purpose. The matcher is a frozen yardstick: it
//! must keep scoring the way MB01 scored, mistakes included, or every number
//! measured against it stops meaning anything. The policy decides what to say.
//! In between there has to be somewhere to decide which of the matcher's hits
//! are worth saying anything about at all, and this is it.

use atlas_match::{Answer, Hit};

/// The hits worth acting on: above the floor, and not a coincidence.
pub fn strong<'a>(answer: &'a Answer, query: &str, floor: f32) -> Vec<&'a Hit> {
    answer
        .hits
        .iter()
        .filter(|hit| hit.score >= floor && !coincidence(hit, query))
        .collect()
}

/// Whether a hit rests on nothing but an alias that appears inside a longer
/// word.
///
/// MB01's verbatim-alias bonus is a substring test, so "flibbertigibbet"
/// contains "bert" and scores 4 against a post about BERT. The matcher keeps
/// that behaviour, because MB02 is comparable to MB01 only while it makes the
/// same mistakes. Declining to *show* it is a different question from how to
/// score it, and this is where that question is answered.
pub fn coincidence(hit: &Hit, query: &str) -> bool {
    let only = hit.signals.len() == 1;
    let quoted = hit
        .signals
        .first()
        .and_then(|s| s.strip_prefix('"'))
        .and_then(|s| s.strip_suffix('"'));
    match (only, quoted) {
        (true, Some(alias)) => !atlas_questions::leakage::verbatim(query, alias),
        _ => false,
    }
}

/// Whether the query's words are in the index at all, coincidences aside.
///
/// The difference between "I do not recognise those words" and "I know those
/// words and have nothing about them", which are different apologies and
/// deserve different sentences.
pub fn recognised(answer: &Answer, query: &str) -> bool {
    answer
        .hits
        .iter()
        .any(|hit| !coincidence(hit, query) || hit.signals.len() > 1)
}

/// What fired, for a trace panel: the numbers a reader can check against the
/// thresholds.
pub fn because(top: &Hit, above: usize, margin: f32) -> String {
    let (score, best) = (top.score, top.best);
    format!("top {score:.1}, margin {margin:.1}, field {best:.1}, {above} above the floor")
}
