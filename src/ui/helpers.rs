// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! UI utility functions.

use std::time::Duration;

use gtk::Entry;
use gtk::glib;
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

/// Format a duration as a string.
///
/// This will format the provided time in the format "<hours>:<minutes>:<seconds>" where the hours
/// field will only be present when the amount of time is over an hour and the minutes and seconds
/// fields will always be two digits.
///
/// # Args
///
/// `total_seconds`:  The amount of time in secounds.
pub fn format_duration_secs(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
 
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// Formats the elapsed time duration into a string.
///
/// See [`format_duration_secs`] for additional information about the format.
///
/// # Args
///
/// `duration`:  The duration to format.
pub fn format_duration(duration: &Duration) -> String {
    format_duration_secs(duration.as_secs())
}

/// insert-text signal handler that restricts input to numbers only.
///
/// # Args
///
/// `entry`:  The entry widget that is being resticted.
///
/// `text`:
///
/// `position`:
pub fn number_only_insert_text(entry: &gtk::Editable, text: &str, _position: &mut i32) {
    const NUMBERS: &str = "0123456789";
    let filtered: String = text.chars()
        .filter(|c| NUMBERS.contains(*c))
        .collect();
    if filtered != text {
        glib::signal::signal_stop_emission_by_name(entry, "insert-text");
    }
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]

    use super::*;

    #[test]
    fn test_zero() {
        assert_eq!(format_duration_secs(0), "00:00");
    }
 
    #[test]
    fn test_seconds_only() {
        assert_eq!(format_duration_secs(7), "00:07");
        assert_eq!(format_duration_secs(45), "00:45");
    }
 
    #[test]
    fn test_minutes_and_seconds() {
        assert_eq!(format_duration_secs(60), "01:00");
        assert_eq!(format_duration_secs(90), "01:30");
        assert_eq!(format_duration_secs(599), "09:59");
    }
 
    #[test]
    fn test_exactly_one_hour() {
        assert_eq!(format_duration_secs(3600), "1:00:00");
    }
 
    #[test]
    fn test_hours_minutes_seconds() {
        assert_eq!(format_duration_secs(3661), "1:01:01");
        assert_eq!(format_duration_secs(7384), "2:03:04");
    }
 
    #[test]
    fn test_hours_omitted_when_zero() {
        // Just under one hour — no hours prefix
        assert_eq!(format_duration_secs(3599), "59:59");
    }
 
    #[test]
    fn test_large_hours() {
        assert_eq!(format_duration_secs(360000), "100:00:00");
    }
 
    #[test]
    fn test_minutes_and_seconds_are_zero_padded() {
        assert_eq!(format_duration_secs(3601), "1:00:01");
        assert_eq!(format_duration_secs(3660), "1:01:00");
    }
}
