#!/bin/bash
set -eu -o pipefail

function usage() {
  cat <<EOF
$0 build <src_dir> <book_dir>
  Build all *.chapter.sh files in the given <src_dir> and write mdbook compatible
  chapter files to <book_dir>. Then execute \`mdbook build\` and output the directory
  with the build book.
EOF
}

function main () {
  if [[ $# -eq 0 ]]; then
    usage
    return 1
  fi

  local mode="${1:?The first argument is the mode of operation}"
  case "$mode" in
    build)
      ;;
    *)
      usage
      return 1
      ;;
  esac

  local src_dir="${2:?The second argument is the directory containing the shell-book source files}"
  local book_dir="${3:?The third argument is the directory into which to generate the book}"

  src_dir="$(cd "$src_dir" && pwd)"

  echo 1>&2 "build book"
  if [[ -e "$book_dir" ]]; then
    echo 1>&2 "Cannot overwrite existing directory"
    return 1
  fi

  local out_dir="$book_dir/static-html"
  mkdir -p "$book_dir"

  book_dir="$(cd "$book_dir" && pwd)"
  cat <<CONFIG > "$book_dir/book.toml"
[book]
title = "Share Secrets Safely"
authors = ["Sebastian Thiel"]
description = "Learn how to share secrets safely with this book."

[build]
build-dir = "$out_dir"
create-missing = false
CONFIG

  find "$src_dir" -type f -name "*.chapter.sh" | sort | {
    local num_chapters=0
    while read -rd chapter_path; do
      local relative_chapter="${chapter_path#${src_dir/}}"
      echo 1>&2 "Processing chapter '${relative_chapter%.chapter.sh}'"
      # shellcheck disable=1090
      source "$chapter_path"
      num_chapters=$(( num_chapters + 1 ))
    done
    if [[ $num_chapters -eq 0 ]]; then
      echo 1>&2 "Didn't find a single chapter in '${src_dir}'."
      return 1
    fi
  }
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
