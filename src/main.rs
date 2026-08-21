mod api;
mod i18n;
mod models;
mod storage;
mod ui;

use adw::prelude::*;

use crate::storage::config::config_exists;
use crate::storage::keyring::load_token;

fn main() {
    i18n::init();

    let app = adw::Application::builder()
        .application_id("de.twofauthclient.TwoFAuthClient")
        .build();

    app.connect_activate(|app| {
        let configured = config_exists() && load_token().is_ok();

        if configured {
            let window = ui::main_window::build_main_window(app);
            window.present();
        } else {
            let window = ui::setup_window::build_setup_window(app);
            window.present();
        }
    });

    app.run();
}
