use std::iter::Peekable;
use std::str::Chars;
use std::collections::HashMap;
use std::slice::Iter;
use std::fs;
use std::env;
use std::io::{ self, Read };

#[derive(Debug, PartialEq)]
enum Token {
    CurlyOpen,
    CurlyClose,
    SquareOpen,
    SquareClose,
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Colon,
    Comma,
}

#[derive(Debug, PartialEq)]
enum JsonValue {
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

fn parse(tokens: &[Token]) -> Result<JsonValue, String> {
    let mut tokens_iter = tokens.iter().peekable();
    parse_value(&mut tokens_iter)
}

fn parse_value(tokens_iter: &mut Peekable<Iter<Token>>) -> Result<JsonValue, String> {
    match tokens_iter.next() {
        Some(Token::String(s)) => Ok(JsonValue::String(s.clone())),
        Some(Token::Number(n)) => Ok(JsonValue::Number(*n)),
        Some(Token::Boolean(b)) => Ok(JsonValue::Boolean(*b)),
        Some(Token::Null) => Ok(JsonValue::Null),
        Some(Token::CurlyOpen) => parse_object(tokens_iter),
        Some(Token::SquareOpen) => parse_array(tokens_iter),
        _ => Err("Unexpected token or format".to_string()),
    }
}

fn parse_object(tokens_iter: &mut Peekable<Iter<Token>>) -> Result<JsonValue, String> {
    let mut object = HashMap::new();
    while let Some(token) = tokens_iter.peek() {
        match token {
            Token::CurlyClose => {
                tokens_iter.next();
                break;
            }
            Token::String(key) => {
                tokens_iter.next(); // consume
                if tokens_iter.next() != Some(&Token::Colon) {
                    return Err("Expected colon in object".to_string());
                }
                let value = parse_value(tokens_iter)?;
                object.insert(key.clone(), value);
            }
            _ => {
                return Err("Expected string key or closing curly brace in object".to_string());
            }
        }
        match tokens_iter.peek() {
            Some(Token::Comma) => {
                tokens_iter.next();
            }
            Some(Token::CurlyClose) => {
                continue;
            }
            _ => {
                return Err("Expected comma or closing curly brace in object".to_string());
            }
        }
    }
    Ok(JsonValue::Object(object))
}

fn parse_array(tokens_iter: &mut Peekable<Iter<Token>>) -> Result<JsonValue, String> {
    let mut array = Vec::new();
    while let Some(token) = tokens_iter.peek() {
        match token {
            Token::SquareClose => {
                tokens_iter.next();
                break;
            }
            _ => {
                let value = parse_value(tokens_iter)?;
                array.push(value);
            }
        }
        match tokens_iter.peek() {
            Some(Token::Comma) => {
                tokens_iter.next();
            }
            Some(Token::SquareClose) => {
                continue;
            }
            _ => {
                return Err("Expected comma or closing square bracket in array".to_string());
            }
        }
    }
    Ok(JsonValue::Array(array))
}

fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '{' => {
                tokens.push(Token::CurlyOpen);
                chars.next();
            }
            '}' => {
                tokens.push(Token::CurlyClose);
                chars.next();
            }
            '[' => {
                tokens.push(Token::SquareOpen);
                chars.next();
            }
            ']' => {
                tokens.push(Token::SquareClose);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            '"' => tokens.push(parse_string(&mut chars)),
            '0'..='9' | '-' => tokens.push(parse_number(&mut chars)),
            _ if c.is_whitespace() => {
                chars.next();
            }
            _ => {
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphabetic() {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "true" => tokens.push(Token::Boolean(true)),
                    "false" => tokens.push(Token::Boolean(false)),
                    "null" => tokens.push(Token::Null),
                    _ => {}
                }
            }
        }
    }

    tokens
}

fn parse_string(chars: &mut Peekable<Chars>) -> Token {
    let mut string = String::new();
    chars.next();
    while let Some(&ch) = chars.peek() {
        if ch == '"' {
            chars.next();
            break;
        }
        string.push(ch);
        chars.next();
    }
    Token::String(string)
}

fn parse_number(chars: &mut Peekable<Chars>) -> Token {
    let mut number = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_digit(10) || ch == '.' || ch == '-' {
            number.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    let number = number.parse::<f64>().unwrap();
    Token::Number(number)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let input = if args.len() > 1 {
        // file read
        fs::read_to_string(&args[1]).unwrap_or_else(|_| panic!("Failed to read file: {}", &args[1]))
    } else {
        // read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).expect("Failed to read from stdin");
        buffer
    };

    let tokens = lex(&input);
    match parse(&tokens) {
        Ok(json) => println!("Parsed JSON: {:?}", json),
        Err(e) => eprintln!("Error parsing JSON: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn is_json_file(entry: &fs::DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.ends_with(".json"))
            .unwrap_or(false)
    }

    #[test]
    fn test_json_files() {
        let test_files = fs::read_dir("tests/").expect("Failed to read tests directory");

        for entry in test_files {
            let entry = entry.expect("Failed to read directory entry");
            if is_json_file(&entry) {
                let file_name = entry.file_name().to_string_lossy().into_owned();
                let contents = fs::read_to_string(entry.path()).expect("Failed to read file");

                let tokens = lex(&contents);
                let result = parse(&tokens);

                if file_name.starts_with("Valid") {
                    assert!(
                        result.is_ok(),
                        "Failed to parse a file that should be valid: {}",
                        file_name
                    );
                } else if file_name.starts_with("Invalid") {
                    assert!(
                        result.is_err(),
                        "Incorrectly parsed a file that should be invalid: {}",
                        file_name
                    );
                }
            }
        }
    }
}
