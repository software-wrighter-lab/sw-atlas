//! The signal index: every word of every resource, in the bucket that says
//! how much it counts for.
//!
//! Its own crate because `atlas-match` was at four modules when the index
//! needed a fifth concern, and the house rule is to make the sibling crate
//! first. It knows about words and weights; it knows nothing about scoring a
//! query, which is the matcher's job.
//!
//! Ported from `moe-microscope`'s `lib/matcher.mlpl`, and the weights are
//! theirs unchanged, because MB02 is comparable to MB01 only if it is the same
//! scorer over a bigger corpus:
//!
//! | Weight | Campus field (MB01) | Atlas field (MB02) |
//! |---|---|---|
//! | 4 | concepts | concepts: the label and its aliases |
//! | 3 | aliases | aliases |
//! | 2 | title | title |
//! | 1.5 | tagline | summary: the authored one-line abstract |
//! | 1 | story and status text, MB01t only | body: a campus story, a video script |
//!
//! Two mappings and one extension are deviations, recorded because a deviation
//! nobody wrote down is how a ported number stops meaning what it says. The
//! campus `tagline` and the Atlas `summary` are both the authored one-liner, so
//! they share a weight. The campus's extra text was stories and status
//! sentences; Atlas's is `body`, which holds the same kind of thing. And an
//! Atlas concept carries aliases where a campus concept was one string, so
//! those aliases are indexed at 4 beside their label -- other spellings of one
//! idea, not new signals.
//!
//! The *shape* is not theirs. MB01 scored nine places by walking each place's
//! buckets per query word, which is fine for nine and was 19 ms for 642. This
//! inverts the index -- word to the resources carrying it -- and computes the
//! same score, because A0 has a 10 ms budget and a matcher that misses it is
//! not the matcher that gets to be the permanent tier.

pub mod build;
pub mod tokens;

pub use build::build;
pub use tokens::tokens;

use std::collections::{BTreeMap, BTreeSet};

/// The five weights, heaviest first. A token's bucket is its weight.
pub const WEIGHTS: [f32; 5] = [4.0, 3.0, 2.0, 1.5, 1.0];

/// Word to the resources that carry it, each with its heaviest bucket.
pub type Owners = BTreeMap<String, Vec<(u32, u8)>>;

/// The index: which resources carry which words, and how heavily.
#[derive(Debug, Default)]
pub struct Signals {
    /// Resource identifiers; a resource is its position here.
    pub ids: Vec<String>,
    /// Every token, to its owners.
    pub(crate) exact: Owners,
    /// Tokens of five characters or more only, to their owners: a partial
    /// match may fire on these and on nothing else.
    pub(crate) long: Owners,
}

impl Signals {
    /// Record every word of a text as belonging to one resource at one weight.
    pub fn add(&mut self, resource: u32, text: &str, bucket: u8) {
        for token in tokens(text) {
            let long = token.chars().count() >= 5;
            for map in [Some(&mut self.exact), long.then_some(&mut self.long)]
                .into_iter()
                .flatten()
            {
                let owners = map.entry(token.clone()).or_default();
                match owners.iter_mut().find(|(r, _)| *r == resource) {
                    Some(seen) if seen.1 <= bucket => {}
                    Some(seen) => seen.1 = bucket,
                    None => owners.push((resource, bucket)),
                }
            }
        }
    }

    /// The weight each resource gives one query word: its heaviest exact
    /// bucket, or 0.8 of its heaviest partial one, never both.
    ///
    /// "Never both" is the part worth stating. MB01 asks each place for an
    /// exact hit first and only falls back to a partial one for that place, so
    /// a resource holding the word outright must not also collect a partial
    /// score from some longer token it happens to contain. Inverting the index
    /// makes that easy to get wrong -- it was wrong here first, and the parity
    /// test against a plain walk is what caught it.
    pub fn weigh(&self, word: &str) -> BTreeMap<u32, f32> {
        let mut out: BTreeMap<u32, f32> = self
            .exact
            .get(word)
            .into_iter()
            .flatten()
            .map(|(r, b)| (*r, WEIGHTS[*b as usize]))
            .collect();
        if word.chars().count() < 5 {
            return out;
        }
        let settled: BTreeSet<u32> = out.keys().copied().collect();
        for (_, owners) in self.long.iter().filter(|(t, _)| tokens::overlaps(t, word)) {
            for (resource, bucket) in owners.iter().filter(|(r, _)| !settled.contains(r)) {
                let partial = WEIGHTS[*bucket as usize] * 0.8;
                let entry = out.entry(*resource).or_insert(0.0);
                if *entry < partial {
                    *entry = partial;
                }
            }
        }
        out
    }

    /// How many distinct signals the index holds, which is what MB01 reported
    /// as its stored size: 284 signals for the campus, 1010 for its text
    /// variant.
    pub fn len(&self) -> usize {
        self.exact.values().map(Vec::len).sum()
    }

    /// Whether the index is empty, because clippy asks whenever `len` exists.
    pub fn is_empty(&self) -> bool {
        self.exact.is_empty()
    }
}
