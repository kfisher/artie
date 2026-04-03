// Copyright 2025-2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! MakeMKV commands.
//!
//! This module contains the functions for running the various MakeMKV commands and processing
//! their output. The primary commands are the "info" command which can be executed with the
//! [`run_info_command`] function and "mkv" which can be run with the [`run_mkv_command`] function.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout, Command};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;

use tokio_util::future::FutureExt;
use tokio_util::sync::CancellationToken;

use crate::data::DiscInfo;
use crate::messages::{self, Message};
use crate::{Error, Progress, Result};

/// Represents the data sent through the channel used to relay output from a running command for
/// processing.
pub enum ChannelData {
    /// Line of text received from the output stream.
    OutTxt(String),

    /// Line of text received from the error output stream.
    ErrTxt(String),
}

/// Output that can be generated while a MakeMKV command is running.
pub enum CommandOutput {
    /// General information message.
    Message(String),

    /// Information about the current progress of the MakeMKV command.
    Progress(Progress),

    /// Error output from MakeMKV.
    Error(String),
}

/// Context object for running MakeMKV commands.
pub struct Context
{
    /// The device path to the target optical drive.
    device: String,

    /// Specifies the channel to send output from the command while the command is running such as
    /// progress updates and general information messages.
    observer: UnboundedSender<CommandOutput>,

    /// The progress of the currently running command.
    progress: Progress,

    /// Information about the disc in the drive, if available.
    ///
    /// This will only contain a value if a command is run that generates the required information
    /// messages which is currently only the `info` command.
    disc_info: Option<DiscInfo>,

    /// When set, the raw output from MakeMKV will be added to the file as the command is run.
    command_log: Option<LogFile>,

    /// Cancellation token used to cancel the running command.
    ct: CancellationToken,
}

impl Context
{
    /// Constructs a new context for the optical drive specified by the provided device path and
    /// output directory.
    pub fn new(
        device: &str,
        observer: &UnboundedSender<CommandOutput>,
        ct: CancellationToken
    ) -> Self {
        Context {
            device: device.to_owned(),
            disc_info: None,
            observer: observer.clone(),
            progress: Progress::new(),
            command_log: None,
            ct,
        }
    }

    /// Enable logging the raw output from the MakeMKV command to the provided filename.
    pub fn log_output(&mut self, path: &Path) -> Result<()> {
        let exists = match path.try_exists() {
            Ok(exists) => exists,
            Err(error) => return Err(Error::LogFileExists {
                path: path.to_path_buf(),
                error: Some(error)
            }),
        };

        if exists {
            return Err(Error::LogFileExists {
                path: path.to_path_buf(),
                error: None })
        }

        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .map_err(|e| Error::LogFileOpenError {
                path: path.to_path_buf(),
                error: e
            })?;

        self.command_log = Some(LogFile { path: path.to_path_buf(), file });

        Ok(())
    }

    /// Takes the [`DiscInfo`] object from the context leaving `None` in its place.
    pub fn take_disc_info(&mut self) -> Option<DiscInfo> {
        self.disc_info.take()
    }

    /// Updates the progress title for the current operation.
    fn set_op_title(&mut self, title: &str) -> Result<()> {
        self.progress.op = title.to_owned();
        self.observer.send(CommandOutput::Progress(self.progress.clone()))
            .map_err(|e| Error::ObserverSendError { error: e })
    }

    /// Updates the progress title for the current suboperation.
    fn set_subop_title(&mut self, title: &str) -> Result<()> {
        self.progress.subop = title.to_owned();
        self.observer.send(CommandOutput::Progress(self.progress.clone()))
            .map_err(|e| Error::ObserverSendError { error: e })
    }

    /// Updates the progress values.
    fn set_progress(&mut self, op: i32, subop: i32, max: i32) -> Result<()> {
        self.progress.op_prog = (op * 100 / max) as u8;
        self.progress.subop_prog = (subop * 100 / max) as u8;
        self.observer.send(CommandOutput::Progress(self.progress.clone()))
            .map_err(|e| Error::ObserverSendError { error: e })
    }
}

/// Runs the "info" MakeMKV command.
///
/// The "info" command extracts information about the contents of a DVD or Blu-ray. This
/// information is written to the [`DiscInfo`] field in `ctx`.
pub async fn run_info_command(ctx: &mut Context) -> Result<ExitStatus>
{
    // TODO: Need to be able to specify the path to the executable. For now, assume it's in
    //       the PATH. Defining an environment variable should be simple enough and suffice.
    let mut cmd = Command::new("makemkvcon");
    cmd.arg("--cache=1");
    cmd.arg("--noscan");
    cmd.arg("--progress=-same");
    cmd.arg("info");
    cmd.arg(format!("dev:{0}", ctx.device));

    run_command(&mut cmd, ctx).await
}

