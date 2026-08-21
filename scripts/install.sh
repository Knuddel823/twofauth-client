#!/usr/bin/env bash

set -euo pipefail

APP_ID="de.twofauthclient.TwoFAuthClient"
BINARY_NAME="twofauth-client"

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$PROJECT_DIR"

echo "Building TwoFAuth Client..."
cargo build --release

echo "Compiling translations..."

mkdir -p po/de/LC_MESSAGES
mkdir -p po/en/LC_MESSAGES

msgfmt \
    po/de/twofauth-client.po \
    -o po/de/LC_MESSAGES/twofauth-client.mo

msgfmt \
    po/en/twofauth-client.po \
    -o po/en/LC_MESSAGES/twofauth-client.mo

echo "Installing binary..."

sudo install -Dm755 \
    "target/release/${BINARY_NAME}" \
    "/usr/local/bin/${BINARY_NAME}"

echo "Installing desktop file..."

sudo install -Dm644 \
    "data/${APP_ID}.desktop" \
    "/usr/local/share/applications/${APP_ID}.desktop"

echo "Installing application icon..."

sudo install -Dm644 \
    "data/icons/hicolor/256x256/apps/${APP_ID}.png" \
    "/usr/local/share/icons/hicolor/256x256/apps/${APP_ID}.png"

echo "Installing AppStream metadata..."

sudo install -Dm644 \
    "data/${APP_ID}.metainfo.xml" \
    "/usr/local/share/metainfo/${APP_ID}.metainfo.xml"

echo "Installing German translation..."

sudo install -Dm644 \
    "po/de/LC_MESSAGES/twofauth-client.mo" \
    "/usr/local/share/locale/de/LC_MESSAGES/twofauth-client.mo"

echo "Installing English translation..."

sudo install -Dm644 \
    "po/en/LC_MESSAGES/twofauth-client.mo" \
    "/usr/local/share/locale/en/LC_MESSAGES/twofauth-client.mo"

if command -v update-desktop-database >/dev/null 2>&1; then
    sudo update-desktop-database \
        /usr/local/share/applications \
        || true
fi

if command -v gtk4-update-icon-cache >/dev/null 2>&1; then
    sudo gtk4-update-icon-cache \
        -f \
        -t \
        /usr/local/share/icons/hicolor \
        || true
elif command -v gtk-update-icon-cache >/dev/null 2>&1; then
    sudo gtk-update-icon-cache \
        -f \
        -t \
        /usr/local/share/icons/hicolor \
        || true
fi

echo
echo "TwoFAuth Client installed successfully."
echo
echo "Binary:"
echo "  /usr/local/bin/${BINARY_NAME}"
echo
echo "Desktop file:"
echo "  /usr/local/share/applications/${APP_ID}.desktop"
echo
echo "Translations:"
echo "  /usr/local/share/locale/de/LC_MESSAGES/twofauth-client.mo"
echo "  /usr/local/share/locale/en/LC_MESSAGES/twofauth-client.mo"
