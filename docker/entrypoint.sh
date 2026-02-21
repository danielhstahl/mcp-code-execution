#!/bin/sh

if [ "$TYPE" = "requirements.txt" ]; then
    # Installing via pip
    python -m pip install -r requirements.txt
    python "$1"
elif [ "$TYPE" = "uv" ]; then
    # Syncing via uv
    uv sync
    uv run python "$1"
else
    # No dependencies selected (running raw python)
    python "$1"
fi
