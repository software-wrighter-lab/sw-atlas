//! Properties of the schema that later steps rely on and that a refactor
//! could quietly break.

use atlas_core::{Provenance, Resource, ResourceKind};

#[test]
fn a_video_can_hold_an_authored_script_without_widening_the_type() {
    // Video scripts exist as authored text in repositories not yet cloned
    // here (plan.md 12.3). When they arrive an ingester fills `body`; the
    // type does not change and nothing migrates.
    let text = r#"Resource(
        id: ResourceId("video:sw-apl-mvp"),
        kind: Video,
        title: "APL on COR24",
        url: "https://example.invalid/v",
        date: "2026-09-01",
        summary: "An APL interpreter running on a 24-bit machine.",
        body: "",
        concepts: [],
        aliases: [],
        maturity: None,
        source_hash: "test",
    )"#;
    let mut video: Resource = ron::from_str(text).expect("parses");
    assert_eq!(video.kind, ResourceKind::Video);
    assert!(
        video.body.is_empty(),
        "no script yet, and that is not absence"
    );
    video
        .body
        .push_str("Today we are going to run APL on a 24-bit machine.");
    assert!(!video.body.is_empty());
}

#[test]
fn provenance_orders_declared_before_teacher() {
    // Metrics must be recomputable with Teacher rows excluded, so the
    // three cases stay distinguishable and comparable.
    assert!(Provenance::Declared < Provenance::Derived);
    assert!(Provenance::Derived < Provenance::Teacher);
}
