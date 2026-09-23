//! `atlas-questions` -- count, freeze and check the evaluation sets.

use atlas_questions::{QuestionSet, rules};
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

include!(concat!(env!("OUT_DIR"), "/build_facts.rs"));

/// Report and check the frozen evaluation sets.
#[derive(Parser)]
#[command(
    name = "atlas-questions",
    version = LONG_VERSION,
    long_version = LONG_VERSION,
    about = "Count, freeze and check the evaluation sets",
    long_about = "\
atlas-questions - the sets that decide whether anything later may claim to be
an improvement.

Prints one line per set: how many rows the owner has confirmed, how many are
still drafted, and the hash that freezes it. Then checks what a drafter
cannot check by eye: that every expected identifier resolves, and that no
paraphrase contains an alias or the title of its own answer, which would
make it a lookup rather than a paraphrase.

EXIT STATUS
  0  every set is internally consistent
  1  a set names a resource that does not exist, or a paraphrase gives away
     its own answer

AI CODING AGENT INSTRUCTIONS:
  1. Run `just questions` after editing anything under sources/questions/.
  2. A row this rejects is fixed by rewording the question, never by
     loosening the check: the no-alias rule is what the paraphrase split
     means.
  3. Do not change a row's status to Confirmed. Only the repository owner
     confirms rows."
)]
struct Args {
    /// The corpus the expectations are checked against.
    #[arg(long, default_value = "build/corpus/corpus.ron")]
    corpus: PathBuf,
    /// The directory of sets.
    #[arg(long, default_value = "sources/questions")]
    sets: PathBuf,
}

fn main() -> ExitCode {
    match run(&Args::parse()) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("atlas-questions: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Every set in the directory, in name order.
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

/// Count, freeze, check.
fn run(args: &Args) -> Result<String, String> {
    let text = std::fs::read_to_string(&args.corpus).map_err(|e| e.to_string())?;
    let corpus: atlas_corpus::Corpus = ron::from_str(&text).map_err(|e| e.to_string())?;
    let mut lines = Vec::new();
    let mut problems = Vec::new();
    for set in sets(&args.sets)? {
        let (confirmed, drafted) = set.counted();
        let digest = &set.digest()[..16];
        lines.push(format!(
            "  {:<12} {confirmed:>4} confirmed  {drafted:>4} drafted   {digest}",
            set.name
        ));
        problems.extend(rules::problems(&set, &corpus));
    }
    if problems.is_empty() {
        return Ok(format!("evaluation sets\n{}", lines.join("\n")));
    }
    let count = problems.len();
    Err(format!("{count} problems\n  {}", problems.join("\n  ")))
}
