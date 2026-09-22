//! Published videos, joined to what was said in them.
//!
//! Two sources, each the authority for one thing. The blog's `video_url`
//! and `video_title` fields are the only complete record of *which* videos
//! are published and *where*; `~/github/softwarewrighter/shorts` holds
//! *what was said*, as authored scripts under version control. Neither
//! records the other's key, so the join is the rule in [`episodes`] plus a
//! hand-written map read by [`join`]. Nothing matches a title.
//!
//! A video with no known project is still a resource, with an empty body
//! -- empty rather than absent, so a later join fills it with no change to
//! the type. See `docs/video-sources.md`.

pub mod episodes;
pub mod join;
pub mod shorts;

use atlas_core::{Concept, Resource, ResourceKind};
use atlas_corpus::{Corpus, content_hash};
use std::collections::BTreeMap;
use std::path::Path;

/// Why joining the videos failed.
#[derive(Debug)]
pub enum Error {
    /// A file the join needs could not be read.
    Io(std::io::Error),
    /// The join map is not valid RON.
    Map(String),
    /// Confirmed rows naming videos no post declares.
    Stale(Vec<String>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "reading shorts: {error}"),
            Self::Map(cause) => write!(f, "video join map: {cause}"),
            Self::Stale(urls) => write!(f, "join map rows no post declares: {urls:?}"),
        }
    }
}

impl std::error::Error for Error {}

/// Where the second source and the map are.
pub struct Sources<'a> {
    /// A checkout of the shorts repository.
    pub shorts: &'a Path,
    /// `sources/video-shorts.ron`.
    pub map: &'a Path,
}

/// Every video the blog declares, with its script where one is known.
///
/// # Errors
///
/// Fails if the map does not parse, names a video no post declares, or
/// names a project the shorts checkout does not have.
pub fn ingest(blog: &Corpus, sources: &Sources) -> Result<Corpus, Error> {
    let map = join::JoinMap::load(sources.map).map_err(Error::Map)?;
    let status = sources.shorts.join("docs/concepts-status.txt");
    let labels = episodes::concepts(&std::fs::read_to_string(status).map_err(Error::Io)?);
    let videos: Vec<&Resource> = blog
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Video)
        .collect();
    let stale = map.unknown(&videos);
    if !stale.is_empty() {
        return Err(Error::Stale(stale));
    }
    let mut corpus = Corpus::new();
    for video in videos {
        let (resource, concepts) = enrich(blog, video, (&map, &labels), sources.shorts)?;
        corpus.resources.push(resource);
        corpus.concepts.extend(concepts);
    }
    corpus.concepts.sort_by(|a, b| a.id.cmp(&b.id));
    corpus.concepts.dedup_by(|a, b| a.id == b.id);
    Ok(corpus)
}

/// One video, with its date, script and declared concepts.
fn enrich(
    blog: &Corpus,
    video: &Resource,
    (map, labels): (&join::JoinMap, &BTreeMap<u32, Vec<String>>),
    shorts: &Path,
) -> Result<(Resource, Vec<Concept>), Error> {
    let mut out = video.clone();
    out.date = declared(blog, video);
    if let Some(project) = map.project(video) {
        let script = shorts::read(&shorts.join("projects").join(project)).map_err(Error::Io)?;
        (out.summary, out.body) = (script.summary, script.body);
    }
    let episode = episodes::episode(&video.title).and_then(|n| labels.get(&n));
    let concepts: Vec<Concept> = episode
        .into_iter()
        .flatten()
        .map(|l| Concept::provisional(l))
        .collect();
    out.concepts = concepts.iter().map(|c| c.id.clone()).collect();
    let facts = [&out.url, &out.title, &out.date, &out.summary, &out.body];
    out.source_hash = content_hash(facts.map(String::as_str).join("\n").as_bytes());
    Ok((out, concepts))
}

/// The date the blog first declared a video: the earliest post linking to
/// it. The video's own publication date is not recorded anywhere this can
/// read, and this is labelled as what it is rather than passed off as one.
fn declared(blog: &Corpus, video: &Resource) -> String {
    let posts = blog.relations.iter().filter(|r| r.to == video.id);
    posts
        .filter_map(|r| blog.resources.iter().find(|p| p.id == r.from))
        .map(|p| p.date.clone())
        .filter(|d| !d.is_empty())
        .min()
        .unwrap_or_default()
}
