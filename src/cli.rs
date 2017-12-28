use clap::{App, Arg};
use std::env;

lazy_static! {
    static ref SHELL: Result<String, env::VarError> = env::var("SHELL");
    static ref EDITOR: Result<String, env::VarError> = env::var("EDITOR");
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
        let gpg_key_id = Arg::with_name("gpg-key-id")
            .multiple(true)
            .required(false)
            .takes_value(true)
            .value_name("userid")
            .help("The key-id of the public key identifying a recipient in your gpg keychain.");
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
                Arg::with_name("at")
                    .long("at")
                    .short("a")
                    .default_value(".")
                    .required(false)
                    .takes_value(true)
                    .value_name("path")
                    .help("The location which is the root of all vault content."),
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
            .arg(gpg_key_id.clone().long("gpg-key-id").short("i"));

        let list = App::new("list")
            .alias("ls")
            .about("List the vault's content.");
        let resource_path = Arg::with_name("path")
            .required(true)
            .multiple(false)
            .takes_value(true)
            .value_name("path")
            .help(
                "Either a vault-relative path to the file as displayed by 'vault show',\
                 a vault-relative path with the '.gpg' suffix, or an absolute \
                 path with or without the '.gpg' suffix.",
            );
        let edit_resource = App::new("edit")
            .arg(
                Arg::with_name("no-create")
                    .long("no-create")
                    .required(false)
                    .help(
                        "If set, the resource you are editing must exist. \
                         Otherwise it will be created on the fly, allowing you to \
                         add new resources by editing them.",
                    ),
            )
            .arg(
                Arg::with_name("editor")
                    .long("editor")
                    .short("e")
                    .required(false)
                    .takes_value(true)
                    .default_value(EDITOR.as_ref().map(String::as_str).unwrap_or("vim"))
                    .help(
                        "The path to your editor program. It receives the decrypted content as first \
                         argument and is expected to write the changes back to that file before quitting.",
                    ),
            )
            .arg(resource_path.clone())
            .about(
                "Edit a resource. This will decrypt the resource to \
                 a temporary file, open up the $EDITOR you have specified, and re-encrypt the \
                 changed content before deleting it on disk.",
            );
        let show_resource = App::new("show")
            .about("Decrypt a resource")
            .arg(resource_path);
        let add_resource = App::new("add")
            .alias("insert")
            .about("Add a new resource to the vault.")
            .arg(
                Arg::with_name("spec")
                    .required(true)
                    .multiple(false)
                    .takes_value(true)
                    .value_name("spec")
                    .help(
                        "A specification identifying a mapping from a source file to be stored \
                         in a location of the vault. It takes the form '<src>:<dst>', where \
                         '<src>' is equivalent to '<src>:<src>'.\
                         <dst> should be vault-relative paths, whereas <src> must point tel a readable file \
                         and can be empty to read from standard input, such as in ':<dst>'.",
                    ),
            );
        let add_recipient = App::new("add")
            .alias("insert")
            .arg(gpg_key_id.required(true))
            .about("Add a new recipient. This will re-encrypt all the vaults content.");
        let recipients = App::new("recipients")
            .alias("recipient")
            .about("Interact with recipients of a vault. They can encrypt and decrypt its contents.")
            .subcommand(add_recipient);
        let vault = App::new("vault")
            .about("a variety of vault interactions")
            .subcommand(init)
            .subcommand(add_resource)
            .subcommand(recipients)
            .subcommand(show_resource)
            .subcommand(edit_resource)
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
