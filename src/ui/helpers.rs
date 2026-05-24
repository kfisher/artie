// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! UI utility functions.

use std::time::Duration;

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

/// Formats the elapsed time duration into a string.
pub fn format_elapsed_time(elapsed_time: &Duration) -> String {
    let total_seconds = elapsed_time.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
