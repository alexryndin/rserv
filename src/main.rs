use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use serv::thread::ThreadPool;

pub struct Request<'a> {
    request_line: &'a str,
    headers_and_body: Vec<&'a str>,
}

impl<'a> Request<'a> {
    pub fn parse(request: &'a str) -> Result<Request<'a>, &'static str> {
        let mut request_s = request.split("\r\n");

        let request_line = match request_s.next() {
            Some(v) => v,
            None => return Err("request not found"),
        };

        let mut headers_and_body = Vec::new();

        for v in request_s {
            headers_and_body.push(v);
        }

        Ok(Request {
            request_line,
            headers_and_body,
        })
    }
}

fn test(u: i32) -> i32 {
    4
}

fn test2(u: &str) -> i32 {
    4
}

fn test3(u: String) -> i32 {
    4
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0; 512];

    stream.read(&mut buf).unwrap();

    let request = &String::from_utf8_lossy(&buf);

    println!("{}", request);

    // let proper_get = b"GET / HTTP/1.1";

    let request_s = Request::parse(request).unwrap();

    let (status, filename) = match request_s.request_line {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK\r\n\r\n", "index.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK\r\n\r\n", "index.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html"),
    };

    let content = fs::read_to_string(filename).unwrap();
    let response = format!("{}{}", status, content);

    stream.write(response.as_bytes()).unwrap();

    stream.flush().unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8081").unwrap();
    let pool = ThreadPool::new(4).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });

        println!("Connection established!");
    }
}
