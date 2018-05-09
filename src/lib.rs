extern crate num_cpus;

use std::thread::*;
use std::sync::mpsc::*;
use std::sync::{Arc, Mutex};

use std::result::Result;

pub mod job;


struct Worker {
    name: String,
    thread_handle: Option<JoinHandle<()>>,
}

impl Worker{
    ///Starts this worker
    pub fn start(name: String, reciver: Arc<Mutex<Receiver<job::ThreadJob>>>) -> Self{

        //TODO use builder for thread
        let thread_name = name.clone();
        let builder = Builder::new().name(name.clone());
        let thread_handle: JoinHandle<()> = builder.spawn(move || {
            //Loop until we recive the ending signal
            let thread_reciver = reciver;

            loop {
                let job = {
                    let msg_lock = thread_reciver.lock().expect("failed to lock job reciver");
                    match msg_lock.recv(){
                        Ok(job) => job,
                        Err(_) => {
                            println!("Sender for thread {} has died, returning.", thread_name);
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

            println!("Shutting down thread {}", thread_name);
        }).expect("Failed to spawn thread!");

        Worker{
            name: name,
            thread_handle: Some(thread_handle),
        }
    }



}

impl Drop for Worker{
    fn drop(&mut self) {
        println!("Ending worker {}", self.name);
        if let Some(handel) = self.thread_handle.take(){
            match handel.join(){
                Ok(_) => {},
                Err(_) => println!("Failed to join thread {}", self.name),
            }
        }else{
            return; //don't need to join
        }
    }
}


///Creates a pool of threads which can execute different tasks based on what is supplied
pub struct ThreadPool {
    name: String,
    pool: Vec<Worker>,
    sender: Sender<job::ThreadJob>,
    reciver: Arc<Mutex<Receiver<job::ThreadJob>>>
}

impl ThreadPool{
    ///Creates a threadpool with the current hardware core count as `size`.
    pub fn new_hardware_optimal(name: String) -> Self{
        let size = num_cpus::get_physical() as u32;
        ThreadPool::new(size, name)
    }

    ///Creats a threadpool with the systems logical core count as `size`.
    pub fn new_logical_optimal(name: String) -> Self{
        let size = num_cpus::get() as u32;
        ThreadPool::new(size, name)
    }

    ///Creates the threadpool with the given number of threads
    pub fn new(size: u32, name: String) -> Self{
        let mut threads = Vec::new();
        //init communication channel
        let (sender, reciver) = channel::<job::ThreadJob>();
        let arc_reciver = Arc::new(Mutex::new(reciver));



        for idx in 0..size{
            threads.push(Worker::start(
                name.clone() + "_" + &idx.to_string(),
                arc_reciver.clone()
            ));
        }

        ThreadPool{
            name: name,
            pool: threads,
            sender: sender,
            reciver: arc_reciver
        }
    }

    ///Takes a closure which will be executed on one of the threads.
    ///**NOTE**: The pool will end if one of the threads ends unexpectedly.
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
        let name = self.name.clone() + "_" + &self.pool.len().to_string();
        self.pool.push(Worker::start(name, self.reciver.clone()));
    }

    ///Returns the size of this threadpool.
    pub fn len(&self) -> usize{
        self.pool.len()
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
