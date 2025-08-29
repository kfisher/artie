// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! MakeMKV commands.
//!
//! This module contains the functions for running the various MakeMKV commands and processing
//! their output. The primary commands are the "info" command which can be executed with the
//! [`run_info_command`] function and "mkv" which can be run with the [`run_mkv_command`] function.
//!
//! Each of these functions have a generic type parameter which is used to specify the runner type
//! to use. To actually run the command using the `makemkvcon` program, use [`OsRunner`]. The other
//! runners that exist are there for development and testing when actually copying a disc isn't
//! desired.

use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;

use crate::data::DiscInfo;
use crate::messages::{self, Message};
use crate::{Error, Result};

use crate::{COPY_CMD_LOG_FILENAME, DISC_INFO_FILENAME, INFO_CMD_LOG_FILENAME};

/// Runs the "info" MakeMKV command.
///
/// The "info" command extracts information about the contents of a DVD or Blu-ray. This
/// information is written to the [`DiscInfo`] field in `ctx`.
pub(crate) fn run_info_command<T: RunCommand>(ctx: &mut Context) -> Result<()> {
    let mut cmd = T::new();
    cmd.add_arg("--cache=1");
    cmd.add_arg("--noscan");
    cmd.add_arg("--progress=-same");
    cmd.add_arg("info");
    cmd.add_arg(format!("dev:{0}", ctx.device));

    run_command(ctx, &mut cmd)
}

/// Runs the "mkv" MakeMKV command.
///
/// The "mkv" command copies titles from a DVD or Blu-ray disc and saves them as MKV files.
pub(crate) fn run_mkv_command<T: RunCommand>(ctx: &mut Context) -> Result<()> {
    let mut cmd = T::new();
    cmd.add_arg("--robot");
    cmd.add_arg("--noscan");
    cmd.add_arg("--progress=-same");
    cmd.add_arg("mkv");
    cmd.add_arg(format!("dev:{0}", ctx.device));
    cmd.add_arg("all");
    cmd.add_arg(ctx.outdir.clone());

    run_command(ctx, &mut cmd)
}

/// Context object for running MakeMKV commands.
pub(crate) struct Context {
    /// The device path to the target optical drive.
    pub(crate) device: String,

    /// The output directory where MakeMKV should write the MKV files to.
    pub(crate) outdir: PathBuf,

    /// Information about the disc in the drive, if available.
    ///
    /// This will only contain a value if a command is run that generates the required information
    /// messages which is currently only the `info` command.
    pub(crate) disc_info: Option<DiscInfo>,
}

impl Context {
    /// Constructs a new context for the optical drive specified by the provided device path and
    /// output directory.
    pub(crate) fn new(device: &str, outdir: &Path) -> Context {
        Context {
            device: device.to_owned(),
            outdir: outdir.to_owned(),
            disc_info: None,
        }
    }

    /// Gets the path to where the "info" command output will be logged to.
    pub(crate) fn info_log_path(&self) -> PathBuf {
        self.outdir.join(INFO_CMD_LOG_FILENAME)
    }

    /// Gets the path to where the "mkv" command output will be logged to.
    pub(crate) fn mkv_log_path(&self) -> PathBuf {
        self.outdir.join(COPY_CMD_LOG_FILENAME)
    }

    /// Gets the path to where the extracted disc information will be written to.
    pub(crate) fn disc_info_path(&self) -> PathBuf {
        self.outdir.join(DISC_INFO_FILENAME)
    }
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
    ///
    /// This will not have any effect on a command that has already started running.
    fn add_arg<T: AsRef<OsStr>>(&mut self, arg: T);

