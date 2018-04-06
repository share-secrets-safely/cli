#[cfg(any(feature = "vault", feature = "completions"))]
use clap::App;

#[cfg(feature = "completions")]
mod completions;
#[cfg(feature = "vault")]
pub mod vault;
#[cfg(feature = "substitute")]
pub mod substitute;
#[cfg(feature = "process")]
pub mod merge;
#[cfg(feature = "extract")]
pub mod extract;
mod util;

#[cfg(any(feature = "vault", feature = "completions"))]
pub struct CLI<'a, 'b>
where
    'a: 'b,
{
    pub app: App<'a, 'b>,
}

#[cfg(feature = "vault")]
impl<'a, 'b> CLI<'a, 'b>
where
    'a: 'b,
{
    pub fn name() -> &'static str {
        "sy"
    }
}

#[cfg(all(feature = "completions", feature = "vault"))]
impl<'a, 'b> CLI<'a, 'b>
where
    'a: 'b,
{
    pub fn new() -> Self {
        use clap::AppSettings;
        let app: App = app_from_crate!()
            .setting(AppSettings::VersionlessSubcommands)
            .setting(AppSettings::DeriveDisplayOrder)
            .setting(AppSettings::SubcommandRequired)
            .name(CLI::name())
            .after_help("Read more on https://share-secrets-safely.github.io/cli")
            .version(include_str!("../../VERSION"))
            .subcommand(vault::new())
            .subcommand(substitute::new())
            .subcommand(merge::new())
            .subcommand(extract::new())
            .subcommand(completions::new());

        Self { app }
    }
}
