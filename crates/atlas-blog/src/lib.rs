//! Read the blog's `_posts` directory, from front matter alone.
//!
//! No model, no GPU, no teacher, and no body text. Every relation this
//! produces was written by a person at publication time and carries
//! [`atlas_core::Provenance::Declared`]; the reader's whole job is to not
//! lose them.
//!
//! This crate reads posts. Assembling them into a corpus, and writing it,
//! belongs to `atlas-ingest`. The blog directory is read-only and nothing
//! here writes to it.

pub mod frontmatter;
pub mod links;
pub mod post;

/// Why reading a post failed.
#[derive(Debug)]
pub enum Error {
    /// A file or directory could not be read.
    Io(std::io::Error),
    /// A post's front matter did not parse.
    FrontMatter {
        /// Which post.
        path: String,
        /// What the parser said.
        cause: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "reading posts: {error}"),
            Self::FrontMatter { path, cause } => write!(f, "{path}: {cause}"),
        }
    }
}

impl std::error::Error for Error {}
