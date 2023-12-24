use std::env;
use std::fs;
use std::io::{ self, Read };

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut count_bytes = false;
    let mut count_lines = false;
    let mut count_words = false;
    let mut count_chars = false;

    let mut filename = None;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-c" => {
                count_bytes = true;
            }
            "-l" => {
                count_lines = true;
            }
            "-w" => {
                count_words = true;
            }
            "-m" => {
                count_chars = true;
            }
            _ => {
                filename = Some(arg);
            }
        }
    }

    // If no options are provided, default to -c, -l, and -w
    if !(count_bytes || count_lines || count_words || count_chars) {
        count_bytes = true;
        count_lines = true;
        count_words = true;
    }

    // if filename.is_empty() {
    //     eprintln!("Filename is required");
    //     std::process::exit(1);
    // }

    let (lines, words, chars, bytes) = match filename {
        Some(f) => count_stats(Some(f), count_bytes, count_lines, count_words, count_chars),
        None => count_stats(None, count_bytes, count_lines, count_words, count_chars),
    };

    let result_string = format!(
        "{}{}{}{}",
        if count_lines {
            format!("{:>8} ", lines)
        } else {
            "".to_string()
        },
        if count_words {
            format!("{:>8} ", words)
        } else {
            "".to_string()
        },
        if count_chars {
            format!("{:>8} ", chars)
        } else {
            "".to_string()
        },
        if count_bytes {
            format!("{:>8}", bytes)
        } else {
            "".to_string()
        }
    )
        .trim_end()
        .to_string();

    if let Some(f) = filename {
        println!("{} {}", result_string, f);
    } else {
        println!("{}", result_string);
    }
}

fn count_stats(
    filename: Option<&str>,
    count_bytes: bool,
    count_lines: bool,
    count_words: bool,
    count_chars: bool
) -> (usize, usize, usize, usize) {
    let contents = match filename {
        Some(f) => fs::read_to_string(f).expect("Error reading file"),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).expect("Error reading from stdin");
            buffer
        }
    };

    let lines = if count_lines { contents.lines().count() } else { 0 };
    let words = if count_words { contents.split_whitespace().count() } else { 0 };
    let chars = if count_chars { contents.chars().count() } else { 0 };
    let bytes = if count_bytes { contents.as_bytes().len() } else { 0 };

    (lines, words, chars, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_count() {
        let filename = "test.txt";
        let expected_line_count = 7145;
        let (actual_line_count, _, _, _) = count_stats(Some(filename), false, true, false, false);
        assert_eq!(expected_line_count, actual_line_count);
    }

    #[test]
    fn test_word_count() {
        let filename = "test.txt";
        let expected_count = 58164;
        let (_, actual_count, _, _) = count_stats(Some(filename), false, false, true, false);
        assert_eq!(expected_count, actual_count);
    }

    #[test]
    fn test_byte_count() {
        let filename = "test.txt";
        let expected_count = 342190;
        let (_, _, actual_count, _) = count_stats(Some(filename), true, false, false, false);
        assert_eq!(expected_count, actual_count);
    }

    // #[test]
    // fn test_char_count() {
    //     let filename = "test.txt";
    //     let expected_count = 339292;
    //     let (_, _, _, actual_count) = count_stats(Some(filename), true, false, false, true);
    //     assert_eq!(expected_count, actual_count);
    // }
}
