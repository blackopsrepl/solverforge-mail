#!/bin/bash
# Master account setup script for SolverForge Mail

echo "╔════════════════════════════════════════════╗"
echo "║     SolverForge Mail - Account Setup        ║"
echo "╚════════════════════════════════════════════╝"
echo ""
echo "This wizard will help you set up your email accounts."
echo ""
echo "Current account status:"
echo "----------------------"

# Check each account
echo -n "• iCloud (default):    "
/opt/himalaya/target/release/himalaya -o json folder list -a icloud >/dev/null 2>&1 && echo "✓ Working" || echo "✗ Needs setup"

echo -n "• blinkenshell:        "
/opt/himalaya/target/release/himalaya -o json folder list -a blinkenshell >/dev/null 2>&1 && echo "✓ Working" || echo "✗ Needs setup"

echo -n "• gmail:               "
/opt/himalaya/target/release/himalaya -o json folder list -a gmail >/dev/null 2>&1 && echo "✓ Working" || echo "✗ Needs setup"

echo -n "• kgmail:              "
/opt/himalaya/target/release/himalaya -o json folder list -a kgmail >/dev/null 2>&1 && echo "✓ Working" || echo "✗ Needs setup"

echo -n "• outlook:             "
/opt/himalaya/target/release/himalaya -o json folder list -a outlook >/dev/null 2>&1 && echo "✓ Working" || echo "✗ Needs setup"

echo -n "• test (local):        "
/opt/himalaya/target/release/himalaya -o json folder list -a test >/dev/null 2>&1 && echo "✓ Working" || echo "✗ Needs setup"

echo ""
echo "Select an account to set up:"
echo "1) iCloud - Requires app-specific password"
echo "2) blinkenshell - Requires password"  
echo "3) Gmail accounts - Requires OAuth browser flow"
echo "4) Outlook - Requires OAuth browser flow"
echo "5) Test the app with working accounts"
echo "6) Exit"
echo ""
echo -n "Choice [1-6]: "
read choice

case $choice in
    1)
        /srv/lab/hack/solverforge-mail/setup-icloud.sh
        ;;
    2)
        /srv/lab/hack/solverforge-mail/setup-blinkenshell.sh
        ;;
    3)
        /srv/lab/hack/solverforge-mail/setup-oauth.sh
        ;;
    4)
        echo "Starting OAuth setup for outlook..."
        /opt/himalaya/target/release/himalaya account configure outlook
        ;;
    5)
        echo ""
        echo "Testing SolverForge Mail..."
        echo "Use Ctrl+C to exit the app"
        echo ""
        # Find first working account
        for acct in icloud blinkenshell gmail kgmail outlook test; do
            if /opt/himalaya/target/release/himalaya -o json folder list -a $acct >/dev/null 2>&1; then
                echo "Starting with account: $acct"
                cd /srv/lab/hack/solverforge-mail
                ./target/release/solverforge-mail --account $acct
                break
            fi
        done
        ;;
    6)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo "Invalid choice"
        ;;
esac

echo ""
echo "Run this script again to set up more accounts: $0"