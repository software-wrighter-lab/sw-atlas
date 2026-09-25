//! The deterministic arms, and how each one answers a question.
//!
//! An arm is anything that proposes ranked candidates. Keeping them behind one
//! shape is what lets a matcher and, later, a model be compared at all: the
//! harness hands over a question and reads back a ranking, and knows nothing
//! about how the ranking was decided.

use atlas_eval::{Ranked, Score, run};
use atlas_match::{Matcher, Variant, graph};
use atlas_questions::Row;

/// The three arms of Saga 2: the mockup's fields, the same plus catalog prose,
/// and that plus one declared hop along the relations.
pub struct Arms {
    /// MB02: concepts, aliases, title, summary.
    pub fields: Matcher,
    /// MB02t: the same plus `body`.
    pub text: Matcher,
}

impl Arms {
    /// Index the corpus once per arm.
    pub fn new(corpus: &atlas_corpus::Corpus) -> Self {
        Self {
            fields: Matcher::new(corpus, Variant::Fields),
            text: Matcher::new(corpus, Variant::Text),
        }
    }

    /// Score all three arms over the same rows.
    pub fn score(
        &self,
        corpus: &atlas_corpus::Corpus,
        rows: &[&Row],
        depth: usize,
    ) -> [(&'static str, Score); 3] {
        let hop = |query: &str| {
            let answer = self.text.answer(query, depth);
            let spread = graph::spread(corpus, &answer.hits, 0.25);
            Ranked {
                ids: spread.into_iter().take(depth).map(|hit| hit.id).collect(),
                intent: format!("{:?}", answer.intent),
            }
        };
        [
            ("MB02", run(rows, &|q| ask(&self.fields, q, depth))),
            ("MB02t", run(rows, &|q| ask(&self.text, q, depth))),
            ("MB02g", run(rows, &hop)),
        ]
    }
}

/// One arm's ranking for one question.
fn ask(matcher: &Matcher, query: &str, depth: usize) -> Ranked {
    let answer = matcher.answer(query, depth);
    Ranked {
        ids: answer.hits.into_iter().map(|hit| hit.id).collect(),
        intent: format!("{:?}", answer.intent),
    }
}
