// SPDX-License-Identifier: MIT OR Apache-2.0
//! The bundle checks: a whole bundle is accepted and served as it came, and
//! each way of being broken is refused for its own reason.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "a test fails by panicking"
)]

use nowayos_ipc::encode;
use nowayos_update::chunk::{CHUNK_LEN, Part, RECORD_LEN, chunks};
use nowayos_update::manifest::{ManifestContents, ManifestFields, Version};
use nowayos_update::trust_list::{ListedKey, Purpose, TrustListContents};
use nowayos_update::{Manifest, NamedSignature, TrustList};
use proptest::prelude::*;
use sha2::{Digest, Sha256};
use update_server::bundle::{Bundle, BundleError, MAX_BUNDLE_LEN, MAX_IMAGE_LEN};

/// A signature no one made: the server never checks one.
const SIGNATURE: NamedSignature = NamedSignature {
    key_id: [2; 32],
    signature: [3; 64],
};

fn trust_list() -> Vec<u8> {
    let keys = [
        ListedKey {
            purpose: Purpose::TrustList,
            key: [1; 64],
        },
        ListedKey {
            purpose: Purpose::Image,
            key: [4; 64],
        },
    ];
    let list = TrustList::new(
        TrustListContents::new(7, false, &keys).unwrap(),
        &[SIGNATURE],
    )
    .unwrap();
    record(&list)
}

fn manifest_for(image: &[u8]) -> Vec<u8> {
    let fields = ManifestFields {
        product: "kiosk",
        version: Version {
            major: 1,
            minor: 2,
            patch: 3,
        },
        security_version: 5,
        trust_list_version: 7,
        development: false,
        image_len: image.len() as u64,
        image_sha256: Sha256::digest(image).into(),
        reference_values: &[],
    };
    record(&Manifest::new(
        ManifestContents::new(&fields).unwrap(),
        &SIGNATURE,
    ))
}

fn record<M: nowayos_ipc::Message>(message: &M) -> Vec<u8> {
    let mut out = vec![0; 2048];
    let len = encode(message, &mut out).unwrap();
    out.truncate(len);
    out
}

/// An image of `len` bytes that are not all alike.
fn image(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}

