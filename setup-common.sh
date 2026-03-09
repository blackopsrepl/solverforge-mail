#!/bin/bash
# Common setup for all setup-* scripts: locate himalaya binary

# Find himalaya binary (mirrors the Rust lookup in src/himalaya/config.rs)
find_himalaya() {
    if command -v solverforge-himalaya &>/dev/null; then
        echo "solverforge-himalaya"
    elif command -v himalaya &>/dev/null; then
        echo "himalaya"
    elif [ -x /opt/himalaya/target/release/himalaya ]; then
        echo "/opt/himalaya/target/release/himalaya"
    else
        echo ""
    fi
}

HIMALAYA="$(find_himalaya)"
if [ -z "$HIMALAYA" ]; then
    echo "ERROR: Could not find himalaya binary."
    echo "Install himalaya or ensure it is on your PATH."
    exit 1
fi
