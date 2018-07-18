use clap::{App, Arg, ArgSettings};
use cli::util::output_formats;
use glob;

pub fn new<'a, 'b>() -> App<'a, 'b> {
    App::new("process")
        .alias("show")
        .alias("merge")
        .about(
            "Merge JSON or YAML files from standard input from specified files. \
             Multi-document YAML files are supported. \
             Merging a single file is explicitly valid and can be used to check for syntax errors.",
        )
        .arg(
            Arg::with_name("select")
                .set(ArgSettings::RequireEquals)
                .alias("from")
                .long("select")
                .short("s")
                .takes_value(true)
                .value_name("pointer")
                .required(false)
                .multiple(true)
                .help("Use a JSON pointer to specify which sub-value to use. \
                       This affects only the next following --environment or <path>. \
                       Valid specifications are for example '0/a/b/4' or 'a.b.0', and they must point to a valid value. \
                       If it is specified last, without a following merged value, a sub-value is selected from the aggregated value."
                )
        )
        .arg(
            Arg::with_name("no-stdin")
                .long("no-stdin")
                .required(false)
                .help("If set, we will not try to read structured data from standard input. This may be required \
                       in some situations where we are blockingly reading from a standard input which is attached \
                       to a pseudo-terminal.")
        )
        .arg(
            Arg::with_name("at")
                .set(ArgSettings::RequireEquals)
                .alias("to")
                .long("at")
                .short("a")
                .takes_value(true)
                .value_name("pointer")
                .required(false)
                .multiple(true)
                .help("Use a JSON pointer to specify an existing mapping at which the next merged value should be placed. \
                       This affects only the next following --environment or <path>. \
                       Valid specifications are for example '0/a/b/4' or 'a.b.0'. \
                       If it is specified last, without a following merged value, the entire aggregated value so far is moved."
                )
        )
        .arg(
            Arg::with_name("environment")
                .set(ArgSettings::RequireEquals)
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
                .alias("no-override")
                .long("no-overwrite")
                .takes_value(false)
                .required(false)
                .multiple(true)
                .help("If set, values in the merged document may not overwrite values already present. This is enabled by default,\
                       and can be explicitly turned off with --overwrite."),
        )
        .arg(
            Arg::with_name("overwrite")
                .alias("override")
                .long("overwrite")
                .takes_value(false)
                .required(false)
                .multiple(true)
                .help("If set, values in the merged document can overwrite values already present. This is disabled by default,\
                       and can be explicitly turned off with --no-overwrite."),
        )
        .arg(
            Arg::with_name("output")
                .set(ArgSettings::RequireEquals)
                .short("o")
                .long("output")
                .takes_value(true)
                .required(false)
                .value_name("mode")
                .default_value("json")
                .possible_values(output_formats())
                .case_insensitive(true)
                .help("Specifies how the merged result should be serialized."),
        )
        .arg(
            Arg::with_name("path")
                .value_name("path-or-value")
                .takes_value(true)
                .required(false)
                .multiple(true)
                .help(
                "The path to the file to include, or '-' to read from standard input. It must be in a format that can be output using the --output flag. \
                 Alternatively it can be a value assignment like 'a=42' or a.b.c=value.",
            ),
        )
}
