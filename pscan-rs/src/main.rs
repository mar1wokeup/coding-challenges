use std::env;
use std::net::{ SocketAddr, TcpStream, ToSocketAddrs };
use std::time::Duration;
use threadpool::ThreadPool;
use std::sync::mpsc::channel;

fn main() {
    let args: Vec<String> = env::args().collect();

    let all_ports: Vec<u16> = (1..=65535).collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: pscan-rs -hosts=<hostname> -port=<port> (empty is full scan) [-timeout=<milliseconds>] [-threads=<num_threads>]"
        );
        std::process::exit(1);
    }

    let host = parse_hosts(&args[1][7..]);

    let mut ports = parse_ports(&args[2][6..]);
    if ports.is_empty() {
        ports = all_ports;
    }

    let timeout = args
        .get(3)
        .and_then(|arg| arg.strip_prefix("-timeout="))
        .and_then(|t| t.parse().ok())
        .unwrap_or(500); // default t.o. 500 ms

    let num_threads = args
        .get(4)
        .and_then(|arg| arg.strip_prefix("-threads="))
        .and_then(|t| t.parse().ok())
        .unwrap_or(100); // default 100 threads

    let pool = ThreadPool::new(num_threads);
    let (tx, rx) = channel();

    // println!("Scanning host: {}", host);

    for host in host {
        let tx = tx.clone();

        // println!("Scanning host: {}", host);
        // let host = host.to_string();
        let p = ports.clone();
        let h = host.clone();

        pool.execute(move || {
            for port in p {
                println!("Scanning port: {} on host {}", port, host);
                let addr = format!("{}:{}", h, port);
                match addr.to_socket_addrs() {
                    Ok(mut addrs) => {
                        if let Some(addr) = addrs.next() {
                            if is_port_open(addr, timeout) {
                                tx.send((port, h.clone())).unwrap_or_else(|e| {
                                    eprintln!("Failed to send port: {}", e);
                                });
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to resolve address for {}: {}", addr, e),
                }
            }
        });
    }

    drop(tx); // close the channel

    for (port, host) in rx.iter() {
        println!(" +++ Port: {} is open on host: {}", port, host);
    }

    println!("\nDone")
}

fn parse_hosts(hosts: &str) -> Vec<String> {
    if hosts.contains("*") {
        expand_wildcard(hosts)
    } else {
        hosts.split(',').map(String::from).collect()
    }
}

fn expand_wildcard(host: &str) -> Vec<String> {
    let mut expanded_hosts = Vec::new();
    let base_ip = host.replace("*", "");
    for i in 1..=255 {
        expanded_hosts.push(format!("{}{}", base_ip, i));
    }
    expanded_hosts
}

fn parse_ports(port_arg: &str) -> Vec<u16> {
    let mut ports = Vec::new();
    for part in port_arg.split(',') {
        if part.contains(':') {
            let range: Vec<&str> = part.split(':').collect();
            if range.len() == 2 {
                if let (Ok(start), Ok(end)) = (range[0].parse::<u16>(), range[1].parse::<u16>()) {
                    ports.extend(start..=end);
                }
            }
        } else if let Ok(port) = part.parse::<u16>() {
            ports.push(port);
        }
    }
    ports
}

fn is_port_open(addr: SocketAddr, timeout_ms: u64) -> bool {
    TcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)).is_ok()
}
