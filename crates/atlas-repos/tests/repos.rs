//! The repository ingest, pinned against a four-entry cache and against
//! the committed one.
//!
//! The fixture holds one of each shape the real cache has: a repository
//! with topics and a language, a fork, one whose homepage is a live demo,
//! and one GitHub knows almost nothing about.

use atlas_core::{Provenance, RelationKind, ResourceId, ResourceKind};
use atlas_corpus::{Corpus, validate};
use atlas_repos::accounts::Accounts;
use atlas_repos::exclusions::Exclusions;
use atlas_supersede::Canonical;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// What the committed cache yields as shipped. Refreshing the cache updates
/// this number in the same commit, with the new count in the message.
const PINNED: usize = 252;

fn ingest(relative: &str, declared: &[&str]) -> Corpus {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    let declared: BTreeSet<ResourceId> = declared.iter().map(|id| ResourceId::new(*id)).collect();
    let none = Exclusions::default();
    atlas_repos::ingest(&path, &declared, (&none, &fixture_pairs(), &accounts()))
        .expect("the cache ingests")
}

/// The fixture's one declared pair.
fn fixture_pairs() -> Canonical {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/canonical.ron");
    Canonical::load(&path).expect("the fixture pairs parse")
}

/// The cache read exactly as `just ingest-repos` reads it: committed
/// exclusions, committed canonical declarations, committed accounts.
fn as_shipped(declared: &[&str]) -> Corpus {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cache/github-repos.json");
    let declared: BTreeSet<ResourceId> = declared.iter().map(|id| ResourceId::new(*id)).collect();
    atlas_repos::ingest(&path, &declared, (&committed(), &real(), &accounts())).expect("ingests")
}

/// The committed accounts, as `just ingest-repos` applies them.
fn accounts() -> Accounts {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sources/accounts.ron");
    Accounts::load(&path).expect("the committed accounts parse")
}

/// The committed declarations, as `just ingest-repos` applies them.
fn real() -> Canonical {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sources/repo-canonical.ron");
    Canonical::load(&path).expect("the committed declarations parse")
}

/// Every `owner/name` in the committed cache.
fn cached_names() -> Vec<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cache/github-repos.json");
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let rows: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap_or_default();
    rows.iter()
        .filter_map(|r| r["full_name"].as_str().map(str::to_string))
        .collect()
}

/// The committed list, as `just ingest-repos` applies it.
fn committed() -> Exclusions {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sources/repo-exclusions.ron");
    Exclusions::load(&path).expect("the committed exclusions parse")
}

fn fixture() -> Corpus {
    ingest("tests/fixtures/github-repos.json", &[])
}

#[test]
fn undeclared_forks_are_excluded_and_nothing_else_is() {
    let corpus = fixture();
    let repos: Vec<&str> = corpus
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Repo)
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(repos.len(), 4);
    assert!(!repos.contains(&"repo:sw-embed/bmp280"), "the fork");
    assert!(validate(&corpus).is_empty(), "{:?}", validate(&corpus));
}

#[test]
fn a_repository_has_the_identifier_a_post_linking_to_it_has() {
    let corpus = fixture();
    let apl = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "repo:sw-embed/sw-cor24-apl")
        .expect("the id the blog and the campus already use");
    assert_eq!(apl.url, "https://github.com/sw-embed/sw-cor24-apl");
    assert_eq!(apl.title, "sw-cor24-apl");
    assert_eq!(apl.summary, "APL for the COR24");
    assert_eq!(apl.date, "2026-09-10");
    let concepts: Vec<&str> = apl.concepts.iter().map(|c| c.as_str()).collect();
    assert_eq!(concepts, ["apl", "array-languages", "rust", "sw-embed"]);
    assert_eq!(apl.aliases, ["sw cor24 apl"]);
    assert!(!apl.source_hash.is_empty());
}

#[test]
fn a_homepage_is_a_declared_demo() {
    let corpus = fixture();
    let edge = corpus
        .relations
        .iter()
        .find(|r| r.from.as_str() == "repo:software-wrighter-lab/sw-campus")
        .expect("the homepage edge");
    assert_eq!(edge.kind, RelationKind::Demos);
    assert_eq!(edge.provenance, Provenance::Declared);
    assert_eq!(
        edge.to.as_str(),
        "demo:software-wrighter-lab.github.io/sw-campus"
    );
    assert_eq!(corpus.relations.len(), 1, "an empty homepage names nothing");
}

#[test]
fn a_repository_github_knows_little_about_is_still_reachable() {
    let corpus = fixture();
    let bare = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "repo:sw-ml-study/emufpga")
        .expect("present");
    assert!(bare.summary.is_empty() && bare.date.is_empty());
    let concepts: Vec<&str> = bare.concepts.iter().map(|c| c.as_str()).collect();
    assert_eq!(concepts, ["sw-ml-study"], "its organisation reaches it");
}

#[test]
fn an_excluded_repository_is_not_in_the_corpus() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cache/github-repos.json");
    let corpus = atlas_repos::ingest(
        &path,
        &BTreeSet::new(),
        (&committed(), &real(), &accounts()),
    )
    .expect("ingests");
    let ids: Vec<&str> = corpus.resources.iter().map(|r| r.id.as_str()).collect();
    assert!(
        !ids.contains(&"repo:softwarewrighter/images"),
        "an asset host"
    );
    assert!(
        !ids.contains(&"repo:softwarewrighter/placeholder"),
        "nothing there yet"
    );
    assert_eq!(
        committed().excluded.len(),
        2,
        "every exclusion carries its reason"
    );
}

