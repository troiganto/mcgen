use rand::Rng;
use rand::distributions::Sample;


pub trait IntoSampleIter<Sup>: Sized + Sample<Sup> {
    fn into_sample_iter<'a, R>(self, rng: &'a mut R) -> SampleIter<'a, Sup, Self, R>
    where
        R: 'a + Rng,
    {
        SampleIter::new(self, rng)
    }
}

impl<S, Sup> IntoSampleIter<Sup> for S
where
    S: Sample<Sup>,
{
}


/// `Iterator` wrapper type around probability distributions.
pub struct SampleIter<'a, Sup, S, R>
where
    S: Sample<Sup>,
    R: 'a + Rng,
{
    rng: &'a mut R,
    sample: S,
    _dummy: ::std::marker::PhantomData<Sup>,
}

impl<'a, Sup, S, R> SampleIter<'a, Sup, S, R>
where
    S: Sample<Sup>,
    R: 'a + Rng,
{
    pub fn new(sample: S, rng: &'a mut R) -> Self {
        SampleIter {
            rng,
            sample,
            _dummy: Default::default(),
        }
    }
}

impl<'a, Sup, S, R> Iterator for SampleIter<'a, Sup, S, R>
where
    S: Sample<Sup>,
    R: 'a + Rng,
{
    type Item = Sup;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.sample.sample(self.rng))
    }
}
