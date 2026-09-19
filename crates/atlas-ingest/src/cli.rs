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

The blog subcommand reads front matter only. It runs no model and reads no
body text, because every fact it needs was written by hand at publication
time: `abstract` is an authored summary, `keywords` is an authored alias
list, and `repo_url`, `video_url`, `demo_url` and `papers` are authored
cross-corpus relations. All of them are recorded as Declared.

EXIT STATUS
  0  the corpus was written and validated
  1  a post failed to parse, a cross-reference did not resolve, or the
     output could not be written

AI CODING AGENT INSTRUCTIONS:
  1. Run `atlas-ingest blog ../blog` from the repository root; the default
     output is build/corpus/blog.ron, which is generated and not committed.
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
    #[arg(long, default_value = "build/corpus/blog.ron", global = true)]
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
}
