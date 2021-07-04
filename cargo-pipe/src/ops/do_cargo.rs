use crate::print::Printer;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

pub(crate) const CARGO_MANIFEST_FILE: &str = "Cargo.toml";

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
    printer.status(&"Cargo", "init")?;
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
    printer: &mut Printer,
) -> anyhow::Result<i32> {
    printer.status(&"Cargo", "check")?;
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("check").arg("--manifest-path").arg(manifest_path);
    if verbose {
        cmd.arg("--verbose");
    }
    if !warning {
        std::env::set_var("RUSTFLAGS", "-Awarnings");
    };
    let (err_code, out) = run_cmd(cmd)?;
    match err_code {
        0 => printer.status(&"Cargo", "check succeed")?,
        _ => printer.error(format!("{}", out))?,
    };
    Ok(err_code)
}
