# macOS Code Signing Guide

This guide explains how to create and use a self-signed certificate for macOS builds. This ensures that app permissions (Accessibility, Input Monitoring, Screen Recording) persist across updates.

## Why Code Signing?

macOS identifies apps by their **code signature**. Without consistent signing:
- Each build is treated as a "different app"
- Users must re-grant permissions after every update
- Gatekeeper shows warnings

With consistent signing:
- ✅ Permissions persist across updates
- ✅ Same identity for all builds
- ✅ Works on any build machine (local, CI/CD)

## Quick Start

### 1. Generate Certificate (One Time Only)

Run this on any Mac:

```bash
cd /path/to/keyviewer
chmod +x scripts/macos-cert-setup.sh
./scripts/macos-cert-setup.sh "YourSecurePassword"
```

This creates:
- `certs/keyviewer-signing.p12` - Certificate file
- `certs/keyviewer-signing.p12.base64` - For GitHub Secrets

### 2. Setup GitHub Secrets

Go to your GitHub repository:
**Settings → Secrets and variables → Actions → New repository secret**

Add these secrets:

| Secret Name | Value |
|-------------|-------|
| `MACOS_SIGNING_CERT` | Contents of `certs/keyviewer-signing.p12.base64` |
| `MACOS_SIGNING_CERT_PWD` | Password you used (e.g., `YourSecurePassword`) |

### 3. Done!

GitHub Actions will now automatically sign macOS builds.

---

## Using on Another Computer

Copy `keyviewer-signing.p12` to the other Mac, then:

```bash
# Import certificate
security import keyviewer-signing.p12 \
    -k ~/Library/Keychains/login.keychain-db \
    -P "YourSecurePassword" \
    -T /usr/bin/codesign

# Verify it's installed
security find-identity -v -p codesigning
# Should show: "KeyViewer Signing"

# Sign manually if needed
codesign --force --deep --sign "KeyViewer Signing" YourApp.app
```

---

## Certificate Renewal

Certificates expire after 10 years. To renew:

```bash
chmod +x scripts/macos-cert-renew.sh
./scripts/macos-cert-renew.sh "NewPassword"
```

Then update GitHub Secrets with the new `keyviewer-signing.p12.base64` content.

⚠️ **Note**: After renewal, existing users may need to re-grant permissions once.

---

## Manual Signing Commands

### Sign an .app bundle
```bash
codesign --force --deep --sign "KeyViewer Signing" path/to/App.app
```

### Sign a .dmg file
```bash
codesign --force --sign "KeyViewer Signing" path/to/App.dmg
```

### Verify signature
```bash
codesign --verify --verbose path/to/App.app
```

### Check certificate details
```bash
security find-identity -v -p codesigning
```

---

## Troubleshooting

### "No identity found"
Certificate not in Keychain. Import it:
```bash
security import keyviewer-signing.p12 -k ~/Library/Keychains/login.keychain-db -P "password" -T /usr/bin/codesign
```

### "User interaction is not allowed"
Keychain is locked. Unlock it:
```bash
security unlock-keychain ~/Library/Keychains/login.keychain-db
```

### "Code signature invalid"
Re-sign with `--force`:
```bash
codesign --force --deep --sign "KeyViewer Signing" App.app
```

### Permission not persisting after update
- Check both builds use the same certificate
- Verify signature: `codesign -dv App.app`
- Compare "Authority" field between versions

---

## Security Notes

- Keep `.p12` files secure - they contain your private key
- Use strong passwords
- Don't commit certificate files to git (add to `.gitignore`)
- GitHub Secrets are encrypted and safe for CI/CD use

---

## Comparison: Self-Signed vs Apple Developer

| Feature | Self-Signed (Free) | Apple Developer ($99/year) |
|---------|-------------------|---------------------------|
| Permission persistence | ✅ | ✅ |
| Gatekeeper warning | ⚠️ "Unidentified developer" | ✅ No warning |
| Notarization | ❌ | ✅ |
| First launch | Right-click → Open | Double-click |
| Cost | Free | $99/year |

Self-signed is sufficient for open-source projects where users understand they need to right-click to open.

