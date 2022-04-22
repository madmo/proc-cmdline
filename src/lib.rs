/*
 * Copyright (c) 2022 Moritz Bitsch <moritz@h6t.eu>
 *
 * Permission to use, copy, modify, and distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */

use std::{iter::Peekable, slice::Iter};

/// A single commandlene parameter
///
/// First element is the parameter name, second is the optional value
pub type CmdlineParam = (Vec<u8>, Option<Vec<u8>>);

/// Parses a sequence of bytes in the format of the linux kernel cmdline (/proc/cmdline)
///
/// # Arguments
///
/// * `cmdline` - sequence of bytes to parse
pub fn parse(cmdline: &[u8]) -> Vec<CmdlineParam> {
    let mut result: Vec<CmdlineParam> = Vec::new();

    let mut iter = cmdline.iter().peekable();
    let mut token: Vec<Token> = Vec::new();

    let mut func: Option<ParseFn> = Some(ParseFn(parse_start));

    while let Some(f) = func {
        func = f.0(&mut iter, &mut token);
    }

    let mut param: Option<Vec<u8>> = None;

    for t in token {
        param = match t {
            Token::Name(p) => {
                if let Some(param) = param {
                    result.push((param, None));
                }

                Some(p)
            }
            Token::Value(v) => {
                result.push((param.unwrap(), Some(v)));
                None
            }
        };
    }

    if let Some(param) = param {
        result.push((param, None));
    }

    result
}

struct ParseFn(fn(chars: &mut Peekable<Iter<u8>>, token: &mut Vec<Token>) -> Option<ParseFn>);

enum Token {
    Name(Vec<u8>),
    Value(Vec<u8>),
}

fn parse_start(chars: &mut Peekable<Iter<u8>>, token: &mut Vec<Token>) -> Option<ParseFn> {
    let mut param: Vec<u8> = Vec::new();
    let mut in_quote = false;
    let mut has_value = false;
    param.clear();

    if let Some(&&c) = chars.peek() {
        if c == b'"' {
            in_quote = true;
            chars.next();
        }
    } else {
        return None;
    }

    for &c in chars {
        if c == b'=' {
            has_value = true;
            break;
        } else if isspace(c) && !in_quote {
            break;
        } else if c == b'"' {
            in_quote = !in_quote;
        } else {
            param.push(c);
        }
    }

    if !param.is_empty() {
        token.push(Token::Name(param));
    }

    if has_value {
        Some(ParseFn(parse_value))
    } else {
        Some(ParseFn(parse_start))
    }
}

fn parse_value(chars: &mut Peekable<Iter<u8>>, token: &mut Vec<Token>) -> Option<ParseFn> {
    let mut value: Vec<u8> = Vec::new();
    let mut in_quote = false;

    for &c in chars {
        if isspace(c) && !in_quote {
            break;
        } else if c == b'"' {
            in_quote = !in_quote;
        } else {
            value.push(c);
        }
    }

    token.push(Token::Value(value));

    Some(ParseFn(parse_start))
}

fn isspace(c: u8) -> bool {
    if c == b' ' {
        return true;
    }

    if c == 0x0c {
        return true;
    }

    if c == b'\n' {
        return true;
    }

    if c == b'\r' {
        return true;
    }

    if c == b'\t' {
        return true;
    }

    if c == 0x0b {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple_parse() {
        assert_eq!(
            crate::parse(b"foo bar"),
            vec![(b"foo".to_vec(), None), (b"bar".to_vec(), None)]
        );
    }

    #[test]
    fn empty() {
        assert_eq!(crate::parse(b""), vec![]);
    }

    #[test]
    fn parmaeters() {
        assert_eq!(
            crate::parse(b"foo= bar=baz"),
            vec![
                (b"foo".to_vec(), Some(vec![])),
                (b"bar".to_vec(), Some(b"baz".to_vec()))
            ]
        );
    }

    #[test]
    fn quoted() {
        assert_eq!(
            crate::parse(b"\"f o\" \"b ar\"=\"ba z\""),
            vec![
                (b"f o".to_vec(), None),
                (b"b ar".to_vec(), Some(b"ba z".to_vec()))
            ]
        );
    }
}
