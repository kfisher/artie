// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;

use crate::messages::{self, Message};
use crate::{Error, Result};

mod info;
mod mkv;

/// Trait for processing output from running MakeMKV commands.
pub(crate) trait ProcessOutput {
    /// Process a message from MakeMKV.
    fn process_message(&mut self, msg: Message) -> Result<()>;

    /// Process a line of error output text from MakeMKV.
    fn process_error_output(&mut self, line: &str) -> Result<()>;
}

/// Trait for running MakeMKV commands.
///
/// The expected use of this is to construct using `new`, add arguments using `add_arg`, and then
/// running with `run`. Once the command is running, `wait` can be used to wait for the command to
/// complete or `kill` to forcibly stop the command.
///
/// While the command is running, the output and error output can be processed by using the streams
/// returned by `run`. Note that if both output and error output need to be processed, they will
/// need to be handled in separate threads.
pub(crate) trait RunCommand {
    /// Constructs a new instance.
    fn new() -> Self;

    /// Adds an argument to the command.
    fn add_arg<T: AsRef<OsStr>>(&mut self, arg: T);

    /// Runs the command and returns the output and error streams or the appropriate error if the
    /// command could not be started.
    ///
    /// This will not block. Call [`RunCommand::wait`] to wait for the command to complete.
    fn run(&mut self) -> Result<CommandOutput>;

    /// Wait for the command to complete returning its exit status or an error if the command
    /// hasn't been started yet.
    fn wait(&mut self) -> Result<ExitStatus>;

    /// Forces the command to exit.
    ///
    /// Returns `Ok(())` if the command has already exited or an error if the command hasn't been
    /// started yet by calling `run`. This will call wait after sending the kill command to ensure
    /// that the OS releases its resources. See docs for `std::process::Child` for more info.
    fn kill(&mut self) -> Result<()>;
}

/// Container for the output and error streams of a command.
pub(crate) struct CommandOutput {
    /// The output stream (e.g. `stdout`).
    out: Box<dyn Read + Send>,

    /// The error output stream (e.g. `stderr`).
    err: Box<dyn Read + Send>,
}

/// Runs command `cmd` and process its output using `proc`.
///
/// `cmd` is expected to be constructed with all desired arguments added, but should not have been
/// started yet.
///
/// # Panics
///
/// Panics when attempting to stop the subprocess fails after an error was received from one of the
/// output processing threads.
///
/// Panics when waiting for the subprocess to exit fails.
pub(crate) fn run_command(cmd: &mut impl RunCommand, proc: &mut impl ProcessOutput) -> Result<()> {
    let streams = cmd.run()?;

    let (tx, rx) = mpsc::channel::<ChannelData>();

    let out_tx = tx.clone();
    let out_thread = thread::spawn(move || {
        let reader = BufReader::new(streams.out);
        for line in reader.lines() {
            let line = line.map_err(Error::CommandOutThreadIoError)?;
            if out_tx.send(ChannelData::OutTxt(line)).is_err() {
                return Err(Error::CommandOutThreadSendError);
            }
        }
        Ok(())
    });

    let err_tx = tx.clone();
    let err_thread = thread::spawn(move || {
        let reader = BufReader::new(streams.err);
        for line in reader.lines() {
            let line = line.map_err(Error::CommandErrThreadIoError)?;
            if err_tx.send(ChannelData::ErrTxt(line)).is_err() {
                return Err(Error::CommandErrThreadSendError);
            }
        }
        Ok(())
    });

    // Must drop the original sender to avoid blocking indefinitely. Once this is dropped, the
    // remaining senders will remain open for as long as their respective threads are active. The
    // threads will exit once command completes and closes the I/O streams. Once all the senders
    // are closed, calling `recv` on the reader will fail causing the while loop below to exit.
    drop(tx);

    while let Ok(data) = rx.recv() {
        let result = match data {
            ChannelData::OutTxt(text) => {
                messages::parse_message(&text).and_then(|m| proc.process_message(m))
            }
            ChannelData::ErrTxt(text) => proc.process_error_output(&text),
        };
        if let Err(error) = result {
            // Calling kill() will also wait for the command to exit to ensure that the system
            // resources are released.
            cmd.kill()?;

            // If either thread panicked, its error will be returned instead of the error received
            // from the channel. This should be fine since the panic would have a higher severity
            // and should also be rare (if even possible).
            let _ = out_thread
                .join()
                .map_err(|_| Error::CommandOutThreadPanicked)?;
            let _ = err_thread
                .join()
                .map_err(|_| Error::CommandErrThreadPanicked)?;

            return Err(error);
        }
    }

    // Ignore the exit code since MakeMKV will sometimes return non-zero values even though it was
    // able to complete the requested task.
    let _ = cmd.wait()?;

    let _ = out_thread
        .join()
        .map_err(|_| Error::CommandOutThreadPanicked)?;
    let _ = err_thread
        .join()
        .map_err(|_| Error::CommandErrThreadPanicked)?;

    Ok(())
}

/// Represents the data sent through the channel used to relay output from a running command for
/// processing.
enum ChannelData {
    /// Line of text received from the output stream.
    OutTxt(String),

