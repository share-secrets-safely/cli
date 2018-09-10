# Share Secrets Safely

...and here is how!

`share secrets safely`, or `sheesy` in short, is a command-line tool which makes
using cryptography powered by [`gpg`][gpg] easy.

It's there for you and your team to help you avoid putting plain-text secrets into
your repository from day one.

However, there is more to it and this guide will give you an overview of the 
difficulties associated with *shared secrets*, and how to overcome them.

[gpg]: https://www.gnupg.org/

## About shared secrets!

`secrets`, that's knowledge whose possession yields a value. It can be credentials
providing access to databases, which in turn may contain confidential data or all
your customers payment information.
Or it can be a token giving full access to a big companys AWS account with unlimited
credit.

The first line of defense is to never, ever store `secrets` in plain text! This cannot
be stressed enough. Never do it, ever, and instead read the ['First Steps'][first-steps]
to learn how to avoid it from day one.

[first-steps]: vault/first-steps.html

## Rotation, Rotation, Rotation!

An interesting property of *shared* secrets is that once they have been read, they
must be considered leaked. By using `sheesy` you try to assure no unauthorized party
gets to easily read them, but authorized parties still read them eventually, adding
the risk of leakage each time.

After adding `sheesy` into your workflow, it *must* be your next goal to make it easy
(e.g. automatically) to rotate these secrets. This invalidates any leak and acts like a
reset. The shorter the secrets remain valid, the better when it comes to risk of leakage.

_If you think further, the safest secrets are the ones that never stay valid for an extended
period of time, and which are tied to a specific entity._

## The Tools

Fortunately all the tooling exists to avoid plain-text secrets and make sharing them
a little safer.
`sheesy` does nothing more than bringing them together in a single binary.

Those are namely

 * **gpg-cryptography**
   * This provides user-identification with public keys as well as proven cryptography
   * The [*Web of Trust*][wot] helps to conveniently assure public keys actually belong
     to the individual, assuring nobody *sneaks in*.
 * **_sheesy_ command-line tool**
   * A binary communicating with the `gpg` cryptography engine via [`gpgme`][gpgme].
   * It provides a great user experience making using `gpg` easy even without prior
     knowledge.
   * It provides capabilities to make it easy to use `sheesy vaults` from your
     build pipeline.
 * **pass - the standard unix password manager**
   * [pass][pass] is a shell-script which drives the `gpg` program to make it easier
     to use in teams. `sheesy` vaults are fully compatible to `pass` vaults.

[wot]: https://www.gnupg.org/gph/en/manual/x547.html
[gpgme]: https://github.com/johnschug/rust-gpgme
[pass]: https://www.passwordstore.org

   
