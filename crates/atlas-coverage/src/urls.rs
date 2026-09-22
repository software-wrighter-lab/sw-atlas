//! The URLs the corpus names, and what the last check said about them.

use atlas_corpus::Corpus;
use atlas_links::{Cache, Status};

/// URL results for one corpus, counted and named.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
    /// Distinct URLs the corpus names.
    pub urls: usize,
    /// How many last answered 2xx or 3xx.
    pub ok: usize,
    /// Gone: the gated failure.
    pub missing: Vec<String>,
    /// Hosts that refused a robot, named so a person can look.
    pub blocked: Vec<String>,
    /// Did not answer, named for the same reason.
    pub errored: Vec<String>,
    /// Never checked, so the gate cannot speak for them.
    pub unchecked: usize,
}

/// Every URL the corpus names, once, in order.
pub fn of(corpus: &Corpus) -> Vec<String> {
    let mut urls: Vec<String> = corpus
        .resources
        .iter()
        .map(|r| r.url.clone())
        .filter(|u| !u.is_empty())
        .collect();
    urls.sort();
    urls.dedup();
    urls
}

/// What the committed results say about those URLs.
pub fn stats(corpus: &Corpus, cache: &Cache) -> Stats {
    let urls = of(corpus);
    let found: Vec<Option<Status>> = urls
        .iter()
        .map(|url| cache.urls.get(url).map(|record| record.status))
        .collect();
    let named = |want: Status| -> Vec<String> {
        urls.iter()
            .zip(&found)
            .filter(|(_, status)| **status == Some(want))
            .map(|(url, _)| url.clone())
            .collect()
    };
    Stats {
        urls: urls.len(),
        ok: named(Status::Ok).len(),
        missing: named(Status::Missing),
        blocked: named(Status::Blocked),
        errored: named(Status::Error),
        unchecked: found.iter().filter(|status| status.is_none()).count(),
    }
}
