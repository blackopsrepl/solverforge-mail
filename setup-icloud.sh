#!/bin/bash
# Setup iCloud app-specific password for himalaya

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/setup-common.sh"

echo "Setting up iCloud email account..."
echo ""
echo "IMPORTANT: iCloud requires an app-specific password for email clients."
echo "1. Go to https://appleid.apple.com/account/manage"
echo "2. Sign in and go to 'Sign-In and Security' > 'App-Specific Passwords'"
echo "3. Generate a new password for 'Himalaya Mail' (or use existing one)"
echo ""

read -rp "iCloud email address (e.g. user@icloud.com): " icloud_email
if [ -z "$icloud_email" ]; then
    echo "Email address is required."
    exit 1
fi

echo "Enter your iCloud app-specific password (16 characters, like xxxx-xxxx-xxxx-xxxx):"
read -rs password
echo

# Check if user has GPG-based auth setup
if [ -f ~/.authinfo.gpg ]; then
    read -rp "GPG key ID or email for encrypting ~/.authinfo.gpg: " gpg_recipient
    if [ -n "$gpg_recipient" ]; then
        # Create temp authinfo
        tmpfile=$(mktemp)
        gpg -q --for-your-eyes-only -d ~/.authinfo.gpg 2>/dev/null | grep -v "imap.mail.me.com\|smtp.mail.me.com" > "$tmpfile" || true

        # Add updated entries
        echo "machine imap.mail.me.com login $icloud_email password $password" >> "$tmpfile"
        echo "machine smtp.mail.me.com login $icloud_email password $password" >> "$tmpfile"

        # Encrypt back
        gpg --batch --yes -e -r "$gpg_recipient" "$tmpfile" 2>/dev/null
        mv "${tmpfile}.gpg" ~/.authinfo.gpg
        rm -f "$tmpfile"

        echo "Password updated in ~/.authinfo.gpg"
    fi
else
    # Store in keyring via secret-tool
    echo -n "$password" | secret-tool store --label="iCloud IMAP password" \
        service icloud-imap \
        username "$icloud_email" \
        application himalaya
    echo -n "$password" | secret-tool store --label="iCloud SMTP password" \
        service icloud-smtp \
        username "$icloud_email" \
        application himalaya
    echo "Password stored in keyring."
fi

echo "Testing connection..."

$HIMALAYA -o json folder list -a icloud 2>&1 | head -5

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo "✓ iCloud account is working!"
else
    echo "✗ Connection failed. Please check:"
    echo "  - You're using an app-specific password (not your Apple ID password)"
    echo "  - The password format is correct (no spaces in xxxx-xxxx-xxxx-xxxx)"
fi
