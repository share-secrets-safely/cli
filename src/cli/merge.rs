use clap::{App, Arg};

arg_enum!{
    #[derive(PartialEq, Debug)]
    pub enum OutputMode {
        Json,
        Yaml
    }
}

pub fn cli<'a, 'b>() -> App<'a, 'b> {
    App::new("merge")
        .alias("show")
        .about(
            "Merge JSON or YAML files from standard input from specified files. \
             Multi-document YAML files are supported. \
             Merging a single file is explicitly valid and can be used to check for syntax errors.",
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .required(false)
                .value_name("mode")
                .default_value("json")
                .possible_values(&OutputMode::variants())
                .case_insensitive(true)
                .help("Specifies how the merged result should be serialized."),
        )
        .arg(
            Arg::with_name("path")
                .takes_value(true)
                .required(false)
                .multiple(true)
                .help("The path to the file to include. It must be in a format that can be output using the --output flag."),
        )
}
