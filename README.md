[![Build Status](https://travis-ci.org/Byron/share-secrets-safely.svg?branch=master)](https://travis-ci.org/Byron/share-secrets-safely)

**sh**are-s**e**cr**e**ts-**s**afel**y** (_sheesy_) is a solution for managing 
shared secrets in teams and build pipelines.

Like [`pass`][pass], `sy` allows to setup a vault to store secrets, and share
them with your team members and tooling.
However, it wants to be a one-stop-shop in a single binary without any dependencies except
for a `gpg` installation,
helping users to work with the `gpg` toolchain and workaround peculiarities.

[![asciicast](https://asciinema.org/a/154953.png)](https://asciinema.org/a/154953)

[pass]: https://www.passwordstore.org/

## Installation

Please note that in order to use `sy`, you will need a working [installation of `gpg`][gpg].

[gpg]: https://www.gnupg.org/download/index.html#binary

### Via [Releases][releases]

Navigate to the [releases page][releases] and download a release binary suitable
for your system. A full example *for linux* looks like this:

```bash
curl -Lo sy.tar.gz https://github.com/Byron/share-secrets-safely/releases/download/1.0.1/sy-linux-musl-x86_64.tar.gz
tar xzf sy.tar.gz
# run sy  - even better when in your PATH
./sy
```

Here is a [recording](https://asciinema.org/a/154952) of how this can look like.

### Via [Cargo][rustup]

If you already have `cargo` available, installation is as easy as the following:

```bash
cargo install sheesy-cli
```

This installation should be preferred as it makes updating the binary much easier.
If you don't have `cargo` yet, you can install it via [instructions on rustup.rs][rustup].

[releases]: https://github.com/Byron/share-secrets-safely/releases
[rustup]: http://rustup.rs

## Getting Started

 * **sy --help** - help yourself
   * You can always use `--help` to learn about what the program can do. It is self-documenting.
 * **sy vault init** - create a new vault in the current directory
 * **sy vault add /path/to/file:file** - add an existing file to the vault 
 * **sy vault add :file** - read a secret from stdin
 * **sy vault recipients add name-of-mate** - add a recipient to the vault
   * Note that the recipient is identified by the ID of a key already imported
     into your gpg keychain.
   * In order for secrets to be re-encrypted, you must trust the new key enough.
     Either (_locally_) sign it, or trust it ultimately. Read more about the [web of trust][gpgweb].
 * **sy vault edit secret** - change a secret
 * **sy vault show secret** - print a secret to standard output

[gpgweb]: https://www.gnupg.org/gph/en/manual/x547.html

## Project Goals

 * **a great user experience**
   * The user experience comes first when designing the tool, making it easy for newcomers while providing experts with all the knobs to tune
   * deploy as *single binary*, without dynamically linked dependencies
 * **proven cryptography**
   * Don't reinvent the wheel, use *gpg* for crypto. It's OK to require `gpg` to be installed
     on the host
   * Thanks to *GPG* each user is identified separately through their public key
 * **automation and scripting is easy**
   * storing structured secrets is as easy as making them available in shell scripts
   * common operations like substituting secrets into a file are are natively supported
   * proper program exit codes make error handling easy
 * **user management**
   * support small and large teams, as well as multiple teams, with ease
   * make use of gpg's *web of trust* to allow inheriting trust even across team boundaries, and incentivize thorough checking of keys
 * **basic access control**
   * partition your secrets and define who can access them
 

## Non-Goals

 * **replicate `pass` or `gpg` functionality directly**
   * having seen what `pass` actually is and how difficult it can be to use it especially in conjunction with `gpg`, this project will not even look at the provided functionality but be driven by its project goals instead.
 * **become something like hashicorp vault**
   * this solution is strictly file based and *offline*, so it can fill be used without any additional setup.
   
## Why would I use `sheesy` over...

### Pass

The [standard unix pass][pass] is a shell script, which requires the presence of various standard unix tools, [among which][pass-deps] are
`tree` and `getopt`. The latter are actually not necessarily present, and if they are they may not produce exactly the same
results. On _OSX_ for example, the `gpg` file suffix is shown instead of hidden, and `pass` goes in an endless loop if
`getopt` is broken, which it is by default when `brew reinstall gnu-getopt` was not invoked.

`sheesy` has only one dependency: `gpg`, and even there it does not depend on the executable, but rather the `gpg-agent`.
It does not invoke the `gpg` command, and thus will not be confused by a change in the way `gpg` interprets its arguments
between minor version increments.

Besides, as `pass` just invokes `gpg`, it suffers from the horrible and hard-to-grok error messages that it produces.
Using `pass` and `gpg` requires to overcome a significant learning barrier, and you are required to know and understand
the 'Web of Trust' and all the error messages that come with not having one big enough to encrypt for the desired
recipients.

`sheesy` is built with _great user experience_ as first class requirement, and even though you will always see the underlying
`gpg` error, it will explain what it means and provide you with hints to solve the issue. When encryption fails, it will
list exactly for which recipient you cannot encrypt, and why.

`pass` even has a [few tests][pass-src], but it's unclear when and where these run. `sheesy` is developed in a test-driven
fashion, and has user-centric tests that model real-world interaction. This is the reason why those interactions are designed
to be understandable, consistent and easy to remember.

[pass]: https://www.passwordstore.org/
[pass-src]: https://git.zx2c4.com/password-store/tree/
[pass-deps]: https://git.zx2c4.com/password-store/tree/README

### Gopass

[`gopass`][gopass] is '_the slightly more awesome standard unix password manager for teams_' as claimed on the projects 
github page. As I have never used it beyond trying it locally, this paragraph might be lacking details. However, a first
impression is worth something, and here we go.

As it is a `go` program, it comes without any dependencies except for the `gpg` executable. It calls it directly, and thus
would be vulnerable to changes to the way `gpg` parses its arguments.

It's feature-ladden and seems overwhelming at first, it is clearly not centered
around user experience. Otherwise the user-journey would be much more streamlined and easier to comprehend. Many advanced
features I certainly don't get to enjoy that way.

Somewhat a sybling of the issue above seems to be that it is hell-bent on being a personal password store.
Thus it will store meta-data in your home directory and really wants a root-store which is placed in your 
home by default. So-called 'mounts' are really just a way to let it know about other `pass` compatible vaults, 
and I believe that makes it a buzz-word. Nonetheless, this made it hard for me to get started with it, and I still
feel highly uncomfortable to use it thanks to it opinionatedness.

Last but not least, and an issue that I find makes the case for not using `gopass` is that it actually 
[abandons the `Web of Trust`][gopass-wot] in favor of simplicity to the user. Even though I understand
why one would do that, I think the `Web of Trust` is an awesome idea, with terrible user experience, which
just begs you to make it usable for the masses thanks to better tooling.

Additionally `gopass` just aims to be a _slightly more awesome_ than `pass`, which shows as it is basically
pass written in `go` with more features.

Even though it certainly is better than `pass`, I wouldn't want to use it in its place because it adds so much
complexity while entirely removing the 'Web of Trust'. The latter seemed to have happened rather sneakily, which
I find problematic.

It should be valued that they [actively increase test-coverage][gopass-tests], but it also means that they don't
do test-driven development, which nurishes my doubt in the quality of the software.

[gopass]: https://github.com/justwatchcom/gopass
[gopass-wot]: https://github.com/justwatchcom/gopass/issues/305
[gopass-tests]: https://github.com/justwatchcom/gopass/search?q=Increase+test+coverage&type=Commits&utf8=%E2%9C%93

## Caveats

 * Many crypto-operations store decrypted data in a temporary file. These touch
   disk and currently might be picked up by attackers. A fix could be 'tempfile', 
   which allows using a secure temporary file - however, it might make getting
   MUSL builds impossible. Static builds should still be alright.
 * GPG2 is required to use the 'sign-key' operation. The latter is required when
   trying to add new unverified recipients via `vault recipients add <fingerprint>`.

## Roadmap

### Add the `pass` subcommand

`sy` aims to be as usable as possible, and breaks compatiblity were needed to
achieve that. However, to allow people to leverage its improved portability
thanks to it being self-contained, it should be possible to let it act as a
stand-in for pass.

Even though its output won't be matched, its input will be matched perfectly, as
well as its behaviour.

### Completing the `extract` subcommand

The `extract` capability makes it feasilbe to store secrets in structured files
like YAML or JSON, as it allows to extract pieces of data in various ways.
That way, you can easily substitute secrets into configuration files using the
well-known `{{handlebar}}` syntax.

### Completing the `vault` subcommand

The first iteration only fulfilled the main journey. Now it's  time to fill the gaps
and add a few more features to provide API symmetry.

 * [x] Stream progress/output messages instead of aggregating them if all succeeded
   * For example, when adding a recipient, parts of the operation succeed, but 
     it is not visible if re-encryption fails.
 * [ ] `vault recipients`
   * [x] list
   * [x] init
   * [ ] remove recipient(s) and re-encrypt
 * [ ] `vault remove` a resource
 * [ ] `vault add`
   * [ ] force overwrite flag
   * [ ] create sub-directories automatically
 * [ ] `vault add :secret` opens an editor if there is a tty and no input from stdin.
 * [ ] `multi-vault`
   * _manage multiple vaults in a single vault configuration file_
   * _it is possible to share public keys, too, so you can implement partitions_
 * [ ] it must be possible to turn off any automation introduced above
 
### UX - The next iteration

GPG is cryptic, and it's usually entirely unclear to the uniniciated user why
encryption just didn't work. Right now, we are not much better than using `pass`.

In this iteration, we want to achieve that for all major user journeys, **no 
gpg error remains unexplained**.

 * [x] Suggest installing GPG if there is none
 * [x] Suggest creating a gpg key if there is none.
 * [ ] try encrypting on edit (before the edit) to fail fast
   * [ ] suggest to import keys or do it for the user
   * [ ] suggest to trust recipients or ((locally) sign) to make encryption possible
   * [ ] possibly allow to disable ownertrust using 'always-trust'
 * [x] list recipients which are unusable when re-encryption fails (lack of trust)
 * [x] list recipients which are not available in the gpg key database.
 * [x] allow future recipients to export their key to the right spot.
 * [ ] it must be possible to turn off any automation introduced above
 * [ ] certain configuration flags should be persisted with the vault configuration

### On our way to the minimal viable product v1.0

 * [x] **setup rust workspace for clear dependency separation**
 * [x] **setup CI for linux and OSX**
 * [x] **standalone deployables without additional dependencies for**
   * [x] OSX (static binary) - _just gettext is still dynamically linked :(_
   * [x] MUSL Linux
 * [x] **shell completions**
 * [x] **complete a happy journey with**
   * [x] initialize a new vault
   * [x] add contents
   * [x] support for multiple vaults
   * [x] list vault contents
   * [x] decrypt vault contents
   * [x] edit vault contents
   * [x] add another user and re-encrypt vault content
 * [x] **installable from crates.io**
 * [x] **release binaries generated by travis for tags**
 
## Development Practices

 * **test-first development**
   * protect against regression and make implementing features easy
   * user docker to test more elaborate user interactions
   * keep it practical, knowing that the Rust compiler already has your back
     for the mundane things, like unhappy code paths.
 * **safety first**
   * handle all errors, never unwrap
   * provide an error chain and make it easy to understand what went wrong.
 * **strive for an MVP and version 1.0 fast...**
   * ...even if that includes only the most common usecases.
 * **Prefer to increment major version rapidly...**
   * ...instead of keeping major version zero for longer than needed.

## Maintenance Guide

### Making a deployment

As a prerequisite, you should be sure the build is green.

 * change the version in the `VERSION` file
 * update the release notes in the `release.md` file.
   * Just prefix it with a description of new features and fixes 
 * run `make tag-release`
   * requires push permissions to this repository
   * requires maintainer or owner privileges on crates.io for all deployed crates

### Making a new Asciinema recording

 * build the latest asciinema docker image
   * `docker build -t asciinema - < etc/docker/Dockerfile.asciinema`
 * drop into the image, possibly prepare it a little more
   * `docker run -it --rm asciinema`
   * `chmod ga+rw $(tty)` to allow changing to `su max` and allow `gpg --gen-key` to work.
 * Start a local recording
   * `asciinema rec -w 1 -t "A tour of sy" sy-demo.json`
 * Possibly upload the recording
   * `asciinema auth`
   * `asciinema upload sy-demo.json`
