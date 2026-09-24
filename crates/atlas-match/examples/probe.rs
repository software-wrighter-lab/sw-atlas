//! Ask the matcher a few questions by hand, before any harness exists.

use atlas_match::{Matcher, Variant};

fn main() {
    let text = std::fs::read_to_string("build/corpus/corpus.ron").expect("run just ingest");
    let corpus: atlas_corpus::Corpus = ron::from_str(&text).expect("parses");
    let started = std::time::Instant::now();
    let matcher = Matcher::new(&corpus, Variant::Text);
    println!(
        "index built in {:?} over {} resources\n",
        started.elapsed(),
        corpus.resources.len()
    );
    for query in std::env::args().skip(1) {
        let began = std::time::Instant::now();
        let answer = matcher.answer(&query, 5);
        println!(
            "{query:?}  intent {:?}  in {:?}",
            answer.intent,
            began.elapsed()
        );
        for hit in &answer.hits {
            println!(
                "   {:>6.2}  {:<52} {}",
                hit.score,
                hit.id,
                hit.signals.join(" ")
            );
        }
        println!();
    }
}
