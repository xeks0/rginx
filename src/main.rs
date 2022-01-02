use std::any::Any;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::io;
use std::path::Path;
mod encoder;
use encoder::Encoder;
extern crate yaml_rust;
use yaml_rust::{YamlLoader, YamlEmitter};

fn handle_connection(mut stream: TcpStream) {
    let file_name = get_path(&stream);
    let file_path = format!("/Users/xeks/images{}", &file_name);
    if !Path::new(&file_path).exists() || !Path::new(&file_path).is_file() {
        let mut content_type =  "Content-type: text/html";
        let headers =
            ["HTTP/1.1 404 OK", content_type,"\r\n"];
        let mut response: Vec<u8> = headers.join("\r\n")
            .to_string()
            .into_bytes();

        // response.extend("File Not found".chars());
        match stream.write(&response) {
            Ok(_) => println!("Response sent"),
            Err(e) => println!("Failed sending response: {}", e),
        }
        return;
    }
    let mut buf = Vec::new();
    let path = Path::new(&file_name);

    let mut file = File::open(&file_path).unwrap();
    let mut content_type =  "Content-type: text/html";
    if path.extension().unwrap() == "png" {
         content_type =  "Content-type: image/png";
    }
    if path.extension().unwrap() == "jpg" || path.extension().unwrap() == "jpeg" {
        content_type =  "Content-type: image/jpeg";
    }
    let headers =
        ["HTTP/1.1 200 OK", content_type,"Transfer-Encoding: chunked","\r\n"];
    file.read_to_end(&mut buf).unwrap();
    let mut encoded = Vec::new();
    {
        let mut encoder = Encoder::with_chunks_size(&mut encoded, buf.len()/8192);
        encoder.write_all(&buf).unwrap();
    }

    let mut response: Vec<u8> = headers.join("\r\n")
        .to_string()
        .into_bytes();
    response.extend(encoded.clone());
    match stream.write(&response) {
        Ok(_) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
}
fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
}
fn get_path(mut stream: &TcpStream) -> String {
    let mut buf = [0u8; 4096];
    match stream.read(&mut buf) {
        Ok(_) => {
            let req_str = String::from_utf8_lossy(&buf);
            let path: Vec<&str> = req_str.lines().next().unwrap().split(" ").collect();
            if path.len()<2 {
             return "".parse().unwrap();
            }
            println!("GET {}", path[1]);
            // println!("{}", req_str);
            return path[1].to_string()
        }
        Err(e) => {
            println!("Unable to read stream: {}", e);
            "/".to_string()
        }
    }
}
fn wait_for_fd(){

}
fn main() -> std::io::Result<()> {


    let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    listener.set_nonblocking(false).expect("Cannot set non-blocking");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                // do something with the TcpStream
                handle_connection(s);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // wait until network socket is ready, typically implemented
                // via platform-specific APIs such as epoll or IOCP
                wait_for_fd();
                continue;
            }
            Err(e) => panic!("encountered IO error: {}", e),
        }
    }
    Ok(())
}
