# The Vault

The `vault` sub-command is quite a complex one as it contains all interactions with **vaults**.
A **vault** contains shared secrets, and is compatible to the [unix password manager][pass].

[pass]: http://passwordstore.org/

```bash,prepare=sy-in-path,hide
set -eu
export PATH="/volume/${EXE_PATH%/*}:$PATH"
```

```bash,use=sy-in-path,exec
sy vault --help
```

TBD About
