use clap::App;
use clap::AppSettings;

mod completions;
pub mod vault;
mod substitute;
mod merge;
mod extract;
mod util;

pub struct CLI<'a, 'b>
where
    'a: 'b,
{
    pub app: App<'a, 'b>,
}

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
