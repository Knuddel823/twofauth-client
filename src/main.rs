mod api;
mod models;

use anyhow::{Context, Result};
use api::client::TwoFAuthClient;

#[tokio::main]
async fn main() -> Result<()> {
    println!("TwoFAuth Client v{}", env!("CARGO_PKG_VERSION"));

    let server_url =
        std::env::var("TWOFAUTH_URL").context("TWOFAUTH_URL environment variable is not set")?;

    let token = std::env::var("TWOFAUTH_TOKEN")
        .context("TWOFAUTH_TOKEN environment variable is not set")?;

    let client = TwoFAuthClient::new(server_url, token);

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        let account_id: u64 = args[1].parse().context("Account ID must be a number")?;

        let otp = client.get_otp(account_id).await?;

        println!("OTP: {}", otp.password);

        return Ok(());
    }

    println!("Connecting to 2FAuth...");

    let accounts = client.get_accounts().await?;

    println!("Found {} accounts:", accounts.len());

    for account in accounts {
        println!(
            "  {:>3}  {:<25} {}",
            account.id, account.service, account.account
        );
    }

    println!();
    println!("Run with an account ID to request an OTP.");
    println!("Example: cargo run -- 14");

    Ok(())
}