#[test]
fn the_committed_cache_holds_the_pinned_number_of_repositories() {
    // Pinned on purpose: refreshing the cache is a corpus change, and its
    // commit updates this number with the new count. Read as shipped, so the
    // number is the one a visitor's corpus actually holds.
    let corpus = as_shipped(&[]);
    let repos = corpus
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Repo);
    assert_eq!(repos.count(), PINNED);
    assert!(validate(&corpus).is_empty());
}

#[test]
fn a_fork_the_owner_wrote_about_is_kept() {
    let corpus = ingest(
        "tests/fixtures/github-repos.json",
        &["repo:sw-embed/bmp280"],
    );
    let fork = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "repo:sw-embed/bmp280")
        .expect("declared by a post, so kept");
    assert_eq!(fork.summary, "Someone else's work, forked");
}

#[test]
fn the_forks_the_blog_names_are_all_in_the_committed_cache() {
    let named = [
        "repo:softwarewrighter/bdh",
        "repo:softwarewrighter/MesaOS",
        "repo:softwarewrighter/viz-hrm-ft",
        "repo:sw-embed/bmp280",
        "repo:sw-embed/sw-cor24-pascal",
        "repo:sw-fun/tt-rs",
        "repo:sw-game-dev/game-mcp-poc",
        "repo:sw-ml-study/Repeated-Sampling",
        "repo:sw-music-tools/rank-wav-rs",
    ];
    let corpus = as_shipped(&named);
    let repos = corpus
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Repo)
        .count();
    assert!(
        repos >= PINNED,
        "naming a fork can only add: {repos} against {PINNED}"
    );
}

#[test]
fn the_live_copy_is_kept_even_though_github_calls_it_a_fork() {
    let corpus = fixture();
    let ids: Vec<&str> = corpus.resources.iter().map(|r| r.id.as_str()).collect();
    assert!(
        ids.contains(&"repo:sw-vibe-coding/sw-install"),
        "declared canonical, and nothing links to it: the old rule dropped it"
    );
    assert!(
        !ids.contains(&"repo:softwarewrighter/sw-install"),
        "the superseded copy is not a second resource for the same work"
    );
}

#[test]
fn the_superseded_name_still_reaches_the_live_work() {
    let corpus = fixture();
    let kept = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "repo:sw-vibe-coding/sw-install")
        .expect("kept");
    assert!(
        kept.aliases.iter().any(|a| a == "sw-install"),
        "a visitor asking for the old name lands here: {:?}",
        kept.aliases
    );
}

#[test]
fn a_declaration_naming_a_repository_that_does_not_exist_is_a_failing_build() {
    let names = cached_names();
    if names.is_empty() {
        println!("skipped: no committed cache");
        return;
    }
    let pairs = real();
    for pair in &pairs.pairs {
        assert!(
            names.contains(&pair.canonical),
            "canonical {} is not in the cache -- typo, or it needs a cache refresh",
            pair.canonical
        );
        assert!(
            names.contains(&pair.superseded),
            "superseded {} is not in the cache",
            pair.superseded
        );
        assert!(!pair.why.is_empty(), "{} has no reason", pair.canonical);
    }
}

#[test]
fn a_pair_never_declares_a_repository_both_real_and_superseded() {
    let pairs = real();
    for pair in &pairs.pairs {
        assert_ne!(pair.canonical, pair.superseded);
        assert!(
            !pairs.drops(&pair.canonical),
            "{} is declared canonical and superseded at once",
            pair.canonical
        );
    }
    assert_eq!(pairs.pairs.len(), 10, "the ten pairs found on 2026-09-26");
}

#[test]
fn no_declared_copy_is_waiting_to_be_caught_up() {
    let pairs = real();
    let pending: Vec<&str> = pairs
        .pairs
        .iter()
        .filter(|p| p.pending_sync)
        .map(|p| p.canonical.as_str())
        .collect();
    assert!(
        pending.is_empty(),
        "sw-install and sw-init were brought level on 2026-09-30; a pair that \
         carries this flag again is a copy a visitor would be sent to while \
         the work happens elsewhere: {pending:?}"
    );
}

#[test]
fn a_copy_waiting_to_be_caught_up_can_still_be_declared() {
    // The flag is not dead code just because no pair carries it today.
    let text = r#"Canonical(pairs: [
        (canonical: "org/thing", superseded: "owner/thing", why: "moving", pending_sync: true),
    ])"#;
    let pairs: Canonical = ron::from_str(text).expect("parses");
    assert!(pairs.pairs[0].pending_sync);
    assert!(pairs.keeps("org/thing") && pairs.drops("owner/thing"));
}

#[test]
fn every_superseded_repository_is_out_of_the_committed_corpus() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cache/github-repos.json");
    if !path.exists() {
        return;
    }
    let corpus = atlas_repos::ingest(
        &path,
        &BTreeSet::new(),
        (&committed(), &real(), &accounts()),
    )
    .expect("ingests");
    let ids: Vec<&str> = corpus.resources.iter().map(|r| r.id.as_str()).collect();
    let pairs = real();
    for pair in &pairs.pairs {
        let gone = format!("repo:{}", pair.superseded);
        assert!(!ids.contains(&gone.as_str()), "{gone} is still ingested");
        let kept = format!("repo:{}", pair.canonical);
        assert!(ids.contains(&kept.as_str()), "{kept} is missing");
    }
}
