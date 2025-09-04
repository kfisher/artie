// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Handles processing Handbrake's JSON output.

pub(crate) mod json;
pub(crate) mod parser;

pub use parser::{Output, Parser};
