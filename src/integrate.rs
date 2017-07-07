use std::ops::Range;

use num::Float;

use rand::distributions;
use rand::distributions::range::SampleRange;

use super::Statistics;
use super::Sample;


pub fn integrate<F, X>(mut f: F, range: Range<X>, sample_size: usize) -> Statistics<X>
where
    F: FnMut(X) -> X,
    X: Float + SampleRange,
{
    let width = range.end - range.start;
    Sample::new(distributions::Range::new(range.start, range.end))
        .take(sample_size)
        .map(|x| f(x) * width)
        .collect()
}
