#!/bin/bash
# macOS Self-Signed Certificate Setup Script
# This creates a code signing certificate for consistent app signing
# Run this ONCE on any Mac, then use the .p12 file for all builds

set -e

CERT_NAME="KeyViewer Signing"
CERT_PASSWORD="${1:-KeyViewerSign2024}"
OUTPUT_DIR="./certs"
VALIDITY_DAYS=3650  # ~10 years

echo "==================================="
echo "macOS Code Signing Certificate Setup"
echo "==================================="
echo ""
echo "Certificate Name: $CERT_NAME"
echo "Validity: $VALIDITY_DAYS days (~$(($VALIDITY_DAYS / 365)) years)"
echo "Output Directory: $OUTPUT_DIR"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Create OpenSSL config for Code Signing extension
cat > "$OUTPUT_DIR/codesign.cnf" << EOF
[req]
distinguished_name = req_distinguished_name
x509_extensions = codesign_ext
prompt = no

[req_distinguished_name]
CN = $CERT_NAME
O = KeyViewer Team
C = US

[codesign_ext]
keyUsage = critical, digitalSignature
extendedKeyUsage = critical, codeSigning
basicConstraints = critical, CA:FALSE
EOF

# Generate private key and self-signed certificate with Code Signing extension
echo "[1/4] Generating private key and certificate..."
openssl req -x509 -newkey rsa:4096 \
    -keyout "$OUTPUT_DIR/key.pem" \
    -out "$OUTPUT_DIR/cert.pem" \
    -days $VALIDITY_DAYS \
    -nodes \
    -config "$OUTPUT_DIR/codesign.cnf"

# Convert to .p12 format with LEGACY option for macOS compatibility
echo "[2/4] Creating .p12 file..."
openssl pkcs12 -export -legacy \
    -out "$OUTPUT_DIR/keyviewer-signing.p12" \
    -inkey "$OUTPUT_DIR/key.pem" \
    -in "$OUTPUT_DIR/cert.pem" \
    -passout "pass:$CERT_PASSWORD"

# Generate base64 encoded version for GitHub Secrets
echo "[3/4] Generating base64 encoded certificate..."
base64 -i "$OUTPUT_DIR/keyviewer-signing.p12" -o "$OUTPUT_DIR/keyviewer-signing.p12.base64"

# Import to local Keychain
echo "[4/4] Importing to login Keychain..."
echo ""
echo "⚠️  If prompted for 'password to unlock keychain', enter your MAC LOGIN PASSWORD"
echo "   (NOT the certificate password)"
echo ""

# First, try to delete existing certificate with same name (ignore errors)
security delete-certificate -c "$CERT_NAME" 2>/dev/null || true

# Import with all necessary flags
# Note: -P requires password without quotes in some macOS versions
if security import "$OUTPUT_DIR/keyviewer-signing.p12" \
    -k ~/Library/Keychains/login.keychain-db \
    -P "$CERT_PASSWORD" \
    -T /usr/bin/codesign \
    -T /usr/bin/security \
    -A 2>&1; then
    echo "✅ Auto-import successful!"
else
    echo ""
    echo "⚠️  Auto-import failed. Trying alternative method..."
    # Try with explicit keychain unlock
    security unlock-keychain -p "" ~/Library/Keychains/login.keychain-db 2>/dev/null || true
    if ! security import "$OUTPUT_DIR/keyviewer-signing.p12" \
        -k ~/Library/Keychains/login.keychain-db \
        -P "$CERT_PASSWORD" \
        -A 2>&1; then
        echo ""
        echo "⚠️  Please import manually:"
        echo "    1. Double-click: $OUTPUT_DIR/keyviewer-signing.p12"
        echo "    2. Enter password: $CERT_PASSWORD"
        echo "    3. Add to 'login' keychain"
    fi
fi

# Allow codesign to access without prompt (may ask for keychain password)
security set-key-partition-list -S apple-tool:,apple: -s -k "" ~/Library/Keychains/login.keychain-db 2>/dev/null || true

# Verify import
echo ""
echo "Verifying certificate in Keychain..."
if security find-identity -v -p codesigning | grep -q "$CERT_NAME"; then
    echo "✅ Certificate successfully imported!"
    security find-identity -v -p codesigning | grep "$CERT_NAME"
else
    echo "⚠️  Certificate not found in Keychain. Manual import may be needed:"
    echo "    ./scripts/macos-cert-import.sh $OUTPUT_DIR/keyviewer-signing.p12 \"$CERT_PASSWORD\""
fi

# Verify
echo ""
echo "==================================="
echo "Certificate created successfully!"
echo "==================================="
echo ""
echo "Files created:"
echo "  - $OUTPUT_DIR/keyviewer-signing.p12      (Certificate file)"
echo "  - $OUTPUT_DIR/keyviewer-signing.p12.base64 (For GitHub Secrets)"
echo "  - $OUTPUT_DIR/cert.pem                   (Certificate only)"
echo "  - $OUTPUT_DIR/key.pem                    (Private key)"
echo ""
echo "Password: $CERT_PASSWORD"
echo ""
echo "==================================="
echo "GitHub Secrets Setup"
echo "==================================="
echo ""

# Copy base64 to clipboard
if command -v pbcopy &> /dev/null; then
    cat "$OUTPUT_DIR/keyviewer-signing.p12.base64" | pbcopy
    echo "✅ MACOS_SIGNING_CERT value copied to clipboard!"
    echo ""
fi

echo "Add these secrets to your GitHub repository:"
echo "  Settings → Secrets and variables → Actions → New repository secret"
echo ""
echo "1. MACOS_SIGNING_CERT"
echo "   Value: (already copied to clipboard - just paste!)"
echo ""
echo "2. MACOS_SIGNING_CERT_PWD"  
echo "   Value: $CERT_PASSWORD"
echo ""
echo "==================================="
echo "Using on Another Computer"
echo "==================================="
echo ""
echo "Copy keyviewer-signing.p12 to the other Mac, then run:"
echo "  security import keyviewer-signing.p12 -k ~/Library/Keychains/login.keychain-db -P \"$CERT_PASSWORD\" -T /usr/bin/codesign"
echo ""
echo "Verify with:"
echo "  security find-identity -v -p codesigning"
echo ""

# Cleanup sensitive files (keep .p12 and base64)
rm -f "$OUTPUT_DIR/key.pem" "$OUTPUT_DIR/cert.pem" "$OUTPUT_DIR/codesign.cnf"

echo "Done! Keep the .p12 file safe - you'll need it for all future builds."

