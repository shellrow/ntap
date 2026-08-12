pub mod tree;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub fn wait_for_stop(stop: &AtomicBool, duration: Duration) -> bool {
    let deadline = Instant::now() + duration;
    while !stop.load(Ordering::Acquire) {
        let now = Instant::now();
        if now >= deadline {
            return false;
        }
        std::thread::sleep((deadline - now).min(Duration::from_millis(50)));
    }
    true
}

pub struct WorkerSet {
    stop: Arc<AtomicBool>,
    handles: Vec<JoinHandle<()>>,
}

impl WorkerSet {
    pub fn new(stop: Arc<AtomicBool>) -> Self {
        Self {
            stop,
            handles: Vec::new(),
        }
    }

    pub fn spawn<F>(&mut self, name: String, worker: F) -> std::io::Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        self.handles
            .push(thread::Builder::new().name(name).spawn(worker)?);
        Ok(())
    }

    pub fn worker_count(&self) -> usize {
        self.handles.len()
    }

    pub fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Release);
        for handle in self.handles.drain(..) {
            if handle.join().is_err() {
                tracing::error!("a worker thread panicked during shutdown");
            }
        }
    }
}

impl Drop for WorkerSet {
    fn drop(&mut self) {
        self.shutdown();
    }
}
