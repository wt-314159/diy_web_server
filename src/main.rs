use std::{
    fs, 
    thread, 
    path::Path, 
    time::Duration,
    io::{prelude::*, BufReader}, 
    net::{TcpListener, TcpStream}, 
};
use diy_web_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        pool.execute(|| handle_connection(stream));
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let parts: Vec<_> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        return;
    }
    else if parts[0] == "GET" {
        handle_get(stream, parts[1])
    }
    else {
        
    }
}

fn handle_get(stream: TcpStream, uri: &str) {
    let status_ok = "HTTP/1.1 200 0K";
    let status_not_found = "HTTP/1.1 404 NOT FOUND";
    let file_path_string = format!("html/{uri}.html");
    let file_path = Path::new(&file_path_string);

    let (status_line, contents) = match uri {
        "/" => (status_ok, fs::read_to_string("html/hello.html").unwrap()),
        "/sleep" => {
            thread::sleep(Duration::from_secs(10));
            (status_ok, fs::read_to_string("html/hello.html").unwrap())
        },
        _ => if file_path.exists() {
            (status_ok, fs::read_to_string(file_path).unwrap())
        }
        else {
            (status_not_found, fs::read_to_string("html/404.html").unwrap())
        }
    };
    send_contents(stream, status_line, &contents);
}

fn send_contents(mut stream: TcpStream, status_line: &str, contents: &str) {
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}

#[allow(dead_code)]
fn print_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");
}