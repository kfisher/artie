// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use iced::Element;
use iced::widget::{self, Column, Row};

use crate::Message;

pub fn view<'a>() -> Element<'a, Message> {
    widget::text("PLACEHOLDER: TRANSCODE PAGE").into()
}
