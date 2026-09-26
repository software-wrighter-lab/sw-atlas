//! The sentences a visitor sees. Every one is a fixed frame with catalog text
//! quoted into it, so the docent cannot say anything the corpus did not
//! already contain.

use crate::policy::{Outcome, Reply};
use atlas_core::Resource;
use atlas_corpus::Corpus;

/// One resource as a line a visitor can act on: what it is called, what it is,
/// and where it lives.
fn entry(resource: &Resource) -> String {
    let name = if resource.title.is_empty() {
        resource.id.as_str()
    } else {
        &resource.title
    };
    let summary = resource
        .summary
        .split_whitespace()
        .take(24)
        .collect::<Vec<_>>()
        .join(" ");
    match (summary.is_empty(), resource.url.is_empty()) {
        (true, true) => format!("  {name}\n"),
        (true, false) => format!("  {name}\n    {}\n", resource.url),
        (false, true) => format!("  {name}\n    {summary}\n"),
        (false, false) => format!("  {name}\n    {summary}\n    {}\n", resource.url),
    }
}

/// The opening sentence for an outcome. The only place the docent's voice
/// lives, so a change of tone is a change to one function.
fn lead(outcome: &Outcome, offered: usize) -> String {
    match outcome {
        Outcome::One => "That is almost certainly this:\n".to_string(),
        Outcome::Several if offered == 1 => "The closest thing I have is:\n".to_string(),
        Outcome::Several => format!("I have {offered} things close to that:\n"),
        Outcome::NotYet(text) => format!("{text}\n"),
        Outcome::Rephrase => {
            "I do not recognise enough of those words to place them. Could you say it \
             another way, or name a subject?\n"
                .to_string()
        }
        Outcome::NothingHere => "I know those words, and nothing here is about that.\n".to_string(),
    }
}

/// The whole reply as prose, frames only.
pub fn render(reply: &Reply, corpus: &Corpus, suggestions: &[String]) -> String {
    let named: Vec<&Resource> = reply
        .offer
        .iter()
        .filter_map(|id| corpus.resources.iter().find(|r| r.id.as_str() == id))
        .collect();
    let lead = lead(&reply.outcome, named.len());
    let body: String = named.iter().map(|r| entry(r)).collect();
    let tail = if suggestions.is_empty() {
        String::new()
    } else {
        format!(
            "\nYou could ask:\n{}",
            suggestions
                .iter()
                .map(|s| format!("  {s}\n"))
                .collect::<String>()
        )
    };
    format!("{lead}{body}{tail}")
}

/// Whether a resource is real but unfinished, and what to say about it.
pub fn unfinished(corpus: &Corpus, id: &str) -> Option<String> {
    let resource = corpus.resources.iter().find(|r| r.id.as_str() == id)?;
    let name = if resource.title.is_empty() {
        resource.id.as_str()
    } else {
        &resource.title
    };
    match resource.maturity {
        Some(atlas_core::Maturity::Planned) => {
            Some(format!("{name} is on the map and not built yet."))
        }
        Some(atlas_core::Maturity::Early) => Some(format!(
            "{name} exists as source, with nothing runnable yet."
        )),
        _ => None,
    }
}
