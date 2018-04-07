
### Via HomeBrew (OSX and Linux)

This is by far the easiest way to get the binary. Just execute the following code:

```bash
brew tap share-secrets-safely/cli
brew install sheesy
```

This will install `gpg` as well, which is required for the [`sheesy vault`][syvault] to work.

[syvault]: vault/about.html

### Via [Releases][releases]

Please note that in order to use `sy`, you will need a working [installation of `gpg`][gpg].

Navigate to the [releases page][releases] and download a release binary suitable
for your system. A full example *for linux* looks like this:

```bash,prepare=sy-in-path,hide
set -eu
export PATH="/volume/${EXE_PATH%/*}:$PATH"
```

```bash,exec,hide
gpg --import /volume/tests/journeys/fixtures/tester.sec.asc &>/dev/null
fpr="$(gpg --list-secret-keys --with-colons --with-fingerprint | grep fpr | head -1)"
fpr=${fpr:12:40}

function trust_key () {
  {
    gpg --export-ownertrust
    echo "${1:?First argument is the long fingerprint of the key to trust}:6:"
  } | gpg --import-ownertrust &>/dev/null
}

trust_key "$fpr"
```

```bash,prepare=sandboxed,hide
sandbox_tempdir="$(mktemp -t sandbox.XXXX -d)"
pushd "$sandbox_tempdir" >/dev/null
```

```bash,use=sandboxed,exec
curl --fail -Lso sy.tar.gz https://github.com/share-secrets-safely/cli/releases/download/4.0.0/sy-cli-Linux-x86_64.tar.gz
curl --fail -Lso sy.tar.gz.gpg https://github.com/share-secrets-safely/cli/releases/download/4.0.0/sy-cli-Linux-x86_64.tar.gz.gpg
# verify 'sy' was built by one of the maintainers
gpg --import <(curl -s https://raw.githubusercontent.com/share-secrets-safely/cli/master/signing-keys.asc) 2>/dev/null
gpg --sign-key --yes --batch 296B26A2B943AFFA &>/dev/null
gpg --verify ./sy.tar.gz.gpg sy.tar.gz
# now that we know it's the real thing, let's use it.
tar xzf sy.tar.gz
# This will print out that the file was created by one of the maintainers. If you chose to
# trust the respective key after verifying it belongs to the maintainers, gpg will tell you
# it is verified.

# Finally put the executable into your PATH
mv ./sy /usr/local/bin
```

Now the `sy` executable is available in your `PATH`.

```bash,exec
sy --help
```

[gpg]: https://www.gnupg.org/download/index.html#binary

### Via [Cargo][rustup]

If you already have `cargo` available, installation is as easy as the following:

```bash
cargo install sheesy-cli
```

This installation should be preferred as it makes updating the binary much easier.
If you don't have `cargo` yet, you can install it via [instructions on rustup.rs][rustup].

Please note that for building on OSX, you are required to locally install [certain dependencies][dep-osx],
which is also the case on [linux systems][dep-debian].

[dep-osx]: https://github.com/share-secrets-safely/cli/blob/ffafeacb744bdbe7af5a6317ecb65ee9aae13311/.travis.yml#L30
[dep-debian]: https://github.com/share-secrets-safely/cli/blob/ffafeacb744bdbe7af5a6317ecb65ee9aae13311/.travis.yml#L22
[releases]: https://github.com/share-secrets-safely/cli/releases
[rustup]: http://rustup.rs
