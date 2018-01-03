#!/bin/bash

set -eu
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../../$exe"
# shellcheck source=./tests/utilities.sh
source "$root/../utilities.sh"

WITH_FAILURE=1

fixture="$root/fixtures"
snapshot="$fixture/snapshots"

title "'vault init' - without GPG"

(with "no GPG installation" 
    it "fails and provides helpful notes" && \
      WITH_SNAPSHOT="$snapshot/vault-init-no-gpg" \
      expect_run $WITH_FAILURE "$exe" vault init
)
