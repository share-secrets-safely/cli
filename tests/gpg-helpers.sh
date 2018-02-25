#!/bin/bash

# shellcheck disable=1090
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/utilities.sh"

function in-space () {
  VAULT="${1:?}"
  [[ -d $VAULT ]] && {
    echo 1>&2 "'$VAULT' directory is already used. Please chose a different name."
    return 2
  }
  { mkdir -p "$VAULT" && cd "$VAULT"; } || return 2
}

function trust_key () {
  {
    gpg --export-ownertrust
    echo "${1:?First argument is the long fingerprint of the key to trust}:6:"
  } | gpg --import-ownertrust &>/dev/null
}

function import_user () {
  local key=${1:?First argument must be the keyfile identifying the user}

  gpg --import --yes --batch "$key" &>/dev/null

  local fpr
  fpr="$(gpg --list-secret-keys --with-colons --with-fingerprint | grep fpr | head -1)"
  fpr=${fpr:12:40}
  trust_key "$fpr"
}

function as_user () {
  local key=${1:?First argument must be the keyfile identifying the user}
  GNUPGHOME="$(mktemp -t gnupg-home.XXXX -d)"
  export GNUPGHOME

  import_user "$key" &>/dev/null
}

function gpg_sandbox () {
  GNUPGHOME="$(mktemp -t gnupg-home.XXXX -d)"
  export GNUPGHOME
}

function sandboxed () {
  sandbox "gpg_sandbox"
}
