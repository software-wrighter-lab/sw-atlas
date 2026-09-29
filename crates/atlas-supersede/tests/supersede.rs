//! Folding two copies of one piece of work into one.
//!
//! The case that matters is the one the cache rule cannot reach: a blog post
//! links to the repository under its old name, so the superseded resource is
//! recreated at merge time no matter what the repository ingester decided.
//! If this pass is wrong, the docent sends a visitor to a repository nobody
//! pushes to, which is the defect the whole step exists to remove.

use atlas_core::{Provenance, Relation, RelationKind, Resource, ResourceId, ResourceKind};
use atlas_corpus::Corpus;
use atlas_supersede::{Canonical, supersede};

fn declarations() -> Canonical {
    let text = r#"Canonical(pairs: [
        (canonical: "sw-vibe-coding/sw-init", superseded: "softwarewrighter/proact",
         why: "renamed on the way out", pending_sync: true),
    ])"#;
    ron::from_str(text).expect("the fixture parses")
}

/// A post, the live repository, and the copy the post actually links to.
fn corpus() -> Corpus {
    let mut corpus = Corpus::new();
    let post = Resource::stub(
        ResourceId::new("blog:2026-01-01-tooling"),
        ResourceKind::Post,
        "Tooling".into(),
        "https://example.invalid/post".into(),
    );
    let live = Resource::stub(
        ResourceId::new("repo:sw-vibe-coding/sw-init"),
        ResourceKind::Repo,
        "sw-init".into(),
        "https://github.com/sw-vibe-coding/sw-init".into(),
    );
    let old = Resource::stub(
        ResourceId::new("repo:softwarewrighter/proact"),
        ResourceKind::Repo,
        "proact".into(),
        "https://github.com/softwarewrighter/proact".into(),
    );
    corpus.concepts.push(atlas_core::Concept {
        id: atlas_core::ConceptId::new("rust"),
        label: "rust".into(),
        aliases: Vec::new(),
        parents: Vec::new(),
        resources: vec![old.id.clone(), live.id.clone()],
    });
    corpus.relations.push(Relation {
        from: post.id.clone(),
        kind: RelationKind::Discusses,
        to: old.id.clone(),
        weight: 1.0,
        provenance: Provenance::Declared,
    });
    corpus.resources.extend([post, live, old]);
    corpus
}

#[test]
fn the_superseded_copy_goes_and_the_post_now_points_at_the_live_one() {
    let mut corpus = corpus();
    let folded = supersede(&mut corpus, &declarations());
    assert_eq!(folded, 1);
    let ids: Vec<&str> = corpus.resources.iter().map(|r| r.id.as_str()).collect();
    assert!(!ids.contains(&"repo:softwarewrighter/proact"));
    assert!(ids.contains(&"repo:sw-vibe-coding/sw-init"));
    assert_eq!(
        corpus.relations[0].to.as_str(),
        "repo:sw-vibe-coding/sw-init",
        "the post linked to the old name and the visitor still wants what it meant"
    );
    assert!(
        atlas_corpus::validate(&corpus).is_empty(),
        "no dangling ids"
    );
}

#[test]
fn the_old_name_becomes_a_way_to_ask_for_the_live_one() {
    let mut corpus = corpus();
    supersede(&mut corpus, &declarations());
    let live = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "repo:sw-vibe-coding/sw-init")
        .expect("kept");
    assert!(
        live.aliases.iter().any(|a| a == "proact"),
        "{:?}",
        live.aliases
    );
}

#[test]
fn a_concept_lists_the_live_copy_once_not_both_copies() {
    let mut corpus = corpus();
    supersede(&mut corpus, &declarations());
    let listed: Vec<&str> = corpus.concepts[0]
        .resources
        .iter()
        .map(|id| id.as_str())
        .collect();
    assert_eq!(
        listed,
        ["repo:sw-vibe-coding/sw-init"],
        "folded, not doubled"
    );
}

#[test]
fn a_corpus_with_nothing_to_fold_is_left_alone() {
    let mut corpus = corpus();
    corpus
        .resources
        .retain(|r| r.id.as_str() != "repo:softwarewrighter/proact");
    corpus.relations.clear();
    let before = corpus.resources.len();
    assert_eq!(supersede(&mut corpus, &Canonical::default()), 0);
    assert_eq!(corpus.resources.len(), before);
}
