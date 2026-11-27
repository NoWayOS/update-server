// SPDX-License-Identifier: MIT OR Apache-2.0
//! A release's bundle, checked before it is served.
//!
//! A **bundle** is what a device receives for one release: the newest trust
//! list, the release's manifest and its image, as a run of `nowayos-update`
//! chunk records, [`RECORD_LEN`] bytes each, in that order. It is the stream
//! NoWayOS's `make update-bundles` writes and a device's update PD reads.
//!
//! The server checks that a bundle is **whole and consistent** before it
//! serves a byte of it, so that it never sends a device something broken:
//!
//! * every record decodes as a chunk;
//! * the parts come in order - trust list, manifest, image - with none
//!   missing;
//! * each part's chunks run on from offset 0, with no gap or overlap;
//! * the trust list and the manifest decode as their records;
//! * the image has the manifest's length and SHA-256.
//!
//! It never checks a signature, and nothing here makes a bundle trusted.
//! Whether a release is authorised is the device's decision alone, against
//! its own trust list and rollback index.

use core::fmt;

use nowayos_update::chunk::{CHUNK_LEN, Chunk, ChunkError, Part, RECORD_LEN};
use nowayos_update::{Manifest, ManifestError, TrustList, TrustListError, manifest, trust_list};
use sha2::{Digest, Sha256};

/// The longest image a bundle may carry: 64 MiB, five times the largest
/// NoWayOS image so far.
pub const MAX_IMAGE_LEN: u64 = 64 * 1024 * 1024;

/// The longest bundle: each part at its longest, in chunk records. A longer
/// file is refused before it is read.
pub const MAX_BUNDLE_LEN: u64 = records_for(trust_list::RECORD_LEN as u64)
    .saturating_add(records_for(manifest::RECORD_LEN as u64))
    .saturating_add(records_for(MAX_IMAGE_LEN))
    .saturating_mul(RECORD_LEN as u64);

/// How many chunk records carry `len` bytes.
const fn records_for(len: u64) -> u64 {
    len.div_ceil(CHUNK_LEN as u64)
}

/// A bundle that is whole and consistent: its records, and the trust list
/// and manifest they carry.
#[derive(Clone)]
pub struct Bundle {
    trust_list: TrustList,
    manifest: Manifest,
    records: Vec<u8>,
}

impl Bundle {
    /// Checks `bytes` as a whole bundle, and keeps a copy to serve.
    ///
    /// # Errors
    ///
    /// The first problem found, in the order the [module documentation](self)
    /// lists the checks.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BundleError> {
        let len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if len > MAX_BUNDLE_LEN {
            return Err(BundleError::TooLong { len });
        }
        let records = bytes.chunks_exact(RECORD_LEN);
        if !records.remainder().is_empty() {
            return Err(BundleError::NotWholeRecords { len });
        }
        let mut parts = Parts::default();
        for (index, record) in records.enumerate() {
            let chunk =
                Chunk::decode(record).map_err(|error| BundleError::Chunk { index, error })?;
            parts.take(index, &chunk)?;
        }
        let (trust_list, manifest) = parts.finish()?;
        Ok(Self {
            trust_list,
            manifest,
            records: bytes.to_vec(),
        })
    }

    /// The trust list the bundle carries.
    #[must_use]
    pub fn trust_list(&self) -> &TrustList {
        &self.trust_list
    }

    /// The release's manifest.
    #[must_use]
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// How many chunk records the bundle is.
    #[must_use]
    pub fn record_count(&self) -> usize {
        self.records.chunks_exact(RECORD_LEN).len()
    }

    /// The chunk records, in order, each [`RECORD_LEN`] bytes: what is sent
    /// to a device.
    pub fn records(&self) -> impl ExactSizeIterator<Item = &[u8]> + '_ {
        self.records.chunks_exact(RECORD_LEN)
    }

    /// The whole bundle, as it was checked.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.records
    }
}

impl fmt::Debug for Bundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bundle")
            .field("product", &self.manifest.product())
            .field("version", &self.manifest.version())
            .field("security_version", &self.manifest.security_version())
            .field("trust_list_version", &self.trust_list.version())
            .field("records", &self.record_count())
            .finish()
    }
}

/// Why a bundle was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BundleError {
    /// Longer than [`MAX_BUNDLE_LEN`].
    TooLong {
        /// Its length in bytes.
        len: u64,
    },
    /// Not a whole number of chunk records.
    NotWholeRecords {
        /// Its length in bytes.
        len: u64,
    },
    /// A record that does not decode as a chunk.
    Chunk {
        /// The record's place in the bundle, from 0.
        index: usize,
        /// Why it does not decode.
        error: ChunkError,
    },
    /// A chunk of a part that came before, after a later part's chunks.
    OutOfOrder {
        /// The record's place in the bundle, from 0.
        index: usize,
        /// The part it carries.
        part: Part,
    },
    /// A part with no chunks: the first missing, in bundle order.
    MissingPart {
        /// The part.
        part: Part,
    },
    /// A chunk that does not start where its part has got to: a gap, an
    /// overlap, or a part that does not start at offset 0.
    Offset {
        /// The record's place in the bundle, from 0.
        index: usize,
        /// The part it carries.
        part: Part,
        /// Where the part has got to.
        expected: u64,
        /// Where the chunk starts.
        found: u64,
    },
    /// A part longer than its record (the trust list, the manifest), or an
    /// image longer than [`MAX_IMAGE_LEN`].
    PartTooLong {
        /// The part.
        part: Part,
    },
    /// The trust list's bytes do not decode as one.
    TrustList(TrustListError),
    /// The manifest's bytes do not decode as one.
    Manifest(ManifestError),
    /// The image is not the length the manifest gives.
    ImageLength {
        /// The manifest's length.
        manifest: u64,
        /// The image's.
        found: u64,
    },
    /// The image's SHA-256 is not the manifest's.
    ImageHash,
}

