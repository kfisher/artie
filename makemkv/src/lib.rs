// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Crate responsible for running the MakeMKV command.

mod data;
mod error;
mod messages;

pub use crate::data::{Attribute, DiscInfo, StreamInfo, TitleInfo};
pub use crate::error::{Error, Result};
