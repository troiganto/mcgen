use num::Float;

use rand::{Rand, Rng};
use rand::distributions::{self, IndependentSample};


pub mod particle;
pub mod experiment;


pub use self::particle::Photon;
pub use self::experiment::{Source, Experiment, simulate_particle};

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


#[derive(Debug, Clone)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }
    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn step(&mut self, d: &Direction, scale: f64) {
        self.x += d.dx() * scale;
        self.y += d.dy() * scale;
    }
}


#[derive(Debug)]
pub struct Direction {
    dx: f64,
    dy: f64,
}

impl Direction {
    pub fn new(mut dx: f64, mut dy: f64) -> Self {
        let len = (dx * dx + dy * dy).sqrt();
        dx /= len;
        dy /= len;
        Direction { dx, dy }
    }

    pub fn dx(&self) -> f64 {
        self.dx
    }
    pub fn dy(&self) -> f64 {
        self.dy
    }

    pub fn rotate(&mut self, angle: f64) {
        let dx = self.dx * angle.cos() - self.dy * angle.sin();
        let dy = self.dx * angle.sin() + self.dy * angle.cos();
        self.dx = dx;
        self.dy = dy;
    }
}

impl Rand for Direction {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let dist = distributions::Range::new(-1.0, 1.0);
        let dx = dist.ind_sample(rng);
        let mut dy = dx.acos().sin();
        if rng.gen::<bool>() {
            dy *= -1.0;
        }
        Direction::new(dx, dy)
    }
}
