#!/bin/bash
# SDK – Säker Digital Kommunikation
# Installationsskript för macOS
# Användning: curl -fsSL https://raw.githubusercontent.com/MrJensK/WebApp-Wrapper/main/install-mac.sh | bash

set -e

APP_NAME="SDK - Säker Digital Kommunikation"
GITHUB_REPO="MrJensK/WebApp-Wrapper"
INSTALL_DIR="/Applications"

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║  SDK – Säker Digital Kommunikation       ║"
echo "║  Installationsskript för macOS            ║"
echo "╚══════════════════════════════════════════╝"
echo ""

# Kontrollera arkitektur
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
  ARCH_FILTER="aarch64"
  echo "→ Arkitektur: Apple Silicon (arm64)"
else
  ARCH_FILTER="x86_64"
  echo "→ Arkitektur: Intel (x86_64)"
fi

# Hämta senaste release-URL från GitHub API
echo "→ Hämtar senaste version..."
DOWNLOAD_URL=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest" \
  | grep "browser_download_url" \
  | grep "$ARCH_FILTER" \
  | grep "\.dmg" \
  | cut -d'"' -f4 \
  | head -1)

if [ -z "$DOWNLOAD_URL" ]; then
  echo ""
  echo "✗ Kunde inte hitta en nedladdningslänk."
  echo "  Kontrollera att det finns en publicerad release på:"
  echo "  https://github.com/$GITHUB_REPO/releases"
  exit 1
fi

echo "→ Laddar ner: $(basename "$DOWNLOAD_URL")"
TMP_DMG=$(mktemp /tmp/sdk-XXXXX.dmg)
curl -L --progress-bar "$DOWNLOAD_URL" -o "$TMP_DMG"

# Montera DMG
echo "→ Monterar diskavbild..."
MOUNT_POINT=$(mktemp -d /tmp/sdk-mount-XXXXX)
hdiutil attach "$TMP_DMG" -mountpoint "$MOUNT_POINT" -nobrowse -quiet

# Kopiera .app till /Applications (skriver över eventuell gammal version)
echo "→ Kopierar till $INSTALL_DIR..."
if [ -d "$INSTALL_DIR/$APP_NAME.app" ]; then
  rm -rf "$INSTALL_DIR/$APP_NAME.app"
fi
cp -r "$MOUNT_POINT/$APP_NAME.app" "$INSTALL_DIR/$APP_NAME.app"

# Ta bort karantän så att Gatekeeper inte blockerar appen
echo "→ Tar bort karantänflagga (kräver inga rättigheter)..."
xattr -rd com.apple.quarantine "$INSTALL_DIR/$APP_NAME.app" 2>/dev/null || true

# Städa upp
hdiutil detach "$MOUNT_POINT" -quiet
rm -f "$TMP_DMG"
rmdir "$MOUNT_POINT" 2>/dev/null || true

echo ""
echo "✓ Installationen klar!"
echo "  Öppna appen från Launchpad eller $INSTALL_DIR"
echo ""
