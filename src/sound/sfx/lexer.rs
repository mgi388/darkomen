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
                panic!("{line}:{column}: expected '/', found 'EOF'");
            };

            if c != '/' {
                panic!("{line}:{column}: expected '/', found '{c}'");
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
                panic!("{line}:{column}: expected directive field, found 'EOF'");
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
                                panic!("{line}:{column}: expected '-----', found '{c}'");
                            }
                            break;
                        }
                        panic!("{line}:{column}: expected '-----', found '{c}'");
                    } else if c == '-' {
                        if s == "-----" {
                            panic!("{line}:{column}: expected '\n', found '{c}'");
                        }
                        s.push(c);
                        chars.next();
                        column += 1;
                    } else {
                        panic!("{line}:{column}: expected '-----', found '{c}'");
                    }
                }

                if s != "-----" {
                    panic!("{line}:{column}: expected '-----', found 'EOF'");
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
                        panic!("{line}:{column}: expected directive field, found '\n'");
                    }
                    column += 1;
                    chars.next();
                    continue;
                }
                if c == ':' {
                    if field.is_empty() {
                        panic!("{line}:{column}: expected directive field, found ':'");
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
                panic!("{line}:{column}: expected directive field, found 'EOF'");
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
                panic!("{line}:{column}: expected 'define', found 'EOF'");
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
                panic!("{line}:{column}: expected 'define', found '{c}'");
            }
            if s != "define" {
                panic!("{line}:{column}: expected 'define', found 'EOF'");
            }
            tokens.push(Token::Define { line, column });

            let mut sfx_name = String::new();
            let mut name_start_column = None;

            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    if c == '\n' {
                        panic!("{line}:{column}: expected name, found '{c}'");
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
                    panic!("{line}:{column}: expected name, found '{c}'");
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
                        panic!("{line}:{column}: expected ID, found '{c}'");
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
                    panic!("{line}:{column}: expected ID, found '{c}'");
                }
            }

            tokens.push(Token::Identifier {
                value: sfx_id,
                line,
                column: id_start_column.unwrap_or(column),
            });
        } else {
            panic!("{line}:{column}: unexpected '{c}'");
        }
    }

    tokens
}
