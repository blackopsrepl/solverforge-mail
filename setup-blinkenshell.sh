#!/bin/bash
# Setup blinkenshell password for himalaya

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/setup-common.sh"

echo "Setting up blinkenshell email account..."

read -rp "Blinkenshell username: " username
if [ -z "$username" ]; then
    echo "Username is required."
    exit 1
fi

echo "Please enter your blinkenshell password:"
read -rs password
echo

# Store in keyring for IMAP
echo -n "$password" | secret-tool store --label="blinkenshell IMAP password" \
    service blinkenshell-imap \
    username "$username" \
    application himalaya

# Store in keyring for SMTP
echo -n "$password" | secret-tool store --label="blinkenshell SMTP password" \
    service blinkenshell-smtp \
    username "$username" \
    application himalaya

echo "Password stored in keyring."
echo "Testing connection..."

$HIMALAYA -o json folder list -a blinkenshell 2>&1 | head -5

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo "✓ blinkenshell account is working!"
else
    echo "✗ Connection failed. Please check your password."
fi
