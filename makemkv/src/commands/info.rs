// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use crate::commands::{ProcessOutput};
use crate::data::DiscInfo;
use crate::error::{Error, Result};
use crate::messages::{self, Message};

/// Handles processing output from the info command.
///
/// This processor will process messages used to construct the disc information from the data
/// output by the info command.
struct Processor {
    /// The information about the disc.
    ///
    /// This is constructed as this processor receives the `CINFO`, `TINFO`, and `SINFO` messages
    /// while the info command is running. It should not be considered valid until the command
    /// completes successfully.
    disc_info: DiscInfo,
}

impl Processor {
    /// Constructs a new `Processor` instance.
    fn new() -> Self {
        Processor {
            disc_info: DiscInfo::new(),
        }
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessOutput for Processor {
    /// Process a message from MakeMKV.
    fn process_message(&mut self, msg: Message) -> Result<()> {
        use messages::Message::*;
        match msg {
            Cinfo { id, code: _, value } => self.disc_info.add_attribute(id, &value),
            Sinfo {
                title_index,
                stream_index,
                id,
                code: _,
                value,
            } => self.disc_info.add_stream_attribute(
                title_index as usize,
                stream_index as usize,
                id,
                &value,
            ),
            Tinfo {
                title_index,
                id,
                code: _,
                value,
            } => self
                .disc_info
                .add_title_attribute(title_index as usize, id, &value),
            _ => Ok(()),
        }
    }

    /// Process a line of error output text from MakeMKV.
    fn process_error_output(&mut self, _line: &str) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Attribute;

    #[test]
    fn process_message_builds_disc_info() {
        let mut processor = Processor::new();

        let messages = vec![
            Message::Cinfo {
                id: Attribute::Name,
                code: 0,
                value: "Disc Name".to_owned(),
            },
            Message::Tinfo {
                title_index: 0,
                id: Attribute::Name,
                code: 0,
                value: "Title Name".to_owned(),
            },
            Message::Sinfo {
                title_index: 0,
                stream_index: 0,
                id: Attribute::Name,
                code: 0,
                value: "Stream Name".to_owned(),
            },
        ];

        for msg in messages {
            assert_eq!(processor.process_message(msg).is_ok(), true);
        }

        let disc_name = processor.disc_info.attributes.get(&Attribute::Name);
        assert_eq!(disc_name, Some(&"Disc Name".to_owned()));

        let title_info = processor.disc_info.titles[0].as_ref().unwrap();
        let title_name = title_info.attributes.get(&Attribute::Name);
        assert_eq!(title_name, Some(&"Title Name".to_owned()));

        let stream_info = title_info.streams[0].as_ref().unwrap();
        let stream_name = stream_info.attributes.get(&Attribute::Name);
        assert_eq!(stream_name, Some(&"Stream Name".to_owned()));
    }
}
