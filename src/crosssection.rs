use rand::distributions;

use dimensioned::si::*;
use dimensioned::Dimensionless;

use super::Sample;


pub trait CrossSection {
    fn eval(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Meter2<f64>;
    fn max(&self, energy: Joule<f64>) -> Meter2<f64>;
}


pub struct RejectionSampler<XS: CrossSection> {
    dist: XS,
    energy: Joule<f64>,
    mu_sample: Sample<f64, distributions::Range<f64>>,
    xsection_sample: Sample<f64, distributions::Range<f64>>,
}

impl<XS: CrossSection> RejectionSampler<XS> {
    pub fn new(dist: XS, energy: Joule<f64>) -> Self {
        let mu_dist = distributions::Range::new(-1.0, 1.0);
        let mu_sample = Sample::from(mu_dist);

        let max_xsection = dist.max(energy) / M2;
        let xsection_dist = distributions::Range::new(-0.0, *max_xsection.value());
        let xsection_sample = Sample::from(xsection_dist);

        RejectionSampler {
            dist,
            energy,
            mu_sample,
            xsection_sample,
        }
    }

    pub fn get_mu(&mut self) -> Unitless<f64> {
        loop {
            let random_mu = Unitless::new(self.mu_sample.get_one());
            let random_xsection = self.xsection_sample.get_one() * M2;
            let max_xsection = self.dist.eval(self.energy, random_mu);
            if random_xsection < max_xsection {
                return random_mu;
            }
        }
    }
}

impl<XS: CrossSection> Iterator for RejectionSampler<XS> {
    type Item = Unitless<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.get_mu())
    }
}
