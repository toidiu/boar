use crate::{AggregateStats, ExecutionPlan, Stats};
use std::{
    fs::{File, create_dir_all},
    io::Write,
};

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct Report {
    pub aggregate: AggregateStats,
    pub plan: ExecutionPlan,
    pub cdf_plot: Option<String>,
}

impl Report {
    pub fn new(plan: &ExecutionPlan, mut stats: Stats) -> Self {
        let dir = Self::create_report_dir(plan);

        let data_file = format!("{}/data.txt", &dir);
        let mut data_file = File::create(data_file).unwrap();
        write!(&mut data_file, "{:#?}", stats).unwrap();

        let cdf_plot = stats.plot_cdf(&dir, &plan);

        let report = Report {
            aggregate: stats.aggregate(),
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
