You might ask yourself why you would chose `sheesy` over other tools. The comparisons
that follow should be helpful in deciding what's best for you.

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
