#!/bin/bash

WHITE="$(tput setaf 9)"
GREEN="$(tput setaf 2)"
RED="$(tput setaf 1)"
IT_COUNT=0

function title () {
  echo "$WHITE-----------------------------------------------------"
  echo "$@"
  echo "-----------------------------------------------------"
}

function it () {
  IT_COUNT=$(( IT_COUNT + 1 ))
  echo 1>&2 -n "$GREEN" "$@"
}

function run () {
  local expected_exit_code=$1
  shift
  local output=
  output="$("$@" 2>&1)"

  local actual_exit_code=$?
  if [[ "$actual_exit_code" == "$expected_exit_code" ]]; then
    echo 1>&2 "$GREEN" " - OK"
  else
    echo 1>&2 "$RED" " - FAIL"
    echo 1>&2 "Expected actual status $actual_exit_code to be $expected_exit_code"
    echo 1>&2 "$output"
    exit $IT_COUNT
  fi
}
