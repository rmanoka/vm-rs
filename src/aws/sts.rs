use anyhow::Result;
use aws_config::SdkConfig;
use aws_sdk_sts::Config;
use chrono::{DateTime, Local};
use home::home_dir;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::aws::update_config;

use super::read_config_section;

pub struct STSConfig {
    sdk_config: SdkConfig,
    aws_creds_path: PathBuf,
}

impl STSConfig {
    pub fn new(sdk_config: SdkConfig) -> Self {
        let cfg_path = PathBuf::from(home_dir().expect("home directory must be set"))
            .join(Path::new(".aws/credentials"));
        Self {
            sdk_config,
            aws_creds_path: cfg_path,
        }
    }

    pub async fn from_env() -> Self {
        // We can't use mfa config here
        let config = aws_config::load_from_env().await;
        STSConfig::new(config)
    }

    pub async fn check_session_expiry(&self) -> Result<Option<bool>> {
        let section = read_config_section(&self.aws_creds_path, "mfa".to_string())?;

        Ok(section
            .and_then(|sec| sec.get("expiration").cloned())
            .map(|exp| DateTime::parse_from_rfc3339(exp.as_str()))
            .transpose()?
            .map(|dt| dt > Local::now()))
    }

    pub async fn create_session_config(&self, mfa_serial: &str, token_code: &str) -> Result<()> {
        let client = aws_sdk_sts::Client::from_conf(Config::new(&self.sdk_config));
        let token = client
            .get_session_token()
            .set_serial_number(Some(mfa_serial.to_string()))
            .set_token_code(Some(token_code.to_string()))
            .send()
            .await?;

        let cred = token.credentials().expect("credentials in token");
        let expiry: SystemTime = cred
            .expiration()
            .expect("expiration in token")
            .clone()
            .try_into()
            .expect("expiration covertible to system time");

        use chrono::prelude::*;
        let expiry: DateTime<Local> = expiry.into();

        let key_id = cred.access_key_id().expect("access_key_id in token");
        let key = cred.secret_access_key().expect("access_key in token");
        let sess = cred.session_token().expect("session_token in token");

        let values = HashMap::from_iter([
            ("aws_access_key_id".to_string(), key_id.to_string()),
            ("aws_secret_access_key".to_string(), key.to_string()),
            ("aws_session_token".to_string(), sess.to_string()),
            ("expiration".to_string(), expiry.to_rfc3339()),
        ]);
        update_config(&self.aws_creds_path, "mfa".to_string(), values)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session() {
        let sts_confg = STSConfig::from_env().await;
        sts_confg
            .create_session_config("arn:aws:iam::261392756586:mfa/rajsekar_mobile", "634563")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_session_expiry() {
        let sts_confg = STSConfig::from_env().await;
        let is_valid = sts_confg.check_session_expiry().await.unwrap().unwrap();
        eprintln!("{is_valid}")
    }
}
