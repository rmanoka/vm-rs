use anyhow::Result;
use home::home_dir;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

pub fn update_ssh_config(host_prefix: &str, hostname: &str) -> Result<()> {
    lazy_static! {
        static ref HOST_LINE: Regex = Regex::new(r"Host\s+(\S+)").unwrap();
        static ref HOSTNAME_LINE: Regex = Regex::new(r"Hostname\s+(\S+)").unwrap();
    }

    let cfg_path = PathBuf::from(home_dir().expect("home directory must be set"))
        .join(Path::new(".ssh/config"));

    let file = BufReader::new(File::open(&cfg_path)?);

    let mut outfile = NamedTempFile::new_in(cfg_path.parent().unwrap())?;
    let mut writer = BufWriter::new(&mut outfile);
    let mut is_matching_host = false;
    for line in file.lines() {
        let line = line?;
        let line = if let Some(caps) = HOST_LINE.captures(&line) {
            let host = caps.get(1).unwrap().as_str();
            is_matching_host = host.starts_with(&host_prefix);
            line
        } else if is_matching_host {
            let line = HOSTNAME_LINE.replace(&line, format!("Hostname {hostname}"));
            line.to_string()
        } else {
            line
        };
        writeln!(&mut writer, "{line}")?;
    }
    drop(writer);
    outfile.persist(cfg_path)?;
    Ok(())
}

#[test]
#[ignore = "replace `.ssh/config`"]
fn test_ssh_config() {
    update_ssh_config("vmrs", "foobar").unwrap();
}
