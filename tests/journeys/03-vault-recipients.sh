#!/bin/bash

set -eu
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../../$exe"
# shellcheck source=./tests/utilities.sh
source "$root/../utilities.sh"

WITH_FAILURE=1
SUCCESSFULLY=0

fixture="$root/fixtures"
snapshot="$fixture/snapshots"
(sandboxed 
  title "'vault recipient add'"
  (with "a vault initialized for a single recipient and an existing secret"
    { import_user "$fixture/tester.sec.asc"
      "$exe" vault init --at secrets --gpg-keys-dir ../etc/keys --recipients-file ../etc/recipients
      echo -n secret | "$exe" vault add :secret
    } &>/dev/null
    
    (when "trying to decrypt with an unknown gpg user"
      (as_user "$fixture/c.sec.asc"
        it "fails" && {
          WITH_SNAPSHOT="$snapshot/vault-show-failure-as-unknown-user" \
          expect_run $WITH_FAILURE "$exe" vault show secret
        }
      )
    )
    
    (when "adding a new recipient using the id of an already imported key"
      gpg --import "$fixture/c.pub.asc" &>/dev/null
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/vault-recipient-add-c" \
        expect_run $SUCCESSFULLY "$exe" vault recipients add c@example.com
      }
      
      it "sets up the metadata correctly" && {
        expect_snapshot "$snapshot/vault-recipient-add-c-keys-dir" etc
      }
      
      it "re-encrypts all secrets to allow the new recipient to decode it" && {
        (as_user "$fixture/c.sec.asc"
          WITH_SNAPSHOT="$snapshot/vault-show-success-as-user-c" \
          expect_run $SUCCESSFULLY "$exe" vault show secret
        )
      }
    )
  )
)
