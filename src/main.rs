extern crate clap;

use clap::{App, Arg};
use std::error::Error;
use std::io::{Error as IoError, ErrorKind};
use std::ops::Deref;
use std::path::Path;
use std::process::{self, Command, Stdio};

fn main() {
    let args = parse_args();
    if let Some(run_type) = select_run_type(args.input) {
        let status = match run_type {
            RunType::SingleFile(file) => single_file_run(&file, &args.carried_args),
            RunType::CargoRun => cargo_run("", &args.carried_args),
            RunType::CargoMultiBin(bin) => cargo_run(&bin, &args.carried_args),
        };
        match status {
            Ok(success) => {
                if success {
                    process::exit(0);
                } else {
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("{}", e.description());
                process::exit(2);
            }
        }
    } else {
        eprintln!("nothing  to run")
    }
}

#[derive(Debug)]
struct Args {
    input: Option<String>,
    carried_args: Option<Vec<String>>,
}

fn parse_args() -> Args {
    let matches = App::new("Rust Runner")
        .bin_name("rrun")
        .version("0.1")
        .author("nextzhou <nextzhou@gmail.com>")
        .arg(
            Arg::with_name("input")
                .help("main source file, execute 'cargo run' if empty")
                .index(1),
        )
        .arg(
            Arg::with_name("args")
                .help("arguments of your program")
                .last(true)
                .multiple(true),
        )
        .get_matches();

    Args {
        input: matches.value_of("input").map(<&str>::into),
        carried_args: matches
            .values_of("args")
            .map(|args| args.into_iter().map(<&str>::into).collect()),
    }
}

fn run(command: &str, args: Option<&[&str]>) -> std::io::Result<bool> {
    let mut command = Command::new(command);
    if let Some(args) = args {
        command.args(args);
    }
    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    match command.spawn() {
        Ok(mut child) => child.wait().map(|status| status.success()),
        Err(e) => Err(e),
    }
}

enum RunType {
    SingleFile(String),
    CargoRun,
    CargoMultiBin(String),
}

fn select_run_type(input: Option<String>) -> Option<RunType> {
    let check_git_root = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if check_git_root.status.success() {
        return match input {
            Some(input) => Some(RunType::CargoMultiBin(input)),
            None => Some(RunType::CargoRun),
        };
    }
    let input = input?;
    if !input.ends_with(".rs") {
        return Some(RunType::SingleFile(input + ".rs"));
    }
    return Some(RunType::SingleFile(input));
}

fn cargo_run(bin: &str, args: &Option<Vec<String>>) -> Result<bool, IoError> {
    let mut cargo_args = vec!["run"];
    if !bin.is_empty() {
        cargo_args.push("--bin");
        cargo_args.push(bin);
    }
    if let Some(ref carried_args) = args {
        cargo_args.push("--");
        cargo_args.extend(carried_args.iter().map(String::deref));
    }
    run("cargo", Some(cargo_args.as_slice())).map_err(|e| {
        IoError::new(
            ErrorKind::Other,
            format!("execute 'cargo run' failed: {}", e.description()),
        )
    })
}

fn single_file_run(file: &str, args: &Option<Vec<String>>) -> Result<bool, IoError> {
    let path = Path::new(&file);
    if !path.is_file() {
        eprintln!("'{}' is not a file", file);
        process::exit(2);
    }
    let mut tmpdir = std::env::temp_dir();
    tmpdir.push(path.file_stem().unwrap());
    tmpdir.set_extension("rrun");
    let tmpfile = tmpdir.to_str().unwrap();
    run("rustc", Some(&["-o", tmpfile, &file]))
        .map_err(|e| {
            IoError::new(
                ErrorKind::Other,
                format!("execute 'rustc' failed: {}", e.description()),
            )
        })
        .and_then(|success| {
            if !success {
                Ok(false)
            } else {
                if let Some(ref carried_args) = args {
                    run(
                        tmpfile,
                        Some(
                            carried_args
                                .iter()
                                .map(String::deref)
                                .collect::<Vec<_>>()
                                .as_slice(),
                        ),
                    )
                } else {
                    run(tmpfile, None)
                }
                .map_err(|e| {
                    IoError::new(
                        ErrorKind::Other,
                        format!("execute '{}' failed: {}", tmpfile, e.description()),
                    )
                })
            }
        })
}
