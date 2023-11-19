use clap::Parser;

#[derive(clap::Parser)]
struct Args {
    input: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();
    println!("Input file {}", args.input.display());

    let example_json = "{ \"name\": \"John Doe\", \"age\": 43 }";

    let mut tokens = Vec::new();
    for mut i in 0..example_json.len() {
        let token = next(&example_json, i);
        match token {
            Ok(token) => {
                i = token.end;
                tokens.push(token);
            }
            Err(description) => {
                panic!("Parsing failed with error: {}", description)
            }
        }
    }
}

fn make_simple_token(token_type: TokenType, index: usize) -> Result<Token, String> {
    return Ok(Token {
        token_type: token_type,
        start: index,
        end: index + 1,
    });
}

fn next(data: &str, start_index: usize) -> Result<Token, String> {
    let c = data.chars().nth(start_index).unwrap();
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
        '0'..='9' => tokenize_number(&data, start_index),
        _ => panic!("Ill formed Json at position {}", start_index),
    }
}

fn tokenize_string(data: &str, start_index: usize) -> Result<Token, String> {
    let first_char = data.chars().nth(start_index).unwrap();
    if first_char != '"' {
        return Err(format!("Ill formed Json at position {}, expected \", got {}",
            start_index, first_char
        ));
    }

    for i in start_index + 1..data.len() {
        let c = data.chars().nth(i).unwrap();
        match c {
            '"' => {
                let prev_char = data.chars().nth(i - 1).unwrap();
                if prev_char == '\\' {
                    continue;
                }
                return Ok(Token {
                    token_type: TokenType::String,
                    start: start_index,
                    end: i,
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

fn tokenize_number(data: &str, start_index: usize) -> Result<Token, String> {
    let mut integer = true;
    for i in start_index..data.len() {
        let c = data.chars().nth(i).unwrap();
        match c {
            '0'..='9' => continue,
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
                    end: i,
                })
            }
        }
    }
    return Err(format!(
        "EOF reached while parsing for number starting at {}",
        start_index
    ));
}

fn tokenize_literal(
    data: &str,
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
    if &data[start_index..start_index + literal.len()] == literal {
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

fn tokenize_true(data: &str, start_index: usize) -> Result<Token, String> {
    tokenize_literal(data, "true", TokenType::True, start_index)
}

fn tokenize_false(data: &str, start_index: usize) -> Result<Token, String> {
    tokenize_literal(data, "false", TokenType::False, start_index)
}

fn tokenize_null(data: &str, start_index: usize) -> Result<Token, String> {
    tokenize_literal(data, "null", TokenType::Null, start_index)
}

#[derive(Debug, PartialEq)]
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
