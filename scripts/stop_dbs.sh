#!/bin/bash

# --- Configuration ---
TIMESCALE_CONTAINER_NAME="timescaledb"

# List of Redis image names/patterns to check as ancestors.
# The script will stop containers derived from any image in this list.
REDIS_ANCESTORS=("redis" "redis/redis-stack")

echo "--- Stopping database images | Start time: $(date) ---"

# 1. Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "[ERROR] Docker command not found. Please install Docker."
    exit 1
fi

# 2. Stop TimescaleDB Container by Name
echo ""
echo "Attempting to stop TimescaleDB container: '${TIMESCALE_CONTAINER_NAME}'..."
# Find the container ID using the exact name match filter (^/name$)
TS_DB_ID=$(docker ps -q -f "name=^/${TIMESCALE_CONTAINER_NAME}$")

if [ -n "$TS_DB_ID" ]; then
    echo "Found running TimescaleDB container (ID: ${TS_DB_ID}). Stopping..."
    if docker stop "$TS_DB_ID"; then
        echo "TimescaleDB container '${TIMESCALE_CONTAINER_NAME}' stopped successfully."
    else
        # Potentially already stopped between check and command, or error
        echo "[WARNING] Failed to stop TimescaleDB container '${TIMESCALE_CONTAINER_NAME}' (ID: ${TS_DB_ID})"
        echo "!!! Maybe already stopped or error occurred !!!"
    fi
else
    echo "No running container found with the exact name '${TIMESCALE_CONTAINER_NAME}'."
fi

# 3. Stop Running Redis Containers by Ancestor Image
echo ""
echo "Attempting to stop running Redis containers (checking ancestors: ${REDIS_ANCESTORS[*]})..."

# Build filter arguments dynamically for docker ps
FILTER_ARGS=()
for ancestor in "${REDIS_ANCESTORS[@]}"; do
    FILTER_ARGS+=("--filter" "ancestor=${ancestor}")
done

# Find running Redis container IDs using the combined filters
# If FILTER_ARGS is empty, this will find nothing specific (prevents errors)
REDIS_IDS=""
if [ ${#FILTER_ARGS[@]} -gt 0 ]; then
    REDIS_IDS=$(docker ps -q "${FILTER_ARGS[@]}")
fi

if [ -z "$REDIS_IDS" ]; then
    echo "No running Redis containers images found."
else
    echo "Attempting to stop found running Redis containers..."
    STOP_COUNT=0
    # Loop through each found ID safely
    echo "$REDIS_IDS" | while IFS= read -r id; do
        # Get container name/image for better logging
        CONTAINER_INFO=$(docker ps --no-trunc -f "id=${id}" --format "{{.Names}} (Image: {{.Image}})")
        echo "  Stopping Redis container ID: ${id} - ${CONTAINER_INFO}..."
        if docker stop "$id"; then
            echo "  Successfully stopped ${id}."
            STOP_COUNT=$((STOP_COUNT + 1))
        else
            echo "  [WARNING] Failed to stop Redis container ${id}. Maybe already stopped or error occurred?"
        fi
    done
    echo "Finished stopping ${STOP_COUNT} Redis container(s)."
fi

echo ""
echo "--- Script Finished ---"
exit 0
