extern crate num;
extern crate rand;
extern crate mcgen;

use num::Float;

use rand::distributions;

use mcgen::{Sample, Statistics};


fn abs2((x, y): (f64, f64)) -> f64 {
    x * x + y * y
}


fn hit_or_miss_circle(sample_size: usize) -> Statistics<f64> {
    // Take a sample of uniformly distributed numbers.
    Sample::new(distributions::Range::new(0.0, 1.0))
        // Convert them into points.
        .as_points()
        // Limit the number of points.
        .take(sample_size)
        // Only count point within the circle. The 4 accounts for the
        // fact that our points only cover one quadrant of 2D space.
        .map(|point| if abs2(point) < 1.0 { 4.0 } else { 0.0 })
        // Collect statistics.
        .collect()
}


fn main() {
    let sample_size = 1_000_000;
    println!(
        "Integration method:\n{}",
        mcgen::integrate(|x| 4.0 * (1.0 - x * x).sqrt(), 0.0..1.0, sample_size)
    );
    println!();
    println!("Rejection method:\n{}", hit_or_miss_circle(sample_size));
}
