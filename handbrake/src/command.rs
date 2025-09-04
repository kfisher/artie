// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handbrake command.

use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::mpsc;
use std::thread;

use crate::Observe;
use crate::error::{Error, Result};
use crate::output::{Output, Parser};

// TODO: There is some duplicity between this module and the command module from the makemkv crate.
//       May want to moved some of the shared items into a shared crate between the two. 

/// Specifies how to encode an audio track when transcoding a video.
#[allow(clippy::upper_case_acronyms)]
pub enum AudioEncodeMethod {
    /// Audio track will be passed thru as is without modification.
    Copy,

    /// Re-encode the track into Advanced Audio Coding (AAC).
    AAC,

    /// Re-encode the track into Dolby Surround (AC-3).
    AC3,

    /// Re-encode the track into Apple Lossless Audio Codec (ALAC) 16 bit.
    ALAC16,

    /// Re-encode the track into Apple Lossless Audio Codec (ALAC) 24 bit.
    ALAC24,

    /// Re-encode the track into Dolby Digital Plus (E-AC-3).
    EAC3,

    /// Re-encode the track into Free Lossless Audio Codec (FLAC) 16 bit.
    Flac16,

    /// Re-encode the track into Free Lossless Audio Codec (FLAC) 24 bit.
    Flac24,

    /// Re-encode the track into MP3.
    MP3,

    /// Re-encode the track into Opus.
    Opus,

    /// Re-encode the track into Vorbis.
    Vorbis,
}

impl AudioEncodeMethod {
    /// Convertes the audio encode method to the command-line argument value given to handbrake.
    fn to_arg(&self) -> &str {
        match self {
            AudioEncodeMethod::Copy => "copy",
            AudioEncodeMethod::AAC => "av_aac",
            AudioEncodeMethod::AC3 => "ac3",
            AudioEncodeMethod::ALAC16 => "alac16",
            AudioEncodeMethod::ALAC24 => "alac24",
            AudioEncodeMethod::EAC3 => "eac3",
            AudioEncodeMethod::Flac16 => "flac16",
            AudioEncodeMethod::Flac24 => "flac24",
            AudioEncodeMethod::MP3 => "mp3",
            AudioEncodeMethod::Opus => "opus",
            AudioEncodeMethod::Vorbis => "vorbis",
        }
    }
}

/// Represents the data sent through the channel used to relay output from a running command for
/// processing.
pub enum ChannelData {
    /// Line of text received from the output stream.
    OutTxt(String),

    /// Line of text received from the error output stream.
    ErrTxt(String),
}

/// Specifies the encoding parameters for an audio track.
pub struct AudioTrackOption {
    /// The audio track number from the original source.
    track_no: i32,

    /// The method to use for encoding.
    method: AudioEncodeMethod,

    /// The name of the audio track.
    name: String,
}

/// Context object for running the Handbrake command.
pub struct Context<'a> {
    /// Observer used to send the initiator of the command status updates while the command is
    /// running.
    observer: &'a mut dyn Observe,

    /// Parser used to parse output data.
    ///
    /// The data outputted by handbrake is formatted as JSON objects over multiple lines. So in
    /// order to output progress data as it comes in, an internal buffer must be kept until a
    /// complete JSON object is received.
    output_parser: Parser,

    /// When `Some`, the raw output from the Handbrake command will be added to the contained file
    /// as the command is run.
    command_log: Option<LogFile>,
}

impl<'a> Context<'a> {
    /// Create a new `Context` instance.
    pub fn new<T>(observer: &'a mut T) -> Context<'a>
    where
        T: Observe,
    {
        Context {
            observer,
            output_parser: Parser::new(),
            command_log: None,
        }
    }

    /// Enable logging the raw output from the Handbrake command to the provided file.
    ///
    /// Each line of output will be prefixed with either `STDOUT` or `STDERR` and a tab character.
    /// It should be noted that Handbrake uses the standard error console to log normal output
    /// while the standard output is used for progress updates.
    /// 
    /// # Errors
    ///
    /// In addition to the typical I/O errors that can be raised with file operations, it will
    /// also fail if the file exists and it will fail if the command has already started running.
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
}

/// Builder for configuring the arguments for running a handbrake command.
pub struct Options {
    /// The Handbrake preset to use when transcoding.
    ///
    /// This will configure the initial values for the transcode parameters. Any additional
    /// configuration done thru this builder will overwrite those values.
    preset: String,

