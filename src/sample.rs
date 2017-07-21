use rand::Rng;
use rand::distributions::Sample;


/// Extension trait that allows conversion to `SampleIter`.
///
/// Everything that implements `Sample` can be converted to
/// `SampleIter`.
pub trait IntoSampleIter<Sup>: Sized + Sample<Sup> {
    /// Performs the conversion.
    fn into_sample_iter<'a, R>(self, rng: &'a mut R) -> SampleIter<'a, Sup, Self, R>
    where
        R: 'a + Rng,
    {
        SampleIter::new(self, rng)
    }
}

impl<S: Sample<Sup>, Sup> IntoSampleIter<Sup> for S {}


/// `Iterator` wrapper type around probability distributions.
///
/// This iterator wraps up a random distribution that implements
/// `Sample` with a source of randomness. Together, this creates an
/// infinite iterator sampling from the given distribution.
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
    /// Wraps up a random distribution with a source of randomness.
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

    /// Samples a new value from the wrapped random distribution.
    ///
    /// This never returns `None`.
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.sample.sample(self.rng))
    }
}
