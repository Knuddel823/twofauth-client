use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use adw::prelude::*;
use gtk::glib;

use crate::api::client::{OtpResponse, TwoFAuthClient};
use crate::i18n::tr;
use crate::models::account::TwoFAccount;
use crate::models::group::TwoFGroup;
use crate::storage::config::load_config;
use crate::storage::keyring::load_token;
use crate::ui::icon_loader::load_account_icon;

enum LoadResult {
    Success {
        accounts: Vec<TwoFAccount>,
        groups: Vec<TwoFGroup>,
    },
    Error(String),
}

enum OtpResult {
    Success(OtpResponse),
    Error(String),
}

struct OtpSyncState {
    server_time: SystemTime,
    received_at: Instant,
    period_index: u64,
}

pub fn build_main_window(app: &adw::Application) -> adw::ApplicationWindow {
    let header = adw::HeaderBar::new();

    let title = gtk::Label::new(Some(&tr("TwoFAuth Client")));
    title.add_css_class("title");
    header.set_title_widget(Some(&title));

    let menu_button = gtk::MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");

    let menu = gtk::gio::Menu::new();
    menu.append(Some(&tr("Settings")), Some("app.settings"));

    menu_button.set_menu_model(Some(&menu));
    header.pack_end(&menu_button);

    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some(&tr("Search accounts...")));
    search.set_hexpand(true);

    let status = gtk::Label::new(Some(&tr("Loading accounts...")));
    status.set_margin_top(24);
    status.set_margin_bottom(24);

    let account_list = gtk::ListBox::new();
    account_list.set_selection_mode(gtk::SelectionMode::Single);
    account_list.add_css_class("boxed-list");

    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_child(Some(&account_list));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    content.append(&search);
    content.append(&status);
    content.append(&scrolled);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.append(&header);
    root.append(&content);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(tr("TwoFAuth Client"))
        .default_width(420)
        .default_height(640)
        .content(&root)
        .build();

    let accounts: Rc<RefCell<Vec<TwoFAccount>>> = Rc::new(RefCell::new(Vec::new()));

    let groups: Rc<RefCell<Vec<TwoFGroup>>> = Rc::new(RefCell::new(Vec::new()));

    {
        let accounts = Rc::clone(&accounts);
        let groups = Rc::clone(&groups);
        let account_list = account_list.clone();

        search.connect_search_changed(move |search| {
            rebuild_account_list(
                &account_list,
                &accounts.borrow(),
                &groups.borrow(),
                &search.text(),
            );
        });
    }

    {
        let accounts = Rc::clone(&accounts);
        let window = window.clone();

        account_list.connect_row_activated(move |_list, row| {
            let account_id = match row.widget_name().as_str().parse::<u64>() {
                Ok(id) => id,
                Err(_) => return,
            };

            let account = {
                accounts
                    .borrow()
                    .iter()
                    .find(|account| account.id == account_id)
                    .cloned()
            };

            if let Some(account) = account {
                show_otp_dialog(&window, account);
            }
        });
    }

    let (sender, receiver) = mpsc::channel::<LoadResult>();

    std::thread::spawn(move || {
        let config = match load_config() {
            Ok(config) => config,

            Err(error) => {
                let _ = sender.send(LoadResult::Error(error.to_string()));
                return;
            }
        };

        let token = match load_token() {
            Ok(token) => token,

            Err(error) => {
                let _ = sender.send(LoadResult::Error(error.to_string()));
                return;
            }
        };

        let server_url = config.server_url;

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,

            Err(error) => {
                let _ = sender.send(LoadResult::Error(error.to_string()));
                return;
            }
        };

        let result = runtime.block_on(async {
            let client = TwoFAuthClient::new(server_url, token);

            let accounts = client.get_accounts().await?;

            // Gruppen sind ein optionales Komfort-Feature.
            // Falls ein älterer 2FAuth-Server den Endpoint nicht anbietet,
            // funktioniert die Accountliste weiterhin ohne Gruppierung.
            let groups = client.get_groups().await.unwrap_or_default();

            Ok::<_, anyhow::Error>((accounts, groups))
        });

        match result {
            Ok((accounts, groups)) => {
                let _ = sender.send(LoadResult::Success { accounts, groups });
            }

            Err(error) => {
                let _ = sender.send(LoadResult::Error(error.to_string()));
            }
        }
    });

    {
        let accounts = Rc::clone(&accounts);
        let groups = Rc::clone(&groups);
        let account_list = account_list.clone();
        let status = status.clone();

        glib::timeout_add_local(Duration::from_millis(100), move || {
            match receiver.try_recv() {
                Ok(LoadResult::Success {
                    accounts: loaded_accounts,
                    groups: loaded_groups,
                }) => {
                    *accounts.borrow_mut() = loaded_accounts;

                    // ID 0 ist bei 2FAuth die virtuelle Gruppe "All".
                    *groups.borrow_mut() = loaded_groups
                        .into_iter()
                        .filter(|group| group.id != 0)
                        .collect();

                    status.set_visible(false);

                    rebuild_account_list(&account_list, &accounts.borrow(), &groups.borrow(), "");

                    glib::ControlFlow::Break
                }

                Ok(LoadResult::Error(error)) => {
                    status.set_text(&format!("{}\n{}", tr("Could not load accounts"), error));

                    glib::ControlFlow::Break
                }

                Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,

                Err(mpsc::TryRecvError::Disconnected) => {
                    status.set_text(&tr("Could not load accounts"));

                    glib::ControlFlow::Break
                }
            }
        });
    }

    window
}

