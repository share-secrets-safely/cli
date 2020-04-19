use crate::cli::util::output_formats;
use clap::ArgSettings;
use clap::{App, Arg};

pub fn new<'a, 'b>() -> App<'a, 'b> {
    App::new("extract")
        .alias("fetch")
        .about(
            "Extract scalar or complex values from any JSON or YAML file. \
             Multi-document YAML files are supported.",
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
            Arg::with_name("output")
                .set(ArgSettings::RequireEquals)
                .short("o")
                .long("output")
                .takes_value(true)
                .required(false)
                .value_name("mode")
                .possible_values(output_formats())
                .case_insensitive(true)
                .help(
                    "Specifies how the extracted result should be serialized. \
                     If the output format is not explicitly set, the output will be a single scalar value per line. \
                     If the output contains a complex value, the default serialization format will be used.",
                ),
        )
        .arg(
            Arg::with_name("file")
                .set(ArgSettings::RequireEquals)
                .value_name("file")
                .long("file")
                .short("f")
                .takes_value(true)
                .required(false)
                .multiple(true)
                .help(
                    "The path to the file to include, or '-' to read from standard input. It must be in a format that can be output using the --output flag."
                ),
        )
        .arg(
            Arg::with_name("pointer")
                .takes_value(true)
                .required(true)
                .multiple(true)
                .help(
                    "Use a JSON pointer to specify which value to extract. \
                     Valid specifications are for example '0/a/b/4' or 'a.b.0', and they must point to a valid value.",
                ),
        )
}
