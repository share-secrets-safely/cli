#!/bin/bash

set -eu
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../../$exe"
# shellcheck source=./tests/utilities.sh
source "$root/../utilities.sh"

WITH_FAILURE=1

fixture="$root/fixtures"
snapshot="$fixture/snapshots"
cd /tmp

title "'vault init' - without GPG"

(with "no GPG installation" 
    it "fails and provides helpful notes" && {
      WITH_SNAPSHOT="$snapshot/vault-init-no-gpg" \
      expect_run $WITH_FAILURE "$exe" vault init
    }
)

(with "a valid vault configuration file"
  echo "secrets: ." > sy-vault.yml
  (with "an empty recipients file"
    touch .gpg-id
    (when "adding a new secret"
      it "fails and provides helpful notes" && {
        WITH_SNAPSHOT="$snapshot/vault-add-empty-recipients" \
        expect_run $WITH_FAILURE "$exe" vault add some:secret
      }
    )
  )
  
  (with "an non-empty recipients file"
    echo 'recipient' > .gpg-id
    (when "adding a new secret"
      it "fails and provides helpful notes" && {
        WITH_SNAPSHOT="$snapshot/vault-add-no-gpg" \
        expect_run $WITH_FAILURE "$exe" vault add some:secret
      }
    )
  )
  
  (with "a resource"
    echo 'fake secret' > secret.gpg
    (when "showing the secret"
      it "fails and provides helpful notes" && {
        WITH_SNAPSHOT="$snapshot/vault-show-no-gpg" \
        expect_run $WITH_FAILURE "$exe" vault show secret
      }
    )
  )
)
