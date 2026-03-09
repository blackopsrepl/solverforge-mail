#!/bin/bash
# Direct password storage in KWallet - edit the passwords below then run

# !!!! EDIT THESE PASSWORDS !!!!
BLINKENSHELL_PASSWORD=""
ICLOUD_APP_PASSWORD=""  # Get from https://appleid.apple.com -> Sign-In & Security -> App-Specific Passwords

if [ -z "$BLINKENSHELL_PASSWORD" ] || [ -z "$ICLOUD_APP_PASSWORD" ]; then
    echo "ERROR: Edit this script first and add your passwords!"
    echo "  1. Edit: $0"
    echo "  2. Add your passwords to the variables at the top"
    echo "  3. Run again"
    exit 1
fi

echo "Storing passwords in KWallet..."

# Store Blinkenshell
echo -n "$BLINKENSHELL_PASSWORD" | secret-tool store --label="Blinkenshell IMAP" service blinkenshell-imap
echo -n "$BLINKENSHELL_PASSWORD" | secret-tool store --label="Blinkenshell SMTP" service blinkenshell-smtp
echo "✓ Blinkenshell stored"

# Store iCloud (remove any spaces/dashes from app-specific password)
ICLOUD_CLEAN=$(echo "$ICLOUD_APP_PASSWORD" | tr -d ' -')
echo -n "$ICLOUD_CLEAN" | secret-tool store --label="iCloud IMAP" service icloud-imap
echo -n "$ICLOUD_CLEAN" | secret-tool store --label="iCloud SMTP" service icloud-smtp
echo "✓ iCloud stored"

echo ""
echo "Testing accounts..."
echo -n "Blinkenshell: "
/opt/himalaya/target/release/himalaya -o json folder list -a blinkenshell >/dev/null 2>&1 && echo "✓ Working!" || echo "✗ Failed"

echo -n "iCloud: "
/opt/himalaya/target/release/himalaya -o json folder list -a icloud >/dev/null 2>&1 && echo "✓ Working!" || echo "✗ Failed"

echo ""
echo "IMPORTANT: Clear the passwords from this script after use!"
echo "  sed -i 's/^BLINKENSHELL_PASSWORD=.*/BLINKENSHELL_PASSWORD=\"\"/' $0"
echo "  sed -i 's/^ICLOUD_APP_PASSWORD=.*/ICLOUD_APP_PASSWORD=\"\"/' $0"