extern crate mcgen;
extern crate rand;

use std::iter;

use rand::Rng;
use rand::distributions::IndependentSample;


fn print_statistics<R, D>(rng: &mut R, dist: D, sample_size: usize)
where
    R: Rng,
    D: IndependentSample<f64>,
{
    let sample = iter::repeat(())
        .take(sample_size)
        .map(|_| dist.ind_sample(rng));

    let mut stats = mcgen::Statistics::new();
    let seconds_needed = mcgen::time::measure_seconds(|| {
        stats = mcgen::Statistics::from_sample(sample);
    });

    println!("{}", stats);
    println!("time: {:.2} s", seconds_needed);
}

// Berechnen Sie den Mittelwert, die Streubreite und die Unsicherheit
// des Mittelwerts (99,73% Vertrauensbereich) jeweils für eine
// Stichprobe von 10^8 gleich- verteilten, exponentialverteilten und
// normalverteilten Zufallszahlen. Führen Sie die Berechnungen jeweils
// anhand eines physikalischen Beispiels durch.

fn main() {
    let mut rng = rand::thread_rng();
    let sample_size = 100_000_000;

    println!("Uniform distribution:");
    let dist = rand::distributions::Range::new(0.0, 1.0);
    print_statistics(&mut rng, dist, sample_size);
    println!();

    println!("Exponential distribution:");
    let dist = rand::distributions::Exp::new(1.0);
    print_statistics(&mut rng, dist, sample_size);
    println!();

    println!("Normal distribution:");
    let dist = rand::distributions::Normal::new(0.0, 1.0);
    print_statistics(&mut rng, dist, sample_size);
    println!();
}
