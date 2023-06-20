use anyhow::{Context, Result};
use aws_sdk_ec2 as ec2;
use ec2::{types::Instance, Client};

pub async fn get_instances(client: &Client) -> Result<Vec<Instance>> {
    let instances = client.describe_instances().send().await?;
    let reservations = instances
        .reservations()
        .with_context(|| "no ec2 instance reservations found!")?;

    let instances: Vec<_> = reservations
        .iter()
        .flat_map(|f| f.instances().into_iter().flat_map(|res| res))
        .cloned()
        .collect();
    Ok(instances)
}

pub async fn get_instance(client: &Client, id: &str) -> Result<Instance> {
    let res = client
        .describe_instances()
        .set_instance_ids(Some(vec![id.to_string()]))
        .send()
        .await?;
    let reservation = &res
        .reservations()
        .with_context(|| format!("no ec2 instance reservations found for {id}"))?[0];
    let instance = &reservation
        .instances()
        .with_context(|| format!("no ec2 instances in reservation found for {id}"))?[0];
    Ok(instance.clone())
}
