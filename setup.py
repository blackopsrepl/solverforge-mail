#!/usr/bin/env python3
"""Set up email account passwords in keyring"""

import subprocess
import getpass
import json
import shutil
import sys
import os


def find_himalaya():
    """Find himalaya binary (mirrors src/himalaya/config.rs lookup)."""
    for name in ("solverforge-himalaya", "himalaya"):
        path = shutil.which(name)
        if path:
            return path
    fallback = "/opt/himalaya/target/release/himalaya"
    if os.path.isfile(fallback) and os.access(fallback, os.X_OK):
        return fallback
    return None


def store_password(service, password):
    """Store password in keyring using secret-tool"""
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


def test_account(himalaya, account):
    """Test if account can connect"""
    try:
        result = subprocess.run(
            [himalaya, '-o', 'json', 'folder', 'list', '-a', account],
            capture_output=True,
            timeout=10
        )
        return result.returncode == 0
    except Exception:
        return False


def list_accounts(himalaya):
    """Get configured account names from himalaya."""
    try:
        result = subprocess.run(
            [himalaya, '-o', 'json', 'account', 'list'],
            capture_output=True, text=True, timeout=10
        )
        if result.returncode == 0:
            data = json.loads(result.stdout)
            return [acc['name'] for acc in data if 'name' in acc]
    except Exception:
        pass
    return []


def main():
    himalaya = find_himalaya()
    if not himalaya:
        print("ERROR: Could not find himalaya binary.")
        print("Install himalaya or ensure it is on your PATH.")
        sys.exit(1)

    print("╔════════════════════════════════════════════╗")
    print("║     Email Account Setup                    ║")
    print("╚════════════════════════════════════════════╝")
    print()

    accounts = list_accounts(himalaya)
    if not accounts:
        print("No accounts found in himalaya config.")
        print("Please configure himalaya first (~/.config/himalaya/config.toml)")
        sys.exit(1)

    # Test current status
    print("Current account status:")
    working = []
    for acc in accounts:
        ok = test_account(himalaya, acc)
        status = "✓ Working" if ok else "✗ Not working"
        print(f"  {acc:15}: {status}")
        if ok:
            working.append(acc)

    if working:
        print(f"\nYou have {len(working)} working account(s): {', '.join(working)}")

    script_dir = os.path.dirname(os.path.abspath(__file__))
    binary = os.path.join(script_dir, "target", "release", "solverforge-mail")
    if os.path.isfile(binary):
        print(f"Run: {binary}")

    print("\nFix accounts? (y/n): ", end='', flush=True)
    if input().lower() != 'y':
        return

    for acc in accounts:
        if test_account(himalaya, acc):
            continue

        print(f"\nAccount: {acc}")
        print("  1) Set password  2) OAuth (browser)  3) Skip")
        choice = input("  Choice [1-3]: ").strip()

        if choice == '1':
            password = getpass.getpass(f"  Password for {acc}: ")
            if password:
                store_password(f'{acc}-imap', password)
                store_password(f'{acc}-smtp', password)
                if test_account(himalaya, acc):
                    print(f"  ✓ {acc} is working!")
                else:
                    print(f"  ✗ {acc} failed - check password")
        elif choice == '2':
            print(f"  Starting OAuth for {acc}...")
            subprocess.run([himalaya, 'account', 'configure', acc])
        else:
            print(f"  Skipping {acc}")

    print("\n" + "=" * 50)
    print("FINAL STATUS:")
    print("=" * 50)
    for acc in accounts:
        status = "✓ Working" if test_account(himalaya, acc) else "✗ Not working"
        print(f"  {acc:15}: {status}")

    if os.path.isfile(binary):
        print(f"\nRun SolverForge Mail:\n  {binary}")


if __name__ == "__main__":
    main()
