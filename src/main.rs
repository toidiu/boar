use boar::{self, DeliveryRate, DownloadDuration, Report, StartupExit, Stats};

mod args;

fn main() -> boar::Result<()> {
    // Cli
    let plan = args::parse();
    // dbg!(&setup, &plan);

    println!("Executing: {:#?}\n-------------", &plan);

    // Network
    plan.network_setup.cleanup()?;
    plan.network_setup.create()?;

    // Run. In docker mode the server lives in a peer container and boar does
    // not own its lifecycle, so run_server returns None.
    let server_handle = plan.endpoint_setup.run_server();

    let mut download_duration = Vec::new();
    let mut delivery_rate = Vec::new();
    for i in 1..=plan.count {
        let client_logs = plan.endpoint_setup.run_client(&plan.download_bytes);
        let metric_download_duration = DownloadDuration::new_from_logs(&client_logs);
        let metric_delivery_rate = DeliveryRate::new_from_logs(&client_logs);

        println!(
            "Run [{}/{}]: Download duration: {:?}. DeliveryRate {:?}",
            i, plan.count, metric_download_duration, metric_delivery_rate
        );

        // If either parser couldn't find its pattern the client probably never
        // completed a download. Dump its stderr so the failure is debuggable
        // rather than silently producing an empty report.
        if metric_download_duration.is_err() || metric_delivery_rate.is_err() {
            eprintln!(
                "--- client_logs (run {}/{}) ---\n{}\n--- end client_logs ---",
                i, plan.count, client_logs
            );
        }

        if let Ok(metric_download_duration) = metric_download_duration {
            download_duration.push(Box::new(metric_download_duration));
        }
        if let Ok(metric_delivery_rate) = metric_delivery_rate {
            delivery_rate.push(Box::new(metric_delivery_rate));
        }
    }

    let server_logs = match &server_handle {
        Some((_, logs)) => logs.lock().unwrap().clone(),
        // Docker mode: server stderr isn't in-process, so we feed StartupExit
        // an empty log set. TODO: wire up a shared log volume to recover it.
        None => Vec::new(),
    };
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

    if let Some((mut server, _)) = server_handle {
        server.kill().unwrap();
    }

    // Report
    let report = Report::new(&plan, vec![download_duration, deliver_rate, startup_exit]);

    println!("{:#?}", report);

    Ok(())
}
