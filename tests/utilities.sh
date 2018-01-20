#!/bin/bash

WHITE="$(tput setaf 9)"
YELLOW="$(tput setaf 3)"
GREEN="$(tput setaf 2)"
RED="$(tput setaf 1)"
OFFSET=( )
STEP="  "

function title () {
  echo "$WHITE-----------------------------------------------------"
  echo "${GREEN}$*"
  echo "$WHITE-----------------------------------------------------"
}

function trust_key () {
  {
    gpg --export-ownertrust
    echo "${1:?First argument is the long fingerprint of the key to trust}:6:"
  } | gpg --import-ownertrust &>/dev/null
}

function import_user () {
  local key=${1:?First argument must be the keyfile identifying the user}
  
  gpg --import --yes --batch "$key" &>/dev/null
  
  local fpr
  fpr="$(gpg --list-secret-keys --with-colons --with-fingerprint | grep fpr | head -1)"
  fpr=${fpr:12:40}
  trust_key "$fpr"
}

function as_user () {
  local key=${1:?First argument must be the keyfile identifying the user}
  GNUPGHOME="$(mktemp -t gnupg-home.XXXX -d)"
  export GNUPGHOME
  
  import_user "$key" &>/dev/null
}

function sandboxed () {
  sandbox_tempdir="$(mktemp -t sandbox.XXXX -d)"
  # shellcheck disable=2064
  trap "popd >/dev/null" EXIT
  pushd "$sandbox_tempdir" >/dev/null \
   || fail "Could not change directory into temporary directory."
  GNUPGHOME="$(mktemp -t gnupg-home.XXXX -d)"
  export GNUPGHOME
}

function with () {
  echo 1>&2 "${YELLOW}${OFFSET[*]}[with] $*"
  OFFSET+=("$STEP")
}

function when () {
  echo 1>&2 "${YELLOW}${OFFSET[*]}[when] $*"
  OFFSET+=("$STEP")
}

function it () {
  echo 1>&2 -n "${OFFSET[*]}${GREEN}[it] ${*//  /}"
}

function precondition () {
  echo 1>&2 -n "${OFFSET[*]}${GREEN}[precondition] ${*//  /}"
}

function shortcoming () {
  echo 1>&2 -n "${OFFSET[*]}${STEP}${GREEN}[shortcoming] ${*//  /}"
}

function fail () {
  echo 1>&2 "$RED" "$@"
  exit 1
}

function expect_equals () {
  expect_run 0 test "${1:?}" = "${2:?}"
}

function expect_exists () {
  expect_run 0 test -e "${1:?}"
}

function expect_run_sh () {
  expect_run ${1:?} bash -c "${2:?}"
}

function expect_snapshot () {
  local expected=${1:?}
  local actual=${2:?}
  if ! [ -e "$expected" ]; then
    cp -R "$actual" "$expected"
  fi
  expect_run 0 diff -r "$expected" "$actual"
}

function expect_run () {
  local expected_exit_code=$1
  shift
  local output=
  set +e
  output="$("$@" 2>&1)"

  local actual_exit_code=$?
  if [[ "$actual_exit_code" == "$expected_exit_code" ]]; then
    if [[ -n "${WITH_SNAPSHOT-}" ]]; then
      local expected="$WITH_SNAPSHOT"
      if ! [ -f "$expected" ]; then
        echo -n "$output" > "$expected" || exit 1
      fi
      if ! diff "$expected" <(echo -n "$output"); then
        echo 1>&2 "${RED} - FAIL"
        echo 1>&2 "${WHITE}\$$*"
        echo 1>&2 "Output snapshot did not match snapshot at '$expected'"
        echo 1>&2 "$output"
        exit 1
      fi
    fi
    echo 1>&2 "${GREEN} - OK"
  else
    echo 1>&2 "${RED} - FAIL"
    echo 1>&2 "${WHITE}\$$*"
    echo 1>&2 "${RED}Expected actual status $actual_exit_code to be $expected_exit_code"
    echo 1>&2 "$output"
    exit 1
  fi
  set -e
}
