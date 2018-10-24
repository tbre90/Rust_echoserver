extern crate chrono;

use std::io::{Read, BufReader, Write};
use std::io::BufRead;
use std::{thread, time};
use std::env;
use std::net::{TcpListener, TcpStream, Shutdown};

use chrono::prelude::*;

const CONNECTION_TIMEOUT: i32 = 60;

fn log_message(message: String) {
    let dt = Local::now();
    println!(
        "{} | {}",
        format!(
            "{:02}:{:02}:{:02} {:02}:{:02}:{:02} {}",
            dt.year(), dt.month(), dt.day(),
            dt.hour(), dt.minute(), dt.second(),
            dt.offset()
        ),
        message
    );
}

fn handle_client(stream: &mut TcpStream) {
    let peer_address = match stream.peer_addr() {
        Ok(sock_addr) => {
            log_message(
                format!("accepted connection from {}.", sock_addr)
            );
            sock_addr
        },
        Err(error) => {
            println!("failed to get peer address from socket. {}", error);
            stream.shutdown(Shutdown::Both).expect("shutdown call failed.");
            return;
        }
    };

    let mut buffer = Vec::new();
    buffer.reserve(256);

    // if no message is received within ~1 minute
    // shutdown the stream
    let mut timer = CONNECTION_TIMEOUT;

    let mut reader = BufReader::new(stream.try_clone().expect("failed to clone tcp stream."));
    'client_communication: loop {
        let tr = &mut reader;
        match tr.take(256).read_until(b'\n', &mut buffer) {
            Ok(n) => {
                if n > 0 {
                    log_message(
                        format!(
                            "message from client: {}", std::str::from_utf8(&buffer).unwrap().to_string().trim_right()
                        )
                    );
                    timer = CONNECTION_TIMEOUT; // message received - reset timer
                }
            },
            Err(error) => { println!("{}", error); break 'client_communication; },
        };

        if buffer.len() > 0 {
            let reply = std::str::from_utf8(&buffer).unwrap().to_string();
            match stream.write_all(format!("You wrote: {}", reply).as_bytes()) {
                Ok(_) => {
                    log_message(
                        format!("reply sent to client {}.",
                                peer_address
                        )
                    );
                },
                Err(error) => {
                    println!("{}", error);
                    break 'client_communication;
                }
            };
        }

        thread::sleep(time::Duration::new(1, 0));
        timer -= 1;
        if timer <= 0 {
            break 'client_communication;
        }

        if buffer.len() > 0 {
            buffer.clear();
        }

        println!("");
    }

    log_message("closing connection.".to_string());
    stream.shutdown(Shutdown::Both).expect("shutdown call failed.");
}

fn listen() -> Result<String, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        let err_str = "Usage: ./server <ip address> <port>".to_string();
        return Err(err_str);
    }

    let ip_address = &args[1];
    let port       = &args[2];

    let listener = TcpListener::bind(format!("{}:{}", ip_address, port)).unwrap();

    for stream in listener.incoming() {
        std::thread::spawn(|| handle_client(&mut stream.unwrap()));
    }

    Ok("Server successfully shut down.".to_string())
}

fn main() {
    match listen() {
        Ok(msg) => println!("{}", msg),
        Err(error) => println!("{}", error),
    }
}
