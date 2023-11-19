use clap::Parser;

#[derive(clap::Parser)]
struct Args {
    input: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();
    println!("Input file {}", args.input.display());

    let example_json = "{ \"name\": \"John Doe\", \"age\": 43 }";

    let tokens = Vec<Token>();
    for i in 0..example_json.len() {
        let (consumed_chars, token) = next(&example_json, i);
        if let Some(token) = token {
            tokens.push(token);
        }
        i += consumed_chars;
    }
}

fn next(data: &str, start_index: usize) -> (usize, Option<Token>) {
    let c = data.chars().nth(start_index).unwrap();
    match c {
            '{' => {
                let token = Token{token_type: TokenType::LeftCurlyBracket, start: start_index, end: start_index+1};
                return (1, Some(token))
            }
            '}' => {
                let token = Token{token_type: TokenType::RightCurlyBracket, start: start_index, end: start_index+1};
                return (1, Some(token));
            }
            '[' => {
                let token = Token{token_type: TokenType::LeftSquareBracket, start: start_index, end: start_index+1};
                return (1, Some(token));
            }
            ']' => {
                let token = Token{token_type: TokenType::RightSquareBracket, start: start_index, end: start_index+1};
                return (1, Some(token));
            }
            ':' => {
                let token = Token{token_type: TokenType::Colon, start: start_index, end: start_index+1};
                return (1, Some(token));
            }
            ',' => {
                let token = Token{token_type: TokenType::Comma, start: start_index, end: start_index+1};
                return (1, Some(token));
            }
            '"' => {
                return tokenize_string(&data, start_index);
            }
            't' => {
                return tokenize_true(&data, start_index);
            }
            'f' => {
                return tokenize_false(&data, start_index);
            }
            'n' => {
                return tokenize_null(&data, start_index);
            }
            '0'..='9' => {
                return tokenize_number(&data, start_index);
            }
            _ => { panic!("Ill formed Json at position {}", i);
        }
}
}

fn tokenize_string(data: &str, start_index: usize) -> (usize, Option<Token>) {
    let first_char = data.chars().nth(start_index).unwrap(); 
    if first_char != '"' {
        panic!("Ill formed Json at position {}, expected \", got {}", start_index, first_char);
    }

    for i in start_index+1..data.len() {
        let c = data.chars().nth(i).unwrap();
        match c {
            '"' => {
                let token = Token{token_type: TokenType::String, start: start_index, end: i};
                return (i - start_index + 1, Some(token));
            },

        }
    }
    return (0, None); 
}

fn tokenize_number(data: &str, start_index: usize) -> (usize, Option<Token>) { return (0, None); }

fn tokenize_true(data: &str, start_index: usize) -> (usize, Option<Token>) { 
    if &data[start_index..start_index+4] == "true" {
        let token = Token{token_type: TokenType::True, start: start_index, end: start_index+4};
        return (4, Some(token));
    }
    return (0, None); 
}

fn tokenize_false(data: &str, start_index: usize) -> (usize, Option<Token>) { 
    if &data[start_index..start_index+5] == "false" {
        let token = Token{token_type: TokenType::False, start: start_index, end: start_index+5};
        return (5, Some(token));
    }
    return (0, None); 
}

fn tokenize_null(data: &str, start_index: usize) -> (usize, Option<Token>) { 
    if &data[start_index..start_index+4] == "null" {
        let token = Token{token_type: TokenType::Null, start: start_index, end: start_index+4};
        return (4, Some(token));
    }
    return (0, None); 
}


enum TokenType {
    LeftCurlyBracket,
    RightCurlyBracket,
    LeftSquareBracket,
    RightSquareBracket,
    Colon,
    Comma,
    String,
    Number,
    Quotes,
    True,
    False,
    Null,
}

struct Token {
    token_type: TokenType,
    start: usize,
    end: usize,
}
