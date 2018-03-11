use clap::{App, Arg};
use clap::AppSettings;

pub fn cli<'a, 'b>() -> App<'a, 'b> {
    App::new("substitute")
        .setting(AppSettings::AllowLeadingHyphen)
        .alias("sub")
        .about("Substitutes templates using structured data. \
                 The idea is to build a tree of data that is used to substitute in various templates, using multiple inputs and outputs.\
                 That way, secrets (like credentials) can be extracted from the vault just once and used wherever needed without them touching disk.\
                 Handlebars is used as templating engine, and it's possible to refer to and inherit from other templates by their file-stem.")
        .arg(
            Arg::with_name("separator")
                .required(false)
                .multiple(false)
                .takes_value(true)
                .long("separator")
                .short("s")
                .default_value("\n")
                .value_name("separator")
                .help("The string to use to separate multiple documents that are written to the same stream. \
                            This can be useful to output a multi-document YAML file from multiple input templates \
                            to stdout if the separator is '---'. \
                            The separator is also used when writing multiple templates into the same file, like in 'a:out b:out'.")
        )
        .arg(
            Arg::with_name("data")
                .required(false)
                .multiple(false)
                .takes_value(true)
                .long("data")
                .short("d")
                .value_name("data")
                .help("Structured data in YAML or JSON format to use when instantiating/substituting the template."),
        )
        .arg(
            Arg::with_name("spec")
                .required(false)
                .multiple(true)
                .takes_value(true)
                .value_name("template-spec")
                .long_help("Identifies the how to map template files to output. \
                     The syntax is '<src>:<dst>'. \
                     <src> and <dst> are a relative or absolute paths to the source templates or \
                     destination files respectively. \
                     If <src> is unspecified, the template will be read from stdin, e.g. ':output'. \
                     If <dst> is unspecified, the substituted template will be output to stdout, e.g 'input.hbs:' \
                     or 'input.hbs'.",
                ),
        )
}
