//! The command line, and the long help `sw-checklist` asks for.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

include!(concat!(env!("OUT_DIR"), "/build_facts.rs"));

/// The long help: what the tool is for, and how an agent should drive it.
pub const LONG_ABOUT: &str = "\
atlas-ingest - build the sw-atlas corpus from the sibling sources.

The sources are read-only. Each subcommand reads one of them and writes a
canonical RON corpus: resources, concepts and relations, sorted so that a
rebuild which changes nothing produces an identical file and an identical
SHA-256. The hash is printed with the counts.

The campus subcommand prefers dist/catalog.json and falls back to the
docent snapshot, recording which it used. The blog subcommand reads front
matter only. It runs no model and reads no
body text, because every fact it needs was written by hand at publication
time: `abstract` is an authored summary, `keywords` is an authored alias
list, and `repo_url`, `video_url`, `demo_url` and `papers` are authored
cross-corpus relations. All of them are recorded as Declared.

The repos subcommand reads cache/github-repos.json, never the network, and
keeps public repositories that are not forks: metadata only, no source.
The videos subcommand takes every video the blog declares and fills in its
script from the shorts repository: Five ML Concepts episodes by rule, every
other video only through a confirmed row in sources/video-shorts.ron.
Nothing matches a title.

EXIT STATUS
  0  the corpus was written and validated
  1  a post failed to parse, a cross-reference did not resolve, or the
     output could not be written, a join-map row names a video no post
     declares, or a mapped shorts project is missing

AI CODING AGENT INSTRUCTIONS:
  1. Run the `just ingest-*` recipes from the repository root, or e.g.
     `atlas-ingest blog ../blog`; the default output is under build/corpus/,
     which is generated and not committed.
  2. A parse failure names the post and the field. Do not skip the post and
     do not loosen the schema to make it pass: the blog writes a shape this
     tool does not yet accept, and accepting it is the fix.
  3. Never write to the source repository. Anything it needs to change is a
     work order in docs/<repo>-requests.md.
  4. If the printed hash changes when no source changed, the canonical
     writer is at fault; that is a bug, not something to re-run.";

/// Build the sw-atlas corpus from the sibling sources.
#[derive(Parser)]
#[command(
    name = "atlas-ingest",
    version = LONG_VERSION,
    long_version = LONG_VERSION,
    about = "Build the sw-atlas corpus from the sibling sources",
    long_about = LONG_ABOUT
)]
pub struct Cli {
    /// Which source to read.
    #[command(subcommand)]
    pub source: Source,
    /// Where to write the canonical corpus.
    #[arg(long, default_value = "build/corpus/corpus.ron", global = true)]
    pub out: PathBuf,
}

/// The sources this build knows how to read.
#[derive(Subcommand)]
pub enum Source {
    /// The blog's `_posts` directory, from front matter alone.
    Blog {
        /// Path to a checkout of the blog repository.
        repo: PathBuf,
    },
    /// The campus catalog: `dist/catalog.json` if it is published, and the
    /// docent snapshot until it is.
    Campus {
        /// Path to a checkout of the sw-campus repository.
        repo: PathBuf,
    },
    /// Public, non-fork repositories, from the committed GitHub cache.
    Repos {
        /// The cache `scripts/fetch-repos` writes.
        #[arg(default_value = "cache/github-repos.json")]
        cache: PathBuf,
    },
    /// Every video the blog declares, joined to its script in `shorts`.
    Videos {
        /// Path to a checkout of the blog repository.
        blog: PathBuf,
        /// Path to a checkout of the shorts repository.
        shorts: PathBuf,
        /// The hand-written video-to-project map.
        #[arg(long, default_value = "sources/video-shorts.ron")]
        map: PathBuf,
    },
}
