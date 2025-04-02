#!/opt/homebrew/bin/bash

# replace shebang command above to point to your bash path. e.g #!/bin/bash

set -e # Exits the script immediately if any command fails
# Treat unset variables as an error when substituting.
# set -u # Optional: Uncomment this for stricter checking, but the explicit check below is often clearer.

# Ensures that if any command in a pipeline (like cargo run | bunyan) fails, the pipeline's exit status reflects that failure.
# Without it, only the exit status of the last command (bunyan) would be considered
set -o pipefail

# --- Configuration ---
# Define the required environment variables and their default values
declare -A REQUIRED_ENV_VARS=(
  [XRF_Q3_PG_DB_URL]="postgres://user:password@localhost/default_db"
  [XRF_Q3_API_KEY]="not_secure_api_key_ilz_aq_12345"
  [XRF_Q3_PORT]="8089"
  [RUST_LOG]="info"
)
# --- End Configuration ---

needs_defaults=false

echo "Checking required environment variables..."

# Loop through the keys (variable names) of the associative array
for var_name in "${!REQUIRED_ENV_VARS[@]}"; do
  # Check if the variable is unset or empty
  # Using indirect expansion: ${!var_name} gets the value of the variable whose name is stored in var_name
  if [ -z "${!var_name}" ]; then
    echo "-> Variable '$var_name' is not set or is empty."
    needs_defaults=true
  else
    echo "-> Variable '$var_name' is set."
  fi
done

# If any variable was found to be missing or empty, set all defined defaults
if [ "$needs_defaults" = true ]; then
  echo "----------------------------------------"
  echo "One or more required variables were missing. Setting defaults..."
  echo "----------------------------------------"
  for var_name in "${!REQUIRED_ENV_VARS[@]}"; do
    default_value="${REQUIRED_ENV_VARS[$var_name]}"
    # Export the variable so it's available to the child process (cargo run)
    export "$var_name"="$default_value"
    echo "   Exported $var_name=$default_value"
  done
else
  echo "----------------------------------------"
  echo "All required variables are already set."
  echo "----------------------------------------"
fi

# --- Execution ---
echo # Add a blank line for readability
echo "Starting Rust application..."
echo "Command: cargo run -q | bunyan"
echo "----------------------------------------"

# Run the cargo command, piping output to bunyan
# The '-q' flag suppresses messages like 'Compiling', 'Running', etc. from cargo
cargo run -q | bunyan

# Capture the exit status of the pipeline
# $? contains the exit status of the last command (bunyan in this case)
# Because of 'set -o pipe fail', this will be non-zero if *either* cargo run *or* bunyan fails.
exit_status=$?

echo "----------------------------------------"
if [ $exit_status -ne 0 ]; then
  echo "Error: Application exited with status $exit_status"
  exit $exit_status
else
  echo "Application finished."
fi

exit 0
