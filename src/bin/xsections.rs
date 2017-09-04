extern crate rand;
extern crate mcgen;
extern crate gnuplot;
extern crate dimensioned;

use std::env;

use rand::thread_rng;
use dimensioned::si::*;
use dimensioned::Dimensionless;
use dimensioned::f64prefixes::*;

use mcgen::IntoSampleIter;
use mcgen::crosssection::*;
use mcgen::Histogram;


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


fn get_args() -> (String, String, usize, usize) {
    let mut args = env::args().skip(1);
    let scatter_type = args.next().expect("missing argument: scatter type");
    let element = args.next().expect("missing argument: element name");
    let n_bins = args.next()
        .map(|s| s.parse::<usize>())
        .expect("missing argument: number of bins")
        .expect("number of bins");
    let n_samples = args.next()
        .map(|s| s.parse::<usize>())
        .expect("missing argument: number of samples")
        .expect("number of samples");
    (scatter_type, element, n_bins, n_samples)
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
    let mut rng = thread_rng();
    let sample = RejectionSampler::new(&xsection, energy)
        .into_sample_iter(&mut rng)
        .take(n_samples);
    let secs = mcgen::time::measure_seconds(
        || {
            let mut hist = Histogram::new(n_bins, -1.0, 1.0);
            for mu in sample {
                hist.fill(*mu.value());
            }
            plot_histogram(filename, hist.bin_centers(), hist.bin_contents());
        },
    );
    println!("{:.2}", secs);
}

fn main() {
    let (scatter_type, element, n_bins, n_samples) = get_args();

    let energy = match element.as_str() {
        "cerium" => 300.0 * KILO * EV,
        "caesium" => 661.7 * KILO * EV,
        _ => panic!("bad element name"),
    };
    let mut filename = element.clone();
    filename.push('_');
    filename.push_str(&scatter_type);
    filename.push_str(".pdf");
    match scatter_type.as_str() {
        "coherent" => {
            let coherent = CoherentCrossSection::new("data/AFF.dat").unwrap();
            handle_cross_section(coherent, &filename, energy, n_bins, n_samples);
        },
        "incoherent" => {
            let incoherent = IncoherentCrossSection::new("data/ISF.dat").unwrap();
            handle_cross_section(incoherent, &filename, energy, n_bins, n_samples);
        },
        _ => panic!("bad scatter type"),
    }
}
