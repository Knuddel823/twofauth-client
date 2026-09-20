use std::path::Path;

use gettextrs::{LocaleCategory, bindtextdomain, gettext, setlocale, textdomain};

const DOMAIN: &str = "twofauth-client";

const DEVELOPMENT_LOCALE_DIR: &str = "po";
const FLATPAK_LOCALE_DIR: &str = "/app/share/locale";
const LOCAL_LOCALE_DIR: &str = "/usr/local/share/locale";
const SYSTEM_LOCALE_DIR: &str = "/usr/share/locale";

fn has_translation(locale_dir: &str) -> bool {
    ["de", "en", "fr", "es", "pt"].iter().any(|language| {
        Path::new(locale_dir)
            .join(language)
            .join("LC_MESSAGES")
            .join(format!("{DOMAIN}.mo"))
            .is_file()
    })
}

fn locale_dir() -> &'static str {
    if has_translation(DEVELOPMENT_LOCALE_DIR) {
        DEVELOPMENT_LOCALE_DIR
    } else if has_translation(FLATPAK_LOCALE_DIR) {
        FLATPAK_LOCALE_DIR
    } else if has_translation(LOCAL_LOCALE_DIR) {
        LOCAL_LOCALE_DIR
    } else {
        SYSTEM_LOCALE_DIR
    }
}

pub fn init(language: &str) {
    if language != "system" {
        // This runs before the GTK application and worker threads are started.
        unsafe {
            std::env::set_var("LANGUAGE", language);
        }
    }

    unsafe {
        setlocale(LocaleCategory::LcAll, "");
    }

    bindtextdomain(DOMAIN, locale_dir()).expect("Failed to bind gettext translation directory");

    textdomain(DOMAIN).expect("Failed to set gettext text domain");
}

pub fn tr(message: &str) -> String {
    gettext(message)
}

use crate::storage::config::ServerUrlError;

pub fn server_url_error_message(error: ServerUrlError) -> String {
    match error {
        ServerUrlError::Empty => tr("Please enter a server address."),
        ServerUrlError::InvalidUrl => tr("The server address is not a valid URL."),
        ServerUrlError::MissingHost => tr("The server address does not contain a valid host."),
        ServerUrlError::InsecureHttpDisabled => tr(
            "HTTP connections are disabled. Enable insecure HTTP connections if you understand and accept the risks.",
        ),
        ServerUrlError::UnsupportedScheme => tr(
            "Only HTTPS connections are supported by default. HTTP can be enabled explicitly in the security settings.",
        ),
    }
}

use crate::api::client::ApiError;

pub fn api_error_message(error: &ApiError) -> String {
    match error {
        ApiError::InvalidServerUrl(error) => server_url_error_message(*error),
        ApiError::ConnectionFailed(_) => tr("Failed to connect to the 2FAuth server."),
        ApiError::AuthenticationFailed => {
            tr("Authentication failed. The personal access token is invalid or has expired.")
        }
        ApiError::UnexpectedStatus(status) => {
            format!(
                "{} {}",
                tr("The 2FAuth server returned HTTP status"),
                status.as_u16()
            )
        }
        ApiError::InvalidResponse(_) => {
            tr("The response from the 2FAuth server could not be processed.")
        }
    }
}
