//! The harness: what every arm scores on the frozen sets, with its baselines.
//!
//! Built before there is anything to evaluate but the matcher, which is the
//! order the saga insists on: a yardstick written after seeing a result is not
//! a yardstick. Three rules it enforces, each because a measurement elsewhere
//! went wrong without it:
//!
//! - **No accuracy without its baseline.** AT01 read 0.697 on intent as a
//!   result; always answering `FindResource` scores 0.737 on the same rows.
//! - **No margin without its interval.** 54 paraphrases carry a 95% interval
//!   of about ±13 points, which cannot tell +20 from +7.
//! - **recall@k is the headline**, not accuracy@1: it is the ceiling on
//!   anything that reranks an arm's candidates, and it is what decides whether
//!   Saga 3's reranker has room to win at all.

pub mod metrics;
pub mod score;

pub use score::{Ranked, run};

use atlas_questions::{QuestionSet, Row, Status};

/// Rows the owner has confirmed. An unconfirmed row is not a yardstick, so it
/// is not scored -- it is counted and named in the report instead.
pub fn confirmed(set: &QuestionSet) -> Vec<&Row> {
    set.rows
        .iter()
        .filter(|row| row.status == Status::Confirmed)
        .collect()
}

/// One arm's score on one set.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Score {
    /// Questions scored.
    pub asked: usize,
    /// Share whose expected answer is the top candidate.
    pub top1: f64,
    /// Share whose expected answer is anywhere in the top k, for k in 1..=20.
    /// The ceiling on anything that reranks this arm's candidates.
    pub recall: Vec<f64>,
    /// Mean reciprocal rank of the first expected answer.
    pub mrr: f64,
    /// Share of intents the rules got right.
    pub intent: f64,
    /// What always answering the commonest intent would score.
    pub intent_baseline: f64,
    /// Share of off-topic questions the arm refused, by proposing nothing.
    pub refused: f64,
    /// Per-question correctness at 1, in set order, for the paired tests.
    pub correct: Vec<bool>,
    /// Questions this arm got wrong at rank 1 but found within the depth: the
    /// ones a reranker could fix.
    pub reorderable: Vec<String>,
    /// Questions whose answer the arm never proposed at all: the ones no
    /// reranker can fix, because the candidate was never offered.
    pub unreachable: Vec<String>,
}

impl Score {
    /// Recall at one k, counting from 1.
    pub fn at(&self, k: usize) -> f64 {
        self.recall.get(k.saturating_sub(1)).copied().unwrap_or(0.0)
    }
}
