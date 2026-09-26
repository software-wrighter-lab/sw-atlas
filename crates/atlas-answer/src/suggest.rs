//! What a visitor could usefully ask next, generated from the catalog.
//!
//! Every suggestion is a question the corpus can answer, built from something
//! it already holds: a concept and how many resources carry it, the next part
//! of a series, the video or demo a post declares. Nothing is invented, which
//! is why a suggestion can be trusted to lead somewhere.

use atlas_core::{RelationKind, ResourceKind};
use atlas_corpus::Corpus;

/// Follow-ups for the resources in hand, and failing that the concepts those
/// resources are about.
///
/// `seed` is what the docent offered, or -- when it offered nothing, as after a
/// "not yet" -- the nearest resources it found anyway. A visitor who asked
/// about a withheld exhibit is better served by questions about its wing than
/// by the corpus's commonest subjects.
pub fn next_questions(corpus: &Corpus, seed: &[String], want: usize) -> Vec<String> {
    let mut out = Vec::new();
    for id in seed {
        out.extend(relations_of(corpus, id));
        if out.len() >= want {
            break;
        }
    }
    if out.len() < want {
        out.extend(nearby_concepts(corpus, seed, want - out.len()));
    }
    out.truncate(want);
    out
}

/// The questions a resource's own declared relations support.
fn relations_of(corpus: &Corpus, id: &str) -> Vec<String> {
    let mut out = Vec::new();
    for edge in corpus.relations.iter().filter(|e| e.from.as_str() == id) {
        let target = corpus.resources.iter().find(|r| r.id == edge.to);
        let Some(target) = target else { continue };
        match (edge.kind, target.kind) {
            (RelationKind::Demos, ResourceKind::Video) => {
                out.push("is there a video of that?".to_string());
            }
            (RelationKind::Demos, ResourceKind::Demo) => {
                out.push("can I run it in the browser?".to_string());
            }
            (RelationKind::Implements, _) => out.push("where is the code?".to_string()),
            (RelationKind::SeriesNext, _) => {
                out.push("what comes next in that series?".to_string())
            }
            _ => {}
        }
    }
    out.sort();
    out.dedup();
    out
}

/// The concepts the resources in hand are about, busiest first; failing that,
/// the corpus's own busiest, which is the last honest fallback.
fn nearby_concepts(corpus: &Corpus, seed: &[String], want: usize) -> Vec<String> {
    let near: Vec<&atlas_core::ConceptId> = corpus
        .resources
        .iter()
        .filter(|r| seed.iter().any(|id| id == r.id.as_str()))
        .flat_map(|r| r.concepts.iter())
        .collect();
    let mut ranked: Vec<(usize, &str)> = corpus
        .concepts
        .iter()
        .filter(|concept| near.is_empty() || near.contains(&&concept.id))
        .map(|concept| (concept.resources.len(), concept.label.as_str()))
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
    ranked
        .iter()
        .take(want)
        .map(|(count, label)| format!("what do you have about {label}? ({count} things)"))
        .collect()
}
