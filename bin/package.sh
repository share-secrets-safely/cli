#!/bin/bash

set -eu

directory="${1:?First argument must be the directory containing executables}"
output=${2:?Second argument is the package file to create}

root="$(cd "${0%/*}" && pwd)"
exeFiles=( `$root/find-executables.sh "$directory"` )

(
  cd "$directory"
  tar czf "$output" "${exeFiles[@]}"
)
mv "$directory/$output" .
gpg --yes --output "$output".gpg --detach-sig "$output"
