#!/bin/bash

cat <<DOCKERFILE
from ${1:?}
run apk del gpg
DOCKERFILE
