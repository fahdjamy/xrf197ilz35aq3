#!/usr/bin/env bash

set -eo pipefail

# 1. Check if Docker command exists
if ! command -v docker &> /dev/null; then
    echo "[ERROR] Docker command not found. Please install Docker."
    exit 1
fi

# ------ Configurations -------
IMAGE_TAG="5.0.4"
HOST_CQL_PORT="9042"
IMAGE_NAME="cassandra"
CONTAINER_CQL_PORT="9042"

CAS_CONTAINER_NAME="cassandra_q3_c"
FULL_IMAGE_NAME="${IMAGE_NAME}:${IMAGE_TAG}"

# 2. Check if the required Docker image
echo "Checking for cassandra DB docker image: name='${FULL_IMAGE_NAME}'"
IMAGE_ID=$(docker images -q "${FULL_IMAGE_NAME}")

if [ -z "$IMAGE_ID" ]; then
    echo "Image '${FULL_IMAGE_NAME}' not found. Pulling..."
    if ! docker pull "${FULL_IMAGE_NAME}" # Pull and also check if pull was successful
    then
        echo "[ERROR] Failed to pull cassandra image '${FULL_IMAGE_NAME}'. Check the image name/tag."
        exit 1
    fi
    echo "cassandra image '${FULL_IMAGE_NAME}' pulled successfully..."
else
    echo "cassandra image '${FULL_IMAGE_NAME}' already exists (ID: ${IMAGE_ID})."
fi

# 3. Check if there's a timescale DB container running
echo "Checking cassandra container status for '${CAS_CONTAINER_NAME}'..."
RUNNING_CONTAINER_ID=$(docker ps -q -f "name=^/${CAS_CONTAINER_NAME}$")

if [ -n "$RUNNING_CONTAINER_ID" ]; then
    echo "[INFO] Container '${CAS_CONTAINER_NAME}' is already running (ID: ${RUNNING_CONTAINER_ID}). No action needed."
    exit 0
fi

# 4. Check if a cassandra DB container with the name exists but is stopped
STOPPED_CONTAINER_ID=$(docker ps -aq -f status=exited -f "name=^/${CAS_CONTAINER_NAME}$")

if [ -n "$STOPPED_CONTAINER_ID" ]; then
    echo "Found stopped container '${CAS_CONTAINER_NAME}' (ID: ${STOPPED_CONTAINER_ID}). Re-starting it..."
    docker start "${CAS_CONTAINER_NAME}"
    if ! docker start "${CAS_CONTAINER_NAME}" # re-start cassandra container and also check that no failure occurred
      then
        echo "[ERROR] Failed to start existing container '${CAS_CONTAINER_NAME}'."
        exit 1
    fi
    echo "Container '${CAS_CONTAINER_NAME}' started successfully."
else
    # 5. If no cassandra container exists (running/stopped), create and start a new one
    echo "No running or stopped container named '${CAS_CONTAINER_NAME}'. Creating and starting a new one..."

    if ! docker run -d --name "${CAS_CONTAINER_NAME}" \
               -p "${HOST_CQL_PORT}:${CONTAINER_CQL_PORT}" \
               -m 2g \
               "${FULL_IMAGE_NAME}"
      then
        echo "[ERROR] Failed to start a cassandra container '${CAS_CONTAINER_NAME}'. Check Docker daemon status and configuration."
        exit 1
    fi
    echo "Cassandra container '${CAS_CONTAINER_NAME}' created and started successfully."
    echo "Cassandra native protocol (CQL) should be accessible locally on port ${HOST_CQL_PORT}."
    echo "cassandra running on port '${HOST_PORT}' ..."
fi

echo "--- Done | Start cassandra script ---"
exit 0
