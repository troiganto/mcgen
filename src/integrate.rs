use std::ops::Range;

use num::Float;

use rand::distributions;
use rand::distributions::range::SampleRange;

use super::Statistics;
use super::Sample;


pub struct Integrate<F, X>
where
    F: FnMut(X) -> X,
    X: Float + SampleRange,
{
    func: F,
    width: X,
    sample: Sample<X, distributions::Range<X>>,
}

impl<F, X> Integrate<F, X>
where
    F: FnMut(X) -> X,
    X: Float + SampleRange,
{
    pub fn new(f: F, range: Range<X>) -> Self {
        let dist = distributions::Range::new(range.start, range.end);
        Integrate {
            func: f,
            width: range.end - range.start,
            sample: Sample::new(dist),
        }
    }
}

impl<F, X> Iterator for Integrate<F, X>
where
    F: FnMut(X) -> X,
    X: Float + SampleRange,
{
    type Item = X;

    fn next(&mut self) -> Option<Self::Item> {
        self.sample.next().map(|x| (self.func)(x) * self.width)
    }
}


pub fn integrate<F, X>(f: F, range: Range<X>, sample_size: usize) -> Statistics<X>
where
    F: FnMut(X) -> X,
    X: Float + SampleRange,
{
    Integrate::new(f, range).take(sample_size).collect()
}
