use std::fmt;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Printer {
    stderr: StandardStream,
}

impl Printer {
    pub fn new() -> Printer {
        Printer {
            stderr: StandardStream::stderr(ColorChoice::Auto),
        }
    }

    pub fn print(
        &mut self,
        status: &dyn fmt::Display,
        message: Option<&dyn fmt::Display>,
        color: Color,
    ) -> anyhow::Result<()> {
        self.stderr.reset()?;
        self.stderr
            .set_color(ColorSpec::new().set_bold(true).set_fg(Some(color)))?;
        // write status
        write!(self.stderr, "{:>12}", status)?;
        // write message
        self.stderr.reset()?;
        match message {
            Some(message) => writeln!(self.stderr, " {}", message)?,
            None => write!(self.stderr, " ")?,
        }
        Ok(())
    }

    pub fn status<T: fmt::Display, U: fmt::Display>(
        &mut self,
        status: T,
        message: U,
    ) -> anyhow::Result<()> {
        self.print(&status, Some(&message), Color::Green)
    }

    pub fn error<T: fmt::Display>(&mut self, message: T) -> anyhow::Result<()> {
        self.print(&"Error", Some(&message), Color::Red)
    }

    pub fn warning<T: fmt::Display>(&mut self, message: T) -> anyhow::Result<()> {
        self.print(&"Warning", Some(&message), Color::Red)
    }

    pub fn result<T: fmt::Display>(&mut self, message: T) -> anyhow::Result<()> {
        self.print(&"Result", Some(&message), Color::White)
    }
}
