use std::ops::*;
use std::iter::FromIterator;
use std::fmt::{self, Debug, Display};

use num::Float;


pub trait Primitive: Copy + Default + Debug {}

impl<T: Copy + Default + Debug> Primitive for T {}


pub trait Collectible
where
    Self: Primitive
        + Add<Output=Self>
        + Sub<Output=Self>
        + Mul<Output=Self>
        + Div<f64, Output = Self>
{
}

impl<T> Collectible for T
where
    T: Primitive
        + Add<Output=T>
        + Sub<Output=T>
        + Mul<Output=T>
        + Div<f64, Output = T>,
{
}

/// Counter-like type to calculate statistics on a sample.
///
/// that allows calculating the mean, standard deviation and standard
/// error of the mean
/// in an incremental manner.
/// The algorithm has been copied from Wikipedia:
/// https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance
#[derive(Clone, Debug, Default)]
pub struct Statistics<F>
where
    F: Collectible,
{
    count: u32,
    mean: F,
    sum_of_squares: F,
}

impl<F> Statistics<F>
where
    F: Float + Collectible,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_sample<I>(sample: I) -> Self
    where
        I: IntoIterator<Item = F>,
    {
        let mut result = Self::new();
        result.add_sample(sample);
        result
    }

    pub fn add_sample<I>(&mut self, sample: I)
    where
        I: IntoIterator<Item = F>,
    {
        for point in sample {
            self.add(point);
        }
    }

    pub fn add(&mut self, sample: F) {
        self.count += 1;
        let delta = sample - self.mean;
        self.mean = self.mean + delta / self.count as f64;
        let delta_2 = sample - self.mean;
        self.sum_of_squares = self.sum_of_squares + delta * delta_2;
    }

    pub fn mean(&self) -> F {
        self.mean
    }

    pub fn variance(&self) -> Option<F> {
        if self.count > 1 {
            Some(self.sum_of_squares / (self.count - 1) as f64)
        } else {
            None
        }
    }

    pub fn standard_deviation(&self) -> Option<F> {
        self.variance().map(F::sqrt)
    }

    pub fn error_of_mean(&self) -> Option<F> {
        self.variance()
            .map(|v| v / self.count as f64)
            .map(F::sqrt)
    }
}

impl<F> Display for Statistics<F>
where
    F: Float + Collectible + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Mean: {0:.5} ± {1:.5}\nStandard deviation: {2:.5}",
            self.mean(),
            self.error_of_mean().expect("cannot calculate variance"),
            self.standard_deviation()
                .expect("cannot calculate variance")
        )
    }
}

impl<F> FromIterator<F> for Statistics<F>
where
    F: Float + Collectible,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = F>,
    {
        Statistics::from_sample(iter)
    }
}


/// Prints statistics and execution time of a process.
pub fn print_stats_and_time<F, Func>(func: Func)
where
    F: Float + Collectible + Display,
    Func: FnOnce() -> Statistics<F>,
{
    use super::time;
    let mut stats = Statistics::default();
    let secs = time::measure_seconds(|| stats = func());
    println!("{}", stats);
    println!("time: {:.3}", secs);
}
