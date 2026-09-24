//! One hop along the relations, as its own arm.
//!
//! The plan calls A0 "lexical and graph matching", but MB01 had no relations
//! to traverse, so this is measured separately and never folded into the MB02
//! row: a scorer with a neighbour boost is a different scorer, and giving it
//! the same name is how a ported number quietly stops being comparable.
//!
//! The move it makes is the one the corpus is for. A visitor asks about a
//! post; the post declares the repository that implements it and the video
//! that shows it; neither of those shares many words with the question, and
//! both are what the visitor wanted. So a resource inherits a fraction of the
//! best score among its declared neighbours.

use crate::Hit;
use atlas_corpus::Corpus;

/// Add a neighbour's evidence at `factor` of its score.
///
/// Only one hop, and only `Declared` edges: a derived or teacher-guessed edge
/// would put a model's opinion inside the deterministic baseline, which is
/// exactly what the baseline exists to be free of.
pub fn spread(corpus: &Corpus, hits: &[Hit], factor: f32) -> Vec<Hit> {
    let mut scores: Vec<Hit> = hits.to_vec();
    for hit in hits {
        for other in neighbours(corpus, &hit.id) {
            let inherited = hit.score * factor;
            match scores.iter_mut().find(|h| h.id == other) {
                Some(existing) if existing.score >= inherited => {}
                Some(existing) => existing.score = inherited,
                None => scores.push(Hit {
                    id: other,
                    score: inherited,
                    signals: vec![format!("via {}", hit.id)],
                }),
            }
        }
    }
    scores.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.id.cmp(&b.id)));
    scores
}

/// Every resource one declared edge away.
fn neighbours(corpus: &Corpus, id: &str) -> Vec<String> {
    corpus
        .relations
        .iter()
        .filter(|e| e.provenance == atlas_core::Provenance::Declared)
        .filter_map(|e| match (e.from.as_str(), e.to.as_str()) {
            (from, to) if from == id => Some(to.to_owned()),
            (from, to) if to == id => Some(from.to_owned()),
            _ => None,
        })
        .collect()
}
