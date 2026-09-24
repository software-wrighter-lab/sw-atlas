//! The rules about the evaluation sets, each shown rejecting a seeded fault.
//!
//! These checks are the only thing standing between "the hybrid scores
//! higher" and "the hybrid was measured on questions that gave away their
//! answers", so each one is demonstrated failing.

use atlas_core::{Concept, ConceptId, Resource, ResourceId, ResourceKind};
use atlas_corpus::Corpus;
use atlas_questions::leakage::{leaks, verbatim};
use atlas_questions::{QuestionSet, rules};
use std::path::PathBuf;

/// A corpus of one post, whose aliases are the ones a paraphrase must avoid.
fn corpus() -> Corpus {
    let mut corpus = Corpus::new();
    let mut post = Resource::stub(
        ResourceId::new("blog:one"),
        ResourceKind::Post,
        "Tiny Recursive Model".into(),
        "https://example.invalid/one".into(),
    );
    post.aliases = vec!["maze solver".into(), "trm".into()];
    post.concepts = vec![ConceptId::new("small-models")];
    corpus.concepts.push(Concept::provisional("small-models"));
    corpus.resources.push(post);
    corpus
}

fn set(rows: &str) -> QuestionSet {
    let text = format!("QuestionSet(name: \"paraphrase\", note: \"test\", rows: [{rows}])");
    ron::from_str(&text).expect("the fixture set parses")
}

const GOOD: &str = r#"(text: "under a thousand weights beating the big ones at those grid puzzles",
    expect: ["blog:one"], intent: FindResource, origin: Teacher, status: Unconfirmed)"#;

#[test]
fn a_paraphrase_that_gives_away_its_answer_is_rejected() {
    let alias = r#"(text: "where is the maze solver post", expect: ["blog:one"],
        intent: FindResource, origin: Teacher, status: Unconfirmed)"#;
    let title = r#"(text: "the tiny recursive model writeup", expect: ["blog:one"],
        intent: FindResource, origin: Teacher, status: Unconfirmed)"#;
    let corpus = corpus();
    assert!(
        rules::problems(&set(GOOD), &corpus).is_empty(),
        "a real paraphrase passes"
    );
    assert_eq!(
        rules::problems(&set(alias), &corpus).len(),
        1,
        "an alias is given away"
    );
    assert_eq!(
        rules::problems(&set(title), &corpus).len(),
        1,
        "so is the title"
    );
}

#[test]
fn an_expectation_naming_no_resource_is_rejected() {
    let missing = r#"(text: "something perfectly reasonable", expect: ["blog:nosuchpost"],
        intent: FindResource, origin: Teacher, status: Unconfirmed)"#;
    let problems = rules::problems(&set(missing), &corpus());
    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("no such resource"), "{problems:?}");
}

#[test]
fn drafted_and_confirmed_rows_are_counted_apart() {
    let confirmed = GOOD.replace("Unconfirmed", "Confirmed");
    let both = set(&format!("{GOOD}, {confirmed}"));
    assert_eq!(both.counted(), (1, 1), "one confirmed, one still drafted");
}

#[test]
fn editing_a_row_changes_the_hash_that_froze_the_set() {
    let before = set(GOOD).digest();
    let after = set(&GOOD.replace("beating", "beatinq")).digest();
    assert_ne!(before, after, "a quiet edit is visible");
    assert_eq!(
        before,
        set(GOOD).digest(),
        "and the same rows hash the same"
    );
}

#[test]
fn a_frozen_question_reused_in_training_is_a_leak() {
    let frozen =
        vec!["under a thousand weights beating the big ones at those grid puzzles".to_string()];
    let same = vec![frozen[0].clone()];
    assert_eq!(leaks(&frozen, &same, 8).len(), 1, "the identical row");
    let edited = vec![
        "so under a thousand weights beating the big ones at those grid puzzles then".to_string(),
    ];
    let found = leaks(&frozen, &edited, 8);
    assert_eq!(
        found.len(),
        1,
        "a lightly edited row still shares eight words"
    );
    assert!(found[0].contains("shares"), "{found:?}");
}

#[test]
fn ordinary_english_is_not_a_leak() {
    let frozen = vec!["where can i find the post about the thing you built".to_string()];
    let unrelated = vec!["where can i find the code for the emulator".to_string()];
    assert!(
        leaks(&frozen, &unrelated, 8).is_empty(),
        "short overlaps are not leaks"
    );
}

#[test]
fn a_term_matches_only_on_whole_words() {
    assert!(verbatim("an apl interpreter", "apl"));
    assert!(!verbatim("applied mathematics", "apl"), "not a fragment");
    assert!(verbatim("the chain of thought debate", "chain of thought"));
    assert!(
        !verbatim("thought chain", "chain of thought"),
        "order matters"
    );
}

#[test]
fn every_committed_set_is_internally_consistent() {
    // Skips loudly rather than failing when the corpus has not been built:
    // a fresh clone without the sibling repositories cannot run `just
    // ingest`, and `just questions` is where this check gates for real.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(text) = std::fs::read_to_string(root.join("build/corpus/corpus.ron")) else {
        println!("skipped: no corpus built; run `just ingest` then `just questions`");
        return;
    };
    let corpus: Corpus = ron::from_str(&text).expect("the corpus parses");
    let dir = root.join("sources/questions");
    let mut rows = 0;
    for entry in std::fs::read_dir(&dir).expect("the sets are committed") {
        let path = entry.expect("readable").path();
        // REVIEW.txt lives here too: it is the owner's marking form, not a set.
        if path.extension().is_none_or(|e| e != "ron") {
            continue;
        }
        let set = QuestionSet::load(&path).expect("parses");
        let problems = rules::problems(&set, &corpus);
        assert!(problems.is_empty(), "{}: {problems:?}", set.name);
        rows += set.rows.len();
    }
    assert!(
        rows >= 196,
        "batch 1 drafted 196 rows; it should not shrink"
    );
}
