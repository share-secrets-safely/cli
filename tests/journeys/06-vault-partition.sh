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
snapshot="$fixture/snapshots/vault/partitions"


title "'vault partition add & remove'"
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
          expect_run $SUCCESSFULLY "$exe" vault partitions insert --name $SHARED_NAME "$ABS_PATH"
        }
        
        it "creates the expected vault file" && {
          expect_snapshot "$snapshot/partition-add-fourth-directory" .
        }
      )
      
      (when "removing an existing partition by index"
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/partition-remove-fourth-by-index" \
          expect_run $SUCCESSFULLY "$exe" vault partition remove 4
        }
        
        it "creates the expected vault file" && {
          expect_snapshot "$snapshot/partition-remove-fourth-by-index-directory" .
        }
      )
      
      (when "removing an existing partition by name"
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/partition-remove-third-by-name" \
          expect_run $SUCCESSFULLY "$exe" vault partition remove third
        }
        
        it "creates the expected vault file" && {
          expect_snapshot "$snapshot/partition-remove-third-by-name-directory" .
        }
      )
      
      (when "removing an existing partition by resource directory"
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/partition-remove-second-by-resource-dir" \
          expect_run $SUCCESSFULLY "$exe" vault partition remove ../second
        }
        
        it "creates the expected vault file" && {
          expect_snapshot "$snapshot/partition-remove-second-by-resource-dir-directory" .
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
      
      (when "removing a partition by path that does not exist"
        it "fails" && {
          WITH_SNAPSHOT="$snapshot/partition-remove-by-path-failure-does-not-exist" \
          expect_run $WITH_FAILURE "$exe" vault partition remove ../some-path-which-is-not-there
        }
      )
      
      (when "removing a partition by index that does not exist"
        it "fails" && {
          WITH_SNAPSHOT="$snapshot/partition-remove-by-index-failure-does-not-exist" \
          expect_run $WITH_FAILURE "$exe" vault partitions delete 99
        }
      )
      
      (when "removing a partition by name that does not exist"
        it "fails" && {
          WITH_SNAPSHOT="$snapshot/partition-remove-by-name-failure-does-not-exist" \
          expect_run $WITH_FAILURE "$exe" vault partition remove some-invalid-name
        }
      )
      
      (when "removing a partition by index that does exist but is the leading vault"
        it "fails" && {
          WITH_SNAPSHOT="$snapshot/partition-remove-by-index-failure-cannot-remove-leader" \
          expect_run $WITH_FAILURE "$exe" vault partition remove 0
        }
      )
    )
  )
  
  (with "a vault with non-unique recipients file configuration across two partition"
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
  
  (with "a vault with non-unique names across two partitions"
    in-space four/vault
    cat <<'YAML' > sy-vault.yml
---
secrets: "one"
name: same
gpg_keys: ".gpg-keys"
recipients: "bar"
---
secrets: "two"
name: same
recipients: "foo"
---
secrets: "three"
name: same
recipients: "baz"
YAML
    (when "removing a partition by name"
      it "fails as the name is ambiguous - there are two partitions" && {
        WITH_SNAPSHOT="$snapshot/partition-remove-failure-ambiguous-name" \
        expect_run $WITH_FAILURE "$exe" vault partition remove same
      }
    )
  )
  (with "a vault with non-unique names across a single partition and the leader"
    in-space five/vault
    cat <<'YAML' > sy-vault.yml
---
secrets: "one"
name: same
gpg_keys: ".gpg-keys"
recipients: "bar"
---
secrets: "two"
name: same
recipients: "foo"
YAML
    (when "removing a partition by name"
      it "succeeds as the leader is not even looked at" && {
        WITH_SNAPSHOT="$snapshot/partition-remove-success-similarly-named-leader" \
        expect_run $SUCCESSFULLY "$exe" vault partition remove same
      }
      it "creates the expected vault file" && {
        expect_snapshot "$snapshot/partition-remove-success-similarly-named-leader-directory" .
      }
    )
  )
)