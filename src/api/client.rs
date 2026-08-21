use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::models::account::TwoFAccount;

#[derive(Debug, Deserialize)]
pub struct OtpResponse {
    pub password: String,
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

        response
            .json::<OtpResponse>()
            .await
            .context("Failed to parse the OTP response")
    }
}
