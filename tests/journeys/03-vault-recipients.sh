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
    
    (with "an unknown user"
      (as_user "$fixture/c.sec.asc"
        (when "trying to decrypt"
          it "fails" && {
            WITH_SNAPSHOT="$snapshot/vault-show-failure-as-unknown-user" \
            expect_run $WITH_FAILURE "$exe" vault show secret
          }
        )
        (when "trying to encrypt a new file"
          { 
            gpg --import 'secrets/../etc/keys/D6339718E9B58FCE3C66C78AAA5B7BF150F48332'
            gpg --sign-key --yes --batch tester@example.com
          } &>/dev/null
          
          it "succeeds" && {
            echo other-secret | \
            WITH_SNAPSHOT="$snapshot/vault-add-success-as-unknown-user" \
            expect_run $SUCCESSFULLY "$exe" vault add :new-secret
          }
          it "cannot be decrypted by yourself" && {
            WITH_SNAPSHOT="$snapshot/vault-show-fail-for-new-secret-as-unknown-user" \
            expect_run $WITH_FAILURE "$exe" vault show new-secret
          }
          rm secrets/new-secret.gpg
        )
        (when "requesting membership"
          it "succeeds" && {
            WITH_SNAPSHOT="$snapshot/vault-recipients-init-single-secret-key" \
            expect_run $SUCCESSFULLY "$exe" vault recipient init
          }
          it "exports only the public key" && {
            expect_snapshot "$snapshot/vault-recipient-init-c-config-dir" etc
          }
          it "lists only the recipients already in the vaults recipients file" && {
            WITH_SNAPSHOT="$snapshot/vault-recipients-list-after-requesting-membership" \
            expect_run $SUCCESSFULLY "$exe" vault recipients 
          }
        )
      )
    )
    
    (when "adding a new recipient using the id of an already imported and signed key"
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
          
      it "lists the new recipient as well" && {
        WITH_SNAPSHOT="$snapshot/vault-recipients-list-after-adding-c-successfully" \
        expect_run $SUCCESSFULLY "$exe" vault recipients list
      }
    )
    
    (when "adding a new recipient using the id of an already imported and unsigned key"
      gpg --import "$fixture/b.pub.asc" &>/dev/null
      it "fails" && {
        WITH_SNAPSHOT="$snapshot/vault-recipient-add-b-failure" \
        expect_run $WITH_FAILURE "$exe" vault recipients add b@example.com
      }
      
      (when "signing the new key and adding the recipient"
        gpg --sign-key --yes --batch b@example.com &>/dev/null
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/vault-recipient-add-b-success-after-signing" \
          expect_run $SUCCESSFULLY "$exe" vault recipients add b@example.com
        }
        
        it "sets up the metadata correctly" && {
          expect_snapshot "$snapshot/vault-recipient-add-b-recipients" etc/recipients
        }
        
        it "re-encrypts all secrets to allow the new recipient to decode it" && {
          (as_user "$fixture/b.sec.asc"
            WITH_SNAPSHOT="$snapshot/vault-show-success-as-user-b" \
            expect_run $SUCCESSFULLY "$exe" vault show secret
          )
        }
      )
    )
    (when "missing a key to encrypt for"
      gpg --delete-key --yes --batch b@example.com
      
      it "fails as it doesn't find the required public key at all - it needs to be imported" && {
        echo ho | \
        WITH_SNAPSHOT="$snapshot/vault-show-failure-encrypt-new-secret-missing-pub-key" \
        expect_run $WITH_FAILURE "$exe" vault add :other-secret
      }
    )
  )
)
