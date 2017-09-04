extern crate rand;
extern crate mcgen;
extern crate gnuplot;
extern crate dimensioned;

use rand::Rng;
use rand::distributions::IndependentSample;

use dimensioned::si::*;
use dimensioned::{Dimensionless, Recip};
use dimensioned::f64prefixes::*;

use mcgen::mc::*;
use mcgen::Function;
use mcgen::Contains;
use mcgen::Histogram;
use mcgen::crosssection::*;


fn choose<R: Rng>(rng: &mut R, weights: &[f64]) -> usize {
    let choice = rng.gen_range(0.0, weights.iter().sum());
    let mut threshold = 0.0;
    for (i, weight) in weights.iter().enumerate() {
        threshold += *weight;
        if choice < threshold {
            return i;
        }
    }
    unreachable!();
}

/// Container for all the necessary information about the experiment.
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
    /// Creates a new object.
    ///
    /// This reads the following files:
    /// - `data/MFWL.dat`: Mean free path of photons in lead (Pb) in
    ///   centimeters depending on the photon energy (in keV)
    ///   - in total,
    ///   - for coherent scattering,
    ///   - for incoherent scattering,
    ///   - for the photo-effect (absorption).
    /// - `data/AFF.dat`: The atomic form factor of lead (Pb) depending
    ///   on the photon energy (in keV).
    /// - `data/ISF.dat`: The incoherent scattering function of lead
    ///   (Pb) depending on the photon energy (in keV).
    fn new() -> Self {
        let mut mean_free_paths = Function::<f64>::multiple_from_file("data/MFWL.dat")
            .expect("MFWL.dat")
            .into_iter();
        ThisTask {
            source: Source::new((0.0 * M, 0.0 * M).into(), 661.7 * KILO * EV),
            coherent_xsection: CoherentCrossSection::new("data/AFF.dat").expect("AFF.dat"),
            incoherent_xsection: IncoherentCrossSection::new("data/ISF.dat").expect("ISF.dat"),
            mfp_tot: mean_free_paths
                .next()
                .expect("mfp_tot")
                .scale(KILO * EV, CENTI * M),
            mfp_coh: mean_free_paths
                .next()
                .expect("mfp_coh")
                .scale(KILO * EV, CENTI * M),
            mfp_inc: mean_free_paths
                .next()
                .expect("mfp_inc")
                .scale(KILO * EV, CENTI * M),
            mfp_pho: mean_free_paths
                .next()
                .expect("mfp_pho")
                .scale(KILO * EV, CENTI * M),
        }
    }

    fn get_pb_mean_free_path(&self, energy: Joule<f64>) -> Meter<f64> {
        self.mfp_tot.call(energy)
    }

    fn choose_pb_process<R: Rng>(&self, energy: Joule<f64>, rng: &mut R) -> Event {
        // We calculate three ranges of floating-point numbers and
        // draw a number from these ranges. The range that the number
        // lies in determines which event will take place.
        //
        // We weight each range by the total macroscopic scattering
        // cross-section Sigma, which is the reciprocal of the mean
        // free path. We multiply Sigma by meters to get a `Valueless`
        // quantity, since `rand` cannot handle units.
        let w_coherent = self.mfp_coh.call(energy).recip() * M;
        let w_incoherent = self.mfp_inc.call(energy).recip() * M;
        let w_photo = self.mfp_pho.call(energy).recip() * M;
        let weights = [*w_coherent.value(), *w_incoherent.value(), *w_photo.value()];
        match choose(rng, &weights) {
            0 => Event::CoherentScatter,
            1 => Event::IncoherentScatter,
            2 => Event::Absorbed,
            _ => unreachable!(),
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
        let (x, y) = location.to_tuple();
        let collimator_x = (0.5 * CENTI * M, 1.5 * CENTI * M);
        let hole_y = (-0.1 * CENTI * M, 0.1 * CENTI * M);

        if collimator_x.contains(x) && !hole_y.contains(y) {
            Material::Absorber
        } else if x > 11.5 * CENTI * M {
            Material::Detector
        } else {
            Material::Air
        }
    }

    fn get_mean_free_path(&self, material: Material, energy: Joule<f64>) -> FreePath<f64> {
        match material {
            Material::Detector => FreePath::Fix(0.0 * M),
            Material::Air => FreePath::Fix(0.1 * CENTI * M),
            Material::Absorber => FreePath::Exp(self.get_pb_mean_free_path(energy)),
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


/// Draws the histogram using `gnuplot`.
///
/// The resulting picture is saved on-disk under the path
/// `filename`. The histogram is drawn with a logarithmic y-axis.
pub fn save_hist(hist: &Histogram, filename: &str) {
    use gnuplot::AutoOption::*;
    use gnuplot::AxesCommon;

    let &(low, high) = hist.range();
    let mut figure = gnuplot::Figure::new();
    figure
        .set_terminal("pdfcairo", filename)
        .axes2d()
        .set_x_range(Fix(low), Fix(high))
        .set_y_log(Some(10.0))
        .set_y_range(Fix(1.0), Auto)
        .boxes(hist.bin_centers(), hist.bin_contents(), &[]);
    figure.show();
}


fn main() {
    let experiment = ThisTask::new();
    let mut energy_hist = Histogram::new(666, 0.0, 666.0);
    let mut radius_hist = Histogram::new(127, 0.0, 1.27);

    let n_particles = match ::std::env::args().skip(1).next() {
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
    save_hist(&energy_hist, "energy_hist.pdf");
    save_hist(&radius_hist, "radius_hist.pdf");
}
