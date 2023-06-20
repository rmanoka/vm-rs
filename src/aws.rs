use anyhow::Result;
use aws_config::SdkConfig;
use ini::Ini;
use std::{collections::HashMap, path::Path};

pub mod ec2;
pub mod sts;

pub async fn mfa_config() -> SdkConfig {
    aws_config::from_env().profile_name("mfa").load().await
}

fn read_config_section(
    path: &Path,
    section_key: String,
) -> Result<Option<HashMap<String, String>>> {
    let cfg = Ini::load_from_file(path)?;
    Ok(cfg.section(Some(section_key)).map(|s| {
        HashMap::from_iter(
            s.iter()
                .map(|(key, val)| (key.to_string(), val.to_string())),
        )
    }))
}

fn update_config(path: &Path, section_key: String, values: HashMap<String, String>) -> Result<()> {
    let mut cfg = Ini::load_from_file(path)?;
    for (key, val) in values.into_iter() {
        cfg.set_to(Some(&section_key), key, val)
    }
    cfg.write_to_file(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_mfa_update() {
        let mut values = HashMap::new();
        values.insert("foo".to_string(), "bar".to_string());
        values.insert("faz".to_string(), "craz".to_string());
        update_config(Path::new("fixtures/test.ini"), "mfa".to_string(), values).unwrap();
    }
}
