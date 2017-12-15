#!/bin/bash

set -u
exe=${1:?First argument is the executable under test}

root="$(cd "${PWD:%/*}" && pwd)"
# shellcheck source=./tests/utilities.sh
source "$root/utilities.sh"

SUCCESSFULLY=0
WITH_FAILURE=1

title "'vault' subcommand"

it "defines a default for the vault configuration file" && \
  run $SUCCESSFULLY "$exe" vault
  
title "'extract' subcommand"

it "needs a file to be defined" && \
  run $WITH_FAILURE "$exe" extract



