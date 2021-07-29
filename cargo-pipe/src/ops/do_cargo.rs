use crate::print::Printer;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

pub(crate) const CARGO_MANIFEST_FILE: &str = "Cargo.toml";
pub(crate) const CARGO_SRC_DIRECTORY: &'static str = "src";
pub(crate) const CARGO_TARGET_DIRECTORY: &'static str = "target";
pub(crate) const CARGO_RELEASE_DIRECTORY: &'static str = "release";
pub(crate) const CARGO_DEBUG_DIRECTORY: &'static str = "debug";
pub(crate) const CARGO_APP_MAIN: &'static str = "main.rs";

fn capture_error_message(captures: Option<Captures>) -> Option<String> {
    let m = match captures {
        Some(captures) => captures.get(1),
        None => return None,
    };
    match m {
        Some(m) => Some(m.as_str().to_owned()),
        None => None,
    }
}

fn capture_error(line: &str) -> Option<String> {
    lazy_static! {
        static ref ERROR_CODE: Regex = Regex::new(r"error\[E\d{4}\]:\s*(.*)").unwrap();
        static ref ERROR: Regex = Regex::new(r"error:\s*(.*)").unwrap();
    }
    match capture_error_message(ERROR_CODE.captures(line)) {
        Some(error_message) => return Some(error_message),
        None => (),
    };
    match capture_error_message(ERROR.captures(line)) {
        Some(error_message) => return Some(error_message),
        None => (),
    };
    return None;
}

fn capture_errors(out: String, printer: &mut Printer) -> anyhow::Result<()> {
    let lines: Vec<&str> = out.split("\n").collect();
    for line in lines {
        match capture_error(line) {
            Some(error_message) => printer.error(format!("{}", error_message))?,
            None => (),
        };
    }
    Ok(())
}

fn run_cmd(mut cmd: Command) -> anyhow::Result<(i32, String)> {
    let output = cmd.output()?;
    match output.status.success() {
        true => {
            let stdout = String::from_utf8(output.stdout)?;
            Ok((0, stdout))
        }
        false => {
            let stderr = String::from_utf8(output.stderr)?;
            let err_code = match output.status.code() {
                Some(err_code) => err_code,
                None => 1,
            };
            Ok((err_code, stderr))
        }
    }
}

fn cargo_binary() -> OsString {
    match std::env::var_os("CARGO") {
        Some(cargo) => cargo,
        None => "cargo".to_owned().into(),
    }
}

pub fn do_cargo_init(path: &Path, printer: &mut Printer) -> anyhow::Result<i32> {
    printer.status(&"Cargo", "init ...")?;
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("init").arg(path);
    let (err_code, out) = run_cmd(cmd)?;
    match err_code {
        0 => printer.status(&"Cargo", "init succeed")?,
        _ => printer.status(&"Cargo", format!("init failed, stderr: {}", out))?,
    };
    Ok(err_code)
}

pub fn do_cargo_fmt(manifest_path: &Path, printer: &mut Printer) -> anyhow::Result<i32> {
    printer.status(&"Cargo", "fmt")?;
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("fmt").arg("--manifest-path").arg(manifest_path);
    let (err_code, out) = run_cmd(cmd)?;
    match err_code {
        0 => printer.status(&"Cargo", "fmt succeed")?,
        _ => printer.error(format!("fmt failed, stderr: {}", out))?,
    };
    Ok(err_code)
}

pub fn do_cargo_check(
    manifest_path: &Path,
    warning: bool,
    verbose: bool,
    debug: bool,
    printer: &mut Printer,
) -> anyhow::Result<i32> {
    printer.status(&"Cargo", "check ...")?;
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("check").arg("--manifest-path").arg(manifest_path);
    if verbose {
        cmd.arg("--verbose");
    }
    if !warning {
        std::env::set_var("RUSTFLAGS", "-Awarnings");
    };
    let (status_code, out) = run_cmd(cmd)?;
    match status_code {
        0 => printer.status(&"Cargo", "check succeed")?,
        _ => {
            printer.error("cargo check failed")?;
            if debug {
                if verbose {
                    printer.error(format!("{}", out))?
                } else {
                    capture_errors(out, printer)?
                }
            }
        }
    };
    Ok(status_code)
}

pub fn do_cargo_build(
    manifest_path: &Path,
    release: bool,
    debug: bool,
    verbose: bool,
    printer: &mut Printer,
) -> anyhow::Result<i32> {
    printer.status(&"Cargo", "build ...")?;
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("build").arg("--manifest-path").arg(manifest_path);
    if release {
        cmd.arg("--release");
    }
    let (status_code, out) = run_cmd(cmd)?;
    match status_code {
        0 => printer.status(&"Cargo", "build succeed")?,
        _ => {
            printer.error("cargo build failed")?;
            if debug {
                if verbose {
                    printer.error(format!("{}", out))?
                } else {
                    capture_errors(out, printer)?
                }
            }
        }
    };
    Ok(status_code)
}
