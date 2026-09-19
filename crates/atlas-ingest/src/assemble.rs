//! Assemble read posts into one validated corpus.

use atlas_blog::{Error, frontmatter, links, post};
use atlas_core::{Provenance, Relation, RelationKind, Resource, ResourceId};
use atlas_corpus::{Corpus, content_hash};
use std::path::Path;

/// Read every post in a `_posts` directory into a corpus.
///
/// # Errors
///
/// Fails if the directory cannot be read, or if any post's front matter
/// does not parse. A partial corpus is never returned: a post the blog can
/// render and this cannot read is a bug here, not a post to skip.
pub fn blog(posts: &Path) -> Result<Corpus, Error> {
    let mut paths: Vec<_> = std::fs::read_dir(posts)
        .map_err(Error::Io)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|e| e == "md" || e == "markdown"))
        .collect();
    paths.sort();
    let mut corpus = Corpus::new();
    let mut series: Vec<(String, u32, ResourceId)> = Vec::new();
    for path in &paths {
        add_post(&mut corpus, &mut series, path)?;
    }
    corpus.relations.extend(series_edges(&mut series));
    dedup(&mut corpus);
    Ok(corpus)
}

/// Read one post into the corpus under construction.
fn add_post(
    corpus: &mut Corpus,
    series: &mut Vec<(String, u32, ResourceId)>,
    path: &Path,
) -> Result<(), Error> {
    let text = std::fs::read_to_string(path).map_err(Error::Io)?;
    let front = frontmatter::parse(&text).map_err(|e| Error::FrontMatter {
        path: path.display().to_string(),
        cause: e.to_string(),
    })?;
    let stem = post::stem(path);
    let resource = post::resource(&front, &stem, content_hash(text.as_bytes()));
    if let (Some(name), Some(part)) = (&front.series, front.series_part) {
        series.push((name.clone(), part, resource.id.clone()));
    }
    for link in links::links(&front) {
        let (target, relation) = links::edge(&resource.id, &link);
        corpus.resources.push(target);
        corpus.relations.push(relation);
    }
    corpus.concepts.extend(post::concepts(&front));
    corpus.resources.push(resource);
    Ok(())
}

/// Edges joining consecutive parts of each series.
fn series_edges(parts: &mut [(String, u32, ResourceId)]) -> Vec<Relation> {
    parts.sort_by(|a, b| (&a.0, a.1).cmp(&(&b.0, b.1)));
    parts
        .windows(2)
        .filter(|w| w[0].0 == w[1].0)
        .map(|w| Relation {
            from: w[0].2.clone(),
            kind: RelationKind::SeriesNext,
            to: w[1].2.clone(),
            weight: 1.0,
            provenance: Provenance::Declared,
        })
        .collect()
}

/// Collapse the duplicates that several posts naming one target produce.
///
/// A resource read from its own source beats a stub a link created, and a
/// stub with a title beats one without.
fn dedup(corpus: &mut Corpus) {
    let known = |r: &Resource| {
        usize::from(!r.source_hash.is_empty()) * 2 + usize::from(!r.title.is_empty())
    };
    corpus
        .resources
        .sort_by(|a, b| (&a.id, known(b)).cmp(&(&b.id, known(a))));
    corpus.resources.dedup_by(|a, b| a.id == b.id);
    corpus.concepts.sort_by(|a, b| a.id.cmp(&b.id));
    corpus.concepts.dedup_by(|a, b| a.id == b.id);
    corpus
        .relations
        .sort_by(|a, b| (&a.from, &a.kind, &a.to).cmp(&(&b.from, &b.kind, &b.to)));
    corpus.relations.dedup();
}
