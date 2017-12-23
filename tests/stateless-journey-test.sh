#!/bin/bash

set -u -o pipefail
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
# shellcheck source=./tests/utilities.sh
source "$root/utilities.sh"

WITH_FAILURE=1
SUCCESSFULLY=0

title "'vault' subcommand"

with "a minimal vault configuration file" && {
  it "succeeds even if there is no further argument" && \
      echo 'at: ' | expect_run $SUCCESSFULLY "$exe" vault -c -
}

title "'extract' subcommand"

with "no data file to read from" && {
    it "fails" && \
      expect_run $WITH_FAILURE "$exe" extract
}

title "'completions' subcommand"

with "with a supported $SHELL" && {
    it "generates a script executable by $SHELL" && \
      expect_run $SUCCESSFULLY "$exe" completions | $SHELL
}

with "an explicit supported shell name" && {
    it "generates a valid script" && \
      expect_run $SUCCESSFULLY "$exe" completions bash | bash
}

with "with an unsupported shell" && {
    it "fails with a suitable error" && \
    WITH_OUTPUT=".*foobar.*unsupported" expect_run $WITH_FAILURE "$exe" completions foobar
}

