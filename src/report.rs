use crate::ExecutionPlan;
use crate::StatsReport;
use crate::stats;

#[derive(Debug)]
pub(crate) struct Report {
    pub stats_report: StatsReport,
    pub plan: ExecutionPlan,
    pub cdf_plot: Option<String>,
}

impl Report {
    pub fn new(plan: &ExecutionPlan, data: Vec<f64>) -> Self {
        let mut data = statrs::statistics::Data::new(data);

        let cdf_plot = stats::plot_cdf(&data, &plan);
        let stats_report = StatsReport::new(&mut data);

        let report = Report {
            stats_report,
            plan: plan.clone(),
            cdf_plot: Some(cdf_plot),
        };

        report
    }
}
