//! The YAML block at the top of a post, and nothing else.
//!
//! No body text is read. Every field below was written by hand at
//! publication time, which is why this ingester needs no model: the
//! abstract is an authored summary, `keywords` is an authored alias list,
//! and the URL fields are authored cross-corpus relations.

use serde::Deserialize;

/// One post's front matter, as the blog actually writes it.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct FrontMatter {
    /// Post title.
    pub title: String,
    /// Authored summary.
    #[serde(default)]
    pub abstract_field: Option<String>,
    /// Broad subject areas.
    #[serde(default)]
    pub categories: Vec<String>,
    /// Finer subject tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Comma-separated alias list, written by hand.
    #[serde(default)]
    pub keywords: Option<String>,
    /// Series this post belongs to.
    #[serde(default)]
    pub series: Option<String>,
    /// Position within the series.
    #[serde(default)]
    pub series_part: Option<u32>,
    /// Single repository link.
    #[serde(default)]
    pub repo_url: Option<String>,
    /// Several repository links.
    #[serde(default)]
    pub repo_urls: Vec<Titled>,
    /// Single video link.
    #[serde(default)]
    pub video_url: Option<String>,
    /// Title of the single video.
    #[serde(default)]
    pub video_title: Option<String>,
    /// Several video links.
    #[serde(default)]
    pub video_urls: Vec<Titled>,
    /// Titles for `video_urls`, positionally. The blog writes these as a
    /// parallel list rather than as mappings, so index `i` of one belongs
    /// to index `i` of the other.
    #[serde(default)]
    pub video_titles: Vec<String>,
    /// A runnable demonstration.
    #[serde(default)]
    pub demo_url: Option<String>,
    /// Outside works this post cites.
    #[serde(default)]
    pub papers: Vec<Titled>,
}

/// A link with a name, as the list-valued front matter fields carry it.
///
/// The blog writes these two ways -- a bare URL string, or a mapping with
/// `url` and `title` -- and both are correct, so both are accepted. An
/// ingester that refused one would be enforcing a house style the author
/// never agreed to.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Titled {
    /// Just the URL.
    Bare(String),
    /// A URL with the author's name for it.
    Named {
        /// Where it points.
        url: String,
        /// What to call it.
        #[serde(default)]
        title: Option<String>,
    },
}

impl Titled {
    /// Where it points.
    pub fn url(&self) -> &str {
        match self {
            Self::Bare(url) => url,
            Self::Named { url, .. } => url,
        }
    }

    /// What the author called it, where they said.
    pub fn title(&self) -> Option<String> {
        match self {
            Self::Bare(_) => None,
            Self::Named { title, .. } => title.clone(),
        }
    }
}

/// Take the YAML block from between the first two `---` fences.
pub fn split(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("---")?.trim_start_matches(['\r', '\n']);
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// Parse a post's front matter.
///
/// # Errors
///
/// Returns the YAML error if the block is not a mapping of the expected
/// shape. A new field the blog starts writing is ignored; a field that
/// changes type fails loudly here, which is the intended behaviour.
pub fn parse(text: &str) -> Result<FrontMatter, serde_yaml_ng::Error> {
    let block = split(text).unwrap_or(text);
    let patched = block.replace("\nabstract:", "\nabstract_field:");
    serde_yaml_ng::from_str(&patched)
}
