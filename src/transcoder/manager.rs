// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO

use crate::{Mode, Result};
use crate::actor::{self, Response};
use crate::bus;
use crate::transcoder::{Handle, Message, TranscodeJob};

/// Requests for the transcode manager.
#[derive(Debug)]
pub enum ManagerRequest {
    /// Get list of transcode jobs.
    GetJobs {
        response: Response<Vec<TranscodeJob>>,
    },

    /// Get list of identifiers for each transcoder.
    GetTranscoders {
        response: Response<Vec<String>>,
    },
}

pub fn init(bus: &bus::Handle, mode: Mode) -> Result<Handle> {
    let msg_processor = MessageProcessor::new(bus.clone(), mode);
    let handle = actor::create_and_run("transcode manager", msg_processor);
    Ok(handle)
}

struct MessageProcessor {
    /// Handle used to send messages to other actors via the message bus.
    bus: bus::Handle,

    /// The mode the application is running in.
    mode: Mode,
}

impl MessageProcessor {
    fn new(bus: bus::Handle, mode: Mode) -> Self {
        Self { bus, mode }
    }
}

impl actor::MessageProcessor<Message> for MessageProcessor {
    async fn process(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Manager { request: _ } => {
                todo!()
            },
            Message::Transcoder { id: _, request: _ } => {
                todo!()
            },
        }
    }
}
