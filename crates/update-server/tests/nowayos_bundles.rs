// SPDX-License-Identifier: MIT OR Apache-2.0
//! The bundles NoWayOS's `make update-bundles` makes, with real keys and
//! signatures, checked as the server checks them. Run by `make bundles-check`,
//! which makes them first and names their folder in `UPDATE_SERVER_BUNDLES`.
//!
//! Every bundle NoWayOS's update tests refuse for a signature, a key, a
//! version or a development mark is whole, so the server serves it: those
//! refusals are the device's. Only `hash` - an image that is not its
//! manifest's - is broken, and the server refuses it.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test fails by panicking"
)]

use std::path::PathBuf;

use update_server::bundle::{Bundle, BundleError};

/// The bundles NoWayOS makes, and whether the server takes each.
const EXPECTED: [(&str, Option<BundleError>); 8] = [
    ("good", None),
    ("unsigned", None),
    ("unknown-key", None),
    ("removed-key", None),
    ("list-version", None),
    ("development", None),
    ("security-version", None),
    ("hash", Some(BundleError::ImageHash)),
];

#[test]
#[ignore = "needs NoWayOS's update bundles: make bundles-check"]
fn nowayos_bundles_are_served_or_refused_as_expected() {
    let folder = PathBuf::from(
        std::env::var_os("UPDATE_SERVER_BUNDLES")
            .expect("UPDATE_SERVER_BUNDLES names the folder make update-bundles wrote"),
    );
    let mut found: Vec<String> = std::fs::read_dir(&folder)
        .unwrap()
        .filter_map(|entry| {
            let name = entry.unwrap().file_name().into_string().unwrap();
            name.strip_suffix(".bundle").map(str::to_owned)
        })
        .collect();
    found.sort();
    let mut expected: Vec<String> = EXPECTED
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect();
    expected.sort();
    assert_eq!(
        found, expected,
        "NoWayOS's update bundles are not the ones this test knows"
    );
    for (name, refusal) in EXPECTED {
        let bytes = std::fs::read(folder.join(format!("{name}.bundle"))).unwrap();
        match (Bundle::from_bytes(&bytes), refusal) {
            (Ok(bundle), None) => assert_eq!(bundle.manifest().product(), "kiosk", "{name}"),
            (Err(error), Some(expected)) => assert_eq!(error, expected, "{name}"),
            (outcome, expected) => panic!("{name}: {outcome:?}, expected {expected:?}"),
        }
    }
}
