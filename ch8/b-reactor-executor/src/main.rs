use std::{
    io::{ErrorKind, Read, Write},
    thread,
    time::Duration,
};

// later fn async_main() {
//     println!("Program starting")
//     let http = Http::new();
//     let txt = manjana http.get("/1000/HelloWorld");
//     let txt2 = manjana http.get("500/HelloWorld2");
//     println!("{txt}");
//     println!("{txt2}");
// }

struct Http;

impl Http {
    fn get(path: &'static str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    path: &'static str,
}

impl HttpGetFuture {
    fn new(path: &'static str) -> Self {
        Self {
            stream: None,
            buffer: vec![],
            path,
        }
    }
}

impl Future for HttpGetFuture {
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {
        // If this is first time polled, start the operation
        // see: https://users.rust-lang.org/t/is-it-bad-behaviour-for-a-future-or-stream-to-do-something-before-being-polled/61353
        // Avoid dns lookup this time
        if self.stream.is_none() {
            println!("FIRST POLL - START OPERATION");
            let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
            stream.set_nonblocking(true).unwrap();
            let mut stream = mio::net::TcpStream::from_std(stream);
            stream.write_all(get_req(self.path).as_bytes()).unwrap();
            self.stream = Some(stream);
            return PollState::NotReady;
        }

        let mut buff = vec![0u8; 4096];
        loop {
            match self.stream.as_mut().unwrap().read(&mut buff) {
                Ok(0) => {
                    let s = String::from_utf8_lossy(&self.buffer);
                    break PollState::Ready(s.to_string());
                }
                Ok(n) => {
                    self.buffer.extend(&buff[0..n]);
                    continue;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    break PollState::NotReady;
                }

                Err(e) => panic!("{e:?}"),
            }
        }
    }
}

enum MyOneStageFut<A> {
    Start(Box<dyn FnOnce() -> Box<dyn Future<Output = A>>>, Box<dyn FnOnce(A)>),
    Wait1(Box<dyn Future<Output = A>>, Box<dyn FnOnce(A)>),
    Resolved,
}

impl<A> MyOneStageFut<A> {
    fn new<F, H>(op1: F, op2: H) -> Self 
    where 
    F: (FnOnce()-> Box<dyn Future<Output = A>>) + 'static,
    H:  FnOnce(A) + 'static,
    {
        Self::Start(Box::new(op1), Box::new(op2))
    }
}

impl<A> Future for MyOneStageFut<A> {
    type Output = ();

    fn poll(&mut self) -> PollState<Self::Output> {
        let mut this = std::mem::replace(self, Self::Resolved);
        match this {
            Self::Start(op, op2) => {
                let fut = op();
                *self = MyOneStageFut::Wait1(fut, op2);
                PollState::NotReady
            }

            Self::Wait1(ref mut fut, ref mut op) => {
                let s = match fut.poll() {
                    PollState::Ready(s) => s,
                    PollState::NotReady => {
                        *self = this;
                        return PollState::NotReady;
                    },
                };
                
                let op = std::mem::replace(op, Box::new(|_|{}));
                
                op(s);

                

                *self = Self::Resolved;
                PollState::Ready(())
            }

            Self::Resolved => panic!("Polled a resolved future"),
        }
    }
}

// enum MyTwoStageFut {
//     Start(Box<dyn Future<Output = ()>>, Box<dyn Future<Output = ()>>),
//     Wait1(Box<dyn Future<Output = ()>>, Box<dyn Future<Output = ()>>),
//     Wait2(Box<dyn Future<Output = ()>>),
//     Resolved,
// }

// impl Future for MyTwoStageFut {
//     type Output = ();

//     fn poll(&mut self) -> PollState<Self::Output> {
//         match self {
//             Self::Start(f1, f2) => {
                
//             }
//         }
//     }
    
// }

trait Future {
    type Output;

    fn poll(&mut self) -> PollState<Self::Output>;
}

enum PollState<T> {
    Ready(T),
    NotReady,
}

fn async_main() -> impl Future<Output = ()> {
    MyOneStageFut::new(move || {
        println!("OP1 -STARTED");
        Box::new(Http::get("/500/HelloWorld"))
    }, move |s| {
                println!("GOT DATA");
                println!("{s}");
    })
}

fn main() {
    let mut future = async_main();

    loop {
        match future.poll() {
            PollState::NotReady => {
                println!("NotReady");
                // call executor sleep
                thread::sleep(Duration::from_millis(100));
            }

            PollState::Ready(s) => break s,
        }
    };
}

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n"
    )
}