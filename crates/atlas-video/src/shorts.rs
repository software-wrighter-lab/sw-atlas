//! What was said in a video, from its project in the shorts repository.
//!
//! `shorts` keeps the text of every production in git on purpose and the
//! media out of it: `work/narration.{md,txt}` is the spoken script and
//! `work/description.{md,txt}` the published description. Both are
//! authored prose, not transcripts, so they are read verbatim.

use std::path::Path;

/// The authored text of one production.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Script {
    /// The description's first paragraph: the published one-line blurb.
    pub summary: String,
    /// The narration, or the whole description where no narration was
    /// kept. Answer material, never training text.
    pub body: String,
}

/// Read a project directory.
///
/// # Errors
///
/// Fails if the directory does not exist: a project the join names and
/// the checkout lacks means the checkout is stale or the map has a typo,
/// and either should stop the build rather than drop a script silently.
pub fn read(project: &Path) -> std::io::Result<Script> {
    if !project.is_dir() {
        let what = format!("no shorts project at {}", project.display());
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, what));
    }
    let work = project.join("work");
    let description = first(&work, &["description.md", "description.txt"]);
    let narration = first(&work, &["narration.md", "narration.txt"]);
    Ok(Script {
        summary: paragraph(&description),
        body: narration.unwrap_or(description.unwrap_or_default()),
    })
}

/// The first of several files that exists, trimmed.
fn first(dir: &Path, names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| std::fs::read_to_string(dir.join(name)).ok())
        .map(|text| text.trim().to_string())
}

/// The first paragraph of a text, on one line.
fn paragraph(text: &Option<String>) -> String {
    let text = text.as_deref().unwrap_or_default();
    let first = text.split("\n\n").next().unwrap_or_default();
    first.split_whitespace().collect::<Vec<_>>().join(" ")
}
