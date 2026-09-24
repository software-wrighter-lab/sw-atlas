//! MB02 against a second implementation of the same rules, and against the
//! cases the rules exist for.
//!
//! The parity test is the important one. MB01 scores nine places by walking
//! each place's buckets; this crate inverts the index for speed, and an
//! inversion that is subtly not the same scorer would quietly invalidate every
//! comparison to MB01. So the test scores every confirmed question both ways --
//! the fast index, and a plain per-resource walk written independently from the
//! weights table -- and requires the rankings to agree.

use atlas_core::{
    Maturity, Provenance, Relation, RelationKind, Resource, ResourceId, ResourceKind,
};
use atlas_corpus::Corpus;
use atlas_decide::Intent;
use atlas_match::{Matcher, Variant, graph, intent};
use atlas_signals as signals;

/// The naive scorer: for each resource, for each query word, the heaviest
/// bucket holding it, else 0.8 of the heaviest bucket whose long token
/// overlaps it, then the bonuses. Written from the rules, not from the
/// crate's code.
fn naive(corpus: &Corpus, query: &str, with_text: bool) -> Vec<(String, f32)> {
    let mut out = Vec::new();
    for resource in &corpus.resources {
        let mut fields: Vec<(Vec<String>, f32)> = Vec::new();
        for id in &resource.concepts {
            let concept = corpus.concepts.iter().find(|c| c.id == *id);
            for text in concept
                .into_iter()
                .flat_map(|c| std::iter::once(c.label.clone()).chain(c.aliases.iter().cloned()))
            {
                fields.push((signals::tokens(&text), 4.0));
            }
        }
        for alias in &resource.aliases {
            fields.push((signals::tokens(alias), 3.0));
        }
        fields.push((signals::tokens(&resource.title), 2.0));
        fields.push((signals::tokens(&resource.summary), 1.5));
        if with_text {
            fields.push((signals::tokens(&resource.body), 1.0));
        }
        let mut score = 0.0;
        for word in signals::tokens(query) {
            score += weight_for(&fields, &word);
        }
        if score > 0.0 {
            let open = matches!(
                resource.maturity,
                Some(Maturity::Working | Maturity::Finished)
            );
            score += f32::from(u8::from(open)) * 0.5;
            score += f32::from(u8::from(resource.kind == ResourceKind::Demo)) * 0.5;
        }
        let lowered = query.to_lowercase();
        let named = resource
            .aliases
            .iter()
            .any(|a| a.chars().count() > 3 && lowered.contains(&a.to_lowercase()));
        if named {
            score += 4.0;
        }
        if score > 0.0 {
            out.push((resource.id.as_str().to_owned(), score));
        }
    }
    out.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    out
}

/// One word's weight against one resource's fields: exact first, and a partial
/// only when there is no exact hit anywhere in the resource.
fn weight_for(fields: &[(Vec<String>, f32)], word: &str) -> f32 {
    let exact = fields
        .iter()
        .filter(|(tokens, _)| tokens.iter().any(|t| t == word))
        .map(|(_, weight)| *weight)
        .fold(0.0f32, f32::max);
    if exact > 0.0 {
        return exact;
    }
    if word.chars().count() < 5 {
        return 0.0;
    }
    let overlap = |t: &String| {
        t.chars().count() >= 5
            && (t.contains(word)
                || word.contains(t.as_str())
                || (5..word.len()).any(|len| {
                    word.is_char_boundary(len)
                        && (t == &word[..len] || t == &word[word.len() - len..])
                }))
    };
    let partial = fields
        .iter()
        .filter(|(tokens, _)| tokens.iter().any(overlap))
        .map(|(_, weight)| *weight)
        .fold(0.0f32, f32::max);
    partial * 0.8
}

fn corpus() -> Option<Corpus> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text = std::fs::read_to_string(root.join("build/corpus/corpus.ron")).ok()?;
    ron::from_str(&text).ok()
}

