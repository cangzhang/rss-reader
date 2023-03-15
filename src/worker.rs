use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::thread_pool::Job;

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = rx.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job, executing....");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected, shutting down");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
