#!/bin/bash

set -u
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
# shellcheck source=./tests/utilities.sh
source "$root/../utilities.sh"

WITH_FAILURE=1

title "'vault init' - without any gpg information"

with "no available gpg key and no key" && {
    it "fails as it cannot identify the user" && \
      expect_run $WITH_FAILURE "$exe" vault init
}

