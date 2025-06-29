#!/bin/bash

# Get the commit message for this commit
COMMIT_MSG="$(git log --format=%s -n 1 $GIT_COMMIT)"

# Skip the specific commit we don't want to modify
if [[ "$COMMIT_MSG" == *"feat: add Meilisearch and NATS configuration for search and messaging"* ]]; then
    exit 0
fi

# Process GIT_AUTHOR_DATE
if [[ -n "$GIT_AUTHOR_DATE" ]]; then
    # Remove the @ symbol and extract timestamp
    timestamp_part="${GIT_AUTHOR_DATE#@}"
    timestamp_part="${timestamp_part%% *}"
    # Extract timezone (everything after the first space)
    timezone_part="${GIT_AUTHOR_DATE#* }"
    
    # Add 120 days in seconds (4 months approximation)
    new_timestamp=$((timestamp_part + 10368000))
    export GIT_AUTHOR_DATE="@$new_timestamp $timezone_part"
fi

# Process GIT_COMMITTER_DATE
if [[ -n "$GIT_COMMITTER_DATE" ]]; then
    # Remove the @ symbol and extract timestamp
    timestamp_part="${GIT_COMMITTER_DATE#@}"
    timestamp_part="${timestamp_part%% *}"
    # Extract timezone (everything after the first space)
    timezone_part="${GIT_COMMITTER_DATE#* }"
    
    # Add 120 days in seconds (4 months approximation)
    new_timestamp=$((timestamp_part + 10368000))
    export GIT_COMMITTER_DATE="@$new_timestamp $timezone_part"
fi
