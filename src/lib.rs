use std::{sync::{mpsc, Arc, Mutex}, thread};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}


impl ThreadPool {
    ///Create a new ThreadPool
    /// 
    /// The size is the number of threads in the pool
    /// 
    /// # Panics
    /// 
    /// The `new` function will panic if the size is zero or less.
    pub fn new(size: usize) -> ThreadPool{
        assert!(size > 0);
        let (sender, receiver) 
            = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

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

        match self.sender.send(Message::NewJob(job)){
            Ok(_) =>  println!("Sending message"),
            Err(e) => eprintln!("Error sending message: {}", e)
        };
    }
}
impl Drop for ThreadPool{
    fn drop(&mut self){
        println!("Sending terminate message");

        for _ in &self.workers{
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down workers");

        for worker in &mut self.workers{
            println!("Shutting dowm worker {}", worker.id);
        
            if let Some(thread) = worker.thread.take(){
                thread.join().unwrap();
            }
        }
    }
}
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(id: usize, receiver: Arc<std::sync::Mutex<mpsc::Receiver<Message>>>) -> Worker{
        let thread = thread::spawn(move || loop {
            let message = match receiver.lock() {
                Ok(lock) => match lock.recv() {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("Worker {} failed to receive job: {}", id, e);
                        break;
                    }
                },
                Err(e) => {
                    eprintln!("Worker {} failed to lock receiver: {}", id, e);
                    break;
                }
            };

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} go a job", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate", id);
                    break;
                }
            }
            
        });

        Worker { id, thread: Some(thread) }
    }
}