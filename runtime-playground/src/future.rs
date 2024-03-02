use std::pin::Pin;

use crate::executor::Waker;

pub enum PollState<T> {
    Pending,
    Ready(T),
}

pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, waker: Waker) -> PollState<Self::Output>;
}
