extern crate num;
extern crate rand;
extern crate itertools;
extern crate csv;
extern crate serde;

pub mod time;

pub mod statistics;
pub mod integrate;
pub mod sample;
pub mod function;

pub use statistics::{Statistics, print_stats_and_time};
pub use sample::Sample;
pub use integrate::integrate;
pub use function::Function;
