// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! UI utility functions.

use gtk::Entry;
use gtk::prelude::*;

/// CSS class added to form fields when its value is invalid.
pub const INVALID_CSS_CLASS: &str = "invalid";

/// Marks the entry as valid or invalid.
///
/// # Args
///
/// `entry`:  The entry widget whose CSS will be updated to add or remove the invalid css class
///           (see: [`INVALID_CSS_CLASS`])
///
/// `valid`:  Indicates if the entry's value is valid or invalid.
pub fn update_validity_style(entry: &Entry, valid: bool) {
    if valid {
        entry_valid(entry);
    } else {
        entry_invalid(entry);
    }
}

/// Marks the entry as valid.
///
/// This will remove the invalid css class (see: [`INVALID_CSS_CLASS`])
pub fn entry_valid(entry: &Entry) {
    entry.remove_css_class(INVALID_CSS_CLASS);
}

/// Marks the entry as invalid.
///
/// This will add the invalid css class (see: [`INVALID_CSS_CLASS`])
pub fn entry_invalid(entry: &Entry) {
    entry.add_css_class(INVALID_CSS_CLASS);
}
