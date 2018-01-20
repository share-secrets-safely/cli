#!/bin/bash

set -eu
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../../$exe"
# shellcheck source=./tests/utilities.sh
source "$root/../gpg-helpers.sh"

WITH_FAILURE=1

fixture="$root/fixtures"
snapshot="$fixture/snapshots"

(sandboxed 
  title "'vault recipient add unverified'"
  (with "a vault initialized for a single recipient and an existing secret"
    { import_user "$fixture/tester.sec.asc"
      mkdir secrets
      "$exe" vault init --secrets-dir secrets --gpg-keys-dir ./etc/keys --recipients-file ./etc/recipients
      echo -n secret | "$exe" vault add :secret
    } &>/dev/null
    
    (with "an untrusted user requesting membership"
      (as_user "$fixture/b.sec.asc"
        "$exe" vault recipient init
      ) > /dev/null
      
      (when "adding them as recipient via fingerprint"
        it "fails with an error message indicating the GPG version doesn't support signing" && {
          WITH_SNAPSHOT="$snapshot/gpg1-vault-recipient-add-untrusted-user-with-fingerprint" \
          expect_run $WITH_FAILURE "$exe" vault recipient add DB9831D842C18D28
        }
        
        it "does not alter the meta-data structure" && {
          expect_snapshot "$snapshot/gpg1-vault-recipient-add-untrusted-user-with-fingerprint-metadata" etc
        }
      )
    )
  )
)
