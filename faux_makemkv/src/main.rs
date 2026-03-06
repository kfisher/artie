// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Simulates running MakeMKV.
//!
//! This simulates running MakeMKV by playing back output. It is mainly used for development
//! purposes to test without having to actually copy a disc.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;
use std::thread;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        if arg == "info" {
            simulate_info_command();
            return;
        }
        if arg == "mkv" {
            simulate_mkv_command();
            return;
        }
    }

    eprintln!("Error: No valid command provided. Use 'info' or 'mkv'.");
    process::exit(1);
}

fn simulate_info_command() {
    let path = match env::var("FAUX_MAKEMKV_INFO_PATH") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Error: FAUX_MAKEMKV_INFO_PATH environment variable must be set");
            process::exit(1);
        }
    };

    echo_file(&path);
}

fn simulate_mkv_command() {
    let path = match env::var("FAUX_MAKEMKV_MKV_PATH") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Error: FAUX_MAKEMKV_MKV_PATH environment variable must be set");
            process::exit(1);
        }
    };

    echo_file(&path);
}

fn get_delay() -> Duration {
    if let Ok(s) = env::var("FAUX_MAKEMKV_DELAY") {
        if let Ok(ms) = s.parse::<u64>() {
            return Duration::from_millis(ms);
        }
    }

    Duration::from_millis(0)
}

fn echo_file(path: &str) {
    let data = match fs::read_to_string(path) {
        Ok(d) => d,
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            process::exit(1);
        }
    };

    let delay = get_delay();
    let sleep_time = Duration::from_millis(10);
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for line in data.lines() {
        if line.trim().is_empty() {
            continue; // Skip empty lines
        }

        writeln!(handle, "{}", line).unwrap();
        handle.flush().unwrap();

        let mut time = Duration::default();
        while time < delay {
            thread::sleep(sleep_time);
            time += sleep_time;
        }
    }
}

