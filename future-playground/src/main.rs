mod time_util;

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
    task::{waker_ref, ArcWake},
    Future,
};
use time_util::get_epoch_ms;

type ArcTask<Fut> = Arc<Task<Fut>>;

struct Task<Fut>
where
    Fut: Future + Send + 'static,
{
    future: Mutex<Option<Pin<Box<Fut>>>>,
    task_sender: SyncSender<ArcTask<Fut>>,
}

impl<Fut> ArcWake for Task<Fut>
where
    Fut: Future + Send + 'static,
{
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

#[derive(Clone)]
struct Spawner<Fut>
where
    Fut: Future + Send + 'static,
{
    task_sender: SyncSender<ArcTask<Fut>>,
}

impl<Fut> Spawner<Fut>
where
    Fut: Future + Send + 'static,
{
    fn new(task_sender: SyncSender<ArcTask<Fut>>) -> Self {
        Self { task_sender }
    }

    fn spawn(&self, future: Fut) {
        let future = Box::pin(future);
        let task = Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        };
        self.task_sender.send(Arc::new(task)).unwrap();
    }
}

struct Executor<Fut>
where
    Fut: Future + Send + 'static,
{
    ready_queue: mpsc::Receiver<ArcTask<Fut>>,
}

impl<Fut> Executor<Fut>
where
    Fut: Future + Send + 'static,
{
    fn new(ready_queue: Receiver<ArcTask<Fut>>) -> Self {
        Self { ready_queue }
    }

    fn run(&self) {
        let mut pending_tasks = 0;
        while let Ok(task) = self.ready_queue.recv() {
            pending_tasks += 1;
            let waker = waker_ref(&task);
            let ctx = &mut Context::from_waker(&waker);
            let mut future_slot = task.future.lock().unwrap();
            if let Some(future) = future_slot.as_mut() {
                if future.as_mut().poll(ctx).is_ready() {
                    if pending_tasks == 0 {
                        break;
                    }
                    pending_tasks -= 1;
                }
            }
        }
    }
}

fn new_executor_and_spawner<Fut>() -> (Executor<Fut>, Spawner<Fut>)
where
    Fut: Future + Send + 'static,
{
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = mpsc::sync_channel(MAX_QUEUED_TASKS);
    (Executor::new(ready_queue), Spawner::new(task_sender))
}

struct SharedState {
    completed: bool,
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

struct TimedWrapper<Fut>
where
    Fut: Future,
{
    fut: Mutex<Pin<Box<Fut>>>,
    start_time: u128,
}

impl<Fut> TimedWrapper<Fut>
where
    Fut: Future + Send + 'static,
{
    pub fn new(fut: Fut) -> Self {
        TimedWrapper {
            fut: Mutex::new(Box::pin(fut)),
            start_time: get_epoch_ms(),
        }
    }
}

impl<Fut> Future for TimedWrapper<Fut>
where
    Fut: Future + Send + 'static,
{
    type Output = (Fut::Output, u128);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut guarded_future = self.fut.lock().unwrap();
        let inner_future = guarded_future.as_mut();
        let end_time = time_util::get_epoch_ms();
        match inner_future.poll(cx) {
            Poll::Ready(output) => Poll::Ready((output, end_time - self.start_time)),
            Poll::Pending => Poll::Pending,
        }
    }
}

async fn create_time_future_instance(second: u64) {
    println!("howdy!");
    TimerFuture::new(Duration::new(second, 0)).await;
    println!("done!");
}

async fn create_time_wrapped_future_instance(second: u64) {
    let (_, time) = TimedWrapper::new(create_time_future_instance(second)).await;
    println!("timestamp: {:?}", time);
}

fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    spawner.spawn(create_time_wrapped_future_instance(2));
    spawner.spawn(create_time_wrapped_future_instance(4));

    drop(spawner);

    executor.run();
}
