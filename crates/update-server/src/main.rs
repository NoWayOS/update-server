// SPDX-License-Identifier: MIT OR Apache-2.0
//! The `update-server` command.
//!
//! `update-server check BUNDLE...` checks each bundle file as the server
//! does before serving it, and prints what it holds or why it is refused.
//! It exits 0 if every bundle is whole, 1 if any is refused, 2 on a wrong
//! command line.

use std::io::Read as _;
use std::process::ExitCode;

use update_server::bundle::{Bundle, MAX_BUNDLE_LEN};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.split_first() {
        Some((command, files)) if command == "check" && !files.is_empty() => check(files),
        _ => {
            eprintln!("usage: update-server check BUNDLE...");
            ExitCode::from(2)
        }
    }
}

/// Checks each file, printing a line for each.
fn check(files: &[String]) -> ExitCode {
    let mut refused = false;
    for file in files {
        let shown = file.escape_debug();
        match read_bundle(file) {
            Ok(bundle) => {
                let manifest = bundle.manifest();
                println!(
                    "{shown}: {} {}, security version {}, trust list version {}, {} records",
                    manifest.product(),
                    manifest.version(),
                    manifest.security_version(),
                    bundle.trust_list().version(),
                    bundle.record_count(),
                );
            }
            Err(reason) => {
                eprintln!("{shown}: refused: {reason}");
                refused = true;
            }
        }
    }
    if refused {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Reads at most one byte more than a bundle may hold, then checks it.
fn read_bundle(path: &str) -> Result<Bundle, String> {
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(MAX_BUNDLE_LEN.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    Bundle::from_bytes(&bytes).map_err(|error| error.to_string())
}
