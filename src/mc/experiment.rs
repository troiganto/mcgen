use rand::{Rng, thread_rng};

use super::{Point, Material, Event, ParticleResult};
use super::particle::Photon;


pub struct Source {
    location: Point,
    energy: f64,
}

impl Source {
    pub fn new(location: Point, energy: f64) -> Self {
        Source { location, energy }
    }

    pub fn emit_photon<R: Rng>(&self, rng: &mut R) -> Photon {
        Photon::emit_at(self.location.clone(), self.energy, rng)
    }
}


pub trait Experiment {
    fn get_source(&self) -> &Source;

    fn x_start(&self) -> f64;

    fn get_material(&self, location: &Point) -> Material;

    fn gen_free_path<R: Rng>(&self, material: Material, energy: f64, rng: &mut R) -> f64;

    fn gen_event<R: Rng>(&self, material: Material, energy: f64, rng: &mut R) -> Event;

    fn gen_coherent_scatter<R: Rng>(&self, material: Material, energy: f64, rng: &mut R) -> f64;

    fn gen_incoherent_scatter<R: Rng>(
        &self,
        material: Material,
        energy: f64,
        rng: &mut R,
    ) -> (f64, f64);

    fn next_detected_photon(&self) -> Photon {
        let source = self.get_source();
        let mut rng = thread_rng();
        let mut result;
        loop {
            // Get a photon.
            let mut photon = source.emit_photon(&mut rng);

            // Make sure it's headed towards the experiment.
            if photon.go_to_x(self.x_start()).is_err() {
                continue;
            }

            // Propagate it until it hits the detector or gets absorbed.
            loop {
                // Move the particle. If it leaves the experiment, stop.
                let material = self.get_material(photon.location());
                let scale = self.gen_free_path(material, photon.energy(), &mut rng);
                photon.step(scale).expect("`scale` cannot be negative");
                if photon.location().x() < self.x_start() {
                    result = ParticleResult::Lost;
                    break;
                }

                // Find the next interaction at the new location.
                let material = self.get_material(photon.location());
                let event = self.gen_event(material, photon.energy(), &mut rng);
                match event {
                    Event::Nothing => {},
                    Event::Absorbed => {
                        result = match material {
                            Material::Detector => ParticleResult::Detected,
                            _ => ParticleResult::Lost,
                        };
                        break;
                    },
                    Event::CoherentScatter => {
                        let angle = self.gen_coherent_scatter(material, photon.energy(), &mut rng);
                        photon.direction_mut().rotate(angle);
                    },
                    Event::IncoherentScatter => {
                        let (angle, energy) =
                            self.gen_incoherent_scatter(material, photon.energy(), &mut rng);
                        photon.direction_mut().rotate(angle);
                        photon.set_energy(energy);
                    },
                }
            }

            // After propagation is done, check if the particle was
            // detected.
            match result {
                ParticleResult::Lost => continue,
                ParticleResult::Detected => return photon,
            }
        }
    }
}
