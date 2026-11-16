//! The packaged binary says its version without opening a window or a server (the start test of the release).
//!
//! Pattern: Example-based test of the real binary.

use std::process::{Command, Output};

fn run(flag: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fs-people"))
        .arg(flag)
        .output()
        .unwrap()
}

#[test]
fn the_version_is_printed_and_nothing_is_started() {
    for flag in ["--version", "-V"] {
        let out = run(flag);
        assert!(
            out.status.success(),
            "{flag}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let said = String::from_utf8_lossy(&out.stdout);
        assert_eq!(
            said.trim(),
            format!("fs-people {}", env!("CARGO_PKG_VERSION"))
        );
    }
}

#[test]
fn help_succeeds_and_prints_usage() {
    let out = run("--help");
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("fs-people"));
}

#[test]
fn an_unknown_option_fails_with_a_message() {
    let out = run("--bogus");
    assert!(!out.status.success());
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("unknown option"), "{said}");
}
