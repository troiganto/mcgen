use std::iter;

use itertools;
use itertools::Itertools;

use rand::{self, ThreadRng};
use rand::distributions::IndependentSample;


/// Helper types for `Sample::batching`.
pub type BatchingFunc<F, D> = for<'r> fn(&'r mut Sample<F, D>) -> Option<(F, F)>;
pub type PointSample<F, D> = itertools::Batching<Sample<F, D>, BatchingFunc<F, D>>;


/// `Iterator` wrapper type around probability distributions.
pub struct Sample<F, D>
where
    D: IndependentSample<F>,
{
    rng: ThreadRng,
    dist: D,
    _dummy: ::std::marker::PhantomData<F>,
}

impl<F, D> Sample<F, D>
where
    D: IndependentSample<F>,
{
    pub fn new(dist: D) -> Self {
        Sample {
            rng: rand::thread_rng(),
            dist: dist,
            _dummy: Default::default(),
        }
    }

    pub fn with_size(dist: D, size: usize) -> iter::Take<Self> {
        Sample::new(dist).take(size)
    }

    pub fn get_one(&mut self) -> F {
        self.dist.ind_sample(&mut self.rng)
    }

    pub fn get_two(&mut self) -> (F, F) {
        (self.get_one(), self.get_one())
    }

    /// Crutch because closure->function pointer coercion is not stable
    /// yet.
    fn get_some_two(&mut self) -> Option<(F, F)> {
        Some(self.get_two())
    }

    pub fn as_points(self) -> PointSample<F, D> {
        self.batching(Self::get_some_two)
    }
}

impl<F, D> From<D> for Sample<F, D>
where
    D: IndependentSample<F>,
{
    fn from(dist: D) -> Self {
        Sample::new(dist)
    }
}

impl<F, D> Iterator for Sample<F, D>
where
    D: IndependentSample<F>,
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.get_one())
    }
}
