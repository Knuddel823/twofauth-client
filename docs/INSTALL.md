# Installation

This document describes the currently supported installation methods for 2FAuth Client.

2FAuth Client is an unofficial native Linux client for 2FAuth.

The application connects to an existing self-hosted 2FAuth instance and stores the API token securely using the system Secret Service/keyring.

## Current release

Current version:

    0.1.7

These instructions refer to version `v0.1.7`.

> **Important**
>
> Flatpak and postmarketOS/Alpine packaging are available in the repository, but 2FAuth Client is not currently distributed through Flathub or the official postmarketOS/Alpine repositories.

---

## Debian / Ubuntu

### Requirements

Install the required development packages:

    sudo apt update

    sudo apt install \
        build-essential \
        cargo \
        rustc \
        pkg-config \
        libgtk-4-dev \
        libadwaita-1-dev \
        gettext

If the Rust version provided by the distribution is too old, install a current Rust toolchain using rustup.

### Clone the repository

    git clone https://github.com/Knuddel823/twofauth-client.git
    cd twofauth-client
    git checkout v0.1.7

### Build

    cargo build --release --locked

The resulting executable is located at:

    target/release/twofauth-client

Start it with:

    ./target/release/twofauth-client

### Run the tests

    cargo test --locked

### Install locally

The repository includes an installation script that builds the application and installs it below `/usr/local`:

    ./scripts/install.sh

The installation includes:

    /usr/local/bin/twofauth-client
    /usr/local/share/applications/io.github.knuddel823.twofauth-client.desktop
    /usr/local/share/metainfo/io.github.knuddel823.twofauth-client.metainfo.xml

Application icons and translations are installed below the corresponding `/usr/local/share` directories.

Only installation steps requiring elevated privileges use `sudo`. The Rust build itself runs as the current user.

After installation, start TwoFAuth Client from the desktop application menu or run:

    twofauth-client

### Uninstall

A matching uninstall script is included:

    ./scripts/uninstall.sh

The uninstall script removes the installed application files.

User configuration, cached icons and keyring data are intentionally left untouched.

---

## Flatpak

The repository contains the Flatpak manifest:

    flatpak/io.github.knuddel823.twofauth-client.yml

The current Flatpak configuration uses:

    org.gnome.Platform 51
    org.gnome.Sdk 51
    org.freedesktop.Sdk.Extension.rust-stable

### Install Flatpak Builder

Make sure Flatpak is installed and Flathub is configured.

Install the GNOME runtime and SDK:

    flatpak install flathub \
        org.gnome.Platform//51 \
        org.gnome.Sdk//51

Install Flatpak Builder:

    flatpak install flathub org.flatpak.Builder

### Clone the repository

The Flatpak packaging file was updated after the immutable `v0.1.7` source tag, so use the current `main` branch:

    git clone https://github.com/Knuddel823/twofauth-client.git
    cd twofauth-client

The manifest itself references the immutable `v0.1.7` release source.

### Build and install

Run from the repository root:

    flatpak run org.flatpak.Builder \
        --force-clean \
        --user \
        --install \
        flatpak/build-dir \
        flatpak/io.github.knuddel823.twofauth-client.yml

After a successful build, start the application with:

    flatpak run io.github.knuddel823.twofauth-client

### Flatpak permissions

The Flatpak requires:

- Network access to communicate with the configured 2FAuth server
- Wayland support
- Fallback X11 support
- GPU access
- Secret Service access for secure API token storage

The required permissions are already defined in the Flatpak manifest.

### Flathub status

2FAuth Client is currently **not available on Flathub**.

The included manifest can be used to build and install the application locally.

---

## Alpine Linux / postmarketOS

A native Alpine Linux `APKBUILD` is included at:

    packaging/postmarketos/APKBUILD

The package has been successfully built and tested on postmarketOS/aarch64.

### Install the Alpine build environment

    sudo apk add alpine-sdk

The `APKBUILD` declares the application-specific build dependencies automatically.

### Configure abuild

If `abuild` has not yet been configured for the current user:

    abuild-keygen -a -i

The user normally also needs to be a member of the `abuild` group.

### Clone the repository

The postmarketOS packaging file was added after the immutable `v0.1.7` source tag, so use the current `main` branch:

    git clone https://github.com/Knuddel823/twofauth-client.git
    cd twofauth-client

The `APKBUILD` itself downloads the official `v0.1.7` release archive.

### Prepare the build directory

    mkdir -p ~/apkbuild-twofauth-client
    cp packaging/postmarketos/APKBUILD ~/apkbuild-twofauth-client/APKBUILD
    cd ~/apkbuild-twofauth-client

### Verify the source

    abuild checksum

### Validate the APKBUILD

    abuild validate

### Build the packages

    abuild -r

The build creates the main application package and a separate language package:

    twofauth-client-0.1.7-r0.apk
    twofauth-client-lang-0.1.7-r0.apk

The exact output directory depends on the local `abuild` configuration and architecture.

A typical aarch64 location is:

    ~/.local/share/abuild/<user>/aarch64/

### Tested platform

Version 0.1.7 has been successfully built and tested on:

    postmarketOS edge
    aarch64
    Phosh

The postmarketOS version uses the same Rust, GTK4 and libadwaita codebase as the desktop application. There is no separate mobile fork.

### Repository status

2FAuth Client is currently **not available from the official postmarketOS or Alpine Linux repositories**.

---

## Build directly from source

### Requirements

2FAuth Client requires:

- Rust
- Cargo
- GTK 4
- libadwaita
- gettext
- pkg-config
- A Secret Service compatible system keyring

Clone the release source:

    git clone https://github.com/Knuddel823/twofauth-client.git
    cd twofauth-client
    git checkout v0.1.7

Build:

    cargo build --release --locked

Run:

    ./target/release/twofauth-client

Run the tests:

    cargo test --locked

---

## First start

On first start, 2FAuth Client opens the setup assistant.

Configure:

1. The URL of your self-hosted 2FAuth instance
2. Your 2FAuth API token
3. The desired language or system default

HTTPS should be used for normal installations.

HTTP connections are disabled by default and must be explicitly enabled when required for a trusted local environment.

The API token is stored in the system keyring using Secret Service.

---

## Troubleshooting

### API token cannot be accessed

Make sure a Secret Service compatible keyring is available and unlocked in the graphical session.

Desktop environments such as GNOME normally provide this through GNOME Keyring or another Secret Service implementation.

### Cannot connect to the 2FAuth server

Check:

- The configured server URL
- Network connectivity
- HTTPS certificate validity
- The API token
- Whether insecure HTTP was explicitly enabled when using an HTTP-only local server

2FAuth Client does not automatically follow HTTP redirects.

### Flatpak cannot access the keyring

The Flatpak manifest grants access to:

    org.freedesktop.secrets

A compatible Secret Service provider must also be running in the host graphical session.

---

## Project

Source code and issue tracker:

https://github.com/Knuddel823/twofauth-client

License:

    GPL-3.0-or-later
