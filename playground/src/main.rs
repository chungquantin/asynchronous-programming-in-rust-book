use std::io::{self, Read, Result, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use mio::{net::TcpStream, Events, Interest, Poll, Token};

// Module contains code related to the syscalls we need to communicate with the host OS
mod ffi;
// Module contains the main abstraction which is a thin layer over epoll
mod poll;

const CLIENT: Token = Token(1);

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n"
    )
}

fn handle_events(events: &Events, streams: &mut [TcpStream]) -> Result<usize> {
    let mut handled_events = 0;
    for event in events {
        let index = event.token();
        let mut stream = &streams[index.0];
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
    let mut poll = Poll::new()?;
    let n_events = 5;
    let mut streams = vec![];

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

    for i in 0..n_events {
        let delay = (n_events - i) * 1000;
        let url_path = format!("/{delay}/request-{i}");
        let request = get_req(&url_path);

        let mut client = TcpStream::connect(socket)?;

        client.write_all(request.as_bytes())?;

        poll.registry()
            .register(&mut client, CLIENT, Interest::READABLE | Interest::WRITABLE)?;

        streams.push(client);
    }

    let mut handled_events = 0;
    while handled_events < n_events {
        let mut events = Events::with_capacity(128);

        poll.poll(&mut events, Some(Duration::from_millis(100)))?;

        handled_events += handle_events(&events, &mut streams)?;
    }
    Ok(())
}