/// Each part as its chunk records, in the order given.
fn bundle(parts: &[(Part, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    for (part, bytes) in parts {
        for chunk in chunks(*part, bytes) {
            out.extend(record(&chunk));
        }
    }
    out
}

fn good() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let image = image(3 * CHUNK_LEN + 100);
    (trust_list(), manifest_for(&image), image)
}

fn good_bundle() -> Vec<u8> {
    let (list, manifest, image) = good();
    bundle(&[
        (Part::TrustList, &list),
        (Part::Manifest, &manifest),
        (Part::Image, &image),
    ])
}

#[test]
fn a_whole_bundle_is_accepted_and_served_as_it_came() {
    let bytes = good_bundle();
    let bundle = Bundle::from_bytes(&bytes).unwrap();
    assert_eq!(bundle.manifest().product(), "kiosk");
    assert_eq!(bundle.manifest().security_version(), 5);
    assert_eq!(bundle.trust_list().version(), 7);
    assert_eq!(bundle.record_count(), bytes.len() / RECORD_LEN);
    assert_eq!(bundle.as_bytes(), bytes.as_slice());
    assert!(bundle.records().all(|record| record.len() == RECORD_LEN));
    assert_eq!(
        bundle.records().flatten().copied().collect::<Vec<u8>>(),
        bytes
    );
}

#[test]
fn nothing_is_no_trust_list() {
    assert_eq!(
        Bundle::from_bytes(&[]).unwrap_err(),
        BundleError::MissingPart {
            part: Part::TrustList
        }
    );
}

#[test]
fn a_part_record_is_refused() {
    let mut bytes = good_bundle();
    bytes.push(0);
    assert_eq!(
        Bundle::from_bytes(&bytes).unwrap_err(),
        BundleError::NotWholeRecords {
            len: bytes.len() as u64
        }
    );
}

#[test]
fn a_record_that_is_no_chunk_is_refused_by_its_place() {
    let mut bytes = good_bundle();
    // The second record's reserved byte after its part code.
    bytes[RECORD_LEN + 8 + 1] = 1;
    assert!(matches!(
        Bundle::from_bytes(&bytes).unwrap_err(),
        BundleError::Chunk { index: 1, .. }
    ));
}

#[test]
fn a_bundle_without_its_trust_list_is_refused() {
    let (_, manifest, image) = good();
    let bytes = bundle(&[(Part::Manifest, &manifest), (Part::Image, &image)]);
    assert_eq!(
        Bundle::from_bytes(&bytes).unwrap_err(),
        BundleError::MissingPart {
            part: Part::TrustList
        }
    );
}

#[test]
fn a_bundle_without_its_image_is_refused() {
    let (list, manifest, _) = good();
    let bytes = bundle(&[(Part::TrustList, &list), (Part::Manifest, &manifest)]);
    assert_eq!(
        Bundle::from_bytes(&bytes).unwrap_err(),
        BundleError::MissingPart { part: Part::Image }
    );
}

#[test]
fn a_part_after_a_later_one_is_refused() {
    let (list, manifest, image) = good();
    let bytes = bundle(&[
        (Part::TrustList, &list),
        (Part::Manifest, &manifest),
        (Part::Image, &image),
        (Part::TrustList, &list),
    ]);
    let first_late = bytes.len() / RECORD_LEN - list.len().div_ceil(CHUNK_LEN);
    assert_eq!(
        Bundle::from_bytes(&bytes).unwrap_err(),
        BundleError::OutOfOrder {
            index: first_late,
            part: Part::TrustList
        }
    );
}

#[test]
fn a_gap_in_the_image_is_refused() {
    let mut bytes = good_bundle();
    let records = bytes.len() / RECORD_LEN;
    // Drop the image's second-to-last chunk.
    let at = (records - 2) * RECORD_LEN;
    bytes.drain(at..at + RECORD_LEN);
    assert!(matches!(
        Bundle::from_bytes(&bytes).unwrap_err(),
        BundleError::Offset {
            part: Part::Image,
            ..
        }
    ));
}

#[test]
fn an_image_cut_short_is_refused() {
    let (list, manifest, image) = good();
    let bytes = bundle(&[
        (Part::TrustList, &list),
        (Part::Manifest, &manifest),
        (Part::Image, &image[..image.len() - 1]),
    ]);
    assert_eq!(
        Bundle::from_bytes(&bytes).unwrap_err(),
        BundleError::ImageLength {
            manifest: image.len() as u64,
            found: image.len() as u64 - 1
        }
    );
}

#[test]
fn an_image_that_is_not_the_manifests_is_refused() {
    let (list, manifest, mut image) = good();
    image[0] ^= 1;
    let bytes = bundle(&[
        (Part::TrustList, &list),
        (Part::Manifest, &manifest),
        (Part::Image, &image),
    ]);
    assert_eq!(
        Bundle::from_bytes(&bytes).unwrap_err(),
        BundleError::ImageHash
    );
}

#[test]
fn a_trust_list_longer_than_its_record_is_refused() {
    let (mut list, manifest, image) = good();
    list.extend([0; CHUNK_LEN]);
    let bytes = bundle(&[
        (Part::TrustList, &list),
        (Part::Manifest, &manifest),
        (Part::Image, &image),
    ]);
    assert_eq!(
        Bundle::from_bytes(&bytes).unwrap_err(),
        BundleError::PartTooLong {
            part: Part::TrustList
        }
    );
}

#[test]
fn a_manifest_that_does_not_decode_is_refused() {
    let (list, mut manifest, image) = good();
    // The development mark: only 0 or 1 decode.
    manifest[8 + 1] = 2;
    let bytes = bundle(&[
        (Part::TrustList, &list),
        (Part::Manifest, &manifest),
        (Part::Image, &image),
    ]);
    assert!(matches!(
        Bundle::from_bytes(&bytes).unwrap_err(),
        BundleError::Manifest(_)
    ));
}

#[test]
fn a_bundle_longer_than_the_most_is_refused_before_it_is_read() {
    assert!(MAX_BUNDLE_LEN >= MAX_IMAGE_LEN.div_ceil(CHUNK_LEN as u64) * RECORD_LEN as u64);
    let bytes = vec![0; usize::try_from(MAX_BUNDLE_LEN).unwrap() + RECORD_LEN];
    assert_eq!(
        Bundle::from_bytes(&bytes).unwrap_err(),
        BundleError::TooLong {
            len: bytes.len() as u64
        }
    );
}

proptest! {
    /// Any bytes at all are refused or accepted, never a panic; whatever is
    /// accepted is served exactly as it came.
    #[test]
    fn any_bytes_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..4 * RECORD_LEN)) {
        if let Ok(bundle) = Bundle::from_bytes(&bytes) {
            prop_assert_eq!(bundle.as_bytes(), bytes.as_slice());
        }
    }

    /// One byte of a whole bundle changed: refused, or, where the server
    /// checks nothing (a signature's bytes), accepted and served as changed.
    #[test]
    fn one_byte_changed(at in 0..good_bundle().len(), flip in 1..=255u8) {
        let mut bytes = good_bundle();
        bytes[at] ^= flip;
        if let Ok(bundle) = Bundle::from_bytes(&bytes) {
            prop_assert_eq!(bundle.as_bytes(), bytes.as_slice());
        }
    }
}