/// Runs the "mkv" MakeMKV command.
///
/// The "mkv" command copies titles from a DVD or Blu-ray disc and saves them as MKV files.
pub async fn run_mkv_command(ctx: &mut Context, out_dir: &Path) -> Result<ExitStatus>
{
    // TODO: Need to be able to specify the path to the executable. For now, assume it's in
    //       the PATH. Defining an environment variable should be simple enough and suffice.
    let mut cmd = Command::new("makemkvcon");
    cmd.arg("--robot");
    cmd.arg("--noscan");
    cmd.arg("--progress=-same");
    cmd.arg("mkv");
    cmd.arg(format!("dev:{0}", ctx.device));
    cmd.arg("all");
    cmd.arg(out_dir);

    run_command(&mut cmd, ctx).await
}

/// `Path` and `File` object for the command log file.
struct LogFile {
    path: PathBuf,
    file: File,
}

impl LogFile {
    /// Returns a [`Error::LogStdOutError`] with the provided I/O error.
    fn stdout_error(&self, e: std::io::Error) -> Error {
        Error::LogStdOutError { path: self.path.clone(), error: e }
    }

    /// Returns a [`Error::LogStdErrError`] with the provided I/O error.
    fn stderr_error(&self, e: std::io::Error) -> Error {
        Error::LogStdErrError { path: self.path.clone(), error: e }
    }
}

