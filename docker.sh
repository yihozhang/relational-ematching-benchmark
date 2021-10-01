#!/usr/bin/env bash

if command -v podman
then
  docker=podman
elif command -v docker
then
  docker=docker
else
  echo "Can't find docker"
  exit 1
fi

image=relational-ematching

# the image workdir
workdir=/usr/src/app

set -ev

$docker build -t $image .
$docker run --rm -v "$PWD":$workdir:z -w $workdir -it $image $@