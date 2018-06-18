extern crate num_cpus;

use std::thread::*;
use std::sync::mpsc::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use std::result::Result;
use std::sync::TryLockError;

pub mod job;


struct Worker {
    name: String,
    thread_handle: Option<JoinHandle<()>>,
    is_working: Arc<Mutex<bool>>,
}

impl Worker{
    ///Starts this worker
    pub fn start(
        name: String,
        reciver: Arc<Mutex<Receiver<job::ThreadJob>>>,
    ) -> Self{

        //TODO use builder for thread
        let thread_name = name.clone();
        let builder = Builder::new().name(name.clone());
        let is_working = Arc::new(Mutex::new(false));
        let my_is_working = is_working.clone();
        let thread_handle: JoinHandle<()> = builder.spawn(move || {
            //Loop until we recive the ending signal
            let thread_reciver = reciver;
            let is_working_changer = my_is_working.clone();

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
                        //Set to working state and lock the mutex
                        let mut is = is_working_changer.lock().expect("Failed to lock queue counter");
                        *is = true;
                        func.exec_box();
                        //Unlock and set to waiting
                        *is = false;
                    },
                    job::ThreadJob::End => {
                        break;
                    } //end the thread
                }
            }
        }).expect("Failed to spawn thread!");

        Worker{
            name: name,
            thread_handle: Some(thread_handle),
            is_working: is_working,
        }
    }

    ///Returns false if the worker is not working or poissened.
    pub fn is_working(&self) -> bool{

        //if not, check the actuall state
        match self.is_working.try_lock(){
            Ok(state) => if *state { return true; } else { return false; }
            Err(er) => {
                match er{
                    TryLockError::Poisoned(_) => return false,
                    TryLockError::WouldBlock => return true,
                }
            }
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
    reciver: Arc<Mutex<Receiver<job::ThreadJob>>>,

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
                arc_reciver.clone(),
            ));
        }

        ThreadPool{
            name: name,
            pool: threads,
            sender: sender,
            reciver: arc_reciver,
        }
    }

    ///Takes a closure which will be executed on one of the threads.
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

    ///Blocks until all threads have finished
    pub fn wait(&mut self){
        //Wait a bit for the event that we call the wait directly after startin
        sleep(Duration::from_millis(10));
        //Check all workers and check if we have any, maybe unfinished jobs left
        'waiting: loop{
            let mut is_working = false;
            for worker in self.pool.iter(){
                //If we changed to the working state once, just check the others
                if !is_working{
                    is_working = worker.is_working();
                }else{
                    continue;
                }
            }

            if !is_working{
                break;
            }
        }
        println!("Finished waiting", );
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
