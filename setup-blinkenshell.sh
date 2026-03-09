#!/bin/bash
# Setup blinkenshell password for himalaya

echo "Setting up blinkenshell email account..."
echo "Please enter your blinkenshell password:"
read -s password

# Store in keyring for IMAP
echo -n "$password" | secret-tool store --label="blinkenshell IMAP password" \
    service blinkenshell-imap \
    username argentodtw \
    application himalaya

# Store in keyring for SMTP  
echo -n "$password" | secret-tool store --label="blinkenshell SMTP password" \
    service blinkenshell-smtp \
    username argentodtw \
    application himalaya

echo "Password stored in keyring."
echo "Testing connection..."

/opt/himalaya/target/release/himalaya -o json folder list -a blinkenshell 2>&1 | head -5

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo "✓ blinkenshell account is working!"
else
    echo "✗ Connection failed. Please check your password."
fi