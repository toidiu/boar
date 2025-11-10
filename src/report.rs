use crate::ExecutionPlan;
use crate::StatsReport;
use crate::stats;
use crate::stats::ToStats;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::Write;

#[derive(Debug)]
pub(crate) struct Report {
    pub stats_report: StatsReport,
    pub plan: ExecutionPlan,
    pub cdf_plot: Option<String>,
}

impl Report {
    pub fn new(plan: &ExecutionPlan, data: Vec<impl ToStats + std::fmt::Debug>) -> Self {
        let dir = Self::create_report_dir(plan);

        let data_file = format!("{}/data.txt", &dir);
        let mut data_file = File::create(data_file).unwrap();
        write!(&mut data_file, "{:#?}", data).unwrap();

        let data: Vec<f64> = data.iter().map(|d| d.as_f64()).collect();
        let mut data = statrs::statistics::Data::new(data);

        let cdf_plot = stats::plot_cdf(&dir, &data, &plan);

        let stats_report = StatsReport::new(&mut data);
        let report = Report {
            stats_report,
            plan: plan.clone(),
            cdf_plot: Some(cdf_plot),
        };

        let report_file = format!("{}/report.txt", &dir);
        let mut report_file = File::create(report_file).unwrap();
        write!(&mut report_file, "{:#?}", report).unwrap();

        report
    }

    fn create_report_dir(plan: &ExecutionPlan) -> String {
        let dir = format!("report/{}", plan.uuid);
        // make "/report"
        // make folder for this report based on uuid
        create_dir_all(&dir).unwrap();

        // print Report to file
        dir
    }
}
