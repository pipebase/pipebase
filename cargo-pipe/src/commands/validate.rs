use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use crate::ops::do_validate::do_exec;
use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("validate")
        .about("Validate pipes and objects in pipe manifest")
        .args(vec![
            Arg::new("pipe")
                .short('p')
                .about("Validate pipes in pipe manifest"),
            Arg::new("object")
                .short('o')
                .about("Validate objects in pipe manifest"),
            Arg::new("cstore")
                .short('c')
                .about("Validate context stores in pipe manifest"),
        ])
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult {
    let pipe = args.is_present("pipe");
    let object = args.is_present("object");
    let cstore = args.is_present("cstore");
    let opts = ValidateOptions::new(pipe, object, cstore);
    do_exec(config, &opts)?;
    Ok(())
}

pub struct ValidateOptions {
    pipe: bool,
    object: bool,
    cstore: bool,
}

impl ValidateOptions {
    pub fn pipe(&self) -> bool {
        self.pipe
    }

    pub fn object(&self) -> bool {
        self.object
    }

    pub fn cstore(&self) -> bool {
        self.cstore
    }

    pub fn new(pipe: bool, object: bool, cstore: bool) -> Self {
        ValidateOptions {
            pipe,
            object,
            cstore,
        }
    }
}

impl Default for ValidateOptions {
    fn default() -> Self {
        ValidateOptions {
            pipe: true,
            object: true,
            cstore: true,
        }
    }
}
