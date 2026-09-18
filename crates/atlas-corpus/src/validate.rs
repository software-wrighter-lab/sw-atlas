//! Cross-reference validation.
//!
//! A corpus that names something it does not contain is a corpus that will
//! hand a visitor a dead link, so this runs in a test and in the gate
//! rather than as an optional lint.

use crate::corpus::Corpus;
use atlas_core::{ConceptId, ResourceId};
use std::collections::BTreeSet;

/// One thing wrong with a corpus, named precisely enough to fix.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Problem {
    /// A relation endpoint names a resource that is not in the corpus.
    UnknownResource {
        /// Where the dangling reference was found.
        context: String,
        /// The identifier that did not resolve.
        id: String,
    },
    /// A resource or concept names a concept that is not in the corpus.
    UnknownConcept {
        /// Where the dangling reference was found.
        context: String,
        /// The identifier that did not resolve.
        id: String,
    },
    /// Two resources or two concepts share an identifier.
    Duplicate {
        /// The identifier that appears more than once.
        id: String,
    },
}

/// Check every identifier the corpus mentions against what it contains.
///
/// Returns every problem found, sorted, rather than the first: a report a
/// human can work through beats an exception they have to rerun for.
pub fn validate(corpus: &Corpus) -> Vec<Problem> {
    let resources = corpus.resource_ids();
    let concepts = corpus.concept_ids();
    let mut found = duplicates(corpus);
    found.extend(dangling_resources(corpus, &resources));
    found.extend(dangling_concepts(corpus, &concepts));
    found.sort();
    found.dedup();
    found
}

/// Identifiers used more than once.
fn duplicates(corpus: &Corpus) -> Vec<Problem> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for id in corpus.resources.iter().map(|r| r.id.as_str()) {
        if !seen.insert(id) {
            out.push(Problem::Duplicate { id: id.to_string() });
        }
    }
    let mut seen = BTreeSet::new();
    for id in corpus.concepts.iter().map(|c| c.id.as_str()) {
        if !seen.insert(id) {
            out.push(Problem::Duplicate { id: id.to_string() });
        }
    }
    out
}

/// Relation endpoints and concept back-references that name no resource.
fn dangling_resources(corpus: &Corpus, known: &BTreeSet<&ResourceId>) -> Vec<Problem> {
    let mut refs: Vec<(String, &ResourceId)> = Vec::new();
    for relation in &corpus.relations {
        let at = format!(
            "relation {:?} from {}",
            relation.kind,
            relation.from.as_str()
        );
        refs.push((at.clone(), &relation.from));
        refs.push((at, &relation.to));
    }
    for concept in &corpus.concepts {
        let at = format!("concept {}", concept.id.as_str());
        refs.extend(concept.resources.iter().map(|id| (at.clone(), id)));
    }
    refs.into_iter()
        .filter(|(_, id)| !known.contains(id))
        .map(|(context, id)| Problem::UnknownResource {
            context,
            id: id.as_str().to_string(),
        })
        .collect()
}

/// Concept references on resources, and concept parents, that name no
/// concept.
fn dangling_concepts(corpus: &Corpus, known: &BTreeSet<&ConceptId>) -> Vec<Problem> {
    let mut refs: Vec<(String, &ConceptId)> = Vec::new();
    for resource in &corpus.resources {
        let at = format!("resource {}", resource.id.as_str());
        refs.extend(resource.concepts.iter().map(|id| (at.clone(), id)));
    }
    for concept in &corpus.concepts {
        let at = format!("parent of {}", concept.id.as_str());
        refs.extend(concept.parents.iter().map(|id| (at.clone(), id)));
    }
    refs.into_iter()
        .filter(|(_, id)| !known.contains(id))
        .map(|(context, id)| Problem::UnknownConcept {
            context,
            id: id.as_str().to_string(),
        })
        .collect()
}
