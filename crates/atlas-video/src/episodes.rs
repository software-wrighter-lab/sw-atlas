//! The Five ML Concepts series: the one join that may be code.
//!
//! Episode N was produced in `projects/5MLC-N`, except episode 1, which
//! predates the naming and lives in `projects/daily5-20260203`. The rule
//! was stated by the owner in step 011's prompt; it is not a title match
//! that happens to work, and nothing else in this crate matches titles.

use std::collections::BTreeMap;

/// The episode number a published title carries, if it is one of the
/// series. The blog writes both `Five ML Concepts - #29` and
/// `Five ML Concepts - 30`.
pub fn episode(title: &str) -> Option<u32> {
    let rest = title.strip_prefix("Five ML Concepts - ")?;
    rest.trim().trim_start_matches('#').parse().ok()
}

/// The shorts project an episode was produced in.
pub fn project(episode: u32) -> String {
    match episode {
        1 => "daily5-20260203".to_string(),
        n => format!("5MLC-{n}"),
    }
}

/// Concept labels per episode, from `docs/concepts-status.txt`.
///
/// Every line of the form `Label (Ep N)` is a declared edge written by the
/// person who made the episode. Labels are kept exactly as written --
/// `Few-shot / Zero-shot` stays one label -- because splitting them is a
/// judgement, and judgements belong to the concept graph step and its
/// override file.
pub fn concepts(text: &str) -> BTreeMap<u32, Vec<String>> {
    let mut out: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    for (label, number) in text.lines().filter_map(line) {
        out.entry(number).or_default().push(label);
    }
    out
}

/// One `Label (Ep N)` line, or nothing.
fn line(text: &str) -> Option<(String, u32)> {
    let (label, tail) = text.trim().rsplit_once(" (Ep ")?;
    let number = tail.strip_suffix(')')?.parse().ok()?;
    Some((label.trim().to_string(), number))
}