impl fmt::Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong { len } => {
                write!(f, "{len} bytes, more than a bundle's {MAX_BUNDLE_LEN}")
            }
            Self::NotWholeRecords { len } => {
                write!(
                    f,
                    "{len} bytes is not a whole number of {RECORD_LEN}-byte chunk records"
                )
            }
            Self::Chunk { index, error } => write!(f, "record {index} is not a chunk: {error}"),
            Self::OutOfOrder { index, part } => {
                write!(
                    f,
                    "record {index} carries the {} after a later part",
                    name(*part)
                )
            }
            Self::MissingPart { part } => write!(f, "no {} in the bundle", name(*part)),
            Self::Offset {
                index,
                part,
                expected,
                found,
            } => write!(
                f,
                "record {index} starts the {} at {found}, where it has got to {expected}",
                name(*part)
            ),
            Self::PartTooLong { part } => write!(f, "the {} is too long", name(*part)),
            Self::TrustList(error) => write!(f, "the trust list does not decode: {error}"),
            Self::Manifest(error) => write!(f, "the manifest does not decode: {error}"),
            Self::ImageLength { manifest, found } => {
                write!(
                    f,
                    "the image is {found} bytes, and the manifest says {manifest}"
                )
            }
            Self::ImageHash => write!(f, "the image's SHA-256 is not the manifest's"),
        }
    }
}

impl core::error::Error for BundleError {}

/// A part's name in a message.
const fn name(part: Part) -> &'static str {
    match part {
        Part::TrustList => "trust list",
        Part::Manifest => "manifest",
        Part::Image => "image",
    }
}

/// A part's place in a bundle.
const fn rank(part: Part) -> u8 {
    match part {
        Part::TrustList => 0,
        Part::Manifest => 1,
        Part::Image => 2,
    }
}

/// The part that comes first after `rank`, if any.
const fn after(rank: Option<u8>) -> Part {
    match rank {
        None => Part::TrustList,
        Some(0) => Part::Manifest,
        _ => Part::Image,
    }
}

/// The parts, put together chunk by chunk.
#[derive(Default)]
struct Parts {
    /// The part the last chunk carried.
    current: Option<Part>,
    /// Where the current part has got to.
    end: u64,
    trust_list: Vec<u8>,
    manifest: Vec<u8>,
    image: Sha256,
    image_len: u64,
}

impl Parts {
    /// Takes the chunk at `index`.
    fn take(&mut self, index: usize, chunk: &Chunk) -> Result<(), BundleError> {
        let part = chunk.part();
        let current = self.current.map(rank);
        if current != Some(rank(part)) {
            if current.is_some_and(|current| rank(part) < current) {
                return Err(BundleError::OutOfOrder { index, part });
            }
            let next = after(current);
            if next != part {
                return Err(BundleError::MissingPart { part: next });
            }
            self.current = Some(part);
            self.end = 0;
        }
        if chunk.offset() != self.end {
            return Err(BundleError::Offset {
                index,
                part,
                expected: self.end,
                found: chunk.offset(),
            });
        }
        self.end = chunk.end();
        match part {
            Part::TrustList => append(
                &mut self.trust_list,
                chunk.bytes(),
                trust_list::RECORD_LEN,
                part,
            ),
            Part::Manifest => append(
                &mut self.manifest,
                chunk.bytes(),
                manifest::RECORD_LEN,
                part,
            ),
            Part::Image => {
                if self.end > MAX_IMAGE_LEN {
                    return Err(BundleError::PartTooLong { part });
                }
                self.image.update(chunk.bytes());
                self.image_len = self.end;
                Ok(())
            }
        }
    }

    /// The trust list and the manifest, once every part has come and the
    /// image matches the manifest.
    fn finish(self) -> Result<(TrustList, Manifest), BundleError> {
        if self.current != Some(Part::Image) {
            return Err(BundleError::MissingPart {
                part: after(self.current.map(rank)),
            });
        }
        let trust_list = TrustList::decode(&self.trust_list).map_err(BundleError::TrustList)?;
        let manifest = Manifest::decode(&self.manifest).map_err(BundleError::Manifest)?;
        if self.image_len != manifest.image_len() {
            return Err(BundleError::ImageLength {
                manifest: manifest.image_len(),
                found: self.image_len,
            });
        }
        let digest: [u8; 32] = self.image.finalize().into();
        if digest != manifest.image_sha256() {
            return Err(BundleError::ImageHash);
        }
        Ok((trust_list, manifest))
    }
}

/// Appends `bytes` to a part held whole, at most `max` bytes in all.
fn append(
    part_bytes: &mut Vec<u8>,
    bytes: &[u8],
    max: usize,
    part: Part,
) -> Result<(), BundleError> {
    match part_bytes.len().checked_add(bytes.len()) {
        Some(len) if len <= max => {
            part_bytes.extend_from_slice(bytes);
            Ok(())
        }
        _ => Err(BundleError::PartTooLong { part }),
    }
}
