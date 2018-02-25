#!/bin/bash
set -eu -o pipefail

[[ $# != 3 ]] && {
  echo 1>&2 "USAGE: $0 <tag> <homebrew-template> <homebrew-file>"
  exit 2
}

VERSION="${1:?}"
TEMPLATE_FILE="${2:?}"
HOMEBREW_FILE="${3:?}"

OSX_FILE=sy-cli-Darwin-x86_64.tar.gz
LINUX_FILE=sy-cli-linux-musl-x86_64.tar.gz

[[ -f $OSX_FILE && -f $LINUX_FILE ]] || {
    echo 1>&2 "Need both files '$OSX_FILE' and '$LINUX_FILE' to be available"
    exit 2
}

SHA_SUM=$(
  which sha256sum 2>/dev/null \
  || which gsha256sum 2>/dev/null \
  || { echo 1>&2 "sha256 program not found"; false; } \
)

OSX_SHA256="$($SHA_SUM $OSX_FILE | awk '{print $1}')"
LINUX_SHA256="$($SHA_SUM $LINUX_FILE | awk '{print $1}')"
TEMPLATE_NOTE="---> DO NOT EDIT <--- (this file was generated from $TEMPLATE_FILE"
export VERSION OSX_SHA256 LINUX_SHA256 TEMPLATE_NOTE

envsubst < "$TEMPLATE_FILE" > "$HOMEBREW_FILE"
