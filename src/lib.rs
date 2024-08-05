use std::{
    thread, 
    sync::{mpsc, Arc, Mutex}
};

pub struct ThreadPool {
    #[allow(dead_code)]
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    /// 
    /// The size is the number of threads in the pool.
    /// 
    /// # Panics
    /// 
    /// The 'new' function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }
    
    pub fn execute<F>(&self, f: F) 
    where 
    F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

#[allow(dead_code)]
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // Call lock on receiver to acquire mutex (only 1 thread can acquire at once)
            // then call recv(), which gets a Job from the from the channel, this blocks,
            // so will wait until a job comes through.
            // N.B. the mutex automatically unlocks again after the MutexGuard<> returned
            // by Mutex.lock() goes out of scope, in this case we call recv() immediately
            // on the wrapped Receiver & the guard is returned before processing the job.
            // N.B. above works as 'let' drops any temporary value used in the expression
            // on the right hand side immediately when the statement ends. If we had used
            // 'if let' or 'while let' or 'match', temporary values would only be dropped
            // at the end of the associated block, keeping the Mutex locked.
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing.");
            job();
            println!("Worker {id} finished job; returning to pool");
        });

        Worker { id, thread }
    }
}