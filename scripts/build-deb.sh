#!/usr/bin/env bash

set -euo pipefail

APP_ID="de.twofauthclient.TwoFAuthClient"
ICON_NAME="twofauth-client"
PACKAGE_NAME="twofauth-client"
VERSION="$(grep '^version = ' Cargo.toml | head -1 | cut -d '"' -f2)"
ARCH="$(dpkg --print-architecture)"

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${PROJECT_DIR}/dist"
BUILD_ROOT="${DIST_DIR}/${PACKAGE_NAME}_${VERSION}_${ARCH}"

cd "$PROJECT_DIR"

echo "Building TwoFAuth Client ${VERSION} for ${ARCH}..."
cargo build --release

echo "Compiling translations..."

LANGUAGES=(de en fr es pt)

for lang in "${LANGUAGES[@]}"; do
    mkdir -p "po/${lang}/LC_MESSAGES"

    msgfmt \
        "po/${lang}/twofauth-client.po" \
        -o "po/${lang}/LC_MESSAGES/twofauth-client.mo"
done

echo "Preparing Debian package tree..."

rm -rf "$BUILD_ROOT"

mkdir -p \
    "$BUILD_ROOT/DEBIAN" \
    "$BUILD_ROOT/usr/bin" \
    "$BUILD_ROOT/usr/share/applications" \
    "$BUILD_ROOT/usr/share/metainfo"

for lang in "${LANGUAGES[@]}"; do
    mkdir -p "$BUILD_ROOT/usr/share/locale/${lang}/LC_MESSAGES"
done

install -Dm755 \
    "target/release/${PACKAGE_NAME}" \
    "$BUILD_ROOT/usr/bin/${PACKAGE_NAME}"

install -Dm644 \
    "data/${APP_ID}.desktop" \
    "$BUILD_ROOT/usr/share/applications/${APP_ID}.desktop"

install -Dm644 \
    "data/${APP_ID}.metainfo.xml" \
    "$BUILD_ROOT/usr/share/metainfo/${APP_ID}.metainfo.xml"

for lang in "${LANGUAGES[@]}"; do
    install -Dm644 \
        "po/${lang}/LC_MESSAGES/twofauth-client.mo" \
        "$BUILD_ROOT/usr/share/locale/${lang}/LC_MESSAGES/twofauth-client.mo"
done

install -Dm644 \
    "data/icons/hicolor/256x256/apps/${APP_ID}.png" \
    "$BUILD_ROOT/usr/share/icons/hicolor/256x256/apps/${ICON_NAME}.png"

cat > "$BUILD_ROOT/DEBIAN/control" <<EOFCONTROL
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: Knuddel823 <161348170+Knuddel823@users.noreply.github.com>
Depends: libadwaita-1-0, libgtk-4-1, libglib2.0-0t64
Recommends: gnome-keyring
Homepage: https://github.com/Knuddel823/twofauth-client
Description: Unofficial native Linux desktop client for 2FAuth
 TwoFAuth Client is a GTK4/libadwaita desktop application for
 accessing existing accounts and OTP codes from a self-hosted
 2FAuth server.
 .
 The application is deliberately read-oriented and does not
 create, modify or delete 2FAuth accounts.
EOFCONTROL

cat > "$BUILD_ROOT/DEBIAN/postinst" <<'EOFPOSTINST'
#!/bin/sh
set -e

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi

if command -v gtk4-update-icon-cache >/dev/null 2>&1; then
    gtk4-update-icon-cache -f -t /usr/share/icons/hicolor || true
elif command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi

exit 0
EOFPOSTINST

cat > "$BUILD_ROOT/DEBIAN/postrm" <<'EOFPOSTRM'
#!/bin/sh
set -e

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi

if command -v gtk4-update-icon-cache >/dev/null 2>&1; then
    gtk4-update-icon-cache -f -t /usr/share/icons/hicolor || true
elif command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi

exit 0
EOFPOSTRM

chmod 755 \
    "$BUILD_ROOT/DEBIAN/postinst" \
    "$BUILD_ROOT/DEBIAN/postrm"

echo "Building Debian package..."

dpkg-deb \
    --root-owner-group \
    --build \
    "$BUILD_ROOT" \
    "${DIST_DIR}/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"

echo
echo "Package created:"
echo "  ${DIST_DIR}/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
echo
