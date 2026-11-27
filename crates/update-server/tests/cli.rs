// SPDX-License-Identifier: MIT OR Apache-2.0
//! The `update-server` command: its exit codes and what it prints.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test fails by panicking"
)]

use std::process::Command;

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_update-server"))
        .args(args)
        .output()
        .unwrap();
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

#[test]
fn a_wrong_command_line_prints_the_usage() {
    for args in [&[][..], &["check"][..], &["serve", "x"][..]] {
        let (code, _, err) = run(args);
        assert_eq!(code, 2, "{args:?}");
        assert!(
            err.contains("usage: update-server check BUNDLE..."),
            "{args:?}"
        );
    }
}

#[test]
fn a_file_that_is_not_a_bundle_is_refused() {
    let path =
        std::env::temp_dir().join(format!("update-server-cli-{}.bundle", std::process::id()));
    std::fs::write(&path, b"not a bundle").unwrap();
    let (code, out, err) = run(&["check", path.to_str().unwrap()]);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(code, 1);
    assert!(out.is_empty());
    assert!(
        err.contains("refused: 12 bytes is not a whole number of 512-byte chunk records"),
        "{err}"
    );
}

#[test]
fn a_missing_file_is_refused() {
    let (code, _, err) = run(&["check", "/nonexistent/update-server.bundle"]);
    assert_eq!(code, 1);
    assert!(err.contains("refused:"), "{err}");
}
