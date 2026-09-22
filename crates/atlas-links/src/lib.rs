//! Whether every URL in the corpus still resolves, and the committed record
//! of the last time anyone looked.
//!
//! Its own crate because it is the only part of the build that speaks HTTP,
//! and because the coverage crate that consumes it should not grow a
//! network dependency to count what it already has.
//!
//! The check is a network call, so it is deliberate and its result is
//! committed: `just report` reads `cache/url-status.json` and never touches
//! the network, and `just report --check` re-checks and rewrites it. A gate
//! that needed someone else's site to be up would fail for reasons that
//! have nothing to do with this corpus.
//!
//! Not every failure is a broken link. A 404 is: the page is gone and a
//! visitor sent there gets nothing. A 403 or a 429 usually means a host
//! dislikes robots -- YouTube and some publishers answer that way -- and a
//! timeout means the network, not the link. Only the first kind fails the
//! build; the rest are reported so a person can look.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// What the last check found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// Answered 2xx or 3xx.
    Ok,
    /// Answered 404 or 410: gone, and a gate failure.
    Missing,
    /// Answered 401, 403, 405 or 429: a host that dislikes robots.
    Blocked,
    /// Did not answer: timeout, DNS, TLS. The network, not the link.
    Error,
}

/// One URL's last result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// What it answered.
    pub status: Status,
    /// The HTTP code, or 0 when there was no answer.
    pub code: u16,
    /// When it was checked, ISO 8601 date.
    pub checked: String,
}

/// `cache/url-status.json`: every URL anyone has checked.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Cache {
    /// URL to its last result.
    #[serde(default)]
    pub urls: BTreeMap<String, Record>,
}

impl Cache {
    /// Read the cache, or start an empty one if it does not exist yet.
    ///
    /// # Errors
    ///
    /// The file exists but is not this cache.
    pub fn load(path: &Path) -> Result<Self, String> {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Ok(Self::default());
        };
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Write the cache, sorted, with a trailing newline.
    ///
    /// # Errors
    ///
    /// The file cannot be written.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, text + "\n").map_err(|e| e.to_string())
    }

    /// Check every URL not already recorded, or all of them when `again`.
    ///
    /// The only network call in this crate. Sequential on purpose: the
    /// order the report is written in is the order the hosts were asked,
    /// and a coverage run is not a load test.
    pub fn refresh(&mut self, urls: &[String], again: bool, today: &str) -> usize {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(15)))
            .user_agent("sw-atlas link check (+https://github.com/software-wrighter-lab/sw-atlas)")
            .build()
            .new_agent();
        let mut checked = 0;
        for url in urls {
            if !again && self.urls.contains_key(url) {
                continue;
            }
            let (status, code) = probe(&agent, url);
            let record = Record {
                status,
                code,
                checked: today.to_string(),
            };
            self.urls.insert(url.clone(), record);
            checked += 1;
        }
        checked
    }
}

/// Ask one URL, HEAD first and GET if the host refuses to answer a HEAD.
fn probe(agent: &ureq::Agent, url: &str) -> (Status, u16) {
    let ask = |head: bool| {
        let sent = if head {
            agent.head(url).call()
        } else {
            agent.get(url).call()
        };
        match sent {
            Ok(response) => Some(response.status().as_u16()),
            Err(ureq::Error::StatusCode(code)) => Some(code),
            Err(_) => None,
        }
    };
    let code = match ask(true) {
        Some(405 | 501) | None => ask(false),
        answered => answered,
    };
    match code {
        Some(200..=399) => (Status::Ok, code.unwrap_or_default()),
        Some(404 | 410) => (Status::Missing, code.unwrap_or_default()),
        Some(401 | 403 | 405 | 429) => (Status::Blocked, code.unwrap_or_default()),
        Some(other) => (Status::Error, other),
        None => (Status::Error, 0),
    }
}
