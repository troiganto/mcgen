
extern crate dimensioned;
extern crate gnuplot;
extern crate mcgen;

use std::env;

use dimensioned::si::*;
use dimensioned::Dimensionless;
use dimensioned::f64prefixes::*;

use mcgen::crosssection::*;


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


fn plot_histogram<Tx, X, Ty, Y>(filename: &str, x: X, y: Y)
where
    Tx: gnuplot::DataType,
    Ty: gnuplot::DataType,
    X: IntoIterator<Item = Tx>,
    Y: IntoIterator<Item = Ty>,
{
    use gnuplot::AutoOption::*;
    use gnuplot::AxesCommon;

    let mut hist = gnuplot::Figure::new();
    hist.set_terminal("pdfcairo", filename)
        .axes2d()
        .set_x_label("µ", &[])
        .set_x_range(Fix(-1.0), Fix(1.0))
        .set_y_range(Fix(0.0), Auto)
        .boxes(x, y, &[]);
    hist.show();
}


fn get_args() -> (String, usize, usize) {
    let mut args = env::args();
    let _executable = args.next().expect("no executable");
    let element = match args.next() {
        Some(s) => s,
        None => panic!("missing argument: element name"),
    };
    let n_bins = match args.next() {
        Some(s) => s.parse::<usize>().expect("not a number"),
        None => panic!("missing argument: number of bins"),
    };
    let n_samples = match args.next() {
        Some(s) => s.parse::<usize>().expect("not a number"),
        None => panic!("missing argument: number of bins"),
    };
    (element, n_bins, n_samples)
}


fn handle_cross_section<XS>(
    xsection: XS,
    filename: &str,
    energy: Joule<f64>,
    n_bins: usize,
    n_samples: usize,
) where
    XS: CrossSection,
{
    let sampler = RejectionSampler::new(&xsection, energy);
    let secs = mcgen::time::measure_seconds(
        || {
            let (x, y) = make_mu_histogram(sampler, n_bins, n_samples);
            plot_histogram(filename, x, y);
        },
    );
    println!("{:.2} s", secs);
}

fn main() {
    let (element, n_bins, n_samples) = get_args();

    let energy = match element.as_str() {
        "cobalt" => 300.0 * KILO * EV,
        "caesium" => 661.7 * KILO * EV,
        _ => panic!("bad element name"),
    };
    {
        let coherent = CoherentCrossSection::new("data/AFF.dat").unwrap();
        let mut filename = element.clone();
        filename.push_str("_coherent.pdf");
        handle_cross_section(coherent, &filename, energy, n_bins, n_samples);
    }
    {
        let incoherent = IncoherentCrossSection::new("data/ISF.dat").unwrap();
        let mut filename = element.clone();
        filename.push_str("_incoherent.pdf");
        handle_cross_section(incoherent, &filename, energy, n_bins, n_samples);
    }
}
