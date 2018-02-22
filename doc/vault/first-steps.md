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
