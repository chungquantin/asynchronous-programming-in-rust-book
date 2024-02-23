use std::{io::Result, net::TcpStream};

use crate::ffi;

type Events = Vec<ffi::Event>;

pub struct Registry {
    raw_fd: i32,
}

impl Registry {
    pub fn register(&self, source: &TcpStream, token: usize, interests: i32) -> Result<()> {
        todo!()
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        todo!()
    }
}

// represents the event queue
pub struct Poll {
    registry: Registry,
}

impl Poll {
    // create a new queue
    pub fn new() -> Result<Self> {
        Ok(Self {
            registry: Registry { raw_fd: 2 },
        })
    }

    // returns a reference to the registry that we can use to register interest to be notified about new events
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    // block the threads it's called on until an event is ready or it times out
    pub fn poll(&mut self, events: &mut Events, timeout: Option<i32>) -> Result<()> {
        todo!()
    }
}
