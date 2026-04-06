#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

find_solverforge_mail() {
    local repo_release="$SCRIPT_DIR/target/release/solverforge-mail"
    local repo_debug="$SCRIPT_DIR/target/debug/solverforge-mail"
    local installed_sibling="$SCRIPT_DIR/../bin/solverforge-mail"
    local installed_home="$HOME/.local/share/solverforge/bin/solverforge-mail"

    if [ -x "$repo_release" ]; then
        printf "%s\n" "$repo_release"
    elif [ -x "$repo_debug" ]; then
        printf "%s\n" "$repo_debug"
    elif [ -x "$installed_sibling" ]; then
        printf "%s\n" "$installed_sibling"
    elif [ -x "$installed_home" ]; then
        printf "%s\n" "$installed_home"
    elif command -v solverforge-mail >/dev/null 2>&1; then
        command -v solverforge-mail
    else
        printf "\n"
    fi
}

SOLVERFORGE_MAIL_BIN="$(find_solverforge_mail)"

if [ -z "$SOLVERFORGE_MAIL_BIN" ]; then
    echo "SolverForge Mail binary not found."
    echo "Build it first with 'cargo build --release' or install it."
    exit 1
fi

exec "$SOLVERFORGE_MAIL_BIN" --setup "$@"
