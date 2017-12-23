#!/bin/bash

set -u
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../../$exe"
# shellcheck source=./tests/utilities.sh
source "$root/../utilities.sh"

WITH_FAILURE=1
SUCCESSFULLY=0

title "'vault init' - without any gpg information"

with "no available gpg key and no key" && {
    it "fails as it cannot identify the user" && \
      WITH_OUTPUT="Please create one and try again" expect_run $WITH_FAILURE "$exe" vault init
}

fixture="$root/fixtures"
mkdir /sandbox && cd /sandbox || exit
with "a single gpg secret key available" && {
    gpg --import "$fixture/tester.sec.asc" &>/dev/null
    it "succeeds as the key is not ambiguous" && {
      expected_vault=./s3-vault.yml
      WITH_OUTPUT="vault initialized at '$expected_vault'" expect_run $SUCCESSFULLY "$exe" vault init
    }
    it "creates a valid vault configuration file" && {
      expect_match "$fixture/default-vault.yml" $expected_vault
    }
}

# TODO: - test actual content of directory and file
#       - non-empty directory
#       - specify vault by .gpg-id file



