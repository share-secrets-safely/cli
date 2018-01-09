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
(sandboxed 
  title "'vault recipient add unverified'"
  (with "a vault initialized for a single recipient and an existing secret"
    { import_user "$fixture/tester.sec.asc"
      "$exe" vault init --at secrets --gpg-keys-dir ../etc/keys --recipients-file ../etc/recipients
      echo -n secret | "$exe" vault add :secret
    } &>/dev/null
    
    (with "some invalid fingerprints and a few valid ones"
      it "won't make any change" && {
        WITH_SNAPSHOT="$snapshot/vault-recipient-add-unverified-invalid-fingerprint" \
        expect_run $WITH_FAILURE $exe vault recipient add something-that-is-not-a-fingerprint \
            also-invalid \
            abc \
            abc1f7d1 \
            2CF6E0B51AAF73F09B1C21174D1DA68C88710E60ffffffff \
            2CF6E0B51AAF73F09B1C21174D1DA68C88710E60 \
            1AAF73F09B1C21174D1DA68C88710E60 \
            9B1C21174D1DA68C88710E60 \
            4D1DA68C88710E60 \
            88710E60
      }

      it "does not alter any files" && {
        expect_snapshot "$snapshot/vault-recipient-add-metadata-right-after-init" ./etc
      }
    )
  )
)
