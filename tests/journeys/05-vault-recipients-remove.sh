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
snapshot="$fixture/snapshots/vault/recipients/remove"

(sandboxed
  title "vault recipient remove"
  (with "a vault with multiple secrets and multiple recipients"
    { import_user "$fixture/tester.sec.asc"
      "$exe" vault init --trust-model=web-of-trust --no-auto-import --gpg-keys-dir ./keys
      echo a | "$exe" vault add :a
      echo b | "$exe" vault add :subdir/b
      gpg --import "$fixture/b.pub.asc" 2>&1
      "$exe" vault recipient add 42C18D28
    } > /dev/null

    RECIPIENTS_KEY_FILE=./keys/7435ACDC03D55429C41637C4DB9831D842C18D28
    precondition "the recipient can show both secrets" && (
      as_user "$fixture/b.sec.asc"
      "$exe" vault show a
      "$exe" vault show subdir/b
      test -f "$RECIPIENTS_KEY_FILE"
      echo 1>&2
    ) >/dev/null

    (when "the recipient to remove does not exist in the vault but in the keychain"
      gpg --import "$fixture/c.pub.asc" &>/dev/null
      it "fails" && {
        WITH_SNAPSHOT="$snapshot/recipient-remove-failure-just-in-keychain" \
        expect_run $WITH_FAILURE "$exe" vault recipient remove c@example.com
      }
    )

    (when "the recipient to remove does not exist in the vault and not in the keychain"
      it "fails" && {
        WITH_SNAPSHOT="$snapshot/recipient-remove-failure-unknown-key" \
        expect_run $WITH_FAILURE "$exe" vault recipient remove unkown@example.com
      }
    )

    (with "one recipient to remove which exists in the vault and another that does not"
      it "fails" && {
        WITH_SNAPSHOT="$snapshot/recipient-remove-failure-one-correct-and-one-unknown-key" \
        expect_run $WITH_FAILURE "$exe" vault recipient remove b@example.com unkown@example.com
      }

      it "does not alter the vault at all" && (
        as_user "$fixture/b.sec.asc"
        expect_run $SUCCESSFULLY "$exe" vault show a
      )
    )

    (when "the recipient to remove does exist in the vault"
      keys_list_before_removal="$(gpg --list-keys)"

      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/recipient-remove-success" \
        expect_run $SUCCESSFULLY "$exe" vault recipient remove b@example.com
      }

      it "prevents the removed recipient from seeing the secrets" && (
        as_user "$fixture/b.sec.asc"
        WITH_SNAPSHOT="$snapshot/recipient-after-remove-show-failure" \
        expect_run $WITH_FAILURE "$exe" vault show subdir/b
      )

      it "removes the recipients key from the vaults key directory" && {
        expect_run $WITH_FAILURE test -f ./keys/7435ACDC03D55429C41637C4DB9831D842C18D28
      }

      it "does not alter the gpg keychain at all" && {
        keys_after_removal="$(gpg --list-keys)"
        expect_run $SUCCESSFULLY diff <(echo "$keys_list_before_removal") <(echo "$keys_after_removal")
      }
    )
  )
)
