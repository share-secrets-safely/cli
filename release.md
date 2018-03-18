## V3.2: Master-Merger

The `merge` subcommand allows to combine JSON or YAML files.
This is useful to partition context and data according to your needs, yet use
all of the values in combination for substitution.
It is particularly useful if some of that content was just decrypted from a vault.

 * [x] **The merge sub-command**
   * [x] with conflicts
   * [x] with overwrite rules
   * [x] move keys to root level before merging
   * [x] insert keys at given value while merging for
     * [x] object paths, e.g `a.b.c` or `a/b/c`
     * [x] for array paths e.g. `a.0.c.1` or `a/0/c/1`
   * [x] merge complete environment into data, or whatever matches the given glob
   * [x] set individual values, simpy via 'a/b/c=42' or 'a.b.0=30'
 * [x] control the escape characters to allow passwords to be escaped properly, as needed, depending on the output format.
       Otherwise there is the chance of producing invalid YAML.

 * **improvements to substitute**
   * [x] `--verify` - try to decode substituted values and fail on error

 * **general improvements**
   * [x] Unify naming scheme of all deployables to make curling code easier
   * [x] find a better name for merge, given that merging is just a side-effect.
      With the action driven interface, it can do pretty much everything on the data
      it has so far. Some commands effect the merging, some pull out and/or print data.
      That way, extract is not a separate subcommand.
      
## V3.1: Substitution-Superpowers

Make it easy to generate property-sets by merging structured files together, and
make said context available to a `handlebars` powered engine to perform substitutions.

This allows to bring together context owned by various entities into a single aggregated
one, with the possibility for later contexts to override earlier ones.

With this capability, it's also possible to substitute secrets into files, for example
like this: `sy sub base.json sub/ours.yaml <(sy vault show secret.yaml) < deployment.yml | kubectl apply -f -`.

Read more [in the documentation](https://byron.github.io/share-secrets-safely/tools/substitute.html).

## V3.0: Support for Partitions

Partitions are just another vault with individual config, but operations on vaults are
aware of partitions. This allows sharing of keys for example, and alters the way
vaults are displayed when showing them.

### Features

 * **add** partitions and **remove** them
 * **initialize new vaults** with **partitions** in mind
 * **show recipients** per **partition**

### Improvements
 * Allow `sy vault` to operate anywhere with a `.gpg-id` file, like pass.
 * Strong validation of the vault configuration to assure consistency

### Breaking Changes

 * `vault --vault-id` is now `vault --select`

## V2.0: Better user experience and documentation

Besides the many improvements, you will also find [a complete book][book] about
the capabilities so far!

We also [sign our binaries][signatures] from here on, and make them available [via *homebrew*][install].

[signatures]: https://byron.github.io/share-secrets-safely/installation.html#via-a-hrefhttpsgithubcombyronshare-secrets-safelyreleasesreleasesa
[install]: https://byron.github.io/share-secrets-safely/installation.html#via-homebrew-osx-and-linux
[book]: https://byron.github.io/share-secrets-safely

### Improvements

 * `vault list` now produces precise URLs.
 * `vault remove` can remove resources from the vault.
 * `vault recipient add` now signs and re-exports added fingerprints to make
   recipient verification part of adding them, and help build a *Web of Trust*.
 * `vault recipients remove` removes recipients and re-encrypts the vaults content.
 * `vault recipient add` also adds recipients which are only in your gpg keychain.
    Previously it would always require an exported public key in the right spot.
 * `vault recipient add --verified` allows to add any recipient by name, but requires
    you to assure you are able to encrypt for that recipient.
 * `vault add` now creates sub-directories automatically.
 * `vault edit` now tries to encrypt before launching the editor.
 * `vault add :something` with a tty as standard input will open an editor automatically.

### Breaking Changes

The breaking change requiring a major version increment is changes to the `sy-vault.yml` file.

 * The `at` field is now called `secrets`
 * `recipients` and `gpg-keys` paths are no relative to the `sy-vault.yml` file, not relative to the
   `secrets` directory.
 * The '--at/-a' flag of `sy vault` is now `--secrets-dir-dir/-s`
 * `recipients add` will now require fingerprints unless `--verified` is specified.

These improvements make handling paths consistent and less suprising.

## V1.0.1: The very first release - with a new name!

`s3` now officially is `sy` on the command-line, and spelled `sheesy`. Crates
were renamed accordingly, too.

## V1.0: The very first release!

This is the first usable version, providing only the minimal amount of features.
A lot of the value contained is a fully automated system for quality assurance
and deployment, which will help keeping the releases coming.

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
