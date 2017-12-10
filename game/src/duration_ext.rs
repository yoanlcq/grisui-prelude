use std::time::Duration;

pub trait DurationExt {
    fn to_f64_seconds(&self) -> f64;
}

impl DurationExt for Duration {
    fn to_f64_seconds(&self) -> f64 {
        self.as_secs() as f64 + (self.subsec_nanos() as f64 / 1_000_000_000_f64)
    }
}
