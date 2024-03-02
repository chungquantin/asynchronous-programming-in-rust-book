use crate::future::Future;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex},
    thread::Thread,
};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type Task<Output>
where
    Output: Sized + Send,
= BoxFuture<'static, Output>;

thread_local! {
 static CURRENT_EXEC : ExecutorCore<String> = ExecutorCore::default();
}

// Each future type per core
#[derive(Default)]
struct ExecutorCore<Output>
where
    Output: Sized + Send,
{
    tasks: RefCell<HashMap<usize, Task<Output>>>,
    ready_queue: Arc<Mutex<Vec<usize>>>,
    next_id: Cell<usize>,
}

pub struct Executor;

impl Executor {
    pub fn spawn<F>(future: F)
    where
        F: Future<Output = String> + 'static + Send,
    {
        CURRENT_EXEC.with(|e| {
            let id = e.next_id.get();
            e.tasks.borrow_mut().insert(id, Box::pin(future));
            e.ready_queue.lock().map(|mut q| q.push(id)).unwrap();
            e.next_id.set(id + 1);
        });
    }
}

#[derive(Clone)]
pub struct Waker {
    thread: Thread,
    id: usize,
    ready_queue: Arc<Mutex<Vec<usize>>>,
}

impl Waker {
    pub fn wake(&self) {
        self.ready_queue
            .lock()
            .map(|mut q| q.push(self.id))
            .unwrap();
        self.thread.unpark();
    }
}
