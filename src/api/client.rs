use std::time::SystemTime;

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::models::account::TwoFAccount;

#[derive(Debug, Deserialize)]
pub struct OtpResponse {
    pub password: String,

    #[serde(skip)]
    pub server_time: Option<SystemTime>,
}

pub struct TwoFAuthClient {
    base_url: String,
    token: String,
    client: Client,
}

impl TwoFAuthClient {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token: token.into(),
            client: Client::new(),
        }
    }

    pub async fn get_accounts(&self) -> Result<Vec<TwoFAccount>> {
        let url = format!("{}/api/v1/twofaccounts", self.base_url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to connect to the 2FAuth server")?;

        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            anyhow::bail!("Authentication failed: invalid or expired token");
        }

        if !status.is_success() {
            anyhow::bail!("2FAuth server returned HTTP {}", status);
        }

        response
            .json::<Vec<TwoFAccount>>()
            .await
            .context("Failed to parse the 2FAuth account list")
    }

    pub async fn get_otp(&self, account_id: u64) -> Result<OtpResponse> {
        let url = format!("{}/api/v1/twofaccounts/{}/otp", self.base_url, account_id);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to request OTP from the 2FAuth server")?;

        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            anyhow::bail!("Authentication failed: invalid or expired token");
        }

        if !status.is_success() {
            anyhow::bail!("2FAuth server returned HTTP {}", status);
        }

        let server_time = response
            .headers()
            .get(reqwest::header::DATE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| httpdate::parse_http_date(value).ok());

        let mut otp = response
            .json::<OtpResponse>()
            .await
            .context("Failed to parse the OTP response")?;

        otp.server_time = server_time;

        Ok(otp)
    }

    pub async fn get_icon(&self, icon: &str) -> Result<Vec<u8>> {
        let url = format!("{}/storage/icons/{}", self.base_url, icon);

        let response = self
            .client
            .get(&url)
            .header("Accept", "image/*")
            .send()
            .await
            .context("Failed to download the 2FAuth account icon")?;

        let status = response.status();

        if !status.is_success() {
            anyhow::bail!("2FAuth server returned HTTP {} for icon {}", status, icon);
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read the 2FAuth account icon")?;

        Ok(bytes.to_vec())
    }
}
