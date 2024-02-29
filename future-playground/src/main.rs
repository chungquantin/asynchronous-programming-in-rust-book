use std::{
    pin::Pin,
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    task::{Context, Poll, Waker},
    time::Duration,
};

use futures::{
    future::BoxFuture,
    task::{waker_ref, ArcWake},
    Future,
};

type ArcTask = Arc<Task>;

struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: SyncSender<ArcTask>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Implement `wake` by sending this task back onto the task channel
        // so that it will be polled again by the executor.
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<ArcTask>,
}

impl Spawner {
    fn new(task_sender: SyncSender<ArcTask>) -> Self {
        Self { task_sender }
    }

    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = Box::pin(future);
        let task = Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        };
        self.task_sender.send(Arc::new(task)).unwrap();
    }
}

struct Executor {
    ready_queue: mpsc::Receiver<ArcTask>,
}

impl Executor {
    fn new(ready_queue: Receiver<ArcTask>) -> Self {
        Self { ready_queue }
    }

    fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            let waker = waker_ref(&task);
            let ctx = &mut Context::from_waker(&waker);
            let mut future_slot = task.future.lock().unwrap();
            if let Some(future) = future_slot.as_mut() {
                if future.as_mut().poll(ctx).is_ready() {
                    break;
                }
            }
        }
    }
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    // Maximum number of tasks to allow queueing in the channel at once.
    // This is just to make `sync_channel` happy, and wouldn't be present in
    // a real executor.
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = mpsc::sync_channel(MAX_QUEUED_TASKS);
    (Executor::new(ready_queue), Spawner::new(task_sender))
}

/// Shared state between the future and the waiting thread
struct SharedState {
    /// Whether or not the sleep time has elapsed
    completed: bool,

    /// The waker for the task that `TimerFuture` is running on.
    /// The thread can use this after setting `completed = true` to tell
    /// `TimerFuture`'s task to wake up, see that `completed = true`, and
    /// move forward.
    waker: Option<Waker>,
}

struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

impl TimerFuture {
    pub fn new(sleep_time: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // Spawn the new thread
        let thread_shared_state = shared_state.clone();
        std::thread::spawn(move || {
            std::thread::sleep(sleep_time);
            let mut shared_state = thread_shared_state.lock().unwrap();
            // Signal that the timer has completed and wake up the last
            // task on which the future was polled, if one exists.
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
        });

        return TimerFuture { shared_state };
    }
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            return Poll::Ready(());
        } else {
            shared_state.waker = Some(cx.waker().clone());
            return Poll::Pending;
        }
    }
}

fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    // Spawn a task to print before and after waiting on a timer.
    spawner.spawn(async {
        println!("howdy!");
        // Wait for our timer future to complete after two seconds.
        TimerFuture::new(Duration::new(2, 0)).await;
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!");
    });

    // Drop the spawner so that our executor knows it is finished and won't
    // receive more incoming tasks to run.
    drop(spawner);

    // Run the executor until the task queue is empty.
    // This will print "howdy!", pause, and then print "done!".
    executor.run();
}
