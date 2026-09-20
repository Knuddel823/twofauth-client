#!/usr/bin/env bash

set -euo pipefail

APP_ID="io.github.knuddel823.twofauth-client"
ICON_NAME="io.github.knuddel823.twofauth-client"
BINARY_NAME="twofauth-client"

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$PROJECT_DIR"

echo "Building TwoFAuth Client..."
cargo build --release

echo "Compiling translations..."

LANGUAGES=(de en fr es pt)

for lang in "${LANGUAGES[@]}"; do
    mkdir -p "po/${lang}/LC_MESSAGES"

    msgfmt \
        "po/${lang}/twofauth-client.po" \
        -o "po/${lang}/LC_MESSAGES/twofauth-client.mo"
done

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
    "/usr/local/share/icons/hicolor/256x256/apps/${ICON_NAME}.png"

echo "Installing AppStream metadata..."

sudo install -Dm644 \
    "data/${APP_ID}.metainfo.xml" \
    "/usr/local/share/metainfo/${APP_ID}.metainfo.xml"

echo "Installing translations..."

for lang in "${LANGUAGES[@]}"; do
    sudo install -Dm644 \
        "po/${lang}/LC_MESSAGES/twofauth-client.mo" \
        "/usr/local/share/locale/${lang}/LC_MESSAGES/twofauth-client.mo"
done

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
for lang in "${LANGUAGES[@]}"; do
    echo "  /usr/local/share/locale/${lang}/LC_MESSAGES/twofauth-client.mo"
done
