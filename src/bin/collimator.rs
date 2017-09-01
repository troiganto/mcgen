extern crate rand;
extern crate mcgen;
extern crate gnuplot;
extern crate dimensioned;

use rand::Rng;
use rand::distributions::IndependentSample;

use dimensioned::si::*;
use dimensioned::{Dimensionless, Recip};
use dimensioned::f64prefixes::*;

use mcgen::crosssection::*;
use mcgen::mc::*;
use mcgen::Function;


struct ThisTask {
    source: Source,
    coherent_xsection: CoherentCrossSection,
    incoherent_xsection: IncoherentCrossSection,
    mfp_tot: Function<Joule<f64>, Meter<f64>>,
    mfp_coh: Function<Joule<f64>, Meter<f64>>,
    mfp_inc: Function<Joule<f64>, Meter<f64>>,
    mfp_pho: Function<Joule<f64>, Meter<f64>>,
}

impl ThisTask {
    fn new() -> Self {
        let mut mean_free_paths = Function::<f64>::multiple_from_file("data/MFWL.dat")
            .expect("MFWL.dat")
            .into_iter();
        ThisTask {
            source: Source::new((0.0 * M, 0.0 * M).into(), 661.7 * KILO * EV),
            coherent_xsection: CoherentCrossSection::new("data/AFF.dat").expect("AFF.dat"),
            incoherent_xsection: IncoherentCrossSection::new("data/ISF.dat").expect("ISF.dat"),
            mfp_tot: mean_free_paths.next().expect("mfp_tot").scale(KILO * EV, CENTI * M),
            mfp_coh: mean_free_paths.next().expect("mfp_coh").scale(KILO * EV, CENTI * M),
            mfp_inc: mean_free_paths.next().expect("mfp_inc").scale(KILO * EV, CENTI * M),
            mfp_pho: mean_free_paths.next().expect("mfp_pho").scale(KILO * EV, CENTI * M),
        }
    }

    fn get_pb_mean_free_path(&self, energy: Joule<f64>) -> Meter<f64> {
        self.mfp_tot.call(energy)
    }


    fn choose_pb_process<R: Rng>(&self, energy: Joule<f64>, rng: &mut R) -> Event {
        let thres_coh = 0.0;
        let thres_inc = self.mfp_coh.call(energy).recip() * M;
        let thres_pho = self.mfp_inc.call(energy).recip() * M + thres_inc;
        let upper_lim = self.mfp_pho.call(energy).recip() * M + thres_pho;

        let value = rng.gen_range(thres_coh, *upper_lim.value());
        if value > *thres_pho.value() {
            Event::Absorbed
        } else if value > *thres_inc.value() {
            Event::IncoherentScatter
        } else {
            Event::CoherentScatter
        }
    }
}

impl Experiment for ThisTask {
    fn source(&self) -> &Source {
        &self.source
    }

    fn x_start(&self) -> Meter<f64> {
        0.5 * CENTI * M
    }

    fn get_material(&self, location: &Point) -> Material {
        let (x, y) = (location.x(), location.y());
        if 0.5 * CENTI * M < x && x < 1.5 * CENTI * M &&
           (y > 0.1 * CENTI * M || y < -0.1 * CENTI * M) {
            Material::Absorber
        } else if x > 11.5 * CENTI * M {
            Material::Detector
        } else {
            Material::Air
        }
    }

    fn get_mean_free_path(&self, material: Material, energy: Joule<f64>) -> Meter<f64> {
        match material {
            Material::Detector => 0.0 * M,
            Material::Air => 0.1 * CENTI * M,
            Material::Absorber => self.get_pb_mean_free_path(energy),
        }
    }

    fn gen_event<R: Rng>(&self, material: Material, energy: Joule<f64>, rng: &mut R) -> Event {
        match material {
            Material::Detector => Event::Absorbed,
            Material::Air => Event::Nothing,
            Material::Absorber => self.choose_pb_process(energy, rng),
        }
    }

    fn gen_coherent_scatter<R: Rng>(
        &self,
        _: Material,
        energy: Joule<f64>,
        rng: &mut R,
    ) -> Unitless<f64> {
        let sampler = RejectionSampler::new(&self.coherent_xsection, energy);
        let mu = sampler.ind_sample(rng);
        let mut angle = mu.value().acos();
        if rng.gen::<bool>() {
            angle *= -1.0;
        }
        Unitless::new(angle)
    }

    fn gen_incoherent_scatter<R: Rng>(
        &self,
        _: Material,
        energy: Joule<f64>,
        rng: &mut R,
    ) -> (Unitless<f64>, Joule<f64>) {
        let sampler = RejectionSampler::new(&self.incoherent_xsection, energy);
        let mu = sampler.ind_sample(rng);
        let mut angle = mu.value().acos();
        if rng.gen::<bool>() {
            angle *= -1.0;
        }
        let new_energy = IncoherentCrossSection::compton_scatter(energy, mu);
        (Unitless::new(angle), new_energy)
    }
}


struct Histogram {
    range: (f64, f64),
    low_edges: Vec<f64>,
    weights: Vec<usize>,
}

impl Histogram {
    pub fn new(nbins: usize, low: f64, high: f64) -> Self {
        let mut low_edges = Vec::with_capacity(nbins);
        let width = (high - low) / (nbins as f64);
        for i in 0..(nbins - 1) {
            low_edges.push(low + width * (i as f64));
        }
        let weights = vec![0; nbins];
        let range = (low, high);
        Histogram {
            low_edges,
            weights,
            range,
        }
    }

    pub fn fill(&mut self, x: f64) {
        if x < self.range.0 || x >= self.range.1 {
            return;
        }
        for (i, bin) in self.low_edges.windows(2).enumerate() {
            let (low, high) = (bin[0], bin[1]);
            if low <= x && x < high {
                self.weights[i] += 1;
                break;
            }
        }
    }

    pub fn show(&self, filename: &str) {
        use gnuplot::AutoOption::*;
        use gnuplot::AxesCommon;

        let (low, high) = self.range;
        let dx = (high - low) / (self.low_edges.len() as f64);
        let centers = self.low_edges
            .iter()
            .map(|low_edge| low_edge + 0.5 * dx);

        let mut hist = gnuplot::Figure::new();
        hist.set_terminal("pdfcairo", filename)
            .axes2d()
            .set_x_range(Fix(low), Fix(high))
            .set_y_log(Some(10.0))
            .set_y_range(Fix(1.0), Auto)
            .boxes(centers, &self.weights, &[]);
        hist.show();
    }
}


fn main() {
    let experiment = ThisTask::new();
    let mut energy_hist = Histogram::new(666, 0.0, 666.0);
    let mut radius_hist = Histogram::new(127, 0.0, 1.27);

    let mut args = ::std::env::args();
    args.next();
    let n_particles = match args.next() {
        Some(s) => s.parse::<usize>().expect("not a number: n_particles"),
        None => panic!("missing argument: n_particles"),
    };

    for _ in 0..n_particles {
        let photon = simulate_particle(&experiment);
        let energy = photon.energy() / (KILO * EV);
        let radius = photon.location().y() / M;
        energy_hist.fill(*energy.value());
        radius_hist.fill(radius.value().abs());
    }
    energy_hist.show("energy_hist.pdf");
    radius_hist.show("radius_hist.pdf");
}
