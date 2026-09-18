//! The two constants every published artifact is stamped with.

use atlas_core::{PRODUCER, SCHEMA_VERSION};

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
