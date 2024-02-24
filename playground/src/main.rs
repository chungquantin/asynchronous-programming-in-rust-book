use std::{
    io::{self, Read, Result, Write},
    net::TcpStream,
};

use ffi::{Event, EPOLLET, EPOLLIN};
use poll::Poll;

// Module contains code related to the syscalls we need to communicate with the host OS
mod ffi;
// Module contains the main abstraction which is a thin layer over epoll
mod poll;

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n"
    )
}

fn handle_events(events: &[Event], streams: &mut [TcpStream]) -> Result<usize> {
    let mut handled_events = 0;
    for event in events {
        let index = event.token();
        let mut stream = &streams[index];
        let mut data = vec![0u8; 4096]; // 4 MB
        loop {
            match stream.read(&mut data) {
                Ok(n) if n == 0 => {
                    handled_events += 1;
                    break;
                }
                Ok(n) => {
                    let txt = String::from_utf8_lossy(&data[..n]);
                    println!("RECEIVED: {:?}", event);
                    println!("{txt}\n......\n");
                }
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
                Err(err) => return Err(err),
            }
        }
    }
    Ok(handled_events)
}

fn main() -> Result<()> {
    let poll = Poll::new()?;
    let n_events = 5;
    let addr = "localhost:8080";
    let mut streams = vec![];
    for i in 0..n_events {
        let delay = (n_events - i) * 1000;
        let url_path = format!("/{delay}/request-{i}");
        let request = get_req(&url_path);

        let mut stream = std::net::TcpStream::connect(addr)?;

        stream.set_nonblocking(true)?;
        stream.write_all(request.as_bytes())?;

        poll.registry().register(&stream, i, EPOLLIN | EPOLLET)?;

        streams.push(stream);
    }

    let mut handled_events = 0;
    while handled_events < n_events {
        let mut events = Vec::with_capacity(10);
        poll.poll(&mut events, None)?;

        handled_events += handle_events(&events, &mut streams)?;
    }
    Ok(())
}
