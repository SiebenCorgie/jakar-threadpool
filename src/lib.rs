use std::thread::*;
use std::sync::mpsc::*;
use std::sync::{Arc, Mutex};

use std::result::Result;

pub mod job;


struct Worker {
    id: u32,
    thread_handle: Option<JoinHandle<()>>,
}

impl Worker{
    ///Starts this worker
    pub fn start(id: u32, reciver: Arc<Mutex<Receiver<job::ThreadJob>>>) -> Self{
        let thread_handle: JoinHandle<()> = spawn(move || {
            //Loop until we recive the ending signal
            let thread_reciver = reciver;
            let thread_id = id;
            loop {
                let job = {
                    let msg_lock = thread_reciver.lock().expect("failed to lock job reciver");
                    match msg_lock.recv(){
                        Ok(job) => job,
                        Err(_) => {
                            println!("Sender for thread {} has died, returning.", thread_id);
                            return;
                        }
                    }
                };

                //If we got this job, let's execute it
                match job{
                    job::ThreadJob::Do(func) => {
                        func.exec_box()
                    },
                    job::ThreadJob::End => break, //end the thread
                }
            }

            println!("Shutting down thread {}", thread_id);
        });

        Worker{
            id: id,
            thread_handle: Some(thread_handle),
        }
    }



}

impl Drop for Worker{
    fn drop(&mut self) {
        println!("Ending worker {}", self.id);
        if let Some(handel) = self.thread_handle.take(){
            match handel.join(){
                Ok(_) => {},
                Err(_) => println!("Failed to join thread {}", self.id),
            }
        }else{
            return; //don't need to join
        }
    }
}


///Creates a pool of threads which can execute different tasks based on what is supplied
pub struct ThreadPool {
    id: u32,
    pool: Vec<Worker>,
    sender: Sender<job::ThreadJob>,
    reciver: Arc<Mutex<Receiver<job::ThreadJob>>>
}

impl ThreadPool{
    pub fn new(size: u32, id: u32) -> Self{
        let mut threads = Vec::new();
        //init communication channel
        let (sender, reciver) = channel::<job::ThreadJob>();
        let arc_reciver = Arc::new(Mutex::new(reciver));

        for idx in 0..size{
            threads.push(Worker::start(idx, arc_reciver.clone()));
        }

        ThreadPool{
            id: id,
            pool: threads,
            sender: sender,
            reciver: arc_reciver
        }
    }

    //Takes a closure which will be executed on one of the threads.
    //**NOTE**: The pool will end if one of the threads ends unexpectedly.
    pub fn execute<T>(&mut self, job: T) where T: FnOnce() + Send + 'static{
        //Create the job box, then send
        let job_box = job::ThreadJob::Do(Box::new(job));
        match self.sender.send(job_box){
            Ok(_) => {},
            Err(error) => {
                println!("failed to send job: {}", error);
            }
        }
    }

    ///Adds one more worker to this ThreadPool
    pub fn add_worker(&mut self){
        let id = self.pool.len() + 1;
        self.pool.push(Worker::start(id as u32, self.reciver.clone()));
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        //First end all threads, then go out of scope
        for _ in self.pool.iter(){
            let _ = self.sender.send(job::ThreadJob::End);
        }
    }
}
