#!/bin/bash

set -u
exe=${1:?First argument is the executable under test}

function title () {
  echo "-----------------------------------------------------"
  echo "$@"
  echo "-----------------------------------------------------"
}

SUCCESSFULLY=0
IT_COUNT=0
function it () {
  IT_COUNT=$(( IT_COUNT + 1 ))
  echo 1>&2 -n "$(tput setaf 2)" "$@"
}

function fail () {
  echo 1>&2 "$(tput setaf 1)" " - FAIL"
  exit $IT_COUNT
}

function run () {
  local expected_exit_code=$1
  shift
  local output=
  output="$("$@" 2>&1)"
  
  if [[ $? == "$expected_exit_code" ]]; then
    echo 1>&2 "$(tput setaf 2)" " - OK"
  else
    echo 1>&2 "$(tput setaf 1)" " - FAIL"
    echo 1>&2 "$output"
    exit $IT_COUNT
  fi
}

title "'vault' subcommand"

it "requires the vault configuration to be set" && \
  run $SUCCESSFULLY $exe vault



