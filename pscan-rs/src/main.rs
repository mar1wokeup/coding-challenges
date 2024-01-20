use std::env;
use std::net::TcpStream;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: pscan-rs -host=<hostname> -port=<port>");
        process::exit(1);
    }

    let host = &args[1][6..];
    let port = &args[2][6..];

    let address = format!("{}:{}", host, port);

    println!("Scanning host: {} port: {}", host, port);

    match TcpStream::connect(&address) {
        Ok(_) => println!("Port: {} is open", port),
        Err(_) => println!("Port: {} is closed", port),
    }
}
