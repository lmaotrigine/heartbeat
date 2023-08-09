#!/bin/bash

NAME="${HB_PROJECT_NAME:-ghcr.io/lmaotrigine/heartbeat}"
TAG="$(git rev-parse --short HEAD)"
IMG="${NAME}:${TAG}"
LATEST="${NAME}:latest"

_dir="$(dirname "$(dirname "$(dirname "$0")")")"

docker build -t "${NAME}" "${_dir}"
docker tag "${IMG}" "${IMG}"
docker tag "${IMG}" "${LATEST}"
docker push -a "${NAME}"
docker rmi "${IMG}"
