//! Both gates, demonstrated failing on a seeded fault and passing without it.
//!
//! A gate nobody has watched fail is a gate nobody should trust, so each of
//! the two is shown rejecting exactly the fault it exists to catch: a
//! resource with no concept and no relation, and a URL the last check found
//! gone.

use atlas_core::{
    Concept, ConceptId, Provenance, Relation, RelationKind, Resource, ResourceId, ResourceKind,
};
use atlas_corpus::Corpus;
use atlas_coverage::{markdown, measure};
use atlas_links::{Cache, Record, Status};

fn resource(id: &str, kind: ResourceKind, url: &str) -> Resource {
    Resource::stub(ResourceId::new(id), kind, id.to_string(), url.to_string())
}

/// A post with a concept, and a repository the post links to: everything
/// here is reachable, and every URL last answered 200.
fn healthy() -> (Corpus, Cache) {
    let mut corpus = Corpus::new();
    let mut post = resource(
        "blog:one",
        ResourceKind::Post,
        "https://example.invalid/one",
    );
    post.concepts = vec![ConceptId::new("apl")];
    corpus.concepts.push(Concept::provisional("apl"));
    let repo = resource(
        "repo:sw-embed/x",
        ResourceKind::Repo,
        "https://example.invalid/x",
    );
    corpus.relations.push(Relation {
        from: post.id.clone(),
        kind: RelationKind::Implements,
        to: repo.id.clone(),
        weight: 1.0,
        provenance: Provenance::Declared,
    });
    corpus.resources.extend([post, repo]);
    let mut cache = Cache::default();
    for url in ["https://example.invalid/one", "https://example.invalid/x"] {
        let record = Record {
            status: Status::Ok,
            code: 200,
            checked: "2026-09-22".into(),
        };
        cache.urls.insert(url.to_string(), record);
    }
    (corpus, cache)
}

#[test]
fn a_healthy_corpus_passes_both_gates() {
    let (corpus, cache) = healthy();
    let coverage = measure(&corpus, &cache, 2);
    assert_eq!(coverage.failures(), Vec::<String>::new());
    assert_eq!(coverage.links.urls, 2);
    assert_eq!(coverage.links.ok, 2);
    assert_eq!(coverage.excluded, 2, "the report states what was excluded");
    assert!(markdown::render(&coverage, "hash", "2026-09-22").contains("**Both gates pass.**"));
}

#[test]
fn a_resource_nothing_can_reach_fails_the_first_gate() {
    let (mut corpus, cache) = healthy();
    // Seeded fault: a repository with no concept and nothing linking to it,
    // exactly the 15 the corpus had before the organisation concept.
    corpus
        .resources
        .push(resource("repo:sw-fun/lonely", ResourceKind::Repo, ""));
    let coverage = measure(&corpus, &cache, 0);
    let failures = coverage.failures();
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(
        failures[0].contains("1 unreachable resources"),
        "{failures:?}"
    );
    assert!(
        failures[0].contains("repo:sw-fun/lonely"),
        "it names the fault"
    );
    let repos = coverage
        .kinds
        .iter()
        .find(|k| k.kind == ResourceKind::Repo)
        .expect("counted per kind");
    assert_eq!(repos.orphans, ["repo:sw-fun/lonely"]);
    assert!(markdown::render(&coverage, "hash", "never").contains("**The gates fail.**"));
}

#[test]
fn a_dead_url_fails_the_second_gate() {
    let (corpus, mut cache) = healthy();
    // Seeded fault: the last check found the post's URL gone.
    let gone = Record {
        status: Status::Missing,
        code: 404,
        checked: "2026-09-22".into(),
    };
    cache
        .urls
        .insert("https://example.invalid/one".into(), gone);
    let coverage = measure(&corpus, &cache, 0);
    let failures = coverage.failures();
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(failures[0].starts_with("1 dead URLs"), "{failures:?}");
    assert!(
        failures[0].contains("example.invalid/one"),
        "it names the URL"
    );
}

#[test]
fn a_blocked_host_and_an_unchecked_url_are_reported_but_not_gated() {
    let (mut corpus, mut cache) = healthy();
    let blocked = Record {
        status: Status::Blocked,
        code: 403,
        checked: "2026-09-22".into(),
    };
    cache
        .urls
        .insert("https://example.invalid/x".into(), blocked);
    corpus.resources.push({
        let mut video = resource(
            "video:abc",
            ResourceKind::Video,
            "https://example.invalid/v",
        );
        video.concepts = vec![ConceptId::new("apl")];
        video
    });
    let coverage = measure(&corpus, &cache, 0);
    assert_eq!(
        coverage.failures(),
        Vec::<String>::new(),
        "neither is this corpus being wrong"
    );
    assert_eq!(coverage.links.blocked, ["https://example.invalid/x"]);
    assert_eq!(coverage.links.unchecked, 1);
    assert_eq!(coverage.links.urls, 3);
}

#[test]
fn a_concept_no_resource_carries_is_reported() {
    let (mut corpus, cache) = healthy();
    corpus.concepts.push(Concept::provisional("forth"));
    let coverage = measure(&corpus, &cache, 0);
    assert_eq!(coverage.unused_concepts, ["forth"]);
    assert_eq!(
        coverage.failures(),
        Vec::<String>::new(),
        "dead weight, not a failure"
    );
}
