#!/bin/bash

set -eu
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../../$exe"
# shellcheck source=./tests/utilities.sh
source "$root/../utilities.sh"

WITH_FAILURE=1
SUCCESSFULLY=0
C_FPR=905E53FE2FC0A500100AB80B056F92A52DF04D4E
TESTER_FPR=D6339718E9B58FCE3C66C78AAA5B7BF150F48332


title "'vault init'"

(with "no available gpg key and no key" 
    it "fails as it cannot identify the user" && \
      WITH_OUTPUT="Please create one and try again" \
      expect_run $WITH_FAILURE "$exe" vault init
)

fixture="$root/fixtures"
snapshot="$fixture/snapshots"
(sandboxed
  title "'vault init' - overwrite protection"

  (with "a single gpg secret key available"
    gpg --import "$fixture/tester.sec.asc" &>/dev/null
    it "succeeds as the key is not ambiguous" && {
      WITH_SNAPSHOT="$snapshot/successful-vault-init" \
      expect_run $SUCCESSFULLY "$exe" vault init
    }
    it "creates a valid vault configuration file, \
        exports the public portion of the key to the correct spot and \
        writes the list of recipients" && {
      expect_snapshot "$snapshot/vault-init-single-user" .
    }
  )
  
  (with "an existing vault configuration file"
    it "fails as it cannot possibly overwrite anything" && {
      WITH_SNAPSHOT="$snapshot/vault-init-will-not-overwrite-vault-config" \
      expect_run $WITH_FAILURE "$exe" vault init
    }
  )
  
  (with "an existing gpg-keys directory"
    it "fails as it cannot possibly overwrite anything" && {
      WITH_SNAPSHOT="$snapshot/vault-init-will-not-write-into-existing-nonempty-directory" \
      expect_run $WITH_FAILURE "$exe" vault -c a-different-file.yml init
    }
  )
  
  (with "an existing recipients file"
    it "fails as it cannot possibly overwrite anything" && {
      WITH_SNAPSHOT="$snapshot/vault-init-will-overwrite-recipients-file" \
      expect_run $WITH_FAILURE "$exe" vault -c a-different-file-too.yml init -k some-nonexisting-directory
    }
  )
)

(sandboxed
  title "'vault init' - with absolute vault directory"
  vault_dir=vault/location
  mkdir -p $vault_dir
  vault_dir=$PWD/$vault_dir
  (with "a an absolute vault directory"
    it "succeeds" && {
      expect_run $SUCCESSFULLY "$exe" vault --config-file "$vault_dir/vault.yml" \
        init --gpg-keys-dir keys --recipients-file recipients
    }
    it "creates the correct folder structure" && {
      expect_snapshot "$snapshot/vault-init-single-user-absolute-directory" "$vault_dir"
    }
  )
)

(sandboxed
  title "'vault init' - multiple gpg keys available"
  (with "a gpg key signed by others"
    gpg --import "$fixture/c.sec.asc" &>/dev/null
    it "fails as it can't decide which gpg key to export" && {
      WITH_SNAPSHOT="$snapshot/vault-init-with-multiple-viable-keys" \
      expect_run $WITH_FAILURE "$exe" vault init
    }
    (with "a selected gpg key (and a vault name)"
      it "succeeds as it just follow instructions" && {
        WITH_SNAPSHOT="$snapshot/vault-init-with-key-specified-explicitly" \
        expect_run $SUCCESSFULLY "$exe" vault --vault-id vault-name init --gpg-key-id c@example.com
      }
      
      it "creates a valid vault configuration file, \
          exports the public portion of the selected key with signatures and \
          writes the list of recipients" && {
        expect_snapshot "$snapshot/vault-init-single-user-with-multiple-signatures" .
      }
    )
  )
)

(sandboxed
  title "'vault init' - use multiple secret keys"
  (with "a multiple selected gpg keys"
    it "succeeds as it just follow instructions" && {
      WITH_SNAPSHOT="$snapshot/vault-init-with-multiple-specified-keys" \
      expect_run $SUCCESSFULLY "$exe" vault init --gpg-key-id c@example.com --gpg-key-id tester@example.com
    }
    
    it "creates the expected folder structure" && {
      expect_snapshot "$snapshot/vault-init-multiple-users" .
    }
  )
)


function trust_key () {
  {
    gpg --export-ownertrust
    echo "${1:?}:6:"
  } | gpg --import-ownertrust
}

(sandboxed 
  title "'vault add'"
  (with "a vault initialized for a single recipient"
    ( gpg --batch --yes --delete-secret-keys $C_FPR
      gpg --batch --yes --delete-keys $C_FPR
      trust_key $TESTER_FPR
      "$exe" vault init
    ) &> /dev/null
    
    (when "adding new resource from stdin"
      it "succeeds" && {
        echo hi | WITH_SNAPSHOT="$snapshot/vault-resource-add-from-stdin" \
        expect_run $SUCCESSFULLY "$exe" vault resource add :from-stdin
      }
      
      it "creates an encrypted file" && {
        expect_exists ./from-stdin.gpg
      }
      
      it "shows the single file without gpg suffix" && {
        WITH_SNAPSHOT="$snapshot/vault-ls-with-single-resource" \
        expect_run $SUCCESSFULLY "$exe" vault ls
      }
    )
    (when "adding the same resource from stdin"
      previous_resource_id="$(md5sum ./from-stdin.gpg)"
      
      it "fails as it won't overwrite existing resources" && {
        echo hi | WITH_SNAPSHOT="$snapshot/vault-resource-add-overwrite-protection" \
        expect_run $WITH_FAILURE "$exe" vault contents add :from-stdin
      }
      
      it "does not change the previous file" && {
        expect_equals "$previous_resource_id" "$(md5sum ./from-stdin.gpg)"
      }
    )
  )
)

# TODO: add actual file to vault and assure it will always be inside of the vault