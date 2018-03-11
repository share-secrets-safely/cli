use failure::{err_msg, Error, ResultExt};
use clap::App;
use clap::ArgMatches;
use std::io::stdout;
use cli::CLI;
use clap::Shell;
use std::path::Path;
use std::str::FromStr;

pub fn generate(mut app: App, args: &ArgMatches) -> Result<(), Error> {
    let shell = args.value_of("shell")
        .ok_or_else(|| err_msg("expected 'shell' argument"))
        .map(|s| {
            Path::new(s)
                .file_name()
                .map(|f| {
                    f.to_str()
                        .expect("os-string to str conversion to work for filename")
                })
                .unwrap_or(s)
        })
        .and_then(|s| {
            Shell::from_str(s)
                .map_err(err_msg)
                .with_context(|_| format!("The shell '{}' is unsupported", s))
                .map_err(Into::into)
        })?;
    app.gen_completions_to(CLI::name(), shell, &mut stdout());
    Ok(())
}
