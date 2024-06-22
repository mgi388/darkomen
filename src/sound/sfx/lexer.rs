#[derive(Debug, Clone)]
pub(crate) enum Token {
    Comment {
        _value: String,
        line: usize,
        column: usize,
    },
    BeginDirective {
        line: usize,
        column: usize,
    },
    Identifier {
        value: String,
        line: usize,
        column: usize,
    },
    SoundDividerDirective {
        line: usize,
        column: usize,
    },
    Define {
        line: usize,
        column: usize,
    },
}

impl Token {
    pub(crate) fn line_column(&self) -> (usize, usize) {
        match self {
            Token::Comment { line, column, .. }
            | Token::BeginDirective { line, column }
            | Token::Identifier { line, column, .. }
            | Token::Define { line, column, .. }
            | Token::SoundDividerDirective { line, column } => (*line, *column),
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
        } else if c == '/' {
            column += 1;
            chars.next();

            let Some(&c) = chars.peek() else {
                panic!("{}:{}: expected '/', found 'EOF'", line, column);
            };

            if c != '/' {
                panic!("{}:{}: expected '/', found '{}'", line, column, c);
            }

            column += 1;
            chars.next();

            let Some(&c) = chars.peek() else {
                tokens.push(Token::Comment {
                    _value: String::new(),
                    line,
                    column,
                });
                break;
            };

            column += 1;
            chars.next();

            // A regular comment.
            if c != '#' {
                let mut value = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '\n' {
                        break;
                    }
                    value.push(c);
                    chars.next();
                    column += 1;
                }

                tokens.push(Token::Comment {
                    _value: value,
                    line,
                    column,
                });
                continue;
            }

            // A directive.
            let Some(&c) = chars.peek() else {
                panic!("{}:{}: expected directive field, found 'EOF'", line, column);
            };

            // Check if the next character is '-' which means we expect the
            // sound divider. Note: There is no whitespace allowed between the
            // '#' and the first '-'.
            if c == '-' {
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() {
                        if c == '\n' {
                            if s != "-----" {
                                panic!("{}:{}: expected '-----', found '{}'", line, column, c);
                            }
                            break;
                        }
                        panic!("{}:{}: expected '-----', found '{}'", line, column, c);
                    } else if c == '-' {
                        if s == "-----" {
                            panic!("{}:{}: expected '\n', found '{}'", line, column, c);
                        }
                        s.push(c);
                        chars.next();
                        column += 1;
                    } else {
                        panic!("{}:{}: expected '-----', found '{}'", line, column, c);
                    }
                }

                if s != "-----" {
                    panic!("{}:{}: expected '-----', found 'EOF'", line, column);
                }

                tokens.push(Token::SoundDividerDirective { line, column });
                continue;
            }

            tokens.push(Token::BeginDirective { line, column });

            let mut field = String::new();
            let mut field_start_column = None;

            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    if c == '\n' {
                        line += 1;
                        column = 1;
                        panic!("{}:{}: expected directive field, found '\n'", line, column);
                    }
                    column += 1;
                    chars.next();
                    continue;
                }
                if c == ':' {
                    if field.is_empty() {
                        panic!("{}:{}: expected directive field, found ':'", line, column);
                    }
                    column += 1;
                    chars.next();
                    break;
                }
                if field_start_column.is_none() {
                    field_start_column = Some(column);
                }
                field.push(c);
                chars.next();
                column += 1;
            }

            if field.is_empty() {
                panic!("{}:{}: expected directive field, found 'EOF'", line, column);
            }

            tokens.push(Token::Identifier {
                value: field,
                line,
                column: field_start_column.unwrap_or(column),
            });

            let mut value = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    if c == '\n' {
                        break;
                    }
                    column += 1;
                    chars.next();
                    continue;
                }
                value.push(c);
                chars.next();
                column += 1;
            }

            tokens.push(Token::Identifier {
                value,
                line,
                column,
            });
        } else if c == '#' {
            column += 1;
            chars.next();

            let mut s = String::new();
            let Some(&c) = chars.peek() else {
                panic!("{}:{}: expected 'define', found 'EOF'", line, column);
            };
            if c.is_alphanumeric() {
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() {
                        s.push(c);
                        chars.next();
                        column += 1;
                    } else {
                        break;
                    }
                }
            } else {
                panic!("{}:{}: expected 'define', found '{}'", line, column, c,);
            }
            if s != "define" {
                panic!("{}:{}: expected 'define', found 'EOF'", line, column);
            }
            tokens.push(Token::Define { line, column });

            let mut sfx_name = String::new();
            let mut name_start_column = None;

            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    if c == '\n' {
                        panic!("{}:{}: expected name, found '{}'", line, column, c);
                    }
                    column += 1;
                    chars.next();
                } else if c.is_alphanumeric() || c == '_' {
                    if name_start_column.is_none() {
                        name_start_column = Some(column);
                    }

                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            sfx_name.push(c);
                            chars.next();
                            column += 1;
                        } else {
                            break;
                        }
                    }
                    break;
                } else {
                    panic!("{}:{}: expected name, found '{}'", line, column, c,);
                }
            }

            tokens.push(Token::Identifier {
                value: sfx_name,
                line,
                column: name_start_column.unwrap_or(column),
            });

            let mut sfx_id = String::new();
            let mut id_start_column = None;

            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    if c == '\n' {
                        panic!("{}:{}: expected ID, found '{}'", line, column, c);
                    }
                    column += 1;
                    chars.next();
                } else if c.is_ascii_digit() {
                    if id_start_column.is_none() {
                        id_start_column = Some(column);
                    }

                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() {
                            sfx_id.push(c);
                            chars.next();
                            column += 1;
                        } else {
                            break;
                        }
                    }
                    break;
                } else {
                    panic!("{}:{}: expected ID, found '{}'", line, column, c,);
                }
            }

            tokens.push(Token::Identifier {
                value: sfx_id,
                line,
                column: id_start_column.unwrap_or(column),
            });
        } else {
            panic!("{}:{}: unexpected '{}'", line, column, c);
        }
    }

    tokens
}
