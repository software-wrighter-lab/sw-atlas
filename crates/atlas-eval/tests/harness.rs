//! The harness measured against arms whose right answers are known in advance.
//!
//! A metric that is wrong in the same direction for every arm still ranks the
//! arms correctly, which is why a harness needs its own tests: the numbers it
//! prints go into a scoreboard and then into an argument about whether to build
//! a model.

use atlas_eval::metrics::{interval, majority, mcnemar};
use atlas_eval::{Ranked, run};
use atlas_questions::QuestionSet;

/// Four questions: two with a destination, one open, one off-topic.
fn set() -> QuestionSet {
    let text = r#"QuestionSet(name: "test", note: "fixture", rows: [
        (text: "first", expect: ["a"], intent: FindResource, origin: Authored, status: Confirmed),
        (text: "second", expect: ["b"], intent: FindResource, origin: Authored, status: Confirmed),
        (text: "open", expect: [], intent: Meta, origin: Authored, status: Confirmed),
        (text: "later", expect: ["c"], intent: Navigate, origin: Teacher, status: Unconfirmed),
    ])"#;
    ron::from_str(text).expect("the fixture parses")
}

/// An arm that answers from a table, and always says `FindResource`.
fn arm(answers: Vec<(&'static str, Vec<&'static str>)>) -> impl Fn(&str) -> Ranked {
    move |query: &str| Ranked {
        ids: answers
            .iter()
            .find(|(asked, _)| *asked == query)
            .map(|(_, ids)| ids.iter().map(|id| (*id).to_string()).collect())
            .unwrap_or_default(),
        intent: "FindResource".to_string(),
    }
}

#[test]
fn an_unconfirmed_row_is_counted_and_not_scored() {
    let set = set();
    assert_eq!(set.counted(), (3, 1));
    assert_eq!(
        atlas_eval::confirmed(&set).len(),
        3,
        "the drafted row is skipped"
    );
}

#[test]
fn a_perfect_arm_scores_one_and_an_empty_arm_scores_zero() {
    let fixture = set();
    let rows = atlas_eval::confirmed(&fixture);
    let perfect = run(
        &rows,
        &arm(vec![("first", vec!["a"]), ("second", vec!["b"])]),
    );
    assert!((perfect.top1 - 1.0).abs() < 1e-9);
    assert!((perfect.mrr - 1.0).abs() < 1e-9);
    assert!((perfect.at(20) - 1.0).abs() < 1e-9);
    assert_eq!(perfect.reorderable.len(), 0);

    let silent = run(&rows, &arm(vec![]));
    assert!((silent.top1 - 0.0).abs() < 1e-9);
    assert_eq!(
        silent.unreachable.len(),
        2,
        "both destinations were never offered"
    );
    assert!(
        (silent.refused - 1.0).abs() < 1e-9,
        "it refused the open question"
    );
}

#[test]
fn a_right_answer_at_rank_three_is_a_rerankers_opportunity_not_a_recall_failure() {
    let fixture = set();
    let rows = atlas_eval::confirmed(&fixture);
    let scored = run(
        &rows,
        &arm(vec![("first", vec!["x", "y", "a"]), ("second", vec!["b"])]),
    );
    assert!((scored.top1 - 0.5).abs() < 1e-9, "one of two at rank 1");
    assert!((scored.at(3) - 1.0).abs() < 1e-9, "both within three");
    assert!((scored.at(2) - 0.5).abs() < 1e-9, "only one within two");
    assert!((scored.mrr - (1.0 + 1.0 / 3.0) / 2.0).abs() < 1e-9);
    assert_eq!(scored.reorderable, ["first"], "a ranking problem");
    assert!(scored.unreachable.is_empty(), "not a recall problem");
}

#[test]
fn an_intent_accuracy_always_arrives_with_the_score_of_a_constant() {
    let fixture = set();
    let rows = atlas_eval::confirmed(&fixture);
    let scored = run(&rows, &arm(vec![]));
    // Two of three confirmed rows are FindResource, which the arm always says.
    assert!((scored.intent - 2.0 / 3.0).abs() < 1e-9);
    assert!((scored.intent_baseline - 2.0 / 3.0).abs() < 1e-9);
    assert!(
        (scored.intent - scored.intent_baseline).abs() < 1e-9,
        "an arm that always says the commonest answer ties its own baseline"
    );
}

#[test]
fn the_majority_baseline_is_the_commonest_answers_share() {
    let intents: Vec<String> = ["a", "a", "a", "b"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    assert!((majority(&intents) - 0.75).abs() < 1e-9);
    assert!((majority(&[]) - 0.0).abs() < 1e-9);
}

#[test]
fn identical_arms_have_no_margin_and_no_significance() {
    let same = vec![true, false, true, true];
    let (wins, losses, p) = mcnemar(&same, &same);
    assert_eq!((wins, losses), (0, 0));
    assert!((p - 1.0).abs() < 1e-9);
    assert_eq!(interval(&same, &same, 500), (0.0, 0.0));
}

#[test]
fn an_arm_that_wins_everywhere_is_significant_and_its_interval_excludes_zero() {
    let better = vec![true; 30];
    let worse = vec![false; 30];
    let (wins, losses, p) = mcnemar(&better, &worse);
    assert_eq!((wins, losses), (30, 0));
    assert!(
        p < 0.001,
        "thirty wins and no losses is not a coin: p = {p}"
    );
    let (low, high) = interval(&better, &worse, 2000);
    assert!(low > 0.0 && (high - 1.0).abs() < 1e-9, "{low} to {high}");
}

#[test]
fn a_one_question_difference_in_thirty_is_not_significant() {
    let mut better = vec![true; 15];
    better.extend(vec![false; 15]);
    let mut worse = better.clone();
    worse[0] = false;
    let (wins, losses, p) = mcnemar(&better, &worse);
    assert_eq!((wins, losses), (1, 0));
    assert!(p > 0.4, "one question is a coin flip: p = {p}");
}
