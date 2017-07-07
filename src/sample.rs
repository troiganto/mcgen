use std::iter;

use rand::{self, ThreadRng};
use rand::distributions::IndependentSample;


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
}

impl<F, D> Iterator for Sample<F, D>
where
    D: IndependentSample<F>,
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.dist.ind_sample(&mut self.rng))
    }
}
