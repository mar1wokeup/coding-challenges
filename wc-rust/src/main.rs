use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: wc-rust -c <filename>");
        std::process::exit(1);
    }

    let filename = &args[2];
    count_bytes(filename);
}

fn count_bytes(filename: &str) {
    match fs::read(filename) {
        Ok(contents) => {
            let byte_count = contents.len();
            println!("{} {}", byte_count, filename);
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    }
}
