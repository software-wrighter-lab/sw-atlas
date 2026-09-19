//! `atlas-ingest` -- build the sw-atlas corpus from the sibling sources.

mod cli;

use atlas_ingest::assemble;

use clap::Parser;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = cli::Cli::parse();
    let cli::Source::Blog { repo } = &args.source;
    match run(&repo.join("_posts"), &args.out) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("atlas-ingest: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Ingest, validate, write, and describe what happened.
fn run(posts: &Path, out: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let corpus = assemble::blog(posts)?;
    let problems = atlas_corpus::validate(&corpus);
    if let Some(first) = problems.first() {
        return Err(format!(
            "{} cross-reference problems, first: {first:?}",
            problems.len()
        )
        .into());
    }
    let (text, hash) = atlas_corpus::canonical(&corpus)?;
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(out, &text)?;
    Ok(report(&corpus, out, &hash))
}

/// One line per thing a reader would want to check.
fn report(corpus: &atlas_corpus::Corpus, out: &Path, hash: &str) -> String {
    use atlas_core::ResourceKind::{Demo, Paper, Post, Repo, Video};
    let count = |kind| corpus.resources.iter().filter(|r| r.kind == kind).count();
    let rows = [
        ("posts", count(Post)),
        ("repos", count(Repo)),
        ("videos", count(Video)),
        ("demos", count(Demo)),
        ("papers", count(Paper)),
        ("concepts", corpus.concepts.len()),
        ("relations", corpus.relations.len()),
    ];
    let counts: Vec<String> = rows
        .iter()
        .map(|(name, n)| format!("  {name:<10} {n}"))
        .collect();
    format!(
        "wrote {}\n  hash       {hash}\n{}",
        out.display(),
        counts.join("\n")
    )
}
