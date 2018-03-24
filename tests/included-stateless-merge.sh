#!/bin/bash

template="$fixture/merge"
snapshot="$fixture/snapshots/merge"

title "'merge' - setting values"
(when "setting values of various types"
  (with "no conflicts"
    it "succeeds" && {
      WITH_SNAPSHOT="$snapshot/set-various-values-without-conflict" \
      expect_run $SUCCESSFULLY "$exe" show -o yaml z= a=42 b=true c=false d=42 e=value f=13.234 x.0.s.t='{"a": [1,2,3]}'
    }
  )
  (with "conflicts"
    it "fails" && {
      WITH_SNAPSHOT="$snapshot/fail-set-various-values-with-conflict" \
      expect_run $WITH_FAILURE "$exe" show -o yaml a.b=42 c=3 a.b=43
    }
  )
  (with "conflicts and overwrite enabled"
    it "succeeds" && {
      WITH_SNAPSHOT="$snapshot/set-various-values-with-conflict-and-overwrite" \
      expect_run $SUCCESSFULLY "$exe" show -o yaml a.b=42 c=3 --overwrite a.b=43
    }
  )
)

title "'merge' environment"
(with "the 'environment' arg set"
  (with "with the --at option set"
    (when "there is no conflict"
      it "succeeds and merges the all matching environment variables into the root and is depleted" && {
        WITH_SNAPSHOT="$snapshot/environment-filtered-at-new-key-a" \
        TEST_MARKER_STRING=value \
        TEST_MARKER_COMPLEX='{"a":1, "b":2, "c": [1,2,3], "d": "val"}' \
        expect_run $SUCCESSFULLY $exe merge -o yaml --at a --environment='TEST_*' --environment='TEST_*'
      }
    )
    (when "the --at flag is nested and starts with an array"
      it "succeeds and merges the all matching environment variables into the root and is depleted" && {
        WITH_SNAPSHOT="$snapshot/environment-filtered-at-new-nested-key-with-array" \
        TEST_MARKER_STRING=value \
        TEST_MARKER_COMPLEX='{"a":1, "b":2, "c": [1,2,3], "d": "val"}' \
        expect_run $SUCCESSFULLY $exe merge -o yaml --at 3/a/0/b --environment='TEST_*' --environment='TEST_*'
      }
    )
    (when "the --at flag is nested and starts with an object"
      it "succeeds and merges the all matching environment variables into the root and is depleted" && {
        WITH_SNAPSHOT="$snapshot/environment-filtered-at-new-nested-key-with-object" \
        TEST_MARKER_STRING=value \
        TEST_MARKER_COMPLEX='{"a":1, "b":2, "c": [1,2,3], "d": "val"}' \
        expect_run $SUCCESSFULLY $exe merge -o yaml --at a.2.b.0 --environment='TEST_*' --environment='TEST_*'
      }
    )
    (when "value exists but is no map"
      it "fails as a map is required" && {
        WITH_SNAPSHOT="$snapshot/fail-environment-filtered-at-existing-key-a-which-is-no-map" \
        expect_run $WITH_FAILURE $exe merge <(echo 'a: 42') --at a --environment='TEST_*'
      }
    )
    (when "value exists and is a map without conflicts"
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/empty-environment-filtered-at-non-existing-key-a" \
        expect_run $SUCCESSFULLY $exe merge --at a --environment='TEST_*'
      }
    )
  )
  (with "no explicit filter"
    it "succeeds and merges the all environment variables into the root" && {
      WITH_SNAPSHOT="$snapshot/environment-unfiltered-at-root" \
      TEST_MARKER_VARIABLE=value \
      expect_run_sh $SUCCESSFULLY "$exe merge -o json --environment | grep '\"TEST_MARKER_VARIABLE\": \"value\",' "
    }
  )
  (with "explicit filter that has matches"
    it "succeeds and merges the all matching environment variables into the root" && {
      WITH_SNAPSHOT="$snapshot/environment-filtered-at-root" \
      TEST_MARKER_STRING=value \
      TEST_MARKER_INT=42 \
      TEST_MARKER_INT_NEGATIVE=-42 \
      TEST_MARKER_FLOAT=42.5 \
      TEST_MARKER_FLOAT_NEGATIVE=-42.5 \
      TEST_MARKER_BOOL_TRUE=true \
      TEST_MARKER_BOOL_FALSE=false \
      TEST_MARKER_INVALID_JSON="{'a:}" \
      TEST_MARKER_COMPLEX='{"a":1, "b":2, "c": [1,2,3], "d": "val"}' \
      expect_run $SUCCESSFULLY $exe merge -o yaml '--environment=TEST_*'
    }
  )
  (with "explicit filter that matches nothing"
    it "succeeds as we believe that errors should happpen later during substitution" && {
      WITH_SNAPSHOT="$snapshot/environment-filtered-no-match-at-root" \
      expect_run $SUCCESSFULLY $exe merge -o json --environment=foobarbaz
    }
  )
  (with "invalid environment pattern"
    it "fails" && {
      WITH_SNAPSHOT="$snapshot/fail-environment-invalid-filter" \
      expect_run $WITH_FAILURE $exe merge --environment=][
    }
  )
)

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
      (with "no --at flag"
        it "outputs yaml" && {
          echo "$INPUT" | \
          WITH_SNAPSHOT="$snapshot/yaml-stdin-to-stdout-as-yaml" \
          expect_run $SUCCESSFULLY "$exe" merge -o yaml
        }
      )
      (with "--at set"
        it "outputs yaml at the correct position" && {
          echo "$INPUT" | \
          WITH_SNAPSHOT="$snapshot/yaml-stdin-with-at-to-stdout-as-yaml" \
          expect_run $SUCCESSFULLY "$exe" merge -o yaml -at a.b
        }
      )
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

