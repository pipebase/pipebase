use crate::print::Printer;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

pub(crate) const CARGO_MANIFEST_FILE: &str = "Cargo.toml";
pub(crate) const CARGO_SRC_DIRECTORY: &'static str = "src";
pub(crate) const CARGO_TARGET_DIRECTORY: &'static str = "target";
pub(crate) const CARGO_RELEASE_DIRECTORY: &'static str = "release";
pub(crate) const CARGO_DEBUG_DIRECTORY: &'static str = "debug";
pub(crate) const CARGO_APP_MAIN: &'static str = "main.rs";
pub(crate) const RUST_COMPILER_ERROR_TYPE_MISMATCH: &'static str = "E0271";
pub(crate) const RUST_COMPILER_ERROR_TRAIT_STRICTER_REQUIREMENTS: &'static str = "E0276";
pub(crate) const RUST_COMPILER_ERROR_TRAIT_NOT_IMPLEMENTED: &'static str = "E0277";

fn error_tag(error_index: &str) -> String {
    format!("error[{}]", error_index)
}

fn is_error_line(line: &str, error_index: &str) -> bool {
    let tag = error_tag(error_index);
    line.starts_with(&tag)
}

fn is_type_miss_match(line: &str) -> bool {
    is_error_line(line, RUST_COMPILER_ERROR_TYPE_MISMATCH)
}

fn is_trait_not_implemented(line: &str) -> bool {
    is_error_line(line, RUST_COMPILER_ERROR_TRAIT_NOT_IMPLEMENTED)
}

fn is_trait_stricter_requirement(line: &str) -> bool {
    is_error_line(line, RUST_COMPILER_ERROR_TRAIT_STRICTER_REQUIREMENTS)
}

fn print_error(out: String, printer: &mut Printer) -> anyhow::Result<()> {
    let lines: Vec<&str> = out.split("\n").collect();
    for line in lines {
        if is_type_miss_match(line) {
            printer.error(format!("{}", line))?;
            continue;
        }
        if is_trait_not_implemented(line) {
            printer.error(format!("{}", line))?;
            continue;
        }
        if is_trait_stricter_requirement(line) {
            printer.error(format!("{}", line))?;
            continue;
        }
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
                    print_error(out, printer)?
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
                    print_error(out, printer)?
                }
            }
        }
    };
    Ok(status_code)
}