fn rebuild_account_list(
    list: &gtk::ListBox,
    accounts: &[TwoFAccount],
    groups: &[TwoFGroup],
    search_text: &str,
) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    let search_text = search_text.trim().to_lowercase();

    let filtered_accounts: Vec<&TwoFAccount> = accounts
        .iter()
        .filter(|account| {
            search_text.is_empty()
                || account.service.to_lowercase().contains(&search_text)
                || account.account.to_lowercase().contains(&search_text)
        })
        .collect();

    if filtered_accounts.is_empty() {
        let empty = gtk::Label::new(Some(&tr("No accounts found")));

        empty.set_margin_top(24);
        empty.set_margin_bottom(24);

        list.append(&empty);
        return;
    }

    // Ohne echte Gruppen bleibt die bisherige flache Darstellung bestehen.
    if groups.is_empty() {
        for account in filtered_accounts {
            let row = build_account_row(account);
            list.append(&row);
        }

        return;
    }

    for group in groups {
        let grouped_accounts: Vec<&TwoFAccount> = filtered_accounts
            .iter()
            .copied()
            .filter(|account| account.group_id == Some(group.id))
            .collect();

        if grouped_accounts.is_empty() {
            continue;
        }

        let visible_count = grouped_accounts.len() as u64;

        let count = if search_text.is_empty() {
            group.twofaccounts_count
        } else {
            visible_count
        };

        append_collapsible_group(list, &group.name, count, grouped_accounts);
    }

    let known_group_ids: Vec<u64> = groups.iter().map(|group| group.id).collect();

    let ungrouped_accounts: Vec<&TwoFAccount> = filtered_accounts
        .iter()
        .copied()
        .filter(|account| match account.group_id {
            None => true,

            Some(group_id) => !known_group_ids.contains(&group_id),
        })
        .collect();

    if !ungrouped_accounts.is_empty() {
        append_collapsible_group(
            list,
            &tr("Without group"),
            ungrouped_accounts.len() as u64,
            ungrouped_accounts,
        );
    }
}

fn append_collapsible_group(
    list: &gtk::ListBox,
    name: &str,
    count: u64,
    accounts: Vec<&TwoFAccount>,
) {
    let expanded = Rc::new(Cell::new(true));

    let arrow = gtk::Image::from_icon_name("pan-down-symbolic");

    arrow.set_pixel_size(16);

    let title = gtk::Label::new(Some(&format!("{} ({})", name, count)));

    title.set_xalign(0.0);
    title.add_css_class("heading");
    title.set_hexpand(true);

    let header_content = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    header_content.set_margin_top(8);
    header_content.set_margin_bottom(8);
    header_content.set_margin_start(12);
    header_content.set_margin_end(12);

    header_content.append(&arrow);
    header_content.append(&title);

    let group_button = gtk::Button::new();
    group_button.set_child(Some(&header_content));
    group_button.set_has_frame(false);
    group_button.set_hexpand(true);

    let header_row = gtk::ListBoxRow::new();
    header_row.set_activatable(false);
    header_row.set_selectable(false);
    header_row.set_child(Some(&group_button));

    list.append(&header_row);

    let account_rows: Vec<gtk::ListBoxRow> = accounts.into_iter().map(build_account_row).collect();

    for row in &account_rows {
        list.append(row);
    }

    {
        let expanded = Rc::clone(&expanded);
        let arrow = arrow.clone();

        let account_rows: Vec<gtk::ListBoxRow> = account_rows.iter().cloned().collect();

        group_button.connect_clicked(move |_| {
            let new_state = !expanded.get();
            expanded.set(new_state);

            for row in &account_rows {
                row.set_visible(new_state);
            }

            if new_state {
                arrow.set_icon_name(Some("pan-down-symbolic"));
            } else {
                arrow.set_icon_name(Some("pan-end-symbolic"));
            }
        });
    }
}

