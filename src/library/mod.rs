// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Manages the video library.
//!
//! # Video Ingest
//!
//! After the titles are copied from the disc, the video and title information in the database can
//! be created using the [`process_copy_operation`] function.

mod ingest;

pub use ingest::process_copy_operation;
