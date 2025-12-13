// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Provides the builder for the trancode page.
//!
//! The transcode page is used to optionally transcode copied media to a different format using
//! HandBrake.

/// Builds the Transcode page.
pub fn build() -> gtk::Text {
    gtk::Text::builder()
        .text("TODO: TRANSCODE PAGE")
        .build()
}
