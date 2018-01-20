#!/bin/bash

set -eu -o pipefail
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
# shellcheck source=./tests/gpg-helpers.sh
source "$root/gpg-helpers.sh"

WITH_FAILURE=1
SUCCESSFULLY=0

title "'vault' subcommand"
snapshot="$root/journeys/fixtures/snapshots/stateless"

(with "a minimal vault configuration file"
  it "succeeds even if there is no further argument" && \
      echo 'secrets: .' | expect_run $SUCCESSFULLY "$exe" vault -c -
)

title "'vault init' subcommand"

(with "an invalid vault path"
  it "fails" && \
      WITH_SNAPSHOT="$snapshot/invalid-vault-path" expect_run $WITH_FAILURE "$exe" vault -c / init
)

title "'extract' subcommand"

(with "no data file to read from"
    it "fails" && \
      WITH_SNAPSHOT="$snapshot/no-data-file" expect_run $WITH_FAILURE "$exe" extract
)

title "'completions' subcommand"

(with "a supported $SHELL"
    it "generates a script executable by $SHELL" && \
      expect_run $SUCCESSFULLY "$exe" completions | $SHELL
)

(with "an explicit supported shell name"
    it "generates a valid script" && \
      expect_run $SUCCESSFULLY "$exe" completions bash | bash
)

(with "an unsupported shell"
    it "fails with a suitable error" && \
    WITH_SNAPSHOT="$snapshot/unsupported-shell" expect_run $WITH_FAILURE "$exe" completions foobar
)