    /// Line of text received from the error output stream.
    ErrTxt(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::mem;

    #[test]
    fn run_command() {
        let mut cmd = TestRunCommand::new();
        cmd.set_stdout(&["TCOUNT:53"]);
        cmd.set_stderr(&["I cast fireball"]);

        let mut proc = TestProcessOutput::new();

        let result = super::run_command(&mut cmd, &mut proc);

        assert_eq!(result.is_ok(), true);

        assert_eq!(proc.messages.len(), 1);
        assert_eq!(proc.messages[0], Message::Tcount { count: 53 });

        assert_eq!(proc.errors.len(), 1);
        assert_eq!(proc.errors[0], "I cast fireball");

        assert_eq!(cmd.waited, true);
        assert_eq!(cmd.killed, false);
    }

    #[test]
    fn run_command_invalid_message() {
        let mut cmd = TestRunCommand::new();
        cmd.set_stdout(&["TCOUNT:INVALID"]);
        cmd.set_stderr(&[]);

        let mut proc = TestProcessOutput::new();

        let result = super::run_command(&mut cmd, &mut proc);

        if let Err(Error::InvalidMessageData { key, data, error }) = &result {
            assert_eq!(key, "TCOUNT");
            assert_eq!(data, "INVALID");
            assert!(error.contains("failed to convert data to int"));
        } else {
            panic!("Expected InvalidMessageData error");
        }

        assert_eq!(cmd.waited, false);
        assert_eq!(cmd.killed, true);
    }

    #[test]
    fn run_command_run_error() {
        let mut cmd = TestRunCommand::new();
        cmd.run_fail = true;
        let mut proc = TestProcessOutput::new();
        let result = super::run_command(&mut cmd, &mut proc);
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn run_command_wait_error() {
        let mut cmd = TestRunCommand::new();
        cmd.wait_fail = true;
        let mut proc = TestProcessOutput::new();
        let result = super::run_command(&mut cmd, &mut proc);
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn run_command_kill_error() {
        let mut cmd = TestRunCommand::new();
        cmd.set_stdout(&["TCOUNT:INVALID"]);
        cmd.kill_fail = true;
        let mut proc = TestProcessOutput::new();
        let result = super::run_command(&mut cmd, &mut proc);
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn run_command_process_message_error() {
        let mut cmd = TestRunCommand::new();
        cmd.set_stdout(&["TCOUNT:53"]);
        cmd.set_stderr(&["I cast fireball"]);

        let mut proc = TestProcessOutput::new();
        proc.process_message_fail = true;

        let result = super::run_command(&mut cmd, &mut proc);
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn run_command_process_error_output_error() {
        let mut cmd = TestRunCommand::new();
        cmd.set_stdout(&["TCOUNT:53"]);
        cmd.set_stderr(&["I cast fireball"]);

        let mut proc = TestProcessOutput::new();
        proc.process_error_fail = true;

        let result = super::run_command(&mut cmd, &mut proc);
        assert_eq!(result.is_ok(), false);
    }

    struct BadRead {}

    impl Read for BadRead {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::Other.into())
        }
    }

    struct TestProcessOutput {
        messages: Vec<Message>,
        errors: Vec<String>,
        process_message_fail: bool,
        process_error_fail: bool,
    }

    impl TestProcessOutput {
        fn new() -> Self {
            TestProcessOutput {
                messages: Vec::new(),
                errors: Vec::new(),
                process_message_fail: false,
                process_error_fail: false,
            }
        }
    }

    impl ProcessOutput for TestProcessOutput {
        fn process_message(&mut self, msg: Message) -> Result<()> {
            self.messages.push(msg);
            if !self.process_message_fail {
                Ok(())
            } else {
                Err(Error::ProcessMessageError)
            }
        }
        fn process_error_output(&mut self, line: &str) -> Result<()> {
            self.errors.push(line.to_string());
            if !self.process_error_fail {
                Ok(())
            } else {
                Err(Error::ProcessErrorOutputError)
            }
        }
    }

    struct TestRunCommand {
        stdout: Cursor<Vec<u8>>,
        stderr: Cursor<Vec<u8>>,
        waited: bool,
        killed: bool,
        run_fail: bool,
        wait_fail: bool,
        kill_fail: bool,
    }

    impl TestRunCommand {
        fn set_stdout(&mut self, lines: &[&str]) {
            let data = lines.join("\n");
            self.stdout = Cursor::new(data.into_bytes());
        }

        fn set_stderr(&mut self, lines: &[&str]) {
            let data = lines.join("\n");
            self.stderr = Cursor::new(data.into_bytes());
        }
    }

    impl RunCommand for TestRunCommand {
        fn new() -> Self {
            TestRunCommand {
                stdout: Cursor::default(),
                stderr: Cursor::default(),
                waited: false,
                killed: false,
                run_fail: false,
                wait_fail: false,
                kill_fail: false,
            }
        }

        fn add_arg<T: AsRef<OsStr>>(&mut self, _arg: T) {}

        fn run(&mut self) -> Result<CommandOutput> {
            if !self.run_fail {
                Ok(CommandOutput {
                    out: Box::new(mem::take(&mut self.stdout)),
                    err: Box::new(mem::take(&mut self.stderr)),
                })
            } else {
                Err(Error::CommandStartError)
            }
        }

        fn wait(&mut self) -> Result<ExitStatus> {
            self.waited = true;
            if !self.wait_fail {
                Ok(ExitStatus::default())
            } else {
                Err(Error::CommandKillError)
            }
        }

        fn kill(&mut self) -> Result<()> {
            self.killed = true;
            if !self.kill_fail {
                Ok(())
            } else {
                Err(Error::CommandKillError)
            }
        }
    }

    // Implement PartialEq for Message to allow comparison in tests. Defining it here since its
    // only needed for tests. Also, it only implements whats needed for the tests below.
    impl PartialEq for Message {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Message::Tcount { count: c1 }, Message::Tcount { count: c2 }) => c1 == c2,
                _ => false,
            }
        }
    }
}
