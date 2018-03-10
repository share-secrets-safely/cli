#!/bin/bash

set -eu -o pipefail

set -x
version=${1:?First argument is the version you want to tag, like '1.0.0'}
notes_path=${2:?Second argument a file path to the release notes file to include.}

find . -name Cargo.toml -type f -not -path './.*' | \
while read -r cf; do
	sed -i "" -E "s/^version = \".*\"/version = \"$version\"/" "$cf"
done

git commit -am "bumping version to $version"
git tag -s -F "${notes_path}" "$version"

for lib in lib/substitute lib/vault .; do
  (
    cd $lib
    cargo publish
  )
done

git push --tags origin master
