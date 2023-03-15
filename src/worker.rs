use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::thread_pool::Job;

pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            while let Ok(job) = rx.lock().unwrap().recv() {
                println!("Worker {id} got a job, executing....");
                job();
            }
        });

        Worker { id, thread }
    }
}
