use crate::types::result::Result;
use crate::utils::threading;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug)]
struct HttpRequest {
  http_version: String,
  method: String,
  path: String,
  headers: HashMap<String, String>,
}

fn http_read_request(stream: &mut TcpStream) -> Result<HttpRequest> {
  const BUFFERED_READ_SIZE: usize = 4096;

  let mut receive = vec![];

  loop {
    let mut receive_buffer: [u8; BUFFERED_READ_SIZE] = [0; BUFFERED_READ_SIZE];
    let bytes_read = stream.read(&mut receive_buffer)?;

    receive.write(&receive_buffer[..bytes_read])?;

    if bytes_read < BUFFERED_READ_SIZE {
      break;
    }
  }

  let mut request = HttpRequest {
    http_version: String::new(),
    method: String::new(),
    path: String::new(),
    headers: HashMap::new(),
  };

  for (index, line) in String::from_utf8(receive)?.split("\r\n").into_iter().enumerate() {
    if line.is_empty() {
      break;
    }

    match index {
      0 => {
        let split: Vec<&str> = line.split(' ').collect();

        request.method = String::from(split[0]);
        request.path = String::from(split[1]);
        request.http_version = String::from(split[2]);
      }
      _ => {
        let split: Vec<&str> = line.split(": ").collect();

        request.headers.insert(String::from(split[0]), String::from(split[1]));
      }
    }
  }

  Ok(request)
}

struct Screen(usize);

fn parse_stream_request(path: &str) -> Result<Option<Screen>> {
  lazy_static! {
    static ref REGEX: Regex = Regex::new(r"^/screen/(\d+)$").unwrap();
  }

  match path {
    "/" => Ok(Some(Screen(0))),
    _ => match REGEX.captures(path) {
      Some(captures) => {
        let screen: usize = captures.get(1).unwrap().as_str().parse()?;

        match screen {
          0 => Ok(None),
          _ => {
            let screen_id = screen - 1;
            Ok(Some(Screen(screen_id)))
          }
        }
      }
      None => Ok(None),
    },
  }
}

fn service_request(request: HttpRequest, stream: &mut TcpStream) -> Result<()> {
  lazy_static! {
    static ref SCREENS: Vec<screenshots::Screen> = screenshots::Screen::all().unwrap();
  }

  match parse_stream_request(&request.path)? {
    Some(Screen(screen_id)) if screen_id < SCREENS.len() => {
      let screen = SCREENS[screen_id];

      println!("Handling request to share screen id {}", screen_id);
      println!("Screen: {:?}", screen);

      let image = screen.capture().unwrap();
      let buffer = image.buffer();

      let mut response_buffer = vec![];
      response_buffer.write("HTTP/1.1 200 OK\r\n".as_bytes())?;
      response_buffer.write("Content-Type: image/png\r\n".as_bytes())?;
      response_buffer.write(format!("Content-Length: {}\r\n", buffer.len()).as_bytes())?;
      response_buffer.write("\r\n".as_bytes())?;
      response_buffer.write(&buffer)?;

      stream.write(&response_buffer)?;
    }
    _ => {
      println!("Not handling request for '{}'", request.path);
      stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes())?;
    }
  }

  Ok(())
}

fn http_server() -> Result<()> {
  let listener = TcpListener::bind("0.0.0.0:8000")?;

  for (index, stream) in listener.incoming().into_iter().enumerate() {
    let mut tcp_stream = stream?;

    let request = http_read_request(&mut tcp_stream)?;

    println!("Received request #{}", index);
    println!("{:#?}", request);

    service_request(request, &mut tcp_stream)?;

    tcp_stream.flush()?;
  }

  Ok(())
}

pub fn run_http_server() -> Result<()> {
  threading::spawn_thread(http_server).join().unwrap();
  Ok(())
}
