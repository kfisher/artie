// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Compression utilities.

use std::io::prelude::*;

use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::{Error, Result};

/// Compress the provided string.
pub fn compress(s: &str) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(s.as_bytes())
        .map_err(|error| Error::CompressIo { error })?;
    encoder.finish()
        .map_err(|error| Error::CompressIo { error })
}
