use crate::{
    endpoint::EndpointSetup,
    error::Result,
    network::NetworkSetup,
    stats::{Stats, delivery_rate::DeliveryRate, download_duration::DownloadDuration},
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
    let mut server = plan.endpoint.run_server();

    let mut metrics = Vec::new();
    let mut ametrics = Vec::new();
    for i in 1..=plan.run_count {
        let logs = plan.endpoint.run_client(&plan.download_bytes);
        let metric = DownloadDuration::new_from_logs(&logs);
        let ametric = DeliveryRate::new_from_logs(&logs);
        println!(
            "Run [{}/{}]: Download duration: {:?}",
            i, plan.run_count, metric
        );
        metrics.push(Box::new(metric));
        ametrics.push(Box::new(ametric));
    }

    let s = Stats::new(metrics.into_iter().map(|ty| ty as _).collect());
    let ast = Stats::new(ametrics.into_iter().map(|ty| ty as _).collect());

    server.kill().unwrap();

    // Report
    let report = report::Report::new(&plan, vec![s, ast]);

    println!("{:#?}", report);

    Ok(())
}
