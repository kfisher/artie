// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Provides the builder for the copy page.
//!
//! The copy page is used to copy media from an optical drive using MakeMKV.

/// Builds the Copy page.
pub fn build() -> gtk::Text {
    gtk::Text::builder()
        .text("TODO: COPY PAGE")
        .build()
}
