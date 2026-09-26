//! MB02: the deterministic matcher, over the whole corpus.
//!
//! `moe-microscope` measured this scorer over 9 campus places (MB01, and
//! MB01t which also reads the catalog's prose) and every trained docent it
//! built lost to it on held-out paraphrases -- 0.685 against 0.407. This crate
//! is that scorer over 642 artifacts. It is a port, not a new idea: the
//! weights, the tokenizer, the partial-match rule, the phrase bonus and the
//! intent rules are theirs, and every place this had to decide something they
//! did not face is written down as a deviation ([`signals`], and the
//! `deviations` note below).
//!
//! It is runtime class A0: no model, no training, nothing downloaded, and it
//! is never deleted. Every tier above it has to beat it to exist.
//!
//! ## Deviations from MB01, and why
//!
//! - **The open and exhibit bonuses.** MB01 adds 0.5 when a place is `open`
//!   and 0.5 when its kind is `exhibit`, both only if something already
//!   matched. Atlas has no `open`: the nearest honest reading of "you can go
//!   and look at this today" is `Maturity::Working` or `Finished`, and the
//!   nearest reading of `exhibit` is a `Demo`, the kind you can run. Both are
//!   mappings of a campus idea onto a corpus that has no campus, so both are
//!   deviations rather than ports.
//! - **Graph traversal is a separate arm.** The plan calls A0 "lexical and
//!   graph matching", but MB01 had no relations to traverse, so folding a
//!   neighbour boost into MB02 would make it a different scorer wearing the
//!   same name. [`graph`] is measured as its own row.

pub mod bonus;
pub mod graph;
pub mod intent;

use atlas_corpus::Corpus;
use atlas_decide::Intent;
use atlas_signals as signals;
use std::collections::BTreeMap;

/// Which variant of the matcher: the mockup's fields, or those plus the
/// catalog's prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    /// MB01's fields: concepts, aliases, title, the one-line summary.
    Fields,
    /// MB01t: the same, plus `body` at weight 1.
    Text,
}

/// One candidate the matcher proposes, and why.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    /// The resource.
    pub id: String,
    /// Its score.
    pub score: f32,
    /// The query words that fired, in query order, for the "why this?" panel
    /// and for a reranker that wants to know what the lexical evidence was.
    pub signals: Vec<String>,
    /// The heaviest single word weight that fired, which says what *kind* of
    /// evidence this is: 4 a concept, 3 an alias, 2 a title, 1.5 a summary, 1
    /// the body, and 0.8 of any of those for a partial word. A hit built only
    /// from body partials is a coincidence far more often than a hit on a
    /// concept, and the policy needs to tell those apart.
    pub best: f32,
}

/// What the matcher answers: an intent from the rules, and ranked candidates.
#[derive(Debug, Clone, PartialEq)]
pub struct Answer {
    /// The intent the keyword rules chose.
    pub intent: Intent,
    /// Candidates, best first. Empty means nothing matched at all, which is
    /// the only way this matcher can say it does not know.
    pub hits: Vec<Hit>,
}

/// The matcher: a signal index plus the little the scorer needs about each
/// resource.
pub struct Matcher {
    signals: signals::Signals,
    aliases: Vec<Vec<String>>,
    bonus: Vec<f32>,
}

impl Matcher {
    /// Build the index. Deterministic, and the only expensive step.
    pub fn new(corpus: &Corpus, variant: Variant) -> Self {
        Self {
            signals: signals::build(corpus, variant == Variant::Text),
            aliases: corpus.resources.iter().map(|r| r.aliases.clone()).collect(),
            bonus: corpus.resources.iter().map(bonus::of).collect(),
        }
    }

    /// How many signals the index holds: MB01 reported 284, MB01t 1010.
    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    /// Answer a query: the intent by rule, and the best `k` candidates.
    ///
    /// Ties break by identifier so two machines rank the same way, which
    /// matters because a paraphrase often scores several resources equally.
    pub fn answer(&self, query: &str, k: usize) -> Answer {
        let mut scores: BTreeMap<u32, (f32, Vec<String>, f32)> = BTreeMap::new();
        for word in signals::tokens(query) {
            for (resource, weight) in self.signals.weigh(&word) {
                let entry = scores.entry(resource).or_insert((0.0, Vec::new(), 0.0));
                entry.0 += weight;
                entry.1.push(word.clone());
                entry.2 = entry.2.max(weight);
            }
        }
        for at in bonus::all_named(&self.aliases, query) {
            scores.entry(at).or_insert((0.0, Vec::new(), 0.0));
        }
        let mut hits: Vec<Hit> = scores
            .into_iter()
            .map(|(at, (score, words, best))| self.finish(at, query, score, (words, best)))
            .filter(|hit| hit.score > 0.0)
            .collect();
        hits.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.id.cmp(&b.id)));
        hits.truncate(k);
        Answer {
            intent: intent::of(query),
            hits,
        }
    }

    /// One candidate's final score: its lexical sum, the bonuses it earns for
    /// having matched at all, and the verbatim-alias bonus.
    fn finish(&self, at: u32, query: &str, score: f32, words: (Vec<String>, f32)) -> Hit {
        let (mut signals, best) = words;
        let resource = at as usize;
        let mut total = score + self.bonus.get(resource).copied().unwrap_or_default();
        let mut strongest = best;
        let aliases = self
            .aliases
            .get(resource)
            .map(Vec::as_slice)
            .unwrap_or_default();
        if let Some(alias) = bonus::named(aliases, query) {
            total += 4.0;
            strongest = strongest.max(4.0);
            signals.push(format!("\"{alias}\""));
        }
        Hit {
            id: self.signals.ids.get(resource).cloned().unwrap_or_default(),
            score: total,
            signals,
            best: strongest,
        }
    }
}
