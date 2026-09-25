# TwoFAuth Client v0.1.7

## English

Version 0.1.7 focuses on improved mobile Linux support, navigation on small displays and expanded Linux packaging support.

### What's new

- **Improved mobile support**
  - Improved usability on small touch displays
  - Better integration with mobile Linux environments such as Phosh
  - Successfully tested on postmarketOS/aarch64

- **Back navigation**
  - Added a back button to the account details window
  - Added a back button to the settings window
  - Improved navigation on smartphones and other small displays

- **Updated application icons**
  - Updated application icons
  - Added multiple icon sizes for improved desktop and mobile integration

- **postmarketOS / Alpine Linux support**
  - Added native Alpine Linux and postmarketOS packaging support
  - APKBUILD successfully tested on postmarketOS/aarch64
  - Application and language packages can be built natively with Alpine's abuild tooling

- **Flatpak support**
  - Flatpak packaging updated for the v0.1.7 release source
  - Successfully built and tested with the GNOME 51 runtime
  - Secret Service integration is used for secure API token storage

### Packaging note

The immutable `v0.1.7` tag contains the application source for this release.

The updated Flatpak manifest and postmarketOS/Alpine packaging were added to the `main` branch immediately after the release tag and reference the unchanged v0.1.7 source.

TwoFAuth Client is currently not available from Flathub or the official postmarketOS/Alpine repositories.

For detailed installation and build instructions, see:

[Installation Guide](https://github.com/Knuddel823/twofauth-client/blob/main/docs/INSTALL.md)

### About

TwoFAuth Client is an unofficial native Linux client for 2FAuth.

It connects to an existing self-hosted 2FAuth instance and stores the API token securely using the system keyring / Secret Service.

---

## Deutsch

Version 0.1.7 konzentriert sich auf eine bessere Unterstützung mobiler Linux-Geräte, eine verbesserte Navigation auf kleinen Displays sowie eine erweiterte Linux-Paketierung.

### Neuerungen

- **Verbesserte Mobile-Unterstützung**
  - Verbesserte Bedienbarkeit auf kleinen Touch-Displays
  - Bessere Integration in mobile Linux-Umgebungen wie Phosh
  - Erfolgreich unter postmarketOS/aarch64 getestet

- **Zurück-Navigation**
  - Zurück-Schaltfläche in der Kontoansicht hinzugefügt
  - Zurück-Schaltfläche in den Einstellungen hinzugefügt
  - Verbesserte Navigation auf Smartphones und anderen kleinen Displays

- **Aktualisierte Anwendungssymbole**
  - Anwendungssymbole aktualisiert
  - Mehrere Icon-Größen für eine bessere Desktop- und Mobile-Integration hinzugefügt

- **postmarketOS / Alpine Linux**
  - Native Paketierung für Alpine Linux und postmarketOS ergänzt
  - APKBUILD erfolgreich unter postmarketOS/aarch64 getestet
  - Anwendung und Sprachpaket können nativ mit den Alpine-abuild-Werkzeugen erstellt werden

- **Flatpak-Unterstützung**
  - Flatpak-Paketierung für den v0.1.7-Release-Quellstand aktualisiert
  - Erfolgreich mit der GNOME-51-Runtime gebaut und getestet
  - Secret-Service-Integration zur sicheren Speicherung des API-Tokens

### Hinweis zur Paketierung

Der unveränderliche Tag `v0.1.7` enthält den eigentlichen Quellstand dieser Version.

Das aktualisierte Flatpak-Manifest sowie die postmarketOS-/Alpine-Paketierung wurden unmittelbar danach auf dem `main`-Branch ergänzt und verwenden weiterhin den unveränderten v0.1.7-Quellstand.

TwoFAuth Client ist derzeit weder über Flathub noch über die offiziellen postmarketOS-/Alpine-Repositories verfügbar.

Ausführliche Installations- und Build-Hinweise befinden sich hier:

[Installationsanleitung](https://github.com/Knuddel823/twofauth-client/blob/main/docs/INSTALL.md)

### Über TwoFAuth Client

TwoFAuth Client ist ein inoffizieller nativer Linux-Client für 2FAuth.

Die Anwendung verbindet sich mit einer bestehenden selbst gehosteten 2FAuth-Instanz und speichert den API-Token sicher im System-Keyring beziehungsweise über Secret Service.
