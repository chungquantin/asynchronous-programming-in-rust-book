# Resources

## Research

### tokio_io_pool

Thread pool for executing short, I/O-heavy futures efficiently

The standard Runtime provided by tokio uses a thread-pool to allow concurrent execution of compute-heavy futures.

This crate provides an alternative implementation of a futures-based thread pool. It spawns a pool of threads that each runs a tokio::runtime::current_thread::Runtime (and thus each have an I/O reactor of their own), and spawns futures onto the pool by assigning the future to threads round-robin. Once a future has been spawned onto a thread, it, and any child futures it may produce through tokio::spawn, remain under the control of that same thread.

### Rust

- https://github.com/Xudong-Huang/may
- https://tokio.rs/tokio/tutorial
- https://docs.rs/tokio/latest/tokio/
- https://docs.rs/rayon/latest/rayon/

- https://ryhl.io/blog/async-what-is-blocking/#the-rayon-crate: the Tokio runtime was not able to swap one task for another, because such a swap can only happen at an .await. Since there is no .await in sleep_then_print, no swapping can happen while it is running.

### Goroutine

Goroutines utilizes a concept that has been around for a while called “coroutines” which essentially means multiplexing a set of independently executing functions “coroutines” which are running on the user level, onto a set of actual threads on the OS level.

#### Stack size

Because Goroutine is a stackful coroutine, which means it spawns its own stack to save the state and run the instruction order.

=> To make the stacks `small`, Go’s run-time uses `resizable, bounded stacks`. A newly minted goroutine is given a few kilobytes, which is almost always enough.

=> `If not enough`, the runtime grows and shrinks the memory for storing the stack automatically.

The CPU overhead averages about three cheap instructions per function call. It is practical to create hundreds of thousands of goroutines in the same address space. If goroutines were just threads, system resources would run out at a much smaller number.

- https://osmh.dev/posts/goroutines-under-the-hood

## RUST

- https://d3s.mff.cuni.cz/teaching/nprg073/lecture_3/
