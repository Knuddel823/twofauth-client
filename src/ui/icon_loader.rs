use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use adw::prelude::*;
use gdk_pixbuf::PixbufLoader;
use gtk::glib;

use crate::api::client::{ApiError, TwoFAuthClient};
use crate::storage::config::load_config;
use crate::storage::keyring::load_token;

const CACHE_DIR_NAME: &str = "twofauth-client";
const ICON_CACHE_DIR_NAME: &str = "icons";

enum IconResult {
    Success(Vec<u8>),
    Error,
}

pub fn load_account_icon(image: &gtk::Image, icon_name: Option<String>) {
    let Some(icon_name) = icon_name else {
        set_fallback_icon(image);
        return;
    };

    let icon_name = icon_name.trim().to_string();

    if icon_name.is_empty() {
        set_fallback_icon(image);
        return;
    }

    let (sender, receiver) = mpsc::channel::<IconResult>();

    std::thread::spawn(move || {
        let cache_path = match icon_cache_path(&icon_name) {
            Some(path) => path,
            None => {
                let _ = sender.send(IconResult::Error);
                return;
            }
        };

        if let Ok(bytes) = fs::read(&cache_path) {
            if !bytes.is_empty() {
                let _ = sender.send(IconResult::Success(bytes));
                return;
            }
        }

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
            let client = TwoFAuthClient::new(config.server_url, token, config.allow_insecure_http)
                .map_err(ApiError::InvalidServerUrl)?;

            client.get_icon(&icon_name).await
        });

        match result {
            Ok(bytes) => {
                if !bytes.is_empty() {
                    let _ = save_icon_to_cache(&cache_path, &bytes);
                    let _ = sender.send(IconResult::Success(bytes));
                } else {
                    let _ = sender.send(IconResult::Error);
                }
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

fn icon_cache_path(icon_name: &str) -> Option<PathBuf> {
    let cache_dir = dirs::cache_dir()?
        .join(CACHE_DIR_NAME)
        .join(ICON_CACHE_DIR_NAME);

    Some(cache_dir.join(safe_icon_filename(icon_name)))
}

fn safe_icon_filename(icon_name: &str) -> String {
    Path::new(icon_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("icon")
        .to_string()
}

fn save_icon_to_cache(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, bytes)
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
