# TwoFAuth Client

TwoFAuth Client is an unofficial native Linux desktop client for
[2FAuth](https://github.com/Bubka/2FAuth).

It is written in Rust and uses GTK4 and libadwaita to provide a simple
desktop interface for accessing OTP accounts stored on a self-hosted
2FAuth server.

> [!IMPORTANT]
> TwoFAuth Client is an independent community project and is not
> affiliated with or endorsed by the official 2FAuth project.

## Current status

TwoFAuth Client is currently in early development.

The first release focuses deliberately on providing a simple and
read-oriented desktop client for existing 2FAuth installations.

## Features

- Connect to a self-hosted 2FAuth server
- Authenticate using a Personal Access Token (PAT)
- Securely store the PAT in the operating system keyring
- Display existing 2FAuth accounts
- Search accounts
- Display TOTP codes
- Support HOTP accounts
- Synchronize TOTP countdowns with the 2FAuth server time
- Copy OTP codes to the clipboard
- Display account icons from 2FAuth
- Cache account icons locally
- Display accounts by 2FAuth group
- Collapse and expand account groups
- Remember collapsed group state
- Display OTP type, algorithm and digit count
- Configure, test and reset the 2FAuth connection
- HTTPS required by default
- Optional insecure HTTP support with explicit security confirmation
- Network connection and request timeouts
- Built-in help for creating a Personal Access Token
- English, German, French, Spanish and Portuguese interface
- Native GTK4/libadwaita user interface

## Read-oriented by design

TwoFAuth Client currently has a deliberately limited scope.

The application retrieves existing account information and OTP codes
from 2FAuth. It does **not** create, modify or delete 2FAuth accounts.

This is intentional.

For the initial releases, the goal is to provide a small and focused
desktop client with as little write access to the 2FAuth installation
as possible.

Account management should continue to be performed through the official
2FAuth web interface.

Support for creating or editing accounts may be considered for a much
later version, but it is currently outside the scope of the project.

## Requirements

TwoFAuth Client currently targets Linux desktops.

Runtime requirements include:

- GTK4
- libadwaita
- a compatible system keyring / Secret Service implementation
- network access to a 2FAuth server
- a 2FAuth Personal Access Token

Building from source additionally requires:

- Rust / Cargo
- gettext
- GTK4 development files
- libadwaita development files

## Building from source

Clone the repository:

~~~bash
git clone https://github.com/Knuddel823/twofauth-client.git
cd twofauth-client
~~~

Build the application:

~~~bash
cargo build --release
~~~

The resulting executable is located at:

~~~text
target/release/twofauth-client
~~~

For development, the application can also be started with:

~~~bash
cargo run
~~~

## Installation

A simple installation script is included:

~~~bash
./scripts/install.sh
~~~

The application is installed below `/usr/local`.

The installation includes:

- `/usr/local/bin/twofauth-client`
- desktop integration
- application icon
- gettext translations

Only installation steps requiring elevated privileges use `sudo`.
The Rust build itself runs as the current user.

After installation, TwoFAuth Client can be started from the desktop
application menu or from a terminal:

~~~bash
twofauth-client
~~~

## Configuration

On first start, TwoFAuth Client asks for:

1. the URL of your 2FAuth server
2. a Personal Access Token

A Personal Access Token can be created in the 2FAuth web interface under:

**Settings → OAuth → Personal Access Tokens**

Create a new token, give it a name and copy the generated token into
TwoFAuth Client.

The server URL is stored in the application configuration.

HTTPS is required by default. Plain HTTP connections are disabled unless
they are explicitly enabled by the user. Enabling HTTP requires a security
confirmation because the Personal Access Token, account information and OTP
codes may otherwise be transmitted without encryption.

The Personal Access Token is stored separately in the operating
system's secure keyring and is not written to the application
configuration file.

## User data

TwoFAuth Client stores different types of local data separately.

Configuration and UI state are stored below the user's configuration
directory, typically:

~~~text
~/.config/twofauth-client/
~~~

Downloaded account icons are cached below:

~~~text
~/.cache/twofauth-client/icons/
~~~

The Personal Access Token is **not** stored in either location. It is
stored in the system keyring.

## Security

TwoFAuth Client handles access to sensitive authentication information.

The project therefore aims to keep its access to the 2FAuth server as
limited as practical.

In particular:

- the Personal Access Token is stored in the system keyring
- the token is not stored in the application configuration file
- account creation is not implemented
- account modification is not implemented
- account deletion is not implemented
- OTP information is retrieved from the configured 2FAuth server when required
- cached account icons do not contain OTP secrets

Development versions should still be treated as development software.
Please review the source code and security implications before using
them with important accounts.

## Translations

TwoFAuth Client currently includes:

- English
- German
- French
- Spanish
- Portuguese

The application follows the system language by default. A language can also
be selected manually in the application settings. Language changes take
effect after restarting TwoFAuth Client.

> **Translation notice:** Some translations were created with the assistance
> of machine translation and may contain inaccuracies. Native speakers are
> welcome to report translation issues or suggest improvements.

Translation source files are located in the `po/` directory.

Compiled `.mo` files are generated during installation and are not
stored in the Git repository.

## Uninstallation

To remove a manual `/usr/local` installation:

~~~bash
./scripts/uninstall.sh
~~~

The uninstall script removes the installed application files.

User configuration, cached icons and keyring data are intentionally
left untouched.

## Roadmap

Possible future improvements include:

- Flatpak packaging
- GitHub Actions for automated builds
- additional translations
- improved accessibility
- further desktop integration
- additional safe, read-oriented 2FAuth API features

Creating, editing or deleting 2FAuth accounts is intentionally **not**
part of the near-term roadmap.

Account management features may be considered in a much later version
after the security and API implications have been evaluated carefully.

## About 2FAuth

TwoFAuth Client requires an existing 2FAuth installation.

2FAuth itself is a separate open-source project maintained independently
from TwoFAuth Client.

For server installation, configuration and account management, please
refer to the official 2FAuth project:

https://github.com/Bubka/2FAuth

## Contributing

Bug reports, feature suggestions, translations and code contributions
are welcome.

Source code:

https://github.com/Knuddel823/twofauth-client

Issue tracker:

https://github.com/Knuddel823/twofauth-client/issues

## Support

TwoFAuth Client is free and open-source software.

If you like the project and would like to support its development, you
can buy me a coffee:

https://buymeacoffee.com/knuddel823

Support is entirely optional and does not unlock additional features.

## License

TwoFAuth Client is licensed under the GNU General Public License,
version 3 or later (`GPL-3.0-or-later`).

See [LICENSE](LICENSE) for the full license text.

Copyright © 2026 Marcel Müller
