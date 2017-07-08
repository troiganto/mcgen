extern crate mcgen;
extern crate rand;

use rand::distributions::{Exp, IndependentSample, Normal, Range};


/// Replacement that takes a distribution instead of a closure.
fn print_stats_and_time<D>(dist: D, sample_size: usize)
where
    D: IndependentSample<f64>,
{
    let sample = mcgen::Sample::with_size(dist, sample_size);
    mcgen::print_stats_and_time(|| sample.collect());
}


// Berechnen Sie den Mittelwert, die Streubreite und die Unsicherheit
// des Mittelwerts (99,73% Vertrauensbereich) jeweils für eine
// Stichprobe von 10^8 gleich- verteilten, exponentialverteilten und
// normalverteilten Zufallszahlen. Führen Sie die Berechnungen jeweils
// anhand eines physikalischen Beispiels durch.

fn main() {
    let sample_size = 100_000_000;
    println!("Uniform distribution:");
    print_stats_and_time(Range::new(0.0, 1.0), sample_size);
    println!();
    println!("Exponential distribution:");
    print_stats_and_time(Exp::new(1.0), sample_size);
    println!();
    println!("Normal distribution:");
    print_stats_and_time(Normal::new(0.0, 1.0), sample_size);
    println!();
}
