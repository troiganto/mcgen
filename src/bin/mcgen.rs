extern crate num;
extern crate rand;
extern crate mcgen;
extern crate dimensioned;

use rand::Rng;
use rand::distributions::{self, IndependentSample};
use num::Float;

use dimensioned::si::*;
use dimensioned::Dimensionless;
use dimensioned::f64prefixes::*;

use mcgen::crosssection::*;
use mcgen::mc::*;
use mcgen::Function;


struct ThisTask {
    source: Source,
    coherent_xsection: CoherentCrossSection,
    incoherent_xsection: IncoherentCrossSection,
    mfp_tot: Function<f64>,
    mfp_coh: Function<f64>,
    mfp_inc: Function<f64>,
    mfp_pho: Function<f64>,
}

impl ThisTask {
    fn new() -> Self {
        let mut mean_free_paths = Function::multiple_from_file("data/MFWL.dat")
            .expect("MFWL.dat")
            .into_iter();
        ThisTask {
            source: Source::new((0.0, 0.0).into(), 661.7 * KILO * EV),
            coherent_xsection: CoherentCrossSection::new("data/AFF.dat").expect("AFF.dat"),
            incoherent_xsection: IncoherentCrossSection::new("data/ISF.dat").expect("ISF.dat"),
            mfp_tot: mean_free_paths.next().expect("mfp_tot"),
            mfp_coh: mean_free_paths.next().expect("mfp_coh"),
            mfp_inc: mean_free_paths.next().expect("mfp_inc"),
            mfp_pho: mean_free_paths.next().expect("mfp_pho"),
        }
    }

    fn sample_pb_free_path<R: Rng>(&self, energy: Joule<f64>, rng: &mut R) -> f64 {
        let energy = energy / (KILO * EV);
        let dist = distributions::Exp::new(self.mfp_tot.call(*energy.value()));
        dist.ind_sample(rng)
    }


    fn choose_pb_process<R: Rng>(&self, energy: Joule<f64>, rng: &mut R) -> Event {
        let energy = energy / (KILO * EV);
        let energy = *energy.value();

        let thres_coh = 0.0;
        let thres_inc = self.mfp_coh.call(energy).recip();
        let thres_pho = self.mfp_inc.call(energy).recip() + thres_inc;
        let upper_lim = self.mfp_pho.call(energy).recip() + thres_pho;

        let value = rng.gen_range(thres_coh, upper_lim);
        if value > thres_pho {
            Event::Absorbed
        } else if value > thres_inc {
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

    fn x_start(&self) -> f64 {
        0.5
    }

    fn get_material(&self, location: &Point) -> Material {
        let (x, y) = (location.x(), location.y());
        if 0.5 < x && x < 1.5 && y.abs() > 0.1 {
            Material::Absorber
        } else if x > 11.5 {
            Material::Detector
        } else {
            Material::Air
        }
    }

    fn gen_free_path<R: Rng>(&self, material: Material, energy: Joule<f64>, rng: &mut R) -> f64 {
        match material {
            Material::Detector => 0.0,
            Material::Air => 0.1,
            Material::Absorber => self.sample_pb_free_path(energy, rng),
        }
    }

    fn gen_event<R: Rng>(&self, material: Material, energy: Joule<f64>, rng: &mut R) -> Event {
        match material {
            Material::Detector => Event::Absorbed,
            Material::Air => Event::Nothing,
            Material::Absorber => self.choose_pb_process(energy, rng),
        }
    }

    fn gen_coherent_scatter<R: Rng>(&self, _: Material, energy: Joule<f64>, rng: &mut R) -> f64 {
        let sampler = RejectionSampler::new(&self.coherent_xsection, energy);
        let mu = sampler.ind_sample(rng);
        let mut angle = mu.value().acos();
        if rng.gen::<bool>() {
            angle *= -1.0;
        }
        angle
    }

    fn gen_incoherent_scatter<R: Rng>(
        &self,
        _: Material,
        energy: Joule<f64>,
        rng: &mut R,
    ) -> (f64, Joule<f64>) {
        let sampler = RejectionSampler::new(&self.incoherent_xsection, energy);
        let mu = sampler.ind_sample(rng);
        let mut angle = mu.value().acos();
        if rng.gen::<bool>() {
            angle *= -1.0;
        }
        let new_energy = IncoherentCrossSection::compton_scatter(energy, mu);
        (angle, new_energy)
    }
}


fn main() {
    let experiment = ThisTask::new();
    let photon = simulate_particle(&experiment);
    println!("{}", photon.energy() * KILO * EV);
}
