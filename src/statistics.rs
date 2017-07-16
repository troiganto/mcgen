use std::ops::*;
use std::iter::{Extend, FromIterator};
use std::fmt::{self, Debug, Display};

pub use dimensioned::traits::Sqrt;


/// A trait alias that specifies all bounds required to store a
/// variable in a `Statistics` variable.
///
/// The bounds are necessary to auto-derive `Clone`, `Default`, and
/// `Debug`. The purpose of this trait is to simplify the signature
/// of the next-higher trait alias, `Cumulable`.
pub trait Primitive: Copy + Default + Debug {}

impl<T: Copy + Default + Debug> Primitive for T {}


/// The trait of all types that can be accumulated.
///
/// This is a trait alias that greatly simplifies the signature of
/// `Stat`. It is implemented for all types that support addition,
/// substraction, and multiplication-by-scalar without changing their
/// type.
pub trait Cumulable
where
    Self: Sized + AddAssign + Sub<Output = Self> + Div<f64, Output = Self>
{
}

impl<T> Cumulable for T
where
    T: Sized + AddAssign + Sub<Output = Self> + Div<f64, Output = Self>,
{
}


/// Trait of all requirements for a type to be fed to `Statistics`.
///
/// Broadly speaking, a type must support the following operations:
/// * we must be able to accumulate it (for the mean);
/// * we must be able to accumulate its squares (for the variance);
/// * we must be able to get the square root of the variance.
///
/// This is expressed by the bounds on `Self` and `Variance`.
pub trait Stat: Primitive + Cumulable {
    type Variance: Primitive + Cumulable + Sqrt<Output = Self::StdDev>;
    type StdDev;

    /// Connects `Self::Variance` with `Self`.
    ///
    /// Ideally, this simply multiplies `d1` and `d2`.
    fn mul(d1: Self, d2: Self) -> Self::Variance;

    /// Connects `Self::StdDev` with `Self::Variance`.
    ///
    /// Ideally, this simply takes the square root of `v`.
    fn sqrt(v: Self::Variance) -> Self::StdDev;
}

impl<T> Stat for T
where
    T: Primitive + Cumulable + Mul,
    <T as Mul>::Output: Primitive + Cumulable + Sqrt,
{
    type Variance = <Self as Mul>::Output;
    type StdDev = <Self::Variance as Sqrt>::Output;

    fn mul(d1: Self, d2: Self) -> Self::Variance {
        d1 * d2
    }

    fn sqrt(v: Self::Variance) -> Self::StdDev {
        v.sqrt()
    }
}

/// Counter-like type to calculate statistics on a sample.
///
/// This type allows calculating the mean, standard deviation and
/// standard error of the mean in an incremental manner.
///
/// The algorithm has been copied from Wikipedia:
/// https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance
#[derive(Clone, Debug, Default)]
pub struct Statistics<X: Stat> {
    count: u32,
    mean: X,
    sum_of_squares: X::Variance,
}

impl<X: Stat> Statistics<X> {
    /// Creates a new, empty `Statistics` object.
    pub fn new() -> Self {
        Default::default()
    }

    /// Takes a new sample point into consideration.
    pub fn push(&mut self, x: X) {
        // Update the counter.
        self.count += 1;
        // Update the mean.
        let delta = x - self.mean;
        self.mean += delta / self.count as f64;
        // Update the sum of squares.
        let delta_2 = x - self.mean;
        self.sum_of_squares += X::mul(delta, delta_2);
    }

    /// Returns the empirical mean of the sample.
    ///
    /// An empty `Statistics` object returns the default value of the
    /// sample type.
    pub fn mean(&self) -> X {
        self.mean
    }

    /// Returns the empirical (unbiased) variance of the sample.
    ///
    /// At least two sample points must have been `push`ed to calculate
    /// the variance. If enough data is available, this function
    /// returns `Some(variance)`, otherwise `None` is returned.
    pub fn variance(&self) -> Option<X::Variance> {
        if self.count > 1 {
            // Subtract one from `count` to get an unbiased estimator
            // for the variance.
            Some(self.sum_of_squares / (self.count - 1) as f64)
        } else {
            None
        }
    }

    /// Returns the biased standard deviation of the sample.
    ///
    /// This simply returns the square root of the variance. While
    /// `variance()` is an unbiased estimator, the square root as a
    /// concave function re-introduces a slight bias. This can
    /// typically be ignored for sample sizes larger than a few dozen.
    ///
    /// If more than two samples have been `push`ed, this returns
    /// `Some(standard_deviation)`, otherwise `None` is returned.
    pub fn standard_deviation(&self) -> Option<X::StdDev> {
        self.variance().map(X::sqrt)
    }

    /// Returns the biased standard error of the mean of the sample.
    ///
    /// This estimator for the standard deviation of the mean of the
    /// sample is biased in the same way as the standard deviation.
    ///
    /// If more than two samples have been `push`ed, this returns
    /// `Some(uncertainty)`, otherwise `None` is returned.
    pub fn error_of_mean(&self) -> Option<X::StdDev> {
        self.variance()
            .map(|v| v / self.count as f64)
            .map(X::sqrt)
    }
}

impl<X: Stat> Extend<X> for Statistics<X> {
    /// Successively `push`es all elements of the iterator to `self`.
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = X>,
    {
        for point in iter {
            self.push(point);
        }
    }
}

impl<X: Stat> FromIterator<X> for Statistics<X> {
    /// Calculates the statistics of the sample provided by the
    /// iterator `iter`.
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = X>,
    {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}


impl<X> Display for Statistics<X>
where
    X: Stat + Display,
    X::Variance: Display,
    X::StdDev: Display,
{
    /// Displays the calculated statistics on two lines.
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

/// Prints statistics and execution time of a process.
pub fn print_stats_and_time<X, Func>(func: Func)
where
    X: Stat + Display,
    X::Variance: Display,
    X::StdDev: Display,
    Func: FnOnce() -> Statistics<X>,
{
    use super::time;
    let mut stats = Statistics::new();
    let secs = time::measure_seconds(|| stats = func());
    println!("{}", stats);
    println!("time: {:.3}", secs);
}
