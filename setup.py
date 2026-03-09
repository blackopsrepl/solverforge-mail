#!/usr/bin/env python3
"""Set up email account passwords in KWallet"""

import subprocess
import getpass
import sys

def store_password(service, password):
    """Store password in KWallet using secret-tool"""
    try:
        proc = subprocess.Popen(
            ['secret-tool', 'store', '--label', service, 'service', service],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        proc.communicate(password.encode())
        return proc.returncode == 0
    except Exception as e:
        print(f"Error storing {service}: {e}")
        return False

def test_account(account):
    """Test if account can connect"""
    try:
        result = subprocess.run(
            ['/opt/himalaya/target/release/himalaya', '-o', 'json', 'folder', 'list', '-a', account],
            capture_output=True,
            timeout=10
        )
        return result.returncode == 0
    except:
        return False

def main():
    print("╔════════════════════════════════════════════╗")
    print("║     Email Account Setup                     ║")
    print("╚════════════════════════════════════════════╝")
    print()
    
    # Test current status
    print("Current account status:")
    accounts = ['icloud', 'blinkenshell', 'gmail', 'kgmail', 'outlook', 'test']
    working = []
    for acc in accounts:
        status = "✓ Working" if test_account(acc) else "✗ Not working"
        print(f"  {acc:15}: {status}")
        if test_account(acc):
            working.append(acc)
    
    if working:
        print(f"\nYou have {len(working)} working account(s): {', '.join(working)}")
        print("Run: /srv/lab/hack/solverforge-mail/solverforge-mail")
        
    print("\nFix accounts? (y/n): ", end='')
    if input().lower() != 'y':
        return
    
    print("\n1. BLINKENSHELL")
    print("Enter blinkenshell.org password (hidden): ")
    blink_pass = getpass.getpass("")
    if blink_pass:
        store_password('blinkenshell-imap', blink_pass)
        store_password('blinkenshell-smtp', blink_pass)
        if test_account('blinkenshell'):
            print("✓ Blinkenshell is working!")
        else:
            print("✗ Blinkenshell failed - check password")
    
    print("\n2. ICLOUD")
    print("You need an app-specific password from https://appleid.apple.com")
    print("Go to Sign-In & Security → App-Specific Passwords → Generate")
    print("Enter iCloud app-specific password (hidden): ")
    icloud_pass = getpass.getpass("")
    if icloud_pass:
        # Clean up the password (remove spaces/dashes)
        icloud_clean = icloud_pass.replace(' ', '').replace('-', '')
        store_password('icloud-imap', icloud_clean)
        store_password('icloud-smtp', icloud_clean)
        if test_account('icloud'):
            print("✓ iCloud is working!")
        else:
            print("✗ iCloud failed - ensure it's an app-specific password")
    
    print("\n3. OAUTH ACCOUNTS")
    print("Gmail/Outlook require browser authentication.")
    print("Run these commands in a terminal with browser access:")
    print("  himalaya account configure gmail")
    print("  himalaya account configure kgmail")
    print("  himalaya account configure outlook")
    
    print("\n" + "="*50)
    print("FINAL STATUS:")
    print("="*50)
    for acc in accounts:
        status = "✓ Working" if test_account(acc) else "✗ Not working"
        print(f"  {acc:15}: {status}")
    
    print("\nRun SolverForge Mail:")
    print("  /srv/lab/hack/solverforge-mail/solverforge-mail")

if __name__ == "__main__":
    main()