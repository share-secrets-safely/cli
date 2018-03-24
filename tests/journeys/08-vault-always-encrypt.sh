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
(with "an invalid trust-model"
  it "fails" && {
    WITH_SNAPSHOT="$snapshot/fail-invalid-trust-model" \
    expect_run $WITH_FAILURE "$exe" vault init --trust-model=something-new
  }
)
