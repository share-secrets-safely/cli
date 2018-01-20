#!/bin/bash

cat <<DOCKERFILE
from ${BASE_IMAGE:?}
run apk del gnupg
DOCKERFILE
