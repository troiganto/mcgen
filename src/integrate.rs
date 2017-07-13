use std::ops::Range;

use num::Float;

use rand::Rng;
use rand::distributions::range::SampleRange;
use rand::distributions::{self, Sample, IndependentSample};

use super::{IntoSampleIter, Statistics};


/// Struct for Monte-Carlo integration of 1D real functions.
///
/// This struct is exposed to allow continuous inspection of
/// the integration result and uncertainty. To just get the result
/// immediately, use the function `integrate`.
pub struct Integrate<F, X>
where
    F: FnMut(X) -> X,
    X: Float + SampleRange,
{
    func: F,
    width: X,
    x_sample: distributions::Range<X>,
}

impl<F, X> Integrate<F, X>
where
    F: FnMut(X) -> X,
    X: Float + SampleRange,
{
    pub fn new(f: F, range: Range<X>) -> Self {
        Integrate {
            func: f,
            width: range.end - range.start,
            x_sample: distributions::Range::new(range.start, range.end),
        }
    }
}

impl<F, X> Sample<X> for Integrate<F, X>
where
    F: FnMut(X) -> X,
    X: Float + SampleRange,
{
    fn sample<R: Rng>(&mut self, rng: &mut R) -> X {
        let x = self.x_sample.sample(rng);
        (self.func)(x) * self.width
    }
}

impl<F, X> IndependentSample<X> for Integrate<F, X>
where
    F: Fn(X) -> X,
    X: Float + SampleRange,
{
    fn ind_sample<R: Rng>(&self, rng: &mut R) -> X {
        let x = self.x_sample.ind_sample(rng);
        (self.func)(x) * self.width
    }
}


/// Integrates a function `f(x)` in a given `range`.
///
/// This function integrates via Mone-Carlo methods. `sample_size` is
/// a measure of the integration precision.
pub fn integrate<F, X, R>(f: F, range: Range<X>, sample_size: usize, rng: &mut R) -> Statistics<X>
where
    F: FnMut(X) -> X,
    X: Float + SampleRange,
    R: Rng,
{
    Integrate::new(f, range)
        .into_sample_iter(rng)
        .take(sample_size)
        .collect()
}
