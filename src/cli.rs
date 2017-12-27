use clap::{App, Arg};
use std::env;

lazy_static! {
    static ref SHELL: Result<String, env::VarError> = env::var("SHELL");
}

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
        "s3"
    }

    pub fn new() -> Self {
        let init = App::new("init")
            .about(
                "Initialize the vault in the current directory. \
                 \
                 If --gpg-key-id is unset, we will use the \
                 only key that you have a secret key for, assuming it is yours.\
                 If you have multiple keys, the --gpg-key-id must be specified \
                 to make the input unambiguous.",
            )
            .arg(
                Arg::with_name("recipients-file-path")
                    .long("recipients-file")
                    .default_value(".gpg-id")
                    .short("r")
                    .required(false)
                    .takes_value(true)
                    .value_name("path")
                    .help(
                        "The directory to hold the public keys identified by \
                         --gpg-key-id, with signatures.",
                    ),
            )
            .arg(
                Arg::with_name("gpg-keys-dir")
                    .long("gpg-keys-dir")
                    .default_value(".gpg-keys")
                    .short("k")
                    .required(false)
                    .takes_value(true)
                    .value_name("directory")
                    .help(
                        "The directory to hold the public keys identified by \
                         --gpg-key-id, with signatures.",
                    ),
            )
            .arg(
                Arg::with_name("gpg-key-id")
                    .long("gpg-key-id")
                    .multiple(true)
                    .short("i")
                    .required(false)
                    .takes_value(true)
                    .value_name("userid")
                    .help("The key-id of the public key identifying your own user(s)."),
            );

        let list = App::new("list")
            .alias("ls")
            .about("List the vault's content.");
        let show_resource = App::new("show").about("Decrypt a resource").arg(
            Arg::with_name("path")
                .required(true)
                .multiple(false)
                .takes_value(true)
                .value_name("path")
                .help(
                    "Either a vault-relative path to the file as displayed by 'vault show',\
                     a vault-relative path with the '.gpg' suffix, or an absolute \
                     path with or without the '.gpg' suffix.",
                ),
        );
        let add_resource = App::new("add")
            .alias("insert")
            .about("Add a new resource to the vault.")
            .arg(
                Arg::with_name("spec")
                    .required(true)
                    .multiple(false)
                    .takes_value(true)
                    .value_name("spec")
                    .help("A specification identifying a mapping from a source file to be stored \
            in a location of the vault. It takes the form '<src>:<dst>', where \
            '<src>' is equivalent to '<src>:<src>'.\
            <dst> should be vault-relative paths, whereas <src> must point tel a readable file \
            and can be empty to read from standard input, such as in ':<dst>'."),
            );
        let vault = App::new("vault")
            .about("a variety of vault interactions")
            .subcommand(init)
            .subcommand(add_resource)
            .subcommand(show_resource)
            .subcommand(list)
            .arg(
                Arg::with_name("vault-id")
                    .short("i")
                    .long("vault-id")
                    .required(false)
                    .value_name("id")
                    .help("Either an index into the vaults list, or the name of the vault.")
                    .default_value("0"),
            )
            .arg(
                Arg::with_name("config-file")
                    .long("config-file")
                    .short("c")
                    .required(true)
                    .value_name("path")
                    .help("Path to the vault configuration YAML file.")
                    .default_value("./s3-vault.yml"),
            );
        let extract = App::new("extract")
            .about("utilities to extract information from structured data files")
            .arg(
                Arg::with_name("file")
                    .short("f")
                    .required(true)
                    .help("Path to the data file to read"),
            );
        let completions = App::new("completions")
            .about("generate completions for supported shell")
            .arg({
                let arg = Arg::with_name("shell").required(SHELL.is_err()).help(
                    "The name of the shell, or the path to the shell as exposed by the \
                     $SHELL variable.",
                );
                if let Ok(shell) = SHELL.as_ref() {
                    arg.default_value(shell)
                } else {
                    arg
                }
            });
        let app: App = app_from_crate!()
            .name(CLI::name())
            .subcommand(vault)
            .subcommand(completions)
            .subcommand(extract);

        Self { app }
    }
}