    /// Runs the command and returns the output and error streams.
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

/// Runs an MakeMKV command.
fn run_command<T: RunCommand>(ctx: &mut Context, cmd: &mut T) -> Result<()> {
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
    // threads will exit once command completes and closes the I/O streams.
    drop(tx);

    // Once all the senders are closed, calling `recv` on the reader will fail causing the while
    // loop below to exit.
    while let Ok(data) = rx.recv() {
        let result = match data {
            ChannelData::OutTxt(text) => process_output_line(ctx, &text),
            ChannelData::ErrTxt(text) => process_error_line(ctx, &text),
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

/// Processes a line of output text (standard out) from a running MakeMKV command.
///
/// For each line of output text, this will append the line to the logfile in the provided context
/// if specified. It will then parse the line into a [`Message`] and perform the appropriate action
/// based on the message type.
///
/// The attribute contained in `CINFO`, `TINFO`, and `SINFO` messages are added to the context's
/// disc information data.
///
/// TODO: Progress messages and general messages.
///
/// `DRV` and `TCOUNT` messages are ignored.
fn process_output_line(ctx: &mut Context, line: &str) -> Result<()> {
    // TODO: Append the raw text to the log file.

    use Message::*;
    match messages::parse_message(line)? {
        Cinfo { id, code: _, value } => ctx
            .disc_info
            .get_or_insert_default()
            .add_attribute(id, &value)?,
        Tinfo { title_index, id, code: _, value } => ctx
            .disc_info
            .get_or_insert_default()
            .add_title_attribute(title_index as usize, id, &value)?,
        Sinfo { title_index, stream_index, id, code: _, value } => ctx
            .disc_info
            .get_or_insert_default()
            .add_stream_attribute(title_index as usize, stream_index as usize, id, &value,)?,
        // Msg { code, flags, count, message, format, args } => todo!(),
        // Prgc { code, id, name } => todo!(),
        // Prgt { code, id, name } => todo!(),
        // Prgv { suboperation, operation, max } => todo!(),
        _ => (),
    };

    Ok(())
}

/// Processes a line of error text (standard error) from a running MakeMKV command.
///
/// For each line of output text, this will append the line to the logfile in the provided context
/// if specified. TODO: document how it notifies the caller.
fn process_error_line(_ctx: &mut Context, _line: &str) -> Result<()> {
    // TODO: Append the raw text to the log file.
    // TODO: Notify the caller
    todo!()
}

/// Command runner which makes system calls to run MakeMKV commands.
///
/// This is the default runner used to run commands. Other types of runners exist mainly for
/// testing and development when you don't want to actually copy a disc.
pub(crate) struct OsRunner {
    cmd: Command,
    child: Option<Child>,
}

impl RunCommand for OsRunner {
    /// Constructs a runner instance.
    fn new() -> Self {
        // TODO: Need to be able to specify the path to the executable. For now, assume it's in
        //       the PATH. Defining an environment variable should be simple enough and suffice.
        OsRunner { cmd: Command::new("makemkvcon"), child: None }
    }

    /// Adds a new argument to the command.
    ///
    /// This will not have any effect on a command that has already started running.
    fn add_arg<T: AsRef<OsStr>>(&mut self, arg: T) {
        self.cmd.arg(arg);
    }

    /// Starts the subprocess returning a [`CommandOutput`] instance which contains the readers for
    /// the subprocess' standard output and standard error streams.
    ///
    /// This will not block. Call [`RunCommand::wait`] to wait for the command to complete.
    fn run(&mut self) -> Result<CommandOutput> {
        if self.child.is_some() {
            return Err(Error::CommandAlreadyRunning);
        }

        // The stdout and stderr streams must be configured to be piped prior to spawning the
        // process.
        self.cmd.stderr(Stdio::piped());
        self.cmd.stdout(Stdio::piped());

        let mut child = self.cmd.spawn().map_err(Error::CommandStartIoError)?;

        // Must take the output & error streams to prevent them from being closed when wait is
        // eventually called. Should be safe to unwrap since the streams were configured to be
        // piped above.
        let streams = CommandOutput {
            out: Box::new(child.stdout.take().unwrap()),
            err: Box::new(child.stderr.take().unwrap()),
        };

        self.child = Some(child);

        Ok(streams)
    }

    /// Wait for the command to complete returning its exit status.
    fn wait(&mut self) -> Result<ExitStatus> {
        match self.child.as_mut() {
            Some(child) => child.wait().map_err(Error::CommandWaitIoError),
            None => Err(Error::CommandWaitInvalidStateError),
        }
    }

    /// Forces the command to exit.
    ///
    /// This will immediately call wait after successfully signalling the subprocess to stop to
    /// ensure the OS resources are released correctly. See the Rust documentation for `Child` for
    /// additional information.
    fn kill(&mut self) -> Result<()> {
        match self.child.as_mut() {
            Some(child) => {
                child.kill().map_err(Error::CommandKillIoError)?;
                child.wait().map_err(Error::CommandWaitIoError)?;
                Ok(())
            },
            None => Err(Error::CommandKillInvalidStateError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::mem;

    use crate::data::Attribute;

    #[test]
    fn process_output_line_updates_disc_info() {
        let mut ctx = Context::new("", Path::new(""));

        let msg = "CINFO:2,0,\"DISC_NAME\"";
        process_output_line(&mut ctx, msg).expect("Expected processing to be successful");


        let msg = "TINFO:0,2,0,\"TITLE_NAME\"";
        process_output_line(&mut ctx, msg).expect("Expected processing to be successful");

        let msg = "SINFO:0,0,2,0,\"STREAM_NAME\"";
        process_output_line(&mut ctx, msg).expect("Expected processing to be successful");

        let disc_info = ctx.disc_info.expect("Expected context to have a value for disc info");

        let disc_name = disc_info.attributes.get(&Attribute::Name);
        assert_eq!(disc_name, Some(&"DISC_NAME".to_owned()));

        let title_info = disc_info.titles[0].as_ref().unwrap();
        let title_name = title_info.attributes.get(&Attribute::Name);
        assert_eq!(title_name, Some(&"TITLE_NAME".to_owned()));

        let stream_info = title_info.streams[0].as_ref().unwrap();
        let stream_name = stream_info.attributes.get(&Attribute::Name);
        assert_eq!(stream_name, Some(&"STREAM_NAME".to_owned()));
    }

    #[test]
    fn process_output_line_invalid_message() {
        let mut ctx = Context::new("", Path::new(""));

        let msg = "TCOUNT:INVALID";
        process_output_line(&mut ctx, msg).expect_err("Expected processing to fail");
    }

    #[test]
    #[ignore]
    fn process_error_line_placeholder() {
        // TODO: Don't have enough of the code implemented to test.
        todo!()
    }

    #[test]
    #[ignore]
    fn run_command_run_error() {
        // TODO: Test cmd.run() error is handled!
        todo!()
    }

    #[test]
    #[ignore]
    fn run_command_wait_error() {
        // TODO: Test cmd.wait() error is handled!
        todo!()
    }

    #[test]
    #[ignore]
    fn run_command_kill_error() {
        // TODO: Test cmd.kill() error is handled!
        todo!()
    }

    #[test]
    #[ignore]
    fn run_command_process_output_error() {
        // TODO: Test that and error from output processing is handled.
        todo!()
    }

    #[test]
    #[ignore]
    fn run_command_process_error_error() {
        // TODO: Test that and error from error processing is handled.
        todo!()
    }

    #[test]
    #[ignore]
    fn run_command() {
        todo!()
    }
}
