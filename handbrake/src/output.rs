// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles processing Handbrake's JSON output.

pub(crate) mod json;
pub(crate) mod parser;

pub use parser::{Output, Parser};
