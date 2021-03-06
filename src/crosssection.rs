use std::path::Path;

use csv;

use rand::Rng;
use rand::distributions::{self, Sample, IndependentSample};

use dimensioned::si::*;
use dimensioned::Dimensionless;
use dimensioned::f64prefixes::*;

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
    form_factor: Function<Joule<f64>, Unitless<f64>>,
}

impl CoherentCrossSection {
    /// Creates a cross-section with the atomic form factor from the
    /// given file.
    pub fn new<P>(form_factor_file: P) -> csv::Result<Self>
    where
        P: AsRef<Path>,
    {
        let form_factor = Function::<f64>::from_file(form_factor_file)?
            .scale(KILO * EV, Unitless::new(1.0));
        let result = CoherentCrossSection { form_factor };
        Ok(result)
    }

    /// Evaluates the atomic form factor at the given energy and `mu`.
    ///
    /// `mu` is `cos(theta)`, where `theta` is the polar angle.
    pub fn form_factor(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Unitless<f64> {
        self.form_factor.call(get_x(energy, mu))
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
    scattering_function: Function<Joule<f64>, Unitless<f64>>,
}

impl IncoherentCrossSection {
    /// Creates a cross-section with the atomic form factor from the
    /// given file.
    pub fn new<P>(scattering_function_file: P) -> csv::Result<Self>
    where
        P: AsRef<Path>,
    {
        let scattering_function = Function::<f64>::from_file(scattering_function_file)?
            .scale(KILO * EV, Unitless::new(1.0));
        let result = IncoherentCrossSection { scattering_function };
        Ok(result)
    }

    /// Calculates the resulting energy of the Compton formula.
    ///
    /// `mu` is `cos(theta)`, where `theta` is the polar angle. `energy`
    /// is the incident particle's energy, the result is the energy
    /// after
    /// the scattering process.
    pub fn compton_scatter(energy: Joule<f64>, mu: Unitless<f64>) -> Joule<f64> {
        let kappa = energy / (M_E * C0 * C0);
        let kappa_antimu = kappa * (1.0 - mu);
        energy / (1.0 + kappa_antimu)
    }

    /// Evaluates the incoherent scattering function at the given
    /// energy and `mu`.
    ///
    /// `mu` is `cos(theta)`, where `theta` is the polar angle.
    pub fn scattering_function(&self, energy: Joule<f64>, mu: Unitless<f64>) -> Unitless<f64> {
        self.scattering_function.call(get_x(energy, mu))
    }

    /// Calculates the Klein–Nishina cross-section at the given energy
    /// and `mu`.
    ///
    /// `mu` is `cos(theta)`, where `theta` is the polar angle.
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
        let max_scatter = *self.scattering_function.max();
        self.klein_nishina(energy, Unitless::new(1.0)) * max_scatter
    }
}


/// Iterator that samples `mu` from a cross-section distribution using
/// the rejection method.
pub struct RejectionSampler<'a, XS>
where
    XS: 'a + CrossSection,
{
    dist: &'a XS,
    energy: Joule<f64>,
    mu_dist: distributions::Range<f64>,
    xsection_dist: distributions::Range<f64>,
}

impl<'a, XS> RejectionSampler<'a, XS>
where
    XS: 'a + CrossSection,
{
    /// Creates a new sampler, sampling the cross-section at the given,
    /// fixed energy.
    pub fn new(dist: &'a XS, energy: Joule<f64>) -> Self {
        let max_xsection = dist.max(energy) / M2;
        let xsection_dist = distributions::Range::new(-0.0, *max_xsection.value());
        let mu_dist = distributions::Range::new(-1.0, 1.0);

        RejectionSampler {
            dist,
            energy,
            mu_dist,
            xsection_dist,
        }
    }

    /// Produces a new `mu` value.
    pub fn gen_mu<R: Rng>(&self, rng: &mut R) -> Unitless<f64> {
        loop {
            let random_mu = Unitless::new(self.mu_dist.ind_sample(rng));
            let random_xsection = self.xsection_dist.ind_sample(rng) * M2;
            let max_xsection = self.dist.eval(self.energy, random_mu);
            if random_xsection < max_xsection {
                return random_mu;
            }
        }
    }
}

impl<'a, XS> Sample<Unitless<f64>> for RejectionSampler<'a, XS>
where
    XS: 'a + CrossSection,
{
    fn sample<R: Rng>(&mut self, rng: &mut R) -> Unitless<f64> {
        self.gen_mu(rng)
    }
}

impl<'a, XS> IndependentSample<Unitless<f64>> for RejectionSampler<'a, XS>
where
    XS: 'a + CrossSection,
{
    fn ind_sample<R: Rng>(&self, rng: &mut R) -> Unitless<f64> {
        self.gen_mu(rng)
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
