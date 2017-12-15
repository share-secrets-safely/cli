#!/bin/bash

set -u
exe=${1:?First argument is the executable under test}
image=${2:?Second argument is the docker image to use as basis}

relative_root="${0%/*}"
absolute_root="$(cd  "$relative_root/.." && pwd)"

find "$relative_root/journeys" -type f -name "*.sh" -maxdepth 1 | sort | \
while read -r journey; do
    make_journey_dockerfile="${journey%/*}/dockerfile-gen/${journey##*/}"
    journey_image="$image"
    if [[ -x "$make_journey_dockerfile" ]]; then
        journey_image="$journey_image"
        BASE_IMAGE="$image" "$make_journey_dockerfile" | docker build -t "$journey_image" -
    fi

    docker run -v "$absolute_root:/volume" --rm -t "$journey_image" "$journey" "$exe"
done



