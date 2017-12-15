#!/bin/bash

set -u
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
# shellcheck source=./tests/utilities.sh
source "$root/utilities.sh"

WITH_FAILURE=1
SUCCESSFULLY=0

title "'vault' subcommand"

with "no existing configuration file" && {
    it "fails even though a default was defined" && \
      expect_run $WITH_FAILURE "$exe" vault
}

with "a minimal vault configuration file" && {
  it "succeeds even if there is no further argument" && \
      echo 'users:' | expect_run $SUCCESSFULLY "$exe" vault -c -
}

title "'extract' subcommand"

with "no data file to read from" && {
    it "fails" && \
      expect_run $WITH_FAILURE "$exe" extract
}



