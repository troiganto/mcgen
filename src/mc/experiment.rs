use rand::{Rng, thread_rng};

use rand::distributions::{self, IndependentSample};

use dimensioned::si::*;
use dimensioned::Dimensionless;

use super::Point;
use super::particle::Photon;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Air,
    Absorber,
    Detector,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Nothing,
    CoherentScatter,
    IncoherentScatter,
    Absorbed,
}


pub struct Source {
    location: Point,
    energy: Joule<f64>,
}

impl Source {
    pub fn new(location: Point, energy: Joule<f64>) -> Self {
        Source { location, energy }
    }

    pub fn emit_photon<R: Rng>(&self, rng: &mut R) -> Photon {
        Photon::new(self.location.clone(), rng.gen(), self.energy)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParticleStatus {
    Lost,
    Propagating,
    Detected,
}


pub trait Experiment {
    fn source(&self) -> &Source;

    fn x_start(&self) -> Meter<f64>;

    fn get_material(&self, location: &Point) -> Material;

    fn get_mean_free_path(&self, material: Material, energy: Joule<f64>) -> Meter<f64>;

    fn gen_event<R: Rng>(&self, material: Material, energy: Joule<f64>, rng: &mut R) -> Event;

    fn gen_coherent_scatter<R: Rng>(
        &self,
        material: Material,
        energy: Joule<f64>,
        rng: &mut R,
    ) -> Unitless<f64>;

    fn gen_incoherent_scatter<R: Rng>(
        &self,
        material: Material,
        energy: Joule<f64>,
        rng: &mut R,
    ) -> (Unitless<f64>, Joule<f64>);
}


pub fn simulate_particle<E>(exp: &E) -> Photon
where
    E: Experiment,
{
    let source = exp.source();
    let mut rng = thread_rng();
    loop {
        // Get a photon.
        let mut photon = source.emit_photon(&mut rng);

        // Make sure it's headed towards the experiment.
        if photon.go_to_x(exp.x_start()).is_err() {
            continue;
        }

        // Propagate it until it hits the detector or gets lost. If it
        // gets detected, the function is done. Otherwise, we have to
        // break the inner loop and continue the outer loop.
        let mut result;
        loop {
            result = propagate(exp, &mut photon, &mut rng);
            match result {
                ParticleStatus::Propagating => {},
                ParticleStatus::Detected => return photon,
                ParticleStatus::Lost => break,
            }
        }
    }
}


fn propagate<E, R>(exp: &E, photon: &mut Photon, rng: &mut R) -> ParticleStatus
where
    E: Experiment,
    R: Rng,
{
    // Move the particle. If it leaves the experiment, stop.
    let material = exp.get_material(photon.location());
    let mean_free_path = exp.get_mean_free_path(material, photon.energy());
    let scale = if mean_free_path == 0.0 * M {
        0.0 * M
    } else {
        let mean_free_path = mean_free_path / M;
        let distribution = distributions::Exp::new(*mean_free_path.value());
        distribution.ind_sample(rng) * M
    };
    photon.step(scale).expect("`scale` cannot be negative");
    if photon.location().x() < exp.x_start() {
        return ParticleStatus::Lost;
    }

    // Find the next interaction at the new location.
    let material = exp.get_material(photon.location());
    let event = exp.gen_event(material, photon.energy(), rng);

    match event {
        Event::Nothing => ParticleStatus::Propagating,
        Event::Absorbed => {
            match material {
                Material::Detector => ParticleStatus::Detected,
                _ => ParticleStatus::Lost,
            }
        },
        Event::CoherentScatter => {
            let angle = exp.gen_coherent_scatter(material, photon.energy(), rng);
            photon.direction_mut().rotate(angle);
            ParticleStatus::Propagating
        },
        Event::IncoherentScatter => {
            let (angle, energy) = exp.gen_incoherent_scatter(material, photon.energy(), rng);
            photon.direction_mut().rotate(angle);
            photon.set_energy(energy);
            ParticleStatus::Propagating
        },
    }
}
