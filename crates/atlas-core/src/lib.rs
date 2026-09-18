//! Canonical corpus schema for sw-atlas.
//!
//! The corpus is the authority for facts; a model only ever maps language
//! onto identifiers defined here. Nothing in this crate knows about
//! inference, browsers, or weights.
//!
//! The types themselves land in step 002 (`corpus-schema`). This crate
//! currently carries only the two constants every later artifact is
//! stamped with: the schema version a snapshot declares, and the name a
//! manifest records as its producer.

/// Version of the corpus schema this build reads and writes.
///
/// A published snapshot records this in its manifest. A runtime that
/// finds a snapshot declaring a different major version refuses it rather
/// than guessing at the difference.
pub const SCHEMA_VERSION: &str = "0.1";

/// Producer name recorded in every snapshot manifest.
pub const PRODUCER: &str = "sw-atlas";

#[cfg(test)]
mod tests {
    use super::{PRODUCER, SCHEMA_VERSION};

    #[test]
    fn schema_version_is_a_major_minor_pair() {
        let parts: Vec<&str> = SCHEMA_VERSION.split('.').collect();
        assert_eq!(parts.len(), 2, "schema version is major.minor");
        for part in parts {
            assert!(
                part.parse::<u32>().is_ok(),
                "schema version components are numbers: {part}"
            );
        }
    }

    #[test]
    fn producer_is_the_repository_name() {
        assert_eq!(PRODUCER, "sw-atlas");
    }
}
