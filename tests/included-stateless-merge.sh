#!/bin/bash

template="$fixture/merge"
snapshot="$fixture/snapshots/merge/stateless"

title "'merge' subcommand"
(with "yaml from stdin"
  INPUT="answer: 42"
  (with "no file to merge"
    (with "no output specification"
      it "outputs json" && {
        echo "$INPUT" | \
        WITH_SNAPSHOT="$snapshot/yaml-stdin-to-stdout-output-format-unspecified" \
        expect_run $SUCCESSFULLY "$exe" merge
      }
    )
    (with "explicit yaml output"
      it "outputs yaml" && {
        echo "$INPUT" | \
        WITH_SNAPSHOT="$snapshot/yaml-stdin-to-stdout-as-yaml" \
        expect_run $SUCCESSFULLY "$exe" merge -o yaml
      }
    )
  )
  (with "the default merge mode"
    (with "multiple yaml documents from stdin and from file"
      it "merges all of them" && {
          WITH_SNAPSHOT="$snapshot/multi-document-yaml-stdin-to-stdout" \
          expect_run $SUCCESSFULLY "$exe" merge "$template/multi-docs-1.yml" < "$template/multi-docs-2.yml"
      }
    )
    (with "a single simple yaml file"
      (with "a single similar value"
        it "succeeds, as there is no conflict" && {
          echo "$INPUT" | \
          WITH_SNAPSHOT="$snapshot/yaml-stdin-with-yaml-file-same-scalar-value-to-stdout" \
          expect_run $SUCCESSFULLY "$exe" merge "$template/good-answer.yml"
        }
      )
    )
    (with "a two similar complex yaml files"
      it "succeeds, as there is no conflict" && {
        WITH_SNAPSHOT="$snapshot/no-stdin-with-two-similar-complex-yaml-files-to-stdout" \
        expect_run $SUCCESSFULLY "$exe" merge "$template/complex.yml" "$template/complex.yml"
      }
    )
  )
)

title "'merge' conflicts"
(with "a single complex yaml file"
  (with "a conflicting nested scalar value from stdin"
    it "fails" && {
      echo '{ "a_nested_map" : { "another_nested_map" : { "hello": "world"}}}' | \
      WITH_SNAPSHOT="$snapshot/fail-yaml-conflicting-nested-scalar-stdin-complex-yaml-file-to-stdout" \
      expect_run $WITH_FAILURE "$exe" merge "$template/complex.yml"
    }
  )
  (with "a conflicting array value from stdin"
    it "fails" && {
      echo "a_sequence: [foo]" | \
      WITH_SNAPSHOT="$snapshot/fail-yaml-conflicting-array-value-stdin-complex-yaml-file-to-stdout" \
      expect_run $WITH_FAILURE "$exe" merge "$template/complex.yml"
    }
  )
)
(with "a single conflicting file"
  (with "conflicting scalar value from stdin as yaml"
    it "fails as it refuses to overwrite keys" && {
      WITH_SNAPSHOT="$snapshot/fail-yaml-stdin-with-yaml-file-conflicting-scalar-to-stdout" \
      expect_run $WITH_FAILURE "$exe" merge "$template/wrong-answer.yml" < "$template/good-answer.yml"
    }
  )
)

title "'merge' subcommand errors"
(with "invalid output format"
  it "fails" && {
    WITH_SNAPSHOT="$snapshot/fail-invalid-output-format" \
    expect_run $WITH_FAILURE "$exe" merge --output foobar
  }
)
(with "no data from stdin"
  (with "no given file"
    it "fails" && {
      WITH_SNAPSHOT="$snapshot/fail-no-data-from-stdin-and-no-file-specified" \
      expect_run $WITH_FAILURE "$exe" merge
    }
  )
)
(with "data invalid data from stdin"
  (with "yaml format"
    it "fails" && {
      echo "a: -" | \
      WITH_SNAPSHOT="$snapshot/fail-invalid-yaml-stdin" \
      expect_run $WITH_FAILURE "$exe" merge
    }
  )
  (with "json format"
    it "fails" && {
      echo "invalid-json" | \
      WITH_SNAPSHOT="$snapshot/fail-invalid-json-stdin" \
      expect_run $WITH_FAILURE "$exe" merge
    }
  )
)
