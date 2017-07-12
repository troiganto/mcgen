use rand::Rng;

use super::{Point, Direction};

#[derive(Debug)]
pub struct Photon {
    location: Point,
    direction: Direction,
    energy: f64,
}

impl Photon {
    pub fn emit_at<R: Rng>(location: Point, energy: f64, rng: &mut R) -> Self {
        let direction = rng.gen();
        Photon {
            location,
            direction,
            energy,
        }
    }

    pub fn location(&self) -> &Point {
        &self.location
    }
    pub fn direction(&self) -> &Direction {
        &self.direction
    }
    pub fn energy(&self) -> f64 {
        self.energy
    }

    pub fn direction_mut(&mut self) -> &mut Direction {
        &mut self.direction
    }
    pub fn set_energy(&mut self, energy: f64) {
        self.energy = energy
    }

    pub fn go_to_x(&mut self, x: f64) -> Result<(), Error> {
        let dx = x - self.location.x();
        let scale = dx / self.direction.dx();
        self.step(scale)
    }

    pub fn go_to_y(&mut self, y: f64) -> Result<(), Error> {
        let dy = y - self.location.y();
        let scale = dy / self.direction.dy();
        self.step(scale)
    }

    pub fn step(&mut self, scale: f64) -> Result<(), Error> {
        if scale > 0.0 {
            self.location.step(&self.direction, scale);
            Ok(())
        } else {
            Err(Error::WrongDirection)
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    WrongDirection,
}
