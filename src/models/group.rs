use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TwoFGroup {
    pub id: u64,
    pub name: String,
    pub twofaccounts_count: u64,
}
