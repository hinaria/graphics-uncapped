use {
    std::collections::VecDeque,
    std::time::Duration,
    std::time::Instant,
    std::sync::Arc,
    std::sync::atomic::AtomicUsize,
    std::sync::atomic::Ordering,
};



pub struct FpsClock {
    queue:  VecDeque<Instant>,
    frames: Arc<AtomicUsize>,
}

impl FpsClock {
    pub fn new() -> FpsClock {
        FpsClock {
            queue:  VecDeque::with_capacity(1024),
            frames: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn update(&mut self) {
        let now      = Instant::now();
        let previous = now - Duration::from_secs(1);

        self.queue.push_back(now);


        while self.queue.front().map(|x| *x < previous).unwrap_or(false) {
            self.queue.pop_front();
        }


        self.frames.store(self.queue.len(), Ordering::Relaxed);
    }

    pub fn client(&self) -> FpsClockClient {
        FpsClockClient {
            frames: self.frames.clone(),
        }
    }
}



pub struct FpsClockClient {
    frames: Arc<AtomicUsize>,
}

impl FpsClockClient {
    pub fn fps(&self) -> usize {
        self.frames.load(Ordering::Relaxed)
    }
}