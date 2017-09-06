use rand::Rng;

use dimensioned::si::*;

use super::geometry::{Point, Direction};
use super::particle::Photon;


/// The common trait of all photon sources.
pub trait Source {
    fn emit_photon<R: Rng>(&self, rng: &mut R) -> Photon;
}


/// An isotropic point source of monoenergetic photons.
pub struct SimpleSource {
    location: Point,
    energy: Joule<f64>,
}

impl SimpleSource {
    /// Creates a new source at the given location.
    ///
    /// The returned source produces photons of the given energy.
    pub fn new(location: Point, energy: Joule<f64>) -> Self {
        Self { location, energy }
    }

    /// Returns the source's location.
    pub fn location(&self) -> &Point {
        &self.location
    }

    /// Returns the energy of the source's photons.
    pub fn energy(&self) -> Joule<f64> {
        self.energy
    }
}

impl Source for SimpleSource {
    /// Emit a photon into a random direction.
    ///
    /// This uses `rng` as a source of randomness.
    fn emit_photon<R: Rng>(&self, rng: &mut R) -> Photon {
        Photon::new(self.location.clone(), rng.gen::<Direction>(), self.energy)
    }
}


/// Like `SimpleSource`, but it emits photons only into one hemisphere.
///
/// The hemisphere is that of the positive X-axis. Hence, this source
/// cannot generate photons going to the left.
pub struct EastPointingSource(SimpleSource);

impl EastPointingSource {
    /// Creates a new source at the given location.
    ///
    /// The returned source produces photons of the given energy.
    pub fn new(location: Point, energy: Joule<f64>) -> Self {
        EastPointingSource(SimpleSource { location, energy })
    }

    /// Returns the source's location.
    pub fn location(&self) -> &Point {
        self.0.location()
    }

    /// Returns the energy of the source's photons.
    pub fn energy(&self) -> Joule<f64> {
        self.0.energy()
    }
}

impl Source for EastPointingSource {
    /// Emit a photon into a random direction.
    ///
    /// This uses `rng` as a source of randomness.
    fn emit_photon<R: Rng>(&self, rng: &mut R) -> Photon {
        let direction = {
            // Generate cos x and use that that sin²x + cos²x = 1.
            let dy = rng.gen_range(-1.0f64, 1.0f64);
            let dx = (1.0 - dy * dy).sqrt();
            Direction::new(Unitless::new(dx), Unitless::new(dy))
        };
        Photon::new(self.location().clone(), direction, self.energy())
    }
}
