//! The decision: which outcome the evidence supports.

use crate::frames;
use crate::{Absent, Policy};
use atlas_corpus::Corpus;
use atlas_evidence as evidence;
use atlas_match::{Answer, Hit};

/// What the docent decided to do.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// The evidence is decisive: one resource, named.
    One,
    /// The usual case: the closest few, offered as a choice.
    Several,
    /// The subject is real and not in the index yet.
    NotYet(String),
    /// Nothing of the question is in the index, so it may be rephrased.
    Rephrase,
    /// The words are known and nothing they point at is here.
    NothingHere,
}

/// The whole reply: what was decided, which resources it names, and the
/// evidence a trace panel shows.
#[derive(Debug, Clone)]
pub struct Reply {
    /// The decision.
    pub outcome: Outcome,
    /// Resource identifiers, in the order a visitor should see them.
    pub offer: Vec<String>,
    /// Why, in the policy's own terms.
    pub because: String,
}

/// Decide what to say about one question.
///
/// The order matters and is deliberate: a knowingly absent subject is
/// recognised before the matcher's near-misses get a chance to look like an
/// answer, because "not yet" is more useful than a route to something adjacent.
pub fn decide(query: &str, answer: &Answer, corpus: &Corpus, rules: (&Policy, &Absent)) -> Reply {
    let (policy, absent) = rules;
    if let Some(reply) = absent.reply(query) {
        return reply;
    }
    let strong = evidence::strong(answer, query, policy.floor);
    // Nothing above the floor, or evidence so weak it is a coincidence anyway:
    // "the weather tomorrow" matches "weather" in a video script and scores 2.
    let Some(top) = strong
        .first()
        .filter(|top| top.best > policy.weak_best || top.score >= policy.weak_top)
    else {
        return thin(query, answer);
    };
    let margin = top.score - strong.get(1).map_or(0.0, |hit| hit.score);
    let (outcome, offer) = settle(&strong, corpus, (policy, margin));
    let because = evidence::because(top, strong.len(), margin);
    Reply {
        outcome,
        offer,
        because,
    }
}

/// How many of the strong hits to offer, and under which outcome.
///
/// A decisive margin offers one; anything else offers the configured few. An
/// unfinished top hit overrides both, because "here it is" about a thing that
/// does not run yet is the one answer a visitor cannot forgive.
fn settle(strong: &[&Hit], corpus: &Corpus, rules: (&Policy, f32)) -> (Outcome, Vec<String>) {
    let (policy, margin) = rules;
    let decisive = margin >= policy.one_margin;
    let offer: Vec<String> = strong
        .iter()
        .take(if decisive { 1 } else { policy.several })
        .map(|hit| hit.id.clone())
        .collect();
    let pending = strong
        .first()
        .and_then(|top| frames::unfinished(corpus, top.id.as_str()));
    match (decisive, pending) {
        (_, Some(text)) => (Outcome::NotYet(text), offer),
        (true, None) => (Outcome::One, offer),
        (false, None) => (Outcome::Several, offer),
    }
}

/// Nothing survived the floor: either the words are unknown to the index, or
/// they are known and lead nowhere.
fn thin(query: &str, answer: &Answer) -> Reply {
    // No word of the question is in the index at all: the visitor may be using
    // vocabulary the corpus does not have, so ask for another phrasing. A hit
    // that rests on an alias buried inside a longer word is not knowledge of
    // the words either -- "flibbertigibbet" is not a question about BERT -- so
    // it counts as nothing. If the words are genuinely known and only lead to
    // prose coincidences, the honest answer is that nothing here is about it.
    let known = evidence::recognised(answer, query);
    let outcome = if known {
        Outcome::NothingHere
    } else {
        Outcome::Rephrase
    };
    Reply {
        outcome,
        offer: Vec::new(),
        because: match answer.hits.first().filter(|_| known) {
            None => "no word of the question is in the index".into(),
            Some(hit) => format!("nothing above the floor; best was {:.1}", hit.score),
        },
    }
}
