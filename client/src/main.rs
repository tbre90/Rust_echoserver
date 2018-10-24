use std::env;
use std::process;
use std::io::{Write, Read, BufReader, BufRead};
use std::net::{TcpStream, Shutdown};

fn handle_communication(stream: &mut TcpStream) {
    stream.set_read_timeout(None).expect("failed to set read timeout.");

    let mut buffer = Vec::new();
    buffer.reserve(256);

    let mut stdin_reader  = BufReader::new(std::io::stdin());
    let mut socket_stream = BufReader::new(stream.try_clone().expect("failed to clone socket stream."));
    'send_receive: loop {
        let client_msg = &mut stdin_reader;
        match client_msg.take(256).read_until(b'\n', &mut buffer) {
            Ok(_) => (),
            Err(error) => {
                println!("{}", error);
                break 'send_receive;
            },
        };

        match stream.write(&buffer) {
            Ok(_)      => {
                buffer.clear();
            },
            Err(error) => {
                println!("{}", error);
                break 'send_receive;
            },
        };

        let server_msg = &mut socket_stream;
        match server_msg.take(256).read_until(b'\n', &mut buffer) {
            Ok(_)      => {
                println!("{}", std::str::from_utf8(&buffer).unwrap());
                buffer.clear();
            },
            Err(error) => {
                println!("{}", error);
                break 'send_receive;
            },
        };
    }

    stream.shutdown(Shutdown::Both).expect("shutdown call failed");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: ./client <ip address> <port>");
        process::exit(-1);
    }

    let ip_address = &args[1];
    let port       = &args[2];

    let mut stream = match TcpStream::connect(format!("{}:{}", ip_address, port)) {
        Ok(s) => s,
        Err(error) => {
            println!("{}", error);
            process::exit(-1);
        },
    };

    handle_communication(&mut stream);
}
