# TwoFAuth Client v0.1.6

## English

Version 0.1.6 focuses on Flatpak support, desktop integration and preparation for wider Linux distribution.

### What's new

- **Flatpak support**
  - Added a complete Flatpak manifest
  - Added offline Cargo dependency sources for reproducible Flatpak builds
  - Updated the Flatpak runtime to GNOME 51
  - Added documentation for local Flatpak builds and installation

- **Application ID migration**
  - Migrated the application ID to `io.github.knuddel823.twofauth-client`
  - Updated desktop integration, AppStream metadata and installation scripts
  - Updated application metadata and localization integration for the new ID

- **AppStream and screenshots**
  - Added application screenshots for AppStream and Flatpak presentation
  - Added updated screenshots to the project documentation
  - Improved AppStream metadata for application presentation

- **Project integration**
  - Updated project metadata to use the GitHub project identity
  - Improved uninstall handling for previous application IDs

- **Security tooling**
  - Added Gitleaks secret scanning through GitHub Actions

### Flatpak status

The Flatpak manifest included with this release can be used for local builds.

TwoFAuth Client is not currently available from Flathub.

### About

TwoFAuth Client is an unofficial native Linux client for 2FAuth.

The application connects to an existing self-hosted 2FAuth instance and stores the Personal Access Token securely in the system keyring.

---

## Deutsch

Version 0.1.6 konzentriert sich auf die Flatpak-Unterstützung, die Desktop-Integration und die Vorbereitung für eine breitere Linux-Verteilung.

### Neuerungen

- **Flatpak-Unterstützung**
  - Vollständiges Flatpak-Manifest hinzugefügt
  - Offline-Cargo-Abhängigkeiten für reproduzierbare Flatpak-Builds ergänzt
  - Flatpak-Runtime auf GNOME 51 aktualisiert
  - Dokumentation für lokale Flatpak-Builds und Installation ergänzt

- **Migration der Application-ID**
  - Application-ID auf `io.github.knuddel823.twofauth-client` umgestellt
  - Desktop-Integration, AppStream-Metadaten und Installationsskripte angepasst
  - Anwendungsmetadaten und Lokalisierungsintegration auf die neue ID aktualisiert

- **AppStream und Screenshots**
  - Anwendungsscreenshots für AppStream und Flatpak ergänzt
  - Aktualisierte Screenshots in die Projektdokumentation aufgenommen
  - AppStream-Metadaten für die Darstellung der Anwendung verbessert

- **Projektintegration**
  - Projektmetadaten auf die GitHub-Projektidentität umgestellt
  - Deinstallation älterer Application-IDs verbessert

- **Sicherheitswerkzeuge**
  - Gitleaks Secret Scanning über GitHub Actions hinzugefügt

### Flatpak-Status

Das mit dieser Version enthaltene Flatpak-Manifest kann für lokale Builds verwendet werden.

TwoFAuth Client ist derzeit noch nicht über Flathub verfügbar.

### Über TwoFAuth Client

TwoFAuth Client ist ein inoffizieller nativer Linux-Client für 2FAuth.

Die Anwendung verbindet sich mit einer bestehenden selbst gehosteten 2FAuth-Instanz und speichert den Personal Access Token sicher im System-Keyring.
