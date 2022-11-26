use clap::builder::ValueParser;

use clap::{Arg, Command};

pub trait CommandExt {
    fn args_model_region(self) -> Command;
}

impl CommandExt for Command {
    fn args_model_region(self) -> Command {
        self.arg(
            required_opt("model", "device model")
                .short('m')
                .value_name("MODEL"),
        )
        .arg(
            required_opt("region", "region model")
                .short('r')
                .value_name("REGION"),
        )
    }
}

pub fn opt(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name).long(name).help(help)
}

pub fn required_opt(name: &'static str, help: &'static str) -> Arg {
    opt(name, help).required(true)
}

pub fn path_arg(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name)
        .help(help)
        .value_parser(ValueParser::path_buf())
}

pub fn required_path_arg(name: &'static str, help: &'static str) -> Arg {
    path_arg(name, help).required(true)
}
