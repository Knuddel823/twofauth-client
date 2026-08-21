use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TwoFAccount {
    pub id: u64,
    pub service: String,
    pub account: String,
    pub otp_type: String,
    pub algorithm: String,
    pub digits: u32,
    pub period: Option<u32>,
    pub counter: Option<u64>,
    pub group_id: Option<u64>,
    pub icon: Option<String>,
}
