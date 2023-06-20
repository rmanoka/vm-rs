use anyhow::{Result, bail};
use clap::Args;
use serde::{Deserialize, Serialize};
use vm_rs::aws::sts::STSConfig;

#[derive(Args)]
pub struct MfaArgs {
    /// Multi-factor OTP token
    token: Option<String>,

    /// Device serial arn (default: use from configuration)
    #[arg(long, short = 'd')]
    device: Option<String>,

    /// Check if current token session has expired
    #[arg(long, short = 'c')]
    check: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct Config {
    device: String,
}

impl MfaArgs {
    pub async fn main(self) -> Result<()> {
        let sts_confg = STSConfig::from_env().await;

        if self.check {
            let is_valid = sts_confg.check_session_expiry().await?.unwrap_or(false);
            if !is_valid {
                bail!("no session or has expired; pls use token to create new session");
            }
        } else {
            let config: Config = confy::load("vm-rs", Some("mfa"))?;
            let token = self.token.unwrap();
            sts_confg
                .create_session_config(&self.device.unwrap_or(config.device), &token)
                .await?;
        }

        Ok(())
    }
}
