
extern crate jakar_threadpool;
use jakar_threadpool::*;
use std::sync::{Arc, Mutex};

struct TestStruct {
    name: String,
}

impl TestStruct{
    fn new(stringy: &str) -> Self{
        TestStruct{
            name: stringy.to_string(),
        }
    }

    fn do_some(&self){
        println!("Hi I am complex mister {}", self.name);
    }

    fn change_name(&mut self, name: String){
        self.name = name;
        println!("Changed name", );
    }
}

fn fibo(n: i32) -> i32{
    if (n == 0) || (n == 1){
        return 1;
    }else{
        return fibo(n-1) + fibo(n-2);
    }
}


fn main(){

    let thread_size = 16;

    //Creates a thread pool, sends it some infos and the shuts down
    let mut pool = ThreadPool::new(thread_size, 0);

    pool.execute(||{
        println!("Hello from thread!", );
    });

    let thingy = 5;
    pool.execute(move ||{
        println!("Captured {}", thingy);
    });

    let complex = TestStruct::new("Namy");
    let arc_complex = Arc::new(Mutex::new(complex));

    let cpy_arc = arc_complex.clone();
    pool.execute(move||{
        let lck = cpy_arc.lock().unwrap();
        lck.do_some();
    });

    let cpy_arc_second = arc_complex.clone();
    pool.execute(move||{
        let mut lck = cpy_arc_second.lock().unwrap();
        lck.change_name(String::from("Rolfus"));
    });

    let cpy_arc_third = arc_complex.clone();
    pool.execute(move||{
        let lck = cpy_arc_third.lock().unwrap();
        lck.do_some();
    });

    println!("Fibo", );
    //println!("Fibo {}", fibo(64));

    for idx in 0..thread_size{
        pool.execute(move||{
            println!("Fibo from {} is {}", idx + 30, fibo(idx as i32 +30));
        });
    }

    pool.execute(||{ assert!(4 == 5); });

    //no go out of scope and end gracefully
}