fn questions() -> Vec<String> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut out = Vec::new();
    for set in [
        "paraphrase",
        "exact-name",
        "ambiguous",
        "off-topic",
        "meta",
        "follow-up",
    ] {
        let path = root.join(format!("sources/questions/{set}.ron"));
        let text = std::fs::read_to_string(path).unwrap_or_default();
        for line in text.lines() {
            if let Some(rest) = line.trim().strip_prefix("(text: \"")
                && let Some(end) = rest.find("\",")
            {
                out.push(rest[..end].replace("\\'", "'"));
            }
        }
    }
    out
}

#[test]
fn the_fast_index_scores_exactly_what_a_plain_walk_scores() {
    let Some(corpus) = corpus() else {
        println!("skipped: no corpus built; run `just ingest`");
        return;
    };
    let asked = questions();
    assert!(
        asked.len() > 380,
        "the confirmed sets should be here: {}",
        asked.len()
    );
    let matcher = Matcher::new(&corpus, Variant::Text);
    for query in &asked {
        let fast = matcher.answer(query, 5);
        let slow = naive(&corpus, query, true);
        let fast_ids: Vec<&str> = fast.hits.iter().map(|h| h.id.as_str()).collect();
        let slow_ids: Vec<&str> = slow.iter().take(5).map(|(id, _)| id.as_str()).collect();
        assert_eq!(fast_ids, slow_ids, "ranking differs for {query:?}");
        for (hit, (_, score)) in fast.hits.iter().zip(&slow) {
            assert!(
                (hit.score - score).abs() < 1e-4,
                "{query:?}: {hit:?} vs {score}"
            );
        }
    }
}

/// A corpus of two resources, built so every rule can be aimed at one of them.
fn fixture() -> Corpus {
    let mut corpus = Corpus::new();
    let mut post = Resource::stub(
        ResourceId::new("blog:one"),
        ResourceKind::Post,
        "Mixture of Experts".into(),
        "https://example.invalid/one".into(),
    );
    post.aliases = vec!["routed experts".into()];
    post.summary = "A microscope for a tiny model".into();
    post.body = "the spoken script mentions perceptrons".into();
    post.concepts = vec![atlas_core::ConceptId::new("mixture-of-experts")];
    corpus.concepts.push(atlas_core::Concept {
        id: atlas_core::ConceptId::new("mixture-of-experts"),
        label: "Mixture of Experts".into(),
        aliases: vec!["moe".into()],
        parents: Vec::new(),
        resources: Vec::new(),
    });
    let mut demo = Resource::stub(
        ResourceId::new("demo:one"),
        ResourceKind::Demo,
        "runnable".into(),
        "https://example.invalid/demo".into(),
    );
    demo.maturity = Some(Maturity::Working);
    demo.summary = "perceptron playground".into();
    corpus.relations.push(Relation {
        from: post.id.clone(),
        kind: RelationKind::Demos,
        to: demo.id.clone(),
        weight: 1.0,
        provenance: Provenance::Declared,
    });
    corpus.resources.extend([post, demo]);
    corpus
}

#[test]
fn a_concept_outweighs_a_title_and_an_alias_outweighs_both() {
    assert_eq!(
        signals::WEIGHTS,
        [4.0, 3.0, 2.0, 1.5, 1.0],
        "MB01's weights, unchanged"
    );
    let corpus = fixture();
    let matcher = Matcher::new(&corpus, Variant::Fields);
    // "moe" is a concept alias (4) and nothing else; the word is under five
    // characters, so no partial match can fire.
    let hits = matcher.answer("moe", 5).hits;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "blog:one");
    assert!((hits[0].score - 4.0).abs() < 1e-6, "{:?}", hits[0]);
}

