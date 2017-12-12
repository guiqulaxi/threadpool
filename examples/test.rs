extern crate threadpool;
use threadpool::threadpool::ThreadPool;
use threadpool::task::DoTask;
use std::thread;
use std::sync::Arc;
#[derive(Debug)]
struct MyTask {
    pub param: i32,
}

impl DoTask<i32> for MyTask {
    fn do_task(&self) -> i32 {
        for i in 0 .. 10000000{
            let mut  i=1f32;
            i=i*1.1;
        }
        self.param + 1
    }
}

impl Clone for MyTask {
    fn clone(&self) -> MyTask {
        MyTask { param: self.param }
    }
}

fn main() {

    let pool = Arc::new(ThreadPool::<MyTask, i32>::new(40, 100));

    let p1 = pool.clone();
    
    let ph = thread::spawn(move || for i in 0..1000 {
                            let mytask = MyTask { param: i };
                              
                               p1.add_task(mytask.clone());
                           });
    let p2 = pool.clone();
    thread::sleep_ms(1000);
    let ch = thread::spawn(move || for _ in 0..1000 {
                               let a = p2.get_result();
                               println!("{:?}", a.unwrap());
                           });

    let p = pool.clone();
    let sh = thread::spawn(move || { p.start(3); });
    


    sh.join().unwrap();
    ph.join().unwrap();
    ch.join().unwrap();

}

