#!/usr/bin/env bash

# shellcheck source=/dev/null
source ./bin/generator-utils/utils.sh
source ./bin/generator-utils/prompts.sh
source ./bin/generator-utils/templates.sh

# Exit if anything fails.
set -euo pipefail

# If argument not provided, print usage and exit
if [ $# -ne 1 ] && [ $# -ne 2 ] && [ $# -ne 3 ]; then
    echo "Usage: bin/generate_practice_exercise.sh <exercise-slug> [difficulty] [author-github-handle]"
    exit 1
fi

# Check if sed is gnu-sed
if ! sed --version | grep -q "GNU sed"; then
    echo "GNU sed is required. Please install it and make sure it's in your PATH."
    exit 1
fi

# Check if jq and curl are installed
command -v jq >/dev/null 2>&1 || {
    echo >&2 "jq is required but not installed. Aborting."
    exit 1
}
command -v curl >/dev/null 2>&1 || {
    echo >&2 "curl is required but not installed. Aborting."
    exit 1
}

# Check if exercise exists in configlet info
check_exercise_existence "$1"

# ==================================================

SLUG="$1"
UNDERSCORED_SLUG=$(dash_to_underscore "$SLUG")
EXERCISE_DIR="exercises/practice/${SLUG}"
EXERCISE_NAME=$(format_exercise_name "$SLUG")
message "info" "Using ${YELLOW}${EXERCISE_NAME}${BLUE} as a default exercise name. You can edit this later in the config.json file"
# using default value for difficulty
EXERCISE_DIFFICULTY=$(validate_difficulty_input "${2:-$(get_exercise_difficulty)}")
message "info" "The exercise difficulty has been set to ${YELLOW}${EXERCISE_DIFFICULTY}${BLUE}. You can edit this later in the config.json file"
# using default value for author
AUTHOR_HANDLE=${3:-$(get_author_handle)}
message "info" "Using ${YELLOW}${AUTHOR_HANDLE}${BLUE} as author's handle. You can edit this later in the 'authors' field in the ${EXERCISE_DIR}/.meta/config.json file"

message "info" "Creating Rust files"
cargo new --lib "$EXERCISE_DIR" -q
mkdir -p "$EXERCISE_DIR"/tests
touch "${EXERCISE_DIR}/tests/${SLUG}.rs"

create_test_file_template "$EXERCISE_DIR" "$SLUG"
create_lib_rs_template "$EXERCISE_DIR" "$SLUG"
create_example_rs_template "$EXERCISE_DIR" "$SLUG"
overwrite_gitignore "$EXERCISE_DIR"

message "success" "Created Rust files succesfully!"

# ==================================================

# build configlet
./bin/fetch-configlet
message "success" "Fetched configlet successfully!"

# Preparing config.json
message "info" "Adding instructions and configuration files..."
UUID=$(bin/configlet uuid)

jq --arg slug "$SLUG" --arg uuid "$UUID" --arg name "$EXERCISE_NAME" --arg difficulty "$EXERCISE_DIFFICULTY" \
    '.exercises.practice += [{slug: $slug, name: $name, uuid: $uuid, practices: [], prerequisites: [], difficulty: $difficulty}]' \
    config.json >config.json.tmp
# jq always rounds whole numbers, but average_run_time needs to be a float
sed -i 's/"average_run_time": \([0-9]\+\)$/"average_run_time": \1.0/' config.json.tmp
mv config.json.tmp config.json
message "success" "Added instructions and configuration files"

# Create instructions and config files
echo "Creating instructions and config files"
./bin/configlet sync --update --yes --docs --metadata --exercise "$SLUG"
./bin/configlet sync --update --yes --filepaths --exercise "$SLUG"
./bin/configlet sync --update --tests include --exercise "$SLUG"
message "success" "Created instructions and config files"

META_CONFIG="$EXERCISE_DIR"/.meta/config.json
jq --arg author "$AUTHOR_HANDLE" '.authors += [$author]' "$META_CONFIG" >"$META_CONFIG".tmp && mv "$META_CONFIG".tmp "$META_CONFIG"
message "success" "You've been added as the author of this exercise."

sed -i "s/name = \".*\"/name = \"$UNDERSCORED_SLUG\"/" "$EXERCISE_DIR"/Cargo.toml

message "done" "All stub files were created."

message "info" "After implementing the solution, tests and configuration, please run:"
echo "./bin/configlet fmt --update --yes --exercise ${SLUG}"
