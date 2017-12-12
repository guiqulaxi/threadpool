use task::DoTask;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicBool, Ordering};
use std::option::Option;
use std::thread;
///T: Task type
///R :Return type
pub struct ThreadPool<T, R>
    where T: DoTask<R> + Clone + Send + 'static,
          R: Send + 'static
{
    inner: Arc<Inner<T, R>>,
}
struct Inner<T, R>
    where T: DoTask<R> + Send + 'static,
          R: Send
{
    pub max_task_num: usize,
    pub max_result_num: usize,
    pub controller: AtomicBool,

    pub taskque_condvar: Condvar,
    pub resultque_condvar: Condvar,

    pub result_count: Mutex<i32>,
    pub task_count: Mutex<i32>,

    pub taskque: Mutex<VecDeque<T>>,
    pub resultque: Mutex<VecDeque<R>>,
}
impl<T, R> Inner<T, R>
    where T: DoTask<R> + Send + 'static,
          R: Send + 'static
{
    pub fn do_task(&self) {
        let task;
        {
            // println!("do_task1");
            let mut task_count = self.task_count.lock().unwrap();
            while *task_count <= 0  {
                task_count = self.taskque_condvar.wait(task_count).unwrap();
            }
            //println!("do_task2");
            let mut taskque = self.taskque.lock().unwrap();
            task = taskque.pop_front().unwrap();
            *task_count=*task_count-1;
            self.taskque_condvar.notify_one();
        }
        
        
        let result = task.do_task();
        
         //println!("do_task3");
        {
            let mut result_count = self.result_count.lock().unwrap();
            while *result_count as usize > self.max_result_num {
                result_count = self.resultque_condvar.wait(result_count).unwrap();
            }
            //println!("do_task4");
            let mut resultque = self.resultque.lock().unwrap();
            resultque.push_back(result);
            *result_count=*result_count+1;
            self.resultque_condvar.notify_one();
        }
        //println!("do_task5");
    }

    pub fn add_task(&self, task: T) {
       
        let mut task_count = self.task_count.lock().unwrap();
        while *task_count > self.max_task_num as i32 {
            task_count = self.taskque_condvar.wait(task_count).unwrap();
        }
        
        let mut queue = self.taskque.lock().unwrap();
        queue.push_back(task);
        self.taskque_condvar.notify_one();
        *task_count=*task_count+1;
        //println!("add_task");

    }
    pub fn get_result(&self) -> Option<R> {
       // println!("get_result1");
        let mut result_count = self.result_count.lock().unwrap();
       // println!("result_count {:?}", *result_count);
        while *result_count <= 0 {
            result_count = self.resultque_condvar.wait(result_count).unwrap();
        }
         // println!("get_result2");
        let mut queue = self.resultque.lock().unwrap();
        let res= queue.pop_front();
        //println!("get_result3");
        *result_count=*result_count-1;
        res
    }
}
impl<T, R> ThreadPool<T, R>
    where T: DoTask<R> + Clone + Send + 'static,
          R: Send + 'static
{
    pub fn new(max_task_num: usize, max_result_num: usize) -> ThreadPool<T, R> {
        ThreadPool {
            inner: Arc::new(Inner {
                                max_task_num: max_task_num,
                                max_result_num: max_result_num,
                                result_count: Mutex::new(0),
                                task_count: Mutex::new(0),
                                controller: AtomicBool::new(true),
                                taskque_condvar: Condvar::new(),
                                resultque_condvar: Condvar::new(),
                                taskque: Mutex::new(VecDeque::with_capacity(max_task_num)),
                                resultque: Mutex::new(VecDeque::with_capacity(max_result_num)),
                            }),
        }
    }
    pub fn add_task(&self, task: T) {
        self.inner.add_task(task);
    }
    pub fn get_result(&self) -> Option<R> {
        self.inner.get_result()

    }
    ///thread_nums :线程数目
    pub fn start(&self, thread_nums: u32) {

        let mut threads = Vec::new();
        for _ in 0..thread_nums {
            let inner = self.inner.clone();
            threads.push(thread::spawn(move || while inner.controller.load(Ordering::Relaxed) {
                                           inner.do_task();
                                       }));

        }
        for thread in threads {
            thread.join().unwrap();
        }
    }
}
