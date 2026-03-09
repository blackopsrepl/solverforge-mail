#!/bin/bash
# Setup OAuth for Gmail/Outlook accounts

echo "OAuth Setup for Gmail/Outlook"
echo "=============================="
echo ""
echo "These accounts use OAuth2 which requires a browser-based authentication flow."
echo ""
echo "To fix Gmail accounts (gmail, kgmail):"
echo "  1. Run: himalaya account configure gmail"
echo "  2. Follow the browser flow to authenticate"
echo "  3. Repeat for kgmail if needed"
echo ""
echo "To fix Outlook account:"
echo "  1. Run: himalaya account configure outlook"
echo "  2. Follow the browser flow to authenticate"
echo ""
echo "The himalaya OAuth wizard will:"
echo "- Open your browser for authentication"
echo "- Store tokens in the keyring automatically"
echo "- Test the connection"
echo ""
echo "Press Enter to continue with gmail setup, or Ctrl+C to cancel..."
read

echo "Starting OAuth setup for gmail..."
/opt/himalaya/target/release/himalaya account configure gmail

echo ""
echo "To set up other OAuth accounts, run:"
echo "  himalaya account configure kgmail"
echo "  himalaya account configure outlook"