/// Task for processing lines of standard output from a running command.
///
/// This task simply reads from `stream` and sends the data on channel `tx`. It will run until the
/// stream is closed which will happen when the command exits or is cancelled.
async fn process_stdout(stream: ChildStdout, tx: UnboundedSender<ChannelData>) -> Result<()> {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await.map_err(|e| Error::OutTaskIoError { error: e })? {
        let data = ChannelData::OutTxt(line);
        tx.send(data).map_err(|e| Error::OutTaskSendError { error: e })?;
    }
    Ok(())
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
/// The general information message `MSG` and progress messages (`PRGC`, `PRGT`, `PTRV`) are
/// relayed to the initiator of the command via the observer within the provided context.
///
/// `DRV` and `TCOUNT` messages are ignored.
fn process_stdout_line(ctx: &mut Context, line: &str) -> Result<()>
{
    if let Some(log) = &mut ctx.command_log {
        writeln!(log.file, "STDOUT\t{}", line).map_err(|e| log.stdout_error(e))?;
    }

    use Message::*;
    match messages::parse_message(line)? {
        CINFO { id, code: _, value } => ctx
            .disc_info
            .get_or_insert_default()
            .add_attribute(id, &value)?,
        TINFO {
            title_index,
            id,
            code: _,
            value,
        } => ctx.disc_info.get_or_insert_default().add_title_attribute(
            title_index as usize,
            id,
            &value,
        )?,
        SINFO {
            title_index,
            stream_index,
            id,
            code: _,
            value,
        } => ctx.disc_info.get_or_insert_default().add_stream_attribute(
            title_index as usize,
            stream_index as usize,
            id,
            &value,
        )?,
        MSG {
            code: _,
            flags: _,
            count: _,
            message,
            format: _,
            args: _,
        } => {
            ctx.observer.send(CommandOutput::Message(message.to_owned()))
                .map_err(|e| Error::ObserverSendError { error: e })?;
        },
        PRGT {
            code: _,
            id: _,
            name,
        } => ctx.set_op_title(&name)?,
        PRGC {
            code: _,
            id: _,
            name,
        } => ctx.set_subop_title(&name)?,
        PRGV {
            suboperation,
            operation,
            max,
        } => ctx.set_progress(operation, suboperation, max)?,
        _ => (),
    };

    Ok(())
}

/// Task for processing lines of standard error from a running command.
async fn process_stderr(stream: ChildStderr, tx: UnboundedSender<ChannelData>) -> Result<()> {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await.map_err(|e| Error::ErrTaskIoError { error: e })? {
        let data = ChannelData::ErrTxt(line);
        tx.send(data).map_err(|e| Error::ErrTaskSendError { error: e })?;
    }
    Ok(())
}

/// Processes a line of error text (standard error) from a running MakeMKV command.
///
/// For each line of output text, this will append the line to the logfile in the provided context
/// if specified. It will also used call the appropriate callback in the context to notify the
/// initiator of the command so they may respond accordingly (e.g. notify the user).
fn process_stderr_line(ctx: &mut Context, line: &str) -> Result<()>
{
    if let Some(log) = &mut ctx.command_log {
        writeln!(log.file, "STDERR\t{}", line).map_err(|e| log.stderr_error(e))?;
    }

    ctx.observer.send(CommandOutput::Error(line.to_owned()))
        .map_err(|e| Error::ObserverSendError { error: e })
}

/// Runs an MakeMKV command.
async fn run_command(cmd: &mut Command, ctx: &mut Context) -> Result<ExitStatus>
{
    // Pipe both STDOUT and STDERR from MakeMKV so it can be read in realtime. It must be done
    // prior to the process being spawned.
    cmd.stderr(Stdio::piped());
    cmd.stdout(Stdio::piped());

    // TODO: Not sure if this is needed or not.
    // cmd.kill_on_drop(true);

    let (tx, mut rx) = mpsc::unbounded_channel::<ChannelData>();

    let mut child = cmd
        .spawn()
        .map_err(|e| Error::CommandIoError { error: e })?;

    // Should be safe to unwrap since the streams were configured to be piped above.
    let out_stream = child.stdout.take().unwrap();
    let out_ct = ctx.ct.clone();
    let out_handle = tokio::spawn(process_stdout(out_stream, tx.clone()))
        .with_cancellation_token_owned(out_ct);

    let err_stream = child.stderr.take().unwrap();
    let err_ct = ctx.ct.clone();
    let err_handle = tokio::spawn(process_stderr(err_stream, tx.clone()))
        .with_cancellation_token_owned(err_ct);

    // Must drop the original sender to avoid blocking indefinitely. Once this is dropped, the
    // remaining senders will remain open for as long as their respective threads are active. The
    // threads will exit once command completes and closes the I/O streams.
    drop(tx);

    let mut process_error: Option<Error> = None;

    loop {
        tokio::select! {
            data = rx.recv() => {
                if let Some(data) = data {
                    let result = match data {
                        ChannelData::OutTxt(text) => process_stdout_line(ctx, &text),
                        ChannelData::ErrTxt(text) => process_stderr_line(ctx, &text),
                    };
                    if let Err(error) = result {
                        // Calling kill() will also wait for the command to exit to ensure that the
                        // system resources are released.
                        child.kill().await.map_err(|e| Error::CommandIoError { error: e })?;
                        process_error = Some(error);
                    }
                } else {
                    break;
                }
            }
            _ = ctx.ct.cancelled() => {
                child.kill().await.map_err(|e| Error::CommandIoError { error: e })?;
                break;
            }
        };
    }

    let exit_status = child.wait().await
        .map_err(|e| Error::CommandIoError { error: e })?;

    if let Some(result) = out_handle.await {
        let _ = result
            .map_err(|e| Error::OutTaskPanicked { error: format!("{:?}", e) })?;
    }

    if let Some(result) = err_handle.await {
        let _ = result
            .map_err(|e| Error::ErrTaskPanicked { error: format!("{:?}", e) })?;
    }

    match process_error {
        Some(error) => Err(error),
        None => Ok(exit_status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use crate::data::Attribute;
    use crate::test_utils::TempFile;

    #[test]
    fn process_output_line_updates_disc_info() {
        let (tx, _rx) = mpsc::unbounded_channel::<CommandOutput>();
        let ct = CancellationToken::new();
        let mut ctx = Context::new("/dev/null", &tx, ct);

        let msg = "CINFO:2,0,\"DISC_NAME\"";
        process_stdout_line(&mut ctx, msg).expect("Expected processing to be successful");

        let msg = "TINFO:0,2,0,\"TITLE_NAME\"";
        process_stdout_line(&mut ctx, msg).expect("Expected processing to be successful");

        let msg = "SINFO:0,0,2,0,\"STREAM_NAME\"";
        process_stdout_line(&mut ctx, msg).expect("Expected processing to be successful");

        let disc_info = ctx
            .disc_info
            .expect("Expected context to have a value for disc info");

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
    fn process_output_line_calls_callbacks() {
        let (tx, mut rx) = mpsc::unbounded_channel::<CommandOutput>();
        let ct = CancellationToken::new();
        let mut ctx = Context::new("/dev/null", &tx, ct);

        let msg = "PRGT:3400,7,\"Title\"";
        process_stdout_line(&mut ctx, msg).expect("Expected processing to be successful");
        let output = rx.try_recv().unwrap();
        assert_eq!(ctx.progress.op, "Title".to_owned());
        if let CommandOutput::Progress(progress) = output {
            assert_eq!(progress.op, "Title".to_owned());
        } else {
            panic!("Expected progress output");
        }

        let msg = "PRGC:3400,7,\"Subtitle\"";
        process_stdout_line(&mut ctx, msg).expect("Expected processing to be successful");
        let output = rx.try_recv().unwrap();
        assert_eq!(ctx.progress.subop, "Subtitle".to_owned());
        if let CommandOutput::Progress(progress) = output {
        assert_eq!(progress.subop, "Subtitle".to_owned());
        } else {
            panic!("Expected progress output");
        }

        let msg = "PRGV:32768,16384,65536";
        process_stdout_line(&mut ctx, msg).expect("Expected processing to be successful");
        let output = rx.try_recv().unwrap();
        assert_eq!(ctx.progress.op_prog, 25);
        assert_eq!(ctx.progress.subop_prog, 50);
        if let CommandOutput::Progress(progress) = output {
            assert_eq!(progress.op_prog, 25);
            assert_eq!(progress.subop_prog, 50);
        } else {
            panic!("Expected progress output");
        }

        let msg = "MSG:3007,0,0,\"Hello There!\",\"Hello There!\"";
        process_stdout_line(&mut ctx, msg).expect("Expected processing to be successful");
        let output = rx.try_recv().unwrap();
        if let CommandOutput::Message(message) = output {
            assert_eq!(message, "Hello There!".to_owned());
        } else {
            panic!("Expected a message.");
        }
    }

    #[test]
    fn process_output_line_appends_log() {
        let dir = std::env::temp_dir();
        let log = "artie.makemkv.test.process_output_line_appends_log";

        // This will delete the file when dropped.
        let temp_file = TempFile(dir.join(log));

        let (tx, _rx) = mpsc::unbounded_channel::<CommandOutput>();
        let ct = CancellationToken::new();
        let mut ctx = Context::new("/dev/null", &tx, ct);
        ctx.log_output(temp_file.path()).unwrap();

        let msg = "TCOUNT:42";
        process_stdout_line(&mut ctx, msg).expect("Expected processing to be successful");

        // Ensure the file is closed before proceding.
        drop(ctx);

        let content = fs::read_to_string(temp_file.path()).expect("");
        assert_eq!(content, "STDOUT\tTCOUNT:42\n".to_owned());
    }

    #[test]
    fn process_output_line_invalid_message() {
        let (tx, _rx) = mpsc::unbounded_channel::<CommandOutput>();
        let ct = CancellationToken::new();
        let mut ctx = Context::new("/dev/null", &tx, ct);

        let msg = "TCOUNT:INVALID";
        process_stdout_line(&mut ctx, msg).expect_err("Expected processing to fail");
    }

    #[test]
    fn process_error_line_calls_callbacks() {
        let (tx, mut rx) = mpsc::unbounded_channel::<CommandOutput>();
        let ct = CancellationToken::new();
        let mut ctx = Context::new("/dev/null", &tx, ct);

        let err = "Failed to read disc.";
        process_stderr_line(&mut ctx, err).expect("Expected processing to be successful");
        let output = rx.try_recv().unwrap();
        if let CommandOutput::Error(error) = output {
            assert_eq!(error, "Failed to read disc.".to_owned());
        } else {
            panic!("Expected a message.");
        }
    }

    #[test]
    fn process_error_line_appends_log() {
        let dir = std::env::temp_dir();
        let log = "artie.makemkv.test.process_error_line_appends_log";

        // This will delete the file when dropped.
        let temp_file = TempFile(dir.join(log));

        let (tx, _rx) = mpsc::unbounded_channel::<CommandOutput>();
        let ct = CancellationToken::new();
        let mut ctx = Context::new("/dev/null", &tx, ct);
        ctx.log_output(temp_file.path()).unwrap();

        let err = "Failed to read disc.";
        process_stderr_line(&mut ctx, err).expect("Expected processing to be successful");

        // Ensure the file is closed before proceding.
        drop(ctx);

        let content = fs::read_to_string(temp_file.path()).expect("");
        assert_eq!(content, "STDERR\tFailed to read disc.\n".to_owned());
    }
}

