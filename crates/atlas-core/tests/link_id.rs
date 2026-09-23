//! What a declared link becomes, pinned on the pairs that went wrong.
//!
//! Both cases here were found by reading the corpus rather than by reading
//! the code, and neither tripped a gate: a duplicate resource is related, so
//! it is not an orphan, and a resource titled `run` has a title, so nothing
//! reported it missing.

use atlas_core::{Link, Provenance, RelationKind, ResourceKind, edge, link_id};

fn link(kind: ResourceKind, url: &str, title: Option<&str>) -> Link {
    Link {
        kind,
        url: url.to_string(),
        title: title.map(str::to_string),
    }
}

#[test]
fn the_campus_page_declared_two_ways_is_one_resource() {
    // The real pair: the campus catalog links to its own front page once
    // plainly and once with a fragment, and the corpus held both.
    let plain = link(
        ResourceKind::Demo,
        "https://software-wrighter-lab.github.io/sw-campus/",
        None,
    );
    let fragment = link(
        ResourceKind::Demo,
        "https://software-wrighter-lab.github.io/sw-campus/#",
        None,
    );
    assert_eq!(link_id(&plain), link_id(&fragment));
    assert_eq!(
        link_id(&plain).as_str(),
        "demo:software-wrighter-lab.github.io/sw-campus"
    );
}

#[test]
fn a_fragment_that_points_somewhere_is_still_the_same_page() {
    let route = link(
        ResourceKind::Demo,
        "https://software-wrighter-lab.github.io/sw-campus/#/campus/computer-science",
        None,
    );
    assert_eq!(
        link_id(&route).as_str(),
        "demo:software-wrighter-lab.github.io/sw-campus",
        "a client-side route is a place within one page, not another artifact"
    );
}

#[test]
fn trailing_slashes_and_spaces_do_not_make_a_second_repository() {
    let bare = link(
        ResourceKind::Repo,
        "https://github.com/sw-embed/sw-cor24-apl",
        None,
    );
    let slashed = link(
        ResourceKind::Repo,
        "  https://github.com/sw-embed/sw-cor24-apl//  ",
        None,
    );
    assert_eq!(link_id(&bare), link_id(&slashed));
    assert_eq!(link_id(&bare).as_str(), "repo:sw-embed/sw-cor24-apl");
}

#[test]
fn a_video_is_still_identified_by_its_id() {
    let watch = link(
        ResourceKind::Video,
        "https://www.youtube.com/watch?v=2RzJP2ofuuU",
        None,
    );
    let short = link(ResourceKind::Video, "https://youtu.be/872RLMBzC_8/", None);
    assert_eq!(link_id(&watch).as_str(), "video:2RzJP2ofuuU");
    assert_eq!(link_id(&short).as_str(), "video:872RLMBzC_8");
}

#[test]
fn a_button_label_is_not_a_demo_title() {
    for label in ["run", "io demo", "Run", " play "] {
        let declared = link(
            ResourceKind::Demo,
            "https://sw-comp-history.github.io/ibm-1130-rs",
            Some(label),
        );
        let (resource, relation) = edge(&link_id(&declared), &declared);
        assert!(
            resource.title.is_empty(),
            "{label:?} is what the link says, not what the demo is called"
        );
        assert_eq!(relation.kind, RelationKind::Demos);
        assert_eq!(relation.provenance, Provenance::Declared);
    }
}

#[test]
fn a_real_name_survives_and_other_kinds_keep_theirs() {
    let named = link(
        ResourceKind::Demo,
        "https://sw-embed.github.io/web-sw-cor24-apl",
        Some("APL in the browser"),
    );
    let (demo, _) = edge(&link_id(&named), &named);
    assert_eq!(demo.title, "APL in the browser");

    // `code` is a button label on a demo and a plausible thing to call a
    // cited work, so the rule applies only where the label appears.
    let paper = link(
        ResourceKind::Paper,
        "https://arxiv.org/abs/1301.3781",
        Some("code"),
    );
    let (cited, _) = edge(&link_id(&paper), &paper);
    assert_eq!(
        cited.title, "code",
        "a citation's name is the declarer's business"
    );
}
