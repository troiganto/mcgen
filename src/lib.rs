extern crate num;
extern crate rand;
extern crate itertools;

pub mod time;

pub mod statistics;
pub mod integrate;
pub mod sample;

pub use statistics::{Statistics, print_stats_and_time};
pub use sample::Sample;
pub use integrate::integrate;
