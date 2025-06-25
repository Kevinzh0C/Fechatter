#!/usr/bin/env python3
import os
import sys

# Get the commit message for this commit
commit_message = os.environ.get('GIT_COMMIT_MESSAGE', '')

# Skip the specific commit we don't want to modify
if "feat: add Meilisearch and NATS configuration for search and messaging" in commit_message:
    sys.exit(0)

# Process GIT_AUTHOR_DATE
if 'GIT_AUTHOR_DATE' in os.environ:
    author_date = os.environ['GIT_AUTHOR_DATE']
    if '@' in author_date:
        timestamp_part, timezone_part = author_date.split('@', 1)
        timestamp_part = timestamp_part.strip()
        
        try:
            # Add 120 days in seconds (4 months approximation)
            new_timestamp = int(timestamp_part) + 10368000
            os.environ['GIT_AUTHOR_DATE'] = f"{new_timestamp} @{timezone_part}"
        except ValueError:
            pass

# Process GIT_COMMITTER_DATE
if 'GIT_COMMITTER_DATE' in os.environ:
    committer_date = os.environ['GIT_COMMITTER_DATE']
    if '@' in committer_date:
        timestamp_part, timezone_part = committer_date.split('@', 1)
        timestamp_part = timestamp_part.strip()
        
        try:
            # Add 120 days in seconds (4 months approximation)
            new_timestamp = int(timestamp_part) + 10368000
            os.environ['GIT_COMMITTER_DATE'] = f"{new_timestamp} @{timezone_part}"
        except ValueError:
            pass
