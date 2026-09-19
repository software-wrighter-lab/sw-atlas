//! The canned stories a place can tell.
//!
//! These are answer material, never training text. The docent reads one
//! aloud on arrival; nothing here is ever a label a model is fitted to.
//! The distinction lives in the type: they land in [`Resource::body`],
//! which is documented as answer material, while the questions a model
//! trains on are `atlas_decide::Question` in a different crate entirely.

use crate::snapshot::Snapshot;
use std::collections::BTreeMap;

/// Each place's stories, gathered into one block of authored text.
///
/// The kind (arrival, detail, feature, anecdote) and the title are kept as
/// headings rather than dropped, because the story-selection policy needs
/// to tell an arrival story from an anecdote, and a heading survives every
/// later step without inventing a type for it now.
pub fn bodies(snapshot: &Snapshot) -> BTreeMap<String, String> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for (index, place) in snapshot.story_places.iter().enumerate() {
        let kind = snapshot.story_kinds.get(index).cloned().unwrap_or_default();
        let title = snapshot
            .story_titles
            .get(index)
            .cloned()
            .unwrap_or_default();
        let text = snapshot.story_texts.get(index).cloned().unwrap_or_default();
        let entry = out.entry(place.clone()).or_default();
        if !entry.is_empty() {
            entry.push_str("\n\n");
        }
        entry.push_str(&format!("## [{kind}] {title}\n{text}"));
    }
    out
}
