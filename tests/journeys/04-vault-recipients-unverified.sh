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
        precondition "b@example.com did not have the prime members signature yet" && {
          expect_run_sh $WITH_FAILURE "gpg --list-packets "$fixture/b.pub.asc" | grep -q 'issuer key ID AA5B7BF150F48332'"
        }
        "$exe" vault recipient init
      ) > /dev/null
      
      (when "adding them as recipient via fingerprint"
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/vault-recipient-add-untrusted-user-with-fingerprint" \
          expect_run $SUCCESSFULLY "$exe" vault recipient add DB9831D842C18D28
        }
        
        it "signs the new recipient with prime members key and exports the key" && {
          expect_run_sh $SUCCESSFULLY "gpg --list-packets etc/keys/7435ACDC03D55429C41637C4DB9831D842C18D28 | grep -q 'issuer key ID AA5B7BF150F48332'"
        }
        
        it "adds the new recipients to the recipients file" && {
          expect_snapshot "$snapshot/vault-recipient-add-untrusted-user-with-fingerprint-metadata" etc/recipients
        }
      )
    )
    
    (with "another usable secret key which is not a vault recipient"
      import_user "$fixture/a.sec.asc"
      
      (when "re-adding the new recipient via fingerprint"
        it "succeeds as it finds a signing key non-ambiguously" && {
          WITH_SNAPSHOT="$snapshot/vault-recipient-add-untrusted-user-with-fingerprint" \
          expect_run $SUCCESSFULLY "$exe" vault recipient add DB9831D842C18D28
        }
        
        it "signs the new recipient with prime members key and exports the key" && {
          expect_run_sh $SUCCESSFULLY "gpg --list-packets etc/keys/7435ACDC03D55429C41637C4DB9831D842C18D28 | grep -q 'issuer key ID AA5B7BF150F48332'"
        }
        
        it "adds the new recipients to the recipients file" && {
          expect_snapshot "$snapshot/vault-recipient-add-untrusted-user-with-fingerprint-metadata" etc/recipients
        }
      )
    )
    
    (with "two usable signing keys"
      ( as_user "$fixture/a.sec.asc"
        "$exe" vault recipient init a@example.com
      ) >/dev/null
      
      (when "adding the new recipient"
        it "succeeds as there still is only one secret key which is also in recipients" && {
          WITH_SNAPSHOT="$snapshot/vault-recipient-add-untrusted-user-a-with-fingerprint" \
          expect_run $SUCCESSFULLY "$exe" vault recipient add EF17047AB488BD82
        }
      )
      
      (when "adding the new recipient again"
        it "succeeds as it takes the first viable signing key" && {
          WITH_SNAPSHOT="$snapshot/vault-recipient-add-untrusted-user-a-with-fingerprint" \
          expect_run $SUCCESSFULLY "$exe" vault recipient add EF17047AB488BD82
        } && shortcoming "it should be possible to select the signing key in case of ambiguity"
      )
    )
  )
)
