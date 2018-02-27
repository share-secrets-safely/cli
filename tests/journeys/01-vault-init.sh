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
snapshot="$fixture/snapshots/vault/init"

title "'vault init'"

(with "no available gpg key and no key"
    it "fails as it cannot identify the user" && \
      WITH_SNAPSHOT="$snapshot/init-failure-no-secret-key" \
      expect_run $WITH_FAILURE "$exe" vault init
)

(sandboxed
  title "'vault init' - overwrite protection"

  (with "a single gpg secret key available"
    gpg --import "$fixture/tester.sec.asc" &>/dev/null
    it "succeeds as the key is not ambiguous" && {
      WITH_SNAPSHOT="$snapshot/successful-init" \
      expect_run $SUCCESSFULLY "$exe" vault init
    }
    it "creates a valid vault configuration file, \
        exports the public portion of the key to the correct spot and \
        writes the list of recipients" && {
      expect_snapshot "$snapshot/init-single-user" .
    }
  )

  (with "an existing vault configuration file"
    it "fails as it cannot possibly overwrite anything" && {
      WITH_SNAPSHOT="$snapshot/init-will-not-overwrite-vault-config" \
      expect_run $WITH_FAILURE "$exe" vault init
    }
  )

  (with "an existing gpg-keys directory"
    it "fails as it cannot possibly overwrite anything" && {
      WITH_SNAPSHOT="$snapshot/init-will-not-write-into-existing-nonempty-directory" \
      expect_run $WITH_FAILURE "$exe" vault -c a-different-file.yml init
    }
  )

  (with "an existing recipients file"
    it "fails as it cannot possibly overwrite anything" && {
      WITH_SNAPSHOT="$snapshot/init-will-overwrite-recipients-file" \
      expect_run $WITH_FAILURE "$exe" vault -c a-different-file-too.yml init -k some-nonexisting-directory
    }
  )
)

(sandboxed
  title "'vault init' without vault description"

  (with "a single gpg user"
    import_user "$fixture/tester.sec.asc"

    (with "pass-compatible gpg-id file in the current directory"
      echo D6339718E9B58FCE3C66C78AAA5B7BF150F48332 > .gpg-id

      (when "adding a new secret"
        it "succeeds" && {
          echo hi | \
          WITH_SNAPSHOT="$snapshot/add-without-vault-description" \
          expect_run $SUCCESSFULLY "$exe" vault add :resource
        }

        it "creates the encrypted resource file" && {
          expect_exists resource.gpg
        }
      )
    )

    (with "pass-compatible gpg-id file in a nested directory"
      DIR=some/sub/directory
      mkdir -p $DIR
      echo D6339718E9B58FCE3C66C78AAA5B7BF150F48332 > $DIR/.gpg-id

      (when "adding a new secret"
        (with "the a non-existing vault configuration file in the correct sub directory"
          it "succeeds" && {
            echo hi | \
            WITH_SNAPSHOT="$snapshot/add-without-vault-description" \
            expect_run $SUCCESSFULLY "$exe" vault --config-file "$DIR/non-existing.yml" add :resource
          }

          it "creates the encrypted resource file" && {
            expect_exists "$DIR/resource.gpg"
          }
        )
      )
    )
  )
)

(sandboxed
  title "'vault init' and pre-empting future partitions"

  (with "a single gpg user"
    import_user "$fixture/tester.sec.asc"

    (when "initializing the vault with a resources directory and without explicit recipients file"
      in-space one
      it "succeeds" && {
        expect_run $SUCCESSFULLY "$exe" vault init --secrets-dir ./secrets
      }

      it "creates the recipients file within the secrets dir to be more suitable for partitions" && {
        expect_snapshot "$snapshot/init-relative-unnamed-recipients" .
      }
    )

    (when "initializing the vault with a resources directory and with an explicit recipients file name"
      in-space two
      it "succeeds" && {
        expect_run $SUCCESSFULLY "$exe" vault init --secrets-dir ./secrets --recipients-file recipients
      }

      it "creates the recipients file within the secrets dir to be more suitable for partitions" && {
        expect_snapshot "$snapshot/init-relative-named-recipients" .
      }
    )

    (when "initializing the vault with a resources directory and with an explicit recipients path"
      in-space three
      it "succeeds" && {
        expect_run $SUCCESSFULLY "$exe" vault init --secrets-dir ./secrets --recipients-file ./recipients
      }

      it "creates the recipients file at that path" && {
        expect_snapshot "$snapshot/init-explicit-recipients-path" .
      }
    )
  )
)

