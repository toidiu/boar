use plotly::{Scatter, layout::GridPattern, layout::Layout, layout::LayoutGrid};
use std::fmt::Debug;

pub trait ToStats {
    type Metric: Debug;

    fn parse_metric(&self, log: &str) -> Self::Metric;
}

pub fn gen_cdf(stats: &[f64]) -> Vec<(f64, f64)> {
    // Generate CDF
    let mut x: Vec<f64> = Vec::new();
    x.extend_from_slice(&stats);

    cdf(&x)
}

pub(crate) fn plot_cdf(data: Vec<(f64, f64)>) {
    let mut plot = plotly::Plot::new();
    let (x, y): (Vec<_>, Vec<_>) = data.into_iter().map(|(a, b)| (a, b)).unzip();

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
    plot.write_html("plot.html");
    // plot.show();
}

// https://users.rust-lang.org/t/observed-cdf-of-a-vector/77566/4
pub(crate) fn cdf(x: &[f64]) -> Vec<(f64, f64)> {
    let ln = x.len() as f64;
    let mut x_ord = x.to_vec();
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
