use std::any::Any;

use clap::builder::ValueParser;

use clap::{Arg, ArgMatches, Command};

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

pub trait ArgMatchesExt {
    fn _get_one<T: Any + Clone + Send + Sync + 'static>(&self, id: &str) -> Option<&T>;

    fn get_model(&self) -> Option<&String> {
        self._get_one("model")
    }

    fn get_region(&self) -> Option<&String> {
        self._get_one("region")
    }
}

impl ArgMatchesExt for ArgMatches {
    fn _get_one<T: Any + Clone + Send + Sync + 'static>(&self, id: &str) -> Option<&T> {
        self.get_one(id)
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
