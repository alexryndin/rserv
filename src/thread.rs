use std::error::Error;
use std::fmt;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send>; //{

//}

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    queue: mpsc::Sender<Message>,
}

#[derive(Debug)]
pub struct PoolCreationError;

impl Error for PoolCreationError {}

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PoolCreationError, number of threads cannot be less than 1"
        )
    }
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The num is the number of thread in the pool
    ///
    pub fn new(num: usize) -> Result<ThreadPool, PoolCreationError> {
        if num == 0 {
            return Err(PoolCreationError);
        };

        let mut workers = Vec::with_capacity(num);

        let (tx, rx) = mpsc::channel();

        let rxa = Arc::new(Mutex::new(rx));

        for id in 0..num {
            workers.push(Worker::new(id, Arc::clone(&rxa)));
        }

        Ok(ThreadPool { workers, queue: tx })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.queue.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {

        for _ in &mut self.workers {
            self.queue.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}

impl Worker {
    /// Create a new Worker
    ///
    /// The id is the id of the new worker
    ///
    fn new(id: usize, queue: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let handle = thread::spawn(move || loop {
            match queue.lock().unwrap().recv().unwrap() {
                Message::NewJob(job) => {
                    println!("worker {} got job", id);
                    job();
                }
                Message::Terminate =>{
                    println!("worker {} terminating...", id);
                    break;
                },
            };
        });

        Worker {
            id,
            handle: Some(handle),
        }
    }
}
