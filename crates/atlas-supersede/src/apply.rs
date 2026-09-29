//! Make one piece of work one resource.
//!
//! Dropping the superseded copy where the cache is read is only half of it: a
//! blog post that linked to the old repository creates the resource again, so
//! after the merge the corpus holds both copies and the docent can still send
//! a visitor to the abandoned one. This pass runs over the merged corpus and
//! finishes the job -- the superseded resource goes, every relation and every
//! concept membership that named it now names the live copy, and the old name
//! becomes an alias so a question about it still arrives somewhere.
//!
//! Repointing rather than deleting matters: the post really does link there,
//! and the visitor really does want what it links to. What they do not want is
//! the copy nobody pushes to.

use crate::Canonical;
use atlas_core::ResourceId;
use atlas_corpus::Corpus;
use std::collections::BTreeMap;

/// Apply the declarations to a merged corpus, returning how many resources
/// were folded away.
pub fn supersede(corpus: &mut Corpus, declared: &Canonical) -> usize {
    let map = renames(declared);
    let before = corpus.resources.len();
    corpus.resources.retain(|r| !map.contains_key(&r.id));
    for resource in &mut corpus.resources {
        resource.aliases.extend(aliases_for(declared, &resource.id));
        resource.aliases.sort();
        resource.aliases.dedup();
    }
    repoint(corpus, &map);
    before - corpus.resources.len()
}

/// Every reference to a superseded resource now names the live one: the
/// relations a post declared, and the concepts both copies were listed under.
/// A relation that folds onto itself is dropped -- a post cannot discuss a
/// resource by being it.
fn repoint(corpus: &mut Corpus, map: &BTreeMap<ResourceId, ResourceId>) {
    let swap = |id: &ResourceId| map.get(id).cloned().unwrap_or_else(|| id.clone());
    for relation in &mut corpus.relations {
        relation.from = swap(&relation.from);
        relation.to = swap(&relation.to);
    }
    corpus.relations.retain(|r| r.from != r.to);
    for concept in &mut corpus.concepts {
        for id in &mut concept.resources {
            *id = swap(id);
        }
        concept.resources.sort();
        concept.resources.dedup();
    }
}

/// Superseded resource identifier to canonical resource identifier.
fn renames(declared: &Canonical) -> BTreeMap<ResourceId, ResourceId> {
    declared
        .pairs
        .iter()
        .map(|pair| {
            (
                ResourceId::new(format!("repo:{}", pair.superseded)),
                ResourceId::new(format!("repo:{}", pair.canonical)),
            )
        })
        .collect()
}

/// The names a canonical resource inherits from the copy it replaces.
fn aliases_for(declared: &Canonical, id: &ResourceId) -> Vec<String> {
    id.as_str()
        .strip_prefix("repo:")
        .map(|full| declared.aliases(full))
        .unwrap_or_default()
}
