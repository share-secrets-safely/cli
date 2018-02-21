First of all, we assume that you have installed the `sy` command-line program already.
If not, please have a look at the [chapter about installation][install].

[install]: installation.html

```bash,use=sy-in-path,prepare=vault-dir,hide
set -eu
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

```bash,use=in-vault-dir,exec=1
sy vault init
```

Assuming we have followed the instructions or already have setup a *gpg key*, you will
get quite a different result.

```bash,prepare=as-tester,hide
source tests/gpg-helpers.sh
as_user tests/journeys/fixtures/tester.sec.asc
```

```bash,use=in-vault-dir,use=as-tester,exec
sy vault init
```
