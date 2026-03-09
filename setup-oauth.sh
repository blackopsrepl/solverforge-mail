#!/bin/bash
# Setup OAuth for Gmail/Outlook accounts

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/setup-common.sh"

echo "OAuth Setup for Gmail/Outlook"
echo "=============================="
echo ""
echo "These accounts use OAuth2 which requires a browser-based authentication flow."
echo ""
echo "Configured accounts that may use OAuth:"

# List accounts from himalaya and let the user pick
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
    echo "  (could not detect accounts — enter name manually)"
    echo ""
    read -rp "Account name to configure: " account
    if [ -n "$account" ]; then
        echo "Starting OAuth setup for $account..."
        $HIMALAYA account configure "$account"
    fi
else
    for i in "${!accounts[@]}"; do
        printf "  %d) %s\n" $((i+1)) "${accounts[$i]}"
    done
    echo ""
    read -rp "Account number to configure (or 'q' to quit): " choice

    if [ "$choice" != "q" ] && [ -n "$choice" ]; then
        idx=$((choice - 1))
        if [ $idx -ge 0 ] && [ $idx -lt ${#accounts[@]} ]; then
            account="${accounts[$idx]}"
            echo "Starting OAuth setup for $account..."
            $HIMALAYA account configure "$account"
        else
            echo "Invalid choice."
        fi
    fi
fi

echo ""
echo "To set up other accounts, run this script again."
