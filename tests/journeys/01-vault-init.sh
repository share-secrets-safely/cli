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
      WITH_OUTPUT="vault initialized at './s3-vault.yml'" expect_run $SUCCESSFULLY "$exe" vault init
    }
    it "creates a valid vault configuration file, \
        exports the public portion of the key to the correct spot and \
        writes the list of recipients" && {
      expect_match "$fixture/vault-init-single-user" .
    }
}

with "an existing vault configuration file" && {
  it "fails as it cannot possibly overwrite anything" && {
    WITH_OUTPUT="Cannot.*overwrite.*s3-vault.yml.*" expect_run $WITH_FAILURE "$exe" vault init
  }
}

with "an existing gpg-keys directory" && {
  it "fails as it cannot possibly overwrite anything" && {
    WITH_OUTPUT="Cannot.*export.*keys.*\.gpg-keys" expect_run $WITH_FAILURE "$exe" vault -c a-different-file.yml init
  }
}

with "an existing recipients file" && {
  it "fails as it cannot possibly overwrite anything" && {
    WITH_OUTPUT="Cannot.*write.*\.gpg-id.*" expect_run $WITH_FAILURE "$exe" vault -c a-different-file-too.yml init -k some-nonexisting-directory
  }
}

# TODO: - assure signatures are exported too (should be, but needs test)



