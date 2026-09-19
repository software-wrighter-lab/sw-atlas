//! The cross-corpus links a post declares, and the resources they name.
//!
//! Every edge leaves the post: the post is what the author was writing
//! when they declared the link, so it is the end that carries the intent.
//! All of them are [`Provenance::Declared`] -- a person wrote them at
//! publication time, and this ingester does not second-guess them.

use crate::frontmatter::{FrontMatter, Titled};
use atlas_core::{Provenance, Relation, RelationKind, Resource, ResourceId, ResourceKind};

/// One declared link, normalised out of the six front matter fields that
/// can carry one.
pub struct Link {
    /// What sort of thing it points at.
    pub kind: ResourceKind,
    /// Where it points.
    pub url: String,
    /// What the author called it, where they said.
    pub title: Option<String>,
}

/// Every link a post declares, in one list.
pub fn links(front: &FrontMatter) -> Vec<Link> {
    let single = |kind, url: &Option<String>, title: Option<String>| {
        url.as_ref().map(|u| Link {
            kind,
            url: u.clone(),
            title,
        })
    };
    let mut out: Vec<Link> = Vec::new();
    out.extend(single(ResourceKind::Repo, &front.repo_url, None));
    let video_title = front.video_title.clone();
    out.extend(single(ResourceKind::Video, &front.video_url, video_title));
    out.extend(single(ResourceKind::Demo, &front.demo_url, None));
    out.extend(listed(ResourceKind::Repo, &front.repo_urls, &[]));
    let titles = &front.video_titles;
    out.extend(listed(ResourceKind::Video, &front.video_urls, titles));
    out.extend(listed(ResourceKind::Paper, &front.papers, &[]));
    out
}

/// A list-valued field's links, with a parallel title list where the blog
/// writes one instead of naming each entry inline.
fn listed(kind: ResourceKind, items: &[Titled], names: &[String]) -> Vec<Link> {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| Link {
            kind,
            url: item.url().to_string(),
            title: item.title().or_else(|| names.get(i).cloned()),
        })
        .collect()
}

/// The identifier a link's target gets.
///
/// Derived from the URL so that two posts naming the same repository,
/// video or paper produce one resource rather than two.
pub fn id(link: &Link) -> ResourceId {
    let trimmed = link
        .url
        .trim_end_matches('/')
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");
    let (prefix, tail) = match link.kind {
        ResourceKind::Repo => ("repo", trimmed.trim_start_matches("github.com/")),
        ResourceKind::Video => (
            "video",
            trimmed.rsplit(['/', '=']).next().unwrap_or(trimmed),
        ),
        ResourceKind::Demo => ("demo", trimmed),
        _ => ("paper", trimmed),
    };
    ResourceId::new(format!("{prefix}:{tail}"))
}

/// The resource a link names and the edge that reaches it.
///
/// The target is known only by what the author said about it: a title if
/// they gave one, the URL, and nothing else. Whatever ingests that kind of
/// resource from its own source later fills in the rest.
pub fn edge(from: &ResourceId, link: &Link) -> (Resource, Relation) {
    let kind = match link.kind {
        ResourceKind::Repo => RelationKind::Implements,
        ResourceKind::Video | ResourceKind::Demo => RelationKind::Demos,
        ResourceKind::Paper => RelationKind::Cites,
        _ => RelationKind::RelatedTo,
    };
    let to = id(link);
    let title = link.title.clone().unwrap_or_default();
    let target = Resource::stub(to.clone(), link.kind, title, link.url.clone());
    let relation = Relation {
        from: from.clone(),
        kind,
        to,
        weight: 1.0,
        provenance: Provenance::Declared,
    };
    (target, relation)
}
