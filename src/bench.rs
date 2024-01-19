use crate::fmt::readable_amount;
use crate::mem::override_lifetime;
use log::{info, warn};
use std::process::exit;
use std::sync::{Barrier, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

pub struct Bench {
    started: bool,

    concurrency: usize,

    finished_counts: Vec<Mutex<u64>>,

    // test
    start_bar: Barrier,
    start: Instant,
}

pub trait TestFn: Fn(usize, &Bench) + Send + 'static + Clone {}
impl<T: Fn(usize, &Bench) + Send + 'static + Clone> TestFn for T {}

impl Bench {
    pub fn new_with_max_threads() -> Self {
        let concurrency = std::thread::available_parallelism().unwrap().get();
        Bench::new(concurrency)
    }

    pub fn new(concurrency: usize) -> Self {
        crate::log::init();
        let mut finished_counts = Vec::with_capacity(concurrency);
        for _ in 0..concurrency {
            finished_counts.push(Mutex::new(0));
        }

        Bench {
            started: false,
            concurrency,
            finished_counts,
            start_bar: Barrier::new(concurrency + 1),
            start: Instant::now(),
        }
    }

    pub fn run_with<R>(mut self, run: R)
    where
        R: TestFn,
    {
        info!("Starting bench with concurrency {}", self.concurrency);
        for i in 0..self.concurrency {
            let rself = override_lifetime(&self);
            let c_run = run.clone();
            spawn(move || {
                c_run(i, rself);
            });
        }
        self.set_ctrlc();

        self.start_bar.wait();
        self.started = true;
        self.start = Instant::now();
        let mut total: u64 = 0;
        loop {
            sleep(Duration::from_secs(1));

            let mut this_total = 0;
            for i in 0..self.concurrency {
                this_total += *self.finished_counts[i].lock().unwrap();
            }
            let delta = this_total - total;
            total = this_total;

            info!(
                "Throughput(op/s): last {}, ave {}",
                readable_amount(delta as f64),
                readable_amount(total as f64 / (self.start.elapsed().as_millis() as f64 / 1000.0)),
            );
        }
    }

    //for task

    pub fn inc_op(&self, i: usize, count: u64) {
        *self.finished_counts[i].lock().unwrap() += count;
    }

    pub fn init_ok(&self) {
        self.start_bar.wait();
    }

    fn set_ctrlc(&self) {
        let c_self = override_lifetime(self);
        ctrlc::set_handler(move || {
            // warn!("Ctrl-C received, exiting");
            if c_self.started {
                let mut this_total = 0;
                for i in 0..c_self.concurrency {
                    this_total += *c_self.finished_counts[i].lock().unwrap();
                }
                info!(
                    "Time {:#?}, Concurrency {}, Throughput {}ops",
                    c_self.start.elapsed(),
                    c_self.concurrency,
                    readable_amount(
                        this_total as f64 / (c_self.start.elapsed().as_millis() as f64 / 1000.0)
                    )
                )
            } else {
                warn!("Bench not started, exiting")
            }
            exit(0);
        })
        .expect("Error setting Ctrl-C handler");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::time::Instant;

    #[test]
    fn test_bench() {
        let bench = Bench::new(100);
        let counter = Arc::new(AtomicU64::new(0));
        let c_counter = counter.clone();
        bench.run_with(move |i, bench| {
            bench.start_bar.wait();
            loop {
                c_counter.fetch_add(1, Ordering::SeqCst);
                bench.inc_op(i, 1);
            }
        });
    }
}
