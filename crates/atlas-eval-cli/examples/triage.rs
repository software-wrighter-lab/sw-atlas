//! Which rows need the owner's eye, and which the drafter can own.
//!
//! Triage, not truth: a row is flagged when the matcher ranks some *other*
//! resource above the expected one and that other resource shares a concept
//! with it -- evidence that the corpus offers a sibling the key might have
//! preferred. Rows carrying a drafter's doubt note are flagged too.

use atlas_match::{Matcher, Variant};
use std::collections::BTreeMap;

/// What the shortlist is and is not.
const HEADER: &str = "\
# Rows that need the owner's eye
#
# Triage, not truth. A row is listed because the corpus offers a sibling the
# key might have preferred -- the matcher ranks another resource above the
# expected one and the two share a concept -- or because the drafter left a
# doubt note. Everything not listed had one plausible answer, and the drafter
# owns those: confirming them is mechanical rather than a judgement.
#
# Row numbers match sources/questions/<set>.ron and the review form. Mark in
# the form (sources/questions/REVIEW.txt), not here.
";

fn main() {
    let text = std::fs::read_to_string("build/corpus/corpus.ron").expect("run just ingest");
    let corpus: atlas_corpus::Corpus = ron::from_str(&text).expect("parses");
    let concepts: BTreeMap<&str, Vec<&str>> = corpus
        .resources
        .iter()
        .map(|r| {
            (
                r.id.as_str(),
                r.concepts.iter().map(|c| c.as_str()).collect(),
            )
        })
        .collect();
    let matcher = Matcher::new(&corpus, Variant::Text);
    let mut out = String::from(HEADER);
    for name in ["paraphrase", "intent", "exact-name", "follow-up"] {
        let path = format!("sources/questions/{name}.ron");
        let set = atlas_questions::QuestionSet::load(std::path::Path::new(&path)).expect("parses");
        let mut listed = Vec::new();
        let mut owned = 0;
        for (n, row) in set.rows.iter().enumerate() {
            let mut why = Vec::new();
            if !row.note.is_empty() {
                why.push(format!("drafter's note: {}", row.note));
            }
            if let Some(want) = row.expect.first() {
                let top = matcher.answer(&row.text, 1).hits;
                if let Some(hit) = top.first() {
                    let shares = concepts
                        .get(hit.id.as_str())
                        .zip(concepts.get(want.as_str()))
                        .is_some_and(|(a, b)| a.iter().any(|c| b.contains(c)));
                    if hit.id != *want && shares {
                        why.push(format!("a sibling outranks the key: {}", hit.id));
                    }
                }
            }
            if why.is_empty() {
                owned += 1;
            } else {
                listed.push(format!(
                    "{:>3} :: {} :: {}\n      {}\n",
                    n + 1,
                    row.text,
                    row.expect.first().map_or("(open)", String::as_str),
                    why.join("; ")
                ));
            }
        }
        out.push_str(&format!(
            "\n{}\n# {name}: {} need a look, {owned} the drafter owns\n{}\n",
            "=".repeat(70),
            listed.len(),
            "=".repeat(70)
        ));
        out.push_str(&listed.concat());
        println!(
            "{name:12} {:>4} need a look, {owned:>4} the drafter can own",
            listed.len()
        );
    }
    std::fs::write("sources/questions/NEEDS-REVIEW.txt", out).expect("writable");
    println!("\nwrote sources/questions/NEEDS-REVIEW.txt");
}
