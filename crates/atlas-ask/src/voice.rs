//! Which resources a follow-up may refer to, and what to call an outcome.

use atlas_answer::Outcome;

/// What the follow-up questions may refer to.
///
/// A follow-up says "that", so it needs a referent. After a "not yet" the
/// nearest resources are the referent -- the wing the withheld exhibit belongs
/// to. After "nothing here" there is none, and the suggestions fall back to
/// what the corpus is about.
pub fn seed(reply: &atlas_answer::Reply, answer: &atlas_match::Answer) -> Vec<String> {
    match &reply.outcome {
        Outcome::NothingHere | Outcome::Rephrase => Vec::new(),
        Outcome::NotYet(_) => answer.hits.iter().take(2).map(|h| h.id.clone()).collect(),
        _ => reply.offer.clone(),
    }
}

/// The outcome's name for the evidence line and the measurement table.
pub fn label(outcome: &Outcome) -> &'static str {
    match outcome {
        Outcome::One => "ONE",
        Outcome::Several => "SEVERAL",
        Outcome::NotYet(_) => "NOT YET",
        Outcome::Rephrase => "REPHRASE",
        Outcome::NothingHere => "NOTHING HERE",
    }
}
