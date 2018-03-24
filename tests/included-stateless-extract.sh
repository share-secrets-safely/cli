#!/bin/bash

template="$fixture/extract"
snapshot="$fixture/snapshots/extract"

title "'extract' - extracting values by pointer"
(with "a value that exists"
  (with "input from stdin"
    (with "explicit output mode"
      (when "it points to a scalar"
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/single-scalar-as-json" \
          expect_run $SUCCESSFULLY "$exe" extract -o=json /a < "$template/sample.yml"
        }
      )
      (when "it points to a null"
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/single-null-as-yaml" \
          expect_run $SUCCESSFULLY "$exe" extract -o=yaml c/null < "$template/sample.yml"
        }
      )
      (when "it points to a complex value"
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/complex-value-as-yaml" \
          expect_run $SUCCESSFULLY "$exe" extract -o=yaml c.complex < "$template/sample.yml"
        }
      )
    )
    (with "default output mode"
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
      (when "it points to null"
        it "succeeds but does not print anything" && {
          WITH_SNAPSHOT="$snapshot/single-null" \
          expect_run $SUCCESSFULLY "$exe" fetch c.null < "$template/sample.yml"
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
)
(with "multiple existing scalar values"
  (with "input from multiple files"
    (when "it points to scalars"
      it "succeeds" && {
        echo '{"c": {"a":"100"}}' | \
        WITH_SNAPSHOT="$snapshot/multiple-scalars" \
        expect_run $SUCCESSFULLY "$exe" extract -f="$template/sample.yml" a b /c/a c.complex.b.1 c.complex.b.0
      }
    )
    (when "it points to scalars and complex value"
      it "succeeds and returns a json array" && {
        cat "$template/sample.yml" | \
        WITH_SNAPSHOT="$snapshot/multiple-scalars-and-complex" \
        expect_run $SUCCESSFULLY "$exe" extract -f='-' --file="$template/sample.yml" a b c.complex.b.1 c.complex.b.0 c.complex
      }
    )
    (when "it points to scalars and an array value"
      (when "the output mode is set to yaml"
        it "succeeds and returns a yaml array" && {
          WITH_SNAPSHOT="$snapshot/multiple-scalars-and-array-explicit-output" \
          expect_run $SUCCESSFULLY "$exe" extract -o yaml -f="$template/sample.yml" \
                                                          -f="$template/sample.yml" \
                                                          a b c.complex.b.1 c.complex.b.0 \
                                                          array
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
    expect_run $WITH_FAILURE "$exe" extract -f=file-that-does-not-exist a.b
  }
)
(with "multiple times to read from standard input"
  it "fails" && {
    WITH_SNAPSHOT="$snapshot/fail-read-multiple-times-from-stdin" \
    expect_run $WITH_FAILURE "$exe" extract -f='-' -f='-' a.b
  }
)
(with "invalid output mode"
  it "fails" && {
    WITH_SNAPSHOT="$snapshot/fail-invalid-output-mode" \
    expect_run $WITH_FAILURE "$exe" extract -o foobar a.b < "$template/sample.yml"
  }
)
(with "no pointer"
  it "fails as there is nothing to extract" && {
    WITH_SNAPSHOT="$snapshot/fail-not-a-single-pointer" \
    expect_run $WITH_FAILURE "$exe" extract < "$template/sample.yml"
  }
)
