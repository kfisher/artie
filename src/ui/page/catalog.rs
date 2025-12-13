// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Provides the builder for the catalog page.
//!
//! The catalog page is used to place copied or transcoded media in the correct location for the
//! media server. It will also be used to archive or delete video files.

/// Builds the Catalog page.
pub fn build() -> gtk::Text {
    gtk::Text::builder()
        .text("TODO: CAT PAGE")
        .build()
}
