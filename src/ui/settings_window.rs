use std::sync::mpsc;
use std::time::Duration;

use adw::prelude::*;
use gtk::glib;

use crate::api::client::TwoFAuthClient;
use crate::i18n::tr;
use crate::storage::config::{AppConfig, delete_config, load_config, save_config};
use crate::storage::keyring::{delete_token, load_token, save_token};

enum SettingsResult {
    Success,
    Error(String),
}

pub fn build_settings_window(
    app: &adw::Application,
    parent: &adw::ApplicationWindow,
) -> adw::ApplicationWindow {
    let current_config = load_config().ok();

    let header = adw::HeaderBar::new();

    let title = gtk::Label::new(Some(&tr("Settings")));
    title.add_css_class("title");
    header.set_title_widget(Some(&title));

    let server_label = gtk::Label::new(Some(&tr("Server")));
    server_label.set_halign(gtk::Align::Start);

    let server_entry = gtk::Entry::new();
    server_entry.set_hexpand(true);

    if let Some(config) = &current_config {
        server_entry.set_text(&config.server_url);
    }

    let token_status = gtk::Label::new(Some(&tr("Access token is stored securely.")));
    token_status.set_halign(gtk::Align::Start);
    token_status.add_css_class("dim-label");

    let token_label = gtk::Label::new(Some(&tr("New personal access token")));
    token_label.set_halign(gtk::Align::Start);

    let token_entry = gtk::PasswordEntry::new();
    token_entry.set_show_peek_icon(true);
    token_entry.set_hexpand(true);
    token_entry.set_placeholder_text(Some(&tr("Leave empty to keep the current token")));

    let status = gtk::Label::new(None);
    status.set_wrap(true);
    status.set_halign(gtk::Align::Start);
    status.set_visible(false);

    let save_button = gtk::Button::with_label(&tr("Test and save"));
    save_button.add_css_class("suggested-action");

    let reset_button = gtk::Button::with_label(&tr("Reset connection"));
    reset_button.add_css_class("destructive-action");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);

    content.append(&server_label);
    content.append(&server_entry);
    content.append(&token_status);
    content.append(&token_label);
    content.append(&token_entry);
    content.append(&status);
    content.append(&save_button);
    content.append(&reset_button);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.append(&header);
    root.append(&content);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .transient_for(parent)
        .modal(true)
        .title(tr("Settings"))
        .default_width(420)
        .default_height(520)
        .content(&root)
        .build();

    {
        let server_entry = server_entry.clone();
        let token_entry = token_entry.clone();
        let status = status.clone();
        let save_button_for_click = save_button.clone();

        save_button.connect_clicked(move |_| {
            let server_url = server_entry.text().trim().to_string();
            let new_token = token_entry.text().to_string();

            if server_url.is_empty() {
                status.set_text(&tr("Please enter a server address."));
                status.set_visible(true);
                return;
            }

            let token = if new_token.is_empty() {
                match load_token() {
                    Ok(token) => token,
                    Err(error) => {
                        status.set_text(&format!(
                            "{}\n{}",
                            tr("Could not load the stored access token."),
                            error
                        ));
                        status.set_visible(true);
                        return;
                    }
                }
            } else {
                new_token.clone()
            };

            save_button_for_click.set_sensitive(false);
            status.set_text(&tr("Testing connection..."));
            status.set_visible(true);

            let (sender, receiver) = mpsc::channel::<SettingsResult>();

            let server_url_for_thread = server_url.clone();
            let token_for_thread = token.clone();
            let should_save_new_token = !new_token.is_empty();

            std::thread::spawn(move || {
                let runtime = match tokio::runtime::Runtime::new() {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        let _ = sender.send(SettingsResult::Error(error.to_string()));
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
                            let _ = sender.send(SettingsResult::Error(error.to_string()));
                            return;
                        }

                        if should_save_new_token {
                            if let Err(error) = save_token(&token_for_thread) {
                                let _ = sender.send(SettingsResult::Error(error.to_string()));
                                return;
                            }
                        }

                        let _ = sender.send(SettingsResult::Success);
                    }

                    Err(error) => {
                        let _ = sender.send(SettingsResult::Error(error.to_string()));
                    }
                }
            });

            let status = status.clone();
            let save_button = save_button_for_click.clone();
            let token_entry = token_entry.clone();

            glib::timeout_add_local(Duration::from_millis(100), move || {
                match receiver.try_recv() {
                    Ok(SettingsResult::Success) => {
                        status.set_text(&tr("Settings saved successfully."));
                        status.set_visible(true);

                        token_entry.set_text("");
                        save_button.set_sensitive(true);

                        glib::ControlFlow::Break
                    }

                    Ok(SettingsResult::Error(error)) => {
                        status.set_text(&format!("{}\n{}", tr("Connection failed"), error));

                        save_button.set_sensitive(true);

                        glib::ControlFlow::Break
                    }

                    Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,

                    Err(mpsc::TryRecvError::Disconnected) => {
                        status.set_text(&tr("Connection failed"));
                        save_button.set_sensitive(true);

                        glib::ControlFlow::Break
                    }
                }
            });
        });
    }

    {
        let app = app.clone();
        let parent = parent.clone();
        let window = window.clone();

        reset_button.connect_clicked(move |_| {
            let config_result = delete_config();
            let token_result = delete_token();

            if let Err(error) = config_result {
                eprintln!("Could not delete config: {error}");
            }

            if let Err(error) = token_result {
                eprintln!("Could not delete token: {error}");
            }

            window.close();
            parent.close();

            let setup = crate::ui::setup_window::build_setup_window(&app);
            setup.present();
        });
    }

    window
}
