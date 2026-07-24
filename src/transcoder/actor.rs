// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO

use crate::actor::Response;
use crate::transcoder::Transcoder;

/// Requests for a transcode actor.
#[derive(Debug)]
pub enum TranscoderRequest {
    /// Get the current status of a transcoder.
    GetStatus {
        response: Response<Transcoder>,
    },
}

