use std::{ops::Mul, time::Duration};

use instant::Instant;

use crate::config::TIME;

// Time the duration once every n iterations.
const TIME_EACH: usize = 64;

pub struct Timer(Option<(usize, Instant)>);

impl Timer {
    #[inline]
    pub fn once() -> Timer {
        Self::each(1, &mut 0)
    }
    #[inline]
    pub fn new(cnt: &mut usize) -> Timer {
        Self::each(TIME_EACH, cnt)
    }
    #[inline]
    pub fn each(each: usize, cnt: &mut usize) -> Timer {
        Timer(if TIME {
            *cnt += 1;
            if *cnt % each == 0 {
                Some((each, instant::Instant::now()))
            } else {
                None
            }
        } else {
            None
        })
    }
    #[inline]
    pub fn end(self, accumulator: &mut Duration) -> Duration {
        if let Timer(Some((each, start_time))) = self {
            let t = start_time.elapsed().mul(each as u32);
            *accumulator += t;
            t
        } else {
            Duration::default()
        }
    }
}

#[test]
fn test_time_each() {
    use std::thread::sleep;
    let mut f = Duration::default();
    let mut cnt = 0;
    let mu = 200;
    for _ in 0..1000000 / mu {
        let t = Timer::new(&mut cnt);
        sleep(instant::Duration::from_micros(mu));
        t.end(&mut f);
    }
    eprintln!("total time: {f:?}");
}

#[test]
fn test_time_speed() {
    let s = instant::Instant::now();
    let mut cnt = 0;
    let mut f = Duration::default();
    for _ in 0..10000000 {
        let t = Timer::each(1, &mut cnt);
        t.end(&mut f);
    }
    let t = s.elapsed().as_secs_f64();
    eprintln!("elapsed:    {t}");
    eprintln!("total time: {f:?}");
}
