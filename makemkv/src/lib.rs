// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Crate responsible for running the MakeMKV command.

mod messages;

/// Error type for the MakeMKV crate.
pub enum Error {
    /// Error when a message from MakeMKV cannot be parsed.
    ParseError(messages::Error),
}
