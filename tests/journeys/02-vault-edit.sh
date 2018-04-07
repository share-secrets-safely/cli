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
snapshot="$fixture/snapshots/vault/edit"
(sandboxed
  title "'vault add'"
  (with "a vault initialized for a single recipient"
    {
      import_user "$fixture/tester.sec.asc"
      "$exe" init --trust-model=web-of-trust --no-auto-import
    } &> /dev/null

    (when "adding new resource from stdin"
      it "succeeds" && {
        echo hi | WITH_SNAPSHOT="$snapshot/resource-add-from-stdin" \
        expect_run $SUCCESSFULLY "$exe" add :from-stdin
      }

      it "creates an encrypted file" && {
        expect_exists ./from-stdin.gpg
      }

      it "shows the single file without gpg suffix" && {
        WITH_SNAPSHOT="$snapshot/ls-with-single-resource" \
        expect_run $SUCCESSFULLY "$exe" ls
      }
    )
    (when "adding new resource from stdin with a tty attached"
      editor="$PWD/my-add-editor.sh"
      (
        cat <<'EDITOR' > "$editor"
#!/bin/bash -e
file_to_edit=${1:?}
echo -n "$file_to_edit" > /tmp/vault-add-file-to-edit
echo "vault add from editor" > $file_to_edit
EDITOR
        chmod +x "$editor"
      )
      export EDITOR="$editor"
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/resource-add-from-stdin-no-tty" \
        expect_run $SUCCESSFULLY "$exe" add :from-stdin-no-tty
      }

      it "creates an encrypted file" && {
        expect_exists ./from-stdin-no-tty.gpg
      }

      it "encrypts the content from the editor" && {
        WITH_SNAPSHOT="$snapshot/add-from-stdin-no-tty-show" \
        expect_run $SUCCESSFULLY "$exe" show from-stdin-no-tty
      }

      it "deletes the temporary file with the plain-text fontent" && {
        expect_run $WITH_FAILURE test -e "$(cat /tmp/vault-add-file-to-edit)"
      }
    )
    (when "adding the same resource from stdin"
      previous_resource_id="$(md5sum ./from-stdin.gpg)"

      it "fails as it won't overwrite existing resources" && {
        echo hi | WITH_SNAPSHOT="$snapshot/resource-add-overwrite-protection" \
        expect_run $WITH_FAILURE "$exe" add :from-stdin
      }

      it "does not change the previous file" && {
        expect_equals "$previous_resource_id" "$(md5sum ./from-stdin.gpg)"
      }
    )
    title "'vault show'"
    (when "showing the previously added resource"
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/resource-show" \
        expect_run $SUCCESSFULLY "$exe" show from-stdin
      }
    )
    (when "showing the previously added resource using the gpg suffix"
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/resource-show" \
        expect_run $SUCCESSFULLY "$exe" show ./from-stdin.gpg
      }
    )
    (when "showing an unknown resource"
      it "fails" && {
        WITH_SNAPSHOT="$snapshot/unknown-resource-show" \
        expect_run $WITH_FAILURE "$exe" show some-unknown-resource
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

    title "'vault edit'"
    (when "editing a known resource"
      (with "a custom editor"
        it "succeeds" && {
          EDITOR="$editor" \
          expect_run $SUCCESSFULLY "$exe" edit from-stdin
        }
        it "changes the file accordingly" && {
          WITH_SNAPSHOT="$snapshot/edit-changed-file" \
          expect_run $SUCCESSFULLY "$exe" show from-stdin.gpg
        }
        it "removes the file with decrypted content" && {
          expect_run $WITH_FAILURE test -f "$(cat /tmp/filepath-with-decrypted-content)"
        }
      )
    )
    (when "editing a new resource with a custom editor"
      it "creates the new resource" && {
        EDITOR="$editor" \
        expect_run $SUCCESSFULLY "$exe" edit new-edited-file
      }
      it "changes creates file accordingly" && {
        WITH_SNAPSHOT="$snapshot/edit-changed-file" \
        expect_run $SUCCESSFULLY "$exe" show new-edited-file
      }
      it "removes the file with decrypted content" && {
        expect_run $WITH_FAILURE test -f "$(cat /tmp/filepath-with-decrypted-content)"
      }
    )
    (when "editing an unknown resource"
      it "fails" && {
        WITH_SNAPSHOT="$snapshot/unknown-resource-edit" \
        expect_run $WITH_FAILURE "$exe" edit --no-create some-unknown-resource
      }
    )
    (with "an editor program that does not exist in path"
      it "fails" && {
        WITH_SNAPSHOT="$snapshot/known-resource-edit-editor-not-in-path" \
        expect_run $WITH_FAILURE "$exe" edit --editor foo from-stdin
      }
    )
    (when "editing a file without being able to encrypt (but to decrypt)"
      {
        gpg --import "$fixture/b.sec.asc"
        "$exe" recipient add 42C18D28
      } &>/dev/null

      (with "no additional flags"
        it "fails because it encrypts in advance to avoid losing the edit" && (
          as_user "$fixture/b.sec.asc"
          gpg --import '.gpg-keys/D6339718E9B58FCE3C66C78AAA5B7BF150F48332' &>/dev/null
          WITH_SNAPSHOT="$snapshot/resource-edit-encrypt-failure" \
          expect_run $WITH_FAILURE "$exe" edit --editor 'does-not-matter' from-stdin
        )
      )
      (with "--no-try-encrypt set"
        it "fails because now it would try to open the non-existing editor right away" && (
          as_user "$fixture/b.sec.asc"

          WITH_SNAPSHOT="$snapshot/resource-edit-encrypt-failure-no-try-encrypt" \
          expect_run $WITH_FAILURE "$exe" edit --no-try-encrypt --editor 'does-not-matter' from-stdin
        )
      )
    )

    title "'vault remove'"
    function add_resource () {
      local res=${1:?}
      echo "$res" | "$exe" add ":$res" > /dev/null
    }
    (when "removing a resource that exists without the gpg extension and with gpg extension"
      add_resource a.ext; add_resource b; add_resource c

      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/resource-remove-multiple-existing" \
        expect_run $SUCCESSFULLY "$exe" remove a.ext b.gpg c
      }

      it "actually removes the files" && {
        WITH_SNAPSHOT="$snapshot/resource-remove-multiple-existing-after" \
        expect_run $SUCCESSFULLY "$exe" list
      }
    )

    (when "removing a resource that does not exist"
      add_resource existing
      it "fails at the first non-existing resource" && {
        WITH_SNAPSHOT="$snapshot/resource-remove-non-existing" \
        expect_run $WITH_FAILURE "$exe" delete existing non-existing existing
      }
    )
  )
)
