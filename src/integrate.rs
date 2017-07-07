use std::ops::Range;

use num::Float;

use rand;
use rand::distributions::range::SampleRange;

use super::Statistics;
use super::Sample;


pub fn integrate<F, X>(mut f: F, range: Range<X>, sample_size: usize) -> Statistics<X>
where
    F: FnMut(X) -> X,
    X: Float + SampleRange,
{
    let width = range.end - range.start;
    let dist = rand::distributions::Range::new(range.start, range.end);
    let sample = Sample::with_size(dist, sample_size).map(|x| f(x) * width);
    Statistics::from_sample(sample)
}
