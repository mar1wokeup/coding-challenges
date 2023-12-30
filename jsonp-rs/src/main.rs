use std::env;
use std::process;
use std::io::{ self, Read };

#[derive(Debug, PartialEq)]
enum Token {
    CurlyOpen,
    CurlyClose,
    String(String),
    Colon,
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
            '"' => {
                // Start of a string
                chars.next(); // Skip the opening quote
                let mut string = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next(); // Skip the closing quote
                        break;
                    } else {
                        string.push(ch);
                        chars.next();
                    }
                }
                tokens.push(Token::String(string));
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            _ if c.is_whitespace() => {
                chars.next();
            }
            _ => {
                chars.next();
            } // Skip unrecognized characters
        }
    }

    tokens
}

fn parse(tokens: &[Token]) -> bool {
    if tokens.len() < 4 {
        return false;
    }

    if tokens[0] != Token::CurlyOpen || tokens.last() != Some(&Token::CurlyClose) {
        return false;
    }

    let mut i = 1;
    while i < tokens.len() - 1 {
        match (&tokens[i], &tokens[i + 1], &tokens[i + 2]) {
            (Token::String(_), Token::Colon, Token::String(_)) => {
                i += 3;
            }
            _ => {
                return false;
            }
        }

        if i < tokens.len() - 1 {
            // Expect a comma here if there are more tokens
            i += 1;
        }
    }

    true
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() > 1 {
        args[1].clone()
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).expect("Failed to read from stdin");
        buffer
    };

    let tokens = lex(&input);
    let is_valid = parse(&tokens);

    if is_valid {
        println!("Valid JSON");
        std::process::exit(0);
    } else {
        println!("Invalid JSON");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_json_object() {
        let tokens = lex("{}");
        assert!(parse(&tokens));
    }

    #[test]
    fn test_simple_json_object() {
        let tokens = lex("{\"key\": \"value\"}");
        assert!(parse(&tokens));
    }

    #[test]
    fn test_invalid_json_object() {
        let tokens = lex("{key: \"value\"}");
        assert!(!parse(&tokens));
    }

    // Additional tests can be added here
}
