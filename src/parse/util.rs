use clap::ArgMatches;
use failure::Error;
use std::ffi::OsStr;
use std::str::FromStr;

pub fn required_os_arg<'a, T>(args: &'a ArgMatches, name: &'static str) -> Result<T, Error>
where
    T: From<&'a OsStr>,
{
    match args.value_of_os(name).map(Into::into) {
        Some(t) => Ok(t),
        None => Err(format_err!("BUG: expected clap argument '{}' to be set", name)),
    }
}

pub fn optional_args<'a, T>(args: &'a ArgMatches, name: &'static str) -> Vec<T>
where
    T: From<&'a str>,
{
    args.values_of(name)
        .map(|v| v.map(Into::into).collect())
        .unwrap_or_else(Vec::new)
}

pub fn required_arg<T>(args: &ArgMatches, name: &'static str) -> Result<T, Error>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + ::std::error::Error + Send + Sync,
{
    match args.value_of(name).map(FromStr::from_str) {
        Some(Ok(t)) => Ok(t),
        Some(Err(e)) => Err(e.into()),
        None => Err(format_err!("BUG: expected clap argument '{}' to be set", name)),
    }
}
