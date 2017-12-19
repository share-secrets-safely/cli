#!/bin/bash

set -u
exe=${1:?First argument is the executable under test}
image=${2:?Second argument is the docker image to use as basis}

relative_root="${0%/*}"
absolute_root="$(cd  "$relative_root/.." && pwd)"
RED="$(tput setaf 1)"

function in_red() {
  echo "$RED" "$@"
}

find "$relative_root/journeys" -type f -name "*.sh" -maxdepth 1 | sort | \
while read -r journey; do
    make_journey_dockerfile="${journey%/*}/dockerfile-gen/${journey##*/}"
    journey_image="$image"
    if [[ -x "$make_journey_dockerfile" ]]; then
        journey_image="${journey##*/}"
        BASE_IMAGE="$image" "$make_journey_dockerfile" | docker build -t "$journey_image" -
    else
        echo 1>&2 "Custom docker image supported via script at 'BASE_IMAGE=$image $make_journey_dockerfile'"
    fi

    docker_args=( docker run -v "$absolute_root:/volume" --rm -t "$journey_image" "$journey" "$exe" )
    in_red "Running '${docker_args[*]}'"
    eval "${docker_args[@]}"
done



