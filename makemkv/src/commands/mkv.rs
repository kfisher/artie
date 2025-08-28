// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use crate::commands::{ProcessOutput};
use crate::error::{Error, Result};
use crate::messages::{self, Message};

struct Processor {
}

impl Processor {
    /// Constructs a new `Processor` instance.
    fn new() -> Self {
        Processor { }
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessOutput for Processor {
    /// Process a message from MakeMKV.
    fn process_message(&mut self, _msg: Message) -> Result<()> {
        todo!()
    }

    /// Process a line of error output text from MakeMKV.
    fn process_error_output(&mut self, _line: &str) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
