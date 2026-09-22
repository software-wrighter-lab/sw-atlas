//! `atlas-ingest` -- build the sw-atlas corpus from the sibling sources.

mod cli;

use atlas_ingest::assemble;

use clap::Parser;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = cli::Cli::parse();
    match run(&args.source, &args.out) {
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
fn run(source: &cli::Source, out: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let corpus = read(source)?;
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
    Ok(cli::report(&corpus, out, &hash))
}

/// Read one source into a corpus.
fn read(source: &cli::Source) -> Result<atlas_corpus::Corpus, Box<dyn std::error::Error>> {
    Ok(match source {
        cli::Source::Blog { repo } => assemble::blog(&repo.join("_posts"))?,
        cli::Source::Campus { repo } => atlas_campus::ingest(repo)?,
        cli::Source::Repos(args) => {
            let blog = assemble::blog(&args.blog.join("_posts"))?;
            let named = [blog, atlas_campus::ingest(&args.campus)?];
            atlas_repos::ingest(&args.cache, &atlas_repos::declared(&named))?
        }
        cli::Source::Videos(args) => {
            let posts = assemble::blog(&args.blog.join("_posts"))?;
            let sources = atlas_video::Sources {
                shorts: &args.shorts,
                map: &args.map,
            };
            atlas_video::ingest(&posts, &sources)?
        }
        cli::Source::Concepts(args) => concepts(args)?,
    })
}

/// Unify the corpora the other subcommands wrote, and write the collision
/// report beside the corpus.
fn concepts(args: &cli::Concepts) -> Result<atlas_corpus::Corpus, Box<dyn std::error::Error>> {
    let mut read = Vec::new();
    for path in &args.corpora {
        read.push(ron::from_str(&std::fs::read_to_string(path)?)?);
    }
    let ov = atlas_graph::overrides::Overrides::load(&args.overrides)?;
    let (corpus, text, unknown) = atlas_graph::build(&read, &ov);
    if let Some(first) = unknown.first() {
        let count = unknown.len();
        return Err(format!("{count} override keys name no concept, first: {first}").into());
    }
    std::fs::write(&args.report, text)?;
    Ok(corpus)
}
