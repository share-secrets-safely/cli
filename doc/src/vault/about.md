
The `vault` sub-command is quite a complex one as it implements all interactions with **vaults**.
A **vault** contains shared secrets, and is compatible to the [unix password manager][pass].

It provides subcommands for dealing with two kinds of items

 * *resources*
 * *recipients*

[pass]: http://passwordstore.org/

### About Resources

Most of the time, when using the *vault*, you will deal with the *resources* contained
within. A *resource* is an encrypted secret so that it is readable and writable by
all *recipients*.

*Resources* can be *add*ed, *remove*d, *edit*ed, *list*ed and *show*n.

**As they are used most of the time, they are found directly in the `vault` sub-command.**

### About Recipients

Each recipient is identified by their *gpg-key*, which is tied to their identity.
New recipients can only be added by existing recipients of the vault, which also requires
them to verify that the new key truly belongs to the respective person.

Recipients may indicate trust-relationships between each other, which allows
to encrypt for recipients whose keys have not been explicitly verified.
This is called the [Web of trust][wot], a feature that `sheesy` makes easier to use.

**As they are used less often, they are tucked away in the `recipients` sub-command.**

[wot]: https://en.wikipedia.org/wiki/Web_of_trust

### The *vault* sub-command

As the `vault` sub-command is only a hub, we recommend you to look at its sub-commands
instead.

```bash,use=sy-in-path,exec
sy vault --help
```
