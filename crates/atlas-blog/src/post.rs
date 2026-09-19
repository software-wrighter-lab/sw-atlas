//! A post, and the vocabulary it declares.

use crate::frontmatter::FrontMatter;
use atlas_core::{Concept, ConceptId, Resource, ResourceId};

/// The dated slug a Jekyll filename carries, without its extension.
pub fn stem(path: &std::path::Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The published URL a dated slug resolves to.
///
/// `2026-09-17-tbt-apl-360-revisited` becomes
/// `https://blog.softwarewrighter.com/2026/09/17/tbt-apl-360-revisited/`.
pub fn url(stem: &str) -> String {
    let parts: Vec<&str> = stem.splitn(4, '-').collect();
    match parts.as_slice() {
        [year, month, day, slug] => {
            format!("https://blog.softwarewrighter.com/{year}/{month}/{day}/{slug}/")
        }
        _ => String::new(),
    }
}

/// One post as a resource, with its authored aliases.
pub fn resource(front: &FrontMatter, stem: &str, source_hash: String) -> Resource {
    let aliases = front
        .keywords
        .iter()
        .flat_map(|k| k.split(','))
        .map(|a| a.trim().to_lowercase())
        .filter(|a| !a.is_empty())
        .collect();
    Resource {
        id: ResourceId::new(format!("blog:{stem}")),
        kind: atlas_core::ResourceKind::Post,
        title: front.title.clone(),
        url: url(stem),
        date: stem.get(..10).unwrap_or_default().to_string(),
        summary: front.abstract_field.clone().unwrap_or_default(),
        body: String::new(),
        concepts: concepts(front).into_iter().map(|c| c.id).collect(),
        aliases,
        maturity: None,
        source_hash,
    }
}

/// A provisional concept record for every category and tag a post
/// declares.
///
/// Provisional because the concept graph step normalises and merges them:
/// `moe` and `mixture-of-experts` are one idea and a visitor would not
/// distinguish them. They are emitted now rather than later so the corpus
/// is valid at every step instead of only at the end.
pub fn concepts(front: &FrontMatter) -> Vec<Concept> {
    front
        .categories
        .iter()
        .chain(front.tags.iter())
        .map(|label| Concept {
            id: ConceptId::new(label.trim().to_lowercase()),
            label: label.trim().to_string(),
            aliases: Vec::new(),
            parents: Vec::new(),
            resources: Vec::new(),
        })
        .fold(Vec::new(), |mut acc: Vec<Concept>, c| {
            if !acc.iter().any(|seen| seen.id == c.id) {
                acc.push(c);
            }
            acc
        })
}
