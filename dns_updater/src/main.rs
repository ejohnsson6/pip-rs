use std::net::IpAddr;

use clap::Parser;
use eyre::Context;
use log::info;
use pinger::IpPinger;

use crate::cloudflare::{CloudflareClient, RecordRequest, RecordType};

pub mod cli;
pub mod cloudflare;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    let args = cli::Args::parse();

    println!("ARGS: {args:?}");

    let mut pinger = IpPinger::builder();
    for remote in args.remotes.iter() {
        pinger = pinger
            .with_remote(remote)
            .wrap_err_with(|| format!("invalid remote {remote}"))?;
    }

    let pinger = pinger.build().wrap_err("Failed to create IP pinger")?;

    let ip = match pinger.ping().await {
        Ok(ip) => ip,
        Err(errors) => {
            let error_messages = errors
                .into_iter()
                .map(|(remote, error)| format!("\t{remote} :: {error}"))
                .collect::<Vec<_>>();
            eyre::bail!(
                "Failed to retrieve IP, errors: \n{}",
                error_messages.join("\n")
            );
        }
    };

    let cloudflare_client = CloudflareClient::new(args.cloudflare_auth_key, args.mock);

    for zone_id in args.zone_ids.iter() {
        let list_response = cloudflare_client
            .list_dns_records(zone_id.as_str())
            .await
            .wrap_err_with(|| format!("Failed to retrieve dns records for zone: {zone_id}"))?;

        let filtered = match ip {
            IpAddr::V4(_) => list_response
                .into_iter()
                .filter(|record| record.record_type == RecordType::A)
                .collect::<Vec<_>>(),
            IpAddr::V6(_) => list_response
                .into_iter()
                .filter(|record| record.record_type == RecordType::AAAA)
                .collect::<Vec<_>>(),
        };

        info!("LIST RESPONSE (IP: {ip:?}): {filtered:#?}");

        let new_content = Some(ip.to_string());

        for record in filtered.into_iter() {
            let id = record.id.clone();
            if record.content.eq(&new_content) {
                info!(
                    "Record {} is already up to date, skipping update",
                    record.name
                );
                continue;
            }

            cloudflare_client
                .overwrite_dns_record(
                    zone_id,
                    &id,
                    RecordRequest::response_with_content(record.clone(), new_content.clone()),
                )
                .await
                .wrap_err("Failed to send overwriting cloudflare record")?;
            println!(
                "Updated record in zone {zone_id} / record {id} from {:?} -> {new_content:?}",
                &record.content
            );
        }
    }

    Ok(())
}
