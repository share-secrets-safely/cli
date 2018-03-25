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
snapshot="$fixture/snapshots/vault/recipients-and-partitions"

title "'vault partitions & recipients"
(sandboxed
  (with "a first user"
    import_user "$fixture/tester.sec.asc"

    (with "a vault ready for partitions and a resource"
      { "$exe" vault init --secrets-dir p1 \
                          --gpg-keys-dir etc/keys \
                          --trust-model web-of-trust \
                          --no-auto-import \
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

        (when "adding the new (trusted) user a partition does not exist"
          it "fails" && {
            WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-recipient-add-to-unknown" \
            expect_run $WITH_FAILURE "$exe" vault recipients add a@example.com --to unknown
          }
        )

        (when "adding the new (trusted) user to both partitions by path and by name respectively"
          it "succeeds" && {
            WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-recipient-add-to-multiple" \
            expect_run $SUCCESSFULLY "$exe" vault recipients add B488BD82 --to p2 --partition third
          }

          it "creates the correct configuration" && {
            expect_snapshot "$snapshot/vault-with-multiple-partitions-after-recipient-add" etc
          }

          (when "impersonating the newly added user"
            as_user "$fixture/a.sec.asc"

            (when "showing the resource in partition p2"
              it "succeeds" && {
                WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-new-recipient-show-resource-in-p2" \
                expect_run $SUCCESSFULLY "$exe" vault show p2/two
              }
            )
            (when "showing the resource in partition p3"
              it "succeeds" && {
                WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-new-recipient-show-resource-in-p3" \
                expect_run $SUCCESSFULLY "$exe" vault show p3/three
              }
            )
            (when "showing the resource in partition p1 (which is not encrypted for this user)"
              it "fails" && {
                WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-new-recipient-show-resource-in-p1" \
                expect_run $WITH_FAILURE "$exe" vault show p1/one
              }
            )

            (when "removing themselves from all partitions they are member of"
              (with "not explicitly imported (missing) keys"
                it "fails" && {
                  WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-new-recipient-removes-themselves-from-p3-no-import" \
                  expect_run $WITH_FAILURE "$exe" vault recipient remove a@example.com --from p3
                }
                it "does not alter the configuration" && {
                  expect_snapshot "$snapshot/vault-with-multiple-partitions-new-recipient-removes-themselves-from-p3-no-import-directory" etc
                }
              )
              (when "removing themselves from a partition they are not member of"
                it "fails" && {
                  WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-new-recipient-removes-themselves-from-p1-fail" \
                  expect_run $WITH_FAILURE "$exe" vault recipient remove a@example.com --from p1
                }
                it "does not alter the configuration" && {
                  expect_snapshot "$snapshot/vault-with-multiple-partitions-new-recipient-removes-themselves-from-p1-fail-directory" etc
                }
              )
              (with "explicitly imported keys as auto-import is off"
                { import_user "$fixture/tester.sec.asc"
                  gpg --sign-key --yes --batch tester@example.com
                } &>/dev/null

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

        (when "removing themselves from the a partition so it is empty"
          it "fails" && {
            WITH_SNAPSHOT="$snapshot/vault-with-multiple-partitions-remove-oneself-from-p3-fails" \
            expect_run $WITH_FAILURE "$exe" vault recipient remove tester@example.com --from p3
          }
          it "did not alter the configuration as one cannot remove oneself if one is the last recipient" && {
            expect_snapshot "$snapshot/vault-with-multiple-partitions-new-recipient-remove-oneself-from-p3-fails-directory" etc
          }
        )
      )
    )
  )
)
