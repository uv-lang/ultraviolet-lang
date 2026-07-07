#!/bin/sh
set -e

ROOT_DIR="$(pwd)"
MODE="${1:-release}"

if [ "$MODE" != "release" ] && [ "$MODE" != "debug" ]; then
    echo "Usage: $0 [release|debug]"
    exit 1
fi

echo "Build mode: $MODE"

for dir in "$ROOT_DIR"/libs/*; do
    if [ -d "$dir" ] && [ -f "$dir/build.sh" ]; then
        echo "==> Building $(basename "$dir")"

        (
            cd "$dir"
            sh ./build.sh "$ROOT_DIR" "$MODE"
        )
    fi
done