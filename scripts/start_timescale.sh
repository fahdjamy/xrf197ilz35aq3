#!/bin/bash

# set -e: If any command in the script fails (returns a non-zero exit code), the entire script will stop immediately.
# set -o pipefail: Normally, in a pipeline of commands (e.g., command1 | command2 the exit code of the entire pipeline
# is determined by the last command. and if any command fails, the whole pipeline is considered failed.
set -eo pipefail

# --- Configuration ---
# Docker container Name
CONTAINER_NAME="timescaledb"
# TimescaleDB Docker image details | Use recommended HA (High Availability) image
IMAGE_NAME="timescale/timescaledb-ha"
# Image version/tag (e.g., pg16, pg17, latest-pg16).
IMAGE_TAG="pg17"

# PostgresSQL DB config.
DB_PASSWORD="your_secure_password"
HOST_PORT="5443"

FULL_IMAGE_NAME="${IMAGE_NAME}:${IMAGE_TAG}"
CONTAINER_PORT="5432" # Default PostgresSQL port inside the container

# 1. Check if Docker command exists
if ! command -v docker &> /dev/null; then
    echo "[ERROR] Docker command not found. Please install Docker."
    exit 1
fi

# check that both psql and sqlx-cli are installed at the very beginning.
if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi


# 2. Check if the required Docker image
echo "Checking for timescale DB docker image: ${FULL_IMAGE_NAME}..."
IMAGE_ID=$(docker images -q "${FULL_IMAGE_NAME}")

if [ -z "$IMAGE_ID" ]; then
    echo "Image '${FULL_IMAGE_NAME}' not found. Pulling..."
    if ! docker pull "${FULL_IMAGE_NAME}" # Pull and also check if pull was successful
    then
        echo "[ERROR] Failed to pull timescale DB docker image '${FULL_IMAGE_NAME}'. Check the image name/tag."
        exit 1
    fi
    echo "Image '${FULL_IMAGE_NAME}' pulled successfully..."
else
    echo "Image '${FULL_IMAGE_NAME}' already exists (ID: ${IMAGE_ID})."
fi

# 3. Check if there's a timescale DB container running
echo "Checking container status for '${CONTAINER_NAME}'..."
RUNNING_CONTAINER_ID=$(docker ps -q -f "name=^/${CONTAINER_NAME}$")

if [ -n "$RUNNING_CONTAINER_ID" ]; then
    echo "[INFO] Container '${CONTAINER_NAME}' is already running (ID: ${RUNNING_CONTAINER_ID}). No action needed."
    exit 0
fi

# 4. Check if a timescale DB container with the name exists but is stopped
STOPPED_CONTAINER_ID=$(docker ps -aq -f status=exited -f "name=^/${CONTAINER_NAME}$")

if [ -n "$STOPPED_CONTAINER_ID" ]; then
    echo "Found stopped container '${CONTAINER_NAME}' (ID: ${STOPPED_CONTAINER_ID}). Re-starting it..."
    docker start "${CONTAINER_NAME}"
    if ! docker start "${CONTAINER_NAME}" # re-start timescale image and also check that no failure occurred
      then
        echo "[ERROR] Failed to start existing container '${CONTAINER_NAME}'."
        exit 1
    fi
    echo "Container '${CONTAINER_NAME}' started successfully."
else
    # 5. If no timescaleDB container exists (running or stopped), create and start a new one
    echo "No running or stopped container named '${CONTAINER_NAME}'. Creating and starting a new one..."

    if ! docker run -d --name "${CONTAINER_NAME}" \
               -p "${HOST_PORT}:${CONTAINER_PORT}" \
               -e POSTGRES_PASSWORD="${DB_PASSWORD}" \
               "${FULL_IMAGE_NAME}" # start timescale image and also check that no failure occurred
      then
        echo "[ERROR] Failed to create or start new container '${CONTAINER_NAME}'. Check Docker daemon status and configuration."
        exit 1
    fi
    echo "New container '${CONTAINER_NAME}' created and started successfully."
    echo "TimescaleDB running on port ${HOST_PORT}."
fi
