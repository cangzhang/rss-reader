use std::sync::{mpsc, Arc, Mutex};

use crate::worker::Worker;

pub type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    pub workers: Vec<crate::worker::Worker>,
    pub sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&rx)));
        }

        Self {
            workers,
            sender: Some(tx),
        }
    }

    pub fn exec<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(th) = worker.thread.take() {
                th.join().unwrap();
            }
        }
    }
}
