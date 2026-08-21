use std::sync::mpsc;
use std::time::Duration;

use adw::prelude::*;
use gdk_pixbuf::PixbufLoader;
use gtk::glib;

use crate::api::client::TwoFAuthClient;
use crate::storage::config::load_config;
use crate::storage::keyring::load_token;

enum IconResult {
    Success(Vec<u8>),
    Error,
}

pub fn load_account_icon(image: &gtk::Image, icon_name: Option<String>) {
    let Some(icon_name) = icon_name else {
        set_fallback_icon(image);
        return;
    };

    if icon_name.trim().is_empty() {
        set_fallback_icon(image);
        return;
    }

    let (sender, receiver) = mpsc::channel::<IconResult>();

    std::thread::spawn(move || {
        let config = match load_config() {
            Ok(config) => config,
            Err(_) => {
                let _ = sender.send(IconResult::Error);
                return;
            }
        };

        let token = load_token().unwrap_or_default();

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(_) => {
                let _ = sender.send(IconResult::Error);
                return;
            }
        };

        let result = runtime.block_on(async {
            let client = TwoFAuthClient::new(config.server_url, token);
            client.get_icon(&icon_name).await
        });

        match result {
            Ok(bytes) => {
                let _ = sender.send(IconResult::Success(bytes));
            }

            Err(_) => {
                let _ = sender.send(IconResult::Error);
            }
        }
    });

    let image = image.clone();

    glib::timeout_add_local(Duration::from_millis(50), move || {
        match receiver.try_recv() {
            Ok(IconResult::Success(bytes)) => {
                if let Some(pixbuf) = decode_icon(&bytes) {
                    image.set_from_pixbuf(Some(&pixbuf));
                } else {
                    set_fallback_icon(&image);
                }

                glib::ControlFlow::Break
            }

            Ok(IconResult::Error) => {
                set_fallback_icon(&image);
                glib::ControlFlow::Break
            }

            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,

            Err(mpsc::TryRecvError::Disconnected) => {
                set_fallback_icon(&image);
                glib::ControlFlow::Break
            }
        }
    });
}

fn decode_icon(bytes: &[u8]) -> Option<gdk_pixbuf::Pixbuf> {
    let loader = PixbufLoader::new();

    if loader.write(bytes).is_err() {
        return None;
    }

    if loader.close().is_err() {
        return None;
    }

    let pixbuf = loader.pixbuf()?;

    pixbuf
        .scale_simple(32, 32, gdk_pixbuf::InterpType::Bilinear)
        .or(Some(pixbuf))
}

fn set_fallback_icon(image: &gtk::Image) {
    image.set_icon_name(Some("dialog-password-symbolic"));
    image.set_pixel_size(32);
}
