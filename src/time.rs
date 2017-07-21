use std::time::{Instant, Duration};

use dimensioned::si::*;
use dimensioned::f64prefixes::*;


/// Measures and returns the time it takes to execute a function.
pub fn measure<F: FnOnce()>(func: F) -> Duration {
    let start = Instant::now();
    func();
    let end = Instant::now();
    end.duration_since(start)
}


/// Wrapper around `measure` that returns the time in seconds.
pub fn measure_seconds<F: FnOnce()>(func: F) -> Second<f64> {
    let duration = measure(func);
    let secs = duration.as_secs() as f64;
    let nanosecs = duration.subsec_nanos() as f64;
    secs * S + nanosecs * NANO * S
}
