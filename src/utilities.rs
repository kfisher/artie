// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! General utility/helper functions.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Returns the language name for the provided code.
///
/// # Args
///
/// `code`  The language code to look up.
pub fn expand_language_code(code: &str) -> String {
    iso_639_3_codes().get(code)
        .copied()
        .map(|s| s.to_string())
        .unwrap_or_else(|| code.to_string())
}

/// Returns the mapping of ISO 639-3 codes.
fn iso_639_3_codes() ->  &'static HashMap<&'static str, &'static str> {
    static THREE_LETTER_CODES: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    THREE_LETTER_CODES.get_or_init(|| HashMap::from([
        ("eng", "English"),
    ]))
}
