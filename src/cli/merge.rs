use clap::{App, Arg};
use glob;

pub fn cli<'a, 'b>() -> App<'a, 'b> {
    App::new("process")
        .alias("show")
        .alias("merge")
        .about(
            "Merge JSON or YAML files from standard input from specified files. \
             Multi-document YAML files are supported. \
             Merging a single file is explicitly valid and can be used to check for syntax errors.",
        )
        .arg(
            Arg::with_name("environment")
                .long("environment")
                .short("e")
                .takes_value(true)
                .default_value("*")
                .value_name("filter")
                .required(false)
                .multiple(true)
                .validator(|v| glob::Pattern::new(&v).map(|_| ()).map_err(|err| format!("{}", err)))
                .help("Import all environment variables matching the given filter. If no filter is set, all variables are imported. \
                       Otherwise it is applied as a glob, e.g. 'FOO*' includes 'FOO_BAR', but not 'BAZ_BAR'.\
                       Other valid meta characters are '?' to find any character, e.g. 'FO?' matches 'FOO'.")
        )
        .arg(
            Arg::with_name("no-overwrite")
                .long("no-overwrite")
                .takes_value(false)
                .required(false)
                .multiple(true)
                .help("If set, values in the merged document may not overwrite values already present. This is enabled by default,\
                       and can be explicitly turned off with --overwrite."),
        )
        .arg(
            Arg::with_name("overwrite")
                .long("overwrite")
                .takes_value(false)
                .required(false)
                .multiple(true)
                .help("If set, values in the merged document can overwrite values already present. This is disabled by default,\
                       and can be explicitly turned off with --no-overwrite."),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .required(false)
                .value_name("mode")
                .default_value("json")
                .possible_values(&["json", "yaml"])
                .case_insensitive(true)
                .help("Specifies how the merged result should be serialized."),
        )
        .arg(
            Arg::with_name("path")
                .takes_value(true)
                .required(false)
                .multiple(true)
                .help(
                "The path to the file to include. It must be in a format that can be output using the --output flag.",
            ),
        )
}
