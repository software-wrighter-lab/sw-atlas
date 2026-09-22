//! The cross-corpus links a post declares.
//!
//! Every edge leaves the post: the post is what the author was writing
//! when they declared the link, so it is the end that carries the intent.
//! All of them are [`atlas_core::Provenance::Declared`] -- a person wrote
//! them at publication time, and this reader does not second-guess them.
//!
//! Turning a link into a resource and an edge is [`atlas_core::edge`],
//! shared with the campus reader so that both produce the same identifier
//! for the same repository.

use crate::frontmatter::{FrontMatter, Titled};
use atlas_core::{Link, ResourceKind};

/// Every link a post declares, in one list.
///
/// An empty URL is how the blog writes "no link here" (`video_url: ""`),
/// so it declares nothing rather than a resource with no address.
pub fn links(front: &FrontMatter) -> Vec<Link> {
    let single = |kind, url: &Option<String>, title: Option<String>| {
        url.as_ref().filter(|u| !u.trim().is_empty()).map(|u| Link {
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
        .filter(|(_, item)| !item.url().trim().is_empty())
        .map(|(i, item)| Link {
            kind,
            url: item.url().to_string(),
            title: item.title().or_else(|| names.get(i).cloned()),
        })
        .collect()
}
