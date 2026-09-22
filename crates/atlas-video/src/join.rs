//! The committed map from a published video to the project it was made in.
//!
//! Titles changed between production and publication -- `projects/trm`'s
//! description opens "Under 1000 Parameters Beats GPT-4 at Mazes" and the
//! video is "976 parameters is more than billions?!" -- so this crate
//! never matches a title to a project. A person writes the pair down, and
//! the pair is `Provenance::Declared`.

use crate::episodes;
use atlas_core::{Link, Resource, ResourceId, ResourceKind, link_id};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// `sources/video-shorts.ron`.
#[derive(Debug, Default, Deserialize)]
pub struct JoinMap {
    /// Video URL, written as a post writes it, to a directory under
    /// `shorts/projects/`. The only rows the ingest reads.
    pub confirmed: BTreeMap<String, String>,
    /// Candidates awaiting the owner. Never read by the ingest; a row is
    /// promoted by moving it into `confirmed`.
    #[serde(default)]
    pub proposed: Vec<Proposal>,
}

/// A pair someone suspects and nobody has confirmed.
#[derive(Debug, Clone, Deserialize)]
pub struct Proposal {
    /// The video, as a post declares it.
    pub url: String,
    /// The project it might have been made in.
    pub project: String,
    /// Why anyone thinks so.
    pub evidence: String,
}

impl JoinMap {
    /// Read the map.
    ///
    /// # Errors
    ///
    /// The file cannot be read, or is not a `JoinMap` in RON.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// The project a video was made in: the series rule first, then a
    /// confirmed row. `None` is an honest answer; most long-form videos
    /// were never shorts.
    pub fn project(&self, video: &Resource) -> Option<String> {
        let by_rule = episodes::episode(&video.title).map(episodes::project);
        by_rule.or_else(|| {
            let (_, project) = self.confirmed.iter().find(|(url, _)| id(url) == video.id)?;
            Some(project.clone())
        })
    }

    /// Confirmed rows whose video no post declares: a stale or mistyped
    /// row, which would otherwise be ignored without a word.
    pub fn unknown(&self, videos: &[&Resource]) -> Vec<String> {
        let known = |url: &String| videos.iter().any(|v| v.id == id(url));
        self.confirmed
            .keys()
            .filter(|url| !known(url))
            .cloned()
            .collect()
    }
}

/// The identifier a post declaring this URL gives the video.
fn id(url: &str) -> ResourceId {
    link_id(&Link {
        kind: ResourceKind::Video,
        url: url.to_string(),
        title: None,
    })
}
