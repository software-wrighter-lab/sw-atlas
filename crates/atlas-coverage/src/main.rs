//! `atlas-report` -- write the coverage document, and gate on it.

use atlas_coverage::{markdown, measure, urls};
use atlas_links::Cache;
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

include!(concat!(env!("OUT_DIR"), "/build_facts.rs"));

/// Regenerate the coverage report from a built corpus.
#[derive(Parser)]
#[command(
    name = "atlas-report",
    version = LONG_VERSION,
    long_version = LONG_VERSION,
    about = "Write docs/reference/coverage.md from the corpus, and gate on it",
    long_about = "\
atlas-report - turn the corpus into a coverage number with a gate behind it.

Reads a corpus written by atlas-ingest and the committed URL results, writes
the coverage document, and exits non-zero if either gate fails: a resource
nothing can reach, or a URL that is gone.

Offline by default. The link check is a network call, so it happens only
with --check, which rewrites cache/url-status.json for committing. Without
it, this reads the committed results and makes no request, so the gate does
not depend on someone else's site being up.

EXIT STATUS
  0  the report was written and both gates pass
  1  a gate failed, or the corpus or cache could not be read

AI CODING AGENT INSTRUCTIONS:
  1. Run `just report` (offline) or `just report --check` (network).
  2. A dead URL is fixed in the source that declares it -- a blog post's
     front matter, the campus catalog -- never by deleting the row here.
  3. An unreachable resource is fixed by giving it a concept or a relation
     in its ingester, not by loosening this gate."
)]
struct Args {
    /// The corpus to measure.
    #[arg(long, default_value = "build/corpus/corpus.ron")]
    corpus: PathBuf,
    /// Where the document goes.
    #[arg(long, default_value = "docs/reference/coverage.md")]
    out: PathBuf,
    /// The committed URL results.
    #[arg(long, default_value = "cache/url-status.json")]
    cache: PathBuf,
    /// The committed repository exclusions, for the count the report states.
    #[arg(long, default_value = "sources/repo-exclusions.ron")]
    exclusions: PathBuf,
    /// Check URLs over the network and rewrite the cache.
    #[arg(long)]
    check: bool,
    /// With `--check`, re-ask URLs that already have a result.
    #[arg(long)]
    again: bool,
    /// Today's date, for the record a check writes. The recipe passes it,
    /// because a date library is a dependency this crate does not need.
    #[arg(long, default_value = "unknown")]
    today: String,
}

fn main() -> ExitCode {
    match run(&Args::parse()) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("atlas-report: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Read the corpus, and the hash that names exactly which corpus it was.
fn corpus_of(path: &std::path::Path) -> Result<(atlas_corpus::Corpus, String), String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let corpus = ron::from_str(&text).map_err(|e| e.to_string())?;
    Ok((corpus, atlas_corpus::content_hash(text.as_bytes())))
}

/// Measure, write, and report what the gates say.
fn run(args: &Args) -> Result<String, String> {
    let (corpus, hash) = corpus_of(&args.corpus)?;
    let mut cache = Cache::load(&args.cache)?;
    if args.check {
        cache.refresh(&urls::of(&corpus), args.again, &args.today);
        cache.save(&args.cache)?;
    }
    let skip = atlas_repos::exclusions::Exclusions::load(&args.exclusions)?;
    let coverage = measure(&corpus, &cache, skip.excluded.len());
    let when = if args.check {
        &args.today
    } else {
        &checked_on(&cache)
    };
    let document = markdown::render(&coverage, &hash, when);
    std::fs::write(&args.out, document).map_err(|e| e.to_string())?;
    let out = args.out.display();
    match coverage.failures() {
        failures if failures.is_empty() => Ok(format!("wrote {out}\n  both gates pass")),
        failures => Err(format!("{}\n  wrote {out}", failures.join("\n  "))),
    }
}

/// The most recent day anything was checked, for the report's header.
fn checked_on(cache: &Cache) -> String {
    let latest = cache.urls.values().map(|r| r.checked.clone()).max();
    latest.unwrap_or_else(|| "never".to_string())
}
