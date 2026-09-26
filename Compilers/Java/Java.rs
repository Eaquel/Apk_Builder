use crate::caps::Capability;
use crate::compiler::{
    Compiled, Compiler, Expected, Identity, Kind, Plan, Probe, Reproducibility, Request, Session,
};
use crate::diag::{Diagnostic, Severity};
use crate::plugin::{Contract, Version};
use crate::FailureClass;

pub const ORIGIN: &str = "plugin.java";

pub const CLASS_MAJOR: u16 = 69;
pub const CLASS_MINOR: u16 = 0;

pub const LANGUAGE_RELEASE: &str = "25";

fn fail(code: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        code,
        Severity::Error,
        FailureClass::UserError,
        ORIGIN,
        message,
    )
}

pub(crate) fn at(code: &str, line: u32, column: u32, message: impl Into<String>) -> Diagnostic {
    fail(code, message)
        .with_context(format!("Line {line}, column {column}"))
        .with_location(crate::diag::Location::at("", line, column))
}

fn at_line(code: &str, line: u32, message: impl Into<String>) -> Diagnostic {
    at(code, line, 1, message)
}

fn unsupported(line: u32, column: u32, what: &str) -> Diagnostic {
    at(
        "EJ900",
        line,
        column,
        format!("{what} is not compiled here."),
    )
    .with_suggestion(
        "This compiler handles a subset of Java, and refuses the rest rather than \
         mis-handling it. What it does and does not take is written at the top of \
         Compilers/Java.rs.",
    )
}

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    Identifier(String),
    Keyword(&'static str),
    Int(i64),
    Long(i64),
    Float(f64),
    Double(f64),
    Char(u16),
    Str(String),
    True,
    False,
    Null,
    Punctuation(&'static str),
    End,
}

impl Token {
    fn describe(&self) -> String {
        match self {
            Token::Identifier(name) => format!("the name `{name}`"),
            Token::Keyword(word) => format!("`{word}`"),
            Token::Int(value) => format!("`{value}`"),
            Token::Long(value) => format!("`{value}L`"),
            Token::Float(value) => format!("`{value}f`"),
            Token::Double(value) => format!("`{value}`"),
            Token::Char(value) => format!("a character literal `{value}`"),
            Token::Str(text) => format!("the text \"{text}\""),
            Token::True => "`true`".to_string(),
            Token::False => "`false`".to_string(),
            Token::Null => "`null`".to_string(),
            Token::Punctuation(mark) => format!("`{mark}`"),
            Token::End => "the end of the file".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Spelled {
    pub token: Token,
    pub line: u32,
    pub column: u32,
}

const KEYWORDS: &[&str] = &[
    "abstract",
    "assert",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extends",
    "final",
    "finally",
    "float",
    "for",
    "goto",
    "if",
    "implements",
    "import",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "try",
    "void",
    "volatile",
    "while",
];

const MARKS: &[&str] = &[
    ">>>=", "<<=", ">>=", ">>>", "...", "->", "::", "++", "--", "&&", "||", "==", "!=", "<=", ">=",
    "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<", ">>", "{", "}", "(", ")", "[", "]", ";",
    ",", ".", "=", ">", "<", "!", "~", "?", ":", "+", "-", "*", "/", "&", "|", "^", "%", "@",
];

pub struct Lexer<'a> {
    source: &'a [u8],
    at: usize,
    line: u32,
    column: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Lexer<'a> {
        Lexer {
            source: source.as_bytes(),
            at: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self, ahead: usize) -> u8 {
        *self.source.get(self.at + ahead).unwrap_or(&0)
    }

    fn bump(&mut self) -> u8 {
        let byte = self.peek(0);
        self.at += 1;
        if byte == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        byte
    }

    fn skip_trivia(&mut self) -> Result<(), Diagnostic> {
        loop {
            match (self.peek(0), self.peek(1)) {
                (b' ' | b'\t' | b'\r' | b'\n' | 0x0c, _) => {
                    self.bump();
                }
                (b'/', b'/') => {
                    while self.at < self.source.len() && self.peek(0) != b'\n' {
                        self.bump();
                    }
                }
                (b'/', b'*') => {
                    let (line, column) = (self.line, self.column);
                    self.bump();
                    self.bump();
                    loop {
                        if self.at >= self.source.len() {
                            return Err(at(
                                "EJ001",
                                line,
                                column,
                                "A comment was opened and never closed.",
                            ));
                        }
                        if self.peek(0) == b'*' && self.peek(1) == b'/' {
                            self.bump();
                            self.bump();
                            break;
                        }
                        self.bump();
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    pub fn tokens(mut self) -> Result<Vec<Spelled>, Diagnostic> {
        let mut out = Vec::new();
        loop {
            self.skip_trivia()?;
            let (line, column) = (self.line, self.column);
            if self.at >= self.source.len() {
                out.push(Spelled {
                    token: Token::End,
                    line,
                    column,
                });
                return Ok(out);
            }
            let token = self.one(line, column)?;
            out.push(Spelled {
                token,
                line,
                column,
            });
        }
    }

    fn one(&mut self, line: u32, column: u32) -> Result<Token, Diagnostic> {
        let byte = self.peek(0);

        if byte == b'"' {
            if self.peek(1) == b'"' && self.peek(2) == b'"' {
                return self.text_block(line, column);
            }
            return self.text(line, column);
        }
        if byte == b'\'' {
            return self.character(line, column);
        }
        if byte.is_ascii_digit() || (byte == b'.' && self.peek(1).is_ascii_digit()) {
            return self.number(line, column);
        }
        if byte == b'_' || byte == b'$' || byte.is_ascii_alphabetic() || byte >= 0x80 {
            return Ok(self.word());
        }

        for mark in MARKS {
            let bytes = mark.as_bytes();
            if self.source[self.at..].starts_with(bytes) {
                for _ in 0..bytes.len() {
                    self.bump();
                }
                return Ok(Token::Punctuation(mark));
            }
        }

        Err(at(
            "EJ002",
            line,
            column,
            format!("`{}` is not part of Java.", byte as char),
        ))
    }

    fn word(&mut self) -> Token {
        let start = self.at;
        while {
            let byte = self.peek(0);
            byte == b'_' || byte == b'$' || byte.is_ascii_alphanumeric() || byte >= 0x80
        } {
            self.bump();
        }
        let text = String::from_utf8_lossy(&self.source[start..self.at]).into_owned();
        match text.as_str() {
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            _ => match KEYWORDS.iter().find(|word| **word == text) {
                Some(word) => Token::Keyword(word),
                None => Token::Identifier(text),
            },
        }
    }

    fn escape(&mut self, line: u32, column: u32) -> Result<u16, Diagnostic> {
        self.bump();
        let byte = self.bump();
        Ok(match byte {
            b'n' => 10,
            b't' => 9,
            b'b' => 8,
            b'r' => 13,
            b'f' => 12,
            b's' => 32,
            b'0'..=b'7' => {
                let mut value = u16::from(byte - b'0');
                while self.peek(0).is_ascii_digit() && self.peek(0) < b'8' && value < 32 {
                    value = value * 8 + u16::from(self.bump() - b'0');
                }
                value
            }
            b'\'' => 39,
            b'"' => 34,
            b'\\' => 92,
            b'u' => {
                while self.peek(0) == b'u' {
                    self.bump();
                }
                let mut value = 0u16;
                for _ in 0..4 {
                    let digit = (self.bump() as char).to_digit(16).ok_or_else(|| {
                        at("EJ003", line, column, "A \\u escape needs four hex digits.")
                    })?;
                    value = value * 16 + digit as u16;
                }
                value
            }
            other => {
                return Err(at(
                    "EJ004",
                    line,
                    column,
                    format!("`\\{}` is not an escape Java knows.", other as char),
                ))
            }
        })
    }

    fn text(&mut self, line: u32, column: u32) -> Result<Token, Diagnostic> {
        self.bump();
        let mut units: Vec<u16> = Vec::new();
        loop {
            match self.peek(0) {
                0 if self.at >= self.source.len() => {
                    return Err(at("EJ005", line, column, "A string was never closed."))
                }
                b'\n' => return Err(at("EJ005", line, column, "A string was never closed.")),
                b'"' => {
                    self.bump();
                    break;
                }
                b'\\' => units.push(self.escape(line, column)?),
                _ => {
                    let start = self.at;
                    self.bump();
                    while self.peek(0) >= 0x80 && self.peek(0) < 0xc0 {
                        self.bump();
                    }
                    let piece = String::from_utf8_lossy(&self.source[start..self.at]).into_owned();
                    units.extend(piece.encode_utf16());
                }
            }
        }
        Ok(Token::Str(crate::binary::modified_utf8::from_units(&units)))
    }

    fn text_block(&mut self, line: u32, column: u32) -> Result<Token, Diagnostic> {
        self.bump();
        self.bump();
        self.bump();

        while matches!(self.peek(0), b' ' | b'\t' | b'\x0c') {
            self.bump();
        }
        if self.peek(0) == b'\r' {
            self.bump();
        }
        if self.peek(0) != b'\n' {
            return Err(at(
                "EJ011",
                line,
                column,
                "A text block opens with `\"\"\"` and then a line break.",
            )
            .with_suggestion("Move the content to the next line."));
        }
        self.bump();

        let mut raw = String::new();
        loop {
            if self.at >= self.source.len() {
                return Err(at("EJ005", line, column, "A text block was never closed."));
            }
            if self.peek(0) == b'"' && self.peek(1) == b'"' && self.peek(2) == b'"' {
                self.bump();
                self.bump();
                self.bump();
                break;
            }
            if self.peek(0) == b'\\' {
                raw.push('\\');
                self.bump();
                if self.at < self.source.len() {
                    let start = self.at;
                    self.bump();
                    while self.peek(0) >= 0x80 && self.peek(0) < 0xc0 {
                        self.bump();
                    }
                    raw.push_str(&String::from_utf8_lossy(&self.source[start..self.at]));
                }
                continue;
            }
            let start = self.at;
            self.bump();
            while self.peek(0) >= 0x80 && self.peek(0) < 0xc0 {
                self.bump();
            }
            raw.push_str(&String::from_utf8_lossy(&self.source[start..self.at]));
        }

        let stripped = strip_incidental_whitespace(&raw);
        Ok(Token::Str(text_block_escapes(&stripped, line, column)?))
    }

    fn character(&mut self, line: u32, column: u32) -> Result<Token, Diagnostic> {
        self.bump();
        let value = if self.peek(0) == b'\\' {
            self.escape(line, column)?
        } else {
            let start = self.at;
            self.bump();
            while self.peek(0) >= 0x80 && self.peek(0) < 0xc0 {
                self.bump();
            }
            let piece = String::from_utf8_lossy(&self.source[start..self.at]).into_owned();
            piece.encode_utf16().next().unwrap_or(0)
        };
        if self.peek(0) != b'\'' {
            return Err(at(
                "EJ006",
                line,
                column,
                "A character literal holds exactly one character.",
            ));
        }
        self.bump();
        Ok(Token::Char(value))
    }

    fn number(&mut self, line: u32, column: u32) -> Result<Token, Diagnostic> {
        let start = self.at;
        let mut radix = 10u32;
        if self.peek(0) == b'0' && matches!(self.peek(1), b'x' | b'X') {
            radix = 16;
            self.bump();
            self.bump();
        } else if self.peek(0) == b'0' && matches!(self.peek(1), b'b' | b'B') {
            radix = 2;
            self.bump();
            self.bump();
        }
        let digits_from = self.at;

        let mut floating = false;
        loop {
            let byte = self.peek(0);
            if byte == b'_' {
                self.bump();
                continue;
            }
            if (byte as char).is_digit(radix) {
                self.bump();
                continue;
            }
            if radix == 10 && byte == b'.' && self.peek(1) != b'.' && !floating {
                floating = true;
                self.bump();
                continue;
            }
            if radix == 10 && matches!(byte, b'e' | b'E') {
                floating = true;
                self.bump();
                if matches!(self.peek(0), b'+' | b'-') {
                    self.bump();
                }
                continue;
            }
            break;
        }

        let digits: String = String::from_utf8_lossy(&self.source[digits_from..self.at])
            .chars()
            .filter(|c| *c != '_')
            .collect();

        let (digits, radix) = if radix == 10
            && !floating
            && digits.len() > 1
            && digits.starts_with('0')
            && !digits.contains('.')
        {
            if digits[1..].bytes().any(|one| !(b'0'..=b'7').contains(&one)) {
                return Err(at(
                    "EJ007",
                    line,
                    column,
                    "A number written with a leading zero is octal, and 8 and 9 are not \
                     octal digits.",
                )
                .with_suggestion("Drop the leading zero, or write it as 0x for hexadecimal."));
            }
            (digits[1..].to_string(), 8u32)
        } else {
            (digits, radix)
        };

        let suffix = self.peek(0);
        if matches!(suffix, b'l' | b'L') {
            self.bump();
            let value = parse_integer(&digits, radix, true).ok_or_else(|| {
                at(
                    "EJ008",
                    line,
                    column,
                    "That number is too large for a long.",
                )
            })?;
            return Ok(Token::Long(value));
        }
        if matches!(suffix, b'f' | b'F') {
            self.bump();
            let value: f64 = digits
                .parse()
                .map_err(|_| at("EJ009", line, column, "That is not a number this reads."))?;
            return Ok(Token::Float(value));
        }
        if matches!(suffix, b'd' | b'D') {
            self.bump();
            floating = true;
        }

        if floating {
            let value: f64 = digits
                .parse()
                .map_err(|_| at("EJ009", line, column, "That is not a number this reads."))?;
            return Ok(Token::Double(value));
        }

        let _ = start;
        let value = parse_integer(&digits, radix, false).ok_or_else(|| {
            at(
                "EJ010",
                line,
                column,
                "That number is too large for an int.",
            )
        })?;
        Ok(Token::Int(value))
    }
}

fn strip_incidental_whitespace(raw: &str) -> String {
    let normalised = raw.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = normalised.split('\n').collect();

    let closing_is_alone = lines
        .last()
        .is_some_and(|last| last.chars().all(char::is_whitespace));

    let mut margin = usize::MAX;
    for (index, line) in lines.iter().enumerate() {
        let blank = line.chars().all(char::is_whitespace);
        let last = index + 1 == lines.len();
        if blank && !(last && closing_is_alone) {
            continue;
        }
        let indent = if blank {
            line.chars().count()
        } else {
            line.chars().take_while(|one| one.is_whitespace()).count()
        };
        margin = margin.min(indent);
    }
    if margin == usize::MAX {
        margin = 0;
    }

    let mut kept: Vec<String> = lines
        .iter()
        .map(|line| {
            let body: String = line.chars().skip(margin).collect();
            body.trim_end().to_string()
        })
        .collect();
    if closing_is_alone && kept.len() > 1 {
        kept.pop();
        kept.push(String::new());
    }
    kept.join("\n")
}

fn text_block_escapes(text: &str, line: u32, column: u32) -> Result<String, Diagnostic> {
    let source: Vec<char> = text.chars().collect();
    let mut units: Vec<u16> = Vec::new();
    let mut cursor = 0usize;
    while cursor < source.len() {
        if source[cursor] != '\\' {
            let mut buffer = [0u16; 2];
            units.extend_from_slice(source[cursor].encode_utf16(&mut buffer));
            cursor += 1;
            continue;
        }
        cursor += 1;
        let Some(&marker) = source.get(cursor) else {
            return Err(at_end_of_escape(line, column));
        };
        cursor += 1;
        match marker {
            'n' => units.push(u16::from(b'\n')),
            't' => units.push(u16::from(b'\t')),
            'r' => units.push(u16::from(b'\r')),
            'b' => units.push(8),
            'f' => units.push(12),
            's' => units.push(u16::from(b' ')),
            '0'..='7' => {
                let mut value = u32::from(marker) - u32::from('0');
                let mut taken = 1;
                while taken < 3 {
                    match source.get(cursor) {
                        Some(next @ '0'..='7') if value * 8 + 7 <= 0o377 => {
                            value = value * 8 + (u32::from(*next) - u32::from('0'));
                            cursor += 1;
                            taken += 1;
                        }
                        _ => break,
                    }
                }
                units.push(value as u16);
            }
            '\'' | '"' | '\\' => {
                let mut buffer = [0u16; 2];
                units.extend_from_slice(marker.encode_utf16(&mut buffer));
            }

            '\n' => {}
            'u' => {
                while source.get(cursor) == Some(&'u') {
                    cursor += 1;
                }
                let mut value = 0u16;
                for _ in 0..4 {
                    let Some(digit) = source.get(cursor).and_then(|one| one.to_digit(16)) else {
                        return Err(at(
                            "EJ006",
                            line,
                            column,
                            "A `\\u` escape wants four hexadecimal digits.",
                        ));
                    };
                    value = value * 16 + digit as u16;
                    cursor += 1;
                }
                units.push(value);
            }
            other => {
                return Err(at(
                    "EJ006",
                    line,
                    column,
                    format!("`\\{other}` is not an escape Java has."),
                ))
            }
        }
    }
    Ok(crate::binary::modified_utf8::from_units(&units))
}

fn at_end_of_escape(line: u32, column: u32) -> Diagnostic {
    at(
        "EJ006",
        line,
        column,
        "A backslash was the last thing in the text block.",
    )
}

fn parse_integer(digits: &str, radix: u32, long: bool) -> Option<i64> {
    let value = u64::from_str_radix(digits, radix).ok()?;
    if long {
        return Some(value as i64);
    }
    if radix == 10 {
        if value > 2_147_483_648 {
            return None;
        }
        return Some(value as i32 as i64);
    }
    if value > u64::from(u32::MAX) {
        return None;
    }
    Some(value as u32 as i32 as i64)
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Written {
    Inferred,
    Void,
    Boolean,
    Byte,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,

    Named(String, Vec<Written>),

    Variable(usize, Box<Written>),

    OwnVariable(String, Box<Written>),
    Array(Box<Written>),
}

impl Written {
    fn of_keyword(word: &str) -> Option<Written> {
        Some(match word {
            "void" => Written::Void,
            "boolean" => Written::Boolean,
            "byte" => Written::Byte,
            "short" => Written::Short,
            "char" => Written::Char,
            "int" => Written::Int,
            "long" => Written::Long,
            "float" => Written::Float,
            "double" => Written::Double,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unary {
    Negate,
    Not,
    Complement,
    Plus,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Binary {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
    And,
    Or,
    Xor,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
    AndAlso,
    OrElse,
}

impl Binary {
    fn of_mark(mark: &str) -> Option<Binary> {
        Some(match mark {
            "+" => Binary::Add,
            "-" => Binary::Subtract,
            "*" => Binary::Multiply,
            "/" => Binary::Divide,
            "%" => Binary::Remainder,
            "<" => Binary::Less,
            "<=" => Binary::LessOrEqual,
            ">" => Binary::Greater,
            ">=" => Binary::GreaterOrEqual,
            "==" => Binary::Equal,
            "!=" => Binary::NotEqual,
            "&" => Binary::And,
            "|" => Binary::Or,
            "^" => Binary::Xor,
            "<<" => Binary::ShiftLeft,
            ">>" => Binary::ShiftRight,
            ">>>" => Binary::UnsignedShiftRight,
            "&&" => Binary::AndAlso,
            "||" => Binary::OrElse,
            _ => return None,
        })
    }

    fn precedence(self) -> u8 {
        match self {
            Binary::Multiply | Binary::Divide | Binary::Remainder => 12,
            Binary::Add | Binary::Subtract => 11,
            Binary::ShiftLeft | Binary::ShiftRight | Binary::UnsignedShiftRight => 10,
            Binary::Less | Binary::LessOrEqual | Binary::Greater | Binary::GreaterOrEqual => 9,
            Binary::Equal | Binary::NotEqual => 8,
            Binary::And => 7,
            Binary::Xor => 6,
            Binary::Or => 5,
            Binary::AndAlso => 4,
            Binary::OrElse => 3,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expression {
    Int(i64),
    Long(i64),
    Float(f64),
    Double(f64),
    Char(u16),
    Str(String),
    Boolean(bool),
    Null,
    This,

    Super,

    Name(String),
    Field {
        of: Box<Expression>,
        name: String,
    },
    Call {
        on: Option<Box<Expression>>,

        super_call: bool,
        name: String,
        arguments: Vec<Expression>,

        held: Vec<Written>,
    },
    New {
        what: Written,
        arguments: Vec<Expression>,

        outer: Option<Box<Expression>>,
    },
    NewArray {
        of: Written,

        lengths: Vec<Expression>,

        empty: usize,
    },

    ArrayOf {
        of: Option<Written>,
        values: Vec<Expression>,
        line: u32,
    },

    ClassLiteral {
        of: Written,
        line: u32,
    },

    OuterThis {
        of: Written,
        line: u32,
    },
    Index {
        of: Box<Expression>,
        at: Box<Expression>,
    },
    Unary {
        operator: Unary,
        of: Box<Expression>,
    },
    Binary {
        operator: Binary,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Assign {
        target: Box<Expression>,

        operator: Option<Binary>,
        value: Box<Expression>,
    },

    Step {
        target: Box<Expression>,
        by: i32,
        after: bool,
    },
    Cast {
        to: Written,
        of: Box<Expression>,
        also: Vec<Written>,
    },
    Conditional {
        condition: Box<Expression>,
        then: Box<Expression>,
        otherwise: Box<Expression>,
    },
    InstanceOf {
        of: Box<Expression>,
        pattern: Box<Pattern>,
    },

    Bind {
        name: String,
        what: Written,
        value: Box<Expression>,
    },

    Switch {
        subject: Box<Expression>,
        arms: Vec<Arm>,
    },

    MethodRef {
        on: Box<Expression>,
        name: String,
        line: u32,
    },

    Lambda {
        parameters: Vec<(Option<Written>, String)>,

        body: Vec<Positioned<Statement>>,

        expression: bool,
        line: u32,
    },

    Anonymous {
        what: Written,
        arguments: Vec<Expression>,
        body: Box<Body>,
        line: u32,
        column: u32,
    },
}

#[derive(Clone, Debug)]
pub struct Body {
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub instance_setup: Vec<Positioned<Statement>>,

    pub static_setup: Vec<Positioned<Statement>>,

    pub nested: Vec<Unit>,
}

#[derive(Clone, Debug)]
pub struct Positioned<T> {
    pub node: T,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug)]
pub enum Statement {
    Block(Vec<Positioned<Statement>>),
    Declare {
        what: Written,
        name: String,
        value: Option<Expression>,
    },
    Express(Expression),
    If {
        condition: Expression,
        then: Box<Positioned<Statement>>,
        otherwise: Option<Box<Positioned<Statement>>>,
    },
    While {
        condition: Expression,
        body: Box<Positioned<Statement>>,
    },
    For {
        start: Vec<Positioned<Statement>>,
        condition: Option<Expression>,
        step: Vec<Expression>,
        body: Box<Positioned<Statement>>,
    },
    DoWhile {
        body: Box<Positioned<Statement>>,
        condition: Expression,
    },

    ForEach {
        what: Written,
        name: String,
        over: Expression,
        body: Box<Positioned<Statement>>,
    },
    Switch {
        subject: Expression,
        arms: Vec<Arm>,
    },
    Try {
        body: Vec<Positioned<Statement>>,
        catches: Vec<Catch>,
        finally: Option<Vec<Positioned<Statement>>>,
    },
    Throw(Expression),

    Labelled {
        label: String,
        body: Box<Positioned<Statement>>,
    },

    Synchronized {
        on: Expression,
        body: Vec<Positioned<Statement>>,
    },

    Locally(Vec<Unit>),

    Assert {
        condition: Expression,
        said: Option<Expression>,
    },
    Return(Option<Expression>),
    Break(Option<String>),
    Continue(Option<String>),

    Yield(Expression),

    Chain {
        to_super: bool,
        arguments: Vec<Expression>,
    },

    Several(Vec<Positioned<Statement>>),
    Nothing,
}

#[derive(Clone, Debug)]
pub struct Arm {
    pub labels: Vec<Expression>,

    pub pattern: Option<Pattern>,

    pub guard: Option<Expression>,

    pub arrow: bool,
    pub body: Vec<Positioned<Statement>>,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug)]
pub struct Pattern {
    pub what: Written,

    pub name: String,

    pub parts: Vec<Pattern>,

    pub taken: bool,
}

fn as_the_class_it_is(mut unit: Unit, constants: &[Constant]) -> Unit {
    let answering = constants.iter().any(|held| held.body.is_some());
    let named = Written::Named(unit.name.clone(), Vec::new());
    let array_of = Written::Array(Box::new(named.clone()));
    let public_constant = Modifiers {
        public: true,
        static_: true,
        final_: true,
        ..Modifiers::default()
    };
    let at = |line: u32, node: Statement| Positioned {
        node,
        line,
        column: 1,
    };
    let set = |line: u32, target: Expression, value: Expression| {
        at(
            line,
            Statement::Express(Expression::Assign {
                target: Box::new(target),
                operator: None,
                value: Box::new(value),
            }),
        )
    };

    let mut setup = Vec::new();
    for (position, constant) in constants.iter().enumerate() {
        unit.fields.push(Field {
            modifiers: public_constant,
            what: named.clone(),
            name: constant.name.clone(),
            value: None,
            line: constant.line,
            annotations: Vec::new(),
        });
        let mut arguments = vec![
            Expression::Str(constant.name.clone()),
            Expression::Int(position as i64),
        ];
        arguments.extend(constant.arguments.iter().cloned());

        let made = match &constant.body {
            Some(body) => Expression::Anonymous {
                what: named.clone(),
                arguments,
                body: Box::new(body.clone()),
                line: constant.line,
                column: constant.column,
            },
            None => Expression::New {
                what: named.clone(),
                arguments,
                outer: None,
            },
        };
        setup.push(set(
            constant.line,
            Expression::Name(constant.name.clone()),
            made,
        ));
    }

    const HELD: &str = "$VALUES";
    unit.fields.push(Field {
        modifiers: Modifiers {
            private: true,
            static_: true,
            final_: true,
            ..Modifiers::default()
        },
        what: array_of.clone(),
        name: HELD.to_string(),
        value: None,
        line: 1,
        annotations: Vec::new(),
    });
    setup.push(set(
        1,
        Expression::Name(HELD.to_string()),
        Expression::NewArray {
            of: named.clone(),
            lengths: vec![Expression::Int(constants.len() as i64)],
            empty: 0,
        },
    ));
    for (position, constant) in constants.iter().enumerate() {
        setup.push(set(
            constant.line,
            Expression::Index {
                of: Box::new(Expression::Name(HELD.to_string())),
                at: Box::new(Expression::Int(position as i64)),
            },
            Expression::Name(constant.name.clone()),
        ));
    }

    setup.extend(std::mem::take(&mut unit.static_setup));
    unit.static_setup = setup;

    if !unit.methods.iter().any(|held| held.constructor) {
        unit.methods.push(Method {
            modifiers: Modifiers {
                private: true,
                ..Modifiers::default()
            },
            returns: Written::Void,
            name: "<init>".to_string(),
            parameters: Vec::new(),
            body: Some(Vec::new()),
            constructor: true,
            variadic: false,
            line: 1,
            bridge: false,
            annotations: Vec::new(),
            parameter_annotations: Vec::new(),
            default_value: None,
            own_variables: Vec::new(),
            own_bounds: Vec::new(),
            throws: Vec::new(),
        });
    }
    for method in &mut unit.methods {
        if !method.constructor {
            continue;
        }
        method.modifiers.public = false;
        method.modifiers.protected = false;
        method.modifiers.private = true;
        let mut parameters = vec![
            (
                Written::Named("String".to_string(), Vec::new()),
                "$name".to_string(),
            ),
            (Written::Int, "$ordinal".to_string()),
        ];
        parameters.append(&mut method.parameters);
        method.parameters = parameters;
        let mut written = method.body.take().unwrap_or_default();
        let passed_on = written.iter_mut().find_map(|one| match &mut one.node {
            Statement::Chain {
                to_super: false,
                arguments,
            } => Some(arguments),
            _ => None,
        });
        if let Some(arguments) = passed_on {
            arguments.splice(
                0..0,
                [
                    Expression::Name("$name".to_string()),
                    Expression::Name("$ordinal".to_string()),
                ],
            );
            method.body = Some(written);
            continue;
        }
        let mut body = vec![at(
            method.line,
            Statement::Chain {
                to_super: true,
                arguments: vec![
                    Expression::Name("$name".to_string()),
                    Expression::Name("$ordinal".to_string()),
                ],
            },
        )];
        body.extend(written);
        method.body = Some(body);
    }

    let mut values = vec![at(
        1,
        Statement::Declare {
            what: array_of.clone(),
            name: "$out".to_string(),
            value: Some(Expression::NewArray {
                of: named.clone(),
                lengths: vec![Expression::Int(constants.len() as i64)],
                empty: 0,
            }),
        },
    )];
    for position in 0..constants.len() {
        values.push(set(
            1,
            Expression::Index {
                of: Box::new(Expression::Name("$out".to_string())),
                at: Box::new(Expression::Int(position as i64)),
            },
            Expression::Index {
                of: Box::new(Expression::Name(HELD.to_string())),
                at: Box::new(Expression::Int(position as i64)),
            },
        ));
    }
    values.push(at(
        1,
        Statement::Return(Some(Expression::Name("$out".to_string()))),
    ));
    unit.methods.push(Method {
        modifiers: Modifiers {
            public: true,
            static_: true,
            ..Modifiers::default()
        },
        returns: array_of,
        name: "values".to_string(),
        parameters: Vec::new(),
        body: Some(values),
        constructor: false,
        variadic: false,
        line: 1,
        bridge: false,
        annotations: Vec::new(),
        parameter_annotations: Vec::new(),
        default_value: None,
        own_variables: Vec::new(),
        own_bounds: Vec::new(),
        throws: Vec::new(),
    });

    let mut value_of = Vec::new();
    for constant in constants {
        value_of.push(at(
            constant.line,
            Statement::If {
                condition: Expression::Call {
                    on: Some(Box::new(Expression::Name("$name".to_string()))),
                    super_call: false,
                    held: Vec::new(),
                    name: "equals".to_string(),
                    arguments: vec![Expression::Str(constant.name.clone())],
                },
                then: Box::new(at(
                    constant.line,
                    Statement::Return(Some(Expression::Name(constant.name.clone()))),
                )),
                otherwise: None,
            },
        ));
    }
    value_of.push(at(
        1,
        Statement::Throw(Expression::New {
            what: Written::Named("IllegalArgumentException".to_string(), Vec::new()),
            arguments: vec![Expression::Name("$name".to_string())],
            outer: None,
        }),
    ));
    unit.methods.push(Method {
        modifiers: Modifiers {
            public: true,
            static_: true,
            ..Modifiers::default()
        },
        returns: named,
        name: "valueOf".to_string(),
        parameters: vec![(
            Written::Named("String".to_string(), Vec::new()),
            "$name".to_string(),
        )],
        body: Some(value_of),
        constructor: false,
        variadic: false,
        line: 1,
        bridge: false,
        annotations: Vec::new(),
        parameter_annotations: Vec::new(),
        default_value: None,
        own_variables: Vec::new(),
        own_bounds: Vec::new(),
        throws: Vec::new(),
    });

    unit.extends = Some("java.lang.Enum".to_string());

    unit.modifiers.final_ = !answering;
    unit
}

const COMPACT: &str = "<compact>";

fn as_the_class_a_record_is(
    mut unit: Unit,
    components: &[(Written, String)],
) -> Result<Unit, Diagnostic> {
    let at = |node: Statement| Positioned {
        node,
        line: 1,
        column: 1,
    };
    let private_final = Modifiers {
        private: true,
        final_: true,
        ..Modifiers::default()
    };
    let public = Modifiers {
        public: true,
        ..Modifiers::default()
    };

    for (what, name) in components {
        if unit.fields.iter().any(|held| held.name == *name) {
            return Err(at_line(
                "EJ117",
                1,
                format!("`{name}` is both a component of this record and a field of it."),
            ));
        }
        unit.fields.push(Field {
            modifiers: private_final,
            what: what.clone(),
            name: name.clone(),
            value: None,
            line: 1,
            annotations: Vec::new(),
        });
    }

    let compact = unit
        .methods
        .iter()
        .position(|held| held.name == COMPACT)
        .map(|at| unit.methods.remove(at));

    let written_canonical = unit
        .methods
        .iter()
        .any(|held| held.constructor && held.parameters.len() == components.len());
    if !written_canonical {
        let mut body: Vec<Positioned<Statement>> = compact
            .as_ref()
            .and_then(|held| held.body.clone())
            .unwrap_or_default();
        body.extend(components.iter().map(|(_, name)| {
            at(Statement::Express(Expression::Assign {
                target: Box::new(Expression::Field {
                    of: Box::new(Expression::This),
                    name: name.clone(),
                }),
                operator: None,
                value: Box::new(Expression::Name(name.clone())),
            }))
        }));
        unit.methods.push(Method {
            modifiers: match &compact {
                Some(held)
                    if held.modifiers.public
                        || held.modifiers.protected
                        || held.modifiers.private =>
                {
                    held.modifiers
                }
                _ => public,
            },
            returns: Written::Void,
            name: "<init>".to_string(),
            parameters: components.to_vec(),
            body: Some(body),
            constructor: true,
            variadic: false,
            line: 1,
            bridge: false,
            annotations: Vec::new(),
            parameter_annotations: Vec::new(),
            default_value: None,
            own_variables: Vec::new(),
            own_bounds: Vec::new(),
            throws: Vec::new(),
        });
    }

    for (what, name) in components {
        if unit
            .methods
            .iter()
            .any(|held| held.name == *name && held.parameters.is_empty())
        {
            continue;
        }
        unit.methods.push(Method {
            modifiers: public,
            returns: what.clone(),
            name: name.clone(),
            parameters: Vec::new(),
            body: Some(vec![at(Statement::Return(Some(Expression::Name(
                name.clone(),
            ))))]),
            constructor: false,
            variadic: false,
            line: 1,
            bridge: false,
            annotations: Vec::new(),
            parameter_annotations: Vec::new(),
            default_value: None,
            own_variables: Vec::new(),
            own_bounds: Vec::new(),
            throws: Vec::new(),
        });
    }

    if !unit
        .methods
        .iter()
        .any(|held| held.name == "toString" && held.parameters.is_empty())
    {
        let simple = unit.name.rsplit('$').next().unwrap_or(&unit.name);
        let mut text = Expression::Str(format!("{simple}["));
        for (index, (_, name)) in components.iter().enumerate() {
            let lead = if index == 0 {
                format!("{name}=")
            } else {
                format!(", {name}=")
            };
            text = Expression::Binary {
                operator: Binary::Add,
                left: Box::new(text),
                right: Box::new(Expression::Str(lead)),
            };
            text = Expression::Binary {
                operator: Binary::Add,
                left: Box::new(text),
                right: Box::new(Expression::Name(name.clone())),
            };
        }
        text = Expression::Binary {
            operator: Binary::Add,
            left: Box::new(text),
            right: Box::new(Expression::Str("]".to_string())),
        };
        unit.methods.push(Method {
            modifiers: public,
            returns: Written::Named("String".to_string(), Vec::new()),
            name: "toString".to_string(),
            parameters: Vec::new(),
            body: Some(vec![at(Statement::Return(Some(text)))]),
            constructor: false,
            variadic: false,
            line: 1,
            bridge: false,
            annotations: Vec::new(),
            parameter_annotations: Vec::new(),
            default_value: None,
            own_variables: Vec::new(),
            own_bounds: Vec::new(),
            throws: Vec::new(),
        });
    }

    const IT: &str = "$it";
    const OTHER: &str = "$other";
    if !unit
        .methods
        .iter()
        .any(|held| held.name == "equals" && held.parameters.len() == 1)
    {
        let mine = |name: &str| Expression::Field {
            of: Box::new(Expression::This),
            name: name.to_string(),
        };
        let theirs = |name: &str| Expression::Field {
            of: Box::new(Expression::Name(IT.to_string())),
            name: name.to_string(),
        };
        let mut same = Expression::Boolean(true);
        for (what, name) in components {
            let one = the_same_component(what, mine(name), theirs(name));
            same = Expression::Binary {
                operator: Binary::AndAlso,
                left: Box::new(same),
                right: Box::new(one),
            };
        }
        let body = vec![
            at(Statement::If {
                condition: Expression::Unary {
                    operator: Unary::Not,
                    of: Box::new(Expression::InstanceOf {
                        of: Box::new(Expression::Name(OTHER.to_string())),
                        pattern: Box::new(Pattern {
                            what: Written::Named(unit.name.clone(), Vec::new()),
                            name: IT.to_string(),
                            parts: Vec::new(),
                            taken: false,
                        }),
                    }),
                },
                then: Box::new(at(Statement::Return(Some(Expression::Boolean(false))))),
                otherwise: None,
            }),
            at(Statement::Return(Some(same))),
        ];
        unit.methods.push(Method {
            modifiers: public,
            returns: Written::Boolean,
            name: "equals".to_string(),
            parameters: vec![(
                Written::Named("Object".to_string(), Vec::new()),
                OTHER.to_string(),
            )],
            body: Some(body),
            constructor: false,
            variadic: false,
            line: 1,
            bridge: false,
            annotations: Vec::new(),
            parameter_annotations: Vec::new(),
            default_value: None,
            own_variables: Vec::new(),
            own_bounds: Vec::new(),
            throws: Vec::new(),
        });
    }

    if !unit
        .methods
        .iter()
        .any(|held| held.name == "hashCode" && held.parameters.is_empty())
    {
        let mut worked = Expression::Int(0);
        for (what, name) in components {
            worked = Expression::Binary {
                operator: Binary::Add,
                left: Box::new(Expression::Binary {
                    operator: Binary::Multiply,
                    left: Box::new(worked),
                    right: Box::new(Expression::Int(31)),
                }),
                right: Box::new(what_one_component_hashes_to(
                    what,
                    Expression::Field {
                        of: Box::new(Expression::This),
                        name: name.clone(),
                    },
                )),
            };
        }
        unit.methods.push(Method {
            modifiers: public,
            returns: Written::Int,
            name: "hashCode".to_string(),
            parameters: Vec::new(),
            body: Some(vec![at(Statement::Return(Some(worked)))]),
            constructor: false,
            variadic: false,
            line: 1,
            bridge: false,
            annotations: Vec::new(),
            parameter_annotations: Vec::new(),
            default_value: None,
            own_variables: Vec::new(),
            own_bounds: Vec::new(),
            throws: Vec::new(),
        });
    }

    unit.extends = Some("java.lang.Record".to_string());
    unit.modifiers.final_ = true;
    Ok(unit)
}

fn the_same_component(what: &Written, mine: Expression, theirs: Expression) -> Expression {
    let compared_by = |class: &str| Expression::Binary {
        operator: Binary::Equal,
        left: Box::new(Expression::Call {
            on: Some(Box::new(Expression::Name(class.to_string()))),
            super_call: false,
            held: Vec::new(),
            name: "compare".to_string(),
            arguments: vec![mine.clone(), theirs.clone()],
        }),
        right: Box::new(Expression::Int(0)),
    };
    match what {
        Written::Float => compared_by("Float"),
        Written::Double => compared_by("Double"),
        Written::Boolean
        | Written::Byte
        | Written::Short
        | Written::Char
        | Written::Int
        | Written::Long => Expression::Binary {
            operator: Binary::Equal,
            left: Box::new(mine),
            right: Box::new(theirs),
        },
        _ => Expression::Conditional {
            condition: Box::new(Expression::Binary {
                operator: Binary::Equal,
                left: Box::new(mine.clone()),
                right: Box::new(Expression::Null),
            }),
            then: Box::new(Expression::Binary {
                operator: Binary::Equal,
                left: Box::new(theirs.clone()),
                right: Box::new(Expression::Null),
            }),
            otherwise: Box::new(Expression::Call {
                on: Some(Box::new(mine)),
                super_call: false,
                held: Vec::new(),
                name: "equals".to_string(),
                arguments: vec![theirs],
            }),
        },
    }
}

fn what_one_component_hashes_to(what: &Written, mine: Expression) -> Expression {
    let hashed_by = |class: &str| Expression::Call {
        on: Some(Box::new(Expression::Name(class.to_string()))),
        super_call: false,
        held: Vec::new(),
        name: "hashCode".to_string(),
        arguments: vec![mine.clone()],
    };
    match what {
        Written::Boolean => hashed_by("Boolean"),
        Written::Byte => hashed_by("Byte"),
        Written::Short => hashed_by("Short"),
        Written::Char => hashed_by("Character"),
        Written::Int => hashed_by("Integer"),
        Written::Long => hashed_by("Long"),
        Written::Float => hashed_by("Float"),
        Written::Double => hashed_by("Double"),
        _ => Expression::Conditional {
            condition: Box::new(Expression::Binary {
                operator: Binary::Equal,
                left: Box::new(mine.clone()),
                right: Box::new(Expression::Null),
            }),
            then: Box::new(Expression::Int(0)),
            otherwise: Box::new(Expression::Call {
                on: Some(Box::new(mine)),
                super_call: false,
                held: Vec::new(),
                name: "hashCode".to_string(),
                arguments: Vec::new(),
            }),
        },
    }
}

fn belonging_to_an_instance(mut unit: Unit, holder: &str, package: &Option<String>) -> Unit {
    let enclosing = match package {
        Some(package) => format!("{}/{holder}", package.replace('.', "/")),
        None => holder.to_string(),
    };
    let written = Written::Named(enclosing.replace('/', "."), Vec::new());

    unit.fields.insert(
        0,
        Field {
            modifiers: Modifiers {
                private: true,
                final_: true,
                ..Modifiers::default()
            },
            what: written.clone(),
            name: OUTER.to_string(),
            value: None,
            line: 1,
            annotations: Vec::new(),
        },
    );

    if !unit.methods.iter().any(|held| held.constructor) {
        unit.methods.push(Method {
            modifiers: Modifiers {
                public: true,
                ..Modifiers::default()
            },
            returns: Written::Void,
            name: "<init>".to_string(),
            parameters: Vec::new(),
            body: Some(Vec::new()),
            constructor: true,
            variadic: false,
            line: 1,
            bridge: false,
            annotations: Vec::new(),
            parameter_annotations: Vec::new(),
            default_value: None,
            own_variables: Vec::new(),
            own_bounds: Vec::new(),
            throws: Vec::new(),
        });
    }
    for method in &mut unit.methods {
        if !method.constructor {
            continue;
        }
        let mut parameters = vec![(written.clone(), OUTER.to_string())];
        parameters.append(&mut method.parameters);
        method.parameters = parameters;

        let body = method.body.take().unwrap_or_default();
        let filling = Positioned {
            node: Statement::Express(Expression::Assign {
                target: Box::new(Expression::Field {
                    of: Box::new(Expression::This),
                    name: OUTER.to_string(),
                }),
                operator: None,
                value: Box::new(Expression::Name(OUTER.to_string())),
            }),
            line: method.line,
            column: 1,
        };
        method.body = Some(filled_before_the_chain(
            body,
            &[filling],
            &[OUTER.to_string()],
            method.line,
        ));
    }

    unit.outer = Some(enclosing);
    unit
}

fn filled_before_the_chain(
    mut body: Vec<Positioned<Statement>>,
    filling: &[Positioned<Statement>],
    handed_on: &[String],
    line: u32,
) -> Vec<Positioned<Statement>> {
    let chained = body
        .iter()
        .position(|one| matches!(one.node, Statement::Chain { .. }));
    match chained.map(|at| (at, &mut body[at].node)) {
        Some((
            _,
            Statement::Chain {
                to_super: false,
                arguments,
            },
        )) => {
            arguments.splice(
                0..0,
                handed_on.iter().map(|name| Expression::Name(name.clone())),
            );
            body
        }
        Some(_) => {
            let mut out = filling.to_vec();
            out.append(&mut body);
            out
        }
        None => {
            let mut out = filling.to_vec();
            out.push(Positioned {
                node: Statement::Chain {
                    to_super: true,
                    arguments: Vec::new(),
                },
                line,
                column: 1,
            });
            out.append(&mut body);
            out
        }
    }
}

fn with_the_assertion_flag(unit: &Unit) -> Unit {
    let mut held = unit.clone();
    held.fields.push(Field {
        modifiers: Modifiers {
            static_: true,
            final_: true,
            ..Modifiers::default()
        },
        what: Written::Boolean,
        name: ASSERTIONS_OFF.to_string(),
        value: None,
        line: 1,
        annotations: Vec::new(),
    });
    held.static_setup.insert(
        0,
        Positioned {
            node: Statement::Express(Expression::Assign {
                target: Box::new(Expression::Name(ASSERTIONS_OFF.to_string())),
                operator: None,
                value: Box::new(Expression::Unary {
                    operator: Unary::Not,
                    of: Box::new(Expression::Call {
                        on: Some(Box::new(Expression::ClassLiteral {
                            of: Written::Named(unit.name.clone(), Vec::new()),
                            line: 1,
                        })),
                        super_call: false,
                        held: Vec::new(),
                        name: "desiredAssertionStatus".to_string(),
                        arguments: Vec::new(),
                    }),
                }),
            }),
            line: 1,
            column: 1,
        },
    );
    held
}

fn holds_an_assert(unit: &Unit) -> bool {
    fn inside(statement: &Statement) -> bool {
        let mut found = matches!(statement, Statement::Assert { .. });
        match statement {
            Statement::Block(held) | Statement::Several(held) => {
                found = found || held.iter().any(|one| inside(&one.node))
            }
            Statement::If {
                then, otherwise, ..
            } => {
                found = found
                    || inside(&then.node)
                    || otherwise.as_ref().is_some_and(|one| inside(&one.node))
            }
            Statement::While { body, .. }
            | Statement::DoWhile { body, .. }
            | Statement::For { body, .. }
            | Statement::ForEach { body, .. }
            | Statement::Labelled { body, .. } => found = found || inside(&body.node),
            Statement::Switch { arms, .. } => {
                found = found
                    || arms
                        .iter()
                        .any(|arm| arm.body.iter().any(|one| inside(&one.node)))
            }
            Statement::Try {
                body,
                catches,
                finally,
            } => {
                found = found
                    || body.iter().any(|one| inside(&one.node))
                    || catches
                        .iter()
                        .any(|catch| catch.body.iter().any(|one| inside(&one.node)))
                    || finally
                        .as_ref()
                        .is_some_and(|held| held.iter().any(|one| inside(&one.node)))
            }
            Statement::Synchronized { body, .. } => {
                found = found || body.iter().any(|one| inside(&one.node))
            }
            _ => {}
        }
        found
    }

    unit.methods
        .iter()
        .flat_map(|method| method.body.iter().flatten())
        .chain(unit.instance_setup.iter())
        .chain(unit.static_setup.iter())
        .any(|one| inside(&one.node))
}

const ASSERTIONS_OFF: &str = "$assertionsDisabled";

const PATTERN_SUBJECT: &str = "$shape";

const OUTER: &str = "$outer";

const REACHING_UP: &str = "lambda$super$";

fn written_for(what: &Type) -> Option<Written> {
    Some(match what {
        Type::Void => return None,
        Type::Boolean => Written::Boolean,
        Type::Byte => Written::Byte,
        Type::Short => Written::Short,
        Type::Char => Written::Char,
        Type::Int => Written::Int,
        Type::Long => Written::Long,
        Type::Float => Written::Float,
        Type::Double => Written::Double,
        Type::Array(of) => Written::Array(Box::new(written_for(of)?)),

        Type::Object(name, held) => Written::Named(
            name.replace('/', "."),
            held.iter().filter_map(written_for).collect(),
        ),
    })
}

fn names_read(body: &Body, out: &mut Vec<String>) {
    free_in_body(
        body,
        &mut |expression, bound, out| {
            let mut found = Vec::new();
            names_in_expression(expression, &mut found);
            out.extend(found.into_iter().filter(|name| !bound.contains(name)));
        },
        out,
    );
}

fn names_assigned(body: &Body, out: &mut Vec<String>) {
    free_in_body(
        body,
        &mut |expression, bound, out| {
            let mut found = Vec::new();
            assigned_in_expression(expression, &mut found);
            out.extend(found.into_iter().filter(|name| !bound.contains(name)));
        },
        out,
    );
}

type Seeing<'a> = dyn FnMut(&Expression, &[String], &mut Vec<String>) + 'a;

fn free_in_body(body: &Body, visit: &mut Seeing<'_>, out: &mut Vec<String>) {
    for field in &body.fields {
        if let Some(value) = &field.value {
            visit(value, &[], out);
        }
    }
    for method in &body.methods {
        let mut bound: Vec<String> = method
            .parameters
            .iter()
            .map(|(_, name)| name.clone())
            .collect();
        in_scope_order(
            method.body.as_deref().unwrap_or_default(),
            &mut bound,
            visit,
            out,
        );
    }
    let mut bound = Vec::new();
    for statement in &body.instance_setup {
        in_scope_statement(&statement.node, &mut bound, visit, out);
    }
}

fn in_scope_order(
    statements: &[Positioned<Statement>],
    bound: &mut Vec<String>,
    visit: &mut Seeing<'_>,
    out: &mut Vec<String>,
) {
    let mark = bound.len();
    for one in statements {
        in_scope_statement(&one.node, bound, visit, out);
    }
    bound.truncate(mark);
}

fn in_scope_statement(
    statement: &Statement,
    bound: &mut Vec<String>,
    visit: &mut Seeing<'_>,
    out: &mut Vec<String>,
) {
    match statement {
        Statement::Declare { name, value, .. } => {
            if let Some(value) = value {
                visit(value, bound, out);
            }
            bound.push(name.clone());
        }
        Statement::Several(held) => {
            for one in held {
                in_scope_statement(&one.node, bound, visit, out);
            }
        }
        Statement::Block(held) => in_scope_order(held, bound, visit, out),
        Statement::If {
            condition,
            then,
            otherwise,
        } => {
            visit(condition, bound, out);
            in_scope_nested(&then.node, bound, visit, out);
            if let Some(otherwise) = otherwise {
                in_scope_nested(&otherwise.node, bound, visit, out);
            }
        }
        Statement::While { condition, body } | Statement::DoWhile { body, condition } => {
            visit(condition, bound, out);
            in_scope_nested(&body.node, bound, visit, out);
        }
        Statement::For {
            start,
            condition,
            step,
            body,
        } => {
            let mark = bound.len();
            for one in start {
                in_scope_statement(&one.node, bound, visit, out);
            }
            if let Some(condition) = condition {
                visit(condition, bound, out);
            }
            for one in step {
                visit(one, bound, out);
            }
            in_scope_statement(&body.node, bound, visit, out);
            bound.truncate(mark);
        }
        Statement::ForEach {
            name, over, body, ..
        } => {
            visit(over, bound, out);
            let mark = bound.len();
            bound.push(name.clone());
            in_scope_statement(&body.node, bound, visit, out);
            bound.truncate(mark);
        }
        Statement::Switch { subject, arms } => {
            visit(subject, bound, out);
            let mark = bound.len();
            for arm in arms {
                for label in &arm.labels {
                    visit(label, bound, out);
                }
                for one in &arm.body {
                    in_scope_statement(&one.node, bound, visit, out);
                }
            }
            bound.truncate(mark);
        }
        Statement::Try {
            body,
            catches,
            finally,
        } => {
            in_scope_order(body, bound, visit, out);
            for catch in catches {
                let mark = bound.len();
                bound.push(catch.name.clone());
                in_scope_order(&catch.body, bound, visit, out);
                bound.truncate(mark);
            }
            in_scope_order(finally.as_deref().unwrap_or_default(), bound, visit, out);
        }
        Statement::Labelled { body, .. } => in_scope_nested(&body.node, bound, visit, out),
        Statement::Synchronized { on, body } => {
            visit(on, bound, out);
            in_scope_order(body, bound, visit, out);
        }
        other => walk_statement(other, &mut |expression| visit(expression, bound, out)),
    }
}

fn in_scope_nested(
    statement: &Statement,
    bound: &mut Vec<String>,
    visit: &mut Seeing<'_>,
    out: &mut Vec<String>,
) {
    let mark = bound.len();
    in_scope_statement(statement, bound, visit, out);
    bound.truncate(mark);
}

fn assigned_in_expression(expression: &Expression, out: &mut Vec<String>) {
    match expression {
        Expression::Assign { target, value, .. } => {
            if let Expression::Name(name) = target.as_ref() {
                out.push(name.clone());
            }
            assigned_in_expression(value, out);
        }
        Expression::Step { target, .. } => {
            if let Expression::Name(name) = target.as_ref() {
                out.push(name.clone());
            }
        }
        Expression::Anonymous {
            arguments, body, ..
        } => {
            for one in arguments {
                assigned_in_expression(one, out);
            }
            names_assigned(body, out);
        }
        Expression::Lambda {
            parameters, body, ..
        } => {
            let mut bound: Vec<String> = parameters.iter().map(|(_, name)| name.clone()).collect();
            in_scope_order(
                body,
                &mut bound,
                &mut |expression, bound, out| {
                    let mut found = Vec::new();
                    assigned_in_expression(expression, &mut found);
                    out.extend(found.into_iter().filter(|name| !bound.contains(name)));
                },
                out,
            );
        }
        _ => walk_expression(expression, &mut |inside| {
            assigned_in_expression(inside, out)
        }),
    }
}

fn names_in_statement(statement: &Statement, out: &mut Vec<String>) {
    walk_statement(statement, &mut |expression| {
        names_in_expression(expression, out)
    });
}

fn names_in_expression(expression: &Expression, out: &mut Vec<String>) {
    match expression {
        Expression::Name(name) => out.push(name.clone()),
        Expression::Anonymous {
            arguments, body, ..
        } => {
            for one in arguments {
                names_in_expression(one, out);
            }
            names_read(body, out);
            return;
        }
        Expression::Lambda {
            parameters, body, ..
        } => {
            let mut bound: Vec<String> = parameters.iter().map(|(_, name)| name.clone()).collect();
            in_scope_order(
                body,
                &mut bound,
                &mut |expression, bound, out| {
                    let mut found = Vec::new();
                    names_in_expression(expression, &mut found);
                    out.extend(found.into_iter().filter(|name| !bound.contains(name)));
                },
                out,
            );
            return;
        }
        _ => {}
    }
    walk_expression(expression, &mut |inside| names_in_expression(inside, out));
}

fn classes_written_inside(unit: &Unit) -> Vec<Unit> {
    let mut found = Vec::new();
    let mut visit = |statement: &Statement| {
        let Statement::Locally(units) = statement else {
            return;
        };
        for (at, one) in units.iter().enumerate() {
            let mut held = one.clone();
            if at == 0 {
                held.name = format!("{}${}", unit.name, one.name);
            }
            held.package = unit.package.clone();
            held.imports = unit.imports.clone();
            held.static_imports = unit.static_imports.clone();
            found.push(held);
        }
    };
    for method in &unit.methods {
        for held in method.body.iter().flatten() {
            every_statement(&held.node, &mut visit);
        }
    }
    for held in unit.instance_setup.iter().chain(unit.static_setup.iter()) {
        every_statement(&held.node, &mut visit);
    }
    found
}

fn every_statement(statement: &Statement, visit: &mut impl FnMut(&Statement)) {
    visit(statement);
    let mut inside = |held: &Positioned<Statement>| every_statement(&held.node, visit);
    match statement {
        Statement::Block(held) | Statement::Several(held) => held.iter().for_each(inside),
        Statement::If {
            then, otherwise, ..
        } => {
            inside(then);
            if let Some(held) = otherwise {
                inside(held);
            }
        }
        Statement::While { body, .. }
        | Statement::DoWhile { body, .. }
        | Statement::ForEach { body, .. }
        | Statement::Labelled { body, .. } => inside(body),
        Statement::For { start, body, .. } => {
            start.iter().for_each(&mut inside);
            inside(body);
        }
        Statement::Switch { arms, .. } => {
            for arm in arms {
                arm.body.iter().for_each(&mut inside);
            }
        }
        Statement::Try {
            body,
            catches,
            finally,
        } => {
            body.iter().for_each(&mut inside);
            for catch in catches {
                catch.body.iter().for_each(&mut inside);
            }
            for held in finally.iter().flatten() {
                inside(held);
            }
        }
        Statement::Synchronized { body, .. } => body.iter().for_each(inside),
        _ => {}
    }
}

fn walk_statement(statement: &Statement, visit: &mut impl FnMut(&Expression)) {
    match statement {
        Statement::Nothing | Statement::Break(_) | Statement::Continue(_) => {}

        Statement::Locally(units) => {
            for unit in units {
                for field in &unit.fields {
                    if let Some(value) = &field.value {
                        visit(value);
                    }
                }
                for method in &unit.methods {
                    for held in method.body.iter().flatten() {
                        walk_statement(&held.node, visit);
                    }
                }
                for held in unit.instance_setup.iter().chain(unit.static_setup.iter()) {
                    walk_statement(&held.node, visit);
                }
            }
        }
        Statement::Block(held) | Statement::Several(held) => {
            for one in held {
                walk_statement(&one.node, visit);
            }
        }
        Statement::Declare { value, .. } => {
            if let Some(value) = value {
                visit(value);
            }
        }
        Statement::Express(value) | Statement::Throw(value) | Statement::Yield(value) => {
            visit(value)
        }
        Statement::Return(value) => {
            if let Some(value) = value {
                visit(value);
            }
        }
        Statement::If {
            condition,
            then,
            otherwise,
        } => {
            visit(condition);
            walk_statement(&then.node, visit);
            if let Some(otherwise) = otherwise {
                walk_statement(&otherwise.node, visit);
            }
        }
        Statement::While { condition, body } | Statement::DoWhile { body, condition } => {
            visit(condition);
            walk_statement(&body.node, visit);
        }
        Statement::For {
            start,
            condition,
            step,
            body,
        } => {
            for one in start {
                walk_statement(&one.node, visit);
            }
            if let Some(condition) = condition {
                visit(condition);
            }
            for one in step {
                visit(one);
            }
            walk_statement(&body.node, visit);
        }
        Statement::ForEach { over, body, .. } => {
            visit(over);
            walk_statement(&body.node, visit);
        }
        Statement::Switch { subject, arms } => {
            visit(subject);
            for arm in arms {
                for label in &arm.labels {
                    visit(label);
                }
                for one in &arm.body {
                    walk_statement(&one.node, visit);
                }
            }
        }
        Statement::Try {
            body,
            catches,
            finally,
        } => {
            for one in body {
                walk_statement(&one.node, visit);
            }
            for catch in catches {
                for one in &catch.body {
                    walk_statement(&one.node, visit);
                }
            }
            for one in finally.iter().flatten() {
                walk_statement(&one.node, visit);
            }
        }
        Statement::Labelled { body, .. } => walk_statement(&body.node, visit),
        Statement::Synchronized { on, body } => {
            visit(on);
            for one in body {
                walk_statement(&one.node, visit);
            }
        }
        Statement::Assert { condition, said } => {
            visit(condition);
            if let Some(said) = said {
                visit(said);
            }
        }
        Statement::Chain { arguments, .. } => {
            for one in arguments {
                visit(one);
            }
        }
    }
}

fn walk_expression(expression: &Expression, visit: &mut impl FnMut(&Expression)) {
    match expression {
        Expression::Int(_)
        | Expression::Long(_)
        | Expression::Float(_)
        | Expression::Double(_)
        | Expression::Char(_)
        | Expression::Str(_)
        | Expression::Boolean(_)
        | Expression::Null
        | Expression::This
        | Expression::Super
        | Expression::Name(_) => {}
        Expression::Field { of, .. } => visit(of),
        Expression::Call { on, arguments, .. } => {
            if let Some(on) = on {
                visit(on);
            }
            arguments.iter().for_each(visit);
        }
        Expression::New { arguments, .. } => arguments.iter().for_each(visit),
        Expression::NewArray { lengths, .. } => {
            for one in lengths {
                visit(one);
            }
        }
        Expression::ArrayOf { values, .. } => {
            for one in values {
                visit(one);
            }
        }
        Expression::ClassLiteral { .. } | Expression::OuterThis { .. } => {}
        Expression::Index { of, at } => {
            visit(of);
            visit(at);
        }
        Expression::Unary { of, .. } => visit(of),
        Expression::Binary { left, right, .. } => {
            visit(left);
            visit(right);
        }
        Expression::Assign { target, value, .. } => {
            visit(target);
            visit(value);
        }
        Expression::Step { target, .. } => visit(target),
        Expression::Cast { of, .. } => visit(of),
        Expression::Conditional {
            condition,
            then,
            otherwise,
        } => {
            visit(condition);
            visit(then);
            visit(otherwise);
        }
        Expression::InstanceOf { of, .. } => visit(of),
        Expression::Bind { value, .. } => visit(value),
        Expression::Switch { subject, arms } => {
            visit(subject);
            for arm in arms {
                arm.labels.iter().for_each(&mut *visit);
                for statement in &arm.body {
                    walk_statement(&statement.node, visit);
                }
            }
        }
        Expression::Lambda { body, .. } => {
            for statement in body {
                walk_statement(&statement.node, visit);
            }
        }
        Expression::MethodRef { on, .. } => visit(on),
        Expression::Anonymous {
            arguments, body, ..
        } => {
            arguments.iter().for_each(&mut *visit);

            for field in &body.fields {
                if let Some(value) = &field.value {
                    visit(value);
                }
            }
            for method in &body.methods {
                for statement in method.body.iter().flatten() {
                    walk_statement(&statement.node, visit);
                }
            }
            for statement in &body.instance_setup {
                walk_statement(&statement.node, visit);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Constant {
    pub name: String,
    pub arguments: Vec<Expression>,

    pub body: Option<Body>,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug)]
pub struct Catch {
    pub types: Vec<Written>,
    pub name: String,
    pub body: Vec<Positioned<Statement>>,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Modifiers {
    pub public: bool,
    pub private: bool,
    pub protected: bool,
    pub static_: bool,
    pub final_: bool,
    pub abstract_: bool,
    pub synchronized: bool,
    pub volatile: bool,
    pub transient: bool,

    pub native_: bool,

    pub sealed: bool,
}

impl Modifiers {
    pub fn access_flags(self, and: u16) -> u16 {
        let mut flags = and;
        if self.public {
            flags |= 0x0001;
        }
        if self.private {
            flags |= 0x0002;
        }
        if self.protected {
            flags |= 0x0004;
        }
        if self.static_ {
            flags |= 0x0008;
        }
        if self.final_ {
            flags |= 0x0010;
        }
        if self.synchronized {
            flags |= 0x0020;
        }
        if self.volatile {
            flags |= 0x0040;
        }
        if self.transient {
            flags |= 0x0080;
        }
        if self.native_ {
            flags |= 0x0100;
        }
        if self.abstract_ {
            flags |= 0x0400;
        }
        flags
    }
}

#[derive(Clone, Debug)]
pub struct Annotated {
    pub name: String,

    pub values: Vec<(String, Expression)>,
    pub line: u32,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub modifiers: Modifiers,
    pub what: Written,
    pub name: String,

    pub value: Option<Expression>,

    pub annotations: Vec<Annotated>,
    pub line: u32,
}

#[derive(Clone, Debug)]
pub struct Method {
    pub modifiers: Modifiers,
    pub returns: Written,
    pub name: String,
    pub parameters: Vec<(Written, String)>,
    pub body: Option<Vec<Positioned<Statement>>>,
    pub constructor: bool,

    pub variadic: bool,

    pub bridge: bool,

    pub annotations: Vec<Annotated>,

    pub parameter_annotations: Vec<Vec<Annotated>>,

    pub default_value: Option<Expression>,

    pub own_variables: Vec<String>,

    pub own_bounds: Vec<Written>,

    pub throws: Vec<String>,
    pub line: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Class,
    Interface,

    Enum,

    Record,
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub shape: Shape,
    pub package: Option<String>,
    pub imports: Vec<String>,
    pub modifiers: Modifiers,
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,

    pub instance_setup: Vec<Positioned<Statement>>,

    pub static_setup: Vec<Positioned<Statement>>,

    pub outer: Option<String>,

    pub above: Vec<Written>,

    pub stands_for_a_lambda: bool,

    pub permits: Vec<String>,

    pub annotation: bool,

    pub static_imports: Vec<String>,

    pub annotations: Vec<Annotated>,

    pub source: Option<String>,

    pub own_variables: Vec<(String, Written)>,

    pub enclosed_by: Option<(String, Option<(String, String)>)>,
}

impl Unit {
    pub fn internal_name(&self) -> String {
        match &self.package {
            Some(package) => format!("{}/{}", package.replace('.', "/"), self.name),
            None => self.name.clone(),
        }
    }
}

struct TypeFrame {
    held: Vec<(String, Written)>,

    of_a_class: bool,
}

pub struct Parser {
    tokens: Vec<Spelled>,
    at: usize,

    held_annotations: Vec<Annotated>,

    parameter_annotations: Vec<Vec<Annotated>>,

    bare_lambda: bool,

    package: Option<String>,
    imports: Vec<String>,

    type_variables: Vec<TypeFrame>,

    implicit_name: Option<String>,

    in_parameters: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Spelled>) -> Parser {
        Parser {
            tokens,
            at: 0,
            held_annotations: Vec::new(),
            parameter_annotations: Vec::new(),
            bare_lambda: true,
            package: None,
            imports: Vec::new(),
            type_variables: Vec::new(),
            implicit_name: None,
            in_parameters: false,
        }
    }

    fn wrap_in_an_implicit_class(&mut self, name: &str) {
        let (line, column) = (self.line(), self.column());
        let opening = [
            Token::Keyword("final"),
            Token::Keyword("class"),
            Token::Identifier(name.to_string()),
            Token::Punctuation("{"),
        ];
        let at = self.at;
        for (offset, token) in opening.into_iter().enumerate() {
            self.tokens.insert(
                at + offset,
                Spelled {
                    token,
                    line,
                    column,
                },
            );
        }
        let last = self.tokens.len() - 1;
        let end_line = self.tokens[last].line;
        self.tokens.insert(
            last,
            Spelled {
                token: Token::Punctuation("}"),
                line: end_line,
                column: 1,
            },
        );
    }

    fn erased(&self, name: &str) -> Option<Written> {
        self.type_variables.iter().rev().find_map(|frame| {
            let at = frame.held.iter().position(|(one, _)| one == name)?;
            let bound = frame.held[at].1.clone();

            if frame.of_a_class {
                Some(Written::Variable(at, Box::new(bound)))
            } else {
                Some(Written::OwnVariable(name.to_string(), Box::new(bound)))
            }
        })
    }

    fn push_type_parameters(&mut self, of_a_class: bool) -> Result<bool, Diagnostic> {
        if !self.is_mark("<") {
            return Ok(false);
        }

        self.want_mark("<")?;
        let mut frame = Vec::new();
        loop {
            let name = self.want_name()?;
            let bound = if self.eat_word("extends") {
                let first = self.written_type()?;
                while self.eat_mark("&") {
                    self.written_type()?;
                }
                first
            } else {
                Written::Named("Object".to_string(), Vec::new())
            };
            frame.push((name, bound));
            if self.eat_mark(",") {
                continue;
            }
            break;
        }
        self.close_type_parameters()?;
        self.type_variables.push(TypeFrame {
            held: frame,
            of_a_class,
        });
        Ok(true)
    }

    fn close_type_parameters(&mut self) -> Result<(), Diagnostic> {
        match self.here().token {
            Token::Punctuation(">") => {
                self.take();
                Ok(())
            }
            Token::Punctuation(">>") => {
                self.tokens[self.at].token = Token::Punctuation(">");
                Ok(())
            }
            Token::Punctuation(">>>") => {
                self.tokens[self.at].token = Token::Punctuation(">>");
                Ok(())
            }
            _ => Err(at(
                "EJ115",
                self.line(),
                self.column(),
                "A type parameter list was opened and never closed.",
            )),
        }
    }

    fn pop_type_parameters(&mut self, pushed: bool) {
        if pushed {
            self.type_variables.pop();
        }
    }

    fn erasing(&self, written: Written) -> Written {
        match &written {
            Written::Named(name, _) => match self.erased(name) {
                Some(bound) => bound,
                None => written,
            },
            _ => written,
        }
    }

    fn here(&self) -> &Spelled {
        &self.tokens[self.at.min(self.tokens.len() - 1)]
    }

    fn ahead(&self, by: usize) -> &Token {
        &self.tokens[(self.at + by).min(self.tokens.len() - 1)].token
    }

    fn line(&self) -> u32 {
        self.here().line
    }

    fn column(&self) -> u32 {
        self.here().column
    }

    fn take(&mut self) -> Token {
        let token = self.here().token.clone();
        if self.at < self.tokens.len() - 1 {
            self.at += 1;
        }
        token
    }

    fn is_mark(&self, mark: &str) -> bool {
        matches!(&self.here().token, Token::Punctuation(found) if *found == mark)
    }

    fn is_word(&self, word: &str) -> bool {
        matches!(&self.here().token, Token::Keyword(found) if *found == word)
    }

    fn eat_mark(&mut self, mark: &str) -> bool {
        if self.is_mark(mark) {
            self.take();
            return true;
        }
        false
    }

    fn eat_word(&mut self, word: &str) -> bool {
        if self.is_word(word) {
            self.take();
            return true;
        }
        false
    }

    fn want_mark(&mut self, mark: &str) -> Result<(), Diagnostic> {
        if self.eat_mark(mark) {
            return Ok(());
        }
        Err(at(
            "EJ100",
            self.line(),
            self.column(),
            format!(
                "`{mark}` was expected, and {} was found.",
                self.here().token.describe()
            ),
        )
        .with_context(format!("Mark: {mark}")))
    }

    fn want_name(&mut self) -> Result<String, Diagnostic> {
        match self.take() {
            Token::Identifier(name) => Ok(name),
            other => Err(at(
                "EJ101",
                self.line(),
                self.column(),
                format!("A name was expected, and {} was found.", other.describe()),
            )),
        }
    }

    fn qualified(&mut self) -> Result<String, Diagnostic> {
        let mut out = self.want_name()?;
        while self.is_mark(".") && matches!(self.ahead(1), Token::Identifier(_)) {
            self.take();
            out.push('.');
            out.push_str(&self.want_name()?);
        }
        Ok(out)
    }

    fn eat_final(&mut self) -> Result<(), Diagnostic> {
        loop {
            if self.eat_word("final") {
                continue;
            }
            if self.is_mark("@") && !matches!(self.ahead(1), Token::Keyword("interface")) {
                let before = self.held_annotations.len();
                self.skip_annotations()?;
                self.held_annotations.truncate(before);
                continue;
            }
            return Ok(());
        }
    }

    fn skip_annotations(&mut self) -> Result<(), Diagnostic> {
        while self.is_mark("@") && !matches!(self.ahead(1), Token::Keyword("interface")) {
            let line = self.line();
            self.take();
            let name = self.qualified()?;
            let mut values = Vec::new();
            if self.is_mark("(") {
                self.take();
                if !self.is_mark(")") {
                    loop {
                        let named = match (&self.here().token, self.ahead(1)) {
                            (Token::Identifier(word), Token::Punctuation("=")) => {
                                let held = word.clone();
                                self.take();
                                self.take();
                                held
                            }
                            _ => "value".to_string(),
                        };
                        values.push((named, self.annotation_value()?));
                        if !self.eat_mark(",") {
                            break;
                        }
                    }
                }
                self.want_mark(")")?;
            }
            self.held_annotations.push(Annotated { name, values, line });
        }
        Ok(())
    }

    fn annotation_value(&mut self) -> Result<Expression, Diagnostic> {
        if self.is_mark("@") {
            self.take();
            let name = self.qualified()?;
            let mut arguments = Vec::new();
            if self.eat_mark("(") {
                if !self.is_mark(")") {
                    loop {
                        let named = match (&self.here().token, self.ahead(1)) {
                            (Token::Identifier(word), Token::Punctuation("=")) => {
                                let held = word.clone();
                                self.take();
                                self.take();
                                held
                            }
                            _ => "value".to_string(),
                        };
                        arguments.push(Expression::Assign {
                            target: Box::new(Expression::Name(named)),
                            operator: None,
                            value: Box::new(self.annotation_value()?),
                        });
                        if !self.eat_mark(",") {
                            break;
                        }
                    }
                }
                self.want_mark(")")?;
            }
            return Ok(Expression::Call {
                on: None,
                super_call: false,
                name: format!("{NESTED_ANNOTATION}{name}"),
                arguments,
                held: Vec::new(),
            });
        }
        if self.is_mark("{") {
            let line = self.line();
            self.take();
            let mut held = Vec::new();
            while !self.is_mark("}") {
                held.push(self.annotation_value()?);
                if !self.eat_mark(",") {
                    break;
                }
            }
            self.want_mark("}")?;
            return Ok(Expression::ArrayOf {
                of: None,
                values: held,
                line,
            });
        }
        self.expression()
    }

    fn take_annotations(&mut self) -> Vec<Annotated> {
        std::mem::take(&mut self.held_annotations)
    }

    fn looks_like_sealed(&self) -> bool {
        if !matches!(&self.here().token, Token::Identifier(word) if word == "sealed") {
            return false;
        }

        matches!(self.ahead(1), Token::Keyword(_))
            || matches!(self.ahead(1), Token::Identifier(word) if word == "record")
    }

    fn looks_like_non_sealed(&self) -> bool {
        matches!(&self.here().token, Token::Identifier(word) if word == "non")
            && matches!(self.ahead(1), Token::Punctuation("-"))
            && matches!(self.ahead(2), Token::Identifier(word) if word == "sealed")
    }

    fn modifiers(&mut self) -> Result<Modifiers, Diagnostic> {
        let mut found = Modifiers::default();
        loop {
            self.skip_annotations()?;

            if self.looks_like_sealed() {
                self.take();
                found.sealed = true;
                continue;
            }
            if self.looks_like_non_sealed() {
                self.take();
                self.take();
                self.take();
                continue;
            }
            let word = match &self.here().token {
                Token::Keyword(word) => *word,
                _ => break,
            };
            match word {
                "public" => found.public = true,
                "private" => found.private = true,
                "protected" => found.protected = true,
                "static" => found.static_ = true,
                "final" => found.final_ = true,
                "abstract" => found.abstract_ = true,

                "default" if !matches!(self.ahead(1), Token::Punctuation(":")) => {}

                "synchronized" if !matches!(self.ahead(1), Token::Punctuation("(")) => {
                    found.synchronized = true
                }
                "volatile" => found.volatile = true,
                "transient" => found.transient = true,

                "strictfp" => {}

                "native" => found.native_ = true,
                _ => break,
            }
            self.take();
        }
        Ok(found)
    }

    fn written_type(&mut self) -> Result<Written, Diagnostic> {
        let base = match self.here().token.clone() {
            Token::Keyword(word) => match Written::of_keyword(word) {
                Some(found) => {
                    self.take();
                    found
                }
                None => {
                    return Err(at(
                        "EJ103",
                        self.line(),
                        self.column(),
                        format!("A type was expected, and `{word}` was found."),
                    ))
                }
            },
            Token::Identifier(name)
                if name == "var"
                    && !matches!(
                        self.ahead(1),
                        Token::Punctuation(".") | Token::Punctuation("[")
                    ) =>
            {
                self.take();
                Written::Inferred
            }
            Token::Identifier(_) => {
                let mut named = self.qualified()?;
                let mut arguments = self.type_arguments()?;

                while self.is_mark(".") && matches!(self.ahead(1), Token::Identifier(_)) {
                    self.take();
                    named.push('.');
                    named.push_str(&self.want_name()?);
                    let held = self.type_arguments()?;
                    if !held.is_empty() {
                        arguments = held;
                    }
                }
                Written::Named(named, arguments)
            }
            other => {
                return Err(at(
                    "EJ103",
                    self.line(),
                    self.column(),
                    format!("A type was expected, and {} was found.", other.describe()),
                ))
            }
        };

        self.skip_type_arguments()?;

        let mut found = self.erasing(base);
        while self.is_mark("[") && matches!(self.ahead(1), Token::Punctuation("]")) {
            self.take();
            self.take();
            found = Written::Array(Box::new(found));
        }
        Ok(found)
    }

    fn past_type_arguments(&self, ahead: usize) -> Option<usize> {
        if !matches!(self.ahead(ahead), Token::Punctuation("<")) {
            return Some(ahead);
        }
        let mut ahead = ahead;
        let mut depth = 0usize;
        loop {
            match self.ahead(ahead) {
                Token::Punctuation("<") => depth += 1,
                Token::Punctuation(">") => depth -= 1,
                Token::Punctuation(">>") => {
                    if depth < 2 {
                        return None;
                    }
                    depth -= 2;
                }
                Token::Punctuation(">>>") => {
                    if depth < 3 {
                        return None;
                    }
                    depth -= 3;
                }

                Token::Identifier(_)
                | Token::Punctuation(".")
                | Token::Punctuation(",")
                | Token::Punctuation("?")
                | Token::Punctuation("[")
                | Token::Punctuation("]")
                | Token::Keyword(_) => {}
                _ => return None,
            }
            ahead += 1;
            if depth == 0 {
                return Some(ahead);
            }
        }
    }

    fn looks_like_declaration(&self) -> bool {
        match &self.here().token {
            Token::Keyword(word) => Written::of_keyword(word).is_some(),
            Token::Identifier(_) => {
                let mut ahead = 0usize;

                loop {
                    if !matches!(self.ahead(ahead), Token::Identifier(_)) {
                        return false;
                    }
                    ahead += 1;
                    if matches!(self.ahead(ahead), Token::Punctuation(".")) {
                        ahead += 1;
                        continue;
                    }
                    break;
                }

                match self.past_type_arguments(ahead) {
                    Some(past) => ahead = past,
                    None => return false,
                }

                while matches!(self.ahead(ahead), Token::Punctuation("."))
                    && matches!(self.ahead(ahead + 1), Token::Identifier(_))
                {
                    ahead += 2;
                    match self.past_type_arguments(ahead) {
                        Some(past) => ahead = past,
                        None => return false,
                    }
                }

                while matches!(self.ahead(ahead), Token::Punctuation("["))
                    && matches!(self.ahead(ahead + 1), Token::Punctuation("]"))
                {
                    ahead += 2;
                }
                matches!(self.ahead(ahead), Token::Identifier(_))
            }
            _ => false,
        }
    }

    pub fn file(&mut self) -> Result<Vec<Unit>, Diagnostic> {
        self.skip_annotations()?;
        let package = if self.eat_word("package") {
            let name = self.qualified()?;
            self.want_mark(";")?;
            Some(name)
        } else {
            None
        };

        let mut imports = Vec::new();
        let mut static_imports = Vec::new();
        let mut through_modules = Vec::new();
        while self.is_word("import") {
            self.take();
            if matches!(&self.here().token, Token::Identifier(word) if word == "module")
                && matches!(self.ahead(1), Token::Identifier(_))
            {
                self.take();
                let (line, column) = (self.line(), self.column());
                let module = self.qualified()?;
                self.want_mark(";")?;
                let Some(packages) = packages_of_the_module(&module) else {
                    return Err(at(
                        "EJ259",
                        line,
                        column,
                        format!("There is no module `{module}` that Android carries."),
                    )
                    .with_context(format!("Module: {module}"))
                    .with_suggestion(
                        "Android carries java.base, java.sql, java.xml, java.logging and \
                         java.prefs; import a package of one of those instead.",
                    ));
                };
                for package in packages {
                    let held = format!("{package}.*");
                    if !through_modules.contains(&held) {
                        through_modules.push(held);
                    }
                }
                continue;
            }
            let is_static = self.eat_word("static");
            let mut name = self.qualified()?;
            if self.eat_mark(".") {
                self.want_mark("*")?;
                name.push_str(".*");
            }
            self.want_mark(";")?;
            if is_static {
                static_imports.push(name);
            } else {
                imports.push(name);
            }
        }

        for held in through_modules {
            if !imports.contains(&held) {
                imports.push(held);
            }
        }

        let start = self.at;
        match self.declarations_in_the_file(&package, &imports, &static_imports) {
            Ok(declared) => Ok(declared),
            Err(error) if error.code == "EJ104" && package.is_none() => {
                let Some(name) = self.implicit_name.clone() else {
                    return Err(error);
                };
                self.at = start;
                self.wrap_in_an_implicit_class(&name);
                for package in packages_of_the_module("java.base").unwrap_or_default() {
                    let held = format!("{package}.*");
                    if !imports.contains(&held) {
                        imports.push(held);
                    }
                }
                self.declarations_in_the_file(&package, &imports, &static_imports)
            }
            Err(error) => Err(error),
        }
    }

    fn declarations_in_the_file(
        &mut self,
        package: &Option<String>,
        imports: &[String],
        static_imports: &[String],
    ) -> Result<Vec<Unit>, Diagnostic> {
        self.package = package.clone();
        self.imports = imports.to_vec();

        let mut declared = Vec::new();
        while !matches!(self.here().token, Token::End) {
            if self.eat_mark(";") {
                continue;
            }
            let mut beside = Vec::new();
            let one = self.declaration_of_a_type(package, imports, None, false, &mut beside)?;
            declared.push(one);
            declared.append(&mut beside);
            for unit in &mut declared {
                if unit.static_imports.is_empty() {
                    unit.static_imports = static_imports.to_vec();
                }
            }
        }
        if declared.is_empty() {
            return Err(at(
                "EJ104",
                self.line(),
                self.column(),
                "This file declares nothing.",
            )
            .with_suggestion("A file compiled here holds a class or an interface."));
        }
        Ok(declared)
    }

    fn declaration_of_a_type(
        &mut self,
        package: &Option<String>,
        imports: &[String],
        inside: Option<&str>,
        inside_an_interface: bool,
        beside: &mut Vec<Unit>,
    ) -> Result<Unit, Diagnostic> {
        self.skip_annotations()?;
        let mut modifiers = self.modifiers()?;

        if inside_an_interface {
            modifiers.public = true;
        }
        let annotations = self.take_annotations();

        let is_record = matches!(&self.here().token, Token::Identifier(word) if word == "record")
            && matches!(self.ahead(1), Token::Identifier(_));

        let annotation = self.is_mark("@") && matches!(self.ahead(1), Token::Keyword("interface"));
        if annotation {
            self.take();
        }
        let shape = if self.eat_word("interface") {
            Shape::Interface
        } else if self.eat_word("class") {
            Shape::Class
        } else if self.eat_word("enum") {
            Shape::Enum
        } else if is_record {
            self.take();
            Shape::Record
        } else {
            if matches!(&self.here().token, Token::Identifier(word) if word == "module")
                || matches!(&self.here().token, Token::Identifier(word) if word == "open")
            {
                return Err(at(
                    "EJ254",
                    self.line(),
                    self.column(),
                    "A module declaration is not something Android reads.",
                )
                .with_suggestion(
                    "The runtime on a device has no module system: every class is found \
                     by the class loader that was given it. Packages and imports do all \
                     the work here.",
                ));
            }
            return Err(at(
                "EJ104",
                self.line(),
                self.column(),
                format!(
                    "A class or an interface was expected, and {} was found.",
                    self.here().token.describe()
                ),
            )
            .with_suggestion(
                "A file compiled here holds classes, interfaces, enums, records and \
                 annotation types. What is and is not taken is written at the top of \
                 Compilers/Java.rs.",
            ));
        };

        let name = match inside {
            Some(holder) => format!("{holder}${}", self.want_name()?),
            None => self.want_name()?,
        };

        let pushed = self.push_type_parameters(true)?;

        let own_variables: Vec<(String, Written)> = if pushed {
            self.type_variables
                .last()
                .map(|frame| frame.held.clone())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let mut permits: Vec<String> = Vec::new();
        let components = if shape == Shape::Record {
            self.parameters_and_shape()?.0
        } else {
            Vec::new()
        };

        let mut extends = None;
        let mut implements = Vec::new();

        let mut above: Vec<Written> = Vec::new();
        if self.eat_word("extends") {
            loop {
                let named = self.qualified()?;
                above.push(Written::Named(named.clone(), self.type_arguments()?));
                self.skip_type_arguments()?;
                match shape {
                    Shape::Class | Shape::Enum => extends = Some(named),

                    Shape::Interface | Shape::Record => implements.push(named),
                }
                if !self.eat_mark(",") {
                    break;
                }
            }
        }
        if self.eat_word("implements") {
            loop {
                let named = self.qualified()?;
                above.push(Written::Named(named.clone(), self.type_arguments()?));
                self.skip_type_arguments()?;
                implements.push(named);
                if !self.eat_mark(",") {
                    break;
                }
            }
        }

        if matches!(&self.here().token, Token::Identifier(word) if word == "permits") {
            self.take();
            loop {
                permits.push(self.qualified()?);
                self.skip_type_arguments()?;
                if !self.eat_mark(",") {
                    break;
                }
            }
        }

        self.want_mark("{")?;

        let mut constants: Vec<Constant> = Vec::new();
        if shape == Shape::Enum {
            while matches!(self.here().token, Token::Identifier(_)) {
                self.skip_annotations()?;
                let (line, column) = (self.line(), self.column());
                let name = self.want_name()?;
                let arguments = if self.is_mark("(") {
                    self.arguments()?
                } else {
                    Vec::new()
                };
                let body = if self.is_mark("{") {
                    Some(self.written_out_body(line, column, "An enum constant's body")?)
                } else {
                    None
                };
                constants.push(Constant {
                    name,
                    arguments,
                    body,
                    line,
                    column,
                });
                if !self.eat_mark(",") {
                    break;
                }
            }

            self.eat_mark(";");
        }

        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut instance_setup = Vec::new();
        let mut static_setup = Vec::new();
        while !self.is_mark("}") {
            if matches!(self.here().token, Token::End) {
                return Err(at(
                    "EJ105",
                    self.line(),
                    self.column(),
                    "The class was opened and never closed.",
                ));
            }
            if self.eat_mark(";") {
                continue;
            }
            self.member(
                shape,
                &name,
                package,
                imports,
                &mut fields,
                &mut methods,
                &mut instance_setup,
                &mut static_setup,
                beside,
            )?;
        }
        self.want_mark("}")?;

        self.pop_type_parameters(pushed);

        let unit = Unit {
            shape,
            package: package.clone(),
            imports: imports.to_vec(),
            modifiers,
            name,
            extends,
            implements,
            fields,
            methods,
            instance_setup,
            static_setup,
            outer: None,
            permits,
            above,
            stands_for_a_lambda: false,
            annotation,
            annotations,
            static_imports: Vec::new(),
            source: None,
            own_variables,
            enclosed_by: None,
        };
        if shape == Shape::Enum {
            return Ok(as_the_class_it_is(unit, &constants));
        }
        if shape == Shape::Record {
            return as_the_class_a_record_is(unit, &components);
        }

        if let Some(holder) = inside {
            if !modifiers.static_ && shape == Shape::Class && !inside_an_interface {
                return Ok(belonging_to_an_instance(unit, holder, package));
            }
        }
        Ok(unit)
    }

    fn a_type_is_declared_here(&self) -> bool {
        self.is_word("class")
            || self.is_word("interface")
            || self.is_word("enum")
            || (self.is_mark("@") && matches!(self.ahead(1), Token::Keyword("interface")))
            || (matches!(&self.here().token, Token::Identifier(word) if word == "record")
                && matches!(self.ahead(1), Token::Identifier(_)))
    }

    #[allow(clippy::too_many_arguments)]
    fn member(
        &mut self,
        shape: Shape,
        class: &str,
        package: &Option<String>,
        imports: &[String],
        fields: &mut Vec<Field>,
        methods: &mut Vec<Method>,
        instance_setup: &mut Vec<Positioned<Statement>>,
        static_setup: &mut Vec<Positioned<Statement>>,
        beside: &mut Vec<Unit>,
    ) -> Result<(), Diagnostic> {
        let line = self.line();

        let began = self.at;
        self.skip_annotations()?;
        let mut modifiers = self.modifiers()?;
        let annotations = self.take_annotations();
        if shape == Shape::Interface && !modifiers.private {
            modifiers.public = true;
        }

        if self.a_type_is_declared_here() {
            self.at = began;
            let mut theirs = Vec::new();
            let one = self.declaration_of_a_type(
                package,
                imports,
                Some(class),
                shape == Shape::Interface,
                &mut theirs,
            )?;
            beside.push(one);
            beside.append(&mut theirs);
            return Ok(());
        }

        if self.is_mark("{") {
            let block = self.braced_block()?;
            if modifiers.static_ {
                static_setup.extend(block);
            } else {
                instance_setup.extend(block);
            }
            return Ok(());
        }

        let simple = class.rsplit('$').next().unwrap_or(class);

        if shape == Shape::Record
            && matches!(&self.here().token, Token::Identifier(found) if found == simple)
            && matches!(self.ahead(1), Token::Punctuation("{"))
        {
            self.take();
            let body = self.braced_block()?;
            methods.push(Method {
                modifiers,
                returns: Written::Void,
                name: COMPACT.to_string(),
                parameters: Vec::new(),
                body: Some(body),
                constructor: true,
                variadic: false,
                line,
                bridge: false,
                annotations,
                parameter_annotations: Vec::new(),
                default_value: None,
                own_variables: Vec::new(),
                own_bounds: Vec::new(),
                throws: Vec::new(),
            });
            return Ok(());
        }

        let pushed = self.push_type_parameters(false)?;

        if matches!(&self.here().token, Token::Identifier(found) if found == simple)
            && matches!(self.ahead(1), Token::Punctuation("("))
        {
            self.take();
            let (parameters, variadic) = self.parameters_and_shape()?;
            let throws = self.throws()?;
            let body = self.method_body()?;
            let (own_variables, own_bounds) = if pushed {
                self.type_variables
                    .last()
                    .map(|frame| {
                        (
                            frame.held.iter().map(|(name, _)| name.clone()).collect(),
                            frame.held.iter().map(|(_, bound)| bound.clone()).collect(),
                        )
                    })
                    .unwrap_or_default()
            } else {
                (Vec::new(), Vec::new())
            };
            self.pop_type_parameters(pushed);
            methods.push(Method {
                modifiers,
                returns: Written::Void,
                name: "<init>".to_string(),
                parameters,
                body,
                constructor: true,
                variadic,
                line,
                bridge: false,
                annotations,
                parameter_annotations: std::mem::take(&mut self.parameter_annotations),
                default_value: None,
                own_variables,
                own_bounds,
                throws,
            });
            return Ok(());
        }

        let what = self.written_type()?;
        let name = self.want_name()?;

        if self.is_mark("(") {
            let (parameters, variadic) = self.parameters_and_shape()?;
            let throws = self.throws()?;

            let default_value = if self.eat_word("default") {
                let held = self.annotation_value()?;
                self.want_mark(";")?;
                Some(held)
            } else {
                None
            };
            let body = if default_value.is_some() {
                None
            } else {
                self.method_body()?
            };

            if body.is_none()
                && !modifiers.abstract_
                && !modifiers.native_
                && shape != Shape::Interface
            {
                return Err(at(
                    "EJ106",
                    line,
                    1,
                    format!("`{name}` has no body and is not abstract."),
                ));
            }

            let (own_variables, own_bounds) = if pushed {
                self.type_variables
                    .last()
                    .map(|frame| {
                        (
                            frame.held.iter().map(|(name, _)| name.clone()).collect(),
                            frame.held.iter().map(|(_, bound)| bound.clone()).collect(),
                        )
                    })
                    .unwrap_or_default()
            } else {
                (Vec::new(), Vec::new())
            };
            self.pop_type_parameters(pushed);
            methods.push(Method {
                modifiers,
                returns: what,
                name,
                parameters,
                body,
                constructor: false,
                variadic,
                line,
                bridge: false,
                annotations,
                parameter_annotations: std::mem::take(&mut self.parameter_annotations),
                default_value,
                own_variables,
                own_bounds,
                throws,
            });
            return Ok(());
        }
        self.pop_type_parameters(pushed);

        let mut declared = Vec::new();
        let mut name = name;
        loop {
            let mut what = what.clone();
            while self.is_mark("[") && matches!(self.ahead(1), Token::Punctuation("]")) {
                self.take();
                self.take();
                what = Written::Array(Box::new(what));
            }
            let value = if self.eat_mark("=") {
                Some(self.value_or_array()?)
            } else {
                None
            };
            declared.push((what, name, value));
            if !self.eat_mark(",") {
                break;
            }
            name = self.want_name()?;
        }
        self.want_mark(";")?;
        for (what, name, value) in declared {
            let mut modifiers = modifiers;
            if shape == Shape::Interface {
                modifiers.static_ = true;
                modifiers.final_ = true;
            }
            fields.push(Field {
                modifiers,
                what,
                name,
                value,
                line,
                annotations: annotations.clone(),
            });
        }
        Ok(())
    }

    fn type_arguments(&mut self) -> Result<Vec<Written>, Diagnostic> {
        if !self.is_mark("<") {
            return Ok(Vec::new());
        }
        let began = self.at;
        let mut found = Vec::new();
        self.want_mark("<")?;

        if self.is_mark(">") {
            self.take();
            return Ok(Vec::new());
        }
        loop {
            if self.is_mark("?") {
                self.take();
                if self.eat_word("extends") {
                    found.push(self.written_type()?);
                } else if self.eat_word("super") {
                    let lower = self.written_type()?;
                    found.push(if self.in_parameters {
                        lower
                    } else {
                        Written::Named("Object".to_string(), Vec::new())
                    });
                } else {
                    found.push(Written::Named("Object".to_string(), Vec::new()));
                }
                if self.eat_mark(",") {
                    continue;
                }
                break;
            }
            match self.written_type() {
                Ok(one) => found.push(one),
                Err(_) => {
                    self.at = began;
                    self.skip_type_arguments()?;
                    return Ok(Vec::new());
                }
            }
            if self.eat_mark(",") {
                continue;
            }
            break;
        }
        match self.here().token {
            Token::Punctuation(">") => {
                self.take();
            }
            Token::Punctuation(">>") => self.tokens[self.at].token = Token::Punctuation(">"),
            Token::Punctuation(">>>") => self.tokens[self.at].token = Token::Punctuation(">>"),
            _ => {
                self.at = began;
                self.skip_type_arguments()?;
                return Ok(Vec::new());
            }
        }
        Ok(found)
    }

    fn skip_type_arguments(&mut self) -> Result<(), Diagnostic> {
        if !self.is_mark("<") {
            return Ok(());
        }
        let (line, column) = (self.line(), self.column());
        let mut depth = 0usize;
        loop {
            let closes = match &self.here().token {
                Token::Punctuation("<") => {
                    depth += 1;
                    0
                }
                Token::Punctuation(">") => 1,

                Token::Punctuation(">>") => 2,
                Token::Punctuation(">>>") => 3,
                Token::End => {
                    return Err(at(
                        "EJ115",
                        line,
                        column,
                        "A type argument list was opened and never closed.",
                    ))
                }
                _ => 0,
            };
            if closes > depth {
                self.tokens[self.at].token = match closes - depth {
                    1 => Token::Punctuation(">"),
                    _ => Token::Punctuation(">>"),
                };
                return Ok(());
            }
            depth -= closes;
            self.take();
            if depth == 0 {
                return Ok(());
            }
        }
    }

    fn parameters_and_shape(&mut self) -> Result<(Vec<(Written, String)>, bool), Diagnostic> {
        let mut variadic = false;
        self.want_mark("(")?;
        let mut found = Vec::new();
        let mut marked: Vec<Vec<Annotated>> = Vec::new();
        let before = self.held_annotations.len();
        if !self.is_mark(")") {
            loop {
                self.skip_annotations()?;
                self.eat_final()?;
                self.skip_annotations()?;
                marked.push(self.held_annotations.split_off(before));
                self.in_parameters = true;
                let what = self.written_type();
                self.in_parameters = false;
                let mut what = what?;
                if self.eat_mark("...") {
                    what = Written::Array(Box::new(what));
                    variadic = true;
                }
                let name = self.want_name()?;
                while self.is_mark("[") && matches!(self.ahead(1), Token::Punctuation("]")) {
                    self.take();
                    self.take();
                    what = Written::Array(Box::new(what));
                }
                found.push((what, name));
                if !self.eat_mark(",") {
                    break;
                }
            }
        }
        self.want_mark(")")?;
        self.parameter_annotations = marked;
        Ok((found, variadic))
    }

    fn throws(&mut self) -> Result<Vec<String>, Diagnostic> {
        let mut named = Vec::new();
        if self.eat_word("throws") {
            loop {
                named.push(self.qualified()?);
                if !self.eat_mark(",") {
                    break;
                }
            }
        }
        Ok(named)
    }

    fn method_body(&mut self) -> Result<Option<Vec<Positioned<Statement>>>, Diagnostic> {
        if self.eat_mark(";") {
            return Ok(None);
        }
        self.want_mark("{")?;
        let mut found = Vec::new();
        while !self.is_mark("}") {
            if matches!(self.here().token, Token::End) {
                return Err(at(
                    "EJ107",
                    self.line(),
                    self.column(),
                    "A method body was opened and never closed.",
                ));
            }
            found.push(self.statement()?);
        }
        self.want_mark("}")?;
        Ok(Some(found))
    }

    fn statement(&mut self) -> Result<Positioned<Statement>, Diagnostic> {
        let (line, column) = (self.line(), self.column());
        let node = self.statement_node(line, column)?;
        Ok(Positioned { node, line, column })
    }

    fn statement_node(&mut self, line: u32, column: u32) -> Result<Statement, Diagnostic> {
        let annotated = self.held_annotations.len();
        self.skip_annotations()?;
        {
            let mut at = 0usize;
            while matches!(self.ahead(at), Token::Keyword(word) if
                matches!(*word, "final" | "abstract" | "static"))
            {
                at += 1;
            }
            let opens = matches!(self.ahead(at), Token::Keyword(word) if
                matches!(*word, "class" | "interface" | "enum"))
                || (matches!(&self.ahead(at), Token::Identifier(word) if *word == "record")
                    && matches!(self.ahead(at + 1), Token::Identifier(_))
                    && matches!(self.ahead(at + 2), Token::Punctuation("(")));
            if opens {
                let package = self.package.clone();
                let imports = self.imports.clone();
                let mut beside = Vec::new();
                let one =
                    self.declaration_of_a_type(&package, &imports, None, false, &mut beside)?;
                let mut made = vec![one];
                made.append(&mut beside);
                return Ok(Statement::Locally(made));
            }
        }
        self.held_annotations.truncate(annotated);

        if self.eat_word("synchronized") {
            self.want_mark("(")?;
            let on = self.expression()?;
            self.want_mark(")")?;
            let body = self.braced_block()?;
            return Ok(Statement::Synchronized { on, body });
        }

        if self.eat_word("assert") {
            let condition = self.expression()?;
            let said = if self.eat_mark(":") {
                Some(self.expression()?)
            } else {
                None
            };
            self.want_mark(";")?;
            let _ = column;
            return Ok(Statement::Assert { condition, said });
        }

        if matches!(self.here().token, Token::Identifier(_))
            && matches!(self.ahead(1), Token::Punctuation(":"))
        {
            let label = self.want_name()?;
            self.want_mark(":")?;
            let body = Box::new(self.statement()?);
            return Ok(Statement::Labelled { label, body });
        }

        if self.is_mark("{") {
            self.take();
            let mut found = Vec::new();
            while !self.is_mark("}") {
                if matches!(self.here().token, Token::End) {
                    return Err(at("EJ108", line, column, "A block was never closed."));
                }
                found.push(self.statement()?);
            }
            self.want_mark("}")?;
            return Ok(Statement::Block(found));
        }

        if self.eat_mark(";") {
            return Ok(Statement::Nothing);
        }

        if self.eat_word("if") {
            self.want_mark("(")?;
            let condition = self.expression()?;
            self.want_mark(")")?;
            let then = Box::new(self.statement()?);
            let otherwise = if self.eat_word("else") {
                Some(Box::new(self.statement()?))
            } else {
                None
            };
            return Ok(Statement::If {
                condition,
                then,
                otherwise,
            });
        }

        if self.eat_word("while") {
            self.want_mark("(")?;
            let condition = self.expression()?;
            self.want_mark(")")?;
            let body = Box::new(self.statement()?);
            return Ok(Statement::While { condition, body });
        }

        for (word, to_super) in [("super", true), ("this", false)] {
            if self.is_word(word) && matches!(self.ahead(1), Token::Punctuation("(")) {
                self.take();
                let arguments = self.arguments()?;
                self.want_mark(";")?;
                return Ok(Statement::Chain {
                    to_super,
                    arguments,
                });
            }
        }

        if self.eat_word("do") {
            let body = Box::new(self.statement()?);
            if !self.eat_word("while") {
                return Err(at(
                    "EJ109",
                    self.line(),
                    self.column(),
                    "A `do` block is followed by `while`.",
                ));
            }
            self.want_mark("(")?;
            let condition = self.expression()?;
            self.want_mark(")")?;
            self.want_mark(";")?;
            return Ok(Statement::DoWhile { body, condition });
        }

        if self.eat_word("for") {
            return self.for_statement(line, column);
        }

        if self.eat_word("switch") {
            return self.switch_statement(line, column);
        }

        if self.eat_word("try") {
            return self.try_statement(line, column);
        }

        if self.eat_word("throw") {
            let what = self.expression()?;
            self.want_mark(";")?;
            return Ok(Statement::Throw(what));
        }

        if matches!(&self.here().token, Token::Identifier(word) if word == "yield")
            && !matches!(
                self.ahead(1),
                Token::Punctuation("(") | Token::Punctuation("=") | Token::Punctuation(".")
            )
        {
            self.take();
            let what = self.expression()?;
            self.want_mark(";")?;
            return Ok(Statement::Yield(what));
        }

        if self.eat_word("return") {
            let value = if self.is_mark(";") {
                None
            } else {
                Some(self.expression()?)
            };
            self.want_mark(";")?;
            return Ok(Statement::Return(value));
        }

        if self.eat_word("break") {
            let label = self.optional_label()?;
            self.want_mark(";")?;
            return Ok(Statement::Break(label));
        }

        if self.eat_word("continue") {
            let label = self.optional_label()?;
            self.want_mark(";")?;
            return Ok(Statement::Continue(label));
        }

        if self.is_word("final") && self.ahead(1) != &Token::End {
            self.eat_final()?;
            let statement = self.declaration()?;
            self.want_mark(";")?;
            return Ok(statement);
        }

        if self.looks_like_declaration() {
            let statement = self.declaration()?;
            self.want_mark(";")?;
            return Ok(statement);
        }

        let expression = self.expression()?;
        self.want_mark(";")?;
        Ok(Statement::Express(expression))
    }

    fn declaration(&mut self) -> Result<Statement, Diagnostic> {
        let base = self.written_type()?;
        let mut declared = Vec::new();
        loop {
            let (line, column) = (self.line(), self.column());
            let name = self.want_name()?;
            let mut what = base.clone();
            while self.is_mark("[") && matches!(self.ahead(1), Token::Punctuation("]")) {
                self.take();
                self.take();
                what = Written::Array(Box::new(what));
            }
            let value = if self.eat_mark("=") {
                Some(self.value_or_array()?)
            } else {
                None
            };
            declared.push(Positioned {
                node: Statement::Declare { what, name, value },
                line,
                column,
            });
            if !self.eat_mark(",") {
                break;
            }
        }
        if declared.len() == 1 {
            return Ok(declared.remove(0).node);
        }
        Ok(Statement::Several(declared))
    }

    fn optional_label(&mut self) -> Result<Option<String>, Diagnostic> {
        if matches!(self.here().token, Token::Identifier(_)) {
            return Ok(Some(self.want_name()?));
        }
        Ok(None)
    }

    fn switch_statement(&mut self, line: u32, column: u32) -> Result<Statement, Diagnostic> {
        self.want_mark("(")?;
        let subject = self.expression()?;
        self.want_mark(")")?;
        self.want_mark("{")?;

        let mut arms: Vec<Arm> = Vec::new();
        let mut arrow_form: Option<bool> = None;
        while !self.is_mark("}") {
            if matches!(self.here().token, Token::End) {
                return Err(at("EJ110", line, column, "A `switch` was never closed."));
            }
            let (arm_line, arm_column) = (self.line(), self.column());

            let mut labels = Vec::new();
            let mut pattern = None;
            let mut guard = None;
            if self.eat_word("default") {
            } else if self.eat_word("case") {
                if self.looks_like_a_type_pattern() {
                    pattern = Some(self.pattern()?);
                    if matches!(&self.here().token, Token::Identifier(word) if word == "when") {
                        self.take();

                        let outside = std::mem::replace(&mut self.bare_lambda, false);
                        guard = Some(self.expression()?);
                        self.bare_lambda = outside;
                    }
                } else {
                    loop {
                        labels.push(self.case_label()?);
                        if !self.eat_mark(",") {
                            break;
                        }
                    }
                }
            } else {
                return Err(at(
                    "EJ111",
                    arm_line,
                    arm_column,
                    format!(
                        "A `switch` holds `case` and `default` arms, and {} was found.",
                        self.here().token.describe()
                    ),
                ));
            }

            let arrow = if self.eat_mark("->") {
                true
            } else {
                self.want_mark(":")?;
                false
            };
            match arrow_form {
                None => arrow_form = Some(arrow),
                Some(first) if first != arrow => {
                    return Err(at(
                        "EJ112",
                        arm_line,
                        arm_column,
                        "A `switch` uses `->` for every arm or `:` for every arm, not both.",
                    ))
                }
                Some(_) => {}
            }

            let mut body = Vec::new();
            if arrow {
                if self.eat_word("throw") {
                    let what = self.expression()?;
                    self.want_mark(";")?;
                    let (l, c) = (arm_line, arm_column);
                    body.push(Positioned {
                        node: Statement::Throw(what),
                        line: l,
                        column: c,
                    });
                } else {
                    body.push(self.statement()?);
                }
            } else {
                while !self.is_mark("}") && !self.is_word("case") && !self.is_word("default") {
                    if matches!(self.here().token, Token::End) {
                        return Err(at("EJ110", line, column, "A `switch` was never closed."));
                    }
                    body.push(self.statement()?);
                }
            }

            arms.push(Arm {
                labels,
                pattern,
                guard,
                arrow,
                body,
                line: arm_line,
                column: arm_column,
            });
        }
        self.want_mark("}")?;

        if arms
            .iter()
            .filter(|arm| arm.labels.is_empty() && arm.pattern.is_none())
            .count()
            > 1
        {
            return Err(at(
                "EJ113",
                line,
                column,
                "A `switch` has one `default` arm at most.",
            ));
        }

        Ok(Statement::Switch { subject, arms })
    }

    fn looks_like_a_type_pattern(&self) -> bool {
        let mut at;

        if matches!(self.ahead(0), Token::Keyword(word) if Written::of_keyword(word).is_some()) {
            at = 1;
        } else {
            if !matches!(self.ahead(0), Token::Identifier(_)) {
                return false;
            }
            at = 1;
            while matches!(self.ahead(at), Token::Punctuation("."))
                && matches!(self.ahead(at + 1), Token::Identifier(_))
            {
                at += 2;
            }

            if matches!(self.ahead(at), Token::Punctuation("<")) {
                let mut depth = 0usize;
                loop {
                    match self.ahead(at) {
                        Token::Punctuation("<") => depth += 1,
                        Token::Punctuation(">") => depth -= 1,
                        Token::Punctuation(">>") => depth = depth.saturating_sub(2),
                        Token::End => return false,
                        _ => {}
                    }
                    at += 1;
                    if depth == 0 {
                        break;
                    }
                }
            }
        }
        while matches!(self.ahead(at), Token::Punctuation("["))
            && matches!(self.ahead(at + 1), Token::Punctuation("]"))
        {
            at += 2;
        }

        matches!(
            self.ahead(at),
            Token::Identifier(_) | Token::Punctuation("(")
        )
    }

    fn pattern(&mut self) -> Result<Pattern, Diagnostic> {
        self.eat_final()?;
        let what = self.written_type()?;
        if self.is_mark("(") {
            self.take();
            let mut parts = Vec::new();
            if !self.is_mark(")") {
                loop {
                    parts.push(self.pattern()?);
                    if !self.eat_mark(",") {
                        break;
                    }
                }
            }
            self.want_mark(")")?;
            return Ok(Pattern {
                what,
                name: String::new(),
                parts,
                taken: true,
            });
        }
        let name = match &self.here().token {
            Token::Identifier(_) => self.want_name()?,
            _ => String::new(),
        };
        Ok(Pattern {
            what,
            name,
            parts: Vec::new(),
            taken: false,
        })
    }

    fn case_label(&mut self) -> Result<Expression, Diagnostic> {
        let (line, column) = (self.line(), self.column());
        let begins = matches!(
            self.here().token,
            Token::Int(_)
                | Token::Long(_)
                | Token::Float(_)
                | Token::Double(_)
                | Token::Char(_)
                | Token::Str(_)
                | Token::True
                | Token::False
                | Token::Identifier(_)
                | Token::Null
                | Token::Punctuation("-" | "+" | "~" | "!" | "(")
        );
        if !begins {
            return Err(at(
                "EJ239",
                line,
                column,
                format!(
                    "A `case` answers to a constant, and {} was found.",
                    self.here().token.describe()
                ),
            ));
        }
        if matches!(self.here().token, Token::Null) {
            self.take();
            return Ok(Expression::Null);
        }
        let outside = std::mem::replace(&mut self.bare_lambda, false);
        let found = self.binary(0);
        self.bare_lambda = outside;
        found
    }

    fn try_statement(&mut self, line: u32, column: u32) -> Result<Statement, Diagnostic> {
        let mut resources: Vec<Positioned<Statement>> = Vec::new();
        if self.eat_mark("(") {
            loop {
                if self.is_mark(")") {
                    break;
                }
                let (line, column) = (self.line(), self.column());
                self.eat_final()?;

                if matches!(self.here().token, Token::Identifier(_))
                    && matches!(self.ahead(1), Token::Punctuation(";" | ")"))
                {
                    let named = self.want_name()?;
                    resources.push(Positioned {
                        node: Statement::Declare {
                            what: Written::Inferred,
                            name: format!("$closing{}", resources.len()),
                            value: Some(Expression::Name(named)),
                        },
                        line,
                        column,
                    });
                    if !self.eat_mark(";") {
                        break;
                    }
                    continue;
                }
                let node = self.declaration()?;
                let Statement::Declare { value: Some(_), .. } = &node else {
                    return Err(at(
                        "EJ118",
                        line,
                        column,
                        "Something a `try` closes has to be given a value.",
                    ));
                };
                resources.push(Positioned { node, line, column });
                if !self.eat_mark(";") {
                    break;
                }
            }
            self.want_mark(")")?;
            if resources.is_empty() {
                return Err(at(
                    "EJ118",
                    line,
                    column,
                    "A `try` with brackets holds at least one thing to close.",
                ));
            }
        }

        let body = self.braced_block()?;

        let mut catches = Vec::new();
        while self.is_word("catch") {
            let (catch_line, catch_column) = (self.line(), self.column());
            self.take();
            self.want_mark("(")?;
            self.eat_final()?;
            let mut types = vec![self.written_type()?];
            while self.eat_mark("|") {
                types.push(self.written_type()?);
            }
            let name = self.want_name()?;
            self.want_mark(")")?;
            let body = self.braced_block()?;
            catches.push(Catch {
                types,
                name,
                body,
                line: catch_line,
                column: catch_column,
            });
        }

        let finally = if self.eat_word("finally") {
            Some(self.braced_block()?)
        } else {
            None
        };

        if catches.is_empty() && finally.is_none() && resources.is_empty() {
            return Err(at(
                "EJ114",
                line,
                column,
                "A `try` needs a `catch`, a `finally`, or something to close.",
            ));
        }

        if resources.is_empty() {
            return Ok(Statement::Try {
                body,
                catches,
                finally,
            });
        }

        let mut inner = Statement::Block(body);
        for resource in resources.into_iter().rev() {
            let Statement::Declare { name, .. } = &resource.node else {
                unreachable!("a resource is a declaration");
            };
            let close = Positioned {
                node: Statement::Express(Expression::Call {
                    on: Some(Box::new(Expression::Name(name.clone()))),
                    super_call: false,
                    held: Vec::new(),
                    name: "close".to_string(),
                    arguments: Vec::new(),
                }),
                line: resource.line,
                column: resource.column,
            };
            let held = Positioned {
                node: inner,
                line: resource.line,
                column: resource.column,
            };
            inner = Statement::Block(vec![
                resource,
                Positioned {
                    node: Statement::Try {
                        body: vec![held],
                        catches: Vec::new(),
                        finally: Some(vec![close]),
                    },
                    line,
                    column,
                },
            ]);
        }

        if catches.is_empty() && finally.is_none() {
            return Ok(inner);
        }
        Ok(Statement::Try {
            body: vec![Positioned {
                node: inner,
                line,
                column,
            }],
            catches,
            finally,
        })
    }

    fn braced_block(&mut self) -> Result<Vec<Positioned<Statement>>, Diagnostic> {
        let (line, column) = (self.line(), self.column());
        self.want_mark("{")?;
        let mut found = Vec::new();
        while !self.is_mark("}") {
            if matches!(self.here().token, Token::End) {
                return Err(at("EJ108", line, column, "A block was never closed."));
            }
            found.push(self.statement()?);
        }
        self.want_mark("}")?;
        Ok(found)
    }

    fn for_statement(&mut self, line: u32, column: u32) -> Result<Statement, Diagnostic> {
        self.want_mark("(")?;

        let enhanced = {
            let mut ahead = 0usize;
            let mut depth = 0usize;
            let mut found = false;
            loop {
                match self.ahead(ahead) {
                    Token::Punctuation("(") => depth += 1,
                    Token::Punctuation(")") if depth == 0 => break,
                    Token::Punctuation(")") => depth -= 1,
                    Token::Punctuation(";" | "=") if depth == 0 => break,
                    Token::Punctuation(":") if depth == 0 => {
                        found = true;
                        break;
                    }
                    Token::End => break,
                    _ => {}
                }
                ahead += 1;
            }
            found
        };

        if enhanced {
            self.eat_final()?;
            let what = self.written_type()?;
            let name = self.want_name()?;
            self.want_mark(":")?;
            let over = self.expression()?;
            self.want_mark(")")?;
            let body = Box::new(self.statement()?);
            let _ = (line, column);
            return Ok(Statement::ForEach {
                what,
                name,
                over,
                body,
            });
        }

        let mut start = Vec::new();
        if !self.is_mark(";") {
            let (line, column) = (self.line(), self.column());
            let node = if self.is_word("final") || self.looks_like_declaration() {
                self.eat_final()?;
                self.declaration()?
            } else {
                Statement::Express(self.expression()?)
            };
            start.push(Positioned { node, line, column });
        }
        self.want_mark(";")?;

        let condition = if self.is_mark(";") {
            None
        } else {
            Some(self.expression()?)
        };
        self.want_mark(";")?;

        let mut step = Vec::new();
        if !self.is_mark(")") {
            loop {
                step.push(self.expression()?);
                if !self.eat_mark(",") {
                    break;
                }
            }
        }
        self.want_mark(")")?;

        let body = Box::new(self.statement()?);
        Ok(Statement::For {
            start,
            condition,
            step,
            body,
        })
    }

    pub fn expression(&mut self) -> Result<Expression, Diagnostic> {
        let left = self.binary(0)?;

        if self.is_mark("?") {
            self.take();
            let then = self.expression()?;
            self.want_mark(":")?;
            let otherwise = self.expression()?;
            return Ok(Expression::Conditional {
                condition: Box::new(left),
                then: Box::new(then),
                otherwise: Box::new(otherwise),
            });
        }

        let compound = [
            ("=", None),
            ("+=", Some(Binary::Add)),
            ("-=", Some(Binary::Subtract)),
            ("*=", Some(Binary::Multiply)),
            ("/=", Some(Binary::Divide)),
            ("%=", Some(Binary::Remainder)),
            ("&=", Some(Binary::And)),
            ("|=", Some(Binary::Or)),
            ("^=", Some(Binary::Xor)),
            ("<<=", Some(Binary::ShiftLeft)),
            (">>=", Some(Binary::ShiftRight)),
            (">>>=", Some(Binary::UnsignedShiftRight)),
        ];
        for (mark, operator) in compound {
            if self.is_mark(mark) {
                self.take();
                let value = self.expression()?;
                return Ok(Expression::Assign {
                    target: Box::new(left),
                    operator,
                    value: Box::new(value),
                });
            }
        }

        Ok(left)
    }

    fn written_out_body(&mut self, line: u32, column: u32, what: &str) -> Result<Body, Diagnostic> {
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut instance_setup = Vec::new();
        let mut static_setup = Vec::new();
        let mut nested = Vec::new();
        self.want_mark("{")?;
        while !self.is_mark("}") {
            if matches!(self.here().token, Token::End) {
                return Err(at(
                    "EJ105",
                    line,
                    column,
                    format!("{what} was opened and never closed."),
                ));
            }
            if self.eat_mark(";") {
                continue;
            }
            let began = self.at;
            self.skip_annotations()?;
            self.modifiers()?;
            self.take_annotations();
            let declares = self.a_type_is_declared_here();
            self.at = began;
            if declares {
                let package = self.package.clone();
                let imports = self.imports.clone();
                let mut theirs = Vec::new();
                let one =
                    self.declaration_of_a_type(&package, &imports, None, false, &mut theirs)?;
                nested.push(one);
                nested.append(&mut theirs);
                continue;
            }
            self.member(
                Shape::Class,
                "",
                &None,
                &[],
                &mut fields,
                &mut methods,
                &mut instance_setup,
                &mut static_setup,
                &mut Vec::new(),
            )?;
        }
        self.want_mark("}")?;
        Ok(Body {
            fields,
            methods,
            instance_setup,
            static_setup,
            nested,
        })
    }

    fn binary(&mut self, least: u8) -> Result<Expression, Diagnostic> {
        let mut left = self.unary()?;
        loop {
            if self.is_word("instanceof") {
                if least > 9 {
                    break;
                }
                self.take();

                let pattern = self.pattern()?;
                left = Expression::InstanceOf {
                    of: Box::new(left),
                    pattern: Box::new(pattern),
                };
                continue;
            }
            let Token::Punctuation(mark) = self.here().token else {
                break;
            };
            let Some(operator) = Binary::of_mark(mark) else {
                break;
            };
            if operator.precedence() < least {
                break;
            }
            self.take();
            let right = self.binary(operator.precedence() + 1)?;
            left = Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expression, Diagnostic> {
        for (mark, by) in [("++", 1i32), ("--", -1)] {
            if self.is_mark(mark) {
                self.take();
                let target = self.unary()?;
                return Ok(Expression::Step {
                    target: Box::new(target),
                    by,
                    after: false,
                });
            }
        }
        for (mark, operator) in [
            ("-", Unary::Negate),
            ("+", Unary::Plus),
            ("!", Unary::Not),
            ("~", Unary::Complement),
        ] {
            if self.is_mark(mark) {
                self.take();
                let of = self.unary()?;

                if operator == Unary::Negate {
                    if let Expression::Int(value) = of {
                        return Ok(Expression::Int(-value));
                    }
                    if let Expression::Long(value) = of {
                        return Ok(Expression::Long(value.wrapping_neg()));
                    }
                }
                return Ok(Expression::Unary {
                    operator,
                    of: Box::new(of),
                });
            }
        }

        if self.is_mark("(") && self.looks_like_cast() {
            self.take();
            let to = self.written_type()?;
            let mut also = Vec::new();
            while self.is_mark("&") {
                self.take();
                also.push(self.written_type()?);
            }
            self.want_mark(")")?;
            let of = self.unary()?;
            return Ok(Expression::Cast {
                to,
                of: Box::new(of),
                also,
            });
        }

        self.postfix()
    }

    fn looks_like_cast(&self) -> bool {
        if let Token::Keyword(word) = self.ahead(1) {
            if Written::of_keyword(word).is_some() {
                return true;
            }
        }
        if !matches!(self.ahead(1), Token::Identifier(_)) {
            return false;
        }
        let mut ahead = 1usize;
        loop {
            if !matches!(self.ahead(ahead), Token::Identifier(_)) {
                return false;
            }
            ahead += 1;

            match self.past_type_arguments(ahead) {
                Some(past) => ahead = past,
                None => return false,
            }
            if matches!(self.ahead(ahead), Token::Punctuation(".")) {
                ahead += 1;
                continue;
            }
            if matches!(self.ahead(ahead), Token::Punctuation("&"))
                && matches!(self.ahead(ahead + 1), Token::Identifier(_))
            {
                ahead += 1;
                continue;
            }
            break;
        }
        while matches!(self.ahead(ahead), Token::Punctuation("["))
            && matches!(self.ahead(ahead + 1), Token::Punctuation("]"))
        {
            ahead += 2;
        }
        if !matches!(self.ahead(ahead), Token::Punctuation(")")) {
            return false;
        }

        matches!(
            self.ahead(ahead + 1),
            Token::Identifier(_)
                | Token::Str(_)
                | Token::Int(_)
                | Token::Long(_)
                | Token::Char(_)
                | Token::Float(_)
                | Token::Double(_)
                | Token::True
                | Token::False
                | Token::Null
                | Token::Keyword("this")
                | Token::Keyword("super")
                | Token::Keyword("new")
                | Token::Keyword("switch")
                | Token::Punctuation("(")
                | Token::Punctuation("!")
                | Token::Punctuation("~")
        ) || matches!(
            self.ahead(ahead + 1),
            Token::Keyword(word) if Written::of_keyword(word).is_some() || *word == "void"
        )
    }

    fn postfix(&mut self) -> Result<Expression, Diagnostic> {
        let (line, _) = (self.line(), self.column());
        let mut found = self.primary()?;
        loop {
            if self.is_mark("::") {
                self.take();
                found = self.method_reference(found, line)?;
                continue;
            }
            if self.is_mark(".") {
                self.take();

                if self.is_word("class") || self.is_word("this") {
                    let Some(named) = as_a_written_type(&found) else {
                        return Err(at(
                            "EJ122",
                            self.line(),
                            self.column(),
                            "Only a class can be written in front of this.",
                        ));
                    };
                    let literal = self.is_word("class");
                    self.take();
                    found = if literal {
                        Expression::ClassLiteral { of: named, line }
                    } else {
                        Expression::OuterThis { of: named, line }
                    };
                    continue;
                }

                if matches!(found, Expression::Super) {
                    let name = self.want_name()?;
                    found = if self.is_mark("(") {
                        Expression::Call {
                            on: None,
                            super_call: true,
                            held: Vec::new(),
                            name,
                            arguments: self.arguments()?,
                        }
                    } else {
                        Expression::Field {
                            of: Box::new(Expression::Super),
                            name,
                        }
                    };
                    continue;
                }

                if self.is_word("super") {
                    self.take();
                    self.want_mark(".")?;
                    let name = self.want_name()?;
                    let arguments = self.arguments()?;
                    found = Expression::Call {
                        on: Some(Box::new(found)),
                        super_call: true,
                        held: Vec::new(),
                        name,
                        arguments,
                    };
                    continue;
                }

                if self.is_word("new") {
                    self.take();
                    let what = self.type_name_only()?;
                    let arguments = self.arguments()?;
                    found = Expression::New {
                        what,
                        arguments,
                        outer: Some(Box::new(found)),
                    };
                    continue;
                }

                let witness = self.type_arguments()?;
                let name = self.want_name()?;
                if self.is_mark("(") {
                    let arguments = self.arguments()?;
                    found = Expression::Call {
                        on: Some(Box::new(found)),
                        super_call: false,
                        held: witness,
                        name,
                        arguments,
                    };
                } else {
                    found = Expression::Field {
                        of: Box::new(found),
                        name,
                    };
                }
                continue;
            }

            if self.is_mark("[") && matches!(self.ahead(1), Token::Punctuation("]")) {
                let Some(mut named) = as_a_written_type(&found) else {
                    return Err(at(
                        "EJ122",
                        self.line(),
                        self.column(),
                        "Only a class can be written in front of this.",
                    ));
                };
                while self.is_mark("[") && matches!(self.ahead(1), Token::Punctuation("]")) {
                    self.take();
                    self.take();
                    named = Written::Array(Box::new(named));
                }
                self.want_mark(".")?;
                if !self.eat_word("class") {
                    return Err(at(
                        "EJ122",
                        self.line(),
                        self.column(),
                        "An array type written here is only there to name its class.",
                    ));
                }
                found = Expression::ClassLiteral { of: named, line };
                continue;
            }
            if self.is_mark("[") {
                self.take();
                let index = self.expression()?;
                self.want_mark("]")?;
                found = Expression::Index {
                    of: Box::new(found),
                    at: Box::new(index),
                };
                continue;
            }
            for (mark, by) in [("++", 1i32), ("--", -1)] {
                if self.is_mark(mark) {
                    self.take();
                    found = Expression::Step {
                        target: Box::new(found),
                        by,
                        after: true,
                    };
                }
            }
            break;
        }
        Ok(found)
    }

    fn type_name_only(&mut self) -> Result<Written, Diagnostic> {
        let base = match self.here().token.clone() {
            Token::Keyword(word) => match Written::of_keyword(word) {
                Some(one) => {
                    self.take();
                    one
                }
                None => {
                    return Err(at(
                        "EJ103",
                        self.line(),
                        self.column(),
                        format!("A type was expected, and `{word}` was found."),
                    ))
                }
            },
            _ => {
                let named = self.qualified()?;
                Written::Named(named, self.type_arguments()?)
            }
        };
        self.skip_type_arguments()?;
        Ok(self.erasing(base))
    }

    fn array_values(&mut self) -> Result<Vec<Expression>, Diagnostic> {
        self.want_mark("{")?;
        let mut found = Vec::new();
        while !self.is_mark("}") {
            found.push(self.value_or_array()?);
            if !self.eat_mark(",") {
                break;
            }
        }
        self.want_mark("}")?;
        Ok(found)
    }

    fn value_or_array(&mut self) -> Result<Expression, Diagnostic> {
        if self.is_mark("{") {
            let line = self.line();
            let values = self.array_values()?;
            return Ok(Expression::ArrayOf {
                of: None,
                values,
                line,
            });
        }
        self.expression()
    }

    fn arguments(&mut self) -> Result<Vec<Expression>, Diagnostic> {
        self.want_mark("(")?;
        let mut found = Vec::new();

        let outside = std::mem::replace(&mut self.bare_lambda, true);
        if !self.is_mark(")") {
            loop {
                found.push(self.expression()?);
                if !self.eat_mark(",") {
                    self.bare_lambda = outside;
                    break;
                }
            }
        } else {
            self.bare_lambda = outside;
        }
        self.want_mark(")")?;
        Ok(found)
    }

    fn looks_like_lambda(&self) -> bool {
        let mut ahead = 1usize;
        let mut depth = 1usize;
        loop {
            match self.ahead(ahead) {
                Token::Punctuation("(") => depth += 1,
                Token::Punctuation(")") => {
                    depth -= 1;
                    if depth == 0 {
                        return matches!(self.ahead(ahead + 1), Token::Punctuation("->"));
                    }
                }
                Token::End => return false,
                _ => {}
            }
            ahead += 1;
            if ahead > 64 {
                return false;
            }
        }
    }

    fn lambda(&mut self, line: u32) -> Result<Expression, Diagnostic> {
        let mut parameters: Vec<(Option<Written>, String)> = Vec::new();
        if self.eat_mark("(") {
            while !self.is_mark(")") {
                self.eat_final()?;

                let written = if matches!(self.here().token, Token::Identifier(_))
                    && matches!(
                        self.ahead(1),
                        Token::Punctuation(",") | Token::Punctuation(")")
                    ) {
                    None
                } else {
                    Some(self.written_type()?)
                };
                parameters.push((written, self.want_name()?));
                if !self.eat_mark(",") {
                    break;
                }
            }
            self.want_mark(")")?;
        } else {
            parameters.push((None, self.want_name()?));
        }
        self.want_mark("->")?;

        if self.is_mark("{") {
            let body = self.braced_block()?;
            return Ok(Expression::Lambda {
                parameters,
                body,
                expression: false,
                line,
            });
        }
        let (value_line, column) = (self.line(), self.column());
        let value = self.expression()?;
        Ok(Expression::Lambda {
            parameters,
            body: vec![Positioned {
                node: Statement::Express(value),
                line: value_line,
                column,
            }],
            expression: true,
            line,
        })
    }

    fn method_reference(&mut self, on: Expression, line: u32) -> Result<Expression, Diagnostic> {
        let name = if self.eat_word("new") {
            "<init>".to_string()
        } else {
            self.want_name()?
        };
        Ok(Expression::MethodRef {
            on: Box::new(on),
            name,
            line,
        })
    }

    fn primary(&mut self) -> Result<Expression, Diagnostic> {
        let (line, column) = (self.line(), self.column());

        if let Token::Keyword(word) = self.here().token {
            if Written::of_keyword(word).is_some() {
                let mut at = 1usize;
                while matches!(self.ahead(at), Token::Punctuation("["))
                    && matches!(self.ahead(at + 1), Token::Punctuation("]"))
                {
                    at += 2;
                }
                if matches!(self.ahead(at), Token::Punctuation("."))
                    && matches!(self.ahead(at + 1), Token::Keyword("class"))
                {
                    let mut of = Written::of_keyword(word).expect("just checked");
                    self.take();
                    while self.is_mark("[") && matches!(self.ahead(1), Token::Punctuation("]")) {
                        self.take();
                        self.take();
                        of = Written::Array(Box::new(of));
                    }
                    self.take();
                    self.take();
                    return Ok(Expression::ClassLiteral { of, line });
                }
            }
        }

        if self.is_mark("(") {
            if self.bare_lambda && self.looks_like_lambda() {
                return self.lambda(line);
            }
            self.take();
            let outside = std::mem::replace(&mut self.bare_lambda, true);
            let inner = self.expression()?;
            self.bare_lambda = outside;
            self.want_mark(")")?;
            return Ok(inner);
        }

        if self.bare_lambda
            && matches!(self.here().token, Token::Identifier(_))
            && matches!(self.ahead(1), Token::Punctuation("->"))
        {
            return self.lambda(line);
        }

        if self.eat_word("this") {
            return Ok(Expression::This);
        }

        if self.eat_word("super") {
            return Ok(Expression::Super);
        }

        if self.eat_word("new") {
            let held = self.at;
            let mut what = self.written_type()?;
            if matches!(what, Written::Array(_)) {
                self.at = held;
                what = self.type_name_only()?;
            }
            if self.is_mark("[") {
                let mut lengths = Vec::new();
                let mut empty = 0usize;
                while self.is_mark("[") {
                    self.take();
                    if self.eat_mark("]") {
                        empty += 1;
                        continue;
                    }
                    if empty > 0 {
                        return Err(at(
                            "EJ120",
                            line,
                            column,
                            "A size cannot follow an empty pair of brackets.",
                        ));
                    }
                    lengths.push(self.expression()?);
                    self.want_mark("]")?;
                }
                if lengths.is_empty() {
                    if empty == 0 {
                        return Err(at("EJ120", line, column, "An array needs a size."));
                    }
                    let mut of = what;
                    for _ in 1..empty {
                        of = Written::Array(Box::new(of));
                    }
                    let values = self.array_values()?;
                    return Ok(Expression::ArrayOf {
                        of: Some(of),
                        values,
                        line,
                    });
                }
                return Ok(Expression::NewArray {
                    of: what,
                    lengths,
                    empty,
                });
            }
            let arguments = self.arguments()?;
            if self.is_mark("{") {
                let body = self.written_out_body(line, column, "An anonymous class")?;
                return Ok(Expression::Anonymous {
                    what,
                    arguments,
                    body: Box::new(body),
                    line,
                    column,
                });
            }
            return Ok(Expression::New {
                what,
                arguments,
                outer: None,
            });
        }

        match self.take() {
            Token::Int(value) => Ok(Expression::Int(value)),
            Token::Long(value) => Ok(Expression::Long(value)),
            Token::Float(value) => Ok(Expression::Float(value)),
            Token::Double(value) => Ok(Expression::Double(value)),
            Token::Char(value) => Ok(Expression::Char(value)),
            Token::Str(text) => Ok(Expression::Str(text)),
            Token::True => Ok(Expression::Boolean(true)),
            Token::False => Ok(Expression::Boolean(false)),
            Token::Null => Ok(Expression::Null),
            Token::Identifier(name) => {
                if self.is_mark("(") {
                    let arguments = self.arguments()?;
                    return Ok(Expression::Call {
                        on: None,
                        super_call: false,
                        held: Vec::new(),
                        name,
                        arguments,
                    });
                }
                Ok(Expression::Name(name))
            }
            Token::Keyword("switch") => {
                let Statement::Switch { subject, arms } = self.switch_statement(line, column)?
                else {
                    unreachable!("a switch parses as a switch")
                };
                Ok(Expression::Switch {
                    subject: Box::new(subject),
                    arms,
                })
            }
            Token::Punctuation("->") => Err(unsupported(line, column, "A lambda")),
            Token::Punctuation("::") => Err(unsupported(line, column, "A method reference")),
            other => Err(at(
                "EJ109",
                line,
                column,
                format!(
                    "An expression was expected, and {} was found.",
                    other.describe()
                ),
            )),
        }
    }
}

pub const NESTED_ANNOTATION: &str = "@";

const ONLY_ON_THE_DESKTOP: &[&str] = &[
    "java.lang.IO",
    "java.lang.ScopedValue",
    "java.lang.StableValue",
    "java.lang.Thread.Builder",
    "java.util.concurrent.StructuredTaskScope",
    "javax.crypto.KDF",
    "javax.crypto.KDFParameters",
    "javax.crypto.spec.HKDFParameterSpec",
    "java.security.DEREncodable",
    "java.security.PEMDecoder",
    "java.security.PEMEncoder",
    "java.security.PEMRecord",
    "java.applet.",
    "java.awt.",
    "java.lang.classfile.",
    "java.lang.foreign.",
    "java.lang.instrument.",
    "java.lang.management.",
    "java.net.http.",
    "java.rmi.",
    "javax.imageio.",
    "javax.management.",
    "javax.naming.",
    "javax.print.",
    "javax.script.",
    "javax.sound.",
    "javax.swing.",
    "javax.tools.",
    "jdk.",
];

const ONLY_ON_THE_DESKTOP_METHODS: &[(&str, &str)] = &[
    ("java/lang/Thread", "ofPlatform"),
    ("java/lang/Thread", "ofVirtual"),
    ("java/lang/Thread", "startVirtualThread"),
    ("java/util/concurrent/Executors", "newThreadPerTaskExecutor"),
    (
        "java/util/concurrent/Executors",
        "newVirtualThreadPerTaskExecutor",
    ),
];

fn desktop_only(whole: &str) -> bool {
    ONLY_ON_THE_DESKTOP.iter().any(|one| {
        if one.ends_with('.') {
            whole.starts_with(one)
        } else {
            whole == *one || whole.starts_with(&format!("{one}."))
        }
    })
}

fn only_on_the_desktop(unit: &Unit, name: &str) -> Option<String> {
    if name.contains('.') {
        let first = name.split('.').next().unwrap_or_default();
        let through = unit
            .imports
            .iter()
            .find(|one| one.rsplit('.').next() == Some(first))
            .map(|one| format!("{}{}", one.strip_suffix(first).unwrap_or(one), name));
        let whole = through.unwrap_or_else(|| name.to_string());
        return desktop_only(&whole).then_some(whole);
    }
    for one in &unit.imports {
        if let Some(package) = one.strip_suffix(".*") {
            let whole = format!("{package}.{name}");
            if desktop_only(&whole) {
                return Some(whole);
            }
        } else if one.rsplit('.').next() == Some(name) && desktop_only(one) {
            return Some(one.clone());
        }
    }
    let whole = format!("java.lang.{name}");
    desktop_only(&whole).then_some(whole)
}

fn only_on_the_desktop_diagnostic(whole: &str, line: u32) -> Diagnostic {
    at(
        "EJ260",
        line,
        1,
        format!("`{whole}` belongs to the desktop JDK, and Android does not have it."),
    )
    .with_context(format!("Name: {whole}"))
    .with_suggestion(
        "Android runs apps on its own runtime, which carries most of java.base but not the \
         JDK's desktop, preview and incubator APIs. Write it with what Android has, or leave \
         this part out of the app.",
    )
}

const MODULES: &[(&str, &[&str], &[&str])] = &[
    (
        "java.base",
        &[
            "java.io",
            "java.lang",
            "java.lang.annotation",
            "java.lang.invoke",
            "java.lang.ref",
            "java.lang.reflect",
            "java.lang.runtime",
            "java.math",
            "java.net",
            "java.nio",
            "java.nio.channels",
            "java.nio.channels.spi",
            "java.nio.charset",
            "java.nio.charset.spi",
            "java.nio.file",
            "java.nio.file.attribute",
            "java.nio.file.spi",
            "java.security",
            "java.security.cert",
            "java.security.interfaces",
            "java.security.spec",
            "java.text",
            "java.time",
            "java.time.chrono",
            "java.time.format",
            "java.time.temporal",
            "java.time.zone",
            "java.util",
            "java.util.concurrent",
            "java.util.concurrent.atomic",
            "java.util.concurrent.locks",
            "java.util.function",
            "java.util.jar",
            "java.util.random",
            "java.util.regex",
            "java.util.stream",
            "java.util.zip",
            "javax.crypto",
            "javax.crypto.interfaces",
            "javax.crypto.spec",
            "javax.net",
            "javax.net.ssl",
            "javax.security.auth",
            "javax.security.auth.callback",
            "javax.security.auth.login",
            "javax.security.auth.x500",
            "javax.security.cert",
        ],
        &[],
    ),
    ("java.logging", &["java.util.logging"], &[]),
    ("java.prefs", &["java.util.prefs"], &[]),
    (
        "java.se",
        &[],
        &[
            "java.base",
            "java.logging",
            "java.prefs",
            "java.sql",
            "java.xml",
        ],
    ),
    (
        "java.sql",
        &["java.sql", "javax.sql"],
        &["java.logging", "java.xml"],
    ),
    (
        "java.xml",
        &[
            "javax.xml",
            "javax.xml.datatype",
            "javax.xml.namespace",
            "javax.xml.parsers",
            "javax.xml.transform",
            "javax.xml.transform.dom",
            "javax.xml.transform.sax",
            "javax.xml.transform.stream",
            "javax.xml.validation",
            "javax.xml.xpath",
            "org.w3c.dom",
            "org.w3c.dom.ls",
            "org.xml.sax",
            "org.xml.sax.ext",
            "org.xml.sax.helpers",
        ],
        &[],
    ),
];

fn packages_of_the_module(module: &str) -> Option<Vec<&'static str>> {
    let mut found = Vec::new();
    let mut waiting = vec![module.to_string()];
    let mut seen = Vec::new();
    while let Some(named) = waiting.pop() {
        if seen.contains(&named) {
            continue;
        }
        let (_, packages, requires) = MODULES.iter().find(|(one, _, _)| *one == named)?;
        seen.push(named);
        found.extend_from_slice(packages);
        for one in *requires {
            waiting.push(one.to_string());
        }
    }
    Some(found)
}

pub fn parse(source: &str) -> Result<Vec<Unit>, Diagnostic> {
    parse_as(source, None)
}

pub fn parse_as(source: &str, implicit_name: Option<&str>) -> Result<Vec<Unit>, Diagnostic> {
    let tokens = Lexer::new(source).tokens()?;
    let mut parser = Parser::new(tokens);
    parser.implicit_name = implicit_name.map(str::to_string);
    let declared = parser.file()?;
    if !matches!(parser.here().token, Token::End) {
        return Err(at(
            "EJ116",
            parser.line(),
            parser.column(),
            format!(
                "There is something after the last declaration in this file: {}.",
                parser.here().token.describe()
            ),
        ));
    }
    Ok(declared)
}

#[derive(Clone, Eq, Debug)]
pub enum Type {
    Void,
    Boolean,
    Byte,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,

    Object(String, Vec<Type>),
    Array(Box<Type>),
}

impl PartialEq for Type {
    fn eq(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Object(here, _), Type::Object(there, _)) => here == there,
            (Type::Array(here), Type::Array(there)) => here == there,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Type {
    pub fn descriptor(&self) -> String {
        match self {
            Type::Void => "V".to_string(),
            Type::Boolean => "Z".to_string(),
            Type::Byte => "B".to_string(),
            Type::Short => "S".to_string(),
            Type::Char => "C".to_string(),
            Type::Int => "I".to_string(),
            Type::Long => "J".to_string(),
            Type::Float => "F".to_string(),
            Type::Double => "D".to_string(),
            Type::Object(name, _) => format!("L{name};"),
            Type::Array(of) => format!("[{}", of.descriptor()),
        }
    }

    pub fn readable(&self) -> String {
        match self {
            Type::Object(name, held) if held.is_empty() => name.replace('/', "."),
            Type::Object(name, held) => format!(
                "{}<{}>",
                name.replace('/', "."),
                held.iter()
                    .map(|one| one.readable())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Array(of) => format!("{}[]", of.readable()),

            Type::Void => "void".to_string(),
            Type::Boolean => "boolean".to_string(),
            Type::Byte => "byte".to_string(),
            Type::Short => "short".to_string(),
            Type::Char => "char".to_string(),
            Type::Int => "int".to_string(),
            Type::Long => "long".to_string(),
            Type::Float => "float".to_string(),
            Type::Double => "double".to_string(),
        }
    }

    pub fn width(&self) -> u16 {
        match self {
            Type::Long | Type::Double => 2,
            Type::Void => 0,
            _ => 1,
        }
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, Type::Object(..) | Type::Array(_))
    }

    pub fn is_int_like(&self) -> bool {
        matches!(
            self,
            Type::Boolean | Type::Byte | Type::Short | Type::Char | Type::Int
        )
    }

    pub fn is_numeric(&self) -> bool {
        self.is_int_like() && !matches!(self, Type::Boolean)
            || matches!(self, Type::Long | Type::Float | Type::Double)
    }

    pub fn may_be_given_to(&self, wanted: &Type) -> bool {
        if self == wanted {
            return true;
        }
        if self.is_reference() && wanted.is_reference() {
            return true;
        }
        matches!(
            (self, wanted),
            (
                Type::Byte,
                Type::Short | Type::Int | Type::Long | Type::Float | Type::Double
            ) | (
                Type::Short | Type::Char,
                Type::Int | Type::Long | Type::Float | Type::Double
            ) | (Type::Int, Type::Long | Type::Float | Type::Double)
                | (Type::Long, Type::Float | Type::Double)
                | (Type::Float, Type::Double)
        )
    }

    pub fn promoted_with(&self, other: &Type) -> Option<Type> {
        for wide in [Type::Double, Type::Float, Type::Long] {
            if *self == wide || *other == wide {
                return Some(wide);
            }
        }
        (self.is_numeric() && other.is_numeric()).then_some(Type::Int)
    }
}

#[derive(Clone, Debug)]
pub struct Signature {
    pub owner: String,
    pub name: String,
    pub parameters: Vec<Type>,
    pub returns: Type,
    pub static_: bool,

    pub interface: bool,

    pub variadic: bool,

    pub abstract_: bool,

    pub returns_variable: Option<usize>,

    pub parameter_variables: Vec<Option<usize>>,

    pub taken_stands_for: Vec<Standing>,
    pub returns_stands_for: Standing,

    pub own_variables: Vec<String>,

    pub witnessed: Vec<(String, Type)>,

    pub throws: Vec<String>,
}

impl Signature {
    pub fn descriptor(&self) -> String {
        let mut out = String::from("(");
        for parameter in &self.parameters {
            out.push_str(&parameter.descriptor());
        }
        out.push(')');
        out.push_str(&self.returns.descriptor());
        out
    }
}

#[derive(Clone, Debug, Default)]
pub struct Classpath {
    known: std::collections::BTreeMap<String, KnownClass>,
}

#[derive(Clone, Debug, Default)]
pub struct KnownClass {
    pub name: String,
    pub superclass: Option<String>,

    pub interfaces: Vec<String>,
    pub methods: Vec<Signature>,
    pub fields: Vec<(String, Type, bool)>,

    pub constants: Vec<(String, Expression)>,
    pub field_standings: Vec<(String, Standing)>,
    pub interface: bool,

    pub abstract_: bool,

    pub outer: Option<String>,

    pub kept_at_runtime: bool,

    pub permits: Vec<String>,

    pub above: Vec<(String, Vec<(Type, Standing)>)>,
}

impl Classpath {
    pub fn new() -> Classpath {
        Classpath::the_library().clone()
    }

    pub fn nothing() -> Classpath {
        Classpath::default()
    }

    pub fn everything() -> &'static Classpath {
        Classpath::the_library()
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.known.keys().map(|held| held.as_str())
    }

    fn the_library() -> &'static Classpath {
        static HELD: std::sync::OnceLock<Classpath> = std::sync::OnceLock::new();
        HELD.get_or_init(|| {
            let mut found = Classpath::default();
            let mut units = Vec::new();
            for (_, text) in THE_LIBRARY {
                if let Ok(held) = parse(text) {
                    units.extend(held);
                }
            }

            for (_, text) in crate::compilers::android::THE_PLATFORM {
                if let Ok(held) = parse(text) {
                    units.extend(held);
                }
            }

            let mut beside = Vec::new();
            for (_, text) in crate::compilers::android::THE_LIBRARY_BESIDE {
                if let Ok(held) = parse(text) {
                    beside.extend(held);
                }
            }

            for unit in &units {
                found.shell(unit);
            }
            let (filling, fresh): (Vec<&Unit>, Vec<&Unit>) = beside
                .iter()
                .partition(|unit| found.known.contains_key(&unit.internal_name()));
            for unit in &fresh {
                found.shell(unit);
            }
            for unit in &units {
                found.declare(unit);
            }
            for unit in &fresh {
                found.declare(unit);
            }
            for unit in &filling {
                found.fill_in(unit);
            }

            if let Some(held) = found.known.get_mut("java/lang/Object") {
                held.superclass = None;
            }
            found
        })
    }

    pub fn shell(&mut self, unit: &Unit) {
        let internal = unit.internal_name();
        self.known.insert(
            internal.clone(),
            KnownClass {
                name: internal,
                superclass: Some(match &unit.extends {
                    Some(named) => named.replace('.', "/"),
                    None => "java/lang/Object".to_string(),
                }),
                interface: unit.shape == Shape::Interface,
                abstract_: unit.modifiers.abstract_,
                outer: unit.outer.clone(),
                kept_at_runtime: kept_at_runtime(unit),
                ..KnownClass::default()
            },
        );
    }

    pub fn place(&mut self, unit: &Unit) {
        let superclass = unit
            .extends
            .as_ref()
            .and_then(|named| resolve_named(self, unit, named));
        let interfaces: Vec<String> = unit
            .implements
            .iter()
            .filter_map(|named| resolve_named(self, unit, named))
            .collect();
        if let Some(known) = self.known.get_mut(&unit.internal_name()) {
            if superclass.is_some() {
                known.superclass = superclass;
            }
            if !interfaces.is_empty() {
                known.interfaces = interfaces;
            }
        }
    }

    pub fn fill_in(&mut self, unit: &Unit) {
        let internal = unit.internal_name();
        let Some(before) = self.known.get(&internal).cloned() else {
            return self.declare(unit);
        };
        self.declare(unit);
        if let Some(after) = self.known.get_mut(&internal) {
            after.superclass = before.superclass;
            let mut interfaces = before.interfaces;
            for one in std::mem::take(&mut after.interfaces) {
                if !interfaces.contains(&one) {
                    interfaces.push(one);
                }
            }
            after.interfaces = interfaces;
            after.interface = before.interface;
            after.abstract_ = before.abstract_;
            after.outer = before.outer;
            after.kept_at_runtime = before.kept_at_runtime;
            after.permits = before.permits;
            after.above = before.above;
        }
    }

    pub fn declare(&mut self, unit: &Unit) {
        let internal = unit.internal_name();
        let mut known = match self.known.get(&internal) {
            Some(found) => found.clone(),
            None => KnownClass {
                name: internal.clone(),
                superclass: Some("java/lang/Object".to_string()),
                interface: unit.shape == Shape::Interface,
                abstract_: unit.modifiers.abstract_,
                outer: unit.outer.clone(),
                kept_at_runtime: kept_at_runtime(unit),
                ..KnownClass::default()
            },
        };

        if let Some(named) = &unit.extends {
            if let Some(found) = resolve_named(self, unit, named) {
                known.superclass = Some(found);
            }
        }
        known.permits = unit
            .permits
            .iter()
            .filter_map(|named| resolve_named(self, unit, named))
            .collect();
        known.interfaces = unit
            .implements
            .iter()
            .filter_map(|named| resolve_named(self, unit, named))
            .collect();

        if unit.annotation {
            known
                .interfaces
                .push("java/lang/annotation/Annotation".to_string());
        }

        known.above = unit
            .above
            .iter()
            .filter_map(|one| {
                let Written::Named(named, arguments) = one else {
                    return None;
                };
                let found = resolve_named(self, unit, named)?;
                let held = arguments
                    .iter()
                    .map(|what| {
                        let erased = resolve_written(self, unit, what)
                            .unwrap_or(Type::Object("java/lang/Object".to_string(), Vec::new()));
                        (erased, standing_of(what))
                    })
                    .collect();
                Some((found, held))
            })
            .collect();

        let shallow = |written: &Written| -> Option<Type> { resolve_written(self, unit, written) };

        for field in &unit.fields {
            if let Some(what) = shallow(&field.what) {
                known
                    .fields
                    .push((field.name.clone(), what, field.modifiers.static_));
                let standing = standing_of(&field.what);
                if !standing.says_nothing() {
                    known.field_standings.push((field.name.clone(), standing));
                }
            }
        }
        for method in &unit.methods {
            let mut parameters = Vec::new();
            let mut whole = true;
            for (what, _) in &method.parameters {
                match shallow(what) {
                    Some(found) => parameters.push(found),
                    None => whole = false,
                }
            }
            let Some(returns) = shallow(&method.returns) else {
                continue;
            };
            if !whole {
                continue;
            }
            known.methods.push(Signature {
                owner: internal.clone(),
                name: method.name.clone(),
                parameters,
                returns,
                static_: method.modifiers.static_,
                interface: unit.shape == Shape::Interface,
                variadic: method.variadic,
                abstract_: method.body.is_none() && !method.modifiers.native_,

                returns_variable: which_variable(&method.returns),
                parameter_variables: method
                    .parameters
                    .iter()
                    .map(|(what, _)| which_variable(what))
                    .collect(),

                taken_stands_for: method
                    .parameters
                    .iter()
                    .map(|(what, _)| standing_of(what))
                    .collect(),
                returns_stands_for: standing_of(&method.returns),
                own_variables: method.own_variables.clone(),
                witnessed: Vec::new(),

                throws: method
                    .throws
                    .iter()
                    .filter_map(|one| resolve_named(self, unit, one))
                    .collect(),
            });
        }
        for field in &unit.fields {
            if !field.modifiers.static_ || !field.modifiers.final_ {
                continue;
            }
            let Some(value) = &field.value else {
                continue;
            };
            if let Some(found) = constant_within(self, unit, value, 0) {
                if let Some(held) = held_as(&field.what, found) {
                    known.constants.push((field.name.clone(), held));
                }
            }
        }
        self.known.insert(internal, known);
    }

    pub fn learn_jar(&mut self, bytes: &[u8]) -> Result<usize, Diagnostic> {
        let mut sink = crate::diag::Sink::new();
        let Some(archive) = crate::archive::read(bytes, &mut sink) else {
            return Err(at("EJ255", 1, 1, "That is not an archive this can read.")
                .with_suggestion("A dependency is a jar, which is a zip of class files."));
        };
        let mut learned = 0usize;
        let mut offered = 0usize;
        for entry in archive.entries() {
            if entry.is_directory() || !entry.name.ends_with(".class") {
                continue;
            }
            offered += 1;
            let from = entry.data_offset as usize;
            let to = from.saturating_add(entry.compressed_size as usize);
            let Some(held) = bytes.get(from..to) else {
                continue;
            };
            let held = match entry.compression {
                crate::archive::Compression::Stored => held.to_vec(),
                crate::archive::Compression::Deflate => match crate::inflate::raw(held) {
                    Ok(found) => found,
                    Err(_) => continue,
                },
                crate::archive::Compression::Other(_) => continue,
            };

            let Ok(class) = crate::jvm::read(&held) else {
                continue;
            };
            if self.learn(&class).is_ok() {
                learned += 1;
            }
        }
        if learned == 0 && offered > 0 {
            return Err(at("EJ255", 1, 1, "That archive holds no class files.")
                .with_suggestion("A dependency is a jar, which is a zip of class files."));
        }
        Ok(learned)
    }

    pub fn learn(&mut self, class: &crate::jvm::Class) -> Result<(), Diagnostic> {
        let interface = class.access_flags & 0x0200 != 0;
        let internal = class.name.replace('.', "/");
        if self.known.contains_key(&internal)
            && Classpath::the_library().known.contains_key(&internal)
        {
            let written: Vec<(String, Expression)> = class
                .fields
                .iter()
                .filter_map(|field| {
                    Some((field.name.clone(), constant_of(field.constant.as_ref()?)?))
                })
                .collect();
            if let Some(known) = self.known.get_mut(&internal) {
                for (name, held) in written {
                    if !known.constants.iter().any(|(one, _)| *one == name) {
                        known.constants.push((name, held));
                    }
                }
            }
            return Ok(());
        }
        let mut known = KnownClass {
            name: internal.clone(),
            superclass: class.superclass.as_ref().map(|name| name.replace('.', "/")),
            interfaces: class
                .interfaces
                .iter()
                .map(|name| name.replace('.', "/"))
                .collect(),
            interface,
            abstract_: class.access_flags & 0x0400 != 0,
            kept_at_runtime: class.access_flags & 0x2000 != 0
                && class.annotations.iter().any(|held| {
                    held.descriptor == "Ljava/lang/annotation/Retention;"
                        && held.values.iter().any(|(_, value)| {
                            matches!(
                                value,
                                crate::jvm::Element::Enum { name, .. } if name == "RUNTIME"
                            )
                        })
                }),
            ..KnownClass::default()
        };

        let named = class
            .signature
            .as_deref()
            .map(type_parameters_of)
            .unwrap_or_default();
        for method in &class.methods {
            if method.access_flags & 0x1000 != 0 {
                continue;
            }
            let Some((parameters, returns)) = read_descriptor(&method.descriptor) else {
                continue;
            };
            let same_class = |raw: Type, better: Option<Type>| match (&raw, better) {
                (Type::Object(had, _), Some(Type::Object(named, held))) if *had == named => {
                    Type::Object(named, held)
                }
                _ => raw,
            };
            let (parameters, returns) = match method
                .signature
                .as_deref()
                .and_then(|held| fixed_parts(held, &named))
            {
                Some((fixed, back)) if fixed.len() == parameters.len() => (
                    parameters
                        .into_iter()
                        .zip(fixed)
                        .map(|(raw, better)| same_class(raw, better))
                        .collect(),
                    same_class(returns, back),
                ),
                _ => (parameters, returns),
            };
            let (returns_variable, parameter_variables) = match &method.signature {
                Some(held) if !named.is_empty() => variables_of(held, &named),
                _ => (None, Vec::new()),
            };

            let (taken_stands_for, returns_stands_for) = method
                .signature
                .as_deref()
                .and_then(|held| standings_of(held, &named))
                .unwrap_or_default();

            let own_variables = method
                .signature
                .as_deref()
                .map(type_parameters_of)
                .unwrap_or_default();
            known.methods.push(Signature {
                owner: internal.clone(),
                name: method.name.clone(),
                parameters,
                returns,
                static_: method.access_flags & 0x0008 != 0,
                interface,

                variadic: method.access_flags & 0x0080 != 0,
                abstract_: method.access_flags & 0x0400 != 0,
                returns_variable,
                parameter_variables,
                taken_stands_for,
                returns_stands_for,
                own_variables,
                witnessed: Vec::new(),
                throws: method
                    .throws
                    .iter()
                    .map(|one| one.replace('.', "/"))
                    .collect(),
            });
        }
        let bounds = class
            .signature
            .as_deref()
            .map(bounds_of)
            .unwrap_or_default();
        known.above = class
            .signature
            .as_deref()
            .map(above_of_signature)
            .unwrap_or_default();
        for field in &class.fields {
            let Some(what) = read_type(&field.descriptor, &mut 0) else {
                continue;
            };
            if let Some(signature) = field.signature.as_deref() {
                let variables = class
                    .signature
                    .as_deref()
                    .map(type_parameters_of)
                    .unwrap_or_default();
                if let Some((_, standing, _)) =
                    placed_at(signature.as_bytes(), &mut 0, &variables, &bounds)
                {
                    if !standing.says_nothing() {
                        known.field_standings.push((field.name.clone(), standing));
                    }
                }
            }
            let what = match field.signature.as_deref() {
                Some(signature) if matches!(what, Type::Object(..)) => {
                    match generic_type_of(signature, &bounds) {
                        Some(Type::Object(named, held))
                            if Type::Object(named.clone(), Vec::new()) == what =>
                        {
                            Type::Object(named, held)
                        }
                        _ => what,
                    }
                }
                _ => what,
            };
            let held = field.constant.as_ref().and_then(constant_of);
            if let Some(held) = held {
                known.constants.push((field.name.clone(), held));
            }
            known
                .fields
                .push((field.name.clone(), what, field.access_flags & 0x0008 != 0));
        }
        self.known.insert(internal, known);
        Ok(())
    }

    pub fn how_many(&self) -> usize {
        self.known.len()
    }

    pub fn add(&mut self, class: KnownClass) {
        self.known.insert(class.name.clone(), class);
    }

    pub fn get(&self, name: &str) -> Option<&KnownClass> {
        self.known.get(name)
    }

    pub fn seen_from(&self, from: &str, held: &[Type], owner: &str) -> Vec<Type> {
        self.seen_from_within(from, held, owner, 0)
            .unwrap_or_else(|| held.to_vec())
    }

    fn seen_from_within(
        &self,
        from: &str,
        held: &[Type],
        owner: &str,
        depth: usize,
    ) -> Option<Vec<Type>> {
        if from == owner {
            return Some(held.to_vec());
        }
        if depth > 16 {
            return None;
        }
        let known = self.known.get(from)?;
        for (above, arguments) in &known.above {
            let next: Vec<Type> = arguments
                .iter()
                .map(|(erased, shape)| shape.filled(erased, held, &[]))
                .collect();
            if let Some(found) = self.seen_from_within(above, &next, owner, depth + 1) {
                return Some(found);
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.known.len()
    }

    pub fn is_empty(&self) -> bool {
        self.known.is_empty()
    }

    pub fn find_methods(&self, owner: &str, name: &str, arity: usize) -> Vec<Signature> {
        let mut found: Vec<Signature> = Vec::new();
        for one in self.ancestors(owner) {
            let Some(known) = self.known.get(&one) else {
                continue;
            };
            for held in &known.methods {
                if held.name != name || held.parameters.len() != arity {
                    continue;
                }

                if found
                    .iter()
                    .any(|before| before.parameters == held.parameters)
                {
                    continue;
                }
                found.push(held.clone());
            }
        }
        found
    }

    pub fn find_method(&self, owner: &str, name: &str, arity: usize) -> Option<&Signature> {
        let mut at = Some(owner.to_string());
        let mut seen = 0;
        while let Some(current) = at {
            let known = self.known.get(&current)?;
            if let Some(found) = known
                .methods
                .iter()
                .find(|one| one.name == name && one.parameters.len() == arity)
            {
                return Some(found);
            }
            seen += 1;
            if seen > 64 {
                return None;
            }
            at = known.superclass.clone();
        }
        None
    }

    pub fn ancestors(&self, owner: &str) -> Vec<String> {
        let mut out = vec![owner.to_string()];
        let mut at = 0usize;
        while at < out.len() {
            if out.len() > 256 {
                break;
            }
            let Some(known) = self.known.get(&out[at]) else {
                at += 1;
                continue;
            };
            let mut above = Vec::new();
            if let Some(one) = &known.superclass {
                above.push(one.clone());
            }
            above.extend(known.interfaces.iter().cloned());
            at += 1;
            for one in above {
                if !out.contains(&one) {
                    out.push(one);
                }
            }
        }
        if !out.iter().any(|held| held == "java/lang/Object") {
            out.push("java/lang/Object".to_string());
        }
        out
    }

    pub fn find_field(
        &self,
        owner: &str,
        name: &str,
    ) -> Option<(&KnownClass, &(String, Type, bool))> {
        let mut waiting = vec![owner.to_string()];
        let mut seen: Vec<String> = Vec::new();
        while let Some(current) = waiting.pop() {
            if seen.contains(&current) || seen.len() > 64 {
                continue;
            }
            seen.push(current.clone());
            let Some(known) = self.known.get(&current) else {
                continue;
            };
            if let Some(found) = known.fields.iter().find(|(held, _, _)| held == name) {
                return Some((known, found));
            }
            waiting.extend(known.superclass.clone());
            waiting.extend(known.interfaces.iter().rev().cloned());
        }
        None
    }

    pub fn constant(&self, owner: &str, name: &str) -> Option<&Expression> {
        let (holder, _) = self.find_field(owner, name)?;
        holder
            .constants
            .iter()
            .find(|(held, _)| held == name)
            .map(|(_, value)| value)
    }
}

fn constant_of(element: &crate::jvm::Element) -> Option<Expression> {
    Some(match element {
        crate::jvm::Element::Int(value) => Expression::Int(i64::from(*value)),
        crate::jvm::Element::Short(value) => Expression::Int(i64::from(*value)),
        crate::jvm::Element::Byte(value) => Expression::Int(i64::from(*value)),
        crate::jvm::Element::Char(value) => Expression::Char(*value),
        crate::jvm::Element::Boolean(value) => Expression::Boolean(*value),
        crate::jvm::Element::Long(value) => Expression::Long(*value),
        crate::jvm::Element::Float(bits) => Expression::Float(f64::from(f32::from_bits(*bits))),
        crate::jvm::Element::Double(bits) => Expression::Double(f64::from_bits(*bits)),
        crate::jvm::Element::Text(value) => Expression::Str(value.clone()),
        _ => return None,
    })
}

pub fn read_type(descriptor: &str, at: &mut usize) -> Option<Type> {
    let bytes = descriptor.as_bytes();
    let byte = *bytes.get(*at)?;
    *at += 1;
    Some(match byte {
        b'V' => Type::Void,
        b'Z' => Type::Boolean,
        b'B' => Type::Byte,
        b'S' => Type::Short,
        b'C' => Type::Char,
        b'I' => Type::Int,
        b'J' => Type::Long,
        b'F' => Type::Float,
        b'D' => Type::Double,
        b'[' => Type::Array(Box::new(read_type(descriptor, at)?)),
        b'L' => {
            let end = descriptor[*at..].find(';')? + *at;
            let name = descriptor[*at..end].to_string();
            *at = end + 1;
            Type::Object(name, Vec::new())
        }
        _ => return None,
    })
}

pub fn read_descriptor(descriptor: &str) -> Option<(Vec<Type>, Type)> {
    let bytes = descriptor.as_bytes();
    if bytes.first() != Some(&b'(') {
        return None;
    }
    let mut at = 1usize;
    let mut parameters = Vec::new();
    while *bytes.get(at)? != b')' {
        parameters.push(read_type(descriptor, &mut at)?);
    }
    at += 1;
    let returns = read_type(descriptor, &mut at)?;
    Some((parameters, returns))
}

#[derive(Clone, Debug, Default)]
pub struct Pool {
    entries: Vec<PoolEntry>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum PoolEntry {
    Utf8(String),
    Integer(i32),
    Long(i64),
    Float(u32),
    Double(u64),
    Class(u16),
    Str(u16),
    NameAndType(u16, u16),
    Field(u16, u16),
    Method(u16, u16),
    InterfaceMethod(u16, u16),

    Unusable,
}

impl Pool {
    pub fn new() -> Pool {
        Pool::default()
    }

    fn put(&mut self, entry: PoolEntry) -> u16 {
        if let Some(found) = self.entries.iter().position(|held| *held == entry) {
            return found as u16 + 1;
        }
        let wide = matches!(entry, PoolEntry::Long(_) | PoolEntry::Double(_));
        self.entries.push(entry);
        let index = self.entries.len() as u16;
        if wide {
            self.entries.push(PoolEntry::Unusable);
        }
        index
    }

    pub fn utf8(&mut self, text: &str) -> u16 {
        self.put(PoolEntry::Utf8(text.to_string()))
    }

    pub fn class(&mut self, internal: &str) -> u16 {
        let name = self.utf8(internal);
        self.put(PoolEntry::Class(name))
    }

    pub fn string(&mut self, text: &str) -> u16 {
        let value = self.utf8(text);
        self.put(PoolEntry::Str(value))
    }

    pub fn integer(&mut self, value: i32) -> u16 {
        self.put(PoolEntry::Integer(value))
    }

    pub fn long(&mut self, value: i64) -> u16 {
        self.put(PoolEntry::Long(value))
    }

    pub fn float(&mut self, value: f32) -> u16 {
        self.put(PoolEntry::Float(value.to_bits()))
    }

    pub fn double(&mut self, value: f64) -> u16 {
        self.put(PoolEntry::Double(value.to_bits()))
    }

    fn name_and_type(&mut self, name: &str, descriptor: &str) -> u16 {
        let name = self.utf8(name);
        let descriptor = self.utf8(descriptor);
        self.put(PoolEntry::NameAndType(name, descriptor))
    }

    pub fn field(&mut self, owner: &str, name: &str, descriptor: &str) -> u16 {
        let owner = self.class(owner);
        let what = self.name_and_type(name, descriptor);
        self.put(PoolEntry::Field(owner, what))
    }

    pub fn method(&mut self, owner: &str, name: &str, descriptor: &str, interface: bool) -> u16 {
        let owner_index = self.class(owner);
        let what = self.name_and_type(name, descriptor);
        if interface {
            return self.put(PoolEntry::InterfaceMethod(owner_index, what));
        }
        self.put(PoolEntry::Method(owner_index, what))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&(self.entries.len() as u16 + 1).to_be_bytes());
        for entry in &self.entries {
            match entry {
                PoolEntry::Unusable => {}
                PoolEntry::Utf8(text) => {
                    out.push(1);
                    let bytes = crate::binary::modified_utf8::encode(text);
                    out.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
                    out.extend_from_slice(&bytes);
                }
                PoolEntry::Integer(value) => {
                    out.push(3);
                    out.extend_from_slice(&value.to_be_bytes());
                }
                PoolEntry::Float(bits) => {
                    out.push(4);
                    out.extend_from_slice(&bits.to_be_bytes());
                }
                PoolEntry::Long(value) => {
                    out.push(5);
                    out.extend_from_slice(&value.to_be_bytes());
                }
                PoolEntry::Double(bits) => {
                    out.push(6);
                    out.extend_from_slice(&bits.to_be_bytes());
                }
                PoolEntry::Class(name) => {
                    out.push(7);
                    out.extend_from_slice(&name.to_be_bytes());
                }
                PoolEntry::Str(value) => {
                    out.push(8);
                    out.extend_from_slice(&value.to_be_bytes());
                }
                PoolEntry::Field(owner, what) => {
                    out.push(9);
                    out.extend_from_slice(&owner.to_be_bytes());
                    out.extend_from_slice(&what.to_be_bytes());
                }
                PoolEntry::Method(owner, what) => {
                    out.push(10);
                    out.extend_from_slice(&owner.to_be_bytes());
                    out.extend_from_slice(&what.to_be_bytes());
                }
                PoolEntry::InterfaceMethod(owner, what) => {
                    out.push(11);
                    out.extend_from_slice(&owner.to_be_bytes());
                    out.extend_from_slice(&what.to_be_bytes());
                }
                PoolEntry::NameAndType(name, descriptor) => {
                    out.push(12);
                    out.extend_from_slice(&name.to_be_bytes());
                    out.extend_from_slice(&descriptor.to_be_bytes());
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Local {
    name: String,
    slot: u16,
    what: Type,
}

#[derive(Clone)]
struct Locally {
    written: String,
    internal: String,
    outer: Option<String>,
    through_the_one_around: bool,
    captured: Vec<Local>,
}

struct Pending {
    at: usize,
    from: usize,

    slots: Vec<Verified>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Verified {
    Top,
    Integer,
    Float,
    Double,
    Long,
    Null,

    UninitializedThis,

    Uninitialized(u16),
    Object(String),
}

impl Verified {
    fn of(what: &Type) -> Verified {
        match what {
            Type::Boolean | Type::Byte | Type::Short | Type::Char | Type::Int => Verified::Integer,
            Type::Long => Verified::Long,
            Type::Float => Verified::Float,
            Type::Double => Verified::Double,
            Type::Void => Verified::Top,
            Type::Object(name, _) => Verified::Object(name.clone()),
            Type::Array(_) => Verified::Object(what.descriptor()),
        }
    }

    fn is_wide(&self) -> bool {
        matches!(self, Verified::Long | Verified::Double)
    }

    fn write(&self, out: &mut Vec<u8>, pool: &mut Pool) {
        match self {
            Verified::Top => out.push(0),
            Verified::Integer => out.push(1),
            Verified::Float => out.push(2),
            Verified::Double => out.push(3),
            Verified::Long => out.push(4),
            Verified::Null => out.push(5),
            Verified::UninitializedThis => out.push(6),
            Verified::Uninitialized(at) => {
                out.push(8);
                out.extend_from_slice(&at.to_be_bytes());
            }
            Verified::Object(name) => {
                out.push(7);
                let index = pool.class(name);
                out.extend_from_slice(&index.to_be_bytes());
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Cleanup {
    Block(Vec<Positioned<Statement>>),
    Unlock(u16, Type),
}

struct Level {
    label: Option<String>,

    loops: bool,
    breaks: Vec<Pending>,
    continues: Vec<Pending>,

    finallys: usize,
}

struct Yielding {
    pending: Vec<Pending>,

    produced: Option<Type>,

    finallys: usize,
}

struct Handler {
    start: usize,
    end: usize,
    target: usize,

    class: Option<String>,
}

#[derive(Clone, Debug)]
struct Frame {
    at: usize,
    locals: Vec<Verified>,
    stack: Vec<Verified>,
}

struct Emitter<'a> {
    pool: &'a mut Pool,
    classpath: &'a Classpath,
    unit: &'a Unit,
    this_class: String,
    code: Vec<u8>,
    locals: Vec<Local>,
    scopes: Vec<usize>,
    next_slot: u16,
    max_slot: u16,
    depth: i32,
    max_depth: i32,

    reachable: bool,

    lines: Vec<(u16, u16)>,

    caught: Vec<Vec<String>>,

    said_to_throw: Vec<String>,

    levels: Vec<Level>,

    finallys: Vec<Cleanup>,

    handlers: Vec<Handler>,

    inlined: Vec<(usize, usize, usize)>,

    pending_label: Option<String>,

    yields: Vec<Yielding>,

    made: &'a mut Vec<Unit>,

    wants_assertions: bool,

    expecting: Option<Type>,

    rethrown: Vec<(u16, Vec<String>)>,

    markers: Vec<String>,

    taken_apart: usize,

    locally: Vec<Locally>,

    writing_a_bridge: bool,
    static_: bool,

    within: Option<(String, String)>,

    returns: Type,

    slots: Vec<Verified>,

    frames: Vec<Frame>,

    stack: Vec<Verified>,

    early: bool,

    too_early: bool,
}

impl<'a> Emitter<'a> {
    fn new(
        pool: &'a mut Pool,
        classpath: &'a Classpath,
        unit: &'a Unit,
        this_class: String,
        static_: bool,
        made: &'a mut Vec<Unit>,
    ) -> Emitter<'a> {
        Emitter {
            made,
            pool,
            classpath,
            unit,
            this_class,
            code: Vec::new(),
            locals: Vec::new(),
            scopes: Vec::new(),
            next_slot: 0,
            max_slot: 0,
            depth: 0,
            max_depth: 0,
            levels: Vec::new(),
            finallys: Vec::new(),
            handlers: Vec::new(),
            inlined: Vec::new(),
            pending_label: None,
            yields: Vec::new(),
            wants_assertions: false,
            expecting: None,
            rethrown: Vec::new(),
            markers: Vec::new(),
            taken_apart: 0,
            locally: Vec::new(),
            writing_a_bridge: false,
            static_,
            within: None,
            returns: Type::Void,
            slots: Vec::new(),
            frames: Vec::new(),
            stack: Vec::new(),
            early: false,
            too_early: false,
            reachable: true,
            lines: Vec::new(),
            caught: Vec::new(),
            said_to_throw: Vec::new(),
        }
    }

    fn pushes(&mut self, what: &Type) {
        if *what == Type::Void {
            return;
        }
        let verified = Verified::of(what);
        if verified.is_wide() {
            self.stack.push(verified);
            self.stack.push(Verified::Top);
        } else {
            self.stack.push(verified);
        }
        self.deepest();
    }

    fn pushes_raw(&mut self, what: Verified) {
        let wide = what.is_wide();
        self.stack.push(what);
        if wide {
            self.stack.push(Verified::Top);
        }
        self.deepest();
    }

    fn duplicates(&mut self) {
        self.op(0x59);
        if let Some(top) = self.stack.last().cloned() {
            self.stack.push(top);
        }
        self.deepest();
    }

    fn duplicates_two(&mut self) {
        self.op(0x5c);
        let held = self.stack.len();
        if held >= 2 {
            let two = self.stack[held - 2..].to_vec();
            self.stack.extend(two);
        }
        self.deepest();
    }

    fn pops(&mut self, slots: i32) {
        for _ in 0..slots.max(0) {
            self.stack.pop();
        }
    }

    fn stack_is(&mut self, what: Vec<Verified>) {
        self.stack = what;
        self.deepest();
    }

    fn deepest(&mut self) {
        self.depth = self.stack.len() as i32;
        if self.depth > self.max_depth {
            self.max_depth = self.depth;
        }
    }

    fn give_them_null(&mut self, outside: &[Verified], inside: &[Verified]) {
        for (at, held) in inside.iter().enumerate() {
            if outside.get(at) == Some(held) {
                continue;
            }

            let (push, store) = match held {
                Verified::Object(_) | Verified::Null => (0x01u8, 0x3au8),
                Verified::Integer => (0x03, 0x36),
                Verified::Float => (0x0b, 0x38),
                Verified::Long => (0x09, 0x37),
                Verified::Double => (0x0e, 0x39),
                _ => continue,
            };
            self.op(push);
            self.pushes_raw(held.clone());
            if at <= 3 {
                let base = match store {
                    0x3a => 0x4b,
                    0x36 => 0x3b,
                    0x38 => 0x43,
                    0x37 => 0x3f,
                    _ => 0x47,
                };
                self.op(base + at as u8);
            } else if at <= 255 {
                self.op1(store, at as u8);
            } else {
                self.op(0xc4);
                self.op2(store, at as u16);
            }
            self.pops(i32::from(held.is_wide()) + 1);
        }
    }

    fn too_early_at(&mut self, line: u32) -> Result<(), Diagnostic> {
        if !self.too_early {
            return Ok(());
        }
        self.too_early = false;
        Err(at(
            "EJ258",
            line,
            1,
            "`this` is used before the superclass constructor has run.",
        )
        .with_suggestion(
            "Before `super(...)` or `this(...)`, only the arguments, locals and fields of this \
             class that have no initializer may be touched.",
        ))
    }

    fn slot_holds(&mut self, slot: u16, what: &Type) {
        let verified = Verified::of(what);
        let wide = verified.is_wide();
        let at = usize::from(slot);
        while self.slots.len() <= at + usize::from(wide) {
            self.slots.push(Verified::Top);
        }
        self.slots[at] = verified;
        if wide {
            self.slots[at + 1] = Verified::Top;
        }
    }

    fn a_branch_lands_here(&mut self) {
        self.reachable = true;
        let at = self.code.len();
        let mut locals = Self::as_a_frame_says_it(&self.slots);
        while matches!(locals.last(), Some(Verified::Top)) {
            locals.pop();
        }
        let frame = Frame {
            at,
            locals,
            stack: Self::as_a_frame_says_it(&self.stack),
        };
        match self.frames.iter().position(|held| held.at == at) {
            Some(index) => self.frames[index] = frame,
            None => self.frames.push(frame),
        }
    }

    fn as_a_frame_says_it(held: &[Verified]) -> Vec<Verified> {
        let mut said = Vec::with_capacity(held.len());
        let mut at = 0;
        while at < held.len() {
            let one = held[at].clone();
            let wide = one.is_wide();
            said.push(one);
            at += if wide && held.get(at + 1) == Some(&Verified::Top) {
                2
            } else {
                1
            };
        }
        said
    }

    fn op(&mut self, opcode: u8) {
        self.code.push(opcode);
    }

    fn op1(&mut self, opcode: u8, operand: u8) {
        self.code.push(opcode);
        self.code.push(operand);
    }

    fn op2(&mut self, opcode: u8, operand: u16) {
        self.code.push(opcode);
        self.code.extend_from_slice(&operand.to_be_bytes());
    }

    fn jump(&mut self, opcode: u8) -> Pending {
        let from = self.code.len();
        self.code.push(opcode);
        self.code.extend_from_slice(&[0, 0]);
        Pending {
            at: self.code.len() - 2,
            from,
            slots: self.slots.clone(),
        }
    }

    fn land(&mut self, pending: Pending) {
        if self.reachable {
            self.only_what_every_road_agrees_on(&pending.slots);
        } else {
            self.slots = pending.slots.clone();
            self.reachable = true;
        }
        let to = self.code.len();
        self.land_at(pending, to);
    }

    fn only_what_every_road_agrees_on(&mut self, other: &[Verified]) {
        let wide = self.slots.len().max(other.len());
        self.slots.resize(wide, Verified::Top);
        for at in 0..wide {
            if other.get(at) != Some(&self.slots[at]) {
                self.slots[at] = Verified::Top;
            }
        }
    }

    fn land_at(&mut self, pending: Pending, to: usize) {
        let offset = (to as i64 - pending.from as i64) as i16;
        self.code[pending.at..pending.at + 2].copy_from_slice(&offset.to_be_bytes());
    }

    fn jump_back(&mut self, opcode: u8, to: usize) {
        let from = self.code.len();
        let offset = (to as i64 - from as i64) as i16;
        self.code.push(opcode);
        self.code.extend_from_slice(&offset.to_be_bytes());
    }

    fn open(&mut self) {
        self.scopes.push(self.locals.len());
    }

    fn close(&mut self) {
        if let Some(mark) = self.scopes.pop() {
            while self.locals.len() > mark {
                if let Some(gone) = self.locals.pop() {
                    self.next_slot -= gone.what.width();
                }
            }
        }

        self.slots.truncate(usize::from(self.next_slot));
    }

    fn declare_with_nothing_in_it(&mut self, name: &str, what: Type) -> u16 {
        let slot = self.declare(name, what);
        let at = usize::from(slot);
        if at < self.slots.len() {
            self.slots[at] = Verified::Top;
        }
        slot
    }

    fn declare(&mut self, name: &str, what: Type) -> u16 {
        let slot = self.next_slot;
        self.slot_holds(slot, &what);
        self.next_slot += what.width();
        if self.next_slot > self.max_slot {
            self.max_slot = self.next_slot;
        }
        self.locals.push(Local {
            name: name.to_string(),
            slot,
            what,
        });
        slot
    }

    fn local(&self, name: &str) -> Option<Local> {
        self.locals
            .iter()
            .rev()
            .find(|held| held.name == name)
            .cloned()
    }

    fn resolve(&self, written: &Written, line: u32) -> Result<Type, Diagnostic> {
        Ok(match written {
            Written::Inferred => {
                return Err(at(
                    "EJ234",
                    line,
                    1,
                    "A `var` takes its type from the value given to it, and there is none here.",
                )
                .with_suggestion("Write the type, or give it a value."))
            }
            Written::Void => Type::Void,
            Written::Boolean => Type::Boolean,
            Written::Byte => Type::Byte,
            Written::Short => Type::Short,
            Written::Char => Type::Char,
            Written::Int => Type::Int,
            Written::Long => Type::Long,
            Written::Float => Type::Float,
            Written::Double => Type::Double,
            Written::Array(of) => Type::Array(Box::new(self.resolve(of, line)?)),

            Written::Variable(_, bound) | Written::OwnVariable(_, bound) => {
                self.resolve(bound, line)?
            }
            Written::Named(name, held) => {
                let mut arguments = Vec::new();
                for one in held {
                    match self.resolve(one, line) {
                        Ok(found) => arguments.push(found),
                        Err(_) => {
                            arguments.clear();
                            break;
                        }
                    }
                }
                Type::Object(self.resolve_class(name, line)?, arguments)
            }
        })
    }

    fn resolve_class(&self, name: &str, line: u32) -> Result<String, Diagnostic> {
        if let Some(held) = self.locally.iter().rev().find(|held| held.written == name) {
            return Ok(held.internal.clone());
        }
        resolve_named(self.classpath, self.unit, name).ok_or_else(|| {
            if let Some(whole) = only_on_the_desktop(self.unit, name) {
                return only_on_the_desktop_diagnostic(&whole, line);
            }
            at(
                "EJ200",
                line,
                1,
                format!("`{name}` is not a type this compilation knows."),
            )
            .with_context(format!("Name: {name}"))
            .with_suggestion(
                "Import it, write it out in full, or hand the class file that declares it \
                 over as a dependency. Nothing is guessed: a name that resolves to nothing \
                 here would become a class file the device refuses.",
            )
        })
    }
}

const BUILT_IN_METHODS: &[(&str, &str, &str, bool)] = &[
    ("java/lang/Object", "<init>", "()V", false),
    (
        "java/lang/Object",
        "toString",
        "()Ljava/lang/String;",
        false,
    ),
    ("java/lang/Object", "hashCode", "()I", false),
    ("java/lang/Object", "equals", "(Ljava/lang/Object;)Z", false),
    ("java/lang/Object", "getClass", "()Ljava/lang/Class;", false),
    ("java/lang/Class", "desiredAssertionStatus", "()Z", false),
    ("java/lang/Class", "getName", "()Ljava/lang/String;", false),
    (
        "java/lang/Class",
        "getSimpleName",
        "()Ljava/lang/String;",
        false,
    ),
    ("java/lang/Class", "isRecord", "()Z", false),
    ("java/lang/Class", "isSealed", "()Z", false),
    ("java/lang/Class", "isEnum", "()Z", false),
    ("java/lang/Class", "isAnnotation", "()Z", false),
    ("java/lang/Class", "isInterface", "()Z", false),
    ("java/lang/Class", "isArray", "()Z", false),
    ("java/lang/Class", "isPrimitive", "()Z", false),
    (
        "java/lang/Class",
        "isInstance",
        "(Ljava/lang/Object;)Z",
        false,
    ),
    (
        "java/lang/Class",
        "isAssignableFrom",
        "(Ljava/lang/Class;)Z",
        false,
    ),
    (
        "java/lang/Class",
        "getSuperclass",
        "()Ljava/lang/Class;",
        false,
    ),
    (
        "java/lang/Class",
        "getAnnotation",
        "(Ljava/lang/Class;)Ljava/lang/annotation/Annotation;",
        false,
    ),
    (
        "java/lang/Class",
        "isAnnotationPresent",
        "(Ljava/lang/Class;)Z",
        false,
    ),
    (
        "java/lang/Class",
        "getPermittedSubclasses",
        "()[Ljava/lang/Class;",
        false,
    ),
    ("java/lang/Enum", "<init>", "(Ljava/lang/String;I)V", false),
    ("java/lang/Enum", "name", "()Ljava/lang/String;", false),
    ("java/lang/Enum", "ordinal", "()I", false),
    ("java/lang/Enum", "toString", "()Ljava/lang/String;", false),
    ("java/lang/Enum", "equals", "(Ljava/lang/Object;)Z", false),
    ("java/lang/Enum", "hashCode", "()I", false),
    (
        "java/lang/Throwable",
        "getMessage",
        "()Ljava/lang/String;",
        false,
    ),
    (
        "java/lang/Throwable",
        "getLocalizedMessage",
        "()Ljava/lang/String;",
        false,
    ),
    (
        "java/lang/Throwable",
        "getCause",
        "()Ljava/lang/Throwable;",
        false,
    ),
    ("java/lang/Throwable", "printStackTrace", "()V", false),
    (
        "java/lang/Throwable",
        "toString",
        "()Ljava/lang/String;",
        false,
    ),
    ("java/lang/String", "length", "()I", false),
    ("java/lang/String", "isEmpty", "()Z", false),
    ("java/lang/String", "isBlank", "()Z", false),
    ("java/lang/String", "charAt", "(I)C", false),
    (
        "java/lang/String",
        "indexOf",
        "(Ljava/lang/String;)I",
        false,
    ),
    (
        "java/lang/String",
        "lastIndexOf",
        "(Ljava/lang/String;)I",
        false,
    ),
    (
        "java/lang/String",
        "substring",
        "(I)Ljava/lang/String;",
        false,
    ),
    (
        "java/lang/String",
        "substring",
        "(II)Ljava/lang/String;",
        false,
    ),
    (
        "java/lang/String",
        "concat",
        "(Ljava/lang/String;)Ljava/lang/String;",
        false,
    ),
    (
        "java/lang/String",
        "contains",
        "(Ljava/lang/CharSequence;)Z",
        false,
    ),
    (
        "java/lang/String",
        "startsWith",
        "(Ljava/lang/String;)Z",
        false,
    ),
    (
        "java/lang/String",
        "endsWith",
        "(Ljava/lang/String;)Z",
        false,
    ),
    (
        "java/lang/String",
        "equalsIgnoreCase",
        "(Ljava/lang/String;)Z",
        false,
    ),
    (
        "java/lang/String",
        "compareTo",
        "(Ljava/lang/String;)I",
        false,
    ),
    (
        "java/lang/String",
        "replace",
        "(CC)Ljava/lang/String;",
        false,
    ),
    (
        "java/lang/String",
        "toUpperCase",
        "()Ljava/lang/String;",
        false,
    ),
    (
        "java/lang/String",
        "toLowerCase",
        "()Ljava/lang/String;",
        false,
    ),
    ("java/lang/String", "trim", "()Ljava/lang/String;", false),
    ("java/lang/String", "strip", "()Ljava/lang/String;", false),
    ("java/lang/String", "repeat", "(I)Ljava/lang/String;", false),
    (
        "java/lang/String",
        "split",
        "(Ljava/lang/String;)[Ljava/lang/String;",
        false,
    ),
    ("java/lang/String", "toCharArray", "()[C", false),
    ("java/lang/String", "<init>", "()V", false),
    ("java/lang/String", "<init>", "(Ljava/lang/String;)V", false),
    ("java/lang/String", "<init>", "([C)V", false),
    ("java/lang/String", "<init>", "([B)V", false),
    ("java/lang/String", "valueOf", "(I)Ljava/lang/String;", true),
    ("java/lang/String", "valueOf", "(J)Ljava/lang/String;", true),
    ("java/lang/String", "valueOf", "(D)Ljava/lang/String;", true),
    ("java/lang/String", "valueOf", "(Z)Ljava/lang/String;", true),
    ("java/lang/String", "valueOf", "(C)Ljava/lang/String;", true),
    (
        "java/lang/String",
        "format",
        "(Ljava/lang/String;[Ljava/lang/Object;)Ljava/lang/String;",
        true,
    ),
    (
        "java/lang/String",
        "join",
        "(Ljava/lang/CharSequence;[Ljava/lang/CharSequence;)Ljava/lang/String;",
        true,
    ),
    (
        "java/util/Arrays",
        "asList",
        "([Ljava/lang/Object;)Ljava/util/List;",
        true,
    ),
    (
        "java/util/Arrays",
        "toString",
        "([Ljava/lang/Object;)Ljava/lang/String;",
        true,
    ),
    ("java/util/Arrays", "sort", "([I)V", true),
    ("java/util/Arrays", "fill", "([II)V", true),
    (
        "java/lang/String",
        "valueOf",
        "(Ljava/lang/Object;)Ljava/lang/String;",
        true,
    ),
    ("java/lang/StringBuilder", "<init>", "()V", false),
    (
        "java/lang/StringBuilder",
        "<init>",
        "(Ljava/lang/String;)V",
        false,
    ),
    (
        "java/lang/StringBuilder",
        "append",
        "(I)Ljava/lang/StringBuilder;",
        false,
    ),
    (
        "java/lang/StringBuilder",
        "append",
        "(J)Ljava/lang/StringBuilder;",
        false,
    ),
    (
        "java/lang/StringBuilder",
        "append",
        "(F)Ljava/lang/StringBuilder;",
        false,
    ),
    (
        "java/lang/StringBuilder",
        "append",
        "(D)Ljava/lang/StringBuilder;",
        false,
    ),
    (
        "java/lang/StringBuilder",
        "append",
        "(Z)Ljava/lang/StringBuilder;",
        false,
    ),
    (
        "java/lang/StringBuilder",
        "append",
        "(C)Ljava/lang/StringBuilder;",
        false,
    ),
    (
        "java/lang/StringBuilder",
        "append",
        "(Ljava/lang/String;)Ljava/lang/StringBuilder;",
        false,
    ),
    (
        "java/lang/StringBuilder",
        "append",
        "(Ljava/lang/Object;)Ljava/lang/StringBuilder;",
        false,
    ),
    (
        "java/lang/StringBuilder",
        "toString",
        "()Ljava/lang/String;",
        false,
    ),
    ("java/lang/StringBuilder", "length", "()I", false),
    ("java/lang/StringBuilder", "setLength", "(I)V", false),
    ("java/lang/StringBuilder", "charAt", "(I)C", false),
    (
        "java/lang/StringBuilder",
        "insert",
        "(ILjava/lang/String;)Ljava/lang/StringBuilder;",
        false,
    ),
    (
        "java/lang/StringBuilder",
        "deleteCharAt",
        "(I)Ljava/lang/StringBuilder;",
        false,
    ),
    (
        "java/lang/StringBuilder",
        "reverse",
        "()Ljava/lang/StringBuilder;",
        false,
    ),
    (
        "java/lang/StringBuilder",
        "indexOf",
        "(Ljava/lang/String;)I",
        false,
    ),
    ("java/lang/StringBuilder", "isEmpty", "()Z", false),
    (
        "java/lang/Integer",
        "parseInt",
        "(Ljava/lang/String;)I",
        true,
    ),
    (
        "java/lang/Integer",
        "valueOf",
        "(I)Ljava/lang/Integer;",
        true,
    ),
    (
        "java/lang/Integer",
        "toString",
        "(I)Ljava/lang/String;",
        true,
    ),
    ("java/lang/Integer", "intValue", "()I", false),
    ("java/lang/Long", "parseLong", "(Ljava/lang/String;)J", true),
    ("java/lang/Long", "valueOf", "(J)Ljava/lang/Long;", true),
    ("java/lang/Long", "toString", "(J)Ljava/lang/String;", true),
    ("java/lang/Long", "longValue", "()J", false),
    (
        "java/lang/Double",
        "parseDouble",
        "(Ljava/lang/String;)D",
        true,
    ),
    ("java/lang/Double", "valueOf", "(D)Ljava/lang/Double;", true),
    ("java/lang/Double", "doubleValue", "()D", false),
    (
        "java/lang/Float",
        "parseFloat",
        "(Ljava/lang/String;)F",
        true,
    ),
    ("java/lang/Float", "valueOf", "(F)Ljava/lang/Float;", true),
    ("java/lang/Float", "floatValue", "()F", false),
    (
        "java/lang/Boolean",
        "parseBoolean",
        "(Ljava/lang/String;)Z",
        true,
    ),
    (
        "java/lang/Boolean",
        "valueOf",
        "(Z)Ljava/lang/Boolean;",
        true,
    ),
    ("java/lang/Boolean", "booleanValue", "()Z", false),
    (
        "java/lang/Character",
        "valueOf",
        "(C)Ljava/lang/Character;",
        true,
    ),
    ("java/lang/Character", "charValue", "()C", false),
    ("java/lang/Character", "isDigit", "(C)Z", true),
    ("java/lang/Character", "isLetter", "(C)Z", true),
    ("java/lang/Character", "isWhitespace", "(C)Z", true),
    ("java/lang/Character", "toUpperCase", "(C)C", true),
    ("java/lang/Character", "toLowerCase", "(C)C", true),
    ("java/lang/Number", "intValue", "()I", false),
    ("java/lang/Number", "longValue", "()J", false),
    ("java/lang/Number", "floatValue", "()F", false),
    ("java/lang/Number", "doubleValue", "()D", false),
    ("java/lang/Number", "byteValue", "()B", false),
    ("java/lang/Number", "shortValue", "()S", false),
    ("java/lang/Integer", "compare", "(II)I", true),
    (
        "java/lang/Integer",
        "compareTo",
        "(Ljava/lang/Integer;)I",
        false,
    ),
    ("java/lang/Integer", "doubleValue", "()D", false),
    ("java/lang/Integer", "longValue", "()J", false),
    ("java/lang/Long", "compare", "(JJ)I", true),
    ("java/lang/Long", "intValue", "()I", false),
    ("java/lang/Long", "doubleValue", "()D", false),
    ("java/lang/Double", "compare", "(DD)I", true),
    ("java/lang/Float", "compare", "(FF)I", true),
    ("java/lang/Boolean", "hashCode", "(Z)I", true),
    ("java/lang/Byte", "hashCode", "(B)I", true),
    ("java/lang/Short", "hashCode", "(S)I", true),
    ("java/lang/Character", "hashCode", "(C)I", true),
    ("java/lang/Integer", "hashCode", "(I)I", true),
    ("java/lang/Long", "hashCode", "(J)I", true),
    ("java/lang/Float", "hashCode", "(F)I", true),
    ("java/lang/Double", "hashCode", "(D)I", true),
    ("java/lang/Double", "intValue", "()I", false),
    ("java/lang/Double", "isNaN", "()Z", false),
    ("java/lang/Float", "doubleValue", "()D", false),
    (
        "java/lang/Boolean",
        "toString",
        "(Z)Ljava/lang/String;",
        true,
    ),
    (
        "java/lang/Character",
        "toString",
        "(C)Ljava/lang/String;",
        true,
    ),
    ("java/lang/Math", "abs", "(I)I", true),
    ("java/lang/Math", "abs", "(J)J", true),
    ("java/lang/Math", "abs", "(F)F", true),
    ("java/lang/Math", "abs", "(D)D", true),
    ("java/lang/Math", "max", "(II)I", true),
    ("java/lang/Math", "max", "(JJ)J", true),
    ("java/lang/Math", "max", "(DD)D", true),
    ("java/lang/Math", "min", "(II)I", true),
    ("java/lang/Math", "min", "(JJ)J", true),
    ("java/lang/Math", "min", "(DD)D", true),
    ("java/lang/Math", "sqrt", "(D)D", true),
    ("java/lang/Math", "pow", "(DD)D", true),
    ("java/lang/Math", "floor", "(D)D", true),
    ("java/lang/Math", "ceil", "(D)D", true),
    ("java/lang/Math", "round", "(D)J", true),
    ("java/lang/Math", "random", "()D", true),
    (
        "java/lang/Iterable",
        "iterator",
        "()Ljava/util/Iterator;",
        false,
    ),
    ("java/util/Iterator", "hasNext", "()Z", false),
    ("java/util/Iterator", "next", "()Ljava/lang/Object;", false),
    ("java/util/Collection", "size", "()I", false),
    ("java/util/Collection", "isEmpty", "()Z", false),
    ("java/util/Collection", "clear", "()V", false),
    (
        "java/util/Collection",
        "add",
        "(Ljava/lang/Object;)Z",
        false,
    ),
    (
        "java/util/Collection",
        "remove",
        "(Ljava/lang/Object;)Z",
        false,
    ),
    (
        "java/util/Collection",
        "contains",
        "(Ljava/lang/Object;)Z",
        false,
    ),
    (
        "java/util/Collection",
        "iterator",
        "()Ljava/util/Iterator;",
        false,
    ),
    ("java/util/List", "get", "(I)Ljava/lang/Object;", false),
    (
        "java/util/List",
        "set",
        "(ILjava/lang/Object;)Ljava/lang/Object;",
        false,
    ),
    ("java/util/List", "add", "(ILjava/lang/Object;)V", false),
    ("java/util/List", "indexOf", "(Ljava/lang/Object;)I", false),
    ("java/util/List", "remove", "(I)Ljava/lang/Object;", false),
    ("java/util/ArrayList", "<init>", "()V", false),
    ("java/util/ArrayList", "<init>", "(I)V", false),
    ("java/util/Map", "size", "()I", false),
    ("java/util/Map", "isEmpty", "()Z", false),
    ("java/util/Map", "clear", "()V", false),
    (
        "java/util/Map",
        "get",
        "(Ljava/lang/Object;)Ljava/lang/Object;",
        false,
    ),
    (
        "java/util/Map",
        "put",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        false,
    ),
    (
        "java/util/Map",
        "remove",
        "(Ljava/lang/Object;)Ljava/lang/Object;",
        false,
    ),
    (
        "java/util/Map",
        "containsKey",
        "(Ljava/lang/Object;)Z",
        false,
    ),
    ("java/util/Map", "keySet", "()Ljava/util/Set;", false),
    ("java/util/Map", "values", "()Ljava/util/Collection;", false),
    ("java/util/Map", "entrySet", "()Ljava/util/Set;", false),
    (
        "java/util/Map",
        "containsValue",
        "(Ljava/lang/Object;)Z",
        false,
    ),
    (
        "java/util/Map",
        "getOrDefault",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        false,
    ),
    (
        "java/util/Map",
        "putIfAbsent",
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        false,
    ),
    ("java/util/Map", "putAll", "(Ljava/util/Map;)V", false),
    (
        "java/util/Map$Entry",
        "getKey",
        "()Ljava/lang/Object;",
        false,
    ),
    (
        "java/util/Map$Entry",
        "getValue",
        "()Ljava/lang/Object;",
        false,
    ),
    (
        "java/util/Collection",
        "addAll",
        "(Ljava/util/Collection;)Z",
        false,
    ),
    (
        "java/util/Collection",
        "removeAll",
        "(Ljava/util/Collection;)Z",
        false,
    ),
    (
        "java/util/Collection",
        "containsAll",
        "(Ljava/util/Collection;)Z",
        false,
    ),
    (
        "java/util/Collection",
        "toArray",
        "()[Ljava/lang/Object;",
        false,
    ),
    ("java/util/List", "subList", "(II)Ljava/util/List;", false),
    (
        "java/util/List",
        "lastIndexOf",
        "(Ljava/lang/Object;)I",
        false,
    ),
    ("java/util/Iterator", "remove", "()V", false),
    ("java/util/ArrayList", "<init>", "()V", false),
    ("java/util/ArrayList", "<init>", "(I)V", false),
    (
        "java/util/ArrayList",
        "<init>",
        "(Ljava/util/Collection;)V",
        false,
    ),
    ("java/util/LinkedList", "<init>", "()V", false),
    (
        "java/util/LinkedList",
        "<init>",
        "(Ljava/util/Collection;)V",
        false,
    ),
    ("java/util/HashSet", "<init>", "()V", false),
    ("java/util/HashSet", "<init>", "(I)V", false),
    (
        "java/util/HashSet",
        "<init>",
        "(Ljava/util/Collection;)V",
        false,
    ),
    ("java/util/LinkedHashSet", "<init>", "()V", false),
    ("java/util/TreeSet", "<init>", "()V", false),
    (
        "java/util/TreeSet",
        "<init>",
        "(Ljava/util/Collection;)V",
        false,
    ),
    ("java/util/HashMap", "<init>", "()V", false),
    ("java/util/HashMap", "<init>", "(I)V", false),
    ("java/util/HashMap", "<init>", "(Ljava/util/Map;)V", false),
    ("java/util/LinkedHashMap", "<init>", "()V", false),
    ("java/util/TreeMap", "<init>", "()V", false),
    ("java/util/TreeMap", "<init>", "(Ljava/util/Map;)V", false),
    ("java/util/Collections", "sort", "(Ljava/util/List;)V", true),
    (
        "java/util/Collections",
        "sort",
        "(Ljava/util/List;Ljava/util/Comparator;)V",
        true,
    ),
    (
        "java/util/Collections",
        "reverse",
        "(Ljava/util/List;)V",
        true,
    ),
    (
        "java/util/Collections",
        "shuffle",
        "(Ljava/util/List;)V",
        true,
    ),
    (
        "java/util/Collections",
        "max",
        "(Ljava/util/Collection;)Ljava/lang/Object;",
        true,
    ),
    (
        "java/util/Collections",
        "min",
        "(Ljava/util/Collection;)Ljava/lang/Object;",
        true,
    ),
    (
        "java/util/Collections",
        "emptyList",
        "()Ljava/util/List;",
        true,
    ),
    (
        "java/util/Collections",
        "unmodifiableList",
        "(Ljava/util/List;)Ljava/util/List;",
        true,
    ),
    (
        "java/util/Collections",
        "unmodifiableMap",
        "(Ljava/util/Map;)Ljava/util/Map;",
        true,
    ),
    (
        "java/util/Comparator",
        "compare",
        "(Ljava/lang/Object;Ljava/lang/Object;)I",
        false,
    ),
    (
        "java/lang/Comparable",
        "compareTo",
        "(Ljava/lang/Object;)I",
        false,
    ),
    ("java/util/Arrays", "sort", "([J)V", true),
    ("java/util/Arrays", "sort", "([D)V", true),
    ("java/util/Arrays", "sort", "([C)V", true),
    ("java/util/Arrays", "sort", "([Ljava/lang/Object;)V", true),
    ("java/util/Arrays", "copyOf", "([II)[I", true),
    (
        "java/util/Arrays",
        "copyOf",
        "([Ljava/lang/Object;I)[Ljava/lang/Object;",
        true,
    ),
    ("java/util/Arrays", "equals", "([I[I)Z", true),
    (
        "java/util/Arrays",
        "toString",
        "([I)Ljava/lang/String;",
        true,
    ),
    ("java/util/Arrays", "hashCode", "([I)I", true),
    (
        "java/util/Objects",
        "requireNonNull",
        "(Ljava/lang/Object;)Ljava/lang/Object;",
        true,
    ),
    (
        "java/util/Objects",
        "equals",
        "(Ljava/lang/Object;Ljava/lang/Object;)Z",
        true,
    ),
    (
        "java/util/Objects",
        "hashCode",
        "(Ljava/lang/Object;)I",
        true,
    ),
    (
        "java/util/Objects",
        "toString",
        "(Ljava/lang/Object;)Ljava/lang/String;",
        true,
    ),
    ("java/lang/AutoCloseable", "close", "()V", false),
    ("java/lang/AssertionError", "<init>", "()V", false),
    (
        "java/lang/AssertionError",
        "<init>",
        "(Ljava/lang/Object;)V",
        false,
    ),
    ("java/lang/Runnable", "run", "()V", false),
    ("java/lang/System", "currentTimeMillis", "()J", true),
    ("java/lang/System", "nanoTime", "()J", true),
    ("java/io/PrintStream", "println", "()V", false),
    (
        "java/io/PrintStream",
        "println",
        "(Ljava/lang/String;)V",
        false,
    ),
    ("java/io/PrintStream", "println", "(I)V", false),
    ("java/io/PrintStream", "println", "(J)V", false),
    ("java/io/PrintStream", "println", "(D)V", false),
    ("java/io/PrintStream", "println", "(Z)V", false),
    ("java/io/PrintStream", "println", "(C)V", false),
    (
        "java/io/PrintStream",
        "println",
        "(Ljava/lang/Object;)V",
        false,
    ),
    (
        "java/io/PrintStream",
        "print",
        "(Ljava/lang/String;)V",
        false,
    ),
    ("java/io/PrintStream", "print", "(I)V", false),
];

const BUILT_IN_ABOVE: &[(&str, &str)] = &[
    ("java/lang/String", "java/lang/CharSequence"),
    ("java/lang/String", "java/lang/Comparable"),
    ("java/lang/StringBuilder", "java/lang/CharSequence"),
    ("java/lang/Exception", "java/lang/Throwable"),
    ("java/lang/Error", "java/lang/Throwable"),
    ("java/lang/RuntimeException", "java/lang/Exception"),
    ("java/lang/AssertionError", "java/lang/Error"),
    (
        "java/lang/ArithmeticException",
        "java/lang/RuntimeException",
    ),
    ("java/lang/ClassCastException", "java/lang/RuntimeException"),
    (
        "java/lang/IllegalArgumentException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/IllegalStateException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/IndexOutOfBoundsException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/ArrayIndexOutOfBoundsException",
        "java/lang/IndexOutOfBoundsException",
    ),
    (
        "java/lang/NullPointerException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/NumberFormatException",
        "java/lang/IllegalArgumentException",
    ),
    (
        "java/lang/UnsupportedOperationException",
        "java/lang/RuntimeException",
    ),
    ("java/lang/InterruptedException", "java/lang/Exception"),
    (
        "java/lang/CloneNotSupportedException",
        "java/lang/Exception",
    ),
    ("java/io/IOException", "java/lang/Exception"),
    ("java/lang/IncompatibleClassChangeError", "java/lang/Error"),
    ("java/lang/MatchException", "java/lang/RuntimeException"),
    ("java/util/Collection", "java/lang/Iterable"),
    ("java/util/List", "java/util/Collection"),
    ("java/util/Set", "java/util/Collection"),
    ("java/util/ArrayList", "java/util/List"),
    ("java/util/LinkedList", "java/util/List"),
    ("java/util/HashSet", "java/util/Set"),
    ("java/util/LinkedHashSet", "java/util/HashSet"),
    ("java/util/TreeSet", "java/util/Set"),
    ("java/util/HashMap", "java/util/Map"),
    ("java/util/LinkedHashMap", "java/util/HashMap"),
    ("java/util/TreeMap", "java/util/Map"),
    ("java/lang/Integer", "java/lang/Number"),
    ("java/lang/Long", "java/lang/Number"),
    ("java/lang/Double", "java/lang/Number"),
    ("java/lang/Float", "java/lang/Number"),
    ("java/lang/Short", "java/lang/Number"),
    ("java/lang/Byte", "java/lang/Number"),
];

const BUILT_IN_VARIADIC: &[(&str, &str)] = &[
    ("java/lang/String", "format"),
    ("java/lang/String", "join"),
    ("java/util/Arrays", "asList"),
    ("java/util/Arrays", "toString"),
];

const BUILT_IN_INTERFACES: &[&str] = &[
    "java/lang/annotation/Annotation",
    "java/lang/Runnable",
    "java/lang/CharSequence",
    "java/lang/Iterable",
    "java/lang/AutoCloseable",
    "java/util/Iterator",
    "java/util/Collection",
    "java/util/List",
    "java/util/Set",
    "java/util/Map",
    "java/util/Map$Entry",
    "java/util/Comparator",
    "java/lang/Comparable",
];

const BUILT_IN_THROWABLES: &[&str] = &[
    "java/lang/AssertionError",
    "java/lang/Throwable",
    "java/lang/Exception",
    "java/lang/RuntimeException",
    "java/lang/Error",
    "java/lang/ArithmeticException",
    "java/lang/ArrayIndexOutOfBoundsException",
    "java/lang/ClassCastException",
    "java/lang/CloneNotSupportedException",
    "java/lang/IllegalArgumentException",
    "java/lang/IllegalStateException",
    "java/lang/IndexOutOfBoundsException",
    "java/lang/InterruptedException",
    "java/lang/NullPointerException",
    "java/lang/NumberFormatException",
    "java/lang/UnsupportedOperationException",
    "java/lang/AssertionError",
    "java/io/IOException",
    "java/lang/IncompatibleClassChangeError",
    "java/lang/MatchException",
];

const BUILT_IN_FIELDS: &[(&str, &str, &str)] = &[
    ("java/lang/System", "out", "Ljava/io/PrintStream;"),
    ("java/lang/System", "err", "Ljava/io/PrintStream;"),
    ("java/lang/Integer", "MAX_VALUE", "I"),
    ("java/lang/Integer", "MIN_VALUE", "I"),
    ("java/lang/Long", "MAX_VALUE", "J"),
    ("java/lang/Long", "MIN_VALUE", "J"),
    ("java/lang/Double", "MAX_VALUE", "D"),
    ("java/lang/Double", "MIN_VALUE", "D"),
    ("java/lang/Float", "MAX_VALUE", "F"),
    ("java/lang/Float", "MIN_VALUE", "F"),
    ("java/lang/Short", "MAX_VALUE", "S"),
    ("java/lang/Short", "MIN_VALUE", "S"),
    ("java/lang/Byte", "MAX_VALUE", "B"),
    ("java/lang/Byte", "MIN_VALUE", "B"),
    ("java/lang/Character", "MAX_VALUE", "C"),
    ("java/lang/Character", "MIN_VALUE", "C"),
];

fn built_in_method(owner: &str, name: &str, count: usize) -> Option<Signature> {
    built_in_overloads(owner, name, count).into_iter().next()
}

fn built_in_overloads(owner: &str, name: &str, count: usize) -> Vec<Signature> {
    if name == "<init>" && count <= 1 && BUILT_IN_THROWABLES.contains(&owner) {
        let parameters = if count == 1 {
            vec![Type::Object("java/lang/String".to_string(), Vec::new())]
        } else {
            Vec::new()
        };
        return vec![Signature {
            owner: owner.to_string(),
            name: name.to_string(),
            parameters,
            returns: Type::Void,
            static_: false,
            interface: false,
            variadic: false,
            abstract_: false,
            returns_variable: None,
            parameter_variables: Vec::new(),
            taken_stands_for: Vec::new(),
            returns_stands_for: Standing::Fixed,
            own_variables: Vec::new(),
            witnessed: Vec::new(),
            throws: Vec::new(),
        }];
    }

    let mut owners = vec![owner.to_string()];
    if BUILT_IN_THROWABLES.contains(&owner) {
        owners.push("java/lang/Throwable".to_string());
    }
    let mut at = owner.to_string();
    while let Some((_, above)) = BUILT_IN_ABOVE.iter().find(|(below, _)| *below == at) {
        if owners.iter().any(|held| held == above) {
            break;
        }
        owners.push((*above).to_string());
        at = (*above).to_string();
    }
    owners.push("java/lang/Object".to_string());

    let mut found = Vec::new();
    for held in owners {
        for (class, held_name, descriptor, static_) in BUILT_IN_METHODS {
            if *class != held || *held_name != name {
                continue;
            }
            let Some((parameters, returns)) = read_descriptor(descriptor) else {
                continue;
            };
            if parameters.len() != count {
                continue;
            }
            found.push(Signature {
                owner: class.to_string(),
                name: name.to_string(),
                parameters,
                returns,
                static_: *static_,
                interface: BUILT_IN_INTERFACES.contains(class),
                variadic: BUILT_IN_VARIADIC.contains(&(*class, *held_name)),
                abstract_: BUILT_IN_INTERFACES.contains(class),
                returns_variable: None,
                parameter_variables: Vec::new(),
                taken_stands_for: Vec::new(),
                returns_stands_for: Standing::Fixed,
                own_variables: Vec::new(),
                witnessed: Vec::new(),
                throws: Vec::new(),
            });
        }
        if !found.is_empty() {
            break;
        }
    }
    found
}

fn built_in_field(owner: &str, name: &str) -> Option<Type> {
    let (_, _, descriptor) = BUILT_IN_FIELDS
        .iter()
        .find(|(class, held, _)| *class == owner && *held == name)?;
    let mut at = 0usize;
    read_type(descriptor, &mut at)
}

fn written_as_a_path(expression: &Expression) -> Option<String> {
    match expression {
        Expression::Name(name) => Some(name.clone()),
        Expression::Field { of, name } => Some(format!("{}.{name}", written_as_a_path(of)?)),
        _ => None,
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum Standing {
    #[default]
    Fixed,

    OfTheClass(usize),

    OfTheMethod(String),

    Holding(Vec<Standing>),

    ArrayOf(Box<Standing>),
}

impl Standing {
    fn mentions_the_class(&self) -> bool {
        match self {
            Standing::OfTheClass(_) => true,
            Standing::Holding(inside) => inside.iter().any(Standing::mentions_the_class),
            Standing::ArrayOf(of) => of.mentions_the_class(),
            _ => false,
        }
    }

    pub fn as_seen(&self, erased: &Type, of_the_class: &[Type]) -> Type {
        self.filled(erased, of_the_class, &[])
    }

    fn says_nothing(&self) -> bool {
        match self {
            Standing::Fixed => true,
            Standing::Holding(inside) => inside.iter().all(Standing::says_nothing),
            Standing::ArrayOf(of) => of.says_nothing(),
            _ => false,
        }
    }

    fn bind(&self, actual: &Type, out: &mut Vec<(String, Type)>) {
        match self {
            Standing::OfTheMethod(name) => {
                let actual = match boxed_name(actual) {
                    Some(boxed) => Type::Object(boxed.to_string(), Vec::new()),
                    None => actual.clone(),
                };
                let nothing = Type::Object("java/lang/Object".to_string(), Vec::new());

                if actual == nothing {
                    return;
                }
                match out.iter_mut().find(|(held, _)| held == name) {
                    Some((_, before)) if *before != actual => {
                        *before = nothing;
                    }
                    Some(_) => {}
                    None => out.push((name.clone(), actual)),
                }
            }
            Standing::Holding(inside) => {
                if let Type::Object(_, arguments) = actual {
                    for (one, held) in inside.iter().zip(arguments.iter()) {
                        one.bind(held, out);
                    }
                }
            }
            Standing::ArrayOf(of) => {
                if let Type::Array(held) = actual {
                    of.bind(held, out);
                }
            }
            _ => {}
        }
    }

    fn filled(
        &self,
        erased: &Type,
        of_the_class: &[Type],
        of_the_method: &[(String, Type)],
    ) -> Type {
        match self {
            Standing::OfTheClass(at) => of_the_class.get(*at).cloned().unwrap_or(erased.clone()),
            Standing::OfTheMethod(name) => of_the_method
                .iter()
                .find(|(held, _)| held == name)
                .map(|(_, what)| what.clone())
                .unwrap_or(erased.clone()),
            Standing::Holding(inside) => {
                let Type::Object(named, arguments) = erased else {
                    return erased.clone();
                };

                let object = Type::Object("java/lang/Object".to_string(), Vec::new());
                let filled = inside
                    .iter()
                    .enumerate()
                    .map(|(at, one)| {
                        one.filled(
                            arguments.get(at).unwrap_or(&object),
                            of_the_class,
                            of_the_method,
                        )
                    })
                    .collect();
                Type::Object(named.clone(), filled)
            }
            Standing::ArrayOf(of) => match erased {
                Type::Array(held) => {
                    Type::Array(Box::new(of.filled(held, of_the_class, of_the_method)))
                }
                _ => erased.clone(),
            },
            Standing::Fixed => erased.clone(),
        }
    }
}

fn standing_of(written: &Written) -> Standing {
    match written {
        Written::Variable(at, _) => Standing::OfTheClass(*at),
        Written::OwnVariable(name, _) => Standing::OfTheMethod(name.clone()),
        Written::Array(of) => {
            let inside = standing_of(of);
            if inside.says_nothing() {
                Standing::Fixed
            } else {
                Standing::ArrayOf(Box::new(inside))
            }
        }
        Written::Named(_, arguments) if !arguments.is_empty() => {
            let inside: Vec<Standing> = arguments.iter().map(standing_of).collect();
            if inside.iter().all(Standing::says_nothing) {
                Standing::Fixed
            } else {
                Standing::Holding(inside)
            }
        }
        _ => Standing::Fixed,
    }
}

fn standing_in(bytes: &[u8], at: &mut usize, of_the_class: &[String]) -> Option<Standing> {
    match *bytes.get(*at)? {
        b'T' => {
            *at += 1;
            let from = *at;
            while *bytes.get(*at)? != b';' {
                *at += 1;
            }
            let name = String::from_utf8_lossy(&bytes[from..*at]).into_owned();
            *at += 1;
            Some(match of_the_class.iter().position(|one| *one == name) {
                Some(held) => Standing::OfTheClass(held),
                None => Standing::OfTheMethod(name),
            })
        }
        b'[' => {
            *at += 1;
            let inside = standing_in(bytes, at, of_the_class)?;
            Some(if inside.says_nothing() {
                Standing::Fixed
            } else {
                Standing::ArrayOf(Box::new(inside))
            })
        }
        b'L' => {
            let mut inside: Vec<Standing> = Vec::new();
            loop {
                match *bytes.get(*at)? {
                    b'<' => {
                        *at += 1;
                        while *bytes.get(*at)? != b'>' {
                            if matches!(*bytes.get(*at)?, b'+' | b'-') {
                                *at += 1;
                                inside.push(standing_in(bytes, at, of_the_class)?);
                                continue;
                            }
                            if *bytes.get(*at)? == b'*' {
                                *at += 1;
                                inside.push(Standing::Fixed);
                                continue;
                            }
                            inside.push(standing_in(bytes, at, of_the_class)?);
                        }
                        *at += 1;
                    }
                    b';' => {
                        *at += 1;
                        break;
                    }
                    _ => *at += 1,
                }
            }
            Some(if inside.iter().all(Standing::says_nothing) {
                Standing::Fixed
            } else {
                Standing::Holding(inside)
            })
        }
        _ => {
            skip_generic_type(bytes, at)?;
            Some(Standing::Fixed)
        }
    }
}

fn not_shadowed(signature: &str, of_the_class: &[String]) -> Vec<String> {
    let own = type_parameters_of(signature);
    of_the_class
        .iter()
        .map(|one| {
            if own.contains(one) {
                String::new()
            } else {
                one.clone()
            }
        })
        .collect()
}

fn standings_of(signature: &str, of_the_class: &[String]) -> Option<(Vec<Standing>, Standing)> {
    let of_the_class = &not_shadowed(signature, of_the_class);
    let bytes = signature.as_bytes();
    let mut at = 0usize;

    if bytes.first() == Some(&b'<') {
        let mut depth = 0usize;
        loop {
            let held = *bytes.get(at)?;
            at += 1;
            match held {
                b'<' => depth += 1,
                b'>' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    if bytes.get(at) != Some(&b'(') {
        return None;
    }
    at += 1;
    let mut taken = Vec::new();
    while *bytes.get(at)? != b')' {
        taken.push(standing_in(bytes, &mut at, of_the_class)?);
    }
    at += 1;
    let returns = standing_in(bytes, &mut at, of_the_class)?;
    Some((taken, returns))
}

fn type_parameters_of(signature: &str) -> Vec<String> {
    let bytes = signature.as_bytes();
    if bytes.first() != Some(&b'<') {
        return Vec::new();
    }
    let mut at = 1usize;
    let mut found = Vec::new();
    while at < bytes.len() && bytes[at] != b'>' {
        let began = at;
        while at < bytes.len() && bytes[at] != b':' {
            at += 1;
        }
        if at >= bytes.len() {
            return Vec::new();
        }
        found.push(signature[began..at].to_string());

        while at < bytes.len() && bytes[at] == b':' {
            at += 1;
            if at < bytes.len()
                && bytes[at] != b':'
                && bytes[at] != b'>'
                && skip_generic_type(bytes, &mut at).is_none()
            {
                return Vec::new();
            }
        }
    }
    found
}

fn read_generic_type(bytes: &[u8], at: &mut usize) -> Option<Option<String>> {
    let began = *at;
    let held = *bytes.get(*at)?;
    if held == b'T' {
        *at += 1;
        let from = *at;
        while *bytes.get(*at)? != b';' {
            *at += 1;
        }
        let name = String::from_utf8_lossy(&bytes[from..*at]).into_owned();
        *at += 1;
        return Some(Some(name));
    }
    skip_generic_type(bytes, at)?;
    let _ = began;
    Some(None)
}

fn bounds_of(signature: &str) -> Vec<(String, Type)> {
    let bytes = signature.as_bytes();
    let mut out: Vec<(String, Type)> = Vec::new();
    if bytes.first() != Some(&b'<') {
        return out;
    }
    let mut at = 1usize;
    while at < bytes.len() && bytes[at] != b'>' {
        let began = at;
        while at < bytes.len() && bytes[at] != b':' {
            at += 1;
        }
        if at >= bytes.len() {
            return out;
        }
        let name = signature[began..at].to_string();
        let mut bound: Option<Type> = None;
        while at < bytes.len() && bytes[at] == b':' {
            at += 1;
            if at < bytes.len() && bytes[at] != b':' && bytes[at] != b'>' {
                let Some(found) = generic_type_at(bytes, &mut at, &out) else {
                    return out;
                };
                if bound.is_none() {
                    bound = Some(match found {
                        Type::Object(named, _) => Type::Object(named, Vec::new()),
                        other => other,
                    });
                }
            }
        }
        out.push((
            name,
            bound.unwrap_or_else(|| Type::Object("java/lang/Object".to_string(), Vec::new())),
        ));
    }
    out
}

type Placed = (Type, Standing, Vec<(Type, Standing)>);

fn placed_at(
    bytes: &[u8],
    at: &mut usize,
    variables: &[String],
    bounds: &[(String, Type)],
) -> Option<Placed> {
    let object = || Type::Object("java/lang/Object".to_string(), Vec::new());
    match *bytes.get(*at)? {
        b'T' => {
            *at += 1;
            let from = *at;
            while *bytes.get(*at)? != b';' {
                *at += 1;
            }
            let name = String::from_utf8_lossy(&bytes[from..*at]).into_owned();
            *at += 1;
            let erased = bounds
                .iter()
                .find(|(held, _)| *held == name)
                .map(|(_, bound)| bound.clone())
                .unwrap_or_else(object);
            let standing = match variables.iter().position(|held| *held == name) {
                Some(index) => Standing::OfTheClass(index),
                None => Standing::Fixed,
            };
            Some((erased, standing, Vec::new()))
        }
        b'[' => {
            *at += 1;
            let (inner, standing, _) = placed_at(bytes, at, variables, bounds)?;
            let standing = match standing {
                Standing::Fixed => Standing::Fixed,
                other => Standing::ArrayOf(Box::new(other)),
            };
            Some((Type::Array(Box::new(inner)), standing, Vec::new()))
        }
        b'L' => {
            *at += 1;
            let mut name = String::new();
            let mut arguments: Vec<(Type, Standing)> = Vec::new();
            loop {
                let held = *bytes.get(*at)?;
                match held {
                    b';' => {
                        *at += 1;
                        break;
                    }
                    b'<' => {
                        *at += 1;
                        arguments.clear();
                        while *bytes.get(*at)? != b'>' {
                            match bytes[*at] {
                                b'*' => {
                                    *at += 1;
                                    arguments.push((object(), Standing::Fixed));
                                }
                                b'-' => {
                                    *at += 1;
                                    let (erased, standing, _) =
                                        placed_at(bytes, at, variables, bounds)?;
                                    arguments.push((erased, standing));
                                }
                                b'+' => {
                                    *at += 1;
                                    let (erased, standing, _) =
                                        placed_at(bytes, at, variables, bounds)?;
                                    arguments.push((erased, standing));
                                }
                                _ => {
                                    let (erased, standing, _) =
                                        placed_at(bytes, at, variables, bounds)?;
                                    arguments.push((erased, standing));
                                }
                            }
                        }
                        *at += 1;
                    }
                    b'.' => {
                        *at += 1;
                        name.push('$');
                        arguments.clear();
                    }
                    other => {
                        *at += 1;
                        name.push(char::from(other));
                    }
                }
            }
            let standing = if arguments.iter().all(|(_, one)| one.says_nothing()) {
                Standing::Fixed
            } else {
                Standing::Holding(arguments.iter().map(|(_, one)| one.clone()).collect())
            };
            let erased = Type::Object(
                name,
                arguments.iter().map(|(erased, _)| erased.clone()).collect(),
            );
            Some((erased, standing, arguments))
        }
        _ => {
            let text = String::from_utf8_lossy(bytes).into_owned();
            Some((read_type(&text, at)?, Standing::Fixed, Vec::new()))
        }
    }
}

type Parts = (Vec<Option<Type>>, Option<Type>);

fn fixed_parts(signature: &str, of_the_class: &[String]) -> Option<Parts> {
    let bytes = signature.as_bytes();
    let mut variables: Vec<String> = of_the_class.to_vec();
    variables.extend(type_parameters_of(signature));
    let mut at = 0usize;
    if bytes.first() == Some(&b'<') {
        let mut depth = 0usize;
        loop {
            match *bytes.get(at)? {
                b'<' => depth += 1,
                b'>' => {
                    depth -= 1;
                    if depth == 0 {
                        at += 1;
                        break;
                    }
                }
                _ => {}
            }
            at += 1;
        }
    }
    if *bytes.get(at)? != b'(' {
        return None;
    }
    at += 1;
    let mut parameters = Vec::new();
    while *bytes.get(at)? != b')' {
        let (erased, standing, _) = placed_at(bytes, &mut at, &variables, &[])?;
        parameters.push(matches!(standing, Standing::Fixed).then_some(erased));
    }
    at += 1;
    let returns = if *bytes.get(at)? == b'V' {
        None
    } else {
        let (erased, standing, _) = placed_at(bytes, &mut at, &variables, &[])?;
        matches!(standing, Standing::Fixed).then_some(erased)
    };
    Some((parameters, returns))
}

fn above_of_signature(signature: &str) -> Vec<(String, Vec<(Type, Standing)>)> {
    let variables = type_parameters_of(signature);
    let bounds = bounds_of(signature);
    let bytes = signature.as_bytes();
    let mut at = 0usize;
    if bytes.first() == Some(&b'<') {
        let mut depth = 0usize;
        while at < bytes.len() {
            match bytes[at] {
                b'<' => depth += 1,
                b'>' => {
                    depth -= 1;
                    if depth == 0 {
                        at += 1;
                        break;
                    }
                }
                _ => {}
            }
            at += 1;
        }
    }
    let mut out = Vec::new();
    while at < bytes.len() {
        let Some((Type::Object(name, _), _, arguments)) =
            placed_at(bytes, &mut at, &variables, &bounds)
        else {
            break;
        };
        if !arguments.is_empty() {
            out.push((name, arguments));
        }
    }
    out
}

fn generic_type_of(signature: &str, bounds: &[(String, Type)]) -> Option<Type> {
    let mut at = 0usize;
    let found = generic_type_at(signature.as_bytes(), &mut at, bounds)?;
    (at == signature.len()).then_some(found)
}

fn generic_type_at(bytes: &[u8], at: &mut usize, bounds: &[(String, Type)]) -> Option<Type> {
    let object = || Type::Object("java/lang/Object".to_string(), Vec::new());
    match *bytes.get(*at)? {
        b'T' => {
            *at += 1;
            let from = *at;
            while *bytes.get(*at)? != b';' {
                *at += 1;
            }
            let name = String::from_utf8_lossy(&bytes[from..*at]).into_owned();
            *at += 1;
            Some(
                bounds
                    .iter()
                    .find(|(held, _)| *held == name)
                    .map(|(_, bound)| bound.clone())
                    .unwrap_or_else(object),
            )
        }
        b'[' => {
            *at += 1;
            Some(Type::Array(Box::new(generic_type_at(bytes, at, bounds)?)))
        }
        b'L' => {
            *at += 1;
            let mut name = String::new();
            let mut arguments = Vec::new();
            loop {
                let held = *bytes.get(*at)?;
                match held {
                    b';' => {
                        *at += 1;
                        break;
                    }
                    b'<' => {
                        *at += 1;
                        arguments.clear();
                        while *bytes.get(*at)? != b'>' {
                            match bytes[*at] {
                                b'*' => {
                                    *at += 1;
                                    arguments.push(object());
                                }
                                b'+' => {
                                    *at += 1;
                                    arguments.push(generic_type_at(bytes, at, bounds)?);
                                }
                                b'-' => {
                                    *at += 1;
                                    generic_type_at(bytes, at, bounds)?;
                                    arguments.push(object());
                                }
                                _ => arguments.push(generic_type_at(bytes, at, bounds)?),
                            }
                        }
                        *at += 1;
                    }
                    b'.' => {
                        *at += 1;
                        name.push('$');
                        arguments.clear();
                    }
                    other => {
                        *at += 1;
                        name.push(char::from(other));
                    }
                }
            }
            Some(Type::Object(name, arguments))
        }
        _ => {
            let text = String::from_utf8_lossy(bytes).into_owned();
            read_type(&text, at)
        }
    }
}

fn skip_generic_type(bytes: &[u8], at: &mut usize) -> Option<()> {
    match *bytes.get(*at)? {
        b'[' => {
            *at += 1;
            skip_generic_type(bytes, at)
        }
        b'T' => {
            while *bytes.get(*at)? != b';' {
                *at += 1;
            }
            *at += 1;
            Some(())
        }
        b'L' => {
            let mut depth = 0usize;
            loop {
                let held = *bytes.get(*at)?;
                *at += 1;
                match held {
                    b'<' => depth += 1,
                    b'>' => depth = depth.checked_sub(1)?,
                    b';' if depth == 0 => return Some(()),
                    _ => {}
                }
            }
        }
        b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z' | b'V' => {
            *at += 1;
            Some(())
        }

        b'*' => {
            *at += 1;
            Some(())
        }
        b'+' | b'-' => {
            *at += 1;
            skip_generic_type(bytes, at)
        }
        _ => None,
    }
}

fn variables_of(signature: &str, parameters: &[String]) -> (Option<usize>, Vec<Option<usize>>) {
    let parameters = &not_shadowed(signature, parameters);
    let bytes = signature.as_bytes();
    let mut at = 0usize;

    if bytes.first() == Some(&b'<') {
        let mut depth = 0usize;
        loop {
            let Some(held) = bytes.get(at) else {
                return (None, Vec::new());
            };
            at += 1;
            match held {
                b'<' => depth += 1,
                b'>' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    if bytes.get(at) != Some(&b'(') {
        return (None, Vec::new());
    }
    at += 1;
    let index = |name: Option<String>| -> Option<usize> {
        let name = name?;
        parameters.iter().position(|one| *one == name)
    };
    let mut taken = Vec::new();
    while bytes.get(at) != Some(&b')') {
        let Some(one) = read_generic_type(bytes, &mut at) else {
            return (None, Vec::new());
        };
        taken.push(index(one));
    }
    at += 1;
    let returns = match read_generic_type(bytes, &mut at) {
        Some(one) => index(one),
        None => None,
    };
    (returns, taken)
}

fn which_variable(written: &Written) -> Option<usize> {
    match written {
        Written::Variable(at, _) => Some(*at),
        _ => None,
    }
}

fn resolve_written(classpath: &Classpath, unit: &Unit, written: &Written) -> Option<Type> {
    Some(match written {
        Written::Void => Type::Void,
        Written::Boolean => Type::Boolean,
        Written::Byte => Type::Byte,
        Written::Short => Type::Short,
        Written::Char => Type::Char,
        Written::Int => Type::Int,
        Written::Long => Type::Long,
        Written::Float => Type::Float,
        Written::Double => Type::Double,
        Written::Array(of) => Type::Array(Box::new(resolve_written(classpath, unit, of)?)),
        Written::Variable(_, bound) | Written::OwnVariable(_, bound) => {
            resolve_written(classpath, unit, bound)?
        }
        Written::Named(name, held) => {
            let mut arguments = Vec::new();
            for one in held {
                match resolve_written(classpath, unit, one) {
                    Some(found) => arguments.push(found),

                    None => {
                        arguments.clear();
                        break;
                    }
                }
            }
            Type::Object(resolve_named(classpath, unit, name)?, arguments)
        }
        Written::Inferred => return None,
    })
}

fn resolve_named(classpath: &Classpath, unit: &Unit, name: &str) -> Option<String> {
    resolve_named_within(classpath, unit, name, true)
}

fn resolve_named_within(
    classpath: &Classpath,
    unit: &Unit,
    name: &str,
    through_what_it_extends: bool,
) -> Option<String> {
    let exists =
        |internal: &str| classpath.get(internal).is_some() || WELL_KNOWN.contains(&internal);
    if name == unit.name {
        return Some(unit.internal_name());
    }

    let mut holder = Some(unit.internal_name());
    while let Some(current) = holder {
        let nested = format!("{current}${name}");
        if exists(&nested) {
            return Some(nested);
        }
        holder = current
            .rsplit_once('$')
            .map(|(before, _)| before.to_string());
    }

    let mut holder = Some(unit.internal_name());
    while let Some(current) = holder {
        let mut walking = Vec::new();
        let own = current == unit.internal_name();
        if !own || through_what_it_extends {
            if let Some(known) = classpath.get(&current) {
                walking.extend(known.superclass.clone());
                walking.extend(known.interfaces.iter().cloned());
            }
        }
        if own && through_what_it_extends {
            if let Some(parent) = unit.extends.clone() {
                walking.extend(resolve_named_within(classpath, unit, &parent, false));
            }
            for one in &unit.implements {
                walking.extend(resolve_named_within(classpath, unit, one, false));
            }
        }
        let mut seen: Vec<String> = Vec::new();
        while let Some(above) = walking.pop() {
            if seen.contains(&above) || seen.len() > 64 {
                continue;
            }
            let nested = format!("{above}${name}");
            if exists(&nested) {
                return Some(nested);
            }
            if let Some(known) = classpath.get(&above) {
                walking.extend(known.superclass.clone());
                walking.extend(known.interfaces.iter().cloned());
            }
            seen.push(above);
        }
        holder = current
            .rsplit_once('$')
            .map(|(before, _)| before.to_string());
    }
    if name.contains('.') {
        let internal = name.replace('.', "/");
        if exists(&internal) {
            return Some(internal);
        }
    }

    if let Some((before, last)) = name.rsplit_once('.') {
        if let Some(holder) = resolve_named_within(classpath, unit, before, through_what_it_extends)
        {
            let nested = format!("{holder}${last}");
            if exists(&nested) {
                return Some(nested);
            }
            for above in classpath.ancestors(&holder).into_iter().skip(1) {
                let nested = format!("{above}${last}");
                if exists(&nested) {
                    return Some(nested);
                }
            }
        }
    }
    if let Some(package) = &unit.package {
        let beside = format!("{}/{name}", package.replace('.', "/"));
        if exists(&beside) {
            return Some(beside);
        }
    } else if exists(name) {
        return Some(name.to_string());
    }
    let nested_or_not = |dotted: &str| -> Option<String> {
        let mut internal = dotted.replace('.', "/");
        for _ in 0..8 {
            if exists(&internal) {
                return Some(internal);
            }
            let cut = internal.rfind('/')?;
            internal.replace_range(cut..=cut, "$");
        }
        None
    };
    for import in &unit.imports {
        if import.rsplit('.').next() == Some(name) {
            if let Some(internal) = nested_or_not(import) {
                return Some(internal);
            }
        }
    }
    for import in &unit.imports {
        if let Some(prefix) = import.strip_suffix(".*") {
            if let Some(candidate) = nested_or_not(&format!("{prefix}.{name}")) {
                return Some(candidate);
            }
        }
    }
    let in_lang = format!("java/lang/{name}");
    exists(&in_lang).then_some(in_lang)
}

pub(crate) const THE_LIBRARY: &[(&str, &str)] = &[
    (
        "java.lang",
        r####"
package java.lang;

public class Object {
    public native Object();
    public native boolean equals(Object other);
    public native int hashCode();
    public native String toString();
    public native Class getClass();
    public native void notify();
    public native void notifyAll();
    public native void wait() throws java.lang.InterruptedException ;
    public native void wait(long millis) throws java.lang.InterruptedException ;
}

public interface CharSequence {
    int length();
    char charAt(int index);
    CharSequence subSequence(int start, int end);
    String toString();
    boolean isEmpty();
}

public interface Comparable<T> {
    int compareTo(T other);
}

public interface Iterable<T> {
    java.util.Iterator<T> iterator();
}

public interface Runnable {
    void run();
}

public interface AutoCloseable {
    void close() throws java.lang.Exception ;
}

public interface Cloneable { }

public interface Appendable {
    Appendable append(CharSequence held) throws java.io.IOException ;
}

public class String implements CharSequence, Comparable<String> {
    public native String();
    public native String(String other);
    public native String(char[] value);
    public native String(char[] value, int offset, int count);
    public native String(byte[] value);
    public native String(byte[] value, String charset) throws java.io.UnsupportedEncodingException ;
    public native String(byte[] value, java.nio.charset.Charset charset);
    public native String(byte[] value, int offset, int count);
    public native String(byte[] value, int offset, int count, String charset) throws java.io.UnsupportedEncodingException ;
    public native String(StringBuilder held);
    public native int length();
    public native boolean isEmpty();
    public native boolean isBlank();
    public native char charAt(int index);
    public native int codePointAt(int index);
    public native int indexOf(int letter);
    public native int indexOf(int letter, int from);
    public native int indexOf(String held);
    public native int indexOf(String held, int from);
    public native int lastIndexOf(int letter);
    public native int lastIndexOf(String held);
    public native String substring(int from);
    public native String substring(int from, int to);
    public native CharSequence subSequence(int start, int end);
    public native String concat(String other);
    public native String replace(char from, char to);
    public native String replace(CharSequence from, CharSequence to);
    public native String replaceAll(String pattern, String with);
    public native String replaceFirst(String pattern, String with);
    public native boolean matches(String pattern);
    public native String[] split(String pattern);
    public native String[] split(String pattern, int limit);
    public native String toLowerCase();
    public native String toUpperCase();
    public native String trim();
    public native String strip();
    public native String repeat(int times);
    public native boolean startsWith(String held);
    public native boolean startsWith(String held, int from);
    public native boolean endsWith(String held);
    public native boolean contains(CharSequence held);
    public native boolean equalsIgnoreCase(String other);
    public native int compareTo(String other);
    public native int compareToIgnoreCase(String other);
    public native char[] toCharArray();
    public native byte[] getBytes();
    public native byte[] getBytes(String charset) throws java.io.UnsupportedEncodingException ;
    public native byte[] getBytes(java.nio.charset.Charset charset);
    public native String stripLeading();
    public native String stripTrailing();
    public native boolean contentEquals(CharSequence other);
    public native boolean regionMatches(int at, String other, int from, int count);
    public native java.util.stream.Stream<String> lines();
    public native String intern();
    public native String formatted(Object... parts);
    public native java.util.stream.IntStream chars();
    public static native String valueOf(int value);
    public static native String valueOf(long value);
    public static native String valueOf(float value);
    public static native String valueOf(double value);
    public static native String valueOf(boolean value);
    public static native String valueOf(char value);
    public static native String valueOf(char[] value);
    public static native String valueOf(char[] value, int offset, int count);
    public static native String valueOf(Object value);
    public static native String format(String pattern, Object... parts);
    public static native String join(CharSequence between, CharSequence... parts);
    public static native String join(CharSequence between, Iterable<CharSequence> parts);
    public static native String copyValueOf(char[] value);
}

public class StringBuilder implements CharSequence, Appendable {
    public native StringBuilder();
    public native StringBuilder(int capacity);
    public native StringBuilder(String held);
    public native StringBuilder(CharSequence held);
    public native StringBuilder append(String held);
    public native StringBuilder append(CharSequence held);
    public native StringBuilder append(Object held);
    public native StringBuilder append(int held);
    public native StringBuilder append(long held);
    public native StringBuilder append(float held);
    public native StringBuilder append(double held);
    public native StringBuilder append(boolean held);
    public native StringBuilder append(char held);
    public native StringBuilder append(char[] held);
    public native StringBuilder insert(int at, String held);
    public native StringBuilder insert(int at, Object held);
    public native StringBuilder insert(int at, char held);
    public native StringBuilder insert(int at, int held);
    public native StringBuilder delete(int from, int to);
    public native StringBuilder deleteCharAt(int at);
    public native StringBuilder replace(int from, int to, String held);
    public native StringBuilder reverse();
    public native void setLength(int held);
    public native int length();
    public native char charAt(int index);
    public native void setCharAt(int index, char held);
    public native int indexOf(String held);
    public native int lastIndexOf(String held);
    public native CharSequence subSequence(int start, int end);
    public native String substring(int from);
    public native String substring(int from, int to);
    public native String toString();
}

public class StringBuffer implements CharSequence, Appendable {
    public native StringBuffer();
    public native StringBuffer(String held);
    public native StringBuffer append(String held);
    public native StringBuffer append(CharSequence held);
    public native StringBuffer append(Object held);
    public native StringBuffer append(int held);
    public native StringBuffer append(char held);
    public native int length();
    public native char charAt(int index);
    public native CharSequence subSequence(int start, int end);
    public native String toString();
}

public class Number {
    public native int intValue();
    public native long longValue();
    public native float floatValue();
    public native double doubleValue();
    public native byte byteValue();
    public native short shortValue();
}

public class Integer extends Number implements Comparable<Integer> {
    public static final int MAX_VALUE = 2147483647;
    public static final int MIN_VALUE = -2147483648;
    public static final int SIZE = 32;
    public static final int BYTES = 4;
    public native Integer(int value) throws java.lang.NumberFormatException ;
    public native int intValue();
    public native int compareTo(Integer other);
    public static native Integer valueOf(int value) throws java.lang.NumberFormatException ;
    public static native Integer valueOf(String held) throws java.lang.NumberFormatException ;
    public static native int parseInt(String held) throws java.lang.NumberFormatException ;
    public static native int parseInt(String held, int radix) throws java.lang.NumberFormatException ;
    public static native String toString(int value);
    public static native String toString(int value, int radix);
    public static native String toBinaryString(int value);
    public static native String toHexString(int value);
    public static native String toOctalString(int value);
    public static native int compare(int left, int right);
    public static native int hashCode(int value);
    public static native int max(int left, int right);
    public static native int min(int left, int right);
    public static native int sum(int left, int right);
    public static native int signum(int value);
    public static native int bitCount(int value);
    public static native int highestOneBit(int value);
    public static native int lowestOneBit(int value);
    public static native int numberOfLeadingZeros(int value);
    public static native int numberOfTrailingZeros(int value);
    public static native int reverse(int value);
    public static native int reverseBytes(int value);
    public static native int rotateLeft(int value, int by);
    public static native int rotateRight(int value, int by);
    public static native int divideUnsigned(int left, int right);
    public static native int remainderUnsigned(int left, int right);
    public static native long toUnsignedLong(int value);
    public static native String toUnsignedString(int value);
    public static native int decode(String held) throws java.lang.NumberFormatException ;
}

public class Long extends Number implements Comparable<Long> {
    public static final long MAX_VALUE;
    public static final long MIN_VALUE;
    public native Long(long value) throws java.lang.NumberFormatException ;
    public native long longValue();
    public native int compareTo(Long other);
    public static native Long valueOf(long value) throws java.lang.NumberFormatException ;
    public static native long parseLong(String held) throws java.lang.NumberFormatException ;
    public static native String toString(long value);
    public static native String toBinaryString(long value);
    public static native String toHexString(long value);
    public static native int compare(long left, long right);
    public static native int hashCode(long value);
    public static native long max(long left, long right);
    public static native long min(long left, long right);
    public static native long sum(long left, long right);
    public static native int signum(long value);
    public static native int bitCount(long value);
    public static native long highestOneBit(long value);
    public static native long lowestOneBit(long value);
    public static native int numberOfLeadingZeros(long value);
    public static native int numberOfTrailingZeros(long value);
    public static native long reverse(long value);
    public static native long reverseBytes(long value);
    public static native long rotateLeft(long value, int by);
    public static native long rotateRight(long value, int by);
    public static native String toOctalString(long value);
    public static native long parseLong(String held, int radix) throws java.lang.NumberFormatException ;
    public static native String toString(long value, int radix);
    public static native Long valueOf(String held) throws java.lang.NumberFormatException ;
}

public class Short extends Number implements Comparable<Short> {
    public static final short MAX_VALUE = 32767;
    public static final short MIN_VALUE = -32768;
    public native Short(short value) throws java.lang.NumberFormatException ;
    public native short shortValue();
    public native int compareTo(Short other);
    public static native Short valueOf(short value) throws java.lang.NumberFormatException ;
    public static native short parseShort(String held) throws java.lang.NumberFormatException ;
    public static native int compare(short left, short right);
    public static native int hashCode(short value);
    public static native String toString(short value);
}

public class Byte extends Number implements Comparable<Byte> {
    public static final byte MAX_VALUE = 127;
    public static final byte MIN_VALUE = -128;
    public native Byte(byte value) throws java.lang.NumberFormatException ;
    public native byte byteValue();
    public native int compareTo(Byte other);
    public static native Byte valueOf(byte value) throws java.lang.NumberFormatException ;
    public static native byte parseByte(String held) throws java.lang.NumberFormatException ;
    public static native int compare(byte left, byte right);
    public static native int hashCode(byte value);
    public static native String toString(byte value);
}

public class Float extends Number implements Comparable<Float> {
    public static final float MAX_VALUE;
    public static final float MIN_VALUE;
    public static final float NaN;
    public static final float POSITIVE_INFINITY;
    public static final float NEGATIVE_INFINITY;
    public native Float(float value) throws java.lang.NumberFormatException ;
    public native float floatValue();
    public native int compareTo(Float other);
    public native boolean isNaN();
    public static native Float valueOf(float value) throws java.lang.NumberFormatException ;
    public static native float parseFloat(String held) throws java.lang.NumberFormatException ;
    public static native int compare(float left, float right);
    public static native int hashCode(float value);
    public static native String toString(float value);
    public static native int floatToIntBits(float value);
    public static native float intBitsToFloat(int value);
}

public class Double extends Number implements Comparable<Double> {
    public static final double MAX_VALUE;
    public static final double MIN_VALUE;
    public static final double NaN;
    public static final double POSITIVE_INFINITY;
    public static final double NEGATIVE_INFINITY;
    public native Double(double value) throws java.lang.NumberFormatException ;
    public native double doubleValue();
    public native int compareTo(Double other);
    public native boolean isNaN();
    public native boolean isInfinite();
    public static native Double valueOf(double value) throws java.lang.NumberFormatException ;
    public static native double parseDouble(String held) throws java.lang.NumberFormatException ;
    public static native int compare(double left, double right);
    public static native int hashCode(double value);
    public static native String toString(double value);
    public static native boolean isNaN(double value);
    public static native long doubleToLongBits(double value);
    public static native double longBitsToDouble(long value);
}

public class Boolean implements Comparable<Boolean> {
    public static final Boolean TRUE;
    public static final Boolean FALSE;
    public native Boolean(boolean value);
    public native boolean booleanValue();
    public native int compareTo(Boolean other);
    public static native Boolean valueOf(boolean value);
    public static native boolean parseBoolean(String held);
    public static native int compare(boolean left, boolean right);
    public static native int hashCode(boolean value);
    public static native String toString(boolean value);
    public static native boolean logicalAnd(boolean left, boolean right);
    public static native boolean logicalOr(boolean left, boolean right);
}

public class Character implements Comparable<Character> {
    public static final char MAX_VALUE = '\uffff';
    public static final char MIN_VALUE = '\u0000';
    public native Character(char value);
    public native char charValue();
    public native int compareTo(Character other);
    public static native Character valueOf(char value);
    public static native boolean isDigit(char value);
    public static native boolean isLetter(char value);
    public static native boolean isLetterOrDigit(char value);
    public static native boolean isWhitespace(char value);
    public static native boolean isUpperCase(char value);
    public static native boolean isLowerCase(char value);
    public static native char toUpperCase(char value);
    public static native char toLowerCase(char value);
    public static native int compare(char left, char right);
    public static native int hashCode(char value);
    public static native String toString(char value);
    public static native int getNumericValue(char value);
}

public class Math {
    public static final double PI;
    public static final double E;
    public static native int abs(int value);
    public static native long abs(long value);
    public static native float abs(float value);
    public static native double abs(double value);
    public static native int max(int left, int right);
    public static native long max(long left, long right);
    public static native float max(float left, float right);
    public static native double max(double left, double right);
    public static native int min(int left, int right);
    public static native long min(long left, long right);
    public static native float min(float left, float right);
    public static native double min(double left, double right);
    public static native double pow(double value, double by);
    public static native double sqrt(double value);
    public static native double cbrt(double value);
    public static native double floor(double value);
    public static native double ceil(double value);
    public static native double rint(double value);
    public static native long round(double value);
    public static native int round(float value);
    public static native double random();
    public static native double log(double value);
    public static native double log10(double value);
    public static native double exp(double value);
    public static native double sin(double value);
    public static native double cos(double value);
    public static native double tan(double value);
    public static native double atan2(double y, double x);
    public static native double hypot(double x, double y);
    public static native double toRadians(double value);
    public static native double toDegrees(double value);
    public static native int floorDiv(int left, int right);
    public static native int floorMod(int left, int right);
    public static native long floorDiv(long left, long right);
    public static native long floorMod(long left, long right);
    public static native int addExact(int left, int right);
    public static native long addExact(long left, long right);
    public static native int subtractExact(int left, int right);
    public static native long subtractExact(long left, long right);
    public static native int multiplyExact(int left, int right);
    public static native long multiplyExact(long left, long right);
    public static native int negateExact(int value);
    public static native int toIntExact(long value);
    public static native int incrementExact(int value);
    public static native int decrementExact(int value);
    public static native double signum(double value);
    public static native double ulp(double value);
    public static native double nextUp(double value);
    public static native double nextDown(double value);
    public static native double copySign(double value, double sign);
    public static native double asin(double value);
    public static native double acos(double value);
    public static native double atan(double value);
    public static native double sinh(double value);
    public static native double cosh(double value);
    public static native double tanh(double value);
    public static native double expm1(double value);
    public static native double log1p(double value);
    public static native double fma(double a, double b, double c);
    public static native int clamp(int value, int low, int high);
    public static native long clamp(long value, long low, long high);
    public static native double clamp(double value, double low, double high);
}

public class System {
    public static final java.io.PrintStream out;
    public static final java.io.PrintStream err;
    public static final java.io.InputStream in;
    public static native long currentTimeMillis();
    public static native long nanoTime();
    public static native void arraycopy(Object from, int at, Object into, int to, int count);
    public static native String getProperty(String name);
    public static native String getProperty(String name, String fallback);
    public static native String getenv(String name);
    public static native void exit(int code);
    public static native void gc();
    public static native int identityHashCode(Object held);
    public static native String lineSeparator();
}

public class Class<T> {
    public native String getName();
    public native String getSimpleName();
    public native String getCanonicalName();
    public native boolean isInstance(Object held);
    public native boolean isInterface();
    public native boolean isArray();
    public native boolean isEnum();
    public native boolean isRecord();
    public native boolean isPrimitive();
    public native Class getSuperclass();
    public native Class[] getInterfaces();
    public native T newInstance() throws java.lang.InstantiationException, java.lang.IllegalAccessException ;
    public native java.lang.reflect.Method getDeclaredMethod(String name, Class... taken) throws java.lang.NoSuchMethodException ;
    public native java.lang.reflect.Method getMethod(String name, Class... taken) throws java.lang.NoSuchMethodException ;
    public native java.lang.reflect.Method[] getDeclaredMethods();
    public native java.lang.reflect.Method[] getMethods();
    public native java.lang.reflect.Field getDeclaredField(String name) throws java.lang.NoSuchFieldException ;
    public native java.lang.reflect.Field getField(String name) throws java.lang.NoSuchFieldException ;
    public native java.lang.reflect.Field[] getDeclaredFields();
    public native java.lang.reflect.Field[] getFields();
    public native java.lang.reflect.Constructor getDeclaredConstructor(Class... taken) throws java.lang.NoSuchMethodException ;
    public native <A extends java.lang.annotation.Annotation> A getAnnotation(Class<A> held);
    public native <A extends java.lang.annotation.Annotation> A getDeclaredAnnotation(Class<A> held);
    public native <A extends java.lang.annotation.Annotation> A[] getAnnotationsByType(Class<A> held);
    public native java.lang.annotation.Annotation[] getAnnotations();
    public native boolean isAnnotationPresent(Class held);
    public native T[] getEnumConstants();
    public native java.lang.reflect.RecordComponent[] getRecordComponents();
    public native T cast(Object one);
    public native <U> Class<? extends U> asSubclass(Class<U> held);
    public static native Class forName(String name) throws java.lang.ClassNotFoundException ;
}

public class Enum<E> implements Comparable<E> {
    public native String name();
    public native int ordinal();
    public native int compareTo(E other);
    public native Class getDeclaringClass();
    public static native <T extends Enum<T>> T valueOf(Class<T> held, String name);
}

public class Record { }

public class Void { }

public class Thread implements Runnable {
    public native Thread();
    public native Thread(Runnable job);
    public native Thread(Runnable job, String name);
    public native void start();
    public native void run();
    public native void join() throws java.lang.InterruptedException ;
    public native void join(long millis) throws java.lang.InterruptedException ;
    public native void interrupt();
    public native boolean isAlive();
    public native boolean isInterrupted();
    public native void setName(String name);
    public native String getName();
    public native void setDaemon(boolean held);
    public native void setPriority(int held);
    public native long getId();
    public static native void sleep(long millis) throws java.lang.InterruptedException ;
    public static native Thread currentThread();
    public static native void yield();
}

public class Throwable {
    public native Throwable();
    public native Throwable(String message);
    public native Throwable(String message, Throwable cause);
    public native Throwable(Throwable cause);
    public native String getMessage();
    public native String getLocalizedMessage();
    public native Throwable getCause();
    public native Throwable initCause(Throwable cause);
    public native void printStackTrace();
    public native StackTraceElement[] getStackTrace();
    public native void addSuppressed(Throwable held);
    public native Throwable[] getSuppressed();
}

public class StackTraceElement {
    public native String getClassName();
    public native String getMethodName();
    public native String getFileName();
    public native int getLineNumber();
}

public class Exception extends Throwable {
    public native Exception();
    public native Exception(String message);
    public native Exception(String message, Throwable cause);
    public native Exception(Throwable cause);
}

public class RuntimeException extends Exception {
    public native RuntimeException();
    public native RuntimeException(String message);
    public native RuntimeException(String message, Throwable cause);
    public native RuntimeException(Throwable cause);
}

public class Error extends Throwable {
    public native Error();
    public native Error(String message);
    public native Error(String message, Throwable cause);
}

public class IllegalArgumentException extends RuntimeException {
    public native IllegalArgumentException();
    public native IllegalArgumentException(String message);
    public native IllegalArgumentException(String message, Throwable cause);
    public native IllegalArgumentException(Throwable cause);
}

public class IllegalStateException extends RuntimeException {
    public native IllegalStateException();
    public native IllegalStateException(String message);
    public native IllegalStateException(String message, Throwable cause);
}

public class NullPointerException extends RuntimeException {
    public native NullPointerException();
    public native NullPointerException(String message);
}

public class ArithmeticException extends RuntimeException {
    public native ArithmeticException();
    public native ArithmeticException(String message);
}

public class IndexOutOfBoundsException extends RuntimeException {
    public native IndexOutOfBoundsException();
    public native IndexOutOfBoundsException(String message);
}

public class ArrayIndexOutOfBoundsException extends IndexOutOfBoundsException {
    public native ArrayIndexOutOfBoundsException();
    public native ArrayIndexOutOfBoundsException(String message);
}

public class StringIndexOutOfBoundsException extends IndexOutOfBoundsException {
    public native StringIndexOutOfBoundsException();
    public native StringIndexOutOfBoundsException(String message);
}

public class ClassCastException extends RuntimeException {
    public native ClassCastException();
    public native ClassCastException(String message);
}

public class NumberFormatException extends IllegalArgumentException {
    public native NumberFormatException();
    public native NumberFormatException(String message);
}

public class UnsupportedOperationException extends RuntimeException {
    public native UnsupportedOperationException();
    public native UnsupportedOperationException(String message);
}

public class InterruptedException extends Exception {
    public native InterruptedException();
    public native InterruptedException(String message);
}

public class CloneNotSupportedException extends Exception {
    public native CloneNotSupportedException();
    public native CloneNotSupportedException(String message);
}

public class ClassNotFoundException extends Exception {
    public native ClassNotFoundException();
    public native ClassNotFoundException(String message);
}

public class NoSuchMethodException extends Exception {
    public native NoSuchMethodException();
    public native NoSuchMethodException(String message);
}

public class NoSuchFieldException extends Exception {
    public native NoSuchFieldException();
    public native NoSuchFieldException(String message);
}

public class SecurityException extends RuntimeException {
    public native SecurityException();
    public native SecurityException(String message);
}

public class AssertionError extends Error {
    public native AssertionError();
    public native AssertionError(Object message);
}

public class OutOfMemoryError extends Error {
    public native OutOfMemoryError();
    public native OutOfMemoryError(String message);
}

public class StackOverflowError extends Error {
    public native StackOverflowError();
}

public class IncompatibleClassChangeError extends Error {
    public native IncompatibleClassChangeError();
    public native IncompatibleClassChangeError(String message);
}

public class MatchException extends RuntimeException {
    public native MatchException(String message, Throwable cause);
}

public class ArrayStoreException extends RuntimeException {
    public native ArrayStoreException();
    public native ArrayStoreException(String message);
}

public class NegativeArraySizeException extends RuntimeException {
    public native NegativeArraySizeException();
    public native NegativeArraySizeException(String message);
}

// The annotations the language itself writes. `@Override` and
// `@SuppressWarnings` are read by the compiler and go no further, which is
// what SOURCE retention means; the rest are written into the class file so
// that reflection can see them.

public @interface Override { }

public @interface SuppressWarnings {
    String[] value();
}

@java.lang.annotation.Retention(java.lang.annotation.RetentionPolicy.RUNTIME)
public @interface Deprecated { }

@java.lang.annotation.Retention(java.lang.annotation.RetentionPolicy.RUNTIME)
public @interface FunctionalInterface { }

@java.lang.annotation.Retention(java.lang.annotation.RetentionPolicy.RUNTIME)
public @interface SafeVarargs { }
"####,
    ),
    (
        "java.lang.annotation",
        r####"
package java.lang.annotation;

public interface Annotation {
    Class annotationType();
}

public class RetentionPolicy extends java.lang.Enum<RetentionPolicy> {
    public static final RetentionPolicy SOURCE;
    public static final RetentionPolicy CLASS;
    public static final RetentionPolicy RUNTIME;
    public static native RetentionPolicy valueOf(String name);
    public static native RetentionPolicy[] values();
}

public class ElementType extends java.lang.Enum<ElementType> {
    public static final ElementType TYPE;
    public static final ElementType FIELD;
    public static final ElementType METHOD;
    public static final ElementType PARAMETER;
    public static final ElementType CONSTRUCTOR;
    public static final ElementType LOCAL_VARIABLE;
    public static final ElementType ANNOTATION_TYPE;
    public static final ElementType PACKAGE;
    public static final ElementType TYPE_PARAMETER;
    public static final ElementType TYPE_USE;
    public static final ElementType RECORD_COMPONENT;
    public static native ElementType valueOf(String name);
    public static native ElementType[] values();
}

@Retention(RetentionPolicy.RUNTIME)
public @interface Retention {
    RetentionPolicy value();
}

@Retention(RetentionPolicy.RUNTIME)
public @interface Target {
    ElementType[] value();
}

@Retention(RetentionPolicy.RUNTIME)
public @interface Documented { }

@Retention(RetentionPolicy.RUNTIME)
public @interface Inherited { }

@Retention(RetentionPolicy.RUNTIME)
public @interface Repeatable {
    Class value();
}
"####,
    ),
    (
        "java.lang.reflect",
        r####"
package java.lang.reflect;

public interface Member {
    String getName();
    int getModifiers();
}

public class AccessibleObject {
    public native void setAccessible(boolean held);
    public native boolean isAccessible();
    public native <T extends java.lang.annotation.Annotation> T getAnnotation(Class<T> held);
    public native <T extends java.lang.annotation.Annotation> T getDeclaredAnnotation(Class<T> held);
    public native java.lang.annotation.Annotation[] getAnnotations();
    public native boolean isAnnotationPresent(Class held);
}

public class Method extends AccessibleObject implements Member {
    public native String getName();
    public native int getModifiers();
    public native Class getReturnType();
    public native Class[] getParameterTypes();
    public native int getParameterCount();
    public native Object invoke(Object on, Object... parts) throws java.lang.IllegalAccessException, java.lang.reflect.InvocationTargetException ;
    public native Class getDeclaringClass();
    public native Object getDefaultValue();
    public native boolean isVarArgs();
    public native boolean isSynthetic();
    public native boolean isBridge();
    public native java.lang.annotation.Annotation[][] getParameterAnnotations();
}

public class Field extends AccessibleObject implements Member {
    public native String getName();
    public native int getModifiers();
    public native Class getType();
    public native Object get(Object on) throws java.lang.IllegalArgumentException, java.lang.IllegalAccessException ;
    public native void set(Object on, Object value) throws java.lang.IllegalArgumentException, java.lang.IllegalAccessException ;
    public native int getInt(Object on) throws java.lang.IllegalArgumentException, java.lang.IllegalAccessException ;
    public native Class getDeclaringClass();
}

public class Constructor extends AccessibleObject implements Member {
    public native String getName();
    public native int getModifiers();
    public native Class[] getParameterTypes();
    public native Object newInstance(Object... parts) throws java.lang.InstantiationException, java.lang.IllegalAccessException, java.lang.IllegalArgumentException, java.lang.reflect.InvocationTargetException ;
}

public class RecordComponent {
    public native String getName();
    public native Class getType();
    public native Method getAccessor();
}

public class Modifier {
    public static final int PUBLIC = 1;
    public static final int PRIVATE = 2;
    public static final int PROTECTED = 4;
    public static final int STATIC = 8;
    public static final int FINAL = 16;
    public static final int ABSTRACT = 1024;
    public static final int NATIVE = 256;
    public static final int SYNCHRONIZED = 32;
    public static native boolean isPublic(int flags);
    public static native boolean isPrivate(int flags);
    public static native boolean isProtected(int flags);
    public static native boolean isStatic(int flags);
    public static native boolean isFinal(int flags);
    public static native boolean isAbstract(int flags);
    public static native boolean isNative(int flags);
    public static native String toString(int flags);
}

public class Array {
    public static native Object newInstance(Class held, int length) throws java.lang.NegativeArraySizeException ;
    public static native int getLength(Object held) throws java.lang.IllegalArgumentException ;
    public static native Object get(Object held, int at) throws java.lang.IllegalArgumentException, java.lang.ArrayIndexOutOfBoundsException ;
    public static native void set(Object held, int at, Object value) throws java.lang.IllegalArgumentException, java.lang.ArrayIndexOutOfBoundsException ;
}

public class InvocationTargetException extends java.lang.Exception {
    public native Throwable getTargetException();
    public native Throwable getCause();
}
"####,
    ),
    (
        "java.io",
        r####"
package java.io;

public interface Serializable { }

public interface Closeable extends java.lang.AutoCloseable {
    void close() throws java.io.IOException ;
}

public interface Flushable {
    void flush() throws java.io.IOException ;
}

public class IOException extends java.lang.Exception {
    public native IOException();
    public native IOException(String message);
    public native IOException(String message, Throwable cause);
    public native IOException(Throwable cause);
}

public class FileNotFoundException extends IOException {
    public native FileNotFoundException();
    public native FileNotFoundException(String message);
}

public class UncheckedIOException extends java.lang.RuntimeException {
    public native UncheckedIOException(IOException cause);
    public native UncheckedIOException(String message, IOException cause);
}

public class InputStream implements Closeable {
    public native int read() throws java.io.IOException ;
    public native int read(byte[] into) throws java.io.IOException ;
    public native int read(byte[] into, int at, int count) throws java.io.IOException ;
    public native byte[] readAllBytes() throws java.io.IOException ;
    public native long skip(long count) throws java.io.IOException ;
    public native int available() throws java.io.IOException ;
    public native void close() throws java.io.IOException ;
}

public class OutputStream implements Closeable, Flushable {
    public native void write(int one) throws java.io.IOException ;
    public native void write(byte[] held) throws java.io.IOException ;
    public native void write(byte[] held, int at, int count) throws java.io.IOException ;
    public native void flush() throws java.io.IOException ;
    public native void close() throws java.io.IOException ;
}

public class ByteArrayInputStream extends InputStream {
    public native ByteArrayInputStream(byte[] held);
}

public class ByteArrayOutputStream extends OutputStream {
    public native ByteArrayOutputStream();
    public native ByteArrayOutputStream(int size);
    public native byte[] toByteArray();
    public native int size();
    public native void reset();
    public native String toString();
}

public class FileInputStream extends InputStream {
    public native FileInputStream(String name) throws java.io.FileNotFoundException ;
    public native FileInputStream(File held) throws java.io.FileNotFoundException ;
}

public class FileOutputStream extends OutputStream {
    public native FileOutputStream(String name) throws java.io.FileNotFoundException ;
    public native FileOutputStream(String name, boolean append) throws java.io.FileNotFoundException ;
    public native FileOutputStream(File held) throws java.io.FileNotFoundException ;
}

public class BufferedInputStream extends InputStream {
    public native BufferedInputStream(InputStream from);
}

public class BufferedOutputStream extends OutputStream {
    public native BufferedOutputStream(OutputStream into);
}

public class PrintStream extends OutputStream {
    public native PrintStream(OutputStream into);
    public native void print(String held);
    public native void print(Object held);
    public native void print(int held);
    public native void print(long held);
    public native void print(float held);
    public native void print(double held);
    public native void print(boolean held);
    public native void print(char held);
    public native void print(char[] held);
    public native void println();
    public native void println(String held);
    public native void println(Object held);
    public native void println(int held);
    public native void println(long held);
    public native void println(float held);
    public native void println(double held);
    public native void println(boolean held);
    public native void println(char held);
    public native void println(char[] held);
    public native PrintStream printf(String pattern, Object... parts);
    public native PrintStream format(String pattern, Object... parts);
}

public class Reader implements Closeable {
    public native int read() throws java.io.IOException ;
    public native int read(char[] into) throws java.io.IOException ;
    public native void close() throws java.io.IOException ;
}

public class Writer implements Closeable, Flushable {
    public native void write(String held) throws java.io.IOException ;
    public native void write(int one) throws java.io.IOException ;
    public native void write(char[] held) throws java.io.IOException ;
    public native Writer append(CharSequence held) throws java.io.IOException ;
    public native void flush() throws java.io.IOException ;
    public native void close() throws java.io.IOException ;
}

public class InputStreamReader extends Reader {
    public native InputStreamReader(InputStream from);
    public native InputStreamReader(InputStream from, String charset) throws java.io.UnsupportedEncodingException ;
}

public class OutputStreamWriter extends Writer {
    public native OutputStreamWriter(OutputStream into);
}

public class BufferedReader extends Reader {
    public native BufferedReader(Reader from);
    public native String readLine() throws java.io.IOException ;
    public native java.util.stream.Stream<String> lines();
}

public class BufferedWriter extends Writer {
    public native BufferedWriter(Writer into);
    public native void newLine() throws java.io.IOException ;
}

public class FileReader extends InputStreamReader {
    public native FileReader(String name) throws java.io.FileNotFoundException ;
    public native FileReader(File held) throws java.io.FileNotFoundException ;
}

public class FileWriter extends OutputStreamWriter {
    public native FileWriter(String name) throws java.io.IOException ;
    public native FileWriter(String name, boolean append) throws java.io.IOException ;
    public native FileWriter(File held) throws java.io.IOException ;
}

public class StringWriter extends Writer {
    public native StringWriter();
    public native String toString();
}

public class StringReader extends Reader {
    public native StringReader(String held);
}

public class PrintWriter extends Writer {
    public native PrintWriter(Writer into);
    public native PrintWriter(OutputStream into);
    public native void print(String held);
    public native void print(Object held);
    public native void println(String held);
    public native void println(Object held);
    public native void println();
    public native PrintWriter printf(String pattern, Object... parts);
}

public class File implements Comparable<File> {
    public static final String separator;
    public native File(String path);
    public native File(String parent, String child);
    public native File(File parent, String child);
    public native String getName();
    public native String getPath();
    public native String getAbsolutePath();
    public native String getParent();
    public native File getParentFile();
    public native File getAbsoluteFile();
    public native boolean exists();
    public native boolean isFile();
    public native boolean isDirectory();
    public native boolean canRead();
    public native boolean canWrite();
    public native long length();
    public native long lastModified();
    public native boolean delete();
    public native boolean mkdir();
    public native boolean mkdirs();
    public native boolean createNewFile() throws java.io.IOException ;
    public native boolean renameTo(File into);
    public native String[] list();
    public native File[] listFiles();
    public native int compareTo(File other);
}
"####,
    ),
    (
        "java.util",
        r####"
package java.util;

public interface Iterator<E> {
    boolean hasNext();
    E next();
    default void remove() { }
    default void forEachRemaining(java.util.function.Consumer<E> what) { }
}

public interface ListIterator<E> extends Iterator<E> {
    boolean hasPrevious();
    E previous();
    int nextIndex();
    int previousIndex();
    void set(E one);
    void add(E one);
}

public interface Collection<E> extends java.lang.Iterable<E> {
    int size();
    boolean isEmpty();
    boolean contains(Object one);
    boolean add(E one);
    boolean remove(Object one);
    boolean containsAll(Collection<E> held);
    boolean addAll(Collection<E> held);
    boolean removeAll(Collection<E> held);
    boolean retainAll(Collection<E> held);
    void clear();
    Object[] toArray();
    <T> T[] toArray(T[] into);
    Iterator<E> iterator();
    default java.util.stream.Stream<E> stream() { return null; }
    default boolean removeIf(java.util.function.Predicate<E> when) { return false; }
    default void forEach(java.util.function.Consumer<E> what) { }
}

public interface List<E> extends Collection<E> {
    E get(int at);
    E set(int at, E one);
    void add(int at, E one);
    E remove(int at);
    int indexOf(Object one);
    int lastIndexOf(Object one);
    ListIterator<E> listIterator();
    List<E> subList(int from, int to);
    void sort(Comparator<E> by);
    void replaceAll(java.util.function.UnaryOperator<E> what);
    static <T> List<T> of();
    static <T> List<T> of(T one);
    static <T> List<T> of(T one, T two);
    static <T> List<T> of(T one, T two, T three);
    static <T> List<T> of(T one, T two, T three, T four);
    static <T> List<T> of(T one, T two, T three, T four, T five);
    static <T> List<T> of(T one, T two, T three, T four, T five, T six);
    static <T> List<T> of(T one, T two, T three, T four, T five, T six, T seven);
    static <T> List<T> of(T one, T two, T three, T four, T five, T six, T seven, T eight);
    static <T> List<T> of(T one, T two, T three, T four, T five, T six, T seven, T eight, T nine);
    static <T> List<T> of(T one, T two, T three, T four, T five, T six, T seven, T eight, T nine, T ten);
    static <T> List<T> of(T... held);
    static <T> List<T> copyOf(Collection<T> held);
}

public interface Set<E> extends Collection<E> {
    static <T> Set<T> of();
    static <T> Set<T> of(T one);
    static <T> Set<T> of(T one, T two);
    static <T> Set<T> of(T one, T two, T three);
    static <T> Set<T> of(T one, T two, T three, T four);
    static <T> Set<T> of(T one, T two, T three, T four, T five);
    static <T> Set<T> of(T one, T two, T three, T four, T five, T six);
    static <T> Set<T> of(T one, T two, T three, T four, T five, T six, T seven);
    static <T> Set<T> of(T one, T two, T three, T four, T five, T six, T seven, T eight);
    static <T> Set<T> of(T one, T two, T three, T four, T five, T six, T seven, T eight, T nine);
    static <T> Set<T> of(T one, T two, T three, T four, T five, T six, T seven, T eight, T nine, T ten);
    static <T> Set<T> of(T... held);
    static <T> Set<T> copyOf(Collection<T> held);
}

public interface SortedSet<E> extends Set<E> {
    E first();
    E last();
    SortedSet<E> headSet(E to);
    SortedSet<E> tailSet(E from);
    SortedSet<E> subSet(E from, E to);
    Comparator<E> comparator();
}

public interface NavigableSet<E> extends SortedSet<E> {
    E floor(E one);
    E ceiling(E one);
    E lower(E one);
    E higher(E one);
    E pollFirst();
    E pollLast();
    NavigableSet<E> descendingSet();
}

public interface Queue<E> extends Collection<E> {
    boolean offer(E one);
    E poll();
    E peek();
    E element();
}

public interface Deque<E> extends Queue<E> {
    void addFirst(E one);
    void addLast(E one);
    E pollFirst();
    E pollLast();
    E peekFirst();
    E peekLast();
    E removeFirst();
    E removeLast();
    void push(E one);
    E pop();
    Iterator<E> descendingIterator();
}

public interface Map<K, V> {
    public interface Entry<K, V> {
        K getKey();
        V getValue();
        V setValue(V one);
    }
    int size();
    boolean isEmpty();
    boolean containsKey(Object key);
    boolean containsValue(Object value);
    V get(Object key);
    V put(K key, V value);
    V remove(Object key);
    void clear();
    Set<K> keySet();
    Collection<V> values();
    Set<Entry<K, V>> entrySet();
    V getOrDefault(Object key, V fallback);
    V putIfAbsent(K key, V value);
    V computeIfAbsent(K key, java.util.function.Function<K, V> what);
    V computeIfPresent(K key, java.util.function.BiFunction<K, V, V> what);
    V compute(K key, java.util.function.BiFunction<K, V, V> what);
    V merge(K key, V value, java.util.function.BiFunction<V, V, V> what);
    void putAll(Map<K, V> held);
    void forEach(java.util.function.BiConsumer<K, V> what);
    static <A, B> Map<A, B> of();
    static <A, B> Map<A, B> of(A oneKey, B one);
    static <A, B> Map<A, B> of(A oneKey, B one, A twoKey, B two);
    static <A, B> Map<A, B> of(A oneKey, B one, A twoKey, B two, A threeKey, B three);
    static <A, B> Map<A, B> of(A oneKey, B one, A twoKey, B two, A threeKey, B three, A fourKey, B four);
    static <A, B> Map<A, B> of(A oneKey, B one, A twoKey, B two, A threeKey, B three, A fourKey, B four, A fiveKey, B five);
    static <A, B> Map<A, B> of(A oneKey, B one, A twoKey, B two, A threeKey, B three, A fourKey, B four, A fiveKey, B five, A sixKey, B six);
    static <A, B> Map<A, B> of(A oneKey, B one, A twoKey, B two, A threeKey, B three, A fourKey, B four, A fiveKey, B five, A sixKey, B six, A sevenKey, B seven);
    static <A, B> Map<A, B> of(A oneKey, B one, A twoKey, B two, A threeKey, B three, A fourKey, B four, A fiveKey, B five, A sixKey, B six, A sevenKey, B seven, A eightKey, B eight);
    static <A, B> Map<A, B> of(A oneKey, B one, A twoKey, B two, A threeKey, B three, A fourKey, B four, A fiveKey, B five, A sixKey, B six, A sevenKey, B seven, A eightKey, B eight, A nineKey, B nine);
    static <A, B> Map<A, B> of(A oneKey, B one, A twoKey, B two, A threeKey, B three, A fourKey, B four, A fiveKey, B five, A sixKey, B six, A sevenKey, B seven, A eightKey, B eight, A nineKey, B nine, A tenKey, B ten);
    static <A, B> Entry<A, B> entry(A key, B value);
    static <A, B> Map<A, B> ofEntries(Entry<A, B>... held);
    static <A, B> Map<A, B> copyOf(Map<A, B> held);
}

public interface SortedMap<K, V> extends Map<K, V> {
    K firstKey();
    K lastKey();
    SortedMap<K, V> headMap(K to);
    SortedMap<K, V> tailMap(K from);
    Comparator<K> comparator();
}

public interface NavigableMap<K, V> extends SortedMap<K, V> {
    K floorKey(K key);
    K ceilingKey(K key);
    K lowerKey(K key);
    K higherKey(K key);
    Map.Entry<K, V> firstEntry();
    Map.Entry<K, V> lastEntry();
    Map.Entry<K, V> pollFirstEntry();
    Map.Entry<K, V> pollLastEntry();
    NavigableMap<K, V> descendingMap();
}

public interface Comparator<T> {
    int compare(T left, T right);
    default Comparator<T> reversed() { return null; }
    default Comparator<T> thenComparing(Comparator<T> then) { return null; }
    default <V> Comparator<T> thenComparing(java.util.function.Function<T, V> by) { return null; }
    static <X> Comparator<X> reverseOrder();
    default Comparator<T> thenComparingInt(java.util.function.ToIntFunction<T> by) { return null; }
    default Comparator<T> thenComparingLong(java.util.function.ToLongFunction<T> by) { return null; }
    default Comparator<T> thenComparingDouble(java.util.function.ToDoubleFunction<T> by) { return null; }
    static <X> Comparator<X> naturalOrder();
    static <X, Y> Comparator<X> comparing(java.util.function.Function<X, Y> by);
    static <X, Y> Comparator<X> comparing(java.util.function.Function<X, Y> by, Comparator<Y> then);
    static <X> Comparator<X> comparingInt(java.util.function.ToIntFunction<X> by);
    static <X> Comparator<X> comparingLong(java.util.function.ToLongFunction<X> by);
    static <X> Comparator<X> comparingDouble(java.util.function.ToDoubleFunction<X> by);
    static <X> Comparator<X> nullsFirst(Comparator<X> then);
    static <X> Comparator<X> nullsLast(Comparator<X> then);
}

public class AbstractCollection<E> implements Collection<E> { }

public class AbstractList<E> extends AbstractCollection<E> implements List<E> { }

public class ArrayList<E> extends AbstractList<E> implements List<E> {
    public native ArrayList();
    public native ArrayList(int capacity);
    public native ArrayList(Collection<E> held);
    public native int size();
    public native boolean isEmpty();
    public native boolean contains(Object one);
    public native boolean add(E one);
    public native void add(int at, E one);
    public native boolean remove(Object one);
    public native E remove(int at);
    public native E get(int at);
    public native E set(int at, E one);
    public native int indexOf(Object one);
    public native int lastIndexOf(Object one);
    public native void clear();
    public native boolean addAll(Collection<E> held);
    public native boolean removeAll(Collection<E> held);
    public native boolean retainAll(Collection<E> held);
    public native boolean containsAll(Collection<E> held);
    public native Object[] toArray();
    public native Iterator<E> iterator();
    public native ListIterator<E> listIterator();
    public native List<E> subList(int from, int to);
    public native void sort(Comparator<E> by);
    public native void replaceAll(java.util.function.UnaryOperator<E> what);
    public native boolean removeIf(java.util.function.Predicate<E> when);
    public native void forEach(java.util.function.Consumer<E> what);
    public native java.util.stream.Stream<E> stream();
    public native void ensureCapacity(int held);
    public native void trimToSize();
}

public class LinkedList<E> extends AbstractList<E> implements List<E>, Deque<E> {
    public native LinkedList();
    public native LinkedList(Collection<E> held);
    public native int size();
    public native boolean isEmpty();
    public native boolean contains(Object one);
    public native boolean add(E one);
    public native void add(int at, E one);
    public native boolean remove(Object one);
    public native E remove(int at);
    public native E get(int at);
    public native E set(int at, E one);
    public native int indexOf(Object one);
    public native void clear();
    public native boolean addAll(Collection<E> held);
    public native Iterator<E> iterator();
    public native Object[] toArray();
    public native java.util.stream.Stream<E> stream();
    public native boolean offer(E one);
    public native E poll();
    public native E peek();
    public native E element();
    public native void addFirst(E one);
    public native void addLast(E one);
    public native E pollFirst();
    public native E pollLast();
    public native E peekFirst();
    public native E peekLast();
    public native E removeFirst();
    public native E removeLast();
    public native E getFirst();
    public native E getLast();
    public native void push(E one);
    public native E pop();
    public native Iterator<E> descendingIterator();
}

public class ArrayDeque<E> extends AbstractCollection<E> implements Deque<E> {
    public native ArrayDeque();
    public native ArrayDeque(int capacity);
    public native ArrayDeque(Collection<E> held);
    public native int size();
    public native boolean isEmpty();
    public native boolean add(E one);
    public native void clear();
    public native Iterator<E> iterator();
    public native boolean offer(E one);
    public native E poll();
    public native E peek();
    public native E element();
    public native void addFirst(E one);
    public native void addLast(E one);
    public native E pollFirst();
    public native E pollLast();
    public native E peekFirst();
    public native E peekLast();
    public native E removeFirst();
    public native E removeLast();
    public native void push(E one);
    public native E pop();
    public native Iterator<E> descendingIterator();
    public native java.util.stream.Stream<E> stream();
}

public class Vector<E> extends AbstractList<E> implements List<E> {
    public native Vector();
    public native int size();
    public native boolean add(E one);
    public native E get(int at);
    public native Iterator<E> iterator();
}

public class Stack<E> extends Vector<E> {
    public native Stack();
    public native E push(E one);
    public native E pop();
    public native E peek();
    public native boolean empty();
    public native int search(Object one);
}

public class AbstractSet<E> extends AbstractCollection<E> implements Set<E> { }

public class HashSet<E> extends AbstractSet<E> implements Set<E> {
    public native HashSet();
    public native HashSet(int capacity);
    public native HashSet(Collection<E> held);
    public native int size();
    public native boolean isEmpty();
    public native boolean contains(Object one);
    public native boolean add(E one);
    public native boolean remove(Object one);
    public native void clear();
    public native boolean addAll(Collection<E> held);
    public native boolean removeAll(Collection<E> held);
    public native boolean retainAll(Collection<E> held);
    public native boolean containsAll(Collection<E> held);
    public native Iterator<E> iterator();
    public native Object[] toArray();
    public native java.util.stream.Stream<E> stream();
    public native boolean removeIf(java.util.function.Predicate<E> when);
    public native void forEach(java.util.function.Consumer<E> what);
}

public class LinkedHashSet<E> extends HashSet<E> {
    public native LinkedHashSet();
    public native LinkedHashSet(int capacity);
    public native LinkedHashSet(Collection<E> held);
}

public class TreeSet<E> extends AbstractSet<E> implements NavigableSet<E> {
    public native TreeSet();
    public native TreeSet(Comparator<E> by);
    public native TreeSet(Collection<E> held);
    public native int size();
    public native boolean isEmpty();
    public native boolean contains(Object one);
    public native boolean add(E one);
    public native boolean remove(Object one);
    public native void clear();
    public native Iterator<E> iterator();
    public native java.util.stream.Stream<E> stream();
    public native E first();
    public native E last();
    public native E floor(E one);
    public native E ceiling(E one);
    public native E lower(E one);
    public native E higher(E one);
    public native E pollFirst();
    public native E pollLast();
    public native NavigableSet<E> descendingSet();
}

public class AbstractMap<K, V> implements Map<K, V> { }

public class HashMap<K, V> extends AbstractMap<K, V> implements Map<K, V> {
    public native HashMap();
    public native HashMap(int capacity);
    public native HashMap(Map<K, V> held);
    public native int size();
    public native boolean isEmpty();
    public native boolean containsKey(Object key);
    public native boolean containsValue(Object value);
    public native V get(Object key);
    public native V put(K key, V value);
    public native V remove(Object key);
    public native void clear();
    public native Set<K> keySet();
    public native Collection<V> values();
    public native Set<Map.Entry<K, V>> entrySet();
    public native V getOrDefault(Object key, V fallback);
    public native V putIfAbsent(K key, V value);
    public native V computeIfAbsent(K key, java.util.function.Function<K, V> what);
    public native V computeIfPresent(K key, java.util.function.BiFunction<K, V, V> what);
    public native V compute(K key, java.util.function.BiFunction<K, V, V> what);
    public native V merge(K key, V value, java.util.function.BiFunction<V, V, V> what);
    public native void putAll(Map<K, V> held);
    public native void forEach(java.util.function.BiConsumer<K, V> what);
}

public class LinkedHashMap<K, V> extends HashMap<K, V> {
    public native LinkedHashMap();
    public native LinkedHashMap(int capacity);
    public native LinkedHashMap(Map<K, V> held);
}

public class TreeMap<K, V> extends AbstractMap<K, V> implements NavigableMap<K, V> {
    public native TreeMap();
    public native TreeMap(Comparator<K> by);
    public native TreeMap(Map<K, V> held);
    public native int size();
    public native boolean isEmpty();
    public native boolean containsKey(Object key);
    public native boolean containsValue(Object value);
    public native V get(Object key);
    public native V put(K key, V value);
    public native V remove(Object key);
    public native void clear();
    public native Set<K> keySet();
    public native Collection<V> values();
    public native Set<Map.Entry<K, V>> entrySet();
    public native V getOrDefault(Object key, V fallback);
    public native void putAll(Map<K, V> held);
    public native void forEach(java.util.function.BiConsumer<K, V> what);
    public native K firstKey();
    public native K lastKey();
    public native K floorKey(K key);
    public native K ceilingKey(K key);
    public native K lowerKey(K key);
    public native K higherKey(K key);
    public native Map.Entry<K, V> firstEntry();
    public native Map.Entry<K, V> lastEntry();
    public native Map.Entry<K, V> pollFirstEntry();
    public native Map.Entry<K, V> pollLastEntry();
    public native NavigableMap<K, V> descendingMap();
    public native SortedMap<K, V> headMap(K to);
    public native SortedMap<K, V> tailMap(K from);
}

public class EnumMap<K extends java.lang.Enum<K>, V> extends AbstractMap<K, V> implements Map<K, V> {
    public native EnumMap(java.lang.Class held);
    public native EnumMap(Map<K, V> held);
    public native int size();
    public native boolean isEmpty();
    public native boolean containsKey(Object key);
    public native V get(Object key);
    public native V put(K key, V value);
    public native V remove(Object key);
    public native void clear();
    public native Set<K> keySet();
    public native Collection<V> values();
    public native Set<Map.Entry<K, V>> entrySet();
    public native void forEach(java.util.function.BiConsumer<K, V> what);
}

public class EnumSet<E extends java.lang.Enum<E>> extends AbstractSet<E> implements Set<E> {
    public static native <T extends java.lang.Enum<T>> EnumSet<T> noneOf(java.lang.Class held);
    public static native <T extends java.lang.Enum<T>> EnumSet<T> allOf(java.lang.Class held);
    public static native <T extends java.lang.Enum<T>> EnumSet<T> of(T one);
    public static native <T extends java.lang.Enum<T>> EnumSet<T> of(T one, T two);
    public static native <T extends java.lang.Enum<T>> EnumSet<T> of(T one, T two, T three);
    public static native <T extends java.lang.Enum<T>> EnumSet<T> of(T one, T two, T three, T four);
    public static native <T extends java.lang.Enum<T>> EnumSet<T> range(T from, T to);
    public static native <T extends java.lang.Enum<T>> EnumSet<T> complementOf(EnumSet<T> held);
    public static native <T extends java.lang.Enum<T>> EnumSet<T> copyOf(Collection<T> held);
}

public class Arrays {
    public static native <T> List<T> asList(T... held);
    public static native void sort(int[] held);
    public static native void sort(long[] held);
    public static native void sort(double[] held);
    public static native void sort(char[] held);
    public static native void sort(byte[] held);
    public static native void sort(Object[] held);
    public static native <T> void sort(T[] held, Comparator<T> by);
    public static native String toString(int[] held);
    public static native String toString(long[] held);
    public static native String toString(double[] held);
    public static native String toString(float[] held);
    public static native String toString(char[] held);
    public static native String toString(byte[] held);
    public static native String toString(short[] held);
    public static native String toString(boolean[] held);
    public static native String toString(Object[] held);
    public static native String deepToString(Object[] held);
    public static native boolean equals(int[] left, int[] right);
    public static native boolean equals(Object[] left, Object[] right);
    public static native boolean deepEquals(Object[] left, Object[] right);
    public static native int hashCode(int[] held);
    public static native int hashCode(Object[] held);
    public static native int[] copyOf(int[] held, int length);
    public static native long[] copyOf(long[] held, int length);
    public static native double[] copyOf(double[] held, int length);
    public static native char[] copyOf(char[] held, int length);
    public static native byte[] copyOf(byte[] held, int length);
    public static native boolean[] copyOf(boolean[] held, int length);
    public static native <T> T[] copyOf(T[] held, int length);
    public static native int[] copyOfRange(int[] held, int from, int to);
    public static native <T> T[] copyOfRange(T[] held, int from, int to);
    public static native void fill(int[] held, int with);
    public static native void fill(long[] held, long with);
    public static native void fill(double[] held, double with);
    public static native void fill(char[] held, char with);
    public static native void fill(boolean[] held, boolean with);
    public static native void fill(Object[] held, Object with);
    public static native int binarySearch(int[] held, int of);
    public static native int binarySearch(Object[] held, Object of);
    public static native java.util.stream.Stream stream(Object[] held);
    public static native java.util.stream.IntStream stream(int[] held);
}

public class Collections {
    public static native void sort(List held);
    public static native <T> void sort(List<T> held, Comparator<T> by);
    public static native void reverse(List held);
    public static native void shuffle(List held);
    public static native void swap(List held, int left, int right);
    public static native <T> T max(Collection<T> held);
    public static native <T> T max(Collection<T> held, Comparator<T> by);
    public static native <T> T min(Collection<T> held);
    public static native <T> T min(Collection<T> held, Comparator<T> by);
    public static native <T> List<T> unmodifiableList(List<T> held);
    public static native <T> Set<T> unmodifiableSet(Set<T> held);
    public static native <K, V> Map<K, V> unmodifiableMap(Map<K, V> held);
    public static native <T> List<T> emptyList();
    public static native <T> Set<T> emptySet();
    public static native <K, V> Map<K, V> emptyMap();
    public static native <T> List<T> singletonList(T one);
    public static native <T> Set<T> singleton(T one);
    public static native <T> boolean addAll(Collection<T> held, T... more);
    public static native <T> List<T> nCopies(int count, T one);
    public static native int frequency(Collection held, Object one);
    public static native <T> List<T> synchronizedList(List<T> held);
    public static native <K, V> Map<K, V> synchronizedMap(Map<K, V> held);
}

public class Objects {
    public static native boolean equals(Object left, Object right);
    public static native boolean deepEquals(Object left, Object right);
    public static native int hashCode(Object one);
    public static native int hash(Object... parts);
    public static native String toString(Object one);
    public static native String toString(Object one, String fallback);
    public static native <T> T requireNonNull(T one);
    public static native <T> T requireNonNull(T one, String message);
    public static native <T> T requireNonNull(T one, java.util.function.Supplier<String> message);
    public static native <T> T requireNonNullElse(T one, T fallback);
    public static native <T> T requireNonNullElseGet(T one, java.util.function.Supplier<T> fallback);
    public static native int checkIndex(int at, int length);
    public static native boolean isNull(Object one);
    public static native boolean nonNull(Object one);
    public static native int compare(Object left, Object right, Comparator by);
}

public class Optional<T> {
    public static native <X> Optional<X> of(X one);
    public static native <X> Optional<X> ofNullable(X one);
    public static native <X> Optional<X> empty();
    public native boolean isPresent();
    public native boolean isEmpty();
    public native T get();
    public native T orElse(T fallback);
    public native T orElseGet(java.util.function.Supplier<T> fallback);
    public native T orElseThrow();
    public native void ifPresent(java.util.function.Consumer<T> what);
    public native void ifPresentOrElse(java.util.function.Consumer<T> what, java.lang.Runnable otherwise);
    public native Optional<T> or(java.util.function.Supplier<Optional<T>> otherwise);
    public native Optional<T> filter(java.util.function.Predicate<T> when);
    public native <R> Optional<R> map(java.util.function.Function<T, R> what);
    public native <R> Optional<R> flatMap(java.util.function.Function<T, Optional<R>> what);
    public native java.util.stream.Stream<T> stream();
}

public class OptionalInt {
    public native boolean isPresent();
    public native boolean isEmpty();
    public native int getAsInt();
    public native int orElse(int fallback);
}

public class OptionalLong {
    public native boolean isPresent();
    public native long getAsLong();
    public native long orElse(long fallback);
}

public class OptionalDouble {
    public native boolean isPresent();
    public native double getAsDouble();
    public native double orElse(double fallback);
}

public class Random {
    public native Random();
    public native Random(long seed);
    public native int nextInt();
    public native int nextInt(int bound);
    public native long nextLong();
    public native double nextDouble();
    public native float nextFloat();
    public native boolean nextBoolean();
    public native double nextGaussian();
    public native void setSeed(long seed);
    public native java.util.stream.IntStream ints(long count);
}

public class Scanner implements java.lang.AutoCloseable {
    public native Scanner(java.io.InputStream from);
    public native Scanner(String from);
    public native boolean hasNext();
    public native boolean hasNextLine();
    public native boolean hasNextInt();
    public native String next();
    public native String nextLine();
    public native int nextInt();
    public native long nextLong();
    public native double nextDouble();
    public native void close();
}

public class StringJoiner {
    public native StringJoiner(CharSequence between);
    public native StringJoiner(CharSequence between, CharSequence before, CharSequence after);
    public native StringJoiner add(CharSequence one);
    public native int length();
    public native String toString();
}

public class UUID {
    public static native UUID randomUUID();
    public static native UUID fromString(String held);
    public native String toString();
    public native long getMostSignificantBits();
    public native long getLeastSignificantBits();
}

public class Date implements Comparable<Date> {
    public native Date();
    public native Date(long millis);
    public native long getTime();
    public native void setTime(long millis);
    public native boolean before(Date other);
    public native boolean after(Date other);
    public native int compareTo(Date other);
}

public class Calendar {
    public static final int YEAR = 1;
    public static final int MONTH = 2;
    public static final int DAY_OF_MONTH = 5;
    public static final int HOUR_OF_DAY = 11;
    public static final int MINUTE = 12;
    public static final int SECOND = 13;
    public static native Calendar getInstance();
    public native int get(int field);
    public native void set(int field, int value);
    public native void add(int field, int amount);
    public native Date getTime();
    public native void setTime(Date held);
    public native long getTimeInMillis();
}

public class GregorianCalendar extends Calendar {
    public native GregorianCalendar();
}

public class TimeZone {
    public static native TimeZone getDefault();
    public static native TimeZone getTimeZone(String id);
    public native String getID();
    public native int getRawOffset();
}

public class Locale {
    public static final Locale US;
    public static final Locale UK;
    public static final Locale ROOT;
    public static final Locale ENGLISH;
    public static final Locale FRANCE;
    public static final Locale GERMANY;
    public native Locale(String language);
    public native Locale(String language, String country);
    public native String getLanguage();
    public native String getCountry();
    public static native Locale getDefault();
}

public class BitSet {
    public native BitSet();
    public native BitSet(int size);
    public native void set(int at);
    public native void set(int at, boolean held);
    public native void clear(int at);
    public native boolean get(int at);
    public native int cardinality();
    public native int length();
    public native int nextSetBit(int from);
}

public class NoSuchElementException extends java.lang.RuntimeException {
    public native NoSuchElementException();
    public native NoSuchElementException(String message);
}

public class ConcurrentModificationException extends java.lang.RuntimeException {
    public native ConcurrentModificationException();
    public native ConcurrentModificationException(String message);
}

public class InputMismatchException extends NoSuchElementException {
    public native InputMismatchException();
    public native InputMismatchException(String message);
}
"####,
    ),
    (
        "java.util.function",
        r####"
package java.util.function;

public interface Function<T, R> {
    R apply(T one);
    default <V> Function<T, V> andThen(Function<R, V> then) { return null; }
    default <V> Function<V, R> compose(Function<V, T> before) { return null; }
    static <X> Function<X, X> identity() { return null; }
}

public interface BiFunction<T, U, R> {
    R apply(T one, U two);
}

public interface Supplier<T> {
    T get();
}

public interface Consumer<T> {
    void accept(T one);
    default Consumer<T> andThen(Consumer<T> then) { return null; }
}

public interface BiConsumer<T, U> {
    void accept(T one, U two);
}

public interface Predicate<T> {
    boolean test(T one);
    default Predicate<T> negate() { return null; }
    default Predicate<T> and(Predicate<T> other) { return null; }
    default Predicate<T> or(Predicate<T> other) { return null; }
    static <X> Predicate<X> isEqual(Object one) { return null; }
    static <X> Predicate<X> not(Predicate<X> one) { return null; }
}

public interface BiPredicate<T, U> {
    boolean test(T one, U two);
}

public interface UnaryOperator<T> extends Function<T, T> {
    // Said again here, because what it stands for is a `T` in and a `T` out,
    // and the one place that can say so is the interface that has the one
    // type parameter.
    T apply(T one);
    static <X> UnaryOperator<X> identity() { return null; }
}

public interface BinaryOperator<T> extends BiFunction<T, T, T> {
    T apply(T left, T right);
    static <X> BinaryOperator<X> minBy(java.util.Comparator<X> by) { return null; }
    static <X> BinaryOperator<X> maxBy(java.util.Comparator<X> by) { return null; }
}

public interface ToIntFunction<T> {
    int applyAsInt(T one);
}

public interface ToLongFunction<T> {
    long applyAsLong(T one);
}

public interface ToDoubleFunction<T> {
    double applyAsDouble(T one);
}

public interface IntToLongFunction {
    long applyAsLong(int one);
}

public interface IntToDoubleFunction {
    double applyAsDouble(int one);
}

public interface IntFunction<R> {
    R apply(int one);
}

public interface IntPredicate {
    boolean test(int one);
}

public interface IntUnaryOperator {
    int applyAsInt(int one);
}

public interface IntBinaryOperator {
    int applyAsInt(int left, int right);
}

public interface IntConsumer {
    void accept(int one);
}

public interface IntSupplier {
    int getAsInt();
}

public interface LongUnaryOperator {
    long applyAsLong(long one);
}

public interface LongConsumer {
    void accept(long one);
}

public interface DoubleUnaryOperator {
    double applyAsDouble(double one);
}

public interface DoubleConsumer {
    void accept(double one);
}

public interface BooleanSupplier {
    boolean getAsBoolean();
}
"####,
    ),
    (
        "java.util.stream",
        r####"
package java.util.stream;

public interface BaseStream<T, S extends BaseStream<T, S>> extends java.lang.AutoCloseable {
    java.util.Iterator<T> iterator();
    S sequential();
    S parallel();
    void close();
}

public interface Stream<T> extends BaseStream<T, Stream<T>> {
    <R> Stream<R> map(java.util.function.Function<T, R> what);
    IntStream mapToInt(java.util.function.ToIntFunction<T> what);
    LongStream mapToLong(java.util.function.ToLongFunction<T> what);
    DoubleStream mapToDouble(java.util.function.ToDoubleFunction<T> what);
    <R> Stream<R> flatMap(java.util.function.Function<T, Stream<R>> what);
    Stream<T> filter(java.util.function.Predicate<T> when);
    Stream<T> distinct();
    Stream<T> sorted();
    Stream<T> sorted(java.util.Comparator<T> by);
    Stream<T> limit(long count);
    Stream<T> skip(long count);
    Stream<T> peek(java.util.function.Consumer<T> what);
    Stream<T> takeWhile(java.util.function.Predicate<T> when);
    Stream<T> dropWhile(java.util.function.Predicate<T> when);
    void forEach(java.util.function.Consumer<T> what);
    void forEachOrdered(java.util.function.Consumer<T> what);
    Object[] toArray();
    java.util.List<T> toList();
    <R> R collect(Collector<T, Object, R> into);
    T reduce(T start, java.util.function.BinaryOperator<T> what);
    java.util.Optional<T> reduce(java.util.function.BinaryOperator<T> what);
    long count();
    boolean anyMatch(java.util.function.Predicate<T> when);
    boolean allMatch(java.util.function.Predicate<T> when);
    boolean noneMatch(java.util.function.Predicate<T> when);
    java.util.Optional<T> findFirst();
    java.util.Optional<T> findAny();
    java.util.Optional<T> min(java.util.Comparator<T> by);
    java.util.Optional<T> max(java.util.Comparator<T> by);
    static <X> Stream<X> of(X... held);
    static <X> Stream<X> empty();
    static <X> Stream<X> concat(Stream<X> left, Stream<X> right);
    static <X> Stream<X> iterate(X start, java.util.function.UnaryOperator<X> what);
    static <X> Stream<X> generate(java.util.function.Supplier<X> what);
}

public interface IntStream extends BaseStream<java.lang.Integer, IntStream> {
    IntStream map(java.util.function.IntUnaryOperator what);
    IntStream filter(java.util.function.IntPredicate when);
    <R> Stream<R> mapToObj(java.util.function.IntFunction<R> what);
    LongStream mapToLong(java.util.function.IntToLongFunction what);
    DoubleStream asDoubleStream();
    Stream<java.lang.Integer> boxed();
    IntStream distinct();
    IntStream sorted();
    IntStream limit(long count);
    IntStream skip(long count);
    void forEach(java.util.function.IntConsumer what);
    int sum();
    long count();
    java.util.OptionalInt min();
    java.util.OptionalInt max();
    java.util.OptionalDouble average();
    int[] toArray();
    boolean anyMatch(java.util.function.IntPredicate when);
    boolean allMatch(java.util.function.IntPredicate when);
    int reduce(int start, java.util.function.IntBinaryOperator what);
    static IntStream range(int from, int to);
    static IntStream rangeClosed(int from, int to);
    static IntStream of(int... held);
}

public interface LongStream extends BaseStream<java.lang.Long, LongStream> {
    LongStream map(java.util.function.LongUnaryOperator what);
    Stream<java.lang.Long> boxed();
    long sum();
    long count();
    long[] toArray();
    static LongStream range(long from, long to);
    static LongStream of(long... held);
}

public interface DoubleStream extends BaseStream<java.lang.Double, DoubleStream> {
    DoubleStream map(java.util.function.DoubleUnaryOperator what);
    Stream<java.lang.Double> boxed();
    double sum();
    long count();
    java.util.OptionalDouble average();
    double[] toArray();
    static DoubleStream of(double... held);
}

public interface Collector<T, A, R> { }

public class Collectors {
    public static native <T> Collector<T, Object, java.util.List<T>> toList();
    public static native <T> Collector<T, Object, java.util.Set<T>> toSet();
    public static native <T> Collector<T, Object, java.util.List<T>> toUnmodifiableList();
    public static native <T, K, V> Collector<T, Object, java.util.Map<K, V>> toMap(java.util.function.Function<T, K> key, java.util.function.Function<T, V> value);
    public static native <T> Collector<T, Object, java.lang.String> joining();
    public static native <T> Collector<T, Object, java.lang.String> joining(java.lang.CharSequence between);
    public static native <T> Collector<T, Object, java.lang.String> joining(java.lang.CharSequence between, java.lang.CharSequence before, java.lang.CharSequence after);
    public static native <T, K> Collector<T, Object, java.util.Map<K, java.util.List<T>>> groupingBy(java.util.function.Function<T, K> by);
    public static native <T, K, A, D> Collector<T, Object, java.util.Map<K, D>> groupingBy(java.util.function.Function<T, K> by, Collector<T, A, D> then);
    public static native <T> Collector<T, Object, java.util.Map<java.lang.Boolean, java.util.List<T>>> partitioningBy(java.util.function.Predicate<T> when);
    public static native <T, A, D> Collector<T, Object, java.util.Map<java.lang.Boolean, D>> partitioningBy(java.util.function.Predicate<T> when, Collector<T, A, D> then);
    public static native <T, U, A, R> Collector<T, Object, R> mapping(java.util.function.Function<T, U> what, Collector<U, A, R> then);
    public static native <T> Collector<T, Object, java.lang.Long> counting();
    public static native <T> Collector<T, Object, java.lang.Integer> summingInt(java.util.function.ToIntFunction<T> by);
    public static native <T> Collector<T, Object, java.lang.Long> summingLong(java.util.function.ToLongFunction<T> by);
    public static native <T> Collector<T, Object, java.lang.Double> summingDouble(java.util.function.ToDoubleFunction<T> by);
    public static native <T> Collector<T, Object, java.lang.Double> averagingInt(java.util.function.ToIntFunction<T> by);
    public static native <T> Collector<T, Object, java.lang.Double> averagingLong(java.util.function.ToLongFunction<T> by);
    public static native <T> Collector<T, Object, java.lang.Double> averagingDouble(java.util.function.ToDoubleFunction<T> by);
    public static native <T, K, V> Collector<T, Object, java.util.Map<K, V>> toMap(java.util.function.Function<T, K> key, java.util.function.Function<T, V> value, java.util.function.BinaryOperator<V> merged);
    public static native <T> Collector<T, Object, java.util.Set<T>> toUnmodifiableSet();
    public static native <T, U, A, R> Collector<T, Object, R> flatMapping(java.util.function.Function<T, Stream<U>> what, Collector<U, A, R> then);
    public static native <T, A, R> Collector<T, Object, R> filtering(java.util.function.Predicate<T> when, Collector<T, A, R> then);
    public static native <T, A, R, S> Collector<T, Object, S> collectingAndThen(Collector<T, A, R> then, java.util.function.Function<R, S> after);
    public static native <T> Collector<T, Object, T> reducing(T start, java.util.function.BinaryOperator<T> what);
    public static native <T> Collector<T, Object, java.util.Optional<T>> minBy(java.util.Comparator<T> by);
    public static native <T> Collector<T, Object, java.util.Optional<T>> maxBy(java.util.Comparator<T> by);
}

"####,
    ),
    (
        "java.util.concurrent",
        r####"
package java.util.concurrent;

public interface Callable<V> {
    V call() throws java.lang.Exception ;
}

public interface Future<V> {
    V get() throws java.lang.InterruptedException, java.util.concurrent.ExecutionException ;
    boolean isDone();
    boolean cancel(boolean interrupt);
    boolean isCancelled();
}

public interface Executor {
    void execute(java.lang.Runnable job);
}

public interface ExecutorService extends Executor {
    <T> Future<T> submit(Callable<T> job);
    Future submit(java.lang.Runnable job);
    void shutdown();
    java.util.List<java.lang.Runnable> shutdownNow();
    boolean isShutdown();
    boolean isTerminated();
    boolean awaitTermination(long amount, TimeUnit unit) throws java.lang.InterruptedException ;
}

public class Executors {
    public static native ExecutorService newSingleThreadExecutor();
    public static native ExecutorService newFixedThreadPool(int size);
    public static native ExecutorService newCachedThreadPool();
}

public class TimeUnit extends java.lang.Enum<TimeUnit> {
    public static final TimeUnit NANOSECONDS;
    public static final TimeUnit MICROSECONDS;
    public static final TimeUnit MILLISECONDS;
    public static final TimeUnit SECONDS;
    public static final TimeUnit MINUTES;
    public static final TimeUnit HOURS;
    public static final TimeUnit DAYS;
    public native void sleep(long amount) throws java.lang.InterruptedException ;
    public native long toMillis(long amount);
    public native long toSeconds(long amount);
    public static native TimeUnit valueOf(String name);
    public static native TimeUnit[] values();
}

public class CountDownLatch {
    public native CountDownLatch(int count);
    public native void countDown();
    public native void await() throws java.lang.InterruptedException ;
    public native boolean await(long amount, TimeUnit unit) throws java.lang.InterruptedException ;
    public native long getCount();
}

public class ConcurrentHashMap<K, V> extends java.util.AbstractMap<K, V> implements java.util.Map<K, V> {
    public native ConcurrentHashMap();
    public native ConcurrentHashMap(int capacity);
    public native int size();
    public native boolean isEmpty();
    public native boolean containsKey(Object key);
    public native boolean containsValue(Object value);
    public native V get(Object key);
    public native V put(K key, V value);
    public native V remove(Object key);
    public native void clear();
    public native java.util.Set<K> keySet();
    public native java.util.Collection<V> values();
    public native java.util.Set<java.util.Map.Entry<K, V>> entrySet();
    public native V getOrDefault(Object key, V fallback);
    public native V putIfAbsent(K key, V value);
    public native V computeIfAbsent(K key, java.util.function.Function<K, V> what);
    public native void forEach(java.util.function.BiConsumer<K, V> what);
}

public class CopyOnWriteArrayList<E> extends java.util.AbstractList<E> implements java.util.List<E> {
    public native CopyOnWriteArrayList();
    public native int size();
    public native boolean add(E one);
    public native E get(int at);
    public native java.util.Iterator<E> iterator();
}

public class ConcurrentLinkedQueue<E> extends java.util.AbstractCollection<E> implements java.util.Queue<E> {
    public native ConcurrentLinkedQueue();
    public native int size();
    public native boolean add(E one);
    public native boolean offer(E one);
    public native E poll();
    public native E peek();
    public native E element();
    public native java.util.Iterator<E> iterator();
}

public class TimeoutException extends java.lang.Exception {
    public native TimeoutException();
    public native TimeoutException(String message);
}

public class ExecutionException extends java.lang.Exception {
    public native ExecutionException(Throwable cause);
}
"####,
    ),
    (
        "java.util.concurrent.atomic",
        r####"
package java.util.concurrent.atomic;

public class AtomicInteger extends java.lang.Number {
    public native AtomicInteger();
    public native AtomicInteger(int value);
    public native int get();
    public native void set(int value);
    public native int incrementAndGet();
    public native int decrementAndGet();
    public native int getAndIncrement();
    public native int getAndDecrement();
    public native int addAndGet(int by);
    public native int getAndAdd(int by);
    public native int getAndSet(int value);
    public native boolean compareAndSet(int expected, int value);
    public native int intValue();
    public native long longValue();
    public native double doubleValue();
}

public class AtomicLong extends java.lang.Number {
    public native AtomicLong();
    public native AtomicLong(long value);
    public native long get();
    public native void set(long value);
    public native long incrementAndGet();
    public native long decrementAndGet();
    public native long getAndIncrement();
    public native long addAndGet(long by);
    public native boolean compareAndSet(long expected, long value);
    public native long longValue();
    public native int intValue();
}

public class AtomicBoolean {
    public native AtomicBoolean();
    public native AtomicBoolean(boolean value);
    public native boolean get();
    public native void set(boolean value);
    public native boolean getAndSet(boolean value);
    public native boolean compareAndSet(boolean expected, boolean value);
}

public class AtomicReference<V> {
    public native AtomicReference();
    public native AtomicReference(V value);
    public native V get();
    public native void set(V value);
    public native V getAndSet(V value);
    public native boolean compareAndSet(V expected, V value);
    public native V updateAndGet(java.util.function.UnaryOperator<V> what);
}
"####,
    ),
    (
        "java.util.regex",
        r####"
package java.util.regex;

public class Pattern {
    public static final int CASE_INSENSITIVE = 2;
    public static final int MULTILINE = 8;
    public static final int DOTALL = 32;
    public static native Pattern compile(String held);
    public static native Pattern compile(String held, int flags);
    public static native boolean matches(String pattern, CharSequence held);
    public static native String quote(String held);
    public native Matcher matcher(CharSequence held);
    public native String pattern();
    public native String[] split(CharSequence held);
    public native java.util.stream.Stream<String> splitAsStream(CharSequence held);
}

public class Matcher {
    public native boolean matches();
    public native boolean find();
    public native boolean lookingAt();
    public native String group();
    public native String group(int at);
    public native String group(String name);
    public native int groupCount();
    public native int start();
    public native int start(int at);
    public native int end();
    public native int end(int at);
    public native String replaceAll(String with);
    public native String replaceFirst(String with);
    public native Matcher reset();
}

public class PatternSyntaxException extends java.lang.IllegalArgumentException {
    public native String getPattern();
    public native String getDescription();
    public native int getIndex();
}
"####,
    ),
    (
        "java.math",
        r####"
package java.math;

public class BigInteger extends java.lang.Number implements Comparable<BigInteger> {
    public static final BigInteger ZERO;
    public static final BigInteger ONE;
    public static final BigInteger TWO;
    public static final BigInteger TEN;
    public native BigInteger(String held);
    public native BigInteger add(BigInteger other);
    public native BigInteger subtract(BigInteger other);
    public native BigInteger multiply(BigInteger other);
    public native BigInteger divide(BigInteger other);
    public native BigInteger mod(BigInteger other);
    public native BigInteger remainder(BigInteger other);
    public native BigInteger pow(int by);
    public native BigInteger negate();
    public native BigInteger abs();
    public native BigInteger gcd(BigInteger other);
    public native int signum();
    public native int compareTo(BigInteger other);
    public native int intValue();
    public native long longValue();
    public native double doubleValue();
    public native String toString();
    public native String toString(int radix);
    public static native BigInteger valueOf(long value);
}

public class BigDecimal extends java.lang.Number implements Comparable<BigDecimal> {
    public static final BigDecimal ZERO;
    public static final BigDecimal ONE;
    public static final BigDecimal TEN;
    public native BigDecimal(String held);
    public native BigDecimal(double value);
    public native BigDecimal(int value);
    public native BigDecimal add(BigDecimal other);
    public native BigDecimal subtract(BigDecimal other);
    public native BigDecimal multiply(BigDecimal other);
    public native BigDecimal divide(BigDecimal other);
    public native BigDecimal divide(BigDecimal other, int scale, RoundingMode how);
    public native BigDecimal setScale(int scale, RoundingMode how);
    public native BigDecimal negate();
    public native BigDecimal abs();
    public native int scale();
    public native int signum();
    public native int compareTo(BigDecimal other);
    public native double doubleValue();
    public native int intValue();
    public native long longValue();
    public native String toString();
    public native String toPlainString();
    public static native BigDecimal valueOf(long value);
    public static native BigDecimal valueOf(double value);
}

public class RoundingMode extends java.lang.Enum<RoundingMode> {
    public static final RoundingMode UP;
    public static final RoundingMode DOWN;
    public static final RoundingMode CEILING;
    public static final RoundingMode FLOOR;
    public static final RoundingMode HALF_UP;
    public static final RoundingMode HALF_DOWN;
    public static final RoundingMode HALF_EVEN;
    public static native RoundingMode valueOf(String name);
    public static native RoundingMode[] values();
}
"####,
    ),
    (
        "java.time.temporal",
        r####"
package java.time.temporal;

public interface TemporalAccessor { }

public interface Temporal extends TemporalAccessor { }

public interface TemporalAmount { }

public interface TemporalUnit { }

public class ChronoUnit extends java.lang.Enum<ChronoUnit> implements TemporalUnit {
    public static final ChronoUnit NANOS;
    public static final ChronoUnit MILLIS;
    public static final ChronoUnit SECONDS;
    public static final ChronoUnit MINUTES;
    public static final ChronoUnit HOURS;
    public static final ChronoUnit DAYS;
    public static final ChronoUnit WEEKS;
    public static final ChronoUnit MONTHS;
    public static final ChronoUnit YEARS;
    public native long between(Temporal from, Temporal to);
    public static native ChronoUnit valueOf(String name);
    public static native ChronoUnit[] values();
}
"####,
    ),
    (
        "java.time.chrono",
        r####"
package java.time.chrono;

public interface ChronoLocalDate extends java.time.temporal.Temporal, Comparable<ChronoLocalDate> { }

public interface ChronoLocalDateTime<D> extends java.time.temporal.Temporal, Comparable<ChronoLocalDateTime<D>> { }

public interface ChronoZonedDateTime<D> extends java.time.temporal.Temporal, Comparable<ChronoZonedDateTime<D>> { }
"####,
    ),
    (
        "java.time",
        r####"
package java.time;

public class Duration implements Comparable<Duration>, java.time.temporal.TemporalAmount {
    public static final Duration ZERO;
    public static native Duration ofDays(long amount);
    public static native Duration ofHours(long amount);
    public static native Duration ofMinutes(long amount);
    public static native Duration ofSeconds(long amount);
    public static native Duration ofMillis(long amount);
    public static native Duration ofNanos(long amount);
    public static native Duration between(java.time.temporal.Temporal from, java.time.temporal.Temporal to);
    public native long toDays();
    public native long toHours();
    public native long toMinutes();
    public native long toSeconds();
    public native long toMillis();
    public native long toNanos();
    public native Duration plus(Duration other);
    public native Duration minus(Duration other);
    public native Duration multipliedBy(long by);
    public native boolean isZero();
    public native boolean isNegative();
    public native int compareTo(Duration other);
}

public class Instant implements Comparable<Instant>, java.time.temporal.Temporal {
    public static final Instant EPOCH;
    public static native Instant now();
    public static native Instant ofEpochMilli(long millis);
    public static native Instant ofEpochSecond(long seconds);
    public native long toEpochMilli();
    public native long getEpochSecond();
    public native Instant plusSeconds(long amount);
    public native Instant plusMillis(long amount);
    public native Instant minusSeconds(long amount);
    public native boolean isBefore(Instant other);
    public native boolean isAfter(Instant other);
    public native int compareTo(Instant other);
    public native String toString();
}

public class DayOfWeek implements java.time.temporal.TemporalAccessor {
    public static final DayOfWeek MONDAY;
    public static final DayOfWeek TUESDAY;
    public static final DayOfWeek WEDNESDAY;
    public static final DayOfWeek THURSDAY;
    public static final DayOfWeek FRIDAY;
    public static final DayOfWeek SATURDAY;
    public static final DayOfWeek SUNDAY;
    public static native DayOfWeek of(int day);
    public static native DayOfWeek valueOf(String name);
    public static native DayOfWeek[] values();
    public native int getValue();
    public native String name();
    public native int ordinal();
    public native DayOfWeek plus(long amount);
    public native DayOfWeek minus(long amount);
    public native String toString();
}

public class Month implements java.time.temporal.TemporalAccessor {
    public static final Month JANUARY;
    public static final Month FEBRUARY;
    public static final Month MARCH;
    public static final Month APRIL;
    public static final Month MAY;
    public static final Month JUNE;
    public static final Month JULY;
    public static final Month AUGUST;
    public static final Month SEPTEMBER;
    public static final Month OCTOBER;
    public static final Month NOVEMBER;
    public static final Month DECEMBER;
    public static native Month of(int month);
    public static native Month valueOf(String name);
    public static native Month[] values();
    public native int getValue();
    public native String name();
    public native int ordinal();
    public native int length(boolean leap);
    public native Month plus(long amount);
    public native Month minus(long amount);
    public native String toString();
}

public class LocalDate implements java.time.chrono.ChronoLocalDate {
    public static final LocalDate MIN;
    public static final LocalDate MAX;
    public static native LocalDate now();
    public static native LocalDate of(int year, int month, int day);
    public static native LocalDate parse(CharSequence held);
    public static native LocalDate ofEpochDay(long day);
    public native int getYear();
    public native int getMonthValue();
    public native int getDayOfMonth();
    public native int getDayOfYear();
    public native DayOfWeek getDayOfWeek();
    public native Month getMonth();
    public native LocalDate plusWeeks(long amount);
    public native LocalDate minusWeeks(long amount);
    public native LocalDate withYear(int year);
    public native LocalDate withMonth(int month);
    public native LocalDate withDayOfYear(int day);
    public native LocalDate plusDays(long amount);
    public native LocalDate plusMonths(long amount);
    public native LocalDate plusYears(long amount);
    public native LocalDate minusDays(long amount);
    public native LocalDate minusMonths(long amount);
    public native LocalDate minusYears(long amount);
    public native LocalDate withDayOfMonth(int day);
    public native long toEpochDay();
    public native LocalDateTime atStartOfDay();
    public native boolean isBefore(java.time.chrono.ChronoLocalDate other);
    public native boolean isAfter(java.time.chrono.ChronoLocalDate other);
    public native boolean isEqual(java.time.chrono.ChronoLocalDate other);
    public native boolean isLeapYear();
    public native int lengthOfMonth();
    public native int lengthOfYear();
    public native int compareTo(java.time.chrono.ChronoLocalDate other);
    public native String format(java.time.format.DateTimeFormatter how);
    public native String toString();
}

public class LocalTime implements Comparable<LocalTime>, java.time.temporal.Temporal {
    public static final LocalTime MIN;
    public static final LocalTime MAX;
    public static final LocalTime NOON;
    public static native LocalTime now();
    public static native LocalTime of(int hour, int minute);
    public static native LocalTime of(int hour, int minute, int second);
    public static native LocalTime parse(CharSequence held);
    public native int getHour();
    public native int getMinute();
    public native int getSecond();
    public native LocalTime plusHours(long amount);
    public native LocalTime plusMinutes(long amount);
    public native LocalTime plusSeconds(long amount);
    public native boolean isBefore(LocalTime other);
    public native boolean isAfter(LocalTime other);
    public native int compareTo(LocalTime other);
    public native String format(java.time.format.DateTimeFormatter how);
    public native String toString();
}

public class LocalDateTime implements java.time.chrono.ChronoLocalDateTime<LocalDate> {
    public static native LocalDateTime now();
    public static native LocalDateTime of(int year, int month, int day, int hour, int minute);
    public static native LocalDateTime of(int year, int month, int day, int hour, int minute, int second);
    public static native LocalDateTime of(LocalDate day, LocalTime time);
    public static native LocalDateTime parse(CharSequence held);
    public native int getYear();
    public native int getMonthValue();
    public native int getDayOfMonth();
    public native int getHour();
    public native int getMinute();
    public native int getSecond();
    public native LocalDate toLocalDate();
    public native LocalTime toLocalTime();
    public native LocalDateTime plusDays(long amount);
    public native LocalDateTime plusHours(long amount);
    public native LocalDateTime plusMinutes(long amount);
    public native LocalDateTime plusSeconds(long amount);
    public native LocalDateTime minusDays(long amount);
    public native boolean isBefore(java.time.chrono.ChronoLocalDateTime<LocalDate> other);
    public native boolean isAfter(java.time.chrono.ChronoLocalDateTime<LocalDate> other);
    public native int compareTo(java.time.chrono.ChronoLocalDateTime<LocalDate> other);
    public native String format(java.time.format.DateTimeFormatter how);
    public native String toString();
}

public class ZoneId {
    public static native ZoneId systemDefault();
    public static native ZoneId of(String name);
    public native String getId();
}

public class ZonedDateTime implements java.time.chrono.ChronoZonedDateTime<LocalDate> {
    public static native ZonedDateTime now();
    public static native ZonedDateTime now(ZoneId where);
    public native LocalDateTime toLocalDateTime();
    public native LocalDate toLocalDate();
    public native Instant toInstant();
    public native int getYear();
    public native int getHour();
    public native int compareTo(java.time.chrono.ChronoZonedDateTime<LocalDate> other);
    public native String format(java.time.format.DateTimeFormatter how);
    public native String toString();
}

public class Period implements java.time.temporal.TemporalAmount {
    public static native Period ofDays(int amount);
    public static native Period ofMonths(int amount);
    public static native Period ofYears(int amount);
    public static native Period between(LocalDate from, LocalDate to);
    public native int getYears();
    public native int getMonths();
    public native int getDays();
    public native long toTotalMonths();
}

public class DateTimeException extends java.lang.RuntimeException {
    public native DateTimeException(String message);
}
"####,
    ),
    (
        "java.time.format",
        r####"
package java.time.format;

public class DateTimeFormatter {
    public static final DateTimeFormatter ISO_LOCAL_DATE;
    public static final DateTimeFormatter ISO_LOCAL_TIME;
    public static final DateTimeFormatter ISO_LOCAL_DATE_TIME;
    public static final DateTimeFormatter ISO_INSTANT;
    public static final DateTimeFormatter ISO_DATE;
    public static native DateTimeFormatter ofPattern(String pattern);
    public static native DateTimeFormatter ofPattern(String pattern, java.util.Locale where);
    public native String format(java.time.temporal.TemporalAccessor held);
    public native DateTimeFormatter withLocale(java.util.Locale where);
}

public class DateTimeParseException extends java.time.DateTimeException {
    public native DateTimeParseException(String message, CharSequence held, int at);
    public native String getParsedString();
    public native int getErrorIndex();
}
"####,
    ),
    (
        "java.text",
        r####"
package java.text;

public class Format {
    public native String format(Object held);
}

public class DateFormat extends Format {
    public static final int SHORT = 3;
    public static final int MEDIUM = 2;
    public static final int LONG = 1;
    public static final int FULL = 0;
    public native String format(java.util.Date held);
    public native java.util.Date parse(String held) throws java.text.ParseException ;
    public native void setTimeZone(java.util.TimeZone held);
    public static native DateFormat getDateInstance();
    public static native DateFormat getTimeInstance();
    public static native DateFormat getDateTimeInstance();
}

public class SimpleDateFormat extends DateFormat {
    public native SimpleDateFormat(String pattern);
    public native SimpleDateFormat(String pattern, java.util.Locale where);
    public native String format(java.util.Date held);
    public native java.util.Date parse(String held);
    public native void applyPattern(String pattern);
    public native String toPattern();
}

public class NumberFormat extends Format {
    public native String format(double held);
    public native String format(long held);
    public native Number parse(String held) throws java.text.ParseException ;
    public native void setMaximumFractionDigits(int held);
    public native void setMinimumFractionDigits(int held);
    public native void setGroupingUsed(boolean held);
    public static native NumberFormat getInstance();
    public static native NumberFormat getInstance(java.util.Locale where);
    public static native NumberFormat getNumberInstance();
    public static native NumberFormat getPercentInstance();
    public static native NumberFormat getCurrencyInstance();
}

public class DecimalFormat extends NumberFormat {
    public native DecimalFormat(String pattern);
    public native void applyPattern(String pattern);
}

public class ParseException extends java.lang.Exception {
    public native ParseException(String message, int at);
    public native int getErrorOffset();
}
"####,
    ),
    (
        "java.nio",
        r####"
package java.nio.charset;

public class Charset {
    public static native Charset forName(String name);
    public static native Charset defaultCharset();
    public native String name();
}

public class StandardCharsets {
    public static final Charset UTF_8;
    public static final Charset US_ASCII;
    public static final Charset ISO_8859_1;
    public static final Charset UTF_16;
}
"####,
    ),
    (
        "java.nio.file.attribute",
        r####"
package java.nio.file.attribute;

public interface FileAttribute<T> {
    String name();
    T value();
}
"####,
    ),
    (
        "java.nio.file",
        r####"
package java.nio.file;

public interface OpenOption { }

public interface CopyOption { }

public class LinkOption extends java.lang.Enum<LinkOption> implements OpenOption, CopyOption {
    public static final LinkOption NOFOLLOW_LINKS;
    public static native LinkOption valueOf(String name);
    public static native LinkOption[] values();
}

public class StandardOpenOption extends java.lang.Enum<StandardOpenOption> implements OpenOption {
    public static final StandardOpenOption READ;
    public static final StandardOpenOption WRITE;
    public static final StandardOpenOption APPEND;
    public static final StandardOpenOption CREATE;
    public static final StandardOpenOption CREATE_NEW;
    public static final StandardOpenOption TRUNCATE_EXISTING;
    public static native StandardOpenOption valueOf(String name);
    public static native StandardOpenOption[] values();
}

public class StandardCopyOption extends java.lang.Enum<StandardCopyOption> implements CopyOption {
    public static final StandardCopyOption REPLACE_EXISTING;
    public static final StandardCopyOption COPY_ATTRIBUTES;
    public static final StandardCopyOption ATOMIC_MOVE;
    public static native StandardCopyOption valueOf(String name);
    public static native StandardCopyOption[] values();
}

public class FileVisitOption extends java.lang.Enum<FileVisitOption> {
    public static final FileVisitOption FOLLOW_LINKS;
    public static native FileVisitOption valueOf(String name);
    public static native FileVisitOption[] values();
}

public interface Path extends Comparable<Path> {
    Path getFileName();
    Path getParent();
    Path resolve(String other);
    Path resolve(Path other);
    Path toAbsolutePath();
    Path normalize();
    java.io.File toFile();
    String toString();
    int getNameCount();
    Path getName(int at);
    boolean startsWith(String other);
    boolean endsWith(String other);
    static Path of(String first, String... more) { return null; }
}

public class Paths {
    public static native Path get(String first, String... more);
}

public class Files {
    public static native boolean exists(Path held, LinkOption... how);
    public static native boolean notExists(Path held, LinkOption... how);
    public static native boolean isDirectory(Path held, LinkOption... how);
    public static native boolean isRegularFile(Path held, LinkOption... how);
    public static native boolean isReadable(Path held);
    public static native boolean isWritable(Path held);
    public static native long size(Path held) throws java.io.IOException ;
    public static native byte[] readAllBytes(Path held) throws java.io.IOException ;
    public static native java.util.List<String> readAllLines(Path held) throws java.io.IOException ;
    public static native String readString(Path held) throws java.io.IOException ;
    public static native Path write(Path held, byte[] what, OpenOption... how);
    public static native Path writeString(Path held, CharSequence what, OpenOption... how) throws java.io.IOException ;
    public static native Path createDirectory(Path held, java.nio.file.attribute.FileAttribute... how) throws java.io.IOException ;
    public static native Path createDirectories(Path held, java.nio.file.attribute.FileAttribute... how) throws java.io.IOException ;
    public static native Path createFile(Path held, java.nio.file.attribute.FileAttribute... how) throws java.io.IOException ;
    public static native void delete(Path held) throws java.io.IOException ;
    public static native boolean deleteIfExists(Path held) throws java.io.IOException ;
    public static native Path copy(Path from, Path to, CopyOption... how);
    public static native Path move(Path from, Path to, CopyOption... how) throws java.io.IOException ;
    public static native java.util.stream.Stream<Path> list(Path held) throws java.io.IOException ;
    public static native java.util.stream.Stream<Path> walk(Path held, FileVisitOption... how) throws java.io.IOException ;
    public static native java.util.stream.Stream<String> lines(Path held) throws java.io.IOException ;
    public static native java.io.BufferedReader newBufferedReader(Path held) throws java.io.IOException ;
    public static native java.io.BufferedWriter newBufferedWriter(Path held, OpenOption... how) throws java.io.IOException ;
    public static native java.io.InputStream newInputStream(Path held, OpenOption... how) throws java.io.IOException ;
    public static native java.io.OutputStream newOutputStream(Path held, OpenOption... how) throws java.io.IOException ;
}

public class NoSuchFileException extends java.io.IOException {
    public native NoSuchFileException(String message);
}

public class FileAlreadyExistsException extends java.io.IOException {
    public native FileAlreadyExistsException(String message);
}
"####,
    ),
    (
        "java.net",
        r####"
package java.net;

public class URI implements Comparable<URI> {
    public native URI(String held) throws java.net.URISyntaxException ;
    public native String getScheme();
    public native String getHost();
    public native int getPort();
    public native String getPath();
    public native String getQuery();
    public native URL toURL() throws java.net.MalformedURLException ;
    public native int compareTo(URI other);
    public native String toString();
    public static native URI create(String held);
}

public class URL {
    public native URL(String held) throws java.net.MalformedURLException ;
    public native String getProtocol();
    public native String getHost();
    public native int getPort();
    public native String getPath();
    public native java.io.InputStream openStream() throws java.io.IOException ;
    public native URLConnection openConnection() throws java.io.IOException ;
    public native String toString();
}

public class URLConnection {
    public native void connect() throws java.io.IOException ;
    public native void setRequestProperty(String name, String value);
    public native void setConnectTimeout(int millis);
    public native void setReadTimeout(int millis);
    public native java.io.InputStream getInputStream() throws java.io.IOException ;
    public native java.io.OutputStream getOutputStream() throws java.io.IOException ;
    public native void setDoOutput(boolean held);
    public native String getContentType();
    public native int getContentLength();
}

public class HttpURLConnection extends URLConnection {
    public static final int HTTP_OK = 200;
    public static final int HTTP_NOT_FOUND = 404;
    public native void setRequestMethod(String name) throws java.net.ProtocolException ;
    public native int getResponseCode() throws java.io.IOException ;
    public native String getResponseMessage() throws java.io.IOException ;
    public native void disconnect();
    public native java.io.InputStream getErrorStream();
}

public class URLEncoder {
    public static native String encode(String held, String charset) throws java.io.UnsupportedEncodingException ;
}

public class URLDecoder {
    public static native String decode(String held, String charset) throws java.io.UnsupportedEncodingException ;
}

public class MalformedURLException extends java.io.IOException {
    public native MalformedURLException(String message);
}

public class UnknownHostException extends java.io.IOException {
    public native UnknownHostException(String message);
}
"####,
    ),
];

const WELL_KNOWN: &[&str] = &[
    "java/lang/Object",
    "java/lang/String",
    "java/lang/annotation/Annotation",
    "java/lang/annotation/Retention",
    "java/lang/annotation/RetentionPolicy",
    "java/lang/annotation/Target",
    "java/lang/annotation/ElementType",
    "java/lang/annotation/Documented",
    "java/lang/annotation/Inherited",
    "java/lang/annotation/Repeatable",
    "java/lang/Enum",
    "java/lang/Record",
    "java/lang/Runnable",
    "java/lang/CharSequence",
    "java/lang/StringBuilder",
    "java/lang/System",
    "java/lang/Math",
    "java/lang/Integer",
    "java/lang/Long",
    "java/lang/Double",
    "java/lang/Float",
    "java/lang/Boolean",
    "java/lang/Character",
    "java/lang/Byte",
    "java/lang/Short",
    "java/lang/Number",
    "java/lang/Class",
    "java/io/PrintStream",
    "java/lang/Iterable",
    "java/lang/AutoCloseable",
    "java/util/Iterator",
    "java/util/Collection",
    "java/util/List",
    "java/util/Set",
    "java/util/Map",
    "java/util/ArrayList",
    "java/util/LinkedList",
    "java/util/HashSet",
    "java/util/LinkedHashSet",
    "java/util/TreeSet",
    "java/util/HashMap",
    "java/util/LinkedHashMap",
    "java/util/TreeMap",
    "java/util/Map$Entry",
    "java/util/Comparator",
    "java/util/Collections",
    "java/util/Objects",
    "java/util/Arrays",
    "java/lang/Comparable",
    "java/lang/Throwable",
    "java/lang/Exception",
    "java/lang/RuntimeException",
    "java/lang/Error",
    "java/lang/ArithmeticException",
    "java/lang/ArrayIndexOutOfBoundsException",
    "java/lang/ClassCastException",
    "java/lang/CloneNotSupportedException",
    "java/lang/IllegalArgumentException",
    "java/lang/IllegalStateException",
    "java/lang/IndexOutOfBoundsException",
    "java/lang/InterruptedException",
    "java/lang/NullPointerException",
    "java/lang/NumberFormatException",
    "java/lang/UnsupportedOperationException",
    "java/lang/AssertionError",
    "java/io/IOException",
];

impl Emitter<'_> {
    fn push_int(&mut self, value: i64) {
        match value {
            -1..=5 => self.op((0x03 + value) as u8),
            -128..=127 => self.op1(0x10, value as i8 as u8),
            -32768..=32767 => self.op2(0x11, value as i16 as u16),
            _ => {
                let index = self.pool.integer(value as i32);
                if index <= 255 {
                    self.op1(0x12, index as u8);
                } else {
                    self.op2(0x13, index);
                }
            }
        }
        self.pushes(&Type::Int);
    }

    fn push_string(&mut self, text: &str) {
        let index = self.pool.string(text);
        if index <= 255 {
            self.op1(0x12, index as u8);
        } else {
            self.op2(0x13, index);
        }
        self.pushes(&Type::Object("java/lang/String".to_string(), Vec::new()));
    }

    fn load(&mut self, slot: u16, what: &Type) {
        if slot == 0 && self.early && !self.static_ {
            self.too_early = true;
        }
        let base = match what {
            Type::Long => 0x16u8,
            Type::Float => 0x17,
            Type::Double => 0x18,
            other if other.is_reference() => 0x19,
            _ => 0x15,
        };
        if slot <= 3 {
            let compact = match base {
                0x15 => 0x1a,
                0x16 => 0x1e,
                0x17 => 0x22,
                0x18 => 0x26,
                _ => 0x2a,
            };
            self.op(compact + slot as u8);
        } else if slot <= 255 {
            self.op1(base, slot as u8);
        } else {
            self.op(0xc4);
            self.op2(base, slot);
        }
        self.pushes(what);
        if slot == 0 && !self.static_ && self.slots.first() == Some(&Verified::UninitializedThis) {
            if let Some(top) = self.stack.last_mut() {
                *top = Verified::UninitializedThis;
            }
        }
    }

    fn store(&mut self, slot: u16, what: &Type) {
        let (base, compact) = match what {
            Type::Long => (0x37u8, 0x3fu8),
            Type::Float => (0x38, 0x43),
            Type::Double => (0x39, 0x47),
            other if other.is_reference() => (0x3a, 0x4b),
            _ => (0x36, 0x3b),
        };
        if slot <= 3 {
            self.op(compact + slot as u8);
        } else if slot <= 255 {
            self.op1(base, slot as u8);
        } else {
            self.op(0xc4);
            self.op2(base, slot);
        }
        self.pops(i32::from(what.width()));

        self.slot_holds(slot, what);
    }

    fn fit(&mut self, found: &Type, wanted: &Type, line: u32) -> Result<bool, Diagnostic> {
        if self.may_be_given(found, wanted) {
            if !found.is_reference() && !wanted.is_reference() {
                self.convert(found, wanted, line)?;
            }

            if found.is_reference() && *wanted != *found && !self.reaches(found, wanted) {
                if let Type::Object(named, _) = wanted {
                    let index = self.pool.class(named);
                    self.op2(0xc0, index);
                    self.pops(1);
                    self.pushes(wanted);
                }
            }
            return Ok(true);
        }
        let Some(now) = self.box_or_unbox(found, wanted) else {
            return Ok(false);
        };
        if !now.is_reference() && !wanted.is_reference() {
            self.convert(&now, wanted, line)?;
        }
        Ok(true)
    }

    fn may_be_given(&self, found: &Type, wanted: &Type) -> bool {
        if !found.is_reference() || !wanted.is_reference() {
            return found.may_be_given_to(wanted);
        }
        if !self.reaches(found, wanted) {
            return self.reaches(wanted, found);
        }

        true
    }

    fn box_or_unbox(&mut self, from: &Type, to: &Type) -> Option<Type> {
        if !from.is_reference() && *from != Type::Void && to.is_reference() {
            let boxed = boxed_name(from)?;

            if let Type::Object(named, _) = to {
                let fits = named == boxed
                    || named == "java/lang/Object"
                    || (named == "java/lang/Number"
                        && !matches!(boxed, "java/lang/Boolean" | "java/lang/Character"))
                    || named == "java/lang/Comparable"
                    || named == "java/io/Serializable";
                if !fits {
                    return None;
                }
            } else {
                return None;
            }
            let descriptor = format!(
                "({}){}",
                from.descriptor(),
                Type::Object(boxed.to_string(), Vec::new()).descriptor()
            );
            let index = self.pool.method(boxed, "valueOf", &descriptor, false);
            self.op2(0xb8, index);
            self.pops(i32::from(from.width()));
            self.pushes(&Type::Object(boxed.to_string(), Vec::new()));
            return Some(Type::Object(boxed.to_string(), Vec::new()));
        }

        if from.is_reference() && !to.is_reference() && *to != Type::Void {
            let Type::Object(named, _) = from else {
                return None;
            };

            let (holder, inner) = match named.as_str() {
                "java/lang/Boolean" => ("booleanValue", Type::Boolean),
                "java/lang/Byte" => ("byteValue", Type::Byte),
                "java/lang/Short" => ("shortValue", Type::Short),
                "java/lang/Character" => ("charValue", Type::Char),
                "java/lang/Integer" => ("intValue", Type::Int),
                "java/lang/Long" => ("longValue", Type::Long),
                "java/lang/Float" => ("floatValue", Type::Float),
                "java/lang/Double" => ("doubleValue", Type::Double),
                _ => return None,
            };
            let descriptor = format!("(){}", inner.descriptor());
            let index = self.pool.method(named, holder, &descriptor, false);
            self.op2(0xb6, index);
            self.pops(1);
            self.pushes(&inner);
            return Some(inner);
        }
        None
    }

    fn what_both_sides_come_to(
        &mut self,
        then: &Expression,
        otherwise: &Expression,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let beside = |what: Type| match boxed_name(&what) {
            Some(boxed) => Type::Object(boxed.to_string(), Vec::new()),
            None => what,
        };
        if matches!(then, Expression::Null) {
            let there = self.peek_type(otherwise, line)?;
            return Ok(beside(there));
        }
        if matches!(otherwise, Expression::Null) {
            let here = self.peek_type(then, line)?;
            return Ok(beside(here));
        }
        let here = self.peek_type(then, line)?;
        let there = self.peek_type(otherwise, line)?;
        if here == there {
            return Ok(here);
        }
        let object = Type::Object("java/lang/Object".to_string(), Vec::new());
        let inside = |what: &Type| match what.is_reference() {
            true => primitive_inside(what),
            false => Some(what.clone()),
        };

        if !here.is_reference() || !there.is_reference() {
            if let (Some(one), Some(two)) = (inside(&here), inside(&there)) {
                if one == two {
                    return Ok(one);
                }
                let fits = |narrow: &Type, value: Option<i32>| match (narrow, value) {
                    (Type::Byte, Some(held)) => i8::try_from(held).is_ok(),
                    (Type::Short, Some(held)) => i16::try_from(held).is_ok(),
                    (Type::Char, Some(held)) => u16::try_from(held).is_ok(),
                    _ => false,
                };
                let constant = |this: &Self, side: &Expression| {
                    constant_int(side)
                        .or_else(|| this.case_constant(side).as_ref().and_then(constant_int))
                };
                if two == Type::Int && fits(&one, constant(self, otherwise)) {
                    return Ok(one);
                }
                if one == Type::Int && fits(&two, constant(self, then)) {
                    return Ok(two);
                }
                if matches!(
                    (&one, &two),
                    (Type::Byte, Type::Short) | (Type::Short, Type::Byte)
                ) {
                    return Ok(Type::Short);
                }
                if one == Type::Boolean || two == Type::Boolean {
                    return Ok(object);
                }
                return Ok(one.promoted_with(&two).unwrap_or(Type::Int));
            }

            return Ok(object);
        }
        if self.reaches(&there, &here) {
            return Ok(here);
        }
        if self.reaches(&here, &there) {
            return Ok(there);
        }
        if let (Type::Object(left, held), Type::Object(right, _)) = (&here, &there) {
            if self.a_class_this_knows(left) && self.a_class_this_knows(right) {
                for above in self.classpath.ancestors(left).into_iter().skip(1) {
                    if above == "java/lang/Object" || !self.a_class_this_knows(&above) {
                        continue;
                    }
                    if self.reaches(&there, &Type::Object(above.clone(), Vec::new())) {
                        let arguments = self.classpath.seen_from(left, held, &above);
                        return Ok(Type::Object(above, arguments));
                    }
                }
            }
        }
        Ok(object)
    }

    fn narrowed_by_the_operator(
        &mut self,
        found: &Type,
        wanted: &Type,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        if found == wanted || found.is_reference() || wanted.is_reference() {
            return Ok(found.clone());
        }

        let narrowing = match (found, wanted) {
            (from, Type::Byte) if from.is_int_like() => Some(0x91u8),
            (from, Type::Char) if from.is_int_like() => Some(0x92),
            (from, Type::Short) if from.is_int_like() => Some(0x93),
            _ => None,
        };
        if let Some(opcode) = narrowing {
            self.op(opcode);
            self.pops(i32::from(found.width()));
            self.pushes(wanted);
            return Ok(wanted.clone());
        }
        self.convert(found, wanted, line)?;
        Ok(wanted.clone())
    }

    fn convert(&mut self, from: &Type, to: &Type, line: u32) -> Result<(), Diagnostic> {
        if from.is_reference() && !to.is_reference() && *to != Type::Void {
            if let Some(inner) = self.box_or_unbox(from, to) {
                if inner == *to {
                    return Ok(());
                }
                return self.convert(&inner, to, line);
            }
        }
        if matches!(from, Type::Long | Type::Float | Type::Double)
            && matches!(to, Type::Byte | Type::Short | Type::Char)
        {
            self.convert(from, &Type::Int, line)?;
            return self.convert(&Type::Int, to, line);
        }
        if from == to || (from.is_int_like() && to.is_int_like()) {
            return Ok(());
        }
        let opcode = match (from, to) {
            (f, Type::Long) if f.is_int_like() => 0x85u8,
            (f, Type::Float) if f.is_int_like() => 0x86,
            (f, Type::Double) if f.is_int_like() => 0x87,
            (Type::Long, Type::Int) => 0x88,
            (Type::Long, Type::Float) => 0x89,
            (Type::Long, Type::Double) => 0x8a,
            (Type::Float, Type::Int) => 0x8b,
            (Type::Float, Type::Long) => 0x8c,
            (Type::Float, Type::Double) => 0x8d,
            (Type::Double, Type::Int) => 0x8e,
            (Type::Double, Type::Long) => 0x8f,
            (Type::Double, Type::Float) => 0x90,
            (Type::Int, Type::Byte) => 0x91,
            (Type::Int, Type::Char) => 0x92,
            (Type::Int, Type::Short) => 0x93,
            _ if from.is_reference() && to.is_reference() => return Ok(()),
            _ => {
                return Err(at(
                    "EJ201",
                    line,
                    1,
                    format!("A {} does not become a {}.", from.readable(), to.readable()),
                ))
            }
        };
        self.op(opcode);
        self.pops(i32::from(from.width()));
        self.pushes(to);
        Ok(())
    }

    fn value(&mut self, expression: &Expression, line: u32) -> Result<Type, Diagnostic> {
        match expression {
            Expression::Int(value) => {
                self.push_int(*value);
                Ok(Type::Int)
            }
            Expression::Char(value) => {
                self.push_int(i64::from(*value));
                Ok(Type::Char)
            }
            Expression::Boolean(value) => {
                self.push_int(i64::from(*value));
                Ok(Type::Boolean)
            }
            Expression::Long(value) => {
                if *value == 0 || *value == 1 {
                    self.op(0x09 + *value as u8);
                } else {
                    let index = self.pool.long(*value);
                    self.op2(0x14, index);
                }
                self.pushes(&Type::Long);
                Ok(Type::Long)
            }
            Expression::Float(value) => {
                let index = self.pool.float(*value as f32);
                if index <= 255 {
                    self.op1(0x12, index as u8);
                } else {
                    self.op2(0x13, index);
                }
                self.pushes(&Type::Float);
                Ok(Type::Float)
            }
            Expression::Double(value) => {
                let index = self.pool.double(*value);
                self.op2(0x14, index);
                self.pushes(&Type::Double);
                Ok(Type::Double)
            }
            Expression::Str(text) => {
                self.push_string(text);
                Ok(Type::Object("java/lang/String".to_string(), Vec::new()))
            }
            Expression::Null => {
                self.op(0x01);
                self.pushes_raw(Verified::Null);
                Ok(Type::Object("java/lang/Object".to_string(), Vec::new()))
            }
            Expression::This => {
                if self.static_ {
                    return Err(at(
                        "EJ202",
                        line,
                        1,
                        "`this` has no meaning in a static method.",
                    ));
                }

                if self.unit.stands_for_a_lambda {
                    if let Some(field) = self.unit.fields.iter().find(|held| held.name == OUTER) {
                        let what = self.resolve(&field.what.clone(), line)?;
                        self.load(0, &Type::Object(self.this_class.clone(), Vec::new()));
                        let index =
                            self.pool
                                .field(&self.this_class.clone(), OUTER, &what.descriptor());
                        self.op2(0xb4, index);
                        self.pops(1);
                        self.pushes(&what);
                        return Ok(what);
                    }
                }
                self.load(0, &Type::Object(self.this_class.clone(), Vec::new()));
                Ok(Type::Object(self.this_class.clone(), Vec::new()))
            }

            Expression::Super => {
                if self.static_ {
                    return Err(at(
                        "EJ222",
                        line,
                        1,
                        "`super` has no meaning in a static method.",
                    ));
                }
                let parent = match self.unit.extends.clone() {
                    Some(named) => self.resolve_class(&named, line)?,
                    None => "java/lang/Object".to_string(),
                };
                let above = Type::Object(parent, Vec::new());
                self.load(0, &above);
                Ok(above)
            }
            Expression::Name(name) => self.name_value(name, line),
            Expression::Field { of, name } => {
                if let Expression::Name(maybe_class) = of.as_ref() {
                    if self.meant_as_a_class(maybe_class) {
                        let owner = self.resolve_class(maybe_class, line)?;
                        return self.read_static_field(&owner, name, line);
                    }
                }

                if let Some(path) = written_as_a_path(of) {
                    let head = path.split('.').next().unwrap_or_default();
                    if self.meant_as_a_class(head) {
                        if let Some(owner) = resolve_named(self.classpath, self.unit, &path) {
                            return self.read_static_field(&owner, name, line);
                        }
                    }
                }
                let owner = self.value(of, line)?;
                self.read_field(&owner, name, line)
            }
            Expression::Index { of, at: index } => {
                let array = self.value(of, line)?;
                let Type::Array(element) = array.clone() else {
                    return Err(at(
                        "EJ203",
                        line,
                        1,
                        format!("A {} cannot be indexed.", array.readable()),
                    ));
                };
                let found = self.value(index, line)?;
                self.convert(&found, &Type::Int, line)?;
                self.op(array_load(&element));
                self.pops(2);
                self.pushes(&element);
                Ok(*element)
            }
            Expression::Cast { to, of, also } => {
                let target = self.resolve(to, line)?;
                if matches!(
                    **of,
                    Expression::Lambda { .. } | Expression::MethodRef { .. }
                ) {
                    let mut bounds = vec![target.clone()];
                    for one in also {
                        bounds.push(self.resolve(one, line)?);
                    }
                    let functional = bounds
                        .iter()
                        .position(|one| match one {
                            Type::Object(named, _) => self.the_one_method_of(named, line).is_ok(),
                            _ => false,
                        })
                        .unwrap_or(0);
                    let stands_for = bounds.remove(functional);
                    self.markers = bounds
                        .iter()
                        .filter_map(|one| match one {
                            Type::Object(named, _) => Some(named.replace('/', ".")),
                            _ => None,
                        })
                        .collect();
                    self.expecting = Some(stands_for);
                    let made = self.value(of, line);
                    self.markers.clear();
                    made?;
                    return Ok(target);
                }
                self.expecting = None;
                let found = self.value(of, line)?;
                if target.is_reference() && found.is_reference() {
                    for one in also {
                        let bound = self.resolve(one, line)?;
                        let index = self.pool.class(&match &bound {
                            Type::Object(name, _) => name.clone(),
                            other => other.descriptor(),
                        });
                        self.op2(0xc0, index);
                    }
                    let index = self.pool.class(&match &target {
                        Type::Object(name, _) => name.clone(),
                        other => other.descriptor(),
                    });
                    self.op2(0xc0, index);
                    self.pops(1);
                    self.pushes(&target);
                    return Ok(target);
                }
                if target.is_reference() && found != Type::Void {
                    if self.box_or_unbox(&found, &target).is_none() {
                        return Err(at(
                            "EJ201",
                            line,
                            1,
                            format!(
                                "A {} does not become a {}.",
                                found.readable(),
                                target.readable()
                            ),
                        ));
                    }
                    self.pops(1);
                    self.pushes(&target);
                    return Ok(target);
                }

                let found = match (&found, boxed_name(&target)) {
                    (Type::Object(named, _), Some(boxed))
                        if matches!(
                            named.as_str(),
                            "java/lang/Object"
                                | "java/lang/Number"
                                | "java/lang/Comparable"
                                | "java/io/Serializable"
                        ) =>
                    {
                        let index = self.pool.class(boxed);
                        self.op2(0xc0, index);
                        self.pops(1);
                        let narrowed = Type::Object(boxed.to_string(), Vec::new());
                        self.pushes(&narrowed);
                        narrowed
                    }
                    _ => found,
                };
                let found = if found.is_reference() && !target.is_reference() {
                    match self.box_or_unbox(&found, &target) {
                        Some(opened) => opened,
                        None => {
                            return Err(at(
                                "EJ201",
                                line,
                                1,
                                format!(
                                    "A {} does not become a {}.",
                                    found.readable(),
                                    target.readable()
                                ),
                            ))
                        }
                    }
                } else {
                    found
                };

                let opcode = match (&found, &target) {
                    (f, t) if f == t => None,
                    (f, Type::Byte) if f.is_int_like() => Some(0x91u8),
                    (f, Type::Char) if f.is_int_like() => Some(0x92),
                    (f, Type::Short) if f.is_int_like() => Some(0x93),
                    _ => {
                        self.convert(&found, &target, line)?;
                        None
                    }
                };
                if let Some(opcode) = opcode {
                    self.op(opcode);
                }
                Ok(target)
            }
            Expression::Unary { operator, of } => {
                let found = self.value(of, line)?;
                match operator {
                    Unary::Plus => Ok(found),
                    Unary::Negate => {
                        let opcode = match found {
                            Type::Long => 0x75u8,
                            Type::Float => 0x76,
                            Type::Double => 0x77,
                            ref other if other.is_int_like() => 0x74,
                            _ => {
                                return Err(at(
                                    "EJ204",
                                    line,
                                    1,
                                    format!("A {} cannot be negated.", found.readable()),
                                ))
                            }
                        };
                        self.op(opcode);
                        Ok(if found.is_int_like() {
                            Type::Int
                        } else {
                            found
                        })
                    }
                    Unary::Complement => {
                        if found == Type::Long {
                            let index = self.pool.long(-1);
                            self.op2(0x14, index);
                            self.pushes(&Type::Long);
                            self.op(0x83);
                            self.pops(2);
                            return Ok(Type::Long);
                        }
                        if !found.is_int_like() {
                            return Err(at(
                                "EJ204",
                                line,
                                1,
                                format!("A {} has no bits to flip.", found.readable()),
                            ));
                        }
                        self.push_int(-1);
                        self.op(0x82);
                        self.pops(1);
                        Ok(Type::Int)
                    }
                    Unary::Not => {
                        let found = self.unboxed_truth(found);
                        if found != Type::Boolean {
                            return Err(at(
                                "EJ205",
                                line,
                                1,
                                format!(
                                    "`!` wants a boolean and was given a {}.",
                                    found.readable()
                                ),
                            ));
                        }

                        self.pops(1);
                        let jump = self.jump(0x99);
                        let beneath = self.stack.clone();
                        self.push_int(0);
                        let over = self.jump(0xa7);
                        self.land(jump);
                        self.stack_is(beneath.clone());
                        self.a_branch_lands_here();
                        self.push_int(1);
                        self.land(over);
                        let mut settled = beneath;
                        settled.push(Verified::Integer);
                        self.stack_is(settled);
                        self.a_branch_lands_here();
                        Ok(Type::Boolean)
                    }
                }
            }
            Expression::Binary {
                operator,
                left,
                right,
            } => self.binary(*operator, left, right, line),
            Expression::Conditional {
                condition,
                then,
                otherwise,
            } => {
                let wanted = self.expecting.take();
                let found = self.truth(condition, line)?;
                if found != Type::Boolean {
                    return Err(at(
                        "EJ206",
                        line,
                        1,
                        format!(
                            "A condition is a boolean, and this is a {}.",
                            found.readable()
                        ),
                    ));
                }
                self.pops(1);

                let functional = |side: &Expression| {
                    matches!(
                        side,
                        Expression::Lambda { .. } | Expression::MethodRef { .. }
                    )
                };
                let landed = match &wanted {
                    Some(target)
                        if target.is_reference() && (functional(then) || functional(otherwise)) =>
                    {
                        target.clone()
                    }
                    _ => self.what_both_sides_come_to(then, otherwise, line)?,
                };
                let to_else = self.jump(0x99);

                let beneath = self.stack.clone();
                let taken = self.value_for(then, &landed, line)?;
                if !self.fit(&taken, &landed, line)? {
                    return Err(at(
                        "EJ207",
                        line,
                        1,
                        format!(
                            "The two sides of `?:` are a {} and a {}.",
                            taken.readable(),
                            landed.readable()
                        ),
                    ));
                }
                let over = self.jump(0xa7);
                self.land(to_else);
                self.stack_is(beneath.clone());
                self.a_branch_lands_here();
                let other = self.value_for(otherwise, &landed, line)?;
                if !self.fit(&other, &landed, line)? {
                    return Err(at(
                        "EJ207",
                        line,
                        1,
                        format!(
                            "The two sides of `?:` are a {} and a {}.",
                            taken.readable(),
                            other.readable()
                        ),
                    ));
                }
                self.land(over);
                let mut settled = beneath;
                let held = Verified::of(&landed);
                let wide = held.is_wide();
                settled.push(held);
                if wide {
                    settled.push(Verified::Top);
                }
                self.stack_is(settled);
                self.a_branch_lands_here();
                Ok(landed)
            }
            Expression::InstanceOf { of, pattern } => {
                if pattern.taken {
                    let held = self.the_whole_pattern(of, pattern, line)?;
                    return self.value(&held, line);
                }
                let (what, binds) = (&pattern.what, &pattern.name);
                let binds = (!binds.is_empty()).then_some(binds);
                let found = self.value(of, line)?;
                let target = self.resolve(what, line)?;
                if !found.is_reference() {
                    return self.a_primitive_tested(&found, &target, binds, line);
                }
                let named = match &target {
                    Type::Object(name, _) => name.clone(),
                    other => match boxed_name(other) {
                        Some(boxed) => boxed.to_string(),
                        None => other.descriptor(),
                    },
                };
                if let Some(binding) = binds {
                    let object = Type::Object("java/lang/Object".to_string(), Vec::new());
                    let held = self.declare("$tested", object.clone());
                    self.store(held, &object);

                    self.zero_of(&target);
                    let slot = self.declare(binding, target.clone());
                    self.store(slot, &target);

                    self.load(held, &object);
                    let index = self.pool.class(&named);
                    self.op2(0xc1, index);

                    self.pops(1);
                    self.pushes(&Type::Boolean);
                    self.duplicates();
                    self.pops(1);
                    let over = self.jump(0x99);

                    self.load(held, &object);
                    let index = self.pool.class(&named);
                    self.op2(0xc0, index);
                    self.pops(1);
                    let wrapper = Type::Object(named.clone(), Vec::new());
                    if target.is_reference() {
                        self.pushes(&target);
                    } else {
                        self.pushes(&wrapper);
                        self.box_or_unbox(&wrapper, &target);
                    }
                    self.store(slot, &target);

                    self.land(over);
                    self.a_branch_lands_here();
                    return Ok(Type::Boolean);
                }
                let index = self.pool.class(&named);
                self.op2(0xc1, index);
                self.pops(1);
                self.pushes(&Type::Boolean);
                Ok(Type::Boolean)
            }
            Expression::Bind { name, what, value } => {
                let wanted = self.resolve(what, line)?;
                let found = self.value_for(value, &wanted, line)?;
                if !self.fit(&found, &wanted, line)? {
                    return Err(at(
                        "EJ228",
                        line,
                        1,
                        format!(
                            "A {} cannot be put in `{name}`, which is a {}.",
                            found.readable(),
                            wanted.readable()
                        ),
                    ));
                }
                let slot = self.declare(name, wanted.clone());
                self.store(slot, &wanted);
                self.push_int(1);
                Ok(Type::Boolean)
            }
            Expression::Switch { subject, arms } => self.switch_value(subject, arms, line),
            Expression::Anonymous {
                what,
                arguments,
                body,
                line: written,
                ..
            } => self.anonymous(what, arguments, body, *written),
            Expression::Lambda {
                parameters,
                body,
                expression,
                line: written,
            } => self.lambda(parameters, body, *expression, *written),
            Expression::MethodRef {
                on,
                name,
                line: written,
            } => self.method_reference(on, name, *written),
            Expression::NewArray { of, lengths, empty } => {
                self.new_array(of, lengths, *empty, line)
            }
            Expression::ArrayOf {
                of,
                values,
                line: written,
            } => {
                let Some(named) = of else {
                    return Err(at(
                        "EJ121",
                        *written,
                        1,
                        "There is nothing here saying what this array holds.",
                    )
                    .with_suggestion(
                        "Write `new` and the type in front of it, or give it to something \
                         declared as an array.",
                    ));
                };
                let element = self.resolve(named, *written)?;
                self.array_of(&Type::Array(Box::new(element)), values, *written)
            }
            Expression::ClassLiteral { of, line: written } => {
                let what = self.resolve(of, *written)?;
                let named = match &what {
                    Type::Object(name, _) => name.clone(),
                    Type::Array(_) => what.descriptor(),

                    other => {
                        let boxed = boxed_name(other).ok_or_else(|| {
                            at("EJ122", *written, 1, "That has no class to name.")
                        })?;
                        let index = self.pool.field(boxed, "TYPE", "Ljava/lang/Class;");
                        self.op2(0xb2, index);
                        let made = Type::Object(
                            "java/lang/Class".to_string(),
                            vec![Type::Object(boxed.to_string(), Vec::new())],
                        );
                        self.pushes(&made);
                        return Ok(made);
                    }
                };
                let index = self.pool.class(&named);
                if index <= 255 {
                    self.op1(0x12, index as u8);
                } else {
                    self.op2(0x13, index);
                }
                let made = Type::Object("java/lang/Class".to_string(), vec![what]);
                self.pushes(&made);
                Ok(made)
            }
            Expression::OuterThis { of, line: written } => self.outer_this(of, *written),
            Expression::New {
                what,
                arguments,
                outer,
            } => self.new_object(what, arguments, outer.as_deref(), line),
            Expression::Call {
                on,
                super_call,
                name,
                arguments,
                held,
            } => self.call(on.as_deref(), *super_call, name, arguments, held, line),
            Expression::Assign {
                target,
                operator,
                value,
            } => self.assign(target, *operator, value, line, true),
            Expression::Step { target, by, after } => self.step(target, *by, *after, line, true),
        }
    }
}

impl Emitter<'_> {
    fn name_value(&mut self, name: &str, line: u32) -> Result<Type, Diagnostic> {
        if let Some(local) = self.local(name) {
            self.load(local.slot, &local.what);
            return Ok(local.what);
        }
        if let Some(field) = self
            .unit
            .fields
            .iter()
            .find(|held| held.name == name)
            .cloned()
        {
            let what = self.resolve(&field.what, line)?;
            let descriptor = what.descriptor();
            let owner = self.this_class.clone();
            if field.modifiers.static_ {
                self.get_static(&owner, name, &what, line)?;
            } else {
                let index = self.pool.field(&owner, name, &descriptor);
                if self.static_ {
                    return Err(at(
                        "EJ210",
                        line,
                        1,
                        format!("`{name}` belongs to an instance and this method is static."),
                    ));
                }
                self.load(0, &Type::Object(owner, Vec::new()));
                self.op2(0xb4, index);
                self.pops(1);
                self.pushes(&what);
            }
            return Ok(what);
        }
        let inherited = match self.unit.extends.clone() {
            _ if !self.unit.stands_for_a_lambda
                && self.classpath.get(&self.this_class).is_some() =>
            {
                Some(self.this_class.clone())
            }
            Some(name) => Some(self.resolve_class(&name, line)?),
            None => None,
        };
        if let Some(owner) = inherited {
            if let Some((holder, (_, what, static_))) = self.classpath.find_field(&owner, name) {
                let (holder, what, static_) = (holder.name.clone(), what.clone(), *static_);
                let descriptor = what.descriptor();
                if static_ {
                    self.get_static(&holder, name, &what, line)?;
                } else {
                    let index = self.pool.field(&holder, name, &descriptor);
                    self.load(0, &Type::Object(self.this_class.clone(), Vec::new()));
                    self.op2(0xb4, index);
                    self.pops(1);
                    self.pushes(&what);
                }
                return Ok(what);
            }
        }

        for enclosing in self.the_classes_around() {
            if let Some((holder, (_, what, static_))) = self.classpath.find_field(&enclosing, name)
            {
                let (holder, what, static_) = (holder.name.clone(), what.clone(), *static_);
                let descriptor = what.descriptor();
                if static_ {
                    self.get_static(&holder, name, &what, line)?;
                    return Ok(what);
                }
                self.reach_the_enclosing_instance(&enclosing)?;
                let index = self.pool.field(&holder, name, &descriptor);
                self.op2(0xb4, index);
                self.pops(1);
                self.pushes(&what);
                return Ok(what);
            }
        }

        if let Some(what) = self.read_from_a_holder(name, line)? {
            return Ok(what);
        }

        for owner in self.imported_statically(name) {
            let held = match self.classpath.find_field(&owner, name) {
                Some((holder, (_, what, true))) => Some((holder.name.clone(), what.clone())),
                Some(_) => None,
                None => built_in_field(&owner, name).map(|what| (owner.clone(), what)),
            };
            let Some((holder, what)) = held else {
                continue;
            };
            self.get_static(&holder, name, &what, line)?;
            return Ok(what);
        }

        Err(at(
            "EJ211",
            line,
            1,
            format!("`{name}` is not anything this method can see."),
        )
        .with_context(format!("Name: {name}"))
        .with_suggestion(
            "Nothing here guesses at a name. A local, a parameter, a field of this \
                 class or of one above it: anything else is refused rather than written \
                 into a class file.",
        ))
    }

    fn meant_as_a_class(&self, name: &str) -> bool {
        if self.local(name).is_some() {
            return false;
        }
        if self.unit.fields.iter().any(|held| held.name == name) {
            return false;
        }
        let inherited = self
            .unit
            .extends
            .clone()
            .and_then(|parent| self.resolve_class(&parent, 1).ok())
            .and_then(|owner| self.classpath.find_field(&owner, name).map(|_| ()));
        if inherited.is_some() {
            return false;
        }

        for enclosing in self.the_classes_around() {
            if self.classpath.find_field(&enclosing, name).is_some() {
                return false;
            }
        }

        for holder in self.holders() {
            if self
                .classpath
                .find_field(&holder, name)
                .is_some_and(|(_, (_, _, static_))| *static_)
            {
                return false;
            }
        }
        if !self.unit.stands_for_a_lambda
            && self.classpath.find_field(&self.this_class, name).is_some()
        {
            return false;
        }
        for owner in self.imported_statically(name) {
            if self
                .classpath
                .find_field(&owner, name)
                .is_some_and(|(_, (_, _, static_))| *static_)
                || built_in_field(&owner, name).is_some()
            {
                return false;
            }
        }
        true
    }

    fn assign_through_the_enclosing_instance(
        &mut self,
        enclosing: &str,
        name: &str,
        operator: Option<Binary>,
        value: &Expression,
        line: u32,
        wanted: bool,
    ) -> Result<Type, Diagnostic> {
        let Some((holder, (_, what, static_))) = self.classpath.find_field(enclosing, name) else {
            return Err(at(
                "EJ229",
                line,
                1,
                format!("`{name}` is nothing that can be assigned to."),
            ));
        };
        let (holder, what, static_) = (holder.name.clone(), what.clone(), *static_);

        if !static_ {
            self.reach_the_enclosing_instance(enclosing)?;
        }
        let found = match operator {
            None => self.value_for(value, &what, line)?,

            Some(operator) => {
                let now =
                    self.binary(operator, &Expression::Name(name.to_string()), value, line)?;
                self.narrowed_by_the_operator(&now, &what, line)?
            }
        };
        if !self.fit(&found, &what, line)? {
            return Err(at(
                "EJ228",
                line,
                1,
                format!(
                    "A {} cannot be put in `{name}`, which is a {}.",
                    found.readable(),
                    what.readable()
                ),
            ));
        }

        self.open();
        let held = wanted.then(|| {
            let slot = self.declare("$assigned", what.clone());
            self.store(slot, &what);
            self.load(slot, &what);
            slot
        });
        let descriptor = what.descriptor();
        let index = self.pool.field(&holder, name, &descriptor);
        if static_ {
            self.op2(0xb3, index);
            self.pops(i32::from(what.width()));
        } else {
            self.op2(0xb5, index);
            self.pops(i32::from(what.width()) + 1);
        }
        if let Some(slot) = held {
            self.load(slot, &what);
        }
        self.close();
        Ok(what)
    }

    fn imported_statically(&self, name: &str) -> Vec<String> {
        let mut found = Vec::new();
        for held in &self.unit.static_imports {
            let Some((owner, last)) = held.rsplit_once('.') else {
                continue;
            };
            if last != "*" && last != name {
                continue;
            }
            if let Some(internal) = resolve_named(self.classpath, self.unit, owner) {
                if !found.contains(&internal) {
                    found.push(internal);
                }
            }
        }
        found
    }

    fn holders(&self) -> Vec<String> {
        let mut found = Vec::new();
        let mut holder = self.this_class.as_str();
        while let Some((outside, _)) = holder.rsplit_once('$') {
            found.push(outside.to_string());
            holder = outside;
        }
        found
    }

    fn read_from_a_holder(&mut self, name: &str, line: u32) -> Result<Option<Type>, Diagnostic> {
        let _ = line;
        for holder in self.holders() {
            let Some((known, (_, what, static_))) = self.classpath.find_field(&holder, name) else {
                continue;
            };
            if !*static_ {
                continue;
            }
            let (owner, what) = (known.name.clone(), what.clone());
            let descriptor = what.descriptor();
            let index = self.pool.field(&owner, name, &descriptor);
            self.op2(0xb2, index);
            self.pushes(&what);
            return Ok(Some(what));
        }
        Ok(None)
    }

    fn an_instance_of(&mut self, enclosing: &str) -> Result<bool, Diagnostic> {
        if self.static_ {
            return Ok(false);
        }
        let wanted = Type::Object(enclosing.to_string(), Vec::new());
        let this = Type::Object(self.this_class.clone(), Vec::new());
        if !self.early && self.reaches(&this, &wanted) {
            self.load(0, &this);
            return Ok(true);
        }
        for named in self.the_classes_around() {
            if self.reaches(&Type::Object(named.clone(), Vec::new()), &wanted) {
                self.reach_the_enclosing_instance(&named)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn reaching_up_from_a_lambda(
        &mut self,
        around: &str,
        name: &str,
        arguments: &[Expression],
        held: &[Written],
        going: Option<Type>,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let Some(owner) = self
            .classpath
            .get(around)
            .and_then(|known| known.superclass.clone())
        else {
            return Err(at(
                "EJ223",
                line,
                1,
                "This class extends nothing to call up into.",
            ));
        };
        let Some(signature) = self.signature_written_out(&owner, name, arguments, held, line)?
        else {
            return Err(self.no_such_method(&owner, name, arguments.len(), line));
        };
        self.reach_the_enclosing_instance(around)?;
        let first = with_what_the_arguments_say(&signature, &[], &[], going.as_ref());
        let given = self.what_was_handed_over(&first, arguments, line)?;
        let shaped = with_what_the_arguments_say(&signature, &[], &given, going.as_ref());
        self.arguments_for_signature(&shaped, arguments, line)?;
        let descriptor = signature.descriptor();
        let index = self
            .pool
            .method(around, &format!("{REACHING_UP}{name}"), &descriptor, false);
        self.op2(0xb6, index);
        let taken: i32 = signature
            .parameters
            .iter()
            .map(|p| i32::from(p.width()))
            .sum();
        self.pops(taken + 1);
        self.pushes(&signature.returns);
        Ok(signature.returns)
    }

    fn reach_the_enclosing_instance(&mut self, enclosing: &str) -> Result<(), Diagnostic> {
        let around = self.the_classes_around();
        let handed_in = self.early.then(|| self.local(OUTER)).flatten();
        if let (Some(held), Some(first)) = (handed_in, around.first()) {
            let first = first.clone();
            self.load(held.slot, &Type::Object(first.clone(), Vec::new()));
            if first == enclosing {
                return Ok(());
            }
            let mut from = first;
            for named in around.into_iter().skip(1) {
                let held = Type::Object(named.clone(), Vec::new());
                let index = self.pool.field(&from, OUTER, &held.descriptor());
                self.op2(0xb4, index);
                self.pops(1);
                self.pushes(&held);
                if named == enclosing {
                    return Ok(());
                }
                from = named;
            }
            return Err(at(
                "EJ252",
                1,
                1,
                format!(
                    "This class is written inside nothing that leads to `{}`.",
                    enclosing.replace('/', ".")
                ),
            ));
        }
        let this = Type::Object(self.this_class.clone(), Vec::new());
        self.load(0, &this);

        let mut from = self.this_class.clone();
        for named in around {
            let held = Type::Object(named.clone(), Vec::new());
            let descriptor = held.descriptor();
            let index = self.pool.field(&from, OUTER, &descriptor);
            self.op2(0xb4, index);
            self.pops(1);
            self.pushes(&held);
            if named == enclosing {
                return Ok(());
            }
            from = named;
        }
        Err(at(
            "EJ252",
            1,
            1,
            format!(
                "This class is written inside nothing that leads to `{}`.",
                enclosing.replace('/', ".")
            ),
        ))
    }

    fn is_answered_for(&self, named: &str) -> bool {
        if named == "java/lang/RuntimeException" || named == "java/lang/Error" {
            return false;
        }
        let mut walking = named.to_string();

        for _ in 0..64 {
            if walking == "java/lang/RuntimeException" || walking == "java/lang/Error" {
                return false;
            }
            if walking == "java/lang/Throwable" || walking == "java/lang/Exception" {
                return true;
            }
            match self
                .classpath
                .get(&walking)
                .and_then(|known| known.superclass.clone())
            {
                Some(above) => walking = above,

                None => return true,
            }
        }
        true
    }

    fn covers(&self, held: &[String], thrown: &str) -> bool {
        held.iter().any(|one| {
            one == thrown
                || self.reaches(
                    &Type::Object(thrown.to_string(), Vec::new()),
                    &Type::Object(one.clone(), Vec::new()),
                )
        })
    }

    fn it_can_throw(&mut self, named: &str, line: u32) -> Result<(), Diagnostic> {
        if !self.is_answered_for(named) {
            return Ok(());
        }
        if self.caught.iter().any(|held| self.covers(held, named)) {
            return Ok(());
        }
        let said = self.said_to_throw.clone();
        if self.covers(&said, named) {
            return Ok(());
        }
        Err(at(
            "EJ256",
            line,
            1,
            format!(
                "`{}` has to be caught or declared to be thrown.",
                named.replace('/', ".")
            ),
        )
        .with_suggestion(
            "Put it in a `try` with a `catch` for it, or write `throws` on the method. \
                Anything under `RuntimeException` or `Error` needs neither.",
        ))
    }

    fn the_classes_around(&self) -> Vec<String> {
        let mut around = Vec::new();
        let mut outer = self.unit.outer.clone();

        while around.len() < 16 {
            let Some(named) = outer else { break };
            if around.contains(&named) {
                break;
            }
            outer = self
                .classpath
                .get(&named)
                .and_then(|known| known.outer.clone());
            around.push(named);
        }
        around
    }

    fn read_static_field(
        &mut self,
        owner: &str,
        name: &str,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let known = self
            .classpath
            .find_field(owner, name)
            .map(|(holder, (_, what, static_))| (holder.name.clone(), what.clone(), *static_));
        let (holder, what) = match known {
            Some((holder, what, true)) => (holder, what),
            Some((_, _, false)) => {
                return Err(at(
                    "EJ214",
                    line,
                    1,
                    format!(
                        "`{name}` belongs to an instance of `{}`, and this names the class.",
                        owner.replace('/', ".")
                    ),
                ))
            }
            None => match built_in_field(owner, name) {
                Some(what) => (owner.to_string(), what),
                None => {
                    return Err(at(
                        "EJ213",
                        line,
                        1,
                        format!(
                            "`{}` has no field called `{name}` that this compilation knows.",
                            owner.replace('/', ".")
                        ),
                    )
                    .with_suggestion(
                        "Hand the class file that declares it over as a dependency. Nothing \
                         is emitted for a field nobody has seen.",
                    ))
                }
            },
        };
        self.get_static(&holder, name, &what, line)?;
        Ok(what)
    }

    fn get_static(
        &mut self,
        holder: &str,
        name: &str,
        what: &Type,
        line: u32,
    ) -> Result<(), Diagnostic> {
        let inlined = self
            .classpath
            .constant(holder, name)
            .cloned()
            .filter(|value| {
                matches!(
                    value,
                    Expression::Int(_)
                        | Expression::Long(_)
                        | Expression::Char(_)
                        | Expression::Boolean(_)
                        | Expression::Str(_)
                        | Expression::Float(_)
                        | Expression::Double(_)
                )
            });
        if let Some(value) = inlined {
            let found = self.value(&value, line)?;
            if found.width() != what.width() && !found.is_reference() && !what.is_reference() {
                self.convert(&found, what, line)?;
            }
            return Ok(());
        }
        let index = self.pool.field(holder, name, &what.descriptor());
        self.op2(0xb2, index);
        self.pushes(what);
        Ok(())
    }

    fn read_field(&mut self, owner: &Type, name: &str, line: u32) -> Result<Type, Diagnostic> {
        if let Type::Array(_) = owner {
            if name == "length" {
                self.op(0xbe);
                self.pops(1);
                self.pushes(&Type::Int);
                return Ok(Type::Int);
            }
        }
        let Type::Object(class, _) = owner else {
            return Err(at(
                "EJ212",
                line,
                1,
                format!("A {} has no fields.", owner.readable()),
            ));
        };
        let Some((holder, (_, what, static_))) = self.classpath.find_field(class, name) else {
            if let Some(what) = built_in_field(class, name) {
                let descriptor = what.descriptor();
                let index = self.pool.field(class, name, &descriptor);
                self.op(0x57);
                self.pops(1);
                self.op2(0xb2, index);
                self.pushes(&what);
                return Ok(what);
            }
            return Err(at(
                "EJ213",
                line,
                1,
                format!(
                    "`{}` has no field called `{name}` that this compilation knows.",
                    class.replace('/', ".")
                ),
            )
            .with_suggestion(
                "Hand the class file that declares it over as a dependency. Nothing is \
                 emitted for a field nobody has seen.",
            ));
        };
        let standing = holder
            .field_standings
            .iter()
            .find(|(held, _)| held == name)
            .map(|(_, standing)| standing.clone());
        let (holder, what, static_) = (holder.name.clone(), what.clone(), *static_);
        let descriptor = what.descriptor();
        let index = self.pool.field(&holder, name, &descriptor);
        if static_ {
            self.op(0x57);
            self.pops(1);
            self.op2(0xb2, index);
            self.pushes(&what);
            return Ok(what);
        }
        self.op2(0xb4, index);
        self.pops(1);
        self.pushes(&what);
        let arguments = match owner {
            Type::Object(_, held) if !held.is_empty() => {
                self.classpath.seen_from(class, held, &holder)
            }
            _ => Vec::new(),
        };
        let seen = match standing {
            Some(standing) if !arguments.is_empty() => standing.filled(&what, &arguments, &[]),
            _ => what.clone(),
        };
        if let (Type::Object(erased, _), Type::Object(narrower, _)) = (&what, &seen) {
            if erased != narrower {
                let index = self.pool.class(narrower);
                self.op2(0xc0, index);
                self.pops(1);
                self.pushes(&seen);
            }
        }
        Ok(seen)
    }

    fn truth(&mut self, expression: &Expression, line: u32) -> Result<Type, Diagnostic> {
        let found = self.value(expression, line)?;
        Ok(self.unboxed_truth(found))
    }

    fn unboxed_truth(&mut self, found: Type) -> Type {
        match &found {
            Type::Object(named, _) if named == "java/lang/Boolean" => {
                self.box_or_unbox(&found, &Type::Boolean).unwrap_or(found)
            }
            _ => found,
        }
    }

    fn binary(
        &mut self,
        operator: Binary,
        left: &Expression,
        right: &Expression,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        self.expecting = None;
        if matches!(operator, Binary::AndAlso | Binary::OrElse) {
            let outside = self.slots.clone();
            let first = self.truth(left, line)?;
            if first != Type::Boolean {
                return Err(at("EJ214", line, 1, "`&&` and `||` want booleans."));
            }
            self.pops(1);
            let shortcut = self.jump(if operator == Binary::AndAlso {
                0x99
            } else {
                0x9a
            });
            let beneath = self.stack.clone();
            let second = self.truth(right, line)?;
            if second != Type::Boolean {
                return Err(at("EJ214", line, 1, "`&&` and `||` want booleans."));
            }
            let inside = self.slots.clone();
            let over = self.jump(0xa7);
            self.land(shortcut);
            self.slots = outside.clone();
            self.stack_is(beneath.clone());
            self.a_branch_lands_here();

            self.give_them_null(&outside, &inside);
            self.push_int(i64::from(operator == Binary::OrElse));
            self.land(over);
            let mut settled = beneath;
            settled.push(Verified::Integer);
            self.slots = inside;
            self.stack_is(settled);
            self.a_branch_lands_here();
            return Ok(Type::Boolean);
        }

        let left_type = self.value(left, line)?;
        if operator == Binary::Add && left_type != text_type() {
            let peeked_right = self.peek_type(right, line)?;
            if peeked_right == text_type() {
                self.open();
                let slot = self.declare("$left", left_type.clone());
                self.store(slot, &left_type);
                let text = self.concatenate(&Expression::Name("$left".to_string()), right, line)?;
                self.close();
                return Ok(text);
            }
        }
        self.binary_with(operator, &left_type, right, line)
    }

    fn binary_with(
        &mut self,
        operator: Binary,
        left_type: &Type,
        right: &Expression,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let left_type = left_type.clone();

        if operator == Binary::Add
            && left_type == Type::Object("java/lang/String".to_string(), Vec::new())
        {
            self.open();
            let slot = self.declare("$left", left_type.clone());
            self.store(slot, &left_type);
            let text = self.concatenate(&Expression::Name("$left".to_string()), right, line)?;
            self.close();
            return Ok(text);
        }

        if matches!(
            operator,
            Binary::ShiftLeft | Binary::ShiftRight | Binary::UnsignedShiftRight
        ) {
            let right_type = self.value(right, line)?;
            self.convert(&right_type, &Type::Int, line)?;
            let long = left_type == Type::Long;
            let opcode = match (operator, long) {
                (Binary::ShiftLeft, false) => 0x78u8,
                (Binary::ShiftLeft, true) => 0x79,
                (Binary::ShiftRight, false) => 0x7a,
                (Binary::ShiftRight, true) => 0x7b,
                (Binary::UnsignedShiftRight, false) => 0x7c,
                _ => 0x7d,
            };
            self.op(opcode);
            self.pops(1);
            return Ok(if long { Type::Long } else { Type::Int });
        }

        let right_peeked = self.peek_type(right, line)?;

        let wants_numbers = !matches!(operator, Binary::Equal | Binary::NotEqual)
            || !(left_type.is_reference() && right_peeked.is_reference());
        let left_type = match primitive_inside(&left_type) {
            Some(inside) if wants_numbers => {
                self.box_or_unbox(&left_type, &inside);
                inside
            }
            _ => left_type,
        };
        let right_peeked = match primitive_inside(&right_peeked) {
            Some(inside) if wants_numbers => inside,
            _ => right_peeked,
        };

        let common = if left_type.is_reference() || right_peeked.is_reference() {
            left_type.clone()
        } else if left_type == Type::Boolean && right_peeked == Type::Boolean {
            Type::Boolean
        } else {
            left_type.promoted_with(&right_peeked).ok_or_else(|| {
                at(
                    "EJ215",
                    line,
                    1,
                    format!(
                        "A {} and a {} have no operator between them.",
                        left_type.readable(),
                        right_peeked.readable()
                    ),
                )
            })?
        };
        if !left_type.is_reference() {
            self.convert(&left_type, &common, line)?;
        }
        let right_type = self.value(right, line)?;
        let right_type = match primitive_inside(&right_type) {
            Some(inside) if wants_numbers => {
                self.box_or_unbox(&right_type, &inside);
                inside
            }
            _ => right_type,
        };
        if !right_type.is_reference() {
            self.convert(&right_type, &common, line)?;
        }

        match operator {
            Binary::Add
            | Binary::Subtract
            | Binary::Multiply
            | Binary::Divide
            | Binary::Remainder => {
                let base = match operator {
                    Binary::Add => 0x60u8,
                    Binary::Subtract => 0x64,
                    Binary::Multiply => 0x68,
                    Binary::Divide => 0x6c,
                    _ => 0x70,
                };
                let step = match common {
                    Type::Long => 1u8,
                    Type::Float => 2,
                    Type::Double => 3,
                    _ => 0,
                };
                self.op(base + step);
                self.pops(i32::from(common.width()));
                Ok(if common.is_int_like() {
                    Type::Int
                } else {
                    common
                })
            }
            Binary::And | Binary::Or | Binary::Xor => {
                if common == Type::Boolean || common.is_int_like() {
                    let base = match operator {
                        Binary::And => 0x7eu8,
                        Binary::Or => 0x80,
                        _ => 0x82,
                    };
                    self.op(base);
                    self.pops(1);
                    return Ok(if left_type == Type::Boolean {
                        Type::Boolean
                    } else {
                        Type::Int
                    });
                }
                if common == Type::Long {
                    let base = match operator {
                        Binary::And => 0x7fu8,
                        Binary::Or => 0x81,
                        _ => 0x83,
                    };
                    self.op(base);
                    self.pops(2);
                    return Ok(Type::Long);
                }
                Err(at(
                    "EJ216",
                    line,
                    1,
                    "That operator wants integers or booleans.",
                ))
            }
            _ => self.compare(operator, &common, line),
        }
    }

    fn compare(&mut self, operator: Binary, common: &Type, line: u32) -> Result<Type, Diagnostic> {
        let reference = common.is_reference();
        let opcode = match (common, operator) {
            (Type::Long, _) => {
                self.op(0x94);
                self.pops(3);
                None
            }
            (Type::Float, _) => {
                self.op(0x95);
                self.pops(1);
                None
            }
            (Type::Double, _) => {
                self.op(0x97);
                self.pops(3);
                None
            }
            _ if reference => match operator {
                Binary::Equal => Some(0xa5u8),
                Binary::NotEqual => Some(0xa6),
                _ => {
                    return Err(at(
                        "EJ217",
                        line,
                        1,
                        "Objects can only be compared with `==` and `!=` here.",
                    ))
                }
            },
            _ => Some(match operator {
                Binary::Equal => 0x9fu8,
                Binary::NotEqual => 0xa0,
                Binary::Less => 0xa1,
                Binary::GreaterOrEqual => 0xa2,
                Binary::Greater => 0xa3,
                _ => 0xa4,
            }),
        };

        let jump = match opcode {
            Some(code) => {
                self.pops(2);
                self.jump(code)
            }
            None => {
                let code = match operator {
                    Binary::Equal => 0x99u8,
                    Binary::NotEqual => 0x9a,
                    Binary::Less => 0x9b,
                    Binary::GreaterOrEqual => 0x9c,
                    Binary::Greater => 0x9d,
                    _ => 0x9e,
                };
                self.pops(1);
                self.jump(code)
            }
        };

        let beneath = self.stack.clone();
        self.push_int(0);
        let over = self.jump(0xa7);
        self.land(jump);
        self.stack_is(beneath.clone());
        self.a_branch_lands_here();
        self.push_int(1);
        self.land(over);
        let mut settled = beneath;
        settled.push(Verified::Integer);
        self.stack_is(settled);
        self.a_branch_lands_here();
        Ok(Type::Boolean)
    }

    fn peek_type(&mut self, expression: &Expression, line: u32) -> Result<Type, Diagnostic> {
        self.type_of(expression, line)
    }

    fn concatenate(
        &mut self,
        left: &Expression,
        right: &Expression,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let builder = "java/lang/StringBuilder";
        let index = self.pool.class(builder);
        self.op2(0xbb, index);
        self.pushes(&Type::Object(builder.to_string(), Vec::new()));
        self.duplicates();
        let init = self.pool.method(builder, "<init>", "()V", false);
        self.op2(0xb7, init);
        self.pops(1);

        for side in [left, right] {
            let what = self.value(side, line)?;
            let taken = match &what {
                Type::Object(name, _) if name == "java/lang/String" => {
                    "Ljava/lang/String;".to_string()
                }
                other if other.is_reference() => "Ljava/lang/Object;".to_string(),
                Type::Boolean => "Z".to_string(),
                Type::Char => "C".to_string(),
                Type::Long => "J".to_string(),
                Type::Float => "F".to_string(),
                Type::Double => "D".to_string(),
                _ => "I".to_string(),
            };
            let descriptor = format!("({taken})Ljava/lang/StringBuilder;");
            let append = self.pool.method(builder, "append", &descriptor, false);
            self.op2(0xb6, append);
            self.pops(i32::from(what.width()));
        }

        let finish = self
            .pool
            .method(builder, "toString", "()Ljava/lang/String;", false);
        self.op2(0xb6, finish);
        let text = Type::Object("java/lang/String".to_string(), Vec::new());
        self.pops(1);
        self.pushes(&text);
        Ok(text)
    }
}

impl Emitter<'_> {
    fn new_object(
        &mut self,
        what: &Written,
        arguments: &[Expression],
        outer: Option<&Expression>,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        if let Written::Named(named, _) = what {
            if let Some(held) = self
                .locally
                .iter()
                .rev()
                .find(|held| held.written == *named)
                .cloned()
            {
                return self.new_local_object(&held, arguments, line);
            }
        }

        let target = match (outer, what) {
            (Some(written), Written::Named(named, held)) if !named.contains('.') => {
                let held = held.clone();
                let named = named.clone();
                let around = self.peek_type(written, line).ok();
                let found = match around {
                    Some(Type::Object(class, _)) => {
                        let mut walking = vec![class];
                        let mut seen: Vec<String> = Vec::new();
                        let mut found = None;
                        while let Some(above) = walking.pop() {
                            if seen.contains(&above) {
                                continue;
                            }
                            let nested = format!("{above}${named}");
                            if self.classpath.get(&nested).is_some() {
                                found = Some(nested);
                                break;
                            }
                            if let Some(known) = self.classpath.get(&above) {
                                walking.extend(known.superclass.clone());
                                walking.extend(known.interfaces.iter().cloned());
                            }
                            seen.push(above);
                        }
                        found
                    }
                    _ => None,
                };
                match found {
                    Some(nested) => {
                        let mut arguments = Vec::new();
                        for one in &held {
                            match self.resolve(one, line) {
                                Ok(found) => arguments.push(found),
                                Err(_) => {
                                    arguments.clear();
                                    break;
                                }
                            }
                        }
                        Type::Object(nested, arguments)
                    }
                    None => self.resolve(what, line)?,
                }
            }
            _ => self.resolve(what, line)?,
        };
        let Type::Object(class, _) = target.clone() else {
            return Err(at(
                "EJ218",
                line,
                1,
                format!("`new` wants a class and was given a {}.", target.readable()),
            ));
        };
        let index = self.pool.class(&class);

        let made_at = self.code.len() as u16;
        self.op2(0xbb, index);
        self.pushes_raw(Verified::Uninitialized(made_at));
        self.duplicates();

        let belongs = self
            .classpath
            .get(&class)
            .and_then(|known| known.outer.clone());
        if let Some(enclosing) = &belongs {
            if let Some(written) = outer {
                let found = self.value(written, line)?;
                if !found.is_reference() {
                    return Err(at(
                        "EJ252",
                        line,
                        1,
                        format!(
                            "`new` on an instance wants an object, and a {} is not one.",
                            found.readable()
                        ),
                    ));
                }
            } else if !self.an_instance_of(enclosing)? {
                return Err(at(
                    "EJ252",
                    line,
                    1,
                    format!(
                        "`{}` belongs to an instance of `{}`, and there is none here.",
                        class.replace('/', "."),
                        enclosing.replace('/', ".")
                    ),
                )
                .with_suggestion(
                    "Write it `static` if it does not need one, or make it from inside \
                     the class it belongs to.",
                ));
            }
        }

        let signature = if belongs.is_some() {
            self.constructor_taking(&class, arguments.len() + 1)
        } else {
            self.signature_for(&class, "<init>", arguments, line)?
        };
        let mut target = target;
        let descriptor = match signature {
            Some(found) if belongs.is_none() => {
                if let Type::Object(named, held) = &target {
                    if held.is_empty()
                        && found
                            .taken_stands_for
                            .iter()
                            .any(|one| matches!(one, Standing::OfTheClass(_)))
                    {
                        let object = Type::Object("java/lang/Object".to_string(), Vec::new());
                        let mut inferred: Vec<Type> = Vec::new();
                        for (at, standing) in found.taken_stands_for.iter().enumerate() {
                            let Standing::OfTheClass(which) = standing else {
                                continue;
                            };
                            let Some(argument) = arguments.get(at) else {
                                continue;
                            };
                            if matches!(argument, Expression::Null) {
                                continue;
                            }
                            let outside = self.expecting.take();
                            let seen = self.type_of(argument, line).ok();
                            self.expecting = outside;
                            let Some(seen) = seen else {
                                continue;
                            };
                            let seen = match boxed_name(&seen) {
                                Some(boxed) => Type::Object(boxed.to_string(), Vec::new()),
                                None => seen,
                            };
                            while inferred.len() <= *which {
                                inferred.push(object.clone());
                            }
                            inferred[*which] = seen;
                        }
                        if inferred.iter().any(|one| *one != object) {
                            target = Type::Object(named.clone(), inferred);
                        }
                    }
                }
                self.arguments_for_signature(&found, arguments, line)?;
                found.descriptor()
            }
            Some(found) => {
                self.what_the_call_can_throw(&found, line)?;

                let wanted = if found.parameters.is_empty() {
                    &found.parameters[..]
                } else {
                    &found.parameters[1..]
                };
                self.arguments_for(wanted, arguments, line)?;
                found.descriptor()
            }
            None if arguments.is_empty() && WELL_KNOWN.contains(&class.as_str()) => {
                "()V".to_string()
            }
            None => {
                return Err(at(
                    "EJ219",
                    line,
                    1,
                    format!(
                    "`{}` has no constructor taking {} argument(s) that this compilation knows.",
                    class.replace('/', "."),
                    arguments.len()
                ),
                )
                .with_suggestion("Hand the class file that declares it over as a dependency."))
            }
        };
        let init = self.pool.method(&class, "<init>", &descriptor, false);
        self.op2(0xb7, init);
        let taken: i32 = read_descriptor(&descriptor)
            .map(|(parameters, _)| parameters.iter().map(|p| i32::from(p.width())).sum())
            .unwrap_or(0);
        self.pops(taken + 1);

        self.now_initialized(made_at, &class);
        Ok(target)
    }

    fn new_local_object(
        &mut self,
        held: &Locally,
        arguments: &[Expression],
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let target = Type::Object(held.internal.clone(), Vec::new());
        let index = self.pool.class(&held.internal);
        let made_at = self.code.len() as u16;
        self.op2(0xbb, index);
        self.pushes_raw(Verified::Uninitialized(made_at));
        self.duplicates();

        let mut descriptor = String::from("(");
        if let Some(enclosing) = &held.outer {
            let what = Type::Object(enclosing.clone(), Vec::new());
            if held.through_the_one_around {
                self.reach_the_enclosing_instance(enclosing)?;
            } else {
                self.load(0, &what);
            }
            descriptor.push_str(&what.descriptor());
        }
        for local in &held.captured {
            self.load(local.slot, &local.what);
            descriptor.push_str(&local.what.descriptor());
        }

        let unwritten = usize::from(held.outer.is_some()) + held.captured.len();
        let all = arguments.len() + unwritten;

        let written_here = self
            .made
            .iter()
            .find(|one| one.internal_name() == held.internal)
            .and_then(|one| {
                one.methods
                    .iter()
                    .find(|method| method.constructor && method.parameters.len() == all)
                    .cloned()
            });
        let wanted = match &written_here {
            Some(method) => {
                let mut parameters = Vec::new();
                for (what, _) in &method.parameters {
                    parameters.push(self.resolve(what, line)?);
                }
                Some(Signature {
                    owner: held.internal.clone(),
                    name: "<init>".to_string(),
                    parameters,
                    returns: Type::Void,
                    static_: false,
                    interface: false,
                    variadic: method.variadic,
                    abstract_: false,
                    returns_variable: None,
                    parameter_variables: Vec::new(),
                    taken_stands_for: Vec::new(),
                    returns_stands_for: Standing::Fixed,
                    own_variables: Vec::new(),
                    witnessed: Vec::new(),
                    throws: Vec::new(),
                })
            }
            None => self
                .classpath
                .find_methods(&held.internal, "<init>", all)
                .into_iter()
                .next(),
        };
        match wanted {
            Some(found) => {
                let mine = found.parameters[unwritten.min(found.parameters.len())..].to_vec();
                self.arguments_for(&mine, arguments, line)?;
                for one in &mine {
                    descriptor.push_str(&one.descriptor());
                }
            }
            None if arguments.is_empty() => {}
            None => {
                return Err(at(
                    "EJ219",
                    line,
                    1,
                    format!(
                        "`{}` has no constructor taking {} argument(s).",
                        held.written,
                        arguments.len()
                    ),
                ))
            }
        }
        descriptor.push_str(")V");

        let init = self
            .pool
            .method(&held.internal, "<init>", &descriptor, false);
        self.op2(0xb7, init);
        let taken: i32 = read_descriptor(&descriptor)
            .map(|(parameters, _)| parameters.iter().map(|one| i32::from(one.width())).sum())
            .unwrap_or(0);
        self.pops(taken + 1);
        self.now_initialized(made_at, &held.internal);
        Ok(target)
    }

    fn now_initialized(&mut self, made_at: u16, class: &str) {
        let settled = Verified::Object(class.to_string());
        for held in self.stack.iter_mut().chain(self.slots.iter_mut()) {
            if *held == Verified::Uninitialized(made_at) {
                *held = settled.clone();
            }
        }
    }

    fn what_the_call_can_throw(
        &mut self,
        signature: &Signature,
        line: u32,
    ) -> Result<(), Diagnostic> {
        for named in signature.throws.clone() {
            self.it_can_throw(&named, line)?;
        }
        Ok(())
    }

    fn arguments_for_signature(
        &mut self,
        signature: &Signature,
        given: &[Expression],
        line: u32,
    ) -> Result<(), Diagnostic> {
        self.what_the_call_can_throw(signature, line)?;
        let wanted = &signature.parameters;
        if !signature.variadic || wanted.is_empty() {
            return self.arguments_for(wanted, given, line);
        }
        let fixed = wanted.len() - 1;
        let Some(Type::Array(element)) = wanted.last().cloned() else {
            return self.arguments_for(wanted, given, line);
        };

        if given.len() == wanted.len() {
            if let Some(last) = given.last() {
                let peeked = self.type_of(last, line)?;

                if self.reaches(&peeked, &Type::Array(element.clone())) {
                    return self.arguments_for(wanted, given, line);
                }
            }
        }
        if given.len() < fixed {
            return Err(at(
                "EJ220",
                line,
                1,
                format!(
                    "That takes at least {fixed} argument(s) and was given {}.",
                    given.len()
                ),
            ));
        }

        self.arguments_for(&wanted[..fixed], &given[..fixed], line)?;
        self.array_of(&Type::Array(element), &given[fixed..], line)?;
        Ok(())
    }

    fn arguments_for(
        &mut self,
        wanted: &[Type],
        given: &[Expression],
        line: u32,
    ) -> Result<(), Diagnostic> {
        if wanted.len() != given.len() {
            return Err(at(
                "EJ220",
                line,
                1,
                format!(
                    "That takes {} argument(s) and was given {}.",
                    wanted.len(),
                    given.len()
                ),
            ));
        }
        for (want, expression) in wanted.iter().zip(given.iter()) {
            let found = self.value_for(expression, want, line)?;
            if !self.fit(&found, want, line)? {
                return Err(at(
                    "EJ221",
                    line,
                    1,
                    format!(
                        "A {} cannot be given where a {} is wanted.",
                        found.readable(),
                        want.readable()
                    ),
                ));
            }
        }
        Ok(())
    }

    fn call(
        &mut self,
        on: Option<&Expression>,
        super_call: bool,
        name: &str,
        arguments: &[Expression],
        held: &[Written],
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let going = self.expecting.clone();

        if super_call && on.is_none() && self.unit.stands_for_a_lambda && !self.static_ {
            if let Some(around) = self.unit.outer.clone() {
                return self.reaching_up_from_a_lambda(&around, name, arguments, held, going, line);
            }
        }

        if super_call {
            if self.static_ {
                return Err(at(
                    "EJ222",
                    line,
                    1,
                    "`super` has no meaning in a static method.",
                ));
            }

            let owner = match on.and_then(written_as_a_path) {
                Some(named) => self.resolve_class(&named, line)?,
                None => match self.unit.extends.clone() {
                    Some(parent) => self.resolve_class(&parent, line)?,
                    None => {
                        let above = self
                            .classpath
                            .get(&self.this_class)
                            .and_then(|known| known.superclass.clone())
                            .filter(|_| self.unit.shape != Shape::Interface);
                        let Some(above) = above else {
                            return Err(at(
                                "EJ223",
                                line,
                                1,
                                "This class extends nothing to call up into.",
                            ));
                        };
                        above
                    }
                },
            };
            let Some(signature) =
                self.signature_written_out(&owner, name, arguments, held, line)?
            else {
                return Err(self.no_such_method(&owner, name, arguments.len(), line));
            };
            self.load(0, &Type::Object(self.this_class.clone(), Vec::new()));
            let first = with_what_the_arguments_say(&signature, &[], &[], going.as_ref());
            let given = self.what_was_handed_over(&first, arguments, line)?;
            let shaped = with_what_the_arguments_say(&signature, &[], &given, going.as_ref());
            self.arguments_for_signature(&shaped, arguments, line)?;
            let descriptor = signature.descriptor();

            let through = self
                .classpath
                .get(&owner)
                .is_some_and(|known| known.interface);
            let index = self.pool.method(&owner, name, &descriptor, through);
            self.op2(0xb7, index);
            let taken: i32 = signature
                .parameters
                .iter()
                .map(|p| i32::from(p.width()))
                .sum();
            self.pops(taken + 1);
            self.pushes(&signature.returns);
            return Ok(signature.returns);
        }

        let on = match on {
            Some(Expression::This) => None,
            other => other,
        };
        let Some(on) = on else {
            let mine: &[Method] = if self.unit.stands_for_a_lambda && !self.writing_a_bridge {
                &[]
            } else {
                &self.unit.methods
            };
            let mut theirs: Vec<Method> = mine
                .iter()
                .filter(|held| held.name == name && held.parameters.len() == arguments.len())
                .cloned()
                .collect();

            if theirs.is_empty() {
                theirs = mine
                    .iter()
                    .filter(|held| {
                        held.name == name
                            && held.variadic
                            && !held.parameters.is_empty()
                            && held.parameters.len() - 1 <= arguments.len()
                    })
                    .cloned()
                    .collect();
            }
            let inherited = self
                .unit
                .extends
                .as_ref()
                .and_then(|parent| resolve_named(self.classpath, self.unit, parent))
                .is_some_and(|parent| {
                    !self
                        .classpath
                        .find_methods(&parent, name, arguments.len())
                        .is_empty()
                });
            let own = self.which_one_of_ours(&theirs, arguments, inherited, line)?;
            let Some(own) = own else {
                if let Some(parent) = self.unit.extends.clone() {
                    let owner = self.resolve_class(&parent, line)?;
                    if let Some(signature) =
                        self.signature_written_out(&owner, name, arguments, held, line)?
                    {
                        if !signature.static_ {
                            if self.static_ {
                                return Err(at(
                                    "EJ225",
                                    line,
                                    1,
                                    format!(
                                        "`{name}` belongs to an instance and this method \
                                         is static."
                                    ),
                                ));
                            }
                            self.load(0, &Type::Object(self.this_class.clone(), Vec::new()));
                        }
                        let first =
                            with_what_the_arguments_say(&signature, &[], &[], going.as_ref());
                        let given = self.what_was_handed_over(&first, arguments, line)?;
                        let shaped =
                            with_what_the_arguments_say(&signature, &[], &given, going.as_ref());
                        self.arguments_for_signature(&shaped, arguments, line)?;

                        let owner = if signature.static_ {
                            signature.owner.clone()
                        } else {
                            self.this_class.clone()
                        };
                        let descriptor = signature.descriptor();
                        let index = self.pool.method(&owner, name, &descriptor, false);
                        self.op2(if signature.static_ { 0xb8 } else { 0xb6 }, index);
                        let taken: i32 = signature
                            .parameters
                            .iter()
                            .map(|one| i32::from(one.width()))
                            .sum();
                        let popped = taken + i32::from(!signature.static_);
                        self.pops(popped);
                        self.pushes(&signature.returns);
                        return self.what_it_really_gives_back(
                            &signature,
                            &[],
                            &given,
                            going.as_ref(),
                            line,
                        );
                    }
                }
                for enclosing in self.the_classes_around() {
                    if let Some(signature) =
                        self.signature_written_out(&enclosing, name, arguments, held, line)?
                    {
                        if !signature.static_ {
                            self.reach_the_enclosing_instance(&enclosing)?;
                        }
                        let first =
                            with_what_the_arguments_say(&signature, &[], &[], going.as_ref());
                        let given = self.what_was_handed_over(&first, arguments, line)?;
                        let shaped =
                            with_what_the_arguments_say(&signature, &[], &given, going.as_ref());
                        self.arguments_for_signature(&shaped, arguments, line)?;
                        let taken: i32 = signature
                            .parameters
                            .iter()
                            .map(|one| i32::from(one.width()))
                            .sum();
                        let descriptor = signature.descriptor();
                        let index = self.pool.method(
                            &signature.owner,
                            name,
                            &descriptor,
                            signature.interface,
                        );

                        match (signature.static_, signature.interface) {
                            (true, _) => self.op2(0xb8, index),
                            (false, false) => self.op2(0xb6, index),
                            (false, true) => {
                                self.code.push(0xb9);
                                self.code.extend_from_slice(&index.to_be_bytes());
                                self.code.push((taken + 1) as u8);
                                self.code.push(0);
                            }
                        }
                        let popped = taken + i32::from(!signature.static_);
                        self.pops(popped);
                        self.pushes(&signature.returns);
                        return self.what_it_really_gives_back(
                            &signature,
                            &[],
                            &given,
                            going.as_ref(),
                            line,
                        );
                    }
                }

                for owner in self.imported_statically(name) {
                    let Some(signature) =
                        self.signature_written_out(&owner, name, arguments, held, line)?
                    else {
                        continue;
                    };
                    if !signature.static_ {
                        continue;
                    }
                    let first = with_what_the_arguments_say(&signature, &[], &[], going.as_ref());
                    let given = self.what_was_handed_over(&first, arguments, line)?;
                    let shaped =
                        with_what_the_arguments_say(&signature, &[], &given, going.as_ref());
                    self.arguments_for_signature(&shaped, arguments, line)?;
                    let descriptor = signature.descriptor();
                    let index =
                        self.pool
                            .method(&signature.owner, name, &descriptor, signature.interface);
                    self.op2(0xb8, index);
                    let taken: i32 = signature
                        .parameters
                        .iter()
                        .map(|one| i32::from(one.width()))
                        .sum();
                    self.pops(taken);
                    self.pushes(&signature.returns);
                    return self.what_it_really_gives_back(
                        &signature,
                        &[],
                        &given,
                        going.as_ref(),
                        line,
                    );
                }

                let mine = self.this_class.clone();
                if let Some(signature) =
                    self.signature_written_out(&mine, name, arguments, held, line)?
                {
                    if !signature.static_ {
                        if self.static_ {
                            return Err(at(
                                "EJ225",
                                line,
                                1,
                                format!(
                                    "`{name}` belongs to an instance and this method is \
                                     static."
                                ),
                            ));
                        }
                        self.load(0, &Type::Object(mine.clone(), Vec::new()));
                    }
                    let first = with_what_the_arguments_say(&signature, &[], &[], going.as_ref());
                    let given = self.what_was_handed_over(&first, arguments, line)?;
                    let shaped =
                        with_what_the_arguments_say(&signature, &[], &given, going.as_ref());
                    self.arguments_for_signature(&shaped, arguments, line)?;
                    let taken: i32 = signature
                        .parameters
                        .iter()
                        .map(|one| i32::from(one.width()))
                        .sum();
                    let descriptor = signature.descriptor();

                    let through = self.unit.shape == Shape::Interface;
                    let owner = if signature.static_ {
                        signature.owner.clone()
                    } else {
                        mine.clone()
                    };
                    let index = self.pool.method(&owner, name, &descriptor, through);
                    match (signature.static_, through) {
                        (true, _) => self.op2(0xb8, index),
                        (false, false) => self.op2(0xb6, index),
                        (false, true) => {
                            self.code.push(0xb9);
                            self.code.extend_from_slice(&index.to_be_bytes());
                            self.code.push((taken + 1) as u8);
                            self.code.push(0);
                        }
                    }
                    self.pops(taken + i32::from(!signature.static_));
                    self.pushes(&signature.returns);
                    return self.what_it_really_gives_back(
                        &signature,
                        &[],
                        &given,
                        going.as_ref(),
                        line,
                    );
                }

                for holder in self.holders() {
                    let Some(signature) =
                        self.signature_written_out(&holder, name, arguments, held, line)?
                    else {
                        continue;
                    };
                    if !signature.static_ {
                        continue;
                    }
                    let first = with_what_the_arguments_say(&signature, &[], &[], going.as_ref());
                    let given = self.what_was_handed_over(&first, arguments, line)?;
                    let shaped =
                        with_what_the_arguments_say(&signature, &[], &given, going.as_ref());
                    self.arguments_for_signature(&shaped, arguments, line)?;
                    let descriptor = signature.descriptor();
                    let index =
                        self.pool
                            .method(&signature.owner, name, &descriptor, signature.interface);
                    self.op2(0xb8, index);
                    let taken: i32 = signature
                        .parameters
                        .iter()
                        .map(|one| i32::from(one.width()))
                        .sum();
                    self.pops(taken);
                    self.pushes(&signature.returns);
                    return self.what_it_really_gives_back(
                        &signature,
                        &[],
                        &given,
                        going.as_ref(),
                        line,
                    );
                }
                return Err(at(
                    "EJ224",
                    line,
                    1,
                    format!(
                        "`{name}` is not a method of this class or of one above it taking \
                         {} argument(s).",
                        arguments.len()
                    ),
                )
                .with_suggestion(
                    "Hand the class file that declares it over as a dependency, or write \
                     the method here.",
                ));
            };
            let mut parameters = Vec::new();
            for (what, _) in &own.parameters {
                parameters.push(self.resolve(what, line)?);
            }
            let returns = self.resolve(&own.returns, line)?;
            let signature = Signature {
                owner: self.this_class.clone(),
                name: name.to_string(),
                parameters: parameters.clone(),
                returns: returns.clone(),
                static_: own.modifiers.static_,
                interface: false,
                variadic: own.variadic,
                abstract_: false,

                returns_variable: which_variable(&own.returns),
                parameter_variables: own
                    .parameters
                    .iter()
                    .map(|(what, _)| which_variable(what))
                    .collect(),
                taken_stands_for: own
                    .parameters
                    .iter()
                    .map(|(what, _)| standing_of(what))
                    .collect(),
                returns_stands_for: standing_of(&own.returns),
                own_variables: own.own_variables.clone(),
                witnessed: Vec::new(),

                throws: own
                    .throws
                    .iter()
                    .filter_map(|one| self.resolve_class(one, line).ok())
                    .collect(),
            };
            let descriptor = signature.descriptor();

            if !own.modifiers.static_ {
                if self.static_ {
                    return Err(at(
                        "EJ225",
                        line,
                        1,
                        format!("`{name}` belongs to an instance and this method is static."),
                    ));
                }
                self.load(0, &Type::Object(self.this_class.clone(), Vec::new()));
            }

            let first = with_what_the_arguments_say(&signature, &[], &[], going.as_ref());
            let given = self.what_was_handed_over(&first, arguments, line)?;
            let shaped = with_what_the_arguments_say(&signature, &[], &given, going.as_ref());
            self.arguments_for_signature(&shaped, arguments, line)?;
            let owner = self.this_class.clone();
            let inside_an_interface = self.unit.shape == Shape::Interface;
            let index = self
                .pool
                .method(&owner, name, &descriptor, inside_an_interface);

            let opcode = match (
                own.modifiers.static_,
                inside_an_interface,
                own.modifiers.private,
            ) {
                (true, _, _) => 0xb8u8,
                (false, true, true) => 0xb7,
                (false, true, false) => 0xb9,
                (false, false, _) => 0xb6,
            };
            let taken: i32 = parameters.iter().map(|p| i32::from(p.width())).sum();
            if opcode == 0xb9 {
                self.code.push(0xb9);
                self.code.extend_from_slice(&index.to_be_bytes());
                self.code.push((taken + 1) as u8);
                self.code.push(0);
            } else {
                self.op2(opcode, index);
            }
            let popped = taken + i32::from(!own.modifiers.static_);
            self.pops(popped);
            self.pushes(&returns);
            return self.what_it_really_gives_back(&signature, &[], &given, going.as_ref(), line);
        };

        if !matches!(on, Expression::Name(_)) {
            if let Some(path) = written_as_a_path(on) {
                let head = path.split('.').next().unwrap_or_default();
                if self.meant_as_a_class(head) {
                    if let Some(owner) = resolve_named(self.classpath, self.unit, &path) {
                        if let Some(signature) =
                            self.signature_written_out(&owner, name, arguments, held, line)?
                        {
                            if signature.static_ {
                                let first = with_what_the_arguments_say(
                                    &signature,
                                    &[],
                                    &[],
                                    going.as_ref(),
                                );
                                let given = self.what_was_handed_over(&first, arguments, line)?;
                                let shaped = with_what_the_arguments_say(
                                    &signature,
                                    &[],
                                    &given,
                                    going.as_ref(),
                                );
                                self.arguments_for_signature(&shaped, arguments, line)?;
                                let descriptor = signature.descriptor();
                                let index = self.pool.method(
                                    &signature.owner,
                                    name,
                                    &descriptor,
                                    signature.interface,
                                );
                                self.op2(0xb8, index);
                                let taken: i32 = signature
                                    .parameters
                                    .iter()
                                    .map(|one| i32::from(one.width()))
                                    .sum();
                                self.pops(taken);
                                self.pushes(&signature.returns);
                                return self.what_it_really_gives_back(
                                    &signature,
                                    &[],
                                    &given,
                                    going.as_ref(),
                                    line,
                                );
                            }
                        }
                    }
                }
            }
        }

        if let Expression::Name(maybe_class) = on {
            if self.meant_as_a_class(maybe_class) {
                let owner = self.resolve_class(maybe_class, line)?;
                let Some(signature) =
                    self.signature_written_out(&owner, name, arguments, held, line)?
                else {
                    return Err(self.no_such_method(&owner, name, arguments.len(), line));
                };
                if !signature.static_ {
                    return Err(at(
                        "EJ234",
                        line,
                        1,
                        format!(
                            "`{name}` belongs to an instance of `{}`, and this names the class.",
                            owner.replace('/', ".")
                        ),
                    ));
                }
                let first = with_what_the_arguments_say(&signature, &[], &[], going.as_ref());
                let given = self.what_was_handed_over(&first, arguments, line)?;
                let shaped = with_what_the_arguments_say(&signature, &[], &given, going.as_ref());
                self.arguments_for_signature(&shaped, arguments, line)?;
                let descriptor = signature.descriptor();

                let index =
                    self.pool
                        .method(&signature.owner, name, &descriptor, signature.interface);
                self.op2(0xb8, index);
                let taken: i32 = signature
                    .parameters
                    .iter()
                    .map(|p| i32::from(p.width()))
                    .sum();
                self.pops(taken);
                self.pushes(&signature.returns);
                return self.what_it_really_gives_back(
                    &signature,
                    &[],
                    &given,
                    going.as_ref(),
                    line,
                );
            }
        }

        let owner_type = self.value(on, line)?;
        if matches!(owner_type, Type::Array(_)) {
            return self.call_on_an_array(&owner_type, name, arguments, line);
        }
        let Type::Object(owner, on_the_receiver) = owner_type.clone() else {
            return Err(at(
                "EJ226",
                line,
                1,
                format!("A {} has no methods to call.", owner_type.readable()),
            ));
        };
        let Some(signature) = self.signature_written_out(&owner, name, arguments, held, line)?
        else {
            return Err(self.no_such_method(&owner, name, arguments.len(), line));
        };

        let held = self
            .classpath
            .seen_from(&owner, &on_the_receiver, &signature.owner);
        let mut shaped = as_written(&signature, &held);

        shaped.witnessed = signature.witnessed.clone();

        let first = with_what_the_arguments_say(&shaped, &held, &[], going.as_ref());
        let given = self.what_was_handed_over(&first, arguments, line)?;
        let shaped = with_what_the_arguments_say(&shaped, &held, &given, going.as_ref());
        self.arguments_for_signature(&shaped, arguments, line)?;

        let descriptor = signature.descriptor();
        let index = self
            .pool
            .method(&signature.owner, name, &descriptor, signature.interface);
        if signature.interface {
            let taken: i32 = signature
                .parameters
                .iter()
                .map(|p| i32::from(p.width()))
                .sum();
            self.code.push(0xb9);
            self.code.extend_from_slice(&index.to_be_bytes());
            self.code.push((taken + 1) as u8);
            self.code.push(0);
            self.pops(taken + 1);
            self.pushes(&signature.returns);
        } else {
            self.op2(0xb6, index);
            let taken: i32 = signature
                .parameters
                .iter()
                .map(|p| i32::from(p.width()))
                .sum();
            self.pops(taken + 1);
            self.pushes(&signature.returns);
        }

        self.what_it_really_gives_back(&signature, &held, &given, going.as_ref(), line)
    }

    fn value_for(
        &mut self,
        expression: &Expression,
        wanted: &Type,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        if let Expression::ArrayOf {
            of: None,
            values,
            line: written,
        } = expression
        {
            let values = values.clone();
            return self.array_of(wanted, &values, *written);
        }

        if matches!(wanted, Type::Byte | Type::Short | Type::Char) {
            let folded = constant_int(expression).or_else(|| {
                self.case_constant(expression)
                    .as_ref()
                    .and_then(constant_int)
            });
            if let Some(value) = folded {
                let fits = match wanted {
                    Type::Byte => (-128..=127).contains(&value),
                    Type::Short => (-32768..=32767).contains(&value),
                    _ => (0..=65535).contains(&value),
                };
                if fits {
                    self.push_int(i64::from(value));
                    return Ok(wanted.clone());
                }
            }
        }
        self.expecting = Some(wanted.clone());
        let found = self.value(expression, line);
        self.expecting = None;
        found
    }

    fn type_of(&mut self, expression: &Expression, line: u32) -> Result<Type, Diagnostic> {
        let made = self.made.len();
        let code = self.code.len();
        let depth = self.depth;
        let max_depth = self.max_depth;
        let stack = self.stack.clone();
        let frames = self.frames.len();
        let locals = self.locals.len();
        let next_slot = self.next_slot;
        let max_slot = self.max_slot;

        let slots = self.slots.clone();
        let handlers = self.handlers.len();
        let inlined = self.inlined.len();

        let found = self.value(expression, line);

        self.code.truncate(code);
        self.depth = depth;
        self.max_depth = max_depth;
        self.stack = stack;
        self.frames.truncate(frames);
        self.locals.truncate(locals);
        self.next_slot = next_slot;
        self.max_slot = max_slot;
        self.slots = slots;
        self.handlers.truncate(handlers);
        self.inlined.truncate(inlined);
        self.made.truncate(made);
        found
    }

    fn find_signature(&self, owner: &str, name: &str, count: usize) -> Option<Signature> {
        if let Some(found) = self.classpath.find_method(owner, name, count) {
            return Some(found.clone());
        }
        built_in_method(owner, name, count)
    }

    fn signature_written_out(
        &mut self,
        owner: &str,
        name: &str,
        arguments: &[Expression],
        held: &[Written],
        line: u32,
    ) -> Result<Option<Signature>, Diagnostic> {
        let Some(mut found) = self.signature_for(owner, name, arguments, line)? else {
            return Ok(None);
        };
        if held.is_empty() || found.own_variables.is_empty() {
            return Ok(Some(found));
        }
        let mut witnessed = Vec::new();
        for (name, written) in found.own_variables.iter().zip(held.iter()) {
            witnessed.push((name.clone(), self.resolve(written, line)?));
        }
        found.witnessed = witnessed;
        Ok(Some(found))
    }

    fn signature_for(
        &mut self,
        owner: &str,
        name: &str,
        arguments: &[Expression],
        line: u32,
    ) -> Result<Option<Signature>, Diagnostic> {
        let candidates = self.candidates(owner, name, arguments.len());
        if candidates.len() <= 1 {
            return Ok(candidates.into_iter().next());
        }

        let anything = self.how_loosely_here(arguments);
        let mut given = Vec::with_capacity(arguments.len());
        for (at, expression) in arguments.iter().enumerate() {
            if anything[at] != Loosely::No {
                given.push(Type::Object("java/lang/Object".to_string(), Vec::new()));
                continue;
            }
            given.push(self.type_of(expression, line)?);
        }

        for round in 0..3 {
            let mut best: Option<(u32, &Signature)> = None;
            for candidate in &candidates {
                if !self.could_take(candidate, &given, &anything, round) {
                    continue;
                }
                let cost = self.what_it_costs(candidate, &given, &anything);
                if best.is_none_or(|(held, _)| cost < held) {
                    best = Some((cost, candidate));
                }
            }
            if let Some((_, found)) = best {
                return Ok(Some(found.clone()));
            }
        }
        Ok(candidates.into_iter().next())
    }

    fn what_it_really_gives_back(
        &mut self,
        signature: &Signature,
        held: &[Type],
        given: &[Type],
        going: Option<&Type>,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let declared = signature.returns.clone();
        let going = going.filter(|wanted| {
            let (Type::Object(asked, _), Type::Object(limit, _)) = (wanted, &declared) else {
                return true;
            };
            !matches!(signature.returns_stands_for, Standing::OfTheMethod(_))
                || limit == "java/lang/Object"
                || self.classpath.get(asked).is_none()
                || self.classpath.get(limit).is_none()
                || self.reaches(wanted, &declared)
        });

        if !signature.returns_stands_for.says_nothing() {
            let bound = what_the_call_says(signature, given, going);
            if !bound.is_empty() {
                let found = signature.returns_stands_for.filled(&declared, held, &bound);

                if found == declared {
                    return Ok(found);
                }
                return self.cast_to(found, declared);
            }
        }

        if !held.is_empty() && signature.returns_stands_for.mentions_the_class() {
            let found = signature.returns_stands_for.filled(&declared, held, &[]);
            if found == declared {
                return Ok(found);
            }
            return self.cast_to(found, declared);
        }

        if !held.is_empty() {
            if let Some((_, _, around, inside)) = ELEMENT_ENTRIES
                .iter()
                .find(|(class, name, _, _)| *class == signature.owner && *name == signature.name)
            {
                return Ok(Type::Object(
                    (*around).to_string(),
                    vec![Type::Object((*inside).to_string(), held.to_vec())],
                ));
            }
        }

        let (which, wrap) = match signature.returns_variable {
            Some(at) => (at, None),
            None => match ELEMENT_RETURN
                .iter()
                .find(|(class, name, _, _)| *class == signature.owner && *name == signature.name)
            {
                Some((_, _, which, wrap)) => (*which, *wrap),
                None => return Ok(declared),
            },
        };
        let Some(argument) = held.get(which) else {
            return Ok(declared);
        };

        if let Some(around) = wrap {
            return Ok(Type::Object(around.to_string(), vec![argument.clone()]));
        }
        let _ = line;
        self.cast_to(argument.clone(), declared)
    }

    fn what_was_handed_over(
        &mut self,
        signature: &Signature,
        arguments: &[Expression],
        line: u32,
    ) -> Result<Vec<Type>, Diagnostic> {
        if signature.returns_stands_for.says_nothing()
            && signature
                .taken_stands_for
                .iter()
                .all(Standing::says_nothing)
        {
            return Ok(Vec::new());
        }
        let mut given = Vec::with_capacity(arguments.len());

        let outside = self.expecting.take();
        let last = signature.parameters.len().saturating_sub(1);
        for (at, one) in arguments.iter().enumerate() {
            if matches!(
                one,
                Expression::Lambda { .. } | Expression::MethodRef { .. }
            ) {
                let want = signature.parameters.get(at.min(last)).cloned();
                let said = want
                    .filter(|_| at <= last)
                    .and_then(|want| self.what_it_would_come_to(one, &want, line));
                given.push(
                    said.unwrap_or_else(|| {
                        Type::Object("java/lang/Object".to_string(), Vec::new())
                    }),
                );
                continue;
            }

            self.expecting = match signature.parameters.get(at.min(last)) {
                Some(Type::Array(element)) if signature.variadic && at >= last => {
                    Some((**element).clone())
                }
                Some(found) if at <= last => Some(found.clone()),
                _ => None,
            };
            match self.type_of(one, line) {
                Ok(found) => given.push(self.as_the_parameter_sees_it(signature, at, found)),
                Err(_) => given.push(Type::Object("java/lang/Object".to_string(), Vec::new())),
            }
        }
        self.expecting = outside;
        Ok(given)
    }

    fn as_the_parameter_sees_it(&self, signature: &Signature, at: usize, found: Type) -> Type {
        let (Some(Type::Object(wanted, _)), Some(Standing::Holding(_)), Type::Object(have, held)) = (
            signature.parameters.get(at),
            signature.taken_stands_for.get(at),
            &found,
        ) else {
            return found;
        };
        if wanted == have || !self.classpath.ancestors(have).contains(wanted) {
            return found;
        }
        let seen = self.classpath.seen_from(have, held, wanted);
        if seen.is_empty() {
            return found;
        }
        Type::Object(wanted.clone(), seen)
    }

    fn what_it_would_come_to(
        &mut self,
        written: &Expression,
        want: &Type,
        line: u32,
    ) -> Option<Type> {
        let Type::Object(named, held) = want else {
            return None;
        };
        let sam = self.the_one_method_of(named, line).ok()?;

        let Standing::OfTheClass(which) = sam.returns_stands_for else {
            return None;
        };
        let shaped = as_written(&sam, held);
        let produced = match written {
            Expression::Lambda {
                parameters,
                body,
                expression,
                ..
            } => {
                if parameters.len() != shaped.parameters.len() {
                    return None;
                }

                let returns = if *expression {
                    match body.first().map(|held| &held.node) {
                        Some(Statement::Express(value)) => value.clone(),
                        _ => return None,
                    }
                } else {
                    let mut found = Vec::new();
                    for held in body.iter() {
                        values_returned(&held.node, &mut found);
                    }
                    found
                        .iter()
                        .find(|one| !matches!(one, Expression::Null))
                        .or_else(|| found.first())
                        .cloned()?
                };
                self.open();
                for ((written, name), what) in parameters.iter().zip(shaped.parameters.iter()) {
                    let what = match written {
                        Some(written) => self.resolve(written, line).unwrap_or(what.clone()),
                        None => what.clone(),
                    };
                    self.declare(name, what);
                }
                let said = std::mem::replace(
                    &mut self.said_to_throw,
                    vec!["java/lang/Throwable".to_string()],
                );
                if !*expression {
                    let mut declared: Vec<(Written, String, Option<Expression>)> = Vec::new();
                    for one in body.iter() {
                        locals_declared(&one.node, &mut declared);
                    }
                    for (what, name, value) in declared {
                        let found = match (&what, &value) {
                            (Written::Inferred, Some(value)) => self.type_of(value, line).ok(),
                            (Written::Inferred, None) => None,
                            _ => self.resolve(&what, line).ok(),
                        };
                        if let Some(found) = found {
                            self.declare(&name, found);
                        }
                    }
                }
                let found = self.type_of(&returns, line);
                self.said_to_throw = said;
                self.close();
                found.ok()?
            }
            Expression::MethodRef { on, name, .. } => {
                let owner = match on.as_ref() {
                    Expression::Name(maybe) if self.meant_as_a_class(maybe) => {
                        self.resolve_class(maybe, line).ok()?
                    }
                    other => match written_as_a_path(other)
                        .filter(|path| {
                            self.meant_as_a_class(path.split('.').next().unwrap_or_default())
                        })
                        .and_then(|path| resolve_named(self.classpath, self.unit, &path))
                    {
                        Some(found) => found,
                        None => match self.type_of(other, line).ok()? {
                            Type::Object(found, _) => found,
                            _ => return None,
                        },
                    },
                };
                if name == "new" {
                    Type::Object(owner, Vec::new())
                } else {
                    let takes = shaped.parameters.len();
                    let found = self
                        .find_signature(&owner, name, takes)
                        .or_else(|| self.find_signature(&owner, name, takes.saturating_sub(1)))?;
                    found.returns
                }
            }
            _ => return None,
        };
        if produced == Type::Void
            || produced == Type::Object("java/lang/Object".to_string(), Vec::new())
        {
            return None;
        }
        let mut filled = held.clone();
        while filled.len() <= which {
            filled.push(Type::Object("java/lang/Object".to_string(), Vec::new()));
        }
        filled[which] = produced;
        Some(Type::Object(named.clone(), filled))
    }

    fn cast_to(&mut self, found: Type, declared: Type) -> Result<Type, Diagnostic> {
        let named = match &found {
            Type::Object(named, _) if named == "java/lang/Object" => {
                return Ok(declared);
            }
            Type::Object(named, _) => named.clone(),
            Type::Array(_) => found.descriptor(),
            _ => return Ok(declared),
        };
        let index = self.pool.class(&named);
        self.op2(0xc0, index);
        self.pops(1);
        self.pushes(&found);
        Ok(found)
    }

    fn reaches(&self, have: &Type, want: &Type) -> bool {
        if let Type::Array(to) = want {
            return match have {
                Type::Array(from) => from == to || self.reaches(from, to),
                _ => false,
            };
        }
        if matches!(have, Type::Array(_)) {
            return matches!(want, Type::Object(named, _) if named == "java/lang/Object");
        }
        let (Type::Object(from, _), Type::Object(to, _)) = (have, want) else {
            return have.may_be_given_to(want);
        };
        if from == to || to == "java/lang/Object" {
            return true;
        }

        let mut seen = vec![from.clone()];
        let mut waiting = vec![from.clone()];
        while let Some(current) = waiting.pop() {
            if seen.len() > 64 {
                break;
            }
            let mut above: Vec<String> = self
                .classpath
                .ancestors(&current)
                .into_iter()
                .filter(|one| *one != current)
                .collect();
            above.extend(
                BUILT_IN_ABOVE
                    .iter()
                    .filter(|(below, _)| *below == current)
                    .map(|(_, held)| (*held).to_string()),
            );
            for one in above {
                if one == *to {
                    return true;
                }
                if !seen.contains(&one) {
                    seen.push(one.clone());
                    waiting.push(one);
                }
            }
        }

        !self.a_class_this_knows(from) || !self.a_class_this_knows(to)
    }

    fn a_class_this_knows(&self, name: &str) -> bool {
        self.classpath.get(name).is_some()
            || WELL_KNOWN.contains(&name)
            || BUILT_IN_INTERFACES.contains(&name)
            || BUILT_IN_THROWABLES.contains(&name)
            || BUILT_IN_ABOVE
                .iter()
                .any(|(below, above)| *below == name || *above == name)
    }

    fn candidates(&self, owner: &str, name: &str, count: usize) -> Vec<Signature> {
        let mut found = self.candidates_taking(owner, name, count);

        for one in self.variadic_taking(owner, name, count) {
            if !found.iter().any(|held| held.parameters == one.parameters) {
                found.push(one);
            }
        }
        found
    }

    fn which_one_of_ours(
        &mut self,
        theirs: &[Method],
        arguments: &[Expression],
        inherited: bool,
        line: u32,
    ) -> Result<Option<Method>, Diagnostic> {
        if theirs.is_empty() || (theirs.len() == 1 && !inherited) {
            return Ok(theirs.first().cloned());
        }

        let anything = self.how_loosely_here(arguments);
        let mut given = Vec::with_capacity(arguments.len());
        for (at, one) in arguments.iter().enumerate() {
            if anything[at] != Loosely::No {
                given.push(Type::Object("java/lang/Object".to_string(), Vec::new()));
                continue;
            }
            given.push(match self.type_of(one, line) {
                Ok(found) => found,
                Err(_) => Type::Object("java/lang/Object".to_string(), Vec::new()),
            });
        }
        let mut shapes = Vec::with_capacity(theirs.len());
        for held in theirs {
            let mut parameters = Vec::new();
            for (what, _) in &held.parameters {
                parameters.push(self.resolve(what, line)?);
            }
            shapes.push(Signature {
                owner: self.this_class.clone(),
                name: held.name.clone(),
                parameters,
                returns: Type::Void,
                static_: held.modifiers.static_,
                interface: false,
                variadic: held.variadic,
                abstract_: false,
                returns_variable: None,
                parameter_variables: Vec::new(),
                taken_stands_for: Vec::new(),
                returns_stands_for: Standing::Fixed,
                own_variables: Vec::new(),
                witnessed: Vec::new(),
                throws: Vec::new(),
            });
        }
        for round in 0..3 {
            let mut able: Vec<(u32, usize)> = Vec::new();
            for (at, shape) in shapes.iter().enumerate() {
                if !self.could_take(shape, &given, &anything, round) {
                    continue;
                }
                able.push((self.what_it_costs(shape, &given, &anything), at));
            }
            let Some(cheapest) = able.iter().map(|(cost, _)| *cost).min() else {
                continue;
            };
            able.retain(|(cost, _)| *cost == cheapest);
            if able.len() == 1 {
                return Ok(theirs.get(able[0].1).cloned());
            }

            let mut narrowest: Option<usize> = None;
            for (_, at) in &able {
                let mine = &shapes[*at];
                if able.iter().all(|(_, other)| {
                    *other == *at || self.at_least_as_specific(mine, &shapes[*other])
                }) {
                    narrowest = Some(*at);
                    break;
                }
            }
            if let Some(at) = narrowest {
                return Ok(theirs.get(at).cloned());
            }
            let named: Vec<String> = able
                .iter()
                .map(|(_, at)| {
                    let held = &shapes[*at];
                    format!(
                        "{}({})",
                        held.name,
                        held.parameters
                            .iter()
                            .map(|one| one.readable())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect();
            return Err(at(
                "EJ262",
                line,
                1,
                format!(
                    "`{}` here could be {} of these, and neither is more specific.",
                    shapes[able[0].1].name,
                    named.len()
                ),
            )
            .with_context(format!("Written: {}", named.join(" or ")))
            .with_suggestion(
                "Say which by casting an argument to the type that one takes. The \
                 language calls this ambiguous rather than choosing, and so does this.",
            ));
        }
        if inherited {
            return Ok(None);
        }
        Ok(theirs.first().cloned())
    }

    fn at_least_as_specific(&self, mine: &Signature, other: &Signature) -> bool {
        if mine.parameters.len() != other.parameters.len() {
            return false;
        }
        mine.parameters
            .iter()
            .zip(other.parameters.iter())
            .all(|(held, wanted)| held == wanted || self.reaches(held, wanted))
    }

    fn what_it_costs(&self, candidate: &Signature, given: &[Type], anything: &[Loosely]) -> u32 {
        let mut total = 0u32;
        let last = candidate.parameters.len().saturating_sub(1);
        for (at, have) in given.iter().enumerate() {
            if matches!(
                anything.get(at),
                Some(Loosely::Anywhere(_) | Loosely::Shaped(..))
            ) {
                continue;
            }
            let want = match candidate.parameters.get(at) {
                Some(found) => found,
                None => match candidate.parameters.last() {
                    Some(Type::Array(element)) if candidate.variadic => element.as_ref(),
                    _ => continue,
                },
            };

            let want = match want {
                Type::Array(element) if candidate.variadic && at >= last => element.as_ref(),
                other => other,
            };
            total = total.saturating_add(how_far(have, want));
        }

        total.saturating_add(u32::from(candidate.variadic))
    }

    fn could_take(
        &self,
        candidate: &Signature,
        given: &[Type],
        anything: &[Loosely],
        round: u8,
    ) -> bool {
        let matched = |at: usize, have: &Type, want: &Type| match anything.get(at) {
            Some(Loosely::Anywhere(counts)) => {
                counts.is_empty()
                    || match self.how_many_the_one_method_takes(want) {
                        Some(takes) => counts.contains(&takes),
                        None => true,
                    }
            }
            Some(Loosely::Shaped(count, gives)) => {
                let Type::Object(named, _) = want else {
                    return false;
                };
                match self.the_one_method_of(named, 1) {
                    Ok(single) => {
                        single.parameters.len() == *count
                            && match gives {
                                Gives::Nothing => single.returns == Type::Void,
                                Gives::Something => single.returns != Type::Void,
                                Gives::Either => true,
                            }
                    }
                    Err(_) => false,
                }
            }
            Some(Loosely::AnyReference) => want.is_reference(),
            _ => match round {
                0 => want == have,
                1 => self.reaches(have, want),
                _ => {
                    self.reaches(have, want)
                        || boxed_name(have)
                            .map(|boxed| Type::Object(boxed.to_string(), Vec::new()))
                            .is_some_and(|boxed| self.reaches(&boxed, want))
                        || primitive_inside(have).is_some_and(|inside| inside.may_be_given_to(want))
                }
            },
        };
        if !candidate.variadic {
            return candidate.parameters.len() == given.len()
                && candidate
                    .parameters
                    .iter()
                    .zip(given.iter())
                    .enumerate()
                    .all(|(at, (want, have))| matched(at, have, want));
        }
        let fixed = candidate.parameters.len().saturating_sub(1);
        if given.len() < fixed {
            return false;
        }
        if !candidate
            .parameters
            .iter()
            .take(fixed)
            .zip(given.iter())
            .enumerate()
            .all(|(at, (want, have))| matched(at, have, want))
        {
            return false;
        }
        let Some(last) = candidate.parameters.last() else {
            return false;
        };
        if given.len() == candidate.parameters.len() && matched(fixed, &given[fixed], last) {
            return true;
        }
        let Type::Array(element) = last else {
            return false;
        };
        given[fixed..]
            .iter()
            .enumerate()
            .all(|(at, have)| matched(fixed + at, have, element))
    }

    fn variadic_taking(&self, owner: &str, name: &str, count: usize) -> Vec<Signature> {
        let mut found = Vec::new();
        for ancestor in self.classpath.ancestors(owner) {
            if let Some(known) = self.classpath.get(&ancestor) {
                found.extend(
                    known
                        .methods
                        .iter()
                        .filter(|one| one.name == name && one.variadic)
                        .cloned(),
                );
            }
            for (class, held, descriptor, static_) in BUILT_IN_METHODS {
                if *class != ancestor
                    || *held != name
                    || !BUILT_IN_VARIADIC.contains(&(*class, *held))
                {
                    continue;
                }
                let Some((parameters, returns)) = read_descriptor(descriptor) else {
                    continue;
                };
                found.push(Signature {
                    owner: class.to_string(),
                    name: name.to_string(),
                    parameters,
                    returns,
                    static_: *static_,
                    interface: BUILT_IN_INTERFACES.contains(class),
                    variadic: true,
                    abstract_: BUILT_IN_INTERFACES.contains(class),
                    returns_variable: None,
                    parameter_variables: Vec::new(),
                    taken_stands_for: Vec::new(),
                    returns_stands_for: Standing::Fixed,
                    own_variables: Vec::new(),
                    witnessed: Vec::new(),
                    throws: Vec::new(),
                });
            }
            if !found.is_empty() {
                break;
            }
        }

        found.retain(|one| !one.parameters.is_empty() && one.parameters.len() - 1 <= count);
        found
    }

    fn candidates_taking(&self, owner: &str, name: &str, count: usize) -> Vec<Signature> {
        let found = self.classpath.find_methods(owner, name, count);
        if !found.is_empty() {
            return found;
        }

        for ancestor in self.classpath.ancestors(owner) {
            let found = built_in_overloads(&ancestor, name, count);
            if !found.is_empty() {
                return found;
            }
        }
        Vec::new()
    }

    fn no_such_method(&self, owner: &str, name: &str, arity: usize, line: u32) -> Diagnostic {
        if ONLY_ON_THE_DESKTOP_METHODS.contains(&(owner, name)) {
            return only_on_the_desktop_diagnostic(
                &format!("{}.{name}", owner.replace(['/', '$'], ".")),
                line,
            );
        }
        at(
            "EJ227",
            line,
            1,
            format!(
                "`{}` has no method `{name}` taking {arity} argument(s) that this compilation knows.",
                owner.replace('/', ".")
            ),
        )
        .with_suggestion(
            "Hand the class file that declares it over as a dependency. Nothing is guessed: \
             a call written against a method nobody has seen would be a class file the \
             device refuses.",
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn assign_to_own_field(
        &mut self,
        field: &Field,
        name: &str,
        operator: Option<Binary>,
        value: &Expression,
        read: &Expression,
        line: u32,
        wanted: bool,
    ) -> Result<Type, Diagnostic> {
        let what = self.resolve(&field.what, line)?;
        let owner = self.this_class.clone();
        if !field.modifiers.static_ {
            if self.static_ {
                return Err(at(
                    "EJ210",
                    line,
                    1,
                    format!("`{name}` belongs to an instance and this method is static."),
                ));
            }
            let before = self.too_early;
            self.load(0, &Type::Object(owner.clone(), Vec::new()));
            if operator.is_none() && field.value.is_none() {
                self.too_early = before;
            }
        }
        let found = match operator {
            None => self.value_for(value, &what, line)?,
            Some(operator) => {
                let now = self.binary(operator, read, value, line)?;
                self.narrowed_by_the_operator(&now, &what, line)?
            }
        };
        if !self.fit(&found, &what, line)? {
            return Err(at(
                "EJ228",
                line,
                1,
                format!(
                    "A {} cannot be put in `{name}`, which is a {}.",
                    found.readable(),
                    what.readable()
                ),
            ));
        }

        self.open();
        let held = wanted.then(|| {
            if field.modifiers.static_ {
                self.op(if what.width() == 2 { 0x5c } else { 0x59 });
                self.pushes(&what);
                return None;
            }
            let slot = self.declare("$assigned", what.clone());
            self.store(slot, &what);
            self.load(slot, &what);
            Some(slot)
        });
        let descriptor = what.descriptor();
        let index = self.pool.field(&owner, name, &descriptor);
        if field.modifiers.static_ {
            self.op2(0xb3, index);
            self.pops(i32::from(what.width()));
        } else {
            self.op2(0xb5, index);
            self.pops(i32::from(what.width()) + 1);
        }
        if let Some(Some(slot)) = held {
            self.load(slot, &what);
        }
        self.close();
        Ok(what)
    }

    fn assign(
        &mut self,
        target: &Expression,
        operator: Option<Binary>,
        value: &Expression,
        line: u32,
        wanted: bool,
    ) -> Result<Type, Diagnostic> {
        match target {
            Expression::Name(name) if self.local(name).is_some() => {
                let local = self.local(name).unwrap();
                let found = match operator {
                    None => self.value_for(value, &local.what, line)?,
                    Some(operator) => {
                        let now =
                            self.binary(operator, &Expression::Name(name.clone()), value, line)?;
                        self.narrowed_by_the_operator(&now, &local.what, line)?
                    }
                };
                if !self.fit(&found, &local.what, line)? {
                    return Err(at(
                        "EJ228",
                        line,
                        1,
                        format!(
                            "A {} cannot be put in `{name}`, which is a {}.",
                            found.readable(),
                            local.what.readable()
                        ),
                    ));
                }
                if wanted {
                    self.op(if local.what.width() == 2 { 0x5c } else { 0x59 });
                    self.pushes(&local.what);
                }
                self.store(local.slot, &local.what);
                Ok(local.what)
            }
            Expression::Name(name) => {
                let own = self
                    .unit
                    .fields
                    .iter()
                    .find(|held| held.name == *name)
                    .cloned();
                let Some(field) = own else {
                    let above = self
                        .unit
                        .extends
                        .clone()
                        .and_then(|parent| self.resolve_class(&parent, line).ok())
                        .and_then(|parent| {
                            self.classpath
                                .find_field(&parent, name)
                                .map(|(holder, (_, _, static_))| (holder.name.clone(), *static_))
                        });
                    if let Some((holder, static_)) = above {
                        if static_ {
                            return self.assign_through_the_enclosing_instance(
                                &holder, name, operator, value, line, wanted,
                            );
                        }
                        if !self.static_ {
                            let through = Expression::Field {
                                of: Box::new(Expression::This),
                                name: name.clone(),
                            };
                            return self.assign(&through, operator, value, line, wanted);
                        }
                    }

                    for enclosing in self.the_classes_around() {
                        if self.classpath.find_field(&enclosing, name).is_some() {
                            return self.assign_through_the_enclosing_instance(
                                &enclosing, name, operator, value, line, wanted,
                            );
                        }
                    }

                    for holder in self.holders() {
                        let held = self
                            .classpath
                            .find_field(&holder, name)
                            .map(|(_, (_, _, static_))| *static_);
                        if held == Some(true) {
                            return self.assign_through_the_enclosing_instance(
                                &holder, name, operator, value, line, wanted,
                            );
                        }
                    }
                    return Err(at(
                        "EJ229",
                        line,
                        1,
                        format!("`{name}` is nothing that can be assigned to."),
                    ));
                };
                let read = Expression::Name(name.clone());
                self.assign_to_own_field(&field, name, operator, value, &read, line, wanted)
            }

            Expression::Field { of, name } => {
                if matches!(of.as_ref(), Expression::This) {
                    let own = self
                        .unit
                        .fields
                        .iter()
                        .find(|held| held.name == *name)
                        .cloned();
                    if let Some(field) = own {
                        let read = Expression::Field {
                            of: Box::new(Expression::This),
                            name: name.clone(),
                        };
                        return self.assign_to_own_field(
                            &field, name, operator, value, &read, line, wanted,
                        );
                    }
                }

                let named = match of.as_ref() {
                    Expression::Name(maybe) if self.meant_as_a_class(maybe) => {
                        Some(self.resolve_class(maybe, line)?)
                    }
                    other => written_as_a_path(other)
                        .filter(|path| {
                            self.meant_as_a_class(path.split('.').next().unwrap_or_default())
                        })
                        .and_then(|path| resolve_named(self.classpath, self.unit, &path)),
                };
                let through_a_value = named.is_none();
                let class = match named {
                    Some(named) => named,
                    None => {
                        let owner = self.value(of, line)?;
                        let Type::Object(class, _) = owner.clone() else {
                            return Err(at(
                                "EJ212",
                                line,
                                1,
                                format!("A {} has no fields.", owner.readable()),
                            ));
                        };
                        class
                    }
                };
                let Some((holder, (_, what, static_))) = self.classpath.find_field(&class, name)
                else {
                    return Err(at(
                        "EJ213",
                        line,
                        1,
                        format!(
                            "`{}` has no field called `{name}` that this compilation knows.",
                            class.replace('/', ".")
                        ),
                    )
                    .with_suggestion(
                        "Hand the class file that declares it over as a dependency.",
                    ));
                };
                let (holder, what, static_) = (holder.name.clone(), what.clone(), *static_);
                if static_ && through_a_value {
                    self.op(0x57);
                    self.pops(1);
                }
                if !static_ && !through_a_value {
                    return Err(at(
                        "EJ210",
                        line,
                        1,
                        format!(
                            "`{name}` belongs to an instance of `{}`, and this names \
                             the class.",
                            class.replace('/', ".")
                        ),
                    ));
                }
                let descriptor = what.descriptor();
                let index = self.pool.field(&holder, name, &descriptor);
                let found = match operator {
                    None => self.value_for(value, &what, line)?,

                    Some(operator) => {
                        if static_ {
                            self.op2(0xb2, index);
                            self.pushes(&what);
                        } else {
                            self.duplicates();
                            self.op2(0xb4, index);
                            self.pops(1);
                            self.pushes(&what);
                        }
                        let now = self.binary_with(operator, &what, value, line)?;
                        self.narrowed_by_the_operator(&now, &what, line)?
                    }
                };
                if !self.fit(&found, &what, line)? {
                    return Err(at(
                        "EJ228",
                        line,
                        1,
                        format!(
                            "A {} cannot be put in `{name}`, which is a {}.",
                            found.readable(),
                            what.readable()
                        ),
                    ));
                }

                self.open();
                let held = wanted.then(|| {
                    if static_ {
                        self.op(if what.width() == 2 { 0x5c } else { 0x59 });
                        self.pushes(&what);
                        return None;
                    }
                    let slot = self.declare("$assigned", what.clone());
                    self.store(slot, &what);
                    self.load(slot, &what);
                    Some(slot)
                });
                if static_ {
                    self.op2(0xb3, index);
                    self.pops(i32::from(what.width()));
                } else {
                    self.op2(0xb5, index);
                    self.pops(i32::from(what.width()) + 1);
                }
                if let Some(Some(slot)) = held {
                    self.load(slot, &what);
                }
                self.close();
                Ok(what)
            }
            Expression::Index { of, at: index } => {
                let array = self.value(of, line)?;
                let Type::Array(element) = array.clone() else {
                    return Err(at("EJ203", line, 1, "That is not an array."));
                };
                let found = self.value(index, line)?;
                self.convert(&found, &Type::Int, line)?;
                match operator {
                    Some(operator) => {
                        self.duplicates_two();
                        self.op(array_load(&element));
                        self.pops(2);
                        self.pushes(&element);
                        let now = self.binary_with(operator, &element, value, line)?;
                        self.narrowed_by_the_operator(&now, &element, line)?;
                    }
                    None => {
                        let given = self.value_for(value, &element, line)?;
                        if given.is_reference() != element.is_reference() {
                            self.fit(&given, &element, line)?;
                        } else if !given.is_reference() {
                            self.convert(&given, &element, line)?;
                        }
                    }
                }

                self.open();
                let held = wanted.then(|| {
                    let slot = self.declare("$assigned", (*element).clone());
                    self.store(slot, &element);
                    self.load(slot, &element);
                    slot
                });
                self.op(array_store(&element));
                self.pops(2 + i32::from(element.width()));
                if let Some(slot) = held {
                    self.load(slot, &element);
                }
                self.close();
                Ok(*element)
            }
            _ => Err(unsupported(line, 1, "Assigning to that")),
        }
    }

    fn step(
        &mut self,
        target: &Expression,
        by: i32,
        after: bool,
        line: u32,
        wanted: bool,
    ) -> Result<Type, Diagnostic> {
        if let Expression::Name(name) = target {
            if let Some(local) = self.local(name) {
                if local.what.is_int_like() {
                    if wanted && after {
                        self.load(local.slot, &local.what);
                    }
                    self.code.push(0x84);
                    self.code.push(local.slot as u8);
                    self.code.push(by as i8 as u8);
                    if wanted && !after {
                        self.load(local.slot, &local.what);
                    }
                    return Ok(local.what);
                }
            }
        }
        let operator = if by > 0 {
            Binary::Add
        } else {
            Binary::Subtract
        };
        let stepped = Expression::Int(i64::from(by.abs()));

        if !wanted || !after {
            let found = self.assign(target, Some(operator), &stepped, line, wanted)?;
            return Ok(if wanted { found } else { Type::Void });
        }

        self.open();
        let target = self.reached_once(target, line)?;
        let was = self.value(&target, line)?;
        let slot = self.declare("$was", was.clone());
        self.store(slot, &was);
        self.assign(&target, Some(operator), &stepped, line, false)?;
        self.load(slot, &was);
        self.close();
        Ok(was)
    }

    fn reached_once(&mut self, target: &Expression, line: u32) -> Result<Expression, Diagnostic> {
        match target {
            Expression::Index { of, at } => Ok(Expression::Index {
                of: Box::new(self.put_in_a_slot(of, "$of", line)?),
                at: Box::new(self.put_in_a_slot(at, "$at", line)?),
            }),
            Expression::Field { of, name } => Ok(Expression::Field {
                of: Box::new(self.put_in_a_slot(of, "$of", line)?),
                name: name.clone(),
            }),
            other => Ok(other.clone()),
        }
    }

    fn put_in_a_slot(
        &mut self,
        part: &Expression,
        called: &str,
        line: u32,
    ) -> Result<Expression, Diagnostic> {
        if reached_the_same_way_twice(part) {
            return Ok(part.clone());
        }
        let what = self.value(part, line)?;
        let slot = self.declare(called, what.clone());
        self.store(slot, &what);
        Ok(Expression::Name(called.to_string()))
    }
}

impl Emitter<'_> {
    fn enter(&mut self, loops: bool) {
        let label = self.pending_label.take();
        self.levels.push(Level {
            label,
            loops,
            breaks: Vec::new(),
            continues: Vec::new(),
            finallys: self.finallys.len(),
        });
    }

    fn leave(&mut self) -> Level {
        self.levels.pop().expect("a level was entered")
    }

    fn level_for(&self, label: Option<&str>, loops: bool) -> Option<usize> {
        self.levels.iter().rposition(|level| match label {
            Some(name) => level.label.as_deref() == Some(name) && (!loops || level.loops),
            None => !loops || level.loops,
        })
    }

    fn no_such_level(&self, label: Option<&str>, line: u32, word: &str) -> Diagnostic {
        match label {
            Some(name) => at(
                "EJ231",
                line,
                1,
                format!("`{word} {name};` names a label that is not around this."),
            ),
            None if word == "continue" => at("EJ231", line, 1, "`continue` is not inside a loop."),
            None => at(
                "EJ231",
                line,
                1,
                "`break` is not inside a loop or a `switch`.",
            ),
        }
    }

    fn run_finallys(&mut self, down_to: usize) -> Result<(), Diagnostic> {
        if self.finallys.len() <= down_to {
            return Ok(());
        }
        let pending = self.finallys.clone();
        let began = self.code.len();
        for index in (down_to..pending.len()).rev() {
            self.finallys.truncate(index);
            match &pending[index] {
                Cleanup::Block(body) => {
                    for one in body {
                        self.statement(one)?;
                    }
                }

                Cleanup::Unlock(slot, what) => {
                    let (slot, what) = (*slot, what.clone());
                    self.load(slot, &what);
                    self.op(0xc3);
                    self.pops(1);
                }
            }
        }
        self.finallys = pending;
        if self.code.len() > began {
            self.inlined.push((began, self.code.len(), down_to));
        }
        Ok(())
    }

    fn set_depth(&mut self, depth: i32) {
        debug_assert_eq!(depth, 0);
        self.stack.clear();
        self.deepest();
    }

    fn protect(
        &mut self,
        start: usize,
        end: usize,
        target: usize,
        class: Option<String>,
        depth: usize,
    ) {
        let mut pieces = vec![(start, end)];
        let inlined = self.inlined.clone();
        for (from, to, ran_from) in inlined {
            if ran_from > depth {
                continue;
            }
            let mut next = Vec::new();
            for (piece_start, piece_end) in pieces {
                if to <= piece_start || from >= piece_end {
                    next.push((piece_start, piece_end));
                    continue;
                }
                if piece_start < from {
                    next.push((piece_start, from));
                }
                if to < piece_end {
                    next.push((to, piece_end));
                }
            }
            pieces = next;
        }
        for (piece_start, piece_end) in pieces {
            if piece_start >= piece_end {
                continue;
            }
            self.handlers.push(Handler {
                start: piece_start,
                end: piece_end,
                target,
                class: class.clone(),
            });
        }
    }

    fn statement(&mut self, statement: &Positioned<Statement>) -> Result<(), Diagnostic> {
        let line = statement.line;

        let began = self.code.len() as u16;
        if line > 0 {
            match self.lines.last_mut() {
                Some((before, said)) if *before == began => *said = line as u16,

                Some((_, said)) if *said == line as u16 => {}
                _ => self.lines.push((began, line as u16)),
            }
        }
        match &statement.node {
            Statement::Nothing => Ok(()),
            Statement::Locally(units) => self.a_class_written_in_here(units, line),
            Statement::Several(inside) => {
                for one in inside {
                    self.statement(one)?;
                }
                Ok(())
            }
            Statement::Chain { .. } => Err(at(
                "EJ242",
                line,
                1,
                "`super(...)` and `this(...)` may only be the first statement of a constructor.",
            )),
            Statement::Block(inside) => {
                self.open();
                for one in inside {
                    self.statement(one)?;
                }
                self.close();
                Ok(())
            }
            Statement::Declare { what, name, value } => {
                if matches!(what, Written::Inferred) {
                    let Some(expression) = value else {
                        return Err(self.resolve(what, line).unwrap_err());
                    };
                    let found = self.value(expression, line)?;
                    if found == Type::Void {
                        return Err(at(
                            "EJ234",
                            line,
                            1,
                            format!(
                                "`{name}` was given the result of something that returns nothing."
                            ),
                        ));
                    }
                    if matches!(expression, Expression::Null) {
                        return Err(at(
                            "EJ234",
                            line,
                            1,
                            format!("`var {name} = null` says nothing about what it holds."),
                        )
                        .with_suggestion("Write the type."));
                    }
                    let slot = self.declare(name, found.clone());
                    self.store(slot, &found);
                    return Ok(());
                }
                let target = self.resolve(what, line)?;
                match value {
                    Some(expression) => {
                        let found = self.value_for(expression, &target, line)?;
                        if !self.fit(&found, &target, line)? {
                            return Err(at(
                                "EJ230",
                                line,
                                1,
                                format!(
                                    "`{name}` is a {} and was given a {}.",
                                    target.readable(),
                                    found.readable()
                                ),
                            ));
                        }
                    }
                    None => {
                        self.declare_with_nothing_in_it(name, target);
                        return Ok(());
                    }
                }
                let slot = self.declare(name, target.clone());
                self.store(slot, &target);
                Ok(())
            }
            Statement::Express(expression) => {
                let what = match expression {
                    Expression::Assign {
                        target,
                        operator,
                        value,
                    } => {
                        self.assign(target, *operator, value, line, false)?;
                        Type::Void
                    }
                    Expression::Step { target, by, after } => {
                        self.step(target, *by, *after, line, false)?;
                        Type::Void
                    }
                    other => self.value(other, line)?,
                };
                match what.width() {
                    0 => {}
                    1 => {
                        self.op(0x57);
                        self.pops(1);
                    }
                    _ => {
                        self.op(0x58);
                        self.pops(2);
                    }
                }
                Ok(())
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                let found = self.truth(condition, line)?;
                if found != Type::Boolean {
                    return Err(at(
                        "EJ206",
                        line,
                        1,
                        format!(
                            "An `if` wants a boolean and was given a {}.",
                            found.readable()
                        ),
                    ));
                }
                self.pops(1);
                let to_else = self.jump(0x99);
                self.statement(then)?;
                match otherwise {
                    Some(otherwise) => {
                        let over = (!never_completes(&then.node)).then(|| self.jump(0xa7));
                        self.land(to_else);
                        self.a_branch_lands_here();
                        self.statement(otherwise)?;
                        if let Some(over) = over {
                            self.land(over);
                            self.a_branch_lands_here();
                        }
                    }
                    None => {
                        self.land(to_else);
                        self.a_branch_lands_here();
                    }
                }
                Ok(())
            }
            Statement::While { condition, body } => {
                let top = self.code.len();

                self.a_branch_lands_here();

                let out = if matches!(condition, Expression::Boolean(true)) {
                    None
                } else {
                    let found = self.truth(condition, line)?;
                    if found != Type::Boolean {
                        return Err(at("EJ206", line, 1, "A `while` wants a boolean."));
                    }
                    self.pops(1);
                    Some(self.jump(0x99))
                };
                self.enter(true);
                self.statement(body)?;
                let level = self.leave();
                for pending in level.continues {
                    self.land(pending);
                }
                self.a_branch_lands_here();
                self.jump_back(0xa7, top);
                let leaves = !level.breaks.is_empty() || out.is_some();
                if let Some(out) = out {
                    self.land(out);
                }
                for pending in level.breaks {
                    self.land(pending);
                }

                if leaves {
                    self.a_branch_lands_here();
                } else {
                    self.reachable = false;
                }
                Ok(())
            }
            Statement::For {
                start,
                condition,
                step,
                body,
            } => {
                self.open();
                for one in start {
                    self.statement(one)?;
                }
                let top = self.code.len();
                self.a_branch_lands_here();
                let out = match condition {
                    Some(condition) => {
                        let found = self.truth(condition, line)?;
                        if found != Type::Boolean {
                            return Err(at("EJ206", line, 1, "A `for` condition is a boolean."));
                        }
                        self.pops(1);
                        let jump = self.jump(0x99);
                        Some(jump)
                    }
                    None => None,
                };
                self.enter(true);
                self.statement(body)?;
                let level = self.leave();
                for pending in level.continues {
                    self.land(pending);
                }
                self.a_branch_lands_here();
                for expression in step {
                    match expression {
                        Expression::Step { target, by, after } => {
                            self.step(target, *by, *after, line, false)?;
                        }
                        Expression::Assign {
                            target,
                            operator,
                            value,
                        } => {
                            self.assign(target, *operator, value, line, false)?;
                        }
                        other => {
                            let what = self.value(other, line)?;
                            match what.width() {
                                0 => {}
                                1 => {
                                    self.op(0x57);
                                    self.pops(1);
                                }
                                _ => {
                                    self.op(0x58);
                                    self.pops(2);
                                }
                            }
                        }
                    }
                }
                self.jump_back(0xa7, top);
                if let Some(out) = out {
                    self.land(out);
                }
                for pending in level.breaks {
                    self.land(pending);
                }
                self.a_branch_lands_here();
                self.close();
                Ok(())
            }
            Statement::DoWhile { body, condition } => {
                let top = self.code.len();
                self.a_branch_lands_here();
                self.enter(true);
                self.statement(body)?;

                let level = self.leave();
                for pending in level.continues {
                    self.land(pending);
                }
                self.a_branch_lands_here();
                let found = self.truth(condition, line)?;
                if found != Type::Boolean {
                    return Err(at("EJ206", line, 1, "A `do`/`while` wants a boolean."));
                }

                self.jump_back(0x9a, top);
                self.pops(1);
                for pending in level.breaks {
                    self.land(pending);
                }
                self.a_branch_lands_here();
                Ok(())
            }
            Statement::ForEach {
                what,
                name,
                over,
                body,
            } => self.for_each(what, name, over, body, line),
            Statement::Switch { subject, arms } => self.switch(subject, arms, line),
            Statement::Try {
                body,
                catches,
                finally,
            } => self.try_catch(body, catches, finally.as_deref(), line),
            Statement::Yield(what) => {
                let Some(level) = self.yields.len().checked_sub(1) else {
                    return Err(at(
                        "EJ244",
                        line,
                        1,
                        "`yield` is not inside a `switch` used for its value.",
                    ));
                };
                let found = self.value(what, line)?;
                self.agree_on_yield(level, &found, line)?;
                let down_to = self.yields[level].finallys;
                self.run_finallys(down_to)?;
                let jump = self.jump(0xa7);
                self.yields[level].pending.push(jump);
                Ok(())
            }
            Statement::Synchronized { on, body } => {
                let found = self.value(on, line)?;
                if !found.is_reference() {
                    return Err(at(
                        "EJ253",
                        line,
                        1,
                        format!(
                            "`synchronized` locks an object, and was given a {}.",
                            found.readable()
                        ),
                    ));
                }
                self.open();
                let held = self.declare("$locked", found.clone());
                self.duplicates();
                self.store(held, &found);
                self.op(0xc2);
                self.pops(1);

                let outer = self.finallys.len();
                self.finallys.push(Cleanup::Unlock(held, found.clone()));

                let outside = self.slots.clone();
                let begun = self.code.len();
                for one in body {
                    self.statement(one)?;
                }
                let leaves = body.iter().any(|one| never_completes(&one.node));
                self.finallys.truncate(outer);
                if !leaves {
                    self.load(held, &found);
                    self.op(0xc3);
                    self.pops(1);
                }
                let ended = self.code.len();

                let over = (!leaves).then(|| self.jump(0xa7));

                let target = self.code.len();
                self.protect(begun, ended, target, None, self.finallys.len());
                let throwable = Type::Object("java/lang/Throwable".to_string(), Vec::new());
                self.slots = outside;
                self.stack_is(vec![Verified::of(&throwable)]);
                self.a_branch_lands_here();
                let thrown = self.declare("$thrown", throwable.clone());
                self.store(thrown, &throwable);
                self.load(held, &found);
                self.op(0xc3);
                self.pops(1);
                self.load(thrown, &throwable);
                self.op(0xbf);
                self.set_depth(0);

                self.close();
                if let Some(over) = over {
                    self.land(over);
                    self.set_depth(0);
                    self.a_branch_lands_here();
                }
                Ok(())
            }
            Statement::Assert { condition, said } => {
                let index = self
                    .pool
                    .field(&self.this_class.clone(), ASSERTIONS_OFF, "Z");
                self.op2(0xb2, index);
                self.pushes(&Type::Boolean);
                self.pops(1);
                let over = self.jump(0x9a);

                let found = self.truth(condition, line)?;
                if found != Type::Boolean {
                    return Err(at(
                        "EJ206",
                        line,
                        1,
                        format!(
                            "An `assert` wants a boolean and was given a {}.",
                            found.readable()
                        ),
                    ));
                }
                self.pops(1);
                let held = self.jump(0x9a);

                let error = self.pool.class("java/lang/AssertionError");
                self.op2(0xbb, error);
                self.pushes(&Type::Object(
                    "java/lang/AssertionError".to_string(),
                    Vec::new(),
                ));
                self.duplicates();
                let descriptor = match said {
                    Some(expression) => {
                        let what = self.value(expression, line)?;

                        match &what {
                            Type::Boolean => "(Z)V",
                            Type::Char => "(C)V",
                            Type::Int | Type::Byte | Type::Short => {
                                self.convert(&what, &Type::Int, line)?;
                                "(I)V"
                            }
                            Type::Long => "(J)V",
                            Type::Float => "(F)V",
                            Type::Double => "(D)V",
                            _ => "(Ljava/lang/Object;)V",
                        }
                    }
                    None => "()V",
                };
                let init =
                    self.pool
                        .method("java/lang/AssertionError", "<init>", descriptor, false);
                self.op2(0xb7, init);
                let taken: i32 = read_descriptor(descriptor)
                    .map(|(parameters, _)| {
                        parameters.iter().map(|one| i32::from(one.width())).sum()
                    })
                    .unwrap_or(0);
                self.pops(taken + 1);
                self.op(0xbf);
                self.set_depth(0);

                self.land(over);
                self.land(held);
                self.set_depth(0);
                self.a_branch_lands_here();
                self.wants_assertions = true;
                Ok(())
            }
            Statement::Throw(what) => {
                let precise = match what {
                    Expression::Name(name) => self.local(name).and_then(|held| {
                        self.rethrown
                            .iter()
                            .rev()
                            .find(|(slot, classes)| *slot == held.slot && classes.len() > 1)
                            .map(|(_, classes)| classes.clone())
                    }),
                    _ => None,
                };
                match precise {
                    Some(classes) => {
                        for named in &classes {
                            self.it_can_throw(named, line)?;
                        }
                    }
                    None => {
                        let found = self.type_of(what, line)?;
                        if let Type::Object(named, _) = &found {
                            self.it_can_throw(named, line)?;
                        }
                    }
                }
                let found = self.value(what, line)?;
                if !found.is_reference() {
                    return Err(at(
                        "EJ235",
                        line,
                        1,
                        format!(
                            "`throw` wants a Throwable and was given a {}.",
                            found.readable()
                        ),
                    ));
                }
                self.op(0xbf);

                self.set_depth(0);
                self.reachable = false;
                Ok(())
            }
            Statement::Labelled { label, body } => {
                if matches!(
                    &body.node,
                    Statement::While { .. }
                        | Statement::DoWhile { .. }
                        | Statement::For { .. }
                        | Statement::ForEach { .. }
                        | Statement::Switch { .. }
                ) {
                    self.pending_label = Some(label.clone());
                    return self.statement(body);
                }
                self.pending_label = Some(label.clone());
                self.enter(false);
                self.statement(body)?;
                let level = self.leave();
                if !level.breaks.is_empty() {
                    for pending in level.breaks {
                        self.land(pending);
                    }
                    self.a_branch_lands_here();
                }
                Ok(())
            }
            Statement::Break(label) => {
                let Some(index) = self.level_for(label.as_deref(), false) else {
                    return Err(self.no_such_level(label.as_deref(), line, "break"));
                };
                self.run_finallys(self.levels[index].finallys)?;
                let jump = self.jump(0xa7);
                self.levels[index].breaks.push(jump);
                Ok(())
            }
            Statement::Continue(label) => {
                let Some(index) = self.level_for(label.as_deref(), true) else {
                    return Err(self.no_such_level(label.as_deref(), line, "continue"));
                };
                self.run_finallys(self.levels[index].finallys)?;
                let jump = self.jump(0xa7);
                self.levels[index].continues.push(jump);
                Ok(())
            }
            Statement::Return(value) => {
                let wanted = self.returns.clone();
                match value {
                    None => {
                        self.run_finallys(0)?;
                        if wanted != Type::Void {
                            return Err(at(
                                "EJ233",
                                line,
                                1,
                                format!(
                                    "This returns a {} and the `return` carries nothing.",
                                    wanted.readable()
                                ),
                            ));
                        }
                        if self.reachable {
                            self.op(0xb1);
                        }
                        self.reachable = false;
                    }
                    Some(expression) => {
                        if wanted == Type::Void {
                            return Err(at(
                                "EJ233",
                                line,
                                1,
                                "This returns nothing, and the `return` carries a value.",
                            ));
                        }
                        let found = self.value_for(expression, &wanted, line)?;
                        if !self.fit(&found, &wanted, line)? {
                            return Err(at(
                                "EJ201",
                                line,
                                1,
                                format!(
                                    "This returns a {} and the `return` carries a {}.",
                                    wanted.readable(),
                                    found.readable()
                                ),
                            ));
                        }

                        if !self.finallys.is_empty() {
                            self.open();
                            let held = self.declare("$returning", wanted.clone());
                            self.store(held, &wanted);
                            self.run_finallys(0)?;

                            if !self.reachable {
                                self.close();
                                return Ok(());
                            }
                            self.load(held, &wanted);
                            self.close();
                        }
                        let opcode = match &wanted {
                            Type::Long => 0xadu8,
                            Type::Float => 0xae,
                            Type::Double => 0xaf,
                            other if other.is_reference() => 0xb0,
                            _ => 0xac,
                        };
                        self.op(opcode);
                        self.pops(i32::from(wanted.width()));
                        self.reachable = false;
                    }
                }
                Ok(())
            }
        }
    }
}

fn stack_map_table(frames: &[Frame], pool: &mut Pool) -> Option<Vec<u8>> {
    if frames.is_empty() {
        return None;
    }
    let mut sorted: Vec<&Frame> = frames.iter().collect();
    sorted.sort_by_key(|frame| frame.at);

    let mut body = Vec::new();
    body.extend_from_slice(&(sorted.len() as u16).to_be_bytes());

    let mut previous: Option<usize> = None;
    for frame in sorted {
        let delta = match previous {
            None => frame.at,
            Some(before) => frame.at - before - 1,
        };
        previous = Some(frame.at);

        body.push(255);
        body.extend_from_slice(&(delta as u16).to_be_bytes());
        body.extend_from_slice(&(frame.locals.len() as u16).to_be_bytes());
        for local in &frame.locals {
            local.write(&mut body, pool);
        }
        body.extend_from_slice(&(frame.stack.len() as u16).to_be_bytes());
        for held in &frame.stack {
            held.write(&mut body, pool);
        }
    }
    Some(body)
}

fn never_completes(statement: &Statement) -> bool {
    match statement {
        Statement::Return(_) | Statement::Throw(_) => true,

        Statement::Break(_) | Statement::Continue(_) => true,
        Statement::Block(inside) => inside.iter().any(|one| never_completes(&one.node)),
        Statement::If {
            then,
            otherwise: Some(otherwise),
            ..
        } => never_completes(&then.node) && never_completes(&otherwise.node),

        Statement::For {
            condition: None,
            body,
            ..
        } => !holds_a_break(&body.node),
        Statement::For {
            condition: Some(condition),
            body,
            ..
        }
        | Statement::While { condition, body } => {
            matches!(condition, Expression::Boolean(true)) && !holds_a_break(&body.node)
        }
        Statement::DoWhile { body, condition } => {
            matches!(condition, Expression::Boolean(true)) && !holds_a_break(&body.node)
        }

        Statement::Synchronized { body, .. } => body.iter().any(|one| never_completes(&one.node)),

        Statement::Assert { .. } => false,

        Statement::Try {
            body,
            catches,
            finally,
        } => {
            if finally
                .as_ref()
                .is_some_and(|inside| inside.iter().any(|one| never_completes(&one.node)))
            {
                return true;
            }
            body.iter().any(|one| never_completes(&one.node))
                && catches
                    .iter()
                    .all(|catch| catch.body.iter().any(|one| never_completes(&one.node)))
        }

        Statement::Switch { arms, .. } => {
            if arms.is_empty()
                || !arms
                    .iter()
                    .any(|arm| arm.labels.is_empty() && arm.pattern.is_none())
            {
                return false;
            }
            if arms
                .iter()
                .any(|arm| arm.body.iter().any(|one| holds_a_break(&one.node)))
            {
                return false;
            }
            if arms.iter().any(|arm| arm.arrow) {
                return arms
                    .iter()
                    .all(|arm| arm.body.iter().any(|one| never_completes(&one.node)));
            }
            arms.last()
                .is_some_and(|arm| arm.body.iter().any(|one| never_completes(&one.node)))
        }
        _ => false,
    }
}

fn holds_a_yield(statement: &Statement) -> bool {
    match statement {
        Statement::Yield(_) => true,
        Statement::Block(inside) | Statement::Several(inside) => {
            inside.iter().any(|one| holds_a_yield(&one.node))
        }
        Statement::Labelled { body, .. } => holds_a_yield(&body.node),
        Statement::If {
            then, otherwise, ..
        } => {
            holds_a_yield(&then.node)
                || otherwise
                    .as_ref()
                    .is_some_and(|otherwise| holds_a_yield(&otherwise.node))
        }
        Statement::While { body, .. }
        | Statement::DoWhile { body, .. }
        | Statement::For { body, .. }
        | Statement::ForEach { body, .. } => holds_a_yield(&body.node),
        Statement::Try {
            body,
            catches,
            finally,
        } => {
            body.iter().any(|one| holds_a_yield(&one.node))
                || catches
                    .iter()
                    .any(|catch| catch.body.iter().any(|one| holds_a_yield(&one.node)))
                || finally
                    .as_ref()
                    .is_some_and(|inside| inside.iter().any(|one| holds_a_yield(&one.node)))
        }
        _ => false,
    }
}

fn holds_a_break(statement: &Statement) -> bool {
    match statement {
        Statement::Break(_) => true,
        Statement::Block(inside) => inside.iter().any(|one| holds_a_break(&one.node)),
        Statement::Labelled { body, .. } => holds_a_break(&body.node),
        Statement::If {
            then, otherwise, ..
        } => {
            holds_a_break(&then.node)
                || otherwise
                    .as_ref()
                    .is_some_and(|otherwise| holds_a_break(&otherwise.node))
        }
        Statement::Try {
            body,
            catches,
            finally,
        } => {
            body.iter().any(|one| holds_a_break(&one.node))
                || catches
                    .iter()
                    .any(|catch| catch.body.iter().any(|one| holds_a_break(&one.node)))
                || finally
                    .as_ref()
                    .is_some_and(|inside| inside.iter().any(|one| holds_a_break(&one.node)))
        }
        _ => false,
    }
}

impl Emitter<'_> {
    fn for_each(
        &mut self,
        what: &Written,
        name: &str,
        over: &Expression,
        body: &Positioned<Statement>,
        line: u32,
    ) -> Result<(), Diagnostic> {
        let found = self.type_of(over, line)?;
        let Type::Array(element) = found.clone() else {
            return self.for_each_over_an_iterator(what, name, over, body, &found, line);
        };

        self.open();
        let found = self.value(over, line)?;

        let declared = match what {
            Written::Inferred => (*element).clone(),
            other => self.resolve(other, line)?,
        };
        let refused = at(
            "EJ237",
            line,
            1,
            format!(
                "`{name}` is a {} and the array holds {}.",
                declared.readable(),
                element.readable()
            ),
        );
        let boxed_across = element.is_reference() != declared.is_reference();
        if !boxed_across && !element.may_be_given_to(&declared) {
            return Err(refused);
        }

        let array = self.declare("$over", found.clone());
        self.store(array, &found);
        self.op(0x03);
        self.pushes(&Type::Int);
        let index = self.declare("$at", Type::Int);
        self.store(index, &Type::Int);

        let top = self.code.len();
        self.a_branch_lands_here();
        self.load(index, &Type::Int);
        self.load(array, &found);
        self.op(0xbe);
        self.pops(2);
        let out = self.jump(0xa2);

        self.open();
        self.load(array, &found);
        self.load(index, &Type::Int);
        self.op(array_load(&element));
        self.pops(2);
        self.pushes(&element);
        if boxed_across {
            let Some(opened) = self.box_or_unbox(&element, &declared) else {
                return Err(refused);
            };
            if !declared.is_reference() && opened != declared {
                self.convert(&opened, &declared, line)?;
            }
        } else if declared != *element {
            self.convert(&element, &declared, line)?;
        }
        let held = self.declare(name, declared.clone());
        self.store(held, &declared);

        self.enter(true);
        self.statement(body)?;
        let level = self.leave();
        self.close();

        for pending in level.continues {
            self.land(pending);
        }
        self.a_branch_lands_here();
        self.bump_local(index, 1);
        self.jump_back(0xa7, top);
        self.land(out);
        for pending in level.breaks {
            self.land(pending);
        }
        self.close();
        self.a_branch_lands_here();
        Ok(())
    }

    fn switch(&mut self, subject: &Expression, arms: &[Arm], line: u32) -> Result<(), Diagnostic> {
        let found = self.value(subject, line)?;
        let found = self.opened_for_a_switch(found, arms);
        let over = self.what_a_switch_is_over(&found, arms, line)?;

        let answers_for_everything = self.covers_everything(&over, &found, arms);

        self.open();
        self.enter(false);
        let mut ends: Vec<Pending> = Vec::new();
        let mut targets: Vec<usize> = Vec::new();

        let over_held = over.clone();
        let dispatch = self.dispatch_for(over, arms, line)?;

        let at_the_dispatch = self.slots.clone();

        let mut default_at: Option<usize> = None;
        for arm in arms {
            let at_here = self.code.len();
            targets.push(at_here);
            if arm.labels.is_empty() && arm.pattern.is_none() {
                default_at = Some(at_here);
            }
            if self.reachable {
                self.only_what_every_road_agrees_on(&at_the_dispatch);
            } else {
                self.slots = at_the_dispatch.clone();
            }
            self.set_depth(0);
            self.a_branch_lands_here();
            let scoped = arm.arrow || arm.pattern.is_some();
            if scoped {
                self.open();
            }
            self.bind_a_pattern_here(arm)?;
            for one in &arm.body {
                self.statement(one)?;
            }
            if scoped {
                self.close();
            }

            if arm.arrow && !arm.body.iter().any(|one| never_completes(&one.node)) {
                ends.push(self.jump(0xa7));
            }
        }

        if default_at.is_none() && answers_for_everything {
            let here = self.code.len();
            self.set_depth(0);
            self.a_branch_lands_here();
            self.nothing_else_is_possible(&over_held, line)?;
            default_at = Some(here);
        }

        let after = self.code.len();
        let arrives = default_at.is_none() || !ends.is_empty();

        if default_at.is_none() {
            if self.reachable {
                self.only_what_every_road_agrees_on(&at_the_dispatch);
            } else {
                self.slots = at_the_dispatch.clone();
                self.reachable = true;
            }
        }
        dispatch.settle(self, &targets, default_at.unwrap_or(after));
        for pending in ends {
            self.land(pending);
        }
        let level = self.leave();
        let arrives = arrives || !level.breaks.is_empty();
        for pending in level.breaks {
            self.land(pending);
        }
        self.close();

        if arrives {
            self.set_depth(0);
            self.a_branch_lands_here();
        } else {
            self.reachable = false;
        }
        Ok(())
    }

    fn switch_value(
        &mut self,
        subject: &Expression,
        arms: &[Arm],
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let written_default = arms
            .iter()
            .any(|arm| arm.labels.is_empty() && arm.pattern.is_none());

        let found = self.value(subject, line)?;
        let found = self.opened_for_a_switch(found, arms);
        let over = self.what_a_switch_is_over(&found, arms, line)?;
        if !written_default && !self.covers_everything(&over, &found, arms) {
            return Err(at(
                "EJ245",
                line,
                1,
                "A `switch` used for its value needs a `default`, because it has to have \
                 an answer for anything.",
            )
            .with_suggestion(
                "An enum whose constants are all named, or a `sealed` type whose subtypes \
                 all are, answers for everything already and needs no `default`.",
            ));
        }

        self.open();
        self.yields.push(Yielding {
            pending: Vec::new(),
            produced: None,
            finallys: self.finallys.len(),
        });
        let level = self.yields.len() - 1;

        let over_held = over.clone();
        let dispatch = self.dispatch_for(over, arms, line)?;

        let beneath = self.stack.clone();

        let mut targets: Vec<usize> = Vec::new();
        let mut default_at: Option<usize> = None;
        for arm in arms {
            let at_here = self.code.len();
            targets.push(at_here);
            if arm.labels.is_empty() && arm.pattern.is_none() {
                default_at = Some(at_here);
            }
            self.stack_is(beneath.clone());
            self.a_branch_lands_here();
            self.open();
            self.bind_a_pattern_here(arm)?;

            let bare = match (arm.arrow, arm.body.len(), arm.body.first()) {
                (true, 1, Some(one)) => match &one.node {
                    Statement::Express(expression) => Some(expression.clone()),
                    _ => None,
                },
                _ => None,
            };
            match bare {
                Some(expression) => {
                    let produced = self.value(&expression, arm.line)?;
                    self.agree_on_yield(level, &produced, arm.line)?;
                    let down_to = self.yields[level].finallys;
                    self.run_finallys(down_to)?;
                    let jump = self.jump(0xa7);
                    self.yields[level].pending.push(jump);
                }
                None => {
                    for one in &arm.body {
                        self.statement(one)?;
                    }
                    if !arm.body.iter().any(|one| never_completes(&one.node))
                        && !arm.body.iter().any(|one| holds_a_yield(&one.node))
                    {
                        return Err(at(
                            "EJ246",
                            arm.line,
                            arm.column,
                            "Every arm of a `switch` used for its value has to produce one.",
                        )
                        .with_suggestion("Write `yield`, or `throw`."));
                    }
                }
            }
            self.close();
        }

        if default_at.is_none() {
            let here = self.code.len();
            self.stack_is(beneath.clone());
            self.a_branch_lands_here();
            self.nothing_else_is_possible(&over_held, line)?;
            default_at = Some(here);
        }
        let after = self.code.len();
        dispatch.settle(self, &targets, default_at.unwrap_or(after));

        let level = self.yields.pop().expect("the level was pushed");
        for pending in level.pending {
            self.land(pending);
        }
        self.close();

        let Some(produced) = level.produced else {
            return Err(at(
                "EJ246",
                line,
                1,
                "No arm of this `switch` produces a value.",
            ));
        };
        let mut settled = beneath;
        let held = Verified::of(&produced);
        let wide = held.is_wide();
        settled.push(held);
        if wide {
            settled.push(Verified::Top);
        }
        self.stack_is(settled);
        self.a_branch_lands_here();
        Ok(produced)
    }

    fn agree_on_yield(&mut self, level: usize, found: &Type, line: u32) -> Result<(), Diagnostic> {
        match self.yields[level].produced.clone() {
            None => {
                self.yields[level].produced = Some(found.clone());
                Ok(())
            }
            Some(wanted) if *found == wanted => Ok(()),
            Some(wanted) if found.may_be_given_to(&wanted) => {
                if !found.is_reference() {
                    self.convert(found, &wanted, line)?;
                }
                Ok(())
            }
            Some(wanted) => Err(at(
                "EJ247",
                line,
                1,
                format!(
                    "One arm of this `switch` produces a {} and another a {}.",
                    wanted.readable(),
                    found.readable()
                ),
            )
            .with_suggestion("Every arm has to produce the same type. A cast settles it.")),
        }
    }

    fn covers_everything(&mut self, over: &Chooser, found: &Type, arms: &[Arm]) -> bool {
        if let Chooser::Primitive(kind) = over {
            let total = arms.iter().any(|arm| {
                if arm.guard.is_some() {
                    return false;
                }
                let Some(pattern) = &arm.pattern else {
                    return false;
                };
                let Ok(target) = self.resolve(&pattern.what, arm.line) else {
                    return false;
                };
                if target.is_reference() {
                    return true;
                }
                matches!(exactness(kind, &target, PATTERN_SUBJECT), Some(None))
            });
            if total {
                return true;
            }
            if *kind == Type::Boolean {
                let said: Vec<bool> = arms
                    .iter()
                    .flat_map(|arm| arm.labels.iter())
                    .filter_map(|label| match label {
                        Expression::Boolean(held) => Some(*held),
                        _ => None,
                    })
                    .collect();
                return said.contains(&true) && said.contains(&false);
            }
            return false;
        }
        match over {
            Chooser::Constant(class) => {
                let Some(known) = self.classpath.get(class) else {
                    return false;
                };
                let wanted: Vec<String> = known
                    .fields
                    .iter()
                    .filter(|(_, what, static_)| {
                        *static_ && *what == Type::Object(class.clone(), Vec::new())
                    })
                    .map(|(name, _, _)| name.clone())
                    .collect();
                if wanted.is_empty() {
                    return false;
                }
                wanted.iter().all(|name| {
                    arms.iter()
                        .flat_map(|arm| arm.labels.iter())
                        .any(|label| enum_label(label) == Some(name.as_str()))
                })
            }
            Chooser::Shape => {
                let Type::Object(class, _) = found else {
                    return false;
                };
                let Some(known) = self.classpath.get(class) else {
                    return false;
                };
                let permitted = known.permits.clone();
                if permitted.is_empty() {
                    return false;
                }

                let named: Vec<String> = arms
                    .iter()
                    .filter(|arm| arm.guard.is_none())
                    .filter_map(|arm| arm.pattern.as_ref())
                    .filter_map(|one| match self.resolve(&one.what, 1) {
                        Ok(Type::Object(held, _)) => Some(held),
                        _ => None,
                    })
                    .collect();
                permitted.iter().all(|one| named.contains(one))
            }
            _ => false,
        }
    }

    fn nothing_else_is_possible(&mut self, over: &Chooser, line: u32) -> Result<(), Diagnostic> {
        let named = match over {
            Chooser::Constant(_) => "java/lang/IncompatibleClassChangeError",
            _ => "java/lang/MatchException",
        };
        let class = self.pool.class(named);
        self.op2(0xbb, class);
        self.pushes(&Type::Object(named.to_string(), Vec::new()));
        self.duplicates();

        let taken = if named == "java/lang/MatchException" {
            self.op(0x01);
            self.op(0x01);
            self.pushes_raw(Verified::Null);
            self.pushes_raw(Verified::Null);
            "(Ljava/lang/String;Ljava/lang/Throwable;)V"
        } else {
            "()V"
        };
        let init = self.pool.method(named, "<init>", taken, false);
        self.op2(0xb7, init);
        self.pops(1 + i32::from(taken != "()V") * 2);
        self.op(0xbf);
        self.set_depth(0);
        let _ = line;
        Ok(())
    }

    fn opened_for_a_switch(&mut self, found: Type, arms: &[Arm]) -> Type {
        let Type::Object(named, _) = &found else {
            return found;
        };
        let wrapped = matches!(
            named.as_str(),
            "java/lang/Integer" | "java/lang/Short" | "java/lang/Byte" | "java/lang/Character"
        );
        let only_constants = arms.iter().all(|arm| {
            arm.pattern.is_none()
                && !arm
                    .labels
                    .iter()
                    .any(|label| matches!(label, Expression::Null))
        });
        if !wrapped || !only_constants {
            return found;
        }
        self.box_or_unbox(&found, &Type::Int).unwrap_or(found)
    }

    fn what_a_switch_is_over(
        &mut self,
        found: &Type,
        arms: &[Arm],
        line: u32,
    ) -> Result<Chooser, Diagnostic> {
        let patterned = arms.iter().any(|arm| arm.pattern.is_some());
        if !found.is_reference() && *found != Type::Void {
            let wide = matches!(
                found,
                Type::Long | Type::Float | Type::Double | Type::Boolean
            );
            if patterned || wide {
                return Ok(Chooser::Primitive(found.clone()));
            }
        }
        if patterned {
            return Ok(Chooser::Shape);
        }
        if found.is_int_like() {
            return Ok(Chooser::Integer);
        }
        if *found == Type::Object("java/lang/String".to_string(), Vec::new()) {
            return Ok(Chooser::Text);
        }

        if let Type::Object(class, _) = found {
            let every = arms.iter().flat_map(|arm| arm.labels.iter()).all(|label| {
                let Some(named) = enum_label(label) else {
                    return false;
                };
                self.classpath
                    .find_field(class, named)
                    .is_some_and(|(_, (_, what, static_))| {
                        *static_ && *what == Type::Object(class.clone(), Vec::new())
                    })
            });
            if every {
                return Ok(Chooser::Constant(class.clone()));
            }
        }
        Err(at(
            "EJ238",
            line,
            1,
            format!(
                "A `switch` takes an integer, a String or an enum, and was given a {}.",
                found.readable()
            ),
        ))
    }

    fn dispatch_for(
        &mut self,
        over: Chooser,
        arms: &[Arm],
        line: u32,
    ) -> Result<Dispatch, Diagnostic> {
        match over {
            Chooser::Integer => self.integer_dispatch(arms, line),
            Chooser::Text => self.text_dispatch(arms, line),
            Chooser::Constant(class) => self.constant_dispatch(&class, arms, line),
            Chooser::Shape => self.shape_dispatch(arms, line),
            Chooser::Primitive(kind) => self.primitive_dispatch(&kind, arms, line),
        }
    }

    fn primitive_dispatch(
        &mut self,
        kind: &Type,
        arms: &[Arm],
        line: u32,
    ) -> Result<Dispatch, Diagnostic> {
        let held = self.declare(PATTERN_SUBJECT, kind.clone());
        self.store(held, kind);

        let mut waiting: Vec<(Pending, usize)> = Vec::new();
        for (index, arm) in arms.iter().enumerate() {
            for label in &arm.labels {
                if matches!(label, Expression::Null) {
                    return Err(at(
                        "EJ239",
                        arm.line,
                        arm.column,
                        format!("A `case null` cannot match a {}.", kind.readable()),
                    ));
                }
                self.load(held, kind);
                let constant = self.value(label, arm.line)?;
                if constant.is_reference()
                    || (*kind == Type::Boolean) != (constant == Type::Boolean)
                {
                    return Err(at(
                        "EJ239",
                        arm.line,
                        arm.column,
                        format!(
                            "A `case` over a {} answers to a constant of that kind, and a {} \
                             was found.",
                            kind.readable(),
                            constant.readable()
                        ),
                    ));
                }
                self.convert(&constant, kind, arm.line)?;
                let (compare, branch) = match kind {
                    Type::Long => (Some((0x94u8, None)), 0x99u8),
                    Type::Float => (Some((0xb8, Some(("java/lang/Float", "(FF)I")))), 0x99),
                    Type::Double => (Some((0xb8, Some(("java/lang/Double", "(DD)I")))), 0x99),
                    _ => (None, 0x9f),
                };
                if let Some((opcode, through)) = compare {
                    match through {
                        Some((owner, descriptor)) => {
                            let index = self.pool.method(owner, "compare", descriptor, false);
                            self.op2(opcode, index);
                        }
                        None => self.op(opcode),
                    }
                    self.pops(2 * i32::from(kind.width()));
                    self.pushes(&Type::Int);
                    self.pops(1);
                } else {
                    self.pops(2);
                }
                waiting.push((self.jump(branch), index));
            }
            let Some(pattern) = arm.pattern.clone() else {
                continue;
            };
            if pattern.taken {
                return Err(at(
                    "EJ208",
                    arm.line,
                    arm.column,
                    format!("A {} has no components to take apart.", kind.readable()),
                ));
            }
            let target = self.resolve(&pattern.what, arm.line)?;
            let test = self.how_a_primitive_is_tested(kind, &target, arm.line)?;

            self.open();
            let past = match test {
                Some(test) => {
                    let found = self.truth(&test, arm.line)?;
                    if found != Type::Boolean {
                        return Err(at(
                            "EJ206",
                            arm.line,
                            arm.column,
                            "A `when` on a `case` wants a boolean.",
                        ));
                    }
                    self.pops(1);
                    Some(self.jump(0x99))
                }
                None => None,
            };
            match &arm.guard {
                None => {
                    self.push_int(1);
                    self.pops(1);
                    waiting.push((self.jump(0x9a), index));
                    self.close();
                }
                Some(guard) => {
                    self.bind_the_pattern(arm, held)?;
                    let found = self.truth(guard, arm.line)?;
                    if found != Type::Boolean {
                        return Err(at(
                            "EJ206",
                            arm.line,
                            arm.column,
                            "A `when` on a `case` wants a boolean.",
                        ));
                    }
                    self.pops(1);
                    waiting.push((self.jump(0x9a), index));
                    self.close();
                }
            }
            if let Some(past) = past {
                self.land(past);
                self.a_branch_lands_here();
            } else if arm.guard.is_some() {
                self.a_branch_lands_here();
            }
        }
        let _ = line;
        let fallthrough = self.jump(0xa7);
        Ok(Dispatch::Chain {
            waiting,
            fallthrough,
        })
    }

    fn zero_of(&mut self, what: &Type) {
        match what {
            Type::Long => {
                self.op(0x09);
            }
            Type::Float => {
                self.op(0x0b);
            }
            Type::Double => {
                self.op(0x0e);
            }
            other if other.is_reference() => {
                self.op(0x01);
                self.pushes_raw(Verified::Null);
                return;
            }
            _ => {
                self.op(0x03);
            }
        }
        self.pushes(what);
    }

    fn a_primitive_tested(
        &mut self,
        found: &Type,
        target: &Type,
        binds: Option<&String>,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let named = format!("$exact{}", self.taken_apart);
        self.taken_apart += 1;
        let held = self.declare(&named, found.clone());
        self.store(held, found);
        let test = self.how_a_named_primitive_is_tested(found, target, &named, line)?;
        let slot = match binds {
            Some(binding) => {
                self.zero_of(target);
                let slot = self.declare(binding, target.clone());
                self.store(slot, target);
                Some(slot)
            }
            None => None,
        };
        let bind = |emitter: &mut Emitter<'_>, slot: u16| -> Result<(), Diagnostic> {
            emitter.load(held, found);
            if target.is_reference() {
                emitter.box_or_unbox(found, target);
            } else {
                emitter.converted_for_a_pattern(found, target, line)?;
            }
            emitter.store(slot, target);
            Ok(())
        };
        match test {
            None => {
                if let Some(slot) = slot {
                    bind(self, slot)?;
                }
                self.push_int(1);
            }
            Some(test) => {
                let said = self.truth(&test, line)?;
                if said != Type::Boolean {
                    return Err(at("EJ206", line, 1, "A test here wants a boolean."));
                }
                if let Some(slot) = slot {
                    self.duplicates();
                    self.pops(1);
                    let over = self.jump(0x99);
                    bind(self, slot)?;
                    self.land(over);
                    self.a_branch_lands_here();
                }
            }
        }
        Ok(Type::Boolean)
    }

    fn how_a_primitive_is_tested(
        &mut self,
        kind: &Type,
        target: &Type,
        line: u32,
    ) -> Result<Option<Expression>, Diagnostic> {
        if target.is_reference() {
            let boxed = boxed_name(kind).unwrap_or("java/lang/Object");
            let fits = match target {
                Type::Object(named, _) => {
                    named == boxed
                        || named == "java/lang/Object"
                        || (named == "java/lang/Number"
                            && !matches!(kind, Type::Boolean | Type::Char))
                        || named == "java/lang/Comparable"
                        || named == "java/io/Serializable"
                }
                _ => false,
            };
            if !fits {
                return Err(at(
                    "EJ238",
                    line,
                    1,
                    format!(
                        "A {} can never be a {}, so this `case` could never match.",
                        kind.readable(),
                        target.readable()
                    ),
                ));
            }
            return Ok(None);
        }
        self.how_a_named_primitive_is_tested(kind, target, PATTERN_SUBJECT, line)
    }

    fn how_a_named_primitive_is_tested(
        &mut self,
        kind: &Type,
        target: &Type,
        name: &str,
        line: u32,
    ) -> Result<Option<Expression>, Diagnostic> {
        if target.is_reference() {
            return self.how_a_primitive_is_tested(kind, target, line);
        }
        match exactness(kind, target, name) {
            Some(test) => Ok(test),
            None => Err(at(
                "EJ238",
                line,
                1,
                format!(
                    "A {} can never be a {}, so this `case` could never match.",
                    kind.readable(),
                    target.readable()
                ),
            )),
        }
    }

    fn converted_for_a_pattern(
        &mut self,
        from: &Type,
        to: &Type,
        line: u32,
    ) -> Result<(), Diagnostic> {
        if from == to {
            return Ok(());
        }
        if matches!(to, Type::Byte | Type::Char | Type::Short) {
            if !from.is_int_like() {
                self.convert(from, &Type::Int, line)?;
            }
            self.op(match to {
                Type::Byte => 0x91,
                Type::Char => 0x92,
                _ => 0x93,
            });
            self.pops(1);
            self.pushes(to);
            return Ok(());
        }
        self.convert(from, to, line)
    }

    fn shape_dispatch(&mut self, arms: &[Arm], line: u32) -> Result<Dispatch, Diagnostic> {
        let object = Type::Object("java/lang/Object".to_string(), Vec::new());
        let held = self.declare(PATTERN_SUBJECT, object.clone());
        self.store(held, &object);

        let mut waiting: Vec<(Pending, usize)> = Vec::new();
        for (index, arm) in arms.iter().enumerate() {
            if arm.labels.iter().any(|one| matches!(one, Expression::Null)) {
                self.load(held, &object);
                self.pops(1);
                waiting.push((self.jump(0xc6), index));
                continue;
            }
            let Some(pattern) = arm.pattern.clone() else {
                continue;
            };
            let mut target = self.resolve(&pattern.what, arm.line)?;
            if !target.is_reference() {
                let Some(boxed) = boxed_name(&target) else {
                    return Err(at(
                        "EJ238",
                        arm.line,
                        arm.column,
                        format!(
                            "A `case` pattern names a class or an array, and a {} is neither.",
                            target.readable()
                        ),
                    ));
                };
                target = Type::Object(boxed.to_string(), Vec::new());
            }

            self.open();
            if pattern.taken {
                let whole = self.the_whole_pattern(
                    &Expression::Name(PATTERN_SUBJECT.to_string()),
                    &pattern,
                    arm.line,
                )?;
                self.value(&whole, arm.line)?;
            } else {
                let named = match &target {
                    Type::Object(named, _) => named.clone(),
                    other => other.descriptor(),
                };
                self.load(held, &object);
                let index_of = self.pool.class(&named);
                self.op2(0xc1, index_of);
                self.pops(1);
                self.pushes(&Type::Boolean);
            }
            match &arm.guard {
                None => {
                    self.pops(1);
                    waiting.push((self.jump(0x9a), index));
                    self.close();
                }
                Some(guard) => {
                    self.pops(1);
                    let past = self.jump(0x99);
                    if !pattern.taken {
                        self.bind_the_pattern(arm, held)?;
                    }
                    let found = self.truth(guard, arm.line)?;
                    if found != Type::Boolean {
                        return Err(at(
                            "EJ206",
                            arm.line,
                            arm.column,
                            "A `when` on a `case` wants a boolean.",
                        ));
                    }
                    self.pops(1);
                    waiting.push((self.jump(0x9a), index));
                    self.close();
                    self.land(past);
                    self.a_branch_lands_here();
                }
            }
        }
        let _ = line;
        let fallthrough = self.jump(0xa7);
        Ok(Dispatch::Chain {
            waiting,
            fallthrough,
        })
    }

    fn bind_a_pattern_here(&mut self, arm: &Arm) -> Result<(), Diagnostic> {
        if arm.pattern.is_none() {
            return Ok(());
        }
        let Some(subject) = self.local(PATTERN_SUBJECT).map(|held| held.slot) else {
            return Ok(());
        };
        self.bind_the_pattern(arm, subject)
    }

    fn bind_the_pattern(&mut self, arm: &Arm, subject: u16) -> Result<(), Diagnostic> {
        let Some(pattern) = &arm.pattern else {
            return Ok(());
        };
        let held = self
            .locals
            .iter()
            .rev()
            .find(|one| one.slot == subject)
            .map(|one| one.what.clone())
            .unwrap_or_else(|| Type::Object("java/lang/Object".to_string(), Vec::new()));
        self.load(subject, &held);
        let pattern = pattern.clone();
        self.bind_every_part(&pattern, &held, arm.line)
    }

    fn bind_every_part(
        &mut self,
        pattern: &Pattern,
        from: &Type,
        line: u32,
    ) -> Result<(), Diagnostic> {
        let target = match pattern.what {
            Written::Inferred => from.clone(),
            ref other => self.resolve(other, line)?,
        };
        if !from.is_reference() && target.is_reference() {
            self.box_or_unbox(from, &target);
        } else if from.is_reference() && !target.is_reference() {
            if let Some(boxed) = boxed_name(&target) {
                let index = self.pool.class(boxed);
                self.op2(0xc0, index);
                self.pops(1);
                let wrapper = Type::Object(boxed.to_string(), Vec::new());
                self.pushes(&wrapper);
                self.box_or_unbox(&wrapper, &target);
            }
        } else if !from.is_reference() && !target.is_reference() {
            self.converted_for_a_pattern(from, &target, line)?;
        } else if target.is_reference() && target != *from {
            let named = match &target {
                Type::Object(named, _) => named.clone(),
                other => other.descriptor(),
            };
            let index = self.pool.class(&named);
            self.op2(0xc0, index);
            self.pops(1);
            self.pushes(&target);
        }
        if !pattern.taken {
            if pattern.name.is_empty() {
                self.op(if target.width() == 2 { 0x58 } else { 0x57 });
                self.pops(i32::from(target.width()));
                return Ok(());
            }
            let slot = self.declare(&pattern.name, target.clone());
            self.store(slot, &target);
            return Ok(());
        }
        let Type::Object(class, _) = target.clone() else {
            return Err(at(
                "EJ208",
                line,
                1,
                format!("A {} has no components to take apart.", target.readable()),
            ));
        };
        let whole = self.declare("$took", target.clone());
        self.store(whole, &target);
        let components = self.what_a_record_holds(&class, line)?;
        for (part, (component, declared)) in pattern.parts.iter().zip(components.iter()) {
            self.load(whole, &target);
            let Some(signature) = self.signature_for(&class, component, &[], line)? else {
                return Err(self.no_such_method(&class, component, 0, line));
            };
            self.call_signature(&signature, line);
            self.pops(1);
            self.pushes(declared);
            self.bind_every_part(part, declared, line)?;
        }
        Ok(())
    }

    fn constant_dispatch(
        &mut self,
        class: &str,
        arms: &[Arm],
        line: u32,
    ) -> Result<Dispatch, Diagnostic> {
        let what = Type::Object(class.to_string(), Vec::new());
        let held = self.declare("$switch", what.clone());
        self.store(held, &what);

        let mut seen: Vec<String> = Vec::new();
        let mut waiting: Vec<(Pending, usize)> = Vec::new();
        for (index, arm) in arms.iter().enumerate() {
            for label in &arm.labels {
                let named = match label {
                    Expression::Name(named) => named,
                    Expression::Field { of, name }
                        if dotted_name(of)
                            .and_then(|written| resolve_named(self.classpath, self.unit, &written))
                            .is_some_and(|owner| owner == class) =>
                    {
                        name
                    }
                    _ => {
                        return Err(at(
                            "EJ239",
                            arm.line,
                            arm.column,
                            "A `case` in a `switch` over an enum names one of its constants.",
                        ))
                    }
                };
                if seen.contains(named) {
                    return Err(at(
                        "EJ240",
                        arm.line,
                        arm.column,
                        format!("`case {named}` is written twice in one `switch`."),
                    ));
                }
                seen.push(named.clone());

                self.load(held, &what);
                self.read_static_field(class, named, arm.line)?;

                self.pops(2);
                waiting.push((self.jump(0xa5), index));
            }
        }
        let _ = line;
        let fallthrough = self.jump(0xa7);
        Ok(Dispatch::Chain {
            waiting,
            fallthrough,
        })
    }

    fn case_constant(&self, label: &Expression) -> Option<Expression> {
        if let Expression::Name(name) = label {
            if self.local(name).is_some() {
                return None;
            }
        }
        if let Some(found) = constant_within(self.classpath, self.unit, label, 0) {
            return Some(found);
        }
        let Expression::Name(name) = label else {
            return None;
        };
        let mut owners = vec![self.this_class.clone()];
        owners.extend(self.the_classes_around());
        owners.extend(self.holders());
        owners.extend(self.imported_statically(name));
        owners
            .iter()
            .find_map(|owner| self.classpath.constant(owner, name).cloned())
    }

    fn integer_dispatch(&mut self, arms: &[Arm], line: u32) -> Result<Dispatch, Diagnostic> {
        let mut keys: Vec<(i32, usize)> = Vec::new();
        for (index, arm) in arms.iter().enumerate() {
            for label in &arm.labels {
                let value = self
                    .case_constant(label)
                    .as_ref()
                    .and_then(constant_int)
                    .ok_or_else(|| {
                        at(
                            "EJ239",
                            arm.line,
                            arm.column,
                            "A `case` over an integer wants a constant this compiler can read.",
                        )
                        .with_suggestion("A number, a character, or a `-` in front of one.")
                    })?;
                if keys.iter().any(|(held, _)| *held == value) {
                    return Err(at(
                        "EJ240",
                        arm.line,
                        arm.column,
                        format!("`case {value}` is written twice in one `switch`."),
                    ));
                }
                keys.push((value, index));
            }
        }
        keys.sort_by_key(|(value, _)| *value);
        let _ = line;

        let opcode_at = self.code.len();
        let low = keys.first().map(|(value, _)| *value).unwrap_or(0);
        let high = keys.last().map(|(value, _)| *value).unwrap_or(0);
        let span = i64::from(high) - i64::from(low) + 1;

        let table = !keys.is_empty() && span <= 2 * keys.len() as i64 + 8;

        self.op(if table { 0xaa } else { 0xab });
        self.pops(1);
        while !self.code.len().is_multiple_of(4) {
            self.code.push(0);
        }

        let default_slot = self.code.len();
        self.code.extend_from_slice(&[0; 4]);
        let mut slots: Vec<(usize, usize)> = Vec::new();
        if table {
            self.code.extend_from_slice(&low.to_be_bytes());
            self.code.extend_from_slice(&high.to_be_bytes());
            for value in low..=high {
                let slot = self.code.len();
                self.code.extend_from_slice(&[0; 4]);
                match keys.iter().find(|(held, _)| *held == value) {
                    Some((_, arm)) => slots.push((slot, *arm)),

                    None => slots.push((slot, usize::MAX)),
                }
                if value == high {
                    break;
                }
            }
        } else {
            self.code
                .extend_from_slice(&(keys.len() as u32).to_be_bytes());
            for (value, arm) in &keys {
                self.code.extend_from_slice(&value.to_be_bytes());
                let slot = self.code.len();
                self.code.extend_from_slice(&[0; 4]);
                slots.push((slot, *arm));
            }
        }

        Ok(Dispatch::Instruction {
            opcode_at,
            default_slot,
            slots,
        })
    }

    fn text_dispatch(&mut self, arms: &[Arm], line: u32) -> Result<Dispatch, Diagnostic> {
        let text = Type::Object("java/lang/String".to_string(), Vec::new());
        let held = self.declare("$switch", text.clone());
        self.store(held, &text);

        let mut seen: Vec<String> = Vec::new();
        let mut waiting: Vec<(Pending, usize)> = Vec::new();
        for (index, arm) in arms.iter().enumerate() {
            for label in &arm.labels {
                let Some(Expression::Str(value)) = self.case_constant(label) else {
                    return Err(at(
                        "EJ239",
                        arm.line,
                        arm.column,
                        "A `case` in a `switch` over a String wants a String constant.",
                    ));
                };
                let value = &value;
                if seen.contains(value) {
                    return Err(at(
                        "EJ240",
                        arm.line,
                        arm.column,
                        format!("`case {value:?}` is written twice in one `switch`."),
                    ));
                }
                seen.push(value.clone());

                self.load(held, &text);
                self.push_string(value);
                let equals =
                    self.pool
                        .method("java/lang/String", "equals", "(Ljava/lang/Object;)Z", false);
                self.op2(0xb6, equals);
                self.pops(1);
                self.pops(1);
                waiting.push((self.jump(0x9a), index));
            }
        }

        if seen.is_empty() {
            self.load(held, &text);
            let class_of =
                self.pool
                    .method("java/lang/Object", "getClass", "()Ljava/lang/Class;", false);
            self.op2(0xb6, class_of);
            self.op(0x57);
            self.pops(1);
        }
        let _ = line;

        let fallthrough = self.jump(0xa7);
        Ok(Dispatch::Chain {
            waiting,
            fallthrough,
        })
    }

    fn try_catch(
        &mut self,
        body: &[Positioned<Statement>],
        catches: &[Catch],
        finally: Option<&[Positioned<Statement>]>,
        line: u32,
    ) -> Result<(), Diagnostic> {
        let outer = self.finallys.len();
        if let Some(finally) = finally {
            self.finallys.push(Cleanup::Block(finally.to_vec()));
        }

        let mut takes: Vec<String> = Vec::new();
        for catch in catches {
            for written in &catch.types {
                if let Ok(Type::Object(named, _)) = self.resolve(written, catch.line) {
                    takes.push(named);
                }
            }
        }
        let begun = self.code.len();
        let before = self.slots.clone();
        self.caught.push(takes);
        self.open();
        let mut written = Ok(());
        for one in body {
            written = self.statement(one);
            if written.is_err() {
                break;
            }
        }
        self.close();
        self.caught.pop();
        written?;
        let ended = self.code.len();

        let mut outs: Vec<Pending> = Vec::new();
        if !body.iter().any(|one| never_completes(&one.node)) {
            self.run_finallys(outer)?;
            outs.push(self.jump(0xa7));
        }

        let throwable = Type::Object("java/lang/Throwable".to_string(), Vec::new());
        let mut caught: Vec<(usize, usize)> = Vec::new();
        for catch in catches {
            let mut classes = Vec::new();
            for written in &catch.types {
                let resolved = self.resolve(written, catch.line)?;
                let Type::Object(name, _) = resolved else {
                    return Err(at(
                        "EJ241",
                        catch.line,
                        catch.column,
                        format!(
                            "`catch` wants a class, and {} is not one.",
                            resolved.readable()
                        ),
                    ));
                };
                classes.push(name);
            }

            let held = if classes.len() == 1 {
                Type::Object(classes[0].clone(), Vec::new())
            } else {
                throwable.clone()
            };

            let target = self.code.len();
            for name in &classes {
                self.protect(begun, ended, target, Some(name.clone()), outer);
            }

            self.slots = before.clone();
            self.stack_is(vec![Verified::of(&held)]);
            self.a_branch_lands_here();
            self.open();
            let slot = self.declare(&catch.name, held.clone());
            self.store(slot, &held);

            let body_begun = self.code.len();
            self.rethrown.push((slot, classes.clone()));
            let done: Result<(), Diagnostic> =
                catch.body.iter().try_for_each(|one| self.statement(one));
            self.rethrown.pop();
            done?;
            let body_ended = self.code.len();
            self.close();
            caught.push((body_begun, body_ended));

            if !catch.body.iter().any(|one| never_completes(&one.node)) {
                self.run_finallys(outer)?;
                outs.push(self.jump(0xa7));
            }
        }

        if let Some(finally) = finally {
            let target = self.code.len();
            self.protect(begun, ended, target, None, outer);
            for (from, to) in caught {
                self.protect(from, to, target, None, outer);
            }

            self.slots = before.clone();
            self.stack_is(vec![Verified::of(&throwable)]);
            self.a_branch_lands_here();
            self.open();
            let slot = self.declare("$thrown", throwable.clone());
            self.store(slot, &throwable);

            let held = std::mem::take(&mut self.finallys);
            self.finallys = held[..outer].to_vec();
            for one in finally {
                self.statement(one)?;
            }
            self.finallys = held;

            if self.reachable {
                self.load(slot, &throwable);
                self.op(0xbf);
                self.set_depth(0);
            }
            self.close();
        }

        self.finallys.truncate(outer);
        let _ = line;

        if outs.is_empty() {
            return Ok(());
        }
        for pending in outs {
            self.land(pending);
        }
        self.set_depth(0);
        self.a_branch_lands_here();
        Ok(())
    }

    fn anonymous(
        &mut self,
        what: &Written,
        arguments: &[Expression],
        body: &Body,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let target = self.resolve(what, line)?;
        self.synthesise(target, arguments, body, false, line)
    }

    fn lambda(
        &mut self,
        parameters: &[(Option<Written>, String)],
        body: &[Positioned<Statement>],
        expression: bool,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let Some(target) = self.expecting.take() else {
            return Err(at(
                "EJ250",
                line,
                1,
                "There is nothing here saying which interface this stands for.",
            )
            .with_suggestion(
                "A lambda takes its type from what it is handed to: an argument, a \
                 declared variable, or what a method returns. Write the type, or write \
                 the class out.",
            ));
        };
        let Type::Object(named, holds) = target.clone() else {
            return Err(at(
                "EJ250",
                line,
                1,
                format!("A lambda cannot stand for a {}.", target.readable()),
            ));
        };

        let single = self.the_one_method_of(&named, line)?;
        let single = as_written(&single, &holds);
        if single.parameters.len() != parameters.len() {
            return Err(at(
                "EJ251",
                line,
                1,
                format!(
                    "`{}.{}` takes {} argument(s) and this was written with {}.",
                    named.replace('/', "."),
                    single.name,
                    single.parameters.len(),
                    parameters.len()
                ),
            ));
        }

        let mut written = Vec::new();
        for ((given, name), wanted) in parameters.iter().zip(single.parameters.iter()) {
            if let Some(given) = given {
                let found = self.resolve(given, line)?;
                if found != *wanted {
                    return Err(at(
                        "EJ251",
                        line,
                        1,
                        format!(
                            "`{name}` is written as a {} and `{}` hands over a {}.",
                            found.readable(),
                            single.name,
                            wanted.readable()
                        ),
                    ));
                }
            }
            let Some(shape) = written_for(wanted) else {
                return Err(at(
                    "EJ251",
                    line,
                    1,
                    format!("`{}` hands over something this cannot name.", single.name),
                ));
            };
            written.push((shape, name.clone()));
        }

        let mut inside = body.to_vec();
        if expression && single.returns != Type::Void {
            if let Some(last) = inside.pop() {
                let Statement::Express(value) = last.node else {
                    return Err(at(
                        "EJ251",
                        line,
                        1,
                        "A lambda body has to produce a value.",
                    ));
                };
                inside.push(Positioned {
                    node: Statement::Return(Some(value)),
                    line: last.line,
                    column: last.column,
                });
            }
        }

        let Some(returns) = written_for(&single.returns).or(Some(Written::Void)) else {
            unreachable!("void is nameable")
        };
        let made = Body {
            static_setup: Vec::new(),
            fields: Vec::new(),
            methods: vec![Method {
                modifiers: Modifiers {
                    public: true,
                    ..Modifiers::default()
                },
                returns: if single.returns == Type::Void {
                    Written::Void
                } else {
                    returns
                },
                name: single.name.clone(),
                parameters: written,
                body: Some(inside),
                constructor: false,
                variadic: false,
                line,
                bridge: false,
                annotations: Vec::new(),
                parameter_annotations: Vec::new(),
                default_value: None,
                own_variables: Vec::new(),
                own_bounds: Vec::new(),

                throws: single
                    .throws
                    .iter()
                    .map(|one| one.replace('/', "."))
                    .collect(),
            }],
            instance_setup: Vec::new(),
            nested: Vec::new(),
        };
        self.synthesise(target, &[], &made, true, line)
    }

    fn method_reference(
        &mut self,
        on: &Expression,
        name: &str,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let Some(target) = self.expecting.clone() else {
            return Err(at(
                "EJ250",
                line,
                1,
                "There is nothing here saying which interface this stands for.",
            )
            .with_suggestion(
                "A method reference takes its type from what it is handed to: an \
                 argument, a declared variable, or what a method returns.",
            ));
        };
        let Type::Object(named, holds) = target else {
            return Err(at(
                "EJ250",
                line,
                1,
                "A method reference cannot stand for something that is not a class.",
            ));
        };
        let single = self.the_one_method_of(&named, line)?;

        let single = as_written(&single, &holds);

        let mut parameters = Vec::new();
        let mut arguments = Vec::new();
        for (position, wanted) in single.parameters.iter().enumerate() {
            let held = format!("$given{position}");
            let Some(shape) = written_for(wanted) else {
                return Err(at(
                    "EJ251",
                    line,
                    1,
                    format!("`{}` hands over something this cannot name.", single.name),
                ));
            };
            parameters.push((Some(shape), held.clone()));
            arguments.push(Expression::Name(held));
        }

        let call = if name == "<init>" {
            let Some(class) = written_as_a_path(on) else {
                return Err(at(
                    "EJ251",
                    line,
                    1,
                    "`::new` is written on the name of a class.",
                ));
            };
            Expression::New {
                what: Written::Named(class.clone(), Vec::new()),
                arguments,
                outer: None,
            }
        } else if self.called_on_the_first_one_handed_over(on, name, arguments.len()) {
            let receiver = arguments.remove(0);
            Expression::Call {
                on: Some(Box::new(receiver)),
                super_call: false,
                held: Vec::new(),
                name: name.to_string(),
                arguments,
            }
        } else {
            Expression::Call {
                on: Some(Box::new(on.clone())),
                super_call: false,
                held: Vec::new(),
                name: name.to_string(),
                arguments,
            }
        };

        let body = vec![Positioned {
            node: Statement::Express(call),
            line,
            column: 1,
        }];
        self.expecting = Some(Type::Object(named, holds));
        self.lambda(&parameters, &body, true, line)
    }

    fn called_on_the_first_one_handed_over(
        &mut self,
        on: &Expression,
        name: &str,
        handed: usize,
    ) -> bool {
        if handed == 0 {
            return false;
        }
        let Some(owner) = self.class_a_reference_names(on) else {
            return false;
        };
        if self
            .candidates(&owner, name, handed)
            .iter()
            .any(|one| one.static_)
        {
            return false;
        }
        self.candidates(&owner, name, handed - 1)
            .iter()
            .any(|one| !one.static_)
    }

    fn class_a_reference_names(&mut self, on: &Expression) -> Option<String> {
        match on {
            Expression::Name(maybe) => {
                if !self.meant_as_a_class(maybe) {
                    return None;
                }
                self.resolve_class(maybe, 1).ok()
            }
            other => written_as_a_path(other)
                .filter(|path| self.meant_as_a_class(path.split('.').next().unwrap_or_default()))
                .and_then(|path| resolve_named(self.classpath, self.unit, &path)),
        }
    }

    fn how_loosely_here(&mut self, arguments: &[Expression]) -> Vec<Loosely> {
        let mut said = how_loosely(arguments);
        for (at, argument) in arguments.iter().enumerate() {
            let Expression::Lambda {
                parameters,
                body,
                expression: true,
                ..
            } = argument
            else {
                continue;
            };
            if !parameters.is_empty() || said[at] != Loosely::Shaped(0, Gives::Either) {
                continue;
            }
            let Some(Statement::Express(called @ Expression::Call { .. })) =
                body.first().map(|one| &one.node)
            else {
                continue;
            };
            let outside = self.expecting.take();
            let found = self.type_of(called, 1);
            self.expecting = outside;
            if found.is_ok_and(|what| what == Type::Void) {
                said[at] = Loosely::Shaped(0, Gives::Nothing);
            }
        }
        for (at, argument) in arguments.iter().enumerate() {
            let Expression::MethodRef { on, name, .. } = argument else {
                continue;
            };
            let named = match on.as_ref() {
                Expression::Name(maybe) if self.meant_as_a_class(maybe) => {
                    self.resolve_class(maybe, 1).ok()
                }
                other => written_as_a_path(other)
                    .filter(|path| {
                        self.meant_as_a_class(path.split('.').next().unwrap_or_default())
                    })
                    .and_then(|path| resolve_named(self.classpath, self.unit, &path)),
            };
            let Some(owner) = named else {
                continue;
            };
            let mut counts: Vec<usize> = Vec::new();

            for arity in 0..=8usize {
                let held = if name == "new" {
                    self.classpath.find_methods(&owner, "<init>", arity)
                } else {
                    self.classpath.find_methods(&owner, name, arity)
                };
                for found in held {
                    let takes = if found.static_ || name == "new" {
                        arity
                    } else {
                        arity + 1
                    };
                    if !counts.contains(&takes) {
                        counts.push(takes);
                    }
                }
            }
            if !counts.is_empty() {
                said[at] = Loosely::Anywhere(counts);
            }
        }
        said
    }

    fn how_many_the_one_method_takes(&self, what: &Type) -> Option<usize> {
        let Type::Object(named, _) = what else {
            return None;
        };
        self.the_one_method_of(named, 1)
            .ok()
            .map(|found| found.parameters.len())
    }

    fn the_one_method_of(&self, named: &str, line: u32) -> Result<Signature, Diagnostic> {
        let an_interface = match self.classpath.get(named) {
            Some(known) => known.interface,
            None => BUILT_IN_INTERFACES.contains(&named),
        };
        if !an_interface {
            return Err(at(
                "EJ250",
                line,
                1,
                format!(
                    "`{}` is a class, and a lambda can only stand for an interface.",
                    named.replace('/', ".")
                ),
            )
            .with_suggestion(
                "A lambda takes its type from what it is handed to. Name the interface \
                    it stands for, or write the class out.",
            ));
        }

        let counts = |one: &Signature| {
            !one.static_
                && one.name != "<init>"
                && one.abstract_
                && !matches!(
                    (one.name.as_str(), one.parameters.len()),
                    ("equals", 1) | ("hashCode", 0) | ("toString", 0)
                )
        };
        let mut found: Vec<Signature> = Vec::new();
        for one in self.classpath.ancestors(named) {
            let Some(known) = self.classpath.get(&one) else {
                continue;
            };
            for held in &known.methods {
                if found
                    .iter()
                    .any(|before| before.name == held.name && before.parameters == held.parameters)
                {
                    continue;
                }
                found.push(held.clone());
            }
        }
        found.retain(counts);

        if found.is_empty() {
            for (class, name, descriptor, static_) in BUILT_IN_METHODS {
                if *class != named || *static_ {
                    continue;
                }
                let Some((parameters, returns)) = read_descriptor(descriptor) else {
                    continue;
                };
                found.push(Signature {
                    owner: named.to_string(),
                    name: (*name).to_string(),
                    parameters,
                    returns,
                    static_: false,
                    interface: true,
                    variadic: false,
                    abstract_: true,
                    returns_variable: None,
                    parameter_variables: Vec::new(),
                    taken_stands_for: Vec::new(),
                    returns_stands_for: Standing::Fixed,
                    own_variables: Vec::new(),
                    witnessed: Vec::new(),
                    throws: Vec::new(),
                });
            }
            found.retain(counts);
        }
        match found.len() {
            1 => Ok(found.remove(0)),
            0 => Err(at(
                "EJ250",
                line,
                1,
                format!(
                    "`{}` has no method this compilation knows, so there is nothing for a \
                     lambda to be.",
                    named.replace('/', ".")
                ),
            )
            .with_suggestion("Hand the class file that declares it over as a dependency.")),
            many => Err(at(
                "EJ250",
                line,
                1,
                format!(
                    "`{}` has {many} methods, so a lambda cannot say which one it is.",
                    named.replace('/', ".")
                ),
            )
            .with_suggestion("Write the class out, naming the method.")),
        }
    }

    fn constructor_taking(&self, class: &str, count: usize) -> Option<Signature> {
        self.classpath
            .find_method(class, "<init>", count)
            .cloned()
            .or_else(|| built_in_method(class, "<init>", count))
    }

    fn synthesise(
        &mut self,
        target: Type,
        arguments: &[Expression],
        body: &Body,
        stands_for_a_lambda: bool,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let Type::Object(named, _) = target.clone() else {
            return Err(at(
                "EJ248",
                line,
                1,
                format!(
                    "A class written where it is used has to name a class or an \
                     interface, and {} is neither.",
                    target.readable()
                ),
            ));
        };
        let is_interface = match self.classpath.get(&named) {
            Some(known) => known.interface,
            None => BUILT_IN_INTERFACES.contains(&named.as_str()),
        };
        if is_interface && !arguments.is_empty() {
            return Err(at(
                "EJ248",
                line,
                1,
                "An interface has no constructor, so there is nothing to hand it.",
            ));
        }

        let mut wanted: Vec<String> = Vec::new();
        names_read(body, &mut wanted);
        let mut captured: Vec<Local> = Vec::new();
        for name in &wanted {
            let Some(local) = self.local(name) else {
                continue;
            };

            if body.fields.iter().any(|held| held.name == *name)
                || self.classpath.find_field(&named, name).is_some()
            {
                continue;
            }
            if captured.iter().any(|held| held.slot == local.slot) {
                continue;
            }
            captured.push(local);
        }
        captured.sort_by_key(|held| held.slot);

        let mut copied: Vec<(String, Type)> = Vec::new();
        if self.unit.stands_for_a_lambda && !self.static_ {
            for name in &wanted {
                if self.local(name).is_some() || copied.iter().any(|(held, _)| held == name) {
                    continue;
                }
                let Some(field) = self
                    .unit
                    .fields
                    .iter()
                    .find(|held| {
                        held.name == *name && held.name != OUTER && !held.modifiers.static_
                    })
                    .cloned()
                else {
                    continue;
                };
                let what = self.resolve(&field.what, line)?;
                copied.push((name.clone(), what));
            }
        }

        let mut assigned: Vec<String> = Vec::new();
        names_assigned(body, &mut assigned);
        for name in &assigned {
            if captured.iter().any(|held| held.name == *name) {
                return Err(at(
                    "EJ249",
                    line,
                    1,
                    format!(
                        "`{name}` belongs to the method around this class, and this \
                         class writes to it."
                    ),
                )
                .with_suggestion(
                    "What a class written here reads from around it is copied in, so a \
                     write would not be seen outside. Use a field, or an array of one.",
                ));
            }
        }

        let constructors: Vec<Signature> = if is_interface {
            Vec::new()
        } else {
            self.classpath
                .get(&named)
                .map(|known| {
                    known
                        .methods
                        .iter()
                        .filter(|one| {
                            one.name == "<init>" && one.parameters.len() == arguments.len()
                        })
                        .cloned()
                        .collect()
                })
                .unwrap_or_default()
        };
        let mut given = Vec::new();
        for (at, argument) in arguments.iter().enumerate() {
            let wanted = match constructors.as_slice() {
                [only] => only.parameters.get(at).cloned().filter(Type::is_reference),
                _ => None,
            };
            self.expecting = wanted;
            let found = self.type_of(argument, line);
            self.expecting = None;
            given.push(found?);
        }

        let inside_a_written_one = self.unit.stands_for_a_lambda;
        let (outer, through_the_one_around) = match () {
            _ if self.static_ => (None, false),
            _ if inside_a_written_one => (self.unit.outer.clone(), true),
            _ => (Some(self.this_class.clone()), false),
        };
        let index = self.made.len() + 1;
        let name = format!("{}${index}", self.unit.name);

        let mut fields = Vec::new();
        let mut parameters: Vec<(Written, String)> = Vec::new();
        let mut filling: Vec<Positioned<Statement>> = Vec::new();
        let held = |what: Written, name: &str, line: u32| Field {
            modifiers: Modifiers {
                private: true,
                final_: true,
                ..Modifiers::default()
            },
            what,
            name: name.to_string(),
            value: None,
            line,
            annotations: Vec::new(),
        };
        let fill = |name: &str, line: u32| Positioned {
            node: Statement::Express(Expression::Assign {
                target: Box::new(Expression::Field {
                    of: Box::new(Expression::This),
                    name: name.to_string(),
                }),
                operator: None,
                value: Box::new(Expression::Name(name.to_string())),
            }),
            line,
            column: 1,
        };

        if let Some(enclosing) = &outer {
            let written = Written::Named(enclosing.replace('/', "."), Vec::new());
            fields.push(held(written.clone(), OUTER, line));
            parameters.push((written, OUTER.to_string()));
            filling.push(fill(OUTER, line));
        }
        for local in &captured {
            let Some(written) = written_for(&local.what) else {
                return Err(at(
                    "EJ249",
                    line,
                    1,
                    format!(
                        "`{}` is a {} and a class written here cannot hold one.",
                        local.name,
                        local.what.readable()
                    ),
                ));
            };
            fields.push(held(written.clone(), &local.name, line));
            parameters.push((written, local.name.clone()));
            filling.push(fill(&local.name, line));
        }
        for (name, what) in &copied {
            let Some(written) = written_for(what) else {
                return Err(at(
                    "EJ249",
                    line,
                    1,
                    format!(
                        "`{name}` is a {} and a class written here cannot hold one.",
                        what.readable()
                    ),
                ));
            };
            fields.push(held(written.clone(), name, line));
            parameters.push((written, name.clone()));
            filling.push(fill(name, line));
        }

        let mut chain = Vec::new();
        for (position, one) in given.iter().enumerate() {
            let Some(written) = written_for(one) else {
                return Err(at(
                    "EJ248",
                    line,
                    1,
                    format!(
                        "A {} cannot be handed to a superclass here.",
                        one.readable()
                    ),
                ));
            };
            let argument = format!("$arg{position}");
            parameters.push((written, argument.clone()));
            chain.push(Expression::Name(argument));
        }

        let constructor_body = filled_before_the_chain(
            vec![Positioned {
                node: Statement::Chain {
                    to_super: true,
                    arguments: chain,
                },
                line,
                column: 1,
            }],
            &filling,
            &[],
            line,
        );

        let mut methods = vec![Method {
            modifiers: Modifiers::default(),
            returns: Written::Void,
            name: "<init>".to_string(),
            parameters: parameters.clone(),
            body: Some(constructor_body),
            constructor: true,
            variadic: false,
            line,
            bridge: false,
            annotations: Vec::new(),
            parameter_annotations: Vec::new(),
            default_value: None,
            own_variables: Vec::new(),
            own_bounds: Vec::new(),
            throws: Vec::new(),
        }];
        methods.extend(body.methods.iter().cloned());
        fields.extend(body.fields.iter().cloned());

        let dotted = named.replace('/', ".");
        self.made.push(Unit {
            shape: Shape::Class,
            package: self.unit.package.clone(),
            imports: self.unit.imports.clone(),
            modifiers: Modifiers {
                final_: true,
                ..Modifiers::default()
            },
            name: name.clone(),
            extends: (!is_interface).then(|| dotted.clone()),
            implements: if is_interface {
                let mut all = vec![dotted];
                all.extend(std::mem::take(&mut self.markers));
                all
            } else {
                Vec::new()
            },
            fields,
            methods,
            instance_setup: body.instance_setup.clone(),
            static_setup: body.static_setup.clone(),
            outer: outer.clone(),
            permits: Vec::new(),
            above: Vec::new(),
            stands_for_a_lambda,
            annotation: false,
            annotations: Vec::new(),
            static_imports: self.unit.static_imports.clone(),

            source: self.unit.source.clone(),

            own_variables: Vec::new(),

            enclosed_by: Some((self.this_class.clone(), self.within.clone())),
        });
        for one in &body.nested {
            let mut held = one.clone();
            let own = !held.name.contains('$');
            held.name = format!("{name}${}", one.name);
            held.package = self.unit.package.clone();
            held.imports = self.unit.imports.clone();
            held.static_imports = self.unit.static_imports.clone();
            held.source = self.unit.source.clone();
            if own && held.shape == Shape::Class && !held.modifiers.static_ {
                held = belonging_to_an_instance(held, &name, &self.unit.package);
            }
            self.made.push(held);
        }

        let made = match &self.unit.package {
            Some(package) => format!("{}/{name}", package.replace('.', "/")),
            None => name,
        };
        let index = self.pool.class(&made);
        self.op2(0xbb, index);
        self.pushes(&Type::Object(made.clone(), Vec::new()));
        self.duplicates();

        let mut descriptor = String::from("(");
        if let Some(enclosing) = &outer {
            let held = Type::Object(enclosing.clone(), Vec::new());
            if through_the_one_around {
                self.reach_the_enclosing_instance(enclosing)?;
            } else {
                self.load(0, &held);
            }
            descriptor.push_str(&held.descriptor());
        }
        for local in &captured {
            self.load(local.slot, &local.what);
            descriptor.push_str(&local.what.descriptor());
        }
        for (name, what) in &copied {
            let this = Type::Object(self.this_class.clone(), Vec::new());
            self.load(0, &this);
            let shape = what.descriptor();
            let index = self.pool.field(&self.this_class.clone(), name, &shape);
            self.op2(0xb4, index);
            self.pops(1);
            self.pushes(what);
            descriptor.push_str(&shape);
        }
        for (argument, want) in arguments.iter().zip(given.iter()) {
            self.expecting = want.is_reference().then(|| want.clone());
            let found = self.value(argument, line);
            self.expecting = None;
            let found = found?;
            if !found.is_reference() {
                self.convert(&found, want, line)?;
            } else if let (Type::Object(had, _), Type::Object(wanted, _)) = (&found, want) {
                if had != wanted && !self.reaches(&found, want) {
                    let index = self.pool.class(wanted);
                    self.op2(0xc0, index);
                    self.pops(1);
                    self.pushes(want);
                }
            }
            descriptor.push_str(&want.descriptor());
        }
        descriptor.push_str(")V");

        let init = self.pool.method(&made, "<init>", &descriptor, false);
        self.op2(0xb7, init);
        let taken: i32 = read_descriptor(&descriptor)
            .map(|(parameters, _)| parameters.iter().map(|one| i32::from(one.width())).sum())
            .unwrap_or(0);
        self.pops(taken + 1);
        Ok(target)
    }

    fn a_class_written_in_here(&mut self, units: &[Unit], line: u32) -> Result<(), Diagnostic> {
        let Some((first, beside)) = units.split_first() else {
            return Ok(());
        };

        let mut wanted: Vec<String> = Vec::new();
        for method in &first.methods {
            for held in method.body.iter().flatten() {
                names_in_statement(&held.node, &mut wanted);
            }
        }
        for field in &first.fields {
            if let Some(value) = &field.value {
                names_in_expression(value, &mut wanted);
            }
        }
        for held in first.instance_setup.iter().chain(first.static_setup.iter()) {
            names_in_statement(&held.node, &mut wanted);
        }

        let mut its_own: Vec<String> = first.fields.iter().map(|held| held.name.clone()).collect();
        let above = first
            .extends
            .iter()
            .chain(first.implements.iter())
            .filter_map(|named| resolve_named(self.classpath, self.unit, named))
            .collect::<Vec<_>>();
        for owner in &above {
            if let Some(known) = self.classpath.get(owner) {
                its_own.extend(known.fields.iter().map(|(name, _, _)| name.clone()));
            }
        }
        let mut captured: Vec<Local> = Vec::new();
        for name in &wanted {
            let Some(local) = self.local(name) else {
                continue;
            };
            if its_own.contains(name)
                || above
                    .iter()
                    .any(|owner| self.classpath.find_field(owner, name).is_some())
            {
                continue;
            }
            if captured.iter().any(|held| held.slot == local.slot) {
                continue;
            }
            captured.push(local);
        }
        captured.sort_by_key(|held| held.slot);

        let inside_a_written_one = self.unit.stands_for_a_lambda;
        let (outer, through_the_one_around) = match () {
            _ if self.static_ => (None, false),
            _ if inside_a_written_one => (self.unit.outer.clone(), true),
            _ => (Some(self.this_class.clone()), false),
        };

        let name = format!("{}${}", self.unit.name, first.name);
        let mut made = first.clone();
        made.name = name.clone();
        made.package = self.unit.package.clone();
        made.source = self.unit.source.clone();
        made.imports = self.unit.imports.clone();
        made.static_imports = self.unit.static_imports.clone();
        made.outer = outer.clone();

        let mut fields = Vec::new();
        let mut parameters: Vec<(Written, String)> = Vec::new();
        let mut filling: Vec<Positioned<Statement>> = Vec::new();
        let held = |what: Written, name: &str| Field {
            modifiers: Modifiers {
                private: true,
                final_: true,
                ..Modifiers::default()
            },
            what,
            name: name.to_string(),
            value: None,
            line,
            annotations: Vec::new(),
        };
        let fill = |name: &str| Positioned {
            node: Statement::Express(Expression::Assign {
                target: Box::new(Expression::Field {
                    of: Box::new(Expression::This),
                    name: name.to_string(),
                }),
                operator: None,
                value: Box::new(Expression::Name(name.to_string())),
            }),
            line,
            column: 1,
        };
        if let Some(enclosing) = &outer {
            let written = Written::Named(enclosing.replace('/', "."), Vec::new());
            fields.push(held(written.clone(), OUTER));
            parameters.push((written, OUTER.to_string()));
            filling.push(fill(OUTER));
        }
        for local in &captured {
            let Some(written) = written_for(&local.what) else {
                return Err(at(
                    "EJ249",
                    line,
                    1,
                    format!(
                        "`{}` is a {} and a class written here cannot hold one.",
                        local.name,
                        local.what.readable()
                    ),
                ));
            };
            fields.push(held(written.clone(), &local.name));
            parameters.push((written, local.name.clone()));
            filling.push(fill(&local.name));
        }
        fields.extend(made.fields);
        made.fields = fields;

        if made.shape == Shape::Interface {
            made.fields.retain(|held| held.value.is_some());
            self.made.push(made);
            for one in beside {
                let mut held = one.clone();
                held.package = self.unit.package.clone();
                held.source = self.unit.source.clone();
                held.source = self.unit.source.clone();
                held.imports = self.unit.imports.clone();
                held.static_imports = self.unit.static_imports.clone();
                self.made.push(held);
            }
            let internal = match &self.unit.package {
                Some(package) => format!("{}/{name}", package.replace('.', "/")),
                None => name,
            };
            self.locally.push(Locally {
                written: first.name.clone(),
                internal,
                outer: None,
                through_the_one_around: false,
                captured: Vec::new(),
            });
            return Ok(());
        }

        if !made.methods.iter().any(|one| one.constructor) {
            made.methods.push(Method {
                modifiers: Modifiers {
                    public: true,
                    ..Modifiers::default()
                },
                returns: Written::Void,
                name: "<init>".to_string(),
                parameters: Vec::new(),
                body: Some(Vec::new()),
                constructor: true,
                variadic: false,
                line,
                bridge: false,
                annotations: Vec::new(),
                parameter_annotations: Vec::new(),
                default_value: None,
                own_variables: Vec::new(),
                own_bounds: Vec::new(),
                throws: Vec::new(),
            });
        }
        for method in &mut made.methods {
            if !method.constructor {
                continue;
            }
            let mut taken = parameters.clone();
            taken.append(&mut method.parameters);
            method.parameters = taken;
            let body = method.body.take().unwrap_or_default();
            let handed_on: Vec<String> = parameters.iter().map(|(_, name)| name.clone()).collect();
            method.body = Some(filled_before_the_chain(body, &filling, &handed_on, line));
        }

        self.made.push(made);
        for one in beside {
            let mut held = one.clone();
            held.package = self.unit.package.clone();
            held.imports = self.unit.imports.clone();
            held.static_imports = self.unit.static_imports.clone();
            self.made.push(held);
        }

        let internal = match &self.unit.package {
            Some(package) => format!("{}/{name}", package.replace('.', "/")),
            None => name,
        };
        self.locally.push(Locally {
            written: first.name.clone(),
            internal,
            outer,
            through_the_one_around,
            captured,
        });
        Ok(())
    }

    fn the_whole_pattern(
        &mut self,
        on: &Expression,
        pattern: &Pattern,
        line: u32,
    ) -> Result<Expression, Diagnostic> {
        let named = format!("$took{}", self.taken_apart);
        self.taken_apart += 1;
        let mut held = Expression::InstanceOf {
            of: Box::new(on.clone()),
            pattern: Box::new(Pattern {
                what: pattern.what.clone(),
                name: named.clone(),
                parts: Vec::new(),
                taken: false,
            }),
        };
        let target = self.resolve(&pattern.what, line)?;
        let Type::Object(class, _) = target.clone() else {
            return Err(at(
                "EJ208",
                line,
                1,
                format!("A {} has no components to take apart.", target.readable()),
            ));
        };
        let components = self.what_a_record_holds(&class, line)?;
        if components.len() != pattern.parts.len() {
            return Err(at(
                "EJ208",
                line,
                1,
                format!(
                    "`{}` holds {} component(s) and the pattern names {}.",
                    class.replace('/', "."),
                    components.len(),
                    pattern.parts.len()
                ),
            ));
        }
        for (part, (component, declared)) in pattern.parts.iter().zip(components.iter()) {
            let reach = Expression::Call {
                on: Some(Box::new(Expression::Name(named.clone()))),
                super_call: false,
                held: Vec::new(),
                name: component.clone(),
                arguments: Vec::new(),
            };

            let inside = if part.taken {
                self.the_whole_pattern(&reach, part, line)?
            } else {
                let wanted = match part.what {
                    Written::Inferred => declared.clone(),
                    ref other => self.resolve(other, line)?,
                };
                if wanted.is_reference() && *declared != wanted {
                    Expression::InstanceOf {
                        of: Box::new(reach),
                        pattern: Box::new(Pattern {
                            what: part.what.clone(),
                            name: part.name.clone(),
                            parts: Vec::new(),
                            taken: false,
                        }),
                    }
                } else {
                    Expression::Bind {
                        name: part.name.clone(),
                        what: match part.what {
                            Written::Inferred => written_for(declared).unwrap_or(Written::Inferred),
                            ref other => other.clone(),
                        },
                        value: Box::new(reach),
                    }
                }
            };
            held = Expression::Binary {
                operator: Binary::AndAlso,
                left: Box::new(held),
                right: Box::new(inside),
            };
        }
        Ok(held)
    }

    fn what_a_record_holds(
        &self,
        class: &str,
        line: u32,
    ) -> Result<Vec<(String, Type)>, Diagnostic> {
        let Some(known) = self.classpath.get(class) else {
            return Err(at(
                "EJ208",
                line,
                1,
                format!(
                    "`{}` is not a record this compilation knows, so there is nothing to \
                     take apart.",
                    class.replace('/', ".")
                ),
            ));
        };
        Ok(known
            .fields
            .iter()
            .filter(|(_, _, static_)| !static_)
            .map(|(name, what, _)| (name.clone(), what.clone()))
            .collect())
    }

    fn call_on_an_array(
        &mut self,
        array: &Type,
        name: &str,
        arguments: &[Expression],
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let owner = array.descriptor();
        let object = Type::Object("java/lang/Object".to_string(), Vec::new());
        let (descriptor, returns) = match (name, arguments.len()) {
            ("clone", 0) => ("()Ljava/lang/Object;", array.clone()),
            ("hashCode", 0) => ("()I", Type::Int),
            ("equals", 1) => ("(Ljava/lang/Object;)Z", Type::Boolean),
            ("toString", 0) => (
                "()Ljava/lang/String;",
                Type::Object("java/lang/String".to_string(), Vec::new()),
            ),
            ("getClass", 0) => (
                "()Ljava/lang/Class;",
                Type::Object("java/lang/Class".to_string(), Vec::new()),
            ),
            _ => {
                return Err(at(
                    "EJ226",
                    line,
                    1,
                    format!(
                        "A {} has `length`, `clone`, and what every object has -- and no \
                         other method.",
                        array.readable()
                    ),
                ))
            }
        };
        if !arguments.is_empty() {
            self.arguments_for(std::slice::from_ref(&object), arguments, line)?;
        }
        let index = self.pool.method(&owner, name, descriptor, false);
        self.op2(0xb6, index);
        let taken: i32 = read_descriptor(descriptor)
            .map(|(parameters, _)| parameters.iter().map(|one| i32::from(one.width())).sum())
            .unwrap_or(0);
        self.pops(taken + 1);
        if name == "clone" {
            self.pushes(&object);
            let class = self.pool.class(&owner);
            self.op2(0xc0, class);
            self.pops(1);
        }
        self.pushes(&returns);
        Ok(returns)
    }

    fn chain_to(
        &mut self,
        owner: &str,
        arguments: &[Expression],
        line: u32,
    ) -> Result<(), Diagnostic> {
        let belongs = (owner != self.this_class)
            .then(|| {
                self.classpath
                    .get(owner)
                    .and_then(|known| known.outer.clone())
            })
            .flatten();
        if let Some(enclosing) = belongs {
            let before = self.too_early;
            self.load(0, &Type::Object(self.this_class.clone(), Vec::new()));
            self.too_early = before;
            if !self.an_instance_of(&enclosing)? {
                return Err(at(
                    "EJ252",
                    line,
                    1,
                    format!(
                        "`{}` belongs to an instance of `{}`, and there is none here.",
                        owner.replace('/', "."),
                        enclosing.replace('/', ".")
                    ),
                ));
            }
            let Some(found) = self.constructor_taking(owner, arguments.len() + 1) else {
                return Err(at(
                    "EJ243",
                    line,
                    1,
                    format!(
                        "`{}` has no constructor taking {} argument(s) that this \
                         compilation knows.",
                        owner.replace('/', "."),
                        arguments.len()
                    ),
                ));
            };
            self.arguments_for(&found.parameters[1..], arguments, line)?;
            let index = self
                .pool
                .method(owner, "<init>", &found.descriptor(), false);
            self.op2(0xb7, index);
            let taken: i32 = found
                .parameters
                .iter()
                .map(|one| i32::from(one.width()))
                .sum();
            self.pops(taken + 1);
            return Ok(());
        }
        let chosen = if arguments.is_empty() {
            None
        } else {
            self.signature_for(owner, "<init>", arguments, line)?
        };
        let signature = if let Some(chosen) = chosen {
            chosen
        } else if owner == self.this_class {
            let own = self
                .unit
                .methods
                .iter()
                .find(|held| held.constructor && held.parameters.len() == arguments.len())
                .cloned();
            let Some(own) = own else {
                return Err(at(
                    "EJ243",
                    line,
                    1,
                    format!(
                        "This class has no constructor taking {} argument(s).",
                        arguments.len()
                    ),
                ));
            };
            let mut parameters = Vec::new();
            for (what, _) in &own.parameters {
                parameters.push(self.resolve(what, line)?);
            }
            Signature {
                owner: owner.to_string(),
                name: "<init>".to_string(),
                parameters,
                returns: Type::Void,
                static_: false,
                interface: false,
                variadic: false,
                abstract_: false,
                returns_variable: None,
                parameter_variables: Vec::new(),
                taken_stands_for: Vec::new(),
                returns_stands_for: Standing::Fixed,
                own_variables: Vec::new(),
                witnessed: Vec::new(),
                throws: Vec::new(),
            }
        } else {
            match self.find_signature(owner, "<init>", arguments.len()) {
                Some(found) => found,
                None if arguments.is_empty() => Signature {
                    owner: owner.to_string(),
                    name: "<init>".to_string(),
                    parameters: Vec::new(),
                    returns: Type::Void,
                    static_: false,
                    interface: false,
                    variadic: false,
                    abstract_: false,
                    returns_variable: None,
                    parameter_variables: Vec::new(),
                    taken_stands_for: Vec::new(),
                    returns_stands_for: Standing::Fixed,
                    own_variables: Vec::new(),
                    witnessed: Vec::new(),
                    throws: Vec::new(),
                },
                None => {
                    return Err(at(
                        "EJ243",
                        line,
                        1,
                        format!(
                            "`{}` has no constructor taking {} argument(s) that this \
                             compilation knows.",
                            owner.replace('/', "."),
                            arguments.len()
                        ),
                    )
                    .with_suggestion("Hand the class file that declares it over as a dependency."))
                }
            }
        };

        let before = self.too_early;
        self.load(0, &Type::Object(self.this_class.clone(), Vec::new()));
        self.too_early = before;
        self.arguments_for_signature(&signature, arguments, line)?;
        let descriptor = signature.descriptor();
        let index = self.pool.method(owner, "<init>", &descriptor, false);
        self.op2(0xb7, index);
        let taken: i32 = signature
            .parameters
            .iter()
            .map(|one| i32::from(one.width()))
            .sum();
        self.pops(taken + 1);
        Ok(())
    }

    fn for_each_over_an_iterator(
        &mut self,
        what: &Written,
        name: &str,
        over: &Expression,
        body: &Positioned<Statement>,
        found: &Type,
        line: u32,
    ) -> Result<(), Diagnostic> {
        let Type::Object(class, arguments) = found.clone() else {
            return Err(at(
                "EJ236",
                line,
                1,
                format!("A `for` cannot walk over a {}.", found.readable()),
            )
            .with_suggestion("An array works, and so does anything with an `iterator()`."));
        };
        let Some(iterator) = self.signature_for(&class, "iterator", &[], line)? else {
            return Err(at(
                "EJ236",
                line,
                1,
                format!(
                    "`{}` has no `iterator()` that this compilation knows.",
                    class.replace('/', ".")
                ),
            )
            .with_suggestion("Hand the class file that declares it over as a dependency."));
        };

        let object = Type::Object("java/lang/Object".to_string(), Vec::new());

        let holds = match arguments.first() {
            Some(found) if *found != object => Some(found.clone()),
            _ => self
                .classpath
                .seen_from(&class, &arguments, "java/lang/Iterable")
                .into_iter()
                .next()
                .filter(|found| *found != object),
        };
        let declared = match what {
            Written::Inferred => holds.clone().unwrap_or(object.clone()),
            other => self.resolve(other, line)?,
        };

        self.open();
        let over_type = self.value(over, line)?;
        let _ = over_type;
        let held = self.declare("$walking", iterator.returns.clone());
        self.call_signature(&iterator, line);
        self.store(held, &iterator.returns);

        let Type::Object(iterator_class, _) = iterator.returns.clone() else {
            return Err(at(
                "EJ236",
                line,
                1,
                "An `iterator()` has to hand over an object.",
            ));
        };
        let Some(has_next) = self.signature_for(&iterator_class, "hasNext", &[], line)? else {
            return Err(at("EJ236", line, 1, "An iterator has to have `hasNext()`."));
        };
        let Some(next) = self.signature_for(&iterator_class, "next", &[], line)? else {
            return Err(at("EJ236", line, 1, "An iterator has to have `next()`."));
        };

        let top = self.code.len();
        self.a_branch_lands_here();
        self.load(held, &iterator.returns);
        self.call_signature(&has_next, line);
        self.pops(1);
        let out = self.jump(0x99);

        self.open();
        self.load(held, &iterator.returns);
        self.call_signature(&next, line);

        let boxed = match boxed_name(&declared) {
            Some(named) => Type::Object(named.to_string(), Vec::new()),
            None => declared.clone(),
        };
        if boxed != object {
            let Type::Object(named, _) = &boxed else {
                return Err(at(
                    "EJ237",
                    line,
                    1,
                    format!(
                        "`{name}` is a {}, and an iterator hands over objects.",
                        declared.readable()
                    ),
                )
                .with_suggestion("Write the loop over a reference type, or over an array."));
            };
            let index = self.pool.class(named);
            self.op2(0xc0, index);
            self.pops(1);
            self.pushes(&boxed);
        }
        if boxed != declared {
            self.box_or_unbox(&boxed, &declared);
        }
        let slot = self.declare(name, declared.clone());
        self.store(slot, &declared);

        self.enter(true);
        self.statement(body)?;
        let level = self.leave();
        self.close();

        for pending in level.continues {
            self.land(pending);
        }
        self.a_branch_lands_here();
        self.jump_back(0xa7, top);
        self.land(out);
        for pending in level.breaks {
            self.land(pending);
        }
        self.close();
        self.a_branch_lands_here();
        Ok(())
    }

    fn call_signature(&mut self, signature: &Signature, line: u32) {
        let _ = line;
        let descriptor = signature.descriptor();
        let index = self.pool.method(
            &signature.owner,
            &signature.name,
            &descriptor,
            signature.interface,
        );
        let taken: i32 = signature
            .parameters
            .iter()
            .map(|one| i32::from(one.width()))
            .sum();
        if signature.interface {
            self.code.push(0xb9);
            self.code.extend_from_slice(&index.to_be_bytes());
            self.code.push((taken + 1) as u8);
            self.code.push(0);
        } else {
            self.op2(0xb6, index);
        }
        self.pops(taken + 1);
        self.pushes(&signature.returns);
    }

    fn outer_this(&mut self, of: &Written, line: u32) -> Result<Type, Diagnostic> {
        let wanted = self.resolve(of, line)?;
        let Type::Object(named, _) = wanted.clone() else {
            return Err(at("EJ252", line, 1, "`this` belongs to a class."));
        };
        if self.this_class == named {
            if self.static_ {
                return Err(at(
                    "EJ222",
                    line,
                    1,
                    "`this` has no meaning in a static method.",
                ));
            }
            self.load(0, &Type::Object(self.this_class.clone(), Vec::new()));
            return Ok(wanted);
        }
        let around = self.the_classes_around();
        if !around.contains(&named) {
            return Err(at(
                "EJ252",
                line,
                1,
                match around.first() {
                    Some(enclosing) => format!(
                        "This class belongs to an instance of `{}`, not of `{}`.",
                        enclosing.replace('/', "."),
                        named.replace('/', ".")
                    ),
                    None => format!(
                        "This class does not belong to an instance of `{}`.",
                        named.replace('/', ".")
                    ),
                },
            ));
        }
        self.reach_the_enclosing_instance(&named)?;
        Ok(wanted)
    }

    fn new_array(
        &mut self,
        of: &Written,
        lengths: &[Expression],
        empty: usize,
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let base = self.resolve(of, line)?;
        let mut whole = base;
        for _ in 0..lengths.len() + empty {
            whole = Type::Array(Box::new(whole));
        }

        if lengths.is_empty() {
            self.one_dimension(&whole, line)?;
            return Ok(whole);
        }
        self.allocate(&whole, lengths, line)
    }

    fn allocate(
        &mut self,
        whole: &Type,
        lengths: &[Expression],
        line: u32,
    ) -> Result<Type, Diagnostic> {
        if lengths.len() > 1 {
            return self.array_of_arrays(whole, lengths, line);
        }
        let found = self.value(&lengths[0], line)?;
        self.convert(&found, &Type::Int, line)?;
        self.one_dimension(whole, line)?;
        Ok(whole.clone())
    }

    fn one_dimension(&mut self, whole: &Type, line: u32) -> Result<(), Diagnostic> {
        let Type::Array(element) = whole.clone() else {
            return Err(at(
                "EJ121",
                line,
                1,
                format!("A {} is not an array.", whole.readable()),
            ));
        };
        match element.as_ref() {
            Type::Object(name, _) => {
                let index = self.pool.class(name);
                self.op2(0xbd, index);
            }
            held @ Type::Array(_) => {
                let index = self.pool.class(&held.descriptor());
                self.op2(0xbd, index);
            }
            primitive => {
                let code = match primitive {
                    Type::Boolean => 4u8,
                    Type::Char => 5,
                    Type::Float => 6,
                    Type::Double => 7,
                    Type::Byte => 8,
                    Type::Short => 9,
                    Type::Int => 10,
                    Type::Long => 11,
                    _ => return Err(at("EJ209", line, 1, "An array of void is not a thing.")),
                };
                self.op1(0xbc, code);
            }
        }

        self.pops(1);
        self.pushes(whole);
        Ok(())
    }

    fn array_of_arrays(
        &mut self,
        whole: &Type,
        lengths: &[Expression],
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let Type::Array(inner) = whole.clone() else {
            unreachable!("an array of arrays is an array")
        };

        self.open();

        let counted = self.value(&lengths[0], line)?;
        self.convert(&counted, &Type::Int, line)?;
        let how_many = self.declare("$many", Type::Int);
        self.store(how_many, &Type::Int);

        self.load(how_many, &Type::Int);
        self.one_dimension(whole, line)?;
        let made = self.declare("$made", whole.clone());
        self.store(made, whole);

        self.op(0x03);
        self.pushes(&Type::Int);
        let at = self.declare("$at", Type::Int);
        self.store(at, &Type::Int);

        let top = self.code.len();
        self.a_branch_lands_here();
        self.load(at, &Type::Int);
        self.load(how_many, &Type::Int);
        self.pops(2);
        let out = self.jump(0xa2);

        self.load(made, whole);
        self.load(at, &Type::Int);

        self.allocate(inner.as_ref(), &lengths[1..], line)?;
        self.op(array_store(&inner));
        self.pops(3);

        self.bump_local(at, 1);
        self.jump_back(0xa7, top);
        self.land(out);
        self.a_branch_lands_here();

        self.load(made, whole);
        self.close();
        Ok(whole.clone())
    }

    fn array_of(
        &mut self,
        whole: &Type,
        values: &[Expression],
        line: u32,
    ) -> Result<Type, Diagnostic> {
        let Type::Array(element) = whole.clone() else {
            return Err(at(
                "EJ121",
                line,
                1,
                format!("A {} is not an array.", whole.readable()),
            ));
        };

        let of = written_for(&element).ok_or_else(|| {
            at(
                "EJ121",
                line,
                1,
                "This array holds something that cannot be named.",
            )
        })?;
        self.new_array(&of, &[Expression::Int(values.len() as i64)], 0, line)?;

        let store = array_store(&element);
        for (position, value) in values.iter().enumerate() {
            self.duplicates();
            self.push_int(position as i64);

            let found = match value {
                Expression::ArrayOf {
                    of: None,
                    values,
                    line: written,
                } => self.array_of(&element, values, *written)?,
                other => self.value_for(other, &element, line)?,
            };

            if !self.fit(&found, &element, line)? {
                return Err(at(
                    "EJ121",
                    line,
                    1,
                    format!(
                        "This array holds {} and was given a {}.",
                        element.readable(),
                        found.readable()
                    ),
                ));
            }
            self.op(store);
            self.pops(2 + i32::from(element.width()));
        }
        Ok(whole.clone())
    }

    fn bump_local(&mut self, slot: u16, by: i8) {
        if slot <= 255 {
            self.op(0x84);
            self.code.push(slot as u8);
            self.code.push(by as u8);
            return;
        }
        self.op(0xc4);
        self.op(0x84);
        self.code.extend_from_slice(&slot.to_be_bytes());
        self.code.extend_from_slice(&i16::from(by).to_be_bytes());
    }
}

#[derive(Clone)]
enum Chooser {
    Integer,
    Text,

    Constant(String),

    Shape,

    Primitive(Type),
}

enum Dispatch {
    Instruction {
        opcode_at: usize,
        default_slot: usize,

        slots: Vec<(usize, usize)>,
    },

    Chain {
        waiting: Vec<(Pending, usize)>,
        fallthrough: Pending,
    },
}

impl Dispatch {
    fn settle(self, emitter: &mut Emitter<'_>, targets: &[usize], default_at: usize) {
        match self {
            Dispatch::Instruction {
                opcode_at,
                default_slot,
                slots,
            } => {
                let offset = |to: usize| (to as i64 - opcode_at as i64) as i32;
                emitter.code[default_slot..default_slot + 4]
                    .copy_from_slice(&offset(default_at).to_be_bytes());
                for (slot, arm) in slots {
                    let to = match targets.get(arm) {
                        Some(found) => *found,
                        None => default_at,
                    };
                    emitter.code[slot..slot + 4].copy_from_slice(&offset(to).to_be_bytes());
                }
            }
            Dispatch::Chain {
                waiting,
                fallthrough,
            } => {
                for (pending, arm) in waiting {
                    let to = targets.get(arm).copied().unwrap_or(default_at);
                    emitter.land_at(pending, to);
                }
                emitter.land_at(fallthrough, default_at);
            }
        }
    }
}

fn held_as(what: &Written, value: Expression) -> Option<Expression> {
    let whole = match &value {
        Expression::Int(held) => Some(*held),
        Expression::Char(held) => Some(i64::from(*held)),
        _ => None,
    };
    Some(match (what, whole) {
        (Written::Char, Some(held)) => Expression::Char(held as u16),
        (Written::Byte, Some(held)) => Expression::Int(i64::from(held as i8)),
        (Written::Short, Some(held)) => Expression::Int(i64::from(held as i16)),
        (Written::Int, Some(held)) => Expression::Int(held),
        (Written::Long, Some(held)) => Expression::Long(held),
        (Written::Long, None) if matches!(value, Expression::Long(_)) => value,
        (Written::Float, Some(held)) => Expression::Float(held as f64),
        (Written::Float, None) => match value {
            Expression::Long(held) => Expression::Float(held as f64),
            Expression::Float(_) => value,
            _ => return None,
        },
        (Written::Double, Some(held)) => Expression::Double(held as f64),
        (Written::Double, None) => match value {
            Expression::Long(held) => Expression::Double(held as f64),
            Expression::Float(held) => Expression::Double(f64::from(held as f32)),
            Expression::Double(_) => value,
            _ => return None,
        },
        (Written::Boolean, None) if matches!(value, Expression::Boolean(_)) => value,
        (Written::Named(named, _), None)
            if matches!(value, Expression::Str(_))
                && (named == "String" || named == "java.lang.String") =>
        {
            value
        }
        (Written::Inferred, _) => value,
        _ => return None,
    })
}

fn enum_label(label: &Expression) -> Option<&str> {
    match label {
        Expression::Name(name) | Expression::Field { name, .. } => Some(name),
        _ => None,
    }
}

fn dotted_name(expression: &Expression) -> Option<String> {
    Some(match expression {
        Expression::Name(name) => name.clone(),
        Expression::Field { of, name } => format!("{}.{name}", dotted_name(of)?),
        _ => return None,
    })
}

fn constant_within(
    classpath: &Classpath,
    unit: &Unit,
    expression: &Expression,
    depth: u8,
) -> Option<Expression> {
    if depth > 24 {
        return None;
    }
    let own = |name: &str| -> Option<Option<Expression>> {
        let field = unit.fields.iter().find(|field| field.name == name)?;
        if !field.modifiers.static_ || !field.modifiers.final_ {
            return Some(None);
        }
        Some(
            field
                .value
                .as_ref()
                .and_then(|value| constant_within(classpath, unit, value, depth + 1))
                .and_then(|found| held_as(&field.what, found)),
        )
    };
    fold_constant(expression, &mut |named| match named {
        Expression::Name(name) => {
            if let Some(found) = own(name) {
                return found;
            }
            let mut owners: Vec<String> = Vec::new();
            owners.extend(
                unit.extends
                    .iter()
                    .filter_map(|one| resolve_named(classpath, unit, one)),
            );
            owners.extend(
                unit.implements
                    .iter()
                    .filter_map(|one| resolve_named(classpath, unit, one)),
            );
            let mut outer = unit.outer.clone();
            while let Some(around) = outer {
                if owners.len() > 64 || owners.contains(&around) {
                    break;
                }
                outer = classpath.get(&around).and_then(|known| known.outer.clone());
                owners.push(around);
            }
            for held in &unit.static_imports {
                let Some((owner, last)) = held.rsplit_once('.') else {
                    continue;
                };
                if last == "*" || last == name {
                    owners.extend(resolve_named(classpath, unit, owner));
                }
            }
            owners
                .iter()
                .find_map(|owner| classpath.constant(owner, name).cloned())
        }
        Expression::Field { of, name } => {
            let owner = resolve_named(classpath, unit, &dotted_name(of)?)?;
            if owner == unit.internal_name() {
                if let Some(found) = own(name) {
                    return found;
                }
            }
            classpath.constant(&owner, name).cloned()
        }
        _ => None,
    })
}

fn fold_constant(
    expression: &Expression,
    look: &mut dyn FnMut(&Expression) -> Option<Expression>,
) -> Option<Expression> {
    fn whole(expression: &Expression) -> Option<i64> {
        match expression {
            Expression::Int(value) => Some(*value),
            Expression::Char(value) => Some(i64::from(*value)),
            _ => None,
        }
    }
    fn wide(expression: &Expression) -> Option<i64> {
        match expression {
            Expression::Long(value) => Some(*value),
            other => whole(other),
        }
    }
    fn spoken(expression: &Expression) -> Option<String> {
        Some(match expression {
            Expression::Str(text) => text.clone(),
            Expression::Int(value) | Expression::Long(value) => value.to_string(),
            Expression::Char(value) => char::from_u32(u32::from(*value))?.to_string(),
            Expression::Boolean(value) => value.to_string(),
            _ => return None,
        })
    }
    fn narrow(value: i64) -> Expression {
        Expression::Int(i64::from(value as i32))
    }
    Some(match expression {
        Expression::Int(_)
        | Expression::Long(_)
        | Expression::Float(_)
        | Expression::Double(_)
        | Expression::Char(_)
        | Expression::Str(_)
        | Expression::Boolean(_) => expression.clone(),
        Expression::Name(_) | Expression::Field { .. } => look(expression)?,
        Expression::Cast { to, of, .. } => {
            let inner = fold_constant(of, look)?;
            match to {
                Written::Named(named, _) if named == "String" || named == "java.lang.String" => {
                    let Expression::Str(_) = inner else {
                        return None;
                    };
                    inner
                }
                Written::Boolean => {
                    let Expression::Boolean(_) = inner else {
                        return None;
                    };
                    inner
                }
                Written::Long => Expression::Long(wide(&inner)?),
                Written::Int => narrow(wide(&inner)?),
                Written::Char => Expression::Char(wide(&inner)? as u16),
                Written::Short => Expression::Int(i64::from(wide(&inner)? as i16)),
                Written::Byte => Expression::Int(i64::from(wide(&inner)? as i8)),
                _ => return None,
            }
        }
        Expression::Unary { operator, of } => {
            let inner = fold_constant(of, look)?;
            match (operator, inner) {
                (Unary::Not, Expression::Boolean(value)) => Expression::Boolean(!value),
                (Unary::Not, _) => return None,
                (Unary::Negate, Expression::Long(value)) => Expression::Long(value.wrapping_neg()),
                (Unary::Complement, Expression::Long(value)) => Expression::Long(!value),
                (Unary::Plus, Expression::Long(value)) => Expression::Long(value),
                (Unary::Negate, other) => narrow(whole(&other)?.wrapping_neg()),
                (Unary::Complement, other) => narrow(!whole(&other)?),
                (Unary::Plus, other) => narrow(whole(&other)?),
            }
        }
        Expression::Conditional {
            condition,
            then,
            otherwise,
        } => {
            let Expression::Boolean(chosen) = fold_constant(condition, look)? else {
                return None;
            };
            let then = fold_constant(then, look)?;
            let otherwise = fold_constant(otherwise, look)?;
            if chosen {
                then
            } else {
                otherwise
            }
        }
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            let left = fold_constant(left, look)?;
            let right = fold_constant(right, look)?;
            if let (Expression::Boolean(one), Expression::Boolean(two)) = (&left, &right) {
                let (one, two) = (*one, *two);
                return Some(Expression::Boolean(match operator {
                    Binary::AndAlso | Binary::And => one && two,
                    Binary::OrElse | Binary::Or => one || two,
                    Binary::Xor | Binary::NotEqual => one != two,
                    Binary::Equal => one == two,
                    _ => return None,
                }));
            }
            let text = matches!(left, Expression::Str(_)) || matches!(right, Expression::Str(_));
            if text {
                if *operator != Binary::Add {
                    return None;
                }
                return Some(Expression::Str(spoken(&left)? + &spoken(&right)?));
            }
            let long = matches!(left, Expression::Long(_)) || matches!(right, Expression::Long(_));
            let (one, two) = (wide(&left)?, wide(&right)?);
            let shifting = matches!(
                operator,
                Binary::ShiftLeft | Binary::ShiftRight | Binary::UnsignedShiftRight
            );
            if shifting {
                let long = matches!(left, Expression::Long(_));
                let by = if long { two & 63 } else { two & 31 } as u32;
                return Some(if long {
                    Expression::Long(match operator {
                        Binary::ShiftLeft => one.wrapping_shl(by),
                        Binary::ShiftRight => one.wrapping_shr(by),
                        _ => ((one as u64) >> by) as i64,
                    })
                } else {
                    let one = one as i32;
                    narrow(i64::from(match operator {
                        Binary::ShiftLeft => one.wrapping_shl(by),
                        Binary::ShiftRight => one.wrapping_shr(by),
                        _ => ((one as u32) >> by) as i32,
                    }))
                });
            }
            let compared = match operator {
                Binary::Less => Some(one < two),
                Binary::LessOrEqual => Some(one <= two),
                Binary::Greater => Some(one > two),
                Binary::GreaterOrEqual => Some(one >= two),
                Binary::Equal => Some(one == two),
                Binary::NotEqual => Some(one != two),
                _ => None,
            };
            if let Some(answer) = compared {
                return Some(Expression::Boolean(answer));
            }
            if long {
                Expression::Long(match operator {
                    Binary::Add => one.wrapping_add(two),
                    Binary::Subtract => one.wrapping_sub(two),
                    Binary::Multiply => one.wrapping_mul(two),
                    Binary::Divide => {
                        if two == 0 {
                            return None;
                        }
                        one.wrapping_div(two)
                    }
                    Binary::Remainder => {
                        if two == 0 {
                            return None;
                        }
                        one.wrapping_rem(two)
                    }
                    Binary::And => one & two,
                    Binary::Or => one | two,
                    Binary::Xor => one ^ two,
                    _ => return None,
                })
            } else {
                let (one, two) = (one as i32, two as i32);
                narrow(i64::from(match operator {
                    Binary::Add => one.wrapping_add(two),
                    Binary::Subtract => one.wrapping_sub(two),
                    Binary::Multiply => one.wrapping_mul(two),
                    Binary::Divide => {
                        if two == 0 {
                            return None;
                        }
                        one.wrapping_div(two)
                    }
                    Binary::Remainder => {
                        if two == 0 {
                            return None;
                        }
                        one.wrapping_rem(two)
                    }
                    Binary::And => one & two,
                    Binary::Or => one | two,
                    Binary::Xor => one ^ two,
                    _ => return None,
                }))
            }
        }
        _ => return None,
    })
}

fn text_type() -> Type {
    Type::Object("java/lang/String".to_string(), Vec::new())
}

fn constant_int(expression: &Expression) -> Option<i32> {
    match expression {
        Expression::Int(value) => i32::try_from(*value).ok(),
        Expression::Char(value) => Some(i32::from(*value)),
        Expression::Boolean(value) => Some(i32::from(*value)),
        Expression::Unary { operator, of } => match operator {
            Unary::Negate => constant_int(of)?.checked_neg(),
            Unary::Plus => constant_int(of),
            Unary::Complement => Some(!constant_int(of)?),
            Unary::Not => None,
        },
        _ => None,
    }
}

fn as_a_written_type(expression: &Expression) -> Option<Written> {
    fn dotted(expression: &Expression) -> Option<String> {
        Some(match expression {
            Expression::Name(name) => name.clone(),
            Expression::Field { of, name } => format!("{}.{name}", dotted(of)?),
            _ => return None,
        })
    }
    Some(Written::Named(dotted(expression)?, Vec::new()))
}

fn kept_at_runtime(unit: &Unit) -> bool {
    if !unit.annotation {
        return false;
    }
    unit.annotations.iter().any(|held| {
        held.name.rsplit('.').next() == Some("Retention")
            && held.values.iter().any(|(_, value)| {
                written_as_a_path(value)
                    .is_some_and(|path| path.rsplit('.').next() == Some("RUNTIME"))
            })
    })
}

const KEPT_AT_RUNTIME: &[&str] = &[
    "java/lang/Deprecated",
    "java/lang/FunctionalInterface",
    "java/lang/SafeVarargs",
    "java/lang/annotation/Documented",
    "java/lang/annotation/Inherited",
    "java/lang/annotation/Repeatable",
    "java/lang/annotation/Retention",
    "java/lang/annotation/Target",
];

fn with_what_the_arguments_say(
    signature: &Signature,
    held: &[Type],
    given: &[Type],
    going: Option<&Type>,
) -> Signature {
    if signature
        .taken_stands_for
        .iter()
        .all(Standing::says_nothing)
    {
        return signature.clone();
    }
    let bound = what_the_call_says(signature, given, going);
    if bound.is_empty() {
        return signature.clone();
    }
    let mut shaped = signature.clone();
    for (at, one) in shaped.parameters.iter_mut().enumerate() {
        let Some(shape) = signature.taken_stands_for.get(at) else {
            continue;
        };
        *one = shape.filled(one, held, &bound);
    }
    shaped
}

fn what_the_call_says(
    signature: &Signature,
    given: &[Type],
    going: Option<&Type>,
) -> Vec<(String, Type)> {
    let mut bound: Vec<(String, Type)> = Vec::new();
    let fixed = signature.taken_stands_for.len().saturating_sub(1);
    for (shape, actual) in signature.taken_stands_for.iter().zip(given.iter()) {
        shape.bind(actual, &mut bound);
    }

    if signature.variadic && given.len() > fixed {
        if let Some(Standing::ArrayOf(inside)) = signature.taken_stands_for.last() {
            let trailing = &given[fixed..];
            let handed_the_array = trailing.len() == 1 && matches!(trailing[0], Type::Array(_));
            if !handed_the_array {
                for actual in trailing {
                    inside.bind(actual, &mut bound);
                }
            }
        }
    }

    if let Some(going) = going {
        let mut theirs: Vec<(String, Type)> = Vec::new();
        signature.returns_stands_for.bind(going, &mut theirs);
        for (name, what) in theirs {
            if !bound.iter().any(|(held, _)| *held == name) {
                bound.push((name, what));
            }
        }
    }

    bound.retain(|(_, what)| *what != Type::Object("java/lang/Object".to_string(), Vec::new()));

    for (name, what) in &signature.witnessed {
        match bound.iter_mut().find(|(held, _)| held == name) {
            Some((_, before)) => *before = what.clone(),
            None => bound.push((name.clone(), what.clone())),
        }
    }
    bound
}

fn as_written(signature: &Signature, held: &[Type]) -> Signature {
    if !held.is_empty()
        && (signature
            .taken_stands_for
            .iter()
            .any(Standing::mentions_the_class)
            || signature.returns_stands_for.mentions_the_class())
    {
        let mut shaped = signature.clone();
        for (at, one) in shaped.parameters.iter_mut().enumerate() {
            let Some(shape) = signature.taken_stands_for.get(at) else {
                continue;
            };
            if !shape.mentions_the_class() {
                continue;
            }
            *one = shape.filled(one, held, &[]);
        }

        if signature.returns_stands_for.mentions_the_class() {
            shaped.returns = signature
                .returns_stands_for
                .filled(&signature.returns, held, &[]);
        }
        return shaped;
    }

    let mine: Vec<usize>;
    let which: &[usize] = if signature
        .parameter_variables
        .iter()
        .any(|one| one.is_some())
    {
        mine = signature
            .parameter_variables
            .iter()
            .map(|one| one.unwrap_or(usize::MAX))
            .collect();
        &mine
    } else {
        match ELEMENT_PARAMETERS
            .iter()
            .find(|(class, name, _)| *class == signature.owner && *name == signature.name)
        {
            Some((_, _, found)) => found,
            None => return signature.clone(),
        }
    };
    let mut shaped = signature.clone();
    for (at, one) in shaped.parameters.iter_mut().enumerate() {
        let Some(argument) = which.get(at).and_then(|index| held.get(*index)) else {
            continue;
        };

        if *one == Type::Object("java/lang/Object".to_string(), Vec::new()) {
            *one = argument.clone();
        }
    }
    shaped
}

const ELEMENT_PARAMETERS: &[(&str, &str, &[usize])] = &[
    ("java/util/Collection", "add", &[0]),
    ("java/util/Collection", "remove", &[0]),
    ("java/util/Collection", "contains", &[0]),
    ("java/util/List", "add", &[usize::MAX, 0]),
    ("java/util/List", "set", &[usize::MAX, 0]),
    ("java/util/List", "indexOf", &[0]),
    ("java/util/List", "lastIndexOf", &[0]),
    ("java/util/Map", "get", &[0]),
    ("java/util/Map", "containsKey", &[0]),
    ("java/util/Map", "containsValue", &[1]),
    ("java/util/Map", "remove", &[0]),
    ("java/util/Map", "put", &[0, 1]),
    ("java/util/Map", "getOrDefault", &[0, 1]),
    ("java/util/Map", "putIfAbsent", &[0, 1]),
];

const ELEMENT_RETURN: &[(&str, &str, usize, Option<&str>)] = &[
    ("java/util/List", "get", 0, None),
    ("java/util/List", "set", 0, None),
    ("java/util/List", "remove", 0, None),
    ("java/util/Iterator", "next", 0, None),
    ("java/util/Map", "get", 1, None),
    ("java/util/Map", "put", 1, None),
    ("java/util/Map", "remove", 1, None),
    ("java/util/Map", "getOrDefault", 1, None),
    ("java/util/Map", "putIfAbsent", 1, None),
    ("java/util/Map", "keySet", 0, Some("java/util/Set")),
    ("java/util/Map", "values", 1, Some("java/util/Collection")),
    ("java/util/Map$Entry", "getKey", 0, None),
    ("java/util/Map$Entry", "getValue", 1, None),
    ("java/util/Map$Entry", "setValue", 1, None),
    (
        "java/util/Collection",
        "iterator",
        0,
        Some("java/util/Iterator"),
    ),
    (
        "java/lang/Iterable",
        "iterator",
        0,
        Some("java/util/Iterator"),
    ),
];

const ELEMENT_ENTRIES: &[(&str, &str, &str, &str)] = &[(
    "java/util/Map",
    "entrySet",
    "java/util/Set",
    "java/util/Map$Entry",
)];

fn primitive_inside(what: &Type) -> Option<Type> {
    let Type::Object(named, _) = what else {
        return None;
    };
    Some(match named.as_str() {
        "java/lang/Boolean" => Type::Boolean,
        "java/lang/Byte" => Type::Byte,
        "java/lang/Short" => Type::Short,
        "java/lang/Character" => Type::Char,
        "java/lang/Integer" => Type::Int,
        "java/lang/Long" => Type::Long,
        "java/lang/Float" => Type::Float,
        "java/lang/Double" => Type::Double,
        _ => return None,
    })
}

fn exactness(from: &Type, to: &Type, name: &str) -> Option<Option<Expression>> {
    use Type::{Boolean, Byte, Char, Double, Float, Int, Long, Short};
    if from == to {
        return Some(None);
    }
    let held = || Expression::Name(name.to_string());
    let cast = |to: Written, of: Expression| Expression::Cast {
        to,
        of: Box::new(of),
        also: Vec::new(),
    };
    let both = |operator: Binary, left: Expression, right: Expression| Expression::Binary {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    };
    let raw_bits = |owner: &str, method: &str| Expression::Call {
        on: Some(Box::new(Expression::Field {
            of: Box::new(Expression::Field {
                of: Box::new(Expression::Name("java".to_string())),
                name: "lang".to_string(),
            }),
            name: owner.to_string(),
        })),
        super_call: false,
        name: method.to_string(),
        arguments: vec![Expression::Name(name.to_string())],
        held: Vec::new(),
    };
    let narrow = |to: &Type| written_for(to).unwrap_or(Written::Int);
    match (from, to) {
        (Boolean, _) | (_, Boolean) => return None,
        (Byte, Short | Int | Long | Float | Double)
        | (Short, Int | Long | Float | Double)
        | (Char, Int | Long | Float | Double)
        | (Int, Long | Double)
        | (Float, Double) => return Some(None),
        _ => {}
    }
    let to_int_from_float = || {
        both(
            Binary::AndAlso,
            both(
                Binary::AndAlso,
                both(
                    Binary::Equal,
                    cast(Written::Float, cast(Written::Int, held())),
                    held(),
                ),
                both(
                    Binary::NotEqual,
                    cast(Written::Int, held()),
                    Expression::Int(i64::from(i32::MAX)),
                ),
            ),
            both(
                Binary::NotEqual,
                raw_bits("Float", "floatToRawIntBits"),
                Expression::Int(i64::from(i32::MIN)),
            ),
        )
    };
    let to_int_from_double = || {
        both(
            Binary::AndAlso,
            both(
                Binary::Equal,
                cast(Written::Double, cast(Written::Int, held())),
                held(),
            ),
            both(
                Binary::NotEqual,
                raw_bits("Double", "doubleToRawLongBits"),
                Expression::Long(i64::MIN),
            ),
        )
    };
    let test = match from {
        Byte | Short | Char | Int => match to {
            Byte | Short | Char => both(Binary::Equal, cast(narrow(to), held()), held()),
            Float => both(
                Binary::AndAlso,
                both(
                    Binary::Equal,
                    held(),
                    cast(Written::Int, cast(Written::Float, held())),
                ),
                both(
                    Binary::NotEqual,
                    held(),
                    Expression::Int(i64::from(i32::MAX)),
                ),
            ),
            _ => return None,
        },
        Long => match to {
            Byte | Short | Char | Int => both(
                Binary::Equal,
                cast(Written::Long, cast(narrow(to), cast(Written::Int, held()))),
                held(),
            ),
            Float | Double => both(
                Binary::AndAlso,
                both(
                    Binary::Equal,
                    held(),
                    cast(Written::Long, cast(narrow(to), held())),
                ),
                both(Binary::NotEqual, held(), Expression::Long(i64::MAX)),
            ),
            _ => return None,
        },
        Float => match to {
            Int => to_int_from_float(),
            Long => both(
                Binary::AndAlso,
                both(
                    Binary::AndAlso,
                    both(
                        Binary::Equal,
                        cast(Written::Float, cast(Written::Long, held())),
                        held(),
                    ),
                    both(
                        Binary::NotEqual,
                        cast(Written::Long, held()),
                        Expression::Long(i64::MAX),
                    ),
                ),
                both(
                    Binary::NotEqual,
                    raw_bits("Float", "floatToRawIntBits"),
                    Expression::Int(i64::from(i32::MIN)),
                ),
            ),
            Byte | Short | Char => both(
                Binary::AndAlso,
                to_int_from_float(),
                both(
                    Binary::Equal,
                    cast(narrow(to), cast(Written::Int, held())),
                    cast(Written::Int, held()),
                ),
            ),
            _ => return None,
        },
        Double => match to {
            Int => to_int_from_double(),
            Long => both(
                Binary::AndAlso,
                both(
                    Binary::AndAlso,
                    both(
                        Binary::Equal,
                        cast(Written::Double, cast(Written::Long, held())),
                        held(),
                    ),
                    both(
                        Binary::NotEqual,
                        cast(Written::Long, held()),
                        Expression::Long(i64::MAX),
                    ),
                ),
                both(
                    Binary::NotEqual,
                    raw_bits("Double", "doubleToRawLongBits"),
                    Expression::Long(i64::MIN),
                ),
            ),
            Float => both(
                Binary::OrElse,
                both(
                    Binary::Equal,
                    cast(Written::Double, cast(Written::Float, held())),
                    held(),
                ),
                both(Binary::NotEqual, held(), held()),
            ),
            Byte | Short | Char => both(
                Binary::AndAlso,
                to_int_from_double(),
                both(
                    Binary::Equal,
                    cast(narrow(to), cast(Written::Int, held())),
                    cast(Written::Int, held()),
                ),
            ),
            _ => return None,
        },
        _ => return None,
    };
    Some(Some(test))
}

fn boxed_name(what: &Type) -> Option<&'static str> {
    Some(match what {
        Type::Boolean => "java/lang/Boolean",
        Type::Byte => "java/lang/Byte",
        Type::Short => "java/lang/Short",
        Type::Char => "java/lang/Character",
        Type::Int => "java/lang/Integer",
        Type::Long => "java/lang/Long",
        Type::Float => "java/lang/Float",
        Type::Double => "java/lang/Double",
        Type::Void => "java/lang/Void",
        _ => return None,
    })
}

fn reached_the_same_way_twice(expression: &Expression) -> bool {
    match expression {
        Expression::This | Expression::Name(_) | Expression::Int(_) => true,
        Expression::Field { of, .. } => reached_the_same_way_twice(of),
        _ => false,
    }
}

#[derive(Clone, PartialEq, Eq)]
enum Loosely {
    No,

    Anywhere(Vec<usize>),
    Shaped(usize, Gives),
    AnyReference,
}

fn locals_declared(statement: &Statement, out: &mut Vec<(Written, String, Option<Expression>)>) {
    let all = |held: &[Positioned<Statement>],
               out: &mut Vec<(Written, String, Option<Expression>)>| {
        for one in held {
            locals_declared(&one.node, out);
        }
    };
    match statement {
        Statement::Declare { what, name, value } => {
            out.push((what.clone(), name.clone(), value.clone()));
        }
        Statement::Block(held) | Statement::Several(held) => all(held, out),
        Statement::If {
            then, otherwise, ..
        } => {
            locals_declared(&then.node, out);
            if let Some(otherwise) = otherwise {
                locals_declared(&otherwise.node, out);
            }
        }
        Statement::ForEach {
            what, name, body, ..
        } => {
            if !matches!(what, Written::Inferred) {
                out.push((what.clone(), name.clone(), None));
            }
            locals_declared(&body.node, out);
        }
        Statement::For { start, body, .. } => {
            all(start, out);
            locals_declared(&body.node, out);
        }
        Statement::While { body, .. }
        | Statement::DoWhile { body, .. }
        | Statement::Labelled { body, .. } => locals_declared(&body.node, out),
        Statement::Switch { arms, .. } => {
            for arm in arms {
                all(&arm.body, out);
            }
        }
        Statement::Try {
            body,
            catches,
            finally,
        } => {
            all(body, out);
            for catch in catches {
                all(&catch.body, out);
            }
            all(finally.as_deref().unwrap_or_default(), out);
        }
        Statement::Synchronized { body, .. } => all(body, out),
        _ => {}
    }
}

fn values_returned(statement: &Statement, out: &mut Vec<Expression>) {
    let all = |held: &[Positioned<Statement>], out: &mut Vec<Expression>| {
        for one in held {
            values_returned(&one.node, out);
        }
    };
    match statement {
        Statement::Return(Some(value)) => out.push(value.clone()),
        Statement::Block(held) | Statement::Several(held) => all(held, out),
        Statement::If {
            then, otherwise, ..
        } => {
            values_returned(&then.node, out);
            if let Some(otherwise) = otherwise {
                values_returned(&otherwise.node, out);
            }
        }
        Statement::While { body, .. }
        | Statement::DoWhile { body, .. }
        | Statement::For { body, .. }
        | Statement::ForEach { body, .. }
        | Statement::Labelled { body, .. } => values_returned(&body.node, out),
        Statement::Switch { arms, .. } => {
            for arm in arms {
                all(&arm.body, out);
            }
        }
        Statement::Try {
            body,
            catches,
            finally,
        } => {
            all(body, out);
            for catch in catches {
                all(&catch.body, out);
            }
            all(finally.as_deref().unwrap_or_default(), out);
        }
        Statement::Synchronized { body, .. } => all(body, out),
        _ => {}
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Gives {
    Nothing,
    Something,
    Either,
}

fn what_a_lambda_gives(body: &[Positioned<Statement>], expression: bool) -> Gives {
    fn returns_in(statement: &Statement, bare: &mut bool, valued: &mut bool) {
        let all = |held: &[Positioned<Statement>], bare: &mut bool, valued: &mut bool| {
            for one in held {
                returns_in(&one.node, bare, valued);
            }
        };
        match statement {
            Statement::Return(None) => *bare = true,
            Statement::Return(Some(_)) => *valued = true,
            Statement::Block(held) | Statement::Several(held) => all(held, bare, valued),
            Statement::If {
                then, otherwise, ..
            } => {
                returns_in(&then.node, bare, valued);
                if let Some(otherwise) = otherwise {
                    returns_in(&otherwise.node, bare, valued);
                }
            }
            Statement::While { body, .. }
            | Statement::DoWhile { body, .. }
            | Statement::For { body, .. }
            | Statement::ForEach { body, .. }
            | Statement::Labelled { body, .. } => returns_in(&body.node, bare, valued),
            Statement::Switch { arms, .. } => {
                for arm in arms {
                    all(&arm.body, bare, valued);
                }
            }
            Statement::Try {
                body,
                catches,
                finally,
            } => {
                all(body, bare, valued);
                for catch in catches {
                    all(&catch.body, bare, valued);
                }
                all(finally.as_deref().unwrap_or_default(), bare, valued);
            }
            Statement::Synchronized { body, .. } => all(body, bare, valued),
            _ => {}
        }
    }
    if expression {
        return match body.first().map(|one| &one.node) {
            Some(Statement::Express(
                Expression::Call { .. }
                | Expression::Assign { .. }
                | Expression::Step { .. }
                | Expression::New { .. }
                | Expression::Anonymous { .. },
            )) => Gives::Either,
            Some(Statement::Express(_)) => Gives::Something,
            _ => Gives::Either,
        };
    }
    let (mut bare, mut valued) = (false, false);
    for one in body {
        returns_in(&one.node, &mut bare, &mut valued);
    }
    match (bare, valued) {
        (true, _) => Gives::Nothing,
        (false, true) => Gives::Something,
        (false, false) if body.last().is_some_and(|one| never_completes(&one.node)) => {
            Gives::Either
        }
        (false, false) => Gives::Nothing,
    }
}

fn how_loosely(arguments: &[Expression]) -> Vec<Loosely> {
    arguments
        .iter()
        .map(|one| match one {
            Expression::Lambda {
                parameters,
                body,
                expression,
                ..
            } => Loosely::Shaped(parameters.len(), what_a_lambda_gives(body, *expression)),

            Expression::MethodRef { .. } => Loosely::Anywhere(Vec::new()),
            Expression::Null => Loosely::AnyReference,
            _ => Loosely::No,
        })
        .collect()
}

fn how_far(have: &Type, want: &Type) -> u32 {
    if have == want {
        return 0;
    }
    if let (Some(from), Some(to)) = (how_wide(have), how_wide(want)) {
        return to.saturating_sub(from);
    }
    match want {
        Type::Object(named, _) if named == "java/lang/Object" => 50,
        _ => 10,
    }
}

fn how_wide(what: &Type) -> Option<u32> {
    Some(match what {
        Type::Byte => 1,
        Type::Short | Type::Char => 2,
        Type::Int => 3,
        Type::Long => 4,
        Type::Float => 5,
        Type::Double => 6,
        _ => return None,
    })
}

fn array_store(element: &Type) -> u8 {
    match element {
        Type::Long => 0x50,
        Type::Float => 0x51,
        Type::Double => 0x52,
        Type::Byte | Type::Boolean => 0x54,
        Type::Char => 0x55,
        Type::Short => 0x56,
        other if other.is_reference() => 0x53,
        _ => 0x4f,
    }
}

fn array_load(element: &Type) -> u8 {
    match element {
        Type::Long => 0x2f,
        Type::Float => 0x30,
        Type::Double => 0x31,
        Type::Byte | Type::Boolean => 0x33,
        Type::Char => 0x34,
        Type::Short => 0x35,
        other if other.is_reference() => 0x32,
        _ => 0x2e,
    }
}

fn folded_constant(field: &Field, what: &Type, pool: &mut Pool) -> Option<u16> {
    if !field.modifiers.static_ || !field.modifiers.final_ {
        return None;
    }
    match (field.value.as_ref()?, what) {
        (Expression::Int(value), one) if one.is_int_like() => {
            Some(pool.integer(i32::try_from(*value).ok()?))
        }
        (Expression::Char(value), Type::Char) => Some(pool.integer(i32::from(*value))),
        (Expression::Boolean(value), Type::Boolean) => Some(pool.integer(i32::from(*value))),
        (Expression::Long(value), Type::Long) => Some(pool.long(*value)),
        (Expression::Float(value), Type::Float) => Some(pool.float(*value as f32)),
        (Expression::Double(value), Type::Double) => Some(pool.double(*value)),
        (Expression::Str(text), Type::Object(named, _)) if named == "java/lang/String" => {
            Some(pool.string(text))
        }
        _ => None,
    }
}

fn says_more_than_the_descriptor(what: &Written) -> bool {
    match what {
        Written::Variable(_, _) | Written::OwnVariable(_, _) => true,
        Written::Array(of) => says_more_than_the_descriptor(of),
        Written::Named(_, held) => !held.is_empty(),
        _ => false,
    }
}

fn written_as_a_signature(
    what: &Written,
    classpath: &Classpath,
    unit: &Unit,
    of_the_class: &[String],
    of_the_method: &[String],
) -> Option<String> {
    Some(match what {
        Written::Void => "V".to_string(),
        Written::Boolean => "Z".to_string(),
        Written::Byte => "B".to_string(),
        Written::Short => "S".to_string(),
        Written::Char => "C".to_string(),
        Written::Int => "I".to_string(),
        Written::Long => "J".to_string(),
        Written::Float => "F".to_string(),
        Written::Double => "D".to_string(),
        Written::Inferred => return None,
        Written::Array(of) => {
            format!(
                "[{}",
                written_as_a_signature(of, classpath, unit, of_the_class, of_the_method)?
            )
        }
        Written::Variable(at, bound) => match of_the_class.get(*at) {
            Some(name) => format!("T{name};"),

            None => written_as_a_signature(bound, classpath, unit, of_the_class, of_the_method)?,
        },
        Written::OwnVariable(name, _) => format!("T{name};"),

        Written::Named(name, held)
            if held.is_empty() && (of_the_method.contains(name) || of_the_class.contains(name)) =>
        {
            format!("T{name};")
        }
        Written::Named(name, held) => {
            let internal = resolve_named(classpath, unit, name)?;
            if held.is_empty() {
                format!("L{internal};")
            } else {
                let mut inside = String::new();
                for one in held {
                    inside.push_str(&written_as_a_signature(
                        one,
                        classpath,
                        unit,
                        of_the_class,
                        of_the_method,
                    )?);
                }
                format!("L{internal}<{inside}>;")
            }
        }
    })
}

fn what_it_holds_open(
    held: &[(String, Written)],
    classpath: &Classpath,
    unit: &Unit,
    of_the_class: &[String],
    of_the_method: &[String],
) -> Option<String> {
    if held.is_empty() {
        return Some(String::new());
    }
    let mut out = String::from("<");
    for (name, bound) in held {
        out.push_str(name);

        out.push(':');
        if let Written::Named(named, _) = bound {
            if resolve_named(classpath, unit, named)
                .and_then(|found| classpath.get(&found).map(|known| known.interface))
                .unwrap_or(false)
            {
                out.push(':');
            }
        }
        out.push_str(&written_as_a_signature(
            bound,
            classpath,
            unit,
            of_the_class,
            of_the_method,
        )?);
    }
    out.push('>');
    Some(out)
}

fn class_signature(classpath: &Classpath, unit: &Unit) -> Option<String> {
    let named: Vec<String> = unit
        .own_variables
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    let generic_above = unit.above.iter().any(says_more_than_the_descriptor);
    if named.is_empty() && !generic_above {
        return None;
    }
    let mut out = what_it_holds_open(&unit.own_variables, classpath, unit, &named, &[])?;

    let above_by_name = |written: &String| -> Option<&Written> {
        unit.above.iter().find(|one| match one {
            Written::Named(name, _) => name == written,
            _ => false,
        })
    };
    match &unit.extends {
        Some(named_above) => {
            let written = above_by_name(named_above);
            match written {
                Some(one) => {
                    out.push_str(&written_as_a_signature(one, classpath, unit, &named, &[])?)
                }
                None => out.push_str(&format!(
                    "L{};",
                    resolve_named(classpath, unit, named_above)?
                )),
            }
        }
        None if unit.shape == Shape::Interface => out.push_str("Ljava/lang/Object;"),
        None => out.push_str("Ljava/lang/Object;"),
    }
    for one in &unit.implements {
        match above_by_name(one) {
            Some(written) => out.push_str(&written_as_a_signature(
                written,
                classpath,
                unit,
                &named,
                &[],
            )?),
            None => out.push_str(&format!("L{};", resolve_named(classpath, unit, one)?)),
        }
    }
    Some(out)
}

fn method_signature(classpath: &Classpath, unit: &Unit, method: &Method) -> Option<String> {
    let of_the_class: Vec<String> = unit
        .own_variables
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    let generic = !method.own_variables.is_empty()
        || says_more_than_the_descriptor(&method.returns)
        || method
            .parameters
            .iter()
            .any(|(what, _)| says_more_than_the_descriptor(what));
    if !generic {
        return None;
    }

    let object = Written::Named("java.lang.Object".to_string(), Vec::new());
    let mine: Vec<(String, Written)> = method
        .own_variables
        .iter()
        .enumerate()
        .map(|(at, name)| {
            (
                name.clone(),
                method.own_bounds.get(at).cloned().unwrap_or(object.clone()),
            )
        })
        .collect();
    let mut out = what_it_holds_open(&mine, classpath, unit, &of_the_class, &method.own_variables)?;
    out.push('(');
    for (what, _) in &method.parameters {
        out.push_str(&written_as_a_signature(
            what,
            classpath,
            unit,
            &of_the_class,
            &method.own_variables,
        )?);
    }
    out.push(')');
    out.push_str(&written_as_a_signature(
        &method.returns,
        classpath,
        unit,
        &of_the_class,
        &method.own_variables,
    )?);
    Some(out)
}

fn thrown_by(
    pool: &mut Pool,
    classpath: &Classpath,
    unit: &Unit,
    method: &Method,
) -> Option<(u16, Vec<u8>)> {
    if method.throws.is_empty() {
        return None;
    }
    let mut named = Vec::new();
    for one in &method.throws {
        if let Some(found) = resolve_named(classpath, unit, one) {
            named.push(pool.class(&found));
        }
    }
    if named.is_empty() {
        return None;
    }
    let mut body = Vec::with_capacity(2 + named.len() * 2);
    body.extend_from_slice(&(named.len() as u16).to_be_bytes());
    for index in named {
        body.extend_from_slice(&index.to_be_bytes());
    }
    Some((pool.utf8("Exceptions"), body))
}

fn write_attribute(out: &mut Vec<u8>, name: u16, body: &[u8]) {
    out.extend_from_slice(&name.to_be_bytes());
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(body);
}

fn runtime_annotations(
    pool: &mut Pool,
    classpath: &Classpath,
    unit: &Unit,
    held: &[Annotated],
) -> Option<(u16, Vec<u8>)> {
    let mut written = Vec::new();
    for one in held {
        let Some(named) = resolve_named(classpath, unit, &one.name) else {
            continue;
        };
        let kept = classpath
            .get(&named)
            .map(|known| known.kept_at_runtime)
            .unwrap_or_else(|| KEPT_AT_RUNTIME.contains(&named.as_str()));
        if !kept {
            continue;
        }
        let mut body = Vec::new();
        let descriptor = pool.utf8(&format!("L{named};"));
        body.extend_from_slice(&descriptor.to_be_bytes());
        let mut pairs = Vec::new();
        for (name, value) in &one.values {
            let Some(encoded) = annotation_value(pool, classpath, unit, value) else {
                continue;
            };
            let index = pool.utf8(name);
            pairs.push((index, encoded));
        }
        body.extend_from_slice(&(pairs.len() as u16).to_be_bytes());
        for (index, encoded) in pairs {
            body.extend_from_slice(&index.to_be_bytes());
            body.extend_from_slice(&encoded);
        }
        written.push(body);
    }
    if written.is_empty() {
        return None;
    }
    let mut body = Vec::new();
    body.extend_from_slice(&(written.len() as u16).to_be_bytes());
    for one in written {
        body.extend_from_slice(&one);
    }
    Some((pool.utf8("RuntimeVisibleAnnotations"), body))
}

fn runtime_parameter_annotations(
    pool: &mut Pool,
    classpath: &Classpath,
    unit: &Unit,
    method: &Method,
) -> Option<(u16, Vec<u8>)> {
    if method.constructor
        || method.parameter_annotations.len() != method.parameters.len()
        || method.parameters.len() > usize::from(u8::MAX)
    {
        return None;
    }
    let mut sets = Vec::new();
    let mut any = false;
    for held in &method.parameter_annotations {
        match runtime_annotations(pool, classpath, unit, held) {
            Some((_, body)) => {
                any = true;
                sets.push(body);
            }
            None => sets.push(vec![0, 0]),
        }
    }
    if !any {
        return None;
    }
    let mut body = vec![method.parameters.len() as u8];
    for one in sets {
        body.extend_from_slice(&one);
    }
    Some((pool.utf8("RuntimeVisibleParameterAnnotations"), body))
}

fn annotation_value(
    pool: &mut Pool,
    classpath: &Classpath,
    unit: &Unit,
    value: &Expression,
) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    match value {
        Expression::Str(text) => {
            out.push(b's');
            out.extend_from_slice(&pool.utf8(text).to_be_bytes());
        }
        Expression::Boolean(held) => {
            out.push(b'Z');
            out.extend_from_slice(&pool.integer(i32::from(*held)).to_be_bytes());
        }
        Expression::Char(held) => {
            out.push(b'C');
            out.extend_from_slice(&pool.integer(i32::from(*held)).to_be_bytes());
        }
        Expression::Long(held) => {
            out.push(b'J');
            out.extend_from_slice(&pool.long(*held).to_be_bytes());
        }
        Expression::Float(held) => {
            out.push(b'F');
            out.extend_from_slice(&pool.float(*held as f32).to_be_bytes());
        }
        Expression::Double(held) => {
            out.push(b'D');
            out.extend_from_slice(&pool.double(*held).to_be_bytes());
        }
        Expression::ClassLiteral { of, .. } => {
            let named = match of {
                Written::Named(name, _) => {
                    Type::Object(resolve_named(classpath, unit, name)?, Vec::new())
                }
                other => shallow_type(classpath, unit, other)?,
            };
            out.push(b'c');
            out.extend_from_slice(&pool.utf8(&named.descriptor()).to_be_bytes());
        }
        Expression::ArrayOf { values, .. } => {
            out.push(b'[');
            let mut held = Vec::new();
            for one in values {
                held.push(annotation_value(pool, classpath, unit, one)?);
            }
            out.extend_from_slice(&(held.len() as u16).to_be_bytes());
            for one in held {
                out.extend_from_slice(&one);
            }
        }
        other => {
            if let Some(value) = constant_int(other) {
                out.push(b'I');
                out.extend_from_slice(&pool.integer(value).to_be_bytes());
                return Some(out);
            }
            let path = written_as_a_path(other)?;
            let (before, last) = path.rsplit_once('.')?;
            let owner = resolve_named(classpath, unit, before)?;
            out.push(b'e');
            out.extend_from_slice(&pool.utf8(&format!("L{owner};")).to_be_bytes());
            out.extend_from_slice(&pool.utf8(last).to_be_bytes());
        }
    }
    Some(out)
}

fn shallow_type(classpath: &Classpath, unit: &Unit, written: &Written) -> Option<Type> {
    resolve_written(classpath, unit, written)
}

fn supers_reached_from_lambdas(
    expression: &Expression,
    inside: bool,
    out: &mut Vec<(String, usize)>,
) {
    match expression {
        Expression::Call {
            on: None,
            super_call: true,
            name,
            arguments,
            ..
        } if inside => {
            if !out
                .iter()
                .any(|(held, count)| held == name && *count == arguments.len())
            {
                out.push((name.clone(), arguments.len()));
            }
        }
        Expression::Anonymous { arguments, .. } => {
            for one in arguments {
                supers_reached_from_lambdas(one, inside, out);
            }
            return;
        }
        _ => {}
    }
    let deeper = inside || matches!(expression, Expression::Lambda { .. });
    walk_expression(expression, &mut |held| {
        supers_reached_from_lambdas(held, deeper, out)
    });
}

fn reaching_up_for_lambdas(unit: &Unit, classpath: &Classpath) -> Vec<Method> {
    if unit.stands_for_a_lambda || unit.shape == Shape::Interface {
        return Vec::new();
    }
    let mut wanted: Vec<(String, usize)> = Vec::new();
    let mut look = |statement: &Statement| {
        walk_statement(statement, &mut |held| {
            supers_reached_from_lambdas(held, false, &mut wanted)
        });
    };
    for method in unit.methods.iter().filter(|one| !one.modifiers.static_) {
        for held in method.body.iter().flatten() {
            look(&held.node);
        }
    }
    for held in &unit.instance_setup {
        look(&held.node);
    }
    for field in unit.fields.iter().filter(|one| !one.modifiers.static_) {
        if let Some(value) = &field.value {
            supers_reached_from_lambdas(value, false, &mut wanted);
        }
    }
    if wanted.is_empty() {
        return Vec::new();
    }

    let mut chain: Vec<String> = Vec::new();
    let mut current = unit
        .extends
        .as_ref()
        .and_then(|named| resolve_named(classpath, unit, named))
        .or_else(|| Some("java/lang/Object".to_string()));
    while let Some(one) = current {
        if chain.contains(&one) || chain.len() > 64 {
            break;
        }
        current = classpath
            .get(&one)
            .and_then(|known| known.superclass.clone());
        chain.push(one);
    }

    let mut made: Vec<Method> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for (name, count) in &wanted {
        for owner in &chain {
            let Some(known) = classpath.get(owner) else {
                continue;
            };
            for found in &known.methods {
                if found.name != *name
                    || found.parameters.len() != *count
                    || found.static_
                    || found.abstract_
                {
                    continue;
                }
                let descriptor = found.descriptor();
                let key = format!("{name}{descriptor}");
                if seen.contains(&key) {
                    continue;
                }
                let parameters: Option<Vec<(Written, String)>> = found
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(at, what)| written_for(what).map(|one| (one, format!("$p{at}"))))
                    .collect();
                let Some(parameters) = parameters else {
                    continue;
                };
                let returns = if found.returns == Type::Void {
                    Written::Void
                } else {
                    let Some(one) = written_for(&found.returns) else {
                        continue;
                    };
                    one
                };
                seen.push(key);
                let reached = Expression::Call {
                    on: None,
                    super_call: true,
                    name: name.clone(),
                    arguments: parameters
                        .iter()
                        .map(|(_, held)| Expression::Name(held.clone()))
                        .collect(),
                    held: Vec::new(),
                };
                let line = unit.methods.first().map_or(1, |one| one.line);
                let node = if found.returns == Type::Void {
                    Statement::Express(reached)
                } else {
                    Statement::Return(Some(reached))
                };
                made.push(Method {
                    modifiers: Modifiers::default(),
                    returns,
                    name: format!("{REACHING_UP}{name}"),
                    parameters,
                    body: Some(vec![Positioned {
                        node,
                        line,
                        column: 1,
                    }]),
                    constructor: false,
                    variadic: false,
                    line,
                    bridge: false,
                    annotations: Vec::new(),
                    parameter_annotations: Vec::new(),
                    default_value: None,
                    own_variables: Vec::new(),
                    own_bounds: Vec::new(),
                    throws: Vec::new(),
                });
            }
        }
    }
    made
}

fn bridges_for(unit: &Unit, classpath: &Classpath) -> Vec<Method> {
    let resolve = |written: &Written| -> Option<Type> { resolve_written(classpath, unit, written) };

    let mut above: Vec<String> = Vec::new();
    let mut waiting: Vec<String> = Vec::new();
    if let Some(named) = &unit.extends {
        if let Some(found) = resolve_named(classpath, unit, named) {
            waiting.push(found);
        }
    }
    for named in &unit.implements {
        if let Some(found) = resolve_named(classpath, unit, named) {
            waiting.push(found);
        }
    }
    while let Some(one) = waiting.pop() {
        if above.contains(&one) || above.len() > 64 {
            continue;
        }
        above.push(one.clone());
        if let Some(known) = classpath.get(&one) {
            if let Some(up) = &known.superclass {
                waiting.push(up.clone());
            }

            for held in &known.interfaces {
                waiting.push(held.clone());
            }
        }
    }

    let mut made = Vec::new();
    for method in &unit.methods {
        if method.constructor || method.modifiers.static_ || method.body.is_none() {
            continue;
        }
        let Some(returns) = resolve(&method.returns) else {
            continue;
        };
        let mut parameters = Vec::new();
        let mut whole = true;
        for (written, _) in &method.parameters {
            match resolve(written) {
                Some(found) => parameters.push(found),
                None => whole = false,
            }
        }
        if !whole {
            continue;
        }

        for owner in &above {
            let Some(known) = classpath.get(owner) else {
                continue;
            };
            let seen = classpath.seen_from_within(&unit.internal_name(), &[], owner, 0);
            for candidate in &known.methods {
                if candidate.name != method.name
                    || candidate.static_
                    || candidate.parameters.len() != parameters.len()
                {
                    continue;
                }
                if candidate.parameters == parameters && candidate.returns == returns {
                    continue;
                }

                let variable = |at: usize, theirs: &Type, ours: &Type| match candidate
                    .taken_stands_for
                    .get(at)
                {
                    None => candidate.taken_stands_for.is_empty(),
                    Some(Standing::OfTheMethod(_)) => true,
                    Some(standing @ (Standing::OfTheClass(_) | Standing::ArrayOf(_))) => {
                        match &seen {
                            Some(arguments) => standing.filled(theirs, arguments, &[]) == *ours,
                            None => true,
                        }
                    }
                    Some(_) => false,
                };
                let fits = candidate
                    .parameters
                    .iter()
                    .zip(&parameters)
                    .enumerate()
                    .all(|(at, (theirs, ours))| {
                        theirs == ours
                            || (theirs.is_reference()
                                && ours.is_reference()
                                && variable(at, theirs, ours))
                    })
                    && (returns == candidate.returns
                        || (returns.is_reference() && candidate.returns.is_reference()));
                if !fits {
                    continue;
                }
                let shape: Vec<(Written, String)> = candidate
                    .parameters
                    .iter()
                    .enumerate()
                    .filter_map(|(at, one)| Some((written_for(one)?, format!("$bridge{at}"))))
                    .collect();
                if shape.len() != parameters.len() {
                    continue;
                }
                if made.iter().any(|held: &Method| {
                    held.name == method.name
                        && held.parameters.len() == shape.len()
                        && held
                            .parameters
                            .iter()
                            .zip(&shape)
                            .all(|((a, _), (b, _))| format!("{a:?}") == format!("{b:?}"))
                }) {
                    continue;
                }

                let arguments: Vec<Expression> = shape
                    .iter()
                    .zip(&method.parameters)
                    .map(|((_, held), (what, _))| Expression::Cast {
                        to: what.clone(),
                        of: Box::new(Expression::Name(held.clone())),
                        also: Vec::new(),
                    })
                    .collect();
                let call = Expression::Call {
                    on: Some(Box::new(Expression::This)),
                    super_call: false,
                    held: Vec::new(),
                    name: method.name.clone(),
                    arguments,
                };
                let body = vec![Positioned {
                    node: if returns == Type::Void {
                        Statement::Express(call)
                    } else {
                        Statement::Return(Some(call))
                    },
                    line: method.line,
                    column: 1,
                }];
                made.push(Method {
                    modifiers: Modifiers {
                        public: true,
                        ..Modifiers::default()
                    },
                    returns: match written_for(&candidate.returns) {
                        Some(found) => found,
                        None => Written::Void,
                    },
                    name: method.name.clone(),
                    parameters: shape,
                    body: Some(body),
                    constructor: false,
                    variadic: false,
                    bridge: true,
                    line: method.line,
                    annotations: Vec::new(),
                    parameter_annotations: Vec::new(),
                    default_value: None,
                    own_variables: Vec::new(),
                    own_bounds: Vec::new(),

                    throws: method.throws.clone(),
                });
            }
        }
    }
    made
}

fn method_shape(unit: &Unit, method: &Method) -> u16 {
    let mut flags = 0u16;

    if method.bridge {
        flags |= 0x0040 | 0x1000;
    }

    if method.variadic {
        flags |= 0x0080;
    }

    if unit.shape == Shape::Interface && method.body.is_none() && !method.modifiers.static_ {
        flags |= 0x0400;
    }
    flags
}

fn everything_abstract_is_implemented(
    unit: &Unit,
    superclass: &str,
    interfaces: &[String],
    classpath: &Classpath,
) -> Result<(), Diagnostic> {
    fn walk(
        name: &str,
        classpath: &Classpath,
        seen: &mut std::collections::BTreeSet<String>,
        required: &mut std::collections::BTreeSet<(String, usize)>,
        provided: &mut std::collections::BTreeSet<(String, usize)>,
    ) {
        if name == "java/lang/Object" || !seen.insert(name.to_string()) {
            return;
        }
        let Some(known) = classpath.get(name) else {
            return;
        };
        let complete = !known.interface && !known.abstract_;
        for method in &known.methods {
            if method.static_ || method.name.starts_with('<') {
                continue;
            }
            let mark = (method.name.clone(), method.parameters.len());
            if method.abstract_ && !complete {
                required.insert(mark);
            } else {
                provided.insert(mark);
            }
        }
        if complete {
            let mut above: Vec<String> = known.superclass.iter().cloned().collect();
            above.extend(known.interfaces.iter().cloned());
            while let Some(one) = above.pop() {
                if !seen.insert(one.clone()) {
                    continue;
                }
                let Some(held) = classpath.get(&one) else {
                    continue;
                };
                for method in &held.methods {
                    if !method.static_ && !method.name.starts_with('<') {
                        provided.insert((method.name.clone(), method.parameters.len()));
                    }
                }
                above.extend(held.superclass.iter().cloned());
                above.extend(held.interfaces.iter().cloned());
            }
            return;
        }
        if let Some(above) = &known.superclass {
            walk(above, classpath, seen, required, provided);
        }
        for one in &known.interfaces {
            walk(one, classpath, seen, required, provided);
        }
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut required = std::collections::BTreeSet::new();
    let mut provided = std::collections::BTreeSet::new();

    for method in &unit.methods {
        if method.constructor || method.name.starts_with('<') {
            continue;
        }
        if method.body.is_some() || method.modifiers.native_ {
            provided.insert((method.name.clone(), method.parameters.len()));
        }
    }

    walk(
        superclass,
        classpath,
        &mut seen,
        &mut required,
        &mut provided,
    );
    for one in interfaces {
        walk(one, classpath, &mut seen, &mut required, &mut provided);
    }

    let missing: Vec<&(String, usize)> = required.difference(&provided).collect();
    let Some((name, count)) = missing.first() else {
        return Ok(());
    };

    let here = unit.internal_name().replace('/', ".");
    let _ = count;
    Err(at(
        "EJ257",
        1,
        1,
        format!(
            "`{here}` is not abstract and does not implement `{name}`, \
             a method its supertypes leave abstract."
        ),
    )
    .with_context(format!("Method: {name}"))
    .with_suggestion("Give the class a body for that method, or mark the class abstract."))
}

pub fn compile_unit(
    unit: &Unit,
    classpath: &Classpath,
) -> Result<(Vec<u8>, Vec<Unit>), Diagnostic> {
    compile_unit_in_nest(unit, classpath, &[])
}

pub fn compile_unit_in_nest(
    unit: &Unit,
    classpath: &Classpath,
    nest: &[String],
) -> Result<(Vec<u8>, Vec<Unit>), Diagnostic> {
    let held;
    let unit = if holds_an_assert(unit) {
        held = with_the_assertion_flag(unit);
        &held
    } else {
        unit
    };

    let this_class = unit.internal_name();
    let mut pool = Pool::new();
    let mut made: Vec<Unit> = Vec::new();

    let superclass = match &unit.extends {
        Some(name) => {
            let probe = Emitter::new(
                &mut pool,
                classpath,
                unit,
                this_class.clone(),
                true,
                &mut made,
            );
            probe.resolve_class(name, 1)?
        }
        None => "java/lang/Object".to_string(),
    };

    let mut interfaces = Vec::new();
    if unit.annotation {
        interfaces.push("java/lang/annotation/Annotation".to_string());
    }
    for name in &unit.implements {
        let probe = Emitter::new(
            &mut pool,
            classpath,
            unit,
            this_class.clone(),
            true,
            &mut made,
        );
        interfaces.push(probe.resolve_class(name, 1)?);
    }

    if unit.shape == Shape::Class && !unit.modifiers.abstract_ {
        everything_abstract_is_implemented(unit, &superclass, &interfaces, classpath)?;
    }

    let mut field_bytes = Vec::new();

    let mut folded: Vec<String> = Vec::new();
    let of_the_class: Vec<String> = unit
        .own_variables
        .iter()
        .map(|(name, _)| name.clone())
        .collect();
    for field in &unit.fields {
        let probe = Emitter::new(
            &mut pool,
            classpath,
            unit,
            this_class.clone(),
            true,
            &mut made,
        );
        let what = probe.resolve(&field.what, field.line)?;
        let name = pool.utf8(&field.name);
        let descriptor = pool.utf8(&what.descriptor());
        let held = runtime_annotations(&mut pool, classpath, unit, &field.annotations);

        let constant = folded_constant(field, &what, &mut pool)
            .map(|index| (pool.utf8("ConstantValue"), index));
        let signature = says_more_than_the_descriptor(&field.what)
            .then(|| written_as_a_signature(&field.what, classpath, unit, &of_the_class, &[]))
            .flatten()
            .map(|written| {
                let index = pool.utf8(&written);
                (pool.utf8("Signature"), index)
            });
        let how_many = u16::from(held.is_some())
            + u16::from(constant.is_some())
            + u16::from(signature.is_some());
        field_bytes.extend_from_slice(&field.modifiers.access_flags(0).to_be_bytes());
        field_bytes.extend_from_slice(&name.to_be_bytes());
        field_bytes.extend_from_slice(&descriptor.to_be_bytes());
        field_bytes.extend_from_slice(&how_many.to_be_bytes());
        if let Some((name, body)) = held {
            write_attribute(&mut field_bytes, name, &body);
        }
        if let Some((name, index)) = constant {
            folded.push(field.name.clone());
            write_attribute(&mut field_bytes, name, &index.to_be_bytes());
        }
        if let Some((name, index)) = signature {
            write_attribute(&mut field_bytes, name, &index.to_be_bytes());
        }
    }

    let assignment = |field: &Field| -> Option<Positioned<Statement>> {
        let value = field.value.clone()?;
        let target = if field.modifiers.static_ {
            Expression::Name(field.name.clone())
        } else {
            Expression::Field {
                of: Box::new(Expression::This),
                name: field.name.clone(),
            }
        };
        Some(Positioned {
            node: Statement::Express(Expression::Assign {
                target: Box::new(target),
                operator: None,
                value: Box::new(value),
            }),
            line: field.line,
            column: 1,
        })
    };

    let mut instance_setup: Vec<Positioned<Statement>> = unit
        .fields
        .iter()
        .filter(|field| !field.modifiers.static_)
        .filter_map(assignment)
        .chain(unit.instance_setup.iter().cloned())
        .collect();
    instance_setup.sort_by_key(|held| held.line);

    let mut static_setup: Vec<Positioned<Statement>> = unit
        .fields
        .iter()
        .filter(|field| field.modifiers.static_)
        .filter(|field| !folded.contains(&field.name))
        .filter_map(assignment)
        .chain(unit.static_setup.iter().cloned())
        .collect();
    static_setup.sort_by_key(|held| held.line);

    let mut methods: Vec<Method> = unit.methods.clone();
    if unit.shape == Shape::Interface {
        for method in &mut methods {
            if method.body.is_none() {
                method.modifiers.abstract_ = true;
            }
        }
    }
    if !static_setup.is_empty() {
        methods.push(Method {
            modifiers: Modifiers {
                static_: true,
                ..Modifiers::default()
            },
            returns: Written::Void,
            name: "<clinit>".to_string(),
            parameters: Vec::new(),
            body: Some(static_setup),
            constructor: false,
            variadic: false,
            line: 1,
            bridge: false,
            annotations: Vec::new(),
            parameter_annotations: Vec::new(),
            default_value: None,
            own_variables: Vec::new(),
            own_bounds: Vec::new(),
            throws: Vec::new(),
        });
    }
    if unit.shape == Shape::Class && !methods.iter().any(|held| held.constructor) {
        methods.insert(
            0,
            Method {
                modifiers: Modifiers {
                    public: unit.modifiers.public,
                    ..Modifiers::default()
                },
                returns: Written::Void,
                name: "<init>".to_string(),
                parameters: Vec::new(),
                body: Some(Vec::new()),
                constructor: true,
                variadic: false,
                line: 1,
                bridge: false,
                annotations: Vec::new(),
                parameter_annotations: Vec::new(),
                default_value: None,
                own_variables: Vec::new(),
                own_bounds: Vec::new(),
                throws: Vec::new(),
            },
        );
    }

    methods.extend(bridges_for(unit, classpath));
    methods.extend(reaching_up_for_lambdas(unit, classpath));

    let code_name = pool.utf8("Code");
    let mut method_bytes = Vec::new();
    let mut written = 0u16;

    for method in &methods {
        let Some(body) = &method.body else {
            let name = pool.utf8(&method.name);
            let probe = Emitter::new(
                &mut pool,
                classpath,
                unit,
                this_class.clone(),
                true,
                &mut made,
            );
            let mut parameters = Vec::new();
            for (what, _) in &method.parameters {
                parameters.push(probe.resolve(what, method.line)?);
            }
            let returns = probe.resolve(&method.returns, method.line)?;
            let descriptor = Signature {
                owner: this_class.clone(),
                name: method.name.clone(),
                parameters,
                returns,
                static_: method.modifiers.static_,
                interface: false,
                variadic: false,
                abstract_: false,
                returns_variable: None,
                parameter_variables: Vec::new(),
                taken_stands_for: Vec::new(),
                returns_stands_for: Standing::Fixed,
                own_variables: Vec::new(),
                witnessed: Vec::new(),
                throws: Vec::new(),
            }
            .descriptor();
            let descriptor = pool.utf8(&descriptor);
            method_bytes.extend_from_slice(
                &method
                    .modifiers
                    .access_flags(method_shape(unit, method))
                    .to_be_bytes(),
            );
            let held = runtime_annotations(&mut pool, classpath, unit, &method.annotations);
            let marked = runtime_parameter_annotations(&mut pool, classpath, unit, method);

            let default_value = method.default_value.as_ref().and_then(|value| {
                let body = annotation_value(&mut pool, classpath, unit, value)?;
                Some((pool.utf8("AnnotationDefault"), body))
            });
            let thrown = thrown_by(&mut pool, classpath, unit, method);
            let signature = method_signature(classpath, unit, method).map(|written| {
                let index = pool.utf8(&written);
                (pool.utf8("Signature"), index)
            });
            let how_many = u16::from(held.is_some())
                + u16::from(marked.is_some())
                + u16::from(default_value.is_some())
                + u16::from(thrown.is_some())
                + u16::from(signature.is_some());
            method_bytes.extend_from_slice(&name.to_be_bytes());
            method_bytes.extend_from_slice(&descriptor.to_be_bytes());
            method_bytes.extend_from_slice(&how_many.to_be_bytes());
            if let Some((name, body)) = held {
                write_attribute(&mut method_bytes, name, &body);
            }
            if let Some((name, body)) = marked {
                write_attribute(&mut method_bytes, name, &body);
            }
            if let Some((name, body)) = default_value {
                write_attribute(&mut method_bytes, name, &body);
            }
            if let Some((name, body)) = thrown {
                write_attribute(&mut method_bytes, name, &body);
            }
            if let Some((name, index)) = signature {
                write_attribute(&mut method_bytes, name, &index.to_be_bytes());
            }
            written += 1;
            continue;
        };

        let mut emitter = Emitter::new(
            &mut pool,
            classpath,
            unit,
            this_class.clone(),
            method.modifiers.static_,
            &mut made,
        );

        emitter.writing_a_bridge = method.bridge;

        emitter.said_to_throw = method
            .throws
            .iter()
            .filter_map(|one| resolve_named(classpath, unit, one))
            .collect();

        if method.name == "<clinit>" {
            emitter
                .said_to_throw
                .push("java/lang/Throwable".to_string());
        }
        emitter.open();
        if !method.modifiers.static_ {
            emitter.declare("this", Type::Object(this_class.clone(), Vec::new()));
        }
        let mut parameters = Vec::new();
        for (what, name) in &method.parameters {
            let resolved = emitter.resolve(what, method.line)?;
            emitter.declare(name, resolved.clone());
            parameters.push(resolved);
        }
        let returns = emitter.resolve(&method.returns, method.line)?;
        emitter.returns = returns.clone();
        if method.name != "<clinit>" {
            let mut descriptor = String::from("(");
            for one in &parameters {
                descriptor.push_str(&one.descriptor());
            }
            descriptor.push(')');
            descriptor.push_str(&returns.descriptor());
            emitter.within = Some((method.name.clone(), descriptor));
        }

        let mut body: &[Positioned<Statement>] = body;
        if method.constructor {
            let split = body
                .iter()
                .position(|one| matches!(one.node, Statement::Chain { .. }))
                .unwrap_or(0);
            let (prologue, rest) = body.split_at(split);
            body = rest;
            emitter.slots[0] = Verified::UninitializedThis;
            emitter.early = true;
            for statement in prologue {
                emitter.statement(statement)?;
                emitter.too_early_at(statement.line)?;
            }
            let chained = match body.first().map(|one| &one.node) {
                Some(Statement::Chain {
                    to_super,
                    arguments,
                }) => {
                    let line = body[0].line;
                    let owner = if *to_super {
                        superclass.clone()
                    } else {
                        this_class.clone()
                    };
                    emitter.chain_to(&owner, arguments, line)?;
                    emitter.too_early_at(line)?;
                    body = &body[1..];
                    Some(*to_super)
                }
                _ => {
                    emitter.early = false;
                    emitter.load(0, &Type::Object(this_class.clone(), Vec::new()));
                    let up = emitter.pool.method(&superclass, "<init>", "()V", false);
                    emitter.op2(0xb7, up);
                    emitter.pops(1);
                    Some(true)
                }
            };
            emitter.early = false;
            emitter.slot_holds(0, &Type::Object(this_class.clone(), Vec::new()));

            if chained == Some(true) {
                for statement in &instance_setup {
                    emitter.statement(statement)?;
                }
            }
        }

        for statement in body {
            emitter.statement(statement)?;
        }

        let ends_returned = !emitter.reachable || body.iter().any(|one| never_completes(&one.node));
        let _ = &instance_setup;
        if returns == Type::Void {
            if !ends_returned {
                emitter.op(0xb1);
            }
        } else if !ends_returned {
            return Err(at(
                "EJ232",
                method.line,
                1,
                format!(
                    "`{}` can reach its end without returning a value.",
                    method.name
                ),
            ));
        }

        let code = emitter.code;
        let max_stack = emitter.max_depth.max(1) as u16;
        let max_locals = emitter.max_slot.max(1);
        let frames = emitter.frames;
        let handlers = emitter.handlers;
        let written_on = emitter.lines;

        let frames: Vec<Frame> = frames
            .into_iter()
            .filter(|frame| frame.at < code.len())
            .collect();
        let table = stack_map_table(&frames, &mut pool);
        let table_name = table.as_ref().map(|_| pool.utf8("StackMapTable"));

        let mut attribute = Vec::new();
        attribute.extend_from_slice(&max_stack.to_be_bytes());
        attribute.extend_from_slice(&max_locals.to_be_bytes());
        attribute.extend_from_slice(&(code.len() as u32).to_be_bytes());
        attribute.extend_from_slice(&code);
        attribute.extend_from_slice(&(handlers.len() as u16).to_be_bytes());
        for handler in &handlers {
            attribute.extend_from_slice(&(handler.start as u16).to_be_bytes());
            attribute.extend_from_slice(&(handler.end as u16).to_be_bytes());
            attribute.extend_from_slice(&(handler.target as u16).to_be_bytes());
            let class = match &handler.class {
                Some(name) => pool.class(name),

                None => 0,
            };
            attribute.extend_from_slice(&class.to_be_bytes());
        }

        let mut lines: Vec<(u16, u16)> = written_on
            .into_iter()
            .filter(|(at, _)| usize::from(*at) < code.len())
            .collect();
        lines.dedup_by_key(|(at, _)| *at);
        let table_of_lines = (!lines.is_empty()).then(|| {
            let mut body = Vec::with_capacity(2 + lines.len() * 4);
            body.extend_from_slice(&(lines.len() as u16).to_be_bytes());
            for (at, said) in &lines {
                body.extend_from_slice(&at.to_be_bytes());
                body.extend_from_slice(&said.to_be_bytes());
            }
            (pool.utf8("LineNumberTable"), body)
        });

        let how_many = u16::from(table.is_some()) + u16::from(table_of_lines.is_some());
        attribute.extend_from_slice(&how_many.to_be_bytes());
        if let (Some(body), Some(name)) = (&table, table_name) {
            write_attribute(&mut attribute, name, body);
        }
        if let Some((name, body)) = &table_of_lines {
            write_attribute(&mut attribute, *name, body);
        }

        let name = pool.utf8(&method.name);
        let descriptor = Signature {
            owner: this_class.clone(),
            name: method.name.clone(),
            parameters,
            returns,
            static_: method.modifiers.static_,
            interface: false,
            variadic: false,
            abstract_: false,
            returns_variable: None,
            parameter_variables: Vec::new(),
            taken_stands_for: Vec::new(),
            returns_stands_for: Standing::Fixed,
            own_variables: Vec::new(),
            witnessed: Vec::new(),
            throws: Vec::new(),
        }
        .descriptor();
        let descriptor = pool.utf8(&descriptor);

        method_bytes.extend_from_slice(
            &method
                .modifiers
                .access_flags(method_shape(unit, method))
                .to_be_bytes(),
        );
        let held = runtime_annotations(&mut pool, classpath, unit, &method.annotations);
        let marked = runtime_parameter_annotations(&mut pool, classpath, unit, method);
        let thrown = thrown_by(&mut pool, classpath, unit, method);
        let signature = method_signature(classpath, unit, method).map(|written| {
            let index = pool.utf8(&written);
            (pool.utf8("Signature"), index)
        });
        let how_many = 1
            + u16::from(held.is_some())
            + u16::from(marked.is_some())
            + u16::from(thrown.is_some())
            + u16::from(signature.is_some());
        method_bytes.extend_from_slice(&name.to_be_bytes());
        method_bytes.extend_from_slice(&descriptor.to_be_bytes());
        method_bytes.extend_from_slice(&how_many.to_be_bytes());
        write_attribute(&mut method_bytes, code_name, &attribute);
        if let Some((name, body)) = held {
            write_attribute(&mut method_bytes, name, &body);
        }
        if let Some((name, body)) = marked {
            write_attribute(&mut method_bytes, name, &body);
        }
        if let Some((name, body)) = thrown {
            write_attribute(&mut method_bytes, name, &body);
        }
        if let Some((name, index)) = signature {
            write_attribute(&mut method_bytes, name, &index.to_be_bytes());
        }
        written += 1;
    }

    let shape_flags = match unit.shape {
        Shape::Class => 0x0020u16,

        Shape::Interface if unit.annotation => 0x0200 | 0x0400 | 0x2000,
        Shape::Interface => 0x0200 | 0x0400,

        Shape::Enum => {
            let left_open = unit
                .methods
                .iter()
                .any(|held| held.body.is_none() && !held.modifiers.static_);
            0x0020 | 0x4000 | if left_open { 0x0400 } else { 0 }
        }

        Shape::Record => 0x0020 | 0x0010,
    };
    let mut class_attributes: Vec<(u16, Vec<u8>)> = Vec::new();
    if let Some(held) = runtime_annotations(&mut pool, classpath, unit, &unit.annotations) {
        class_attributes.push(held);
    }

    let written_in = unit.source.clone().unwrap_or_else(|| {
        let outermost = unit.name.split('$').next().unwrap_or(&unit.name);
        format!("{outermost}.java")
    });
    let held = pool.utf8(&written_in);
    class_attributes.push((pool.utf8("SourceFile"), held.to_be_bytes().to_vec()));

    if let Some(written) = class_signature(classpath, unit) {
        let index = pool.utf8(&written);
        class_attributes.push((pool.utf8("Signature"), index.to_be_bytes().to_vec()));
    }

    if !unit.permits.is_empty() {
        let mut named = Vec::new();
        for one in &unit.permits {
            let probe = Emitter::new(
                &mut pool,
                classpath,
                unit,
                this_class.clone(),
                true,
                &mut made,
            );
            named.push(probe.resolve_class(one, 1)?);
        }
        let mut body = Vec::new();
        body.extend_from_slice(&(named.len() as u16).to_be_bytes());
        for one in &named {
            body.extend_from_slice(&pool.class(one).to_be_bytes());
        }
        class_attributes.push((pool.utf8("PermittedSubclasses"), body));
    }

    if unit.shape == Shape::Record {
        let components: Vec<&Field> = unit
            .fields
            .iter()
            .filter(|field| !field.modifiers.static_)
            .collect();
        let mut body = Vec::new();
        body.extend_from_slice(&(components.len() as u16).to_be_bytes());
        for field in components {
            let probe = Emitter::new(
                &mut pool,
                classpath,
                unit,
                this_class.clone(),
                true,
                &mut made,
            );
            let what = probe.resolve(&field.what, field.line)?;
            let name = pool.utf8(&field.name);
            let descriptor = pool.utf8(&what.descriptor());
            body.extend_from_slice(&name.to_be_bytes());
            body.extend_from_slice(&descriptor.to_be_bytes());

            body.extend_from_slice(&0u16.to_be_bytes());
        }
        class_attributes.push((pool.utf8("Record"), body));
    }

    match this_class.split_once('$') {
        Some((host, _)) => {
            let index = pool.class(host);
            class_attributes.push((pool.utf8("NestHost"), index.to_be_bytes().to_vec()));
        }
        None => {
            let mine: Vec<&String> = nest
                .iter()
                .filter(|held| held.starts_with(&format!("{this_class}$")) && **held != this_class)
                .collect();
            if !mine.is_empty() {
                let mut body = Vec::new();
                body.extend_from_slice(&(mine.len() as u16).to_be_bytes());
                for one in mine {
                    body.extend_from_slice(&pool.class(one).to_be_bytes());
                }
                class_attributes.push((pool.utf8("NestMembers"), body));
            }
        }
    }

    {
        let top = this_class.split('$').next().unwrap_or("").to_string();
        let mut family: Vec<String> = nest
            .iter()
            .filter(|held| held.contains('$') && held.split('$').next() == Some(top.as_str()))
            .cloned()
            .collect();
        if this_class.contains('$') && !family.contains(&this_class) {
            family.push(this_class.clone());
        }
        family.sort();
        family.dedup();
        if !family.is_empty() {
            let mut body = Vec::new();
            body.extend_from_slice(&(family.len() as u16).to_be_bytes());
            for one in &family {
                let (holder, simple) = one.rsplit_once('$').unwrap_or(("", one.as_str()));

                let unnamed = simple.chars().all(|held| held.is_ascii_digit());
                let inner = pool.class(one);
                let outer = if unnamed { 0 } else { pool.class(holder) };
                let named = if unnamed { 0 } else { pool.utf8(simple) };

                let flags = if *one == this_class {
                    unit.modifiers.access_flags(shape_flags & !0x0020)
                } else {
                    0x0001
                };
                body.extend_from_slice(&inner.to_be_bytes());
                body.extend_from_slice(&outer.to_be_bytes());
                body.extend_from_slice(&named.to_be_bytes());
                body.extend_from_slice(&flags.to_be_bytes());
            }
            class_attributes.push((pool.utf8("InnerClasses"), body));
        }
    }

    if let Some((holder, method)) = &unit.enclosed_by {
        let mut body = Vec::new();
        body.extend_from_slice(&pool.class(holder).to_be_bytes());
        let method = match method {
            Some((name, descriptor)) => pool.name_and_type(name, descriptor),
            None => 0,
        };
        body.extend_from_slice(&method.to_be_bytes());
        class_attributes.push((pool.utf8("EnclosingMethod"), body));
    }

    let this_index = pool.class(&this_class);
    let super_index = pool.class(&superclass);
    let interface_indices: Vec<u16> = interfaces.iter().map(|name| pool.class(name)).collect();

    let mut out = Vec::with_capacity(1024 + method_bytes.len());
    out.extend_from_slice(&0xcafe_babeu32.to_be_bytes());
    out.extend_from_slice(&CLASS_MINOR.to_be_bytes());
    out.extend_from_slice(&CLASS_MAJOR.to_be_bytes());
    pool.write(&mut out);
    out.extend_from_slice(&unit.modifiers.access_flags(shape_flags).to_be_bytes());
    out.extend_from_slice(&this_index.to_be_bytes());
    out.extend_from_slice(&super_index.to_be_bytes());
    out.extend_from_slice(&(interface_indices.len() as u16).to_be_bytes());
    for index in interface_indices {
        out.extend_from_slice(&index.to_be_bytes());
    }
    out.extend_from_slice(&(unit.fields.len() as u16).to_be_bytes());
    out.extend_from_slice(&field_bytes);
    out.extend_from_slice(&written.to_be_bytes());
    out.extend_from_slice(&method_bytes);
    out.extend_from_slice(&(class_attributes.len() as u16).to_be_bytes());
    for (name, body) in &class_attributes {
        write_attribute(&mut out, *name, body);
    }
    Ok((out, made))
}

pub fn compile(source: &str, classpath: &Classpath) -> Result<Vec<(String, Vec<u8>)>, Diagnostic> {
    compile_together(&[("".to_string(), source.to_string())], classpath)
}

type Declared = (Vec<(String, Unit)>, Classpath);

fn parsed_and_declared(
    sources: &[(String, String)],
    classpath: &Classpath,
) -> Result<Declared, Diagnostic> {
    let mut declared = Vec::new();

    for (label, text) in sources {
        let stem = label
            .rsplit(['/', '\\'])
            .next()
            .and_then(|held| held.strip_suffix(".java"))
            .filter(|held| {
                held.chars()
                    .next()
                    .is_some_and(|first| first.is_alphabetic() || first == '_')
                    && held
                        .chars()
                        .all(|one| one.is_alphanumeric() || one == '_' || one == '$')
            });
        let units = parse_as(text, stem).map_err(|error| named_file(error, label))?;

        let source = label
            .rsplit(['/', '\\'])
            .next()
            .filter(|held| !held.is_empty())
            .map(|held| held.to_string());
        for mut unit in units {
            unit.source.clone_from(&source);
            declared.push((label.clone(), unit));
        }
    }

    let mut together = classpath.clone();

    let inside: Vec<Unit> = declared
        .iter()
        .flat_map(|(_, unit)| classes_written_inside(unit))
        .collect();
    for (_, unit) in &declared {
        together.shell(unit);
    }
    for unit in &inside {
        together.shell(unit);
    }
    for _ in 0..2 {
        for (_, unit) in &declared {
            together.place(unit);
        }
        for unit in &inside {
            together.place(unit);
        }
    }
    for (_, unit) in &declared {
        together.declare(unit);
    }
    for unit in &inside {
        together.declare(unit);
    }

    Ok((declared, together))
}

pub fn declared_together(
    sources: &[(String, String)],
    classpath: &Classpath,
) -> Result<Classpath, Diagnostic> {
    Ok(parsed_and_declared(sources, classpath)?.1)
}

pub fn compile_together(
    sources: &[(String, String)],
    classpath: &Classpath,
) -> Result<Vec<(String, Vec<u8>)>, Diagnostic> {
    let (declared, mut together) = parsed_and_declared(sources, classpath)?;

    for (index, (label, unit)) in declared.iter().enumerate() {
        let name = unit.internal_name();
        if let Some((first, _)) = declared
            .iter()
            .take(index)
            .find(|(_, held)| held.internal_name() == name)
        {
            return Err(named_file(
                at(
                    "EJ253",
                    1,
                    1,
                    format!("`{}` is declared twice.", name.replace('/', ".")),
                )
                .with_context(format!("Also in: {first}"))
                .with_suggestion("One type, one place. Rename one of them."),
                label,
            ));
        }
    }

    let mut nest: Vec<String> = Vec::new();
    let mut waiting = declared.clone();
    let mut rounds = 0usize;
    while !waiting.is_empty() {
        rounds += 1;
        if rounds > 64 {
            return Err(at(
                "EJ119",
                1,
                1,
                "This keeps producing classes that produce more classes.",
            ));
        }
        let mut next = Vec::new();
        for (label, unit) in &waiting {
            nest.push(unit.internal_name());
            let (_, made) =
                compile_unit(unit, &together).map_err(|error| named_file(error, label))?;
            next.extend(made.into_iter().map(|one| (label.clone(), one)));
        }
        let inside: Vec<Unit> = next
            .iter()
            .flat_map(|(_, unit)| classes_written_inside(unit))
            .collect();
        for (_, unit) in &next {
            together.shell(unit);
        }
        for unit in &inside {
            together.shell(unit);
        }
        for (_, unit) in &next {
            together.place(unit);
        }
        for unit in &inside {
            together.place(unit);
        }
        for (_, unit) in &next {
            together.declare(unit);
        }
        for unit in &inside {
            together.declare(unit);
        }
        waiting = next;
    }

    let mut out = Vec::new();
    let mut waiting = declared;
    while !waiting.is_empty() {
        let mut next = Vec::new();
        for (label, unit) in &waiting {
            let name = format!("{}.class", unit.internal_name());
            let (bytes, made) = compile_unit_in_nest(unit, &together, &nest)
                .map_err(|error| named_file(error, label))?;
            out.push((name, bytes));
            next.extend(made.into_iter().map(|one| (label.clone(), one)));
        }
        waiting = next;
    }
    Ok(out)
}

fn named_file(mut error: Diagnostic, label: &str) -> Diagnostic {
    if label.is_empty() {
        return error;
    }
    match error.location.as_mut() {
        Some(place) if place.file.is_empty() => place.file = label.to_string(),
        Some(_) => {}
        None => error.location = Some(crate::diag::Location::file(label)),
    }
    error.with_context(format!("File: {label}"))
}

pub static CONTRACT: Contract = Contract {
    id: "plugin.java",
    display_name: "Java",
    version: Version::new(0, 2, 0),
    produces_artifacts: true,
    summary: "Reads Java source and writes class files, without a javac anywhere. \
              A subset of the language, named at the top of Compilers/Java.rs, with \
              everything outside it refused by name and line rather than mis-handled.",
    inputs: &["java.source", "jvm.class"],
    outputs: &["jvm.class"],
    required_capabilities: &[Capability::Cache, Capability::TempStorage],
    forbidden_capabilities: &[
        Capability::Network,
        Capability::Internet,
        Capability::ProcessExec,
        Capability::KeyAccess,
        Capability::SensitiveOutput,
    ],
    non_responsibilities: &[
        "Converting class files to DEX; that is another step's work.",
        "Packaging or signing anything.",
        "Resolving dependencies. What it may call, it is handed.",
        "Verifying the class files it writes. That is the device's, and d8's.",
    ],
};

pub struct JavaCompiler;

pub static COMPILER: JavaCompiler = JavaCompiler;

pub struct JavaPlugin;

pub static PLUGIN: JavaPlugin = JavaPlugin;

impl crate::plugin::Plugin for JavaPlugin {
    fn contract(&self) -> &'static Contract {
        &CONTRACT
    }

    fn execute(
        &self,
        _ctx: &mut crate::plugin::Context<'_>,
    ) -> Result<crate::plugin::Outcome, Diagnostic> {
        Err(fail(
            "EJ300",
            "Java compiles through the compiler contract, not through this one.",
        )
        .with_suggestion(
            "The plugin surface has no request to compile and nowhere to put what comes \
             out. Use crate::compiler::Compiler, which has both.",
        ))
    }
}

impl JavaCompiler {
    fn classpath_from(
        &self,
        session: &Session<'_>,
        request: &Request,
    ) -> Result<Classpath, Diagnostic> {
        let mut classpath = Classpath::new();
        for digest in &request.dependencies {
            let bytes = session.store().get(*digest)?;
            if bytes.len() < 4 || bytes[..4] != 0xcafe_babeu32.to_be_bytes() {
                continue;
            }
            let class = crate::jvm::read(&bytes)?;
            classpath.learn(&class)?;
        }
        Ok(classpath)
    }
}

impl Compiler for JavaCompiler {
    fn contract(&self) -> &'static Contract {
        &CONTRACT
    }

    fn probe(&self) -> Probe {
        Probe::usable(
            Identity::new("java", "build.java", env!("CARGO_PKG_VERSION"), "jvm")
                .with("languageRelease", LANGUAGE_RELEASE)
                .with("classFileMajor", CLASS_MAJOR.to_string())
                .with("stackMapFrames", "none"),
            Reproducibility::Always,
        )
    }

    fn plan(&self, request: &Request) -> Result<Plan, Diagnostic> {
        if !crate::compiler::api_is_supported(request.api_level) {
            return Err(fail(
                "EJ301",
                "The chosen Android API is outside what this builds for.",
            )
            .with_context(format!("Chosen: API {}", request.api_level))
            .with_context(format!(
                "Supported: {} to {}",
                crate::compiler::OLDEST_API,
                crate::compiler::NEWEST_API
            )));
        }

        let identity = self
            .probe()
            .identity
            .expect("this compiler is always usable");
        let mut expected = Vec::new();
        for source in request.ordered_sources() {
            expected.push(Expected {
                name: source.path.clone(),
                kind: Kind::JvmClass,
            });
        }

        Ok(Plan {
            key: Plan::key_for(&identity, request),
            identity,
            expected,
            reproducibility: Reproducibility::Always,
        })
    }

    fn compile(
        &self,
        plan: &Plan,
        request: &Request,
        mut session: Session<'_>,
    ) -> Result<Compiled, Diagnostic> {
        let classpath = self.classpath_from(&session, request)?;
        let sources = request.ordered_sources();

        for source in &sources {
            session.carry_on()?;
            let bytes = session.store().get(source.digest)?;
            let text = String::from_utf8(bytes).map_err(|_| {
                fail("EJ302", "A Java source file is not text.")
                    .with_context(format!("Path: {}", source.path))
            })?;

            let produced = compile(&text, &classpath)
                .map_err(|error| error.with_context(format!("File: {}", source.path)))?;
            for (name, class) in produced {
                session.offer(name, Kind::JvmClass, class)?;
            }
        }

        session.commit(plan)
    }
}
