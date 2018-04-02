use failure::Error;
use vault::{print_causes, error::{first_cause_of_type, DecryptionError}};
use std::io::{stderr, stdout, Write};
use std::process;
use clap::ArgMatches;
use cli::CLI;

pub fn amend_error_info<T>(r: Result<T, Error>) -> Result<T, Error> {
    r.map_err(|e| {
        let ctx = match first_cause_of_type::<DecryptionError>(&e) {
            Some(_err) => Some(format!(
                "Export your public key using '{} vault recipient init', then \
                 ask one of the existing recipients to import your public key \
                 using '{} vault recipients add <your-userid>.'",
                CLI::name(),
                CLI::name()
            )),
            None => None,
        };
        (e, ctx)
    }).map_err(|(e, msg)| match msg {
        Some(msg) => e.context(msg).into(),
        None => e,
    })
}

pub fn ok_or_exit<T, E>(r: Result<T, E>) -> T
where
    E: Into<Error>,
{
    match r {
        Ok(r) => r,
        Err(e) => {
            stdout().flush().ok();
            write!(stderr(), "error: ").ok();
            print_causes(e, stderr());
            process::exit(1);
        }
    }
}

pub fn usage_and_exit(args: &ArgMatches) -> ! {
    println!("{}", args.usage());
    process::exit(1)
}
