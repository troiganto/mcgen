use std::ops;
use std::cmp::PartialOrd;

use rand::Rng;
use rand::distributions::range::SampleRange;
use rand::distributions::{self, Sample, IndependentSample};

use super::{IntoSampleIter, Stat, Statistics};


/// Struct for Monte-Carlo integration of 1D real functions.
///
/// This struct is exposed to allow continuous inspection of
/// the integration result and uncertainty via `Statistics`.
///
/// For the general use case, where you just just want the result
/// of the integration, use the function `integrate()`.
pub struct Integrate<F, X, Y = X>
where
    F: FnMut(X) -> Y,
    X: Copy + SampleRange + PartialOrd + ops::Sub<Output = X>,
    Y: ops::Mul<X>,
{
    x_sample: distributions::Range<X>,
    width: X,
    func: F,
}

impl<F, X, Y> Integrate<F, X, Y>
where
    F: FnMut(X) -> Y,
    X: Copy + SampleRange + PartialOrd + ops::Sub<Output = X>,
    Y: ops::Mul<X>,
{
    /// Creates a new object that integrates `f` in the given `range`.
    pub fn new(f: F, range: ops::Range<X>) -> Self {
        Integrate {
            func: f,
            width: range.end - range.start,
            x_sample: distributions::Range::new(range.start, range.end),
        }
    }
}

impl<F, X, Y> Sample<<Y as ops::Mul<X>>::Output> for Integrate<F, X, Y>
where
    F: FnMut(X) -> Y,
    X: Copy + SampleRange + PartialOrd + ops::Sub<Output = X>,
    Y: ops::Mul<X>,
{
    fn sample<R: Rng>(&mut self, rng: &mut R) -> <Y as ops::Mul<X>>::Output {
        let x = self.x_sample.sample(rng);
        (self.func)(x) * self.width
    }
}

impl<F, X, Y> IndependentSample<<Y as ops::Mul<X>>::Output> for Integrate<F, X, Y>
where
    F: Fn(X) -> Y,
    X: Copy + SampleRange + PartialOrd + ops::Sub<Output = X>,
    Y: ops::Mul<X>,
{
    fn ind_sample<R: Rng>(&self, rng: &mut R) -> <Y as ops::Mul<X>>::Output {
        let x = self.x_sample.ind_sample(rng);
        (self.func)(x) * self.width
    }
}


/// Integrates a function `f(x)` in a given `range`.
///
/// The argument `sample_size` specifies how many iterations should be
/// done to get the integration result. `rng` is used as a source of
/// randomness.
///
/// The returned `Statistics` object provides the integration result
/// via its `mean()` method. The integration precision is given by the
/// method `error_of_mean()`.
pub fn integrate<F, X, Y, R>(
    f: F,
    range: ops::Range<X>,
    sample_size: usize,
    rng: &mut R,
) -> Statistics<Y::Output>
where
    F: FnMut(X) -> Y,
    X: Copy + SampleRange + PartialOrd + ops::Sub<Output = X>,
    Y: ops::Mul<X>,
    Y::Output: Stat,
    R: Rng,
{
    Integrate::new(f, range)
        .into_sample_iter(rng)
        .take(sample_size)
        .collect()
}
