extern crate mcgen;
extern crate rand;

use rand::distributions::{self, IndependentSample};
use mcgen::{time, Sample, Statistics};

fn print_statistics<D>(dist: D, sample_size: usize)
where
    D: IndependentSample<f64>,
{
    let mut stats = Statistics::default();
    let sample = Sample::new(dist).take(sample_size);
    let seconds_needed = time::measure_seconds(|| stats = sample.collect());
    println!("{}", stats);
    println!("time: {:.2} s", seconds_needed);
}

// Berechnen Sie den Mittelwert, die Streubreite und die Unsicherheit
// des Mittelwerts (99,73% Vertrauensbereich) jeweils für eine
// Stichprobe von 10^8 gleich- verteilten, exponentialverteilten und
// normalverteilten Zufallszahlen. Führen Sie die Berechnungen jeweils
// anhand eines physikalischen Beispiels durch.

fn main() {
    let sample_size = 100_000_000;
    println!("Uniform distribution:");
    print_statistics(distributions::Range::new(0.0, 1.0), sample_size);
    println!();
    println!("Exponential distribution:");
    print_statistics(distributions::Exp::new(1.0), sample_size);
    println!();
    println!("Normal distribution:");
    print_statistics(distributions::Normal::new(0.0, 1.0), sample_size);
    println!();
}
