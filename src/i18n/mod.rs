use gettextrs::{LocaleCategory, bindtextdomain, gettext, setlocale, textdomain};

const DOMAIN: &str = "twofauth-client";

pub fn init() {
    unsafe {
        setlocale(LocaleCategory::LcAll, "");
    }

    bindtextdomain(DOMAIN, "po").expect("Failed to bind gettext translation directory");

    textdomain(DOMAIN).expect("Failed to set gettext text domain");
}

pub fn tr(message: &str) -> String {
    gettext(message)
}
