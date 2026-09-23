//! Whether a frozen question has leaked into training.
//!
//! Two ways to leak, and both are checked. The obvious one is the same
//! question appearing in both places, caught by hashing the text. The one
//! that actually happens is a generated training row that reuses most of a
//! held-out question's wording, which no hash catches: so any eight-word
//! run shared between a frozen row and a training row is a leak too.
//!
//! Eight words is a judgement, and worth stating. Shorter runs fire on
//! ordinary English -- "where can i find the post about the" is nobody's
//! leak -- and longer ones miss a paraphrase that was lightly edited.

use std::collections::BTreeSet;

/// The words of a string, lowercased, punctuation dropped.
fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

/// Whether a term appears in a text as whole words rather than as a
/// fragment inside a longer word: `apl` is in "apl interpreter" and not in
/// "applied".
pub fn verbatim(text: &str, term: &str) -> bool {
    let (haystack, needle) = (words(text), words(term));
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|run| run == needle.as_slice())
}

/// Every run of `len` words in a text.
fn runs(text: &str, len: usize) -> BTreeSet<String> {
    let words = words(text);
    if words.len() < len {
        return BTreeSet::new();
    }
    words.windows(len).map(|run| run.join(" ")).collect()
}

/// Frozen questions that leaked into the training rows, and how.
///
/// `frozen` and `training` are question texts. A pair is reported once,
/// with the run that gave it away, so a person can see what to delete.
pub fn leaks(frozen: &[String], training: &[String], len: usize) -> Vec<String> {
    let seen: Vec<(String, BTreeSet<String>)> = training
        .iter()
        .map(|row| (row.clone(), runs(row, len)))
        .collect();
    let mut out = Vec::new();
    for question in frozen {
        let asked = runs(question, len);
        for (row, theirs) in &seen {
            if row.eq_ignore_ascii_case(question) {
                out.push(format!("{question:?} is also a training row"));
                break;
            }
            if let Some(run) = asked.intersection(theirs).next() {
                out.push(format!("{question:?} shares {run:?} with {row:?}"));
                break;
            }
        }
    }
    out
}
