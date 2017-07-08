extern crate num;
extern crate rand;
extern crate mcgen;

use std::iter;

use rand::distributions;

use mcgen::Sample;


type F64Range = distributions::Range<f64>;
type F64PointSample = mcgen::sample::PointSample<f64, F64Range>;
type Rejection = iter::Map<F64PointSample, fn((f64, f64)) -> f64>;
type Integration = mcgen::integrate::Integrate<fn(f64) -> f64, f64>;


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


fn main() {
    let sample_size = 1_000_000;
    println!("Integration method:");
    mcgen::print_stats_and_time(
        || {
            get_integration_pi_calculator()
                .take(sample_size)
                .collect()
        },
    );
    println!();
    println!("Rejection method:");
    mcgen::print_stats_and_time(|| get_rejection_pi_calculator().take(sample_size).collect());
}
