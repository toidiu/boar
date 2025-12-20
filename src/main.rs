use crate::{
    endpoint::EndpointSetup,
    error::Result,
    network::NetworkSetup,
    stats::{
        Stats, delivery_rate::DeliveryRate, download_duration::DownloadDuration,
        startup_exit::StartupExit,
    },
};
use byte_unit::Byte;
use std::fmt::Debug;
use uuid::Uuid;

mod args;
mod endpoint;
mod error;
mod network;
mod report;
mod stats;

#[derive(Debug, Clone)]
struct ExecutionPlan {
    uuid: Uuid,
    network: NetworkSetup,
    endpoint: EndpointSetup,
    download_bytes: Byte,
    run_count: u16,
}

fn main() -> Result<()> {
    // Cli
    let plan = args::parse();
    // dbg!(&setup, &plan);

    // println!("Executing: {:#?}", &plan);

    // Network
    plan.network.cleanup()?;
    plan.network.create()?;

    // Run
    let (mut server, server_logs) = plan.endpoint.run_server();

    let mut download_duration = Vec::new();
    let mut delivery_rate = Vec::new();
    for i in 1..=plan.run_count {
        let client_logs = plan.endpoint.run_client(&plan.download_bytes);
        let metric_download_duration = DownloadDuration::new_from_logs(&client_logs);
        println!(
            "Run [{}/{}]: Download duration: {:?}",
            i, plan.run_count, metric_download_duration
        );
        download_duration.push(Box::new(metric_download_duration));
        delivery_rate.push(Box::new(DeliveryRate::new_from_logs(&client_logs)));
    }

    let server_logs = server_logs.lock().unwrap().clone();
    let startup_exit = StartupExit::new_from_logs(&server_logs);

    let download_duration =
        Stats::new::<DownloadDuration>(download_duration.into_iter().map(|ty| ty as _).collect());
    let deliver_rate =
        Stats::new::<DeliveryRate>(delivery_rate.into_iter().map(|ty| ty as _).collect());
    let startup_exit = Stats::new::<StartupExit>(
        startup_exit
            .into_iter()
            .map(|ty| Box::new(ty) as _)
            .collect(),
    );

    server.kill().unwrap();

    // Report
    let report = report::Report::new(&plan, vec![download_duration, deliver_rate, startup_exit]);

    println!("{:#?}", report);

    Ok(())
}
