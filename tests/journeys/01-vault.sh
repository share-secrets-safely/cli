#!/bin/bash

set -eu
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
exe="$root/../../$exe"
# shellcheck source=./tests/utilities.sh
source "$root/../utilities.sh"

WITH_FAILURE=1
SUCCESSFULLY=0

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
  title "'vault init' - with change '--at' location"

  (with "a single gpg secret key"
    import_user "$fixture/tester.sec.asc"
  
    vault_dir=vault
    mkdir $vault_dir
    (with "a specified --at location and relative directories for keys and recipients"
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/vault-init-with-at-argument" \
        expect_run $SUCCESSFULLY "$exe" vault \
          -c $vault_dir/vault.yml \
          --vault-id customized \
          init --at secrets -k ../etc/keys -r ../etc/recipients
      }
      
      it "creates the expected folder structure" && {
        expect_snapshot "$snapshot/vault-init-change-at-location-folder-structure" .
      }
    )
    
    (when "adding a secret"
      mkdir -p $vault_dir/secrets
      it "succeeds" && {
        echo hi | expect_run $SUCCESSFULLY "$exe" vault -c $vault_dir/vault.yml add :secret
      }
      
      it "puts the file to the right spot" && {
        expect_exists $vault_dir/secrets/secret.gpg
      }
    )
  )
)

(sandboxed
  title "'vault init' - with absolute vault directory"
  subdir=location
  vault_dir=vault/$subdir
  mkdir -p $vault_dir
  vault_dir=$PWD/$vault_dir
  import_user "$fixture/tester.sec.asc"
  
  (with "a an absolute vault directory"
    it "succeeds" && {
      expect_run $SUCCESSFULLY "$exe" vault --config-file "$vault_dir/vault.yml" \
        init --gpg-keys-dir keys --recipients-file recipients
    }
    it "creates the correct folder structure" && {
      expect_snapshot "$snapshot/vault-init-single-user-absolute-directory" "$vault_dir"
    }
    (when "editing a file"
      editor="$PWD/my-simple-editor.sh"
      (
        cat <<'EDITOR' > "$editor"
#!/bin/bash -e
echo "made by simple editor" > ${1:?}
EDITOR
        chmod +x "$editor"
      )
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/vault-edit-with-absolute-vault-directory" \
        EDITOR="$editor" \
        expect_run $SUCCESSFULLY "$exe" vault -c "$vault_dir/vault.yml" \
          edit new-resource
      }
      it "creates an the newly edited encrypted file" && {
        expect_exists "$vault_dir/new-resource.gpg"
      }
    )
    ( cd "$vault_dir/.."
      echo 'content' > content
      cd "$vault_dir"
      
      it "does not add resources which walk to the parent directory" && {
        WITH_SNAPSHOT="$snapshot/vault-resource-add-relative-dir-failure" \
        expect_run $WITH_FAILURE "$exe" \
          vault -c vault.yml add ../content
      }
      
      it "does add resource which walk to the parent directory if destination is specified" && {
        WITH_SNAPSHOT="$snapshot/vault-resource-add-relative-dir-success" \
        expect_run $SUCCESSFULLY "$exe" \
          vault -c vault.yml add ../content:content
      }
      
      it "creates an the encrypted file" && {
        expect_exists content.gpg
      }
    )
  )
)

