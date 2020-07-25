// experiment for stopwatch - measuring time

use std::time::{Duration, Instant};

pub fn main() {
    let mut s:Stopwatch = newStopwatch();
    s.start();
    // DO STUFF HERE
    let diff = s.stop();

    println!("{:?}", diff);
}

pub struct Stopwatch {
    pub last: Instant, // last time
}

pub fn newStopwatch() -> Stopwatch {
    Stopwatch{last:Instant::now()}
}

impl Stopwatch {
    pub fn start(&mut self) {
        self.last = Instant::now();
    }

    pub fn stop(self) -> Duration {
        let now = Instant::now();
        now.duration_since(self.last)
    }
}
