use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Clone, Debug)]
pub enum TaskState {
    Pending,
    NotReady,
    Ready,
}

/// Task that can stop and resume at specific points
pub struct Task {
    state: TaskState,
}

impl Future for Task {
    type Output = ();

    // fn poll(&self, waker: Waker) -> Result<TaskState> {
    //     return Ok(self.state.clone());
    // }

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

/// Responsible for scheduling futures
pub struct Executor<F> {
    futures: Vec<F>,
}

pub struct Waker {}

/// Responsible for notifying about I/O events
pub struct Reactor {
    waker: &'static Waker,
}

impl Reactor {
    pub fn wake() {}
}

// 1. Executor creates a Waker and passes it in Future::poll()

// 2. Reactor stores a copy of a Waker

#[test]
fn test_endpoint() {
    println!("Hello world");
}
