#!/bin/bash
echo "SCRIPT EXECUTED at $(date)" >> /tmp/filter_debug.log
echo "GIT_COMMIT: $GIT_COMMIT" >> /tmp/filter_debug.log
COMMIT_MSG="$(git log --format=%s -n 1 $GIT_COMMIT)"
echo "COMMIT_MSG: $COMMIT_MSG" >> /tmp/filter_debug.log
echo "GIT_AUTHOR_DATE: $GIT_AUTHOR_DATE" >> /tmp/filter_debug.log
echo "GIT_COMMITTER_DATE: $GIT_COMMITTER_DATE" >> /tmp/filter_debug.log

if [[ "$COMMIT_MSG" == *"feat: add Meilisearch and NATS configuration for search and messaging"* ]]; then
    echo "SKIPPING Meilisearch commit" >> /tmp/filter_debug.log
    exit 0
fi

echo "PROCESSING commit" >> /tmp/filter_debug.log

# Process GIT_AUTHOR_DATE
if [[ -n "$GIT_AUTHOR_DATE" ]]; then
    timestamp_part="${GIT_AUTHOR_DATE#@}"
    timestamp_part="${timestamp_part%% *}"
    timezone_part="${GIT_AUTHOR_DATE#* }"
    
    new_timestamp=$((timestamp_part + 10368000))
    export GIT_AUTHOR_DATE="@$new_timestamp $timezone_part"
    echo "NEW GIT_AUTHOR_DATE: $GIT_AUTHOR_DATE" >> /tmp/filter_debug.log
fi

# Process GIT_COMMITTER_DATE
if [[ -n "$GIT_COMMITTER_DATE" ]]; then
    timestamp_part="${GIT_COMMITTER_DATE#@}"
    timestamp_part="${timestamp_part%% *}"
    timezone_part="${GIT_COMMITTER_DATE#* }"
    
    new_timestamp=$((timestamp_part + 10368000))
    export GIT_COMMITTER_DATE="@$new_timestamp $timezone_part"
    echo "NEW GIT_COMMITTER_DATE: $GIT_COMMITTER_DATE" >> /tmp/filter_debug.log
fi

echo "---" >> /tmp/filter_debug.log
