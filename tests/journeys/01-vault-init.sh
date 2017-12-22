#!/bin/bash

set -u
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
# shellcheck source=./tests/utilities.sh
source "$root/../utilities.sh"

WITH_FAILURE=1
SUCCESSFULLY=0

title "'vault init' - without any gpg information"

with "no available gpg key and no key" && {
    it "fails as it cannot identify the user" && \
      WITH_OUTPUT="Please create one and try again" expect_run $WITH_FAILURE "$exe" vault init
}

with "a single gpg secret key available" && {
    gpg --import "$root/fixtures/tester.sec.asc"
    it "succeeds as the key is not ambiguous" && \
      WITH_OUTPUT="vault initialized at '$PWD'" expect_run $SUCCESSFULLY "$exe" vault init 
}



