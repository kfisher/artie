// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;

use crate::messages::{self, Message};
use crate::{Error, Result};

/// Trait for processing output from running MakeMKV commands.
pub(crate) trait ProcessOutput {
    /// Process a message from MakeMKV.
    fn process_message(&self, msg: Message) -> Result<()>;

    /// Process a line of error output text from MakeMKV.
    fn process_error_output(&self, line: &str) -> Result<()>;
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
pub(crate) fn run_command(cmd: &mut impl RunCommand, proc: &impl ProcessOutput) -> Result<()> {
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
                messages::parse_message(&text).map(|m| proc.process_message(m))?
            }
            ChannelData::ErrTxt(text) => proc.process_error_output(&text),
        };
        if let Err(error) = result {
            // The following really should never fail since the process should be running or have
            // been run. Both cases should almost always succeed. If it does fail, panic since has
            // gone very wrong.
            //
            // Additionally, calling kill() will also wait for the command to exit to ensure that
            // the system resources are released.
            cmd.kill().expect("Failed to kill subprocess");

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
    //
    // Panicking here because this really should not be possible to fail if the command was started
    // successfully (even if the command already exited when this is called). So if the wait fail,
    // there is either a coding error or the program is in a state not accounted for.
    let _ = cmd.wait().expect("Waiting for process to exit failed");

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
