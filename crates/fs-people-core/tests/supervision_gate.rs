//! The gate of `is_supervised`: red as soon as the code or the schema holds a supervision field while
//! `SUPERVISION_NOT_BUILT` is still true.
//!
//! Pattern: Architecture test (reads the source tree; its own oracle: the list of words below).
//! Mutant: add a field `supervised_by` to `src/card/model.rs`, or a line with "supervis" in a schema,
//! and leave the constant `true`: this test is red.

// The test is about asserting on a constant: that is the gate.
#![allow(clippy::assertions_on_constants)]

use std::fs;
use std::path::Path;

use fs_people_core::supervision::{SUPERVISION_NOT_BUILT, is_supervised};

/// Files that may talk about supervision (this module and its docs), by file name (`lib.rs` only declares the module).
const ALLOWED: [&str; 2] = ["supervision.rs", "lib.rs"];

fn files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(&path, out);
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("rs" | "json" | "capnp")
        ) {
            out.push(path);
        }
    }
}

/// Where the card, the roles and the schema live: nothing here may name supervision.
fn mentions() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut all = Vec::new();
    files(&root, &mut all);
    all.iter()
        .filter(|p| !ALLOWED.contains(&p.file_name().unwrap().to_str().unwrap()))
        .filter(|p| {
            let text = fs::read_to_string(p).unwrap().to_lowercase();
            text.contains("supervis") || text.contains("betreu")
        })
        .map(|p| p.display().to_string())
        .collect()
}

#[test]
fn a_supervision_field_with_the_constant_still_true_is_red() {
    let found = mentions();
    assert!(
        !SUPERVISION_NOT_BUILT || found.is_empty(),
        "supervision appears in {found:?} but SUPERVISION_NOT_BUILT is still true: build `is_supervised` from the stored relation and set the constant to false"
    );
}

#[test]
fn while_not_built_nobody_is_supervised() {
    assert!(SUPERVISION_NOT_BUILT);
    assert_eq!(is_supervised(fs_objects::ObjectId::new()), Some(false));
}
