#!/bin/bash
# Setup a generic password-based IMAP/SMTP account for himalaya

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/setup-common.sh"

echo "Setting up a password-based email account..."
echo ""

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

echo "Available himalaya accounts:"
for acct in "${accounts[@]}"; do
    echo "  - $acct"
done
echo ""

read -rp "Account name to configure: " account
if [ -z "$account" ]; then
    echo "Account name is required."
    exit 1
fi

account_found=0
for acct in "${accounts[@]}"; do
    if [ "$acct" = "$account" ]; then
        account_found=1
        break
    fi
done

if [ "$account_found" -ne 1 ]; then
    echo "Account '$account' was not found in himalaya config."
    exit 1
fi

read -rp "Username/login for $account: " username
if [ -z "$username" ]; then
    echo "Username is required."
    exit 1
fi

echo "Please enter the password for $account:"
read -rs password
echo

# Store in keyring for IMAP
echo -n "$password" | secret-tool store --label="$account IMAP password" \
    service "${account}-imap" \
    username "$username" \
    application himalaya

# Store in keyring for SMTP
echo -n "$password" | secret-tool store --label="$account SMTP password" \
    service "${account}-smtp" \
    username "$username" \
    application himalaya

echo "Password stored in keyring."
echo "Testing connection..."

$HIMALAYA -o json folder list -a "$account" 2>&1 | head -5

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo "✓ $account is working!"
else
    echo "✗ Connection failed for $account. Please check the stored credentials and account settings."
fi
