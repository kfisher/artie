// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Compression utilities.

use std::io::prelude::*;

use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::Result;

/// Compress the provided string.
///
/// # Args
///
/// `s`:  The string to compress.
///
/// # Errors
///
/// [`crate::Error::StdIo`] Raised if the provided string cannot be compressed.
pub fn compress(s: &str) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(s.as_bytes())?;
    encoder.finish().map_err(|e| e.into())
}
