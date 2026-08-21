mod api;
mod i18n;
mod models;
mod ui;

use adw::prelude::*;

fn main() {
    i18n::init();

    let app = adw::Application::builder()
        .application_id("de.twofauthclient.TwoFAuthClient")
        .build();

    app.connect_activate(|app| {
        let window = ui::main_window::build_main_window(app);
        window.present();
    });

    app.run();
}
