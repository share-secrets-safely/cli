use clap::App;
use clap::AppSettings;

#[cfg(feature = "rest")]
mod completions;
#[cfg(feature = "vault")]
pub mod vault;
#[cfg(feature = "rest")]
mod substitute;
#[cfg(feature = "rest")]
mod merge;
#[cfg(feature = "rest")]
mod extract;
mod util;

pub struct CLI<'a, 'b>
where
    'a: 'b,
{
    pub app: App<'a, 'b>,
}

#[cfg(all(feature = "rest", feature = "vault"))]
impl<'a, 'b> CLI<'a, 'b>
where
    'a: 'b,
{
    pub fn name() -> &'static str {
        "sy"
    }

    pub fn new() -> Self {
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
