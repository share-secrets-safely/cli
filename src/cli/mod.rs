use clap::App;
use clap::AppSettings;

mod completions;
mod vault;
mod substitute;
mod merge;

pub use self::merge::OutputMode;

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
            .name(CLI::name())
            .after_help("Read more on https://byron.github.io/share-secrets-safely")
            .version(include_str!("../../VERSION"))
            .subcommand(vault::cli())
            .subcommand(substitute::cli())
            .subcommand(merge::cli())
            .subcommand(completions::cli());

        Self { app }
    }
}
