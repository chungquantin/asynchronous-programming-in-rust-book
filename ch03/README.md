# Chapter 3 - Understanding OS-backed Event Queues, System Calls and Cross Platform Abstractions

This folder contains the code examples for Chapter 3.

## Why use an OS-backed event queue?

## Readiness-based event queues

These event queues listed below are variants of each other in different operatin systems: Linux, Windows or MacOS.

### epoll

The code below defines FFI function signatures from Rust code to low-level C methods in the source code of the kernel.

```rs
pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLLIN: i32 = 0x1;
pub const EPOLLET: i32 = 1 << 31;
#[link(name = "c")]
extern "C" {
    pub fn epoll_create(size: i32) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut Event) -> i32;
    pub fn epoll_wait(epfd: i32, events: *mut Event, maxevents: i32, timeout: i32) -> i32;
}
```

### kqueue

```rs
fn kqueue() -> Result<RawFd>
fn kevent(kq: RawFd, changelist: &[KEvent], eventlist: &mut [KEvent], timeout_ms: usize) -> Result<usize>

pub struct KEvent {
    pub ident: uintptr_t,
    pub filter: EventFilter,
    pub flags: EventFlag,
    pub fflags: FilterFlag,
    pub data: intptr_t,
    pub udata: usize,
}
```

## Completion-based event queues

### IOCP (Input/output complettion port)

## Syscalls, FFI and cross-platform abstractions
