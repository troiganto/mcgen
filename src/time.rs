use std::time::{Instant, Duration};


pub fn measure<F: FnOnce()>(func: F) -> Duration {
    let start = Instant::now();
    func();
    let end = Instant::now();
    end.duration_since(start)
}


pub fn measure_seconds<F: FnOnce()>(func: F) -> f64 {
    let duration = measure(func);
    let secs = duration.as_secs() as f64;
    let nanosecs = duration.subsec_nanos() as f64;
    secs + nanosecs/1e9
}
