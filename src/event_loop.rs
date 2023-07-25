use crate::types::{RedisError, Result};
use log::{error, info};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

pub type TaskType = Box<dyn FnOnce() + Send + 'static>;

pub enum Command {
    Task(TaskType),
    Shutdown,
}

pub struct EventLoop {
    capacity: usize,
    workers: Vec<Worker>,
    sender: Sender<Command>,
}

pub struct Worker {
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Command>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let receiver = receiver.lock().unwrap().recv();

            match receiver {
                Ok(cmd) => match cmd {
                    Command::Task(func) => {
                        info!("\x1b[32m[Worker #{}]\x1b[0m ", id);
                        func();
                    }
                    Command::Shutdown => {
                        info!("\x1b[32m[Shutting down worker #{}]\x1b[0m ", id);

                        break;
                    }
                },
                Err(err) => {
                    error!("Error occured : {:?}", err)
                }
            }
        });

        Self { thread }
    }

    pub fn join(self) -> Result<()> {
        self.thread
            .join()
            .map_err(|err| RedisError::Custom(format!("Error while joining thread: {:?}", err)))?;

        Ok(())
    }
}

impl EventLoop {
    pub fn new(threads: usize) -> Self {
        let mut workers = Vec::with_capacity(threads);

        let (sender, receiver) = mpsc::channel::<Command>();

        let receiver_ref = Arc::new(Mutex::new(receiver));

        for id in 0..threads {
            workers.push(Worker::new(id, receiver_ref.clone()))
        }

        Self {
            capacity: threads,
            workers,
            sender,
        }
    }

    pub fn send(&mut self, cmd: Command) -> Result<()> {
        self.sender.send(cmd).map_err(|err| {
            RedisError::Custom(format!(
                "Error while sending command to event loop: {:?}",
                err
            ))
        })?;

        Ok(())
    }

    pub fn terminate(&mut self) -> Result<()> {
        for _ in 0..self.capacity {
            self.sender.send(Command::Shutdown).map_err(|err| {
                RedisError::Custom(format!(
                    "Error while sending shutdown command to event loop: {:?}",
                    err
                ))
            })?;
        }

        while let Some(worker) = self.workers.pop() {
            worker.join().map_err(|err| {
                RedisError::Custom(format!("Error while joining worker thread: {:?}", err))
            })?;
        }

        Ok(())
    }
}
