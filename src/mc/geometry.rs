use rand::{Rand, Rng};
use rand::distributions::{self, IndependentSample};

use dimensioned::si::*;


#[derive(Debug, Clone)]
pub struct Point {
    x: Meter<f64>,
    y: Meter<f64>,
}

impl Point {
    pub fn new(x: Meter<f64>, y: Meter<f64>) -> Self {
        Point { x, y }
    }

    pub fn x(&self) -> Meter<f64> {
        self.x
    }
    pub fn y(&self) -> Meter<f64> {
        self.y
    }

    pub fn step(&mut self, d: &Direction, length: Meter<f64>) {
        self.x += d.dx() * length;
        self.y += d.dy() * length;
    }

    pub fn to_tuple(&self) -> (Meter<f64>, Meter<f64>) {
        (self.x, self.y)
    }
}

impl From<(Meter<f64>, Meter<f64>)> for Point {
    fn from((x, y): (Meter<f64>, Meter<f64>)) -> Self {
        Point::new(x, y)
    }
}


#[derive(Debug)]
pub struct Direction {
    dx: Unitless<f64>,
    dy: Unitless<f64>,
}

impl Direction {
    pub fn new(mut dx: Unitless<f64>, mut dy: Unitless<f64>) -> Self {
        let len = (dx * dx + dy * dy).sqrt();
        dx /= len;
        dy /= len;
        Direction { dx, dy }
    }

    pub fn dx(&self) -> Unitless<f64> {
        self.dx
    }
    pub fn dy(&self) -> Unitless<f64> {
        self.dy
    }

    pub fn rotate(&mut self, angle: Unitless<f64>) {
        let dx = self.dx * angle.cos() - self.dy * angle.sin();
        let dy = self.dx * angle.sin() + self.dy * angle.cos();
        self.dx = dx;
        self.dy = dy;
    }
}

impl Rand for Direction {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let dist = distributions::Range::new(-1.0f64, 1.0f64);
        let dx = dist.ind_sample(rng);
        let mut dy = dx.acos().sin();
        if rng.gen::<bool>() {
            dy *= -1.0;
        }
        Direction::new(Unitless::new(dx), Unitless::new(dy))
    }
}
