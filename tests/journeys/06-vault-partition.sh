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
snapshot="$fixture/snapshots"


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
          WITH_SNAPSHOT="$snapshot/vault-partition-add-simple" \
          expect_run $SUCCESSFULLY "$exe" vault partition add two
        }
        
        it "creates the expected vault file with another vault whose secrets dir is a sibling" && {
          expect_snapshot "$snapshot/vault-partition-simple" .
        }
      )
      
      (when "adding another named partition"
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/vault-partition-add-second-named" \
          expect_run $SUCCESSFULLY "$exe" vault partition add --name second two
        }
        
        it "creates the expected vault file" && {
          expect_snapshot "$snapshot/vault-partition-add-second-named-directory" .
        }
      )
      
      (when "adding another unnamed partition"
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/vault-partition-add-third-unnamed" \
          expect_run $SUCCESSFULLY "$exe" vault partition add subdir/third
        }
        
        it "creates the expected vault file" && {
          expect_snapshot "$snapshot/vault-partition-add-second-named-directory" .
        }
      )
    )
  )
    
  (with "an unnamed vault and the default secrets directory"
    in-space two/vault
    "$exe" vault init >/dev/null
    
    (when "adding an unnamed partition"
      it "fails as the partition is contained in the the first partition" && {
        WITH_SNAPSHOT="$snapshot/vault-partition-add-failure" \
        expect_run $WITH_FAILURE "$exe" vault partition add two
      }
    )
  )
)