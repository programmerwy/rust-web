use std::sync::Mutex;
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;

pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;
enum Message {
  NewJob(Job),
  Terminate,
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    println!("Sending terminate message to all workers.");

    for _ in &mut self.workers {
      self.sender.send(Message::Terminate).unwrap();
    }

    println!("Shutting down all workers.");

    for worker in &mut self.workers {
      println!("Shutting down worker {}", worker.id);

      if let Some(thread) = worker.thread.take() {
        thread.join().unwrap();
      }
    }
  }
}

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
      sender: sx,
    }
  }

  pub fn excute<F>(&self, f: F)
    where
      F: FnOnce() + Send + 'static
  {
    let job = Box::new(f);

    self.sender.send(Message::NewJob(job)).unwrap();
  }
}

struct Worker {
  id: usize,
  thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
  fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
    let thread = thread::spawn(move || {
      loop {
        let message = rx.lock().unwrap().recv().unwrap();

        match message {
          Message::NewJob(job) => {
            println!("Worker {} got a job; executing.", id);

            job();
          },
          Message::Terminate => {
            println!("Worker {} was told to terminate.", id);

            break;
          }
        }
      }
    });

    Worker {
      id,
      thread: Some(thread),
    }
  }
}