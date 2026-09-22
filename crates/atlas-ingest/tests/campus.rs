//! The campus ingest, pinned against snapshot A.
//!
//! The fixture under `tests/fixtures/campus` is sw-campus's published
//! `pages/docent/snapshot-a.json`, verbatim. Snapshot A deliberately
//! excludes the IBM 1442 card read punch and its radio demo, which two
//! later experiments depend on, so that exclusion is asserted here rather
//! than trusted.

use atlas_campus::{ingest, source};
use atlas_core::{Provenance, RelationKind, ResourceKind};
use atlas_corpus::{Corpus, validate};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/campus")
}

fn corpus() -> Corpus {
    ingest(&fixture()).expect("snapshot A ingests")
}

/// How many `PartOf` edges lie between a place and the campus root.
fn depth(corpus: &Corpus, id: &str) -> usize {
    let mut at = id.to_string();
    let mut steps = 0;
    while let Some(edge) = corpus
        .relations
        .iter()
        .find(|r| r.kind == RelationKind::PartOf && r.from.as_str() == at)
    {
        at = edge.to.as_str().to_string();
        steps += 1;
    }
    steps
}

#[test]
fn snapshot_a_holds_nine_places() {
    let corpus = corpus();
    let places = corpus
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Campus)
        .count();
    assert_eq!(places, 9);
}

#[test]
fn the_place_kind_equals_its_depth_so_it_need_not_be_stored() {
    // campus 0, building 1, wing 2, exhibit 3. If a future catalog breaks
    // this, the kind stops being derivable and the schema needs a field --
    // which is what this test is here to tell us.
    let corpus = corpus();
    for (id, expected) in [
        ("campus:campus", 0),
        ("campus:computer-history", 1),
        ("campus:ibm-1130", 2),
        ("campus:ibm-1130-emulator", 3),
        ("campus:computer-science", 1),
        ("campus:language-experiments", 2),
        ("campus:apl", 3),
    ] {
        assert_eq!(depth(&corpus, id), expected, "{id}");
    }
}

#[test]
fn the_1442_is_not_a_place_in_snapshot_a() {
    // Reserved for snapshot B. moe-microscope's incremental-learning
    // experiment and this project's SN01 (Saga 7) both spend it exactly once.
    let corpus = corpus();
    let named: Vec<&str> = corpus
        .resources
        .iter()
        .map(|r| r.id.as_str())
        .filter(|id| id.contains("1442"))
        .collect();
    assert!(named.is_empty(), "the 1442 leaked in as {named:?}");
}

#[test]
fn but_the_1442_is_mentioned_in_prose_and_sn01_must_account_for_it() {
    // The 1130 wing's arrival story lists the peripherals plugged into the
    // machine, and the 1442 is one of them. So the withheld knowledge is
    // the *exhibit and its demo*, not the string: a retrieval tier reading
    // catalog text can match the word today. SN01 has to ask where the
    // card reader plays music, not whether "1442" appears anywhere.
    let corpus = corpus();
    let mentions = corpus
        .resources
        .iter()
        .filter(|r| r.body.contains("1442"))
        .count();
    assert_eq!(mentions, 1, "one story mentions it; see docs/plan.md, SN01");
}

#[test]
fn stories_land_in_the_body_with_their_kind_and_title() {
    let corpus = corpus();
    let campus = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "campus:campus")
        .expect("the campus itself");
    assert!(campus.body.contains("## [arrival] Welcome to the campus"));
    assert!(
        campus
            .body
            .contains("## [detail] Why a map instead of a list")
    );
}

#[test]
fn a_place_keeps_the_aliases_and_concepts_the_author_wrote() {
    let corpus = corpus();
    let apl = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "campus:apl")
        .expect("the APL exhibit");
    assert!(apl.aliases.iter().any(|a| a == "array programming"));
    assert!(apl.aliases.iter().any(|a| a == "apl on cor24"));
    assert!(apl.concepts.iter().any(|c| c.as_str() == "glyphs"));
    assert_eq!(
        apl.url,
        "https://software-wrighter-lab.github.io/sw-campus/#/campus/computer-science/language-experiments/apl"
    );
}

#[test]
fn a_campus_link_and_a_blog_link_name_the_same_repository() {
    // This is the whole point of deriving identifiers from URLs: a visitor
    // standing in front of the 1130 exhibit can be shown the post about
    // it, because both sources produced `repo:sw-comp-history/ibm-1130-rs`.
    let corpus = corpus();
    let ids: Vec<&str> = corpus.resources.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"repo:sw-comp-history/ibm-1130-rs"));
    assert!(ids.contains(&"demo:sw-comp-history.github.io/ibm-1130-rs"));
    assert!(ids.contains(&"repo:sw-embed/sw-cor24-apl"));
}

#[test]
fn nothing_the_campus_declared_is_attributed_to_a_model() {
    assert!(
        corpus()
            .relations
            .iter()
            .all(|r| r.provenance == Provenance::Declared)
    );
}

#[test]
fn the_snapshot_is_used_until_a_catalog_is_published() {
    let chosen = source(&fixture()).expect("a source exists");
    assert!(
        chosen.ends_with("pages/docent/snapshot-a.json"),
        "sw-campus has not published dist/catalog.json yet: {chosen:?}"
    );
}

#[test]
fn the_ingested_corpus_validates() {
    assert_eq!(validate(&corpus()), Vec::new());
}
