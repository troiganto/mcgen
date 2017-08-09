use dimensioned::si::Joule;

use dimensioned::si::*;

use mc::geometry::{Point, Direction};


/// Type that represents a photon ("light particle").
///
/// Photons, according to this simulation program, have three
/// properties:
/// - a `location`,
/// - a `direction`, and
/// - an energy.
///
/// The typical lifecycle of a photon is:
/// 1. It is created by some source.
/// 2. It travels in a certain direction for a while.
/// 3. At some point, it interacts with the medium. This interaction
///    may either be *scattering*, which may change the photons
///    direction and energy, or *absorption*, which ends the photon's
///    lifecycle.
#[derive(Debug)]
pub struct Photon {
    location: Point,
    direction: Direction,
    energy: Joule<f64>,
}

impl Photon {
    /// Creates a new photon with the given properties.
    pub fn new(location: Point, direction: Direction, energy: Joule<f64>) -> Self {
        Photon {
            location,
            direction,
            energy,
        }
    }

    /// Immutably borrows the location of the photon.
    pub fn location(&self) -> &Point {
        &self.location
    }

    /// Immutably borrows the direction of the photon.
    pub fn direction(&self) -> &Direction {
        &self.direction
    }

    /// Returns the energy of the photon.
    pub fn energy(&self) -> Joule<f64> {
        self.energy
    }

    /// Mutably borrows the direction of the photon.
    ///
    /// This allows changes to be made.
    pub fn direction_mut(&mut self) -> &mut Direction {
        &mut self.direction
    }

    /// Set the energy of the photon to a new value.
    pub fn set_energy(&mut self, energy: Joule<f64>) {
        self.energy = energy
    }

    /// Moves the photon for a given length into its current direction.
    ///
    /// This updates the photon's `location`, but leaves its other
    /// parameters untouched.
    ///
    /// # Errors
    /// This fails with `Error::WrongDirection` if `length` is negative
    /// or zero.
    pub fn step(&mut self, length: Meter<f64>) -> Result<(), Error> {
        if length > 0.0 * M {
            self.location.step(&self.direction, length);
            Ok(())
        } else {
            Err(Error::WrongDirection)
        }
    }

    /// Moves the photon into its current direction until it reaches a
    /// certain value on the X-axis.
    ///
    /// # Errors
    /// This fails with `Error::WrongDirection` if the given value
    /// cannot be reached. This is the case if the photon's direction
    /// is pointing away from it.
    pub fn go_to_x(&mut self, x: Meter<f64>) -> Result<(), Error> {
        let dx = x - self.location.x();
        let scale = dx / self.direction.dx();
        self.step(scale)
    }

    /// Moves the photon into its current direction until it reaches a
    /// certain value on the Y-axis.
    ///
    /// # Errors
    /// This fails with `Error::WrongDirection` if the given value
    /// cannot be reached. This is the case if the photon's direction
    /// is pointing away from it.
    pub fn go_to_y(&mut self, y: Meter<f64>) -> Result<(), Error> {
        let dy = y - self.location.y();
        let scale = dy / self.direction.dy();
        self.step(scale)
    }
}


/// The error type returned by the moving functions of `Photon`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    WrongDirection,
}
