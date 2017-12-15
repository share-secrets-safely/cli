#!/bin/bash

set -u
exe=${1:?First argument is the executable under test}

root="$(cd "${0%/*}" && pwd)"
# shellcheck source=./tests/utilities.sh
source "$root/utilities.sh"

SUCCESSFULLY=0
WITH_FAILURE=1

title "'vault' subcommand"

it "defines a default for the vault configuration file and fails if it doesn't exist" && \
  run $WITH_FAILURE "$exe" vault
  
title "'extract' subcommand"

it "needs a file to be defined" && \
  run $WITH_FAILURE "$exe" extract



