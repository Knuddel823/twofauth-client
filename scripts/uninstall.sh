#!/usr/bin/env bash

set -euo pipefail

APP_ID="de.twofauthclient.TwoFAuthClient"
BINARY_NAME="twofauth-client"

echo "Removing TwoFAuth Client..."

sudo rm -f "/usr/local/bin/${BINARY_NAME}"

sudo rm -f \
    "/usr/local/share/applications/${APP_ID}.desktop"

for size in 16 24 32 48 64 128 256; do
    sudo rm -f \
        "/usr/local/share/icons/hicolor/${size}x${size}/apps/${APP_ID}.png"
done

sudo rm -f \
    "/usr/local/share/metainfo/${APP_ID}.metainfo.xml"

sudo rm -f \
    "/usr/local/share/locale/de/LC_MESSAGES/twofauth-client.mo"

sudo rm -f \
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
echo "TwoFAuth Client has been removed."
echo
echo "User configuration, cached icons and keyring data were not removed."
