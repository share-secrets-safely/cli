[![Build Status](https://travis-ci.org/Byron/share-secrets-safely.svg?branch=master)](https://travis-ci.org/Byron/share-secrets-safely)
[![dependency
status](https://deps.rs/repo/github/byron/share-secrets-safely/status.svg)](https://deps.rs/repo/github/byron/share-secrets-safely)

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

Please read the [installation notes here][installation].

[installation]: https://byron.github.io/share-secrets-safely/installation.html

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
 * **support old wheels - pass compatibility**
   * something `pass` does really well is to setup a vault with minimal infrastructure and configuration.
     We use said infrastructure and don't reinvent the wheel.
   * This makes us **compatible with pass**, allowing you use `pass` on a `sheesy` vault with default configuration.  


## Non-Goals

 * **replicate `pass` or `gpg` functionality directly**
   * having seen what `pass` actually is and how difficult it can be to use it especially in conjunction with `gpg`, this project will not even look at the provided functionality but be driven by its project goals instead.
 * **become something like hashicorp vault**
   * this solution is strictly file based and *offline*, so it can fill be used without any additional setup.

## Why would I use `sheesy` over...

You will find various and probably biased and opinionated comparisons [in our book][compare].
However, it's a fun read, and please feel free to make PRs for corrections.

[compare]: https://byron.github.io/share-secrets-safely/compare.html

## Caveats

 * Many crypto-operations store decrypted data in a temporary file. These touch
   disk and currently might be picked up by attackers. A fix could be 'tempfile',
   which allows using a secure temporary file - however, it might make getting
   MUSL builds impossible. Static builds should still be alright.
 * GPG2 is required to use the 'sign-key' operation. The latter is required when
   trying to add new unverified recipients via `vault recipients add <fingerprint>`.


## Roadmap to Future

## Roadmap to 5.1

### Add the `pass` subcommand

`sy` aims to be as usable as possible, and breaks compatiblity were needed to
achieve that. However, to allow people to leverage its improved portability
thanks to it being self-contained, it should be possible to let it act as a
stand-in for pass.

Even though its output won't be matched, its input will be matched perfectly, as
well as its behaviour.

## Roadmap to 5.0

### Adding the `extract` subcommand

The `extract` capability makes it feasilbe to store secrets in structured files
like YAML or JSON, as it allows to extract pieces of data in various ways.
That way, you can easily substitute secrets into configuration files using the
well-known `{{handlebar}}` syntax.

### Sub-Commands as standalone programs

Even though the main binary should by `sy` as before, the code should be structured to
provide `cli` versions of the respective subcommand, e.g. `vault-cli`.
That way, people can also use special-purpose sub-programs directly without having
a binary that contains all the other cruft.

This can be useful to make `pass` standins more approachable, and also build custom
`sy` binaries with just a sub-set of the functionality (for example, without `pass`
stand-in capability).

 * [ ] move vault-cli into own library and use it from `hub` cli.

## Roadmap to 4.0

### Web of Trust for everyone

The web-of-trust is powerful if used correctly, and helps to assure you are encrypting
only for trusted keys.

 * [ ] Configure web-of-trust options on per-partition basis and use that when encrypting.
 * [ ] Suggestion engine to learn how to encrypt for everyone in partition(s) with the
       least amount of work. It will suggest 'ownertrust' to well-connected people
       and make available everyone they signed for.
 * [ ] suggest to import keys or do it for the user
 * [ ] suggest to trust recipients or ((locally) sign) to make encryption possible
 * [ ] possibly allow to disable ownertrust using 'always-trust'

## Roadmap to 3.0

### Partition Support

Partitions are just another vault with individual config, but operations on vaults are
aware of partitions. This allows sharing key-lists, for example, and alters the way
vaults are displayed when showing them.

 * [ ] `multi-vault`
   * _manage multiple vaults in a single vault configuration file_
   * _it is possible to share public keys, too, so you can implement partitions_
 * [ ] Show vault with all partitions as tree
 * [ ] Show recipients per partition

## Roadmap to 2.0

### Documentation

Currently the only documentation that exists is the program itself, it is self-documentating
after all, as well as the journey tests. The latter are probably not what a user would look
like, so we should provide something more along the lines of how users want to use `sheesy`.

The most important thing to me is to test the documentation as well to assure it is always
accurately reflecting what was actually released. That way, they can also serve as high-level
smoke-tests.

How can that be done, you wonder? With `shell-book`! It allows you to run specially made
scripts in various modes:

 * **interactively**
   * The user runs chapters one by one and can press enter to run actual commands and see
     their output.
 * **test**
   * This one is run on CI and runs everything automatically, verifying the program invocations work as expected.
 * **mdbook**
   * generate pages suitable to be rendered by [`mdbook`][mdbook-github] (or equivalent)
     and deploy them to github pages.

Here is what would have to be done, in greater detail:

 * [x] `pulldown-cmark` event serialization back to markdown.
 * [x] test for processing code blocks to execute lines and capture their output.
 
 For more details, have a look at [`termbook`][termbook].

[termbook]: https://github.com/Byron/termbook
[mdbook-github]: https://github.com/rust-lang-nursery/mdBook

### Signed binaries

Even though it's great that travis is building binaries, the question is if it should be trusted.
Thus I believe the created archives should be processed after they have been created to re-assure
they are what we think they should be.

We should test the linux version again and sign it if all tests still work, and if the binary we
create locally matches the one on CI.

The same should be done on the host system (OSX) in that case, and if at all possible.
Providing signatures would also help prevent third parties distribute changed binaries on their own,
making the binaries produced here the only ones that are endorsed.

 * [x] Make deployment local and adjust scripts to include signature.
   * Unfortunately travis deployments don't really work well yet, and it's
     somewhat difficult to test.
   * Also it seems easiest to assure users can trust the binaries.

### Completing the `vault` subcommand

The first iteration only fulfilled the main journey. Now it's  time to fill the gaps
and add a few more features to provide API symmetry.

 * [x] Stream progress/output messages instead of aggregating them if all succeeded
   * For example, when adding a recipient, parts of the operation succeed, but
     it is not visible if re-encryption fails.
 * [x] `vault recipients`
   * [x] list
   * [x] init
   * [x] remove recipient(s) and re-encrypt
 * [x] `vault remove` a resource
 * [x] `vault add` create sub-directories automatically
 * [x] `vault add :secret` opens an editor if there is a tty and no input from stdin.

### UX - The next iteration

GPG is cryptic, and it's usually entirely unclear to the uninitiated user why
encryption just didn't work. Right now, we are not much better than using `pass`.

In this iteration, we want to achieve that for all major user journeys, **no
gpg error remains unexplained**.

 * [x] Suggest installing GPG if there is none
 * [x] Suggest creating a gpg key if there is none.
 * [x] try encrypting on edit (before the edit) to fail fast
 * [x] list recipients which are unusable when re-encryption fails (lack of trust)
 * [x] list recipients which are not available in the gpg key database.
 * [x] allow future recipients to export their key to the right spot.

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

### Making a release

As a prerequisite, you should be sure the build is green.

 * run `clippy` and fix all warnings with `cargo +nightly clippy --all`
 * change the version in the `VERSION` file
 * update the release notes in the `release.md` file.
   * Just prefix it with a description of new features and fixes
 * run `make tag-release`
   * requires push permissions to this repository
   * requires maintainer or owner privileges on crates.io for all deployed crates

### Making a deployment

As a prerequisite you must have made a release and your worktree must be clean,
with the HEAD at a commit.

For safety, tests will run once more as CI doesn't prevent you from publishing
red builds just yet.

  * run `make deployment`.
  * copy all text from the `release.md` file and copy it into the release text on github.
  * drag & drop all _tar.gz_  into the release and publish it.

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
