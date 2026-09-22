//! The repository ingest, pinned against a four-entry cache and against
//! the committed one.
//!
//! The fixture holds one of each shape the real cache has: a repository
//! with topics and a language, a fork, one whose homepage is a live demo,
//! and one GitHub knows almost nothing about.

use atlas_core::{Provenance, RelationKind, ResourceKind};
use atlas_corpus::{Corpus, validate};
use std::path::PathBuf;

fn ingest(relative: &str) -> Corpus {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    atlas_repos::ingest(&path).expect("the cache ingests")
}

fn fixture() -> Corpus {
    ingest("tests/fixtures/github-repos.json")
}

#[test]
fn forks_are_excluded_and_nothing_else_is() {
    let corpus = fixture();
    let repos: Vec<&str> = corpus
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Repo)
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(repos.len(), 3);
    assert!(!repos.contains(&"repo:sw-embed/bmp280"), "the fork");
    assert!(validate(&corpus).is_empty(), "{:?}", validate(&corpus));
}

#[test]
fn a_repository_has_the_identifier_a_post_linking_to_it_has() {
    let corpus = fixture();
    let apl = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "repo:sw-embed/sw-cor24-apl")
        .expect("the id the blog and the campus already use");
    assert_eq!(apl.url, "https://github.com/sw-embed/sw-cor24-apl");
    assert_eq!(apl.title, "sw-cor24-apl");
    assert_eq!(apl.summary, "APL for the COR24");
    assert_eq!(apl.date, "2026-09-10");
    let concepts: Vec<&str> = apl.concepts.iter().map(|c| c.as_str()).collect();
    assert_eq!(concepts, ["apl", "array-languages", "rust"]);
    assert_eq!(apl.aliases, ["sw cor24 apl"]);
    assert!(!apl.source_hash.is_empty());
}

#[test]
fn a_homepage_is_a_declared_demo() {
    let corpus = fixture();
    let edge = corpus
        .relations
        .iter()
        .find(|r| r.from.as_str() == "repo:software-wrighter-lab/sw-campus")
        .expect("the homepage edge");
    assert_eq!(edge.kind, RelationKind::Demos);
    assert_eq!(edge.provenance, Provenance::Declared);
    assert_eq!(
        edge.to.as_str(),
        "demo:software-wrighter-lab.github.io/sw-campus"
    );
    assert_eq!(corpus.relations.len(), 1, "an empty homepage names nothing");
}

#[test]
fn a_repository_github_knows_little_about_is_still_a_resource() {
    let corpus = fixture();
    let bare = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "repo:sw-ml-study/emufpga")
        .expect("present");
    assert!(bare.summary.is_empty() && bare.date.is_empty() && bare.concepts.is_empty());
}

#[test]
fn the_committed_cache_holds_242_public_non_fork_repositories() {
    // Pinned on purpose: refreshing the cache is a corpus change, and its
    // commit updates this number with the new count.
    let corpus = ingest("../../cache/github-repos.json");
    let repos = corpus
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Repo);
    assert_eq!(repos.count(), 242);
    assert!(validate(&corpus).is_empty());
}
