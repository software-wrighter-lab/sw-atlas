//! The two bonuses MB01 adds on top of the word weights, and the one that
//! reaches a resource no word touched.
//!
//! The verbatim-alias bonus is the interesting one. MB01 adds it *outside* the
//! "did anything match" test, so a resource whose alias appears anywhere in the
//! query scores 4 even when no word of the query weighed against it. It is also
//! a substring test rather than a whole-word one, which is a wart worth keeping
//! rather than fixing: "properly" contains "rope", so a post whose alias is
//! `rope` scores 4 on a question about rendering a readme. MB02 is comparable to
//! MB01 only while it makes the same mistakes.

use atlas_core::{Maturity, Resource, ResourceKind};

/// MB01's two matched-something bonuses, mapped onto a corpus that has no
/// campus: 0.5 for something a visitor can look at today, 0.5 for something
/// runnable.
pub fn of(resource: &Resource) -> f32 {
    let open = matches!(
        resource.maturity,
        Some(Maturity::Working | Maturity::Finished)
    );
    let runnable = resource.kind == ResourceKind::Demo;
    f32::from(u8::from(open)).mul_add(0.5, f32::from(u8::from(runnable)) * 0.5)
}

/// The alias of more than three characters that appears in the query verbatim,
/// if any.
pub fn named<'a>(aliases: &'a [String], query: &str) -> Option<&'a String> {
    let lowered = query.to_lowercase();
    aliases
        .iter()
        .find(|alias| alias.chars().count() > 3 && lowered.contains(&alias.to_lowercase()))
}

/// Every resource whose alias the query names, including ones no word of the
/// query weighed against.
pub fn all_named(aliases: &[Vec<String>], query: &str) -> Vec<u32> {
    aliases
        .iter()
        .enumerate()
        .filter(|(_, list)| named(list, query).is_some())
        .map(|(at, _)| u32::try_from(at).unwrap_or(u32::MAX))
        .collect()
}
