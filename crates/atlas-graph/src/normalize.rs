//! Turning many spellings of one idea into one concept, mechanically.
//!
//! Two rules, both reversible by reading the report:
//!
//! 1. **Spelling.** Lowercase, and treat `-`, `_` and `/` as word breaks,
//!    so `chain-of-thought`, `chain of thought` and `Chain of Thought` are
//!    one concept. The blog writes tags hyphenated and the video index
//!    writes them spaced, so this rule alone does most of the work.
//! 2. **Plurals, only in pairs.** `agents` folds into `agent` when *both*
//!    spellings occur in the corpus. Nothing is stemmed: a rule that
//!    rewrote every trailing `s` would merge words that merely look alike,
//!    and no reader could tell which merges were real.
//!
//! Everything else -- an acronym and its expansion, a typo, a narrower
//! idea inside a broader one -- is a judgement, and judgements live in
//! `sources/concept-overrides.ron`.

use std::collections::BTreeMap;

/// One concept the normalizer formed, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// The identifier: the normalized key.
    pub key: String,
    /// The spelling to show a visitor.
    pub label: String,
    /// Every other spelling seen, lowercased.
    pub aliases: Vec<String>,
    /// How many resources used each spelling, most used first.
    pub spellings: Vec<(String, usize)>,
}

/// The identifier a label normalizes to: lowercase, words joined by
/// hyphens, as `plan.md` section 7 spells a `ConceptId`.
pub fn key(label: &str) -> String {
    let broken: String = label
        .to_lowercase()
        .chars()
        .map(|c| if "-_/".contains(c) { ' ' } else { c })
        .collect();
    broken.split_whitespace().collect::<Vec<_>>().join("-")
}

/// Plural keys that fold into a singular key present in the same corpus,
/// except the pairs the override file keeps apart.
fn plurals(
    keys: &BTreeMap<String, Vec<(String, usize)>>,
    keep_apart: &[(String, String)],
) -> BTreeMap<String, String> {
    keys.keys()
        .filter_map(|plural| {
            let singular = plural.strip_suffix('s')?;
            let apart = keep_apart
                .iter()
                .any(|(a, b)| (a == plural && b == singular) || (b == plural && a == singular));
            let known = keys.contains_key(singular) && !singular.ends_with('s') && !apart;
            known.then(|| (plural.clone(), singular.to_string()))
        })
        .collect()
}

/// One concept from every spelling that normalized to its key.
///
/// The label is the string a page prints, so prose beats a slug and the
/// singular beats the plural: of `chain-of-thought` and `Chain of Thought`
/// the second is the label, and the concept `compiler` is not shown as
/// `compilers` merely because more posts are tagged that way. Ties break by
/// use, then length, then alphabetically, so every machine agrees.
fn group_of(key: String, mut spellings: Vec<(String, usize)>) -> Group {
    spellings.sort_by(|a, b| (b.1, a.0.clone()).cmp(&(a.1, b.0.clone())));
    let marks = |text: &str| text.chars().filter(|c| "-_/".contains(*c)).count();
    let best = spellings.iter().max_by_key(|(text, count)| {
        (
            usize::from(self::key(text) == key),
            std::cmp::Reverse(marks(text)),
            *count,
            std::cmp::Reverse(text.len()),
            std::cmp::Reverse(text.clone()),
        )
    });
    let label = best.map(|(text, _)| text.clone()).unwrap_or_default();
    let aliases = spellings
        .iter()
        .map(|(text, _)| text.to_lowercase())
        .filter(|text| *text != key)
        .collect();
    Group {
        key,
        label,
        aliases,
        spellings,
    }
}

/// Group every spelling seen in the corpus into concepts.
///
/// `seen` is every label as some source wrote it, with how many resources
/// wrote it that way.
pub fn groups(seen: &[(String, usize)], keep_apart: &[(String, String)]) -> Vec<Group> {
    let mut by_key: BTreeMap<String, Vec<(String, usize)>> = BTreeMap::new();
    for (label, count) in seen {
        by_key
            .entry(key(label))
            .or_default()
            .push((label.clone(), *count));
    }
    for (plural, singular) in &plurals(&by_key, keep_apart) {
        let moved = by_key.remove(plural).unwrap_or_default();
        by_key.entry(singular.clone()).or_default().extend(moved);
    }
    by_key.into_iter().map(|(k, s)| group_of(k, s)).collect()
}
