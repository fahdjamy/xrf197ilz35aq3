#!/bin/bash

# --- Configuration ---
TIMESCALE_CONTAINER_NAME="timescaledb"

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

# 3. Stop Running Redis Containers (Alternative: Filter by Image Name)
echo ""
echo "Attempting to stop running Redis containers (checking image names containing 'redis')..."

# Get IDs and Image names of all running containers
# Using a loop to process each line from docker ps output
STOP_COUNT=0
FOUND_REDIS=0

# Use process substitution to feed the while loop
# i.e Keep looping as long as you can read a full line of input exactly as it is (without interpreting backslashes or
# trimming leading/trailing whitespace), and store that complete line in the variable named line
while IFS= read -r line; do
    # Extract ID and Image name robustly, handling potential spaces
    id=$(echo "$line" | awk '{print $1}')
    image_name=$(echo "$line" | awk '{print $2}')

    # Check if the image name CONTAINS 'redis' (case-insensitive check can be added if needed)
    # You might want to make this more specific, e.g., check for redis: or /redis
    if [[ "$image_name" == *"redis"* ]]; then
        FOUND_REDIS=1
        # Lists running Docker containers, -f only show information for the container whose ID exactly matches the
        # value currently stored in the Bash variable ${id}
        # --no-trunc: prevents Docker from shortening potentially long output like container or image names. Get full values
        # example out put my-redis-container (Image: redis:latest)
        CONTAINER_INFO=$(docker ps --no-trunc -f "id=${id}" --format "{{.Names}} (Image: {{.Image}})")
        echo "  Found potential Redis container: ${CONTAINER_INFO}"
        echo "  Stopping Redis container ID: ${id}"
        if docker stop "$id"; then
            echo "  Successfully stopped ${id}."
            STOP_COUNT=$((STOP_COUNT + 1))
        else
            echo "  [WARNING] Failed to stop Redis container ${id}. Maybe already stopped or error occurred?"
        fi
    fi
done < <(docker ps --format "{{.ID}} {{.Image}}") # Feed docker ps output into the loop

if [ $FOUND_REDIS -eq 0 ]; then
     echo "No running containers found with 'redis' in their image name."
else
    echo "Finished stopping ${STOP_COUNT} found Redis container(s)."
fi

echo ""
echo "!!!!!!!! Stopping DB containers FINISHED !!!!!!!!"
exit 0
