#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
    Int(i64),
    Float(f32),
    Str(String),
    Name(String),
    Punct(&'static str),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
    pub text: String,
}

const PUNCTS: [&str; 37] = [
    "+=", "-=", "*=", "/=", "&=", "|=", "==", "!=", "<=", ">=", "&&", "||", "{", "}", "(", ")",
    "[", "]", ";", ":", ",", ".", "=", "<", ">", "+", "-", "*", "/", "%", "!", "&", "|", "^",
    "~", "?", "@",
];

pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut pos = 0;
    let mut line = 1u32;

    while pos < bytes.len() {
        let c = bytes[pos];

        if c == b'\n' {
            line += 1;
            pos += 1;
            continue;
        }

        if c.is_ascii_whitespace() || !c.is_ascii() {
            pos += 1;
            continue;
        }

        if bytes[pos..].starts_with(b"//") {
            while pos < bytes.len() && bytes[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }

        if bytes[pos..].starts_with(b"/*") {
            pos += 2;
            while pos < bytes.len() && !bytes[pos..].starts_with(b"*/") {
                if bytes[pos] == b'\n' {
                    line += 1;
                }
                pos += 1;
            }
            pos += 2;
            continue;
        }

        let start = pos;
        let token_line = line;

        let tok = if c.is_ascii_alphabetic() || c == b'_' {
            while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                pos += 1;
            }
            Tok::Ident(source[start..pos].to_owned())
        } else if c.is_ascii_digit() || (c == b'.' && bytes.get(pos + 1).is_some_and(u8::is_ascii_digit)) {
            lex_number(source, &mut pos)?
        } else if c == b'"' || c == b'\'' {
            let (text, newlines) = lex_quoted(source, &mut pos, c)?;
            line += newlines;
            match c {
                b'"' => Tok::Str(text),
                _ => Tok::Name(text),
            }
        } else {
            let punct = PUNCTS
                .iter()
                .find(|p| bytes[pos..].starts_with(p.as_bytes()))
                .ok_or_else(|| format!("line {line}: unexpected character '{}'", c as char))?;
            pos += punct.len();
            Tok::Punct(punct)
        };

        tokens.push(Token {
            tok,
            line: token_line,
            text: source[start..pos].to_owned(),
        });
    }

    tokens.push(Token {
        tok: Tok::Eof,
        line,
        text: String::new(),
    });

    Ok(tokens)
}

fn lex_number(source: &str, pos: &mut usize) -> Result<Tok, String> {
    let bytes = source.as_bytes();
    let start = *pos;

    if bytes[start..].starts_with(b"0x") || bytes[start..].starts_with(b"0X") {
        *pos += 2;
        while *pos < bytes.len() && bytes[*pos].is_ascii_hexdigit() {
            *pos += 1;
        }
        let value = i64::from_str_radix(&source[start + 2..*pos], 16).map_err(|e| e.to_string())?;
        return Ok(Tok::Int(value));
    }

    let mut is_float = false;
    while *pos < bytes.len() && (bytes[*pos].is_ascii_digit() || bytes[*pos] == b'.') {
        is_float |= bytes[*pos] == b'.';
        *pos += 1;
    }

    let digits = &source[start..*pos];

    if *pos < bytes.len() && (bytes[*pos] == b'f' || bytes[*pos] == b'F') {
        *pos += 1;
        is_float = true;
    }

    match is_float {
        true => digits
            .parse::<f32>()
            .map(Tok::Float)
            .map_err(|e| format!("bad float '{digits}': {e}")),
        false => digits
            .parse::<i64>()
            .map(Tok::Int)
            .map_err(|e| format!("bad int '{digits}': {e}")),
    }
}

fn lex_quoted(source: &str, pos: &mut usize, quote: u8) -> Result<(String, u32), String> {
    let bytes = source.as_bytes();
    let mut text = Vec::new();
    let mut newlines = 0;
    *pos += 1;

    while *pos < bytes.len() && bytes[*pos] != quote {
        let c = bytes[*pos];
        if c == b'\n' {
            newlines += 1;
        }

        if c == b'\\' && bytes.get(*pos + 1) == Some(&quote) {
            *pos += 1;
            text.push(quote);
        } else {
            text.push(c);
        }
        *pos += 1;
    }

    if *pos >= bytes.len() {
        return Err("unterminated literal".to_owned());
    }
    *pos += 1;

    Ok((String::from_utf8_lossy(&text).into_owned(), newlines))
}
