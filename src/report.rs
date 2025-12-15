use crate::{ExecutionPlan, Stats, stats::AggregateStats};
use std::{
    fs::{File, create_dir_all},
    io::Write,
};

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct Report {
    pub plan: ExecutionPlan,
    stat_report: Vec<StatsReport>,
}

#[allow(dead_code)]
#[derive(Debug, Default)]
struct StatsReport {
    pub aggregate: AggregateStats,
    pub cdf_path: String,
}

impl Report {
    // TODO make Vec<Stats>
    // pub fn new(plan: &ExecutionPlan, mut stats: Vec<Stats>) -> Self {
    pub fn new(plan: &ExecutionPlan, stats: Vec<Stats>) -> Self {
        let dir = Self::create_report_dir(plan);
        let mut report = Report {
            plan: plan.clone(),
            stat_report: vec![],
        };

        for mut stat in stats {
            let data_file = format!("{}/data_{}.txt", &dir, stat.name());
            let mut data_file = File::create(data_file).unwrap();
            write!(&mut data_file, "{:#?}", stat).unwrap();

            let cdf_path = stat.plot_cdf(&dir);
            let aggregate = stat.aggregate();
            let stat_report = StatsReport {
                aggregate,
                cdf_path,
            };
            report.stat_report.push(stat_report);
        }

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
