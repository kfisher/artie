// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handbrake output parser.

use std::io::{Cursor, Read, Seek, Write};

use crate::{Error, Progress, Result, Version};
use crate::output::json;

/// Specifies the different potential non-error outputs from the parser.
pub enum Output {
    None,
    Progress(Progress),
    Version(Version),
}

/// Parses progress information from Handbrake's output.
/// 
/// Handbrake will output progress information as JSON formatted text that gets printed over
/// multiple lines of output. This parser will store a buffer of those lines and then parse the
/// output to generate a progress update when it has the complete JSON object.
pub struct Parser {
    pub(crate) state: ParserState,
    pub(crate) cursor: Cursor<Vec<u8>>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            state: ParserState::Waiting,
            cursor: Cursor::new(Vec::new()),
        }
    }

    /// Parse a line of output.
    ///
    /// This will parse a line of Handbrake's standard output. If after parsing, the parser does
    /// not have a complete object, it will return `None`. Otherwise, it will return the data from
    /// the parsed object.
    ///
    /// Handbrake must be run with the `--json` flag for this method to work correctly.
    ///
    /// # Ignored Objects
    ///
    /// There are several different types of progress updates from Handbrake. This will only
    /// process the `WORKING` state project updates. The other updates only seem to happen at the
    /// end of the command and don't provide any useful information.
    pub fn parse(&mut self, text: &str) -> Result<Output> {
        match self.state {
            ParserState::Waiting => self.parse_waiting(text),
            _ => self.parse_reading(text),
        }
    }

    /// Parsing handling used when the parser is in the waiting state.
    ///
    /// In the waiting state, the parser is waiting to receive the start of a JSON object. In the
    /// Handbrake output, the JSON objects start with "Label: {" where Label corresponds to the
    /// type of object.
    ///
    /// If it successfully parses the start of an object, it will transition the parser into the
    /// appropriate state based on the object.
    fn parse_waiting(&mut self, text: &str) -> Result<Output> {
        if text == "Version: {" {
            self.state = ParserState::ReadingVersion;
        } else if text == "Progress: {" {
            self.state = ParserState::ReadingProgress;
        } else {
            return Err(Error::UnexpectedOutput { text: text.to_owned() });
        }

        // Start the JSON object in the internal buffer. Just need the opening bracket, not the
        // label that prefixed it.
        self.cursor.write(b"{").map_err(|e| Error::ProgressBufferWriteError { 
            text: text.to_owned(),
            error: e 
        })?;

        Ok(Output::None)
    }

    /// Parsing handling used when the parser is in a reading state.
    ///
    /// While in the reading state, output will be appended to an internal buffer. Once all data
    /// for the JSON object has be received, it will return the object and transition the parser
    /// back to the waiting state.
    fn parse_reading(&mut self, text: &str) -> Result<Output> {
        // Add the next line of output to the internal buffer.
        self.cursor.write(text.as_bytes()).map_err(|e| Error::ProgressBufferWriteError {
            text: text.to_owned(),
            error: e 
        })?;

        // Check for end of the root JSON object. It is currently assumed to be when a line
        // containing a closing bracket with no leading whitespace. While not the most robust way
        // to handle it, it far easier then tracking all internal opening and closing brackets.
        if text != "}" {
            return Ok(Output::None);
        }

        // Record the current position of the cursor and then rewind it back to the start so that
        // the buffer can be read by the parser.
        let bytes = self.cursor.position();
        self.rewind()?;

        let output = match self.state {
            ParserState::ReadingProgress => parse_progress(&mut self.cursor, bytes),
            ParserState::ReadingVersion => parse_version(&mut self.cursor, bytes),
            // Panic here because it should never be possible for `parse_reading` to be called
            // while in the `waiting` state.
            _ => panic!("Parser is in an invalid state"),
        }?;

        // Move the cursor back to the start prior to starting to write the next JSON object to the
        // buffer. Don't need to clear the existing data since we'll only ever read what was
        // written since this reset. 
        self.rewind()?;

        self.state = ParserState::Waiting;

        Ok(output)
    }

    /// Convenience function for resetting the cursor position to the start.
    fn rewind(&mut self) -> Result<()> {
        self.cursor.rewind().map_err(|e| Error::ParseOutputIoError { error: e })
    }
}

