use std::time::{Duration, Instant};

pub struct FPSManager {
    frame_count: u32,
    last_frame_time: Instant,
    last_update_time: Instant,
    pub time_delta: Duration,
}

impl FPSManager {
    pub fn new() -> Self {
        FPSManager {
            frame_count: 0,
            last_frame_time: Instant::now(),
            last_update_time: Instant::now(),
            time_delta: Duration::default(),
        }
    }

    pub fn update<T: FnMut(u32)>(&mut self, mut update_fn: T) {
        self.frame_count += 1;
        let now = Instant::now();
        self.time_delta = now.duration_since(self.last_frame_time);
        if now.duration_since(self.last_update_time) >= Duration::from_secs(1) {
            update_fn(self.frame_count);
            self.frame_count = 0;
            self.last_update_time = now;
        }
        self.last_frame_time = now;
    }
}
