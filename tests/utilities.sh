#!/bin/bash

WHITE="$(tput setaf 9)"
YELLOW="$(tput setaf 3)"
GREEN="$(tput setaf 2)"
RED="$(tput setaf 1)"
IT_COUNT=0
CONTEXT=

function title () {
  echo "$WHITE-----------------------------------------------------"
  echo "$@"
  echo "-----------------------------------------------------"
}

function with () {
    CONTEXT="[with] $* "
}

function without_context() {
    # shellcheck disable=SC2034
    CONTEXT=""
}

function it () {
  IT_COUNT=$(( IT_COUNT + 1 ))
  echo 1>&2 -n "${YELLOW}${CONTEXT}${GREEN}[it] $*"
}

function expect_run () {
  local expected_exit_code=$1
  shift
  local output=
  output="$("$@" 2>&1)"

  local actual_exit_code=$?
  if [[ "$actual_exit_code" == "$expected_exit_code" ]]; then
    if [[ -n "${WITH_OUTPUT-}" ]]; then
        if ! echo "$output" | tr '\n' ' ' | grep -qE "$WITH_OUTPUT"; then
            echo 1>&2 "$RED" " - FAIL"
            echo 1>&2 "Output did not match '$WITH_OUTPUT'"
            echo 1>&2 "$output"
            exit $IT_COUNT
        fi
    fi
    echo 1>&2 "$GREEN" " - OK"
  else
    echo 1>&2 "$RED" " - FAIL"
    echo 1>&2 "Expected actual status $actual_exit_code to be $expected_exit_code"
    echo 1>&2 "$output"
    exit $IT_COUNT
  fi
}