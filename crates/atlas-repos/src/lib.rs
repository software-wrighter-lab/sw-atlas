//! Read the committed GitHub repository cache into a corpus.
//!
//! The cache (`cache/github-repos.json`, refreshed deliberately by
//! `scripts/fetch-repos`) is the only network-derived input in the build.
//! Reading it rather than the API keeps every rebuild reproducible and
//! offline, and means a corpus cannot change because someone pushed.
//!
//! Metadata only. The claim this supports is "I know about this
//! repository", never "I know what is in it": no source file is read.
//! A fork is included only when the owner has written about it -- when a
//! blog post or a campus place links to it. Any other fork is someone
//! else's work sitting in an owner's account, and is excluded. (Owner
//! decision, 2026-09-22.) Nothing marks an included fork as one: the flag
//! only decides inclusion, and the cache does not record the upstream a
//! visitor-facing "fork of X" would need.

pub mod record;

use atlas_core::{Concept, ResourceId, ResourceKind};
use atlas_corpus::{Corpus, content_hash};
use std::collections::BTreeSet;
use std::path::Path;

/// Why reading the cache failed.
#[derive(Debug)]
pub enum Error {
    /// The cache could not be read.
    Io(std::io::Error),
    /// The cache is not the JSON array `scripts/fetch-repos` writes.
    Cache {
        /// What the parser said.
        cause: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "reading the repository cache: {error}"),
            Self::Cache { cause } => write!(f, "repository cache: {cause}"),
        }
    }
}

impl std::error::Error for Error {}

/// Read the cache into a corpus of repositories and the demos they name.
///
/// `declared` is every repository the owner has written about, from
/// [`declared`]; it decides which forks are kept.
///
/// # Errors
///
/// Fails if the cache cannot be read or any entry lacks a required field.
/// An entry this cannot read is a bug here, not an entry to skip.
pub fn ingest(cache: &Path, declared: &BTreeSet<ResourceId>) -> Result<Corpus, Error> {
    let text = std::fs::read_to_string(cache).map_err(Error::Io)?;
    let values: Vec<serde_json::Value> = serde_json::from_str(&text).map_err(cause)?;
    let mut corpus = Corpus::new();
    for value in values {
        let hash = content_hash(value.to_string().as_bytes());
        let record: record::Record = serde_json::from_value(value).map_err(cause)?;
        let resource = record.resource(hash);
        if record.fork && !declared.contains(&resource.id) {
            continue;
        }
        corpus.resources.extend(record.demo().map(|(demo, _)| demo));
        corpus.relations.extend(record.demo().map(|(_, edge)| edge));
        corpus.concepts.extend(
            record
                .concepts()
                .iter()
                .map(|c| Concept::provisional(c.as_str())),
        );
        corpus.resources.push(resource);
    }
    corpus.concepts.sort_by(|a, b| a.id.cmp(&b.id));
    corpus.concepts.dedup_by(|a, b| a.id == b.id);
    Ok(corpus)
}

/// A parser error, kept as text so the error type stays simple.
fn cause(error: serde_json::Error) -> Error {
    Error::Cache {
        cause: error.to_string(),
    }
}

/// Every repository the given corpora link to: the ones the owner has
/// written about.
pub fn declared(corpora: &[Corpus]) -> BTreeSet<ResourceId> {
    corpora
        .iter()
        .flat_map(|c| c.resources.iter())
        .filter(|r| r.kind == ResourceKind::Repo)
        .map(|r| r.id.clone())
        .collect()
}
