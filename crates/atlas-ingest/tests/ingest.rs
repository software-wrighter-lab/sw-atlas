//! The blog ingest, pinned against three real posts.
//!
//! The fixtures under `tests/fixtures/_posts` are the front matter of
//! three published posts, verbatim, with the body replaced. They were
//! chosen for their shapes: one writes `video_urls` as bare strings with a
//! parallel `video_titles` list, one writes `repo_urls` as mappings *and*
//! repeats one of them in `repo_url`, and one carries a `demo_url`. Two
//! are parts 5 and 11 of the same series.

use atlas_core::{Provenance, RelationKind, ResourceKind};
use atlas_corpus::{Corpus, validate};
use atlas_ingest::assemble::blog;
use std::path::PathBuf;

fn corpus() -> Corpus {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/_posts");
    blog(&dir).expect("the fixtures ingest")
}

fn ids(corpus: &Corpus, kind: ResourceKind) -> Vec<String> {
    let mut out: Vec<String> = corpus
        .resources
        .iter()
        .filter(|r| r.kind == kind)
        .map(|r| r.id.as_str().to_string())
        .collect();
    out.sort();
    out
}

#[test]
fn a_post_becomes_a_resource_with_its_published_url() {
    let corpus = corpus();
    let post = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "blog:2026-09-17-tbt-apl-360-revisited")
        .expect("the APL post");
    assert_eq!(post.kind, ResourceKind::Post);
    assert_eq!(post.date, "2026-09-17");
    assert_eq!(
        post.url,
        "https://blog.softwarewrighter.com/2026/09/17/tbt-apl-360-revisited/"
    );
    assert!(post.summary.starts_with("APL"), "the authored abstract");
    assert!(post.body.is_empty(), "no body text is read");
}

#[test]
fn keywords_become_aliases_as_the_author_wrote_them() {
    let corpus = corpus();
    let post = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "blog:2026-09-17-tbt-apl-360-revisited")
        .expect("the APL post");
    for expected in ["kenneth iverson", "sw-apl", "notation as a tool of thought"] {
        assert!(
            post.aliases.iter().any(|a| a == expected),
            "{expected} missing from {:?}",
            post.aliases
        );
    }
}

#[test]
fn every_declared_link_becomes_a_resource_with_a_stable_id() {
    let corpus = corpus();
    assert_eq!(
        ids(&corpus, ResourceKind::Repo),
        [
            "repo:softwarewrighter/mHC-poc",
            "repo:sw-comp-history/ibm-1130-rs",
            "repo:sw-embed/sw-cor24-apl",
            "repo:sw-ml-study/sw-mlpl",
            "repo:sw-vibe-coding/sw-apl",
        ]
    );
    assert_eq!(
        ids(&corpus, ResourceKind::Video),
        [
            "video:2RzJP2ofuuU",
            "video:BOuBFn5e1gA",
            "video:MYTXVYDtCEU",
            "video:fh21_zIK2ZE",
        ],
        "a /shorts/ URL and a watch?v= URL must yield the same shape of id"
    );
    assert_eq!(
        ids(&corpus, ResourceKind::Demo),
        ["demo:sw-comp-history.github.io/ibm-1130-rs"]
    );
    assert_eq!(
        ids(&corpus, ResourceKind::Paper),
        ["paper:arxiv.org/abs/2512.24880"]
    );
}

#[test]
fn a_repository_named_twice_by_one_post_is_one_resource_and_one_edge() {
    // The APL post declares sw-apl in `repo_url` and again in `repo_urls`.
    let corpus = corpus();
    let resources = corpus
        .resources
        .iter()
        .filter(|r| r.id.as_str() == "repo:sw-vibe-coding/sw-apl")
        .count();
    let edges = corpus
        .relations
        .iter()
        .filter(|r| {
            r.to.as_str() == "repo:sw-vibe-coding/sw-apl"
                && r.from.as_str() == "blog:2026-09-17-tbt-apl-360-revisited"
        })
        .count();
    assert_eq!(resources, 1);
    assert_eq!(edges, 1);
}

#[test]
fn a_parallel_title_list_names_its_video() {
    let corpus = corpus();
    let video = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "video:fh21_zIK2ZE")
        .expect("the first mHC video");
    assert_eq!(video.title, "DeepSeek's mHC Fix for Gradient Explosion");
}

#[test]
fn relation_kinds_follow_the_field_that_declared_them() {
    let corpus = corpus();
    let kinds = |kind| corpus.relations.iter().filter(|r| r.kind == kind).count();
    assert_eq!(kinds(RelationKind::Implements), 5, "five repositories");
    assert_eq!(kinds(RelationKind::Demos), 5, "four videos and one demo");
    assert_eq!(kinds(RelationKind::Cites), 1, "one paper");
    assert_eq!(kinds(RelationKind::SeriesNext), 1, "parts 5 and 11");
    assert_eq!(corpus.relations.len(), 12);
}

#[test]
fn the_series_edge_runs_from_the_earlier_part_to_the_later() {
    let corpus = corpus();
    let edge = corpus
        .relations
        .iter()
        .find(|r| r.kind == RelationKind::SeriesNext)
        .expect("a series edge");
    assert_eq!(
        edge.from.as_str(),
        "blog:2026-02-26-ibm-1130-system-emulator"
    );
    assert_eq!(edge.to.as_str(), "blog:2026-09-17-tbt-apl-360-revisited");
}

#[test]
fn nothing_the_blog_declared_is_attributed_to_a_model() {
    let corpus = corpus();
    assert!(
        corpus
            .relations
            .iter()
            .all(|r| r.provenance == Provenance::Declared),
        "a person wrote every one of these at publication time"
    );
}

#[test]
fn the_ingested_corpus_validates() {
    assert_eq!(validate(&corpus()), Vec::new());
}
