//! The concept graph: one vocabulary over four sources.
//!
//! The ingesters emit a provisional concept for every label their source
//! wrote, so the same idea arrives spelled several ways -- the blog writes
//! `chain-of-thought`, the video index writes `Chain of Thought`. This
//! crate unifies the spellings mechanically ([`normalize`]), applies the
//! judgements a person committed ([`overrides`]), rewrites every resource's
//! concept references to the surviving identifiers, fills in the reverse
//! index, and writes the report that makes all of it auditable
//! ([`report`]).
//!
//! Concepts are the model's output vocabulary, which is why this is worth a
//! step of its own: a model asked to tell `moe` from `mixture of experts`
//! is being asked to distinguish two spellings of one thing.

pub mod normalize;
pub mod overrides;
pub mod report;

use atlas_core::{Concept, ConceptId};
use atlas_corpus::Corpus;
use overrides::Overrides;
use std::collections::{BTreeMap, BTreeSet};

/// Every label the corpus uses, with how many resources use it.
fn spellings(corpus: &Corpus) -> Vec<(String, usize)> {
    let uses = |id: &ConceptId| {
        corpus
            .resources
            .iter()
            .filter(|r| r.concepts.contains(id))
            .count()
    };
    let mut seen: Vec<(String, usize)> = corpus
        .concepts
        .iter()
        .map(|concept| (concept.label.clone(), uses(&concept.id)))
        .collect();
    seen.sort();
    seen.dedup();
    seen
}

/// The surviving concepts, and where every old identifier goes.
///
/// A group's key survives unless the override file folds it into another,
/// in which case its spellings become aliases of the survivor.
fn plan(
    groups: &[normalize::Group],
    ov: &Overrides,
) -> (Vec<Concept>, BTreeMap<ConceptId, ConceptId>) {
    let mut concepts: BTreeMap<String, Concept> = BTreeMap::new();
    let mut moves: BTreeMap<ConceptId, ConceptId> = BTreeMap::new();
    for group in groups {
        let key = ov.canonical(&group.key);
        let entry = concepts
            .entry(key.clone())
            .or_insert_with(|| record(&key, group, ov));
        for (spelling, _) in &group.spellings {
            entry.aliases.push(spelling.to_lowercase());
            moves.insert(
                ConceptId::new(spelling.to_lowercase()),
                ConceptId::new(&key),
            );
        }
        entry.aliases.retain(|alias| *alias != key);
        entry.aliases.sort();
        entry.aliases.dedup();
    }
    (concepts.into_values().collect(), moves)
}

/// The concept a surviving key becomes, before its aliases and resources
/// are filled in.
fn record(key: &str, group: &normalize::Group, ov: &Overrides) -> Concept {
    let parents = ov
        .parents
        .get(key)
        .map(|ps| ps.iter().map(ConceptId::new).collect());
    Concept {
        id: ConceptId::new(key),
        label: ov
            .labels
            .get(key)
            .cloned()
            .unwrap_or_else(|| group.label.clone()),
        aliases: Vec::new(),
        parents: parents.unwrap_or_default(),
        resources: Vec::new(),
    }
}

/// Unify the corpora into one with a single concept vocabulary.
///
/// Returns the corpus, the collision report, and any key the override file
/// names that no source wrote -- a typo or a leftover correction, which the
/// caller should refuse to build on rather than ignore.
pub fn build(corpora: &[Corpus], ov: &Overrides) -> (Corpus, String, Vec<String>) {
    let mut corpus = Corpus::merge(corpora);
    let groups = normalize::groups(&spellings(&corpus), &ov.keep_apart);
    let keys: BTreeSet<String> = groups.iter().map(|g| g.key.clone()).collect();
    let text = report::markdown(&groups, &report::near_misses(&keys), ov);
    let (mut concepts, moves) = plan(&groups, ov);
    for resource in &mut corpus.resources {
        let mapped = resource
            .concepts
            .iter()
            .map(|id| moves.get(id).unwrap_or(id).clone());
        resource.concepts = mapped.collect();
        resource.concepts.sort();
        resource.concepts.dedup();
    }
    for concept in &mut concepts {
        let holders = corpus
            .resources
            .iter()
            .filter(|r| r.concepts.contains(&concept.id));
        concept.resources = holders.map(|r| r.id.clone()).collect();
    }
    corpus.concepts = concepts;
    (corpus, text, ov.unknown(&keys))
}
