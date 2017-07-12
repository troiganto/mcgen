use std::path::Path;

use csv;

use rand::distributions;

use dimensioned::si::*;
use dimensioned::Dimensionless;
use dimensioned::f64prefixes::*;

use super::Sample;
use super::Function;


/// Common trait of all angular spectral cross-sections.
pub trait CrossSection {
    /// Evaluates the cross-section in an infinitesimal phase-space
    /// volume around the given `energy` and `mu`.
    ///
    /// `mu` is `theta.cos()`, where `theta` is the angle between the
    /// particle's original and new direction. `theta` lies between
    /// 0° and 180°, so `mu` lies between +1 and –1.
    fn eval(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Meter2<f64>;

    /// Returns the maximum angular spectral cross-section for a given
    /// energy.
    ///
    /// This is necessary for the rejection method to work.
    fn max(&self, energy: Joule<f64>) -> Meter2<f64>;
}


/// A coherent scattering cross-section that depends on an atomic form
/// factor.
#[derive(Debug)]
pub struct CoherentCrossSection {
    form_factor: Function<f64>,
}

impl CoherentCrossSection {
    pub fn new<P>(form_factor_file: P) -> csv::Result<Self>
    where
        P: AsRef<Path>,
    {
        let form_factor = Function::from_file(form_factor_file)?;
        let result = CoherentCrossSection { form_factor };
        Ok(result)
    }

    pub fn form_factor(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Unitless<f64> {
        let x = get_x(energy, mu) / (KILO * EV);
        let form_factor = self.form_factor.call(*x.value());
        Unitless::new(form_factor)
    }
}

impl CrossSection for CoherentCrossSection {
    fn eval(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Meter2<f64> {
        let form_factor = self.form_factor(energy, mu);
        r_e() * r_e() * (1.0 + mu * mu) / 2.0 * form_factor * form_factor
    }

    fn max(&self, energy: Joule<f64>) -> Meter2<f64> {
        self.eval(energy, Unitless::new(1.0))
    }
}


/// An incoherent scattering cross-section that depends on an
/// incoherent scattering function.
#[derive(Debug)]
pub struct IncoherentCrossSection {
    scattering_function: Function<f64>,
}

impl IncoherentCrossSection {
    pub fn new<P>(scattering_function_file: P) -> csv::Result<Self>
    where
        P: AsRef<Path>,
    {
        let scattering_function = Function::from_file(scattering_function_file)?;
        let result = IncoherentCrossSection { scattering_function };
        Ok(result)
    }

    pub fn scattering_function(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Unitless<f64> {
        let x = get_x(energy, mu) / (KILO * EV);
        let scattering_function = self.scattering_function.call(*x.value());
        Unitless::new(scattering_function)
    }

    pub fn klein_nishina(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Meter2<f64> {
        let kappa = energy / (M_E * C0 * C0);
        let kappa_antimu = kappa * (1.0 - mu);
        let alpha_func = 1.0 / (1.0 + kappa_antimu);
        r_e() * r_e() / 2.0 * alpha_func * alpha_func * (alpha_func + kappa_antimu + mu * mu)
    }
}

impl CrossSection for IncoherentCrossSection {
    fn eval(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Meter2<f64> {
        self.klein_nishina(energy, mu) * self.scattering_function(energy, mu)
    }

    fn max(&self, energy: Joule<f64>) -> Meter2<f64> {
        self.klein_nishina(energy, Unitless::new(1.0))
    }
}


/// Iterator that samples `mu` from a cross-section distribution using
/// the rejection method.
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


/// Returns the classical electron radius.
fn r_e() -> Meter<f64> {
    let alpha = Unitless::new(1.0 / 137.0);
    R_BOHR * alpha * alpha
}


/// Calculates the parameter `x = E * sin(theta/2)`.
fn get_x(energy: Joule<f64>, mu: Unitless<f64>) -> Joule<f64> {
    let angle = mu.acos();
    energy * (angle / 2.0).sin()
}