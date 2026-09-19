//! Read a published campus catalog snapshot into a corpus.
//!
//! The campus is a hierarchy of places -- campus, building, wing, exhibit
//! -- each with a title, a tagline, aliases, concepts, links to what runs
//! there, and canned stories. All of it was written by hand, so all of it
//! is [`atlas_core::Provenance::Declared`].
//!
//! The source directory is read-only. Nothing here writes to it.

pub mod place;
pub mod snapshot;
pub mod story;

use atlas_core::{Concept, ConceptId, Resource};
use atlas_corpus::{Corpus, content_hash};
use std::path::Path;

/// Why reading a snapshot failed.
#[derive(Debug)]
pub enum Error {
    /// The file could not be read.
    Io(std::io::Error),
    /// The snapshot did not parse.
    Snapshot {
        /// Which file.
        path: String,
        /// What the parser said.
        cause: String,
    },
    /// Neither a published catalog nor a snapshot was found.
    NoSource {
        /// Where both were looked for.
        repo: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "reading the campus: {error}"),
            Self::Snapshot { path, cause } => write!(f, "{path}: {cause}"),
            Self::NoSource { repo } => write!(
                f,
                "{repo}: neither dist/catalog.json nor pages/docent/snapshot-a.json"
            ),
        }
    }
}

impl std::error::Error for Error {}

/// The catalog this run read, and where it came from.
///
/// sw-campus intends to publish `dist/catalog.json` canonically with its
/// SHA-256; until it does, the docent snapshot is the source. Which one
/// was used is recorded on every resource, so the switch is visible when
/// it happens rather than silent.
pub fn source(repo: &Path) -> Result<std::path::PathBuf, Error> {
    for candidate in ["dist/catalog.json", "pages/docent/snapshot-a.json"] {
        let path = repo.join(candidate);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(Error::NoSource {
        repo: repo.display().to_string(),
    })
}

/// Read a campus checkout into a corpus.
///
/// # Errors
///
/// Fails if no catalog is present, or if the one present does not parse.
pub fn ingest(repo: &Path) -> Result<Corpus, Error> {
    let path = source(repo)?;
    let text = std::fs::read_to_string(&path).map_err(Error::Io)?;
    let snapshot = snapshot::load(&path)?;
    let hash = content_hash(text.as_bytes());
    let bodies = story::bodies(&snapshot);
    let mut corpus = Corpus::new();
    for index in 0..snapshot.ids.len() {
        let body = bodies
            .get(&snapshot.ids[index])
            .cloned()
            .unwrap_or_default();
        let mut resource = place::resource(&snapshot, index, body);
        resource.source_hash = hash.clone();
        corpus.relations.extend(place::part_of(&snapshot, index));
        for (target, relation) in place::links(&snapshot, index) {
            corpus.resources.push(target);
            corpus.relations.push(relation);
        }
        corpus.concepts.extend(concepts(&resource));
        corpus.resources.push(resource);
    }
    Ok(corpus)
}

/// A provisional concept record for every concept a place declared.
fn concepts(resource: &Resource) -> Vec<Concept> {
    resource
        .concepts
        .iter()
        .map(|id| Concept {
            id: ConceptId::new(id.as_str()),
            label: id.as_str().to_string(),
            aliases: Vec::new(),
            parents: Vec::new(),
            resources: Vec::new(),
        })
        .collect()
}
