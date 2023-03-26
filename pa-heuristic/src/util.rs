use instant::Instant;

// Time the duration once every `TIME_EACH` iterations.
const TIME_EACH: usize = 1024;

pub struct Timer(Option<Instant>);

impl Timer {
    pub fn new(cnt: &mut usize) -> Timer {
        *cnt += 1;
        if *cnt % TIME_EACH == 0 {
            Timer(Some(instant::Instant::now()))
        } else {
            Timer(None)
        }
    }
    pub fn end(self, accumulator: &mut f64) -> f64 {
        if let Timer(Some(start_time)) = self {
            let t = TIME_EACH as f64 * start_time.elapsed().as_secs_f64();
            *accumulator += t;
            t
        } else {
            0.
        }
    }
}
