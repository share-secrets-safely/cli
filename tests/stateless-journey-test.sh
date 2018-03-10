#!/bin/bash

set -eu -o pipefail
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
# shellcheck source=./tests/gpg-helpers.sh
source "$root/gpg-helpers.sh"

WITH_FAILURE=1
SUCCESSFULLY=0

title "'substitute' subcommand"
fixture="$root/journeys/fixtures"
template="$fixture/substitute"
snapshot="$fixture/snapshots/substitute/stateless"

(with "stdin for data"
  (with "input as json"
    (with "single template from a file (absolute path)"
      it "outputs the substituted data to stdout" && {
        echo '{"the-answer": 42}' | \
        WITH_SNAPSHOT="$snapshot/data-stdin-json-single-template-stdout" \
        expect_run $SUCCESSFULLY "$exe" substitute "$template/the-answer.hbs"
      }
    )
  )
  (with "input as yaml"
    (with "single template from a file (absolute path)"
      it "outputs the substituted data to stdout" && {
        echo "the-answer: 42" | \
        WITH_SNAPSHOT="$snapshot/data-stdin-yaml-single-template-stdout" \
        expect_run $SUCCESSFULLY "$exe" substitute "$template/the-answer.hbs"
      }
    )
    (sandbox
      (with "single template from a file (relative path)"
        cp "$template/the-answer.hbs" template.hbs
        it "outputs the substituted data to stdout" && {
          echo "the-answer: 42" | \
          WITH_SNAPSHOT="$snapshot/data-stdin-yaml-single-relative-template-stdout" \
          expect_run $SUCCESSFULLY "$exe" substitute template.hbs
        }
      )
    )
    (with "multiple templates from a file (absolute path)"
      (with "the default document separator"
        it "outputs the substituted data to stdout" && {
          echo "the-answer: 42" | \
          WITH_SNAPSHOT="$snapshot/data-stdin-yaml-multi-template-stdout" \
          expect_run $SUCCESSFULLY "$exe" substitute "$template/the-answer.hbs" "$template/the-answer.hbs"
        }
      )
      (with "an explicit document separator"
        it "outputs the substituted data to stdout" && {
          echo "the-answer: 42" | \
          WITH_SNAPSHOT="$snapshot/data-stdin-yaml-multi-template-stdout-explicit-separator" \
          expect_run $SUCCESSFULLY "$exe" substitute --separator "\\n<->\\n" "$template/the-answer.hbs" "$template/the-answer.hbs"
        }
      )
    )
  )
)


(with "stdin for data"
  (with "input as yaml"
    (with "multiple template from a file to the same output file"
      (sandbox
        (with "the default document separator"
          it "succeeds" && {
            echo "the-answer: 42" | \
            WITH_SNAPSHOT="$snapshot/data-stdin-yaml-multi-template-to-same-file" \
            expect_run $SUCCESSFULLY "$exe" substitute "$template/the-answer.hbs:output" "$template/the-answer.hbs:output"
          }

          it "produces the expected output" && {
            expect_snapshot "$snapshot/data-stdin-yaml-multi-template-to-same-file-output" output
          }
        )
      )
      (sandbox
        (with "the explicit document separator"
          it "succeeds" && {
            echo "the-answer: 42" | \
            WITH_SNAPSHOT="$snapshot/data-stdin-yaml-multi-template-to-same-file-explicit-separator" \
            expect_run $SUCCESSFULLY "$exe" substitute -s "\\n---\\n" "$template/the-answer.hbs:output" "$template/the-answer.hbs:$PWD/output"
          }
          it "produces the expected output" && {
            expect_snapshot "$snapshot/data-stdin-yaml-multi-template-to-same-file-explicit-separator-output" output
          }
        )
      )
    )
  )
)
(with "stdin for templates"
  (with "data from file"
    (when "writing the output to stdout"
      (with "implicit syntax"
        it "succeeds" && {
          echo "hello {{to-what}}" | \
          WITH_SNAPSHOT="$snapshot/template-stdin-hbs-output-stdout" \
          expect_run $SUCCESSFULLY "$exe" substitute -d <(echo "to-what: world")
        }
      )
      (with "explicit syntax"
        it "succeeds" && {
          echo "hello {{to-what}}" | \
          WITH_SNAPSHOT="$snapshot/template-stdin-hbs-output-stdout" \
          expect_run $SUCCESSFULLY "$exe" substitute -d <(echo '{"to-what": "world"}') :
        }
      )
    )
    (sandbox
      (when "writing the output to a file"
        (with "implicit syntax"
          it "succeeds" && {
            echo "hello {{to-what}}" | \
            WITH_SNAPSHOT="$snapshot/template-stdin-hbs-output-stdout-to-file" \
            expect_run $SUCCESSFULLY "$exe" substitute -d <(echo "to-what: world") :output
          }
          it "produces the expected output" && {
            expect_snapshot "$snapshot/template-stdin-hbs-output-stdout-to-file" output
          }
        )
      )
    )
  )
)


title "'substitute' subcommand error cases"
(with "a spec that tries to write the output to the input template"
  (with "a single spec"
    it "fails as it refuses" && {
      WITH_SNAPSHOT="$snapshot/fail-source-is-destination-single-spec" \
      expect_run $WITH_FAILURE "$exe" substitute -d <(echo does not matter) "$fixture/the-answer.hbs:$fixture/the-answer.hbs"
    }
  )
)
(with "multiple specs indicating to read them from stdin"
  it "fails as this cannot be done" && {
    WITH_SNAPSHOT="$snapshot/fail-multiple-templates-from-stdin" \
    expect_run $WITH_FAILURE "$exe" substitute -d <(echo does not matter) :first.out :second.out
  }
)

title "'vault' subcommand"
snapshot="$fixture/snapshots/vault/stateless"

(with "a minimal vault configuration file"
  it "succeeds even if there is no further argument" && \
      echo 'secrets: .' | expect_run $SUCCESSFULLY "$exe" vault -c -
)

title "'vault init' subcommand"

(with "an invalid vault path"
  it "fails" && \
      WITH_SNAPSHOT="$snapshot/invalid-vault-path" expect_run $WITH_FAILURE "$exe" vault -c / init
)

title "'completions' subcommand"

(with "a supported $SHELL"
    it "generates a script executable by $SHELL" && \
      expect_run $SUCCESSFULLY "$exe" completions | $SHELL
)

(with "an explicit supported shell name"
    it "generates a valid script" && \
      expect_run $SUCCESSFULLY "$exe" completions bash | bash
)

(with "an unsupported shell"
    it "fails with a suitable error" && {
      WITH_SNAPSHOT="$snapshot/unsupported-shell" \
      expect_run $WITH_FAILURE "$exe" completions foobar
    }
)
