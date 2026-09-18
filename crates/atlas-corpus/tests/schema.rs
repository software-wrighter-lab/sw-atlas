//! The contract every later step depends on, checked against two fixtures:
//! one corpus that is correct and one that is wrong in every way the
//! validator is supposed to catch.

use atlas_corpus::{Corpus, Problem, canonical, to_ron, validate};

fn load(name: &str) -> Corpus {
    let path = format!("{}/tests/fixtures/{name}.ron", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).expect("fixture is readable");
    ron::from_str(&text).expect("fixture parses")
}

#[test]
fn a_correct_corpus_has_no_problems() {
    assert_eq!(validate(&load("valid")), Vec::new());
}

#[test]
fn round_trip_preserves_the_corpus() {
    let corpus = load("valid");
    let text = to_ron(&corpus).expect("serialises");
    let back: Corpus = ron::from_str(&text).expect("re-parses");
    assert_eq!(validate(&back), Vec::new());
    assert_eq!(back.resources.len(), corpus.resources.len());
    assert_eq!(back.relations, corpus.relations);
}

#[test]
fn canonical_form_is_byte_stable() {
    let corpus = load("valid");
    let (first, first_hash) = canonical(&corpus).expect("serialises");
    let mut shuffled = corpus.clone();
    shuffled.resources.reverse();
    shuffled.concepts[0].aliases.reverse();
    let (second, second_hash) = canonical(&shuffled).expect("serialises");
    assert_eq!(first, second, "input order must not change the bytes");
    assert_eq!(first_hash, second_hash);
    assert_eq!(first_hash.len(), 64, "sha-256 in lowercase hex");
}

#[test]
fn a_changed_fact_changes_the_hash() {
    let corpus = load("valid");
    let (_, before) = canonical(&corpus).expect("serialises");
    let mut edited = corpus.clone();
    edited.resources[0].summary.push_str(" Revised.");
    let (_, after) = canonical(&edited).expect("serialises");
    assert_ne!(before, after);
}

#[test]
fn the_broken_fixture_fails_every_way_it_should() {
    let problems = validate(&load("broken"));
    let duplicates = problems
        .iter()
        .filter(|p| matches!(p, Problem::Duplicate { .. }))
        .count();
    let unknown_resources = problems
        .iter()
        .filter(|p| matches!(p, Problem::UnknownResource { .. }))
        .count();
    let unknown_concepts = problems
        .iter()
        .filter(|p| matches!(p, Problem::UnknownConcept { .. }))
        .count();
    assert_eq!(duplicates, 1, "the repeated resource id");
    assert_eq!(
        unknown_resources, 2,
        "the relation endpoint and the back-reference"
    );
    assert_eq!(unknown_concepts, 2, "the resource's concept and the parent");
}

#[test]
fn problems_name_where_they_were_found() {
    let problems = validate(&load("broken"));
    let contexts: Vec<&str> = problems
        .iter()
        .filter_map(|p| match p {
            Problem::UnknownResource { context, .. } => Some(context.as_str()),
            Problem::UnknownConcept { context, .. } => Some(context.as_str()),
            Problem::Duplicate { .. } => None,
        })
        .collect();
    assert!(
        contexts.iter().any(|c| c.contains("parent of orphaned")),
        "a dangling parent says whose parent it is: {contexts:?}"
    );
}
