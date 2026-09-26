//! The outcome distribution over the frozen question sets.
//!
//! Not an accuracy: it says how often the policy takes each of its five
//! escape hatches, and how often the thing it offered was the expected
//! destination. A policy that answered NOTHING HERE to everything would be
//! perfectly precise and useless, so the share column is half the number.

use crate::Docent;
use crate::voice::label;
use atlas_answer::decide;
use std::collections::BTreeMap;

/// How often each outcome fires over every confirmed row, and how right the
/// offer is when it does.
pub fn report(docent: &Docent) -> String {
    let mut counts: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for name in [
        "paraphrase",
        "exact-name",
        "ambiguous",
        "follow-up",
        "off-topic",
        "meta",
        "intent",
    ] {
        tally(docent, name, &mut counts);
    }
    table(&counts)
}

/// The counts as a markdown table: how often each outcome fires, and how often
/// what it offered held the expected destination.
fn table(counts: &BTreeMap<&str, (usize, usize)>) -> String {
    let total: usize = counts.values().map(|(n, _)| n).sum();
    let mut out = format!(
        "over {total} confirmed questions\n\n| Outcome | Fires | Share | Offer contains the expected answer |\n|---|---:|---:|---:|\n"
    );
    for (label, (n, hits)) in counts {
        let share = *n as f64 / total as f64;
        let precision = *hits as f64 / *n as f64;
        out.push_str(&format!(
            "| {label} | {n} | {share:.3} | {precision:.3} |\n"
        ));
    }
    out
}

/// The confirmed rows of a set.
fn confirmed(set: &atlas_questions::QuestionSet) -> Vec<&atlas_questions::Row> {
    set.rows
        .iter()
        .filter(|row| row.status == atlas_questions::Status::Confirmed)
        .collect()
}

/// One question set's rows added to the running counts.
fn tally(docent: &Docent, name: &str, counts: &mut BTreeMap<&'static str, (usize, usize)>) {
    let path = format!("sources/questions/{name}.ron");
    let Ok(set) = atlas_questions::QuestionSet::load(std::path::Path::new(&path)) else {
        return;
    };
    for row in confirmed(&set) {
        let answer = docent.matcher.answer(&row.text, 10);
        let reply = decide(
            &row.text,
            &answer,
            &docent.corpus,
            (&docent.policy, &docent.absent),
        );
        let hit = reply.offer.iter().any(|id| row.expect.contains(id));
        let entry = counts.entry(label(&reply.outcome)).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += usize::from(hit);
    }
}
