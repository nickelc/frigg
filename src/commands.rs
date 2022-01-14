pub type App = clap::App<'static>;
pub type Arg = clap::Arg<'static>;

pub trait AppExt {
    fn args_model_region(self) -> App;
}

impl AppExt for App {
    fn args_model_region(self) -> App {
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
    Arg::new(name).help(help).allow_invalid_utf8(true)
}

pub fn required_path_arg(name: &'static str, help: &'static str) -> Arg {
    path_arg(name, help).required(true)
}
