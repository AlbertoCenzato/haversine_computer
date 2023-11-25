use clap::Parser;

#[derive(clap::Parser)]
struct Args {
    input: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();
    println!("Input file {}", args.input.display());

    //let example_json = "{ \"name\": \"John Doe\", \"age\": 43 }";
    // read input file to string
    let example_json = std::fs::read_to_string(args.input).unwrap();
    let json = example_json.chars().collect::<Vec<char>>();

    let start = std::time::Instant::now();

    println!("Tokens: ");
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < example_json.len() {
        let token = next(&json, i);
        match token {
            Ok(token) => {
                i = token.end;
                //println!(
                //    "{:?}, value: {}",
                //    &token,
                //    example_json[token.start..token.end].to_string()
                //);
                tokens.push(token);
            }
            Err(description) => {
                panic!("Parsing failed with error: {}", description)
            }
        }
    }

    let duration = start.elapsed();
    println!("Time elapsed in while loop is: {:?}", duration);
}

fn make_simple_token(token_type: TokenType, index: usize) -> Result<Token, String> {
    return Ok(Token {
        token_type: token_type,
        start: index,
        end: index + 1,
    });
}

enum Status {
    None,
    InString(usize),
    InNumber(usize),
    InTrue(usize),
    InFalse(usize),
    InNull(usize),
    Error(String),
}

fn next(data: &[char], start_index: usize) -> Result<Token, String> {
    let c = data[start_index];
    match c {
        '{' => make_simple_token(TokenType::LeftCurlyBracket, start_index),
        '}' => make_simple_token(TokenType::RightCurlyBracket, start_index),
        '[' => make_simple_token(TokenType::LeftSquareBracket, start_index),
        ']' => make_simple_token(TokenType::RightSquareBracket, start_index),
        ':' => make_simple_token(TokenType::Colon, start_index),
        ',' => make_simple_token(TokenType::Comma, start_index),
        '"' => tokenize_string(&data, start_index),
        't' => tokenize_true(&data, start_index),
        'f' => tokenize_false(&data, start_index),
        'n' => tokenize_null(&data, start_index),
        '0'..='9' | '-' => tokenize_number(&data, start_index),
        ' ' | '\n' | '\t' | '\r' => {
            let index = ignore_whitespace(&data, start_index);
            match index {
                Some(index) => next(&data, index),
                None => Err(format!("EOF reached while ignoring whitespace")),
            }
        }
        _ => Err(format!("Ill formed Json at position {}", start_index)),
    }
}

fn ignore_whitespace(data: &[char], start_index: usize) -> Option<usize> {
    for i in start_index..data.len() {
        match data[i] {
            ' ' | '\n' | '\t' | '\r' => continue,
            _ => return Some(i),
        }
    }
    return None;
}

fn tokenize_string(data: &[char], start_index: usize) -> Result<Token, String> {
    let first_char = data[start_index];
    if first_char != '"' {
        return Err(format!(
            "Ill formed Json at position {}, expected \", got {}",
            start_index, first_char
        ));
    }

    for i in start_index + 1..data.len() {
        let c = data[i];
        match c {
            '"' => {
                let prev_char = data[i - 1];
                if prev_char == '\\' {
                    continue;
                }
                return Ok(Token {
                    token_type: TokenType::String,
                    start: start_index,
                    end: i + 1,
                });
            }
            '\n' => {
                return Err(format!(
                    "Ill formed Json at position {}, expected \", got {}",
                    start_index, c
                ))
            }
            _ => continue,
        }
    }
    return Err(format!(
        "EOF reached while tokenizing string starting at {}",
        start_index
    ));
}

fn tokenize_number(data: &[char], start_index: usize) -> Result<Token, String> {
    let mut integer = true;
    for i in start_index..data.len() {
        let c = data[i];
        match c {
            '0'..='9' | '-' => continue,
            '.' => {
                if integer {
                    integer = false;
                    continue;
                } else {
                    return Err(format!(
                        "Ill formed Json at position {}, expected \", got {}",
                        start_index, c
                    ));
                }
            }
            _ => {
                return Ok(Token {
                    token_type: TokenType::Number,
                    start: start_index,
                    end: i + 1,
                })
            }
        }
    }
    return Err(format!(
        "EOF reached while parsing for number starting at {}",
        start_index
    ));
}

fn equals(data: &[char], literal: &str) -> bool {
    if data.len() != literal.len() {
        return false;
    }
    for i in 0..literal.len() {
        if data[i] != literal.as_bytes()[i] as char {
            return false;
        }
    }
    return true;
}

fn tokenize_literal(
    data: &[char],
    literal: &str,
    token_type: TokenType,
    start_index: usize,
) -> Result<Token, String> {
    if data.len() < start_index + literal.len() {
        return Err(format!(
            "EOF reached while parsing for '{}' literal starting at {}",
            literal, start_index
        ));
    }
    if equals(&data[start_index..start_index + literal.len()], literal) {
        return Ok(Token {
            token_type: token_type,
            start: start_index,
            end: start_index + literal.len(),
        });
    }
    return Err(format!(
        "Could not parse '{}' literal at position {}",
        literal, start_index
    ));
}

fn tokenize_true(data: &[char], start_index: usize) -> Result<Token, String> {
    tokenize_literal(data, "true", TokenType::True, start_index)
}

fn tokenize_false(data: &[char], start_index: usize) -> Result<Token, String> {
    tokenize_literal(data, "false", TokenType::False, start_index)
}

fn tokenize_null(data: &[char], start_index: usize) -> Result<Token, String> {
    tokenize_literal(data, "null", TokenType::Null, start_index)
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
enum TokenType {
    LeftCurlyBracket,
    RightCurlyBracket,
    LeftSquareBracket,
    RightSquareBracket,
    Colon,
    Comma,
    String,
    Number,
    True,
    False,
    Null,
}

#[derive(Debug)]
struct Token {
    token_type: TokenType,
    start: usize,
    end: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_true() {
        let token = tokenize_true("true", 0);
        let token = token.unwrap();
        assert_eq!(token.token_type, TokenType::True);
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 4);

        let token = tokenize_true("True", 0);
        assert!(token.is_err());
    }

    #[test]
    fn test_tokenize_false() {
        let token = tokenize_false("false", 0);
        let token = token.unwrap();
        assert_eq!(token.token_type, TokenType::False);
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 5);

        let token = tokenize_false(" false", 0);
        assert!(token.is_err());
    }

    #[test]
    fn test_tokenize_null() {
        let token = tokenize_null("null", 0);
        let token = token.unwrap();
        assert_eq!(token.token_type, TokenType::Null);
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 4);

        let token = tokenize_null("nul", 0);
        assert!(token.is_err());
    }

    #[test]
    fn test_tokenize_string() {
        let token = tokenize_string("\"hell15896_\t o\"", 0);
        let token = token.unwrap();
        assert_eq!(token.token_type, TokenType::String);
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 14);

        let token = tokenize_string("hello\"", 0);
        assert!(token.is_err());
        let token = tokenize_string("\"hel\nlo\"", 0);
        assert!(token.is_err());
        let token = tokenize_string("\"hello", 0);
        assert!(token.is_err());
    }
}
