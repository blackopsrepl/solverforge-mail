#!/bin/bash
# Direct password storage in keyring - edit the passwords below then run

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/setup-common.sh"

# !!!! EDIT THESE BEFORE RUNNING !!!!
ACCOUNT_1_SERVICE=""       # e.g. "blinkenshell"
ACCOUNT_1_PASSWORD=""

ACCOUNT_2_SERVICE=""       # e.g. "icloud"
ACCOUNT_2_PASSWORD=""

store_account() {
    local service="$1"
    local password="$2"
    if [ -z "$service" ] || [ -z "$password" ]; then
        return
    fi

    echo -n "$password" | secret-tool store --label="$service IMAP" service "${service}-imap"
    echo -n "$password" | secret-tool store --label="$service SMTP" service "${service}-smtp"
    echo "✓ $service stored"
}

if [ -z "$ACCOUNT_1_SERVICE" ] && [ -z "$ACCOUNT_2_SERVICE" ]; then
    echo "ERROR: Edit this script first and add your account details!"
    echo "  1. Edit: $0"
    echo "  2. Set ACCOUNT_1_SERVICE, ACCOUNT_1_PASSWORD (and optionally ACCOUNT_2_*)"
    echo "  3. Run again"
    exit 1
fi

echo "Storing passwords in keyring..."

store_account "$ACCOUNT_1_SERVICE" "$ACCOUNT_1_PASSWORD"
store_account "$ACCOUNT_2_SERVICE" "$ACCOUNT_2_PASSWORD"

echo ""
echo "Testing accounts..."
for svc in "$ACCOUNT_1_SERVICE" "$ACCOUNT_2_SERVICE"; do
    if [ -n "$svc" ]; then
        printf "%-15s: " "$svc"
        if $HIMALAYA -o json folder list -a "$svc" >/dev/null 2>&1; then
            echo "✓ Working!"
        else
            echo "✗ Failed"
        fi
    fi
done

echo ""
echo "IMPORTANT: Clear the passwords from this script after use!"
echo "  Edit $0 and reset the PASSWORD variables to empty strings."
