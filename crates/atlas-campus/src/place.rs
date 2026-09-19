//! Places, their URLs, and the links they carry.

use crate::snapshot::{Snapshot, split};
use atlas_core::{
    ConceptId, Link, Provenance, Relation, RelationKind, Resource, ResourceId, ResourceKind, edge,
};

/// The URL a place resolves to, built from its ancestors.
///
/// `apl` sits under `language-experiments` under `computer-science` under
/// the campus, so its URL is the base plus that path. The campus itself is
/// the base.
pub fn url(snapshot: &Snapshot, index: usize) -> String {
    let mut path: Vec<&str> = Vec::new();
    let mut at = index;
    loop {
        let parent = &snapshot.parents[at];
        if parent.is_empty() {
            break;
        }
        path.push(&snapshot.ids[at]);
        match snapshot.ids.iter().position(|id| id == parent) {
            Some(next) => at = next,
            None => break,
        }
    }
    path.reverse();
    if path.is_empty() {
        snapshot.url_base.clone()
    } else {
        format!("{}/{}", snapshot.url_base, path.join("/"))
    }
}

/// One place as a resource.
///
/// The place kind -- campus, building, wing, exhibit -- is not stored: it
/// equals the depth of the `PartOf` chain, which a test asserts, so
/// recording it as well would be recording the same fact twice.
pub fn resource(snapshot: &Snapshot, index: usize, body: String) -> Resource {
    Resource {
        id: ResourceId::new(format!("campus:{}", snapshot.ids[index])),
        kind: ResourceKind::Campus,
        title: snapshot.titles[index].clone(),
        url: url(snapshot, index),
        date: String::new(),
        summary: snapshot.taglines[index].clone(),
        body,
        concepts: split(&snapshot.concepts[index])
            .into_iter()
            .map(ConceptId::new)
            .collect(),
        aliases: split(&snapshot.aliases[index]),
        maturity: None,
        source_hash: String::new(),
    }
}

/// The edge from a place to the place that contains it.
pub fn part_of(snapshot: &Snapshot, index: usize) -> Option<Relation> {
    let parent = &snapshot.parents[index];
    (!parent.is_empty()).then(|| Relation {
        from: ResourceId::new(format!("campus:{}", snapshot.ids[index])),
        kind: RelationKind::PartOf,
        to: ResourceId::new(format!("campus:{parent}")),
        weight: 1.0,
        provenance: Provenance::Declared,
    })
}

/// The demos and repositories a place links to.
///
/// A link labelled `source` names a repository; anything else runnable is
/// a demo. Both are stubs: whatever reads a repository from its own source
/// later fills in the rest, and the identifier is what joins the two.
pub fn links(snapshot: &Snapshot, index: usize) -> Vec<(Resource, Relation)> {
    let from = ResourceId::new(format!("campus:{}", snapshot.ids[index]));
    split(&snapshot.links[index])
        .into_iter()
        .filter_map(|pair| {
            let (label, url) = pair.split_once('=')?;
            let repo = label.contains("source");
            let kind = if repo {
                ResourceKind::Repo
            } else {
                ResourceKind::Demo
            };
            let link = Link {
                kind,
                url: url.to_string(),
                title: Some(label.to_string()),
            };
            Some(edge(&from, &link))
        })
        .collect()
}
