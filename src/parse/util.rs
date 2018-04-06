use clap::ArgMatches;
#[cfg(any(feature = "vault", feature = "substitute"))]
use failure::Error;
#[cfg(any(feature = "vault", feature = "substitute"))]
use std::ffi::OsStr;
#[cfg(feature = "vault")]
use std::str::FromStr;

#[cfg(any(feature = "process", feature = "extract"))]
pub fn optional_args_with_value<F, T>(args: &ArgMatches, name: &'static str, into: F) -> Vec<(T, usize)>
where
    F: Fn(&str) -> T,
{
    if args.occurrences_of(name) > 0 {
        match (args.values_of(name), args.indices_of(name)) {
            (Some(v), Some(i)) => v.map(|v| into(v)).zip(i).collect(),
            (None, None) => Vec::new(),
            _ => unreachable!("expecting clap to work"),
        }
    } else {
        Vec::new()
    }
}

#[cfg(any(feature = "vault", feature = "substitute"))]
pub fn required_os_arg<'a, T>(args: &'a ArgMatches, name: &'static str) -> Result<T, Error>
where
    T: From<&'a OsStr>,
{
    match args.value_of_os(name).map(Into::into) {
        Some(t) => Ok(t),
        None => Err(format_err!("BUG: expected clap argument '{}' to be set", name)),
    }
}

#[cfg(feature = "vault")]
pub fn optional_args<'a, T>(args: &'a ArgMatches, name: &'static str) -> Vec<T>
where
    T: From<&'a str>,
{
    args.values_of(name)
        .map(|v| v.map(Into::into).collect())
        .unwrap_or_else(Vec::new)
}

#[cfg(feature = "vault")]
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
