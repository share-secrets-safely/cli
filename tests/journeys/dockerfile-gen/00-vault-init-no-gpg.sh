#!/bin/bash

cat <<DOCKERFILE
from alpine:latest
run apk -U --no-cache add bash ncurses
DOCKERFILE
