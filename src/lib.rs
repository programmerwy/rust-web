use std::sync::Mutex;
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;

pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
  /// 创建线程池。
  ///
  /// 线程池中线程的数量。
  ///
  /// # Panics
  ///
  /// `new` 函数在 size 为 0 时会 panic。
  pub fn new(size: usize) -> ThreadPool {
    assert!(size > 0);

    let (sx, rx) = mpsc::channel();
    let rx = Arc::new(Mutex::new(rx));

    let mut workers = Vec::with_capacity(size);

    for id in 0..size {
      workers.push(Worker::new(id, Arc::clone(&rx)));
    }

    ThreadPool {
      workers,
      sender: sx
    }
  }

  pub fn excute<F>(&self, f: F)
    where
      F: FnOnce() + Send + 'static
  {
    let job = Box::new(f);

    self.sender.send(job).unwrap();
  }
}

struct Worker {
  id: usize,
  thread: thread::JoinHandle<Arc<Mutex<mpsc::Receiver<Job>>>>,
}

impl Worker {
  fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
    let thread = thread::spawn(move || {
      loop {
        let job = rx.lock().unwrap().recv().unwrap();

        println!("Worker {} got a job; executing.", id);

        job();
      }
    });

    Worker {
      id,
      thread
    }
  }
}