#!/bin/bash

set -eu
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../../$exe"
# shellcheck source=./tests/gpg-helpers.sh
source "$root/../gpg-helpers.sh"

WITH_FAILURE=1
SUCCESSFULLY=0

fixture="$root/fixtures"
snapshot="$fixture/snapshots/vault/partitions/add"


title "'vault partition add'"
(sandboxed
  (with "a single user"
    import_user "$fixture/tester.sec.asc"
    init_args=( -k etc/keys -r etc/recipients )
      
    (with "an unnamed vault with an explicit secrets directory"
      in-space one/vault
      "$exe" vault init "${init_args[@]}" --secrets-dir ../one >/dev/null
      
      (when "adding an unnnamed partition"
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/partition-first-unnamed" \
          expect_run $SUCCESSFULLY "$exe" vault partition add first
        }
        
        it "creates the expected vault file with another vault whose secrets dir is a sibling" && {
          expect_snapshot "$snapshot/partition-first-unnamed-directory" .
        }
      )
      
      SHARED_NAME=second-partition
      (when "adding another named partition"
        it "succeeds and defaults the name to the path" && {
          WITH_SNAPSHOT="$snapshot/partition-add-second-named" \
          expect_run $SUCCESSFULLY "$exe" vault partition add --name $SHARED_NAME second
        }
        
        it "creates the expected vault file" && {
          expect_snapshot "$snapshot/partition-add-second-named-directory" .
        }
      )
      
      (when "adding another unnamed partition"
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/partition-add-third-unnamed-relative-path" \
          expect_run $SUCCESSFULLY "$exe" vault partition add subdir/third
        }
        
        it "creates the expected vault file" && {
          expect_snapshot "$snapshot/partition-add-third-named-directory" .
        }
      )
      
      (when "adding a partition with a name that was already used and an absolute path"
        ABS_PATH=/tmp/some-empty-dir
        mkdir -p $ABS_PATH
        
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/partition-add-fourth-named-absolute-path" \
          expect_run $SUCCESSFULLY "$exe" vault partition add --name $SHARED_NAME "$ABS_PATH"
        }
        
        it "creates the expected vault file" && {
          expect_snapshot "$snapshot/partition-add-fourth-directory" .
        }
      )
    )
    
    (with "an unnamed vault and the default secrets directory"
      in-space two/vault
      "$exe" vault init --secrets-dir . >/dev/null
      
      (when "adding an unnamed partition"
        it "fails as the partition is contained in the the first partition" && {
          WITH_SNAPSHOT="$snapshot/partition-add-failure" \
          expect_run $WITH_FAILURE "$exe" vault partition add two
        }
      )
    )
    (with "a vault with a custom recipients file that exists in another partition"
      in-space three/vault
      cat <<'YAML' > sy-vault.yml
---
secrets: "one"
gpg_keys: ".gpg-keys"
recipients: ".gpg-id"
---
secrets: "two"
gpg_keys: ".gpg-keys"
recipients: ".gpg-id"
YAML
        it "fails as recipients files must be unique" && {
          WITH_SNAPSHOT="$snapshot/partition-add-failure-duplicate-recipients" \
          expect_run $WITH_FAILURE "$exe" vault partition add any
        }
    )
  )
)