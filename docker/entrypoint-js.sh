#!/bin/sh

if [ "$TYPE" = "npm" ]; then
    # Installing via npm
    npm ci
    node "$1"
elif [ "$TYPE" = "uv" ]; then
    # Syncing via yarn
    yarn install --immutable
    node "$1"
else
    # No dependencies selected (running raw node)
    node "$1"
fi
