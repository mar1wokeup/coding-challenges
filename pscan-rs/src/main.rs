use std::env;
use std::net::{ SocketAddr, TcpStream, ToSocketAddrs };
use std::time::Duration;
use threadpool::ThreadPool;
use std::sync::mpsc::channel;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: pscan-rs -host=<hostname> [-timeout=<milliseconds>] [-threads=<num_threads>]"
        );
        std::process::exit(1);
    }

    let host = &args[1][6..];
    let timeout = args
        .get(2)
        .and_then(|arg| arg.strip_prefix("-timeout="))
        .and_then(|t| t.parse().ok())
        .unwrap_or(500); // default t.o. 500 ms

    let num_threads = args
        .get(3)
        .and_then(|arg| arg.strip_prefix("-threads="))
        .and_then(|t| t.parse().ok())
        .unwrap_or(100); // default 100 threads

    let pool = ThreadPool::new(num_threads);
    let (tx, rx) = channel();

    println!("Scanning host: {}", host);

    for port in 1..=65535 {
        let tx = tx.clone();
        let host = host.to_string();
        pool.execute(move || {
            let addr = format!("{}:{}", host, port);

            match addr.to_socket_addrs() {
                Ok(mut addrs) => {
                    if let Some(addr) = addrs.next() {
                        if is_port_open(addr, timeout) {
                            tx.send(port).unwrap_or_else(|e| {
                                eprintln!("Failed to send port: {}", e);
                            });
                        }
                    }
                }
                Err(e) => eprintln!("Failed to resolve address for {}: {}", addr, e),
            }
        });
    }

    drop(tx); // close the channel

    for port in rx.iter() {
        println!("Port: {} is open", port);
    }

    println!("\nDone")
}

fn is_port_open(addr: SocketAddr, timeout_ms: u64) -> bool {
    TcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)).is_ok()
}
