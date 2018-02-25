#!/bin/bash

set -eu
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../../$exe"
# shellcheck source=./tests/gpg-helpers.sh
source "$root/../gpg-helpers.sh"

WITH_FAILURE=1

fixture="$root/fixtures"
snapshot="$fixture/snapshots/vault/init-no-gpg-installed"
cd /tmp

title "'vault init' - without GPG"

(with "no GPG installation"
    it "fails and provides helpful notes" && {
      WITH_SNAPSHOT="$snapshot/init-no-gpg" \
      expect_run $WITH_FAILURE "$exe" vault init
    }
)

(with "a valid vault configuration file"
  echo "secrets: ." > sy-vault.yml
  (with "an empty recipients file"
    touch .gpg-id
    (when "adding a new secret"
      it "fails and provides helpful notes" && {
        WITH_SNAPSHOT="$snapshot/add-empty-recipients" \
        expect_run $WITH_FAILURE "$exe" vault add some:secret
      }
    )
  )

  (with "an non-empty recipients file"
    echo 'recipient' > .gpg-id
    (when "adding a new secret"
      it "fails and provides helpful notes" && {
        WITH_SNAPSHOT="$snapshot/add-no-gpg" \
        expect_run $WITH_FAILURE "$exe" vault add some:secret
      }
    )
  )

  (with "a resource"
    echo 'fake secret' > secret.gpg
    (when "showing the secret"
      it "fails and provides helpful notes" && {
        WITH_SNAPSHOT="$snapshot/show-no-gpg" \
        expect_run $WITH_FAILURE "$exe" vault show secret
      }
    )
  )
)
