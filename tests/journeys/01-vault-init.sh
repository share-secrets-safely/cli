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
(sandboxed && {
  with "a single gpg secret key available" && {
    gpg --import "$fixture/tester.sec.asc" &>/dev/null
    it "succeeds as the key is not ambiguous" && {
      WITH_OUTPUT="vault initialized at './s3-vault.yml'" expect_run $SUCCESSFULLY "$exe" vault init
    }
    it "creates a valid vault configuration file, \
        exports the public portion of the key to the correct spot and \
        writes the list of recipients" && {
      expect_snapshot "$fixture/vault-init-single-user" .
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
})

(sandboxed && {
  with "a gpg key signed by others" && {
    gpg --import "$fixture/c.sec.asc" &>/dev/null
    it "fails as it can't decide which gpg key to export" && {
      WITH_OUTPUT="Found 2 viable keys for key-ids" expect_run $WITH_FAILURE "$exe" vault init
    }
    with "a selected gpg key" && {
      it "succeeds as it just follow instructions" && {
        WITH_OUTPUT="vault initialized at './s3-vault.yml'" expect_run $SUCCESSFULLY "$exe" vault init --gpg-key-id c@example.com
      }
      
      it "creates a valid vault configuration file, \
          exports the public portion of the selected key with signatures and \
          writes the list of recipients" && {
        expect_snapshot "$fixture/vault-init-single-user-with-multiple-signatures" .
      }
    }
  }
})

(sandboxed && {
  with "a multiple selected gpg keys" && {
    it "succeeds as it just follow instructions" && {
      WITH_OUTPUT="vault initialized at './s3-vault.yml'" expect_run $SUCCESSFULLY "$exe" vault init --gpg-key-id c@example.com --gpg-key-id tester@example.com
    }
    
    it "creates the expected folder structure" && {
      expect_snapshot "$fixture/vault-init-multiple-users" .
    }
  }
})
