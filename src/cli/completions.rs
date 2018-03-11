use clap::{App, Arg};
use std::env;

lazy_static! {
    static ref SHELL: Result<String, env::VarError> = env::var("SHELL");
}

pub fn cli<'a, 'b>() -> App<'a, 'b> {
    App::new("completions")
        .about("generate completions for supported shell")
        .arg({
            let arg = Arg::with_name("shell").required(SHELL.is_err()).help(
                "The name of the shell, or the path to the shell as exposed by the \
                 $SHELL variable.",
            );
            if let Ok(shell) = SHELL.as_ref() {
                arg.default_value(shell)
            } else {
                arg
            }
        })
}
