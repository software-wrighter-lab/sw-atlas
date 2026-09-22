//! "Atlas knows about everything" as a number, with a gate behind it.
//!
//! Two questions, asked of the corpus rather than of anyone's memory. Is
//! every artifact *reachable* -- can concepts and relations lead a visitor
//! to it, or does it merely exist in a file? And does every URL still
//! resolve? Either answer failing stops a publish, because an index that
//! cannot reach a resource is not indexing it, and a link that 404s is
//! worse than no link.
//!
//! Counted per kind, so a shortfall says where it is. Everything else the
//! report prints -- tagged, related, excluded, unchecked -- is context for
//! reading the two gated numbers honestly.

pub mod markdown;
pub mod urls;

use atlas_core::{Resource, ResourceKind};
use atlas_corpus::Corpus;
use atlas_links::Cache;

/// One kind of artifact, counted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kind {
    /// Which kind.
    pub kind: ResourceKind,
    /// How many are in the corpus.
    pub total: usize,
    /// How many carry at least one concept.
    pub tagged: usize,
    /// How many are an endpoint of at least one relation.
    pub related: usize,
    /// How many have no concept and no relation: unreachable.
    pub orphans: Vec<String>,
}

/// What a coverage run found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coverage {
    /// Per kind, in the order the report prints them.
    pub kinds: Vec<Kind>,
    /// Concepts that no resource uses, which are dead vocabulary.
    pub unused_concepts: Vec<String>,
    /// What the last link check found.
    pub links: urls::Stats,
    /// Repositories excluded by name as not artifacts.
    pub excluded: usize,
}

/// The kinds the report prints, in that order.
const KINDS: [ResourceKind; 6] = [
    ResourceKind::Post,
    ResourceKind::Campus,
    ResourceKind::Repo,
    ResourceKind::Video,
    ResourceKind::Demo,
    ResourceKind::Paper,
];

/// Measure a corpus against the committed URL results.
pub fn measure(corpus: &Corpus, cache: &Cache, excluded: usize) -> Coverage {
    Coverage {
        kinds: KINDS.iter().map(|kind| count(corpus, *kind)).collect(),
        unused_concepts: unused(corpus),
        links: urls::stats(corpus, cache),
        excluded,
    }
}

/// Count one kind, and name whatever nothing can reach.
fn count(corpus: &Corpus, kind: ResourceKind) -> Kind {
    let reaches = |r: &Resource| {
        corpus
            .relations
            .iter()
            .any(|edge| edge.from == r.id || edge.to == r.id)
    };
    let of_kind: Vec<&Resource> = corpus.resources.iter().filter(|r| r.kind == kind).collect();
    Kind {
        kind,
        total: of_kind.len(),
        tagged: of_kind.iter().filter(|r| !r.concepts.is_empty()).count(),
        related: of_kind.iter().filter(|r| reaches(r)).count(),
        orphans: of_kind
            .iter()
            .filter(|r| r.concepts.is_empty() && !reaches(r))
            .map(|r| r.id.as_str().to_string())
            .collect(),
    }
}

/// Concepts no resource carries.
fn unused(corpus: &Corpus) -> Vec<String> {
    corpus
        .concepts
        .iter()
        .filter(|c| !corpus.resources.iter().any(|r| r.concepts.contains(&c.id)))
        .map(|c| c.id.as_str().to_string())
        .collect()
}

impl Coverage {
    /// Why a publish must not happen, in the order a person should read it.
    ///
    /// Empty means both gates pass. Only two things are gated: a resource
    /// nothing can reach, and a URL that is gone. A blocked host, a network
    /// error and an unchecked URL are all reported and none of them is this
    /// corpus being wrong.
    pub fn failures(&self) -> Vec<String> {
        let orphans: Vec<&String> = self.kinds.iter().flat_map(|k| &k.orphans).collect();
        let mut out = Vec::new();
        if !orphans.is_empty() {
            let names: Vec<&str> = orphans.iter().take(5).map(|s| s.as_str()).collect();
            let count = orphans.len();
            out.push(format!(
                "{count} unreachable resources, first: {}",
                names.join(", ")
            ));
        }
        if !self.links.missing.is_empty() {
            let count = self.links.missing.len();
            out.push(format!(
                "{count} dead URLs, first: {}",
                self.links.missing[0]
            ));
        }
        out
    }
}
