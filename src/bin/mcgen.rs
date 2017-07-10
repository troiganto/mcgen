
#![allow(dead_code)]

extern crate csv;
extern crate dimensioned;
extern crate gnuplot;
extern crate mcgen;

use std::path::Path;
use std::env;
use dimensioned::Dimensionless;
use dimensioned::si::*;
use dimensioned::f64prefixes::*;

use mcgen::{CrossSection, RejectionSampler};


fn r_e() -> Meter<f64> {
    let alpha = Unitless::new(1.0 / 137.0);
    R_BOHR * alpha * alpha
}

struct CoherentCrossSection {
    form_factor: mcgen::Function<f64>,
}

impl CoherentCrossSection {
    fn new<P>(form_factor_file: P) -> csv::Result<Self>
    where
        P: AsRef<Path>,
    {
        let form_factor = mcgen::Function::from_file(form_factor_file, b'\t', 2)?;
        let result = CoherentCrossSection { form_factor };
        Ok(result)
    }

    pub fn form_factor(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Unitless<f64> {
        let angle = mu.acos();
        let x = energy * (angle / 2.0).sin();
        let x = x / (KILO * EV);
        let form_factor = self.form_factor.call(*x.value());
        Unitless::new(form_factor)
    }
}

impl CrossSection for CoherentCrossSection {
    fn eval(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Meter2<f64> {
        let form_factor = self.form_factor(energy, mu);
        r_e() * r_e() * (1.0 + mu * mu) / 2.0 * form_factor * form_factor
    }

    fn max(&self, energy: Joule<f64>) -> Meter2<f64> {
        self.eval(energy, Unitless::new(1.0))
    }
}


struct IncoherentCrossSection {
    scattering_function: mcgen::Function<f64>,
}

impl IncoherentCrossSection {
    fn new<P>(scattering_function_file: P) -> csv::Result<Self>
    where
        P: AsRef<Path>,
    {
        let scattering_function = mcgen::Function::from_file(scattering_function_file, b'\t', 2)?;
        let result = IncoherentCrossSection { scattering_function };
        Ok(result)
    }

    pub fn scattering_function(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Unitless<f64> {
        let angle = mu.acos();
        let x = energy * (angle / 2.0).sin();
        let x = x / (KILO * EV);
        let scattering_function = self.scattering_function.call(*x.value());
        Unitless::new(scattering_function)
    }

    pub fn klein_nishina(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Meter2<f64> {
        let kappa = energy / (M_E * C0 * C0);
        let kappa_antimu = kappa * (1.0 - mu);
        let alpha_func = 1.0 / (1.0 + kappa_antimu);
        r_e() * r_e() / 2.0 * alpha_func * alpha_func * (alpha_func + kappa_antimu + mu * mu)
    }
}

impl CrossSection for IncoherentCrossSection {
    fn eval(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Meter2<f64> {
        self.klein_nishina(energy, mu) * self.scattering_function(energy, mu)
        //~ self.klein_nishina(energy, mu)
    }

    fn max(&self, energy: Joule<f64>) -> Meter2<f64> {
        self.klein_nishina(energy, Unitless::new(1.0))
    }
}


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

    //~ let coherent =
    //~ CoherentCrossSection::new("../data/AFF.dat").unwrap();
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