fn build_account_row(account: &TwoFAccount) -> gtk::ListBoxRow {
    let icon = gtk::Image::from_icon_name("dialog-password-symbolic");

    icon.set_pixel_size(32);
    icon.set_size_request(40, 40);
    icon.set_halign(gtk::Align::Center);
    icon.set_valign(gtk::Align::Center);

    load_account_icon(&icon, account.icon.clone());

    let service = gtk::Label::new(Some(&account.service));

    service.set_xalign(0.0);
    service.add_css_class("heading");

    service.set_ellipsize(gtk::pango::EllipsizeMode::End);

    let account_name = gtk::Label::new(Some(&account.account));

    account_name.set_xalign(0.0);
    account_name.add_css_class("dim-label");

    account_name.set_ellipsize(gtk::pango::EllipsizeMode::End);

    let labels = gtk::Box::new(gtk::Orientation::Vertical, 2);

    labels.set_hexpand(true);
    labels.append(&service);
    labels.append(&account_name);

    let arrow = gtk::Image::from_icon_name("go-next-symbolic");

    let row_content = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    row_content.set_margin_top(10);
    row_content.set_margin_bottom(10);
    row_content.set_margin_start(12);
    row_content.set_margin_end(12);

    row_content.append(&icon);
    row_content.append(&labels);
    row_content.append(&arrow);

    let row = gtk::ListBoxRow::new();

    row.set_widget_name(&account.id.to_string());

    row.set_activatable(true);
    row.set_child(Some(&row_content));

    row
}

fn show_otp_dialog(parent: &adw::ApplicationWindow, account: TwoFAccount) {
    let otp_type = account.otp_type.to_lowercase();

    let is_totp = otp_type == "totp";

    let digits = account.digits.max(1);

    let period = account.period.unwrap_or(30).max(1);

    let dialog = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(&account.service)
        .default_width(360)
        .default_height(360)
        .build();

    let header = adw::HeaderBar::new();

    let service = gtk::Label::new(Some(&account.service));

    service.add_css_class("title");

    header.set_title_widget(Some(&service));

    let account_icon = gtk::Image::from_icon_name("dialog-password-symbolic");

    account_icon.set_pixel_size(48);
    account_icon.set_size_request(56, 56);

    load_account_icon(&account_icon, account.icon.clone());

    let account_name = gtk::Label::new(Some(&account.account));

    account_name.add_css_class("dim-label");

    let otp_label = gtk::Label::new(Some(&tr("Loading code...")));

    otp_label.add_css_class("title-1");
    otp_label.set_selectable(true);

    let progress = gtk::ProgressBar::new();

    progress.set_fraction(0.0);
    progress.set_hexpand(true);
    progress.set_visible(is_totp);

    let countdown = gtk::Label::new(None);

    countdown.add_css_class("dim-label");
    countdown.set_visible(is_totp);

    let counter_label = gtk::Label::new(None);

    counter_label.add_css_class("dim-label");
    counter_label.set_visible(!is_totp);

    if !is_totp {
        if let Some(counter) = account.counter {
            counter_label.set_text(&format!("{}: {}", tr("Counter"), counter));
        } else {
            counter_label.set_text("HOTP");
        }
    }

    let copy_button = gtk::Button::with_label(&tr("Copy"));

    copy_button.set_sensitive(false);
    copy_button.add_css_class("suggested-action");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 16);

    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);

    content.append(&account_icon);
    content.append(&account_name);
    content.append(&otp_label);
    content.append(&progress);
    content.append(&countdown);
    content.append(&counter_label);
    content.append(&copy_button);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);

    root.append(&header);
    root.append(&content);

    dialog.set_content(Some(&root));

    let raw_code = Rc::new(RefCell::new(String::new()));

    let sync_state: Rc<RefCell<Option<OtpSyncState>>> = Rc::new(RefCell::new(None));

    let request_in_progress = Rc::new(Cell::new(false));

    {
        let raw_code = Rc::clone(&raw_code);

        copy_button.connect_clicked(move |_| {
            let code = raw_code.borrow();

            if !code.is_empty() {
                if let Some(display) = gtk::gdk::Display::default() {
                    display.clipboard().set_text(&code);
                }
            }
        });
    }

    request_otp(
        account.id,
        digits,
        period,
        &otp_label,
        &copy_button,
        &raw_code,
        &sync_state,
        &request_in_progress,
    );

    if is_totp {
        let dialog = dialog.clone();
        let otp_label = otp_label.clone();
        let copy_button = copy_button.clone();
        let progress = progress.clone();
        let countdown = countdown.clone();

        let raw_code = Rc::clone(&raw_code);

        let sync_state = Rc::clone(&sync_state);

        let request_in_progress = Rc::clone(&request_in_progress);

        let account_id = account.id;

        glib::timeout_add_local(Duration::from_millis(250), move || {
            if !dialog.is_visible() {
                return glib::ControlFlow::Break;
            }

            let mut should_refresh = false;

            if let Some(state) = sync_state.borrow().as_ref() {
                let elapsed = state.received_at.elapsed();

                let current_server_time = state
                    .server_time
                    .checked_add(elapsed)
                    .unwrap_or(state.server_time);

                if let Ok(since_epoch) = current_server_time.duration_since(UNIX_EPOCH) {
                    let server_seconds = since_epoch.as_secs_f64();

                    let period_f64 = period as f64;

                    let position = server_seconds % period_f64;

                    let remaining = period_f64 - position;

                    let fraction = (remaining / period_f64).clamp(0.0, 1.0);

                    progress.set_fraction(fraction);

                    let seconds_left = remaining.ceil() as u64;

                    countdown.set_text(&format!(
                        "{} {} {}",
                        tr("Valid for"),
                        seconds_left,
                        tr("seconds")
                    ));

                    let current_period_index = (server_seconds / period_f64).floor() as u64;

                    if current_period_index > state.period_index {
                        should_refresh = true;
                    }
                }
            }

            if should_refresh && !request_in_progress.get() {
                request_otp(
                    account_id,
                    digits,
                    period,
                    &otp_label,
                    &copy_button,
                    &raw_code,
                    &sync_state,
                    &request_in_progress,
                );
            }

            glib::ControlFlow::Continue
        });
    }

    dialog.present();
}

