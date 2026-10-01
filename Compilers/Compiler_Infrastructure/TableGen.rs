use super::compiler_infrastructure::fail;
use crate::diag::Diagnostic;
use std::collections::{BTreeMap, BTreeSet};

pub const TABLEGEN_VERSION: &str = "omni-tablegen 1 (llvm-tblgen 24.0.0git semantics)";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Location {
    pub buffer: u32,
    pub offset: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MessageKind {
    Error,
    Warning,
    Note,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Message {
    pub kind: MessageKind,
    pub location: Option<Location>,
    pub text: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stop;

type Flow<T> = Result<T, Stop>;

pub struct SourceBuffer<'a> {
    pub name: String,
    pub text: &'a [u8],
    pub included_at: Option<Location>,
}

pub struct SourceManager<'a> {
    pub buffers: Vec<SourceBuffer<'a>>,
}

impl<'a> SourceManager<'a> {
    pub fn line_and_column(&self, location: Location) -> (usize, usize) {
        let Some(buffer) = self.buffers.get(location.buffer as usize) else {
            return (0, 0);
        };
        let offset = (location.offset as usize).min(buffer.text.len());
        let before = &buffer.text[..offset];
        let line = before.iter().filter(|&&byte| byte == b'\n').count() + 1;
        let line_start = before
            .iter()
            .rposition(|&byte| byte == b'\n')
            .map(|at| at + 1)
            .unwrap_or(0);
        (line, offset - line_start + 1)
    }

    pub fn render(&self, message: &Message) -> String {
        let kind = match message.kind {
            MessageKind::Error => "error",
            MessageKind::Warning => "warning",
            MessageKind::Note => "note",
        };
        let Some(location) = message.location else {
            return format!("{kind}: {}\n", message.text);
        };
        let Some(buffer) = self.buffers.get(location.buffer as usize) else {
            return format!("{kind}: {}\n", message.text);
        };
        let mut rendered = String::new();
        let mut trail = Vec::new();
        let mut parent = buffer.included_at;
        while let Some(include) = parent {
            let (line, _) = self.line_and_column(include);
            let name = &self.buffers[include.buffer as usize].name;
            trail.push(format!("Included from {name}:{line}:\n"));
            parent = self.buffers[include.buffer as usize].included_at;
        }
        for line in trail.iter().rev() {
            rendered.push_str(line);
        }
        let (line, column) = self.line_and_column(location);
        rendered.push_str(&format!(
            "{}:{line}:{column}: {kind}: {}\n",
            buffer.name, message.text
        ));
        let offset = (location.offset as usize).min(buffer.text.len());
        let start = buffer.text[..offset]
            .iter()
            .rposition(|&byte| byte == b'\n' || byte == b'\r')
            .map(|at| at + 1)
            .unwrap_or(0);
        let end = buffer.text[offset..]
            .iter()
            .position(|&byte| byte == b'\n' || byte == b'\r')
            .map(|at| offset + at)
            .unwrap_or(buffer.text.len());
        let source_line = String::from_utf8_lossy(&buffer.text[start..end]).into_owned();
        rendered.push_str(&source_line);
        rendered.push('\n');
        let mut caret = String::new();
        for (index, byte) in buffer.text[start..offset].iter().enumerate() {
            let _ = index;
            caret.push(if *byte == b'\t' { '\t' } else { ' ' });
        }
        caret.push('^');
        rendered.push_str(&caret);
        rendered.push('\n');
        rendered
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BangOperator {
    Concat,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    LogTwo,
    And,
    Or,
    ExclusiveOr,
    ShiftRightArithmetic,
    ShiftRightLogical,
    ShiftLeft,
    ListConcat,
    ListFlatten,
    ListSplat,
    StringConcat,
    Interleave,
    Substring,
    Find,
    Match,
    Cast,
    Substitute,
    ForEach,
    Filter,
    FoldLeft,
    Head,
    Tail,
    Size,
    Empty,
    Initialized,
    Instances,
    If,
    Condition,
    Switch,
    Equal,
    IsA,
    Dag,
    NotEqual,
    LessOrEqual,
    Less,
    GreaterOrEqual,
    Greater,
    SetDagOperator,
    GetDagOperator,
    SetDagOperatorName,
    GetDagOperatorName,
    Exists,
    ListRemove,
    ToLower,
    ToUpper,
    Range,
    Sort,
    GetDagArgument,
    GetDagName,
    SetDagArgument,
    SetDagName,
    Represent,
}

const BANG_OPERATORS: &[(&str, BangOperator)] = &[
    ("add", BangOperator::Add),
    ("and", BangOperator::And),
    ("cast", BangOperator::Cast),
    ("con", BangOperator::Concat),
    ("cond", BangOperator::Condition),
    ("dag", BangOperator::Dag),
    ("div", BangOperator::Divide),
    ("empty", BangOperator::Empty),
    ("eq", BangOperator::Equal),
    ("exists", BangOperator::Exists),
    ("filter", BangOperator::Filter),
    ("find", BangOperator::Find),
    ("foldl", BangOperator::FoldLeft),
    ("foreach", BangOperator::ForEach),
    ("ge", BangOperator::GreaterOrEqual),
    ("getdagarg", BangOperator::GetDagArgument),
    ("getdagname", BangOperator::GetDagName),
    ("getdagop", BangOperator::GetDagOperator),
    ("getdagopname", BangOperator::GetDagOperatorName),
    ("gt", BangOperator::Greater),
    ("head", BangOperator::Head),
    ("if", BangOperator::If),
    ("initialized", BangOperator::Initialized),
    ("instances", BangOperator::Instances),
    ("interleave", BangOperator::Interleave),
    ("isa", BangOperator::IsA),
    ("le", BangOperator::LessOrEqual),
    ("listconcat", BangOperator::ListConcat),
    ("listflatten", BangOperator::ListFlatten),
    ("listremove", BangOperator::ListRemove),
    ("listsplat", BangOperator::ListSplat),
    ("logtwo", BangOperator::LogTwo),
    ("lt", BangOperator::Less),
    ("match", BangOperator::Match),
    ("mul", BangOperator::Multiply),
    ("ne", BangOperator::NotEqual),
    ("not", BangOperator::Not),
    ("or", BangOperator::Or),
    ("range", BangOperator::Range),
    ("repr", BangOperator::Represent),
    ("setdagarg", BangOperator::SetDagArgument),
    ("setdagname", BangOperator::SetDagName),
    ("setdagop", BangOperator::SetDagOperator),
    ("setdagopname", BangOperator::SetDagOperatorName),
    ("shl", BangOperator::ShiftLeft),
    ("size", BangOperator::Size),
    ("sort", BangOperator::Sort),
    ("sra", BangOperator::ShiftRightArithmetic),
    ("srl", BangOperator::ShiftRightLogical),
    ("strconcat", BangOperator::StringConcat),
    ("sub", BangOperator::Subtract),
    ("subst", BangOperator::Substitute),
    ("substr", BangOperator::Substring),
    ("switch", BangOperator::Switch),
    ("tail", BangOperator::Tail),
    ("tolower", BangOperator::ToLower),
    ("toupper", BangOperator::ToUpper),
    ("xor", BangOperator::ExclusiveOr),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Token {
    End,
    Error,
    Minus,
    Plus,
    LeftSquare,
    RightSquare,
    LeftBrace,
    RightBrace,
    LeftParenthesis,
    RightParenthesis,
    Less,
    Greater,
    Colon,
    Semicolon,
    Comma,
    Dot,
    Equal,
    Question,
    Paste,
    Ellipsis,
    True,
    False,
    Integer,
    BinaryInteger,
    Ifdef,
    Ifndef,
    Else,
    Endif,
    Define,
    Bit,
    Bits,
    Code,
    Dag,
    ElseKeyword,
    Field,
    In,
    Include,
    Int,
    List,
    String,
    Then,
    Assert,
    Class,
    Def,
    Defm,
    Defset,
    Deftype,
    Defvar,
    Dump,
    Foreach,
    If,
    Let,
    MultiClass,
    Bang(BangOperator),
    Identifier,
    StringValue,
    VariableName,
    CodeFragment,
}

impl Token {
    fn starts_object(self) -> bool {
        matches!(
            self,
            Token::Assert
                | Token::Class
                | Token::Def
                | Token::Defm
                | Token::Defset
                | Token::Deftype
                | Token::Defvar
                | Token::Dump
                | Token::Foreach
                | Token::If
                | Token::Let
                | Token::MultiClass
        )
    }
}

const KEYWORDS: &[(&str, Token)] = &[
    ("assert", Token::Assert),
    ("bit", Token::Bit),
    ("bits", Token::Bits),
    ("class", Token::Class),
    ("code", Token::Code),
    ("dag", Token::Dag),
    ("def", Token::Def),
    ("defm", Token::Defm),
    ("defset", Token::Defset),
    ("deftype", Token::Deftype),
    ("defvar", Token::Defvar),
    ("dump", Token::Dump),
    ("else", Token::ElseKeyword),
    ("false", Token::False),
    ("field", Token::Field),
    ("foreach", Token::Foreach),
    ("if", Token::If),
    ("in", Token::In),
    ("include", Token::Include),
    ("int", Token::Int),
    ("let", Token::Let),
    ("list", Token::List),
    ("multiclass", Token::MultiClass),
    ("string", Token::String),
    ("then", Token::Then),
    ("true", Token::True),
];

const DIRECTIVES: &[(&str, Token)] = &[
    ("ifdef", Token::Ifdef),
    ("ifndef", Token::Ifndef),
    ("else", Token::Else),
    ("endif", Token::Endif),
    ("define", Token::Define),
];

fn identifier_character(character: u8, first: bool) -> bool {
    if character == b'_' || character.is_ascii_alphabetic() {
        return true;
    }
    !first && character.is_ascii_digit()
}

fn macro_name_length(text: &[u8]) -> usize {
    if text.is_empty() || !identifier_character(text[0], true) {
        return 0;
    }
    let mut length = 1;
    while length < text.len() && identifier_character(text[length], false) {
        length += 1;
    }
    length
}

pub fn join_path(directory: &str, file: &str) -> String {
    if directory.is_empty() {
        return file.to_string();
    }
    if directory.ends_with('/') {
        format!("{directory}{file}")
    } else {
        format!("{directory}/{file}")
    }
}

pub fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    let joined = parts.join("/");
    if path.starts_with('/') {
        format!("/{joined}")
    } else {
        joined
    }
}

#[derive(Clone, Copy)]
struct PreprocessorControl {
    kind: Token,
    defined: bool,
    location: Location,
}

const END_OF_FILE: i32 = -1;

struct Lexer<'a> {
    files: &'a BTreeMap<String, String>,
    include_directories: Vec<String>,
    sources: SourceManager<'a>,
    buffer: usize,
    at: usize,
    token_start: usize,
    token: Token,
    text: String,
    integer: i64,
    dependencies: BTreeSet<String>,
    macros: BTreeSet<String>,
    controls: Vec<Vec<PreprocessorControl>>,
    messages: Vec<Message>,
}

impl<'a> Lexer<'a> {
    fn new(
        files: &'a BTreeMap<String, String>,
        main: &str,
        include_directories: &[String],
        macros: &[String],
    ) -> Option<Lexer<'a>> {
        let text = Lexer::lookup(files, main)?;
        let mut defined = BTreeSet::new();
        for name in macros {
            defined.insert(name.clone());
        }
        Some(Lexer {
            files,
            include_directories: include_directories.to_vec(),
            sources: SourceManager {
                buffers: vec![SourceBuffer {
                    name: main.to_string(),
                    text: text.as_bytes(),
                    included_at: None,
                }],
            },
            buffer: 0,
            at: 0,
            token_start: 0,
            token: Token::End,
            text: String::new(),
            integer: 0,
            dependencies: BTreeSet::new(),
            macros: defined,
            controls: vec![Vec::new()],
            messages: Vec::new(),
        })
    }

    fn lookup<'b>(files: &'b BTreeMap<String, String>, path: &str) -> Option<&'b String> {
        files.get(path).or_else(|| files.get(&normalize_path(path)))
    }

    fn bytes(&self) -> &'a [u8] {
        self.sources.buffers[self.buffer].text
    }

    fn byte(&self, index: usize) -> u8 {
        self.bytes().get(index).copied().unwrap_or(0)
    }

    fn signed_byte(&self, index: usize) -> i32 {
        self.byte(index) as i8 as i32
    }

    fn end(&self) -> usize {
        self.bytes().len()
    }

    fn location_at(&self, offset: usize) -> Location {
        Location {
            buffer: self.buffer as u32,
            offset: offset as u32,
        }
    }

    fn location(&self) -> Location {
        self.location_at(self.token_start)
    }

    fn report(&mut self, kind: MessageKind, location: Location, text: impl Into<String>) {
        self.messages.push(Message {
            kind,
            location: Some(location),
            text: text.into(),
        });
    }

    fn error_at(&mut self, offset: usize, text: impl Into<String>) -> Token {
        let location = self.location_at(offset);
        self.report(MessageKind::Error, location, text);
        Token::Error
    }

    fn lex(&mut self) -> Token {
        let start = self.at == 0;
        self.token = self.lex_token(start);
        self.token
    }

    fn process_end_of_file(&mut self) -> bool {
        if let Some(parent) = self.sources.buffers[self.buffer].included_at {
            if !self.exit_include(false) {
                return false;
            }
            self.buffer = parent.buffer as usize;
            self.at = parent.offset as usize;
            self.token_start = self.at;
            return true;
        }
        self.exit_include(true);
        false
    }

    fn next_character(&mut self) -> i32 {
        let current = self.byte(self.at);
        self.at += 1;
        match current {
            0 => {
                if self.at > self.end() {
                    self.at -= 1;
                    return END_OF_FILE;
                }
                let location = self.location();
                self.report(
                    MessageKind::Error,
                    location,
                    "NUL character is invalid in source; treated as space",
                );
                b' ' as i32
            }
            b'\n' | b'\r' => {
                let next = self.byte(self.at);
                if (next == b'\n' || next == b'\r') && next != current {
                    self.at += 1;
                }
                b'\n' as i32
            }
            other => other as i32,
        }
    }

    fn lex_token(&mut self, mut line_start: bool) -> Token {
        loop {
            self.token_start = self.at;
            let current = self.next_character();
            if current == END_OF_FILE {
                if self.process_end_of_file() {
                    line_start = false;
                    continue;
                }
                return Token::End;
            }
            let character = current as u8;
            match character {
                b':' => return Token::Colon,
                b';' => return Token::Semicolon,
                b',' => return Token::Comma,
                b'<' => return Token::Less,
                b'>' => return Token::Greater,
                b']' => return Token::RightSquare,
                b'{' => return Token::LeftBrace,
                b'}' => return Token::RightBrace,
                b'(' => return Token::LeftParenthesis,
                b')' => return Token::RightParenthesis,
                b'=' => return Token::Equal,
                b'?' => return Token::Question,
                b'#' => {
                    if line_start {
                        let kind = self.directive_kind();
                        if kind != Token::Error {
                            return self.preprocess(kind, true);
                        }
                    }
                    return Token::Paste;
                }
                b'.' => {
                    if self.byte(self.at) == b'.' {
                        self.at += 1;
                        if self.byte(self.at) == b'.' {
                            self.at += 1;
                            return Token::Ellipsis;
                        }
                        return self.error_at(self.token_start, "invalid '..' punctuation");
                    }
                    return Token::Dot;
                }
                b' ' | b'\t' => {}
                b'\n' => line_start = true,
                b'/' => {
                    if self.byte(self.at) == b'/' {
                        self.skip_line_comment();
                    } else if self.byte(self.at) == b'*' {
                        if self.skip_block_comment() {
                            return Token::Error;
                        }
                    } else {
                        return self.error_at(self.token_start, "unexpected character");
                    }
                }
                b'-' | b'+' | b'0'..=b'9' => {
                    let mut next = 0u8;
                    if character.is_ascii_digit() {
                        let mut index = 0;
                        loop {
                            next = self.byte(self.at + index);
                            index += 1;
                            if !next.is_ascii_digit() {
                                break;
                            }
                        }
                        if next == b'x' || next == b'b' {
                            let after = self.byte(self.at + index);
                            let binary_digit = after == b'0' || after == b'1';
                            if binary_digit && next == b'b' {
                                return self.lex_number();
                            }
                            if after.is_ascii_hexdigit() && next == b'x' {
                                return self.lex_number();
                            }
                        }
                    }
                    if identifier_character(next, true) {
                        return self.lex_identifier();
                    }
                    return self.lex_number();
                }
                b'"' => return self.lex_string(),
                b'$' => return self.lex_variable_name(),
                b'[' => return self.lex_bracket(),
                b'!' => return self.lex_exclaim(),
                other => {
                    if identifier_character(other, true) {
                        return self.lex_identifier();
                    }
                    return self.error_at(self.token_start, "unexpected character");
                }
            }
        }
    }

    fn lex_string(&mut self) -> Token {
        let start = self.at;
        let mut value: Vec<u8> = Vec::new();
        loop {
            let current = self.byte(self.at);
            if current == b'"' {
                break;
            }
            if current == 0 && self.at >= self.end() {
                return self.error_at(start, "end of file in string literal");
            }
            if current == b'\n' || current == b'\r' {
                return self.error_at(start, "end of line in string literal");
            }
            if current != b'\\' {
                value.push(current);
                self.at += 1;
                continue;
            }
            self.at += 1;
            match self.byte(self.at) {
                escaped @ (b'\\' | b'\'' | b'"') => {
                    value.push(escaped);
                    self.at += 1;
                }
                b't' => {
                    value.push(b'\t');
                    self.at += 1;
                }
                b'n' => {
                    value.push(b'\n');
                    self.at += 1;
                }
                b'\n' | b'\r' => {
                    return self.error_at(self.at, "escaped newlines not supported in tblgen");
                }
                0 if self.at >= self.end() => {
                    return self.error_at(start, "end of file in string literal");
                }
                _ => return self.error_at(self.at, "invalid escape in string literal"),
            }
        }
        self.at += 1;
        self.text = String::from_utf8_lossy(&value).into_owned();
        Token::StringValue
    }

    fn lex_variable_name(&mut self) -> Token {
        if !identifier_character(self.byte(self.at), true) {
            return self.error_at(self.token_start, "invalid variable name");
        }
        let start = self.at;
        self.at += 1;
        while identifier_character(self.byte(self.at), false) {
            self.at += 1;
        }
        self.text = String::from_utf8_lossy(&self.bytes()[start..self.at]).into_owned();
        Token::VariableName
    }

    fn lex_identifier(&mut self) -> Token {
        let start = self.token_start;
        while identifier_character(self.byte(self.at), false) {
            self.at += 1;
        }
        let word = &self.bytes()[start..self.at];
        let kind = KEYWORDS
            .iter()
            .find(|(keyword, _)| keyword.as_bytes() == word)
            .map(|(_, token)| *token)
            .unwrap_or(Token::Identifier);
        match kind {
            Token::Include => {
                if self.lex_include() {
                    return Token::Error;
                }
                self.lex()
            }
            Token::Identifier => {
                self.text = String::from_utf8_lossy(word).into_owned();
                Token::Identifier
            }
            other => other,
        }
    }

    fn lex_include(&mut self) -> bool {
        let token = self.lex_token(false);
        if token == Token::Error {
            return true;
        }
        if token != Token::StringValue {
            let location = self.location();
            self.report(
                MessageKind::Error,
                location,
                "expected filename after include",
            );
            return true;
        }
        let file = self.text.clone();
        let mut found = None;
        if let Some(text) = Lexer::lookup(self.files, &file) {
            found = Some((file.clone(), text));
        }
        if found.is_none() {
            for directory in &self.include_directories {
                let candidate = join_path(directory, &file);
                if let Some(text) = Lexer::lookup(self.files, &candidate) {
                    found = Some((candidate, text));
                    break;
                }
            }
        }
        let Some((name, text)) = found else {
            let location = self.location();
            self.report(
                MessageKind::Error,
                location,
                format!("could not find include file '{file}'"),
            );
            return true;
        };
        let included_at = self.location_at(self.at);
        self.dependencies.insert(name.clone());
        self.sources.buffers.push(SourceBuffer {
            name,
            text: text.as_bytes(),
            included_at: Some(included_at),
        });
        self.buffer = self.sources.buffers.len() - 1;
        self.at = 0;
        self.controls.push(Vec::new());
        false
    }

    fn skip_line_comment(&mut self) {
        self.at += 1;
        let bytes = self.bytes();
        self.at = bytes[self.at.min(bytes.len())..]
            .iter()
            .position(|&byte| byte == b'\n' || byte == b'\r')
            .map(|offset| self.at + offset)
            .unwrap_or(bytes.len());
    }

    fn skip_block_comment(&mut self) -> bool {
        self.at += 1;
        let mut depth = 1u32;
        loop {
            let current = self.next_character();
            if current == END_OF_FILE {
                let start = self.token_start;
                self.error_at(start, "unterminated comment");
                return true;
            }
            if current == b'*' as i32 {
                if self.byte(self.at) != b'/' {
                    continue;
                }
                self.at += 1;
                depth -= 1;
                if depth == 0 {
                    return false;
                }
            } else if current == b'/' as i32 {
                if self.byte(self.at) != b'*' {
                    continue;
                }
                self.at += 1;
                depth += 1;
            }
        }
    }

    fn lex_number(&mut self) -> Token {
        let mut base = 0u32;
        let mut digits_start = self.at;
        if self.at >= 1 && self.byte(self.at - 1) == b'0' {
            digits_start = self.at + 1;
            if self.byte(self.at) == b'x' {
                base = 16;
                loop {
                    self.at += 1;
                    if !self.byte(self.at).is_ascii_hexdigit() {
                        break;
                    }
                }
            } else if self.byte(self.at) == b'b' {
                base = 2;
                loop {
                    self.at += 1;
                    let current = self.byte(self.at);
                    if current != b'0' && current != b'1' {
                        break;
                    }
                }
            }
        }
        let mut negative = false;
        if base == 0 {
            let previous = self.byte(self.at - 1);
            if !self.byte(self.at).is_ascii_digit() {
                if previous == b'-' {
                    return Token::Minus;
                } else if previous == b'+' {
                    return Token::Plus;
                }
            }
            base = 10;
            digits_start = self.token_start;
            negative = previous == b'-';
            while self.byte(self.at).is_ascii_digit() {
                self.at += 1;
            }
        }
        if self.at == digits_start {
            return self.error_at(self.token_start, "invalid number");
        }
        let text = &self.bytes()[digits_start..self.at];
        let (sign, digits) = match text.first() {
            Some(b'-') => (true, &text[1..]),
            Some(b'+') => (false, &text[1..]),
            _ => (false, text),
        };
        let mut magnitude: u64 = 0;
        let mut overflow = false;
        for &digit in digits {
            let value = (digit as char).to_digit(base).unwrap_or(0) as u64;
            match magnitude
                .checked_mul(base as u64)
                .and_then(|scaled| scaled.checked_add(value))
            {
                Some(next) => magnitude = next,
                None => overflow = true,
            }
        }
        if negative {
            if overflow || magnitude > (i64::MAX as u64) + 1 {
                return self.error_at(self.token_start, "number out of range");
            }
            self.integer = (magnitude as i64).wrapping_neg();
        } else {
            if overflow {
                return self.error_at(self.token_start, "number out of range");
            }
            let _ = sign;
            self.integer = magnitude as i64;
        }
        if base == 2 {
            Token::BinaryInteger
        } else {
            Token::Integer
        }
    }

    fn binary_width(&self) -> usize {
        self.at - self.token_start - 2
    }

    fn lex_bracket(&mut self) -> Token {
        if self.byte(self.at) != b'{' {
            return Token::LeftSquare;
        }
        self.at += 1;
        let code_start = self.at;
        loop {
            let mut current = self.next_character();
            if current == END_OF_FILE {
                break;
            }
            if current != b'}' as i32 {
                continue;
            }
            current = self.next_character();
            if current == END_OF_FILE {
                break;
            }
            if current == b']' as i32 {
                self.text =
                    String::from_utf8_lossy(&self.bytes()[code_start..self.at - 2]).into_owned();
                return Token::CodeFragment;
            }
        }
        self.error_at(code_start - 2, "unterminated code block")
    }

    fn lex_exclaim(&mut self) -> Token {
        if !self.byte(self.at).is_ascii_alphabetic() {
            return self.error_at(self.at - 1, "invalid \"!operator\"");
        }
        let start = self.at;
        self.at += 1;
        while self.byte(self.at).is_ascii_alphabetic() {
            self.at += 1;
        }
        let word = &self.bytes()[start..self.at];
        match BANG_OPERATORS
            .iter()
            .find(|(name, _)| name.as_bytes() == word)
        {
            Some((_, operator)) => Token::Bang(*operator),
            None => self.error_at(start - 1, "unknown operator"),
        }
    }

    fn exit_include(&mut self, must_be_empty: bool) -> bool {
        if self.controls.last().is_some_and(|top| !top.is_empty()) {
            self.report_control_stack_error();
            return false;
        }
        self.controls.pop();
        let _ = must_be_empty;
        true
    }

    fn directive_kind(&self) -> Token {
        for (word, kind) in DIRECTIVES {
            let length = word.len();
            let matches =
                (0..length).all(|index| self.byte(self.at + index) == word.as_bytes()[index]);
            if !matches {
                continue;
            }
            let next = self.signed_byte(self.at + length);
            if next == b' ' as i32
                || next == b'\t' as i32
                || next == END_OF_FILE
                || next == b'\n' as i32
                || next == b'\r' as i32
            {
                return *kind;
            }
            if next == b'/' as i32 {
                let after = self.byte(self.at + length + 1);
                if after == b'*' || after == b'/' {
                    return *kind;
                }
            }
        }
        Token::Error
    }

    fn eat_directive(&mut self, kind: Token) {
        self.token_start = self.at;
        for (word, directive) in DIRECTIVES {
            if *directive == kind {
                self.at += word.len();
                return;
            }
        }
    }

    fn preprocess(&mut self, kind: Token, return_next_live_token: bool) -> Token {
        self.eat_directive(kind);
        match kind {
            Token::Ifdef | Token::Ifndef => {
                let name = self.macro_name();
                let directive = if kind == Token::Ifdef {
                    "#ifdef"
                } else {
                    "#ifndef"
                };
                if name.is_empty() {
                    return self.error_at(
                        self.token_start,
                        format!("expected macro name after {directive}"),
                    );
                }
                let mut defined = self.macros.contains(&name);
                if kind == Token::Ifndef {
                    defined = !defined;
                }
                let location = self.location();
                if let Some(top) = self.controls.last_mut() {
                    top.push(PreprocessorControl {
                        kind: Token::Ifdef,
                        defined,
                        location,
                    });
                }
                if !self.skip_directive_end() {
                    return self.error_at(
                        self.at,
                        format!("only comments are supported after {directive} NAME"),
                    );
                }
                if !return_next_live_token {
                    return kind;
                }
                if defined {
                    return self.lex_token(false);
                }
                if self.skip_region() {
                    return self.lex_token(false);
                }
                Token::Error
            }
            Token::Else => {
                let Some(entry) = self.controls.last().and_then(|top| top.last()).copied() else {
                    return self.error_at(self.token_start, "#else without #ifdef or #ifndef");
                };
                if entry.kind != Token::Ifdef {
                    let start = self.token_start;
                    self.error_at(start, "double #else");
                    self.report(MessageKind::Error, entry.location, "previous #else is here");
                    return Token::Error;
                }
                let location = self.location();
                if let Some(top) = self.controls.last_mut() {
                    if let Some(last) = top.last_mut() {
                        *last = PreprocessorControl {
                            kind,
                            defined: !entry.defined,
                            location,
                        };
                    }
                }
                if !self.skip_directive_end() {
                    return self.error_at(self.at, "only comments are supported after #else");
                }
                if return_next_live_token {
                    if self.skip_region() {
                        return self.lex_token(false);
                    }
                    return Token::Error;
                }
                kind
            }
            Token::Endif => {
                if self.controls.last().is_none_or(|top| top.is_empty()) {
                    return self.error_at(self.token_start, "#endif without #ifdef");
                }
                if !self.skip_directive_end() {
                    return self.error_at(self.at, "only comments are supported after #endif");
                }
                if let Some(top) = self.controls.last_mut() {
                    top.pop();
                }
                if return_next_live_token {
                    return self.lex_token(false);
                }
                kind
            }
            _ => {
                let name = self.macro_name();
                if name.is_empty() {
                    return self.error_at(self.token_start, "expected macro name after #define");
                }
                if !self.macros.insert(name.clone()) {
                    let location = self.location();
                    self.report(
                        MessageKind::Warning,
                        location,
                        format!("duplicate definition of macro: {name}"),
                    );
                }
                if !self.skip_directive_end() {
                    return self
                        .error_at(self.at, "only comments are supported after #define NAME");
                }
                self.lex_token(false)
            }
        }
    }

    fn skip_region(&mut self) -> bool {
        loop {
            while self.at < self.end() && self.byte(self.at) != b'\n' && self.byte(self.at) != b'\r'
            {
                self.at += 1;
            }
            if !self.skip_line_begin() {
                return false;
            }
            if self.byte(self.at) == b'#' && self.at < self.end() {
                self.at += 1;
                let kind = self.directive_kind();
                if kind != Token::Error && kind != Token::Define {
                    let processed = self.preprocess(kind, false);
                    if processed == Token::Error {
                        return false;
                    }
                    if self.processing_enabled() {
                        return true;
                    }
                }
            }
            if self.at >= self.end() {
                break;
            }
        }
        self.report_control_stack_error();
        false
    }

    fn macro_name(&mut self) -> String {
        while self.byte(self.at) == b' ' || self.byte(self.at) == b'\t' {
            self.at += 1;
        }
        self.token_start = self.at;
        let rest = &self.bytes()[self.at.min(self.end())..];
        let length = macro_name_length(rest);
        self.at += length;
        String::from_utf8_lossy(&rest[..length]).into_owned()
    }

    fn skip_line_begin(&mut self) -> bool {
        while self.at < self.end() {
            match self.byte(self.at) {
                b' ' | b'\t' | b'\n' | b'\r' => {}
                b'/' => {
                    if self.byte(self.at + 1) == b'*' {
                        self.token_start = self.at;
                        self.at += 1;
                        if self.skip_block_comment() {
                            return false;
                        }
                    } else {
                        return true;
                    }
                    continue;
                }
                _ => return true,
            }
            self.at += 1;
        }
        true
    }

    fn skip_directive_end(&mut self) -> bool {
        while self.at < self.end() {
            match self.byte(self.at) {
                b' ' | b'\t' => {}
                b'\n' | b'\r' => return true,
                b'/' => {
                    let next = self.byte(self.at + 1);
                    if next == b'/' {
                        self.at += 1;
                        self.skip_line_comment();
                    } else if next == b'*' {
                        self.token_start = self.at;
                        self.at += 1;
                        if self.skip_block_comment() {
                            return false;
                        }
                    } else {
                        self.token_start = self.at;
                        let at = self.at;
                        self.error_at(at, "unexpected character");
                        return false;
                    }
                    continue;
                }
                _ => {
                    self.token_start = self.at;
                    return false;
                }
            }
            self.at += 1;
        }
        true
    }

    fn processing_enabled(&self) -> bool {
        self.controls
            .last()
            .is_none_or(|top| top.iter().all(|control| control.defined))
    }

    fn report_control_stack_error(&mut self) {
        let control = self.controls.last().and_then(|top| top.last()).copied();
        let end = self.end();
        self.error_at(end, "reached EOF without matching #endif");
        if let Some(control) = control {
            self.report(
                MessageKind::Error,
                control.location,
                "the latest preprocessor control is here",
            );
        }
        self.token_start = self.at;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Type(u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Value(u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct RecordIndex(u32);

impl Type {
    const NONE: Type = Type(u32::MAX);
}

impl Value {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl RecordIndex {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TypeShape {
    Bit,
    Bits(u32),
    Integer,
    String,
    List(Type),
    Dag,
    Record(Vec<RecordIndex>),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum UnaryOperator {
    ToLower,
    ToUpper,
    Cast,
    Not,
    Head,
    Tail,
    Size,
    Empty,
    GetDagOperator,
    GetDagOperatorName,
    LogTwo,
    Represent,
    ListFlatten,
    Initialized,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    And,
    Or,
    ExclusiveOr,
    ShiftLeft,
    ShiftRightArithmetic,
    ShiftRightLogical,
    ListConcat,
    ListSplat,
    ListRemove,
    ListElement,
    ListSlice,
    RangeClosed,
    StringConcat,
    Interleave,
    Concat,
    Match,
    Equal,
    NotEqual,
    LessOrEqual,
    Less,
    GreaterOrEqual,
    Greater,
    GetDagArgument,
    GetDagName,
    SetDagOperator,
    SetDagOperatorName,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TernaryOperator {
    Substitute,
    ForEach,
    Filter,
    If,
    Dag,
    Range,
    Substring,
    Find,
    SetDagArgument,
    SetDagName,
    Sort,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Node {
    Unset,
    Argument {
        value: Value,
        name: Option<Value>,
        position: u32,
    },
    Bit(bool),
    Bits(Box<[Value]>),
    Integer(i64),
    AnonymousName(u32),
    String {
        text: u32,
        code: bool,
    },
    List {
        elements: Box<[Value]>,
        element: Type,
    },
    Unary {
        operator: UnaryOperator,
        operand: Value,
        result: Type,
    },
    Binary {
        operator: BinaryOperator,
        left: Value,
        right: Value,
        result: Type,
    },
    Ternary {
        operator: TernaryOperator,
        left: Value,
        middle: Value,
        right: Value,
        result: Type,
    },
    Condition {
        conditions: Box<[Value]>,
        values: Box<[Value]>,
        result: Type,
    },
    Fold {
        start: Value,
        list: Value,
        accumulator: Value,
        element: Value,
        expression: Value,
        result: Type,
    },
    IsA {
        checked: Type,
        expression: Value,
    },
    Exists {
        checked: Type,
        expression: Value,
    },
    Instances {
        checked: Type,
        pattern: Value,
    },
    Variable {
        name: Value,
        variable_type: Type,
    },
    VariableBit {
        variable: Value,
        bit: u32,
    },
    Definition(RecordIndex),
    VariableDefinition {
        class: RecordIndex,
        arguments: Box<[Value]>,
    },
    Field {
        record: Value,
        name: Value,
    },
    Dag {
        operator: Value,
        name: Option<Value>,
        arguments: Box<[Value]>,
        names: Box<[Option<Value>]>,
    },
}

const HASH_MULTIPLIER: u64 = 0x517c_c1b7_2722_0a95;

fn mix(hash: u64, word: u64) -> u64 {
    (hash.rotate_left(5) ^ word).wrapping_mul(HASH_MULTIPLIER)
}

fn mix_values(mut hash: u64, values: &[Value]) -> u64 {
    hash = mix(hash, values.len() as u64);
    for value in values {
        hash = mix(hash, value.0 as u64);
    }
    hash
}

fn optional(value: Option<Value>) -> u64 {
    value.map(|value| value.0 as u64).unwrap_or(u64::MAX)
}

fn node_hash(node: &Node) -> u64 {
    match node {
        Node::Unset => mix(1, 0),
        Node::Argument {
            value,
            name,
            position,
        } => mix(
            mix(mix(2, value.0 as u64), optional(*name)),
            *position as u64,
        ),
        Node::Bit(bit) => mix(3, *bit as u64),
        Node::Bits(bits) => mix_values(4, bits),
        Node::Integer(integer) => mix(5, *integer as u64),
        Node::AnonymousName(number) => mix(6, *number as u64),
        Node::String { text, code } => mix(mix(7, *text as u64), *code as u64),
        Node::List { elements, element } => mix(mix_values(8, elements), element.0 as u64),
        Node::Unary {
            operator,
            operand,
            result,
        } => mix(
            mix(mix(9, *operator as u64), operand.0 as u64),
            result.0 as u64,
        ),
        Node::Binary {
            operator,
            left,
            right,
            result,
        } => mix(
            mix(
                mix(mix(10, *operator as u64), left.0 as u64),
                right.0 as u64,
            ),
            result.0 as u64,
        ),
        Node::Ternary {
            operator,
            left,
            middle,
            right,
            result,
        } => mix(
            mix(
                mix(
                    mix(mix(11, *operator as u64), left.0 as u64),
                    middle.0 as u64,
                ),
                right.0 as u64,
            ),
            result.0 as u64,
        ),
        Node::Condition {
            conditions,
            values,
            result,
        } => mix(
            mix_values(mix_values(12, conditions), values),
            result.0 as u64,
        ),
        Node::Fold {
            start,
            list,
            accumulator,
            element,
            expression,
            result,
        } => mix_values(
            mix(13, result.0 as u64),
            &[*start, *list, *accumulator, *element, *expression],
        ),
        Node::IsA {
            checked,
            expression,
        } => mix(mix(14, checked.0 as u64), expression.0 as u64),
        Node::Exists {
            checked,
            expression,
        } => mix(mix(15, checked.0 as u64), expression.0 as u64),
        Node::Instances { checked, pattern } => mix(mix(16, checked.0 as u64), pattern.0 as u64),
        Node::Variable {
            name,
            variable_type,
        } => mix(mix(17, name.0 as u64), variable_type.0 as u64),
        Node::VariableBit { variable, bit } => mix(mix(18, variable.0 as u64), *bit as u64),
        Node::Definition(record) => mix(19, record.0 as u64),
        Node::VariableDefinition { class, arguments } => {
            mix_values(mix(20, class.0 as u64), arguments)
        }
        Node::Field { record, name } => mix(mix(21, record.0 as u64), name.0 as u64),
        Node::Dag {
            operator,
            name,
            arguments,
            names,
        } => {
            let mut hash = mix_values(mix(mix(22, operator.0 as u64), optional(*name)), arguments);
            for name in names.iter() {
                hash = mix(hash, optional(*name));
            }
            hash
        }
    }
}

fn text_hash(text: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in text.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[derive(Default)]
struct Interner {
    slots: Vec<(u64, u32)>,
    count: usize,
}

impl Interner {
    fn find(&self, hash: u64, mut same: impl FnMut(u32) -> bool) -> Result<u32, usize> {
        if self.slots.is_empty() {
            return Err(usize::MAX);
        }
        let mask = self.slots.len() - 1;
        let mut at = (hash as usize) & mask;
        loop {
            let (held_hash, held) = self.slots[at];
            if held == 0 {
                return Err(at);
            }
            if held_hash == hash && same(held - 1) {
                return Ok(held - 1);
            }
            at = (at + 1) & mask;
        }
    }

    fn insert(&mut self, hash: u64, id: u32, slot: usize, hashes: impl Fn(u32) -> u64) {
        if self.slots.is_empty() || (self.count + 1) * 4 > self.slots.len() * 3 {
            let size = (self.slots.len() * 2).max(1024);
            let old = std::mem::replace(&mut self.slots, vec![(0, 0); size]);
            let mask = size - 1;
            for (held_hash, held) in old {
                if held == 0 {
                    continue;
                }
                let _ = &hashes;
                let mut at = (held_hash as usize) & mask;
                while self.slots[at].1 != 0 {
                    at = (at + 1) & mask;
                }
                self.slots[at] = (held_hash, held);
            }
            let mut at = (hash as usize) & mask;
            while self.slots[at].1 != 0 {
                at = (at + 1) & mask;
            }
            self.slots[at] = (hash, id + 1);
        } else {
            self.slots[slot] = (hash, id + 1);
        }
        self.count += 1;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FieldKind {
    Normal,
    NonconcreteAllowed,
    TemplateArgument,
}

#[derive(Clone, Debug)]
pub struct RecordField {
    pub name: Value,
    pub location: Location,
    pub field_type: Type,
    pub kind: FieldKind,
    pub value: Value,
    pub used: bool,
}

#[derive(Clone, Debug)]
pub struct Assertion {
    pub location: Location,
    pub condition: Value,
    pub message: Value,
}

#[derive(Clone, Debug)]
pub struct DumpRequest {
    pub location: Location,
    pub message: Value,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RecordKind {
    Definition,
    AnonymousDefinition,
    Class,
    MultiClass,
}

#[derive(Clone, Debug)]
pub struct Record {
    pub name: Value,
    pub locations: Vec<Location>,
    pub template_arguments: Vec<Value>,
    pub fields: Vec<RecordField>,
    pub assertions: Vec<Assertion>,
    pub dumps: Vec<DumpRequest>,
    pub superclasses: Vec<(RecordIndex, Location)>,
    pub definition: Option<Value>,
    pub identifier: u32,
    pub kind: RecordKind,
}

impl Record {
    pub fn field(&self, name: Value) -> Option<usize> {
        self.fields.iter().position(|field| field.name == name)
    }

    pub fn is_template_argument(&self, name: Value) -> bool {
        self.template_arguments.contains(&name)
    }
}

struct VariableDefinitionState {
    location: Location,
    definition: Option<Value>,
}

pub struct RecordKeeper {
    type_shapes: Vec<TypeShape>,
    type_lookup: BTreeMap<TypeShape, Type>,
    nodes: Vec<Node>,
    node_types: Vec<Type>,
    interner: Interner,
    texts: Vec<Box<str>>,
    text_interner: Interner,
    records: Vec<Option<Record>>,
    free_records: Vec<u32>,
    pub classes: BTreeMap<String, RecordIndex>,
    pub definitions: BTreeMap<String, RecordIndex>,
    pub globals: BTreeMap<String, Value>,
    derived_cache: BTreeMap<String, Vec<RecordIndex>>,
    anonymous_counter: u32,
    record_counter: u32,
    variable_definitions: BTreeMap<Value, VariableDefinitionState>,
    pub messages: Vec<Message>,
    pub errors: usize,
    pub stopped: bool,
    bit_type: Type,
    integer_type: Type,
    string_type: Type,
    dag_type: Type,
    any_record_type: Type,
    unset: Value,
    true_bit: Value,
    false_bit: Value,
}

pub trait Resolver {
    fn resolve(&mut self, keeper: &mut RecordKeeper, name: Value) -> Flow<Option<Value>>;
    fn current_record(&self) -> Option<RecordIndex>;
    fn is_final(&self) -> bool;
    fn keep_unset_bits(&self) -> bool {
        false
    }
}

pub struct MapResolver {
    record: Option<RecordIndex>,
    entries: Vec<(Value, Value, bool)>,
    finished: bool,
}

impl MapResolver {
    pub fn new(record: Option<RecordIndex>) -> MapResolver {
        MapResolver {
            record,
            entries: Vec::new(),
            finished: false,
        }
    }

    pub fn set(&mut self, key: Value, value: Value) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.0 == key) {
            entry.1 = value;
            entry.2 = false;
        } else {
            self.entries.push((key, value, false));
        }
    }

    pub fn set_final(&mut self, finished: bool) {
        self.finished = finished;
    }
}

impl Resolver for MapResolver {
    fn resolve(&mut self, keeper: &mut RecordKeeper, name: Value) -> Flow<Option<Value>> {
        let Some(at) = self.entries.iter().position(|entry| entry.0 == name) else {
            return Ok(None);
        };
        let (_, mut value, resolved) = self.entries[at];
        if !resolved && self.entries.len() > 1 {
            self.entries.remove(at);
            value = keeper.resolve(value, self)?;
            self.entries.push((name, value, true));
        }
        Ok(Some(value))
    }

    fn current_record(&self) -> Option<RecordIndex> {
        self.record
    }

    fn is_final(&self) -> bool {
        self.finished
    }
}

pub struct RecordResolver {
    record: RecordIndex,
    cache: BTreeMap<Value, Value>,
    stack: Vec<Value>,
    name: Option<Value>,
}

impl RecordResolver {
    pub fn new(record: RecordIndex) -> RecordResolver {
        RecordResolver {
            record,
            cache: BTreeMap::new(),
            stack: Vec::new(),
            name: None,
        }
    }
}

impl Resolver for RecordResolver {
    fn resolve(&mut self, keeper: &mut RecordKeeper, name: Value) -> Flow<Option<Value>> {
        if let Some(found) = self.cache.get(&name) {
            return Ok(Some(*found));
        }
        if self.stack.contains(&name) {
            return Ok(None);
        }
        let record = keeper.record(self.record);
        let mut found = None;
        if let Some(at) = record.field(name) {
            let value = record.fields[at].value;
            if !keeper.is_unset(value) {
                self.stack.push(name);
                let resolved = keeper.resolve(value, self)?;
                self.stack.pop();
                found = Some(resolved);
            }
        } else if let Some(new_name) = self.name {
            if name == record.name {
                self.stack.push(name);
                let resolved = keeper.resolve(new_name, self)?;
                self.stack.pop();
                found = Some(resolved);
            }
        }
        if let Some(value) = found {
            self.cache.insert(name, value);
        }
        Ok(found)
    }

    fn current_record(&self) -> Option<RecordIndex> {
        Some(self.record)
    }

    fn is_final(&self) -> bool {
        true
    }

    fn keep_unset_bits(&self) -> bool {
        true
    }
}

struct ShadowResolver<'r> {
    inner: &'r mut dyn Resolver,
    shadowed: Vec<Value>,
}

impl Resolver for ShadowResolver<'_> {
    fn resolve(&mut self, keeper: &mut RecordKeeper, name: Value) -> Flow<Option<Value>> {
        if self.shadowed.contains(&name) {
            return Ok(None);
        }
        self.inner.resolve(keeper, name)
    }

    fn current_record(&self) -> Option<RecordIndex> {
        self.inner.current_record()
    }

    fn is_final(&self) -> bool {
        self.inner.is_final()
    }
}

struct TrackUnresolvedResolver<'r> {
    inner: Option<&'r mut dyn Resolver>,
    found_unresolved: bool,
}

impl Resolver for TrackUnresolvedResolver<'_> {
    fn resolve(&mut self, keeper: &mut RecordKeeper, name: Value) -> Flow<Option<Value>> {
        let mut found = None;
        if let Some(inner) = self.inner.as_mut() {
            found = inner.resolve(keeper, name)?;
            if let Some(value) = found {
                if !self.found_unresolved {
                    let mut sub = TrackUnresolvedResolver {
                        inner: None,
                        found_unresolved: false,
                    };
                    keeper.resolve(value, &mut sub)?;
                    self.found_unresolved |= sub.found_unresolved;
                }
            }
        }
        if found.is_none() {
            self.found_unresolved = true;
        }
        Ok(found)
    }

    fn current_record(&self) -> Option<RecordIndex> {
        self.inner.as_ref().and_then(|inner| inner.current_record())
    }

    fn is_final(&self) -> bool {
        false
    }
}

struct HasReferenceResolver {
    target: Option<Value>,
    found: bool,
    finished: bool,
}

impl Resolver for HasReferenceResolver {
    fn resolve(&mut self, _keeper: &mut RecordKeeper, name: Value) -> Flow<Option<Value>> {
        if Some(name) == self.target {
            self.found = true;
        }
        Ok(None)
    }

    fn current_record(&self) -> Option<RecordIndex> {
        None
    }

    fn is_final(&self) -> bool {
        self.finished
    }
}

pub fn compare_numeric(left: &str, right: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let left = left.as_bytes();
    let right = right.as_bytes();
    let end = left.len().min(right.len());
    let mut index = 0;
    while index != end {
        if left[index].is_ascii_digit() && right[index].is_ascii_digit() {
            let mut stop = index + 1;
            while stop != end + 1 {
                let left_digit = stop < left.len() && left[stop].is_ascii_digit();
                let right_digit = stop < right.len() && right[stop].is_ascii_digit();
                if left_digit != right_digit {
                    return if right_digit {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    };
                }
                if !right_digit {
                    break;
                }
                stop += 1;
            }
            let order = left[index..stop].cmp(&right[index..stop]);
            if order != Ordering::Equal {
                return order;
            }
            index = stop;
            continue;
        }
        if left[index] != right[index] {
            return left[index].cmp(&right[index]);
        }
        index += 1;
    }
    left.len().cmp(&right.len())
}

fn fits_in_bits(value: i64, width: u32) -> bool {
    width >= 64 || (value >> width) == 0 || (value >> (width - 1)) == -1
}

impl Default for RecordKeeper {
    fn default() -> Self {
        RecordKeeper::new()
    }
}

impl RecordKeeper {
    pub fn new() -> RecordKeeper {
        let mut keeper = RecordKeeper {
            type_shapes: Vec::new(),
            type_lookup: BTreeMap::new(),
            nodes: Vec::new(),
            node_types: Vec::new(),
            interner: Interner::default(),
            texts: Vec::new(),
            text_interner: Interner::default(),
            records: Vec::new(),
            free_records: Vec::new(),
            classes: BTreeMap::new(),
            definitions: BTreeMap::new(),
            globals: BTreeMap::new(),
            derived_cache: BTreeMap::new(),
            anonymous_counter: 0,
            record_counter: 0,
            variable_definitions: BTreeMap::new(),
            messages: Vec::new(),
            errors: 0,
            stopped: false,
            bit_type: Type::NONE,
            integer_type: Type::NONE,
            string_type: Type::NONE,
            dag_type: Type::NONE,
            any_record_type: Type::NONE,
            unset: Value(0),
            true_bit: Value(0),
            false_bit: Value(0),
        };
        keeper.bit_type = keeper.intern_type(TypeShape::Bit);
        keeper.integer_type = keeper.intern_type(TypeShape::Integer);
        keeper.string_type = keeper.intern_type(TypeShape::String);
        keeper.dag_type = keeper.intern_type(TypeShape::Dag);
        keeper.any_record_type = keeper.intern_type(TypeShape::Record(Vec::new()));
        keeper.unset = keeper.intern(Node::Unset, Type::NONE);
        keeper.true_bit = keeper.intern(Node::Bit(true), keeper.bit_type);
        keeper.false_bit = keeper.intern(Node::Bit(false), keeper.bit_type);
        keeper
    }

    pub fn report(&mut self, kind: MessageKind, locations: &[Location], text: impl Into<String>) {
        if kind == MessageKind::Error {
            self.errors += 1;
        }
        self.messages.push(Message {
            kind,
            location: locations.first().copied(),
            text: text.into(),
        });
        for location in locations.iter().skip(1) {
            self.messages.push(Message {
                kind: MessageKind::Note,
                location: Some(*location),
                text: "instantiated from multiclass".to_string(),
            });
        }
    }

    pub fn fatal(&mut self, locations: &[Location], text: impl Into<String>) -> Stop {
        self.report(MessageKind::Error, locations, text);
        self.stopped = true;
        Stop
    }

    fn record_locations(&self, record: Option<RecordIndex>) -> Vec<Location> {
        record
            .map(|record| self.record(record).locations.clone())
            .unwrap_or_default()
    }

    fn intern_type(&mut self, shape: TypeShape) -> Type {
        if let Some(found) = self.type_lookup.get(&shape) {
            return *found;
        }
        let made = Type(self.type_shapes.len() as u32);
        self.type_shapes.push(shape.clone());
        self.type_lookup.insert(shape, made);
        made
    }

    pub fn shape(&self, of: Type) -> &TypeShape {
        &self.type_shapes[of.0 as usize]
    }

    pub fn bit_type(&self) -> Type {
        self.bit_type
    }

    pub fn integer_type(&self) -> Type {
        self.integer_type
    }

    pub fn string_type(&self) -> Type {
        self.string_type
    }

    pub fn dag_type(&self) -> Type {
        self.dag_type
    }

    pub fn bits_type(&mut self, width: u32) -> Type {
        self.intern_type(TypeShape::Bits(width))
    }

    pub fn list_type(&mut self, element: Type) -> Type {
        self.intern_type(TypeShape::List(element))
    }

    pub fn record_type(&mut self, classes: &[RecordIndex]) -> Type {
        let mut sorted = classes.to_vec();
        sorted.sort_by(|left, right| {
            self.record_name_text(*left)
                .cmp(&self.record_name_text(*right))
        });
        self.intern_type(TypeShape::Record(sorted))
    }

    fn is_bit(&self, of: Type) -> bool {
        of != Type::NONE && matches!(self.shape(of), TypeShape::Bit)
    }

    fn is_integer(&self, of: Type) -> bool {
        of != Type::NONE && matches!(self.shape(of), TypeShape::Integer)
    }

    fn is_string_type(&self, of: Type) -> bool {
        of != Type::NONE && matches!(self.shape(of), TypeShape::String)
    }

    fn is_dag_type(&self, of: Type) -> bool {
        of != Type::NONE && matches!(self.shape(of), TypeShape::Dag)
    }

    fn list_element(&self, of: Type) -> Option<Type> {
        if of == Type::NONE {
            return None;
        }
        match self.shape(of) {
            TypeShape::List(element) => Some(*element),
            _ => None,
        }
    }

    fn bits_width(&self, of: Type) -> Option<u32> {
        if of == Type::NONE {
            return None;
        }
        match self.shape(of) {
            TypeShape::Bits(width) => Some(*width),
            _ => None,
        }
    }

    fn record_classes(&self, of: Type) -> Option<&[RecordIndex]> {
        if of == Type::NONE {
            return None;
        }
        match self.shape(of) {
            TypeShape::Record(classes) => Some(classes),
            _ => None,
        }
    }

    pub fn type_name(&self, of: Type) -> String {
        if of == Type::NONE {
            return String::new();
        }
        match self.shape(of) {
            TypeShape::Bit => "bit".to_string(),
            TypeShape::Bits(width) => format!("bits<{width}>"),
            TypeShape::Integer => "int".to_string(),
            TypeShape::String => "string".to_string(),
            TypeShape::List(element) => format!("list<{}>", self.type_name(*element)),
            TypeShape::Dag => "dag".to_string(),
            TypeShape::Record(classes) => {
                if classes.len() == 1 {
                    return self.record_name_text(classes[0]);
                }
                let names: Vec<String> = classes
                    .iter()
                    .map(|class| self.record_name_text(*class))
                    .collect();
                format!("{{{}}}", names.join(", "))
            }
        }
    }

    pub fn is_subclass_of(&self, record: RecordIndex, class: RecordIndex) -> bool {
        self.record(record)
            .superclasses
            .iter()
            .any(|(superclass, _)| *superclass == class || self.is_subclass_of(*superclass, class))
    }

    fn classes_include(&self, classes: &[RecordIndex], class: RecordIndex) -> bool {
        classes
            .iter()
            .any(|held| *held == class || self.is_subclass_of(*held, class))
    }

    pub fn type_convertible(&self, from: Type, to: Type) -> bool {
        if from == Type::NONE || to == Type::NONE {
            return false;
        }
        match (self.shape(from), self.shape(to)) {
            (TypeShape::Bit, TypeShape::Bit | TypeShape::Integer) => true,
            (TypeShape::Bit, TypeShape::Bits(width)) => *width == 1,
            (TypeShape::Bits(width), TypeShape::Bits(other)) => width == other,
            (TypeShape::Bits(width), TypeShape::Bit) => *width == 1,
            (TypeShape::Bits(_), TypeShape::Integer) => true,
            (TypeShape::Integer, TypeShape::Bit | TypeShape::Bits(_) | TypeShape::Integer) => true,
            (TypeShape::String, TypeShape::String) => true,
            (TypeShape::List(element), TypeShape::List(other)) => {
                self.type_convertible(*element, *other)
            }
            (TypeShape::Dag, TypeShape::Dag) => true,
            (TypeShape::Record(classes), TypeShape::Record(targets)) => {
                from == to
                    || targets
                        .iter()
                        .all(|target| self.classes_include(classes, *target))
            }
            _ => false,
        }
    }

    pub fn type_is_a(&self, from: Type, to: Type) -> bool {
        if from == Type::NONE || to == Type::NONE {
            return false;
        }
        match (self.shape(from), self.shape(to)) {
            (TypeShape::List(element), TypeShape::List(other)) => self.type_is_a(*element, *other),
            (TypeShape::List(_), _) => false,
            (TypeShape::Record(_), _) => self.type_convertible(from, to),
            _ => from == to,
        }
    }

    pub fn resolve_types(&mut self, first: Type, second: Type) -> Option<Type> {
        if first == second {
            return Some(first);
        }
        if let (Some(left), Some(right)) = (
            self.record_classes(first).map(|classes| classes.to_vec()),
            self.record_classes(second).map(|classes| classes.to_vec()),
        ) {
            let mut common = Vec::new();
            let mut stack = left;
            while let Some(record) = stack.pop() {
                if self.classes_include(&right, record) {
                    common.push(record);
                } else {
                    for (superclass, _) in &self.record(record).superclasses {
                        stack.push(*superclass);
                    }
                }
            }
            return Some(self.record_type(&common));
        }
        if self.type_convertible(first, second) {
            return Some(second);
        }
        if self.type_convertible(second, first) {
            return Some(first);
        }
        if let (Some(left), Some(right)) = (self.list_element(first), self.list_element(second)) {
            if let Some(element) = self.resolve_types(left, right) {
                return Some(self.list_type(element));
            }
        }
        None
    }

    fn intern(&mut self, node: Node, of: Type) -> Value {
        let hash = node_hash(&node);
        let nodes = &self.nodes;
        match self
            .interner
            .find(hash, |held| nodes[held as usize] == node)
        {
            Ok(found) => Value(found),
            Err(slot) => {
                let made = self.nodes.len() as u32;
                self.nodes.push(node);
                self.node_types.push(of);
                self.interner.insert(hash, made, slot, |_| 0);
                Value(made)
            }
        }
    }

    fn fresh(&mut self, node: Node, of: Type) -> Value {
        let made = self.nodes.len() as u32;
        self.nodes.push(node);
        self.node_types.push(of);
        Value(made)
    }

    pub fn node(&self, value: Value) -> &Node {
        &self.nodes[value.index()]
    }

    pub fn type_of(&self, value: Value) -> Option<Type> {
        let found = self.node_types[value.index()];
        if found == Type::NONE {
            None
        } else {
            Some(found)
        }
    }

    fn typed(&self, value: Value) -> bool {
        !matches!(self.node(value), Node::Unset | Node::Argument { .. })
    }

    pub fn is_unset(&self, value: Value) -> bool {
        matches!(self.node(value), Node::Unset)
    }

    fn intern_text(&mut self, text: &str) -> u32 {
        let hash = text_hash(text);
        let texts = &self.texts;
        match self
            .text_interner
            .find(hash, |held| &*texts[held as usize] == text)
        {
            Ok(found) => found,
            Err(slot) => {
                let made = self.texts.len() as u32;
                self.texts.push(text.into());
                self.text_interner.insert(hash, made, slot, |_| 0);
                made
            }
        }
    }

    pub fn text(&self, id: u32) -> &str {
        &self.texts[id as usize]
    }

    pub fn string_of(&self, value: Value) -> Option<&str> {
        match self.node(value) {
            Node::String { text, .. } => Some(self.text(*text)),
            _ => None,
        }
    }

    pub fn integer_of(&self, value: Value) -> Option<i64> {
        match self.node(value) {
            Node::Integer(integer) => Some(*integer),
            _ => None,
        }
    }

    pub fn unset(&self) -> Value {
        self.unset
    }

    pub fn initial_value(&mut self, of: Type) -> Value {
        match self.bits_width(of) {
            Some(width) => {
                let unset = self.unset;
                self.bits(vec![unset; width as usize])
            }
            None => self.unset,
        }
    }

    pub fn bit(&self, bit: bool) -> Value {
        if bit {
            self.true_bit
        } else {
            self.false_bit
        }
    }

    pub fn integer(&mut self, integer: i64) -> Value {
        let of = self.integer_type;
        self.intern(Node::Integer(integer), of)
    }

    pub fn string(&mut self, text: &str) -> Value {
        self.string_with_format(text, false)
    }

    pub fn string_with_format(&mut self, text: &str, code: bool) -> Value {
        let id = self.intern_text(text);
        let of = self.string_type;
        self.intern(Node::String { text: id, code }, of)
    }

    pub fn bits(&mut self, bits: Vec<Value>) -> Value {
        let of = self.bits_type(bits.len() as u32);
        self.intern(Node::Bits(bits.into_boxed_slice()), of)
    }

    pub fn list(&mut self, elements: Vec<Value>, element: Type) -> Value {
        let of = self.list_type(element);
        self.intern(
            Node::List {
                elements: elements.into_boxed_slice(),
                element,
            },
            of,
        )
    }

    pub fn unary(&mut self, operator: UnaryOperator, operand: Value, result: Type) -> Value {
        self.intern(
            Node::Unary {
                operator,
                operand,
                result,
            },
            result,
        )
    }

    pub fn binary(
        &mut self,
        operator: BinaryOperator,
        left: Value,
        right: Value,
        result: Type,
    ) -> Value {
        self.intern(
            Node::Binary {
                operator,
                left,
                right,
                result,
            },
            result,
        )
    }

    pub fn ternary(
        &mut self,
        operator: TernaryOperator,
        left: Value,
        middle: Value,
        right: Value,
        result: Type,
    ) -> Value {
        self.intern(
            Node::Ternary {
                operator,
                left,
                middle,
                right,
                result,
            },
            result,
        )
    }

    pub fn condition(&mut self, conditions: Vec<Value>, values: Vec<Value>, result: Type) -> Value {
        self.intern(
            Node::Condition {
                conditions: conditions.into_boxed_slice(),
                values: values.into_boxed_slice(),
                result,
            },
            result,
        )
    }

    pub fn fold_node(
        &mut self,
        start: Value,
        list: Value,
        accumulator: Value,
        element: Value,
        expression: Value,
        result: Type,
    ) -> Value {
        self.intern(
            Node::Fold {
                start,
                list,
                accumulator,
                element,
                expression,
                result,
            },
            result,
        )
    }

    pub fn is_a(&mut self, checked: Type, expression: Value) -> Value {
        let of = self.integer_type;
        self.intern(
            Node::IsA {
                checked,
                expression,
            },
            of,
        )
    }

    pub fn exists(&mut self, checked: Type, expression: Value) -> Value {
        let of = self.integer_type;
        self.intern(
            Node::Exists {
                checked,
                expression,
            },
            of,
        )
    }

    pub fn instances(&mut self, checked: Type, pattern: Value) -> Value {
        let of = self.list_type(checked);
        self.intern(Node::Instances { checked, pattern }, of)
    }

    pub fn variable(&mut self, name: Value, variable_type: Type) -> Value {
        self.intern(
            Node::Variable {
                name,
                variable_type,
            },
            variable_type,
        )
    }

    pub fn variable_bit(&mut self, variable: Value, bit: u32) -> Value {
        let of = self.bit_type;
        self.intern(Node::VariableBit { variable, bit }, of)
    }

    pub fn field_reference(&mut self, record: Value, name: Value) -> Value {
        let of = self.field_type(record, name).unwrap_or(Type::NONE);
        self.intern(Node::Field { record, name }, of)
    }

    pub fn dag(
        &mut self,
        operator: Value,
        name: Option<Value>,
        arguments: Vec<Value>,
        names: Vec<Option<Value>>,
    ) -> Value {
        let of = self.dag_type;
        self.intern(
            Node::Dag {
                operator,
                name,
                arguments: arguments.into_boxed_slice(),
                names: names.into_boxed_slice(),
            },
            of,
        )
    }

    pub fn argument(&mut self, value: Value, name: Option<Value>, position: u32) -> Value {
        self.intern(
            Node::Argument {
                value,
                name,
                position,
            },
            Type::NONE,
        )
    }

    fn argument_parts(&self, argument: Value) -> (Value, Option<Value>, u32) {
        match self.node(argument) {
            Node::Argument {
                value,
                name,
                position,
            } => (*value, *name, *position),
            _ => (argument, None, 0),
        }
    }

    pub fn variable_definition(
        &mut self,
        location: Location,
        class: RecordIndex,
        arguments: Vec<Value>,
    ) -> Value {
        let of = self.record_type(&[class]);
        let made = self.intern(
            Node::VariableDefinition {
                class,
                arguments: arguments.into_boxed_slice(),
            },
            of,
        );
        self.variable_definitions
            .entry(made)
            .or_insert(VariableDefinitionState {
                location,
                definition: None,
            });
        made
    }

    pub fn new_anonymous_name(&mut self) -> Value {
        let number = self.anonymous_counter;
        self.anonymous_counter += 1;
        let of = self.string_type;
        self.fresh(Node::AnonymousName(number), of)
    }

    pub fn record(&self, index: RecordIndex) -> &Record {
        self.records[index.index()].as_ref().expect("a live record")
    }

    pub fn record_mut(&mut self, index: RecordIndex) -> &mut Record {
        self.records[index.index()].as_mut().expect("a live record")
    }

    fn store_record(&mut self, record: Record) -> RecordIndex {
        if let Some(free) = self.free_records.pop() {
            self.records[free as usize] = Some(record);
            RecordIndex(free)
        } else {
            self.records.push(Some(record));
            RecordIndex(self.records.len() as u32 - 1)
        }
    }

    pub fn new_record(
        &mut self,
        name: Value,
        locations: Vec<Location>,
        kind: RecordKind,
    ) -> Flow<RecordIndex> {
        let identifier = self.record_counter;
        self.record_counter += 1;
        let made = self.store_record(Record {
            name,
            locations,
            template_arguments: Vec::new(),
            fields: Vec::new(),
            assertions: Vec::new(),
            dumps: Vec::new(),
            superclasses: Vec::new(),
            definition: None,
            identifier,
            kind,
        });
        self.check_record_name(made)?;
        Ok(made)
    }

    fn check_record_name(&mut self, record: RecordIndex) -> Flow<()> {
        let name = self.record(record).name;
        let string_type = self.string_type;
        if self.type_of(name) != Some(string_type) {
            let locations = self.record(record).locations.clone();
            let text = format!("Record name '{}' is not a string!", self.text_of(name));
            return Err(self.fatal(&locations, text));
        }
        Ok(())
    }

    pub fn set_record_name(&mut self, record: RecordIndex, name: Value) -> Flow<()> {
        self.record_mut(record).name = name;
        self.check_record_name(record)
    }

    pub fn copy_record(&mut self, source: RecordIndex) -> RecordIndex {
        let identifier = self.record_counter;
        self.record_counter += 1;
        let original = self.record(source);
        let copy = Record {
            name: original.name,
            locations: original.locations.clone(),
            template_arguments: original.template_arguments.clone(),
            fields: original.fields.clone(),
            assertions: original.assertions.clone(),
            dumps: Vec::new(),
            superclasses: original.superclasses.clone(),
            definition: None,
            identifier,
            kind: original.kind,
        };
        self.store_record(copy)
    }

    pub fn release_record(&mut self, record: RecordIndex) {
        self.records[record.index()] = None;
        self.free_records.push(record.0);
    }

    pub fn record_name_text(&self, record: RecordIndex) -> String {
        self.unquoted(self.record(record).name)
    }

    pub fn record_type_of(&mut self, record: RecordIndex) -> Type {
        let classes: Vec<RecordIndex> = self
            .record(record)
            .superclasses
            .iter()
            .map(|(class, _)| *class)
            .collect();
        self.record_type(&classes)
    }

    pub fn definition_of(&mut self, record: RecordIndex) -> Value {
        if let Some(found) = self.record(record).definition {
            return found;
        }
        let of = self.record_type_of(record);
        let made = self.intern(Node::Definition(record), of);
        self.record_mut(record).definition = Some(made);
        made
    }

    pub fn find_definition(&self, name: &str) -> Option<RecordIndex> {
        self.definitions.get(name).copied()
    }

    pub fn find_class(&self, name: &str) -> Option<RecordIndex> {
        self.classes.get(name).copied()
    }

    pub fn global(&mut self, name: &str) -> Option<Value> {
        if let Some(record) = self.find_definition(name) {
            return Some(self.definition_of(record));
        }
        self.globals.get(name).copied()
    }

    pub fn add_definition(&mut self, record: RecordIndex) {
        let name = self.record_name_text(record);
        self.definitions.insert(name, record);
        self.derived_cache.clear();
    }

    pub fn superclass_list(&self, record: RecordIndex) -> Vec<RecordIndex> {
        let mut found = Vec::new();
        self.collect_superclasses(record, &mut found);
        found
    }

    fn collect_superclasses(&self, record: RecordIndex, found: &mut Vec<RecordIndex>) {
        for (class, _) in &self.record(record).superclasses {
            self.collect_superclasses(*class, found);
            found.push(*class);
        }
    }

    pub fn derived_definitions(&mut self, class_name: &str) -> Vec<RecordIndex> {
        if let Some(found) = self.derived_cache.get(class_name) {
            return found.clone();
        }
        let Some(class) = self.find_class(class_name) else {
            return Vec::new();
        };
        let mut found: Vec<RecordIndex> = self
            .definitions
            .values()
            .copied()
            .filter(|record| self.is_subclass_of(*record, class))
            .collect();
        found.sort_by(|left, right| {
            compare_numeric(
                &self.record_name_text(*left),
                &self.record_name_text(*right),
            )
        });
        self.derived_cache
            .insert(class_name.to_string(), found.clone());
        found
    }

    pub fn field_type(&self, value: Value, name: Value) -> Option<Type> {
        if let Node::Definition(record) = self.node(value) {
            let record = self.record(*record);
            return record.field(name).map(|at| record.fields[at].field_type);
        }
        let of = self.type_of(value)?;
        let classes = self.record_classes(of)?;
        for class in classes {
            let class = self.record(*class);
            if let Some(at) = class.field(name) {
                return Some(class.fields[at].field_type);
            }
        }
        None
    }

    pub fn text_of(&self, value: Value) -> String {
        let mut out = String::new();
        self.write_value(value, &mut out);
        out
    }

    pub fn unquoted(&self, value: Value) -> String {
        match self.node(value) {
            Node::String { text, .. } => self.text(*text).to_string(),
            _ => self.text_of(value),
        }
    }

    fn write_unquoted(&self, value: Value, out: &mut String) {
        match self.node(value) {
            Node::String { text, .. } => out.push_str(self.text(*text)),
            _ => self.write_value(value, out),
        }
    }

    fn write_values(&self, values: &[Value], out: &mut String) {
        for (index, value) in values.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            self.write_value(*value, out);
        }
    }

    pub fn write_value(&self, value: Value, out: &mut String) {
        match self.node(value) {
            Node::Unset => out.push('?'),
            Node::Argument {
                value,
                name,
                position,
            } => {
                match name {
                    Some(name) => self.write_value(*name, out),
                    None => out.push_str(&position.to_string()),
                }
                out.push_str(": ");
                self.write_value(*value, out);
            }
            Node::Bit(bit) => out.push(if *bit { '1' } else { '0' }),
            Node::Bits(bits) => {
                out.push_str("{ ");
                for (index, bit) in bits.iter().rev().enumerate() {
                    if index > 0 {
                        out.push_str(", ");
                    }
                    self.write_value(*bit, out);
                }
                out.push_str(" }");
            }
            Node::Integer(integer) => out.push_str(&integer.to_string()),
            Node::AnonymousName(number) => {
                out.push_str("anonymous_");
                out.push_str(&number.to_string());
            }
            Node::String { text, code } => {
                if *code {
                    out.push_str("[{");
                    out.push_str(self.text(*text));
                    out.push_str("}]");
                } else {
                    out.push('"');
                    out.push_str(self.text(*text));
                    out.push('"');
                }
            }
            Node::List { elements, .. } => {
                out.push('[');
                self.write_values(elements, out);
                out.push(']');
            }
            Node::Unary {
                operator,
                operand,
                result,
            } => {
                let name = match operator {
                    UnaryOperator::Cast => format!("!cast<{}>", self.type_name(*result)),
                    UnaryOperator::Not => "!not".to_string(),
                    UnaryOperator::Head => "!head".to_string(),
                    UnaryOperator::Tail => "!tail".to_string(),
                    UnaryOperator::Size => "!size".to_string(),
                    UnaryOperator::Empty => "!empty".to_string(),
                    UnaryOperator::GetDagOperator => "!getdagop".to_string(),
                    UnaryOperator::GetDagOperatorName => "!getdagopname".to_string(),
                    UnaryOperator::LogTwo => "!logtwo".to_string(),
                    UnaryOperator::ListFlatten => "!listflatten".to_string(),
                    UnaryOperator::Represent => "!repr".to_string(),
                    UnaryOperator::ToLower => "!tolower".to_string(),
                    UnaryOperator::ToUpper => "!toupper".to_string(),
                    UnaryOperator::Initialized => "!initialized".to_string(),
                };
                out.push_str(&name);
                out.push('(');
                self.write_value(*operand, out);
                out.push(')');
            }
            Node::Binary {
                operator,
                left,
                right,
                result,
            } => {
                let name = match operator {
                    BinaryOperator::ListElement | BinaryOperator::ListSlice => {
                        self.write_value(*left, out);
                        out.push('[');
                        self.write_value(*right, out);
                        out.push(']');
                        return;
                    }
                    BinaryOperator::RangeClosed => {
                        self.write_value(*left, out);
                        out.push_str("...");
                        self.write_value(*right, out);
                        return;
                    }
                    BinaryOperator::Concat => "!con".to_string(),
                    BinaryOperator::Match => "!match".to_string(),
                    BinaryOperator::Add => "!add".to_string(),
                    BinaryOperator::Subtract => "!sub".to_string(),
                    BinaryOperator::Multiply => "!mul".to_string(),
                    BinaryOperator::Divide => "!div".to_string(),
                    BinaryOperator::And => "!and".to_string(),
                    BinaryOperator::Or => "!or".to_string(),
                    BinaryOperator::ExclusiveOr => "!xor".to_string(),
                    BinaryOperator::ShiftLeft => "!shl".to_string(),
                    BinaryOperator::ShiftRightArithmetic => "!sra".to_string(),
                    BinaryOperator::ShiftRightLogical => "!srl".to_string(),
                    BinaryOperator::Equal => "!eq".to_string(),
                    BinaryOperator::NotEqual => "!ne".to_string(),
                    BinaryOperator::LessOrEqual => "!le".to_string(),
                    BinaryOperator::Less => "!lt".to_string(),
                    BinaryOperator::GreaterOrEqual => "!ge".to_string(),
                    BinaryOperator::Greater => "!gt".to_string(),
                    BinaryOperator::ListConcat => "!listconcat".to_string(),
                    BinaryOperator::ListSplat => "!listsplat".to_string(),
                    BinaryOperator::ListRemove => "!listremove".to_string(),
                    BinaryOperator::StringConcat => "!strconcat".to_string(),
                    BinaryOperator::Interleave => "!interleave".to_string(),
                    BinaryOperator::SetDagOperator => "!setdagop".to_string(),
                    BinaryOperator::SetDagOperatorName => "!setdagopname".to_string(),
                    BinaryOperator::GetDagArgument => {
                        format!("!getdagarg<{}>", self.type_name(*result))
                    }
                    BinaryOperator::GetDagName => "!getdagname".to_string(),
                };
                out.push_str(&name);
                out.push('(');
                self.write_value(*left, out);
                out.push_str(", ");
                self.write_value(*right, out);
                out.push(')');
            }
            Node::Ternary {
                operator,
                left,
                middle,
                right,
                ..
            } => {
                let (name, unquoted) = match operator {
                    TernaryOperator::Dag => ("!dag", false),
                    TernaryOperator::Filter => ("!filter", true),
                    TernaryOperator::ForEach => ("!foreach", true),
                    TernaryOperator::Sort => ("!sort", true),
                    TernaryOperator::If => ("!if", false),
                    TernaryOperator::Range => ("!range", false),
                    TernaryOperator::Substitute => ("!subst", false),
                    TernaryOperator::Substring => ("!substr", false),
                    TernaryOperator::Find => ("!find", false),
                    TernaryOperator::SetDagArgument => ("!setdagarg", false),
                    TernaryOperator::SetDagName => ("!setdagname", false),
                };
                out.push_str(name);
                out.push('(');
                if unquoted {
                    self.write_unquoted(*left, out);
                } else {
                    self.write_value(*left, out);
                }
                out.push_str(", ");
                self.write_value(*middle, out);
                out.push_str(", ");
                self.write_value(*right, out);
                out.push(')');
            }
            Node::Condition {
                conditions, values, ..
            } => {
                out.push_str("!cond(");
                for (index, (condition, value)) in conditions.iter().zip(values.iter()).enumerate()
                {
                    if index > 0 {
                        out.push_str(", ");
                    }
                    self.write_value(*condition, out);
                    out.push_str(": ");
                    self.write_value(*value, out);
                }
                out.push(')');
            }
            Node::Fold {
                start,
                list,
                accumulator,
                element,
                expression,
                ..
            } => {
                out.push_str("!foldl(");
                self.write_value(*start, out);
                out.push_str(", ");
                self.write_value(*list, out);
                out.push_str(", ");
                self.write_unquoted(*accumulator, out);
                out.push_str(", ");
                self.write_unquoted(*element, out);
                out.push_str(", ");
                self.write_value(*expression, out);
                out.push(')');
            }
            Node::IsA {
                checked,
                expression,
            } => {
                out.push_str(&format!("!isa<{}>(", self.type_name(*checked)));
                self.write_value(*expression, out);
                out.push(')');
            }
            Node::Exists {
                checked,
                expression,
            } => {
                out.push_str(&format!("!exists<{}>(", self.type_name(*checked)));
                self.write_value(*expression, out);
                out.push(')');
            }
            Node::Instances { checked, pattern } => {
                out.push_str(&format!("!instances<{}>(", self.type_name(*checked)));
                self.write_value(*pattern, out);
                out.push(')');
            }
            Node::Variable { name, .. } => self.write_unquoted(*name, out),
            Node::VariableBit { variable, bit } => {
                self.write_value(*variable, out);
                out.push('{');
                out.push_str(&bit.to_string());
                out.push('}');
            }
            Node::Definition(record) => self.write_unquoted(self.record(*record).name, out),
            Node::VariableDefinition { class, arguments } => {
                out.push_str(&self.record_name_text(*class));
                out.push('<');
                self.write_values(arguments, out);
                out.push('>');
            }
            Node::Field { record, name } => {
                self.write_value(*record, out);
                out.push('.');
                self.write_unquoted(*name, out);
            }
            Node::Dag {
                operator,
                name,
                arguments,
                names,
            } => {
                out.push('(');
                self.write_value(*operator, out);
                if let Some(name) = name {
                    out.push_str(":$");
                    self.write_unquoted(*name, out);
                }
                if !arguments.is_empty() {
                    out.push(' ');
                    for (index, (argument, name)) in arguments.iter().zip(names.iter()).enumerate()
                    {
                        if index > 0 {
                            out.push_str(", ");
                        }
                        self.write_value(*argument, out);
                        if let Some(name) = name {
                            out.push_str(":$");
                            self.write_unquoted(*name, out);
                        }
                    }
                }
                out.push(')');
            }
        }
    }

    pub fn is_complete(&self, value: Value) -> bool {
        match self.node(value) {
            Node::Unset | Node::Argument { .. } => false,
            Node::Bits(bits) => bits.iter().all(|bit| self.is_complete(*bit)),
            Node::List { elements, .. } => {
                elements.iter().all(|element| self.is_complete(*element))
            }
            Node::Ternary {
                left,
                middle,
                right,
                ..
            } => self.is_complete(*left) && self.is_complete(*middle) && self.is_complete(*right),
            Node::Condition {
                conditions, values, ..
            } => conditions
                .iter()
                .zip(values.iter())
                .all(|(condition, value)| self.is_complete(*condition) && self.is_complete(*value)),
            Node::Fold { .. } | Node::IsA { .. } | Node::Exists { .. } | Node::Instances { .. } => {
                false
            }
            _ => true,
        }
    }

    pub fn is_concrete(&self, value: Value) -> bool {
        match self.node(value) {
            Node::Unset
            | Node::Bit(_)
            | Node::Integer(_)
            | Node::String { .. }
            | Node::Definition(_) => true,
            Node::Bits(bits) => bits.iter().all(|bit| self.is_concrete(*bit)),
            Node::List { elements, .. } => {
                elements.iter().all(|element| self.is_concrete(*element))
            }
            Node::Condition {
                conditions, values, ..
            } => conditions
                .iter()
                .zip(values.iter())
                .all(|(condition, value)| self.is_concrete(*condition) && self.is_concrete(*value)),
            Node::Dag {
                operator,
                arguments,
                ..
            } => {
                self.is_concrete(*operator)
                    && arguments.iter().all(|argument| self.is_concrete(*argument))
            }
            Node::Field { record, name } => {
                if let Node::Definition(definition) = self.node(*record) {
                    let definition = self.record(*definition);
                    if let Some(at) = definition.field(*name) {
                        return self.is_concrete(definition.fields[at].value);
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn get_bit(&mut self, value: Value, bit: u32) -> Value {
        match self.node(value) {
            Node::Unset | Node::Bit(_) | Node::VariableBit { .. } => value,
            Node::Argument { value: inner, .. } => {
                let inner = *inner;
                self.get_bit(inner, bit)
            }
            Node::Bits(bits) => bits.get(bit as usize).copied().unwrap_or(value),
            Node::Integer(integer) => {
                let set = bit < 64 && (*integer as u64) & (1u64 << bit) != 0;
                self.bit(set)
            }
            Node::Unary { .. }
            | Node::Binary { .. }
            | Node::Ternary { .. }
            | Node::Condition { .. }
            | Node::Fold { .. }
            | Node::Variable { .. }
            | Node::Field { .. } => {
                let of = self.node_types[value.index()];
                if self.is_bit(of) {
                    value
                } else {
                    self.variable_bit(value, bit)
                }
            }
            Node::IsA { .. } | Node::Exists { .. } => self.variable_bit(value, bit),
            _ => value,
        }
    }

    pub fn convert_to(&mut self, value: Value, to: Type) -> Option<Value> {
        match self.node(value).clone() {
            Node::Unset => Some(value),
            Node::Argument { value: inner, .. } => self.convert_to(inner, to),
            Node::Bit(bit) => match self.shape(to).clone() {
                TypeShape::Bit => Some(value),
                TypeShape::Integer => Some(self.integer(bit as i64)),
                TypeShape::Bits(1) => Some(self.bits(vec![value])),
                _ => None,
            },
            Node::Bits(bits) => match self.shape(to).clone() {
                TypeShape::Bit => {
                    if bits.len() != 1 {
                        None
                    } else {
                        Some(bits[0])
                    }
                }
                TypeShape::Bits(width) => {
                    if width as usize != bits.len() {
                        None
                    } else {
                        Some(value)
                    }
                }
                TypeShape::Integer => {
                    let mut result: i64 = 0;
                    for (index, bit) in bits.iter().enumerate() {
                        match self.node(*bit) {
                            Node::Bit(set) => {
                                if *set && index < 64 {
                                    result |= 1i64 << index;
                                }
                            }
                            _ => return None,
                        }
                    }
                    Some(self.integer(result))
                }
                _ => None,
            },
            Node::Integer(integer) => match self.shape(to).clone() {
                TypeShape::Integer => Some(value),
                TypeShape::Bit => {
                    if integer != 0 && integer != 1 {
                        None
                    } else {
                        Some(self.bit(integer != 0))
                    }
                }
                TypeShape::Bits(width) => {
                    if !fits_in_bits(integer, width) {
                        return None;
                    }
                    let bits = (0..width)
                        .map(|index| {
                            self.bit(index < 64 && (integer as u64) & (1u64 << index) != 0)
                        })
                        .collect();
                    Some(self.bits(bits))
                }
                _ => None,
            },
            Node::String { .. } => {
                if self.is_string_type(to) {
                    Some(value)
                } else {
                    None
                }
            }
            Node::List { elements, .. } => {
                if self.node_types[value.index()] == to {
                    return Some(value);
                }
                let element = self.list_element(to)?;
                let mut converted = Vec::with_capacity(elements.len());
                let mut changed = false;
                for item in elements.iter() {
                    let made = self.convert_to(*item, element)?;
                    if made != *item {
                        changed = true;
                    }
                    converted.push(made);
                }
                if !changed {
                    return Some(value);
                }
                Some(self.list(converted, element))
            }
            Node::Definition(_) => {
                let of = self.node_types[value.index()];
                if self.record_classes(to).is_some() && self.type_convertible(of, to) {
                    Some(value)
                } else {
                    None
                }
            }
            _ => {
                let of = self.node_types[value.index()];
                if self.type_is_a(of, to) {
                    return Some(value);
                }
                if self.is_bit(of) && self.bits_width(to) == Some(1) {
                    return Some(self.bits(vec![value]));
                }
                None
            }
        }
    }

    pub fn cast_to(&mut self, value: Value, to: Type) -> Flow<Option<Value>> {
        match self.node(value) {
            Node::Unset => return Ok(Some(value)),
            Node::Argument { value: inner, .. } => {
                let inner = *inner;
                return self.cast_to(inner, to);
            }
            _ => {}
        }
        let of = self.node_types[value.index()];
        if self.type_is_a(of, to) {
            return Ok(Some(value));
        }
        if let Some(converted) = self.convert_to(value, to) {
            return Ok(Some(converted));
        }
        if !self.type_convertible(of, to) {
            return Ok(None);
        }
        let made = self.unary(UnaryOperator::Cast, value, to);
        Ok(Some(self.fold(made, None, false)?))
    }

    pub fn convert_bit_range(&mut self, value: Value, bits: &[u32]) -> Option<Value> {
        match self.node(value).clone() {
            Node::Unset | Node::Argument { .. } => None,
            Node::Bits(held) => {
                let mut made = Vec::with_capacity(bits.len());
                for bit in bits {
                    made.push(*held.get(*bit as usize)?);
                }
                Some(self.bits(made))
            }
            Node::Integer(integer) => {
                let mut made = Vec::with_capacity(bits.len());
                for bit in bits {
                    if *bit >= 64 {
                        return None;
                    }
                    made.push(self.bit((integer as u64) & (1u64 << bit) != 0));
                }
                Some(self.bits(made))
            }
            _ => {
                let of = self.node_types[value.index()];
                let width = self.bits_width(of)?;
                let mut made = Vec::with_capacity(bits.len());
                for bit in bits {
                    if *bit >= width {
                        return None;
                    }
                    made.push(self.variable_bit(value, *bit));
                }
                Some(self.bits(made))
            }
        }
    }

    fn as_integer(&mut self, value: Value) -> Option<i64> {
        let of = self.integer_type;
        let converted = self.convert_to(value, of)?;
        self.integer_of(converted)
    }
}

impl RecordKeeper {
    pub fn resolve(&mut self, value: Value, resolver: &mut dyn Resolver) -> Flow<Value> {
        match self.node(value).clone() {
            Node::Unset
            | Node::Bit(_)
            | Node::Integer(_)
            | Node::String { .. }
            | Node::Definition(_) => Ok(value),
            Node::Argument {
                value: inner,
                name,
                position,
            } => {
                let resolved = self.resolve(inner, resolver)?;
                if resolved != inner {
                    Ok(self.argument(resolved, name, position))
                } else {
                    Ok(value)
                }
            }
            Node::AnonymousName(_) => {
                let found = resolver.resolve(self, value)?.unwrap_or(value);
                if resolver.is_final() {
                    if let Node::AnonymousName(number) = self.node(found) {
                        let text = format!("anonymous_{number}");
                        return Ok(self.string(&text));
                    }
                }
                Ok(found)
            }
            Node::Bits(bits) => {
                let mut changed = false;
                let mut made = Vec::with_capacity(bits.len());
                let mut cached_reference: Option<Value> = None;
                let mut cached_resolution = value;
                for current in bits.iter() {
                    let mut next = if let Node::VariableBit { variable, bit } = self.node(*current)
                    {
                        let (variable, bit) = (*variable, *bit);
                        if cached_reference != Some(variable) {
                            cached_reference = Some(variable);
                            cached_resolution = self.resolve(variable, resolver)?;
                        }
                        self.get_bit(cached_resolution, bit)
                    } else {
                        let resolved = self.resolve(*current, resolver)?;
                        self.get_bit(resolved, 0)
                    };
                    if self.is_unset(next) && resolver.keep_unset_bits() {
                        next = *current;
                    }
                    changed |= next != *current;
                    made.push(next);
                }
                if changed {
                    Ok(self.bits(made))
                } else {
                    Ok(value)
                }
            }
            Node::List { elements, element } => {
                let mut changed = false;
                let mut made = Vec::with_capacity(elements.len());
                for current in elements.iter() {
                    let next = self.resolve(*current, resolver)?;
                    changed |= next != *current;
                    made.push(next);
                }
                if changed {
                    Ok(self.list(made, element))
                } else {
                    Ok(value)
                }
            }
            Node::Unary {
                operator,
                operand,
                result,
            } => {
                let resolved = self.resolve(operand, resolver)?;
                if resolved != operand || (resolver.is_final() && operator == UnaryOperator::Cast) {
                    let made = self.unary(operator, resolved, result);
                    let current = resolver.current_record();
                    let finished = resolver.is_final();
                    return self.fold(made, current, finished);
                }
                Ok(value)
            }
            Node::Binary {
                operator,
                left,
                right,
                result,
            } => {
                let new_left = self.resolve(left, resolver)?;
                if operator == BinaryOperator::And || operator == BinaryOperator::Or {
                    if let Some(integer) = self.as_integer(new_left) {
                        if (operator == BinaryOperator::And && integer == 0)
                            || (operator == BinaryOperator::Or && integer == -1)
                        {
                            return Ok(self.integer(integer));
                        }
                    }
                }
                let new_right = self.resolve(right, resolver)?;
                if new_left != left || new_right != right {
                    let made = self.binary(operator, new_left, new_right, result);
                    let current = resolver.current_record();
                    return self.fold(made, current, false);
                }
                Ok(value)
            }
            Node::Ternary {
                operator,
                left,
                middle,
                right,
                result,
            } => {
                let new_left = self.resolve(left, resolver)?;
                if operator == TernaryOperator::If && new_left != left {
                    if let Some(integer) = self.as_integer(new_left) {
                        if integer != 0 {
                            return self.resolve(middle, resolver);
                        }
                        return self.resolve(right, resolver);
                    }
                }
                let new_middle = self.resolve(middle, resolver)?;
                let new_right = if matches!(
                    operator,
                    TernaryOperator::ForEach | TernaryOperator::Filter | TernaryOperator::Sort
                ) {
                    let mut shadow = ShadowResolver {
                        inner: resolver,
                        shadowed: vec![new_left],
                    };
                    self.resolve(right, &mut shadow)?
                } else {
                    self.resolve(right, resolver)?
                };
                if new_left != left || new_middle != middle || new_right != right {
                    let made = self.ternary(operator, new_left, new_middle, new_right, result);
                    let current = resolver.current_record();
                    return self.fold(made, current, false);
                }
                Ok(value)
            }
            Node::Condition {
                conditions,
                values,
                result,
            } => {
                let mut new_conditions = Vec::new();
                let mut new_values = Vec::new();
                let mut changed = false;
                for (condition, item) in conditions.iter().zip(values.iter()) {
                    let new_condition = self.resolve(*condition, resolver)?;
                    new_conditions.push(new_condition);
                    changed |= new_condition != *condition;
                    let new_value = self.resolve(*item, resolver)?;
                    new_values.push(new_value);
                    changed |= new_value != *item;
                    if let Some(integer) = self.as_integer(new_condition) {
                        if integer != 0 {
                            changed = true;
                            break;
                        }
                    }
                }
                if changed {
                    let made = self.condition(new_conditions, new_values, result);
                    let current = resolver.current_record();
                    return self.fold(made, current, false);
                }
                Ok(value)
            }
            Node::Fold {
                start,
                list,
                accumulator,
                element,
                expression,
                result,
            } => {
                let new_start = self.resolve(start, resolver)?;
                let new_list = self.resolve(list, resolver)?;
                let new_expression = {
                    let mut shadow = ShadowResolver {
                        inner: resolver,
                        shadowed: vec![accumulator, element],
                    };
                    self.resolve(expression, &mut shadow)?
                };
                if start == new_start && list == new_list && expression == new_expression {
                    return Ok(value);
                }
                let made = self.fold_node(
                    new_start,
                    new_list,
                    accumulator,
                    element,
                    new_expression,
                    result,
                );
                let current = resolver.current_record();
                self.fold(made, current, false)
            }
            Node::IsA {
                checked,
                expression,
            } => {
                let resolved = self.resolve(expression, resolver)?;
                if resolved != expression {
                    let made = self.is_a(checked, resolved);
                    return self.fold(made, None, false);
                }
                Ok(value)
            }
            Node::Exists {
                checked,
                expression,
            } => {
                let resolved = self.resolve(expression, resolver)?;
                if resolved != expression || resolver.is_final() {
                    let made = self.exists(checked, resolved);
                    let current = resolver.current_record();
                    let finished = resolver.is_final();
                    return self.fold(made, current, finished);
                }
                Ok(value)
            }
            Node::Instances { checked, pattern } => {
                let resolved = self.resolve(pattern, resolver)?;
                if resolved != pattern || resolver.is_final() {
                    let made = self.instances(checked, resolved);
                    let current = resolver.current_record();
                    let finished = resolver.is_final();
                    return self.fold(made, current, finished);
                }
                Ok(value)
            }
            Node::Variable { name, .. } => Ok(resolver.resolve(self, name)?.unwrap_or(value)),
            Node::VariableBit { variable, bit } => {
                let resolved = self.resolve(variable, resolver)?;
                if resolved != variable {
                    return Ok(self.get_bit(resolved, bit));
                }
                Ok(value)
            }
            Node::VariableDefinition { class, arguments } => {
                let mut tracker = TrackUnresolvedResolver {
                    inner: Some(resolver),
                    found_unresolved: false,
                };
                let mut changed = false;
                let mut made = Vec::with_capacity(arguments.len());
                for argument in arguments.iter() {
                    let next = self.resolve(*argument, &mut tracker)?;
                    changed |= next != *argument;
                    made.push(next);
                }
                if changed {
                    let location = self
                        .variable_definitions
                        .get(&value)
                        .map(|state| state.location)
                        .unwrap_or_default();
                    let fresh = self.variable_definition(location, class, made);
                    if !tracker.found_unresolved {
                        return self.instantiate(fresh);
                    }
                    return Ok(fresh);
                }
                Ok(value)
            }
            Node::Field { record, name } => {
                let resolved = self.resolve(record, resolver)?;
                if resolved != record {
                    let made = self.field_reference(resolved, name);
                    let current = resolver.current_record();
                    return self.fold(made, current, false);
                }
                Ok(value)
            }
            Node::Dag {
                operator,
                name,
                arguments,
                names,
            } => {
                let mut changed = false;
                let mut made = Vec::with_capacity(arguments.len());
                for argument in arguments.iter() {
                    let next = self.resolve(*argument, resolver)?;
                    changed |= next != *argument;
                    made.push(next);
                }
                let new_operator = self.resolve(operator, resolver)?;
                if new_operator != operator || changed {
                    return Ok(self.dag(new_operator, name, made, names.to_vec()));
                }
                Ok(value)
            }
        }
    }

    fn apply_item(
        &mut self,
        variable: Value,
        item: Value,
        expression: Value,
        current: Option<RecordIndex>,
    ) -> Flow<Value> {
        let mut resolver = MapResolver::new(current);
        resolver.set(variable, item);
        self.resolve(expression, &mut resolver)
    }

    fn for_each_dag(
        &mut self,
        variable: Value,
        dag: Value,
        expression: Value,
        current: Option<RecordIndex>,
    ) -> Flow<Value> {
        let Node::Dag {
            operator,
            name,
            arguments,
            names,
        } = self.node(dag).clone()
        else {
            return Ok(dag);
        };
        let mut changed = false;
        let new_operator = self.apply_item(variable, operator, expression, current)?;
        if new_operator != operator {
            changed = true;
        }
        let mut made = Vec::with_capacity(arguments.len());
        for argument in arguments.iter() {
            let next = if matches!(self.node(*argument), Node::Dag { .. }) {
                self.for_each_dag(variable, *argument, expression, current)?
            } else {
                self.apply_item(variable, *argument, expression, current)?
            };
            if next != *argument {
                changed = true;
            }
            made.push(next);
        }
        if changed {
            return Ok(self.dag(new_operator, name, made, names.to_vec()));
        }
        Ok(dag)
    }

    fn dag_argument_number(&self, dag: Value, key: Value) -> Result<usize, String> {
        let Node::Dag {
            arguments, names, ..
        } = self.node(dag)
        else {
            return Err(String::new());
        };
        if let Node::Integer(position) = self.node(key) {
            let position = *position;
            if position < 0 {
                return Err(format!("index {position} is negative"));
            }
            if position as usize >= arguments.len() {
                return Err(format!(
                    "index {position} is out of range (dag has {} arguments)",
                    arguments.len()
                ));
            }
            return Ok(position as usize);
        }
        let wanted = self.string_of(key).unwrap_or("");
        for (index, name) in names.iter().enumerate() {
            if let Some(name) = name {
                if self.string_of(*name) == Some(wanted) {
                    return Ok(index);
                }
            }
        }
        Err(format!("key '{wanted}' is not found"))
    }

    fn compare(&mut self, operator: BinaryOperator, left: Value, right: Value) -> Option<bool> {
        let left_integer = self.as_integer(left);
        let right_integer = self.as_integer(right);
        if let (Some(left), Some(right)) = (left_integer, right_integer) {
            return Some(match operator {
                BinaryOperator::Equal => left == right,
                BinaryOperator::NotEqual => left != right,
                BinaryOperator::LessOrEqual => left <= right,
                BinaryOperator::Less => left < right,
                BinaryOperator::GreaterOrEqual => left >= right,
                _ => left > right,
            });
        }
        if let (Some(left), Some(right)) = (self.string_of(left), self.string_of(right)) {
            return Some(match operator {
                BinaryOperator::Equal => left == right,
                BinaryOperator::NotEqual => left != right,
                BinaryOperator::LessOrEqual => left <= right,
                BinaryOperator::Less => left < right,
                BinaryOperator::GreaterOrEqual => left >= right,
                _ => left > right,
            });
        }
        if operator == BinaryOperator::Equal || operator == BinaryOperator::NotEqual {
            if let (Node::Definition(left), Node::Definition(right)) =
                (self.node(left), self.node(right))
            {
                let same = left == right;
                return Some(if operator == BinaryOperator::Equal {
                    same
                } else {
                    !same
                });
            }
        }
        None
    }

    pub fn fold(
        &mut self,
        value: Value,
        current: Option<RecordIndex>,
        finished: bool,
    ) -> Flow<Value> {
        match self.node(value).clone() {
            Node::Unary {
                operator,
                operand,
                result,
            } => self.fold_unary(value, operator, operand, result, current, finished),
            Node::Binary {
                operator,
                left,
                right,
                result,
            } => self.fold_binary(value, operator, left, right, result, current),
            Node::Ternary {
                operator,
                left,
                middle,
                right,
                result,
            } => self.fold_ternary(value, operator, left, middle, right, result, current),
            Node::Condition {
                conditions,
                values,
                result,
            } => {
                for (condition, item) in conditions.iter().zip(values.iter()) {
                    match self.as_integer(*condition) {
                        Some(integer) => {
                            if integer != 0 {
                                return Ok(self.convert_to(*item, result).unwrap_or(*item));
                            }
                        }
                        None => return Ok(value),
                    }
                }
                let locations = self.record_locations(current);
                let name = current
                    .map(|record| self.record_name_text(record))
                    .unwrap_or_default();
                let text = format!(
                    "{name} does not have any true condition in:{}",
                    self.text_of(value)
                );
                Err(self.fatal(&locations, text))
            }
            Node::Fold {
                start,
                list,
                accumulator,
                element,
                expression,
                ..
            } => {
                let Node::List { elements, .. } = self.node(list).clone() else {
                    return Ok(value);
                };
                let mut gathered = start;
                for item in elements.iter() {
                    let mut resolver = MapResolver::new(current);
                    resolver.set(accumulator, gathered);
                    resolver.set(element, *item);
                    gathered = self.resolve(expression, &mut resolver)?;
                }
                Ok(gathered)
            }
            Node::IsA {
                checked,
                expression,
            } => {
                if !self.typed(expression) {
                    return Ok(value);
                }
                let of = self.node_types[expression.index()];
                if self.type_convertible(of, checked) {
                    return Ok(self.integer(1));
                }
                if self.record_classes(checked).is_some() {
                    if (!self.type_convertible(checked, of) && self.is_concrete(expression))
                        || matches!(self.node(expression), Node::Definition(_))
                    {
                        return Ok(self.integer(0));
                    }
                } else {
                    return Ok(self.integer(0));
                }
                Ok(value)
            }
            Node::Exists {
                checked,
                expression,
            } => {
                let Some(name) = self.string_of(expression).map(str::to_string) else {
                    return Ok(value);
                };
                if let Some(record) = self.find_definition(&name) {
                    let definition = self.definition_of(record);
                    let of = self.node_types[definition.index()];
                    let answer = self.type_is_a(of, checked);
                    return Ok(self.integer(answer as i64));
                }
                if let Some(record) = current {
                    if self.names_current_record(expression, record) {
                        if !finished {
                            return Ok(value);
                        }
                        let of = self.record_type_of(record);
                        let answer = self.type_is_a(of, checked);
                        return Ok(self.integer(answer as i64));
                    }
                }
                if finished {
                    return Ok(self.integer(0));
                }
                Ok(value)
            }
            Node::Instances { checked, pattern } => {
                if current.is_some() && !finished {
                    return Ok(value);
                }
                let Some(expression) = self.string_of(pattern).map(str::to_string) else {
                    return Ok(value);
                };
                let Some(regex) = Pattern::compile(&expression) else {
                    let text = format!("invalid regex '{expression}'");
                    return Err(self.fatal(&[], text));
                };
                let class_name = self.type_name(checked);
                let mut selected = Vec::new();
                for record in self.derived_definitions(&class_name) {
                    if regex.search(self.record_name_text(record).as_bytes()) {
                        selected.push(self.definition_of(record));
                    }
                }
                Ok(self.list(selected, checked))
            }
            Node::VariableDefinition { arguments, .. } => {
                if let Some(definition) = self
                    .variable_definitions
                    .get(&value)
                    .and_then(|state| state.definition)
                {
                    return Ok(definition);
                }
                let mut tracker = TrackUnresolvedResolver {
                    inner: None,
                    found_unresolved: false,
                };
                for argument in arguments.iter() {
                    self.resolve(*argument, &mut tracker)?;
                }
                if !tracker.found_unresolved {
                    return self.instantiate(value);
                }
                Ok(value)
            }
            Node::Field { record, name } => {
                if let Node::Definition(definition) = self.node(record) {
                    let definition = *definition;
                    if Some(definition) == current {
                        let locations = self.record_locations(current);
                        let text = format!(
                            "Attempting to access field '{}' of '{}' is a forbidden self-reference",
                            self.unquoted(name),
                            self.text_of(record)
                        );
                        return Err(self.fatal(&locations, text));
                    }
                    let held = self.record(definition);
                    if let Some(at) = held.field(name) {
                        let found = held.fields[at].value;
                        if self.is_concrete(found) {
                            return Ok(found);
                        }
                    }
                }
                Ok(value)
            }
            _ => Ok(value),
        }
    }

    fn names_current_record(&mut self, name: Value, record: RecordIndex) -> bool {
        let record_name = self.record(record).name;
        if name == record_name {
            return true;
        }
        if let Node::AnonymousName(number) = self.node(record_name) {
            let text = format!("anonymous_{number}");
            return self.string_of(name) == Some(text.as_str());
        }
        false
    }

    fn fold_unary(
        &mut self,
        value: Value,
        operator: UnaryOperator,
        operand: Value,
        result: Type,
        current: Option<RecordIndex>,
        finished: bool,
    ) -> Flow<Value> {
        match operator {
            UnaryOperator::Represent => {
                if self.is_concrete(operand) {
                    let text = if let Node::Definition(record) = self.node(operand) {
                        self.record_text(*record)
                    } else {
                        self.text_of(operand)
                    };
                    return Ok(self.string(&text));
                }
            }
            UnaryOperator::ToLower => {
                if let Some(text) = self.string_of(operand) {
                    let lower = text.to_ascii_lowercase();
                    return Ok(self.string(&lower));
                }
            }
            UnaryOperator::ToUpper => {
                if let Some(text) = self.string_of(operand) {
                    let upper = text.to_ascii_uppercase();
                    return Ok(self.string(&upper));
                }
            }
            UnaryOperator::Cast => {
                if self.is_string_type(result) {
                    match self.node(operand) {
                        Node::String { .. } => return Ok(operand),
                        Node::Definition(_) => {
                            let text = self.text_of(operand);
                            return Ok(self.string(&text));
                        }
                        _ => {
                            if let Some(integer) = self.as_integer(operand) {
                                return Ok(self.string(&integer.to_string()));
                            }
                        }
                    }
                } else if self.record_classes(result).is_some() {
                    if let Some(name) = self.string_of(operand).map(str::to_string) {
                        let mut found = self.find_definition(&name);
                        if found.is_none() {
                            if let Some(record) = current {
                                if self.names_current_record(operand, record) {
                                    if !finished {
                                        return Ok(value);
                                    }
                                    found = Some(record);
                                }
                            }
                        }
                        let locations = self.record_locations(current);
                        let Some(record) = found else {
                            if finished {
                                let text = format!("Undefined reference to record: '{name}'\n");
                                return Err(self.fatal(&locations, text));
                            }
                            return Ok(value);
                        };
                        let definition = self.definition_of(record);
                        let of = self.node_types[definition.index()];
                        if !self.type_is_a(of, result) {
                            let text = format!(
                                "Expected type '{}', got '{}' in: {}\n",
                                self.type_name(result),
                                self.type_name(of),
                                self.text_of(value)
                            );
                            return Err(self.fatal(&locations, text));
                        }
                        return Ok(definition);
                    }
                }
                if let Some(converted) = self.convert_to(operand, result) {
                    return Ok(converted);
                }
            }
            UnaryOperator::Initialized => {
                if self.is_unset(operand) {
                    return Ok(self.integer(0));
                }
                if self.is_concrete(operand) {
                    return Ok(self.integer(1));
                }
            }
            UnaryOperator::Not => {
                if let Some(integer) = self.as_integer(operand) {
                    return Ok(self.integer(if integer != 0 { 0 } else { 1 }));
                }
            }
            UnaryOperator::Head => {
                if let Node::List { elements, .. } = self.node(operand) {
                    if let Some(first) = elements.first() {
                        return Ok(*first);
                    }
                }
            }
            UnaryOperator::Tail => {
                if let Node::List { elements, element } = self.node(operand).clone() {
                    if !elements.is_empty() {
                        return Ok(self.list(elements[1..].to_vec(), element));
                    }
                }
            }
            UnaryOperator::Size | UnaryOperator::Empty => {
                let size = match self.node(operand) {
                    Node::List { elements, .. } => Some(elements.len()),
                    Node::Dag { arguments, .. } => Some(arguments.len()),
                    Node::String { text, .. } => Some(self.text(*text).len()),
                    _ => None,
                };
                if let Some(size) = size {
                    if operator == UnaryOperator::Size {
                        return Ok(self.integer(size as i64));
                    }
                    return Ok(self.integer((size == 0) as i64));
                }
            }
            UnaryOperator::GetDagOperator => {
                if let Node::Dag { operator, .. } = self.node(operand) {
                    let operator = *operator;
                    if !self.typed(operator) {
                        return Ok(value);
                    }
                    let of = self.node_types[operator.index()];
                    if !self.type_is_a(of, result) {
                        let locations = self.record_locations(current);
                        let text = format!(
                            "Expected type '{}', got '{}' in: {}\n",
                            self.type_name(result),
                            self.type_name(of),
                            self.text_of(value)
                        );
                        return Err(self.fatal(&locations, text));
                    }
                    return Ok(operator);
                }
            }
            UnaryOperator::GetDagOperatorName => {
                if let Node::Dag { name, .. } = self.node(operand) {
                    return Ok(name.unwrap_or(self.unset));
                }
            }
            UnaryOperator::LogTwo => {
                if let Some(integer) = self.as_integer(operand) {
                    if integer <= 0 {
                        let locations = self.record_locations(current);
                        return Err(self.fatal(
                            &locations,
                            "Illegal operation: logtwo is undefined on arguments less than or equal to 0",
                        ));
                    }
                    let logarithm = 63 - (integer as u64).leading_zeros() as i64;
                    return Ok(self.integer(logarithm));
                }
            }
            UnaryOperator::ListFlatten => {
                if let Node::List { elements, element } = self.node(operand).clone() {
                    let Some(inner) = self.list_element(element) else {
                        return Ok(operand);
                    };
                    let mut flattened = Vec::new();
                    let mut complete = true;
                    for item in elements.iter() {
                        match self.node(*item) {
                            Node::List { elements, .. } => {
                                flattened.extend(elements.iter().copied())
                            }
                            _ => {
                                complete = false;
                                break;
                            }
                        }
                    }
                    if complete {
                        return Ok(self.list(flattened, inner));
                    }
                }
            }
        }
        Ok(value)
    }

    fn fold_binary(
        &mut self,
        value: Value,
        operator: BinaryOperator,
        left: Value,
        right: Value,
        result: Type,
        current: Option<RecordIndex>,
    ) -> Flow<Value> {
        match operator {
            BinaryOperator::Concat => {
                let (
                    Node::Dag {
                        operator: left_operator,
                        name: left_name,
                        arguments: left_arguments,
                        names: left_names,
                    },
                    Node::Dag {
                        operator: right_operator,
                        name: right_name,
                        arguments: right_arguments,
                        names: right_names,
                    },
                ) = (self.node(left).clone(), self.node(right).clone())
                else {
                    return Ok(value);
                };
                let left_definition = matches!(self.node(left_operator), Node::Definition(_));
                let right_definition = matches!(self.node(right_operator), Node::Definition(_));
                if (!left_definition && !self.is_unset(left_operator))
                    || (!right_definition && !self.is_unset(right_operator))
                {
                    return Ok(value);
                }
                if left_definition && right_definition && left_operator != right_operator {
                    let text = format!(
                        "Concatenated Dag operators do not match: '{}' vs. '{}'",
                        self.text_of(left),
                        self.text_of(right)
                    );
                    return Err(self.fatal(&[], text));
                }
                let chosen = if left_definition {
                    left_operator
                } else if right_definition {
                    right_operator
                } else {
                    self.unset
                };
                let mut arguments = left_arguments.to_vec();
                arguments.extend(right_arguments.iter().copied());
                let mut names = left_names.to_vec();
                names.extend(right_names.iter().copied());
                let name = left_name.or(right_name);
                Ok(self.dag(chosen, name, arguments, names))
            }
            BinaryOperator::Match => {
                let (Some(text), Some(expression)) = (
                    self.string_of(left).map(str::to_string),
                    self.string_of(right).map(str::to_string),
                ) else {
                    return Ok(value);
                };
                let Some(regex) = Pattern::compile(&expression) else {
                    let text = format!("invalid regex '{expression}'");
                    return Err(self.fatal(&[], text));
                };
                Ok(self.bit(regex.search(text.as_bytes())))
            }
            BinaryOperator::ListConcat => {
                if let (
                    Node::List {
                        elements: first,
                        element,
                    },
                    Node::List {
                        elements: second, ..
                    },
                ) = (self.node(left).clone(), self.node(right).clone())
                {
                    let mut joined = first.to_vec();
                    joined.extend(second.iter().copied());
                    return Ok(self.list(joined, element));
                }
                Ok(value)
            }
            BinaryOperator::ListSplat => {
                if !self.typed(left) {
                    return Ok(value);
                }
                let Some(count) = self.integer_of(right) else {
                    return Ok(value);
                };
                if count < 0 {
                    let text = format!("!listsplat count {count} is negative");
                    return Err(self.fatal(&[], text));
                }
                let of = self.node_types[left.index()];
                Ok(self.list(vec![left; count as usize], of))
            }
            BinaryOperator::ListRemove => {
                if let (
                    Node::List {
                        elements: first,
                        element,
                    },
                    Node::List {
                        elements: second, ..
                    },
                ) = (self.node(left).clone(), self.node(right).clone())
                {
                    let mut kept = Vec::new();
                    for item in first.iter() {
                        let mut found = false;
                        for other in second.iter() {
                            if self.compare(BinaryOperator::Equal, *item, *other) == Some(true) {
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            kept.push(*item);
                        }
                    }
                    return Ok(self.list(kept, element));
                }
                Ok(value)
            }
            BinaryOperator::ListElement => {
                let (Node::List { elements, .. }, Some(index)) =
                    (self.node(left), self.integer_of(right))
                else {
                    return Ok(value);
                };
                if index < 0 || index as usize >= elements.len() {
                    return Ok(value);
                }
                Ok(elements[index as usize])
            }
            BinaryOperator::ListSlice => {
                let (
                    Node::List { elements, element },
                    Node::List {
                        elements: indices, ..
                    },
                ) = (self.node(left).clone(), self.node(right).clone())
                else {
                    return Ok(value);
                };
                let mut picked = Vec::with_capacity(indices.len());
                for index in indices.iter() {
                    let Some(index) = self.integer_of(*index) else {
                        return Ok(value);
                    };
                    if index < 0 || index as usize >= elements.len() {
                        return Ok(value);
                    }
                    picked.push(elements[index as usize]);
                }
                Ok(self.list(picked, element))
            }
            BinaryOperator::RangeClosed => {
                let (Some(start), Some(end)) = (self.integer_of(left), self.integer_of(right))
                else {
                    return Ok(value);
                };
                let mut items = Vec::new();
                if start <= end {
                    let mut at = start;
                    loop {
                        items.push(self.integer(at));
                        if at == end {
                            break;
                        }
                        at += 1;
                    }
                } else {
                    let mut at = start;
                    loop {
                        items.push(self.integer(at));
                        if at == end {
                            break;
                        }
                        at -= 1;
                    }
                }
                let of = self.integer_type;
                Ok(self.list(items, of))
            }
            BinaryOperator::StringConcat => {
                if let (
                    Node::String {
                        text: first,
                        code: first_code,
                    },
                    Node::String {
                        text: second,
                        code: second_code,
                    },
                ) = (self.node(left).clone(), self.node(right).clone())
                {
                    let joined = format!("{}{}", self.text(first), self.text(second));
                    return Ok(self.string_with_format(&joined, first_code || second_code));
                }
                Ok(value)
            }
            BinaryOperator::Interleave => {
                let (Node::List { elements, element }, Some(delimiter)) = (
                    self.node(left).clone(),
                    self.string_of(right).map(str::to_string),
                ) else {
                    return Ok(value);
                };
                if elements.is_empty() {
                    return Ok(self.string(""));
                }
                if self.is_string_type(element) {
                    let mut joined = String::new();
                    let mut code = false;
                    for (index, item) in elements.iter().enumerate() {
                        let Node::String {
                            text,
                            code: item_code,
                        } = self.node(*item)
                        else {
                            return Ok(value);
                        };
                        if index > 0 {
                            joined.push_str(&delimiter);
                            code |= *item_code;
                        }
                        joined.push_str(self.text(*text));
                    }
                    return Ok(self.string_with_format(&joined, code));
                }
                let mut joined = String::new();
                for (index, item) in elements.iter().enumerate() {
                    let Some(integer) = self.as_integer(*item) else {
                        return Ok(value);
                    };
                    if index > 0 {
                        joined.push_str(&delimiter);
                    }
                    joined.push_str(&integer.to_string());
                }
                Ok(self.string(&joined))
            }
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessOrEqual
            | BinaryOperator::Less
            | BinaryOperator::GreaterOrEqual
            | BinaryOperator::Greater => match self.compare(operator, left, right) {
                Some(answer) => Ok(self.bit(answer)),
                None => Ok(value),
            },
            BinaryOperator::GetDagArgument => {
                if !matches!(self.node(left), Node::Dag { .. })
                    || !matches!(self.node(right), Node::Integer(_) | Node::String { .. })
                {
                    return Ok(value);
                }
                match self.dag_argument_number(left, right) {
                    Err(problem) => {
                        let locations = self.record_locations(current);
                        Err(self.fatal(&locations, format!("!getdagarg {problem}")))
                    }
                    Ok(number) => {
                        let Node::Dag { arguments, .. } = self.node(left) else {
                            return Ok(value);
                        };
                        let argument = arguments[number];
                        if self.typed(argument) {
                            let of = self.node_types[argument.index()];
                            if !self.type_convertible(of, result) {
                                return Ok(self.unset);
                            }
                        }
                        Ok(argument)
                    }
                }
            }
            BinaryOperator::GetDagName => {
                let (
                    Node::Dag {
                        arguments, names, ..
                    },
                    Some(position),
                ) = (self.node(left).clone(), self.integer_of(right))
                else {
                    return Ok(value);
                };
                if position < 0 || position as usize >= arguments.len() {
                    let locations = self.record_locations(current);
                    let text = format!(
                        "!getdagname index is out of range 0...{}: {position}",
                        arguments.len() as i64 - 1
                    );
                    self.report(MessageKind::Error, &locations, text);
                    return Ok(self.unset);
                }
                Ok(names[position as usize].unwrap_or(self.unset))
            }
            BinaryOperator::SetDagOperator => {
                let Node::Dag {
                    arguments, names, ..
                } = self.node(left).clone()
                else {
                    return Ok(value);
                };
                if !matches!(self.node(right), Node::Definition(_)) {
                    return Ok(value);
                }
                Ok(self.dag(right, None, arguments.to_vec(), names.to_vec()))
            }
            BinaryOperator::SetDagOperatorName => {
                let Node::Dag {
                    operator,
                    arguments,
                    names,
                    ..
                } = self.node(left).clone()
                else {
                    return Ok(value);
                };
                if self.string_of(right).is_none() {
                    return Ok(value);
                }
                Ok(self.dag(operator, Some(right), arguments.to_vec(), names.to_vec()))
            }
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::And
            | BinaryOperator::Or
            | BinaryOperator::ExclusiveOr
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRightArithmetic
            | BinaryOperator::ShiftRightLogical => {
                let (Some(first), Some(second)) = (self.as_integer(left), self.as_integer(right))
                else {
                    return Ok(value);
                };
                let locations = self.record_locations(current);
                let answer = match operator {
                    BinaryOperator::Add => first.wrapping_add(second),
                    BinaryOperator::Subtract => first.wrapping_sub(second),
                    BinaryOperator::Multiply => first.wrapping_mul(second),
                    BinaryOperator::Divide => {
                        if second == 0 {
                            return Err(
                                self.fatal(&locations, "Illegal operation: division by zero")
                            );
                        }
                        if first == i64::MIN && second == -1 {
                            return Err(self.fatal(&locations, "Illegal operation: INT64_MIN / -1"));
                        }
                        first / second
                    }
                    BinaryOperator::And => first & second,
                    BinaryOperator::Or => first | second,
                    BinaryOperator::ExclusiveOr => first ^ second,
                    _ => {
                        if !(0..64).contains(&second) {
                            return Err(
                                self.fatal(&locations, "Illegal operation: out of bounds shift")
                            );
                        }
                        match operator {
                            BinaryOperator::ShiftLeft => ((first as u64) << second) as i64,
                            BinaryOperator::ShiftRightArithmetic => first >> second,
                            _ => ((first as u64) >> second) as i64,
                        }
                    }
                };
                Ok(self.integer(answer))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn fold_ternary(
        &mut self,
        value: Value,
        operator: TernaryOperator,
        left: Value,
        middle: Value,
        right: Value,
        result: Type,
        current: Option<RecordIndex>,
    ) -> Flow<Value> {
        match operator {
            TernaryOperator::Substitute => {
                match (
                    self.node(left).clone(),
                    self.node(middle).clone(),
                    self.node(right).clone(),
                ) {
                    (
                        Node::Definition(_),
                        Node::Definition(middle_record),
                        Node::Definition(right_record),
                    ) => {
                        let mut chosen = right_record;
                        if self.text_of(left) == self.text_of(right) {
                            chosen = middle_record;
                        }
                        Ok(self.definition_of(chosen))
                    }
                    (
                        Node::Variable { .. },
                        Node::Variable {
                            name: middle_name, ..
                        },
                        Node::Variable {
                            name: right_name, ..
                        },
                    ) => {
                        let mut chosen = self.unquoted(right_name);
                        if self.text_of(left) == self.text_of(right) {
                            chosen = self.unquoted(middle_name);
                        }
                        let name = self.string(&chosen);
                        Ok(self.variable(name, result))
                    }
                    (
                        Node::String { text: pattern, .. },
                        Node::String {
                            text: replacement, ..
                        },
                        Node::String { text: subject, .. },
                    ) => {
                        let pattern = self.text(pattern).to_string();
                        let replacement = self.text(replacement).to_string();
                        let mut text = self.text(subject).to_string();
                        let mut from = 0usize;
                        while let Some(found) = find_from(&text, &pattern, from) {
                            text.replace_range(found..found + pattern.len(), &replacement);
                            from = found + replacement.len();
                        }
                        Ok(self.string(&text))
                    }
                    _ => Ok(value),
                }
            }
            TernaryOperator::ForEach => {
                if matches!(self.node(middle), Node::Dag { .. }) {
                    return self.for_each_dag(left, middle, right, current);
                }
                if let Node::List { elements, .. } = self.node(middle).clone() {
                    let mut made = elements.to_vec();
                    for item in made.iter_mut() {
                        *item = self.apply_item(left, *item, right, current)?;
                    }
                    let element = self.list_element(result).unwrap_or(Type::NONE);
                    return Ok(self.list(made, element));
                }
                Ok(value)
            }
            TernaryOperator::Filter => {
                if let Node::List { elements, .. } = self.node(middle).clone() {
                    let mut kept = Vec::new();
                    for item in elements.iter() {
                        let include = self.apply_item(left, *item, right, current)?;
                        match self.as_integer(include) {
                            Some(integer) => {
                                if integer != 0 {
                                    kept.push(*item);
                                }
                            }
                            None => return Ok(value),
                        }
                    }
                    let element = self.list_element(result).unwrap_or(Type::NONE);
                    return Ok(self.list(kept, element));
                }
                Ok(value)
            }
            TernaryOperator::Sort => {
                let Node::List { elements, .. } = self.node(middle).clone() else {
                    return Ok(value);
                };
                let element = self.list_element(result).unwrap_or(Type::NONE);
                let mut keyed = Vec::with_capacity(elements.len());
                for item in elements.iter() {
                    let key = self.apply_item(left, *item, right, current)?;
                    keyed.push((key, *item));
                }
                if keyed.is_empty() {
                    return Ok(self.list(Vec::new(), element));
                }
                let use_integer = self.as_integer(keyed[0].0).is_some();
                let mut integer_keys = Vec::with_capacity(keyed.len());
                let mut string_keys = Vec::with_capacity(keyed.len());
                for (key, item) in &keyed {
                    if use_integer {
                        let Some(integer) = self.as_integer(*key) else {
                            return Ok(value);
                        };
                        integer_keys.push((integer, *item));
                    } else {
                        let Some(text) = self.string_of(*key) else {
                            return Ok(value);
                        };
                        string_keys.push((text.to_string(), *item));
                    }
                }
                let sorted: Vec<Value> = if use_integer {
                    integer_keys.sort_by_key(|entry| entry.0);
                    integer_keys.into_iter().map(|(_, item)| item).collect()
                } else {
                    string_keys
                        .sort_by(|first, second| first.0.as_bytes().cmp(second.0.as_bytes()));
                    string_keys.into_iter().map(|(_, item)| item).collect()
                };
                Ok(self.list(sorted, element))
            }
            TernaryOperator::If => match self.as_integer(left) {
                Some(integer) => Ok(if integer != 0 { middle } else { right }),
                None => Ok(value),
            },
            TernaryOperator::Dag => {
                let middle_list = match self.node(middle) {
                    Node::List { elements, .. } => Some(elements.clone()),
                    _ => None,
                };
                let right_list = match self.node(right) {
                    Node::List { elements, .. } => Some(elements.clone()),
                    _ => None,
                };
                let middle_unset = self.is_unset(middle);
                let right_unset = self.is_unset(right);
                if middle_unset && right_unset {
                    return Ok(value);
                }
                let middle_ok = middle_list.is_some() || middle_unset;
                let right_ok = right_list.is_some() || right_unset;
                let sizes_match = match (&middle_list, &right_list) {
                    (Some(first), Some(second)) => first.len() == second.len(),
                    _ => true,
                };
                if middle_ok && right_ok && sizes_match {
                    let size = middle_list
                        .as_ref()
                        .map(|list| list.len())
                        .or_else(|| right_list.as_ref().map(|list| list.len()))
                        .unwrap_or(0);
                    let mut arguments = Vec::with_capacity(size);
                    let mut names = Vec::with_capacity(size);
                    for index in 0..size {
                        let node = middle_list
                            .as_ref()
                            .map(|list| list[index])
                            .unwrap_or(self.unset);
                        let name = right_list
                            .as_ref()
                            .map(|list| list[index])
                            .unwrap_or(self.unset);
                        let is_string = self.string_of(name).is_some();
                        if !is_string && !self.is_unset(name) {
                            return Ok(value);
                        }
                        arguments.push(node);
                        names.push(if is_string { Some(name) } else { None });
                    }
                    return Ok(self.dag(left, None, arguments, names));
                }
                Ok(value)
            }
            TernaryOperator::Range => {
                let (Some(start), Some(end), Some(step)) = (
                    self.integer_of(left),
                    self.integer_of(middle),
                    self.integer_of(right),
                ) else {
                    return Ok(value);
                };
                if step == 0 {
                    let locations = self.record_locations(current);
                    self.report(MessageKind::Error, &locations, "Step of !range can't be 0");
                }
                let mut items = Vec::new();
                if start < end && step > 0 {
                    let mut at = start;
                    while at < end {
                        items.push(self.integer(at));
                        at = match at.checked_add(step) {
                            Some(next) => next,
                            None => break,
                        };
                    }
                } else if start > end && step < 0 {
                    let mut at = start;
                    while at > end {
                        items.push(self.integer(at));
                        at = match at.checked_add(step) {
                            Some(next) => next,
                            None => break,
                        };
                    }
                }
                let of = self.integer_type;
                Ok(self.list(items, of))
            }
            TernaryOperator::Substring => {
                let (Node::String { text, code }, Some(start), Some(length)) = (
                    self.node(left).clone(),
                    self.integer_of(middle),
                    self.integer_of(right),
                ) else {
                    return Ok(value);
                };
                let source = self.text(text).to_string();
                let size = source.len() as i64;
                if start < 0 || start > size {
                    let locations = self.record_locations(current);
                    let message =
                        format!("!substr start position is out of range 0...{size}: {start}");
                    self.report(MessageKind::Error, &locations, message);
                }
                if length < 0 {
                    let locations = self.record_locations(current);
                    self.report(
                        MessageKind::Error,
                        &locations,
                        "!substr length must be nonnegative",
                    );
                }
                let begin = if start < 0 || start > size {
                    source.len()
                } else {
                    start as usize
                };
                let available = source.len() - begin;
                let taken = if length < 0 {
                    available
                } else {
                    (length as u64).min(available as u64) as usize
                };
                let piece =
                    String::from_utf8_lossy(&source.as_bytes()[begin..begin + taken]).into_owned();
                Ok(self.string_with_format(&piece, code))
            }
            TernaryOperator::Find => {
                let (Some(source), Some(target), Some(start)) = (
                    self.string_of(left).map(str::to_string),
                    self.string_of(middle).map(str::to_string),
                    self.integer_of(right),
                ) else {
                    return Ok(value);
                };
                let size = source.len() as i64;
                if start < 0 || start > size {
                    let locations = self.record_locations(current);
                    let message =
                        format!("!find start position is out of range 0...{size}: {start}");
                    self.report(MessageKind::Error, &locations, message);
                }
                if start < 0 || start > size {
                    return Ok(self.integer(-1));
                }
                match find_from(&source, &target, start as usize) {
                    Some(found) => Ok(self.integer(found as i64)),
                    None => Ok(self.integer(-1)),
                }
            }
            TernaryOperator::SetDagArgument | TernaryOperator::SetDagName => {
                let Node::Dag {
                    operator: dag_operator,
                    name,
                    arguments,
                    names,
                } = self.node(left).clone()
                else {
                    return Ok(value);
                };
                if !matches!(self.node(middle), Node::Integer(_) | Node::String { .. }) {
                    return Ok(value);
                }
                match self.dag_argument_number(left, middle) {
                    Err(problem) => {
                        let locations = self.record_locations(current);
                        let prefix = if operator == TernaryOperator::SetDagArgument {
                            "!setdagarg"
                        } else {
                            "!setdagname"
                        };
                        Err(self.fatal(&locations, format!("{prefix} {problem}")))
                    }
                    Ok(number) => {
                        let mut arguments = arguments.to_vec();
                        let mut names = names.to_vec();
                        if operator == TernaryOperator::SetDagArgument {
                            arguments[number] = right;
                        } else {
                            names[number] = if self.string_of(right).is_some() {
                                Some(right)
                            } else {
                                None
                            };
                        }
                        Ok(self.dag(dag_operator, name, arguments, names))
                    }
                }
            }
        }
    }
}

fn find_from(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    let bytes = haystack.as_bytes();
    let wanted = needle.as_bytes();
    if from > bytes.len() {
        return None;
    }
    if wanted.is_empty() {
        return Some(from);
    }
    if wanted.len() > bytes.len() {
        return None;
    }
    (from..=bytes.len() - wanted.len()).find(|&at| &bytes[at..at + wanted.len()] == wanted)
}

enum PatternNode {
    Literal(u8),
    Any,
    Set(Box<[bool; 256]>),
    Start,
    End,
    Sequence(Vec<PatternNode>),
    Alternative(Vec<PatternNode>),
    Repeat(Box<PatternNode>, u32, Option<u32>),
}

pub struct Pattern {
    root: PatternNode,
}

struct PatternParser<'p> {
    text: &'p [u8],
    at: usize,
}

impl PatternParser<'_> {
    fn peek(&self) -> Option<u8> {
        self.text.get(self.at).copied()
    }

    fn alternation(&mut self, depth: u32) -> Option<PatternNode> {
        let mut branches = vec![self.sequence(depth)?];
        while self.peek() == Some(b'|') {
            self.at += 1;
            branches.push(self.sequence(depth)?);
        }
        if branches.len() == 1 {
            branches.pop()
        } else {
            Some(PatternNode::Alternative(branches))
        }
    }

    fn sequence(&mut self, depth: u32) -> Option<PatternNode> {
        let mut items = Vec::new();
        while let Some(current) = self.peek() {
            if current == b'|' || (current == b')' && depth > 0) {
                break;
            }
            let atom = self.atom(depth)?;
            items.push(self.quantified(atom)?);
        }
        if items.is_empty() {
            return None;
        }
        Some(PatternNode::Sequence(items))
    }

    fn number(&mut self) -> Option<u32> {
        let start = self.at;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.at += 1;
        }
        if start == self.at {
            return None;
        }
        std::str::from_utf8(&self.text[start..self.at])
            .ok()?
            .parse()
            .ok()
    }

    fn quantified(&mut self, mut atom: PatternNode) -> Option<PatternNode> {
        loop {
            match self.peek() {
                Some(b'*') => {
                    self.at += 1;
                    atom = PatternNode::Repeat(Box::new(atom), 0, None);
                }
                Some(b'+') => {
                    self.at += 1;
                    atom = PatternNode::Repeat(Box::new(atom), 1, None);
                }
                Some(b'?') => {
                    self.at += 1;
                    atom = PatternNode::Repeat(Box::new(atom), 0, Some(1));
                }
                Some(b'{')
                    if self
                        .text
                        .get(self.at + 1)
                        .is_some_and(|byte| byte.is_ascii_digit()) =>
                {
                    self.at += 1;
                    let low = self.number()?;
                    let mut high = Some(low);
                    if self.peek() == Some(b',') {
                        self.at += 1;
                        high = if self.peek() == Some(b'}') {
                            None
                        } else {
                            Some(self.number()?)
                        };
                    }
                    if self.peek() != Some(b'}') {
                        return None;
                    }
                    self.at += 1;
                    if high.is_some_and(|high| high < low) {
                        return None;
                    }
                    atom = PatternNode::Repeat(Box::new(atom), low, high);
                }
                _ => return Some(atom),
            }
        }
    }

    fn atom(&mut self, depth: u32) -> Option<PatternNode> {
        let current = self.peek()?;
        self.at += 1;
        match current {
            b'(' => {
                if self.peek() == Some(b')') {
                    return None;
                }
                let inner = self.alternation(depth + 1)?;
                if self.peek() != Some(b')') {
                    return None;
                }
                self.at += 1;
                Some(inner)
            }
            b'.' => Some(PatternNode::Any),
            b'^' => Some(PatternNode::Start),
            b'$' => Some(PatternNode::End),
            b'[' => self.bracket(),
            b'\\' => {
                let escaped = self.peek()?;
                self.at += 1;
                Some(PatternNode::Literal(escaped))
            }
            b'*' | b'+' | b'?' => None,
            other => Some(PatternNode::Literal(other)),
        }
    }

    fn bracket(&mut self) -> Option<PatternNode> {
        let mut set = Box::new([false; 256]);
        let mut negate = false;
        if self.peek() == Some(b'^') {
            negate = true;
            self.at += 1;
        }
        let mut first = true;
        loop {
            let current = self.peek()?;
            if current == b']' && !first {
                self.at += 1;
                break;
            }
            first = false;
            if current == b'[' && self.text.get(self.at + 1) == Some(&b':') {
                let close = self.text[self.at + 2..]
                    .windows(2)
                    .position(|pair| pair == b":]")?;
                let name = &self.text[self.at + 2..self.at + 2 + close];
                self.at += close + 4;
                let test: fn(u8) -> bool = match name {
                    b"alnum" => |byte| byte.is_ascii_alphanumeric(),
                    b"alpha" => |byte| byte.is_ascii_alphabetic(),
                    b"blank" => |byte| byte == b' ' || byte == b'\t',
                    b"cntrl" => |byte| byte < 32 || byte == 127,
                    b"digit" => |byte| byte.is_ascii_digit(),
                    b"graph" => |byte| (33..=126).contains(&byte),
                    b"lower" => |byte| byte.is_ascii_lowercase(),
                    b"print" => |byte| (32..=126).contains(&byte),
                    b"punct" => |byte| byte.is_ascii_punctuation(),
                    b"space" => |byte| matches!(byte, b' ' | b'\t' | b'\n' | 11 | 12 | b'\r'),
                    b"upper" => |byte| byte.is_ascii_uppercase(),
                    b"xdigit" => |byte| byte.is_ascii_hexdigit(),
                    _ => return None,
                };
                for byte in 0..=255u8 {
                    if test(byte) {
                        set[byte as usize] = true;
                    }
                }
                continue;
            }
            self.at += 1;
            if self.peek() == Some(b'-')
                && self.text.get(self.at + 1).is_some_and(|byte| *byte != b']')
            {
                let high = self.text[self.at + 1];
                self.at += 2;
                if high < current {
                    return None;
                }
                for byte in current..=high {
                    set[byte as usize] = true;
                }
            } else {
                set[current as usize] = true;
            }
        }
        if negate {
            for slot in set.iter_mut() {
                *slot = !*slot;
            }
        }
        Some(PatternNode::Set(set))
    }
}

impl Pattern {
    pub fn compile(text: &str) -> Option<Pattern> {
        let mut parser = PatternParser {
            text: text.as_bytes(),
            at: 0,
        };
        if text.is_empty() {
            return None;
        }
        let root = parser.alternation(0)?;
        if parser.at != parser.text.len() {
            return None;
        }
        Some(Pattern { root })
    }

    pub fn search(&self, text: &[u8]) -> bool {
        (0..=text.len()).any(|start| Pattern::step(&self.root, text, start, &mut |_| true))
    }

    fn step(
        node: &PatternNode,
        text: &[u8],
        at: usize,
        next: &mut dyn FnMut(usize) -> bool,
    ) -> bool {
        match node {
            PatternNode::Literal(byte) => at < text.len() && text[at] == *byte && next(at + 1),
            PatternNode::Any => at < text.len() && next(at + 1),
            PatternNode::Set(set) => at < text.len() && set[text[at] as usize] && next(at + 1),
            PatternNode::Start => at == 0 && next(at),
            PatternNode::End => at == text.len() && next(at),
            PatternNode::Sequence(items) => Pattern::sequence(items, text, at, next),
            PatternNode::Alternative(branches) => branches
                .iter()
                .any(|branch| Pattern::step(branch, text, at, next)),
            PatternNode::Repeat(inner, low, high) => {
                Pattern::repeat(inner, *low, *high, 0, text, at, next)
            }
        }
    }

    fn sequence(
        items: &[PatternNode],
        text: &[u8],
        at: usize,
        next: &mut dyn FnMut(usize) -> bool,
    ) -> bool {
        match items.split_first() {
            None => next(at),
            Some((first, rest)) => Pattern::step(first, text, at, &mut |after| {
                Pattern::sequence(rest, text, after, next)
            }),
        }
    }

    fn repeat(
        inner: &PatternNode,
        low: u32,
        high: Option<u32>,
        count: u32,
        text: &[u8],
        at: usize,
        next: &mut dyn FnMut(usize) -> bool,
    ) -> bool {
        if high.is_none_or(|high| count < high)
            && Pattern::step(inner, text, at, &mut |after| {
                (after != at || count < low)
                    && Pattern::repeat(inner, low, high, count + 1, text, after, next)
            })
        {
            return true;
        }
        count >= low && next(at)
    }
}

impl RecordKeeper {
    pub fn set_field_value(
        &mut self,
        record: RecordIndex,
        index: usize,
        value: Value,
    ) -> Flow<bool> {
        let field_type = self.record(record).fields[index].field_type;
        let Some(mut made) = self.cast_to(value, field_type)? else {
            return Ok(true);
        };
        if let Some(width) = self.bits_width(field_type) {
            if !matches!(self.node(made), Node::Bits(_)) {
                let bits = (0..width).map(|bit| self.get_bit(made, bit)).collect();
                made = self.bits(bits);
            }
        }
        self.record_mut(record).fields[index].value = made;
        Ok(false)
    }

    pub fn resolve_record(
        &mut self,
        record: RecordIndex,
        resolver: &mut dyn Resolver,
        skip: Option<usize>,
    ) -> Flow<()> {
        let old_name = self.record(record).name;
        let new_name = self.resolve(old_name, resolver)?;
        if new_name != old_name {
            self.set_record_name(record, new_name)?;
        }
        let count = self.record(record).fields.len();
        for index in 0..count {
            if Some(index) == skip {
                continue;
            }
            let current = self.record(record).fields[index].value;
            let resolved = self.resolve(current, resolver)?;
            if self.set_field_value(record, index, resolved)? {
                let field = self.record(record).fields[index].clone();
                let mut described = String::new();
                if let Some(of) = self.type_of(resolved) {
                    described = format!("of type '{}' ", self.type_name(of));
                }
                let text = format!(
                    "Invalid value {described}found when setting field '{}' of type '{}' after resolving references: {}\n",
                    self.unquoted(field.name),
                    self.type_name(field.field_type),
                    self.unquoted(resolved)
                );
                let locations = self.record(record).locations.clone();
                return Err(self.fatal(&locations, text));
            }
        }
        let assertions = self.record(record).assertions.len();
        for index in 0..assertions {
            let condition = self.record(record).assertions[index].condition;
            let condition = self.resolve(condition, resolver)?;
            self.record_mut(record).assertions[index].condition = condition;
            let message = self.record(record).assertions[index].message;
            let message = self.resolve(message, resolver)?;
            self.record_mut(record).assertions[index].message = message;
        }
        let dumps = self.record(record).dumps.len();
        for index in 0..dumps {
            let message = self.record(record).dumps[index].message;
            let message = self.resolve(message, resolver)?;
            self.record_mut(record).dumps[index].message = message;
        }
        Ok(())
    }

    pub fn resolve_record_final(
        &mut self,
        record: RecordIndex,
        new_name: Option<Value>,
    ) -> Flow<()> {
        let mut resolver = RecordResolver::new(record);
        resolver.name = new_name;
        self.resolve_record(record, &mut resolver, None)
    }

    fn instantiate(&mut self, variable_definition: Value) -> Flow<Value> {
        let location = match self.variable_definitions.get(&variable_definition) {
            Some(state) => {
                if let Some(definition) = state.definition {
                    return Ok(definition);
                }
                state.location
            }
            None => Location::default(),
        };
        let Node::VariableDefinition { class, arguments } = self.node(variable_definition).clone()
        else {
            return Ok(variable_definition);
        };
        let name = self.new_anonymous_name();
        let made = self.new_record(name, vec![location], RecordKind::AnonymousDefinition)?;
        let class_record = self.record(class).clone();
        {
            let target = self.record_mut(made);
            target.fields = class_record.fields.clone();
            target
                .assertions
                .extend(class_record.assertions.iter().cloned());
            target.dumps.extend(class_record.dumps.iter().cloned());
        }
        let mut resolver = MapResolver::new(Some(made));
        for argument in &class_record.template_arguments {
            if let Some(at) = self.record(made).field(*argument) {
                let default = self.record(made).fields[at].value;
                resolver.set(*argument, default);
                self.record_mut(made).fields.remove(at);
            }
        }
        for argument in arguments.iter() {
            let (value, name, position) = self.argument_parts(*argument);
            match name {
                Some(name) => resolver.set(name, value),
                None => {
                    if let Some(target) = class_record.template_arguments.get(position as usize) {
                        resolver.set(*target, value);
                    }
                }
            }
        }
        self.resolve_record(made, &mut resolver, None)?;
        let class_location = class_record.locations.last().copied().unwrap_or_default();
        self.record_mut(made)
            .superclasses
            .push((class, class_location));
        self.resolve_record_final(made, None)?;
        self.add_definition(made);
        self.check_assertions(made)?;
        self.emit_dumps(made)?;
        let definition = self.definition_of(made);
        if let Some(state) = self.variable_definitions.get_mut(&variable_definition) {
            state.definition = Some(definition);
        }
        Ok(definition)
    }

    pub fn check_assert(&mut self, location: Location, condition: Value, message: Value) -> bool {
        let Some(integer) = self.as_integer(condition) else {
            self.report(
                MessageKind::Error,
                &[location],
                "assert condition must of type bit, bits, or int.",
            );
            return true;
        };
        if integer == 0 {
            let text = self
                .string_of(message)
                .unwrap_or("(assert message is not a string)")
                .to_string();
            self.report(
                MessageKind::Error,
                &[location],
                format!("assertion failed: {text}"),
            );
            return true;
        }
        false
    }

    pub fn dump_message(&mut self, location: Location, message: Value) {
        match self.string_of(message).map(str::to_string) {
            Some(text) => self.report(MessageKind::Note, &[location], text),
            None => self.report(
                MessageKind::Error,
                &[location],
                "dump value is not of type string",
            ),
        }
    }

    pub fn check_assertions(&mut self, record: RecordIndex) -> Flow<()> {
        let mut resolver = RecordResolver::new(record);
        let mut failed = false;
        let assertions = self.record(record).assertions.clone();
        for assertion in assertions {
            let condition = self.resolve(assertion.condition, &mut resolver)?;
            let message = self.resolve(assertion.message, &mut resolver)?;
            failed |= self.check_assert(assertion.location, condition, message);
        }
        if failed {
            let locations = self.record(record).locations.clone();
            self.report(
                MessageKind::Error,
                &locations,
                "assertion failed in this record",
            );
        }
        Ok(())
    }

    pub fn emit_dumps(&mut self, record: RecordIndex) -> Flow<()> {
        let mut resolver = RecordResolver::new(record);
        let dumps = self.record(record).dumps.clone();
        for dump in dumps {
            let message = self.resolve(dump.message, &mut resolver)?;
            self.dump_message(dump.location, message);
        }
        Ok(())
    }

    fn print_type_of_field(&self, field: &RecordField) -> String {
        if self.is_string_type(field.field_type) {
            if let Node::String { code: true, .. } = self.node(field.value) {
                return "code".to_string();
            }
            return "string".to_string();
        }
        self.type_name(field.field_type)
    }

    pub fn write_field(&self, field: &RecordField, out: &mut String) {
        if field.kind == FieldKind::NonconcreteAllowed {
            out.push_str("field ");
        }
        out.push_str(&self.print_type_of_field(field));
        out.push(' ');
        self.write_unquoted(field.name, out);
        out.push_str(" = ");
        self.write_value(field.value, out);
    }

    pub fn write_record(&self, index: RecordIndex, out: &mut String) {
        let record = self.record(index);
        self.write_unquoted(record.name, out);
        if !record.template_arguments.is_empty() {
            out.push('<');
            for (position, argument) in record.template_arguments.iter().enumerate() {
                if position > 0 {
                    out.push_str(", ");
                }
                if let Some(at) = record.field(*argument) {
                    self.write_field(&record.fields[at], out);
                }
            }
            out.push('>');
        }
        out.push_str(" {");
        let superclasses = self.superclass_list(index);
        if !superclasses.is_empty() {
            out.push_str("\t//");
            for class in superclasses {
                out.push(' ');
                self.write_unquoted(self.record(class).name, out);
            }
        }
        out.push('\n');
        for pass in [true, false] {
            for field in &record.fields {
                let nonconcrete = field.kind == FieldKind::NonconcreteAllowed;
                if nonconcrete == pass && !record.is_template_argument(field.name) {
                    out.push_str("  ");
                    self.write_field(field, out);
                    out.push_str(";\n");
                }
            }
        }
        out.push_str("}\n");
    }

    pub fn record_text(&self, index: RecordIndex) -> String {
        let mut out = String::new();
        self.write_record(index, &mut out);
        out
    }

    pub fn print_records(&self) -> String {
        let mut out = String::new();
        out.push_str("------------- Classes -----------------\n");
        for class in self.classes.values() {
            out.push_str("class ");
            self.write_record(*class, &mut out);
        }
        out.push_str("------------- Defs -----------------\n");
        for definition in self.definitions.values() {
            out.push_str("def ");
            self.write_record(*definition, &mut out);
        }
        out
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LetMode {
    Replace,
    Append,
    Prepend,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NameMode {
    Value,
    Name,
}

#[derive(Clone)]
struct LetRecord {
    name: Value,
    bits: Vec<u32>,
    value: Value,
    location: Location,
    mode: LetMode,
}

#[derive(Clone)]
enum Entry {
    Record(RecordIndex),
    Loop(Box<ForeachLoop>),
    Assertion(Assertion),
    Dump(DumpRequest),
}

#[derive(Clone)]
struct ForeachLoop {
    location: Location,
    iterator: Option<Value>,
    list: Value,
    entries: Vec<Entry>,
}

struct MultiClass {
    record: RecordIndex,
    entries: Vec<Entry>,
}

#[derive(Clone, Copy)]
enum ScopeKind {
    Local,
    Record(RecordIndex),
    Loop(Option<Value>),
    MultiClass(usize),
}

struct Scope {
    kind: ScopeKind,
    variables: BTreeMap<String, Value>,
}

struct DefsetRecord {
    location: Location,
    element: Type,
    elements: Vec<Value>,
}

struct SubclassReference {
    start: Location,
    record: RecordIndex,
    arguments: Vec<Value>,
}

struct SubMultiClassReference {
    start: Location,
    multiclass: usize,
    arguments: Vec<Value>,
}

type Substitutions = Vec<(Value, Value)>;

struct Parser<'a> {
    lexer: Lexer<'a>,
    keeper: RecordKeeper,
    let_stack: Vec<Vec<LetRecord>>,
    multiclass_names: BTreeMap<String, usize>,
    multiclasses: Vec<MultiClass>,
    type_aliases: BTreeMap<String, Type>,
    loops: Vec<ForeachLoop>,
    defsets: Vec<DefsetRecord>,
    current_multiclass: Option<usize>,
    scopes: Vec<Scope>,
    warn_unused_template_arguments: bool,
}

impl<'a> Parser<'a> {
    fn absorb(&mut self) {
        for message in self.lexer.messages.drain(..) {
            if message.kind == MessageKind::Error {
                self.keeper.errors += 1;
            }
            self.keeper.messages.push(message);
        }
    }

    fn lex(&mut self) -> Token {
        let token = self.lexer.lex();
        self.absorb();
        token
    }

    fn token(&self) -> Token {
        self.lexer.token
    }

    fn soft<T>(&mut self, outcome: Flow<T>) -> Flow<Option<T>> {
        match outcome {
            Ok(found) => Ok(Some(found)),
            Err(stop) if self.keeper.stopped => Err(stop),
            Err(_) => Ok(None),
        }
    }

    fn recover(&mut self, outcome: Flow<()>) -> Flow<()> {
        if outcome.is_err() && self.keeper.stopped {
            return Err(Stop);
        }
        Ok(())
    }

    fn location(&self) -> Location {
        self.lexer.location()
    }

    fn consume(&mut self, token: Token) -> bool {
        if self.token() == token {
            self.lex();
            true
        } else {
            false
        }
    }

    fn report_error(&mut self, location: Location, text: impl Into<String>) {
        self.keeper.report(MessageKind::Error, &[location], text);
    }

    fn error<T>(&mut self, location: Location, text: impl Into<String>) -> Flow<T> {
        self.report_error(location, text);
        Err(Stop)
    }

    fn token_error<T>(&mut self, text: impl Into<String>) -> Flow<T> {
        let location = self.location();
        self.error(location, text)
    }

    fn note(&mut self, locations: &[Location], text: impl Into<String>) {
        self.keeper.report(MessageKind::Note, locations, text);
    }

    fn string_concat(&mut self, left: Value, right: Value) -> Value {
        if let (
            Node::String {
                text: first,
                code: first_code,
            },
            Node::String {
                text: second,
                code: second_code,
            },
        ) = (
            self.keeper.node(left).clone(),
            self.keeper.node(right).clone(),
        ) {
            let joined = format!("{}{}", self.keeper.text(first), self.keeper.text(second));
            return self
                .keeper
                .string_with_format(&joined, first_code || second_code);
        }
        let of = self.keeper.string_type();
        self.keeper
            .binary(BinaryOperator::StringConcat, left, right, of)
    }

    fn list_concat(&mut self, left: Value, right: Value) -> Value {
        if let (
            Node::List {
                elements: first,
                element,
            },
            Node::List {
                elements: second, ..
            },
        ) = (
            self.keeper.node(left).clone(),
            self.keeper.node(right).clone(),
        ) {
            let mut joined = first.to_vec();
            joined.extend(second.iter().copied());
            return self.keeper.list(joined, element);
        }
        let of = self.keeper.type_of(left).unwrap_or(Type::NONE);
        self.keeper
            .binary(BinaryOperator::ListConcat, left, right, of)
    }

    fn qualify(&mut self, record: RecordIndex, name: Value) -> Flow<Value> {
        let separator = if self.keeper.record(record).kind == RecordKind::MultiClass {
            "::"
        } else {
            ":"
        };
        let record_name = self.keeper.record(record).name;
        let separator = self.keeper.string(separator);
        let first = self.string_concat(record_name, separator);
        let joined = self.string_concat(first, name);
        if matches!(self.keeper.node(joined), Node::Binary { .. }) {
            return self.keeper.fold(joined, Some(record), false);
        }
        Ok(joined)
    }

    fn implicit_name(&mut self, record: RecordIndex) -> Flow<Value> {
        let name = self.keeper.string("NAME");
        self.qualify(record, name)
    }

    fn find_in_arguments(&mut self, record: RecordIndex, name: Value) -> Flow<Option<Value>> {
        let argument = self.qualify(record, name)?;
        if self.keeper.record(record).is_template_argument(argument) {
            if let Some(at) = self.keeper.record(record).field(argument) {
                self.keeper.record_mut(record).fields[at].used = true;
                let of = self.keeper.record(record).fields[at].field_type;
                return Ok(Some(self.keeper.variable(argument, of)));
            }
        }
        if self.keeper.string_of(name) == Some("NAME") {
            let of = self.keeper.string_type();
            return Ok(Some(self.keeper.variable(argument, of)));
        }
        Ok(None)
    }

    fn find_variable(&mut self, name: Value) -> Flow<Option<Value>> {
        let text = self.keeper.string_of(name).unwrap_or("").to_string();
        for depth in (0..self.scopes.len()).rev() {
            if let Some(found) = self.scopes[depth].variables.get(&text) {
                return Ok(Some(*found));
            }
            match self.scopes[depth].kind {
                ScopeKind::Local => {}
                ScopeKind::Record(record) => {
                    if let Some(at) = self.keeper.record(record).field(name) {
                        let of = self.keeper.record(record).fields[at].field_type;
                        return Ok(Some(self.keeper.variable(name, of)));
                    }
                    if self.keeper.record(record).kind == RecordKind::Class {
                        if let Some(found) = self.find_in_arguments(record, name)? {
                            return Ok(Some(found));
                        }
                    }
                }
                ScopeKind::Loop(iterator) => {
                    if let Some(iterator) = iterator {
                        if let Node::Variable {
                            name: iterator_name,
                            ..
                        } = self.keeper.node(iterator)
                        {
                            if *iterator_name == name {
                                return Ok(Some(iterator));
                            }
                        }
                    }
                }
                ScopeKind::MultiClass(index) => {
                    let record = self.multiclasses[index].record;
                    if let Some(found) = self.find_in_arguments(record, name)? {
                        return Ok(Some(found));
                    }
                }
            }
        }
        Ok(None)
    }

    fn push_scope(&mut self, kind: ScopeKind) {
        self.scopes.push(Scope {
            kind,
            variables: BTreeMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn target_record(&self, record: Option<RecordIndex>) -> RecordIndex {
        record.unwrap_or_else(|| self.multiclasses[self.current_multiclass.unwrap_or(0)].record)
    }

    fn add_value(
        &mut self,
        record: Option<RecordIndex>,
        location: Location,
        field: RecordField,
    ) -> Flow<()> {
        let record = self.target_record(record);
        if let Some(at) = self.keeper.record(record).field(field.name) {
            if self.keeper.set_field_value(record, at, field.value)? {
                let existing = self.keeper.record(record).fields[at].field_type;
                let text = format!(
                    "New definition of '{}' of type '{}' is incompatible with previous definition of type '{}'",
                    self.keeper.unquoted(field.name),
                    self.keeper.type_name(field.field_type),
                    self.keeper.type_name(existing)
                );
                return self.error(location, text);
            }
        } else {
            self.keeper.record_mut(record).fields.push(field);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn set_value(
        &mut self,
        record: Option<RecordIndex>,
        location: Location,
        name: Value,
        bits: &[u32],
        value: Value,
        allow_self_assignment: bool,
        override_location: bool,
        mode: LetMode,
    ) -> Flow<()> {
        let record = self.target_record(record);
        let Some(at) = self.keeper.record(record).field(name) else {
            let text = format!("Value '{}' unknown!", self.keeper.unquoted(name));
            return self.error(location, text);
        };
        let mut value = value;
        if mode != LetMode::Replace {
            if !bits.is_empty() {
                return self.error(location, "Cannot use append/prepend with bit range");
            }
            let current = self.keeper.record(record).fields[at].value;
            let field_type = self.keeper.record(record).fields[at].field_type;
            if !self.keeper.is_unset(current) {
                let appending = mode == LetMode::Append;
                let (left, right) = if appending {
                    (current, value)
                } else {
                    (value, current)
                };
                let operator = if self.keeper.list_element(field_type).is_some() {
                    BinaryOperator::ListConcat
                } else if self.keeper.is_string_type(field_type) {
                    BinaryOperator::StringConcat
                } else if self.keeper.is_dag_type(field_type) {
                    BinaryOperator::Concat
                } else {
                    let text = format!(
                        "Cannot {} field '{}' of type '{}' (expected list, string, code, or dag)",
                        if appending { "append to" } else { "prepend to" },
                        self.keeper.unquoted(name),
                        self.keeper.type_name(field_type)
                    );
                    return self.error(location, text);
                };
                let made = self.keeper.binary(operator, left, right, field_type);
                value = self.keeper.fold(made, Some(record), false)?;
            }
        }
        if bits.is_empty() {
            if let Node::Variable { name: variable, .. } = self.keeper.node(value) {
                if *variable == name && !allow_self_assignment {
                    return self.error(location, "Recursion / self-assignment forbidden");
                }
            }
        }
        if !bits.is_empty() {
            let current = self.keeper.record(record).fields[at].value;
            let Node::Bits(current_bits) = self.keeper.node(current).clone() else {
                let text = format!("Value '{}' is not a bits type", self.keeper.unquoted(name));
                return self.error(location, text);
            };
            let width = self.keeper.bits_type(bits.len() as u32);
            let Some(converted) = self.keeper.cast_to(value, width)? else {
                return self.error(location, "Initializer is not compatible with bit range");
            };
            let mut made: Vec<Option<Value>> = vec![None; current_bits.len()];
            for (index, bit) in bits.iter().enumerate() {
                let bit = *bit as usize;
                if bit >= made.len() || made[bit].is_some() {
                    let text = format!(
                        "Cannot set bit #{bit} of value '{}' more than once",
                        self.keeper.unquoted(name)
                    );
                    return self.error(location, text);
                }
                made[bit] = Some(self.keeper.get_bit(converted, index as u32));
            }
            let filled = made
                .iter()
                .enumerate()
                .map(|(index, bit)| bit.unwrap_or(current_bits[index]))
                .collect();
            value = self.keeper.bits(filled);
        }
        if override_location {
            self.keeper.record_mut(record).fields[at].location = location;
        }
        if self.keeper.set_field_value(record, at, value)? {
            let field_type = self.keeper.record(record).fields[at].field_type;
            let described = match self.keeper.node(value) {
                Node::Bits(held) => format!("' of type bit initializer with length {}", held.len()),
                _ => match self.keeper.type_of(value) {
                    Some(of) => format!("' of type '{}'", self.keeper.type_name(of)),
                    None => String::new(),
                },
            };
            let text = format!(
                "Field '{}' of type '{}' is incompatible with value '{}{described}",
                self.keeper.unquoted(name),
                self.keeper.type_name(field_type),
                self.keeper.text_of(value)
            );
            return self.error(location, text);
        }
        Ok(())
    }

    fn resolve_arguments(
        &mut self,
        record: RecordIndex,
        arguments: &[Value],
        location: Location,
    ) -> Flow<Substitutions> {
        let names = self.keeper.record(record).template_arguments.clone();
        let mut unsolved = names.clone();
        let mut pairs = Vec::new();
        for argument in arguments {
            let (value, name, position) = self.keeper.argument_parts(*argument);
            let name = name.unwrap_or_else(|| names[position as usize]);
            if !unsolved.contains(&name) {
                let text = format!(
                    "We can only specify the template argument '{}' once",
                    self.keeper.unquoted(name)
                );
                return self.error(location, text);
            }
            pairs.push((name, value));
            unsolved.retain(|held| *held != name);
        }
        for name in unsolved {
            let held = self.keeper.record(record);
            let at = held.field(name);
            let default = at
                .map(|at| held.fields[at].value)
                .unwrap_or(self.keeper.unset());
            if !self.keeper.is_complete(default) {
                let text = self.keeper.unquoted(name);
                self.report_error(
                    location,
                    format!("value not specified for template argument '{text}'"),
                );
                let field_location = at
                    .map(|at| self.keeper.record(record).fields[at].location)
                    .unwrap_or_default();
                let record_name = self.keeper.record_name_text(record);
                self.note(&[field_location], format!("declared in '{record_name}'"));
                return Err(Stop);
            }
            pairs.push((name, default));
        }
        Ok(pairs)
    }

    fn add_subclass(&mut self, record: RecordIndex, reference: &SubclassReference) -> Flow<()> {
        let class = reference.record;
        let mut resolver = MapResolver::new(Some(record));
        let fields = self.keeper.record(class).fields.clone();
        for field in fields {
            if field.kind != FieldKind::TemplateArgument {
                self.add_value(Some(record), reference.start, field)?;
            }
        }
        for (name, value) in self.resolve_arguments(class, &reference.arguments, reference.start)? {
            resolver.set(name, value);
        }
        let assertions = self.keeper.record(class).assertions.clone();
        let dumps = self.keeper.record(class).dumps.clone();
        self.keeper.record_mut(record).assertions.extend(assertions);
        self.keeper.record_mut(record).dumps.extend(dumps);
        let name = if self.keeper.record(record).kind == RecordKind::Class {
            let implicit = self.implicit_name(record)?;
            let of = self.keeper.string_type();
            self.keeper.variable(implicit, of)
        } else {
            self.keeper.record(record).name
        };
        let class_implicit = self.implicit_name(class)?;
        resolver.set(class_implicit, name);
        self.keeper.resolve_record(record, &mut resolver, None)?;
        if self.keeper.is_subclass_of(record, class) {
            let text = format!(
                "Already subclass of '{}'!\n",
                self.keeper.record_name_text(class)
            );
            return self.error(reference.start, text);
        }
        self.keeper
            .record_mut(record)
            .superclasses
            .push((class, reference.start));
        Ok(())
    }

    fn add_subclass_to_entry(&mut self, entry: &Entry, reference: &SubclassReference) -> Flow<()> {
        match entry {
            Entry::Record(record) => self.add_subclass(*record, reference),
            Entry::Loop(inner) => {
                for nested in &inner.entries {
                    self.add_subclass_to_entry(nested, reference)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn resolve_arguments_of_multiclass(
        &mut self,
        multiclass: usize,
        arguments: &[Value],
        defm_name: Value,
        location: Location,
    ) -> Flow<Substitutions> {
        let record = self.multiclasses[multiclass].record;
        let implicit = self.implicit_name(record)?;
        let mut substitutions = vec![(implicit, defm_name)];
        substitutions.extend(self.resolve_arguments(record, arguments, location)?);
        Ok(substitutions)
    }

    fn add_submulticlass(
        &mut self,
        current: usize,
        reference: &SubMultiClassReference,
    ) -> Flow<()> {
        let current_record = self.multiclasses[current].record;
        let implicit = self.implicit_name(current_record)?;
        let of = self.keeper.string_type();
        let name = self.keeper.variable(implicit, of);
        let mut substitutions = self.resolve_arguments_of_multiclass(
            reference.multiclass,
            &reference.arguments,
            name,
            reference.start,
        )?;
        let source = self.multiclasses[reference.multiclass].entries.clone();
        let mut made = Vec::new();
        self.resolve_entries(&source, &mut substitutions, false, Some(&mut made), None)?;
        self.multiclasses[current].entries.extend(made);
        Ok(())
    }

    fn add_entry(&mut self, entry: Entry) -> Flow<()> {
        if let Some(innermost) = self.loops.last_mut() {
            innermost.entries.push(entry);
            return Ok(());
        }
        if let Entry::Loop(each) = &entry {
            let mut substitutions = Vec::new();
            if let Some(multiclass) = self.current_multiclass {
                let mut made = Vec::new();
                self.resolve_loop(each, &mut substitutions, false, Some(&mut made), None)?;
                self.multiclasses[multiclass].entries.extend(made);
                return Ok(());
            }
            return self.resolve_loop(each, &mut substitutions, true, None, None);
        }
        if let Some(multiclass) = self.current_multiclass {
            self.multiclasses[multiclass].entries.push(entry);
            return Ok(());
        }
        match entry {
            Entry::Assertion(assertion) => {
                self.keeper.check_assert(
                    assertion.location,
                    assertion.condition,
                    assertion.message,
                );
                Ok(())
            }
            Entry::Dump(dump) => {
                self.keeper.dump_message(dump.location, dump.message);
                Ok(())
            }
            Entry::Record(record) => self.add_definition_once(record),
            Entry::Loop(_) => Ok(()),
        }
    }

    fn resolve_loop(
        &mut self,
        each: &ForeachLoop,
        substitutions: &mut Substitutions,
        finished: bool,
        destination: Option<&mut Vec<Entry>>,
        location: Option<Location>,
    ) -> Flow<()> {
        let mut resolver = MapResolver::new(None);
        for (key, value) in substitutions.iter() {
            resolver.set(*key, *value);
        }
        let mut list = self.keeper.resolve(each.list, &mut resolver)?;
        if finished {
            if let Node::Ternary {
                operator: TernaryOperator::If,
                left,
                middle,
                right,
                result,
            } = self.keeper.node(list).clone()
            {
                resolver.set_final(true);
                let resolved = self.keeper.resolve(left, &mut resolver)?;
                if resolved == left {
                    let text = format!(
                        "unable to resolve if condition '{}' at end of containing scope",
                        self.keeper.text_of(resolved)
                    );
                    return self.error(each.location, text);
                }
                let made =
                    self.keeper
                        .ternary(TernaryOperator::If, resolved, middle, right, result);
                list = self.keeper.fold(made, None, false)?;
            }
        }
        let Node::List { elements, .. } = self.keeper.node(list).clone() else {
            if !finished {
                let mut nested = ForeachLoop {
                    location: each.location,
                    iterator: each.iterator,
                    list,
                    entries: Vec::new(),
                };
                self.resolve_entries(
                    &each.entries,
                    substitutions,
                    finished,
                    Some(&mut nested.entries),
                    location,
                )?;
                if let Some(destination) = destination {
                    destination.push(Entry::Loop(Box::new(nested)));
                }
                return Ok(());
            }
            let text = format!(
                "attempting to loop over '{}', expected a list",
                self.keeper.text_of(list)
            );
            return self.error(each.location, text);
        };
        let mut destination = destination;
        for element in elements.iter() {
            if let Some(iterator) = each.iterator {
                if let Node::Variable { name, .. } = self.keeper.node(iterator) {
                    substitutions.push((*name, *element));
                }
            }
            let outcome = self.resolve_entries(
                &each.entries,
                substitutions,
                finished,
                destination.as_deref_mut(),
                None,
            );
            if each.iterator.is_some() {
                substitutions.pop();
            }
            outcome?;
        }
        Ok(())
    }

    fn resolve_entries(
        &mut self,
        source: &[Entry],
        substitutions: &mut Substitutions,
        finished: bool,
        mut destination: Option<&mut Vec<Entry>>,
        location: Option<Location>,
    ) -> Flow<()> {
        for entry in source {
            match entry {
                Entry::Loop(each) => {
                    self.resolve_loop(
                        each,
                        substitutions,
                        finished,
                        destination.as_deref_mut(),
                        None,
                    )?;
                }
                Entry::Assertion(assertion) => {
                    let mut resolver = MapResolver::new(None);
                    for (key, value) in substitutions.iter() {
                        resolver.set(*key, *value);
                    }
                    let condition = self.keeper.resolve(assertion.condition, &mut resolver)?;
                    let message = self.keeper.resolve(assertion.message, &mut resolver)?;
                    match destination.as_deref_mut() {
                        Some(destination) => destination.push(Entry::Assertion(Assertion {
                            location: assertion.location,
                            condition,
                            message,
                        })),
                        None => {
                            self.keeper
                                .check_assert(assertion.location, condition, message);
                        }
                    }
                }
                Entry::Dump(dump) => {
                    let mut resolver = MapResolver::new(None);
                    for (key, value) in substitutions.iter() {
                        resolver.set(*key, *value);
                    }
                    let message = self.keeper.resolve(dump.message, &mut resolver)?;
                    match destination.as_deref_mut() {
                        Some(destination) => destination.push(Entry::Dump(DumpRequest {
                            location: dump.location,
                            message,
                        })),
                        None => self.keeper.dump_message(dump.location, message),
                    }
                }
                Entry::Record(record) => {
                    let copy = self.keeper.copy_record(*record);
                    if let Some(location) = location {
                        self.keeper.record_mut(copy).locations.push(location);
                    }
                    let mut resolver = MapResolver::new(Some(copy));
                    for (key, value) in substitutions.iter() {
                        resolver.set(*key, *value);
                    }
                    self.keeper.resolve_record(copy, &mut resolver, None)?;
                    match destination.as_deref_mut() {
                        Some(destination) => destination.push(Entry::Record(copy)),
                        None => self.add_definition_once(copy)?,
                    }
                }
            }
        }
        Ok(())
    }

    fn bits_concrete(&self, record: RecordIndex, value: Value) -> bool {
        let Node::Bits(bits) = self.keeper.node(value) else {
            return true;
        };
        for bit in bits.iter() {
            let mut reference = false;
            match self.keeper.node(*bit) {
                Node::VariableBit { variable, .. } => {
                    if let Node::Variable { name, .. } = self.keeper.node(*variable) {
                        let wanted = self.keeper.string_of(*name);
                        let held = self.keeper.record(record);
                        if held
                            .fields
                            .iter()
                            .any(|field| self.keeper.string_of(field.name) == wanted)
                        {
                            reference = true;
                        }
                    }
                }
                Node::Variable { .. } => reference = true,
                _ => {}
            }
            if !(reference || self.keeper.is_concrete(*bit)) {
                return false;
            }
        }
        true
    }

    fn check_concrete(&mut self, record: RecordIndex) {
        let fields = self.keeper.record(record).fields.clone();
        for field in fields {
            if field.kind == FieldKind::NonconcreteAllowed {
                continue;
            }
            let fine = if matches!(self.keeper.node(field.value), Node::Bits(_)) {
                self.bits_concrete(record, field.value)
            } else {
                self.keeper.is_concrete(field.value)
            };
            if !fine {
                let locations = self.keeper.record(record).locations.clone();
                let text = format!(
                    "Initializer of '{}' in '{}' could not be fully resolved: {}",
                    self.keeper.unquoted(field.name),
                    self.keeper.record_name_text(record),
                    self.keeper.text_of(field.value)
                );
                self.keeper.report(MessageKind::Error, &locations, text);
            }
        }
    }

    fn add_definition_once(&mut self, record: RecordIndex) -> Flow<()> {
        let mut new_name = None;
        let name = self.keeper.record_name_text(record);
        if let Some(previous) = self.keeper.find_definition(&name) {
            if self.keeper.record(record).kind != RecordKind::AnonymousDefinition {
                let locations = self.keeper.record(record).locations.clone();
                self.keeper.report(
                    MessageKind::Error,
                    &locations,
                    format!("def already exists: {name}"),
                );
                let previous_locations = self.keeper.record(previous).locations.clone();
                self.note(&previous_locations, "location of previous definition");
                return Err(Stop);
            }
            new_name = Some(self.keeper.new_anonymous_name());
        }
        self.keeper.resolve_record_final(record, new_name)?;
        self.check_concrete(record);
        let record_name = self.keeper.record(record).name;
        if self.keeper.string_of(record_name).is_none() {
            let locations = self.keeper.record(record).locations.clone();
            let text = format!(
                "record name '{}' could not be fully resolved",
                self.keeper.text_of(record_name)
            );
            self.keeper.report(MessageKind::Error, &locations, text);
            return Err(Stop);
        }
        self.keeper.check_assertions(record)?;
        self.keeper.emit_dumps(record)?;
        for index in 0..self.defsets.len() {
            let definition = self.keeper.definition_of(record);
            let of = self.keeper.type_of(definition).unwrap_or(Type::NONE);
            let element = self.defsets[index].element;
            if !self.keeper.type_is_a(of, element) {
                let locations = self.keeper.record(record).locations.clone();
                let text = format!(
                    "adding record of incompatible type '{}' to defset",
                    self.keeper.type_name(of)
                );
                self.keeper.report(MessageKind::Error, &locations, text);
                let defset_location = self.defsets[index].location;
                self.note(&[defset_location], "location of defset declaration");
                return Err(Stop);
            }
            self.defsets[index].elements.push(definition);
        }
        self.keeper.add_definition(record);
        Ok(())
    }

    fn check_unused_template_arguments(&mut self, record: RecordIndex) {
        if !self.warn_unused_template_arguments {
            return;
        }
        let held = self.keeper.record(record).clone();
        for argument in &held.template_arguments {
            if let Some(at) = held.field(*argument) {
                let field = &held.fields[at];
                if !field.used {
                    let text = format!(
                        "unused template argument: {}",
                        self.keeper.unquoted(field.name)
                    );
                    self.keeper
                        .report(MessageKind::Warning, &[field.location], text);
                }
            }
        }
    }
}

impl<'a> Parser<'a> {
    fn parse_class_identifier(&mut self) -> Flow<RecordIndex> {
        if self.token() != Token::Identifier {
            return self.token_error("expected name for ClassID");
        }
        let name = self.lexer.text.clone();
        let found = self.keeper.find_class(&name);
        if found.is_none() {
            let text = format!("Couldn't find class '{name}'");
            if self.multiclass_names.contains_key(&name) {
                let location = self.location();
                self.report_error(
                    location,
                    format!("{text}. Use 'defm' if you meant to use multiclass '{name}'"),
                );
            } else {
                let location = self.location();
                self.report_error(location, text);
            }
        }
        self.lex();
        found.ok_or(Stop)
    }

    fn parse_multiclass_identifier(&mut self) -> Flow<usize> {
        if self.token() != Token::Identifier {
            return self.token_error("expected name for MultiClassID");
        }
        let name = self.lexer.text.clone();
        let found = self.multiclass_names.get(&name).copied();
        if found.is_none() {
            let location = self.location();
            self.report_error(location, format!("Couldn't find multiclass '{name}'"));
        }
        self.lex();
        found.ok_or(Stop)
    }

    fn parse_subclass_reference(
        &mut self,
        current: Option<RecordIndex>,
        defm: bool,
    ) -> Flow<SubclassReference> {
        let start = self.location();
        let record = if defm {
            let multiclass = self.parse_multiclass_identifier()?;
            self.multiclasses[multiclass].record
        } else {
            self.parse_class_identifier()?
        };
        if !self.consume(Token::Less) {
            return Ok(SubclassReference {
                start,
                record,
                arguments: Vec::new(),
            });
        }
        let (mut arguments, locations) = self.parse_template_argument_values(current, record)?;
        self.check_template_argument_values(&mut arguments, &locations, record)?;
        Ok(SubclassReference {
            start,
            record,
            arguments,
        })
    }

    fn parse_submulticlass_reference(&mut self, current: usize) -> Flow<SubMultiClassReference> {
        let start = self.location();
        let multiclass = self.parse_multiclass_identifier()?;
        if !self.consume(Token::Less) {
            return Ok(SubMultiClassReference {
                start,
                multiclass,
                arguments: Vec::new(),
            });
        }
        let current_record = self.multiclasses[current].record;
        let target = self.multiclasses[multiclass].record;
        let (mut arguments, locations) =
            self.parse_template_argument_values(Some(current_record), target)?;
        self.check_template_argument_values(&mut arguments, &locations, target)?;
        Ok(SubMultiClassReference {
            start,
            multiclass,
            arguments,
        })
    }

    fn type_or_none(&self, value: Value) -> Type {
        self.keeper.type_of(value).unwrap_or(Type::NONE)
    }

    fn parse_slice_element(&mut self, current: Option<RecordIndex>) -> Flow<Value> {
        let left_location = self.location();
        let left = self.parse_value(current, None, NameMode::Value)?;
        let right;
        match self.token() {
            Token::Ellipsis | Token::Minus => {
                self.lex();
                let right_location = self.location();
                right = self.parse_value(current, None, NameMode::Value)?;
                let of = self.type_or_none(right);
                if !self.keeper.is_integer(of) {
                    let text = format!("expected int...int, got {}", self.keeper.type_name(of));
                    return self.error(right_location, text);
                }
            }
            Token::Integer => {
                let negated = self.lexer.integer.wrapping_neg();
                if negated < 0 {
                    return self.token_error("invalid range, cannot be negative");
                }
                right = self.keeper.integer(negated);
                self.lex();
            }
            _ => return Ok(left),
        }
        let of = self.type_or_none(left);
        if !self.keeper.is_integer(of) {
            let text = format!("expected int...int, got {}", self.keeper.type_name(of));
            return self.error(left_location, text);
        }
        let integer = self.keeper.integer_type();
        let list = self.keeper.list_type(integer);
        let made = self
            .keeper
            .binary(BinaryOperator::RangeClosed, left, right, list);
        self.keeper.fold(made, current, false)
    }

    fn parse_slice_elements(
        &mut self,
        current: Option<RecordIndex>,
        mut single: bool,
    ) -> Flow<Value> {
        let mut elements: Vec<Value> = Vec::new();
        let mut slices: Vec<Value> = Vec::new();
        let mut pending;
        loop {
            let left_location = self.location();
            pending = Some(self.parse_slice_element(current)?);
            let value = pending.unwrap_or(self.keeper.unset());
            let of = self.type_or_none(value);
            if let Some(element) = self.keeper.list_element(of) {
                if !self.keeper.is_integer(element) {
                    let text = format!("expected list<int>, got {}", self.keeper.type_name(of));
                    return self.error(left_location, text);
                }
                if !elements.is_empty() {
                    let integer = self.keeper.integer_type();
                    let made = self.keeper.list(std::mem::take(&mut elements), integer);
                    slices.push(made);
                }
                slices.push(value);
                single = false;
                pending = None;
            } else if !self.keeper.is_integer(of) {
                let text = format!("unhandled type {} in range", self.keeper.type_name(of));
                return self.error(left_location, text);
            }
            if self.token() != Token::Comma {
                break;
            }
            self.lex();
            single = false;
            if let Some(value) = pending.take() {
                elements.push(value);
            }
            if self.token() == Token::RightSquare {
                break;
            }
        }
        if let Some(value) = pending {
            if single {
                return Ok(value);
            }
            elements.push(value);
        }
        if !elements.is_empty() {
            let integer = self.keeper.integer_type();
            let made = self.keeper.list(elements, integer);
            slices.push(made);
        }
        let mut result: Option<Value> = None;
        for slice in slices {
            result = Some(match result {
                Some(earlier) => self.list_concat(earlier, slice),
                None => slice,
            });
        }
        result.ok_or(Stop)
    }

    fn parse_range_piece(&mut self, ranges: &mut Vec<u32>, first: Option<Value>) -> Flow<()> {
        let value = match first {
            Some(value) => Some(value),
            None => {
                let parsed = self.parse_value(None, None, NameMode::Value);
                self.soft(parsed)?
            }
        };
        let Some(start) = value.and_then(|value| self.keeper.integer_of(value)) else {
            return self.token_error("expected integer or bitrange");
        };
        if start < 0 {
            return self.token_error("invalid range, cannot be negative");
        }
        let end;
        match self.token() {
            Token::Ellipsis | Token::Minus => {
                self.lex();
                let parsed = self.parse_value(None, None, NameMode::Value);
                let last = self.soft(parsed)?;
                let Some(last) = last.and_then(|last| self.keeper.integer_of(last)) else {
                    return self.token_error("expected integer value as end of range");
                };
                end = last;
            }
            Token::Integer => {
                end = self.lexer.integer.wrapping_neg();
                self.lex();
            }
            _ => {
                ranges.push(start as u32);
                return Ok(());
            }
        }
        if end < 0 {
            return self.token_error("invalid range, cannot be negative");
        }
        if start < end {
            for bit in start..=end {
                ranges.push(bit as u32);
            }
        } else {
            let mut bit = start;
            while bit >= end {
                ranges.push(bit as u32);
                bit -= 1;
            }
        }
        Ok(())
    }

    fn parse_range_list(&mut self) -> Flow<Vec<u32>> {
        let mut ranges = Vec::new();
        self.parse_range_piece(&mut ranges, None)?;
        while self.consume(Token::Comma) {
            self.parse_range_piece(&mut ranges, None)?;
        }
        Ok(ranges)
    }

    fn parse_optional_range_list(&mut self) -> Flow<Vec<u32>> {
        let start = self.location();
        if !self.consume(Token::LeftBrace) {
            return Ok(Vec::new());
        }
        let ranges = self.parse_range_list()?;
        if !self.consume(Token::RightBrace) {
            let location = self.location();
            self.report_error(location, "expected '}' at end of bit list");
            return self.error(start, "to match this '{'");
        }
        Ok(ranges)
    }

    fn parse_type(&mut self) -> Flow<Type> {
        match self.token() {
            Token::String | Token::Code => {
                self.lex();
                Ok(self.keeper.string_type())
            }
            Token::Bit => {
                self.lex();
                Ok(self.keeper.bit_type())
            }
            Token::Int => {
                self.lex();
                Ok(self.keeper.integer_type())
            }
            Token::Dag => {
                self.lex();
                Ok(self.keeper.dag_type())
            }
            Token::Identifier => {
                if let Some(alias) = self.type_aliases.get(&self.lexer.text).copied() {
                    self.lex();
                    return Ok(alias);
                }
                match self.parse_class_identifier() {
                    Ok(class) => Ok(self.keeper.record_type(&[class])),
                    Err(_) => self.token_error("unknown class name"),
                }
            }
            Token::Bits => {
                if self.lex() != Token::Less {
                    return self.token_error("expected '<' after bits type");
                }
                if self.lex() != Token::Integer {
                    return self.token_error("expected integer in bits<n> type");
                }
                let width = self.lexer.integer as u64;
                if self.lex() != Token::Greater {
                    return self.token_error("expected '>' at end of bits<n> type");
                }
                self.lex();
                Ok(self.keeper.bits_type(width as u32))
            }
            Token::List => {
                if self.lex() != Token::Less {
                    return self.token_error("expected '<' after list type");
                }
                self.lex();
                let element = self.parse_type()?;
                if !self.consume(Token::Greater) {
                    return self.token_error("expected '>' at end of list<ty> type");
                }
                Ok(self.keeper.list_type(element))
            }
            _ => self.token_error("Unknown token when expecting a type"),
        }
    }

    fn parse_identifier_value(
        &mut self,
        current: Option<RecordIndex>,
        name: Value,
        location: Location,
        mode: NameMode,
    ) -> Flow<Value> {
        if let Some(found) = self.find_variable(name)? {
            return Ok(found);
        }
        if mode == NameMode::Name {
            return Ok(name);
        }
        let text = self.keeper.string_of(name).unwrap_or("").to_string();
        if let Some(found) = self.keeper.global(&text) {
            return Ok(found);
        }
        if let Some(record) = current {
            if self.keeper.record(record).kind != RecordKind::Class
                && self.current_multiclass.is_none()
                && self.keeper.record(record).name == name
            {
                let of = self.keeper.record_type_of(record);
                return Ok(self.keeper.unary(UnaryOperator::Cast, name, of));
            }
        }
        self.error(location, format!("Variable not defined: '{text}'"))
    }

    fn parse_operator_type(&mut self) -> Flow<Type> {
        if !self.consume(Token::Less) {
            return self.token_error("expected type name for operator");
        }
        if self.token() == Token::Code {
            let location = self.location();
            self.report_error(
                location,
                "the 'code' type is not allowed in bang operators; use 'string'",
            );
        }
        let Ok(found) = self.parse_type() else {
            return self.token_error("expected type name for operator");
        };
        if !self.consume(Token::Greater) {
            return self.token_error("expected type name for operator");
        }
        Ok(found)
    }

    fn expect(&mut self, token: Token, text: &str) -> Flow<()> {
        if !self.consume(token) {
            return self.token_error(text);
        }
        Ok(())
    }

    fn parse_operation(
        &mut self,
        current: Option<RecordIndex>,
        item_type: Option<Type>,
    ) -> Flow<Value> {
        let Token::Bang(operator) = self.token() else {
            return self.token_error("unknown bang operator");
        };
        match operator {
            BangOperator::Not
            | BangOperator::ToLower
            | BangOperator::ToUpper
            | BangOperator::ListFlatten
            | BangOperator::LogTwo
            | BangOperator::Head
            | BangOperator::Tail
            | BangOperator::Size
            | BangOperator::Empty
            | BangOperator::Cast
            | BangOperator::Represent
            | BangOperator::GetDagOperator
            | BangOperator::GetDagOperatorName
            | BangOperator::Initialized => self.parse_unary(current, operator),
            BangOperator::IsA => {
                self.lex();
                let checked = self.parse_operator_type()?;
                self.expect(Token::LeftParenthesis, "expected '(' after type of !isa")?;
                let operand = self.parse_value(current, None, NameMode::Value)?;
                self.expect(Token::RightParenthesis, "expected ')' in !isa")?;
                let made = self.keeper.is_a(checked, operand);
                self.keeper.fold(made, None, false)
            }
            BangOperator::Exists => {
                self.lex();
                let checked = self.parse_operator_type()?;
                self.expect(Token::LeftParenthesis, "expected '(' after type of !exists")?;
                let location = self.location();
                let expression = self.parse_value(current, None, NameMode::Value)?;
                let Some(of) = self.keeper.type_of(expression) else {
                    return self.error(
                        location,
                        "expected string type argument in !exists operator",
                    );
                };
                if self.keeper.record_classes(of).is_some() {
                    return self.error(
                        location,
                        "expected string type argument in !exists operator, please use !isa instead",
                    );
                }
                if !self.keeper.is_string_type(of) {
                    return self.error(
                        location,
                        "expected string type argument in !exists operator",
                    );
                }
                self.expect(Token::RightParenthesis, "expected ')' in !exists")?;
                let made = self.keeper.exists(checked, expression);
                self.keeper.fold(made, current, false)
            }
            BangOperator::Instances => {
                self.lex();
                let checked = self.parse_operator_type()?;
                self.expect(
                    Token::LeftParenthesis,
                    "expected '(' after type of !instances",
                )?;
                let pattern = if self.token() != Token::RightParenthesis {
                    let location = self.location();
                    let pattern = self.parse_value(current, None, NameMode::Value)?;
                    let of = self.keeper.type_of(pattern);
                    if !of.is_some_and(|of| self.keeper.is_string_type(of)) {
                        return self.error(
                            location,
                            "expected string type argument in !instances operator",
                        );
                    }
                    pattern
                } else {
                    self.keeper.string(".*")
                };
                self.expect(Token::RightParenthesis, "expected ')' in !instances")?;
                let made = self.keeper.instances(checked, pattern);
                self.keeper.fold(made, current, false)
            }
            BangOperator::Concat
            | BangOperator::Match
            | BangOperator::Add
            | BangOperator::Subtract
            | BangOperator::Multiply
            | BangOperator::Divide
            | BangOperator::And
            | BangOperator::Or
            | BangOperator::ExclusiveOr
            | BangOperator::ShiftRightArithmetic
            | BangOperator::ShiftRightLogical
            | BangOperator::ShiftLeft
            | BangOperator::Equal
            | BangOperator::NotEqual
            | BangOperator::LessOrEqual
            | BangOperator::Less
            | BangOperator::GreaterOrEqual
            | BangOperator::Greater
            | BangOperator::ListConcat
            | BangOperator::ListSplat
            | BangOperator::ListRemove
            | BangOperator::StringConcat
            | BangOperator::Interleave
            | BangOperator::GetDagArgument
            | BangOperator::GetDagName
            | BangOperator::SetDagOperator
            | BangOperator::SetDagOperatorName => self.parse_binary(current, operator, item_type),
            BangOperator::ForEach | BangOperator::Filter | BangOperator::Sort => {
                self.parse_list_comprehension(current, item_type)
            }
            BangOperator::Range => self.parse_range(current),
            BangOperator::SetDagArgument
            | BangOperator::SetDagName
            | BangOperator::Dag
            | BangOperator::If
            | BangOperator::Substitute => self.parse_ternary(current, operator, item_type),
            BangOperator::Substring => self.parse_substring(current, item_type),
            BangOperator::Find => self.parse_find(current, item_type),
            BangOperator::Condition => self.parse_condition(current, item_type),
            BangOperator::Switch => self.parse_switch(current, item_type),
            BangOperator::FoldLeft => self.parse_fold(current),
        }
    }

    fn parse_unary(&mut self, current: Option<RecordIndex>, operator: BangOperator) -> Flow<Value> {
        let integer = self.keeper.integer_type();
        let string = self.keeper.string_type();
        let (code, mut result) = match operator {
            BangOperator::Cast => {
                self.lex();
                let Ok(found) = self.parse_operator_type() else {
                    return self.token_error("did not get type for unary operator");
                };
                (UnaryOperator::Cast, Some(found))
            }
            BangOperator::Represent => {
                self.lex();
                (UnaryOperator::Represent, Some(string))
            }
            BangOperator::ToLower => {
                self.lex();
                (UnaryOperator::ToLower, Some(string))
            }
            BangOperator::ToUpper => {
                self.lex();
                (UnaryOperator::ToUpper, Some(string))
            }
            BangOperator::Not => {
                self.lex();
                (UnaryOperator::Not, Some(integer))
            }
            BangOperator::ListFlatten => {
                self.lex();
                (UnaryOperator::ListFlatten, Some(integer))
            }
            BangOperator::LogTwo => {
                self.lex();
                (UnaryOperator::LogTwo, Some(integer))
            }
            BangOperator::Head => {
                self.lex();
                (UnaryOperator::Head, None)
            }
            BangOperator::Tail => {
                self.lex();
                (UnaryOperator::Tail, None)
            }
            BangOperator::Size => {
                self.lex();
                (UnaryOperator::Size, Some(integer))
            }
            BangOperator::Empty => {
                self.lex();
                (UnaryOperator::Empty, Some(integer))
            }
            BangOperator::GetDagOperator => {
                self.lex();
                let found = if self.token() == Token::Less {
                    let Ok(found) = self.parse_operator_type() else {
                        return self.token_error("did not get type for unary operator");
                    };
                    if self.keeper.record_classes(found).is_none() {
                        let location = self.location();
                        self.report_error(location, "type for !getdagop must be a record type");
                    }
                    found
                } else {
                    self.keeper.record_type(&[])
                };
                (UnaryOperator::GetDagOperator, Some(found))
            }
            BangOperator::GetDagOperatorName => {
                self.lex();
                (UnaryOperator::GetDagOperatorName, Some(string))
            }
            _ => {
                self.lex();
                (UnaryOperator::Initialized, Some(integer))
            }
        };
        self.expect(Token::LeftParenthesis, "expected '(' after unary operator")?;
        let operand = self.parse_value(current, None, NameMode::Value)?;
        let operand_type = self.keeper.type_of(operand);
        if code == UnaryOperator::Empty || code == UnaryOperator::Size {
            let shaped = match operand_type {
                Some(of) => {
                    self.keeper.list_element(of).is_some()
                        || self.keeper.is_string_type(of)
                        || self.keeper.is_dag_type(of)
                }
                None => false,
            };
            if !shaped {
                return self
                    .token_error("expected string, list, or dag type argument in unary operator");
            }
        }
        if matches!(
            code,
            UnaryOperator::Head | UnaryOperator::Tail | UnaryOperator::ListFlatten
        ) {
            let Some(of) = operand_type else {
                return self.token_error("expected list type argument in unary operator");
            };
            let Some(element) = self.keeper.list_element(of) else {
                return self.token_error("expected list type argument in unary operator");
            };
            let use_element = code == UnaryOperator::Head || code == UnaryOperator::ListFlatten;
            if let Node::List { elements, .. } = self.keeper.node(operand).clone() {
                let Some(first) = elements.first().copied() else {
                    return self.token_error("empty list argument in unary operator");
                };
                let Some(first_type) = self.keeper.type_of(first) else {
                    return self.token_error("untyped list element in unary operator");
                };
                result = Some(if use_element {
                    first_type
                } else {
                    self.keeper.list_type(first_type)
                });
            } else {
                result = Some(if use_element { element } else { of });
            }
            if code == UnaryOperator::ListFlatten {
                let held = result.unwrap_or(Type::NONE);
                result = Some(match self.keeper.list_element(held) {
                    Some(inner) => self.keeper.list_type(inner),
                    None => self.keeper.list_type(held),
                });
            }
        }
        self.expect(Token::RightParenthesis, "expected ')' in unary operator")?;
        let made = self
            .keeper
            .unary(code, operand, result.unwrap_or(Type::NONE));
        self.keeper.fold(made, current, false)
    }

    fn parse_binary(
        &mut self,
        current: Option<RecordIndex>,
        operator: BangOperator,
        item_type: Option<Type>,
    ) -> Flow<Value> {
        let operator_location = self.location();
        self.lex();
        let code = match operator {
            BangOperator::Concat => BinaryOperator::Concat,
            BangOperator::Match => BinaryOperator::Match,
            BangOperator::Add => BinaryOperator::Add,
            BangOperator::Subtract => BinaryOperator::Subtract,
            BangOperator::Multiply => BinaryOperator::Multiply,
            BangOperator::Divide => BinaryOperator::Divide,
            BangOperator::And => BinaryOperator::And,
            BangOperator::Or => BinaryOperator::Or,
            BangOperator::ExclusiveOr => BinaryOperator::ExclusiveOr,
            BangOperator::ShiftRightArithmetic => BinaryOperator::ShiftRightArithmetic,
            BangOperator::ShiftRightLogical => BinaryOperator::ShiftRightLogical,
            BangOperator::ShiftLeft => BinaryOperator::ShiftLeft,
            BangOperator::Equal => BinaryOperator::Equal,
            BangOperator::NotEqual => BinaryOperator::NotEqual,
            BangOperator::LessOrEqual => BinaryOperator::LessOrEqual,
            BangOperator::Less => BinaryOperator::Less,
            BangOperator::GreaterOrEqual => BinaryOperator::GreaterOrEqual,
            BangOperator::Greater => BinaryOperator::Greater,
            BangOperator::ListConcat => BinaryOperator::ListConcat,
            BangOperator::ListSplat => BinaryOperator::ListSplat,
            BangOperator::ListRemove => BinaryOperator::ListRemove,
            BangOperator::StringConcat => BinaryOperator::StringConcat,
            BangOperator::Interleave => BinaryOperator::Interleave,
            BangOperator::SetDagOperator => BinaryOperator::SetDagOperator,
            BangOperator::SetDagOperatorName => BinaryOperator::SetDagOperatorName,
            BangOperator::GetDagArgument => BinaryOperator::GetDagArgument,
            _ => BinaryOperator::GetDagName,
        };
        let bit = self.keeper.bit_type();
        let integer = self.keeper.integer_type();
        let string = self.keeper.string_type();
        let dag = self.keeper.dag_type();
        let (mut result, mut argument_type): (Option<Type>, Option<Type>) = match code {
            BinaryOperator::Match => (Some(bit), Some(string)),
            BinaryOperator::Concat | BinaryOperator::SetDagOperator => (Some(dag), Some(dag)),
            BinaryOperator::GetDagArgument => {
                let Ok(found) = self.parse_operator_type() else {
                    return self.token_error("did not get type for !getdagarg operator");
                };
                (Some(found), Some(dag))
            }
            BinaryOperator::SetDagOperatorName => (Some(dag), Some(dag)),
            BinaryOperator::GetDagName => (Some(string), Some(dag)),
            BinaryOperator::And
            | BinaryOperator::Or
            | BinaryOperator::ExclusiveOr
            | BinaryOperator::ShiftRightArithmetic
            | BinaryOperator::ShiftRightLogical
            | BinaryOperator::ShiftLeft
            | BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide => (Some(integer), Some(integer)),
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessOrEqual
            | BinaryOperator::Less
            | BinaryOperator::GreaterOrEqual
            | BinaryOperator::Greater => (Some(bit), None),
            BinaryOperator::ListConcat | BinaryOperator::ListRemove => (None, item_type),
            BinaryOperator::ListSplat => (None, None),
            BinaryOperator::StringConcat => (Some(string), Some(string)),
            _ => (Some(string), None),
        };
        if let (Some(of), Some(item)) = (result, item_type) {
            if !self.keeper.type_convertible(of, item) {
                let text = format!(
                    "expected value of type '{}', got '{}'",
                    self.keeper.type_name(item),
                    self.keeper.type_name(of)
                );
                return self.error(operator_location, text);
            }
        }
        self.expect(Token::LeftParenthesis, "expected '(' after binary operator")?;
        let mut values: Vec<Value> = Vec::new();
        loop {
            let value_location = self.location();
            let value = self.parse_value(current, argument_type, NameMode::Value)?;
            values.push(value);
            let Some(value_type) = self.keeper.type_of(value) else {
                let text = format!(
                    "expected value to be a typed value, got '{}'",
                    self.keeper.text_of(value)
                );
                return self.error(operator_location, text);
            };
            match argument_type {
                None => {
                    argument_type = Some(value_type);
                    let held = value_type;
                    match code {
                        BinaryOperator::ListConcat | BinaryOperator::ListRemove => {
                            if self.keeper.list_element(held).is_none() {
                                let text = format!(
                                    "expected a list, got value of type '{}'",
                                    self.keeper.type_name(held)
                                );
                                return self.error(value_location, text);
                            }
                        }
                        BinaryOperator::ListSplat => {
                            if let Some(item) = item_type {
                                if values.len() == 1 {
                                    let Some(item_element) = self.keeper.list_element(item) else {
                                        let text = format!(
                                            "expected output type to be a list, got type '{}'",
                                            self.keeper.type_name(item)
                                        );
                                        return self.error(operator_location, text);
                                    };
                                    let listed = self.keeper.list_type(held);
                                    if !self.keeper.type_convertible(listed, item) {
                                        let text = format!(
                                            "expected first arg type to be '{}', got value of type '{}'",
                                            self.keeper.type_name(held),
                                            self.keeper.type_name(item_element)
                                        );
                                        return self.error(operator_location, text);
                                    }
                                }
                            }
                            if values.len() == 2 && !self.keeper.is_integer(held) {
                                let text = format!(
                                    "expected second parameter to be an int, got value of type '{}'",
                                    self.keeper.type_name(held)
                                );
                                return self.error(value_location, text);
                            }
                            argument_type = None;
                        }
                        BinaryOperator::Equal | BinaryOperator::NotEqual => {
                            let any_record = self.keeper.record_type(&[]);
                            if !self.keeper.type_convertible(held, integer)
                                && !self.keeper.type_convertible(held, string)
                                && !self.keeper.type_convertible(held, any_record)
                            {
                                let text = format!(
                                    "expected bit, bits, int, string, or record; got value of type '{}'",
                                    self.keeper.type_name(held)
                                );
                                return self.error(value_location, text);
                            }
                        }
                        BinaryOperator::GetDagArgument
                        | BinaryOperator::LessOrEqual
                        | BinaryOperator::Less
                        | BinaryOperator::GreaterOrEqual
                        | BinaryOperator::Greater => {
                            if !self.keeper.type_convertible(held, integer)
                                && !self.keeper.type_convertible(held, string)
                            {
                                let text = format!(
                                    "expected bit, bits, int, or string; got value of type '{}'",
                                    self.keeper.type_name(held)
                                );
                                return self.error(value_location, text);
                            }
                        }
                        BinaryOperator::Interleave => {
                            match values.len() {
                                1 => {
                                    let string_list = self.keeper.list_type(string);
                                    let integer_list = self.keeper.list_type(integer);
                                    if held != string_list
                                        && !self.keeper.type_convertible(held, integer_list)
                                    {
                                        let text = format!(
                                            "expected list of string, int, bits, or bit; got value of type '{}'",
                                            self.keeper.type_name(held)
                                        );
                                        return self.error(value_location, text);
                                    }
                                }
                                2 if !self.keeper.is_string_type(held) => {
                                    let text = format!(
                                        "expected second argument to be a string, got value of type '{}'",
                                        self.keeper.type_name(held)
                                    );
                                    return self.error(value_location, text);
                                }
                                _ => {}
                            }
                            argument_type = None;
                        }
                        _ => {}
                    }
                }
                Some(expected) => {
                    let Some(resolved) = self.keeper.resolve_types(expected, value_type) else {
                        let text = format!(
                            "expected value of type '{}', got '{}'",
                            self.keeper.type_name(expected),
                            self.keeper.type_name(value_type)
                        );
                        return self.error(value_location, text);
                    };
                    if !matches!(
                        code,
                        BinaryOperator::Add
                            | BinaryOperator::Subtract
                            | BinaryOperator::And
                            | BinaryOperator::Or
                            | BinaryOperator::ExclusiveOr
                            | BinaryOperator::ShiftRightArithmetic
                            | BinaryOperator::ShiftRightLogical
                            | BinaryOperator::ShiftLeft
                            | BinaryOperator::Multiply
                            | BinaryOperator::Divide
                    ) {
                        argument_type = Some(resolved);
                    }
                }
            }
            match code {
                BinaryOperator::SetDagOperatorName => argument_type = Some(string),
                BinaryOperator::SetDagOperator => {
                    argument_type = Some(self.keeper.record_type(&[]))
                }
                BinaryOperator::GetDagArgument => argument_type = None,
                BinaryOperator::GetDagName => argument_type = Some(integer),
                _ => {}
            }
            if !self.consume(Token::Comma) {
                break;
            }
        }
        self.expect(Token::RightParenthesis, "expected ')' in operator")?;
        if code == BinaryOperator::ListConcat || code == BinaryOperator::ListRemove {
            result = argument_type;
        }
        if code == BinaryOperator::ListSplat {
            let first = self.type_or_none(values[0]);
            result = Some(self.keeper.list_type(first));
        }
        let result = result.unwrap_or(Type::NONE);
        if matches!(
            code,
            BinaryOperator::StringConcat
                | BinaryOperator::ListConcat
                | BinaryOperator::Concat
                | BinaryOperator::Add
                | BinaryOperator::And
                | BinaryOperator::Or
                | BinaryOperator::ExclusiveOr
                | BinaryOperator::Multiply
        ) {
            while values.len() > 2 {
                let right = values.pop().unwrap_or(self.keeper.unset());
                let left = *values.last().unwrap_or(&right);
                let made = self.keeper.binary(code, left, right, result);
                let folded = self.keeper.fold(made, current, false)?;
                if let Some(last) = values.last_mut() {
                    *last = folded;
                }
            }
        }
        if values.len() == 2 {
            let made = self.keeper.binary(code, values[0], values[1], result);
            return self.keeper.fold(made, current, false);
        }
        self.error(operator_location, "expected two operands to operator")
    }

    fn parse_range(&mut self, current: Option<RecordIndex>) -> Flow<Value> {
        let operator_location = self.location();
        self.lex();
        self.expect(Token::LeftParenthesis, "expected '(' after !range operator")?;
        let mut arguments: Vec<Value> = Vec::new();
        let mut first_is_list = false;
        loop {
            if arguments.len() >= 3 {
                return self.token_error("expected at most three values of integer");
            }
            let value_location = self.location();
            let value = self.parse_value(current, None, NameMode::Value)?;
            arguments.push(value);
            let Some(of) = self.keeper.type_of(value) else {
                let text = format!(
                    "expected value to be a typed value, got '{}'",
                    self.keeper.text_of(value)
                );
                return self.error(operator_location, text);
            };
            if !first_is_list || arguments.len() == 1 {
                if arguments.len() == 1 && self.keeper.list_element(of).is_some() {
                    first_is_list = true;
                } else if self.keeper.is_integer(of) {
                } else {
                    let text = if arguments.len() != 1 {
                        format!(
                            "expected value of type 'int', got '{}'",
                            self.keeper.type_name(of)
                        )
                    } else {
                        format!(
                            "expected list or int, got value of type '{}'",
                            self.keeper.type_name(of)
                        )
                    };
                    return self.error(value_location, text);
                }
            } else {
                let text = format!(
                    "expected one list, got extra value of type '{}'",
                    self.keeper.type_name(of)
                );
                return self.error(value_location, text);
            }
            if !self.consume(Token::Comma) {
                break;
            }
        }
        self.expect(Token::RightParenthesis, "expected ')' in operator")?;
        let integer = self.keeper.integer_type();
        let first = arguments[0];
        let (left, middle, right);
        if arguments.len() == 1 {
            left = self.keeper.integer(0);
            if first_is_list {
                let made = self.keeper.unary(UnaryOperator::Size, first, integer);
                middle = self.keeper.fold(made, current, false)?;
            } else {
                middle = first;
            }
            right = self.keeper.integer(1);
        } else {
            left = first;
            middle = arguments[1];
            right = if arguments.len() == 3 {
                arguments[2]
            } else {
                self.keeper.integer(1)
            };
        }
        let list = self.keeper.list_type(integer);
        let made = self
            .keeper
            .ternary(TernaryOperator::Range, left, middle, right, list);
        self.keeper.fold(made, current, false)
    }

    fn parse_ternary(
        &mut self,
        current: Option<RecordIndex>,
        operator: BangOperator,
        mut item_type: Option<Type>,
    ) -> Flow<Value> {
        self.lex();
        let dag = self.keeper.dag_type();
        let (code, mut result) = match operator {
            BangOperator::Dag => {
                item_type = None;
                (TernaryOperator::Dag, Some(dag))
            }
            BangOperator::If => (TernaryOperator::If, None),
            BangOperator::Substitute => (TernaryOperator::Substitute, None),
            BangOperator::SetDagArgument => {
                item_type = None;
                (TernaryOperator::SetDagArgument, Some(dag))
            }
            _ => {
                item_type = None;
                (TernaryOperator::SetDagName, Some(dag))
            }
        };
        self.expect(
            Token::LeftParenthesis,
            "expected '(' after ternary operator",
        )?;
        let left = self.parse_value(current, None, NameMode::Value)?;
        self.expect(Token::Comma, "expected ',' in ternary operator")?;
        let middle_location = self.location();
        let middle = self.parse_value(current, item_type, NameMode::Value)?;
        self.expect(Token::Comma, "expected ',' in ternary operator")?;
        let right_location = self.location();
        let right = self.parse_value(current, item_type, NameMode::Value)?;
        self.expect(Token::RightParenthesis, "expected ')' in binary operator")?;
        match code {
            TernaryOperator::Dag => {
                let middle_type = self.keeper.type_of(middle);
                if middle_type.is_none() && !self.keeper.is_unset(middle) {
                    return self.error(
                        middle_location,
                        "could not determine type of the child list in !dag",
                    );
                }
                if let Some(of) = middle_type {
                    if self.keeper.list_element(of).is_none() {
                        let text = format!(
                            "expected list of children, got type '{}'",
                            self.keeper.type_name(of)
                        );
                        return self.error(middle_location, text);
                    }
                }
                let right_type = self.keeper.type_of(right);
                if right_type.is_none() && !self.keeper.is_unset(right) {
                    return self.error(
                        right_location,
                        "could not determine type of the name list in !dag",
                    );
                }
                if let Some(of) = right_type {
                    let string = self.keeper.string_type();
                    let string_list = self.keeper.list_type(string);
                    if of != string_list {
                        let text = format!(
                            "expected list<string>, got type '{}'",
                            self.keeper.type_name(of)
                        );
                        return self.error(right_location, text);
                    }
                }
                if middle_type.is_none() && right_type.is_none() {
                    return self.error(
                        middle_location,
                        "cannot have both unset children and unset names in !dag",
                    );
                }
            }
            TernaryOperator::If => {
                let mut middle_type = self.keeper.type_of(middle);
                let mut right_type = self.keeper.type_of(right);
                if self.keeper.is_unset(middle) {
                    middle_type = right_type;
                }
                if self.keeper.is_unset(right) {
                    right_type = middle_type;
                }
                let (Some(first), Some(second)) = (middle_type, right_type) else {
                    return self.token_error("could not get type for !if");
                };
                match self.keeper.resolve_types(first, second) {
                    Some(found) => result = Some(found),
                    None => {
                        let text = format!(
                            "inconsistent types '{}' and '{}' for !if",
                            self.keeper.type_name(first),
                            self.keeper.type_name(second)
                        );
                        return self.token_error(text);
                    }
                }
            }
            TernaryOperator::Substitute => match self.keeper.type_of(right) {
                Some(of) => result = Some(of),
                None => return self.token_error("could not get type for !subst"),
            },
            _ => {
                let middle_type = self.keeper.type_of(middle);
                let acceptable = middle_type
                    .is_some_and(|of| self.keeper.is_integer(of) || self.keeper.is_string_type(of));
                if !acceptable {
                    let text = match middle_type {
                        Some(of) => format!(
                            "expected integer index or string name, got type '{}'",
                            self.keeper.type_name(of)
                        ),
                        None => format!(
                            "expected integer index or string name, got '{}'",
                            self.keeper.text_of(middle)
                        ),
                    };
                    return self.error(middle_location, text);
                }
                if code == TernaryOperator::SetDagName {
                    if let Some(of) = self.keeper.type_of(right) {
                        if !self.keeper.is_string_type(of) {
                            let text = format!(
                                "expected string or unset name, got type '{}'",
                                self.keeper.type_name(of)
                            );
                            return self.error(right_location, text);
                        }
                    }
                }
            }
        }
        let made = self
            .keeper
            .ternary(code, left, middle, right, result.unwrap_or(Type::NONE));
        self.keeper.fold(made, current, false)
    }

    fn parse_substring(
        &mut self,
        current: Option<RecordIndex>,
        item_type: Option<Type>,
    ) -> Flow<Value> {
        let string = self.keeper.string_type();
        self.lex();
        self.expect(
            Token::LeftParenthesis,
            "expected '(' after !substr operator",
        )?;
        let left = self.parse_value(current, None, NameMode::Value)?;
        self.expect(Token::Comma, "expected ',' in !substr operator")?;
        let middle_location = self.location();
        let middle = self.parse_value(current, None, NameMode::Value)?;
        let mut right_location = self.location();
        let right = if self.consume(Token::Comma) {
            right_location = self.location();
            self.parse_value(current, None, NameMode::Value)?
        } else {
            self.keeper.integer(i64::MAX)
        };
        self.expect(Token::RightParenthesis, "expected ')' in !substr operator")?;
        if let Some(item) = item_type {
            if !self.keeper.type_convertible(string, item) {
                let text = format!(
                    "expected value of type '{}', got '{}'",
                    self.keeper.type_name(item),
                    self.keeper.type_name(string)
                );
                self.report_error(right_location, text);
            }
        }
        match self.keeper.type_of(left) {
            None if !self.keeper.is_unset(left) => {
                return self.token_error("could not determine type of the string in !substr");
            }
            Some(of) if !self.keeper.is_string_type(of) => {
                let text = format!("expected string, got type '{}'", self.keeper.type_name(of));
                return self.token_error(text);
            }
            _ => {}
        }
        match self.keeper.type_of(middle) {
            None if !self.keeper.is_unset(middle) => {
                return self
                    .token_error("could not determine type of the start position in !substr");
            }
            Some(of) if !self.keeper.is_integer(of) => {
                let text = format!("expected int, got type '{}'", self.keeper.type_name(of));
                return self.error(middle_location, text);
            }
            _ => {}
        }
        match self.keeper.type_of(right) {
            None if !self.keeper.is_unset(right) => {
                return self.token_error("could not determine type of the length in !substr");
            }
            Some(of) if !self.keeper.is_integer(of) => {
                let text = format!("expected int, got type '{}'", self.keeper.type_name(of));
                return self.token_error(text);
            }
            _ => {}
        }
        let made = self
            .keeper
            .ternary(TernaryOperator::Substring, left, middle, right, string);
        self.keeper.fold(made, current, false)
    }

    fn parse_find(&mut self, current: Option<RecordIndex>, item_type: Option<Type>) -> Flow<Value> {
        let integer = self.keeper.integer_type();
        self.lex();
        self.expect(Token::LeftParenthesis, "expected '(' after !find operator")?;
        let left = self.parse_value(current, None, NameMode::Value)?;
        self.expect(Token::Comma, "expected ',' in !find operator")?;
        let middle_location = self.location();
        let middle = self.parse_value(current, None, NameMode::Value)?;
        let mut right_location = self.location();
        let right = if self.consume(Token::Comma) {
            right_location = self.location();
            self.parse_value(current, None, NameMode::Value)?
        } else {
            self.keeper.integer(0)
        };
        self.expect(Token::RightParenthesis, "expected ')' in !find operator")?;
        if let Some(item) = item_type {
            if !self.keeper.type_convertible(integer, item) {
                let text = format!(
                    "expected value of type '{}', got '{}'",
                    self.keeper.type_name(item),
                    self.keeper.type_name(integer)
                );
                self.report_error(right_location, text);
            }
        }
        match self.keeper.type_of(left) {
            None if !self.keeper.is_unset(left) => {
                return self.token_error("could not determine type of the source string in !find");
            }
            Some(of) if !self.keeper.is_string_type(of) => {
                let text = format!("expected string, got type '{}'", self.keeper.type_name(of));
                return self.token_error(text);
            }
            _ => {}
        }
        match self.keeper.type_of(middle) {
            None if !self.keeper.is_unset(middle) => {
                return self.token_error("could not determine type of the target string in !find");
            }
            Some(of) if !self.keeper.is_string_type(of) => {
                let text = format!("expected string, got type '{}'", self.keeper.type_name(of));
                return self.error(middle_location, text);
            }
            _ => {}
        }
        match self.keeper.type_of(right) {
            None if !self.keeper.is_unset(right) => {
                return self.token_error("could not determine type of the start position in !find");
            }
            Some(of) if !self.keeper.is_integer(of) => {
                let text = format!("expected int, got type '{}'", self.keeper.type_name(of));
                return self.token_error(text);
            }
            _ => {}
        }
        let made = self
            .keeper
            .ternary(TernaryOperator::Find, left, middle, right, integer);
        self.keeper.fold(made, current, false)
    }

    fn temporary_record(&mut self, current: Option<RecordIndex>) -> Flow<(RecordIndex, bool)> {
        if let Some(record) = current {
            return Ok((record, false));
        }
        let name = self.keeper.string(".parse");
        let made = self
            .keeper
            .new_record(name, Vec::new(), RecordKind::Definition)?;
        Ok((made, true))
    }

    fn parse_list_comprehension(
        &mut self,
        current: Option<RecordIndex>,
        item_type: Option<Type>,
    ) -> Flow<Value> {
        let operator_location = self.location();
        let Token::Bang(operation) = self.token() else {
            return Err(Stop);
        };
        self.lex();
        if self.token() != Token::LeftParenthesis {
            return self.token_error("expected '(' after !foreach/!filter");
        }
        if self.lex() != Token::Identifier {
            return self.token_error("first argument of !foreach/!filter must be an identifier");
        }
        let left = self.keeper.string(&self.lexer.text.clone());
        self.lex();
        if let Some(record) = current {
            if self.keeper.record(record).field(left).is_some() {
                let text = format!(
                    "iteration variable '{}' is already defined",
                    self.keeper.text_of(left)
                );
                return self.token_error(text);
            }
        }
        self.expect(Token::Comma, "expected ',' in !foreach/!filter")?;
        let middle = self.parse_value(current, None, NameMode::Value)?;
        self.expect(Token::Comma, "expected ',' in !foreach/!filter")?;
        let Some(middle_type) = self.keeper.type_of(middle) else {
            return self.token_error("could not get type of !foreach/!filter list or dag");
        };
        let mut expression_element = None;
        let input_element;
        let mut is_dag = false;
        if let Some(element) = self.keeper.list_element(middle_type) {
            input_element = element;
            if let Some(item) = item_type {
                match self.keeper.list_element(item) {
                    Some(output) => {
                        expression_element = match operation {
                            BangOperator::ForEach => Some(output),
                            BangOperator::Filter => Some(self.keeper.integer_type()),
                            _ => None,
                        };
                    }
                    None => {
                        let text = format!(
                            "expected value of type '{}', but got list type",
                            self.keeper.type_name(item)
                        );
                        return self.error(operator_location, text);
                    }
                }
            }
        } else if self.keeper.is_dag_type(middle_type) {
            match operation {
                BangOperator::Filter => {
                    return self.token_error("!filter must have a list argument")
                }
                BangOperator::Sort => return self.token_error("!sort must have a list argument"),
                _ => {}
            }
            input_element = middle_type;
            if let Some(item) = item_type {
                if !self.keeper.is_dag_type(item) {
                    let text = format!(
                        "expected value of type '{}', but got dag type",
                        self.keeper.type_name(item)
                    );
                    return self.error(operator_location, text);
                }
            }
            is_dag = true;
        } else {
            return match operation {
                BangOperator::ForEach => {
                    self.token_error("!foreach must have a list or dag argument")
                }
                BangOperator::Filter => self.token_error("!filter must have a list argument"),
                _ => self.token_error("!sort must have a list argument"),
            };
        }
        let (holder, temporary) = self.temporary_record(current)?;
        self.push_scope(ScopeKind::Record(holder));
        let location = Location::default();
        let unset = self.keeper.initial_value(input_element);
        self.keeper.record_mut(holder).fields.push(RecordField {
            name: left,
            location,
            field_type: input_element,
            kind: FieldKind::Normal,
            value: unset,
            used: false,
        });
        let outcome = self.parse_value(Some(holder), expression_element, NameMode::Value);
        if let Some(at) = self.keeper.record(holder).field(left) {
            self.keeper.record_mut(holder).fields.remove(at);
        }
        self.pop_scope();
        if temporary {
            self.keeper.release_record(holder);
        }
        let right = outcome?;
        self.expect(Token::RightParenthesis, "expected ')' in !foreach/!filter")?;
        let (code, output) = match operation {
            BangOperator::ForEach => {
                let output = if is_dag {
                    input_element
                } else {
                    let Some(of) = self.keeper.type_of(right) else {
                        return self
                            .token_error("could not get type of !foreach result expression");
                    };
                    self.keeper.list_type(of)
                };
                (TernaryOperator::ForEach, output)
            }
            BangOperator::Filter => (
                TernaryOperator::Filter,
                self.keeper.list_type(input_element),
            ),
            _ => (TernaryOperator::Sort, self.keeper.list_type(input_element)),
        };
        let made = self.keeper.ternary(code, left, middle, right, output);
        self.keeper.fold(made, current, false)
    }

    fn resolve_value_types(&mut self, values: &[Value], context: &str) -> Flow<Option<Type>> {
        let mut found: Option<Type> = None;
        for value in values {
            if self.keeper.is_unset(*value) {
                continue;
            }
            let Some(of) = self.keeper.type_of(*value) else {
                continue;
            };
            match found {
                None => found = Some(of),
                Some(held) => match self.keeper.resolve_types(held, of) {
                    Some(resolved) => found = Some(resolved),
                    None => {
                        let text = format!(
                            "inconsistent types '{}' and '{}' {context}",
                            self.keeper.type_name(held),
                            self.keeper.type_name(of)
                        );
                        return self.token_error(text);
                    }
                },
            }
        }
        Ok(found)
    }

    fn parse_condition(
        &mut self,
        current: Option<RecordIndex>,
        item_type: Option<Type>,
    ) -> Flow<Value> {
        self.lex();
        self.expect(Token::LeftParenthesis, "expected '(' after !cond operator")?;
        let mut cases = Vec::new();
        let mut values = Vec::new();
        loop {
            if self.consume(Token::RightParenthesis) {
                break;
            }
            let case = self.parse_value(current, None, NameMode::Value)?;
            cases.push(case);
            self.expect(
                Token::Colon,
                "expected ':'  following a condition in !cond operator",
            )?;
            let value = self.parse_value(current, item_type, NameMode::Value)?;
            values.push(value);
            if self.consume(Token::RightParenthesis) {
                break;
            }
            self.expect(
                Token::Comma,
                "expected ',' or ')' following a value in !cond operator",
            )?;
        }
        if cases.is_empty() {
            return self.token_error(
                "there should be at least 1 'condition : value' in the !cond operator",
            );
        }
        let Some(result) = self.resolve_value_types(&values, "for !cond")? else {
            return self.token_error("could not determine type for !cond from its arguments");
        };
        let made = self.keeper.condition(cases, values, result);
        self.keeper.fold(made, current, false)
    }

    fn parse_switch(
        &mut self,
        current: Option<RecordIndex>,
        item_type: Option<Type>,
    ) -> Flow<Value> {
        self.lex();
        self.expect(
            Token::LeftParenthesis,
            "expected '(' after !switch operator",
        )?;
        let key = self.parse_value(current, None, NameMode::Value)?;
        let mut keys = vec![key];
        self.expect(Token::Comma, "expected ',' after key in !switch operator")?;
        let mut values = Vec::new();
        loop {
            let mut value = self.parse_value(current, None, NameMode::Value)?;
            if self.consume(Token::RightParenthesis) {
                if let Some(item) = item_type {
                    if let Some(coerced) = self.keeper.convert_to(value, item) {
                        value = coerced;
                    }
                }
                values.push(value);
                break;
            }
            self.expect(
                Token::Colon,
                "expected ':' after case key, or ')' to close !switch operator",
            )?;
            keys.push(value);
            let value = self.parse_value(current, item_type, NameMode::Value)?;
            values.push(value);
            self.expect(
                Token::Comma,
                "expected ',' after case value in !switch operator",
            )?;
        }
        if keys.len() < 2 {
            return self
                .token_error("there should be at least 1 'case: value' in the !switch operator");
        }
        let Some(result) = self.resolve_value_types(&values, "for !switch values")? else {
            return self.token_error("could not determine type for !switch from its arguments");
        };
        self.resolve_value_types(&keys, "between !switch key and case keys")?;
        let bit = self.keeper.bit_type();
        let mut conditions = Vec::with_capacity(values.len());
        for case in keys.iter().skip(1) {
            let made = self.keeper.binary(BinaryOperator::Equal, key, *case, bit);
            conditions.push(self.keeper.fold(made, current, false)?);
        }
        conditions.push(self.keeper.integer(1));
        let made = self.keeper.condition(conditions, values, result);
        self.keeper.fold(made, current, false)
    }

    fn parse_fold(&mut self, current: Option<RecordIndex>) -> Flow<Value> {
        self.lex();
        self.expect(Token::LeftParenthesis, "expected '(' after !foldl")?;
        let start = self.parse_value(current, None, NameMode::Value)?;
        let Some(start_type) = self.keeper.type_of(start) else {
            let text = format!(
                "could not get type of !foldl start: '{}'",
                self.keeper.text_of(start)
            );
            return self.token_error(text);
        };
        self.expect(Token::Comma, "expected ',' in !foldl")?;
        let list = self.parse_value(current, None, NameMode::Value)?;
        let Some(list_type) = self.keeper.type_of(list) else {
            let text = format!(
                "could not get type of !foldl list: '{}'",
                self.keeper.text_of(list)
            );
            return self.token_error(text);
        };
        let Some(element_type) = self.keeper.list_element(list_type) else {
            let text = format!(
                "!foldl list must be a list, but is of type '{}'",
                self.keeper.type_name(list_type)
            );
            return self.token_error(text);
        };
        if self.token() != Token::Comma {
            return self.token_error("expected ',' in !foldl");
        }
        if self.lex() != Token::Identifier {
            return self.token_error("third argument of !foldl must be an identifier");
        }
        let accumulator = self.keeper.string(&self.lexer.text.clone());
        if let Some(record) = current {
            if self.keeper.record(record).field(accumulator).is_some() {
                let text = format!(
                    "left !foldl variable '{}' already defined",
                    self.keeper.text_of(accumulator)
                );
                return self.token_error(text);
            }
        }
        if self.lex() != Token::Comma {
            return self.token_error("expected ',' in !foldl");
        }
        if self.lex() != Token::Identifier {
            return self.token_error("fourth argument of !foldl must be an identifier");
        }
        let element = self.keeper.string(&self.lexer.text.clone());
        if let Some(record) = current {
            if self.keeper.record(record).field(element).is_some() {
                let text = format!(
                    "right !foldl variable '{}' already defined",
                    self.keeper.text_of(element)
                );
                return self.token_error(text);
            }
        }
        if self.lex() != Token::Comma {
            return self.token_error("expected ',' in !foldl");
        }
        self.lex();
        let (holder, temporary) = self.temporary_record(current)?;
        self.push_scope(ScopeKind::Record(holder));
        for (name, of) in [(accumulator, start_type), (element, element_type)] {
            let unset = self.keeper.initial_value(of);
            self.keeper.record_mut(holder).fields.push(RecordField {
                name,
                location: Location::default(),
                field_type: of,
                kind: FieldKind::Normal,
                value: unset,
                used: false,
            });
        }
        let outcome = self.parse_value(Some(holder), None, NameMode::Value);
        for name in [accumulator, element] {
            if let Some(at) = self.keeper.record(holder).field(name) {
                self.keeper.record_mut(holder).fields.remove(at);
            }
        }
        self.pop_scope();
        if temporary {
            self.keeper.release_record(holder);
        }
        let expression = outcome?;
        let Some(expression_type) = self.keeper.type_of(expression) else {
            return self.token_error("could not get type of !foldl expression");
        };
        if expression_type != start_type {
            let text = format!(
                "!foldl expression must be of same type as start ({}), but is of type {}",
                self.keeper.type_name(start_type),
                self.keeper.type_name(expression_type)
            );
            return self.token_error(text);
        }
        self.expect(Token::RightParenthesis, "expected ')' in fold operator")?;
        let made = self
            .keeper
            .fold_node(start, list, accumulator, element, expression, start_type);
        self.keeper.fold(made, current, false)
    }
}

impl<'a> Parser<'a> {
    fn parse_simple_value(
        &mut self,
        current: Option<RecordIndex>,
        item_type: Option<Type>,
        mode: NameMode,
    ) -> Flow<Value> {
        match self.token() {
            Token::Bang(_) => self.parse_operation(current, item_type),
            Token::True => {
                self.lex();
                Ok(self.keeper.integer(1))
            }
            Token::False => {
                self.lex();
                Ok(self.keeper.integer(0))
            }
            Token::Integer => {
                let value = self.lexer.integer;
                self.lex();
                Ok(self.keeper.integer(value))
            }
            Token::BinaryInteger => {
                let value = self.lexer.integer as u64;
                let width = self.lexer.binary_width();
                let bits = (0..width)
                    .map(|index| self.keeper.bit(index < 64 && value & (1u64 << index) != 0))
                    .collect();
                self.lex();
                Ok(self.keeper.bits(bits))
            }
            Token::StringValue => {
                let mut text = self.lexer.text.clone();
                self.lex();
                while self.token() == Token::StringValue {
                    text.push_str(&self.lexer.text);
                    self.lex();
                }
                Ok(self.keeper.string(&text))
            }
            Token::CodeFragment => {
                let text = self.lexer.text.clone();
                self.lex();
                Ok(self.keeper.string_with_format(&text, true))
            }
            Token::Question => {
                self.lex();
                Ok(self.keeper.unset())
            }
            Token::Identifier => {
                let start = self.location();
                let text = self.lexer.text.clone();
                let name = self.keeper.string(&text);
                let next = self.lex();
                if next == Token::Equal {
                    return Ok(name);
                }
                if next != Token::Less {
                    return self.parse_identifier_value(current, name, start, mode);
                }
                let Some(class) = self.keeper.find_class(&text) else {
                    return self.error(start, format!("Expected a class name, got '{text}'"));
                };
                self.lex();
                let (mut arguments, locations) =
                    self.parse_template_argument_values(current, class)?;
                self.check_template_argument_values(&mut arguments, &locations, class)?;
                self.resolve_arguments(class, &arguments, start)?;
                let made = self.keeper.variable_definition(start, class, arguments);
                self.keeper.fold(made, current, false)
            }
            Token::LeftBrace => {
                let brace = self.location();
                self.lex();
                let mut values = Vec::new();
                if self.token() != Token::RightBrace {
                    values = self.parse_value_list(current, None)?;
                }
                if !self.consume(Token::RightBrace) {
                    return self.token_error("expected '}' at end of bit list value");
                }
                let mut bits = Vec::new();
                for (index, value) in values.iter().enumerate() {
                    if let Node::Bits(held) = self.keeper.node(*value) {
                        for bit in held.iter().rev() {
                            bits.push(*bit);
                        }
                        continue;
                    }
                    if let Node::Variable { variable_type, .. } = self.keeper.node(*value) {
                        if let Some(width) = self.keeper.bits_width(*variable_type) {
                            for bit in (0..width).rev() {
                                bits.push(self.keeper.get_bit(*value, bit));
                            }
                            continue;
                        }
                    }
                    let bit_type = self.keeper.bit_type();
                    let Some(bit) = self.keeper.cast_to(*value, bit_type)? else {
                        let text = format!(
                            "Element #{index} ({}) is not convertable to a bit",
                            self.keeper.text_of(*value)
                        );
                        return self.error(brace, text);
                    };
                    bits.push(bit);
                }
                bits.reverse();
                Ok(self.keeper.bits(bits))
            }
            Token::LeftSquare => {
                self.lex();
                let mut given_list = None;
                if let Some(item) = item_type {
                    match self.keeper.list_element(item) {
                        Some(element) => given_list = Some(element),
                        None => {
                            let text = format!(
                                "Encountered a list when expecting a {}",
                                self.keeper.type_name(item)
                            );
                            return self.token_error(text);
                        }
                    }
                }
                let mut values = Vec::new();
                if self.token() != Token::RightSquare {
                    values = self.parse_value_list(current, given_list)?;
                }
                if !self.consume(Token::RightSquare) {
                    return self.token_error("expected ']' at end of list value");
                }
                let mut given_element = None;
                if self.consume(Token::Less) {
                    given_element = Some(self.parse_type()?);
                    if !self.consume(Token::Greater) {
                        return self.token_error("expected '>' at end of list element type");
                    }
                }
                let mut element: Option<Type> = None;
                for value in &values {
                    if let Some(of) = self.keeper.type_of(*value) {
                        element = match element {
                            Some(held) => match self.keeper.resolve_types(held, of) {
                                Some(found) => Some(found),
                                None => {
                                    return self.token_error("Incompatible types in list elements")
                                }
                            },
                            None => Some(of),
                        };
                    }
                }
                if let Some(given) = given_element {
                    if let Some(held) = element {
                        if !self.keeper.type_convertible(held, given) {
                            return self.token_error("Incompatible types in list elements");
                        }
                    }
                    element = Some(given);
                }
                let deduced = match element {
                    None => match given_list {
                        Some(given) => given,
                        None => return self.token_error("No type for list"),
                    },
                    Some(held) => {
                        if let Some(given) = given_list {
                            if !self.keeper.type_convertible(held, given) {
                                let text = format!(
                                    "Element type mismatch for list: element type '{}' not convertible to '{}",
                                    self.keeper.type_name(held),
                                    self.keeper.type_name(given)
                                );
                                return self.token_error(text);
                            }
                        }
                        held
                    }
                };
                Ok(self.keeper.list(values, deduced))
            }
            Token::LeftParenthesis => {
                self.lex();
                if !matches!(
                    self.token(),
                    Token::Identifier
                        | Token::Bang(BangOperator::Cast)
                        | Token::Question
                        | Token::Bang(BangOperator::GetDagOperator)
                        | Token::LeftSquare
                ) {
                    return self
                        .token_error("expected identifier or list of value types in dag init");
                }
                let operator = self.parse_value(current, None, NameMode::Value)?;
                let mut operator_name = None;
                if self.consume(Token::Colon) {
                    if self.token() != Token::VariableName {
                        return self.token_error("expected variable name in dag operator");
                    }
                    operator_name = Some(self.keeper.string(&self.lexer.text.clone()));
                    self.lex();
                }
                let mut arguments = Vec::new();
                let mut names = Vec::new();
                if self.token() != Token::RightParenthesis {
                    let pairs = self.parse_dag_arguments(current)?;
                    for (argument, name) in pairs {
                        arguments.push(argument);
                        names.push(name);
                    }
                }
                if !self.consume(Token::RightParenthesis) {
                    return self.token_error("expected ')' in dag init");
                }
                Ok(self.keeper.dag(operator, operator_name, arguments, names))
            }
            _ => self.token_error("Unknown or reserved token when parsing a value"),
        }
    }

    fn parse_value(
        &mut self,
        current: Option<RecordIndex>,
        item_type: Option<Type>,
        mode: NameMode,
    ) -> Flow<Value> {
        let left_location = self.location();
        let mut result = self.parse_simple_value(current, item_type, mode)?;
        loop {
            match self.token() {
                Token::LeftBrace => {
                    if mode == NameMode::Name {
                        return Ok(result);
                    }
                    let curly = self.location();
                    self.lex();
                    let mut ranges = self.parse_range_list()?;
                    ranges.reverse();
                    let Some(converted) = self.keeper.convert_bit_range(result, &ranges) else {
                        return self.error(curly, "Invalid bit range for value");
                    };
                    result = converted;
                    if !self.consume(Token::RightBrace) {
                        return self.token_error("expected '}' at end of bit range list");
                    }
                }
                Token::LeftSquare => {
                    let Some(of) = self.keeper.type_of(result) else {
                        return self.error(left_location, "Invalid value, list expected");
                    };
                    let Some(element) = self.keeper.list_element(of) else {
                        let text = format!(
                            "Type '{}' is invalid, list expected",
                            self.keeper.type_name(of)
                        );
                        return self.error(left_location, text);
                    };
                    self.lex();
                    let right = self.parse_slice_elements(current, true)?;
                    let right_type = self.type_or_none(right);
                    let made = if self.keeper.list_element(right_type).is_some() {
                        self.keeper
                            .binary(BinaryOperator::ListSlice, result, right, of)
                    } else {
                        self.keeper
                            .binary(BinaryOperator::ListElement, result, right, element)
                    };
                    result = self.keeper.fold(made, current, false)?;
                    if !self.consume(Token::RightSquare) {
                        return self.token_error("expected ']' at end of list slice");
                    }
                }
                Token::Dot => {
                    if self.lex() != Token::Identifier {
                        return self.token_error("expected field identifier after '.'");
                    }
                    let text = self.lexer.text.clone();
                    let field = self.keeper.string(&text);
                    if self.keeper.field_type(result, field).is_none() {
                        let message = format!(
                            "Cannot access field '{text}' of value '{}'",
                            self.keeper.text_of(result)
                        );
                        return self.token_error(message);
                    }
                    let made = self.keeper.field_reference(result, field);
                    result = self.keeper.fold(made, current, false)?;
                    self.lex();
                }
                Token::Paste => {
                    let paste = self.location();
                    let Some(of) = self.keeper.type_of(result) else {
                        return self.error(paste, "LHS of paste is not typed!");
                    };
                    if self.keeper.list_element(of).is_some() {
                        self.lex();
                        match self.token() {
                            Token::Colon | Token::Semicolon | Token::LeftBrace => {}
                            _ => {
                                let right =
                                    self.parse_value(current, item_type, NameMode::Value)?;
                                result = self.list_concat(result, right);
                            }
                        }
                        continue;
                    }
                    let string = self.keeper.string_type();
                    let mut left = result;
                    if of != string {
                        let made = self.keeper.unary(UnaryOperator::Cast, left, string);
                        let cast = self.keeper.fold(made, current, false)?;
                        if self.keeper.type_of(cast).is_none() {
                            let text =
                                format!("can't cast '{}' to string", self.keeper.text_of(left));
                            return self.error(paste, text);
                        }
                        left = cast;
                    }
                    self.lex();
                    let right = match self.token() {
                        Token::Colon | Token::Semicolon | Token::LeftBrace => {
                            self.keeper.string("")
                        }
                        _ => {
                            let mut right = self.parse_value(current, None, NameMode::Name)?;
                            let Some(right_type) = self.keeper.type_of(right) else {
                                return self.error(paste, "RHS of paste is not typed!");
                            };
                            if right_type != string {
                                let made = self.keeper.unary(UnaryOperator::Cast, right, string);
                                let cast = self.keeper.fold(made, current, false)?;
                                if self.keeper.type_of(cast).is_none() {
                                    let text = format!(
                                        "can't cast '{}' to string",
                                        self.keeper.text_of(right)
                                    );
                                    return self.error(paste, text);
                                }
                                right = cast;
                            }
                            right
                        }
                    };
                    result = self.string_concat(left, right);
                }
                _ => return Ok(result),
            }
        }
    }

    fn parse_dag_arguments(
        &mut self,
        current: Option<RecordIndex>,
    ) -> Flow<Vec<(Value, Option<Value>)>> {
        let mut pairs = Vec::new();
        loop {
            if self.token() == Token::VariableName {
                let name = self.keeper.string(&self.lexer.text.clone());
                pairs.push((self.keeper.unset(), Some(name)));
                self.lex();
            } else {
                let value = self.parse_value(current, None, NameMode::Value)?;
                let mut name = None;
                if self.token() == Token::Colon {
                    if self.lex() != Token::VariableName {
                        return self.token_error("expected variable name in dag literal");
                    }
                    name = Some(self.keeper.string(&self.lexer.text.clone()));
                    self.lex();
                }
                pairs.push((value, name));
            }
            if !self.consume(Token::Comma) {
                break;
            }
        }
        Ok(pairs)
    }

    fn parse_value_list(
        &mut self,
        current: Option<RecordIndex>,
        item_type: Option<Type>,
    ) -> Flow<Vec<Value>> {
        let mut values = vec![self.parse_value(current, item_type, NameMode::Value)?];
        while self.consume(Token::Comma) {
            if self.token() == Token::RightSquare {
                return Ok(values);
            }
            values.push(self.parse_value(current, item_type, NameMode::Value)?);
        }
        Ok(values)
    }

    fn parse_template_argument_values(
        &mut self,
        current: Option<RecordIndex>,
        target: RecordIndex,
    ) -> Flow<(Vec<Value>, Vec<Location>)> {
        let names = self.keeper.record(target).template_arguments.clone();
        let mut arguments = Vec::new();
        let mut locations = Vec::new();
        if self.consume(Token::Greater) {
            return Ok((arguments, locations));
        }
        let mut named = false;
        let mut index = 0usize;
        loop {
            if index >= names.len() {
                return self.token_error(format!("Too many template arguments: {}", index + 1));
            }
            let mut value_location = self.location();
            locations.push(value_location);
            let expected = if named {
                None
            } else {
                let held = self.keeper.record(target);
                held.field(names[index])
                    .map(|at| held.fields[at].field_type)
            };
            let value = self.parse_value(current, expected, NameMode::Value)?;
            if self.token() == Token::Equal {
                if !matches!(self.keeper.node(value), Node::String { .. }) {
                    return self.error(
                        value_location,
                        "The name of named argument should be a valid identifier",
                    );
                }
                let qualified = self.qualify(target, value)?;
                let held = self.keeper.record(target);
                let Some(at) = held.field(qualified) else {
                    let text = format!("Argument {} doesn't exist", self.keeper.text_of(value));
                    return self.error(value_location, text);
                };
                let field_type = held.fields[at].field_type;
                self.lex();
                value_location = self.location();
                let named_value = self.parse_value(current, Some(field_type), NameMode::Value)?;
                if self.keeper.is_unset(named_value) {
                    let text = format!(
                        "The value of named argument should be initialized, but we got '{}'",
                        self.keeper.text_of(named_value)
                    );
                    return self.error(value_location, text);
                }
                arguments.push(self.keeper.argument(named_value, Some(qualified), 0));
                named = true;
            } else {
                if named {
                    return self.error(
                        value_location,
                        "Positional argument should be put before named argument",
                    );
                }
                arguments.push(self.keeper.argument(value, None, index as u32));
            }
            if self.consume(Token::Greater) {
                return Ok((arguments, locations));
            }
            if !self.consume(Token::Comma) {
                return self.token_error("Expected comma before next argument");
            }
            index += 1;
        }
    }

    fn check_template_argument_values(
        &mut self,
        arguments: &mut [Value],
        locations: &[Location],
        target: RecordIndex,
    ) -> Flow<()> {
        let names = self.keeper.record(target).template_arguments.clone();
        let mut failed = false;
        for (index, argument) in arguments.iter_mut().enumerate() {
            let (value, name, position) = self.keeper.argument_parts(*argument);
            let argument_name = name.unwrap_or_else(|| names[position as usize]);
            let held = self.keeper.record(target);
            let Some(at) = held.field(argument_name) else {
                continue;
            };
            let field_type = held.fields[at].field_type;
            let Some(value_type) = self.keeper.type_of(value) else {
                continue;
            };
            match self.keeper.cast_to(value, field_type)? {
                Some(cast) => *argument = self.keeper.argument(cast, name, position),
                None => {
                    let text = format!(
                        "Value specified for template argument '{}' is of type {}; expected type {}: {}",
                        self.keeper.unquoted(argument_name),
                        self.keeper.type_name(value_type),
                        self.keeper.type_name(field_type),
                        self.keeper.text_of(value)
                    );
                    let location = locations.get(index).copied().unwrap_or_default();
                    self.report_error(location, text);
                    failed = true;
                }
            }
        }
        if failed {
            return Err(Stop);
        }
        Ok(())
    }

    fn parse_declaration(&mut self, current: Option<RecordIndex>, template: bool) -> Flow<Value> {
        let has_field = self.consume(Token::Field);
        let declared_type = self.parse_type()?;
        if self.token() != Token::Identifier {
            return self.token_error("Expected identifier in declaration");
        }
        let text = self.lexer.text.clone();
        if text == "NAME" {
            return self.token_error(format!("'{text}' is a reserved variable name"));
        }
        if !template
            && self
                .scopes
                .last()
                .is_some_and(|scope| scope.variables.contains_key(&text))
        {
            return self.token_error("local variable of this name already exists");
        }
        let identifier_location = self.location();
        let mut declared = self.keeper.string(&text);
        self.lex();
        let unset = self.keeper.initial_value(declared_type);
        if !template {
            let kind = if has_field {
                FieldKind::NonconcreteAllowed
            } else {
                FieldKind::Normal
            };
            self.add_value(
                current,
                identifier_location,
                RecordField {
                    name: declared,
                    location: identifier_location,
                    field_type: declared_type,
                    kind,
                    value: unset,
                    used: false,
                },
            )?;
        } else {
            let holder = self.target_record(current);
            declared = self.qualify(holder, declared)?;
            self.add_value(
                current,
                identifier_location,
                RecordField {
                    name: declared,
                    location: identifier_location,
                    field_type: declared_type,
                    kind: FieldKind::TemplateArgument,
                    value: unset,
                    used: false,
                },
            )?;
        }
        if self.consume(Token::Equal) {
            let value_location = self.location();
            let outcome = match self.parse_value(current, Some(declared_type), NameMode::Value) {
                Ok(value) => self.set_value(
                    current,
                    value_location,
                    declared,
                    &[],
                    value,
                    false,
                    false,
                    LetMode::Replace,
                ),
                Err(stop) => Err(stop),
            };
            self.recover(outcome)?;
        }
        Ok(declared)
    }

    fn parse_foreach_declaration(&mut self) -> Flow<Option<(Value, Value)>> {
        if self.token() != Token::Identifier {
            return self.token_error("Expected identifier in foreach declaration");
        }
        let declared = self.keeper.string(&self.lexer.text.clone());
        self.lex();
        if !self.consume(Token::Equal) {
            return self.token_error("Expected '=' in foreach declaration");
        }
        let mut iterator_type = None;
        let mut list_value = None;
        let mut ranges = Vec::new();
        if self.token() == Token::LeftBrace {
            self.lex();
            let parsed = self.parse_range_list();
            ranges = self.soft(parsed)?.unwrap_or_default();
            if !self.consume(Token::RightBrace) {
                return self.token_error("expected '}' at end of bit range list");
            }
        } else {
            let value_location = self.location();
            let value = self.parse_value(None, None, NameMode::Value)?;
            match self.keeper.type_of(value) {
                Some(of) if self.keeper.list_element(of).is_some() => {
                    list_value = Some(value);
                    iterator_type = self.keeper.list_element(of);
                }
                Some(_) => self.parse_range_piece(&mut ranges, Some(value))?,
                None => {
                    let text = format!("expected a list, got '{}'", self.keeper.text_of(value));
                    self.report_error(value_location, text);
                    if self.current_multiclass.is_some() {
                        self.note(
                            &[],
                            "references to multiclass template arguments cannot be resolved at this time",
                        );
                    }
                    return Err(Stop);
                }
            }
        }
        if !ranges.is_empty() {
            let integer = self.keeper.integer_type();
            iterator_type = Some(integer);
            let values = ranges
                .iter()
                .map(|range| self.keeper.integer(*range as i64))
                .collect();
            list_value = Some(self.keeper.list(values, integer));
        }
        let (Some(of), Some(list)) = (iterator_type, list_value) else {
            return Ok(None);
        };
        Ok(Some((self.keeper.variable(declared, of), list)))
    }

    fn parse_template_argument_list(&mut self, current: Option<RecordIndex>) -> Flow<()> {
        self.lex();
        let holder = self.target_record(current);
        let argument = self.parse_declaration(current, true)?;
        self.keeper
            .record_mut(holder)
            .template_arguments
            .push(argument);
        while self.consume(Token::Comma) {
            let location = self.location();
            let argument = self.parse_declaration(current, true)?;
            if self.keeper.record(holder).is_template_argument(argument) {
                return self.error(
                    location,
                    "template argument with the same name has already been defined",
                );
            }
            self.keeper
                .record_mut(holder)
                .template_arguments
                .push(argument);
        }
        if !self.consume(Token::Greater) {
            return self.token_error("expected '>' at end of template argument list");
        }
        Ok(())
    }

    fn parse_let_mode_and_name(&mut self) -> (LetMode, Location, String) {
        let location = self.location();
        let text = self.lexer.text.clone();
        let mode = match text.as_str() {
            "append" => LetMode::Append,
            "prepend" => LetMode::Prepend,
            _ => LetMode::Replace,
        };
        self.lex();
        if mode != LetMode::Replace && self.token() == Token::Identifier {
            let found = (mode, self.location(), self.lexer.text.clone());
            self.lex();
            return found;
        }
        (LetMode::Replace, location, text)
    }

    fn parse_body_item(&mut self, record: RecordIndex) -> Flow<()> {
        match self.token() {
            Token::Assert => return self.parse_assert(Some(record)),
            Token::Defvar => return self.parse_defvar(Some(record)),
            Token::Dump => return self.parse_dump(Some(record)),
            Token::Let => {}
            _ => {
                self.parse_declaration(Some(record), false)?;
                if !self.consume(Token::Semicolon) {
                    return self.token_error("expected ';' after declaration");
                }
                return Ok(());
            }
        }
        self.lex();
        if self.token() != Token::Identifier {
            return self.token_error("expected field identifier after let");
        }
        let (mode, location, text) = self.parse_let_mode_and_name();
        let name = self.keeper.string(&text);
        let mut bits = self.parse_optional_range_list()?;
        bits.reverse();
        if !self.consume(Token::Equal) {
            return self.token_error("expected '=' in let expression");
        }
        let Some(at) = self.keeper.record(record).field(name) else {
            return self.error(location, format!("Value '{text}' unknown!"));
        };
        let mut expected = self.keeper.record(record).fields[at].field_type;
        if !bits.is_empty() && self.keeper.bits_width(expected).is_some() {
            expected = self.keeper.bits_type(bits.len() as u32);
        }
        let value = self.parse_value(Some(record), Some(expected), NameMode::Value)?;
        if !self.consume(Token::Semicolon) {
            return self.token_error("expected ';' after let expression");
        }
        self.set_value(
            Some(record),
            location,
            name,
            &bits,
            value,
            false,
            true,
            mode,
        )
    }

    fn parse_body(&mut self, record: RecordIndex) -> Flow<()> {
        if self.consume(Token::Semicolon) {
            return Ok(());
        }
        if !self.consume(Token::LeftBrace) {
            return self.token_error("Expected '{' to start body or ';' for declaration only");
        }
        while self.token() != Token::RightBrace {
            self.parse_body_item(record)?;
        }
        self.lex();
        let semicolon = self.location();
        if self.consume(Token::Semicolon) {
            self.report_error(
                semicolon,
                "A class or def body should not end with a semicolon",
            );
            self.note(&[], "Semicolon ignored; remove to eliminate this error");
        }
        Ok(())
    }

    fn apply_let_stack(&mut self, record: RecordIndex) -> Flow<()> {
        let stack = self.let_stack.clone();
        for level in &stack {
            for item in level {
                self.set_value(
                    Some(record),
                    item.location,
                    item.name,
                    &item.bits,
                    item.value,
                    false,
                    true,
                    item.mode,
                )?;
            }
        }
        Ok(())
    }

    fn apply_let_stack_to_entry(&mut self, entry: &Entry) -> Flow<()> {
        match entry {
            Entry::Record(record) => self.apply_let_stack(*record),
            Entry::Loop(each) => {
                for nested in &each.entries {
                    self.apply_let_stack_to_entry(nested)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn parse_object_body(&mut self, record: RecordIndex) -> Flow<()> {
        self.push_scope(ScopeKind::Record(record));
        if self.consume(Token::Colon) {
            loop {
                let reference = self.parse_subclass_reference(Some(record), false)?;
                self.add_subclass(record, &reference)?;
                if !self.consume(Token::Comma) {
                    break;
                }
            }
        }
        self.apply_let_stack(record)?;
        self.parse_body(record)?;
        self.pop_scope();
        Ok(())
    }

    fn parse_def(&mut self, multiclass: Option<usize>) -> Flow<()> {
        let def_location = self.location();
        self.lex();
        let name_location = if self.token() == Token::Identifier {
            self.location()
        } else {
            def_location
        };
        let name = self.parse_object_name(multiclass)?;
        let record = if self.keeper.is_unset(name) {
            let anonymous = self.keeper.new_anonymous_name();
            self.keeper.new_record(
                anonymous,
                vec![def_location],
                RecordKind::AnonymousDefinition,
            )?
        } else {
            self.keeper
                .new_record(name, vec![name_location], RecordKind::Definition)?
        };
        self.parse_object_body(record)?;
        self.add_entry(Entry::Record(record))
    }

    fn parse_object_name(&mut self, multiclass: Option<usize>) -> Flow<Value> {
        if matches!(
            self.token(),
            Token::Colon | Token::Semicolon | Token::LeftBrace
        ) {
            return Ok(self.keeper.unset());
        }
        let current = multiclass.map(|index| self.multiclasses[index].record);
        let string = self.keeper.string_type();
        let mut name = self.parse_value(current, Some(string), NameMode::Name)?;
        if let Some(record) = current {
            let implicit = self.implicit_name(record)?;
            let mut tracker = HasReferenceResolver {
                target: Some(implicit),
                found: false,
                finished: false,
            };
            self.keeper.resolve(name, &mut tracker)?;
            if !tracker.found {
                let variable = self.keeper.variable(implicit, string);
                name = self.string_concat(variable, name);
            }
        }
        Ok(name)
    }

    fn parse_defset(&mut self) -> Flow<()> {
        self.lex();
        let location = self.location();
        let declared = self.parse_type()?;
        let Some(element) = self.keeper.list_element(declared) else {
            return self.error(location, "expected list type");
        };
        if self.token() != Token::Identifier {
            return self.token_error("expected identifier");
        }
        let name = self.lexer.text.clone();
        if self.keeper.global(&name).is_some() {
            return self.token_error("def or global variable of this name already exists");
        }
        if self.lex() != Token::Equal {
            return self.token_error("expected '='");
        }
        if self.lex() != Token::LeftBrace {
            return self.token_error("expected '{'");
        }
        let brace = self.location();
        self.lex();
        self.defsets.push(DefsetRecord {
            location,
            element,
            elements: Vec::new(),
        });
        let outcome = self.parse_object_list(None);
        let defset = self.defsets.pop();
        outcome?;
        if !self.consume(Token::RightBrace) {
            let here = self.location();
            self.report_error(here, "expected '}' at end of defset");
            return self.error(brace, "to match this '{'");
        }
        let elements = defset.map(|defset| defset.elements).unwrap_or_default();
        let list = self.keeper.list(elements, element);
        self.keeper.globals.insert(name, list);
        Ok(())
    }

    fn parse_deftype(&mut self) -> Flow<()> {
        self.lex();
        if self.token() != Token::Identifier {
            return self.token_error("expected identifier");
        }
        let name = self.lexer.text.clone();
        if self.type_aliases.contains_key(&name) || self.keeper.find_class(&name).is_some() {
            return self.token_error(format!("type of this name '{name}' already exists"));
        }
        self.lex();
        if !self.consume(Token::Equal) {
            return self.token_error("expected '='");
        }
        let location = self.location();
        let declared = self.parse_type()?;
        if self.keeper.record_classes(declared).is_some() {
            let text = format!(
                "cannot define type alias for class type '{}'",
                self.keeper.type_name(declared)
            );
            return self.error(location, text);
        }
        self.type_aliases.insert(name, declared);
        if !self.consume(Token::Semicolon) {
            return self.token_error("expected ';'");
        }
        Ok(())
    }

    fn parse_defvar(&mut self, record: Option<RecordIndex>) -> Flow<()> {
        self.lex();
        if self.token() != Token::Identifier {
            return self.token_error("expected identifier");
        }
        let name = self.lexer.text.clone();
        if self
            .scopes
            .last()
            .is_some_and(|scope| scope.variables.contains_key(&name))
        {
            return self.token_error("local variable of this name already exists");
        }
        if let Some(record) = record {
            let named = self.keeper.string(&name);
            let held = self.keeper.record(record);
            if let Some(at) = held.field(named) {
                if held.fields[at].kind != FieldKind::TemplateArgument {
                    return self.token_error("field of this name already exists");
                }
            }
        }
        let outermost = self.scopes.len() == 1;
        if outermost && self.keeper.global(&name).is_some() {
            return self.token_error("def or global variable of this name already exists");
        }
        self.lex();
        if !self.consume(Token::Equal) {
            return self.token_error("expected '='");
        }
        let value = self.parse_value(record, None, NameMode::Value)?;
        if !self.consume(Token::Semicolon) {
            return self.token_error("expected ';'");
        }
        if !outermost {
            if let Some(scope) = self.scopes.last_mut() {
                scope.variables.insert(name, value);
            }
        } else {
            self.keeper.globals.insert(name, value);
        }
        Ok(())
    }

    fn parse_foreach(&mut self, multiclass: Option<usize>) -> Flow<()> {
        let location = self.location();
        self.lex();
        let declared = self.parse_foreach_declaration();
        let Some((iterator, list)) = self.soft(declared)?.flatten() else {
            return self.token_error("expected declaration in for");
        };
        if !self.consume(Token::In) {
            return self.token_error("Unknown tok");
        }
        self.push_scope(ScopeKind::Loop(Some(iterator)));
        self.loops.push(ForeachLoop {
            location,
            iterator: Some(iterator),
            list,
            entries: Vec::new(),
        });
        if self.token() != Token::LeftBrace {
            self.parse_object(multiclass)?;
        } else {
            let brace = self.location();
            self.lex();
            self.parse_object_list(multiclass)?;
            if !self.consume(Token::RightBrace) {
                let here = self.location();
                self.report_error(here, "expected '}' at end of foreach command");
                return self.error(brace, "to match this '{'");
            }
        }
        self.pop_scope();
        let Some(finished) = self.loops.pop() else {
            return Err(Stop);
        };
        self.add_entry(Entry::Loop(Box::new(finished)))
    }

    fn parse_if(&mut self, multiclass: Option<usize>) -> Flow<()> {
        let location = self.location();
        self.lex();
        let condition = self.parse_value(None, None, NameMode::Value)?;
        if !self.consume(Token::Then) {
            return self.token_error("Unknown tok");
        }
        let bit = self.keeper.bit_type();
        let empty = self.keeper.list(Vec::new(), bit);
        let truth = self.keeper.bit(true);
        let singleton = self.keeper.list(vec![truth], bit);
        let bit_list = self.keeper.list_type(bit);
        let made = self
            .keeper
            .ternary(TernaryOperator::If, condition, singleton, empty, bit_list);
        let then_list = self.keeper.fold(made, None, false)?;
        self.loops.push(ForeachLoop {
            location,
            iterator: None,
            list: then_list,
            entries: Vec::new(),
        });
        self.parse_if_body(multiclass, "then")?;
        let Some(then_loop) = self.loops.pop() else {
            return Err(Stop);
        };
        self.add_entry(Entry::Loop(Box::new(then_loop)))?;
        if self.consume(Token::ElseKeyword) {
            let made =
                self.keeper
                    .ternary(TernaryOperator::If, condition, empty, singleton, bit_list);
            let else_list = self.keeper.fold(made, None, false)?;
            self.loops.push(ForeachLoop {
                location,
                iterator: None,
                list: else_list,
                entries: Vec::new(),
            });
            self.parse_if_body(multiclass, "else")?;
            let Some(else_loop) = self.loops.pop() else {
                return Err(Stop);
            };
            self.add_entry(Entry::Loop(Box::new(else_loop)))?;
        }
        Ok(())
    }

    fn parse_if_body(&mut self, multiclass: Option<usize>, kind: &str) -> Flow<()> {
        self.push_scope(ScopeKind::Local);
        if self.token() != Token::LeftBrace {
            self.parse_object(multiclass)?;
        } else {
            let brace = self.location();
            self.lex();
            self.parse_object_list(multiclass)?;
            if !self.consume(Token::RightBrace) {
                let here = self.location();
                self.report_error(here, format!("expected '}}' at end of '{kind}' clause"));
                return self.error(brace, "to match this '{'");
            }
        }
        self.pop_scope();
        Ok(())
    }

    fn parse_assert(&mut self, record: Option<RecordIndex>) -> Flow<()> {
        self.lex();
        let location = self.location();
        let condition = self.parse_value(record, None, NameMode::Value)?;
        if !self.consume(Token::Comma) {
            return self.token_error("expected ',' in assert statement");
        }
        let message = self.parse_value(record, None, NameMode::Value)?;
        if !self.consume(Token::Semicolon) {
            return self.token_error("expected ';'");
        }
        let assertion = Assertion {
            location,
            condition,
            message,
        };
        match record {
            Some(record) => self.keeper.record_mut(record).assertions.push(assertion),
            None => {
                let outcome = self.add_entry(Entry::Assertion(assertion));
                self.recover(outcome)?;
            }
        }
        Ok(())
    }

    fn parse_dump(&mut self, record: Option<RecordIndex>) -> Flow<()> {
        let location = self.location();
        self.lex();
        let mut message = self.parse_value(record, None, NameMode::Value)?;
        if matches!(self.keeper.node(message), Node::Definition(_)) {
            let string = self.keeper.string_type();
            let made = self.keeper.unary(UnaryOperator::Represent, message, string);
            message = self.keeper.fold(made, record, false)?;
        }
        if !self.consume(Token::Semicolon) {
            return self.token_error("expected ';'");
        }
        match record {
            Some(record) => self
                .keeper
                .record_mut(record)
                .dumps
                .push(DumpRequest { location, message }),
            None => {
                let mut resolver = HasReferenceResolver {
                    target: None,
                    found: false,
                    finished: true,
                };
                let resolved = self.keeper.resolve(message, &mut resolver)?;
                let outcome = self.add_entry(Entry::Dump(DumpRequest {
                    location,
                    message: resolved,
                }));
                self.recover(outcome)?;
            }
        }
        Ok(())
    }

    fn parse_class(&mut self) -> Flow<()> {
        self.lex();
        if self.token() != Token::Identifier {
            return self.token_error("expected class name after 'class' keyword");
        }
        let name = self.lexer.text.clone();
        let location = self.location();
        let record = match self.keeper.find_class(&name) {
            Some(existing) => {
                let held = self.keeper.record(existing);
                if !held.fields.is_empty()
                    || !held.superclasses.is_empty()
                    || !held.template_arguments.is_empty()
                {
                    let text = format!(
                        "Class '{}' already defined",
                        self.keeper.record_name_text(existing)
                    );
                    return self.token_error(text);
                }
                self.keeper.record_mut(existing).locations = vec![location];
                existing
            }
            None => {
                let named = self.keeper.string(&name);
                let made = self
                    .keeper
                    .new_record(named, vec![location], RecordKind::Class)?;
                self.keeper.classes.insert(name.clone(), made);
                made
            }
        };
        if self.type_aliases.contains_key(&name) {
            return self.token_error(format!("there is already a defined type alias '{name}'"));
        }
        self.lex();
        self.push_scope(ScopeKind::Record(record));
        if self.token() == Token::Less {
            self.parse_template_argument_list(Some(record))?;
        }
        self.parse_object_body(record)?;
        self.check_unused_template_arguments(record);
        self.pop_scope();
        Ok(())
    }

    fn parse_let_list(&mut self) -> Flow<Vec<LetRecord>> {
        let mut found = Vec::new();
        loop {
            if self.token() != Token::Identifier {
                return self.token_error("expected identifier in let definition");
            }
            let (mode, location, text) = self.parse_let_mode_and_name();
            let name = self.keeper.string(&text);
            let mut bits = self.parse_optional_range_list()?;
            bits.reverse();
            if !self.consume(Token::Equal) {
                return self.token_error("expected '=' in let expression");
            }
            let value = self.parse_value(None, None, NameMode::Value)?;
            found.push(LetRecord {
                name,
                bits,
                value,
                location,
                mode,
            });
            if !self.consume(Token::Comma) {
                break;
            }
        }
        Ok(found)
    }

    fn parse_top_level_let(&mut self, multiclass: Option<usize>) -> Flow<()> {
        self.lex();
        let lets = self.parse_let_list()?;
        self.let_stack.push(lets);
        if !self.consume(Token::In) {
            return self.token_error("expected 'in' at end of top-level 'let'");
        }
        if self.token() != Token::LeftBrace {
            self.parse_object(multiclass)?;
        } else {
            let brace = self.location();
            self.lex();
            self.push_scope(ScopeKind::Local);
            self.parse_object_list(multiclass)?;
            if !self.consume(Token::RightBrace) {
                let here = self.location();
                self.report_error(here, "expected '}' at end of top level let command");
                return self.error(brace, "to match this '{'");
            }
            self.pop_scope();
        }
        self.let_stack.pop();
        Ok(())
    }

    fn parse_multiclass(&mut self) -> Flow<()> {
        self.lex();
        if self.token() != Token::Identifier {
            return self.token_error("expected identifier after multiclass for name");
        }
        let name = self.lexer.text.clone();
        if self.multiclass_names.contains_key(&name) {
            return self.token_error(format!("multiclass '{name}' already defined"));
        }
        let named = self.keeper.string(&name);
        let location = self.location();
        let record = self
            .keeper
            .new_record(named, vec![location], RecordKind::MultiClass)?;
        let index = self.multiclasses.len();
        self.multiclasses.push(MultiClass {
            record,
            entries: Vec::new(),
        });
        self.multiclass_names.insert(name, index);
        self.current_multiclass = Some(index);
        self.lex();
        self.push_scope(ScopeKind::MultiClass(index));
        if self.token() == Token::Less {
            self.parse_template_argument_list(None)?;
        }
        let mut inherits = false;
        if self.consume(Token::Colon) {
            inherits = true;
            loop {
                let reference = self.parse_submulticlass_reference(index)?;
                self.add_submulticlass(index, &reference)?;
                if !self.consume(Token::Comma) {
                    break;
                }
            }
        }
        if self.token() != Token::LeftBrace {
            if !inherits {
                return self.token_error("expected '{' in multiclass definition");
            }
            if !self.consume(Token::Semicolon) {
                return self.token_error("expected ';' in multiclass definition");
            }
        } else {
            if self.lex() == Token::RightBrace {
                return self.token_error("multiclass must contain at least one def");
            }
            while self.token() != Token::RightBrace {
                match self.token() {
                    Token::Assert
                    | Token::Def
                    | Token::Defm
                    | Token::Defvar
                    | Token::Dump
                    | Token::Foreach
                    | Token::If
                    | Token::Let => self.parse_object(Some(index))?,
                    _ => {
                        return self.token_error(
                            "expected 'assert', 'def', 'defm', 'defvar', 'dump', 'foreach', 'if', or 'let' in multiclass body",
                        )
                    }
                }
            }
            self.lex();
            let semicolon = self.location();
            if self.consume(Token::Semicolon) {
                self.report_error(
                    semicolon,
                    "A multiclass body should not end with a semicolon",
                );
                self.note(&[], "Semicolon ignored; remove to eliminate this error");
            }
        }
        self.check_unused_template_arguments(record);
        self.pop_scope();
        self.current_multiclass = None;
        Ok(())
    }

    fn parse_defm(&mut self, multiclass: Option<usize>) -> Flow<()> {
        self.lex();
        let mut defm_name = self.parse_object_name(multiclass)?;
        if self.keeper.is_unset(defm_name) {
            defm_name = self.keeper.new_anonymous_name();
            if let Some(index) = multiclass {
                let record = self.multiclasses[index].record;
                let implicit = self.implicit_name(record)?;
                let string = self.keeper.string_type();
                let variable = self.keeper.variable(implicit, string);
                defm_name = self.string_concat(variable, defm_name);
            }
        }
        if self.token() != Token::Colon {
            return self.token_error("expected ':' after defm identifier");
        }
        let mut made: Vec<Entry> = Vec::new();
        let mut inherit_from_class = false;
        self.lex();
        let mut subclass_location = self.location();
        let mut reference = self.parse_subclass_reference(None, true)?;
        loop {
            let class_name = self.keeper.record_name_text(reference.record);
            let Some(index) = self.multiclass_names.get(&class_name).copied() else {
                return Err(Stop);
            };
            let mut substitutions = self.resolve_arguments_of_multiclass(
                index,
                &reference.arguments,
                defm_name,
                subclass_location,
            )?;
            let source = self.multiclasses[index].entries.clone();
            let finished = multiclass.is_none() && self.loops.is_empty();
            self.resolve_entries(
                &source,
                &mut substitutions,
                finished,
                Some(&mut made),
                Some(subclass_location),
            )?;
            if !self.consume(Token::Comma) {
                break;
            }
            if self.token() != Token::Identifier {
                return self.token_error("expected identifier");
            }
            subclass_location = self.location();
            inherit_from_class = self.keeper.find_class(&self.lexer.text).is_some();
            if inherit_from_class {
                break;
            }
            reference = self.parse_subclass_reference(None, true)?;
        }
        if inherit_from_class {
            loop {
                let class = self.parse_subclass_reference(None, false)?;
                for entry in &made {
                    self.add_subclass_to_entry(entry, &class)?;
                }
                if !self.consume(Token::Comma) {
                    break;
                }
            }
        }
        for entry in made {
            self.apply_let_stack_to_entry(&entry)?;
            let outcome = self.add_entry(entry);
            self.recover(outcome)?;
        }
        if !self.consume(Token::Semicolon) {
            return self.token_error("expected ';' at end of defm");
        }
        Ok(())
    }

    fn parse_object(&mut self, multiclass: Option<usize>) -> Flow<()> {
        match self.token() {
            Token::Assert => self.parse_assert(None),
            Token::Def => self.parse_def(multiclass),
            Token::Defm => self.parse_defm(multiclass),
            Token::Deftype => self.parse_deftype(),
            Token::Defvar => self.parse_defvar(None),
            Token::Dump => self.parse_dump(None),
            Token::Foreach => self.parse_foreach(multiclass),
            Token::If => self.parse_if(multiclass),
            Token::Let => self.parse_top_level_let(multiclass),
            Token::Defset => {
                if multiclass.is_some() {
                    return self.token_error("defset is not allowed inside multiclass");
                }
                self.parse_defset()
            }
            Token::Class => {
                if multiclass.is_some() {
                    return self.token_error("class is not allowed inside multiclass");
                }
                if !self.loops.is_empty() {
                    return self.token_error("class is not allowed inside foreach loop");
                }
                self.parse_class()
            }
            Token::MultiClass => {
                if !self.loops.is_empty() {
                    return self.token_error("multiclass is not allowed inside foreach loop");
                }
                self.parse_multiclass()
            }
            _ => self.token_error(
                "Expected assert, class, def, defm, defset, dump, foreach, if, or let",
            ),
        }
    }

    fn parse_object_list(&mut self, multiclass: Option<usize>) -> Flow<()> {
        while self.token().starts_object() {
            self.parse_object(multiclass)?;
        }
        Ok(())
    }

    fn parse_file(&mut self) -> Flow<()> {
        self.lex();
        self.push_scope(ScopeKind::Local);
        self.parse_object_list(None)?;
        self.pop_scope();
        match self.token() {
            Token::End => Ok(()),
            Token::Error => Err(Stop),
            _ => self.token_error("Unexpected token at top level"),
        }
    }
}

pub struct Reading<'a> {
    pub sources: SourceManager<'a>,
    pub keeper: RecordKeeper,
    pub dependencies: BTreeSet<String>,
    pub parsed: bool,
    pub succeeded: bool,
}

impl Reading<'_> {
    pub fn render_messages(&self) -> String {
        let mut out = String::new();
        for message in &self.keeper.messages {
            out.push_str(&self.sources.render(message));
        }
        out
    }

    fn short_location(&self, location: Location) -> String {
        let (line, _) = self.sources.line_and_column(location);
        let name = self
            .sources
            .buffers
            .get(location.buffer as usize)
            .map(|buffer| buffer.name.rsplit('/').next().unwrap_or("").to_string())
            .unwrap_or_default();
        format!("{name}:{line}")
    }

    fn detailed_record(&self, index: RecordIndex, class: bool, out: &mut String) {
        let keeper = &self.keeper;
        let record = keeper.record(index);
        let name = keeper.record_name_text(index);
        let shown = if name.is_empty() {
            "\"\"".to_string()
        } else {
            name
        };
        let first = record.locations.first().copied().unwrap_or_default();
        out.push_str(&format!("\n{shown}  |{}|\n", self.short_location(first)));
        if class {
            if record.template_arguments.is_empty() {
                out.push_str("  Template args: (none)\n");
            } else {
                out.push_str("  Template args:\n");
                for argument in &record.template_arguments {
                    if let Some(at) = record.field(*argument) {
                        out.push_str("    ");
                        keeper.write_field(&record.fields[at], out);
                        out.push_str(&format!(
                            "  |{}|\n",
                            self.short_location(record.fields[at].location)
                        ));
                    }
                }
            }
        } else if record.locations.len() >= 2 {
            out.push_str("  Defm sequence:");
            for location in record.locations.iter().rev() {
                out.push_str(&format!(" |{}|", self.short_location(*location)));
            }
            out.push('\n');
        }
        let superclasses = keeper.superclass_list(index);
        if superclasses.is_empty() {
            out.push_str("  Superclasses: (none)\n");
        } else {
            out.push_str("  Superclasses:");
            for class in superclasses {
                let class_name = keeper.record_name_text(class);
                if record
                    .superclasses
                    .iter()
                    .any(|(direct, _)| *direct == class)
                {
                    out.push_str(&format!(" {class_name}"));
                } else {
                    out.push_str(&format!(" ({class_name})"));
                }
            }
            out.push('\n');
        }
        if record.fields.is_empty() {
            out.push_str("  Fields: (none)\n");
            return;
        }
        out.push_str("  Fields:\n");
        for field in &record.fields {
            if record.is_template_argument(field.name) {
                continue;
            }
            out.push_str("    ");
            keeper.write_field(field, out);
            out.push_str(&format!("  |{}|\n", self.short_location(field.location)));
        }
    }

    pub fn print_detailed_records(&self, input: &str) -> String {
        let keeper = &self.keeper;
        let mut out = format!("DETAILED RECORDS for file {input}\n");
        let rule = "--------------------";
        out.push_str(&format!(
            "\n{rule} Global Variables ({}) {rule}\n\n",
            keeper.globals.len()
        ));
        for (name, value) in &keeper.globals {
            out.push_str(&format!("{name} = {}\n", keeper.text_of(*value)));
        }
        out.push_str(&format!(
            "\n{rule} Classes ({}) {rule}\n",
            keeper.classes.len()
        ));
        for class in keeper.classes.values() {
            self.detailed_record(*class, true, &mut out);
        }
        out.push_str(&format!(
            "\n{rule} Records ({}) {rule}\n",
            keeper.definitions.len()
        ));
        for definition in keeper.definitions.values() {
            self.detailed_record(*definition, false, &mut out);
        }
        out.push_str(&format!("\n{rule} Memory Allocation Stats {rule}\n"));
        out.push_str(&format!(
            "Number of records instantiated = {}\nNumber of anonymous records = {}\n",
            keeper.record_counter, keeper.anonymous_counter
        ));
        out
    }

    pub fn diagnostic(&self) -> Diagnostic {
        let mut diagnostic = fail("EI101", "The TableGen source could not be read.");
        if let Some(message) = self
            .keeper
            .messages
            .iter()
            .find(|message| message.kind == MessageKind::Error)
        {
            if let Some(location) = message.location {
                let (line, column) = self.sources.line_and_column(location);
                let file = self
                    .sources
                    .buffers
                    .get(location.buffer as usize)
                    .map(|buffer| buffer.name.clone())
                    .unwrap_or_default();
                diagnostic = diagnostic
                    .with_context(format!("File: {file}"))
                    .with_context(format!("Line: {line}"))
                    .with_context(format!("Column: {column}"));
            }
            diagnostic = diagnostic.with_context(format!("Reason: {}", message.text));
        }
        diagnostic.with_context(format!("Errors: {}", self.keeper.errors))
    }
}

pub fn read_records<'a>(
    files: &'a BTreeMap<String, String>,
    main: &str,
    include_directories: &[String],
    macros: &[String],
    warn_unused_template_arguments: bool,
) -> Reading<'a> {
    let Some(lexer) = Lexer::new(files, main, include_directories, macros) else {
        let mut keeper = RecordKeeper::new();
        keeper.report(
            MessageKind::Error,
            &[],
            format!("Could not open input file '{main}'"),
        );
        return Reading {
            sources: SourceManager {
                buffers: Vec::new(),
            },
            keeper,
            dependencies: BTreeSet::new(),
            parsed: false,
            succeeded: false,
        };
    };
    let mut parser = Parser {
        lexer,
        keeper: RecordKeeper::new(),
        let_stack: Vec::new(),
        multiclass_names: BTreeMap::new(),
        multiclasses: Vec::new(),
        type_aliases: BTreeMap::new(),
        loops: Vec::new(),
        defsets: Vec::new(),
        current_multiclass: None,
        scopes: Vec::new(),
        warn_unused_template_arguments,
    };
    let mut valid_macros = true;
    for name in macros {
        if macro_name_length(name.as_bytes()) != name.len() {
            parser.keeper.report(
                MessageKind::Error,
                &[],
                format!("invalid macro name `{name}` specified on command line"),
            );
            valid_macros = false;
            break;
        }
    }
    let outcome = if valid_macros {
        parser.parse_file()
    } else {
        Err(Stop)
    };
    parser.absorb();
    let parsed = outcome.is_ok();
    let succeeded = parsed && parser.keeper.errors == 0;
    let Parser { lexer, keeper, .. } = parser;
    Reading {
        sources: lexer.sources,
        keeper,
        dependencies: lexer.dependencies,
        parsed,
        succeeded,
    }
}
