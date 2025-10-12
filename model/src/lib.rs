// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Crate responsible for defining the shared data models.

use std::fmt::{Display, Formatter, Result};

/// Specifies the different types of media.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MediaType {
    #[default]
    Movie,
    Show,
}

impl MediaType {
    /// All available themes.
    pub const ALL: &'static [Self] = &[
        Self::Movie,
        Self::Show,
    ];
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            MediaType::Movie => write!(f, "Movie"),
            MediaType::Show => write!(f, "Show"),
        }
    }
}
