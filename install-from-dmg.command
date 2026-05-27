#!/bin/bash
# Dubbelklicka på den här filen för att installera SDK-appen.
APP_NAME="SDK - Säker Digital Kommunikation"
DIR="$(cd "$(dirname "$0")" && pwd)"

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║  SDK – Installerar till /Applications   ║"
echo "╚══════════════════════════════════════════╝"
echo ""

# Kopiera .app till /Applications (skriver över gammal version)
if [ -d "/Applications/$APP_NAME.app" ]; then
  echo "→ Tar bort tidigare version..."
  rm -rf "/Applications/$APP_NAME.app"
fi

echo "→ Kopierar appen..."
cp -r "$DIR/$APP_NAME.app" "/Applications/$APP_NAME.app"

# Ta bort karantänflaggan
echo "→ Tar bort karantänflagga..."
xattr -rd com.apple.quarantine "/Applications/$APP_NAME.app" 2>/dev/null || true

echo ""
echo "✓ Installationen klar!"
echo ""
echo "→ Startar appen..."
sleep 1
open "/Applications/$APP_NAME.app"