title "'merge' - selecting trees"
(with "an object with a scalar value"
  (with "no --to specification"
    it "succeeds" && {
      WITH_SNAPSHOT="$snapshot/select-single-scalar-value" \
      expect_run $SUCCESSFULLY "$exe" process --select=a <(echo a: 42)
    }
  )
  (with "--to specification"
    it "succeeds" && {
      WITH_SNAPSHOT="$snapshot/select-single-scalar-value-with-move" \
      expect_run $SUCCESSFULLY "$exe" process --select=a --to=c.d.e <(echo a: 42)
    }
  )
)
(with "a nested object with an array"
  it "succeeds" && {
    WITH_SNAPSHOT="$snapshot/select-nested-scalar-value-array" \
    expect_run $SUCCESSFULLY "$exe" process --select=a/b <(echo '{"a":{"b":[1,2,3]}}')
  }
)
(when "selecting a value without a following merge"
  (with "no --to specification"
    it "succeeds and applies the transformation to the merged result" && {
      WITH_SNAPSHOT="$snapshot/select-nested-scalar-value-array-on-merged-value" \
      expect_run $SUCCESSFULLY "$exe" process <(echo '{"a":{"b":[1,2,3]}}') <(echo c: 42) --select c
    }
  )
  (with "--to specification"
    it "succeeds and applies the transformation to the merged result" && {
      WITH_SNAPSHOT="$snapshot/select-nested-scalar-value-array-on-merged-value-with-move" \
      expect_run $SUCCESSFULLY "$exe" process <(echo '{"a":{"b":[1,2,3]}}') <(echo c: 42) --from c --to a.b
    }
  )
)
(with "a selection pointer which points to no object"
  it "fails" && {
    WITH_SNAPSHOT="$snapshot/fail-select-non-existing-object" \
    expect_run $WITH_FAILURE "$exe" show -s=foo <(echo 'a: 42')
  }
)

title "'merge' --at and stdin"
(with "data from stdin"
  INPUT="from-stdin: 42"
  (with "no location specification for stdin"
    (with "--at before a path to merge"
      it "succeeds and applies the --at to stdin" && {
        echo "$INPUT" | \
        WITH_SNAPSHOT="$snapshot/json-stdin-at-before-path-to-stdout" \
        expect_run $SUCCESSFULLY "$exe" merge --at=a.b "$template/good-answer.yml"
      }
    )
  )
  (with "location specification for stdin"
    (with "--at before a path to merge"
      it "succeeds and applies the --at to the path" && {
        echo "$INPUT" | \
        WITH_SNAPSHOT="$snapshot/json-stdin-at-before-path-with-explicit-stdin-marker-to-stdout" \
        expect_run $SUCCESSFULLY "$exe" merge - --at=a.b "$template/good-answer.yml"
      }
    )
  )
)
(with "--at at various locations"
  it "succeeds and consumes the --at flags for each merge" && {
    cat "$template/good-answer.yml" | \
    WITH_SNAPSHOT="$snapshot/json-stdin-at-before-path-with-explicit-stdin-marker-and-various-at-flags-to-stdout" \
    expect_run $SUCCESSFULLY "$exe" merge - --at=a.b "$template/good-answer.yml" <(echo 'foo: bar') --at=c "$template/good-answer.yml"
  }
)
(with "an unconsumed --at flag"
  it "succeeds as it applies the operation to the final value" && {
    WITH_SNAPSHOT="$snapshot/unconsumed-at-flag-applies-to-final-value" \
    expect_run $SUCCESSFULLY "$exe" merge "$template/good-answer.yml" --at c
  }
)

