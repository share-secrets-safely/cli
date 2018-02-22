First of all, we assume that you have installed the `sy` command-line program already.
If not, please have a look at the [chapter about installation][install].

[install]: installation.html

```bash,use=sy-in-path,prepare=vault-dir,hide
set -eu
export SRC_DIR=$PWD
export VAULT_DIR=/tmp/share-secrets-safely-docs-vault
```
```bash,use=vault-dir,hide,exec
rm -Rf $VAULT_DIR
mkdir -p $VAULT_DIR
```
```bash,use=vault-dir,prepare=in-vault-dir,hide
cd $VAULT_DIR
```

### Initializing the vault

Assuming your current directory is empty, just running `vault` will error.

```bash,use=in-vault-dir,exec=1
sy vault
```

What you want to do is to initialize the vault. This will add yourself as the first
[recipient][recipients] and add some state to the directory you chose or that you are in.

[recipients]: vault/about.html#about-recipients

```bash,use=in-vault-dir,prepare=as-nobody,hide
GNUPGHOME="$(mktemp -t gnupg-home.XXXX -d)"
export GNUPGHOME
```

```bash,use=as-nobody,exec=1
sy vault init
```

Assuming we have followed the instructions or already have setup a *gpg key*, you will
get quite a different result.

```bash,use=in-vault-dir,prepare=as-tester,hide
source $SRC_DIR/tests/gpg-helpers.sh
as_user $SRC_DIR/tests/journeys/fixtures/tester.sec.asc
```

```bash,use=as-tester,exec
sy vault init
```

### Adding Resources

Usually what happens next is to add some [resources][resource]. For now they will
only be encrypted for you, and thus can only be read by you.

Resources are added via resource *specs*, or can be created by editing.

There are various ways to add new resources...

...from existing files.

```bash,use=as-tester,exec
echo hi | sy vault add :from-program
```

...by gathering output from a program.

```bash,use=as-tester,exec
sy vault add $SRC_DIR/README.md:README.md
```

You can *list* existing resources...
```bash,use=as-tester,exec
sy vault list
```

or you can *show* them.
```bash,use=as-tester,exec
sy vault show from-program
```

[resource]: vault/about.html#about-resources

### Meet Alexis!

Even though secrets that are not shared with anyone (but yourself) are great for
security, they are not too useful in many settings.

So we will add our first recipient, *Alexis*!

As always, *Alexis* will require an own *gpg key*, and for the sake of simplicity
we will assume it is already present.

Usually it's also easiest to let new recipients 'knock at the door of the vault',
and leave it to existing recipients of the vault to 'let them in'.

In this analogy, 'knocking the door' is equivalent to placing their key in the vaults
keys directory. 'Letting them in' means re-encrypting all *resources* for the
current and the new recipients after verifying their key truly belongs to them.

That's quite a lot to digest, so let's start and make small steps.

Let's let *Alexis* look at the vault:

```bash,use=in-vault-dir,prepare=as-alexis,hide
source $SRC_DIR/tests/gpg-helpers.sh
as_user $SRC_DIR/tests/journeys/fixtures/c.sec.asc
```

```bash,use=as-alexis,exec
sy vault
```

What a good start! We can list resource, but does that mean we can also see them?

```bash,use=as-alexis,exec=1
sy vault show from-program
```

Phew, that's good actually! It's also good that it tells you right away how to
solve the issue. Let's follow along.

```bash,use=as-alexis,exec
sy vault recipient init
```

*Cough*, let's ignore this key seems to be for *c <c@example.com>*, *Alexis* loves
simplicity!

Let's see what changed - where is this key exactly?

```bash,use=as-alexis,exec
tree -a
```

It looks like the keyfile is actually stored in a hidden directory. But as you can see,
it's just something that can be configured to your liking.

```bash,use=as-alexis,exec
cat sy-vault.yml
```

That's all we can do here, now back to the *prime recipient*.

### Adding *Alexis*

Back with the very first recipient of the vault who has already been informed
about *Alexis* arrival. We received an e-mail and know it's `c@example.com`,
maybe we can use that.

```bash,use=as-tester,exec=1
sy vault recipient add c@example.com
```

Looks like it doesn't like the format. The problem is that for verification purposes,
it wants you to add the fingerprint, which you should also have received by *Alexis*.
This links the key (identified by its fingerprint) to *Alexis*.

Let's spell it out:

```bash,use=as-tester,exec=0
sy vault recipient add 2DF04D4E
```

Let's look at the steps that it performs in details:

 * *import*
   * it finds *Alexis* key as identified by the fingerprint in the vaults keys
     directory and *imports* it into the *gpg keychain*.
 * *signing*
   * *Alexis* key is signed with the one of the prime recipient, which indicates
     we verified that it truly belongs to *Alexis*.
 * *export*
   * *Alexis* key is re-exported as it now also contains said signature. The fact
     that the prime recipient believes that the key belongs to *Alexis* is communicated
     to others that way, which helps building the [Web of Trust][wot].
 * *encrypt*
   * Each *resource* of the vault is re-encrypted for all recipients. This means
     *Alexis* will be able to get to peek inside.

*(If we would already have *Alexis* in our keychain and signed their key, you
  could also more easily add them using their email alongside the `--verified`
  flag.
  Find all possible flags of the `sy-vault-recipients-add` [here][svra])*

[svra]: vault/recipients/add.html
[wot]: https://www.gnupg.org/gph/en/manual/x547.html

### Back with Alexis, the latest recipient of the vault

Now that *Alexis* has been added as a recipient, it should be possible to peek
at the secrets it contains!

```bash,use=as-alexis,exec
sy vault show from-program
```

Beautiful!

And what's even better: *Alexis* now can add *recipients* on their own!

### Next steps

This is just the beginnings! Feel free to add more *resources* and *recipients*, add
the contents of the vault to git and distribute it that way, or add it to your tooling
to extract secrets when building your software.
