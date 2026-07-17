// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! String validation functions.

/// Validation function for validating a string.
pub type Validator = fn(&str) -> bool;

/// Returns `true` if `value` is not empty and contains more then just whitespace characters to
/// indicate that it is valid or `false` otherwise.
pub fn not_empty_or_only_whitespace(value: &str) -> bool {
    !value.trim().is_empty()
}

/// Returns `true` if `value` is a valid 32 bit signed integer or `false` otherwise.
pub fn is_number_i32(value: &str) -> bool {
    value.parse::<i32>().is_ok()
}

/// Returns `true` if `value` is a valid 32 bit unsigned integer or `false` otherwise.
pub fn is_number_u32(value: &str) -> bool {
    value.parse::<u32>().is_ok()
}

/// Returns `true` if `value` is a valid 16 bit signed integer or `false` otherwise.
pub fn is_number_i16(value: &str) -> bool {
    value.parse::<i16>().is_ok()
}

/// Returns `true` if `value` is a valid 16 bit unsigned integer or `false` otherwise.
pub fn is_number_u16(value: &str) -> bool {
    value.parse::<u16>().is_ok()
}

/// Returns `true` if `value` is a valid 8 bit signed integer or `false` otherwise.
pub fn is_number_i8(value: &str) -> bool {
    value.parse::<i8>().is_ok()
}

/// Returns `true` if `value` is a valid 8 bit unsigned integer or `false` otherwise.
pub fn is_number_u8(value: &str) -> bool {
    value.parse::<u8>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_empty_or_only_whitespace_true_for_normal_string() {
        assert!(not_empty_or_only_whitespace("hello"));
    }

    #[test]
    fn not_empty_or_only_whitespace_true_for_string_with_surrounding_whitespace() {
        assert!(not_empty_or_only_whitespace("  hello  "));
    }

    #[test]
    fn not_empty_or_only_whitespace_false_for_empty_string() {
        assert!(!not_empty_or_only_whitespace(""));
    }

    #[test]
    fn not_empty_or_only_whitespace_false_for_spaces_only() {
        assert!(!not_empty_or_only_whitespace("   "));
    }

    #[test]
    fn not_empty_or_only_whitespace_false_for_tabs_and_newlines() {
        assert!(!not_empty_or_only_whitespace("\t\n\r  \n"));
    }

    #[test]
    fn is_number_i32_true_for_valid_positive() {
        assert!(is_number_i32("12345"));
    }

    #[test]
    fn is_number_i32_true_for_valid_negative() {
        assert!(is_number_i32("-12345"));
    }

    #[test]
    fn is_number_i32_true_for_boundaries() {
        assert!(is_number_i32(&i32::MAX.to_string()));
        assert!(is_number_i32(&i32::MIN.to_string()));
    }

    #[test]
    fn is_number_i32_false_for_out_of_range() {
        let too_big = (i32::MAX as i64 + 1).to_string();
        let too_small = (i32::MIN as i64 - 1).to_string();
        assert!(!is_number_i32(&too_big));
        assert!(!is_number_i32(&too_small));
    }

    #[test]
    fn is_number_i32_false_for_non_numeric() {
        assert!(!is_number_i32("abc"));
    }

    #[test]
    fn is_number_i32_false_for_empty_string() {
        assert!(!is_number_i32(""));
    }

    #[test]
    fn is_number_i32_false_for_decimal() {
        assert!(!is_number_i32("1.5"));
    }

    #[test]
    fn is_number_i32_false_for_whitespace_padded() {
        assert!(!is_number_i32(" 123 "));
    }

    #[test]
    fn is_number_u32_true_for_valid_value() {
        assert!(is_number_u32("12345"));
    }

    #[test]
    fn is_number_u32_true_for_boundaries() {
        assert!(is_number_u32("0"));
        assert!(is_number_u32(&u32::MAX.to_string()));
    }

    #[test]
    fn is_number_u32_false_for_negative() {
        assert!(!is_number_u32("-1"));
    }

    #[test]
    fn is_number_u32_false_for_out_of_range() {
        let too_big = (u32::MAX as u64 + 1).to_string();
        assert!(!is_number_u32(&too_big));
    }

    #[test]
    fn is_number_u32_false_for_non_numeric() {
        assert!(!is_number_u32("xyz"));
    }

    #[test]
    fn is_number_i16_true_for_valid_value() {
        assert!(is_number_i16("-100"));
    }

    #[test]
    fn is_number_i16_true_for_boundaries() {
        assert!(is_number_i16(&i16::MAX.to_string()));
        assert!(is_number_i16(&i16::MIN.to_string()));
    }

    #[test]
    fn is_number_i16_false_for_out_of_range() {
        let too_big = (i16::MAX as i32 + 1).to_string();
        let too_small = (i16::MIN as i32 - 1).to_string();
        assert!(!is_number_i16(&too_big));
        assert!(!is_number_i16(&too_small));
    }

    #[test]
    fn is_number_i16_false_for_non_numeric() {
        assert!(!is_number_i16("not_a_number"));
    }

    #[test]
    fn is_number_u16_true_for_valid_value() {
        assert!(is_number_u16("100"));
    }

    #[test]
    fn is_number_u16_true_for_boundaries() {
        assert!(is_number_u16("0"));
        assert!(is_number_u16(&u16::MAX.to_string()));
    }

    #[test]
    fn is_number_u16_false_for_negative() {
        assert!(!is_number_u16("-1"));
    }

    #[test]
    fn is_number_u16_false_for_out_of_range() {
        let too_big = (u16::MAX as u32 + 1).to_string();
        assert!(!is_number_u16(&too_big));
    }

    #[test]
    fn is_number_i8_true_for_valid_value() {
        assert!(is_number_i8("-42"));
    }

    #[test]
    fn is_number_i8_true_for_boundaries() {
        assert!(is_number_i8(&i8::MAX.to_string()));
        assert!(is_number_i8(&i8::MIN.to_string()));
    }

    #[test]
    fn is_number_i8_false_for_out_of_range() {
        let too_big = (i8::MAX as i16 + 1).to_string();
        let too_small = (i8::MIN as i16 - 1).to_string();
        assert!(!is_number_i8(&too_big));
        assert!(!is_number_i8(&too_small));
    }

    #[test]
    fn is_number_i8_false_for_non_numeric() {
        assert!(!is_number_i8("nope"));
    }

    #[test]
    fn is_number_u8_true_for_valid_value() {
        assert!(is_number_u8("42"));
    }

    #[test]
    fn is_number_u8_true_for_boundaries() {
        assert!(is_number_u8("0"));
        assert!(is_number_u8(&u8::MAX.to_string()));
    }

    #[test]
    fn is_number_u8_false_for_negative() {
        assert!(!is_number_u8("-1"));
    }

    #[test]
    fn is_number_u8_false_for_out_of_range() {
        let too_big = (u8::MAX as u16 + 1).to_string();
        assert!(!is_number_u8(&too_big));
    }

    #[test]
    fn is_number_u8_false_for_non_numeric() {
        assert!(!is_number_u8("not_a_number"));
    }
}
