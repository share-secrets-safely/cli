#!/bin/bash

cat <<DOCKERFILE
from ${BASE_IMAGE:?}
run apk del gnupg
run apk -U --no-cache add curl alpine-sdk
run mkdir -p gnupg1 && \\
    curl https://gnupg.org/ftp/gcrypt/gnupg/gnupg-1.4.22.tar.bz2 \\
    | tar -jx -C gnupg1 --strip-components 1
run cd gnupg1 && \\
    ./configure && \\
    make && \\
    make install && \\
    cd / && \\
    rm -Rf gnupg1
DOCKERFILE
