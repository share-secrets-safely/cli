#!/bin/bash

set -eu
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../../$exe"
# shellcheck source=./tests/gpg-helpers.sh
source "$root/../gpg-helpers.sh"

WITH_FAILURE=1
SUCCESSFULLY=0

fixture="$root/fixtures"
snapshot="$fixture/snapshots/vault/always-encrypt"

(sandboxed
  title "'vault' init -- always encrypt"
  (with "a vault initialized for a single recipient and an existing secret and custom secrets dir and default trust model"
    { import_user "$fixture/tester.sec.asc"
      mkdir secrets
      "$exe" vault init --secrets-dir ./secrets --gpg-keys-dir ./etc/keys --recipients-file ./etc/recipients
      echo -n secret | "$exe" vault add :secret
    }  &>/dev/null
    
    precondition "the vault configuration is what we expect" && {
      expect_snapshot "$snapshot/precondition-vault-config-after-init" sy-vault.yml
    }

    (when "listing the vault content"
      it "looks as expected" && {
        WITH_SNAPSHOT="$snapshot/list-with-relative-secrets-dir" \
        expect_run $SUCCESSFULLY "$exe" vault
      }
    )

    (with "an unknown user"
      (as_user "$fixture/c.sec.asc"
        (when "trying to decrypt"
          it "fails" && {
            WITH_SNAPSHOT="$snapshot/show-failure-as-unknown-user" \
            expect_run $WITH_FAILURE "$exe" vault show secret
          }
        )
        (when "trying to encrypt a new file without a signed tester@example.com key"
          {
            gpg --import './etc/keys/D6339718E9B58FCE3C66C78AAA5B7BF150F48332'
          } &>/dev/null

          it "succeeds" && {
            echo other-secret | \
            WITH_SNAPSHOT="$snapshot/add-success-as-unknown-user" \
            expect_run $SUCCESSFULLY "$exe" vault add :new-secret
          }
          it "cannot be decrypted by yourself" && {
            WITH_SNAPSHOT="$snapshot/show-fail-for-new-secret-as-unknown-user" \
            expect_run $WITH_FAILURE "$exe" vault show new-secret
          }
          rm secrets/new-secret.gpg
        )
      )
    )
  )
)

snapshot="$fixture/snapshots/vault/recipients-and-partitions-always-encrypt"
title "'vault partitions & recipients -- always encrypt"
(sandboxed
  (with "a first user"
    import_user "$fixture/tester.sec.asc"

    (with "a vault ready for partitions and a resource"
      { "$exe" vault init --secrets-dir p1 \
                          --gpg-keys-dir etc/keys \
                          --recipients-file etc/p1
        echo 1 | "$exe" vault add :one
      } &>/dev/bull

      (with "a two more partitions"
        { "$exe" vault partition add --recipients-file etc/p2 --name second p2
          "$exe" vault partition add --recipients-file etc/p3 --name third p3
          echo 2 | "$exe" vault add :p2/two
          echo 3 | "$exe" vault add :p3/three
        } &>/dev/null

        precondition "the vault structure is what we expect" && {
          expect_snapshot "$snapshot/precondition-vault-with-multiple-partitions" etc
        }
        precondition "the vault configuration is what we expect" && {
          expect_snapshot "$snapshot/precondition-vault-with-multiple-partitions-config" sy-vault.yml
        }

        (when "impersonating another user"
          as_user "$fixture/a.sec.asc"

          (when "adding the other user as recipient choosing the partition explicitly"
            it "succeeds, even though it's the same outcome as when the partition was not chosen" && {
              WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-recipient-init" \
              expect_run $SUCCESSFULLY "$exe" vault --select p2 recipients init
            }

            it "adds the public key to the gpg-keys directory" && {
              expect_snapshot "$snapshot/vault-with-multiple-partitions-after-recipient-init" etc
            }
          )
        )

        (when "adding the new (trusted) user to both partitions by path and by name respectively"
          it "succeeds" && {
            WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-recipient-add-to-multiple" \
            expect_run $SUCCESSFULLY "$exe" vault recipients add B488BD82 --to p2 --partition third
          }
        )
        
        it "creates the correct configuration" && {
          expect_snapshot "$snapshot/vault-with-multiple-partitions-after-recipient-add" etc
        }

        (when "impersonating the newly added user and explicitly imported keys (TODO: UX issue)"
          { as_user "$fixture/a.sec.asc"
            import_user "$fixture/tester.sec.asc" 
          } &>/dev/null

          (when "adding a new resource in partition p2"
            it "succeeds" && {
              echo "hi from new one" | \
              WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-new-recipient-add-resource-to-p2" \
              expect_run $SUCCESSFULLY "$exe" vault add :p2/added-by-new-user
            }
          )
          (when "adding a new resource in partition p3"
            it "succeeds" && {
              echo "hi from new one too" | \
              WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-new-recipient-add-resource-to-p3" \
              expect_run $SUCCESSFULLY "$exe" vault add :p3/added-by-new-user
            }
          )
          (when "adding a resource in partition p1 (which is not encrypted for this user)"
            it "succeeds" && {
              echo "another new resource" | \
              WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-new-recipient-add-resource-to-p1" \
              expect_run $SUCCESSFULLY "$exe" vault add :p1/added-by-new-user
            }
          )

          (when "removing themselves from all partitions they are member of"
            (when "removing from a single partitition (p3)"
              it "succeeds" && {
                WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-new-recipient-removes-themselves-from-p3-with-import" \
                expect_run $SUCCESSFULLY "$exe" vault recipient remove a@example.com --from p3
              }
              it "writes the configuration correctly, but does not delete its key from the gpg-keys-dir as it's still used in another partition" && {
                expect_snapshot "$snapshot/vault-with-multiple-partitions-new-recipient-removes-themselves-from-p3-directory" etc
              }
            )
            (when "removing from the last remaining assigned partitition (p2)"
              it "succeeds" && {
                WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-new-recipient-removes-themselves-from-p2-with-import" \
                expect_run $SUCCESSFULLY "$exe" vault recipient remove a@example.com --partition second
              }

              it "writes the configuration correctly, and removes its key" && {
                expect_snapshot "$snapshot/vault-with-multiple-partitions-new-recipient-removes-themselves-from-p2-directory" etc
              }
            )
          )
        )
      )
    )
  )
)

(with "an invalid trust-model"
  it "fails" && {
    WITH_SNAPSHOT="$snapshot/fail-invalid-trust-model" \
    expect_run $WITH_FAILURE "$exe" vault init --trust-model=something-new
  }
)
