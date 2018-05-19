#!/bin/bash

set -eu

root="$(cd "${0%/*}" && pwd)"
# shellcheck source=./tests/utilities.sh
source "$root/../utilities.sh"

SUCCESSFULLY=0

fixture="$root/fixtures"
snapshot="$fixture/snapshots/getting-started"

(sandbox
  title "getting started"
  (with "a standard alpine image"
    (with "a clone of the getting-started repository"
      {
        git clone https://github.com/share-secrets-safely/getting-started /getting-started
        cd /getting-started/
      } > /dev/null

      (when "executing one of the wrappers"
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/syv-output-on-alpine" \
          expect_run $SUCCESSFULLY ./syp a=42
        }
      )
    )
  )
)
