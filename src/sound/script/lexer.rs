#[derive(Debug, Clone)]
pub(crate) enum Token {
    OpenBrace {
        line: usize,
        column: usize,
    },
    CloseBrace {
        line: usize,
        column: usize,
    },
    Identifier {
        value: String,
        line: usize,
        column: usize,
    },
    String {
        value: String,
        line: usize,
        column: usize,
    },
    Keyword {
        value: String,
        line: usize,
        column: usize,
    },
    Number {
        value: i32,
        line: usize,
        column: usize,
    },
}

impl Token {
    pub(crate) fn line_column(&self) -> (usize, usize) {
        match self {
            Token::OpenBrace { line, column }
            | Token::CloseBrace { line, column }
            | Token::Identifier { line, column, .. }
            | Token::String { line, column, .. }
            | Token::Keyword { line, column, .. }
            | Token::Number { line, column, .. } => (*line, *column),
        }
    }
}

pub(crate) fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut line = 1;
    let mut column = 1;

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
            chars.next();
        } else if c == '#' {
            // Consume characters until newline.
            while let Some(&c) = chars.peek() {
                if c == '\n' {
                    break;
                }
                chars.next();
            }
        } else if c.is_ascii_digit() {
            let mut s = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    s.push(c);
                    chars.next();
                    column += 1;
                } else {
                    break;
                }
            }
            // Check the next character.
            if let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '-' {
                    // If the next character is a letter or a hyphen, treat the
                    // whole sequence as a string.
                    s.push(c);
                    chars.next();
                    column += 1;
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '-' {
                            s.push(c);
                            chars.next();
                            column += 1;
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::String {
                        value: s,
                        line,
                        column,
                    });
                } else {
                    // If the next character is not a letter or a hyphen, treat
                    // the sequence as a number.
                    tokens.push(Token::Number {
                        value: s.parse().unwrap(),
                        line,
                        column,
                    });
                }
            } else {
                // If there is no next character, treat the sequence as a
                // number.
                tokens.push(Token::Number {
                    value: s.parse().unwrap(),
                    line,
                    column,
                });
            }
        } else if c.is_alphanumeric() || c == '-' {
            let mut s = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '-' {
                    s.push(c);
                    chars.next();
                    column += 1;
                } else {
                    break;
                }
            }
            if [
                "state",
                "start-state",
                "start-pattern",
                "sample",
                "pattern",
                "sequence",
                "state-table",
            ]
            .contains(&s.as_str())
            {
                tokens.push(Token::Keyword {
                    value: s.clone(),
                    line,
                    column,
                });

                if s == "sample" {
                    let Some(_) = chars.peek() else {
                        panic!(
                            "{}:{}: expected a string after 'sample', found 'EOF'",
                            line, column
                        );
                    };

                    // After a keyword "sample", expect a string.
                    while let Some(&c) = chars.peek() {
                        if c.is_whitespace() {
                            if c == '\n' {
                                panic!(
                                    "{}:{}: expected a string after 'sample', found '{}'",
                                    line, column, c
                                );
                            } else {
                                column += 1;
                            }
                            chars.next();
                        } else if c.is_alphanumeric() || c == '-' {
                            let mut s = String::new();
                            while let Some(&c) = chars.peek() {
                                if c.is_alphanumeric() || c == '-' {
                                    s.push(c);
                                    chars.next();
                                    column += 1;
                                } else {
                                    break;
                                }
                            }
                            tokens.push(Token::String {
                                value: s,
                                line,
                                column,
                            });
                            break;
                        }
                    }

                    let Some(_) = chars.peek() else {
                        panic!(
                            "{}:{}: expected an identifier after 'sample' string, found 'EOF'",
                            line, column
                        );
                    };

                    // After a string, expect an identifier.
                    while let Some(&c) = chars.peek() {
                        if c.is_whitespace() {
                            if c == '\n' {
                                panic!(
                                    "{}:{}: expected an identifier after 'sample' string, found '{}'",
                                    line, column, c
                                );
                            } else {
                                column += 1;
                            }
                            chars.next();
                        } else if c.is_alphanumeric() || c == '-' {
                            let mut s = String::new();
                            while let Some(&c) = chars.peek() {
                                if c.is_alphanumeric() || c == '-' {
                                    s.push(c);
                                    chars.next();
                                    column += 1;
                                } else {
                                    break;
                                }
                            }
                            tokens.push(Token::Identifier {
                                value: s,
                                line,
                                column,
                            });
                            break;
                        }
                    }
                }
            } else {
                tokens.push(Token::String {
                    value: s,
                    line,
                    column,
                });
            }
        } else if c == '{' {
            tokens.push(Token::OpenBrace { line, column });
            chars.next();
            column += 1;
        } else if c == '}' {
            tokens.push(Token::CloseBrace { line, column });
            chars.next();
            column += 1;
        } else {
            panic!("{}:{}: unexpected '{}'", line, column, c);
        }
    }

    tokens
}
