#!/bin/bash

# 1. Check if Docker command exists
if ! command -v docker &> /dev/null; then
    echo "[ERROR] Docker command not found. Please install Docker."
    exit 1
fi

# ------ Configurations -------
CONTAINER_NAME="cassandra"
IMAGE_NAME="cassandra"
IMAGE_TAG="5.0.4"
# Local host port to map to the container's Cassandra CQL port (9042)
# If you have another service using port 9042 locally, change this (e.g., 9043)
HOST_CQL_PORT="9044"

FULL_IMAGE_NAME="${IMAGE_NAME}:${IMAGE_TAG}"
CONTAINER_CQL_PORT="9042" # Default Cassandra CQL native protocol port

# --- End Configuration ---

echo "!!!!!!!! Setting up Cassandra container !!!!!!!!"

# 2. Check if the required Docker image exists locally
echo "Checking for Docker image: ${FULL_IMAGE_NAME}..."
IMAGE_ID=$(docker images -q "${FULL_IMAGE_NAME}")

if [ -z "$IMAGE_ID" ]; then
    echo "cassandra image '${FULL_IMAGE_NAME}' not found locally. Pulling..."
    # Check if pull was successful
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
echo "Checking cassandra container status for '${CONTAINER_NAME}'..."
RUNNING_CONTAINER_ID=$(docker ps -q -f "name=^/${CONTAINER_NAME}$")

if [ -n "$RUNNING_CONTAINER_ID" ]; then
    echo "[INFO] Container '${CONTAINER_NAME}' is already running (ID: ${RUNNING_CONTAINER_ID}). No action needed."
    # Optional: Add a check here to see if Cassandra is ready within the container if needed
    echo "****** Cassandra container setup Complete ******"
    exit 0
fi

# 4. Check if a container with the name exists but is stopped
STOPPED_CONTAINER_ID=$(docker ps -aq -f status=exited -f "name=^/${CONTAINER_NAME}$")

if [ -n "$STOPPED_CONTAINER_ID" ]; then
    echo "Found stopped container '${CONTAINER_NAME}' (ID: ${STOPPED_CONTAINER_ID}). Starting it..."
    docker start "${CONTAINER_NAME}"
    if ! docker start "${CONTAINER_NAME}"; then
        echo "[ERROR] Failed to start existing container '${CONTAINER_NAME}'."
        exit 1
    fi
    echo "Container '${CONTAINER_NAME}' started successfully."
    echo "Cassandra may take a minute or two to fully initialize after starting."
else
    # 5. If container doesn't exist (neither running nor stopped), create and start a new one
    echo "No running or stopped container named '${CONTAINER_NAME}' found. Creating and starting a new one..."
    # Note: For multi-node clusters, more complex networking and configuration is needed.
    # This starts a single node, suitable for development/testing.
    # Adding -m 2g to limit memory, adjust as needed. Remove if you want default Docker limits.

    if ! docker run -d --name "${CONTAINER_NAME}" \
               -p "${HOST_CQL_PORT}:${CONTAINER_CQL_PORT}" \
               "${FULL_IMAGE_NAME}";
      then
        echo "[ERROR] Failed to create or start new container '${CONTAINER_NAME}'. Check Docker daemon status and configuration."
        exit 1
    fi
    echo "Cassandra container '${CONTAINER_NAME}' created and started successfully."
    echo "Cassandra native protocol (CQL) should be accessible locally on port ${HOST_CQL_PORT}."
    echo "Cassandra may take a minute or two to fully initialize and be ready for connections."
fi

echo "****** Cassandra container setup Complete ******"
exit 0
