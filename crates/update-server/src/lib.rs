// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Update Server: the server NoWayOS devices check in to for updates.
//!
//! A device built on NoWayOS takes an update as one signed system image. Its
//! update PD checks every byte against the trust anchors built into it and
//! its rollback index, then installs the image in the slot it is not running
//! and boots it once on trial. Nothing the server says is trusted for that:
//! a release is authorised by its signatures alone, so a compromised server
//! can withhold a release, never install one of its own.
//!
//! What the server is for is everything signatures cannot give a device:
//! serving the bundle it needs, and, as it grows, telling it which release
//! to take, when, and that the answer is fresh. It reads a **release
//! repository** - the signed trust lists, manifests and bundles of a
//! product's releases, as static files - and holds no key that signs an
//! image.
//!
//! So far:
//!
//! * [`bundle`]: a release's bundle - its trust list, manifest and image as
//!   the chunk records a device reads - checked whole before any of it is
//!   served.
//!
//! The records are NoWayOS's own (`nowayos-update`), from the NoWayOS
//! commit this repository records, so the server and the device read and
//! write them with the same code.

#![forbid(unsafe_code)]

pub mod bundle;

pub use bundle::{Bundle, BundleError};
