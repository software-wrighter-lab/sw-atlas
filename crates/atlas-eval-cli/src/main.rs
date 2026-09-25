//! `atlas-eval` -- score every arm on the frozen sets.

mod arms;
mod report;

use arms::Arms;
use atlas_eval::score;
use atlas_questions::QuestionSet;
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

include!(concat!(env!("OUT_DIR"), "/build_facts.rs"));

/// Score the deterministic arms over the confirmed question sets.
#[derive(Parser)]
#[command(
    name = "atlas-eval",
    version = LONG_VERSION,
    long_version = LONG_VERSION,
    about = "Score every arm on the frozen question sets, with baselines and intervals",
    long_about = "\
atlas-eval - the yardstick.

Scores each arm over the confirmed rows of every frozen set and prints a table
per set plus the paired comparisons between arms. Unconfirmed rows are counted
and skipped: a row the owner has not passed over is not a yardstick.

Three things it will not do, each because a measurement went wrong without the
rule: print an accuracy without the score a constant answer would get beside
it; print a margin without its 95% interval; or lead with accuracy@1 when
recall@k is what bounds everything that reranks.

EXIT STATUS
  0  the arms were scored
  1  the corpus or the sets could not be read

AI CODING AGENT INSTRUCTIONS:
  1. Run `just eval`. It needs build/corpus/corpus.ron, so run `just ingest`
     first.
  2. Do not add an arm that reads the frozen sets during training. The leakage
     check in atlas-questions exists because that is the easy mistake.
  3. A number here belongs in docs/reference/results.md only through the
     scoreboard step, which records the command and the corpus hash beside it."
)]
struct Args {
    /// The corpus to match against.
    #[arg(long, default_value = "build/corpus/corpus.ron")]
    corpus: PathBuf,
    /// The directory of question sets.
    #[arg(long, default_value = "sources/questions")]
    sets: PathBuf,
    /// How many candidates each arm may propose.
    #[arg(long, default_value_t = score::DEPTH)]
    depth: usize,
}

fn main() -> ExitCode {
    match run_all(&Args::parse()) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("atlas-eval: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Every set in the directory, by name.
fn sets(dir: &std::path::Path) -> Result<Vec<QuestionSet>, String> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "ron"))
        .collect();
    paths.sort();
    paths.iter().map(|path| QuestionSet::load(path)).collect()
}

/// Score every arm on every set, and compare the arms.
fn run_all(args: &Args) -> Result<String, String> {
    let text = std::fs::read_to_string(&args.corpus).map_err(|e| e.to_string())?;
    let corpus: atlas_corpus::Corpus = ron::from_str(&text).map_err(|e| e.to_string())?;
    let hash = atlas_corpus::content_hash(text.as_bytes());
    let arms = Arms::new(&corpus);
    let mut out = format!(
        "corpus {hash}\nsignals: MB02 {}, MB02t {}\n",
        arms.fields.signal_count(),
        arms.text.signal_count()
    );
    for set in sets(&args.sets)? {
        out.push_str(&one_set(&set, &corpus, &arms, args.depth));
    }
    Ok(out)
}

/// One set: every arm's line, the margins between them, and where the misses
/// live.
fn one_set(set: &QuestionSet, corpus: &atlas_corpus::Corpus, arms: &Arms, depth: usize) -> String {
    let rows = atlas_eval::confirmed(set);
    let (confirmed, drafted) = set.counted();
    let mut out = format!(
        "\n## {} ({confirmed} confirmed, {drafted} drafted)\n\n",
        set.name
    );
    if rows.is_empty() {
        out.push_str("nothing confirmed yet: not scored\n");
        return out;
    }
    let scored = arms.score(corpus, &rows, depth);
    out.push_str(&report::table(&scored));
    out.push('\n');
    out.push_str(&report::misses(&scored[1].1, 5));
    out
}
