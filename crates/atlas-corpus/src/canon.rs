//! Canonical form and content hash.
//!
//! A rebuild that changes nothing must produce an identical file. That is
//! what lets the nightly pipeline tell a factual change from a semantic
//! one, lets a manifest name the exact corpus a model was trained on, and
//! lets a returning visitor transfer only the shards that actually moved.

use crate::corpus::Corpus;
use ron::ser::PrettyConfig;
use sha2::{Digest, Sha256};

/// Sort every collection into the one order a canonical file may have.
///
/// Resources and concepts by identifier; relations by endpoints and kind.
/// Two ingesters that produce the same facts in different orders produce
/// the same bytes here.
pub fn sorted(corpus: &Corpus) -> Corpus {
    let mut out = corpus.clone();
    out.resources.sort_by(|a, b| a.id.cmp(&b.id));
    out.concepts.sort_by(|a, b| a.id.cmp(&b.id));
    out.relations
        .sort_by(|a, b| (&a.from, &a.kind, &a.to).cmp(&(&b.from, &b.kind, &b.to)));
    for resource in &mut out.resources {
        resource.concepts.sort();
        resource.aliases.sort();
    }
    for concept in &mut out.concepts {
        concept.parents.sort();
        concept.resources.sort();
        concept.aliases.sort();
    }
    out
}

/// Render a corpus in canonical RON.
///
/// # Errors
///
/// Returns the serializer error if the corpus cannot be represented, which
/// in practice means a float that is not a number.
pub fn to_ron(corpus: &Corpus) -> Result<String, ron::Error> {
    let config = PrettyConfig::new().struct_names(true).indentor("  ");
    ron::ser::to_string_pretty(&sorted(corpus), config)
}

/// SHA-256 of some bytes, lowercase hex.
pub fn content_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().fold(String::new(), |mut acc, byte| {
        acc.push_str(&format!("{byte:02x}"));
        acc
    })
}

/// Canonical RON and its hash, which is the pair every writer needs.
///
/// # Errors
///
/// Propagates a serializer error from [`to_ron`].
pub fn canonical(corpus: &Corpus) -> Result<(String, String), ron::Error> {
    let text = to_ron(corpus)?;
    let hash = content_hash(text.as_bytes());
    Ok((text, hash))
}
