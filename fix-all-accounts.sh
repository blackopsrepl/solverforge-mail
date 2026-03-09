#!/bin/bash
# Fix all email accounts - store passwords in KWallet

set -e

echo "╔════════════════════════════════════════════╗"
echo "║     Fixing All Email Accounts               ║"
echo "╚════════════════════════════════════════════╝"
echo ""

# 1. BLINKENSHELL
echo "1. BLINKENSHELL ACCOUNT"
echo "Enter your blinkenshell.org password:"
read -s BLINK_PASS
echo ""
echo -n "$BLINK_PASS" | secret-tool store --label="Blinkenshell IMAP" service blinkenshell-imap
echo -n "$BLINK_PASS" | secret-tool store --label="Blinkenshell SMTP" service blinkenshell-smtp
echo "✓ Blinkenshell password stored"

# Test blinkenshell
echo "Testing blinkenshell..."
if /opt/himalaya/target/release/himalaya -o json folder list -a blinkenshell >/dev/null 2>&1; then
    echo "✓ Blinkenshell is working!"
else
    echo "✗ Blinkenshell auth failed - check password"
fi
echo ""

# 2. ICLOUD
echo "2. ICLOUD ACCOUNT" 
echo "You need an app-specific password from https://appleid.apple.com"
echo "Enter your iCloud app-specific password (format: xxxx-xxxx-xxxx-xxxx):"
read -s ICLOUD_PASS
echo ""
# Remove spaces/dashes if entered
ICLOUD_PASS=$(echo "$ICLOUD_PASS" | tr -d ' -')
echo -n "$ICLOUD_PASS" | secret-tool store --label="iCloud IMAP" service icloud-imap
echo -n "$ICLOUD_PASS" | secret-tool store --label="iCloud SMTP" service icloud-smtp
echo "✓ iCloud password stored"

# Test iCloud
echo "Testing iCloud..."
if /opt/himalaya/target/release/himalaya -o json folder list -a icloud >/dev/null 2>&1; then
    echo "✓ iCloud is working!"
else
    echo "✗ iCloud auth failed - ensure you're using an app-specific password"
fi
echo ""

# 3. OAuth accounts
echo "3. OAUTH ACCOUNTS (Gmail, Outlook)"
echo "These require browser authentication."
echo ""

echo "Fix Gmail accounts? (y/n): "
read -n1 FIX_GMAIL
echo ""
if [ "$FIX_GMAIL" = "y" ]; then
    echo "Setting up gmail (vmeilichios@gmail.com)..."
    /opt/himalaya/target/release/himalaya account configure gmail || true
    
    echo "Setting up kgmail (blackopsrepl@gmail.com)..."
    /opt/himalaya/target/release/himalaya account configure kgmail || true
fi

echo "Fix Outlook account? (y/n): "
read -n1 FIX_OUTLOOK
echo ""
if [ "$FIX_OUTLOOK" = "y" ]; then
    echo "Setting up outlook..."
    /opt/himalaya/target/release/himalaya account configure outlook || true
fi

echo ""
echo "═══════════════════════════════════════════════"
echo "ACCOUNT STATUS:"
echo "═══════════════════════════════════════════════"

for acct in icloud blinkenshell gmail kgmail outlook test; do
    printf "%-15s: " "$acct"
    if /opt/himalaya/target/release/himalaya -o json folder list -a $acct >/dev/null 2>&1; then
        echo "✓ Working"
    else
        echo "✗ Not working"
    fi
done

echo ""
echo "Launch SolverForge Mail: /srv/lab/hack/solverforge-mail/solverforge-mail"