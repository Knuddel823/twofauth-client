mod api;
mod i18n;
mod models;
mod storage;
mod ui;

use adw::prelude::*;
use gtk::gio;

use crate::storage::config::load_config;
use crate::storage::keyring::load_token;

fn main() {
    let language = load_config()
        .map(|config| config.language)
        .unwrap_or_else(|_| "system".to_string());

    i18n::init(&language);

    let app = adw::Application::builder()
        .application_id("io.github.knuddel823.twofauth-client")
        .build();

    let settings_action = gio::SimpleAction::new("settings", None);

    {
        let app = app.clone();

        settings_action.connect_activate(move |_, _| {
            let Some(active_window) = app.active_window() else {
                return;
            };

            let Ok(parent) = active_window.downcast::<adw::ApplicationWindow>() else {
                return;
            };

            let settings = ui::settings_window::build_settings_window(&app, &parent);

            settings.present();
        });
    }

    app.add_action(&settings_action);

    app.connect_activate(|app| {
        if let Some(window) = app.active_window() {
            window.present();
            return;
        }

        let configured = load_config().is_ok() && load_token().is_ok();

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
