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
//!
//! That flag is not the same question as who wrote the code, and taking it
//! for the answer sent visitors to abandoned repositories: work moved into an
//! organisation by forking is marked as somebody else's forever, while the
//! copy left behind keeps the fork flag off. `sources/repo-canonical.ron`
//! names the pairs where that happened and says which copy is real; the
//! declarations and the corpus pass that applies them live in `atlas-supersede`.

pub mod exclusions;
pub mod record;

use atlas_core::{Concept, ResourceId, ResourceKind};
use atlas_corpus::{Corpus, content_hash};
use atlas_supersede::Canonical;
use exclusions::Exclusions;
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
/// [`declared`]; it decides which forks are kept. `rules` is the committed
/// pair: repositories that are not artifacts at all, and the declarations of
/// which copy is real where a repository exists twice.
///
/// # Errors
///
/// Fails if the cache cannot be read or any entry lacks a required field.
/// An entry this cannot read is a bug here, not an entry to skip.
pub fn ingest(
    cache: &Path,
    declared: &BTreeSet<ResourceId>,
    rules: (&Exclusions, &Canonical),
) -> Result<Corpus, Error> {
    let (_, canonical) = rules;
    let text = std::fs::read_to_string(cache).map_err(Error::Io)?;
    let read = |e: serde_json::Error| Error::Cache {
        cause: e.to_string(),
    };
    let values: Vec<serde_json::Value> = serde_json::from_str(&text).map_err(read)?;
    let mut corpus = Corpus::new();
    for value in values {
        let hash = content_hash(value.to_string().as_bytes());
        let record: record::Record = serde_json::from_value(value).map_err(read)?;
        let resource = record.resource(hash);
        if exclusions::wanted(&record, &resource.id, declared, rules) {
            keep(&mut corpus, &record, resource, canonical);
        }
    }
    corpus.concepts.sort_by(|a, b| a.id.cmp(&b.id));
    corpus.concepts.dedup_by(|a, b| a.id == b.id);
    Ok(corpus)
}

/// Add one repository, its concepts, the demo its homepage names, and the
/// names of any copy it supersedes, so the old name still reaches it.
fn keep(
    corpus: &mut Corpus,
    record: &record::Record,
    mut resource: atlas_core::Resource,
    canonical: &Canonical,
) {
    resource
        .aliases
        .extend(canonical.aliases(&record.full_name));
    resource.aliases.sort();
    resource.aliases.dedup();
    corpus.resources.extend(record.demo().map(|(demo, _)| demo));
    corpus.relations.extend(record.demo().map(|(_, edge)| edge));
    let concepts = record.concepts();
    corpus
        .concepts
        .extend(concepts.iter().map(|c| Concept::provisional(c.as_str())));
    corpus.resources.push(resource);
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
