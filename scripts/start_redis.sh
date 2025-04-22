#!/usr/bin/env bash
set -x
set -eo pipefail

# 1. Check if Docker command exists
if ! command -v docker &> /dev/null; then
    echo "[ERROR] Docker command not found. Please install Docker."
    exit 1
fi

# if a redis container is running, print instructions to kill it and exit
RUNNING_CONTAINER=$(docker ps --filter 'name=redis' --format '{{.ID}}')
if [[ -n $RUNNING_CONTAINER ]]; then
  echo >&2 "there is a redis container already running, kill it with"
  echo >&2 "    docker kill ${RUNNING_CONTAINER}"
  exit 1
fi

# Launch Redis using Docker
docker run \
  -p "6379:6379" \
  -d \
  --name "redis_$(date '+%s')" \
  redis:7

>&2 echo "Redis is running...!"
