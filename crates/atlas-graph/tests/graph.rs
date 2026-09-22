//! The concept graph, pinned on a corpus small enough to read.
//!
//! Every case here is one the real corpus contains: a tag spelled three
//! ways, a plural beside its singular, an acronym beside its expansion
//! that must *not* merge on its own, and a narrower idea inside a broader
//! one.

use atlas_core::{Concept, ConceptId, Resource, ResourceId, ResourceKind};
use atlas_corpus::{Corpus, validate};
use atlas_graph::build;
use atlas_graph::normalize::{groups, key};
use atlas_graph::overrides::Overrides;
use atlas_graph::report::near_misses;
use std::collections::BTreeSet;

/// A resource tagged with labels, plus the provisional concept records an
/// ingester would emit beside it.
fn source(id: &str, labels: &[&str]) -> Corpus {
    let mut corpus = Corpus::new();
    let mut resource = Resource::stub(
        ResourceId::new(id),
        ResourceKind::Post,
        id.to_string(),
        format!("https://example.invalid/{id}"),
    );
    resource.concepts = labels
        .iter()
        .map(|l| ConceptId::new(l.to_lowercase()))
        .collect();
    resource.source_hash = "pinned".into();
    corpus.concepts = labels.iter().map(|l| Concept::provisional(l)).collect();
    corpus.resources.push(resource);
    corpus
}

fn corpora() -> Vec<Corpus> {
    vec![
        source(
            "blog:one",
            &["chain-of-thought", "MoE", "agents", "attention"],
        ),
        source(
            "blog:two",
            &["Chain of Thought", "agent", "flash-attention"],
        ),
        source("video:three", &["mixture of experts", "Attention"]),
    ]
}

fn overrides() -> Overrides {
    let text = r#"Overrides(
        merge: { "mixture-of-experts": ["moe"] },
        parents: { "flash-attention": ["attention"] },
        labels: { "mixture-of-experts": "Mixture of Experts" },
        keep_apart: [],
    )"#;
    ron::from_str(text).expect("the fixture overrides parse")
}

fn built() -> (Corpus, String, Vec<String>) {
    build(&corpora(), &overrides())
}

#[test]
fn one_idea_spelled_three_ways_is_one_concept() {
    assert_eq!(key("Chain of Thought"), "chain-of-thought");
    assert_eq!(key("chain_of/thought"), "chain-of-thought");
    let (corpus, _, _) = built();
    let ids: Vec<&str> = corpus.concepts.iter().map(|c| c.id.as_str()).collect();
    assert!(ids.contains(&"chain-of-thought"));
    assert_eq!(ids.iter().filter(|id| id.contains("chain")).count(), 1);
    assert!(validate(&corpus).is_empty(), "{:?}", validate(&corpus));
}

#[test]
fn a_plural_folds_only_when_its_singular_is_there_too() {
    let seen = [("agents".to_string(), 2), ("agent".to_string(), 1)];
    assert_eq!(groups(&seen, &[]).len(), 1, "both spellings present");
    let alone = [("agents".to_string(), 2)];
    assert_eq!(groups(&alone, &[])[0].key, "agents", "nothing is stemmed");
    let apart = [("agents".to_string(), 1), ("agent".to_string(), 1)];
    let blocked = groups(&apart, &[("agents".into(), "agent".into())]);
    assert_eq!(blocked.len(), 2, "the override keeps them apart");
}

#[test]
fn the_label_is_prose_and_singular_even_when_the_slug_is_commoner() {
    let (corpus, _, _) = built();
    let find = |id: &str| {
        corpus
            .concepts
            .iter()
            .find(|c| c.id.as_str() == id)
            .expect(id)
    };
    assert_eq!(find("chain-of-thought").label, "Chain of Thought");
    assert_eq!(find("agent").label, "agent", "not agents");
}

#[test]
fn an_acronym_is_reported_but_merged_only_by_an_override() {
    let keys: BTreeSet<String> = ["moe", "mixture-of-experts", "large-language-monkeys", "llm"]
        .iter()
        .map(|k| (*k).to_string())
        .collect();
    let near = near_misses(&keys);
    let acronyms: Vec<&str> = near
        .iter()
        .filter(|m| m.kind == "acronym")
        .map(|m| m.other.as_str())
        .collect();
    assert_eq!(acronyms, ["large-language-monkeys", "mixture-of-experts"]);
    let (corpus, _, _) = built();
    let moe = corpus
        .concepts
        .iter()
        .find(|c| c.id.as_str() == "mixture-of-experts")
        .expect("merged by the override");
    assert_eq!(moe.label, "Mixture of Experts");
    assert!(moe.aliases.contains(&"moe".to_string()));
    assert!(!corpus.concepts.iter().any(|c| c.id.as_str() == "moe"));
}

#[test]
fn every_resource_points_at_a_surviving_concept_and_back() {
    let (corpus, _, _) = built();
    let one = corpus
        .resources
        .iter()
        .find(|r| r.id.as_str() == "blog:one")
        .expect("present");
    let ids: Vec<&str> = one.concepts.iter().map(|c| c.as_str()).collect();
    assert!(
        ids.contains(&"mixture-of-experts"),
        "remapped from moe: {ids:?}"
    );
    assert!(!ids.contains(&"moe"));
    let attention = corpus
        .concepts
        .iter()
        .find(|c| c.id.as_str() == "attention")
        .expect("present");
    assert_eq!(attention.resources.len(), 2, "the reverse index is filled");
    let flash = corpus
        .concepts
        .iter()
        .find(|c| c.id.as_str() == "flash-attention")
        .expect("present");
    assert_eq!(flash.parents, [ConceptId::new("attention")]);
}

#[test]
fn an_override_naming_a_concept_nobody_wrote_is_reported() {
    let text = r#"Overrides(merge: { "kv-cache": ["nosuchconcept"] })"#;
    let ov: Overrides = ron::from_str(text).expect("parses");
    let (_, _, unknown) = build(&corpora(), &ov);
    assert_eq!(unknown, ["kv-cache", "nosuchconcept"]);
}

#[test]
fn the_report_names_both_the_merges_and_what_it_declined() {
    let (_, report, _) = built();
    assert!(report.contains("| `chain-of-thought` | Chain of Thought |"));
    assert!(report.contains("merged by override"), "{report}");
    assert!(report.contains("parent by override"));
}
