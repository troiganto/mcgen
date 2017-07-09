extern crate csv;
extern crate mcgen;
extern crate dimensioned;

use std::path::Path;
use dimensioned::Dimensionless;
use dimensioned::si::*;
use dimensioned::f64prefixes::*;


struct CoherentCrossSection {
    form_factor: mcgen::Function<f64>,
}

impl CoherentCrossSection {
    fn new<P>(form_factor_file: P) -> csv::Result<Self> where P: AsRef<Path> {
        let form_factor = mcgen::Function::from_file(form_factor_file, b'\t', 2)?;
        let result = CoherentCrossSection { form_factor };
        Ok(result)
    }

    fn call(&self, energy: Joule<f64>, angle: Unitless<f64>) -> Meter2<f64> {
        let mu = angle.cos();
        let x_kev = energy * (angle / 2.0).sin() / (KILO * EV);
        let form_factor = self.form_factor.call(*x_kev.value());

        let alpha = Unitless::new(1.0 / 137.0);
        let r_elec = R_BOHR * alpha * alpha;

        r_elec * r_elec * (1.0 + mu*mu)/2.0 * form_factor*form_factor
    }
}


fn main() {
    let xsection = CoherentCrossSection::new("../data/AFF.dat").unwrap();
    println!("{:.2e}", xsection.call(1.0 * MEGA * EV, Unitless::new(0.0)));
}
