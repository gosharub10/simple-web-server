use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    ///Creat a new ThreadPool
    ///
    /// The size is a number of threads in the pool
    ///
    /// #Panic
    ///
    /// The 'new' function will panic if the size is zero
    pub fn new(_size: usize) -> ThreadPool {
        assert!(_size > 0);
        let (sender, receiver) = mpsc::channel();

        let mut workers = Vec::with_capacity(_size);

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0.._size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, _f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(_f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job");
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
