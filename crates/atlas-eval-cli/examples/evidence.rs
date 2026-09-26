//! Dump the evidence behind every confirmed question, so a policy's thresholds
//! can be fitted to data instead of chosen by taste.

use atlas_match::{Matcher, Variant};

fn main() {
    let text = std::fs::read_to_string("build/corpus/corpus.ron").expect("run just ingest");
    let corpus: atlas_corpus::Corpus = ron::from_str(&text).expect("parses");
    let matcher = Matcher::new(&corpus, Variant::Text);
    println!("set\ttop\tmargin\tbest\tsignals\twords\tcorrect1\tin5\thas_expect");
    for name in [
        "paraphrase",
        "exact-name",
        "ambiguous",
        "follow-up",
        "off-topic",
        "meta",
        "intent",
    ] {
        let path = format!("sources/questions/{name}.ron");
        let set = atlas_questions::QuestionSet::load(std::path::Path::new(&path)).expect("parses");
        for row in &set.rows {
            let answer = matcher.answer(&row.text, 5);
            let top = answer.hits.first().map_or(0.0, |h| h.score);
            let second = answer.hits.get(1).map_or(0.0, |h| h.score);
            let signals = answer.hits.first().map_or(0, |h| h.signals.len());
            let best = answer.hits.first().map_or(0.0, |h| h.best);
            let words = atlas_signals::tokens(&row.text).len();
            let correct = answer
                .hits
                .first()
                .is_some_and(|h| row.expect.contains(&h.id));
            let in5 = answer.hits.iter().any(|h| row.expect.contains(&h.id));
            println!(
                "{name}\t{top:.2}\t{:.2}\t{best:.1}\t{signals}\t{words}\t{}\t{}\t{}",
                top - second,
                u8::from(correct),
                u8::from(in5),
                u8::from(!row.expect.is_empty())
            );
        }
    }
}
