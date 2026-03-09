#!/bin/bash
# Setup iCloud app-specific password for himalaya

echo "Setting up iCloud email account..."
echo ""
echo "IMPORTANT: iCloud requires an app-specific password for email clients."
echo "1. Go to https://appleid.apple.com/account/manage"
echo "2. Sign in and go to 'Sign-In and Security' > 'App-Specific Passwords'"
echo "3. Generate a new password for 'Himalaya Mail' (or use existing one)"
echo ""
echo "Enter your iCloud app-specific password (16 characters, like xxxx-xxxx-xxxx-xxxx):"
read -s password

# Create temp authinfo
tmpfile=$(mktemp)
gpg -q --for-your-eyes-only -d ~/.authinfo.gpg 2>/dev/null | grep -v "imap.mail.me.com\|smtp.mail.me.com" > "$tmpfile" || true

# Add updated entries
echo "machine imap.mail.me.com login blackopsrepl@icloud.com password $password" >> "$tmpfile"
echo "machine smtp.mail.me.com login blackopsrepl@icloud.com password $password" >> "$tmpfile"

# Encrypt back
gpg --batch --yes -e -r "info@vdistefano.studio" "$tmpfile" 2>/dev/null
mv "${tmpfile}.gpg" ~/.authinfo.gpg
rm -f "$tmpfile"

echo "Password updated in ~/.authinfo.gpg"
echo "Testing connection..."

/opt/himalaya/target/release/himalaya -o json folder list -a icloud 2>&1 | head -5

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo "✓ iCloud account is working!"
else
    echo "✗ Connection failed. Please check:"
    echo "  - You're using an app-specific password (not your Apple ID password)"
    echo "  - The password format is correct (no spaces in xxxx-xxxx-xxxx-xxxx)"
fi