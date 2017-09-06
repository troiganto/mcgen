use rand::{Rng, thread_rng};

use rand::distributions::{self, IndependentSample};

use dimensioned::si::*;
use dimensioned::Dimensionless;

use super::Point;
use super::source::Source;
use super::particle::Photon;


/// The type of all materials that can exist at a given point.
///
/// This type is used by `Experiment` to describe the experimental
/// setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    /// A non-interactive material.
    Air,
    /// A highly absorbing material.
    Absorber,
    /// A material that can detect photons.
    Detector,
}


/// The type returned by `Experiment::get_mean_free_path()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreePath<T> {
    /// The free path should always be this exact value.
    Fix(Meter<T>),
    /// The free path should follow an exponential distribution with
    /// the given mean.
    Exp(Meter<T>),
}


/// The type of all possible outcomes of an interaction.
///
/// This type is used by `Experiment::gen_event` to find out which
/// cross-section to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// No iteraction occurred.
    Nothing,
    /// Coherent scattering occured.
    CoherentScatter,
    /// Incoherent scattering occurred.
    IncoherentScatter,
    /// The photon was absorbed.
    Absorbed,
}


/// Private type that describes the outcome of an interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParticleStatus {
    /// The particle has been lost, e.g. due to absorption.
    Lost,
    /// The particle is still propagating through the experiment.
    Propagating,
    /// The particle has been absorbed by the detector material.
    Detected,
}


/// The trait of all types that describe an experiment.
///
/// This trait provides an interface through which the function
/// `simulate_particle()` may query information that is necessary to
/// correctly simulate the particle's path through the setup.
///
/// This trait is not particularly general and has several arbitrary
/// restrictions to keep it simple.
pub trait Experiment {
    type Source: Source;

    /// Returns a reference for the photon particle source of the
    /// experiment.
    ///
    /// By design, this trait currently allows only one source to be
    /// used.
    fn source(&self) -> &Self::Source;

    /// Returns the X-coordinate at which the experiment begins.
    ///
    /// This trait assumes that the particle source lies somewhere to
    /// the left and particles enter the setup from this side. The
    /// purpose of the method `x_start` is to filter out particles that
    /// move away from the experiment as early as possible.
    fn x_start(&self) -> Meter<f64>;

    /// Describes the setup of the experiment.
    ///
    /// This function must be able to determine the material of the
    /// object that takes up space at any given location. It thus
    /// serves a complete description of what the experimental setup
    /// is.
    fn get_material(&self, location: &Point) -> Material;

    /// Returns the mean free path associated with at material.
    ///
    /// The mean free path is allowed to depend on the particle's
    /// energy. The free path may either be fixed or follow an
    /// exponential distribution.
    fn get_mean_free_path(&self, material: Material, energy: Joule<f64>) -> FreePath<f64>;

    /// Decides whether a collision occurs at a certain point.
    ///
    /// This function should randomly decide what kind of interaction
    /// with the medium occurs, if any. This decision may depend on the
    /// medium's material, the particle's energy, and a source of
    /// randomness, `rng`.
    fn gen_event<R: Rng>(&self, material: Material, energy: Joule<f64>, rng: &mut R) -> Event;

    /// Returns a random scattering angle due to elastic scattering.
    ///
    /// If the decision has been made that an elastic-scattering event
    /// shall take place, this function is called to determine by which
    /// angle the particle should be scattered. The results of this
    /// function should be distributed symmetrically around `0`.
    fn gen_coherent_scatter<R: Rng>(
        &self,
        material: Material,
        energy: Joule<f64>,
        rng: &mut R,
    ) -> Unitless<f64>;

    /// Returns the result of an inelastic-scattering event.
    ///
    /// If the decision has been made that an inelastic-scattering
    /// event shall take place, this function is called to determine by
    /// which angle the particle should be scattered *and* what its new
    /// energy should be. The returned angle should be distributed
    /// symmetrically around `0`.
    fn gen_incoherent_scatter<R: Rng>(
        &self,
        material: Material,
        energy: Joule<f64>,
        rng: &mut R,
    ) -> (Unitless<f64>, Joule<f64>);
}


/// Simulates a single photon passing through an experiment.
///
/// This creates a photon at the experiment's source and simulates its
/// path through the experiment. If the photon is lost on its way, the
/// procedure is repeated from the start. This process is repeated
/// until eventually a photon is detected.
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


/// Private function that iterates a particle by one time step.
///
/// More specifically, this samples the free path of the particle and
/// moves it by this distance. Then, an interaction with the medium is
/// simulated. The particle may either scatter, be absorbed, or go on
/// unhindered.
///
/// The return value reports the result of the particle's interaction.
fn propagate<E, R>(exp: &E, photon: &mut Photon, rng: &mut R) -> ParticleStatus
where
    E: Experiment,
    R: Rng,
{
    // Move the particle. If it leaves the experiment, stop.
    let material = exp.get_material(photon.location());
    let scale = match exp.get_mean_free_path(material, photon.energy()) {
        FreePath::Fix(scale) => scale,
        FreePath::Exp(mean) => {
            let lambda = M / mean;
            let distribution = distributions::Exp::new(*lambda.value());
            distribution.ind_sample(rng) * M
        },
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
