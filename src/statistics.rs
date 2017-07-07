use std::iter::FromIterator;
use std::fmt::{self, Display};

use num::Float;


#[derive(Clone, Debug)]
pub struct Statistics<F> {
    count: usize,
    mean: F,
    sum_of_squares: F,
}

impl<F: Float> Statistics<F> {
    pub fn new() -> Self {
        Statistics {
            count: 0,
            mean: F::zero(),
            sum_of_squares: F::zero(),
        }
    }

    pub fn from_sample<I>(sample: I) -> Self
    where
        I: IntoIterator<Item = F>,
    {
        let mut result = Self::new();
        result.add_sample(sample);
        result
    }

    pub fn add_sample<I>(&mut self, sample: I)
    where
        I: IntoIterator<Item = F>,
    {
        for point in sample.into_iter() {
            self.add(point);
        }
    }

    pub fn add(&mut self, sample: F) {
        self.count += 1;
        let delta = sample - self.mean;
        self.mean = self.mean + delta / Self::to_float(self.count);
        let delta_2 = sample - self.mean;
        self.sum_of_squares = self.sum_of_squares + delta*delta_2;
    }

    pub fn mean(&self) -> F {
        self.mean
    }

    pub fn variance(&self) -> F {
        if self.count < 2 {
            F::nan()
        } else {
            self.sum_of_squares / Self::to_float(self.count - 1)
        }
    }

    pub fn standard_deviation(&self) -> F {
        self.variance().sqrt()
    }

    pub fn error_of_mean(&self) -> F {
        (self.variance() / Self::to_float(self.count)).sqrt()
    }

    fn to_float(n: usize) -> F {
        F::from(n).expect("cast usize to Float")
    }
}

impl<F: Float> Default for Statistics<F> {
    fn default() -> Self {
        Statistics::<F>::new()
    }
}

impl<F: Float + Display> Display for Statistics<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Mean: {0:.5} ± {1:.5}\nStandard deviation: {2:.5}",
            self.mean(),
            self.error_of_mean(),
            self.standard_deviation()
        )
    }
}

impl<F: Float> FromIterator<F> for Statistics<F> {
    fn from_iter<T>(iter: T) -> Self
    where T: IntoIterator<Item=F> {
        Statistics::from_sample(iter)
    }
}
