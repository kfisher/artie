// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO

use iced::futures::Stream;
use iced::stream;

// TODO
#[derive(Clone, Debug)]
pub enum Event {
    DriveDirector
}

// TODO
pub fn drive_director() -> impl Stream<Item = Event> + Send + 'static {
    stream::channel(10, async |mut _output| {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await
    })
}

