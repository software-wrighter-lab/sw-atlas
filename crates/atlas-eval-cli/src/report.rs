//! The table a person reads, and the rows a machine keeps.

use atlas_eval::Score;
use atlas_eval::metrics::{interval, mcnemar};

/// One arm's line in the report.
pub fn line(name: &str, score: &Score) -> String {
    format!(
        "| {name} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} / {:.3} | {:.2} |\n",
        score.asked,
        score.top1,
        score.at(3),
        score.at(5),
        score.at(20),
        score.mrr,
        score.intent,
        score.intent_baseline,
        score.refused
    )
}

/// Every arm's line, then each arm against the one before it.
pub fn table(scored: &[(&str, Score)]) -> String {
    let columns = "| Arm | Asked | @1 | @3 | @5 | @20 | MRR | Intent / baseline | Refused |";
    let mut out = format!("{columns}\n|---|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for (name, score) in scored {
        out.push_str(&line(name, score));
    }
    out.push('\n');
    for pair in scored.windows(2) {
        out.push_str(&margin((pair[1].0, &pair[1].1), (pair[0].0, &pair[0].1)));
    }
    out
}

/// Two arms compared on the same questions, with the paired statistics that
/// say whether the difference is worth reading.
pub fn margin(a: (&str, &Score), b: (&str, &Score)) -> String {
    let (wins, losses, p) = mcnemar(&a.1.correct, &b.1.correct);
    let (low, high) = interval(&a.1.correct, &b.1.correct, 2000);
    let difference = a.1.top1 - b.1.top1;
    format!(
        "{} against {}: {:+.3} at rank 1 (95% interval {:+.3} to {:+.3}), \
         {wins} questions won and {losses} lost, McNemar p = {p:.3}\n",
        a.0, b.0, difference, low, high
    )
}

/// Where an arm's failures live: what a reranker could fix, and what it could
/// not because the answer was never proposed.
///
/// The split is the whole argument about where to spend effort. A question
/// whose answer sits at rank 4 is a ranking problem; one whose answer is not in
/// the candidates at all is a recall problem, and no model downstream of the
/// candidates can touch it.
pub fn misses(score: &Score, show: usize) -> String {
    let reorderable = score.reorderable.len();
    let unreachable = score.unreachable.len();
    let mut out = format!(
        "misses: {reorderable} found but not first (a reranker's opportunity), \
         {unreachable} never proposed at all (a recall problem)\n"
    );
    for text in score.unreachable.iter().take(show) {
        out.push_str(&format!("  never proposed: {text}\n"));
    }
    out
}