(sandboxed
  title "'vault init' - multiple gpg keys available"
  gpg --import "$fixture/tester.sec.asc" "$fixture/c.sec.asc" &>/dev/null
  
  (with "a gpg key signed by others"
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
  gpg --import "$fixture/tester.sec.asc" "$fixture/c.sec.asc" &>/dev/null

  
  (with "multiple selected gpg keys"
    it "succeeds as it just follow instructions" && {
      WITH_SNAPSHOT="$snapshot/vault-init-with-multiple-specified-keys" \
      expect_run $SUCCESSFULLY "$exe" vault init --gpg-key-id c@example.com --gpg-key-id tester@example.com
    }
    
    it "creates the expected folder structure" && {
      expect_snapshot "$snapshot/vault-init-multiple-users" .
    }
  )
)

(sandboxed 
  title "'vault add', 'show' and 'edit'"
  (with "a vault initialized for a single recipient"
    {
      import_user "$fixture/tester.sec.asc"
      "$exe" vault init
    } &> /dev/null
    
    (when "adding new resource from stdin"
      it "succeeds" && {
        echo hi | WITH_SNAPSHOT="$snapshot/vault-resource-add-from-stdin" \
        expect_run $SUCCESSFULLY "$exe" vault add :from-stdin
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
        expect_run $WITH_FAILURE "$exe" vault add :from-stdin
      }
      
      it "does not change the previous file" && {
        expect_equals "$previous_resource_id" "$(md5sum ./from-stdin.gpg)"
      }
    )
    (when "showing the previously added resource"
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/vault-resource-show" \
        expect_run $SUCCESSFULLY "$exe" vault show from-stdin
      }
    )
    (when "showing the previously added resource using the gpg suffix"
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/vault-resource-show" \
        expect_run $SUCCESSFULLY "$exe" vault show ./from-stdin.gpg
      }
    )
    (when "showing an unknown resource"
      it "fails" && {
        WITH_SNAPSHOT="$snapshot/vault-unknown-resource-show" \
        expect_run $WITH_FAILURE "$exe" vault show some-unknown-resource
      }
    )
    
    editor="$PWD/my-editor.sh"
    (
      cat <<'EDITOR' > "$editor"
#!/bin/bash -e
file_to_edit=${1:?}
echo -n "$file_to_edit" > /tmp/filepath-with-decrypted-content
echo "ho" > $file_to_edit
EDITOR
      chmod +x "$editor"
    )
    
    (when "editing a known resource"
      (with "a custom editor"
        it "succeeds" && {
          EDITOR="$editor" \
          expect_run $SUCCESSFULLY "$exe" vault edit from-stdin
        }
        it "changes the file accordingly" && {
          WITH_SNAPSHOT="$snapshot/vault-edit-changed-file" \
          expect_run $SUCCESSFULLY "$exe" vault show from-stdin.gpg
        }
        it "removes the file with decrypted content" && {
          expect_run $WITH_FAILURE test -f "$(cat /tmp/filepath-with-decrypted-content)"
        }
      )
    )
    (when "editing a new resource with a custom editor"
      it "creates the new resource" && {
        EDITOR="$editor" \
        expect_run $SUCCESSFULLY "$exe" vault edit new-edited-file
      }
      it "changes creates file accordingly" && {
        WITH_SNAPSHOT="$snapshot/vault-edit-changed-file" \
        expect_run $SUCCESSFULLY "$exe" vault show new-edited-file
      }
      it "removes the file with decrypted content" && {
        expect_run $WITH_FAILURE test -f "$(cat /tmp/filepath-with-decrypted-content)"
      }
    )
    (when "editing an unknown resource"
      it "fails" && {
        WITH_SNAPSHOT="$snapshot/vault-unknown-resource-edit" \
        expect_run $WITH_FAILURE "$exe" vault edit --no-create some-unknown-resource
      }
    )
    (with "an editor program that does not exist in path"
      it "fails" && {
        WITH_SNAPSHOT="$snapshot/vault-known-resource-edit-editor-not-in-path" \
        expect_run $WITH_FAILURE "$exe" vault edit --editor foo from-stdin
      }
    )
  )
)
(sandboxed 
  title "'vault recipient add'"
  (with "a vault initialized for a single recipient and an existing secret"
    { import_user "$fixture/tester.sec.asc"
      "$exe" vault init --at secrets --gpg-keys-dir ../etc/keys --recipients-file ../etc/recipients
      echo -n secret | "$exe" vault add :secret
    } &>/dev/null
    
    (when "trying to decrypt with an unknown gpg user"
      (as_user "$fixture/c.sec.asc"
        it "fails" && {
          WITH_SNAPSHOT="$snapshot/vault-show-failure-as-unknown-user" \
          expect_run $WITH_FAILURE "$exe" vault show secret
        }
      )
    )
    
    (when "adding a new recipient using the id of an already imported key"
      gpg --import "$fixture/c.pub.asc" &>/dev/null
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/vault-recipient-add-c" \
        expect_run $SUCCESSFULLY "$exe" vault recipients add c@example.com
      }
      
      it "sets up the metadata correctly" && {
        expect_snapshot "$snapshot/vault-recipient-add-c-keys-dir" etc
      }
      
      it "re-encrypts all secrets to allow the new recipient to decode it" && {
        (as_user "$fixture/c.sec.asc"
          WITH_SNAPSHOT="$snapshot/vault-show-success-as-user-c" \
          expect_run $SUCCESSFULLY "$exe" vault show secret
        )
      }
    )
  )
)