title "'merge' overwrite control"
(with "a file"
  (with "another file conflicting with the first"
    (with "overwrite enabled"
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/allow-overwrite-no-stdin-with-two-conflicting-files-to-stdout" \
        expect_run $SUCCESSFULLY "$exe" merge "$template/good-answer.yml" --overwrite "$template/wrong-answer.yml"
      }
      (when "followed by another conflicting document"
        it "succeeds as the overwrite mode persists - it must be toggled" && {
          WITH_SNAPSHOT="$snapshot/allow-overwrite-no-stdin-with-two-conflicting-files-to-stdout-overwrite-persists" \
          expect_run $SUCCESSFULLY "$exe" merge "$template/good-answer.yml" --overwrite "$template/wrong-answer.yml" "$template/good-answer.yml"
        }
      )
    )
    (with "overwrite disabled"
      it "fails" && {
        WITH_SNAPSHOT="$snapshot/fail-no-stdin-with-two-conflicting-files-to-stdout" \
        expect_run $WITH_FAILURE "$exe" merge "$template/good-answer.yml" --overwrite --no-overwrite "$template/wrong-answer.yml"
      }
    )
  )
  (with "a conflicting file from stdin"
    (with "overwrite enabeld"
      it "succeeds as all overwrite arguments are applied to the input from stdin" && {
        cat "$template/good-answer.yml" | \
        WITH_SNAPSHOT="$snapshot/allow-overwrite-file-from-stdin-with-one-conflicting-file-to-stdout" \
        expect_run $SUCCESSFULLY "$exe" merge --overwrite --no-overwrite --overwrite "$template/wrong-answer.yml"
      }
    )
  )
)

title "'merge' conflicts"
(with "a single complex yaml file"
  (with "a conflicting nested scalar value from stdin"
    INPUT='{ "a_nested_map" : { "another_nested_map" : { "hello": "world"}}}'
    (with "default overwrite settings"
      it "fails" && {
        cat "$template/complex.yml" | \
        WITH_SNAPSHOT="$snapshot/fail-yaml-conflicting-nested-scalar-stdin-complex-yaml-file-to-stdout" \
        expect_run $WITH_FAILURE "$exe" merge <(echo "$INPUT")
      }
    )
    (with "overwrite enabled"
      it "succeeds" && {
        cat "$template/complex.yml" | \
        WITH_SNAPSHOT="$snapshot/allow-overwrite-yaml-conflicting-nested-scalar-stdin-complex-yaml-file-to-stdout" \
        expect_run $SUCCESSFULLY "$exe" merge --overwrite <(echo "$INPUT")
      }
    )
  )
  (with "a conflicting array value from stdin"
    INPUT="a_sequence: [foo]"
    (with "default overwrite settings"
      it "fails" && {
        cat "$template/complex.yml" | \
        WITH_SNAPSHOT="$snapshot/fail-yaml-conflicting-array-value-stdin-complex-yaml-file-to-stdout" \
        expect_run $WITH_FAILURE "$exe" merge <(echo "$INPUT")
      }
    )
    (with "overwrite enabled"
      it "succeeds" && {
        cat "$template/complex.yml" | \
        WITH_SNAPSHOT="$snapshot/allow-overwrite-yaml-conflicting-array-value-stdin-complex-yaml-file-to-stdout" \
        expect_run $SUCCESSFULLY "$exe" merge --overwrite <(echo "$INPUT")
      }
    )
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
(with "a single multi-document file with conflicts"
  (when "fed from stdin"
    (with "default overwrite settings"
      it "fails as it refuses to overwrite keys" && {
        cat "$template/multi-docs-conflict.yml" | \
        WITH_SNAPSHOT="$snapshot/fail-multi-doc-yaml-with-conflict-from-stdin-to-stdout" \
        expect_run $WITH_FAILURE "$exe" merge
      }
    )
    (with "with overwrite enabled"
      it "succeeds" && {
        cat "$template/multi-docs-conflict.yml" | \
        WITH_SNAPSHOT="$snapshot/allow-overwrite-multi-doc-yaml-with-conflict-from-stdin-to-stdout" \
        expect_run $SUCCESSFULLY "$exe" merge --overwrite
      }
    )
  )
  (when "fed as file"
    (with "default overwrite settings"
      it "fails as it refuses to overwrite keys" && {
        WITH_SNAPSHOT="$snapshot/fail-multi-doc-yaml-with-conflict-from-file-to-stdout" \
        expect_run $WITH_FAILURE "$exe" merge "$template/multi-docs-conflict.yml"
      }
    )
    (with "with overwrite enabled"
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/allow-overwrite-multi-doc-yaml-with-conflict-from-file-to-stdout" \
        expect_run $SUCCESSFULLY "$exe" merge --overwrite "$template/multi-docs-conflict.yml"
      }
    )
  )
)

title "'merge' subcommand errors"
(with "invalid output format"
  it "fails" && {
    WITH_SNAPSHOT="$snapshot/fail-invalid-output-format" \
    expect_run $WITH_FAILURE "$exe" merge --output foobar
  }
)
(with "multiple mentions of stdin"
  it "fails" && {
    WITH_SNAPSHOT="$snapshot/fail-use-of-stdin-more-than-once" \
    expect_run $WITH_FAILURE "$exe" merge - -
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
      echo '{"hi}' | \
      WITH_SNAPSHOT="$snapshot/fail-invalid-json-stdin" \
      expect_run $WITH_FAILURE "$exe" merge
    }
  )
)
(with "an --at pointer in dot format that does not exist"
  it "fails" && {
    echo "a: ~" | \
    WITH_SNAPSHOT="$snapshot/fail-invalid-dotted-pointer" \
    expect_run $WITH_FAILURE "$exe" show --from a.b.c
  }
)