    /// Path to the input video file.
    src_path: PathBuf,

    /// Path to where to save the transcoded video.
    dst_path: PathBuf,

    /// When set, specifies the offset in seconds to start transcoding.
    start_at: Option<i32>,

    /// When set, specifies the duration of the transcode.
    stop_at: Option<i32>,

    /// List of audio tracks to include in the transcoded video.
    audio: Vec<AudioTrackOption>,
}

impl Options {
    /// Constructs the options with the minimally required arguments.
    ///
    /// The `preset` is the Handbrake preset settings to use. Any additional options added via this
    /// builder will modify the corresponding settings from the preset. `src_path` and `dst_path`
    /// are the paths to the input file to transcode and where to save the transcoded video
    /// respectively.
    pub fn new(preset: &str, src_path: &Path, dst_path: &Path) -> Options {
        Options {
            preset: preset.to_owned(),
            src_path: src_path.to_path_buf(),
            dst_path: dst_path.to_path_buf(),
            start_at: None,
            stop_at: None,
            audio: Vec::new(),
        }
    }

    /// Adds an audio track.
    pub fn audio_track(&mut self, no: i32, method: AudioEncodeMethod, name: &str) -> &mut Options {
        self.audio.push(AudioTrackOption { track_no: no, method, name: name.to_owned() });
        self
    }

    /// Sets the offset in seconds from the start of the input video to start transcoding.
    pub fn start_at(&mut self, seconds: i32) -> &mut Options {
        self.start_at = Some(seconds);
        self
    }

    /// Sets the duration of the transcode in seconds.
    ///
    /// This will be relative to the `start_at` seconds if it was also set. So if the `start_at`
    /// seconds is 30 seconds and `stop_at` is 60 seconds, then the transcoded video will be a
    /// segment of the source video from the 30 second mark to the 90 second mark.
    pub fn stop_at(&mut self, seconds: i32) -> &mut Options {
        self.stop_at = Some(seconds);
        self
    }

    /// Returns a list of command-line arguments based on the configured options.
    fn get_options(&self) -> Result<Vec<String>> {
        if !self.src_path.is_file() {
            return Err(Error::InvalidOption { 
                option: String::from("src_path"), 
                error: String::from("input file does not exist"),
            });
        }

        let Some(src_path) = self.src_path.to_str() else {
            return Err(Error::InvalidOption { 
                option: String::from("src_path"), 
                error: String::from("failed to convert input path"),
            });
        };

        if self.dst_path.exists() {
            return Err(Error::InvalidOption { 
                option: String::from("dst_path"), 
                error: String::from("output file already exists"),
            });
        }

        let Some(dst_path) = self.dst_path.to_str() else {
            return Err(Error::InvalidOption { 
                option: String::from("dst_path"), 
                error: String::from("failed to convert output path"),
            });
        };

        let mut args = vec![
            String::from("--preset"),
            self.preset.to_owned(),
            String::from("--input"),
            src_path.to_owned(),
            String::from("--output"),
            dst_path.to_owned(),
        ];

        if let Some(start_at) = self.start_at {
            if start_at >= 0 {
                args.push(String::from("--start-at"));
                args.push(format!("seconds:{}", start_at));
            } else {
                return Err(Error::InvalidOption { 
                    option: String::from("start_at"), 
                    error: String::from("start_at less than zero"),
                });
            }
        }

        if let Some(stop_at) = self.stop_at {
            if stop_at > 0 {
                args.push(String::from("--stop-at"));
                args.push(format!("seconds:{}", stop_at));
            } else {
                return Err(Error::InvalidOption { 
                    option: String::from("stop_at"), 
                    error: String::from("stop_at less than or equal to zero"),
                });
            }
        }

        if self.audio.is_empty() {
            args.push(String::from("--audio"));
            args.push(String::from("none"));
        } else {
            let mut tracks = String::new();
            let mut methods = String::new();
            let mut names = String::new();

            let mut iter = self.audio.iter();

            // Should be safe to unwrap here as there should be at least one item if we're in this
            // else block.
            let first = iter.next().unwrap();
            tracks.push_str(&first.track_no.to_string());
            methods.push_str(first.method.to_arg());
            names.push_str(&first.name);

            for v in iter {
                tracks.push(',');
                tracks.push_str(&v.track_no.to_string());

                methods.push(',');
                methods.push_str(v.method.to_arg());

                names.push(',');
                names.push_str(&v.name);
            }

            args.push(String::from("--audio"));
            args.push(tracks);

            args.push(String::from("--aencoder"));
            args.push(methods);

            args.push(String::from("--aname"));
            args.push(names);
        }

        Ok(args)
    }
}

/// Runs HandBrake.
pub fn run_handbrake(ctx: &mut Context, opts: &Options) -> Result<ExitStatus>
{
    let mut cmd = OsRunner::new();
    run_command(ctx, &mut cmd, opts)
}

/// Trait for running HandBrake.
///
/// The expected use of this is to construct using `new`, add arguments using `add_arg`, and then
/// running with `run`. Once the command is running, `wait` can be used to wait for the command to
/// complete or `kill` to forcibly stop the command.
///
/// While the command is running, the output and error output can be processed by using the streams
/// returned by `run`. Note that if both output and error output need to be processed, they will
/// need to be handled in separate threads.
trait RunCommand {
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
struct CommandOutput {
    /// The output stream (e.g. `stdout`).
    out: Box<dyn Read + Send>,

