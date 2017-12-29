[![Build Status](https://travis-ci.org/Byron/share-secrets-safely.svg?branch=master)](https://travis-ci.org/Byron/share-secrets-safely)

`share-secrets-safely` is a GPG based solution for managing shared secrets.

## Installation

### Via [Releases][releases]

Navigate to the [releases page][releases] and download a release binary suitable
for your system. A full example *for linux* looks like this:

```bash
curl -Lo s3.tar.gz https://github.com/Byron/share-secrets-safely/releases/download/1.0.0/s3-linux-musl-x86_64.tar.gz
tar xzf s3.tar.gz
# run s3 - please feel free to put it into your PATH
./s3
```

### Via [Cargo][rustup]

If you already have `cargo` available, installation is as easy as the following:

```bash
cargo install s3-cli
```

This installation should be preferred as it makes updating the binary much easier.
If you don't have `cargo` yet, you can install it via [instructions on rustup.rs][rustup].

[releases]: https://github.com/Byron/share-secrets-safely/releases
[rustup]: http://rustup.rs

## Features

 * **easy access to GPG cryptography**
 * **easy to deploy with just a single binary**
 * **compatible to pass**
 * **state-of-art command-line interface**
 
## Project Goals

 * **a great user experience**
   * The user experience comes first when designing the tool, making it easy for newcomers while providing experts with all the knobs to tune
   * deploy as *single binary*, no additional executables or dependencies are required to use all of the features
 * **proven cryptography**
   * Don't reinvent the wheel, use *gpg* for crypto
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

## Roadmap

### Add the `pass` subcommand

`s3` aims to be as usable as possible, and breaks compatiblity were needed to
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

 * [ ] `vault recipients`
   * [ ] list
   * [ ] remove recipient(s) and re-encrypt
 * [ ] `vault remove` a resource
 * [ ] `vault add` force overwrite flag
 * [ ] `multi-vault`
   * _manage multiple vaults in a single vault configuration file_
   * _it is possible to share public keys, too, so you can implement partitions_

### UX - The next iteration

GPG is cryptic, and it's usually entirely unclear to the uniniciated user why
encryption just didn't work. Right now, we are not much better than using `pass`.

In this iteration, we want to achieve that for all major user journeys, **no 
gpg error remains unexplained**.

 * [ ] Suggest creating a gpg key if there is none.
 * [ ] try encrypting on edit (before the edit) to fail fast
   * [ ] suggest to import keys or do it for the user
   * [ ] suggest to trust recipients or ((locally) sign) to make encryption possible
   * [ ] possibly allow the user (locally sign) recipients
   * [ ] possibly allow to disable ownertrust using 'always-trust'
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
 
## Caveats

 * Many crypto-operations store decrypted data in a temporary file. These touch
   disk and currently might be picked up by attackers. A fix could be 'tempfile', 
   which allows using a secure temporary file - however, it might make getting
   MUSL builds impossible. Static builds should still be alright.

## Development Practices

 * **test-first development**
   * protect against regression and make implementing features easy
   * user docker to test more elaborate user interactions
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
   * `chmod a+rw $(tty)` to allow changing to `su max` and allow `gpg --gen-key` to work.
 * Start a local recording
   * `asciinema rec -w 1 -t "A tour of S3" s3-demo.json`
 * Possibly upload the recording
   * `asciinema auth`
   * `asciinema upload s3-demo.json`
