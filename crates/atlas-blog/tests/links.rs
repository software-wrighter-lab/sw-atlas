//! Empty link fields declare nothing.
//!
//! `2026-03-13-rabbit-hole-rust-to-unsupported-isa` writes `video_url: ""`,
//! which once produced a video resource with the identifier `video:` and
//! no address. The front matter below is that shape, trimmed.

use atlas_blog::{frontmatter, links};
use atlas_core::ResourceKind;

const FRONT: &str = "---
title: A post with no video yet
video_url: \"\"
repo_url: https://github.com/sw-embed/sw-cor24-apl
video_urls:
  - \"\"
  - https://youtu.be/872RLMBzC_8
video_titles:
  - nothing
  - 90s Pipelines
---
";

#[test]
fn an_empty_url_is_not_a_link() {
    let front = frontmatter::parse(FRONT).expect("the front matter parses");
    let found = links::links(&front);
    assert!(found.iter().all(|l| !l.url.is_empty()), "{:?}", found.len());
    let videos: Vec<_> = found
        .iter()
        .filter(|l| l.kind == ResourceKind::Video)
        .collect();
    assert_eq!(videos.len(), 1);
}

#[test]
fn skipping_an_empty_entry_keeps_the_titles_aligned() {
    let front = frontmatter::parse(FRONT).expect("the front matter parses");
    let video = links::links(&front)
        .into_iter()
        .find(|l| l.kind == ResourceKind::Video)
        .expect("the one real video");
    assert_eq!(video.title.as_deref(), Some("90s Pipelines"));
}