    /// The error output stream (e.g. `stderr`).
    err: Box<dyn Read + Send>,
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

/// Command runner which makes system calls to run MakeMKV commands.
///
/// This is the default runner used to run commands. Other types of runners exist mainly for
/// testing and development when you don't want to actually copy a disc.
struct OsRunner {
    cmd: Command,
    child: Option<Child>,
}

impl RunCommand for OsRunner {
    /// Constructs a runner instance.
    fn new() -> Self {
        // TODO: Need to be able to specify the path to the executable. For now, assume it's in
        //       the PATH. Defining an environment variable should be simple enough and suffice.
        OsRunner {
            cmd: Command::new("handbrake"),
            child: None,
        }
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

        let mut child = self.cmd.spawn().map_err(|e| Error::CommandIoError { error: e })?;

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
            Some(child) => child.wait().map_err(|e| Error::CommandIoError { error: e }),
            None => Err(Error::CommandNotStarted),
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
                child.kill().map_err(|e| Error::CommandIoError { error: e })?;
                child.wait().map_err(|e| Error::CommandIoError { error: e })?;
                Ok(())
            }
            None => Err(Error::CommandNotStarted),
        }
    }
}

/// Process a line of output from the standard output console.
fn process_stdout_line(ctx: &mut Context, line: &str) -> Result<()> {
    if let Some(log) = &mut ctx.command_log {
        writeln!(log.file, "STDOUT\t{}", line).map_err(|e| log.stdout_error(e))?;
    }

    match ctx.output_parser.parse(line)? {
        Output::None => Ok(()),
        Output::Progress(progress) => {
            ctx.observer.progress(progress);
            Ok(())
        },
        Output::Version(version) => {
            ctx.observer.version(version);
            Ok(())
        },
    }
}

/// Process a line of error from the standard error console.
fn process_stderr_line(ctx: &mut Context, line: &str) -> Result<()> {
    if let Some(log) = &mut ctx.command_log {
        writeln!(log.file, "STDERR\t{}", line).map_err(|e| log.stderr_error(e))?;
    }

    // For some reason, handbrake uses the error console for general information messages.
    ctx.observer.message(line);

    Ok(())
}

/// Runs HandBrake with the provided runner.
fn run_command<T>(ctx: &mut Context, cmd: &mut T, opts: &Options) -> Result<ExitStatus>
where
    T: RunCommand,
{
    cmd.add_arg("--json");
    for arg in opts.get_options()? {
        cmd.add_arg(arg);
    }

    let streams = cmd.run()?;

    let (tx, rx) = mpsc::channel::<ChannelData>();

    let out_tx = tx.clone();
    let out_thread = thread::spawn(move || -> Result<()> {
        let reader = BufReader::new(streams.out);
        for line in reader.lines() {
            let line = line.map_err(|e| Error::OutThreadIoError { error: e })?;
            out_tx.send(ChannelData::OutTxt(line)).map_err(|e| Error::OutThreadSendError {
                error: e 
            })?;
        }
        Ok(())
    });

    let err_tx = tx.clone();
    let err_thread = thread::spawn(move || -> Result<()> {
        let reader = BufReader::new(streams.err);
        for line in reader.lines() {
            let line = line.map_err(|e| Error::ErrThreadIoError { error: e })?;
            err_tx.send(ChannelData::ErrTxt(line)).map_err(|e| Error::ErrThreadSendError {
                error: e 
            })?;
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
            ChannelData::OutTxt(text) => process_stdout_line(ctx, &text),
            ChannelData::ErrTxt(text) => process_stderr_line(ctx, &text),
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
                .map_err(|e| Error::OutThreadPanicked { error: format!("{:?}", e) })?;
            let _ = err_thread
                .join()
                .map_err(|e| Error::ErrThreadPanicked { error: format!("{:?}", e) })?;

            return Err(error);
        }
    }

    let exit_code = cmd.wait()?;

    let _ = out_thread
        .join()
        .map_err(|e| Error::OutThreadPanicked { error: format!("{:?}", e) })?;
    let _ = err_thread
        .join()
        .map_err(|e| Error::ErrThreadPanicked { error: format!("{:?}", e) })?;

    Ok(exit_code)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::io::Cursor;
    use std::ffi::OsStr;
    use std::fs;
    use std::mem;
    use std::path::{Path, PathBuf};
    use std::thread;

    use crate::{Progress, Version};
    use crate::output::parser::ParserState;

    pub struct TempFile(pub PathBuf);

    impl TempFile {
        fn new(file_name: &Path) -> TempFile {
            TempFile(env::temp_dir().join(file_name))
        }

        fn path(&self) -> &Path {
            let TempFile(ref p) = *self;
            p
        }

        fn create(&self) {
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(self.path())
                .expect("Failed to create file");
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let TempFile(ref p) = *self;
            if !p.exists() {
                return
            }
            let result = fs::remove_file(p);
            // Avoid panicking while panicking as this causes the process to immediately abort,
            // without displaying test results.
            if !thread::panicking() {
                result.unwrap();
            }
        }
    }

    struct TestObserver {
        messages: Vec<String>,
        progress: Vec<Progress>,
        version: Option<Version>,
    }

    impl TestObserver {
        fn new() -> TestObserver {
            TestObserver { 
                messages: Vec::new(),
                progress: Vec::new(),
                version: None,
            }
        }
    }

    impl Observe for TestObserver {
        fn message(&mut self, msg: &str) {
            self.messages.push(msg.to_owned());
        }

        fn progress(&mut self, progress: Progress) {
            self.progress.push(progress);
        }

        fn version(&mut self, version: Version) {
            match self.version {
                Some(_) => panic!("Received more than one version"),
                None => self.version = Some(version),
            }
        }
    }

    struct TestRunner {
        stdout: Cursor<Vec<u8>>,
        stderr: Cursor<Vec<u8>>,
        waited: bool,
        killed: bool,
        args: Vec<String>,
    }

    impl TestRunner {
        fn new() -> TestRunner {
            TestRunner {
                stdout: Cursor::default(),
                stderr: Cursor::default(),
                waited: false,
                killed: false,
                args: Vec::new(),
            }
        }

        fn set_stdout(&mut self, lines: &[&str]) {
            let data = lines.join("\n");
            self.stdout = Cursor::new(data.into_bytes());
        }

        fn set_stderr(&mut self, lines: &[&str]) {
            let data = lines.join("\n");
            self.stderr = Cursor::new(data.into_bytes());
        }
    }

    impl RunCommand for TestRunner {
        fn new() -> Self {
            TestRunner::new()
        }

        fn add_arg<T: AsRef<OsStr>>(&mut self, arg: T) {
            let arg = arg.as_ref().to_str().unwrap();
            self.args.push(arg.to_owned());
        }

        fn run(&mut self) -> Result<CommandOutput> {
            Ok(CommandOutput {
                out: Box::new(mem::take(&mut self.stdout)),
                err: Box::new(mem::take(&mut self.stderr)),
            })
        }

        fn wait(&mut self) -> Result<ExitStatus> {
            self.waited = true;
            Ok(ExitStatus::default())
        }

        fn kill(&mut self) -> Result<()> {
            self.killed = true;
            Ok(())
        }
    }

    #[test]
    fn options_builder() {
        let src_file = TempFile::new(Path::new("artie.handbrake.test.options_builder.mkv"));
        src_file.create();

        let dst_file = TempFile::new(Path::new("artie.handbrake.test.options_builder.mp4"));

        let opts = Options::new("Fast 1080p30", src_file.path(), dst_file.path());

        let args = opts.get_options().unwrap();
        assert_eq!(&args[0], "--preset");
        assert_eq!(&args[1], "Fast 1080p30");
        assert_eq!(&args[2], "--input");
        assert_eq!(&args[3], src_file.path().to_str().unwrap());
        assert_eq!(&args[4], "--output");
        assert_eq!(&args[5], dst_file.path().to_str().unwrap());
        assert_eq!(&args[6], "--audio");
        assert_eq!(&args[7], "none");

        let mut opts = Options::new("Fast 1080p30", src_file.path(), dst_file.path());
        opts.start_at(10);
        opts.stop_at(20);

        let args = opts.get_options().unwrap();
        assert_eq!(&args[0], "--preset");
        assert_eq!(&args[1], "Fast 1080p30");
        assert_eq!(&args[2], "--input");
        assert_eq!(&args[3], src_file.path().to_str().unwrap());
        assert_eq!(&args[4], "--output");
        assert_eq!(&args[5], dst_file.path().to_str().unwrap());
        assert_eq!(&args[6], "--start-at");
        assert_eq!(&args[7], "seconds:10");
        assert_eq!(&args[8], "--stop-at");
        assert_eq!(&args[9], "seconds:20");
        assert_eq!(&args[10], "--audio");
        assert_eq!(&args[11], "none");

        let mut opts = Options::new("Fast 1080p30", src_file.path(), dst_file.path());
        opts.audio_track(0, AudioEncodeMethod::Copy, "Surround Sound");
        opts.audio_track(1, AudioEncodeMethod::AAC, "Audio Commentary");

        let args = opts.get_options().unwrap();
        assert_eq!(&args[0], "--preset");
        assert_eq!(&args[1], "Fast 1080p30");
        assert_eq!(&args[2], "--input");
        assert_eq!(&args[3], src_file.path().to_str().unwrap());
        assert_eq!(&args[4], "--output");
        assert_eq!(&args[5], dst_file.path().to_str().unwrap());
        assert_eq!(&args[6], "--audio");
        assert_eq!(&args[7], "0,1");
        assert_eq!(&args[8], "--aencoder");
        assert_eq!(&args[9], "copy,av_aac");
        assert_eq!(&args[10], "--aname");
        assert_eq!(&args[11], "Surround Sound,Audio Commentary");

        let mut opts = Options::new("Fast 1080p30", src_file.path(), dst_file.path());
        opts.start_at(-1);
        opts.get_options().expect_err("Expected an error");

        let mut opts = Options::new("Fast 1080p30", src_file.path(), dst_file.path());
        opts.stop_at(0);
        opts.get_options().expect_err("Expected an error");

        dst_file.create();
        let opts = Options::new("Fast 1080p30", src_file.path(), dst_file.path());
        opts.get_options().expect_err("Expected an error");

        let opts = Options::new("Fast 1080p30", src_file.path(), dst_file.path());
        drop(src_file);
        drop(dst_file);
        opts.get_options().expect_err("Expected an error");
    }


    #[test]
    fn process_stdout_line() {
        let log_file = TempFile::new(Path::new("artie.handbrake.test.process_stdout_line"));
        let mut obs = TestObserver::new();
        let mut ctx = Context::new(&mut obs);
        ctx.log_output(log_file.path()).unwrap();

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
Progress: {
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
Progress: {
    "Muxing": {
        "Progress": 0.0
    },
    "State": "MUXING"
}
Progress: {
    "State": "WORKDONE",
    "WorkDone": {
        "Error": 0,
        "SequenceID": 1
    }
}
"#;

        for line in text.lines() {
            super::process_stdout_line(&mut ctx, &line).unwrap();
        }

        // This will cause the opened file to be closed.
        drop(ctx);

        // Verifies that the output was logged.
        let content = fs::read_to_string(log_file.path()).unwrap();
        let expected = r#"STDOUT	Version: {
STDOUT	    "Arch": "x86_64",
STDOUT	    "Name": "HandBrake",
STDOUT	    "Official": true,
STDOUT	    "RepoDate": "2024-08-07 17:31:52",
STDOUT	    "RepoHash": "77f199ab02ff2e3bca4ca653e922e9fef67dec43",
STDOUT	    "System": "MinGW",
STDOUT	    "Type": "release",
STDOUT	    "Version": {
STDOUT	        "Major": 1,
STDOUT	        "Minor": 8,
STDOUT	        "Point": 2
STDOUT	    },
STDOUT	    "VersionString": "1.8.2"
STDOUT	}
STDOUT	Progress: {
STDOUT	    "State": "WORKING",
STDOUT	    "Working": {
STDOUT	        "ETASeconds": 1,
STDOUT	        "Hours": 0,
STDOUT	        "Minutes": 0,
STDOUT	        "Pass": 1,
STDOUT	        "PassCount": 2,
STDOUT	        "PassID": -1,
STDOUT	        "Paused": 0,
STDOUT	        "Progress": 0.094762548804283142,
STDOUT	        "Rate": 0.0,
STDOUT	        "RateAvg": 0.0,
STDOUT	        "Seconds": 1,
STDOUT	        "SequenceID": 1
STDOUT	    }
STDOUT	}
STDOUT	Progress: {
STDOUT	    "Muxing": {
STDOUT	        "Progress": 0.0
STDOUT	    },
STDOUT	    "State": "MUXING"
STDOUT	}
STDOUT	Progress: {
STDOUT	    "State": "WORKDONE",
STDOUT	    "WorkDone": {
STDOUT	        "Error": 0,
STDOUT	        "SequenceID": 1
STDOUT	    }
STDOUT	}
"#;
        assert_eq!(content, expected);

        let progress = &obs.progress[0];
        assert_eq!(progress.pass, 1);
        assert_eq!(progress.pass_count, 2);
        assert_eq!(progress.progress, 9);

        let version = &obs.version.unwrap();
        assert_eq!(version.arch, "x86_64");
        assert_eq!(version.system, "MinGW");
        assert_eq!(version.version, "1.8.2");
    }

    #[test]
    fn process_stderr_line() {
        let log_file = TempFile::new(Path::new("artie.handbrake.test.process_stderr_line"));
        let mut obs = TestObserver::new();
        let mut ctx = Context::new(&mut obs);
        ctx.log_output(log_file.path()).unwrap();

        let line = "Encode done!";
        super::process_stderr_line(&mut ctx, &line).unwrap();

        // This will cause the opened file to be closed.
        drop(ctx);

        // Verifies that the message was relayed to the observer.
        assert_eq!(obs.messages[0], line);

        // Verifies that the output was logged.
        let content = fs::read_to_string(log_file.path()).unwrap();
        let expected = format!("STDERR\t{}\n", line);
        assert_eq!(content, expected);
    }

    #[test]
    fn run_command() {
        let mut obs = TestObserver::new();

        let mut ctx = Context::new(&mut obs);

        let mut cmd = TestRunner::new();
        cmd.set_stdout(&["Version: {"]);
        cmd.set_stderr(&["HandBrake has exited."]);

        let src_file = TempFile::new(Path::new("artie.handbrake.test.run_command.mkv"));
        src_file.create();

        let dst_file = TempFile::new(Path::new("artie.handbrake.test.run_command.mp4"));

        let opts = Options::new("Fast 1080p30", src_file.path(), dst_file.path());

        super::run_command(&mut ctx, &mut cmd, &opts).unwrap();

        assert_eq!(cmd.args[0], "--json");
        assert_eq!(cmd.args[1], "--preset");
        assert_eq!(cmd.args[2], "Fast 1080p30");
        assert_eq!(cmd.args[3], "--input");
        assert_eq!(cmd.args[4], src_file.path().to_str().unwrap());
        assert_eq!(cmd.args[5], "--output");
        assert_eq!(cmd.args[6], dst_file.path().to_str().unwrap());
        assert_eq!(cmd.args[7], "--audio");
        assert_eq!(cmd.args[8], "none");

        // Just need to confirm the process functions were called with the expected values. These
        // functions are tested in greater detail in another test.
        match ctx.output_parser.state {
            ParserState::ReadingVersion => (),
            _ => panic!("Expected {:?}, Got: {:?}", ParserState::Waiting, ctx.output_parser.state),
        }
        assert_eq!(obs.messages[0], "HandBrake has exited.");

        assert_eq!(cmd.waited, true);
        assert_eq!(cmd.killed, false);
    }
}
