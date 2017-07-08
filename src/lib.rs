extern crate num;
extern crate rand;
extern crate itertools;

pub mod time;

mod statistics;
mod integrate;
mod sample;

pub use statistics::Statistics;
pub use sample::Sample;
pub use integrate::integrate;
