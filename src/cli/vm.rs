use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2 as ec2;
use aws_types::region::Region;
use ec2::{config::Builder, types::Instance};
use std::process::Stdio;

use anyhow::{bail, Context, Ok, Result};
use clap::{Args, ValueEnum};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    join,
    process::Command,
};
use vm_rs::{
    aws::{
        ec2::{get_instance, get_instances},
        mfa_config,
    },
    ssh::update_ssh_config,
};

#[derive(Args)]
pub struct VmArgs {
    /// Command to run
    #[arg(value_enum, default_value_t = Cmd::Print)]
    cmd: Cmd,

    /// VM id to operate with
    #[arg(long, short = 'i')]
    id: Option<String>,

    /// EC2 region for `choose` command
    #[arg(long, short = 'r')]
    region: Option<String>,
}

impl VmArgs {
    pub async fn main(self) -> Result<()> {
        let region = match self.region {
            Some(s) => Some(Region::new(s)),
            None => RegionProviderChain::default_provider().region().await,
        };

        let sdk_config = mfa_config().await;
        let config = Builder::from(&sdk_config).region(region.clone()).build();
        let client = ec2::Client::from_conf(config);

        let mut instance = None;
        let id = match self.id {
            Some(id) => id,
            None => {
                if region.is_none() {
                    bail!("no default region set and none specified.  Use `--region` to explicitly pass a region");
                }
                let inst = choose_vm(&client).await?;
                let instance_id = inst.instance_id().expect("instance has id").to_string();
                instance = Some(inst);
                instance_id
            }
        };

        match self.cmd {
            Cmd::Start => {
                let _res = client
                    .start_instances()
                    .set_instance_ids(Some(vec![id.clone()]))
                    .send()
                    .await?;
            }
            Cmd::Stop => {
                let _res = client
                    .stop_instances()
                    .set_instance_ids(Some(vec![id.clone()]))
                    .send()
                    .await?;
            }
            Cmd::SetupSsh => {
                let instance = match instance {
                    Some(i) => i,
                    None => get_instance(&client, &id).await?,
                };
                let ip = instance
                    .public_ip_address()
                    .with_context(|| format!("no public ip address found for {id}"))?;
                update_ssh_config("vmrs", ip)?;
            }
            Cmd::Describe => {
                let instance = match instance {
                    Some(i) => i,
                    None => get_instance(&client, &id).await?,
                };
                println!("{:#?}", instance);
            }
            Cmd::Print => {}
        }
        println!("{id}");
        Ok(())
    }
}

async fn choose_vm(client: &ec2::Client) -> Result<Instance> {
    let instances = get_instances(client).await?;
    let choices = instances.iter().map(|ins| {
        let name = ins
            .tags()
            .into_iter()
            .flat_map(|tags| tags.iter())
            .find(|t| t.key() == Some("Name"))
            .map(|t| t.value())
            .flatten()
            .unwrap_or("no-name");

        let id = ins.instance_id().expect("instance_id not-null");
        let state = ins
            .state()
            .expect("instance.state not null")
            .name()
            .expect("instance state has a name");

        format!("{name} {id} [{state}]", state = state.as_str())
    });
    let vm_idx = fzf_choose(choices).await?;
    Ok(instances[vm_idx].to_owned())
}

async fn fzf_choose(choices: impl IntoIterator<Item = String>) -> Result<usize> {
    let mut cmd = Command::new("fzf")
        .args(["--print0", "--read0", "--with-nth=2.."])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut child_in = cmd.stdin.take().expect("child must have stdin");
    let writer = async move {
        for (i, choice) in choices.into_iter().enumerate() {
            child_in
                .write_all(
                    format!("{sep}{i} {choice}", sep = if i > 0 { "\0" } else { "" }).as_bytes(),
                )
                .await?;
        }
        drop(child_in);
        Ok(())
    };

    let mut child_out = cmd.stdout.take().expect("child must have stdout");
    let mut output = String::new();
    let (_, _) = join!(writer, child_out.read_to_string(&mut output));

    let first = output.split("\0").nth(0).context("nothing selected!")?;
    let idx = first
        .split(" ")
        .nth(0)
        .expect("fzf output parseable")
        .parse()
        .expect("fzf idx parseable");
    Ok(idx)
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Cmd {
    /// Start a VM
    Start,
    /// Stop a VM
    Stop,
    /// Setup SSH configuration
    SetupSsh,
    /// Print VM id (used for retreiving choice when no id is specified)
    Print,
    /// Describe VM
    Describe,
}
