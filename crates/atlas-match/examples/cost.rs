//! What MB02 costs: index size, index time, and per-query latency over the
//! confirmed evaluation sets. A0's budget is 10 MiB and 10 ms.

use atlas_match::{Matcher, Variant};

fn questions() -> Vec<String> {
    let mut out = Vec::new();
    for set in [
        "paraphrase",
        "exact-name",
        "ambiguous",
        "off-topic",
        "meta",
        "follow-up",
    ] {
        let text =
            std::fs::read_to_string(format!("sources/questions/{set}.ron")).unwrap_or_default();
        for line in text.lines() {
            if let Some(rest) = line.trim().strip_prefix("(text: \"")
                && let Some(end) = rest.find("\",")
            {
                out.push(rest[..end].to_string());
            }
        }
    }
    out
}

fn main() {
    let text = std::fs::read_to_string("build/corpus/corpus.ron").expect("run just ingest");
    let corpus: atlas_corpus::Corpus = ron::from_str(&text).expect("parses");
    let asked = questions();
    for (name, variant) in [("MB02 ", Variant::Fields), ("MB02t", Variant::Text)] {
        let started = std::time::Instant::now();
        let matcher = Matcher::new(&corpus, variant);
        let build = started.elapsed();
        let mut times: Vec<u128> = Vec::new();
        let mut empty = 0;
        for query in &asked {
            let began = std::time::Instant::now();
            let answer = matcher.answer(query, 10);
            times.push(began.elapsed().as_micros());
            empty += usize::from(answer.hits.is_empty());
        }
        times.sort_unstable();
        let at = |p: usize| times[times.len() * p / 100] as f64 / 1000.0;
        println!(
            "{name}  signals {:>6}  index {:>6.1} ms  p50 {:>6.2} ms  p95 {:>6.2} ms  max {:>6.2} ms  no-match {empty}/{}",
            matcher.signal_count(),
            build.as_secs_f64() * 1000.0,
            at(50),
            at(95),
            *times.last().unwrap() as f64 / 1000.0,
            asked.len()
        );
    }
}
