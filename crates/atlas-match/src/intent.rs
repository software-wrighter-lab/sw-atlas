//! What the visitor wants, by keyword rule.
//!
//! The campus mockup's rules in the mockup's order, ported as data rather
//! than as regular expressions buried in code, so that a reader can see the
//! whole of A0's understanding of language on one screen. It is not much, and
//! that is the point: this scored 0.463 to 0.481 on held-out paraphrases in
//! `moe-microscope`, and it is the number a trained head has to beat.
//!
//! Two things A0 cannot say, recorded because the gap is the argument for the
//! model rather than a defect to hide: `FindResource` and `Compare` are not in
//! these rules and cannot be, because no keyword distinguishes "where did you
//! write about X" from "take me to X", and nothing here parses two subjects
//! out of one sentence.

use atlas_decide::Intent;

/// The rules, in order. The first whose phrase appears as whole words wins.
const RULES: [(&str, Intent); 7] = [
    ("again|repeat|once more", Intent::Story),
    (
        "status|finished|done|complete|completed|working|in progress|progress|ready|how far|wip|work in progress|planned|what works",
        Intent::Status,
    ),
    (
        "story|stories|anecdote|tell me about|history of|background",
        Intent::Story,
    ),
    (
        "where am i|what is here|whats here|around here|nearby",
        Intent::Explain,
    ),
    (
        "where|take me|go to|show me|find|get to|how do i get",
        Intent::Navigate,
    ),
    (
        "recommend|suggest|interested|i like|something|worth seeing|what should",
        Intent::Recommend,
    ),
    (
        "what is|whats|explain|about|why|how does|what did",
        Intent::Explain,
    ),
];

/// Whether any of the pipe-separated phrases appears in the query as whole
/// words, on the mockup's normalization: lowercase, punctuation to spaces.
fn has_phrase(query: &str, phrases: &str) -> bool {
    let cleaned: String = query
        .chars()
        .map(|c| {
            let lower = c.to_ascii_lowercase();
            if lower.is_ascii_alphanumeric() {
                lower
            } else {
                ' '
            }
        })
        .collect();
    let padded = format!(
        " {} ",
        cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
    );
    phrases
        .split('|')
        .any(|phrase| padded.contains(&format!(" {phrase} ")))
}

/// The intent a query's words declare. `Navigate` is the default, as in the
/// mockup: a visitor who names a thing and nothing else wants to be taken to
/// it.
pub fn of(query: &str) -> Intent {
    RULES
        .iter()
        .find(|(phrases, _)| has_phrase(query, phrases))
        .map_or(Intent::Navigate, |(_, intent)| *intent)
}
