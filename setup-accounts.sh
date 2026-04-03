#!/bin/bash
# Master account setup script for SolverForge Mail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/setup-common.sh"

echo "╔════════════════════════════════════════════╗"
echo "║     SolverForge Mail - Account Setup       ║"
echo "╚════════════════════════════════════════════╝"
echo ""
echo "This wizard will help you set up your email accounts."
echo ""

# Detect configured accounts from himalaya
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
    echo "Please configure himalaya first (~/.config/himalaya/config.toml)"
    exit 1
fi

echo "Current account status:"
echo "----------------------"
for acct in "${accounts[@]}"; do
    printf "• %-20s" "$acct:"
    if $HIMALAYA -o json folder list -a "$acct" >/dev/null 2>&1; then
        echo "✓ Working"
    else
        echo "✗ Needs setup"
    fi
done

echo ""
echo "Select an action:"
echo "1) Set up a password-based account (IMAP/SMTP)"
echo "2) Set up an iCloud account (app-specific password)"
echo "3) Set up an OAuth account (Gmail/Outlook browser flow)"
echo "4) Test the app with working accounts"
echo "5) Exit"
echo ""
read -rp "Choice [1-5]: " choice

case $choice in
    1)
        "$SCRIPT_DIR/setup-password-account.sh"
        ;;
    2)
        "$SCRIPT_DIR/setup-icloud.sh"
        ;;
    3)
        "$SCRIPT_DIR/setup-oauth.sh"
        ;;
    4)
        echo ""
        echo "Testing SolverForge Mail..."
        echo "Use Ctrl+C to exit the app"
        echo ""
        # Find first working account
        for acct in "${accounts[@]}"; do
            if $HIMALAYA -o json folder list -a "$acct" >/dev/null 2>&1; then
                echo "Starting with account: $acct"
                "$SCRIPT_DIR/target/release/solverforge-mail" --account "$acct"
                break
            fi
        done
        ;;
    5)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo "Invalid choice"
        ;;
esac

echo ""
echo "Run this script again to set up more accounts: $0"
