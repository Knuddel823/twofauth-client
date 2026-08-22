use std::path::Path;

use gettextrs::{LocaleCategory, bindtextdomain, gettext, setlocale, textdomain};

const DOMAIN: &str = "twofauth-client";

const DEVELOPMENT_LOCALE_DIR: &str = "po";
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
    if Path::new(DEVELOPMENT_LOCALE_DIR).is_dir() {
        DEVELOPMENT_LOCALE_DIR
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
