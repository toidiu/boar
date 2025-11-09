use plotly::{Scatter, layout::GridPattern, layout::Layout, layout::LayoutGrid};
use statrs::statistics::{Data, Distribution, OrderStatistics};
use std::fmt::Debug;

pub trait ToStats {
    type Metric: Debug;

    fn as_f64(&self) -> f64;

    fn new_from_logs(logs: &str) -> Self;
}

pub(crate) fn plot_cdf(data: &Data<Vec<f64>>) -> String {
    let cdf_data = cdf(data);

    let mut plot = plotly::Plot::new();
    let (x, y): (Vec<_>, Vec<_>) = cdf_data.into_iter().map(|(a, b)| (a, b)).unzip();

    // Graph
    let trace = Scatter::new(x, y)
        // dont show legend for CDF
        .show_legend(false)
        .x_axis("x")
        .y_axis("y");
    plot.add_trace(trace);

    let title = format!("{}", "title");
    let layout = Layout::new()
        .title(format!("{} Cumulative distribution function", title))
        .show_legend(true)
        .height(1000)
        .grid(
            LayoutGrid::new()
                .rows(1)
                .columns(1)
                .pattern(GridPattern::Independent),
        );
    plot.set_layout(layout);
    let file = "plot.html";
    plot.write_html(file);

    file.to_owned()
}

// https://users.rust-lang.org/t/observed-cdf-of-a-vector/77566/4
fn cdf(data: &Data<Vec<f64>>) -> Vec<(f64, f64)> {
    let ln = data.len() as f64;
    // TODO: can we avoid the clone here?
    let mut x_ord: Vec<f64> = data.iter().cloned().collect();

    x_ord.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if let Some(mut previous) = x_ord.get(0).map(|&f| f) {
        let mut cdf = Vec::new();
        for (i, f) in x_ord.into_iter().enumerate() {
            if f != previous {
                cdf.push((previous, i as f64 / ln));
                previous = f;
            }
        }

        cdf.push((previous, 1.0));
        cdf
    } else {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct StatsReport {
    median: f64,
    mean: Option<f64>,
    p0: f64,
    p25: f64,
    p50: f64,
    p75: f64,
    p90: f64,
    p99: f64,
    p100: f64,
}

impl StatsReport {
    pub(crate) fn new(data: &mut Data<Vec<f64>>) -> Self {
        StatsReport {
            median: data.median(),
            mean: data.mean(),
            p0: data.percentile(0),
            p25: data.percentile(25),
            p50: data.percentile(50),
            p75: data.percentile(75),
            p90: data.percentile(90),
            p99: data.percentile(99),
            p100: data.percentile(100),
        }
    }
}
