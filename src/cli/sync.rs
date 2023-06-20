use anyhow::{bail, Result};
use clap::Args;
use shlex::join;
use std::{env::current_dir, process::Command};
use vm_rs::sync::find_corresponding_path;

#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Only print corresponding path and exit
    #[arg(long, short = 'p', group = "exec_type")]
    path: bool,

    /// Only print command and exit
    #[arg(long, group = "exec_type")]
    print: bool,

    /// Only list files in corresponding path and exit
    #[arg(long, conflicts_with = "path")]
    list: bool,

    /// Sync with s3 via `.s3-prefix` file (default: to vm with `.vm-prefix`)
    #[arg(long, short = '3')]
    s3: bool,

    /// Sync only git files
    #[arg(long, short = 'g', group = "feed", conflicts_with = "s3")]
    git: bool,

    /// Sync from stdin
    #[arg(long, group = "feed", conflicts_with = "s3")]
    pipe: Option<String>,

    /// Sync from source (default: to source)
    #[arg(long, short = 'f')]
    from: bool,

    /// Host to connect to for host-based sync (rsync / ssh)
    #[arg(long, default_value = "vmrs-script", conflicts_with = "s3")]
    host: String,

    /// Arguments to pass to inner program (rsync / ssh / aws s3)
    #[arg(trailing_var_arg = true)]
    prog_args: Vec<String>,
}

impl SyncArgs {
    pub async fn main(self) -> Result<()> {
        let path = current_dir()?;
        let filename = if self.s3 { ".s3-prefix" } else { ".vm-prefix" };
        let corr_path = find_corresponding_path(&path, filename)?
            .to_string_lossy()
            .to_string();

        if self.path {
            println!("{corr_path}");
            return Ok(());
        }

        let mut cmd: Vec<String> = vec![];
        let remote = if self.s3 {
            cmd.extend(
                ["aws", "--profile", "mfa", "s3"]
                    .into_iter()
                    .map(ToString::to_string),
            );
            cmd.push(if self.list { "ls" } else { "sync" }.to_string());

            corr_path
        } else {
            if !self.list {
                cmd.extend(["rsync", "-azsc"].into_iter().map(String::from));
                format!("{host}:{prefix}", host = self.host, prefix = corr_path)
            } else {
                cmd.extend(
                    ["ssh", &self.host, "ls", "-l"]
                        .into_iter()
                        .map(String::from),
                );
                corr_path
            }
        };
        if let Some(from) = self.pipe {
            cmd.push(format!("--files-from={from}"));
        }
        cmd.extend(self.prog_args);

        if self.list {
            cmd.push(remote);
        } else {
            cmd.extend(if self.from {
                [remote, "./".to_string()]
            } else {
                ["./".to_string(), remote]
            })
        };

        let cmd = join(cmd.iter().map(|s| s.as_str()));
        let cmd = if self.git {
            format!("git ls-files | {cmd}")
        } else {
            cmd
        };

        if self.print {
            println!("{cmd}");
            return Ok(());
        }

        let status = Command::new("sh").arg("-c").arg(&cmd).status()?;
        if !status.success() {
            bail!("sync cmd: {cmd} failed with code {status}");
        }

        Ok(())
    }
}
