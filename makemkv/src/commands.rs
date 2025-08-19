// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//--] use std::io::{BufRead, BufReader};
//--] use std::process::{Command, Stdio};
//--] use std::thread;

//--] use super::{DiscInfo, Error};

//--] pub fn run_info_command(device: &str) -> Result<DiscInfo, Error> {
//--]     // TODO: makemkv should be configurable.
//--]     let child = Command::new("makemkv")
//--]         .arg("--robot")
//--]         .arg("--cache=1")
//--]         .arg("--noscan")
//--]         .arg("--progress=-same")
//--]         .arg("--info")
//--]         .arg(format!("dev:{0}", device))
//--]         .stdout(Stdio::piped())
//--]         .stderr(Stdio::piped())
//--]         .spawn();
//--]
//--]     let mut child = match child {
//--]         Ok(child) => child,
//--]         Err(error) => return Err(Error::CommandFailed(error)),
//--]     };
//--]
//--]     // NOTES:
//--]     // - Must take the output and error streams to prevent them from
//--]     //   automatically being closed once wait is called.
//--]     // - stdout and stderr are set when the command is created above, so it
//--]     //   should be safe to call unwrap here.
//--]     // - In order to process both standard output and the standard error streams
//--]     //   in real time, must process both in their own thread.
//--]
//--]     let stdout = child.stdout.take().unwrap();
//--]     let stderr = child.stderr.take().unwrap();
//--]
//--]     let stdout_thread = thread::spawn(move || {
//--]         let reader = BufReader::new(stdout);
//--]         for line in reader.lines() {
//--]             println!("[STDOUT] {}", line.unwrap());
//--]         }
//--]     });
//--]
//--]     let stderr_thread = thread::spawn(move || {
//--]         let reader = BufReader::new(stderr);
//--]         for line in reader.lines() {
//--]             eprintln!("[STDERR] {}", line.unwrap());
//--]         }
//--]     });
//--]
//--]     let _ = child.wait();
//--]     stdout_thread.join().unwrap();
//--]     stderr_thread.join().unwrap();
//--]
//--]     Ok(DiscInfo{})
//--] }
