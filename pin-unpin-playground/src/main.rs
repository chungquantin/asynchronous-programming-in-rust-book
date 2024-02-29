use std::{
    future::Future,
    pin::Pin,
    thread::sleep,
    time::{Duration, Instant},
};

#[derive(Default)]
struct RandFuture;

impl Future for RandFuture {
    type Output = u32;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::task::Poll::Ready(rand::random())
    }
}

#[pin_project::pin_project]
struct TimedWrapper<Fut>
where
    Fut: Future,
{
    start: Option<Instant>,
    #[pin]
    future: Fut,
}

impl<Fut: Future> TimedWrapper<Fut> {
    pub fn new(future: Fut) -> Self {
        Self {
            future,
            start: None,
        }
    }
}

impl<Fut> Future for TimedWrapper<Fut>
where
    Fut: Future,
{
    type Output = (Fut::Output, Duration);

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        let start = this.start.get_or_insert_with(Instant::now);
        let elapsed = start.elapsed();

        match this.future.poll(cx) {
            std::task::Poll::Ready(output) => std::task::Poll::Ready((output, elapsed)),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

#[tokio::main]
async fn main() {
    let timewrapped_fut = TimedWrapper::new(RandFuture::default());
    let (value, elapsed_time) = timewrapped_fut.await;
    println!("value: {:?} - ms: {:?}", value, elapsed_time);
}