fn request_otp(
    account_id: u64,
    digits: u32,
    period: u32,
    otp_label: &gtk::Label,
    copy_button: &gtk::Button,
    raw_code: &Rc<RefCell<String>>,
    sync_state: &Rc<RefCell<Option<OtpSyncState>>>,
    request_in_progress: &Rc<Cell<bool>>,
) {
    if request_in_progress.get() {
        return;
    }

    request_in_progress.set(true);

    let (sender, receiver) = mpsc::channel::<OtpResult>();

    std::thread::spawn(move || {
        let config = match load_config() {
            Ok(config) => config,

            Err(error) => {
                let _ = sender.send(OtpResult::Error(error.to_string()));

                return;
            }
        };

        let token = match load_token() {
            Ok(token) => token,

            Err(error) => {
                let _ = sender.send(OtpResult::Error(error.to_string()));

                return;
            }
        };

        let server_url = config.server_url;

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,

            Err(error) => {
                let _ = sender.send(OtpResult::Error(error.to_string()));

                return;
            }
        };

        let result = runtime.block_on(async {
            let client = TwoFAuthClient::new(server_url, token);

            client.get_otp(account_id).await
        });

        match result {
            Ok(response) => {
                let _ = sender.send(OtpResult::Success(response));
            }

            Err(error) => {
                let _ = sender.send(OtpResult::Error(error.to_string()));
            }
        }
    });

    let otp_label = otp_label.clone();

    let copy_button = copy_button.clone();

    let raw_code = Rc::clone(raw_code);

    let sync_state = Rc::clone(sync_state);

    let request_in_progress = Rc::clone(request_in_progress);

    glib::timeout_add_local(Duration::from_millis(100), move || {
        match receiver.try_recv() {
            Ok(OtpResult::Success(response)) => {
                let formatted = format_otp(&response.password, digits);

                *raw_code.borrow_mut() = response.password;

                otp_label.set_text(&formatted);

                copy_button.set_sensitive(true);

                let server_time = response.server_time.unwrap_or_else(SystemTime::now);

                let period_index = server_time
                    .duration_since(UNIX_EPOCH)
                    .map(|duration| duration.as_secs() / period as u64)
                    .unwrap_or(0);

                *sync_state.borrow_mut() = Some(OtpSyncState {
                    server_time,
                    received_at: Instant::now(),
                    period_index,
                });

                request_in_progress.set(false);

                glib::ControlFlow::Break
            }

            Ok(OtpResult::Error(error)) => {
                otp_label.set_text(&format!("{}\n{}", tr("Could not load code"), error));

                copy_button.set_sensitive(false);

                request_in_progress.set(false);

                glib::ControlFlow::Break
            }

            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,

            Err(mpsc::TryRecvError::Disconnected) => {
                otp_label.set_text(&tr("Could not load code"));

                copy_button.set_sensitive(false);

                request_in_progress.set(false);

                glib::ControlFlow::Break
            }
        }
    });
}

fn format_otp(code: &str, digits: u32) -> String {
    if !code.chars().all(|character| character.is_ascii_digit()) {
        return code.to_string();
    }

    if code.len() != digits as usize {
        return code.to_string();
    }

    match code.len() {
        6 => {
            format!("{} {}", &code[..3], &code[3..])
        }

        8 => {
            format!("{} {}", &code[..4], &code[4..])
        }

        length if length > 4 => {
            let split = length / 2;

            format!("{} {}", &code[..split], &code[split..])
        }

        _ => code.to_string(),
    }
}
