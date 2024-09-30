use std::{
    fs,
    io::{prelude::*, BufRead, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use hello::ThreadPool;
//chill time
fn main() {
    let listners = TcpListener::bind("127.0.0.1:7878").unwrap();
    match listners.accept(){
        Ok((_, addr)) => println!("New client {addr}"),
        Err(e) => println!("Client can't connect {e}")
    }

    let pool = ThreadPool::new(4);

    for stream in listners.incoming().take(2) {
        match stream {
            Ok(stream) =>{
                pool.execute(||{
                    handle_connection(stream);
                });
            }
            Err(_) => println!("Can't make a stream")
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_read = BufReader::new(&mut stream);
    let request_line = buf_read.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "../pages/hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "../pages/hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "../pages/404.html"),
    };
    
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
