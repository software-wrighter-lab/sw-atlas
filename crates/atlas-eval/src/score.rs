//! Scoring one arm over one set of questions.
//!
//! An arm is anything that proposes ranked candidates: MB02, MB02t, MB02 plus
//! one declared hop, and later a hybrid. The harness knows nothing about how an
//! arm decides -- it hands over a question and reads back a ranking, which is
//! what keeps a model and a matcher comparable.

use crate::Score;
use crate::metrics::majority;
use atlas_questions::Row;

/// The most candidates any metric here looks at.
pub const DEPTH: usize = 20;

/// What an arm answered for one question.
pub struct Ranked {
    /// Candidate identifiers, best first.
    pub ids: Vec<String>,
    /// The intent the arm reported.
    pub intent: String,
}

/// Score one arm over the confirmed rows of one set.
///
/// `answer` is the arm. Rows with no expected resource (meta, off-topic, and
/// the open intent questions) score intent and refusal only: there is no
/// destination to be right about, and pretending otherwise would put a
/// meaningless zero in the destination column.
pub fn run(rows: &[&Row], answer: &dyn Fn(&str) -> Ranked) -> Score {
    let intents: Vec<String> = rows.iter().map(|r| format!("{:?}", r.intent)).collect();
    let mut score = Score {
        asked: rows.len(),
        recall: vec![0.0; DEPTH],
        intent_baseline: majority(&intents),
        ..Score::default()
    };
    let mut destinations = 0usize;
    let mut refusals = (0usize, 0usize);
    for row in rows {
        one(
            &mut score,
            row,
            &answer(&row.text),
            (&mut destinations, &mut refusals),
        );
    }
    finish(&mut score, destinations, refusals);
    score
}

/// Fold one question into the totals: its intent always, and then either its
/// destination or, where it has none, whether the arm refused.
fn one(score: &mut Score, row: &Row, got: &Ranked, counts: (&mut usize, &mut (usize, usize))) {
    let (destinations, refusals) = counts;
    score.intent += f64::from(u8::from(format!("{:?}", row.intent) == got.intent));
    if row.expect.is_empty() {
        refusals.1 += 1;
        refusals.0 += usize::from(got.ids.is_empty());
        return;
    }
    *destinations += 1;
    let found = got
        .ids
        .iter()
        .take(DEPTH)
        .position(|id| row.expect.contains(id));
    record(score, found, row);
}

/// Fold one question's rank into the running totals.
fn record(score: &mut Score, rank: Option<usize>, row: &Row) {
    match rank {
        Some(at) => {
            for k in at..DEPTH {
                score.recall[k] += 1.0;
            }
            score.mrr += 1.0 / (at + 1) as f64;
            score.top1 += f64::from(u8::from(at == 0));
            score.correct.push(at == 0);
            if at != 0 {
                score.reorderable.push(row.text.clone());
            }
        }
        None => {
            score.correct.push(false);
            score.unreachable.push(row.text.clone());
        }
    }
}

/// Turn the totals into shares.
fn finish(score: &mut Score, destinations: usize, refusals: (usize, usize)) {
    let asked = score.asked as f64;
    if asked > 0.0 {
        score.intent /= asked;
    }
    if destinations > 0 {
        let n = destinations as f64;
        score.top1 /= n;
        score.mrr /= n;
        for value in &mut score.recall {
            *value /= n;
        }
    }
    score.refused = if refusals.1 > 0 {
        refusals.0 as f64 / refusals.1 as f64
    } else {
        0.0
    };
}