#[test]
fn naming_a_thing_outright_earns_the_phrase_bonus() {
    let matcher = Matcher::new(&fixture(), Variant::Fields);
    let hits = matcher.answer("where are the routed experts", 5).hits;
    // routed (3) + experts (4, the concept label) + the +4 verbatim alias.
    assert!((hits[0].score - 11.0).abs() < 1e-6, "{:?}", hits[0]);
    assert!(hits[0].signals.iter().any(|s| s == "\"routed experts\""));
}

#[test]
fn the_text_variant_reads_the_body_and_the_field_variant_does_not() {
    let corpus = fixture();
    let fields = Matcher::new(&corpus, Variant::Fields);
    let text = Matcher::new(&corpus, Variant::Text);
    // The post's only "perceptrons" is in its body, so only the text variant
    // finds it. The demo's summary says "perceptron", which both variants read.
    let without = fields.answer("perceptrons", 5).hits;
    assert!(!without.iter().any(|h| h.id == "blog:one"), "{without:?}");
    let hits = text.answer("perceptrons", 5).hits;
    assert!(
        hits.iter().any(|h| h.id == "blog:one"),
        "the body is read now"
    );
    let hits: Vec<_> = hits.into_iter().filter(|h| h.id == "blog:one").collect();
    // The body holds the word outright, so it is weight 1 and not a partial.
    assert!((hits[0].score - 1.0).abs() < 1e-6, "{:?}", hits[0]);
    // The singular is a partial match against the body's plural: 0.8 of 1.
    // The demo outranks it by holding the word outright in its summary, which
    // is the ordering the weights are for.
    let partial = text.answer("perceptron", 5).hits;
    assert_eq!(partial[0].id, "demo:one");
    let post = partial
        .iter()
        .find(|h| h.id == "blog:one")
        .expect("scored too");
    assert!((post.score - 0.8).abs() < 1e-6, "{post:?}");
    assert!(text.signal_count() > fields.signal_count());
}

#[test]
fn a_demo_that_works_collects_both_bonuses() {
    let matcher = Matcher::new(&fixture(), Variant::Fields);
    let hits = matcher.answer("playground", 5).hits;
    // summary 1.5 + 0.5 working + 0.5 runnable.
    assert_eq!(hits[0].id, "demo:one");
    assert!((hits[0].score - 2.5).abs() < 1e-6, "{:?}", hits[0]);
}

#[test]
fn the_intent_rules_are_the_mockups_rules_in_the_mockups_order() {
    assert_eq!(intent::of("where is the apl exhibit"), Intent::Navigate);
    assert_eq!(intent::of("is the card reader finished"), Intent::Status);
    assert_eq!(intent::of("tell me about the 1130"), Intent::Story);
    assert_eq!(intent::of("what should i look at"), Intent::Recommend);
    assert_eq!(intent::of("what is apl"), Intent::Explain);
    assert_eq!(intent::of("again"), Intent::Story);
    assert_eq!(intent::of("the 1130"), Intent::Navigate, "the default");
    // A0 cannot express these two, which is the argument for a trained head.
    let corpus = fixture();
    let matcher = Matcher::new(&corpus, Variant::Fields);
    let asked = matcher.answer("where did you write about experts", 5);
    assert_ne!(
        asked.intent,
        Intent::FindResource,
        "no keyword can say this"
    );
    assert_ne!(asked.intent, Intent::Compare);
}

#[test]
fn one_hop_along_a_declared_relation_carries_evidence() {
    let corpus = fixture();
    let matcher = Matcher::new(&corpus, Variant::Fields);
    let hits = matcher.answer("moe", 5).hits;
    assert_eq!(hits.len(), 1, "only the post matches lexically");
    let spread = graph::spread(&corpus, &hits, 0.25);
    assert_eq!(spread.len(), 2, "the demo it declares is reached");
    assert_eq!(spread[1].id, "demo:one");
    assert!((spread[1].score - 1.0).abs() < 1e-6, "a quarter of 4.0");
    assert_eq!(spread[1].signals, ["via blog:one"]);
}
