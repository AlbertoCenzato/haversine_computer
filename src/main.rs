use clap::Parser;

#[derive(clap::Parser)]
struct Args {
    input: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();
    println!("Input file {}", args.input.display());

    // TODO(alberto): read file in chunks
    let example_json = std::fs::read_to_string(args.input).unwrap();
    // TODO(alberto): remove this copy
    let json = example_json.chars().collect::<Vec<char>>();

    let start = std::time::Instant::now();

    println!("Tokens: ");
    let mut tokens = Vec::new();
    let mut i = 0;
    let mut state = State::None;
    while i < json.len() {
        let (new_state, token, consumed_chars) = next(json[i], i, state);
        match new_state {
            State::Error(error) => panic!("Parsing failed with error: {}", error),
            _ => {
                state = new_state;
                i += consumed_chars;
            }
        }

        match token {
            Some(t) => {
                //println!("{:?}, value {}", t, &example_json[t.start..t.end]);
                tokens.push(t)
            }
            None => (),
        }
    }

    let duration = start.elapsed();
    println!("Time elapsed in while loop is: {:?}", duration);
    for t in tokens {
        println!("{:?}, value {}", t, &example_json[t.start..t.end]);
    }
}

fn make_simple_token(token_type: TokenType, index: usize) -> (State, Option<Token>, usize) {
    let token = Token {
        token_type: token_type,
        start: index,
        end: index + 1,
    };
    return (State::None, Some(token), 1);
}

struct StringState {
    start_index: usize,
    escape: bool,
}

struct NumberState {
    start_index: usize,
    decimal_point: bool,
}

struct LiteralState {
    start_index: usize,
    current_index: usize,
}

enum State {
    None,
    InString(StringState),
    InNumber(NumberState),
    InTrue(LiteralState),
    InFalse(LiteralState),
    InNull(LiteralState),
    Error(String),
}

fn next(c: char, index: usize, state: State) -> (State, Option<Token>, usize) {
    match state {
        State::None => match c {
            '{' => make_simple_token(TokenType::LeftCurlyBracket, index),
            '}' => make_simple_token(TokenType::RightCurlyBracket, index),
            '[' => make_simple_token(TokenType::LeftSquareBracket, index),
            ']' => make_simple_token(TokenType::RightSquareBracket, index),
            ':' => make_simple_token(TokenType::Colon, index),
            ',' => make_simple_token(TokenType::Comma, index),
            '"' => (
                State::InString(StringState {
                    start_index: index,
                    escape: false,
                }),
                None,
                1,
            ),
            't' => (
                State::InTrue(LiteralState {
                    start_index: index,
                    current_index: 1,
                }),
                None,
                1,
            ),
            'f' => (
                State::InFalse(LiteralState {
                    start_index: index,
                    current_index: 1,
                }),
                None,
                1,
            ),
            'n' => (
                State::InNull(LiteralState {
                    start_index: index,
                    current_index: 1,
                }),
                None,
                1,
            ),
            '0'..='9' | '-' => (
                State::InNumber(NumberState {
                    start_index: index,
                    decimal_point: false,
                }),
                None,
                1,
            ),
            ' ' | '\n' | '\t' | '\r' => (State::None, None, 1),
            _ => (
                State::Error(format!("Ill formed Json at position {}", index)),
                None,
                0,
            ),
        },
        State::InString(string_state) => tokenize_string(c, index, string_state),
        State::InNumber(number_state) => tokenize_number(c, index, number_state),
        State::InTrue(literal_state) => tokenize_literal(c, index, TokenType::True, literal_state),
        State::InFalse(literal_state) => {
            tokenize_literal(c, index, TokenType::False, literal_state)
        }
        State::InNull(literal_state) => tokenize_literal(c, index, TokenType::Null, literal_state),
        State::Error(error) => (State::Error(error), None, 0),
    }
}

fn tokenize_string(c: char, index: usize, mut state: StringState) -> (State, Option<Token>, usize) {
    if state.escape {
        state.escape = false;
        return (State::InString(state), None, 1);
    }

    match c {
        '"' => {
            let token = Token {
                token_type: TokenType::String,
                start: state.start_index,
                end: index + 1,
            };
            (State::None, Some(token), 1)
        }
        '\n' => {
            return (
                State::Error(format!(
                    "Ill formed Json at position {}, expected \", got {}",
                    index, c
                )),
                None,
                0,
            )
        }
        _ => return (State::InString(state), None, 1),
    }
}

fn tokenize_number(c: char, index: usize, mut state: NumberState) -> (State, Option<Token>, usize) {
    match c {
        '0'..='9' => (State::InNumber(state), None, 1),
        '.' => {
            if state.decimal_point {
                return (
                    State::Error(format!(
                        "Ill formed Json at position {}, expected \", got {}",
                        index, c
                    )),
                    None,
                    0,
                );
            } else {
                state.decimal_point = true;
                return (State::InNumber(state), None, 1);
            }
        }
        _ => {
            let token = Token {
                token_type: TokenType::Number,
                start: state.start_index,
                end: index,
            };
            return (State::None, Some(token), 0);
        }
    }
}

fn tokenize_literal(
    c: char,
    index: usize,
    token_type: TokenType,
    mut state: LiteralState,
) -> (State, Option<Token>, usize) {
    let literal = match token_type {
        TokenType::True => "true",
        TokenType::False => "false",
        TokenType::Null => "null",
        _ => panic!("Invalid token type"),
    };
    let expected = literal.as_bytes()[state.current_index] as char;
    if c != expected {
        return (
            State::Error(format!(
                "Failed tokenizing literal {} at position {}, expected {}, found {}",
                &literal,
                state.start_index + state.current_index,
                expected,
                c
            )),
            None,
            0,
        );
    }
    if state.current_index == literal.len() - 1 {
        let token = Token {
            token_type: token_type,
            start: state.start_index,
            end: index,
        };
        return (State::None, Some(token), 1);
    }

    state.current_index += 1;
    return match token_type {
        TokenType::True => (State::InTrue(state), None, 1),
        TokenType::False => (State::InFalse(state), None, 1),
        TokenType::Null => (State::InNull(state), None, 1),
        _ => panic!("Invalid token type"),
    };
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
    fn test_tokenize_true() {}

    #[test]
    fn test_tokenize_false() {}

    #[test]
    fn test_tokenize_null() {}

    #[test]
    fn test_tokenize_string() {}
}
