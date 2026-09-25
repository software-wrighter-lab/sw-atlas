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
    /// Write a review form to this path instead of checking: every row on one
    /// line with a mark column the repository owner edits in place.
    #[arg(long)]
    form: Option<PathBuf>,
}

/// What the review form says about itself, above the rows.
const FORM_HEADER: &str = "\
# Evaluation questions: review form
#
# Edit in place, then say so; the marks are applied to sources/questions/*.ron.
# Replace the leading `_` on a row with one character:
#
#     Y   confirmed: a question a visitor might type, expected answer right
#     N   rejected: the row is deleted
#     ?   unsure: the row stays drafted and is discussed
#
# A row left `_` is unreviewed and is never used as a yardstick. `Y ALL` under
# a heading confirms that whole section; an individual N or ? beats it.
#
# Confirming is the owner's judgement, not an agent's: rows drafted and
# confirmed by the same party measure that party's idea of the corpus rather
# than the corpus. The mechanical checks are run separately by `just
# questions` -- every expected id resolves, and no paraphrase contains an
# alias or title of its own answer.
#
# Columns are separated by ` :: `, not `|`, because a follow-up row contains a
# `|` inside the question itself.
#
# SHORT ON TIME? sources/questions/NEEDS-REVIEW.txt lists the rows that
# actually need a judgement -- the ones where the corpus offers a sibling the
# key might have preferred, or where the drafter left a doubt. The rest had one
# plausible answer and the drafter owns them.
";

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

/// Write the review form: one line per row, marks left for a person.
fn form(sets: &[QuestionSet], path: &std::path::Path) -> Result<String, String> {
    let mut out = String::from(FORM_HEADER);
    let mut rows = 0;
    for set in sets {
        let rule = "=".repeat(70);
        out.push_str(&format!("\n{rule}\n# {} :: {}\n", set.name, set.note));
        for (n, row) in set.rows.iter().enumerate() {
            out.push_str(&row.review_line(n + 1));
            rows += 1;
        }
    }
    std::fs::write(path, out).map_err(|e| e.to_string())?;
    Ok(format!("wrote {} with {rows} rows", path.display()))
}

/// Count, freeze, check.
fn run(args: &Args) -> Result<String, String> {
    let all = sets(&args.sets)?;
    if let Some(path) = &args.form {
        return form(&all, path);
    }
    let text = std::fs::read_to_string(&args.corpus).map_err(|e| e.to_string())?;
    let corpus: atlas_corpus::Corpus = ron::from_str(&text).map_err(|e| e.to_string())?;
    let mut lines = Vec::new();
    let mut problems = Vec::new();
    for set in all {
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
