extern crate num;
extern crate rand;
extern crate mcgen;
extern crate gnuplot;

use std::iter;
use std::f64::consts;

use rand::distributions;

use mcgen::{Sample, Statistics};


type F64Range = distributions::Range<f64>;
type F64PointSample = mcgen::sample::PointSample<f64, F64Range>;
type Rejection = iter::Map<F64PointSample, fn((f64, f64)) -> f64>;
type Integration = mcgen::integrate::Integrate<fn(f64) -> f64, f64>;


const SAMPLE_SIZE: usize = 1_000_000;


#[derive(Debug, Default)]
struct PlotData {
    pub epochs: Vec<usize>,
    pub means: Vec<f64>,
    pub mean_uncertainties: Vec<f64>,
    pub abs_errors: Vec<f64>,
    pub rel_errors: Vec<f64>,
}

impl PlotData {
    pub fn new() -> Self {
        PlotData::default()
    }

    pub fn clear(&mut self) {
        self.epochs.clear();
        self.means.clear();
        self.mean_uncertainties.clear();
        self.abs_errors.clear();
        self.rel_errors.clear();
    }

    pub fn fill<I>(&mut self, mut sample: I, target: f64)
    where
        I: Iterator<Item = f64>,
    {
        self.clear();
        self.fill_epochs();
        let mut data_taken = 0;
        let mut stats = Statistics::new();
        for epoch in &self.epochs {
            stats.add_sample(sample.by_ref().take(epoch - data_taken));
            data_taken = *epoch;
            self.means.push(stats.mean());
            self.mean_uncertainties.push(stats.error_of_mean());
            self.abs_errors.push(stats.mean() - target);
            self.rel_errors.push(stats.mean() / target - 1.0);
        }
    }

    fn fill_epochs(&mut self) {
        let mut epoch = 10;
        while epoch <= SAMPLE_SIZE {
            self.epochs.push(epoch);
            epoch *= 10;
        }
    }

    pub fn plot_means(first: &Self, second: &Self) {
        use gnuplot::{Figure, AxesCommon};
        use gnuplot::PlotOption::*;
        use gnuplot::AutoOption::*;
        use gnuplot::TickOption;

        let mut means = Figure::new();
        means
            .set_terminal("pdfcairo", "means.pdf")
            .axes2d()
            .set_x_label("Sample size", &[])
            .set_x_range(Fix(1.0), Fix(SAMPLE_SIZE as f64))
            .set_x_log(Some(10.0))
            .set_x_ticks(Some((Auto, 0)), &[TickOption::Format("10^{%T}")], &[])
            .set_y_label("~π{0.8∼}", &[])
            .set_y_range(Fix(1.5), Fix(4.5))
            .y_error_lines(
                &first.epochs,
                &first.means,
                &first.mean_uncertainties,
                &[Color("black"), Caption("Integration method")],
            )
            .y_error_lines(
                &second.epochs,
                &second.means,
                &second.mean_uncertainties,
                &[Color("red"), Caption("Rejection method")],
            );
        means.show();
    }

    pub fn plot_abs_errors(first: &Self, second: &Self) {
        use gnuplot::{Figure, AxesCommon};
        use gnuplot::PlotOption::*;
        use gnuplot::AutoOption::*;
        use gnuplot::TickOption;

        let mut means = Figure::new();
        means
            .set_terminal("pdfcairo", "abs_errors.pdf")
            .axes2d()
            .set_x_label("Sample size", &[])
            .set_x_range(Fix(1.0), Fix(SAMPLE_SIZE as f64))
            .set_x_log(Some(10.0))
            .set_x_ticks(Some((Auto, 0)), &[TickOption::Format("10^{%T}")], &[])
            .set_y_label("~π{0.8∼}&{−}− π", &[])
            .set_y_range(Fix(-1.5), Fix(1.5))
            .y_error_lines(
                &first.epochs,
                &first.abs_errors,
                &first.mean_uncertainties,
                &[Color("black"), Caption("Integration method")],
            )
            .y_error_lines(
                &second.epochs,
                &second.abs_errors,
                &second.mean_uncertainties,
                &[Color("red"), Caption("Rejection method")],
            );
        means.show();
    }

    pub fn plot_rel_errors(first: &Self, second: &Self) {
        use gnuplot::{Figure, AxesCommon};
        use gnuplot::PlotOption::*;
        use gnuplot::AutoOption::*;
        use gnuplot::TickOption;

        let mut means = Figure::new();
        means
            .set_terminal("pdfcairo", "rel_errors.pdf")
            .axes2d()
            .set_x_label("Sample size", &[])
            .set_x_range(Fix(1.0), Fix(SAMPLE_SIZE as f64))
            .set_x_log(Some(10.0))
            .set_x_ticks(Some((Auto, 0)), &[TickOption::Format("10^{%T}")], &[])
            .set_y_label("~π{0.8∼}&{−}/π − 1", &[])
            .set_y_range(Fix(-1.0), Fix(1.0))
            .points(
                &first.epochs,
                &first.rel_errors,
                &[Color("black"), Caption("Integration method")],
            )
            .points(
                &second.epochs,
                &second.rel_errors,
                &[Color("red"), Caption("Rejection method")],
            );
        means.show();
    }
}


fn get_point_weight((x, y): (f64, f64)) -> f64 {
    if x * x + y * y < 1.0 { 4.0 } else { 0.0 }
}


fn circle_graph(x: f64) -> f64 {
    4.0 * (1.0 - x * x).sqrt()
}


fn get_rejection_pi_calculator() -> Rejection {
    Sample::from(distributions::Range::new(0.0, 1.0))
        .as_points()
        .map(get_point_weight)
}


fn get_integration_pi_calculator() -> Integration {
    mcgen::integrate::Integrate::new(circle_graph, 0.0..1.0)
}


fn results_and_time_of_full_run() {
    println!("Integration method:");
    mcgen::print_stats_and_time(
        || {
            get_integration_pi_calculator()
                .take(SAMPLE_SIZE)
                .collect()
        },
    );
    println!();
    println!("Rejection method:");
    mcgen::print_stats_and_time(|| get_rejection_pi_calculator().take(SAMPLE_SIZE).collect());
}


fn make_incremental_plots() {
    // Create vectors for plotting.
    let mut integration_data = PlotData::new();
    integration_data.fill(get_integration_pi_calculator(), consts::PI);
    let mut rejection_data = PlotData::new();
    rejection_data.fill(get_rejection_pi_calculator(), consts::PI);
    PlotData::plot_means(&integration_data, &rejection_data);
    PlotData::plot_abs_errors(&integration_data, &rejection_data);
    PlotData::plot_rel_errors(&integration_data, &rejection_data);
}


fn main() {
    results_and_time_of_full_run();
    make_incremental_plots();
}