/// Represents the different states of [`Parser`]
#[derive(Debug)]
pub(crate) enum ParserState {
    /// The parser is wait to receive the start of the next JSON object.
    Waiting,

    /// The parser is reading a progress JSON object.
    ReadingProgress,

    /// The parser is reading a version JSON object.
    ReadingVersion,
}

/// Parses a [`json::Progress`] object and translates it into a [`Progress`] object.
///
/// This expects the content between the current position in the reader plus the provided number of
/// bites to contain a complete JSON object.
fn parse_progress<T>(reader: &mut T, bytes: u64) -> Result<Output> 
where
    T: Read
{
    let mut reader = reader.take(bytes);
    let progress = json::parse_progress(&mut reader)?;
    
    let Some(progress) = progress.working else {
        return Ok(Output::None);
    };

    let progress = Progress {
        pass: progress.pass,
        pass_count: progress.pass_count,
        progress: (progress.progress * 100.0) as i32
    };

    Ok(Output::Progress(progress))
}

/// Parses a [`json::Version`] object and translates it into a [`Version`] object.
///
/// This expects the content between the current position in the reader plus the provided number of
/// bites to contain a complete JSON object.
fn parse_version<T>(reader: &mut T, bytes: u64) -> Result<Output> 
where
    T: Read
{
    let mut reader = reader.take(bytes);
    let version = json::parse_version(&mut reader)?;

    let version = Version {
        arch: version.arch,
        system: version.system,
        version: version.version_string,
    };

    Ok(Output::Version(version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let mut parser = Parser::new();

        let text = r#"Version: {
    "Arch": "x86_64",
    "Name": "HandBrake",
    "Official": true,
    "RepoDate": "2024-08-07 17:31:52",
    "RepoHash": "77f199ab02ff2e3bca4ca653e922e9fef67dec43",
    "System": "MinGW",
    "Type": "release",
    "Version": {
        "Major": 1,
        "Minor": 8,
        "Point": 2
    },
    "VersionString": "1.8.2"
}
"#;

        let mut output = Vec::new();
        for line in text.lines() {
            output.push(parser.parse(line).unwrap());
        }

        for result in &output[0..output.len() - 1] {
            match result {
                Output::None => (),
                _ => panic!("Did not expect output data until the final item"),
            }
        }

        if let Output::Version(version) = &output[output.len() - 1] {
            assert_eq!(version.arch, "x86_64");
            assert_eq!(version.system, "MinGW");
            assert_eq!(version.version, "1.8.2");
        } else {
            panic!("Expected version data")
        }

        let text = r#"Progress: {
    "State": "WORKING",
    "Working": {
        "ETASeconds": 1,
        "Hours": 0,
        "Minutes": 0,
        "Pass": 1,
        "PassCount": 2,
        "PassID": -1,
        "Paused": 0,
        "Progress": 0.094762548804283142,
        "Rate": 0.0,
        "RateAvg": 0.0,
        "Seconds": 1,
        "SequenceID": 1
    }
}
"#;
        let mut output = Vec::new();
        for line in text.lines() {
            output.push(parser.parse(line).unwrap());
        }

        for result in &output[0..output.len() - 1] {
            match result {
                Output::None => (),
                _ => panic!("Did not expect output data until the final item"),
            }
        }

        if let Output::Progress(progress) = &output[output.len() - 1] {
            assert_eq!(progress.pass, 1);
            assert_eq!(progress.pass_count, 2);
            assert_eq!(progress.progress, 9);
        } else {
            panic!("Expected progress data")
        }

        let text = r#"Progress: {
    "Muxing": {
        "Progress": 0.0
    },
    "State": "MUXING"
}
"#;
        let mut output = Vec::new();
        for line in text.lines() {
            output.push(parser.parse(line).unwrap());
        }

        for result in output {
            match result {
                Output::None => (),
                _ => panic!("Did not expect any output"),
            }
        }

        let text = r#"Progress: {
    "State": "WORKDONE",
    "WorkDone": {
        "Error": 0,
        "SequenceID": 1
    }
}
"#;
        let mut output = Vec::new();
        for line in text.lines() {
            output.push(parser.parse(line).unwrap());
        }

        for result in output {
            match result {
                Output::None => (),
                _ => panic!("Did not expect any output"),
            }
        }
    }
}
