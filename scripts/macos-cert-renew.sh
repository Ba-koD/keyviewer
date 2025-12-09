#!/bin/bash
# macOS Self-Signed Certificate Renewal Script
# Run this when the certificate expires (after 10 years) or if you need a new one
# IMPORTANT: After renewal, all users will need to re-grant permissions on first launch

set -e

CERT_NAME="KeyViewer Signing"
CERT_PASSWORD="${1:-KeyViewerSign2024}"
OUTPUT_DIR="./certs"
VALIDITY_DAYS=3650  # 10 years
BACKUP_DIR="./certs/backup-$(date +%Y%m%d-%H%M%S)"

echo "==================================="
echo "macOS Certificate Renewal"
echo "==================================="
echo ""

# Backup existing certificate
if [ -f "$OUTPUT_DIR/keyviewer-signing.p12" ]; then
    echo "[Backup] Saving existing certificate..."
    mkdir -p "$BACKUP_DIR"
    cp "$OUTPUT_DIR/keyviewer-signing.p12" "$BACKUP_DIR/"
    cp "$OUTPUT_DIR/keyviewer-signing.p12.base64" "$BACKUP_DIR/" 2>/dev/null || true
    echo "Backup saved to: $BACKUP_DIR"
fi

# Remove old certificate from Keychain
echo "[Cleanup] Removing old certificate from Keychain..."
security delete-certificate -c "$CERT_NAME" 2>/dev/null || echo "No existing certificate in Keychain"

# Generate new certificate (same as setup)
echo "[Generate] Creating new certificate..."
mkdir -p "$OUTPUT_DIR"

openssl req -x509 -newkey rsa:4096 \
    -keyout "$OUTPUT_DIR/key.pem" \
    -out "$OUTPUT_DIR/cert.pem" \
    -days $VALIDITY_DAYS \
    -nodes \
    -subj "/CN=$CERT_NAME/O=KeyViewer Team/C=US"

openssl pkcs12 -export \
    -out "$OUTPUT_DIR/keyviewer-signing.p12" \
    -inkey "$OUTPUT_DIR/key.pem" \
    -in "$OUTPUT_DIR/cert.pem" \
    -password pass:"$CERT_PASSWORD"

base64 -i "$OUTPUT_DIR/keyviewer-signing.p12" -o "$OUTPUT_DIR/keyviewer-signing.p12.base64"

# Import to Keychain
echo "[Import] Adding new certificate to Keychain..."
security import "$OUTPUT_DIR/keyviewer-signing.p12" \
    -k ~/Library/Keychains/login.keychain-db \
    -P "$CERT_PASSWORD" \
    -T /usr/bin/codesign \
    -T /usr/bin/security

# Cleanup
rm -f "$OUTPUT_DIR/key.pem" "$OUTPUT_DIR/cert.pem"

echo ""
echo "==================================="
echo "Certificate renewed successfully!"
echo "==================================="
echo ""
echo "⚠️  IMPORTANT: Update GitHub Secrets!"
echo ""
echo "1. Go to: GitHub → Settings → Secrets → Actions"
echo "2. Update MACOS_SIGNING_CERT with contents of:"
echo "   $OUTPUT_DIR/keyviewer-signing.p12.base64"
echo ""
echo "3. Copy the new .p12 to other build machines"
echo ""
echo "Note: Users who already installed the app may need to"
echo "re-grant permissions after updating to a new build."

