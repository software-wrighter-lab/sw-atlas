//! `atlas-ask` -- ask the docent a question and see what it says.

use atlas_answer::Absent;
use atlas_answer::{Policy, decide, frames, suggest};
use atlas_match::{Matcher, Variant};
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

mod measure;
mod voice;

include!(concat!(env!("OUT_DIR"), "/build_facts.rs"));

/// Ask the deterministic docent, or measure what it does over the frozen sets.
#[derive(Parser)]
#[command(
    name = "atlas-ask",
    version = LONG_VERSION,
    long_version = LONG_VERSION,
    about = "Ask the docent a question; --measure reports what it does over the frozen sets",
    long_about = "\
atlas-ask - the docent, with no model in it.

Prints what a visitor would see, then the evidence the policy acted on. There
are five outcomes and no confident-single-answer at ordinary thresholds,
because the measurement says one is not available: over 397 confirmed
questions no score-and-margin combination answers with a single resource
better than 0.54 precision. Offering the closest three is right 0.435 of the
time and says it is a list.

EXIT STATUS
  0  the question was answered, or the measurement ran
  1  the corpus, the policy or the absent list could not be read

AI CODING AGENT INSTRUCTIONS:
  1. `just ask \"where is the 1130\"` for one question; `just ask --measure`
     for the distribution over every confirmed set.
  2. Thresholds live in sources/answer-policy.ron with the measurement that
     produced them. Do not tune them by eye -- refit and record.
  3. Every sentence a visitor sees is a frame in atlas-answer::frames. Adding
     a sentence means adding a frame, never composing one at runtime."
)]
struct Args {
    /// The question. Several words need no quotes.
    question: Vec<String>,
    /// Report how often each outcome fires over the confirmed sets instead.
    #[arg(long)]
    measure: bool,
    /// The corpus.
    #[arg(long, default_value = "build/corpus/corpus.ron")]
    corpus: PathBuf,
    /// The fitted thresholds.
    #[arg(long, default_value = "sources/answer-policy.ron")]
    policy: PathBuf,
    /// Subjects the corpus knowingly does not hold.
    #[arg(long, default_value = "sources/absent.ron")]
    absent: PathBuf,
}

fn main() -> ExitCode {
    match run(&Args::parse()) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("atlas-ask: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Everything the command needs, read once.
pub struct Docent {
    /// Every resource, concept and relation.
    pub corpus: atlas_corpus::Corpus,
    /// MB02 over that corpus, body text included.
    pub matcher: Matcher,
    /// The fitted thresholds.
    pub policy: Policy,
    /// Subjects the corpus knowingly does not hold.
    pub absent: Absent,
}

impl Docent {
    /// Answer one question the way a visitor sees it, with the evidence under
    /// it.
    fn ask(&self, question: &str) -> String {
        let answer = self.matcher.answer(question, 10);
        let reply = decide(
            question,
            &answer,
            &self.corpus,
            (&self.policy, &self.absent),
        );
        let suggestions = suggest::next_questions(&self.corpus, &voice::seed(&reply, &answer), 3);
        let shown = frames::render(&reply, &self.corpus, &suggestions);
        format!(
            "> {question}\n\n{shown}\n  -- {}: {}, intent {:?}\n",
            voice::label(&reply.outcome),
            reply.because,
            answer.intent
        )
    }
}

/// Read the corpus and the policy, then answer or measure.
fn run(args: &Args) -> Result<String, String> {
    let text = std::fs::read_to_string(&args.corpus).map_err(|e| e.to_string())?;
    let corpus: atlas_corpus::Corpus = ron::from_str(&text).map_err(|e| e.to_string())?;
    let docent = Docent {
        matcher: Matcher::new(&corpus, Variant::Text),
        policy: Policy::load(&args.policy)?,
        absent: Absent::load(&args.absent)?,
        corpus,
    };
    if args.measure {
        return Ok(measure::report(&docent));
    }
    let question = args.question.join(" ");
    if question.is_empty() {
        return Err("ask a question, or pass --measure".to_string());
    }
    Ok(docent.ask(&question))
}
