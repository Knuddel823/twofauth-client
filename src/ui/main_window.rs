use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use adw::prelude::*;
use gtk::glib;

use crate::api::client::{OtpResponse, TwoFAuthClient};
use crate::i18n::tr;
use crate::models::account::TwoFAccount;

enum LoadResult {
    Success(Vec<TwoFAccount>),
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

    {
        let accounts = Rc::clone(&accounts);
        let account_list = account_list.clone();

        search.connect_search_changed(move |search| {
            rebuild_account_list(&account_list, &accounts.borrow(), &search.text());
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
        let server_url = match std::env::var("TWOFAUTH_URL") {
            Ok(value) => value,
            Err(_) => {
                let _ = sender.send(LoadResult::Error("TWOFAUTH_URL is not set".to_string()));
                return;
            }
        };

        let token = match std::env::var("TWOFAUTH_TOKEN") {
            Ok(value) => value,
            Err(_) => {
                let _ = sender.send(LoadResult::Error("TWOFAUTH_TOKEN is not set".to_string()));
                return;
            }
        };

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                let _ = sender.send(LoadResult::Error(error.to_string()));
                return;
            }
        };

        let result = runtime.block_on(async {
            let client = TwoFAuthClient::new(server_url, token);
            client.get_accounts().await
        });

        match result {
            Ok(accounts) => {
                let _ = sender.send(LoadResult::Success(accounts));
            }
            Err(error) => {
                let _ = sender.send(LoadResult::Error(error.to_string()));
            }
        }
    });

    {
        let accounts = Rc::clone(&accounts);
        let account_list = account_list.clone();
        let status = status.clone();

        glib::timeout_add_local(Duration::from_millis(100), move || {
            match receiver.try_recv() {
                Ok(LoadResult::Success(loaded_accounts)) => {
                    *accounts.borrow_mut() = loaded_accounts;

                    status.set_visible(false);

                    rebuild_account_list(&account_list, &accounts.borrow(), "");

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

fn rebuild_account_list(list: &gtk::ListBox, accounts: &[TwoFAccount], search_text: &str) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    let search_text = search_text.trim().to_lowercase();

    let filtered: Vec<&TwoFAccount> = accounts
        .iter()
        .filter(|account| {
            search_text.is_empty()
                || account.service.to_lowercase().contains(&search_text)
                || account.account.to_lowercase().contains(&search_text)
        })
        .collect();

    if filtered.is_empty() {
        let empty = gtk::Label::new(Some(&tr("No accounts found")));
        empty.set_margin_top(24);
        empty.set_margin_bottom(24);
        list.append(&empty);
        return;
    }

    for account in filtered {
        let service = gtk::Label::new(Some(&account.service));
        service.set_xalign(0.0);
        service.add_css_class("heading");

        let account_name = gtk::Label::new(Some(&account.account));
        account_name.set_xalign(0.0);
        account_name.add_css_class("dim-label");

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
        row_content.append(&labels);
        row_content.append(&arrow);

        let row = gtk::ListBoxRow::new();
        row.set_widget_name(&account.id.to_string());
        row.set_activatable(true);
        row.set_child(Some(&row_content));

        list.append(&row);
    }
}

fn show_otp_dialog(parent: &adw::ApplicationWindow, account: TwoFAccount) {
    let period = account.period.unwrap_or(30).max(1);

    let dialog = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(&account.service)
        .default_width(360)
        .default_height(340)
        .build();

    let header = adw::HeaderBar::new();

    let service = gtk::Label::new(Some(&account.service));
    service.add_css_class("title");
    header.set_title_widget(Some(&service));

    let account_name = gtk::Label::new(Some(&account.account));
    account_name.add_css_class("dim-label");

    let otp_label = gtk::Label::new(Some(&tr("Loading code...")));
    otp_label.add_css_class("title-1");
    otp_label.set_selectable(true);

    let progress = gtk::ProgressBar::new();
    progress.set_fraction(0.0);
    progress.set_hexpand(true);

    let countdown = gtk::Label::new(None);
    countdown.add_css_class("dim-label");

    let copy_button = gtk::Button::with_label(&tr("Copy"));
    copy_button.set_sensitive(false);
    copy_button.add_css_class("suggested-action");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);

    content.append(&account_name);
    content.append(&otp_label);
    content.append(&progress);
    content.append(&countdown);
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
        period,
        &otp_label,
        &copy_button,
        &raw_code,
        &sync_state,
        &request_in_progress,
    );

    {
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
        let server_url = match std::env::var("TWOFAUTH_URL") {
            Ok(value) => value,
            Err(_) => {
                let _ = sender.send(OtpResult::Error("TWOFAUTH_URL is not set".to_string()));
                return;
            }
        };

        let token = match std::env::var("TWOFAUTH_TOKEN") {
            Ok(value) => value,
            Err(_) => {
                let _ = sender.send(OtpResult::Error("TWOFAUTH_TOKEN is not set".to_string()));
                return;
            }
        };

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
                let formatted = format_otp(&response.password);

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

fn format_otp(code: &str) -> String {
    if code.len() == 6 && code.chars().all(|character| character.is_ascii_digit()) {
        format!("{} {}", &code[..3], &code[3..])
    } else {
        code.to_string()
    }
}
