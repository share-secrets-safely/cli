#!/bin/bash

template="$fixture/extract"
snapshot="$fixture/snapshots/extract/stateless"

title "'extract' - extracting values by pointer"
(with "a value that exists"
  (with "input from stdin"
    (when "it points to a scalar"
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/single-scalar" \
        expect_run $SUCCESSFULLY "$exe" extract /a < "$template/sample.yml"
      }
    )
    (when "it points to a string"
      it "succeeds and outputs the blank value" && {
        WITH_SNAPSHOT="$snapshot/single-string" \
        expect_run $SUCCESSFULLY "$exe" fetch c.str < "$template/sample.yml"
      }
    )
    (when "it points to a complex value"
      it "succeeds and outputs JSON" && {
        WITH_SNAPSHOT="$snapshot/complex-value-json-output" \
        expect_run $SUCCESSFULLY "$exe" extract c.complex < "$template/sample.yml"
      }
    )
  )
)
(with "multiple existing scalar values"
  (with "input from multiple files"
    (when "it points to scalars"
      it "succeeds" && {
        cat "$template/sample.yml" | \
        WITH_SNAPSHOT="$snapshot/multiple-scalars" \
        expect_run $SUCCESSFULLY "$exe" extract -f "$template/sample.yml" a b /c/a c/b.0 c.b.1
      }
    )
    (when "it points to scalars and complex value"
      it "succeeds and returns a json array" && {
        WITH_SNAPSHOT="$snapshot/multiple-scalars" \
        expect_run $SUCCESSFULLY "$exe" extract -f "$template/sample.yml" --file "$template/sample.yml" a b /c/a c/b.0 c.b.1 c.complex
      }
    )
    (when "it points to scalars and an array value"
      (when "the output mode is set to yaml"
        it "succeeds and returns a yaml array" && {
          WITH_SNAPSHOT="$snapshot/multiple-scalars" \
          expect_run $SUCCESSFULLY "$exe" extract -o yaml -f "$template/sample.yml" \
                                                          -f "$template/sample.yml" \
                                                          a b /c/a c/b.0 c.b.1 array
        }
      )
    )
  )
)
(with "intput from stdin"
  (with "a single value that not exists"
    it "fails" && {
      WITH_SNAPSHOT="$snapshot/fail-single-missing-key" \
      expect_run $WITH_FAILURE "$exe" extract foo/bar < "$template/sample.yml"
    }
  )
  (with "multiple values that do not exists"
    it "fails on the first missing key" && {
      WITH_SNAPSHOT="$snapshot/fail-multiple-missing-keys" \
      expect_run $WITH_FAILURE "$exe" extract foo/bar bar.baz < "$template/sample.yml"
    }
  )
)
(with "no value to read"
  it "fails" && {
    WITH_SNAPSHOT="$snapshot/fail-no-input" \
    expect_run $WITH_FAILURE "$exe" extract foo/bar
  }
)
(with "a file to read that does not exist"
  it "fails" && {
    WITH_SNAPSHOT="$snapshot/fail-file-cannot-be-read" \
    expect_run $WITH_FAILURE "$exe" extract -f file-that-does-not-exist
  }
)
(with "no pointer"
  it "fails as there is nothing to extract" && {
    WITH_SNAPSHOT="$snapshot/fail-not-a-single-pointer" \
    expect_run $WITH_FAILURE "$exe" extract < "$template/sample.yml"
  }
)
