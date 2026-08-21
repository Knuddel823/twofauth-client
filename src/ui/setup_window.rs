use std::sync::mpsc;
use std::time::Duration;

use adw::prelude::*;
use gtk::glib;

use crate::api::client::TwoFAuthClient;
use crate::i18n::tr;
use crate::storage::config::{AppConfig, save_config};
use crate::storage::keyring::save_token;

enum SetupResult {
    Success,
    Error(String),
}

pub fn build_setup_window(app: &adw::Application) -> adw::ApplicationWindow {
    let header = adw::HeaderBar::new();

    let title = gtk::Label::new(Some(&tr("TwoFAuth Client")));
    title.add_css_class("title");
    header.set_title_widget(Some(&title));

    let heading = gtk::Label::new(Some(&tr("Connect to 2FAuth")));
    heading.add_css_class("title-1");
    heading.set_halign(gtk::Align::Start);

    let description = gtk::Label::new(Some(&tr(
        "Enter the address of your 2FAuth server and a personal access token.",
    )));
    description.set_wrap(true);
    description.set_halign(gtk::Align::Start);
    description.add_css_class("dim-label");

    let server_label = gtk::Label::new(Some(&tr("Server")));
    server_label.set_halign(gtk::Align::Start);

    let server_entry = gtk::Entry::new();
    server_entry.set_placeholder_text(Some("https://2fauth.example.com"));
    server_entry.set_hexpand(true);

    let token_label = gtk::Label::new(Some(&tr("Personal access token")));
    token_label.set_halign(gtk::Align::Start);

    let token_entry = gtk::PasswordEntry::new();
    token_entry.set_show_peek_icon(true);
    token_entry.set_hexpand(true);

    let status = gtk::Label::new(None);
    status.set_wrap(true);
    status.set_halign(gtk::Align::Start);
    status.set_visible(false);

    let connect_button = gtk::Button::with_label(&tr("Connect"));
    connect_button.add_css_class("suggested-action");
    connect_button.add_css_class("pill");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);

    content.append(&heading);
    content.append(&description);
    content.append(&server_label);
    content.append(&server_entry);
    content.append(&token_label);
    content.append(&token_entry);
    content.append(&status);
    content.append(&connect_button);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.append(&header);
    root.append(&content);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(tr("TwoFAuth Client"))
        .default_width(420)
        .default_height(500)
        .content(&root)
        .build();

    {
        let app = app.clone();
        let window = window.clone();
        let server_entry = server_entry.clone();
        let token_entry = token_entry.clone();
        let status = status.clone();

        // Eigener Clone für die Callback-Closure.
        // Dadurch vermeiden wir E0505 beim connect_clicked-Aufruf.
        let connect_button_for_click = connect_button.clone();

        connect_button.connect_clicked(move |_| {
            let server_url = server_entry.text().trim().to_string();
            let token = token_entry.text().to_string();

            if server_url.is_empty() {
                status.set_text(&tr("Please enter a server address."));
                status.set_visible(true);
                return;
            }

            if token.is_empty() {
                status.set_text(&tr("Please enter a personal access token."));
                status.set_visible(true);
                return;
            }

            connect_button_for_click.set_sensitive(false);

            status.set_text(&tr("Testing connection..."));
            status.set_visible(true);

            let (sender, receiver) = mpsc::channel::<SetupResult>();

            let server_url_for_thread = server_url.clone();
            let token_for_thread = token.clone();

            std::thread::spawn(move || {
                let runtime = match tokio::runtime::Runtime::new() {
                    Ok(runtime) => runtime,

                    Err(error) => {
                        let _ = sender.send(SetupResult::Error(error.to_string()));
                        return;
                    }
                };

                let result = runtime.block_on(async {
                    let client = TwoFAuthClient::new(&server_url_for_thread, &token_for_thread);

                    client.get_accounts().await
                });

                match result {
                    Ok(_) => {
                        let config = AppConfig::new(&server_url_for_thread);

                        if let Err(error) = save_config(&config) {
                            let _ = sender.send(SetupResult::Error(error.to_string()));
                            return;
                        }

                        if let Err(error) = save_token(&token_for_thread) {
                            let _ = sender.send(SetupResult::Error(error.to_string()));
                            return;
                        }

                        let _ = sender.send(SetupResult::Success);
                    }

                    Err(error) => {
                        let _ = sender.send(SetupResult::Error(error.to_string()));
                    }
                }
            });

            let app = app.clone();
            let window = window.clone();
            let status = status.clone();
            let connect_button = connect_button_for_click.clone();

            glib::timeout_add_local(Duration::from_millis(100), move || {
                match receiver.try_recv() {
                    Ok(SetupResult::Success) => {
                        let main_window = crate::ui::main_window::build_main_window(&app);

                        main_window.present();
                        window.close();

                        glib::ControlFlow::Break
                    }

                    Ok(SetupResult::Error(error)) => {
                        status.set_text(&format!("{}\n{}", tr("Connection failed"), error));

                        connect_button.set_sensitive(true);

                        glib::ControlFlow::Break
                    }

                    Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,

                    Err(mpsc::TryRecvError::Disconnected) => {
                        status.set_text(&tr("Connection failed"));
                        connect_button.set_sensitive(true);

                        glib::ControlFlow::Break
                    }
                }
            });
        });
    }

    window
}
