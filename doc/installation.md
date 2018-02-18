
### Via HomeBrew (OSX and Linux)

This is by far the easiest way to get the binary. Just execute the following code:

```bash
brew tap byron/share-secrets-safely https://github.com/Byron/share-secrets-safely.git
brew install sheesy
```

This will install `gpg` as well, which is required for the [`sheesy vault`][syvault] to work.

[syvault]: vault/about.html

### Via [Releases][releases]

Please note that in order to use `sy`, you will need a working [installation of `gpg`][gpg].

Navigate to the [releases page][releases] and download a release binary suitable
for your system. A full example *for linux* looks like this:

```bash
curl -Lo sy.tar.gz https://github.com/Byron/share-secrets-safely/releases/download/2.0.0/sy-linux-musl-x86_64.tar.gz
tar xzf sy.tar.gz
# verify 'sy' was built by one of the maintainers
gpg --import <(curl -s https://raw.githubusercontent.com/Byron/share-secrets-safely/master/signing-keys.asc)
gpg --verify ./sy.gpg sy
# This will print out that the file was created by one of the maintainers. If you chose to
# trust the respective key after verifying it belongs to the maintainers, gpg will tell you
# it is verified.
# run sy if it was verified - even better when in your PATH
./sy
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

[dep-osx]: https://github.com/Byron/share-secrets-safely/blob/ffafeacb744bdbe7af5a6317ecb65ee9aae13311/.travis.yml#L30
[dep-debian]: https://github.com/Byron/share-secrets-safely/blob/ffafeacb744bdbe7af5a6317ecb65ee9aae13311/.travis.yml#L22
[releases]: https://github.com/Byron/share-secrets-safely/releases
[rustup]: http://rustup.rs
