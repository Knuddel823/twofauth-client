use std::error::Error;
use std::fmt;
use std::time::SystemTime;

use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::api::http_client::http_client;
use crate::models::account::TwoFAccount;
use crate::models::group::TwoFGroup;
use crate::storage::config::{ServerUrlError, validate_server_url};

#[derive(Debug)]
pub enum ApiError {
    InvalidServerUrl(ServerUrlError),
    ConnectionFailed(reqwest::Error),
    AuthenticationFailed,
    UnexpectedStatus(StatusCode),
    InvalidResponse(reqwest::Error),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidServerUrl(error) => write!(f, "Invalid server URL: {error:?}"),
            Self::ConnectionFailed(_) => write!(f, "Failed to connect to the 2FAuth server"),
            Self::AuthenticationFailed => {
                write!(f, "Authentication failed: invalid or expired token")
            }
            Self::UnexpectedStatus(status) => {
                write!(f, "2FAuth server returned HTTP {status}")
            }
            Self::InvalidResponse(_) => {
                write!(f, "Failed to process the response from the 2FAuth server")
            }
        }
    }
}

impl Error for ApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ConnectionFailed(error) | Self::InvalidResponse(error) => Some(error),
            Self::InvalidServerUrl(_) | Self::AuthenticationFailed | Self::UnexpectedStatus(_) => {
                None
            }
        }
    }
}

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
    pub fn new(
        base_url: impl Into<String>,
        token: impl Into<String>,
        allow_insecure_http: bool,
    ) -> Result<Self, ApiError> {
        let base_url = base_url.into().trim_end_matches('/').to_string();

        validate_server_url(&base_url, allow_insecure_http).map_err(ApiError::InvalidServerUrl)?;

        Ok(Self {
            base_url,
            token: token.into(),
            client: http_client().clone(),
        })
    }

    pub async fn get_accounts(&self) -> Result<Vec<TwoFAccount>, ApiError> {
        let url = format!("{}/api/v1/twofaccounts", self.base_url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(ApiError::ConnectionFailed)?;

        check_response_status(response.status())?;

        response
            .json::<Vec<TwoFAccount>>()
            .await
            .map_err(ApiError::InvalidResponse)
    }

    pub async fn get_groups(&self) -> Result<Vec<TwoFGroup>, ApiError> {
        let url = format!("{}/api/v1/groups", self.base_url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(ApiError::ConnectionFailed)?;

        check_response_status(response.status())?;

        response
            .json::<Vec<TwoFGroup>>()
            .await
            .map_err(ApiError::InvalidResponse)
    }

    pub async fn get_otp(&self, account_id: u64) -> Result<OtpResponse, ApiError> {
        let url = format!("{}/api/v1/twofaccounts/{}/otp", self.base_url, account_id);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(ApiError::ConnectionFailed)?;

        check_response_status(response.status())?;

        let server_time = response
            .headers()
            .get(reqwest::header::DATE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| httpdate::parse_http_date(value).ok());

        let mut otp = response
            .json::<OtpResponse>()
            .await
            .map_err(ApiError::InvalidResponse)?;

        otp.server_time = server_time;

        Ok(otp)
    }

    pub async fn get_icon(&self, icon: &str) -> Result<Vec<u8>, ApiError> {
        let url = format!("{}/storage/icons/{}", self.base_url, icon);

        let response = self
            .client
            .get(&url)
            .header("Accept", "image/*")
            .send()
            .await
            .map_err(ApiError::ConnectionFailed)?;

        check_response_status(response.status())?;

        let bytes = response.bytes().await.map_err(ApiError::InvalidResponse)?;

        Ok(bytes.to_vec())
    }
}

fn check_response_status(status: StatusCode) -> Result<(), ApiError> {
    if status == StatusCode::UNAUTHORIZED {
        return Err(ApiError::AuthenticationFailed);
    }

    if !status.is_success() {
        return Err(ApiError::UnexpectedStatus(status));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_status_is_accepted() {
        assert!(check_response_status(StatusCode::OK).is_ok());
    }

    #[test]
    fn unauthorized_status_is_authentication_error() {
        let result = check_response_status(StatusCode::UNAUTHORIZED);

        assert!(matches!(result, Err(ApiError::AuthenticationFailed)));
    }

    #[test]
    fn server_error_is_unexpected_status() {
        let result = check_response_status(StatusCode::INTERNAL_SERVER_ERROR);

        assert!(matches!(
            result,
            Err(ApiError::UnexpectedStatus(
                StatusCode::INTERNAL_SERVER_ERROR
            ))
        ));
    }

    #[test]
    fn not_found_is_unexpected_status() {
        let result = check_response_status(StatusCode::NOT_FOUND);

        assert!(matches!(
            result,
            Err(ApiError::UnexpectedStatus(StatusCode::NOT_FOUND))
        ));
    }
}
