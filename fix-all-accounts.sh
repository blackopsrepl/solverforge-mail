#!/bin/bash
# Fix all email accounts - store passwords in keyring and configure OAuth

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/setup-common.sh"

echo "╔════════════════════════════════════════════╗"
echo "║     Fixing All Email Accounts              ║"
echo "╚════════════════════════════════════════════╝"
echo ""

# Detect configured accounts
readarray -t accounts < <($HIMALAYA -o json account list 2>/dev/null | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    for acc in data:
        name = acc.get('name', '')
        if name:
            print(name)
except:
    pass
" 2>/dev/null)

if [ ${#accounts[@]} -eq 0 ]; then
    echo "No accounts found in himalaya config."
    exit 1
fi

# Show current status
echo "Current status:"
for acct in "${accounts[@]}"; do
    printf "  %-15s: " "$acct"
    if $HIMALAYA -o json folder list -a "$acct" >/dev/null 2>&1; then
        echo "✓ Working"
    else
        echo "✗ Not working"
    fi
done
echo ""

# Fix each non-working account
for acct in "${accounts[@]}"; do
    if $HIMALAYA -o json folder list -a "$acct" >/dev/null 2>&1; then
        continue
    fi

    echo "────────────────────────────────────────"
    echo "Fix account: $acct"
    echo ""
    echo "How should this account authenticate?"
    echo "  1) Password (IMAP/SMTP)"
    echo "  2) OAuth (browser flow)"
    echo "  3) Skip"
    read -rp "Choice [1-3]: " auth_choice

    case $auth_choice in
        1)
            read -rp "Enter password for $acct: " -s pass
            echo
            echo -n "$pass" | secret-tool store --label="$acct IMAP" service "${acct}-imap"
            echo -n "$pass" | secret-tool store --label="$acct SMTP" service "${acct}-smtp"
            echo "✓ Password stored for $acct"

            if $HIMALAYA -o json folder list -a "$acct" >/dev/null 2>&1; then
                echo "✓ $acct is working!"
            else
                echo "✗ $acct auth failed - check password"
            fi
            ;;
        2)
            echo "Starting OAuth setup for $acct..."
            $HIMALAYA account configure "$acct" || true
            ;;
        3)
            echo "Skipping $acct"
            ;;
    esac
    echo ""
done

echo "═══════════════════════════════════════════════"
echo "ACCOUNT STATUS:"
echo "═══════════════════════════════════════════════"

for acct in "${accounts[@]}"; do
    printf "  %-15s: " "$acct"
    if $HIMALAYA -o json folder list -a "$acct" >/dev/null 2>&1; then
        echo "✓ Working"
    else
        echo "✗ Not working"
    fi
done

echo ""
echo "Launch SolverForge Mail: $SCRIPT_DIR/target/release/solverforge-mail"
