use std::path::Path;

use gettextrs::{LocaleCategory, bindtextdomain, gettext, setlocale, textdomain};

const DOMAIN: &str = "twofauth-client";
const DEVELOPMENT_LOCALE_DIR: &str = "po";
const SYSTEM_LOCALE_DIR: &str = "/usr/local/share/locale";

pub fn init() {
    unsafe {
        setlocale(LocaleCategory::LcAll, "");
    }

    let locale_dir = if Path::new(DEVELOPMENT_LOCALE_DIR).is_dir() {
        DEVELOPMENT_LOCALE_DIR
    } else {
        SYSTEM_LOCALE_DIR
    };

    bindtextdomain(DOMAIN, locale_dir).expect("Failed to bind gettext translation directory");

    textdomain(DOMAIN).expect("Failed to set gettext text domain");
}

pub fn tr(message: &str) -> String {
    gettext(message)
}
