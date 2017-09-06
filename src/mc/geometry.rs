use rand::{Rand, Rng};

use dimensioned::si::*;


/// Type that describes a location in 2D-space.
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    x: Meter<f64>,
    y: Meter<f64>,
}

impl Point {
    pub fn new(x: Meter<f64>, y: Meter<f64>) -> Self {
        Point { x, y }
    }

    /// Returns the X-coordinate of the point.
    pub fn x(&self) -> Meter<f64> {
        self.x
    }

    /// Returns the Y-coordinate of the point.
    pub fn y(&self) -> Meter<f64> {
        self.y
    }

    /// Sets the X-coordinate of the point to a new value.
    pub fn set_x(&mut self, x: Meter<f64>) {
        self.x = x;
    }

    /// Sets the Y-coordinate of the point to a new value.
    pub fn set_y(&mut self, y: Meter<f64>) {
        self.y = y;
    }

    /// Moves the point a certain distance in a given direction.
    ///
    /// # Example
    ///
    /// ```
    /// extern crate mcgen;
    /// extern crate dimensioned;
    ///
    /// use mcgen::mc::geometry::*;
    /// use dimensioned::si::*;
    ///
    /// point = Point::new(1.0 * M, 1.0 * M);
    /// point.step(Direction::from_angle(0.0), 3.0 * M);
    ///
    /// assert_eq!(point.to_tuple(), (4.0 * M, 1.0 * M));
    /// ```
    pub fn step(&mut self, d: &Direction, length: Meter<f64>) {
        self.x += d.dx() * length;
        self.y += d.dy() * length;
    }

    /// Returns the coordinates of this point as a tuple.
    pub fn to_tuple(&self) -> (Meter<f64>, Meter<f64>) {
        (self.x, self.y)
    }
}

impl From<Point> for (Meter<f64>, Meter<f64>) {
    fn from(point: Point) -> Self {
        point.to_tuple()
    }
}

impl From<(Meter<f64>, Meter<f64>)> for Point {
    fn from((x, y): (Meter<f64>, Meter<f64>)) -> Self {
        Point::new(x, y)
    }
}


/// Type that describes a direction in 2D-space.
///
/// `Direction`s are similar to `Point`s, but they are normalized to
/// a length of `1` and don't carry a physical unit.
#[derive(Clone, Debug, PartialEq)]
pub struct Direction {
    dx: Unitless<f64>,
    dy: Unitless<f64>,
}

impl Direction {
    /// Creates a new direction from the given vector.
    ///
    /// The numbers `dx` and `dy` are interpreted as X- and
    /// Y-coordinate of a 2D vector describing the desired direction.
    /// The returned direction is formed by normalizing the length of
    /// the vector `(dx, dy)`.
    pub fn new(mut dx: Unitless<f64>, mut dy: Unitless<f64>) -> Self {
        let len = (dx * dx + dy * dy).sqrt();
        dx /= len;
        dy /= len;
        Direction { dx, dy }
    }

    /// Creates a new direction from a given angle.
    ///
    /// The angle is interpreted as going counter-clockwise from the
    /// positive X-axis to the vector of the desired direction.
    pub fn from_angle(angle: Unitless<f64>) -> Self {
        Direction {
            dx: Unitless::new(angle.cos()),
            dy: Unitless::new(angle.sin()),
        }
    }

    /// Returns the X-component of the vector describing the direction.
    pub fn dx(&self) -> Unitless<f64> {
        self.dx
    }

    /// Returns the Y-component of the vector describing the direction.
    pub fn dy(&self) -> Unitless<f64> {
        self.dy
    }

    /// Rotates the direction by a given angle.
    ///
    /// A positive angle rotates the direction counter-clockwise.
    pub fn rotate(&mut self, angle: Unitless<f64>) {
        let dx = self.dx * angle.cos() - self.dy * angle.sin();
        let dy = self.dx * angle.sin() + self.dy * angle.cos();
        self.dx = dx;
        self.dy = dy;
    }
}

impl Rand for Direction {
    /// Generates a 2D vector pointing in a random direction.
    fn rand<R: Rng>(rng: &mut R) -> Self {
        // Generate sin x and use that that sin²x + cos²x = 1.
        // The multiplication by -1 solves the issue that squaring the
        // sine loses its sign.
        let dx = rng.gen_range(-1.0f64, 1.0f64);
        let dy = (1.0 - dx * dx).sqrt() * if rng.gen::<bool>() { 1.0 } else { -1.0 };
        Direction::new(Unitless::new(dx), Unitless::new(dy))
    }
}