(sandboxed
  title "'vault init' - with change '--secrets-dir' location"

  (with "a single gpg secret key"
    import_user "$fixture/tester.sec.asc"

    vault_dir=vault
    mkdir $vault_dir
    (with "a specified --secrets-dir location and relative directories for keys and recipients"
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/init-with-at-argument" \
        expect_run $SUCCESSFULLY "$exe" vault \
          -c $vault_dir/vault.yml \
          init --name customized --secrets-dir secrets -k ./etc/keys -r ./etc/recipients
      }

      it "creates the expected folder structure" && {
        expect_snapshot "$snapshot/init-change-secrets-location-folder-structure" $vault_dir/etc
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

      it "lists the content as expected" && {
        WITH_SNAPSHOT="$snapshot/list-changed-secrets-location" \
        expect_run $SUCCESSFULLY "$exe" vault -c $vault_dir/vault.yml list
      }
    )

    (when "adding a secret in a subdirectory"
      it "succeeds" && {
        echo with-sub-dirs | \
        WITH_SNAPSHOT="$snapshot/add-with-subdirectory" \
        expect_run $SUCCESSFULLY "$exe" vault -c $vault_dir/vault.yml add :partition/subdir/secret
      }

      it "creates the encrypted secret file in the correct location" && {
        expect_exists "$vault_dir/secrets/partition/subdir/secret.gpg"
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

  (with "a an absolute vault directory (and a custom name)"
    it "succeeds" && {
      expect_run $SUCCESSFULLY "$exe" vault --config-file "$vault_dir/vault.yml" \
        init -n custom-name --gpg-keys-dir keys --recipients-file recipients
    }
    it "creates the correct folder structure" && {
      expect_snapshot "$snapshot/init-single-user-absolute-directory" "$vault_dir"
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
        WITH_SNAPSHOT="$snapshot/edit-with-absolute-vault-directory" \
        EDITOR="$editor" \
        expect_run $SUCCESSFULLY "$exe" vault -c "$vault_dir/vault.yml" \
          edit new-resource
      }
      it "creates an the newly edited encrypted file" && {
        expect_exists "$vault_dir/new-resource.gpg"
      }
      ( cd "$vault_dir/.."
        echo 'content' > content
        cd "$vault_dir"

        it "does not add resources which walk to the parent directory" && {
          WITH_SNAPSHOT="$snapshot/resource-add-relative-dir-failure" \
          expect_run $WITH_FAILURE "$exe" \
            vault -c vault.yml add ../content
        }

        it "does add resource which walk to the parent directory if destination is specified" && {
          WITH_SNAPSHOT="$snapshot/resource-add-relative-dir-success" \
          expect_run $SUCCESSFULLY "$exe" \
            vault -c vault.yml add ../content:content
        }

        it "creates an the encrypted file" && {
          expect_exists content.gpg
        }
      )
    )
  )
)

(sandboxed
  title "'vault init' - multiple gpg keys available"
  gpg --import "$fixture/tester.sec.asc" "$fixture/c.sec.asc" &>/dev/null

  (with "a gpg key signed by others"
    it "fails as it can't decide which gpg key to export" && {
      WITH_SNAPSHOT="$snapshot/init-with-multiple-viable-keys" \
      expect_run $WITH_FAILURE "$exe" vault init
    }
    (with "a selected gpg key and a vault name"
      it "succeeds as it just follow instructions" && {
        WITH_SNAPSHOT="$snapshot/init-with-key-specified-explicitly" \
        expect_run $SUCCESSFULLY "$exe" vault init --name vault-name --gpg-key-id c@example.com
      }

      it "creates a valid vault configuration file, \
          exports the public portion of the selected key with signatures and \
          writes the list of recipients" && {
        expect_snapshot "$snapshot/init-single-user-with-multiple-signatures" .
      }

      it "uses the vault name when showing its contents" && {
        WITH_SNAPSHOT="$snapshot/list-with-name" \
        expect_run $SUCCESSFULLY "$exe" vault list
      }
    )
  )
)

(sandboxed
  title "'vault init' - use multiple secret keys"
  gpg --import "$fixture/tester.sec.asc" "$fixture/c.sec.asc" &>/dev/null


  (with "multiple selected gpg keys"
    it "succeeds as it just follow instructions" && {
      WITH_SNAPSHOT="$snapshot/init-with-multiple-specified-keys" \
      expect_run $SUCCESSFULLY "$exe" vault init --gpg-key-id c@example.com --gpg-key-id tester@example.com
    }

    it "creates the expected folder structure" && {
      expect_snapshot "$snapshot/init-multiple-users" .
    }
  )
)
