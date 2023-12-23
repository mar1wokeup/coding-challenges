use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut count_bytes = false;
    let mut count_lines = false;
    let mut count_words = false;
    let mut count_chars = false;

    let mut filename = "";

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-c" => count_bytes = true,
            "-l" => count_lines = true,
            "-w" => count_words = true,
            "-m" => count_chars = true,
            _ => filename = arg,
        }
    }

    // If no options are provided, default to -c, -l, and -w
    if !(count_bytes || count_lines || count_words || count_chars) {
        count_bytes = true;
        count_lines = true;
        count_words = true;
    }

    if filename.is_empty() {
        eprintln!("Filename is required");
        std::process::exit(1);
    }

    count_stats(filename, count_bytes, count_lines, count_words, count_chars);
}

fn count_stats(filename: &str, count_bytes: bool, count_lines: bool, count_words: bool, count_chars: bool) {
    match fs::read_to_string(filename) {
        Ok(contents) => {
            let mut results = vec![];

            if count_lines {
                let lines = contents.lines().count();
                results.push(format!("{:>8}", lines));
            }

            if count_words {
                let words = contents.split_whitespace().count();
                results.push(format!("{:>8}", words));
            }

            if count_chars {
                let chars = contents.chars().count();
                results.push(format!("{:>8}", chars));
            }

            if count_bytes {
                let bytes = contents.as_bytes().len();
                results.push(format!("{:>8}", bytes));
            }

            let result_string = results.join(" ");
            println!("{} {}", result_string, filename);
        },
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    }
}




