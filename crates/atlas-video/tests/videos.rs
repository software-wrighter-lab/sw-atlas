//! The video join, pinned against a small shorts tree and a blog corpus
//! built in place.
//!
//! The fixture shorts tree has three series episodes (1, which lives in
//! its own oddly named project, 3, and 30, whose title drops the `#`), one
//! production joined by a confirmed row, and one whose description repeats
//! a published title word for word but which only a *proposed* row names.
//! That last one is the trap: it must stay unjoined.

use atlas_core::{Link, Resource, ResourceKind, edge};
use atlas_corpus::{Corpus, validate};
use atlas_video::{Error, Sources, ingest};
use std::path::PathBuf;

const VIDEOS: [(&str, &str); 6] = [
    (
        "https://www.youtube.com/shorts/Zu3zreN8c0Q",
        "Five ML Concepts - #1",
    ),
    (
        "https://www.youtube.com/shorts/U-_yZZdZurU",
        "Five ML Concepts - #3",
    ),
    (
        "https://www.youtube.com/shorts/JDtrfiBecNo",
        "Five ML Concepts - 30",
    ),
    (
        "https://www.youtube.com/shorts/O6U06cGkKc4",
        "976 parameters is more than billions?!",
    ),
    (
        "https://www.youtube.com/shorts/QelUyffoo1g",
        "XSkill: A Memory Layer for Multimodal Agents",
    ),
    (
        "https://youtu.be/872RLMBzC_8",
        "90s Pipelines Rust/WASM homage #TBT",
    ),
];

/// Two posts; the later one links only to the maze video, the earlier to
/// all six.
fn blog() -> Corpus {
    let mut corpus = Corpus::new();
    for (stem, videos) in [
        ("2026-01-31-a", &VIDEOS[..]),
        ("2026-02-10-b", &VIDEOS[3..4]),
    ] {
        let mut post = Resource::stub(
            atlas_core::ResourceId::new(format!("blog:{stem}")),
            ResourceKind::Post,
            stem.into(),
            String::new(),
        );
        post.date = stem[..10].to_string();
        for (url, title) in videos {
            let link = Link {
                kind: ResourceKind::Video,
                url: (*url).into(),
                title: Some((*title).into()),
            };
            let (video, relation) = edge(&post.id, &link);
            corpus.resources.push(video);
            corpus.relations.push(relation);
        }
        corpus.resources.push(post);
    }
    corpus.resources.sort_by(|a, b| a.id.cmp(&b.id));
    corpus.resources.dedup_by(|a, b| a.id == b.id);
    corpus
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn run(map: &str) -> Result<Corpus, Error> {
    let (shorts, map) = (fixture("shorts"), fixture(map));
    ingest(
        &blog(),
        &Sources {
            shorts: &shorts,
            map: &map,
        },
    )
}

fn video<'a>(corpus: &'a Corpus, id: &str) -> &'a Resource {
    corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == format!("video:{id}"))
        .expect(id)
}

#[test]
fn every_declared_video_is_a_resource_and_nothing_else_is() {
    let corpus = run("map.ron").expect("the join succeeds");
    assert_eq!(corpus.resources.len(), 6);
    assert!(
        corpus
            .resources
            .iter()
            .all(|r| r.kind == ResourceKind::Video)
    );
    assert!(validate(&corpus).is_empty(), "{:?}", validate(&corpus));
}

#[test]
fn series_episodes_join_by_rule_including_the_two_odd_ones() {
    let corpus = run("map.ron").expect("the join succeeds");
    assert_eq!(
        video(&corpus, "Zu3zreN8c0Q").body,
        "Concept one, backpropagation."
    );
    assert_eq!(
        video(&corpus, "U-_yZZdZurU").summary,
        "EPISODE 3 - Five machine learning concepts."
    );
    assert_eq!(
        video(&corpus, "JDtrfiBecNo").body,
        "EPISODE 30 - Five more.",
        "no narration: the description"
    );
}

#[test]
fn episode_concepts_are_declared_as_written() {
    let corpus = run("map.ron").expect("the join succeeds");
    let three: Vec<&str> = video(&corpus, "U-_yZZdZurU")
        .concepts
        .iter()
        .map(|c| c.as_str())
        .collect();
    assert_eq!(three, ["loss function", "few-shot / zero-shot", "lora"]);
    assert!(
        video(&corpus, "JDtrfiBecNo").concepts.is_empty(),
        "the status file stops at 26"
    );
    assert_eq!(corpus.concepts.len(), 5);
}

#[test]
fn a_confirmed_row_joins_whatever_url_form_it_was_written_in() {
    let corpus = run("map.ron").expect("the join succeeds");
    let maze = video(&corpus, "O6U06cGkKc4");
    assert_eq!(maze.summary, "Under 1000 Parameters Beats GPT-4 at Mazes");
    assert_eq!(maze.date, "2026-01-31", "the earliest post declaring it");
}

#[test]
fn a_matching_title_joins_nothing_without_a_confirmed_row() {
    let corpus = run("map.ron").expect("the join succeeds");
    let proposed = video(&corpus, "QelUyffoo1g");
    assert!(proposed.body.is_empty() && proposed.summary.is_empty());
    assert!(
        video(&corpus, "872RLMBzC_8").body.is_empty(),
        "never a short"
    );
}

#[test]
fn a_stale_row_or_a_missing_project_stops_the_build() {
    assert!(matches!(run("stale.ron"), Err(Error::Stale(urls)) if urls.len() == 1));
    assert!(matches!(run("missing.ron"), Err(Error::Io(_))));
}
