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
  title "vault recipients add unverified - multi-recipient-key"
  (with "a key file containing multiple recipients"
    { import_user "$fixture/tester.sec.asc"
      "$exe" vault init --gpg-keys-dir ./keys 
      cat $fixture/b.pub.asc $fixture/c.pub.asc > ./keys/7435ACDC03D55429C41637C4DB9831D842C18D28
    } >/dev/null
    
    (when "adding only one recipient from that file"
      it "fails as only one of the keys was specified" && {
        WITH_SNAPSHOT="$snapshot/vault-recipients-add-unverified-failure-with-multi-recipient-keyfile" \
        expect_run $WITH_FAILURE "$exe" vault recipients add 42C18D28
      }
      
      it "does not alter the vault at all" && {
        expect_snapshot "$snapshot/vault-recipient-add-unverified-unchanged-state" .
      }
    )
  )
)

(sandboxed 
  title "'vault recipient add unverified'"
  (with "a vault initialized for a single recipient and an existing secret"
    { import_user "$fixture/tester.sec.asc"
      mkdir secrets
      "$exe" vault init --secrets-dir secrets --gpg-keys-dir ./etc/keys --recipients-file ./etc/recipients
      echo -n secret | "$exe" vault add :secret
    } &>/dev/null
    
    (with "some invalid fingerprints and a few valid ones"
      it "won't make any change" && {
        WITH_SNAPSHOT="$snapshot/vault-recipient-add-unverified-invalid-fingerprint" \
        expect_run $WITH_FAILURE "$exe" vault recipient add something-that-is-not-a-fingerprint \
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
    
    (when "adding an unknown recipient with a valid fingerprint"
      it "fails" && {
        WITH_SNAPSHOT="$snapshot/vault-recipient-add-valid-fingerprint-key-not-present-in-keys-dir" \
        expect_run $WITH_FAILURE "$exe" vault recipient add abcabc12
      }
      
      it "does not alter any files" && {
        expect_snapshot "$snapshot/vault-recipient-add-metadata-right-after-init" ./etc
      }
    )
    
    (with "an untrusted user requesting membership"
      (as_user "$fixture/b.sec.asc"
        "$exe" vault recipient init
      ) > /dev/null
      
      (when "adding them as recipient via fingerprint"
        echo "WIP"
        exit 0
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/vault-recipient-add-untrusted-user-with-fingerprint" \
          expect_run $SUCCESSFULLY "$exe" vault recipient add DB9831D842C18D28
        }
        
        it "re-exports the public key to contain the signature" && {
          WITH_SNAPSHOT="$snapshot/vault-recipient-add-diff-gpg-list-packets-signed-key" \
          expect_run $WITH_FAILURE diff <(gpg --list-packets "$fixture/b.pub.asc") \
                                        <(gpg --list-packets etc/keys/7435ACDC03D55429C41637C4DB9831D842C18D28)
        }
        
        it "creates the expected meta-data structure" && {
          expect_snapshot "$snapshot/vault-recipient-add-untrusted-user-with-fingerprint-metadata" etc
        }
      )
    )
  )
)
