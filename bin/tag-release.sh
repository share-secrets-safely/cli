#!/bin/bash

set -eu -o pipefail

version=${1:?First argument is the version you want to tag, like '1.0.0'}

find . -name Cargo.toml -type f -not -path './.*' | \
while read -r cf; do
	sed -i "" -E "s/^version = \".*\"/version = \"$version\"/" $cf
done
