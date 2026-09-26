//! The five outcomes, aimed one at a time.
//!
//! The policy's whole job is to decline gracefully, so the tests that matter
//! are the ones where it must not answer: nonsense, a real subject the index
//! does not hold, and a word that lands in prose by coincidence. A policy
//! tested only on questions it gets right would be a policy tested on 2% of
//! its behaviour -- SEVERAL fires on 0.961 of the confirmed sets, and the rest
//! is the part a visitor remembers.

use atlas_answer::{Absent, Outcome, Policy, decide, frames, suggest};
use atlas_core::{
    Concept, ConceptId, Maturity, Provenance, Relation, RelationKind, Resource, ResourceId,
    ResourceKind,
};
use atlas_corpus::Corpus;
use atlas_match::{Matcher, Variant};
use std::path::Path;

/// A corpus small enough to reason about: a post about BERT with the alias
/// that causes the coincidence, a working demo, and a planned exhibit.
fn corpus() -> Corpus {
    let mut corpus = Corpus::new();
    corpus.concepts.push(Concept {
        id: ConceptId::new("transformers"),
        label: "transformers".into(),
        aliases: Vec::new(),
        parents: Vec::new(),
        resources: vec![ResourceId::new("blog:bert")],
    });
    let mut post = Resource::stub(
        ResourceId::new("blog:bert"),
        ResourceKind::Post,
        "Five concepts".into(),
        "https://example.invalid/bert".into(),
    );
    post.aliases = vec!["bert".into()];
    post.concepts = vec![ConceptId::new("transformers")];
    post.body = "the weather was mentioned once in passing".into();
    let mut demo = Resource::stub(
        ResourceId::new("demo:keypunch"),
        ResourceKind::Demo,
        "Keypunch".into(),
        "https://example.invalid/keypunch".into(),
    );
    demo.maturity = Some(Maturity::Working);
    demo.summary = "a browser keypunch".into();
    let mut planned = Resource::stub(
        ResourceId::new("campus:plotter"),
        ResourceKind::Campus,
        "Plotter Wing".into(),
        "https://example.invalid/plotter".into(),
    );
    planned.maturity = Some(Maturity::Planned);
    planned.aliases = vec!["plotter wing".into()];
    corpus.relations.push(Relation {
        from: post.id.clone(),
        kind: RelationKind::Demos,
        to: demo.id.clone(),
        weight: 1.0,
        provenance: Provenance::Declared,
    });
    corpus.resources.extend([post, demo, planned]);
    corpus
}

/// The thresholds this repository actually ships, not invented ones: a test
/// against hand-picked numbers would pass while the shipped policy misbehaves.
fn rules() -> (Policy, Absent) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    (
        Policy::load(&root.join("sources/answer-policy.ron")).expect("the fitted policy"),
        Absent::load(&root.join("sources/absent.ron")).expect("the absent list"),
    )
}

fn ask(corpus: &Corpus, query: &str) -> atlas_answer::Reply {
    let (policy, absent) = rules();
    let matcher = Matcher::new(corpus, Variant::Text);
    decide(
        query,
        &matcher.answer(query, 10),
        corpus,
        (&policy, &absent),
    )
}

#[test]
fn an_alias_hiding_inside_a_longer_word_is_not_evidence_of_anything() {
    let corpus = corpus();
    // MB02 keeps MB01's substring alias rule, so it scores this at 4: the
    // matcher must not change, because its comparability to MB01 depends on
    // making MB01's mistakes. The policy is where the mistake stops.
    let raw = Matcher::new(&corpus, Variant::Text).answer("zzzq flibbertigibbet", 10);
    assert_eq!(raw.hits.len(), 1, "the matcher still scores it: {raw:?}");
    assert!(raw.hits[0].score >= 4.0, "{:?}", raw.hits[0]);

    let reply = ask(&corpus, "zzzq flibbertigibbet");
    assert_eq!(reply.outcome, Outcome::Rephrase, "{reply:?}");
    assert!(reply.offer.is_empty(), "nothing is offered: {reply:?}");
}

#[test]
fn a_word_the_corpus_only_has_in_prose_is_not_a_subject_match() {
    let reply = ask(&corpus(), "what is the weather tomorrow");
    assert_eq!(reply.outcome, Outcome::NothingHere, "{reply:?}");
    assert!(reply.offer.is_empty());
}

#[test]
fn a_named_subject_the_corpus_holds_is_offered_and_a_planned_one_says_so() {
    let corpus = corpus();
    let reply = ask(&corpus, "is there a keypunch demo");
    assert!(
        reply.offer.contains(&"demo:keypunch".to_string()),
        "{reply:?}"
    );
    assert!(
        matches!(reply.outcome, Outcome::One | Outcome::Several),
        "{reply:?}"
    );

    let planned = ask(&corpus, "where is the plotter wing");
    match planned.outcome {
        Outcome::NotYet(text) => assert!(text.contains("not built yet"), "{text}"),
        other => panic!("a planned exhibit should say so: {other:?}"),
    }
}

#[test]
fn the_absent_list_answers_for_a_subject_the_index_does_not_hold() {
    let reply = ask(&corpus(), "is the card reader exhibit finished yet");
    match reply.outcome {
        Outcome::NotYet(text) => assert!(text.contains("1442"), "{text}"),
        other => panic!("the absent list should have caught this: {other:?}"),
    }
    assert!(
        reply.offer.is_empty(),
        "there is nothing to offer for a thing that does not exist"
    );
}

#[test]
fn one_offered_resource_reads_as_one_and_three_read_as_three() {
    let corpus = corpus();
    let reply = atlas_answer::Reply {
        outcome: Outcome::Several,
        offer: vec!["demo:keypunch".to_string()],
        because: "fixture".to_string(),
    };
    let shown = frames::render(&reply, &corpus, &[]);
    assert!(shown.contains("The closest thing I have is"), "{shown}");
    assert!(!shown.contains("1 things"), "no plural of one: {shown}");
}

#[test]
fn a_suggestion_says_that_only_when_there_is_a_that() {
    let corpus = corpus();
    let with = suggest::next_questions(&corpus, &["blog:bert".to_string()], 3);
    assert!(
        with.iter().any(|s| s.contains("run it in the browser")),
        "the post declares a working demo: {with:?}"
    );
    let without = suggest::next_questions(&corpus, &[], 3);
    assert!(
        without.iter().all(|s| !s.contains("that?")),
        "nothing was offered, so nothing can be referred to: {without:?}"
    );
    assert!(
        without
            .iter()
            .all(|s| s.starts_with("what do you have about")),
        "{without:?}"
    );
}
