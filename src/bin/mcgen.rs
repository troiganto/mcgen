
extern crate dimensioned;
extern crate gnuplot;
extern crate mcgen;

use std::env;

use dimensioned::si::*;
use dimensioned::Dimensionless;
use dimensioned::f64prefixes::*;

use mcgen::{CoherentCrossSection, IncoherentCrossSection, RejectionSampler};


fn make_mu_histogram<I>(mut sample: I, n_bins: usize, n_samples: usize) -> (Vec<f64>, Vec<f64>)
where
    I: Iterator<Item = Unitless<f64>>,
{
    let dmu = 2.0 / n_bins as f64;
    let mu_coords = {
        let mut mu_coords = Vec::with_capacity(n_bins);
        let mut mu = -1.0;
        for _ in 0..n_bins {
            mu_coords.push(mu);
            mu += dmu;
        }
        mu_coords
    };

    let bins = {
        let mut bins = vec![0.0; n_bins];
        for _ in 0..n_samples {
            let mu = sample.next().expect("not enough data");
            let mu = *mu.value();
            let i = ((mu - (-1.0)) / dmu) as usize;
            bins[i] += 1.0;
        }

        for bin in bins.iter_mut() {
            *bin /= n_samples as f64;
        }
        bins
    };

    (mu_coords, bins)
}


fn plot_histogram<Tx, X, Ty, Y>(x: X, y: Y)
where
    Tx: gnuplot::DataType,
    Ty: gnuplot::DataType,
    X: IntoIterator<Item = Tx>,
    Y: IntoIterator<Item = Ty>,
{
    use gnuplot::AutoOption::*;
    use gnuplot::AxesCommon;

    let mut hist = gnuplot::Figure::new();
    hist.axes2d()
        .set_x_label("µ", &[])
        .set_x_range(Fix(-1.0), Fix(1.0))
        .set_y_range(Fix(0.0), Auto)
        .boxes(x, y, &[]);
    hist.show();
}


fn get_args() -> (Joule<f64>, usize, usize) {
    let mut args = env::args();
    let _executable = args.next().expect("no executable");
    let energy = match args.next() {
        Some(s) => s.parse::<f64>().expect("not a number") * KILO * EV,
        None => panic!("missing argument: energy in keV"),
    };
    let n_bins = match args.next() {
        Some(s) => s.parse::<usize>().expect("not a number"),
        None => panic!("missing argument: number of bins"),
    };
    let n_samples = match args.next() {
        Some(s) => s.parse::<usize>().expect("not a number"),
        None => panic!("missing argument: number of bins"),
    };
    (energy, n_bins, n_samples)
}


fn main() {
    let (energy, n_bins, n_samples) = get_args();

    let incoherent = IncoherentCrossSection::new("../data/ISF.dat").unwrap();
    let sampler = RejectionSampler::new(incoherent, energy);

    let secs = mcgen::time::measure_seconds(
        || {
            let (x, y) = make_mu_histogram(sampler, n_bins, n_samples);
            plot_histogram(x, y);
        },
    );
    println!("{:.2} s", secs);
}
