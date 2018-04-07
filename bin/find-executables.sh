#!/bin/bash
set -eu

directory="${1:?First argument must be the directory containing executables}"

for f in $directory/*; do 
  [[ -x "$f" && -f "$f" ]] && echo "${f##*/}"
done

exit 0
