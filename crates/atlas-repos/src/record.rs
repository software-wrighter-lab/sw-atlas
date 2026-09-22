//! One repository as the GitHub API describes it, and what it becomes.

use atlas_core::{ConceptId, Link, Relation, Resource, ResourceKind, edge, link_id};
use serde::Deserialize;

/// The fields `scripts/fetch-repos` keeps from the GitHub API.
///
/// Everything here was set by the owner on GitHub -- the description, the
/// topics, the homepage -- except `language`, which GitHub's linguist
/// computes from the source. It is kept because "what have you written in
/// Rust" is a question a visitor asks, and it is labelled for what it is.
#[derive(Debug, Clone, Deserialize)]
pub struct Record {
    /// `owner/name`, as GitHub spells it.
    pub full_name: String,
    /// The one-line description set on GitHub.
    #[serde(default)]
    pub description: Option<String>,
    /// Topics set on GitHub.
    #[serde(default)]
    pub topics: Vec<String>,
    /// Primary language, as linguist computed it.
    #[serde(default)]
    pub language: Option<String>,
    /// The homepage set on GitHub; for these repositories, a live demo.
    #[serde(default)]
    pub homepage: Option<String>,
    /// Last push, ISO 8601.
    #[serde(default)]
    pub pushed_at: Option<String>,
    /// Whether GitHub records this as a fork of another repository.
    pub fork: bool,
}

impl Record {
    /// The link a post would declare to this repository.
    fn link(&self) -> Link {
        Link {
            kind: ResourceKind::Repo,
            url: format!("https://github.com/{}", self.full_name),
            title: None,
        }
    }

    /// The concepts this repository declares: its topics, then its
    /// language, lowercased the way the other ingesters lowercase theirs.
    pub fn concepts(&self) -> Vec<ConceptId> {
        let mut out: Vec<ConceptId> = self
            .topics
            .iter()
            .chain(self.language.iter())
            .map(|label| ConceptId::new(label.trim().to_lowercase()))
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// The repository as a resource, identified exactly as a post linking
    /// to it would identify it, so the two join without inference.
    pub fn resource(&self, source_hash: String) -> Resource {
        let link = self.link();
        let name = self.full_name.rsplit('/').next().unwrap_or_default();
        let spoken = name.replace(['-', '_'], " ").to_lowercase();
        let mut resource = Resource::stub(
            link_id(&link),
            ResourceKind::Repo,
            name.to_string(),
            link.url,
        );
        resource.date = self
            .pushed_at
            .as_deref()
            .unwrap_or_default()
            .chars()
            .take(10)
            .collect();
        resource.summary = self.description.clone().unwrap_or_default();
        resource.concepts = self.concepts();
        resource.aliases = [spoken].into_iter().filter(|a| a != name).collect();
        resource.source_hash = source_hash;
        resource
    }

    /// The live demo the owner named as this repository's homepage, and
    /// the declared edge that reaches it.
    pub fn demo(&self) -> Option<(Resource, Relation)> {
        let url = self
            .homepage
            .as_deref()
            .map(str::trim)
            .filter(|u| !u.is_empty())?;
        let link = Link {
            kind: ResourceKind::Demo,
            url: url.to_string(),
            title: None,
        };
        Some(edge(&link_id(&self.link()), &link))
    }
}
