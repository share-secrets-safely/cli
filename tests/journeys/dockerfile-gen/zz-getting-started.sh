#!/bin/bash

cat <<DOCKERFILE
from alpine:latest

run apk -U --no-cache add git bash
DOCKERFILE
