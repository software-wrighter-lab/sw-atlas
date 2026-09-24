//! Indexing a corpus: which field of which resource lands in which bucket.
//!
//! | Weight | Campus field (MB01) | Atlas field (MB02) |
//! |---|---|---|
//! | 4 | concepts | concepts: the label and its aliases |
//! | 3 | aliases | aliases |
//! | 2 | title | title |
//! | 1.5 | tagline | summary: the authored one-line abstract |
//! | 1 | story and status text, MB01t only | body: a campus story, a video script |
//!
//! Two mappings and one extension are deviations from MB01, recorded because a
//! deviation nobody wrote down is how a ported number stops meaning what it
//! says. The campus `tagline` and the Atlas `summary` are both the authored
//! one-liner, so they share a weight. The campus's extra text was stories and
//! status sentences; Atlas's is `body`, which holds the same kind of thing. And
//! an Atlas concept carries aliases where a campus concept was one string, so
//! those aliases are indexed at 4 beside their label -- other spellings of one
//! idea, not new signals.

use crate::Signals;
use atlas_core::Concept;
use atlas_corpus::Corpus;
use std::collections::BTreeMap;

/// A resource's concepts, at weight 4: the label and every alias, because an
/// Atlas concept keeps its other spellings where a campus concept was one
/// string.
fn concepts_of(
    signals: &mut Signals,
    at: u32,
    ids: &[atlas_core::ConceptId],
    known: &BTreeMap<&str, &Concept>,
) {
    for concept in ids.iter().filter_map(|id| known.get(id.as_str())) {
        signals.add(at, &concept.label, 0);
        for alias in &concept.aliases {
            signals.add(at, alias, 0);
        }
    }
}

/// Index one resource's fields at their weights.
fn one(
    signals: &mut Signals,
    at: u32,
    resource: &atlas_core::Resource,
    known: &BTreeMap<&str, &Concept>,
    with_text: bool,
) {
    concepts_of(signals, at, &resource.concepts, known);
    for alias in &resource.aliases {
        signals.add(at, alias, 1);
    }
    signals.add(at, &resource.title, 2);
    signals.add(at, &resource.summary, 3);
    if with_text {
        signals.add(at, &resource.body, 4);
    }
}

/// Index every resource in the corpus.
///
/// `with_text` is the MB01t variant: it also reads `body`, which is where a
/// campus story and a video script live.
pub fn build(corpus: &Corpus, with_text: bool) -> Signals {
    let concepts: BTreeMap<&str, &Concept> =
        corpus.concepts.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut signals = Signals::default();
    for (position, resource) in corpus.resources.iter().enumerate() {
        let at = u32::try_from(position).unwrap_or(u32::MAX);
        signals.ids.push(resource.id.as_str().to_owned());
        one(&mut signals, at, resource, &concepts, with_text);
    }
    signals
}
