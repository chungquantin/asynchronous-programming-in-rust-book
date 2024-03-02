use crate::executor::Waker;
use mio::{Events, Poll, Registry, Token};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc, Mutex, OnceLock},
};

type WakerRegistry = Arc<Mutex<HashMap<usize, Waker>>>;

static REACTOR: OnceLock<Reactor> = OnceLock::new();

pub fn reactor() -> &'static Reactor {
    REACTOR.get().expect("Called outside an runtime context")
}

pub struct Reactor {
    wakers: WakerRegistry,
    registry: Registry,
    next_id: AtomicUsize,
}

impl Reactor {
    pub fn start() {
        let waker_registry: HashMap<usize, Waker> = HashMap::new();
        let wakers = Arc::new(Mutex::new(waker_registry));

        let poll = Poll::new().unwrap();
        let registry = poll.registry().try_clone().unwrap();
        let default_id = AtomicUsize::new(1);

        let reactor = Reactor {
            wakers: wakers.clone(),
            registry,
            next_id: default_id,
        };
        REACTOR.set(reactor).ok().expect("Reactor already running");
        // spawnning a new thread for the event loop
        // so the main thread won't be blocked
        std::thread::spawn(move || event_loop(poll, wakers));
    }
}

fn event_loop(mut poll: Poll, wakers: WakerRegistry) {
    // Event queue accepts up to 100 events
    let mut events = Events::with_capacity(100);
    loop {
        // Polling events to the events queue
        poll.poll(&mut events, None).unwrap();
        for e in events.iter() {
            // Get the token id of the event
            let Token(id) = e.token();
            let wakers = wakers.lock().unwrap();

            if let Some(waker) = wakers.get(&id) {
                waker.wake();
            }
        }
    }
}
