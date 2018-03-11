#!/bin/bash

set -eu -o pipefail
exe=${1:?First argument is the executable under test}
exe="$(cd "${exe%/*}" && pwd)/${exe##*/}"

rela_root="${0%/*}"
root="$(cd "${rela_root}" && pwd)"
# shellcheck source=./tests/gpg-helpers.sh
source "$root/gpg-helpers.sh"

WITH_FAILURE=1
SUCCESSFULLY=0

fixture="$root/journeys/fixtures"

# shellcheck source=./tests/included-stateless-merge.sh
source "$root/included-stateless-merge.sh"

# shellcheck source=./tests/included-stateless-substitute.sh
source "$root/included-stateless-substitute.sh"

# shellcheck source=./tests/included-stateless-vault.sh
source "$root/included-stateless-vault.sh"
