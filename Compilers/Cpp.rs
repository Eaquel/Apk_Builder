use crate::caps::Capability;
use crate::diag::Diagnostic;
use crate::plugin::{unimplemented_diagnostic, Context, Contract, Outcome, Plugin, Version};

pub static CONTRACT: Contract = Contract {
    id: "plugin.cpp",
    display_name: "C/C++",
    version: Version::new(0, 1, 0),
    produces_artifacts: false,
    summary: "Compiles C and C++ translation units into Android native libraries.",
    inputs: &["cpp.source", "cpp.header", "native.build.config"],
    outputs: &["native.object", "native.library"],
    required_capabilities: &[
        Capability::FsRead,
        Capability::TempStorage,
        Capability::Cache,
        Capability::Native,
    ],
    forbidden_capabilities: &[
        Capability::Network,
        Capability::Internet,
        Capability::KeyAccess,
        Capability::SensitiveOutput,
    ],
    non_responsibilities: &[
        "Packaging libraries into an APK; that is the Apk plugin's work.",
        "Providing a C or C++ standard library implementation.",
        "Deciding which ABIs a project targets; that comes from the project model.",
    ],
};

pub struct CppPlugin;

pub static PLUGIN: CppPlugin = CppPlugin;

impl Plugin for CppPlugin {
    fn contract(&self) -> &'static Contract {
        &CONTRACT
    }

    fn execute(&self, _ctx: &mut Context<'_>) -> Result<Outcome, Diagnostic> {
        Err(unimplemented_diagnostic(&CONTRACT))
    }
}

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum CStandard {
    C89,
    C94,
    C99,
    C11,
    C17,
    C23,
    C2y,
}

impl CStandard {
    pub const fn version_macro(self) -> Option<&'static str> {
        match self {
            CStandard::C89 => None,
            CStandard::C94 => Some("199409L"),
            CStandard::C99 => Some("199901L"),
            CStandard::C11 => Some("201112L"),
            CStandard::C17 => Some("201710L"),
            CStandard::C23 => Some("202311L"),
            CStandard::C2y => Some("202400L"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LanguageOptions {
    pub standard: CStandard,
    pub gnu: bool,
    pub trigraphs: bool,
    pub digraphs: bool,
    pub line_comments: bool,
    pub dollar_identifiers: bool,
    pub microsoft_extensions: bool,
    pub assembly: bool,
    pub traditional: bool,
    pub operator_names: bool,
    pub char_is_signed: bool,
    pub preprocessed_output: bool,
    pub header_file: bool,
}

impl LanguageOptions {
    pub fn for_standard(name: &str) -> Option<LanguageOptions> {
        let (standard, gnu) = match name {
            "c89" | "c90" | "iso9899:1990" => (CStandard::C89, false),
            "iso9899:199409" => (CStandard::C94, false),
            "gnu89" | "gnu90" => (CStandard::C89, true),
            "c99" | "iso9899:1999" | "c9x" | "iso9899:199x" => (CStandard::C99, false),
            "gnu99" | "gnu9x" => (CStandard::C99, true),
            "c11" | "iso9899:2011" | "c1x" | "iso9899:201x" => (CStandard::C11, false),
            "gnu11" | "gnu1x" => (CStandard::C11, true),
            "c17" | "iso9899:2017" | "c18" | "iso9899:2018" => (CStandard::C17, false),
            "gnu17" | "gnu18" => (CStandard::C17, true),
            "c23" | "iso9899:2024" | "c2x" => (CStandard::C23, false),
            "gnu23" | "gnu2x" => (CStandard::C23, true),
            "c2y" => (CStandard::C2y, false),
            "gnu2y" => (CStandard::C2y, true),
            _ => return None,
        };
        Some(LanguageOptions {
            standard,
            gnu,
            trigraphs: !gnu && standard < CStandard::C23,
            digraphs: standard >= CStandard::C94 || gnu,
            line_comments: standard >= CStandard::C99 || gnu,
            dollar_identifiers: true,
            microsoft_extensions: false,
            assembly: false,
            traditional: false,
            operator_names: false,
            char_is_signed: true,
            preprocessed_output: false,
            header_file: false,
        })
    }

    pub fn at_least(&self, standard: CStandard) -> bool {
        self.standard >= standard
    }
}

impl Default for LanguageOptions {
    fn default() -> LanguageOptions {
        LanguageOptions::for_standard("gnu17").unwrap_or(LanguageOptions {
            standard: CStandard::C17,
            gnu: true,
            trigraphs: false,
            digraphs: true,
            line_comments: true,
            dollar_identifiers: true,
            microsoft_extensions: false,
            assembly: false,
            traditional: false,
            operator_names: false,
            char_is_signed: true,
            preprocessed_output: false,
            header_file: false,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Place {
    pub buffer: u32,
    pub offset: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileCharacter {
    User,
    System,
    ExternCSystem,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferKind {
    File,
    BuiltIn,
    CommandLine,
    Scratch,
    Pragma,
}

#[derive(Clone, Debug)]
struct LineEntry {
    offset: u32,
    line: u32,
    name: Option<Rc<str>>,
    character: FileCharacter,
    include: Option<Place>,
}

#[derive(Clone, Debug)]
pub struct SourceBuffer {
    pub name: Rc<str>,
    pub bytes: Rc<[u8]>,
    pub kind: BufferKind,
    pub included_at: Option<Place>,
    pub character: FileCharacter,
    line_starts: Vec<u32>,
    entries: Vec<LineEntry>,
}

impl SourceBuffer {
    fn new(name: &str, bytes: Rc<[u8]>, kind: BufferKind) -> SourceBuffer {
        let mut line_starts = vec![0u32];
        let mut at = 0usize;
        while at < bytes.len() {
            match bytes[at] {
                b'\n' => line_starts.push((at + 1) as u32),
                b'\r' => {
                    if bytes.get(at + 1) == Some(&b'\n') {
                        at += 1;
                    }
                    line_starts.push((at + 1) as u32);
                }
                _ => {}
            }
            at += 1;
        }
        SourceBuffer {
            name: Rc::from(name),
            bytes,
            kind,
            included_at: None,
            character: FileCharacter::User,
            line_starts,
            entries: Vec::new(),
        }
    }

    pub fn physical_line(&self, offset: u32) -> u32 {
        match self.line_starts.binary_search(&offset) {
            Ok(found) => found as u32 + 1,
            Err(after) => after as u32,
        }
    }

    pub fn physical_column(&self, offset: u32) -> u32 {
        let line = self.physical_line(offset);
        offset - self.line_starts[(line - 1) as usize] + 1
    }

    pub fn line_start(&self, line: u32) -> Option<u32> {
        if line == 0 {
            return None;
        }
        self.line_starts.get((line - 1) as usize).copied()
    }

    pub fn line_text(&self, line: u32) -> &[u8] {
        let Some(start) = self.line_start(line) else {
            return &[];
        };
        let start = start as usize;
        let mut end = start;
        while end < self.bytes.len() && self.bytes[end] != b'\n' && self.bytes[end] != b'\r' {
            end += 1;
        }
        &self.bytes[start..end]
    }

    fn entry_at(&self, offset: u32) -> Option<&LineEntry> {
        let index = self.entries.partition_point(|entry| entry.offset <= offset);
        if index == 0 {
            None
        } else {
            Some(&self.entries[index - 1])
        }
    }
}

#[derive(Clone, Debug)]
pub struct PresumedPlace {
    pub name: Rc<str>,
    pub line: u32,
    pub column: u32,
    pub buffer: u32,
    pub character: FileCharacter,
    pub include: Option<Place>,
}

#[derive(Clone, Debug, Default)]
pub struct SourceMap {
    pub buffers: Vec<SourceBuffer>,
}

impl SourceMap {
    fn add(&mut self, buffer: SourceBuffer) -> u32 {
        self.buffers.push(buffer);
        (self.buffers.len() - 1) as u32
    }

    pub fn buffer(&self, index: u32) -> &SourceBuffer {
        &self.buffers[index as usize]
    }

    pub fn presumed(&self, place: Place) -> PresumedPlace {
        let buffer = self.buffer(place.buffer);
        let physical = buffer.physical_line(place.offset);
        let column = buffer.physical_column(place.offset);
        match buffer.entry_at(place.offset) {
            Some(entry) => {
                let base = buffer.physical_line(entry.offset);
                PresumedPlace {
                    name: entry.name.clone().unwrap_or_else(|| buffer.name.clone()),
                    line: entry.line.wrapping_add(physical.wrapping_sub(base)),
                    column,
                    buffer: place.buffer,
                    character: entry.character,
                    include: entry.include.or(buffer.included_at),
                }
            }
            None => PresumedPlace {
                name: buffer.name.clone(),
                line: physical,
                column,
                buffer: place.buffer,
                character: buffer.character,
                include: buffer.included_at,
            },
        }
    }

    pub fn presumed_line_of(&self, buffer: u32, physical_line: u32) -> u32 {
        match self.buffer(buffer).line_start(physical_line) {
            Some(offset) => self.presumed(Place { buffer, offset }).line,
            None => physical_line,
        }
    }

    fn add_line_entry(
        &mut self,
        at: Place,
        line: u32,
        name: Option<Rc<str>>,
        entry_exit: u8,
        character: FileCharacter,
        marker: Place,
    ) {
        let offset = at.offset;
        let target = &mut self.buffers[at.buffer as usize];
        let mut include = None;
        let mut name = name;
        if entry_exit == 1 {
            include = Some(marker);
        } else {
            let mut previous = target.entries.last().cloned();
            if entry_exit == 2 {
                if let Some(Some(popped)) = previous.as_ref().map(|entry| entry.include) {
                    previous = target.entry_at(popped.offset).cloned();
                    if previous.is_none() {
                        previous = Some(LineEntry {
                            offset: 0,
                            line: 1,
                            name: None,
                            character: target.character,
                            include: None,
                        });
                    }
                }
            }
            if let Some(previous) = previous {
                include = previous.include;
                if name.is_none() {
                    name = previous.name.clone();
                }
            }
        }
        let entry = LineEntry {
            offset,
            line,
            name,
            character,
            include,
        };
        target.entries.retain(|held| held.offset < offset);
        target.entries.push(entry);
    }

    pub fn include_chain(&self, place: Place) -> Vec<PresumedPlace> {
        let mut chain = Vec::new();
        let mut current = self.presumed(place).include;
        let mut guard = 0;
        while let Some(at) = current {
            let presumed = self.presumed(at);
            current = presumed.include;
            chain.push(presumed);
            guard += 1;
            if guard > 400 {
                break;
            }
        }
        chain
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum LiteralPrefix {
    Plain,
    Wide,
    Utf8,
    Utf16,
    Utf32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum TokenKind {
    EndOfFile,
    EndOfDirective,
    Identifier,
    Number,
    Character(LiteralPrefix),
    String(LiteralPrefix),
    HeaderName,
    Comment,
    Unknown,
    LeftSquare,
    RightSquare,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Period,
    Ellipsis,
    Amp,
    AmpAmp,
    AmpEqual,
    Star,
    StarEqual,
    Plus,
    PlusPlus,
    PlusEqual,
    Minus,
    Arrow,
    MinusMinus,
    MinusEqual,
    Tilde,
    Exclaim,
    ExclaimEqual,
    Slash,
    SlashEqual,
    Percent,
    PercentEqual,
    Less,
    LessLess,
    LessEqual,
    LessLessEqual,
    Greater,
    GreaterGreater,
    GreaterEqual,
    GreaterGreaterEqual,
    Caret,
    CaretEqual,
    Pipe,
    PipePipe,
    PipeEqual,
    Question,
    Colon,
    ColonColon,
    Semi,
    Equal,
    EqualEqual,
    Comma,
    Hash,
    HashHash,
    HashAt,
    At,
    Embed,
}

impl TokenKind {
    pub fn is_literal(self) -> bool {
        matches!(
            self,
            TokenKind::Number
                | TokenKind::Character(_)
                | TokenKind::String(_)
                | TokenKind::HeaderName
        )
    }

    pub fn is_string_or_character(self) -> bool {
        matches!(self, TokenKind::Character(_) | TokenKind::String(_))
    }

    pub fn punctuator_spelling(self) -> Option<&'static str> {
        Some(match self {
            TokenKind::LeftSquare => "[",
            TokenKind::RightSquare => "]",
            TokenKind::LeftParen => "(",
            TokenKind::RightParen => ")",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::Period => ".",
            TokenKind::Ellipsis => "...",
            TokenKind::Amp => "&",
            TokenKind::AmpAmp => "&&",
            TokenKind::AmpEqual => "&=",
            TokenKind::Star => "*",
            TokenKind::StarEqual => "*=",
            TokenKind::Plus => "+",
            TokenKind::PlusPlus => "++",
            TokenKind::PlusEqual => "+=",
            TokenKind::Minus => "-",
            TokenKind::Arrow => "->",
            TokenKind::MinusMinus => "--",
            TokenKind::MinusEqual => "-=",
            TokenKind::Tilde => "~",
            TokenKind::Exclaim => "!",
            TokenKind::ExclaimEqual => "!=",
            TokenKind::Slash => "/",
            TokenKind::SlashEqual => "/=",
            TokenKind::Percent => "%",
            TokenKind::PercentEqual => "%=",
            TokenKind::Less => "<",
            TokenKind::LessLess => "<<",
            TokenKind::LessEqual => "<=",
            TokenKind::LessLessEqual => "<<=",
            TokenKind::Greater => ">",
            TokenKind::GreaterGreater => ">>",
            TokenKind::GreaterEqual => ">=",
            TokenKind::GreaterGreaterEqual => ">>=",
            TokenKind::Caret => "^",
            TokenKind::CaretEqual => "^=",
            TokenKind::Pipe => "|",
            TokenKind::PipePipe => "||",
            TokenKind::PipeEqual => "|=",
            TokenKind::Question => "?",
            TokenKind::Colon => ":",
            TokenKind::ColonColon => "::",
            TokenKind::Semi => ";",
            TokenKind::Equal => "=",
            TokenKind::EqualEqual => "==",
            TokenKind::Comma => ",",
            TokenKind::Hash => "#",
            TokenKind::HashHash => "##",
            TokenKind::HashAt => "#@",
            TokenKind::At => "@",
            _ => return None,
        })
    }
}

const START_OF_LINE: u16 = 1;
const LEADING_SPACE: u16 = 1 << 1;
const DISABLE_EXPAND: u16 = 1 << 2;
const NEEDS_CLEANING: u16 = 1 << 3;
const PHYSICAL_START_OF_LINE: u16 = 1 << 4;
const STRINGIFIED_IN_MACRO: u16 = 1 << 5;
const COMMA_AFTER_ELIDED: u16 = 1 << 6;
const LEADING_EMPTY_MACRO: u16 = 1 << 7;
const HAS_UCN: u16 = 1 << 8;
const PLACEHOLDER: u16 = 1 << 9;

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub spelling: Rc<[u8]>,
    pub place: Place,
    pub length: u32,
    pub site: Place,
    flags: u16,
}

impl Token {
    fn new(kind: TokenKind, spelling: Rc<[u8]>, place: Place, length: u32) -> Token {
        Token {
            kind,
            spelling,
            place,
            length,
            site: place,
            flags: 0,
        }
    }

    fn marker(kind: TokenKind, place: Place) -> Token {
        Token::new(kind, Rc::from(&b""[..]), place, 0)
    }

    pub fn text(&self) -> &str {
        std::str::from_utf8(&self.spelling).unwrap_or("")
    }

    pub fn is(&self, kind: TokenKind) -> bool {
        self.kind == kind
    }

    pub fn is_identifier(&self, name: &str) -> bool {
        self.kind == TokenKind::Identifier && &*self.spelling == name.as_bytes()
    }

    pub fn at_start_of_line(&self) -> bool {
        self.flags & START_OF_LINE != 0
    }

    pub fn has_leading_space(&self) -> bool {
        self.flags & LEADING_SPACE != 0
    }

    fn has(&self, flag: u16) -> bool {
        self.flags & flag != 0
    }

    fn set(&mut self, flag: u16, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }
}

fn is_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn is_horizontal_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | 0x0c | 0x0b)
}

fn is_vertical_space(byte: u8) -> bool {
    matches!(byte, b'\n' | b'\r')
}

fn is_space(byte: u8) -> bool {
    is_horizontal_space(byte) || is_vertical_space(byte)
}

fn is_number_body(byte: u8) -> bool {
    is_identifier_continue(byte) || byte == b'.'
}

fn trigraph_of(letter: u8) -> Option<u8> {
    Some(match letter {
        b'=' => b'#',
        b')' => b']',
        b'(' => b'[',
        b'!' => b'|',
        b'\'' => b'^',
        b'>' => b'}',
        b'/' => b'\\',
        b'<' => b'{',
        b'-' => b'~',
        _ => return None,
    })
}

fn escaped_newline_size(bytes: &[u8], at: usize) -> usize {
    let mut size = 0usize;
    while at + size < bytes.len() && is_space(bytes[at + size]) {
        size += 1;
        let last = bytes[at + size - 1];
        if last != b'\n' && last != b'\r' {
            continue;
        }
        if at + size < bytes.len() {
            let next = bytes[at + size];
            if (next == b'\r' || next == b'\n') && next != last {
                size += 1;
            }
        }
        return size;
    }
    0
}

fn decode_utf8(bytes: &[u8], at: usize) -> Option<(u32, usize)> {
    let first = *bytes.get(at)?;
    let (length, initial) = match first {
        0xc2..=0xdf => (2, u32::from(first & 0x1f)),
        0xe0..=0xef => (3, u32::from(first & 0x0f)),
        0xf0..=0xf4 => (4, u32::from(first & 0x07)),
        _ => return None,
    };
    if at + length > bytes.len() {
        return None;
    }
    let mut value = initial;
    for index in 1..length {
        let byte = bytes[at + index];
        if byte & 0xc0 != 0x80 {
            return None;
        }
        value = (value << 6) | u32::from(byte & 0x3f);
    }
    let minimum = match length {
        2 => 0x80,
        3 => 0x800,
        _ => 0x10000,
    };
    if value < minimum || value > 0x10ffff || (0xd800..=0xdfff).contains(&value) {
        return None;
    }
    Some((value, length))
}

fn encode_utf8(value: u32, out: &mut Vec<u8>) {
    match char::from_u32(value) {
        Some(character) => {
            let mut scratch = [0u8; 4];
            out.extend_from_slice(character.encode_utf8(&mut scratch).as_bytes());
        }
        None => out.push(b'?'),
    }
}

const C11_IDENTIFIER_RANGES: &[(u32, u32)] = &[
    (0x00a8, 0x00a8),
    (0x00aa, 0x00aa),
    (0x00ad, 0x00ad),
    (0x00af, 0x00af),
    (0x00b2, 0x00b5),
    (0x00b7, 0x00ba),
    (0x00bc, 0x00be),
    (0x00c0, 0x00d6),
    (0x00d8, 0x00f6),
    (0x00f8, 0x00ff),
    (0x0100, 0x167f),
    (0x1681, 0x180d),
    (0x180f, 0x1fff),
    (0x200b, 0x200d),
    (0x202a, 0x202e),
    (0x203f, 0x2040),
    (0x2054, 0x2054),
    (0x2060, 0x206f),
    (0x2070, 0x218f),
    (0x2460, 0x24ff),
    (0x2776, 0x2793),
    (0x2c00, 0x2dff),
    (0x2e80, 0x2fff),
    (0x3004, 0x3007),
    (0x3021, 0x302f),
    (0x3031, 0x303f),
    (0x3040, 0xd7ff),
    (0xf900, 0xfd3d),
    (0xfd40, 0xfdcf),
    (0xfdf0, 0xfe44),
    (0xfe47, 0xfffd),
    (0x10000, 0x1fffd),
    (0x20000, 0x2fffd),
    (0x30000, 0x3fffd),
    (0x40000, 0x4fffd),
    (0x50000, 0x5fffd),
    (0x60000, 0x6fffd),
    (0x70000, 0x7fffd),
    (0x80000, 0x8fffd),
    (0x90000, 0x9fffd),
    (0xa0000, 0xafffd),
    (0xb0000, 0xbfffd),
    (0xc0000, 0xcfffd),
    (0xd0000, 0xdfffd),
    (0xe0000, 0xefffd),
];

const C11_NOT_INITIAL_RANGES: &[(u32, u32)] = &[
    (0x0300, 0x036f),
    (0x1dc0, 0x1dff),
    (0x20d0, 0x20ff),
    (0xfe20, 0xfe2f),
];

fn in_ranges(value: u32, ranges: &[(u32, u32)]) -> bool {
    let index = ranges.partition_point(|(_, high)| *high < value);
    index < ranges.len() && ranges[index].0 <= value
}

fn allowed_in_identifier(value: u32) -> bool {
    in_ranges(value, C11_IDENTIFIER_RANGES)
}

fn allowed_at_identifier_start(value: u32) -> bool {
    allowed_in_identifier(value) && !in_ranges(value, C11_NOT_INITIAL_RANGES)
}

fn is_unicode_whitespace(value: u32) -> bool {
    matches!(
        value,
        0x0085 | 0x00a0 | 0x1680 | 0x2000..=0x200a | 0x2028 | 0x2029 | 0x202f | 0x205f | 0x3000
    )
}

#[derive(Clone, Debug)]
struct SeenComment {
    start: Place,
    text: Rc<[u8]>,
}

#[derive(Clone, Debug)]
struct LexerReport {
    message: Message,
    place: Place,
    arguments: Vec<String>,
}

struct Lexer {
    bytes: Rc<[u8]>,
    buffer: u32,
    position: usize,
    options: LanguageOptions,
    at_start_of_line: bool,
    at_physical_start_of_line: bool,
    leading_space: bool,
    in_directive: bool,
    parsing_filename: bool,
    keep_comments: bool,
    line_comments: bool,
    raw: bool,
    pragma_lexer: bool,
    reports: Vec<LexerReport>,
    comments: Vec<SeenComment>,
}

impl Lexer {
    fn new(bytes: Rc<[u8]>, buffer: u32, options: &LanguageOptions) -> Lexer {
        Lexer {
            bytes,
            buffer,
            position: 0,
            options: options.clone(),
            at_start_of_line: true,
            at_physical_start_of_line: true,
            leading_space: false,
            in_directive: false,
            parsing_filename: false,
            keep_comments: false,
            line_comments: options.line_comments,
            raw: false,
            pragma_lexer: false,
            reports: Vec::new(),
            comments: Vec::new(),
        }
    }

    fn byte(&self, at: usize) -> u8 {
        self.bytes.get(at).copied().unwrap_or(0)
    }

    fn at_end(&self, at: usize) -> bool {
        at >= self.bytes.len()
    }

    fn place(&self, at: usize) -> Place {
        Place {
            buffer: self.buffer,
            offset: at.min(self.bytes.len()) as u32,
        }
    }

    fn report(&mut self, message: Message, at: usize, arguments: Vec<String>) {
        if !self.raw {
            let place = self.place(at);
            self.reports.push(LexerReport {
                message,
                place,
                arguments,
            });
        }
    }

    fn character(&self, at: usize) -> (u8, usize) {
        let byte = self.byte(at);
        if byte != b'\\' && byte != b'?' {
            return (byte, 1);
        }
        self.decode(at, &mut Vec::new(), &mut 0)
    }

    fn decode(
        &self,
        start: usize,
        notes: &mut Vec<(Message, usize, String)>,
        flags: &mut u16,
    ) -> (u8, usize) {
        let mut at = start;
        let mut size = 0usize;
        loop {
            let byte = self.byte(at);
            if byte == b'\\' && !self.at_end(at) {
                let after = at + 1;
                if !is_space(self.byte(after)) || self.at_end(after) {
                    return (b'\\', size + 1);
                }
                let newline = escaped_newline_size(&self.bytes, after);
                if newline == 0 {
                    return (b'\\', size + 1);
                }
                *flags |= NEEDS_CLEANING;
                if !is_vertical_space(self.byte(after)) {
                    notes.push((Message::BackslashNewlineSpace, after, String::new()));
                }
                size += 1 + newline;
                at = after + newline;
                continue;
            }
            if byte == b'?' && self.byte(at + 1) == b'?' && at + 2 < self.bytes.len() {
                if let Some(converted) = trigraph_of(self.byte(at + 2)) {
                    if !self.options.trigraphs {
                        notes.push((Message::TrigraphIgnored, at, String::new()));
                        return (b'?', size + 1);
                    }
                    notes.push((
                        Message::TrigraphConverted,
                        at,
                        (converted as char).to_string(),
                    ));
                    *flags |= NEEDS_CLEANING;
                    if converted == b'\\' {
                        let after = at + 3;
                        if is_space(self.byte(after)) && !self.at_end(after) {
                            let newline = escaped_newline_size(&self.bytes, after);
                            if newline != 0 {
                                if !is_vertical_space(self.byte(after)) {
                                    notes.push((
                                        Message::BackslashNewlineSpace,
                                        after,
                                        String::new(),
                                    ));
                                }
                                size += 3 + newline;
                                at = after + newline;
                                continue;
                            }
                        }
                    }
                    return (converted, size + 3);
                }
            }
            return (byte, size + 1);
        }
    }

    fn take(&mut self, at: usize, flags: &mut u16) -> (u8, usize) {
        let byte = self.byte(at);
        if byte != b'\\' && byte != b'?' {
            return (byte, at + 1);
        }
        let mut notes = Vec::new();
        let (character, size) = self.decode(at, &mut notes, flags);
        for (message, place, argument) in notes {
            let arguments = if argument.is_empty() {
                Vec::new()
            } else {
                vec![argument]
            };
            self.report(message, place, arguments);
        }
        (character, at + size)
    }

    fn consume(&mut self, at: usize, size: usize, flags: &mut u16) -> usize {
        if size == 1 {
            return at + 1;
        }
        self.take(at, flags).1
    }

    fn clean_spelling(&self, start: usize, end: usize, flags: u16) -> Vec<u8> {
        if flags & NEEDS_CLEANING == 0 {
            return self.bytes[start..end.min(self.bytes.len())].to_vec();
        }
        let mut out = Vec::with_capacity(end - start);
        let mut at = start;
        while at < end {
            let (character, size) = self.decode(at, &mut Vec::new(), &mut 0);
            out.push(character);
            at += size.max(1);
        }
        out
    }

    fn form(&mut self, start: usize, end: usize, kind: TokenKind, flags: u16) -> Token {
        let spelling = self.clean_spelling(start, end, flags);
        let mut token = Token::new(
            kind,
            Rc::from(spelling),
            self.place(start),
            (end - start) as u32,
        );
        token.flags = flags;
        self.position = end;
        token
    }

    fn lex(&mut self) -> Token {
        let mut flags = 0u16;
        if self.at_start_of_line {
            flags |= START_OF_LINE;
            self.at_start_of_line = false;
        }
        if self.at_physical_start_of_line {
            flags |= PHYSICAL_START_OF_LINE;
            self.at_physical_start_of_line = false;
        }
        if self.leading_space {
            flags |= LEADING_SPACE;
            self.leading_space = false;
        }
        self.lex_internal(flags)
    }

    fn skip_whitespace(&mut self, mut at: usize, flags: &mut u16) -> Option<usize> {
        let mut saw_newline = at > 0 && is_vertical_space(self.byte(at - 1));
        loop {
            while !self.at_end(at) && is_horizontal_space(self.byte(at)) {
                at += 1;
            }
            if self.at_end(at) || !is_vertical_space(self.byte(at)) {
                break;
            }
            if self.in_directive {
                self.position = at;
                return None;
            }
            saw_newline = true;
            at += 1;
        }
        let previous = if at > 0 { self.byte(at - 1) } else { b'\n' };
        if is_vertical_space(previous) {
            *flags &= !LEADING_SPACE;
        } else {
            *flags |= LEADING_SPACE;
        }
        if saw_newline {
            *flags |= START_OF_LINE | PHYSICAL_START_OF_LINE;
        }
        self.position = at;
        None
    }

    fn handle_comment(&mut self, start: usize, end: usize) {
        if self.raw {
            return;
        }
        let text: Rc<[u8]> = Rc::from(&self.bytes[start..end.min(self.bytes.len())]);
        let comment = SeenComment {
            start: self.place(start),
            text,
        };
        self.comments.push(comment);
    }

    fn skip_line_comment(&mut self, start: usize, mut at: usize, flags: &mut u16) -> Option<Token> {
        if !self.line_comments {
            self.report(Message::LineCommentExtension, start, Vec::new());
            self.line_comments = true;
        }
        loop {
            while !self.at_end(at) && !is_vertical_space(self.byte(at)) {
                at += 1;
            }
            if self.at_end(at) {
                break;
            }
            let next_line = at;
            let mut escape = at as isize - 1;
            let mut has_space = false;
            while escape >= 0 && is_horizontal_space(self.byte(escape as usize)) {
                escape -= 1;
                has_space = true;
            }
            if escape >= 0 && self.byte(escape as usize) == b'\\' {
                at = escape as usize;
            } else if escape >= 2
                && self.byte(escape as usize) == b'/'
                && self.byte(escape as usize - 1) == b'?'
                && self.byte(escape as usize - 2) == b'?'
                && self.options.trigraphs
            {
                at = escape as usize - 2;
            } else {
                break;
            }
            if has_space {
                self.report(Message::BackslashNewlineSpace, escape as usize, Vec::new());
            }
            let old = at;
            let (character, size) = self.decode(at, &mut Vec::new(), flags);
            at = old + size;
            if size == 1 {
                at = next_line;
                break;
            }
            if character != b'/' && (at > self.bytes.len() || self.byte(at) != b'/') {
                let mut probe = old;
                while probe < at {
                    if is_vertical_space(self.byte(probe)) {
                        let mut skip_warning = false;
                        if is_space(character) {
                            let mut forward = at;
                            while !self.at_end(forward) && is_space(self.byte(forward)) {
                                forward += 1;
                            }
                            if self.byte(forward) == b'/' && self.byte(forward + 1) == b'/' {
                                skip_warning = true;
                            }
                        }
                        if !skip_warning {
                            self.report(
                                Message::MultiLineLineComment,
                                probe.saturating_sub(1),
                                Vec::new(),
                            );
                        }
                        break;
                    }
                    probe += 1;
                }
            }
            if is_vertical_space(character) || at > self.bytes.len() {
                at -= 1;
                break;
            }
        }
        self.handle_comment(start, at);
        if self.keep_comments {
            let mut token = self.form(start, at, TokenKind::Comment, *flags);
            if self.in_directive {
                let mut spelling = token.spelling.to_vec();
                if spelling.len() >= 2 {
                    spelling[1] = b'*';
                }
                spelling.extend_from_slice(b"*/");
                token.spelling = Rc::from(spelling);
            }
            return Some(token);
        }
        if self.in_directive || self.at_end(at) {
            self.position = at;
            return None;
        }
        *flags |= START_OF_LINE | PHYSICAL_START_OF_LINE;
        *flags &= !LEADING_SPACE;
        self.position = at + 1;
        None
    }

    fn block_comment_ends_with_escaped_newline(&mut self, mut at: isize) -> bool {
        let mut trigraph: Option<usize> = None;
        let mut space: Option<usize> = None;
        loop {
            at -= 1;
            if at < 0 {
                return false;
            }
            let here = self.byte(at as usize);
            if here == b'\n' || here == b'\r' {
                if here == self.byte(at as usize + 1) {
                    return false;
                }
                at -= 1;
                if at < 0 {
                    return false;
                }
            }
            while at >= 0
                && (is_horizontal_space(self.byte(at as usize)) || self.byte(at as usize) == 0)
            {
                space = Some(at as usize);
                at -= 1;
            }
            if at < 0 {
                return false;
            }
            if self.byte(at as usize) == b'\\' {
                at -= 1;
            } else if at >= 2
                && self.byte(at as usize) == b'/'
                && self.byte(at as usize - 1) == b'?'
                && self.byte(at as usize - 2) == b'?'
            {
                trigraph = Some(at as usize - 2);
                at -= 3;
            } else {
                return false;
            }
            if at < 0 {
                return false;
            }
            let here = self.byte(at as usize);
            if here == b'*' {
                break;
            }
            if here != b'\n' && here != b'\r' {
                return false;
            }
        }
        if let Some(position) = trigraph {
            if !self.options.trigraphs {
                self.report(Message::TrigraphIgnoredBlockComment, position, Vec::new());
                return false;
            }
            self.report(Message::TrigraphEndsBlockComment, position, Vec::new());
        }
        self.report(
            Message::EscapedNewlineBlockCommentEnd,
            at as usize + 1,
            Vec::new(),
        );
        if let Some(position) = space {
            self.report(Message::BackslashNewlineSpace, position, Vec::new());
        }
        true
    }

    fn skip_block_comment(
        &mut self,
        start: usize,
        at: usize,
        flags: &mut u16,
    ) -> Result<Option<Token>, ()> {
        let (first, size) = self.character(at);
        let mut cursor = at + size;
        if first == 0 && self.at_end(at) {
            self.report(Message::UnterminatedBlockComment, start, Vec::new());
            self.position = self.bytes.len();
            return Err(());
        }
        let mut character = first;
        if character == b'/' {
            character = self.byte(cursor);
            cursor += 1;
        }
        loop {
            while character != b'/' {
                if character == 0 && cursor > self.bytes.len() {
                    self.report(Message::UnterminatedBlockComment, start, Vec::new());
                    self.position = self.bytes.len();
                    return Err(());
                }
                character = self.byte(cursor);
                cursor += 1;
            }
            let before = self.byte(cursor - 2);
            if before == b'*' {
                break;
            }
            if (before == b'\n' || before == b'\r')
                && self.block_comment_ends_with_escaped_newline(cursor as isize - 2)
            {
                break;
            }
            if self.byte(cursor) == b'*' && self.byte(cursor + 1) != b'/' {
                self.report(Message::NestedBlockComment, cursor - 1, Vec::new());
            }
            character = self.byte(cursor);
            cursor += 1;
        }
        let at = cursor;
        self.handle_comment(start, at);
        if self.keep_comments {
            let token = self.form(start, at, TokenKind::Comment, *flags);
            return Ok(Some(token));
        }
        if !self.at_end(at) && is_horizontal_space(self.byte(at)) {
            self.skip_whitespace(at + 1, flags);
            *flags |= LEADING_SPACE;
            return Ok(None);
        }
        self.position = at;
        *flags |= LEADING_SPACE;
        Ok(None)
    }

    fn try_read_ucn(
        &mut self,
        start: usize,
        flags: &mut u16,
        consume: bool,
    ) -> Option<(u32, usize)> {
        let (kind, size) = self.character(start + 1);
        let digits = match kind {
            b'u' => 4,
            b'U' => 8,
            _ => return None,
        };
        let mut at = start + 1 + size;
        let mut value = 0u32;
        for _ in 0..digits {
            let (digit, digit_size) = self.character(at);
            let number = (digit as char).to_digit(16)?;
            value = value.wrapping_mul(16).wrapping_add(number);
            at += digit_size;
        }
        if consume {
            let mut cursor = start;
            while cursor < at {
                let (_, next) = self.take(cursor, flags);
                cursor = next;
            }
        }
        Some((value, at))
    }

    fn lex_identifier_rest(&mut self, start: usize, mut at: usize, mut flags: u16) -> Token {
        loop {
            let (character, size) = self.character(at);
            if is_identifier_continue(character) {
                at = self.consume(at, size, &mut flags);
                continue;
            }
            if character == b'$' && self.options.dollar_identifiers && !self.at_end(at) {
                self.report(Message::DollarInIdentifier, at, Vec::new());
                at = self.consume(at, size, &mut flags);
                continue;
            }
            if character == b'\\' {
                if let Some((value, end)) = self.try_read_ucn(at, &mut 0, false) {
                    if allowed_in_identifier(value) {
                        self.try_read_ucn(at, &mut flags, true);
                        flags |= HAS_UCN;
                        at = end;
                        continue;
                    }
                }
            }
            if character >= 0x80 {
                if let Some((value, length)) = decode_utf8(&self.bytes, at) {
                    if allowed_in_identifier(value) && !is_unicode_whitespace(value) {
                        at += length;
                        continue;
                    }
                }
            }
            break;
        }
        let mut token = self.form(start, at, TokenKind::Identifier, flags);
        if flags & HAS_UCN != 0 {
            token.spelling = Rc::from(expand_ucns(&token.spelling));
        }
        token
    }

    fn lex_number(&mut self, start: usize, mut at: usize, mut flags: u16) -> Token {
        let (mut character, mut size) = self.character(at);
        let mut previous = 0u8;
        loop {
            while is_number_body(character) && !self.at_end(at) {
                at = self.consume(at, size, &mut flags);
                previous = character;
                let next = self.character(at);
                character = next.0;
                size = next.1;
            }
            if (character == b'-' || character == b'+') && (previous == b'e' || previous == b'E') {
                at = self.consume(at, size, &mut flags);
                previous = character;
                let next = self.character(at);
                character = next.0;
                size = next.1;
                continue;
            }
            if (character == b'-' || character == b'+') && (previous == b'p' || previous == b'P') {
                let mut hex_float = true;
                if !self.options.at_least(CStandard::C99) {
                    let head = self.clean_spelling(start, at, flags);
                    hex_float =
                        head.len() >= 2 && head[0] == b'0' && (head[1] == b'x' || head[1] == b'X');
                    if hex_float && head.contains(&b'_') {
                        hex_float = false;
                    }
                }
                if hex_float {
                    at = self.consume(at, size, &mut flags);
                    previous = character;
                    let next = self.character(at);
                    character = next.0;
                    size = next.1;
                    continue;
                }
            }
            if character == b'\'' && self.options.at_least(CStandard::C23) {
                let (next, next_size) = self.character(at + size);
                if is_identifier_continue(next) {
                    self.report(Message::DigitSeparatorCompatibility, at, Vec::new());
                    at = self.consume(at, size, &mut flags);
                    at = self.consume(at, next_size, &mut flags);
                    previous = next;
                    let next = self.character(at);
                    character = next.0;
                    size = next.1;
                    continue;
                }
            }
            if character == b'$' && self.options.dollar_identifiers && !self.at_end(at) {
                at = self.consume(at, size, &mut flags);
                previous = character;
                let next = self.character(at);
                character = next.0;
                size = next.1;
                continue;
            }
            if character == b'\\' {
                if let Some((value, end)) = self.try_read_ucn(at, &mut 0, false) {
                    if allowed_in_identifier(value) {
                        self.try_read_ucn(at, &mut flags, true);
                        flags |= HAS_UCN;
                        at = end;
                        previous = 0;
                        let next = self.character(at);
                        character = next.0;
                        size = next.1;
                        continue;
                    }
                }
            }
            if character >= 0x80 {
                if let Some((value, length)) = decode_utf8(&self.bytes, at) {
                    if allowed_in_identifier(value) && !is_unicode_whitespace(value) {
                        at += length;
                        previous = 0;
                        let next = self.character(at);
                        character = next.0;
                        size = next.1;
                        continue;
                    }
                }
            }
            break;
        }
        self.form(start, at, TokenKind::Number, flags)
    }

    fn lex_string(
        &mut self,
        start: usize,
        mut at: usize,
        mut flags: u16,
        kind: TokenKind,
    ) -> Token {
        let mut null_at: Option<usize> = None;
        let (mut character, mut next) = self.take(at, &mut flags);
        at = next;
        while character != b'"' {
            if character == b'\\' {
                let saved = at;
                let taken = self.take(at, &mut flags);
                character = taken.0;
                at = taken.1;
                if self.parsing_filename && character == b'"' {
                    at = saved;
                }
            }
            if is_vertical_space(character) || (character == 0 && at > self.bytes.len()) {
                if !self.options.assembly {
                    self.report(
                        Message::UnterminatedCharacterOrString,
                        start,
                        vec!["1".to_string()],
                    );
                }
                return self.form(start, at - 1, TokenKind::Unknown, flags);
            }
            if character == 0 {
                null_at = Some(at - 1);
            }
            let taken = self.take(at, &mut flags);
            character = taken.0;
            next = taken.1;
            at = next;
        }
        if let Some(place) = null_at {
            self.report(
                Message::NullInCharacterOrString,
                place,
                vec!["1".to_string()],
            );
        }
        self.form(start, at, kind, flags)
    }

    fn lex_angled_header(&mut self, start: usize, at: usize, flags: u16) -> Option<Token> {
        let mut flags = flags;
        let (mut character, mut at) = self.take(at, &mut flags);
        while character != b'>' {
            if character == b'\\' {
                let taken = self.take(at, &mut flags);
                character = taken.0;
                at = taken.1;
            }
            if is_vertical_space(character) || (character == 0 && at > self.bytes.len()) {
                return None;
            }
            let taken = self.take(at, &mut flags);
            character = taken.0;
            at = taken.1;
        }
        Some(self.form(start, at, TokenKind::HeaderName, flags))
    }

    fn lex_character(&mut self, start: usize, at: usize, mut flags: u16, kind: TokenKind) -> Token {
        let mut null_at: Option<usize> = None;
        let (mut character, mut at) = self.take(at, &mut flags);
        if character == b'\'' {
            if !self.options.assembly {
                self.report(Message::EmptyCharacter, start, Vec::new());
            }
            return self.form(start, at, TokenKind::Unknown, flags);
        }
        while character != b'\'' {
            if character == b'\\' {
                let taken = self.take(at, &mut flags);
                character = taken.0;
                at = taken.1;
            }
            if is_vertical_space(character) || (character == 0 && at > self.bytes.len()) {
                if !self.options.assembly {
                    self.report(
                        Message::UnterminatedCharacterOrString,
                        start,
                        vec!["0".to_string()],
                    );
                }
                return self.form(start, at - 1, TokenKind::Unknown, flags);
            }
            if character == 0 {
                null_at = Some(at - 1);
            }
            let taken = self.take(at, &mut flags);
            character = taken.0;
            at = taken.1;
        }
        if let Some(place) = null_at {
            self.report(
                Message::NullInCharacterOrString,
                place,
                vec!["0".to_string()],
            );
        }
        self.form(start, at, kind, flags)
    }

    fn lex_internal(&mut self, mut flags: u16) -> Token {
        'restart: loop {
            let mut at = self.position;
            if !self.at_end(at) && is_horizontal_space(self.byte(at)) {
                while !self.at_end(at) && is_horizontal_space(self.byte(at)) {
                    at += 1;
                }
                self.position = at;
                flags |= LEADING_SPACE;
            }
            let start = self.position;
            if self.at_end(start) {
                return self.lex_end_of_buffer(start, flags);
            }
            let (character, after) = self.take(start, &mut flags);
            let mut at = after;
            let kind: TokenKind;
            match character {
                0 => {
                    if self.at_end(start) {
                        return self.lex_end_of_buffer(start, flags);
                    }
                    self.report(Message::NullInFile, start, Vec::new());
                    flags |= LEADING_SPACE;
                    self.skip_whitespace(at, &mut flags);
                    flags &= !NEEDS_CLEANING;
                    continue 'restart;
                }
                b'\r' | b'\n' => {
                    if character == b'\r' && self.byte(at) == b'\n' {
                        at += 1;
                    }
                    if self.in_directive {
                        self.in_directive = false;
                        self.parsing_filename = false;
                        self.at_start_of_line = true;
                        self.at_physical_start_of_line = true;
                        let token = self.form(start, at, TokenKind::EndOfDirective, flags);
                        return token;
                    }
                    flags &= !LEADING_SPACE;
                    self.skip_whitespace(at, &mut flags);
                    flags &= !NEEDS_CLEANING;
                    continue 'restart;
                }
                b' ' | b'\t' | 0x0c | 0x0b => {
                    flags |= LEADING_SPACE;
                    self.skip_whitespace(at, &mut flags);
                    continue 'restart;
                }
                b'0'..=b'9' => return self.lex_number(start, at, flags),
                b'u' | b'U' | b'L' => {
                    let (next, next_size) = self.character(at);
                    let modern = self.options.at_least(CStandard::C11);
                    if character == b'L' || modern {
                        let prefix = match character {
                            b'L' => LiteralPrefix::Wide,
                            b'u' => LiteralPrefix::Utf16,
                            _ => LiteralPrefix::Utf32,
                        };
                        if next == b'"' {
                            if prefix == LiteralPrefix::Utf16 || prefix == LiteralPrefix::Utf32 {
                                self.report(
                                    Message::UnicodeLiteralCompatibility,
                                    start,
                                    Vec::new(),
                                );
                            }
                            let at = self.consume(at, next_size, &mut flags);
                            return self.lex_string(start, at, flags, TokenKind::String(prefix));
                        }
                        if next == b'\'' {
                            if prefix == LiteralPrefix::Utf16 || prefix == LiteralPrefix::Utf32 {
                                self.report(
                                    Message::UnicodeLiteralCompatibility,
                                    start,
                                    Vec::new(),
                                );
                            }
                            let at = self.consume(at, next_size, &mut flags);
                            return self.lex_character(
                                start,
                                at,
                                flags,
                                TokenKind::Character(prefix),
                            );
                        }
                        if character == b'u' && next == b'8' {
                            let (third, third_size) = self.character(at + next_size);
                            if third == b'"' {
                                let at = self.consume(at, next_size, &mut flags);
                                let at = self.consume(at, third_size, &mut flags);
                                return self.lex_string(
                                    start,
                                    at,
                                    flags,
                                    TokenKind::String(LiteralPrefix::Utf8),
                                );
                            }
                            if third == b'\'' && self.options.at_least(CStandard::C23) {
                                let at = self.consume(at, next_size, &mut flags);
                                let at = self.consume(at, third_size, &mut flags);
                                return self.lex_character(
                                    start,
                                    at,
                                    flags,
                                    TokenKind::Character(LiteralPrefix::Utf8),
                                );
                            }
                        }
                    }
                    return self.lex_identifier_rest(start, at, flags);
                }
                b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                    return self.lex_identifier_rest(start, at, flags)
                }
                b'$' => {
                    if self.options.dollar_identifiers {
                        self.report(Message::DollarInIdentifier, start, Vec::new());
                        return self.lex_identifier_rest(start, at, flags);
                    }
                    kind = TokenKind::Unknown;
                }
                b'\'' => {
                    return self.lex_character(
                        start,
                        at,
                        flags,
                        TokenKind::Character(LiteralPrefix::Plain),
                    )
                }
                b'"' => {
                    let kind = if self.parsing_filename {
                        TokenKind::HeaderName
                    } else {
                        TokenKind::String(LiteralPrefix::Plain)
                    };
                    return self.lex_string(start, at, flags, kind);
                }
                b'?' => kind = TokenKind::Question,
                b'[' => kind = TokenKind::LeftSquare,
                b']' => kind = TokenKind::RightSquare,
                b'(' => kind = TokenKind::LeftParen,
                b')' => kind = TokenKind::RightParen,
                b'{' => kind = TokenKind::LeftBrace,
                b'}' => kind = TokenKind::RightBrace,
                b'.' => {
                    let (next, size) = self.character(at);
                    if next.is_ascii_digit() {
                        let at = self.consume(at, size, &mut flags);
                        return self.lex_number(start, at, flags);
                    } else if next == b'.' && self.character(at + size).0 == b'.' {
                        let second = self.character(at + size).1;
                        at = self.consume(at, size, &mut flags);
                        at = self.consume(at, second, &mut flags);
                        kind = TokenKind::Ellipsis;
                    } else {
                        kind = TokenKind::Period;
                    }
                }
                b'&' => {
                    let (next, size) = self.character(at);
                    if next == b'&' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::AmpAmp;
                    } else if next == b'=' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::AmpEqual;
                    } else {
                        kind = TokenKind::Amp;
                    }
                }
                b'*' => {
                    let (next, size) = self.character(at);
                    if next == b'=' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::StarEqual;
                    } else {
                        kind = TokenKind::Star;
                    }
                }
                b'+' => {
                    let (next, size) = self.character(at);
                    if next == b'+' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::PlusPlus;
                    } else if next == b'=' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::PlusEqual;
                    } else {
                        kind = TokenKind::Plus;
                    }
                }
                b'-' => {
                    let (next, size) = self.character(at);
                    if next == b'-' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::MinusMinus;
                    } else if next == b'>' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::Arrow;
                    } else if next == b'=' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::MinusEqual;
                    } else {
                        kind = TokenKind::Minus;
                    }
                }
                b'~' => kind = TokenKind::Tilde,
                b'!' => {
                    let (next, size) = self.character(at);
                    if next == b'=' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::ExclaimEqual;
                    } else {
                        kind = TokenKind::Exclaim;
                    }
                }
                b'/' => {
                    let (next, size) = self.character(at);
                    if next == b'/' {
                        let mut treat = self.line_comments && !self.options.traditional;
                        if !treat && !self.options.preprocessed_output {
                            treat = self.character(at + size).0 != b'*';
                        }
                        if treat {
                            let body = self.consume(at, size, &mut flags);
                            if let Some(token) = self.skip_line_comment(start, body, &mut flags) {
                                return token;
                            }
                            flags &= !NEEDS_CLEANING;
                            continue 'restart;
                        }
                    }
                    if next == b'*' {
                        let body = self.consume(at, size, &mut flags);
                        match self.skip_block_comment(start, body, &mut flags) {
                            Ok(Some(token)) => return token,
                            Ok(None) => {
                                flags &= !NEEDS_CLEANING;
                                continue 'restart;
                            }
                            Err(()) => {
                                return self.lex_end_of_buffer(self.bytes.len(), flags);
                            }
                        }
                    }
                    if next == b'=' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::SlashEqual;
                    } else {
                        kind = TokenKind::Slash;
                    }
                }
                b'%' => {
                    let (next, size) = self.character(at);
                    if next == b'=' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::PercentEqual;
                    } else if self.options.digraphs && next == b'>' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::RightBrace;
                    } else if self.options.digraphs && next == b':' {
                        at = self.consume(at, size, &mut flags);
                        let (third, third_size) = self.character(at);
                        if third == b'%' && self.character(at + third_size).0 == b':' {
                            let fourth = self.character(at + third_size).1;
                            at = self.consume(at, third_size, &mut flags);
                            at = self.consume(at, fourth, &mut flags);
                            kind = TokenKind::HashHash;
                        } else if third == b'@' && self.options.microsoft_extensions {
                            at = self.consume(at, third_size, &mut flags);
                            self.report(Message::MicrosoftCharize, start, Vec::new());
                            kind = TokenKind::HashAt;
                        } else {
                            kind = TokenKind::Hash;
                        }
                    } else {
                        kind = TokenKind::Percent;
                    }
                }
                b'<' => {
                    if self.parsing_filename {
                        if let Some(token) = self.lex_angled_header(start, at, flags) {
                            return token;
                        }
                    }
                    let (next, size) = self.character(at);
                    if next == b'<' {
                        let (third, third_size) = self.character(at + size);
                        if third == b'=' {
                            at = self.consume(at, size, &mut flags);
                            at = self.consume(at, third_size, &mut flags);
                            kind = TokenKind::LessLessEqual;
                        } else {
                            at = self.consume(at, size, &mut flags);
                            kind = TokenKind::LessLess;
                        }
                    } else if next == b'=' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::LessEqual;
                    } else if self.options.digraphs && next == b':' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::LeftSquare;
                    } else if self.options.digraphs && next == b'%' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::LeftBrace;
                    } else {
                        kind = TokenKind::Less;
                    }
                }
                b'>' => {
                    let (next, size) = self.character(at);
                    if next == b'=' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::GreaterEqual;
                    } else if next == b'>' {
                        let (third, third_size) = self.character(at + size);
                        if third == b'=' {
                            at = self.consume(at, size, &mut flags);
                            at = self.consume(at, third_size, &mut flags);
                            kind = TokenKind::GreaterGreaterEqual;
                        } else {
                            at = self.consume(at, size, &mut flags);
                            kind = TokenKind::GreaterGreater;
                        }
                    } else {
                        kind = TokenKind::Greater;
                    }
                }
                b'^' => {
                    let (next, size) = self.character(at);
                    if next == b'=' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::CaretEqual;
                    } else {
                        kind = TokenKind::Caret;
                    }
                }
                b'|' => {
                    let (next, size) = self.character(at);
                    if next == b'=' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::PipeEqual;
                    } else if next == b'|' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::PipePipe;
                    } else {
                        kind = TokenKind::Pipe;
                    }
                }
                b':' => {
                    let (next, size) = self.character(at);
                    if self.options.digraphs && next == b'>' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::RightSquare;
                    } else if next == b':' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::ColonColon;
                    } else {
                        kind = TokenKind::Colon;
                    }
                }
                b';' => kind = TokenKind::Semi,
                b'=' => {
                    let (next, size) = self.character(at);
                    if next == b'=' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::EqualEqual;
                    } else {
                        kind = TokenKind::Equal;
                    }
                }
                b',' => kind = TokenKind::Comma,
                b'#' => {
                    let (next, size) = self.character(at);
                    if next == b'#' {
                        at = self.consume(at, size, &mut flags);
                        kind = TokenKind::HashHash;
                    } else if next == b'@' && self.options.microsoft_extensions {
                        at = self.consume(at, size, &mut flags);
                        self.report(Message::MicrosoftCharize, start, Vec::new());
                        kind = TokenKind::HashAt;
                    } else {
                        kind = TokenKind::Hash;
                    }
                }
                b'\\' => {
                    if let Some((value, end)) = self.try_read_ucn(start, &mut 0, false) {
                        if is_unicode_whitespace(value) {
                            self.report(
                                Message::UnicodeWhitespace,
                                start,
                                vec![codepoint_name(value)],
                            );
                            flags |= LEADING_SPACE;
                            self.position = end;
                            continue 'restart;
                        }
                        if allowed_at_identifier_start(value) {
                            self.try_read_ucn(start, &mut flags, true);
                            flags |= HAS_UCN;
                            return self.lex_identifier_rest(start, end, flags);
                        }
                        if allowed_in_identifier(value) {
                            self.report(
                                Message::CharacterNotAllowedInIdentifier,
                                start,
                                vec![codepoint_name(value), "1".to_string()],
                            );
                            self.try_read_ucn(start, &mut flags, true);
                            flags |= HAS_UCN;
                            return self.lex_identifier_rest(start, end, flags);
                        }
                    }
                    kind = TokenKind::Unknown;
                }
                _ => {
                    if character < 0x80 {
                        kind = TokenKind::Unknown;
                    } else if let Some((value, length)) = decode_utf8(&self.bytes, start) {
                        let end = start + length;
                        let quiet =
                            self.raw || self.in_directive || self.options.preprocessed_output;
                        if quiet && !allowed_at_identifier_start(value) {
                            at = end;
                            kind = TokenKind::Unknown;
                        } else if is_unicode_whitespace(value) {
                            self.report(
                                Message::UnicodeWhitespace,
                                start,
                                vec![codepoint_name(value)],
                            );
                            flags |= LEADING_SPACE;
                            self.position = end;
                            continue 'restart;
                        } else if allowed_at_identifier_start(value) {
                            return self.lex_identifier_rest(start, end, flags);
                        } else if allowed_in_identifier(value) {
                            self.report(
                                Message::CharacterNotAllowedInIdentifier,
                                start,
                                vec![codepoint_name(value), "1".to_string()],
                            );
                            return self.lex_identifier_rest(start, end, flags);
                        } else {
                            self.report(
                                Message::CharacterNotAllowed,
                                start,
                                vec![codepoint_name(value)],
                            );
                            at = end;
                            kind = TokenKind::Unknown;
                        }
                    } else if self.raw || self.in_directive {
                        kind = TokenKind::Unknown;
                    } else {
                        self.report(Message::InvalidUtf8, start, Vec::new());
                        self.position = start + 1;
                        flags &= !NEEDS_CLEANING;
                        continue 'restart;
                    }
                }
            }
            return self.form(start, at, kind, flags);
        }
    }

    fn lex_end_of_buffer(&mut self, at: usize, flags: u16) -> Token {
        if self.in_directive {
            self.in_directive = false;
            self.parsing_filename = false;
            let mut token = Token::marker(TokenKind::EndOfDirective, self.place(at));
            token.flags = flags;
            self.position = at;
            return token;
        }
        self.position = self.bytes.len();
        let mut token = Token::marker(TokenKind::EndOfFile, self.place(self.bytes.len()));
        token.flags = flags;
        token
    }

    fn read_to_end_of_line(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut at = self.position;
        loop {
            if self.at_end(at) {
                break;
            }
            let (character, next) = self.take(at, &mut 0);
            if is_vertical_space(character) {
                break;
            }
            out.push(character);
            at = next;
        }
        self.position = at;
        out
    }

    fn peek_is_left_paren(&mut self) -> Option<bool> {
        let saved_position = self.position;
        let saved = (
            self.at_start_of_line,
            self.at_physical_start_of_line,
            self.leading_space,
            self.in_directive,
            self.raw,
        );
        let comments = self.comments.len();
        let reports = self.reports.len();
        self.raw = true;
        let token = self.lex();
        self.position = saved_position;
        self.at_start_of_line = saved.0;
        self.at_physical_start_of_line = saved.1;
        self.leading_space = saved.2;
        self.in_directive = saved.3;
        self.raw = saved.4;
        self.comments.truncate(comments);
        self.reports.truncate(reports);
        match token.kind {
            TokenKind::EndOfFile => None,
            TokenKind::LeftParen => Some(true),
            _ => Some(false),
        }
    }
}

fn codepoint_name(value: u32) -> String {
    format!("<U+{value:04X}>")
}

fn expand_ucns(spelling: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(spelling.len());
    let mut at = 0usize;
    while at < spelling.len() {
        if spelling[at] == b'\\' && at + 1 < spelling.len() {
            let digits = match spelling[at + 1] {
                b'u' => 4,
                b'U' => 8,
                _ => 0,
            };
            if digits > 0 && at + 2 + digits <= spelling.len() {
                let text = std::str::from_utf8(&spelling[at + 2..at + 2 + digits]).unwrap_or("");
                if let Ok(value) = u32::from_str_radix(text, 16) {
                    encode_utf8(value, &mut out);
                    at += 2 + digits;
                    continue;
                }
            }
        }
        out.push(spelling[at]);
        at += 1;
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MessageClass {
    Error,
    Fatal,
    Warning,
    WarningOff,
    WarningNoWerror,
    WarningInSystemHeaders,
    Extension,
    ExtWarn,
    ExtWarnError,
    Note,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Message {
    NullInCharacterOrString,
    NullInFile,
    NestedBlockComment,
    EscapedNewlineBlockCommentEnd,
    BackslashNewlineSpace,
    TrigraphIgnored,
    TrigraphIgnoredBlockComment,
    TrigraphEndsBlockComment,
    TrigraphConverted,
    MultiLineLineComment,
    LineCommentExtension,
    NoNewlineAtEndOfFile,
    DollarInIdentifier,
    MicrosoftCharize,
    UnterminatedCharacterOrString,
    EmptyCharacter,
    UnterminatedBlockComment,
    InvalidCharacterToCharify,
    CounterExtension,
    CounterCompatibility,
    MultiCharacterConstant,
    FourCharacterConstant,
    InvalidUtf8,
    CharacterNotAllowed,
    CharacterNotAllowedInIdentifier,
    UnicodeWhitespace,
    DigitSeparatorCompatibility,
    UnicodeLiteralCompatibility,
    CharacterConstantTooLong,
    InvalidDigit,
    InvalidSuffix,
    BinaryLiteralExtension,
    BinaryLiteralCompatibility,
    UnknownEscape,
    NonstandardEscape,
    HexEscapeNoDigits,
    EscapeTooLarge,
    MultipleCharactersInWide,
    CharacterTooLarge,
    IncompleteUniversalCharacterName,
    InvalidUniversalCharacter,
    UniversalCharacterBasicSet,
    IntegerLiteralTooLarge,
    IntegerLiteralTooLargeForSigned,
    LongLongExtension,
    ExponentHasNoDigits,
    HashWarning,
    HashError,
    IncludeNextInPrimary,
    IncludeNextAbsolutePath,
    C99WhitespaceAfterMacroName,
    WhitespaceAfterMacroName,
    PragmaOnceInMainFile,
    PragmaSystemHeaderInMainFile,
    PoisoningExistingMacro,
    UndefBuiltinMacro,
    RedefBuiltinMacro,
    DisabledMacroExpansion,
    MacroNotUsed,
    UndefIdentifier,
    UndefTrueIdentifier,
    MacroHidesKeyword,
    MacroReservedIdentifier,
    InvalidStringLiteral,
    ExpressionOverflow,
    ConvertToPositive,
    ImportDirective,
    IdentDirective,
    IncludeNextDirective,
    WarningDirectiveExtension,
    WarningDirectiveCompatibility,
    EmbedDirectiveExtension,
    EmbedDirectiveCompatibility,
    ExtraTokensAtEndOfDirective,
    CommaInIfOperand,
    BadVaArgsUse,
    BadVaOptUse,
    MissingLeftParenAfterVaOpt,
    NestedVaOpt,
    VaOptPasteAtStart,
    VaOptPasteAtEnd,
    MacroRedefined,
    VariadicMacroExtension,
    NamedVariadicMacro,
    EmbeddedDirective,
    EmbeddedDirectiveError,
    MissingVariadicArgumentExtension,
    MissingVariadicArgumentCompatibility,
    EmptyMacroArgumentExtension,
    MacroDefinedHere,
    PreviousDefinition,
    GnuLineMarker,
    InvalidDirective,
    FileNotFound,
    FileNotFoundAngled,
    EmptyFilename,
    IncludeTooDeep,
    ExpectsFilename,
    MacroNotIdentifier,
    MissingMacroName,
    MissingRightParenInMacroDefinition,
    InvalidTokenInParameterList,
    ExpectedIdentifierInParameterList,
    ExpectedCommaInParameterList,
    DuplicateParameterName,
    StringizeNotParameter,
    MalformedIdent,
    UnterminatedConditional,
    ElseAfterElse,
    ElifAfterElse,
    ElseWithoutIf,
    ElifWithoutIf,
    EndifWithoutIf,
    ExpectedValueInExpression,
    ExpectedRightParenInExpression,
    ExpectedEndOfLineInExpression,
    MissingAfter,
    ColonWithoutQuestion,
    DivisionByZero,
    RemainderByZero,
    BadBinaryOperator,
    FunctionLikeMacroNotDefined,
    InvalidTokenAtStart,
    InvalidPoison,
    UsedPoisoned,
    FeatureCheckMalformed,
    HasWarningInvalidOption,
    IdentifierArgumentNotIdentifier,
    PragmaOperatorMalformed,
    PragmaMessageMalformed,
    PushPopMacroMalformed,
    PopMacroWithoutPush,
    PushPopMacroEmptyString,
    PragmaMessage,
    PragmaError,
    UnknownPragma,
    PragmaDiagnosticInvalid,
    PragmaDiagnosticCannotPop,
    PragmaDiagnosticInvalidOption,
    PragmaDiagnosticInvalidToken,
    PragmaDiagnosticUnknownWarning,
    OnOffSwitchSyntax,
    PragmaSyntaxEndOfDirective,
    DefinedMacroName,
    DefinedInEmbed,
    PasteAtStart,
    PasteAtEnd,
    PasteComma,
    UnterminatedMacroInvocation,
    TooManyArguments,
    TooFewArguments,
    BadPaste,
    FloatingLiteralInExpression,
    LineRequiresInteger,
    LineZero,
    LineInvalidFilename,
    LineDecimal,
    LineDigitSequence,
    LinemarkerRequiresInteger,
    LinemarkerInvalidFilename,
    LinemarkerInvalidFlag,
    LinemarkerInvalidPop,
    LineTooBig,
    DateTime,
    C23DirectiveExtension,
    C23DirectiveCompatibility,
    ExpansionToDefinedObject,
    ExpansionToDefinedFunction,
    HeaderGuard,
    HeaderGuardNote,
    Expected,
    ToMatch,
    ExpectedStringLiteral,
    UnterminatedPragmaOperator,
    UnknownParameter,
    EmbedDuplicateParameter,
    EmbedCannotOpen,
    UnknownWarningOption,
    TooManyErrors,
    UnterminatedMicrosoftPragma,
    PragmaEnteredHere,
    AmbiguousMacro,
    OperatorUsedAsMacroName,
    DirectiveRequired,
    RequiresPositiveValue,
    PragmaDebugMissingCommand,
    PragmaDebugUnexpectedCommand,
    PragmaWarningExpected,
    IncludeCycle,
    WhitespaceRecommendedAfterMacroName,
    UnknownDirectiveSuggestion,
    HexConstantRequires,
    HexFloatExtension,
    BadCharacterEncodingWarning,
    BadCharacterEncoding,
    UniversalCharacterControl,
    MacroExpansionRequestedHere,
    NestedParentheses,
}

impl Message {
    fn info(self) -> (MessageClass, &'static str, &'static str) {
        use MessageClass::*;
        match self {
            Message::NullInCharacterOrString => (
                Warning,
                "null-character",
                "null character(s) preserved in %select{char|string}0 literal",
            ),
            Message::NullInFile => (Warning, "null-character", "null character ignored"),
            Message::NestedBlockComment => (Warning, "comment", "'/*' within block comment"),
            Message::EscapedNewlineBlockCommentEnd => (
                Warning,
                "comment",
                "escaped newline between */ characters at block comment end",
            ),
            Message::BackslashNewlineSpace => (
                Warning,
                "backslash-newline-escape",
                "backslash and newline separated by space",
            ),
            Message::TrigraphIgnored => (Warning, "trigraphs", "trigraph ignored"),
            Message::TrigraphIgnoredBlockComment => {
                (Warning, "trigraphs", "ignored trigraph would end block comment")
            }
            Message::TrigraphEndsBlockComment => (Warning, "trigraphs", "trigraph ends block comment"),
            Message::TrigraphConverted => (Warning, "trigraphs", "trigraph converted to '%0' character"),
            Message::MultiLineLineComment => (Extension, "comment", "multi-line // comment"),
            Message::LineCommentExtension => {
                (Extension, "comment", "// comments are not allowed in this language")
            }
            Message::NoNewlineAtEndOfFile => (WarningOff, "newline-eof", "no newline at end of file"),
            Message::DollarInIdentifier => {
                (Extension, "dollar-in-identifier-extension", "'$' in identifier")
            }
            Message::MicrosoftCharize => (
                Extension,
                "microsoft-charize",
                "charizing operator #@ is a Microsoft extension",
            ),
            Message::UnterminatedCharacterOrString => (
                ExtWarn,
                "invalid-pp-token",
                "missing terminating %select{'|'\"'}0 character",
            ),
            Message::EmptyCharacter => (ExtWarn, "invalid-pp-token", "empty character constant"),
            Message::UnterminatedBlockComment => (Error, "", "unterminated /* comment"),
            Message::InvalidCharacterToCharify => {
                (Error, "", "invalid argument to convert to character")
            }
            Message::CounterExtension => {
                (Extension, "counter-extension", "'__COUNTER__' is a C2y extension")
            }
            Message::CounterCompatibility => (
                WarningOff,
                "pre-c2y-compat",
                "'__COUNTER__' is incompatible with standards before C2y",
            ),
            Message::MultiCharacterConstant => {
                (Warning, "multichar", "multi-character character constant")
            }
            Message::FourCharacterConstant => {
                (WarningOff, "four-char-constants", "multi-character character constant")
            }
            Message::InvalidUtf8 => (Error, "", "source file is not valid UTF-8"),
            Message::CharacterNotAllowed => (Error, "", "unexpected character %0"),
            Message::CharacterNotAllowedInIdentifier => (
                Error,
                "",
                "character %0 not allowed %select{in|at the start of}1 an identifier",
            ),
            Message::UnicodeWhitespace => {
                (ExtWarnError, "unicode-whitespace", "treating character %0 as whitespace")
            }
            Message::DigitSeparatorCompatibility => (
                WarningOff,
                "pre-c23-compat",
                "digit separators are incompatible with C standards before C23",
            ),
            Message::UnicodeLiteralCompatibility => {
                (WarningOff, "c99-compat", "unicode literals are incompatible with C99")
            }
            Message::CharacterConstantTooLong => (Warning, "", "character constant too long for its type"),
            Message::InvalidDigit => (
                Error,
                "",
                "invalid digit '%0' in %select{decimal|octal|binary}1 constant",
            ),
            Message::InvalidSuffix => (
                Error,
                "",
                "invalid suffix '%0' on %select{integer|floating|fixed-point}1 constant",
            ),
            Message::BinaryLiteralExtension => {
                (Extension, "c23-extensions", "binary integer literals are a C23 extension")
            }
            Message::BinaryLiteralCompatibility => (
                WarningOff,
                "pre-c23-compat",
                "binary integer literals are incompatible with C standards before C23",
            ),
            Message::UnknownEscape => {
                (ExtWarn, "unknown-escape-sequence", "unknown escape sequence '\\%0'")
            }
            Message::NonstandardEscape => {
                (Extension, "", "use of non-standard escape character '\\%0'")
            }
            Message::HexEscapeNoDigits => (Error, "", "\\%0 used with no following hex digits"),
            Message::EscapeTooLarge => {
                (Error, "", "%select{hex|octal}0 escape sequence out of range")
            }
            Message::MultipleCharactersInWide => (
                Error,
                "",
                "%select{wide|Unicode}0 character literals may not contain multiple characters",
            ),
            Message::CharacterTooLarge => {
                (Error, "", "character too large for enclosing character literal type")
            }
            Message::IncompleteUniversalCharacterName => {
                (Error, "", "incomplete universal character name")
            }
            Message::InvalidUniversalCharacter => (Error, "", "invalid universal character"),
            Message::UniversalCharacterBasicSet => (
                Error,
                "",
                "character '%0' cannot be specified by a universal character name",
            ),
            Message::IntegerLiteralTooLarge => (
                Error,
                "",
                "integer literal is too large to be represented in any %select{signed |}0integer type",
            ),
            Message::IntegerLiteralTooLargeForSigned => (
                ExtWarn,
                "implicitly-unsigned-literal",
                "integer literal is too large to be represented in a signed integer type, interpreting as unsigned",
            ),
            Message::LongLongExtension => (
                Extension,
                "long-long",
                "'long long' is an extension when C99 mode is not enabled",
            ),
            Message::ExponentHasNoDigits => (Error, "", "exponent has no digits"),
            Message::HashWarning => (WarningInSystemHeaders, "#warnings", "%0"),
            Message::HashError => (Error, "", "%0"),
            Message::IncludeNextInPrimary => (
                Warning,
                "include-next-outside-header",
                "#include_next in primary source file; will search from start of include path",
            ),
            Message::IncludeNextAbsolutePath => (
                Warning,
                "include-next-absolute-path",
                "#include_next in file found relative to primary source file or found by absolute path; will search from start of include path",
            ),
            Message::C99WhitespaceAfterMacroName => (
                ExtWarn,
                "c99-extensions",
                "ISO C99 requires whitespace after the macro name",
            ),
            Message::WhitespaceAfterMacroName => (ExtWarn, "", "whitespace required after macro name"),
            Message::PragmaOnceInMainFile => {
                (Warning, "pragma-once-outside-header", "#pragma once in main file")
            }
            Message::PragmaSystemHeaderInMainFile => (
                Warning,
                "pragma-system-header-outside-header",
                "#pragma system_header ignored in main file",
            ),
            Message::PoisoningExistingMacro => (Warning, "", "poisoning existing macro"),
            Message::UndefBuiltinMacro => {
                (ExtWarn, "builtin-macro-redefined", "undefining builtin macro")
            }
            Message::RedefBuiltinMacro => {
                (ExtWarn, "builtin-macro-redefined", "redefining builtin macro")
            }
            Message::DisabledMacroExpansion => (
                WarningOff,
                "disabled-macro-expansion",
                "disabled expansion of recursive macro",
            ),
            Message::MacroNotUsed => (WarningOff, "unused-macros", "macro is not used"),
            Message::UndefIdentifier => (WarningOff, "undef", "%0 is not defined, evaluates to 0"),
            Message::UndefTrueIdentifier => {
                (Warning, "undef-true", "'true' is not defined, evaluates to 0")
            }
            Message::MacroHidesKeyword => {
                (Extension, "keyword-macro", "keyword is hidden by macro definition")
            }
            Message::MacroReservedIdentifier => (
                WarningOff,
                "reserved-macro-identifier",
                "macro name is a reserved identifier",
            ),
            Message::InvalidStringLiteral => {
                (Warning, "", "invalid string literal, ignoring final '\\'")
            }
            Message::ExpressionOverflow => (Warning, "", "integer overflow in preprocessor expression"),
            Message::ConvertToPositive => (
                Warning,
                "",
                "%select{left|right}0 side of operator converted from negative value to unsigned: %1",
            ),
            Message::ImportDirective => (
                Extension,
                "import-preprocessor-directive-pedantic",
                "#import is a language extension",
            ),
            Message::IdentDirective => (Extension, "", "#ident is a language extension"),
            Message::IncludeNextDirective => {
                (Extension, "gnu-include-next", "#include_next is a language extension")
            }
            Message::WarningDirectiveExtension => {
                (Extension, "", "#warning is a %select{C23|C++23}0 extension")
            }
            Message::WarningDirectiveCompatibility => (
                WarningOff,
                "pre-c23-compat",
                "#warning is incompatible with C standards before C23",
            ),
            Message::EmbedDirectiveExtension => {
                (ExtWarn, "c23-extensions", "#embed is a %select{C23|Clang}0 extension")
            }
            Message::EmbedDirectiveCompatibility => (
                WarningOff,
                "pre-c23-compat",
                "#embed is incompatible with C standards before C23",
            ),
            Message::ExtraTokensAtEndOfDirective => {
                (ExtWarn, "extra-tokens", "extra tokens at end of #%0 directive")
            }
            Message::CommaInIfOperand => (Extension, "", "comma operator in operand of #if"),
            Message::BadVaArgsUse => (
                Extension,
                "",
                "__VA_ARGS__ can only appear in the expansion of a C99 variadic macro",
            ),
            Message::BadVaOptUse => (
                ExtWarn,
                "variadic-macros",
                "__VA_OPT__ can only appear in the expansion of a variadic macro",
            ),
            Message::MissingLeftParenAfterVaOpt => (Error, "", "missing '(' following __VA_OPT__"),
            Message::NestedVaOpt => {
                (Error, "", "__VA_OPT__ cannot be nested within its own replacement tokens")
            }
            Message::VaOptPasteAtStart => {
                (Error, "", "'##' cannot appear at start of __VA_OPT__ argument")
            }
            Message::VaOptPasteAtEnd => (Error, "", "'##' cannot appear at end of __VA_OPT__ argument"),
            Message::MacroRedefined => (ExtWarn, "macro-redefined", "%0 macro redefined"),
            Message::VariadicMacroExtension => {
                (Extension, "variadic-macros", "variadic macros are a C99 feature")
            }
            Message::NamedVariadicMacro => {
                (Extension, "variadic-macros", "named variadic macros are a GNU extension")
            }
            Message::EmbeddedDirective => (
                Extension,
                "embedded-directive",
                "embedding a directive within macro arguments has undefined behavior",
            ),
            Message::EmbeddedDirectiveError => (
                Error,
                "",
                "embedding a %select{|#}0%1 directive within macro arguments is not supported",
            ),
            Message::MissingVariadicArgumentExtension => (
                Extension,
                "variadic-macro-arguments-omitted",
                "passing no argument for the '...' parameter of a variadic macro is a C23 extension",
            ),
            Message::MissingVariadicArgumentCompatibility => (
                WarningOff,
                "variadic-macro-arguments-omitted",
                "passing no argument for the '...' parameter of a variadic macro is incompatible with C standards before C23",
            ),
            Message::EmptyMacroArgumentExtension => {
                (Extension, "c99-extensions", "empty macro arguments are a C99 feature")
            }
            Message::MacroDefinedHere => (Note, "", "macro %0 defined here"),
            Message::PreviousDefinition => (Note, "", "previous definition is here"),
            Message::GnuLineMarker => (
                Extension,
                "gnu-line-marker",
                "this style of line directive is a GNU extension",
            ),
            Message::InvalidDirective => (
                Error,
                "",
                "invalid preprocessing directive%select{|, did you mean '#%1'?}0",
            ),
            Message::FileNotFound => (Fatal, "", "'%0' file not found"),
            Message::FileNotFoundAngled => (
                Error,
                "",
                "'%0' file not found with <angled> %select{include|import}1; use \"quotes\" instead",
            ),
            Message::EmptyFilename => (Error, "", "empty filename"),
            Message::IncludeTooDeep => (Error, "", "#include nested too deeply"),
            Message::ExpectsFilename => (Error, "", "expected \"FILENAME\" or <FILENAME>"),
            Message::MacroNotIdentifier => (Error, "", "macro name must be an identifier"),
            Message::MissingMacroName => (Error, "", "macro name missing"),
            Message::MissingRightParenInMacroDefinition => {
                (Error, "", "missing ')' in macro parameter list")
            }
            Message::InvalidTokenInParameterList => (Error, "", "invalid token in macro parameter list"),
            Message::ExpectedIdentifierInParameterList => {
                (Error, "", "expected identifier in macro parameter list")
            }
            Message::ExpectedCommaInParameterList => {
                (Error, "", "expected comma in macro parameter list")
            }
            Message::DuplicateParameterName => (Error, "", "duplicate macro parameter name %0"),
            Message::StringizeNotParameter => {
                (Error, "", "'%select{#|#@}0' is not followed by a macro parameter")
            }
            Message::MalformedIdent => (Error, "", "invalid #ident directive"),
            Message::UnterminatedConditional => (Error, "", "unterminated conditional directive"),
            Message::ElseAfterElse => (Error, "", "#else after #else"),
            Message::ElifAfterElse => (Error, "", "%select{#elif|#elifdef|#elifndef}0 after #else"),
            Message::ElseWithoutIf => (Error, "", "#else without #if"),
            Message::ElifWithoutIf => (Error, "", "%select{#elif|#elifdef|#elifndef}0 without #if"),
            Message::EndifWithoutIf => (Error, "", "#endif without #if"),
            Message::ExpectedValueInExpression => (Error, "", "expected value in expression"),
            Message::ExpectedRightParenInExpression => {
                (Error, "", "expected ')' in preprocessor expression")
            }
            Message::ExpectedEndOfLineInExpression => {
                (Error, "", "expected end of line in preprocessor expression")
            }
            Message::MissingAfter => (Error, "", "missing %1 after %0"),
            Message::ColonWithoutQuestion => (Error, "", "':' without preceding '?'"),
            Message::DivisionByZero => (Error, "", "division by zero in preprocessor expression"),
            Message::RemainderByZero => (Error, "", "remainder by zero in preprocessor expression"),
            Message::BadBinaryOperator => (
                Error,
                "",
                "token is not a valid binary operator in a preprocessor subexpression",
            ),
            Message::FunctionLikeMacroNotDefined => (Error, "", "function-like macro %0 is not defined"),
            Message::InvalidTokenAtStart => {
                (Error, "", "invalid token at start of a preprocessor expression")
            }
            Message::InvalidPoison => (Error, "", "can only poison identifier tokens"),
            Message::UsedPoisoned => (Error, "", "attempt to use a poisoned identifier"),
            Message::FeatureCheckMalformed => (
                Error,
                "",
                "builtin feature check macro requires a parenthesized identifier",
            ),
            Message::HasWarningInvalidOption => (
                ExtWarn,
                "malformed-warning-check",
                "__has_warning expected option name (e.g. \"-Wundef\")",
            ),
            Message::IdentifierArgumentNotIdentifier => {
                (Error, "", "cannot convert %0 token to an identifier")
            }
            Message::PragmaOperatorMalformed => {
                (Error, "", "_Pragma takes a parenthesized string literal")
            }
            Message::PragmaMessageMalformed => (
                Error,
                "",
                "pragma %select{message|warning|error}0 requires parenthesized string",
            ),
            Message::PushPopMacroMalformed => (Error, "", "pragma %0 requires a parenthesized string"),
            Message::PopMacroWithoutPush => (
                Warning,
                "ignored-pragmas",
                "pragma pop_macro could not pop '%0', no matching push_macro",
            ),
            Message::PushPopMacroEmptyString => (
                Warning,
                "ignored-pragmas",
                "'#pragma %select{push_macro|pop_macro}0' expected a non-empty string",
            ),
            Message::PragmaMessage => (WarningNoWerror, "#pragma-messages", "%0"),
            Message::PragmaError => (Error, "", "%0"),
            Message::UnknownPragma => (WarningOff, "unknown-pragmas", "unknown pragma ignored"),
            Message::PragmaDiagnosticInvalid => (
                ExtWarn,
                "unknown-pragmas",
                "pragma diagnostic expected 'error', 'warning', 'ignored', 'fatal', 'push', or 'pop'",
            ),
            Message::PragmaDiagnosticCannotPop => (
                ExtWarn,
                "unknown-pragmas",
                "pragma diagnostic pop could not pop, no matching push",
            ),
            Message::PragmaDiagnosticInvalidOption => (
                ExtWarn,
                "unknown-pragmas",
                "pragma diagnostic expected option name (e.g. \"-Wundef\")",
            ),
            Message::PragmaDiagnosticInvalidToken => {
                (ExtWarn, "unknown-pragmas", "unexpected token in pragma diagnostic")
            }
            Message::PragmaDiagnosticUnknownWarning => {
                (ExtWarn, "unknown-warning-option", "unknown warning group '%0', ignored")
            }
            Message::OnOffSwitchSyntax => (
                ExtWarn,
                "unknown-pragmas",
                "expected 'ON' or 'OFF' or 'DEFAULT' in pragma",
            ),
            Message::PragmaSyntaxEndOfDirective => {
                (ExtWarn, "unknown-pragmas", "expected end of directive in pragma")
            }
            Message::DefinedMacroName => (Error, "", "'defined' cannot be used as a macro name"),
            Message::DefinedInEmbed => (Error, "", "'defined' cannot appear within this context"),
            Message::PasteAtStart => (Error, "", "'##' cannot appear at start of macro expansion"),
            Message::PasteAtEnd => (Error, "", "'##' cannot appear at end of macro expansion"),
            Message::PasteComma => (
                Extension,
                "gnu-zero-variadic-macro-arguments",
                "token pasting of ',' and '__VA_ARGS__' is a GNU extension%select{|; consider using '__VA_OPT__(,)' instead}0",
            ),
            Message::UnterminatedMacroInvocation => {
                (Error, "", "unterminated function-like macro invocation")
            }
            Message::TooManyArguments => {
                (Error, "", "too many arguments provided to function-like macro invocation")
            }
            Message::TooFewArguments => {
                (Error, "", "too few arguments provided to function-like macro invocation")
            }
            Message::BadPaste => (Error, "", "pasting formed '%0', an invalid preprocessing token"),
            Message::FloatingLiteralInExpression => {
                (Error, "", "floating point literal in preprocessor expression")
            }
            Message::LineRequiresInteger => {
                (Error, "", "#line directive requires a positive integer argument")
            }
            Message::LineZero => (
                Extension,
                "gnu-zero-line-directive",
                "#line directive with zero argument is a GNU extension",
            ),
            Message::LineInvalidFilename => (Error, "", "invalid filename for #line directive"),
            Message::LineDecimal => (
                Warning,
                "",
                "%select{#line|GNU line marker}0 directive interprets number as decimal, not octal",
            ),
            Message::LineDigitSequence => (
                Error,
                "",
                "%select{#line|GNU line marker}0 directive requires a simple digit sequence",
            ),
            Message::LinemarkerRequiresInteger => {
                (Error, "", "line marker directive requires a positive integer argument")
            }
            Message::LinemarkerInvalidFilename => {
                (Error, "", "invalid filename for line marker directive")
            }
            Message::LinemarkerInvalidFlag => (Error, "", "invalid flag line marker directive"),
            Message::LinemarkerInvalidPop => (
                Error,
                "",
                "invalid line marker flag '2': cannot pop empty include stack",
            ),
            Message::LineTooBig => (
                Extension,
                "",
                "C requires #line number to be less than %0, allowed as extension",
            ),
            Message::DateTime => (
                WarningOff,
                "date-time",
                "expansion of date or time macro is not reproducible",
            ),
            Message::C23DirectiveExtension => (
                ExtWarn,
                "c23-extensions",
                "use of a '#%select{<BUG IF SEEN>|elifdef|elifndef}0' directive is a C23 extension",
            ),
            Message::C23DirectiveCompatibility => (
                WarningOff,
                "pre-c23-compat",
                "use of a '#%select{<BUG IF SEEN>|elifdef|elifndef}0' directive is incompatible with C standards before C23",
            ),
            Message::ExpansionToDefinedObject => (
                Warning,
                "expansion-to-defined",
                "macro expansion producing 'defined' has undefined behavior",
            ),
            Message::ExpansionToDefinedFunction => (
                Extension,
                "expansion-to-defined",
                "macro expansion producing 'defined' has undefined behavior",
            ),
            Message::HeaderGuard => (
                Warning,
                "header-guard",
                "%0 is used as a header guard here, followed by #define of a different macro",
            ),
            Message::HeaderGuardNote => (Note, "", "%0 is defined here; did you mean %1?"),
            Message::Expected => (Error, "", "expected %0"),
            Message::ToMatch => (Note, "", "to match this %0"),
            Message::ExpectedStringLiteral => (Error, "", "expected string literal in %0"),
            Message::UnterminatedPragmaOperator => (Error, "", "missing terminating ')' character"),
            Message::UnknownParameter => {
                (Error, "", "unknown%select{ | embed}0 preprocessor parameter '%1'")
            }
            Message::EmbedDuplicateParameter => (
                Error,
                "",
                "cannot specify parameter '%0' twice in the same '#embed' directive",
            ),
            Message::EmbedCannotOpen => (Fatal, "", "'%0' file not found"),
            Message::TooManyErrors => (Fatal, "", "too many errors emitted, stopping now"),
            Message::UnterminatedMicrosoftPragma => (Error, "", "missing terminating ')' character"),
            Message::UnknownWarningOption => {
                (Warning, "unknown-warning-option", "unknown warning option '%0'")
            }
            Message::PragmaEnteredHere => (Note, "", "#pragma entered here"),
            Message::AmbiguousMacro => (Warning, "ambiguous-macro", "ambiguous expansion of macro %0"),
            Message::OperatorUsedAsMacroName => {
                (Error, "", "C++ operator %0 (aka %1) used as a macro name")
            }
            Message::DirectiveRequired => {
                (Error, "", "%0 must be used within a preprocessing directive")
            }
            Message::RequiresPositiveValue => (
                Error,
                "",
                "%select{invalid value '%0'; must be positive|value '%0' is too large}1",
            ),
            Message::PragmaDebugMissingCommand => {
                (Warning, "ignored-pragmas", "missing debug command")
            }
            Message::PragmaDebugUnexpectedCommand => {
                (Warning, "ignored-pragmas", "unexpected debug command '%0'")
            }
            Message::PragmaWarningExpected => {
                (ExtWarn, "unknown-pragmas", "#pragma warning expected '%0'")
            }
            Message::IncludeCycle => (Error, "", "#include nested too deeply"),
            Message::WhitespaceRecommendedAfterMacroName => {
                (Warning, "", "whitespace recommended after macro name")
            }
            Message::UnknownDirectiveSuggestion => (
                Warning,
                "unknown-directives",
                "invalid preprocessing directive%select{|, did you mean '#%1'?}0",
            ),
            Message::HexConstantRequires => (
                Error,
                "",
                "hexadecimal floating %select{constant|literal}0 requires %select{an exponent|a significand}1",
            ),
            Message::HexFloatExtension => (
                Extension,
                "c99-extensions",
                "hexadecimal floating constants are a C99 feature",
            ),
            Message::BadCharacterEncodingWarning => (
                ExtWarn,
                "invalid-source-encoding",
                "illegal character encoding in character literal",
            ),
            Message::BadCharacterEncoding => {
                (Error, "", "illegal character encoding in character literal")
            }
            Message::UniversalCharacterControl => {
                (Error, "", "universal character name refers to a control character")
            }
            Message::MacroExpansionRequestedHere => (Note, "", "expansion of macro %0 requested here"),
            Message::NestedParentheses => (Error, "", "nested parentheses not permitted in %0"),
        }
    }

    fn class(self) -> MessageClass {
        self.info().0
    }

    fn group(self) -> &'static str {
        self.info().1
    }

    fn is_extension(self) -> bool {
        matches!(
            self.class(),
            MessageClass::Extension | MessageClass::ExtWarn | MessageClass::ExtWarnError
        )
    }

    fn is_warning_like(self) -> bool {
        !matches!(
            self.class(),
            MessageClass::Error | MessageClass::Fatal | MessageClass::Note
        )
    }

    pub fn render(self, arguments: &[String]) -> String {
        render_message(self.info().2, arguments)
    }
}

fn render_message(template: &str, arguments: &[String]) -> String {
    let bytes = template.as_bytes();
    let mut out = String::with_capacity(template.len() + 16);
    let mut at = 0usize;
    while at < bytes.len() {
        if bytes[at] != b'%' {
            let next = template[at..]
                .find('%')
                .map(|found| at + found)
                .unwrap_or(bytes.len());
            out.push_str(&template[at..next]);
            at = next;
            continue;
        }
        if template[at..].starts_with("%select{") {
            let open = at + "%select{".len();
            let mut depth = 1usize;
            let mut close = open;
            while close < bytes.len() {
                match bytes[close] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                close += 1;
            }
            let mut number_end = close + 1;
            while number_end < bytes.len() && bytes[number_end].is_ascii_digit() {
                number_end += 1;
            }
            let index: usize = template[close + 1..number_end].parse().unwrap_or(0);
            let chosen: usize = arguments
                .get(index)
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            let mut choices = Vec::new();
            let mut depth = 0usize;
            let mut start = open;
            for position in open..close {
                match bytes[position] {
                    b'{' => depth += 1,
                    b'}' => depth = depth.saturating_sub(1),
                    b'|' if depth == 0 => {
                        choices.push(&template[start..position]);
                        start = position + 1;
                    }
                    _ => {}
                }
            }
            choices.push(&template[start..close]);
            let picked = choices.get(chosen).copied().unwrap_or("");
            out.push_str(&render_message(picked, arguments));
            at = number_end;
            continue;
        }
        let mut number_end = at + 1;
        while number_end < bytes.len() && bytes[number_end].is_ascii_digit() {
            number_end += 1;
        }
        if number_end == at + 1 {
            out.push('%');
            at += 1;
            continue;
        }
        let index: usize = template[at + 1..number_end].parse().unwrap_or(0);
        if let Some(value) = arguments.get(index) {
            out.push_str(value);
        }
        at = number_end;
    }
    out
}

fn quoted(name: &[u8]) -> String {
    format!("'{}'", String::from_utf8_lossy(name))
}

const WARNING_GROUP_TREE: &str = "\
#pragma-messages\n\
#warnings\n\
CFString-literal\n\
IndependentClass-attribute\n\
NSObject-attribute\n\
TU-local-entity-exposure\n\
aarch64-sme-attributes\n\
absolute-value\n\
abstract-final-class\n\
abstract-vbase-init\n\
address-of-packed-member\n\
address-of-temporary\n\
aix-compat\n\
alias-template-in-declaration-name\n\
align-mismatch\n\
all most parentheses switch switch-bool misleading-indentation packed-non-pod vla-cxx-extension\n\
alloc-size\n\
alloca\n\
alloca-with-align-alignof\n\
always-inline-coroutine\n\
ambiguous-delete\n\
ambiguous-ellipsis\n\
ambiguous-macro\n\
ambiguous-member-template\n\
ambiguous-reversed-operator\n\
amdgpu-av\n\
amdgpu-unguarded-builtin-usage\n\
amdgpu-wait-event-mask\n\
analyzer-incompatible-plugin\n\
android-unversioned-fallback\n\
anon-enum-enum-conversion deprecated-anon-enum-enum-conversion\n\
anonymous-pack-parens\n\
apinotes\n\
arc arc-unsafe-retained-assign arc-retain-cycles arc-non-pod-memaccess\n\
arc-bridge-casts-disallowed-in-nonarc\n\
arc-maybe-repeated-use-of-weak\n\
arc-non-pod-memaccess\n\
arc-performSelector-leaks\n\
arc-repeated-use-of-weak arc-maybe-repeated-use-of-weak\n\
arc-retain-cycles\n\
arc-unsafe-retained-assign\n\
argument-outside-range\n\
argument-undefined-behaviour\n\
arm-interrupt-save-fp-no-vfp-unit\n\
arm-interrupt-vfp-clobber\n\
array-bounds\n\
array-bounds-pointer-arithmetic\n\
array-compare\n\
array-compare-cxx26\n\
array-parameter\n\
asm asm-operand-widths\n\
asm-operand-widths\n\
assign-enum\n\
assume\n\
atimport-in-framework-header\n\
atomic-access\n\
atomic-alignment\n\
atomic-hint-type\n\
atomic-implicit-seq-cst\n\
atomic-memory-ordering\n\
atomic-properties implicit-atomic-properties custom-atomic-properties\n\
atomic-property-with-user-defined-accessor\n\
attribute-alias\n\
attribute-packed-for-bitfield\n\
attribute-preprocessor-tokens\n\
attribute-statement\n\
attribute-warning\n\
attributes unknown-attributes ignored-attributes\n\
auto-decl-extensions\n\
auto-disable-vptr-sanitizer\n\
auto-storage-class\n\
auto-var-id\n\
availability\n\
avr-rtlib-linking-quirks\n\
backend-plugin\n\
backslash-newline-escape\n\
bad-function-cast\n\
binary-literal c++14-binary-literal c++98-c++11-compat-binary-literal\n\
bind-to-temporary-copy c++98-compat-bind-to-temporary-copy\n\
bit-int-extension\n\
bitfield-constant-conversion single-bit-bitfield-constant-conversion\n\
bitfield-enum-conversion\n\
bitfield-width\n\
bitwise-conditional-parentheses\n\
bitwise-instead-of-logical\n\
bitwise-op-parentheses\n\
block-capture-autoreleasing\n\
bool-conversion pointer-bool-conversion undefined-bool-conversion\n\
bool-operation bitwise-instead-of-logical\n\
bounds-safety-counted-by-elt-type-unknown-size\n\
braced-scalar-init\n\
branch-protection\n\
bridge-cast\n\
builtin-assume-aligned-alignment\n\
builtin-macro-redefined\n\
builtin-memcpy-chk-size\n\
builtin-requires-header\n\
c++-compat implicit-void-ptr-cast default-const-init implicit-int-enum-cast c++-hidden-decl c++-unterminated-string-initialization c++-keyword tentative-definition-compat jump-misses-init duplicate-decl-specifier\n\
c++-hidden-decl\n\
c++-keyword\n\
c++-unterminated-string-initialization\n\
c++11-compat c++11-narrowing c++11-compat-reserved-user-defined-literal c++11-compat-deprecated-writable-strings pre-c++14-compat pre-c++17-compat pre-c++20-compat pre-c++23-compat pre-c++26-compat\n\
c++11-compat-deprecated-writable-strings\n\
c++11-compat-pedantic c++11-compat pre-c++14-compat-pedantic pre-c++17-compat-pedantic pre-c++20-compat-pedantic pre-c++23-compat-pedantic pre-c++26-compat-pedantic\n\
c++11-compat-reserved-user-defined-literal\n\
c++11-extensions c++11-extra-semi c++11-inline-namespace c++11-long-long\n\
c++11-extra-semi\n\
c++11-inline-namespace\n\
c++11-long-long\n\
c++11-narrowing c++11-narrowing-const-reference\n\
c++11-narrowing-const-reference\n\
c++14-attribute-extensions\n\
c++14-binary-literal\n\
c++14-compat pre-c++17-compat pre-c++20-compat pre-c++23-compat pre-c++26-compat\n\
c++14-compat-pedantic c++14-compat pre-c++17-compat-pedantic pre-c++20-compat-pedantic pre-c++23-compat-pedantic pre-c++26-compat-pedantic\n\
c++14-extensions c++14-binary-literal c++14-attribute-extensions\n\
c++17-attribute-extensions\n\
c++17-compat deprecated-register deprecated-increment-bool c++17-compat-mangling pre-c++20-compat pre-c++23-compat pre-c++26-compat\n\
c++17-compat-mangling\n\
c++17-compat-pedantic c++17-compat pre-c++20-compat-pedantic pre-c++23-compat-pedantic pre-c++26-compat-pedantic\n\
c++17-extensions c++17-attribute-extensions\n\
c++20-attribute-extensions\n\
c++20-compat pre-c++23-compat pre-c++26-compat\n\
c++20-compat-pedantic c++20-compat pre-c++23-compat-pedantic pre-c++26-compat-pedantic\n\
c++20-designator\n\
c++20-extensions c++20-designator c++20-attribute-extensions variadic-macro-arguments-omitted\n\
c++23-attribute-extensions\n\
c++23-compat pre-c++26-compat\n\
c++23-extensions c++23-lambda-attributes\n\
c++23-lambda-attributes\n\
c++26-extensions\n\
c++2c-compat delete-incomplete\n\
c++2d-extensions\n\
c++98-c++11-compat-binary-literal\n\
c++98-compat c++98-compat-local-type-template-args c++98-compat-unnamed-type-template-args pre-c++14-compat pre-c++17-compat pre-c++20-compat pre-c++23-compat pre-c++26-compat\n\
c++98-compat-bind-to-temporary-copy\n\
c++98-compat-extra-semi\n\
c++98-compat-local-type-template-args\n\
c++98-compat-pedantic c++98-compat c++98-compat-bind-to-temporary-copy c++98-compat-extra-semi pre-c++14-compat-pedantic pre-c++17-compat-pedantic pre-c++20-compat-pedantic pre-c++23-compat-pedantic pre-c++26-compat-pedantic\n\
c++98-compat-unnamed-type-template-args\n\
c-attribute-extension\n\
c11-extensions\n\
c23-compat\n\
c23-extensions variadic-macro-arguments-omitted\n\
c2y-extensions static-in-inline counter-extension\n\
c99-compat\n\
c99-designator c++20-designator initializer-overrides reorder-init-list\n\
c99-extensions c99-designator\n\
call-graph-section-no-prototype\n\
call-to-pure-virtual-from-ctor-dtor\n\
called-once-parameter completion-handler\n\
cast-align\n\
cast-calling-convention\n\
cast-function-type cast-function-type-strict cast-function-type-mismatch\n\
cast-function-type-mismatch\n\
cast-function-type-strict\n\
cast-of-sel-type\n\
cast-qual\n\
cast-qual-unrelated\n\
cfi-unchecked-callee\n\
char-subscripts\n\
character-conversion\n\
clang-cl-pch\n\
clangir\n\
class-conversion\n\
class-varargs non-pod-varargs\n\
cmse-union-leak\n\
comma\n\
comment\n\
compare-distinct-pointer-types\n\
completion-handler\n\
complex-component-init\n\
compound-token-split compound-token-split-by-macro compound-token-split-by-space\n\
compound-token-split-by-macro\n\
compound-token-split-by-space\n\
conditional-type-mismatch\n\
conditional-uninitialized\n\
config-macros\n\
constant-conversion bitfield-constant-conversion objc-bool-constant-conversion\n\
constant-evaluated\n\
constant-logical-operand\n\
constexpr-not-const\n\
consumed\n\
conversion bool-conversion character-conversion constant-conversion enum-conversion bitfield-enum-conversion float-conversion int-conversion implicit-int-conversion implicit-float-conversion overflow-behavior-conversion literal-conversion non-literal-null-conversion null-conversion objc-literal-conversion sign-conversion string-conversion\n\
coro-non-aligned-allocation-function\n\
coro-type-aware-allocation-function\n\
coroutine coroutine-missing-unhandled-exception deprecated-coroutine always-inline-coroutine coro-non-aligned-allocation-function coro-type-aware-allocation-function\n\
coroutine-missing-unhandled-exception\n\
coroutines-unsupported-target\n\
counter-extension\n\
covered-switch-default\n\
cstring-format-directive\n\
ctad-maybe-unsupported\n\
ctu\n\
ctu-remarks\n\
cuda-compat\n\
custom-atomic-properties\n\
cxx-attribute-extension\n\
dangling dangling-assignment dangling-assignment-gsl dangling-capture dangling-field dangling-initializer-list dangling-gsl return-stack-address\n\
dangling-assignment\n\
dangling-assignment-gsl\n\
dangling-capture\n\
dangling-else\n\
dangling-field\n\
dangling-gsl\n\
dangling-initializer-list\n\
darwin-sdk-settings\n\
date-time\n\
dealloc-in-category\n\
debug-compression-unavailable\n\
declaration-after-statement\n\
decls-in-multiple-modules\n\
default-const-init default-const-init-field default-const-init-var default-const-init-unsafe\n\
default-const-init-field\n\
default-const-init-field-unsafe\n\
default-const-init-unsafe default-const-init-field-unsafe default-const-init-var-unsafe\n\
default-const-init-var\n\
default-const-init-var-unsafe\n\
defaulted-function-deleted\n\
delayed-template-parsing-in-cxx20\n\
delegating-ctor-cycles\n\
delete-abstract-non-virtual-dtor\n\
delete-incomplete\n\
delete-non-abstract-non-virtual-dtor\n\
delete-non-virtual-dtor delete-non-abstract-non-virtual-dtor delete-abstract-non-virtual-dtor\n\
delimited-escape-sequence-extension\n\
deprecate-lax-vec-conv-all\n\
deprecated deprecated-anon-enum-enum-conversion deprecated-array-compare deprecated-attributes deprecated-comma-subscript deprecated-copy deprecated-copy-with-dtor deprecated-declarations deprecated-dynamic-exception-spec deprecated-enum-compare deprecated-enum-compare-conditional deprecated-enum-enum-conversion deprecated-enum-float-conversion deprecated-builtins deprecated-increment-bool deprecated-literal-operator deprecated-pragma deprecated-register deprecated-ofast deprecated-this-capture deprecated-type deprecated-volatile deprecated-writable-strings deprecated-redundant-constexpr-static-def deprecated-missing-comma-variadic-parameter deprecated-octal-literals\n\
deprecated-altivec-src-compat\n\
deprecated-anon-enum-enum-conversion\n\
deprecated-array-compare\n\
deprecated-attributes\n\
deprecated-builtins\n\
deprecated-comma-subscript\n\
deprecated-copy deprecated-copy-with-user-provided-copy\n\
deprecated-copy-with-dtor deprecated-copy-with-user-provided-dtor\n\
deprecated-copy-with-user-provided-copy\n\
deprecated-copy-with-user-provided-dtor\n\
deprecated-coroutine\n\
deprecated-declarations deprecated-declarations-switch-case\n\
deprecated-declarations-switch-case\n\
deprecated-dynamic-exception-spec\n\
deprecated-enum-compare\n\
deprecated-enum-compare-conditional\n\
deprecated-enum-enum-conversion\n\
deprecated-enum-float-conversion\n\
deprecated-implementations\n\
deprecated-increment-bool\n\
deprecated-literal-operator\n\
deprecated-missing-comma-variadic-parameter\n\
deprecated-module-dot-map\n\
deprecated-non-prototype\n\
deprecated-objc-isa-usage\n\
deprecated-objc-pointer-introspection deprecated-objc-pointer-introspection-performSelector\n\
deprecated-objc-pointer-introspection-performSelector\n\
deprecated-octal-literals\n\
deprecated-ofast\n\
deprecated-pragma\n\
deprecated-redundant-constexpr-static-def\n\
deprecated-register\n\
deprecated-this-capture\n\
deprecated-type\n\
deprecated-volatile\n\
deprecated-writable-strings c++11-compat-deprecated-writable-strings\n\
direct-ivar-access\n\
disabled-macro-expansion\n\
distributed-object-modifiers\n\
division-by-zero\n\
dll-attribute-on-redeclaration\n\
dllexport-explicit-instantiation dllexport-explicit-instantiation-decl\n\
dllexport-explicit-instantiation-decl\n\
dllimport-static-field-def\n\
documentation documentation-html documentation-deprecated-sync\n\
documentation-deprecated-sync\n\
documentation-html\n\
documentation-pedantic documentation-unknown-command\n\
documentation-unknown-command\n\
dollar-in-identifier-extension\n\
double-promotion\n\
dtor-name\n\
dtor-typedef\n\
duplicate-decl-specifier\n\
duplicate-enum\n\
duplicate-header-ownership\n\
duplicate-interrupt-type\n\
duplicate-method-arg\n\
duplicate-method-match\n\
duplicate-protocol\n\
dxil-validation\n\
dynamic-class-memaccess\n\
dynamic-debugging\n\
dynamic-exception-spec deprecated-dynamic-exception-spec\n\
eager-load-cxx-named-modules\n\
elaborated-enum-base\n\
elaborated-enum-class\n\
embedded-directive\n\
empty-body\n\
empty-decomposition\n\
empty-init-stmt\n\
empty-translation-unit\n\
encode-type\n\
enum-compare enum-compare-switch deprecated-enum-compare\n\
enum-compare-conditional deprecated-enum-compare-conditional\n\
enum-compare-switch\n\
enum-compare-typo\n\
enum-conversion enum-enum-conversion implicit-enum-enum-cast enum-float-conversion enum-compare-conditional\n\
enum-enum-conversion deprecated-enum-enum-conversion\n\
enum-float-conversion deprecated-enum-float-conversion\n\
enum-too-large\n\
exceptions\n\
excess-initializers\n\
excessive-regsave\n\
exit-time-destructors\n\
expansion-stmt-missing-braces\n\
expansion-to-defined\n\
experimental-header-units\n\
experimental-option\n\
explicit-initialize-call\n\
explicit-ownership-type\n\
explicit-specialization-storage-class\n\
export-unnamed\n\
ext-cxx-type-aware-allocators\n\
extern-c-compat\n\
extern-initializer\n\
extra deprecated-copy missing-field-initializers ignored-qualifiers initializer-overrides semicolon-before-method-body missing-method-return-type sign-compare unused-parameter unused-but-set-parameter null-pointer-arithmetic null-pointer-subtraction empty-init-stmt string-concatenation fuse-ld-path cast-function-type-mismatch unterminated-string-initialization unnecessary-virtual-specifier\n\
extra-qualification\n\
extra-semi c++98-compat-extra-semi c++11-extra-semi\n\
extra-semi-stmt empty-init-stmt\n\
extra-tokens\n\
extractapi-misuse\n\
extraneous-template-head\n\
final-dtor-non-final-class\n\
final-macro\n\
fixed-point-overflow\n\
flag-enum\n\
flexible-array-extensions\n\
float-conversion float-overflow-conversion float-zero-conversion\n\
float-equal\n\
float-overflow-conversion\n\
float-zero-conversion\n\
for-loop-analysis\n\
format format-extra-args format-zero-length nonnull format-security format-y2k format-invalid-specifier format-insufficient-args format-overflow format-truncation\n\
format-extra-args\n\
format-insufficient-args\n\
format-invalid-specifier\n\
format-non-iso\n\
format-nonliteral missing-format-attribute\n\
format-overflow format-overflow-non-kprintf\n\
format-overflow-non-kprintf\n\
format-pedantic\n\
format-security\n\
format-signedness\n\
format-truncation format-truncation-non-kprintf\n\
format-truncation-non-kprintf\n\
format-type-confusion\n\
format-y2k\n\
format-zero-length\n\
format=2 format-nonliteral format-security format-y2k\n\
fortify-source format-overflow format-truncation\n\
four-char-constants\n\
frame-address\n\
frame-larger-than\n\
framework-include-private-from-public\n\
free-nonheap-object\n\
friend-enum\n\
function-def-in-objc-container\n\
function-effect-redeclarations\n\
function-effects\n\
function-multiversion target-clones-mixed-specifiers\n\
fuse-ld-path\n\
future-attribute-extensions c++14-attribute-extensions c++17-attribute-extensions c++20-attribute-extensions c++23-attribute-extensions\n\
future-compat\n\
gcc-compat\n\
gcc-install-dir-libstdcxx\n\
global-constructors\n\
global-isel\n\
gnu gnu-alignof-expression gnu-anonymous-struct gnu-auto-type gnu-binary-literal gnu-case-range gnu-complex-integer gnu-compound-literal-initializer gnu-conditional-omitted-operand gnu-designator gnu-empty-struct vla-extension gnu-flexible-array-initializer gnu-flexible-array-union-member gnu-folding-constant gnu-imaginary-constant gnu-include-next gnu-label-as-value gnu-line-marker gnu-null-pointer-arithmetic gnu-offsetof-extensions gnu-pointer-arith redeclared-class-member gnu-redeclared-enum gnu-statement-expression gnu-static-float-init gnu-string-literal-operator-template gnu-union-cast gnu-variable-sized-type-not-at-end zero-length-array gnu-zero-line-directive gnu-zero-variadic-macro-arguments\n\
gnu-alignof-expression\n\
gnu-anonymous-struct\n\
gnu-array-member-paren-init\n\
gnu-auto-type\n\
gnu-binary-literal\n\
gnu-case-range\n\
gnu-complex-integer\n\
gnu-compound-literal-initializer\n\
gnu-conditional-omitted-operand\n\
gnu-designator\n\
gnu-empty-struct\n\
gnu-flexible-array-initializer\n\
gnu-flexible-array-union-member\n\
gnu-folding-constant\n\
gnu-imaginary-constant\n\
gnu-include-next\n\
gnu-inline-cpp-without-extern\n\
gnu-label-as-value\n\
gnu-line-marker\n\
gnu-null-pointer-arithmetic\n\
gnu-offsetof-extensions\n\
gnu-pointer-arith\n\
gnu-redeclared-enum\n\
gnu-statement-expression gnu-statement-expression-from-macro-expansion\n\
gnu-statement-expression-from-macro-expansion\n\
gnu-static-float-init\n\
gnu-string-literal-operator-template\n\
gnu-union-cast\n\
gnu-variable-sized-type-not-at-end\n\
gnu-zero-line-directive\n\
gnu-zero-variadic-macro-arguments variadic-macro-arguments-omitted\n\
gpu-maybe-wrong-side\n\
groupshared-initializer\n\
header-guard\n\
header-hygiene\n\
hexagon-reserved-caller-saved\n\
higher-precision-for-complex-division\n\
hip-deprecated-builtins\n\
hip-omp-target-directives\n\
hip-only\n\
hlsl-202y-extensions\n\
hlsl-availability\n\
hlsl-dxc-compatability\n\
hlsl-explicit-binding\n\
hlsl-extensions hlsl-202y-extensions\n\
hlsl-implicit-binding\n\
idiomatic-parentheses\n\
ignored-attributes dllexport-explicit-instantiation\n\
ignored-availability-without-sdk-settings\n\
ignored-base-class-qualifiers ignored-qualifiers\n\
ignored-gch\n\
ignored-optimization-argument\n\
ignored-pragma-intrinsic\n\
ignored-pragma-optimize\n\
ignored-pragmas ignored-pragma-intrinsic ignored-pragma-optimize\n\
ignored-qualifiers ignored-reference-qualifiers\n\
ignored-reference-qualifiers\n\
implicit implicit-function-declaration implicit-int\n\
implicit-atomic-properties\n\
implicit-const-int-float-conversion\n\
implicit-conversion-floating-point-to-bool\n\
implicit-enum-enum-cast\n\
implicit-exception-spec-mismatch\n\
implicit-fallthrough implicit-fallthrough-per-function\n\
implicit-fallthrough-per-function\n\
implicit-fixed-point-conversion\n\
implicit-float-conversion implicit-int-float-conversion objc-signed-char-bool-implicit-float-conversion\n\
implicit-function-declaration\n\
implicit-int\n\
implicit-int-conversion shorten-64-to-32 objc-signed-char-bool-implicit-int-conversion implicit-int-conversion-on-negation\n\
implicit-int-conversion-on-negation\n\
implicit-int-enum-cast implicit-enum-enum-cast\n\
implicit-int-float-conversion implicit-const-int-float-conversion\n\
implicit-overflow-behavior-conversion implicit-overflow-behavior-conversion-assignment implicit-overflow-behavior-conversion-assignment-pedantic implicit-overflow-behavior-conversion-pedantic implicit-overflow-behavior-conversion-function-boundary implicit-overflow-behavior-conversion-function-boundary-pedantic\n\
implicit-overflow-behavior-conversion-assignment\n\
implicit-overflow-behavior-conversion-assignment-pedantic\n\
implicit-overflow-behavior-conversion-function-boundary\n\
implicit-overflow-behavior-conversion-function-boundary-pedantic\n\
implicit-overflow-behavior-conversion-pedantic\n\
implicit-retain-self\n\
implicit-void-ptr-cast\n\
implicitly-unsigned-literal\n\
import-implementation-partition-unit-in-interface-unit\n\
import-preprocessor-directive-pedantic\n\
inaccessible-base\n\
include-angled-in-module-purview\n\
include-next-absolute-path\n\
include-next-outside-header\n\
incompatible-exception-spec\n\
incompatible-function-pointer-types\n\
incompatible-function-pointer-types-strict\n\
incompatible-library-redeclaration\n\
incompatible-ms-pragma-section\n\
incompatible-ms-struct\n\
incompatible-pointer-types incompatible-pointer-types-discards-qualifiers incompatible-pointer-types-discards-overflow-behavior incompatible-function-pointer-types\n\
incompatible-pointer-types-discards-overflow-behavior\n\
incompatible-pointer-types-discards-qualifiers\n\
incompatible-property-type\n\
incompatible-sysroot\n\
incomplete-framework-module-declaration\n\
incomplete-implementation\n\
incomplete-module incomplete-umbrella non-modular-include-in-module\n\
incomplete-setjmp-declaration\n\
incomplete-umbrella\n\
inconsistent-dllimport\n\
inconsistent-missing-destructor-override\n\
inconsistent-missing-override\n\
increment-bool deprecated-increment-bool\n\
infinite-recursion\n\
init-priority-reserved\n\
initializer-overrides\n\
injected-class-name\n\
inline-asm\n\
inline-namespace-reopened-noninline\n\
inline-new-delete\n\
installapi-violation\n\
instantiation-after-specialization\n\
int-conversion\n\
int-in-bool-context\n\
int-to-pointer-cast int-to-void-pointer-cast\n\
int-to-void-pointer-cast\n\
integer-overflow\n\
invalid-command-line-argument ignored-optimization-argument\n\
invalid-constexpr\n\
invalid-feature-combination\n\
invalid-gnu-asm-cast\n\
invalid-iboutlet\n\
invalid-initializer-from-system-header\n\
invalid-ios-deployment-target\n\
invalid-no-builtin-names\n\
invalid-noreturn\n\
invalid-offsetof\n\
invalid-or-nonexistent-directory\n\
invalid-partial-specialization\n\
invalid-pp-token\n\
invalid-source-encoding\n\
invalid-specialization\n\
invalid-static-assert-message\n\
invalid-token-paste\n\
invalid-unevaluated-string\n\
invalid-utf8\n\
invalid-version-availability\n\
jump-misses-init\n\
jump-seh-finally\n\
keyword-compat\n\
keyword-macro\n\
knr-promoted-parameter\n\
language-extension-token\n\
large-by-value-copy\n\
legacy-constant-register-binding\n\
liblto\n\
lifetime-safety lifetime-safety-permissive lifetime-safety-strict\n\
lifetime-safety-all lifetime-safety lifetime-safety-suggestions lifetime-safety-validations\n\
lifetime-safety-annotation-placement lifetime-safety-inapplicable-lifetimebound lifetime-safety-misplaced-lifetimebound\n\
lifetime-safety-annotation-verification lifetime-safety-noescape lifetime-safety-lifetimebound-violation\n\
lifetime-safety-cross-tu-constructor-suggestions\n\
lifetime-safety-cross-tu-misplaced-lifetimebound\n\
lifetime-safety-cross-tu-suggestions lifetime-safety-cross-tu-constructor-suggestions\n\
lifetime-safety-dangling-field\n\
lifetime-safety-dangling-field-lambda-capture\n\
lifetime-safety-dangling-field-moved\n\
lifetime-safety-dangling-global\n\
lifetime-safety-dangling-global-in-main\n\
lifetime-safety-dangling-global-moved\n\
lifetime-safety-inapplicable-lifetimebound\n\
lifetime-safety-intra-tu-constructor-suggestions\n\
lifetime-safety-intra-tu-misplaced-lifetimebound\n\
lifetime-safety-intra-tu-suggestions lifetime-safety-intra-tu-constructor-suggestions\n\
lifetime-safety-invalidation\n\
lifetime-safety-lifetimebound-violation\n\
lifetime-safety-misplaced-lifetimebound lifetime-safety-cross-tu-misplaced-lifetimebound lifetime-safety-intra-tu-misplaced-lifetimebound\n\
lifetime-safety-noescape\n\
lifetime-safety-permissive lifetime-safety-use-after-scope lifetime-safety-return-stack-addr lifetime-safety-dangling-field lifetime-safety-dangling-global lifetime-safety-use-after-free\n\
lifetime-safety-return-stack-addr\n\
lifetime-safety-return-stack-addr-moved\n\
lifetime-safety-strict lifetime-safety-permissive lifetime-safety-use-after-scope-moved lifetime-safety-return-stack-addr-moved lifetime-safety-dangling-field-moved lifetime-safety-dangling-field-lambda-capture lifetime-safety-dangling-global-moved lifetime-safety-dangling-global-in-main lifetime-safety-invalidation\n\
lifetime-safety-suggestions lifetime-safety-cross-tu-suggestions lifetime-safety-intra-tu-suggestions\n\
lifetime-safety-use-after-free\n\
lifetime-safety-use-after-scope\n\
lifetime-safety-use-after-scope-moved\n\
lifetime-safety-validations lifetime-safety-annotation-placement lifetime-safety-annotation-verification\n\
linker-warnings\n\
literal-conversion\n\
literal-range\n\
local-type-template-args c++98-compat-local-type-template-args\n\
logical-not-parentheses\n\
logical-op-parentheses\n\
long-long c++11-long-long\n\
loop-analysis for-loop-analysis range-loop-analysis\n\
macro-redefined\n\
main\n\
main-attached-to-named-module\n\
main-return-type\n\
malformed-warning-check\n\
many-braces-around-scalar-init\n\
math-errno-enabled-with-veclib\n\
mathematical-notation-identifier-extension\n\
matrix-conversion\n\
max-tokens\n\
max-unsigned-zero\n\
memset-transposed-args\n\
memsize-comparison\n\
method-signatures\n\
microsoft microsoft-charize microsoft-drectve-section microsoft-include microsoft-cpp-macro microsoft-fixed-enum microsoft-sealed microsoft-abstract microsoft-unqualified-friend microsoft-exception-spec microsoft-using-decl microsoft-mutable-reference microsoft-pure-definition microsoft-union-member-reference microsoft-explicit-constructor-call microsoft-enum-value microsoft-default-arg-redefinition microsoft-template microsoft-redeclare-static microsoft-enum-forward-reference microsoft-goto microsoft-flexible-array microsoft-extra-qualification microsoft-cast microsoft-const-init microsoft-void-pseudo-dtor microsoft-anon-tag microsoft-comment-paste microsoft-end-of-file microsoft-init-from-predefined microsoft-string-literal-from-predefined inconsistent-dllimport microsoft-inline-on-non-function microsoft-empty-enum\n\
microsoft-abstract\n\
microsoft-anon-tag\n\
microsoft-cast\n\
microsoft-charize\n\
microsoft-comment-paste\n\
microsoft-const-init\n\
microsoft-cpp-macro\n\
microsoft-default-arg-redefinition\n\
microsoft-drectve-section\n\
microsoft-empty-enum\n\
microsoft-end-of-file\n\
microsoft-enum-forward-reference\n\
microsoft-enum-value\n\
microsoft-exception-spec\n\
microsoft-exists\n\
microsoft-explicit-constructor-call\n\
microsoft-extra-qualification\n\
microsoft-fixed-enum\n\
microsoft-flexible-array\n\
microsoft-goto\n\
microsoft-inaccessible-base\n\
microsoft-include\n\
microsoft-init-from-predefined\n\
microsoft-inline-on-non-function\n\
microsoft-mutable-reference\n\
microsoft-pure-definition\n\
microsoft-redeclare-static\n\
microsoft-sealed\n\
microsoft-string-literal-from-predefined\n\
microsoft-template microsoft-template-shadow\n\
microsoft-template-shadow\n\
microsoft-union-member-reference\n\
microsoft-unqualified-friend\n\
microsoft-using-decl\n\
microsoft-void-pseudo-dtor\n\
misexpect\n\
misleading-indentation\n\
mismatched-new-delete\n\
mismatched-parameter-types\n\
mismatched-return-types\n\
mismatched-tags\n\
missing-braces\n\
missing-constinit\n\
missing-declarations\n\
missing-designated-field-initializers\n\
missing-exception-spec\n\
missing-field-initializers missing-designated-field-initializers\n\
missing-format-attribute\n\
missing-include-dirs\n\
missing-method-return-type\n\
missing-multilib\n\
missing-noescape\n\
missing-noreturn\n\
missing-prototype-for-cc\n\
missing-prototypes\n\
missing-selector-name\n\
missing-sysroot\n\
missing-template-arg-list-after-template-kw\n\
missing-variable-declarations\n\
misspelled-assumption\n\
mix-packoffset\n\
mmap-deprecated-symlink-to-modular-header\n\
module-build\n\
module-conflict\n\
module-file-config-mismatch\n\
module-file-extension\n\
module-file-mapping-mismatch\n\
module-import\n\
module-import-in-extern-c\n\
module-include-translation\n\
module-lock\n\
module-map\n\
module-map-path-outside-directory\n\
module-validation\n\
modules-ambiguous-internal-linkage\n\
modules-driver\n\
modules-import-nested-redundant\n\
most array-parameter bool-operation char-subscripts comment delete-non-virtual-dtor format for-loop-analysis frame-address implicit infinite-recursion int-in-bool-context mismatched-tags missing-braces move multichar range-loop-construct reorder return-type self-assign self-move sizeof-array-argument sizeof-array-decay string-plus-int tautological-compare trigraphs uninitialized unknown-pragmas unused volatile-register-var objc-missing-super-calls objc-designated-initializers objc-flexible-array overloaded-virtual private-extern cast-of-sel-type extern-c-compat user-defined-warnings\n\
move pessimizing-move redundant-move return-std-move self-move\n\
ms-bitfield-padding\n\
msvc-not-found\n\
multi-gpu\n\
multichar\n\
multilib-not-found\n\
multiple-move-vbase\n\
nan-infinity-disabled\n\
nested-anon-types\n\
new-returns-null\n\
newline-eof\n\
noderef\n\
noexcept-type c++17-compat-mangling\n\
non-c-typedef-for-linkage\n\
non-gcc sign-compare conversion literal-range\n\
non-literal-null-conversion\n\
non-modular-include-in-framework-module\n\
non-modular-include-in-module non-modular-include-in-framework-module\n\
non-pod-varargs\n\
non-power-of-two-alignment\n\
non-virtual-dtor\n\
nonnull\n\
nonportable-include-path nonportable-include-path-separator\n\
nonportable-include-path-separator\n\
nonportable-private-apinotes-path\n\
nonportable-private-system-apinotes-path\n\
nonportable-sycl\n\
nonportable-system-include-path\n\
nonportable-vector-initialization\n\
nontrivial-memaccess nontrivial-memcall\n\
nontrivial-memcall\n\
nrvo\n\
nsconsumed-mismatch\n\
nsreturns-mismatch\n\
null-arithmetic\n\
null-character\n\
null-conversion\n\
null-dereference\n\
null-pointer-arithmetic gnu-null-pointer-arithmetic\n\
null-pointer-subtraction\n\
nullability\n\
nullability-completeness nullability-completeness-on-arrays\n\
nullability-completeness-on-arrays\n\
nullability-declspec\n\
nullability-extension\n\
nullability-inferred-on-nested-type\n\
nullable-to-nonnull-conversion\n\
nvcc-compat\n\
objc-autosynthesis-property-ivar-name-match\n\
objc-bool-constant-conversion\n\
objc-boxing\n\
objc-circular-container\n\
objc-cocoa-api objc-redundant-api-use\n\
objc-designated-initializers\n\
objc-dictionary-duplicate-keys\n\
objc-duplicate-category-definition\n\
objc-flexible-array\n\
objc-forward-class-redefinition\n\
objc-interface-ivars\n\
objc-literal-compare objc-string-compare\n\
objc-literal-conversion\n\
objc-macro-redefinition\n\
objc-messaging-id\n\
objc-method-access\n\
objc-missing-property-synthesis\n\
objc-missing-super-calls\n\
objc-multiple-method-names\n\
objc-noncopy-retain-block-property\n\
objc-nonunified-exceptions\n\
objc-property-assign-on-object-type\n\
objc-property-implementation\n\
objc-property-implicit-mismatch\n\
objc-property-matches-cocoa-ownership-rule\n\
objc-property-no-attribute\n\
objc-property-synthesis\n\
objc-protocol-method-implementation\n\
objc-protocol-property-synthesis\n\
objc-protocol-qualifiers\n\
objc-readonly-with-setter-property\n\
objc-redundant-api-use objc-redundant-literal-use\n\
objc-redundant-literal-use\n\
objc-root-class\n\
objc-signed-char-bool objc-signed-char-bool-implicit-int-conversion objc-signed-char-bool-implicit-float-conversion objc-bool-constant-conversion tautological-objc-bool-compare\n\
objc-signed-char-bool-implicit-float-conversion\n\
objc-signed-char-bool-implicit-int-conversion\n\
objc-string-compare\n\
objc-string-concatenation\n\
objc-unsafe-perform-selector\n\
octal-prefix-extension\n\
odr\n\
old-style-cast\n\
openacc source-uses-openacc\n\
openacc-cache-var-inside-loop\n\
openacc-confusing-routine-name\n\
openacc-deprecated-clause-alias\n\
openacc-extension\n\
openacc-self-if-potential-conflict\n\
openacc-var-lacks-operation\n\
openacc-var-non-const-array\n\
opencl-unsupported-rgba\n\
openmp source-uses-openmp openmp-clauses openmp-loop-form openmp-target openmp-mapping openmp-51-extensions openmp-extensions openmp-target-exception openmp-future\n\
openmp-51-extensions\n\
openmp-clauses\n\
openmp-extensions\n\
openmp-future\n\
openmp-loop-form\n\
openmp-mapping\n\
openmp-target openmp-mapping\n\
openmp-target-exception\n\
option-ignored\n\
ordered-compare-function-pointers\n\
out-of-line-declaration\n\
out-of-scope-function\n\
over-aligned\n\
overflow-behavior-attribute-ignored\n\
overflow-behavior-conversion implicit-overflow-behavior-conversion\n\
overlength-strings\n\
overloaded-shift-op-parentheses\n\
overloaded-virtual\n\
override-module\n\
overriding-complex-range\n\
overriding-deployment-version\n\
overriding-method-mismatch\n\
overriding-option\n\
packed packed-non-pod\n\
packed-non-pod\n\
padded padded-bitfield\n\
padded-bitfield\n\
parentheses logical-op-parentheses logical-not-parentheses bitwise-conditional-parentheses bitwise-op-parentheses shift-op-parentheses overloaded-shift-op-parentheses parentheses-equality dangling-else\n\
parentheses-equality\n\
pass\n\
pass-analysis\n\
pass-failed\n\
pass-missed\n\
pch-date-time\n\
pch-vfs-diff\n\
pedantic\n\
pedantic-core-features\n\
pedantic-macros deprecated-pragma macro-redefined builtin-macro-redefined restrict-expansion final-macro\n\
perf-constraint-implies-noexcept\n\
pessimizing-move\n\
pgo-coverage\n\
pointer-arith gnu-pointer-arith\n\
pointer-bool-conversion\n\
pointer-compare\n\
pointer-integer-compare\n\
pointer-sign\n\
pointer-to-enum-cast void-pointer-to-enum-cast\n\
pointer-to-int-cast pointer-to-enum-cast void-pointer-to-int-cast\n\
pointer-type-mismatch\n\
poison-system-directories\n\
potentially-direct-selector\n\
potentially-evaluated-expression\n\
pragma-clang-attribute\n\
pragma-once-outside-header\n\
pragma-pack pragma-pack-suspicious-include\n\
pragma-pack-suspicious-include\n\
pragma-system-header-outside-header\n\
pragmas unknown-pragmas ignored-pragmas pragma-clang-attribute pragma-pack\n\
pre-c++14-compat\n\
pre-c++14-compat-pedantic pre-c++14-compat c++98-c++11-compat-binary-literal\n\
pre-c++17-compat\n\
pre-c++17-compat-pedantic pre-c++17-compat\n\
pre-c++20-compat\n\
pre-c++20-compat-pedantic pre-c++20-compat\n\
pre-c++23-compat\n\
pre-c++23-compat-pedantic pre-c++23-compat\n\
pre-c++26-compat\n\
pre-c++26-compat-pedantic pre-c++26-compat\n\
pre-c++2d-compat\n\
pre-c++2d-compat-pedantic pre-c++2d-compat\n\
pre-c11-compat\n\
pre-c11-compat-pedantic pre-c11-compat\n\
pre-c23-compat variadic-macro-arguments-omitted\n\
pre-c23-compat-pedantic pre-c23-compat\n\
pre-c2y-compat\n\
pre-c2y-compat-pedantic pre-c2y-compat\n\
pre-openmp-51-compat\n\
predefined-identifier-outside-function\n\
preferred-type-bitfield-enum-conversion\n\
preferred-type-bitfield-width\n\
private-extern\n\
private-header\n\
private-module\n\
profile-instr-missing\n\
profile-instr-out-of-date\n\
profile-instr-unprofiled\n\
property-access-dot-syntax\n\
property-attribute-mismatch\n\
protocol\n\
protocol-property-synthesis-ambiguity\n\
psabi\n\
ptrauth-null-pointers\n\
qualified-void-return-type\n\
quoted-include-in-framework-header\n\
range-loop-analysis range-loop-construct range-loop-bind-reference\n\
range-loop-bind-reference\n\
range-loop-construct\n\
read-only-types\n\
readonly-iboutlet-property\n\
receiver-expr\n\
receiver-forward-class\n\
redeclared-class-member\n\
reduced-bmi-output-overrided\n\
redundant-attribute\n\
redundant-consteval-if\n\
redundant-move\n\
redundant-parens\n\
reference-tu-local-entity-in-other-tu\n\
register deprecated-register\n\
reinterpret-base-class\n\
relaxed-constant-fold\n\
remark-backend-plugin\n\
remark-clangir\n\
reorder reorder-ctor reorder-init-list\n\
reorder-ctor\n\
reorder-init-list\n\
requires-super-attribute\n\
reserved-attribute-identifier\n\
reserved-id-macro reserved-macro-identifier\n\
reserved-identifier reserved-macro-identifier reserved-module-identifier user-defined-literals reserved-attribute-identifier\n\
reserved-macro-identifier\n\
reserved-module-identifier\n\
reserved-user-defined-literal c++11-compat-reserved-user-defined-literal\n\
restrict-expansion\n\
retained-language-linkage\n\
return-mismatch\n\
return-stack-address\n\
return-std-move\n\
return-type return-type-c-linkage return-mismatch\n\
return-type-c-linkage\n\
rewrite-not-bool\n\
riscv-be-experimental\n\
round-trip-cc1-args\n\
rtti\n\
sanitize-address\n\
sarif-format-unstable\n\
scalable-static-analysis-framework\n\
search-path-usage\n\
section\n\
selector selector-type-mismatch\n\
selector-type-mismatch\n\
self-assign self-assign-overloaded self-assign-field\n\
self-assign-field\n\
self-assign-overloaded\n\
self-move\n\
semicolon-before-method-body\n\
sentinel\n\
serialized-diagnostics\n\
shadow shadow-field-in-constructor-modified shadow-ivar\n\
shadow-all shadow shadow-field-in-constructor shadow-uncaptured-local shadow-field\n\
shadow-field\n\
shadow-field-in-constructor shadow-field-in-constructor-modified\n\
shadow-field-in-constructor-modified\n\
shadow-header\n\
shadow-ivar\n\
shadow-uncaptured-local\n\
shift-bool\n\
shift-count-negative\n\
shift-count-overflow\n\
shift-negative-value\n\
shift-op-parentheses\n\
shift-overflow\n\
shift-sign-overflow\n\
shorten-64-to-32\n\
sign-compare\n\
sign-conversion\n\
signed-enum-bitfield\n\
signed-unsigned-wchar\n\
simdlen\n\
single-bit-bitfield-constant-conversion\n\
sizeof-array-argument\n\
sizeof-array-decay\n\
sizeof-array-div\n\
sizeof-pointer-div\n\
sizeof-pointer-memaccess\n\
slash-u-filename\n\
slh-asm-goto\n\
sloc-usage\n\
sometimes-uninitialized\n\
source-mgr\n\
source-uses-openacc\n\
source-uses-openmp\n\
spir-compat\n\
spirv-validation\n\
stack-exhausted\n\
stack-protector\n\
static-float-init gnu-static-float-init\n\
static-in-inline\n\
static-inline-explicit-instantiation\n\
static-local-in-inline\n\
static-self-init\n\
stdlibcxx-not-found\n\
strict-potentially-direct-selector potentially-direct-selector\n\
strict-primary-template-shadow\n\
strict-prototypes deprecated-non-prototype\n\
strict-selector-match\n\
string-compare\n\
string-concatenation\n\
string-conversion\n\
string-plus-char\n\
string-plus-int\n\
stringop-overread\n\
strlcpy-strlcat-size\n\
strncat-size\n\
suggest-destructor-override\n\
suggest-override\n\
super-class-method-mismatch\n\
suspicious-bzero\n\
suspicious-memaccess sizeof-pointer-memaccess dynamic-class-memaccess nontrivial-memaccess memset-transposed-args suspicious-bzero\n\
swift-name-attribute\n\
switch\n\
switch-bool\n\
switch-default\n\
switch-enum\n\
sync-alignment\n\
sync-fetch-and-nand-semantics-changed\n\
target-clones-mixed-specifiers\n\
tautological-bitwise-compare\n\
tautological-compare tautological-constant-compare tautological-pointer-compare tautological-overlap-compare tautological-bitwise-compare tautological-undefined-compare tautological-objc-bool-compare tautological-negation-compare\n\
tautological-constant-compare tautological-constant-out-of-range-compare\n\
tautological-constant-in-range-compare type-limits tautological-value-range-compare\n\
tautological-constant-out-of-range-compare\n\
tautological-negation-compare\n\
tautological-objc-bool-compare\n\
tautological-overlap-compare\n\
tautological-pointer-compare\n\
tautological-type-limit-compare\n\
tautological-undefined-compare\n\
tautological-unsigned-char-zero-compare\n\
tautological-unsigned-enum-zero-compare\n\
tautological-unsigned-zero-compare\n\
tautological-value-range-compare\n\
tcb-enforcement\n\
template-in-declaration-name\n\
tentative-definition-array\n\
tentative-definition-compat\n\
tentative-definition-incomplete-type\n\
thread-safety thread-safety-attributes thread-safety-analysis thread-safety-precise thread-safety-reference\n\
thread-safety-analysis\n\
thread-safety-attributes\n\
thread-safety-beta\n\
thread-safety-negative\n\
thread-safety-pointer\n\
thread-safety-precise\n\
thread-safety-reference thread-safety-reference-return\n\
thread-safety-reference-return\n\
thread-safety-verbose\n\
trigraphs\n\
trivial-auto-var-init\n\
type-limits tautological-type-limit-compare tautological-unsigned-zero-compare tautological-unsigned-char-zero-compare tautological-unsigned-enum-zero-compare\n\
type-safety\n\
typedef-redefinition\n\
typename-missing\n\
unable-to-open-stats-file\n\
unaligned-access\n\
unaligned-qualifier-implicit-cast\n\
unavailable-declarations\n\
undeclared-selector\n\
undef undef-true\n\
undef-prefix\n\
undef-true\n\
undefined-arm-za\n\
undefined-arm-zt0\n\
undefined-bool-conversion\n\
undefined-func-template\n\
undefined-inline\n\
undefined-internal\n\
undefined-internal-type\n\
undefined-reinterpret-cast\n\
undefined-var-template\n\
underaligned-exception-object\n\
underlying-atomic-qualifier-ignored\n\
underlying-cv-qualifier-ignored\n\
unevaluated-expression potentially-evaluated-expression\n\
unguarded-availability unguarded-availability-new\n\
unguarded-availability-new\n\
unicode\n\
unicode-homoglyph\n\
unicode-whitespace\n\
unicode-zero-width\n\
uninitialized sometimes-uninitialized static-self-init uninitialized-const-reference uninitialized-const-pointer\n\
uninitialized-const-pointer\n\
uninitialized-const-reference\n\
uninitialized-explicit-init\n\
unique-object-duplication\n\
unknown-acc-extension-clause\n\
unknown-argument\n\
unknown-assumption\n\
unknown-attributes\n\
unknown-cuda-version\n\
unknown-directives\n\
unknown-escape-sequence\n\
unknown-pragmas\n\
unknown-sanitizers\n\
unknown-warning-option\n\
unnamed-type-template-args c++98-compat-unnamed-type-template-args\n\
unnecessary-virtual-specifier\n\
unneeded-internal-declaration\n\
unneeded-member-function\n\
unqualified-std-cast-call\n\
unreachable-code unreachable-code-loop-increment unreachable-code-fallthrough unreachable-code-generic-assoc\n\
unreachable-code-aggressive unreachable-code unreachable-code-break unreachable-code-return\n\
unreachable-code-break\n\
unreachable-code-fallthrough\n\
unreachable-code-generic-assoc\n\
unreachable-code-loop-increment\n\
unreachable-code-return\n\
unsafe-buffer-usage unsafe-buffer-usage-in-container unsafe-buffer-usage-in-libc-call unsafe-buffer-usage-in-unique-ptr-array-access unsafe-buffer-usage-in-static-sized-array unsafe-buffer-usage-in-format-attr-call unsafe-buffer-usage-in-string-view\n\
unsafe-buffer-usage-in-container\n\
unsafe-buffer-usage-in-format-attr-call\n\
unsafe-buffer-usage-in-libc-call\n\
unsafe-buffer-usage-in-static-sized-array\n\
unsafe-buffer-usage-in-string-view\n\
unsafe-buffer-usage-in-unique-ptr-array-access\n\
unsequenced\n\
unsupported-abi\n\
unsupported-abs\n\
unsupported-availability-guard\n\
unsupported-cb\n\
unsupported-dll-base-class-template\n\
unsupported-floating-point-opt\n\
unsupported-gpopt\n\
unsupported-nan\n\
unsupported-target-opt\n\
unsupported-visibility\n\
unterminated-string-initialization\n\
unusable-partial-specialization\n\
unused unused-argument unused-function unused-label unused-private-field unused-lambda-capture unused-local-typedef unused-value unused-variable unused-but-set-variable unused-property-ivar\n\
unused-argument\n\
unused-but-set-global\n\
unused-but-set-parameter\n\
unused-but-set-variable unused-but-set-global\n\
unused-command-line-argument\n\
unused-comparison\n\
unused-const-variable\n\
unused-exception-parameter\n\
unused-function unneeded-internal-declaration\n\
unused-getter-return-value\n\
unused-label\n\
unused-lambda-capture\n\
unused-local-typedef\n\
unused-macros\n\
unused-member-function unneeded-member-function\n\
unused-parameter\n\
unused-private-field\n\
unused-property-ivar\n\
unused-result\n\
unused-template unneeded-internal-declaration\n\
unused-value unused-comparison unused-result unevaluated-expression\n\
unused-variable unused-const-variable\n\
unused-volatile-lvalue\n\
used-but-marked-unused\n\
user-defined-literals\n\
user-defined-warnings\n\
varargs\n\
variadic-macro-arguments-omitted\n\
variadic-macros\n\
vec-elem-size\n\
vector-conversion\n\
vexing-parse\n\
visibility\n\
vla vla-extension\n\
vla-cxx-extension vla-extension-static-assert\n\
vla-extension vla-cxx-extension\n\
vla-extension-static-assert\n\
void-pointer-to-enum-cast\n\
void-pointer-to-int-cast void-pointer-to-enum-cast\n\
void-ptr-dereference\n\
volatile-register-var\n\
wasm-exception-spec\n\
weak-template-vtables\n\
weak-vtables\n\
writable-strings deprecated-writable-strings\n\
write-strings writable-strings\n\
xor-used-as-pow\n\
zero-as-null-pointer-constant\n\
zero-length-array\n\
";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum CSeverity {
    Ignored,
    Note,
    Remark,
    Warning,
    Error,
    Fatal,
}

impl CSeverity {
    pub const fn label(self) -> &'static str {
        match self {
            CSeverity::Ignored => "ignored",
            CSeverity::Note => "note",
            CSeverity::Remark => "remark",
            CSeverity::Warning => "warning",
            CSeverity::Error => "error",
            CSeverity::Fatal => "fatal error",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CDiagnostic {
    pub severity: CSeverity,
    pub message: String,
    pub option: Option<String>,
    pub place: Option<Place>,
    pub presumed: Option<PresumedPlace>,
    pub include_stack: Vec<PresumedPlace>,
    pub kind: Message,
}

#[derive(Clone, Debug, Default)]
pub struct WarningOptions {
    pub flags: Vec<String>,
    pub pedantic: bool,
    pub pedantic_errors: bool,
    pub suppress_all: bool,
    pub everything: bool,
    pub warnings_as_errors: bool,
    pub fatal_errors: bool,
    pub show_in_system_headers: bool,
    pub error_limit: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct GroupMapping {
    severity: Option<CSeverity>,
    no_werror: bool,
    user: bool,
}

fn group_tree() -> &'static BTreeMap<&'static str, Vec<&'static str>> {
    static TREE: std::sync::OnceLock<BTreeMap<&'static str, Vec<&'static str>>> =
        std::sync::OnceLock::new();
    TREE.get_or_init(|| {
        let mut tree = BTreeMap::new();
        for line in WARNING_GROUP_TREE.lines() {
            let mut words = line.split(' ').filter(|word| !word.is_empty());
            if let Some(name) = words.next() {
                tree.insert(name, words.collect());
            }
        }
        tree
    })
}

pub fn warning_group_exists(name: &str) -> bool {
    group_tree().contains_key(name)
}

fn group_descendants(name: &str) -> BTreeSet<&'static str> {
    let tree = group_tree();
    let mut found = BTreeSet::new();
    let Some((key, _)) = tree.get_key_value(name) else {
        return found;
    };
    let mut pending = vec![*key];
    while let Some(next) = pending.pop() {
        if found.insert(next) {
            if let Some(children) = tree.get(next) {
                pending.extend(children.iter().copied());
            }
        }
    }
    found
}

#[derive(Clone, Debug, Default)]
struct DiagnosticState {
    groups: BTreeMap<&'static str, GroupMapping>,
    extension_behavior: Option<CSeverity>,
    ignore_all: bool,
    enable_all: bool,
    warnings_as_errors: bool,
    fatal_errors: bool,
}

#[derive(Clone, Debug, Default)]
struct DiagnosticEngine {
    state: DiagnosticState,
    pushed: Vec<DiagnosticState>,
    show_in_system_headers: bool,
    fatal_occurred: bool,
    last_ignored: bool,
    last_fatal: bool,
    error_limit: usize,
    diagnostics: Vec<CDiagnostic>,
    errors: usize,
    warnings: usize,
    unknown_flags: Vec<String>,
}

impl DiagnosticEngine {
    fn new(options: &WarningOptions) -> DiagnosticEngine {
        let mut engine = DiagnosticEngine {
            show_in_system_headers: options.show_in_system_headers,
            error_limit: options.error_limit,
            ..DiagnosticEngine::default()
        };
        engine.state.ignore_all = options.suppress_all;
        engine.state.enable_all = options.everything;
        engine.state.warnings_as_errors = options.warnings_as_errors;
        engine.state.fatal_errors = options.fatal_errors;
        if options.pedantic_errors {
            engine.state.extension_behavior = Some(CSeverity::Error);
        } else if options.pedantic {
            engine.state.extension_behavior = Some(CSeverity::Warning);
        }
        for flag in &options.flags {
            if !engine.apply_flag(flag) {
                engine.unknown_flags.push(flag.clone());
            }
        }
        engine
    }

    fn apply_flag(&mut self, flag: &str) -> bool {
        match flag {
            "error" => {
                self.state.warnings_as_errors = true;
                return true;
            }
            "no-error" => {
                self.state.warnings_as_errors = false;
                return true;
            }
            "everything" => {
                self.state.enable_all = true;
                return true;
            }
            "no-everything" => {
                self.state.enable_all = false;
                self.state.ignore_all = true;
                return true;
            }
            "fatal-errors" => {
                self.state.fatal_errors = true;
                return true;
            }
            "no-fatal-errors" => {
                self.state.fatal_errors = false;
                return true;
            }
            "pedantic" => {
                if self.state.extension_behavior != Some(CSeverity::Error) {
                    self.state.extension_behavior = Some(CSeverity::Warning);
                }
                return true;
            }
            "no-pedantic" => {
                self.state.extension_behavior = None;
                return true;
            }
            "system-headers" => {
                self.show_in_system_headers = true;
                return true;
            }
            "no-system-headers" => {
                self.show_in_system_headers = false;
                return true;
            }
            _ => {}
        }
        if let Some(group) = flag.strip_prefix("error=") {
            return self.set_group(group, Some(CSeverity::Error), None);
        }
        if let Some(group) = flag.strip_prefix("no-error=") {
            return self.set_group(group, None, Some(true));
        }
        if let Some(group) = flag.strip_prefix("no-") {
            return self.set_group(group, Some(CSeverity::Ignored), None);
        }
        self.set_group(flag, Some(CSeverity::Warning), None)
    }

    fn set_group(
        &mut self,
        group: &str,
        severity: Option<CSeverity>,
        no_werror: Option<bool>,
    ) -> bool {
        let members = group_descendants(group);
        if members.is_empty() {
            return false;
        }
        for member in members {
            let mapping = self.state.groups.entry(member).or_default();
            if let Some(severity) = severity {
                let keeps_error = severity == CSeverity::Warning
                    && matches!(
                        mapping.severity,
                        Some(CSeverity::Error) | Some(CSeverity::Fatal)
                    );
                if !keeps_error {
                    mapping.severity = Some(severity);
                }
                if severity == CSeverity::Error {
                    mapping.no_werror = false;
                }
            }
            if let Some(value) = no_werror {
                mapping.no_werror = value;
                if matches!(
                    mapping.severity,
                    Some(CSeverity::Error) | Some(CSeverity::Fatal)
                ) {
                    mapping.severity = Some(CSeverity::Warning);
                }
            }
            mapping.user = true;
        }
        true
    }

    fn push(&mut self) {
        self.pushed.push(self.state.clone());
    }

    fn pop(&mut self) -> bool {
        match self.pushed.pop() {
            Some(state) => {
                self.state = state;
                true
            }
            None => false,
        }
    }

    fn severity_of(&self, message: Message, in_system_header: bool) -> (CSeverity, bool) {
        let class = message.class();
        let default = match class {
            MessageClass::Error => CSeverity::Error,
            MessageClass::Fatal => CSeverity::Fatal,
            MessageClass::Note => CSeverity::Note,
            MessageClass::Warning
            | MessageClass::WarningNoWerror
            | MessageClass::WarningInSystemHeaders
            | MessageClass::ExtWarn => CSeverity::Warning,
            MessageClass::WarningOff | MessageClass::Extension => CSeverity::Ignored,
            MessageClass::ExtWarnError => CSeverity::Error,
        };
        if class == MessageClass::Note {
            return (CSeverity::Note, false);
        }
        let group = message.group();
        let mapping = if group.is_empty() {
            GroupMapping::default()
        } else {
            self.state.groups.get(group).copied().unwrap_or_default()
        };
        let mut result = match mapping.severity {
            Some(CSeverity::Warning) if default >= CSeverity::Error => default,
            Some(severity) => severity,
            None => {
                if mapping.no_werror && default == CSeverity::Error && message.is_warning_like() {
                    CSeverity::Warning
                } else {
                    default
                }
            }
        };
        if self.state.enable_all && result == CSeverity::Ignored && !mapping.user {
            result = CSeverity::Warning;
        }
        if message.is_extension() && !mapping.user {
            if let Some(behavior) = self.state.extension_behavior {
                result = result.max(behavior);
            }
        }
        if result == CSeverity::Ignored {
            return (result, false);
        }
        let default_is_error = default >= CSeverity::Error;
        if self.state.ignore_all
            && (result == CSeverity::Warning || (result >= CSeverity::Error && !default_is_error))
        {
            return (CSeverity::Ignored, false);
        }
        let mut upgraded = false;
        if result == CSeverity::Warning
            && self.state.warnings_as_errors
            && !mapping.no_werror
            && class != MessageClass::WarningNoWerror
        {
            result = CSeverity::Error;
            upgraded = true;
        }
        if result == CSeverity::Error && self.state.fatal_errors {
            result = CSeverity::Fatal;
        }
        if in_system_header
            && !self.show_in_system_headers
            && message.is_warning_like()
            && class != MessageClass::WarningInSystemHeaders
        {
            return (CSeverity::Ignored, false);
        }
        (
            result,
            upgraded || (result >= CSeverity::Error && !default_is_error),
        )
    }

    fn option_text(&self, message: Message, severity: CSeverity, as_error: bool) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();
        if severity >= CSeverity::Error && as_error && message.is_warning_like() {
            parts.push("-Werror".to_string());
        }
        let group = message.group();
        if !group.is_empty() {
            parts.push(format!("-W{group}"));
        } else if message.class() == MessageClass::Extension {
            parts.push("-Wpedantic".to_string());
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(","))
        }
    }
}

pub fn render_diagnostic(diagnostic: &CDiagnostic, map: &SourceMap, show_source: bool) -> String {
    let mut out = String::new();
    for include in diagnostic.include_stack.iter() {
        out.push_str(&format!(
            "In file included from {}:{}:\n",
            include.name, include.line
        ));
    }
    if let Some(place) = &diagnostic.presumed {
        out.push_str(&format!("{}:{}:{}: ", place.name, place.line, place.column));
    }
    out.push_str(diagnostic.severity.label());
    out.push_str(": ");
    out.push_str(&diagnostic.message);
    if let Some(option) = &diagnostic.option {
        out.push_str(&format!(" [{option}]"));
    }
    out.push('\n');
    if show_source {
        if let Some(place) = diagnostic.place {
            if (place.buffer as usize) < map.buffers.len() {
                let buffer = map.buffer(place.buffer);
                if buffer.kind == BufferKind::File {
                    let line = buffer.physical_line(place.offset);
                    let text = buffer.line_text(line);
                    let column = buffer.physical_column(place.offset) as usize;
                    out.push_str(&String::from_utf8_lossy(text));
                    out.push('\n');
                    let mut caret = String::new();
                    for (index, byte) in text.iter().enumerate() {
                        if index + 1 >= column {
                            break;
                        }
                        caret.push(if *byte == b'\t' { '\t' } else { ' ' });
                    }
                    caret.push('^');
                    out.push_str(&caret);
                    out.push('\n');
                }
            }
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Builtin {
    Line,
    File,
    Date,
    Time,
    Counter,
    Pragma,
    FloatEvaluationMethod,
    BaseFile,
    IncludeLevel,
    Timestamp,
    FileName,
    HasFeature,
    HasExtension,
    HasBuiltinFunction,
    HasConstexprBuiltinFunction,
    HasAttribute,
    HasCAttribute,
    HasDeclspecAttribute,
    HasEmbed,
    HasInclude,
    HasIncludeNext,
    HasWarning,
    IsIdentifier,
    IsTargetArchitecture,
    IsTargetVendor,
    IsTargetSystem,
    IsTargetEnvironment,
    IsTargetVariantSystem,
    IsTargetVariantEnvironment,
    BuildingModule,
    MicrosoftPragma,
}

const BUILTIN_MACROS: &[(&str, Builtin)] = &[
    ("_Pragma", Builtin::Pragma),
    ("__BASE_FILE__", Builtin::BaseFile),
    ("__COUNTER__", Builtin::Counter),
    ("__DATE__", Builtin::Date),
    ("__FILE_NAME__", Builtin::FileName),
    ("__FILE__", Builtin::File),
    ("__FLT_EVAL_METHOD__", Builtin::FloatEvaluationMethod),
    ("__INCLUDE_LEVEL__", Builtin::IncludeLevel),
    ("__LINE__", Builtin::Line),
    ("__TIMESTAMP__", Builtin::Timestamp),
    ("__TIME__", Builtin::Time),
    ("__building_module", Builtin::BuildingModule),
    ("__has_attribute", Builtin::HasAttribute),
    ("__has_builtin", Builtin::HasBuiltinFunction),
    ("__has_c_attribute", Builtin::HasCAttribute),
    (
        "__has_constexpr_builtin",
        Builtin::HasConstexprBuiltinFunction,
    ),
    ("__has_declspec_attribute", Builtin::HasDeclspecAttribute),
    ("__has_embed", Builtin::HasEmbed),
    ("__has_extension", Builtin::HasExtension),
    ("__has_feature", Builtin::HasFeature),
    ("__has_include", Builtin::HasInclude),
    ("__has_include_next", Builtin::HasIncludeNext),
    ("__has_warning", Builtin::HasWarning),
    ("__is_identifier", Builtin::IsIdentifier),
    ("__is_target_arch", Builtin::IsTargetArchitecture),
    ("__is_target_environment", Builtin::IsTargetEnvironment),
    ("__is_target_os", Builtin::IsTargetSystem),
    (
        "__is_target_variant_environment",
        Builtin::IsTargetVariantEnvironment,
    ),
    ("__is_target_variant_os", Builtin::IsTargetVariantSystem),
    ("__is_target_vendor", Builtin::IsTargetVendor),
];

#[derive(Debug)]
pub struct MacroDefinition {
    pub name: Rc<[u8]>,
    pub parameters: Vec<Rc<[u8]>>,
    pub function_like: bool,
    pub variadic: bool,
    pub gnu_variadic: bool,
    pub body: Rc<[Token]>,
    pub defined_at: Place,
    builtin: Option<Builtin>,
    enabled: std::cell::Cell<bool>,
    used: std::cell::Cell<bool>,
    warn_if_unused: std::cell::Cell<bool>,
    comma_pasting: bool,
}

impl MacroDefinition {
    fn parameter_index(&self, name: &[u8]) -> Option<usize> {
        self.parameters
            .iter()
            .position(|parameter| &**parameter == name)
    }

    fn is_variadic(&self) -> bool {
        self.variadic || self.gnu_variadic
    }

    fn identical_to(&self, other: &MacroDefinition, syntactic: bool) -> bool {
        if self.function_like != other.function_like
            || self.variadic != other.variadic
            || self.gnu_variadic != other.gnu_variadic
            || self.parameters.len() != other.parameters.len()
            || (!syntactic && self.parameters != other.parameters)
            || self.body.len() != other.body.len()
        {
            return false;
        }
        for (index, (left, right)) in self.body.iter().zip(other.body.iter()).enumerate() {
            if left.kind != right.kind {
                return false;
            }
            if index != 0
                && (left.at_start_of_line() != right.at_start_of_line()
                    || left.has_leading_space() != right.has_leading_space())
            {
                return false;
            }
            if left.spelling != right.spelling {
                if !syntactic || left.kind != TokenKind::Identifier {
                    return false;
                }
                let position = self.parameter_index(&left.spelling);
                if position.is_none() || position != other.parameter_index(&right.spelling) {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Clone, Copy, Debug)]
struct Conditional {
    place: Place,
    was_skipping: bool,
    found_non_skip: bool,
    found_else: bool,
}

#[derive(Clone, Debug, Default)]
struct GuardState {
    read_any: bool,
    after_ifndef: bool,
    expanded: bool,
    controlling: Option<Rc<[u8]>>,
}

impl GuardState {
    fn invalidate(&mut self) {
        self.read_any = true;
        self.after_ifndef = false;
        self.controlling = None;
    }

    fn enter_ifndef(&mut self, name: Rc<[u8]>) {
        if self.controlling.is_some() || self.expanded {
            self.invalidate();
            return;
        }
        self.read_any = true;
        self.after_ifndef = true;
        self.controlling = Some(name);
    }

    fn exit_conditional(&mut self) {
        if self.controlling.is_none() {
            self.invalidate();
            return;
        }
        self.read_any = false;
        self.after_ifndef = false;
    }

    fn read_token(&mut self) {
        self.read_any = true;
        self.after_ifndef = false;
    }
}

struct FileSource {
    lexer: Lexer,
    conditionals: Vec<Conditional>,
    directory: String,
    search_index: Option<usize>,
    found_by_relative: bool,
    path: Rc<str>,
    buffer: u32,
    guard: GuardState,
    guard_key: Option<String>,
}

struct TokenSource {
    tokens: Rc<[Token]>,
    index: usize,
    macro_definition: Option<Rc<MacroDefinition>>,
    disable_expansion: bool,
    at_start_of_line: bool,
    leading_space: bool,
    next_gets_space: bool,
    site: Option<(Place, Place)>,
}

enum Source {
    File(FileSource),
    Tokens(TokenSource),
}

struct MacroArguments {
    unexpanded: Vec<Vec<Token>>,
    expanded: Vec<Option<Vec<Token>>>,
    varargs_elided: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MacroOption {
    Define(String),
    Undefine(String),
}

#[derive(Clone, Debug)]
pub struct SearchDirectory {
    pub path: String,
    pub character: FileCharacter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputMode {
    Preprocessed,
    Nothing,
    MacrosOnly,
}

#[derive(Clone, Debug)]
pub struct TargetInfo {
    pub triple: String,
    pub architecture: String,
    pub vendor: String,
    pub system: String,
    pub environment: String,
}

impl TargetInfo {
    pub fn from_triple(triple: &str) -> TargetInfo {
        let mut parts = triple.split('-');
        let architecture = parts.next().unwrap_or("x86_64").to_string();
        let rest: Vec<&str> = parts.collect();
        let (vendor, system, environment) = match rest.len() {
            0 => ("unknown".to_string(), "unknown".to_string(), String::new()),
            1 => ("unknown".to_string(), rest[0].to_string(), String::new()),
            2 => (rest[0].to_string(), rest[1].to_string(), String::new()),
            _ => (
                rest[0].to_string(),
                rest[1].to_string(),
                rest[2..].join("-"),
            ),
        };
        TargetInfo {
            triple: triple.to_string(),
            architecture,
            vendor,
            system,
            environment,
        }
    }

    pub fn pointer_width(&self) -> u32 {
        match self.architecture.as_str() {
            "x86_64" | "aarch64" | "arm64" | "riscv64" | "powerpc64" | "powerpc64le" | "mips64"
            | "mips64el" | "sparcv9" | "s390x" | "loongarch64" | "wasm64" | "amdgcn"
            | "nvptx64" | "bpfel" | "bpfeb" | "ve" => {
                if self.environment == "gnux32" {
                    32
                } else {
                    64
                }
            }
            _ => 32,
        }
    }

    pub fn is_windows(&self) -> bool {
        self.system.starts_with("windows") || self.system.starts_with("win32")
    }

    pub fn long_width(&self) -> u32 {
        if self.is_windows() {
            32
        } else {
            self.pointer_width()
        }
    }

    pub fn wchar_signed(&self) -> bool {
        !(self.is_windows()
            || self.architecture.starts_with("arm")
            || self.architecture.starts_with("thumb")
            || self.architecture == "aarch64"
            || self.architecture == "arm64")
            || self.system.starts_with("darwin")
            || self.system.starts_with("macos")
            || self.system.starts_with("ios")
    }

    pub fn char_signed(&self) -> bool {
        let unsigned_char = matches!(
            self.architecture.as_str(),
            "aarch64"
                | "arm"
                | "armv7"
                | "armv7a"
                | "armeb"
                | "thumb"
                | "powerpc"
                | "powerpc64"
                | "powerpc64le"
                | "s390x"
                | "riscv32"
                | "riscv64"
                | "hexagon"
                | "msp430"
                | "xcore"
        );
        !unsigned_char
            || self.system.starts_with("darwin")
            || self.system.starts_with("ios")
            || self.system.starts_with("macos")
            || self.is_windows()
    }
}

impl Default for TargetInfo {
    fn default() -> TargetInfo {
        TargetInfo::from_triple("x86_64-unknown-linux-gnu")
    }
}

#[derive(Clone, Debug)]
pub struct PreprocessorOptions {
    pub language: LanguageOptions,
    pub target: TargetInfo,
    pub macros: Vec<MacroOption>,
    pub quoted_directories: Vec<String>,
    pub directories: Vec<SearchDirectory>,
    pub forced_includes: Vec<String>,
    pub keep_comments: bool,
    pub keep_macro_comments: bool,
    pub line_markers: bool,
    pub use_line_directives: bool,
    pub minimize_whitespace: bool,
    pub dump_defines: bool,
    pub output: OutputMode,
    pub warnings: WarningOptions,
    pub undefine_predefined: bool,
    pub source_date_epoch: Option<u64>,
    pub working_directory: String,
    pub maximum_include_depth: usize,
    pub collect_comments: bool,
    pub syntax_only: bool,
    pub freestanding: bool,
    pub gnuc_version: Option<(u32, u32, u32)>,
    pub optimize: u32,
    pub optimize_size: bool,
    pub no_inline: bool,
    pub pic_level: u32,
    pub pie: bool,
}

impl Default for PreprocessorOptions {
    fn default() -> PreprocessorOptions {
        PreprocessorOptions {
            language: LanguageOptions::default(),
            target: TargetInfo::default(),
            macros: Vec::new(),
            quoted_directories: Vec::new(),
            directories: Vec::new(),
            forced_includes: Vec::new(),
            keep_comments: false,
            keep_macro_comments: false,
            line_markers: true,
            use_line_directives: false,
            minimize_whitespace: false,
            dump_defines: false,
            output: OutputMode::Preprocessed,
            warnings: WarningOptions::default(),
            undefine_predefined: false,
            source_date_epoch: None,
            working_directory: String::new(),
            maximum_include_depth: 200,
            collect_comments: false,
            syntax_only: false,
            freestanding: false,
            gnuc_version: None,
            optimize: 0,
            optimize_size: false,
            no_inline: false,
            pic_level: 0,
            pie: false,
        }
    }
}

pub trait SourceFiles {
    fn read(&self, path: &str) -> Option<Vec<u8>>;
    fn is_directory(&self, _path: &str) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct VerifyComment {
    pub place: Place,
    pub text: Rc<[u8]>,
}

pub struct PreprocessOutcome {
    pub output: Vec<u8>,
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<CDiagnostic>,
    pub comments: Vec<VerifyComment>,
    pub sources: SourceMap,
    pub errors: usize,
    pub warnings: usize,
    pub main_buffer: Option<u32>,
    pub fatal: bool,
    pub unknown_warning_flags: Vec<String>,
}

struct Printer {
    out: Vec<u8>,
    line: u32,
    name: Rc<str>,
    character: FileCharacter,
    tokens_on_line: bool,
    directive_on_line: bool,
    initialized: bool,
    first_file_entered: bool,
    line_markers: bool,
    line_directives: bool,
    minimize: bool,
    dump_defines: bool,
    previous: Option<Token>,
    before_previous: Option<Token>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ChangeReason {
    Enter,
    Exit,
    Rename,
    SystemHeader,
}

struct SearchOrigin {
    directory: String,
    search_index: Option<usize>,
    found_by_relative: bool,
}

type FoundFile = (String, Rc<[u8]>, Option<usize>, bool, FileCharacter);

type MacroParameters = (Vec<Rc<[u8]>>, bool, bool);

pub struct Preprocessor<'files> {
    files: &'files dyn SourceFiles,
    options: PreprocessorOptions,
    language: LanguageOptions,
    pub sources: SourceMap,
    stack: Vec<Source>,
    macros: BTreeMap<Rc<[u8]>, Rc<MacroDefinition>>,
    pushed_macros: BTreeMap<Rc<[u8]>, Vec<Option<Rc<MacroDefinition>>>>,
    poisoned: BTreeSet<Rc<[u8]>>,
    once_files: BTreeSet<String>,
    engine: DiagnosticEngine,
    printer: Option<Printer>,
    comments: Vec<VerifyComment>,
    disable_expansion: bool,
    in_macro_arguments: bool,
    argument_macro_name: Option<Token>,
    in_argument_preexpansion: bool,
    parsing_if_directive: bool,
    counter: u32,
    scratch: u32,
    scratch_offset: u32,
    main_buffer: Option<u32>,
    builtin_buffer: Option<u32>,
    include_depth: usize,
    reached_include_limit: bool,
    included_files: BTreeSet<String>,
    unused_macros: Vec<Place>,
    read_cache: BTreeMap<String, Option<Rc<[u8]>>>,
    include_guards: BTreeMap<String, Rc<[u8]>>,
    pragma_site: Option<Place>,
    defined_state: Option<(bool, Rc<[u8]>)>,
    directive_read_before: bool,
    last_expression_simple: bool,
}

impl<'files> Preprocessor<'files> {
    pub fn new(
        files: &'files dyn SourceFiles,
        options: PreprocessorOptions,
    ) -> Preprocessor<'files> {
        let engine = DiagnosticEngine::new(&options.warnings);
        let language = options.language.clone();
        let mut sources = SourceMap::default();
        let scratch = sources.add(SourceBuffer::new(
            "<scratch space>",
            Rc::from(&b""[..]),
            BufferKind::Scratch,
        ));
        let printer = if options.output == OutputMode::Preprocessed {
            Some(Printer {
                out: Vec::new(),
                line: 0,
                name: Rc::from("<uninit>"),
                character: FileCharacter::User,
                tokens_on_line: false,
                directive_on_line: false,
                initialized: false,
                first_file_entered: false,
                line_markers: options.line_markers,
                line_directives: options.use_line_directives,
                minimize: options.minimize_whitespace,
                dump_defines: options.dump_defines,
                previous: None,
                before_previous: None,
            })
        } else {
            None
        };
        let mut preprocessor = Preprocessor {
            files,
            options,
            language,
            sources,
            stack: Vec::new(),
            macros: BTreeMap::new(),
            pushed_macros: BTreeMap::new(),
            poisoned: BTreeSet::new(),
            once_files: BTreeSet::new(),
            engine,
            printer,
            comments: Vec::new(),
            disable_expansion: false,
            in_macro_arguments: false,
            argument_macro_name: None,
            in_argument_preexpansion: false,
            parsing_if_directive: false,
            counter: 0,
            scratch,
            scratch_offset: 0,
            main_buffer: None,
            builtin_buffer: None,
            include_depth: 0,
            reached_include_limit: false,
            included_files: BTreeSet::new(),
            unused_macros: Vec::new(),
            read_cache: BTreeMap::new(),
            include_guards: BTreeMap::new(),
            pragma_site: None,
            defined_state: None,
            directive_read_before: true,
            last_expression_simple: false,
        };
        for (name, builtin) in BUILTIN_MACROS {
            let key: Rc<[u8]> = Rc::from(name.as_bytes());
            let definition = MacroDefinition {
                name: key.clone(),
                parameters: Vec::new(),
                function_like: false,
                variadic: false,
                gnu_variadic: false,
                body: Rc::from(Vec::new()),
                defined_at: Place::default(),
                builtin: Some(*builtin),
                enabled: std::cell::Cell::new(true),
                used: std::cell::Cell::new(false),
                warn_if_unused: std::cell::Cell::new(false),
                comma_pasting: false,
            };
            preprocessor.macros.insert(key, Rc::new(definition));
        }
        if preprocessor.language.microsoft_extensions {
            let key: Rc<[u8]> = Rc::from(&b"__pragma"[..]);
            let definition = MacroDefinition {
                name: key.clone(),
                parameters: Vec::new(),
                function_like: false,
                variadic: false,
                gnu_variadic: false,
                body: Rc::from(Vec::new()),
                defined_at: Place::default(),
                builtin: Some(Builtin::MicrosoftPragma),
                enabled: std::cell::Cell::new(true),
                used: std::cell::Cell::new(false),
                warn_if_unused: std::cell::Cell::new(false),
                comma_pasting: false,
            };
            preprocessor.macros.insert(key, Rc::new(definition));
        }
        preprocessor.poisoned.insert(Rc::from(&b"__VA_ARGS__"[..]));
        preprocessor.poisoned.insert(Rc::from(&b"__VA_OPT__"[..]));
        preprocessor
    }

    pub fn language(&self) -> &LanguageOptions {
        &self.language
    }

    fn scratch_place(&mut self, length: usize) -> Place {
        let place = Place {
            buffer: self.scratch,
            offset: self.scratch_offset,
        };
        self.scratch_offset = self.scratch_offset.wrapping_add(length as u32 + 2);
        place
    }

    fn make_token(&mut self, kind: TokenKind, spelling: Vec<u8>, site: (Place, Place)) -> Token {
        let length = spelling.len();
        let place = self.scratch_place(length);
        let mut token = Token::new(kind, Rc::from(spelling), place, length as u32);
        token.site = site.0;
        token
    }

    fn in_system_header(&self, place: Place) -> bool {
        if place.buffer as usize >= self.sources.buffers.len() {
            return false;
        }
        let buffer = self.sources.buffer(place.buffer);
        if buffer.kind == BufferKind::Scratch {
            return false;
        }
        self.sources.presumed(place).character != FileCharacter::User
    }

    fn report(&mut self, message: Message, place: Place, arguments: Vec<String>) {
        let in_system = self.in_system_header(place);
        let (severity, as_error) = self.engine.severity_of(message, in_system);
        if severity != CSeverity::Note {
            if self.engine.last_fatal {
                self.engine.fatal_occurred = true;
            }
            self.engine.last_fatal = severity == CSeverity::Fatal;
        }
        if self.engine.fatal_occurred {
            if matches!(severity, CSeverity::Error | CSeverity::Fatal) {
                self.engine.errors += 1;
            }
            return;
        }
        if severity == CSeverity::Error
            && self.engine.error_limit > 0
            && self.engine.errors + 1 > self.engine.error_limit
        {
            self.engine.errors += 1;
            self.engine.last_ignored = false;
            self.report(Message::TooManyErrors, place, Vec::new());
            return;
        }
        if severity == CSeverity::Note {
            if self.engine.last_ignored {
                return;
            }
        } else {
            self.engine.last_ignored = severity == CSeverity::Ignored;
        }
        if severity == CSeverity::Ignored {
            return;
        }
        let option = if message == Message::TooManyErrors {
            Some("-ferror-limit=".to_string())
        } else if message.is_warning_like() {
            self.engine.option_text(message, severity, as_error)
        } else {
            None
        };
        let valid = (place.buffer as usize) < self.sources.buffers.len()
            && self.sources.buffer(place.buffer).kind != BufferKind::Scratch;
        let presumed = if valid {
            Some(self.sources.presumed(place))
        } else {
            None
        };
        let include_stack = if valid {
            self.sources.include_chain(place)
        } else {
            Vec::new()
        };
        match severity {
            CSeverity::Error | CSeverity::Fatal => self.engine.errors += 1,
            CSeverity::Warning => self.engine.warnings += 1,
            _ => {}
        }
        if message == Message::TooManyErrors {
            self.engine.fatal_occurred = true;
        }
        self.engine.diagnostics.push(CDiagnostic {
            severity,
            message: message.render(&arguments),
            option,
            place: if valid { Some(place) } else { None },
            presumed,
            include_stack,
            kind: message,
        });
    }

    fn report_token(&mut self, message: Message, token: &Token, arguments: Vec<String>) {
        self.report(message, token.site, arguments);
    }

    fn message_ignored(&self, message: Message, place: Place) -> bool {
        let in_system = self.in_system_header(place);
        self.engine.severity_of(message, in_system).0 == CSeverity::Ignored
    }

    fn drain_lexer(&mut self, index: usize) {
        let (reports, comments) = match self.stack.get_mut(index) {
            Some(Source::File(file)) => (
                std::mem::take(&mut file.lexer.reports),
                std::mem::take(&mut file.lexer.comments),
            ),
            _ => return,
        };
        for report in reports {
            self.report(report.message, report.place, report.arguments);
        }
        if self.options.collect_comments {
            for comment in comments {
                self.comments.push(VerifyComment {
                    place: comment.start,
                    text: comment.text,
                });
            }
        }
    }

    fn current_file_index(&self) -> Option<usize> {
        (0..self.stack.len())
            .rev()
            .find(|index| matches!(self.stack[*index], Source::File(_)))
    }

    fn in_main_file(&self, place: Place) -> bool {
        if Some(place.buffer) != self.main_buffer {
            return false;
        }
        match self.sources.buffer(place.buffer).entry_at(place.offset) {
            Some(entry) => entry.include.is_none(),
            None => true,
        }
    }

    fn real_file_index(&self) -> Option<usize> {
        (0..self.stack.len()).rev().find(
            |index| matches!(&self.stack[*index], Source::File(file) if !file.lexer.pragma_lexer),
        )
    }

    fn in_primary_file(&self) -> bool {
        self.stack
            .iter()
            .filter(|source| matches!(source, Source::File(file) if !file.lexer.pragma_lexer))
            .count()
            <= 1
    }

    fn current_file(&mut self) -> Option<&mut FileSource> {
        let index = self.current_file_index()?;
        match &mut self.stack[index] {
            Source::File(file) => Some(file),
            Source::Tokens(_) => None,
        }
    }

    fn top_is_file(&self) -> bool {
        matches!(self.stack.last(), Some(Source::File(_)))
    }

    fn propagate(&mut self, start_of_line: bool, leading_space: bool) {
        match self.stack.last_mut() {
            Some(Source::File(file)) => {
                if start_of_line {
                    file.lexer.at_start_of_line = true;
                }
                if leading_space {
                    file.lexer.leading_space = true;
                }
            }
            Some(Source::Tokens(source)) => {
                source.at_start_of_line = start_of_line;
                source.leading_space = leading_space;
            }
            None => {}
        }
    }

    fn push_tokens(&mut self, tokens: Vec<Token>, disable_expansion: bool) {
        let (start, leading) = tokens
            .first()
            .map(|first| (first.at_start_of_line(), first.has_leading_space()))
            .unwrap_or((false, false));
        self.stack.push(Source::Tokens(TokenSource {
            tokens: Rc::from(tokens),
            index: 0,
            macro_definition: None,
            disable_expansion,
            at_start_of_line: start,
            leading_space: leading,
            next_gets_space: false,
            site: None,
        }));
    }

    pub fn lex(&mut self) -> Token {
        loop {
            let Some(top) = self.stack.len().checked_sub(1) else {
                return Token::marker(TokenKind::EndOfFile, Place::default());
            };
            let is_file = matches!(self.stack[top], Source::File(_));
            if !is_file {
                let finished = match &self.stack[top] {
                    Source::Tokens(source) => source.index >= source.tokens.len(),
                    Source::File(_) => false,
                };
                if finished {
                    let (start, leading) = match self.stack.pop() {
                        Some(Source::Tokens(source)) => {
                            if let Some(definition) = &source.macro_definition {
                                definition.enabled.set(true);
                            }
                            let first = source.index == 0;
                            (
                                source.at_start_of_line && first,
                                source.leading_space || source.next_gets_space,
                            )
                        }
                        _ => (false, false),
                    };
                    self.propagate(start, leading);
                    continue;
                }
                let mut token = self.next_from_tokens(top);
                if token.kind == TokenKind::Identifier {
                    let disable = match &self.stack.get(top) {
                        Some(Source::Tokens(source)) => source.disable_expansion,
                        _ => false,
                    };
                    if !disable {
                        if let Some(result) = self.handle_identifier(&mut token, false) {
                            return result;
                        }
                        continue;
                    }
                }
                return token;
            }
            let mut token = match &mut self.stack[top] {
                Source::File(file) => {
                    let token = file.lexer.lex();
                    if !matches!(
                        token.kind,
                        TokenKind::EndOfDirective | TokenKind::EndOfFile | TokenKind::Comment
                    ) && !(token.kind == TokenKind::Hash && token.has(PHYSICAL_START_OF_LINE))
                    {
                        file.guard.read_token();
                    }
                    token
                }
                Source::Tokens(_) => continue,
            };
            self.drain_lexer(top);
            let pragma_lexer = match &self.stack[top] {
                Source::File(file) => file.lexer.pragma_lexer,
                _ => false,
            };
            if pragma_lexer {
                if let Some(site) = self.pragma_site {
                    token.site = site;
                }
            }
            match token.kind {
                TokenKind::Hash if token.has(PHYSICAL_START_OF_LINE) && !pragma_lexer => {
                    self.handle_directive(token);
                    continue;
                }
                TokenKind::EndOfFile => {
                    if let Some(result) = self.handle_end_of_file(top, token) {
                        return result;
                    }
                    continue;
                }
                TokenKind::Identifier => {
                    let mut token = token;
                    if let Some(result) = self.handle_identifier(&mut token, true) {
                        return result;
                    }
                    continue;
                }
                _ => return token,
            }
        }
    }

    fn lex_unexpanded(&mut self) -> Token {
        let saved = self.disable_expansion;
        self.disable_expansion = true;
        let token = self.lex();
        self.disable_expansion = saved;
        token
    }

    fn lex_non_comment(&mut self) -> Token {
        loop {
            let token = self.lex();
            if token.kind != TokenKind::Comment {
                return token;
            }
        }
    }

    fn lex_unexpanded_non_comment(&mut self) -> Token {
        loop {
            let token = self.lex_unexpanded();
            if token.kind != TokenKind::Comment {
                return token;
            }
        }
    }

    fn next_from_tokens(&mut self, top: usize) -> Token {
        let (mut token, paste_next, first, site, start, leading) = {
            let Some(Source::Tokens(source)) = self.stack.get_mut(top) else {
                return Token::marker(TokenKind::EndOfFile, Place::default());
            };
            let first = source.index == 0;
            let token = source.tokens[source.index].clone();
            source.index += 1;
            let paste_next = source.macro_definition.is_some()
                && source.index < source.tokens.len()
                && source.tokens[source.index].kind == TokenKind::HashHash;
            let start = source.at_start_of_line;
            let leading = source.leading_space;
            source.at_start_of_line = false;
            source.leading_space = false;
            (token, paste_next, first, source.site, start, leading)
        };
        let mut from_paste = false;
        if paste_next {
            token = self.paste_tokens(top, token);
            from_paste = true;
        }
        if let Some((begin, _)) = site {
            token.site = begin;
        }
        if first {
            token.set(START_OF_LINE, start);
            token.set(LEADING_SPACE, leading);
        } else {
            if start {
                token.flags |= START_OF_LINE;
            }
            if leading {
                token.flags |= LEADING_SPACE;
            }
        }
        if from_paste
            && token.kind == TokenKind::Identifier
            && self.poisoned.contains(&token.spelling)
        {
            self.report_poisoned(&token);
        }
        token
    }

    fn site_end_of(&self, token: &Token) -> Place {
        for source in self.stack.iter().rev() {
            if let Source::Tokens(tokens) = source {
                if let Some((begin, end)) = tokens.site {
                    if begin == token.site {
                        return end;
                    }
                }
            }
        }
        token.site
    }

    fn report_poisoned(&mut self, token: &Token) {
        let message = match &*token.spelling {
            b"__VA_ARGS__" => Message::BadVaArgsUse,
            b"__VA_OPT__" => Message::BadVaOptUse,
            _ => Message::UsedPoisoned,
        };
        self.report_token(message, token, Vec::new());
    }

    fn next_is_left_paren(&mut self) -> bool {
        let Some(top) = self.stack.len().checked_sub(1) else {
            return false;
        };
        let mut index = top;
        loop {
            match &mut self.stack[index] {
                Source::Tokens(source) => {
                    if source.index < source.tokens.len() {
                        return source.tokens[source.index].kind == TokenKind::LeftParen;
                    }
                }
                Source::File(file) => {
                    let found = file.lexer.peek_is_left_paren();
                    return found.unwrap_or(false);
                }
            }
            if index == 0 {
                return false;
            }
            index -= 1;
        }
    }

    fn handle_identifier(&mut self, token: &mut Token, from_file: bool) -> Option<Token> {
        if from_file && self.poisoned.contains(&token.spelling) {
            self.report_poisoned(token);
        }
        let Some(definition) = self.macros.get(&token.spelling).cloned() else {
            return Some(token.clone());
        };
        if self.disable_expansion {
            return Some(token.clone());
        }
        if !token.has(DISABLE_EXPAND) && definition.enabled.get() {
            if !definition.function_like || self.next_is_left_paren() {
                if let Some(Source::File(file)) = self.stack.last_mut() {
                    file.guard.expanded = true;
                }
                return self.expand_macro(token.clone(), definition);
            }
        } else {
            token.flags |= DISABLE_EXPAND;
            if !definition.function_like || self.next_is_left_paren() {
                self.report_token(Message::DisabledMacroExpansion, token, Vec::new());
            }
        }
        Some(token.clone())
    }

    fn mark_used(&mut self, definition: &MacroDefinition) {
        if definition.warn_if_unused.get() && !definition.used.get() {
            let place = definition.defined_at;
            self.unused_macros.retain(|held| *held != place);
        }
        definition.used.set(true);
    }

    fn expand_macro(
        &mut self,
        identifier: Token,
        definition: Rc<MacroDefinition>,
    ) -> Option<Token> {
        if let Some(builtin) = definition.builtin {
            return self.expand_builtin(identifier, builtin);
        }
        let mut arguments = None;
        let mut end_site = identifier.site;
        if definition.function_like {
            let saved_in_args = self.in_macro_arguments;
            let saved_name = self.argument_macro_name.replace(identifier.clone());
            self.in_macro_arguments = true;
            let read = self.read_macro_arguments(&identifier, &definition, &mut end_site);
            self.in_macro_arguments = saved_in_args;
            self.argument_macro_name = saved_name;
            match read {
                Ok(read) => arguments = Some(read),
                Err(Some(replacement)) => return Some(replacement),
                Err(None) => return Some(identifier),
            }
        }
        self.mark_used(&definition);
        if definition.body.is_empty() {
            let mut identifier = identifier;
            identifier.flags |= LEADING_EMPTY_MACRO;
            self.propagate(
                identifier.at_start_of_line(),
                identifier.has_leading_space(),
            );
            return None;
        }
        if definition.body.len() == 1 && self.is_trivial_expansion(&definition, &identifier) {
            let mut replacement = definition.body[0].clone();
            replacement.set(START_OF_LINE, identifier.at_start_of_line());
            replacement.set(LEADING_SPACE, identifier.has_leading_space());
            replacement.site = identifier.site;
            if replacement.kind == TokenKind::Identifier {
                if let Some(other) = self.macros.get(&replacement.spelling).cloned() {
                    let same = Rc::ptr_eq(&other, &definition);
                    if !other.enabled.get() || same {
                        replacement.flags |= DISABLE_EXPAND;
                        if !same || definition.function_like {
                            self.report_token(
                                Message::DisabledMacroExpansion,
                                &replacement,
                                Vec::new(),
                            );
                        }
                    }
                }
            }
            return Some(replacement);
        }
        let tokens: Rc<[Token]> = match arguments {
            Some(mut arguments) if !definition.parameters.is_empty() => {
                let (expanded, next_gets_space) =
                    self.expand_function_arguments(&definition, &mut arguments);
                definition.enabled.set(false);
                self.stack.push(Source::Tokens(TokenSource {
                    tokens: Rc::from(expanded),
                    index: 0,
                    macro_definition: Some(definition.clone()),
                    disable_expansion: false,
                    at_start_of_line: identifier.at_start_of_line(),
                    leading_space: identifier.has_leading_space(),
                    next_gets_space,
                    site: Some((identifier.site, end_site)),
                }));
                return None;
            }
            _ => definition.body.clone(),
        };
        definition.enabled.set(false);
        self.stack.push(Source::Tokens(TokenSource {
            tokens,
            index: 0,
            macro_definition: Some(definition.clone()),
            disable_expansion: false,
            at_start_of_line: identifier.at_start_of_line(),
            leading_space: identifier.has_leading_space(),
            next_gets_space: false,
            site: Some((identifier.site, end_site)),
        }));
        None
    }

    fn is_trivial_expansion(&self, definition: &MacroDefinition, identifier: &Token) -> bool {
        let replacement = &definition.body[0];
        if replacement.kind != TokenKind::Identifier {
            return true;
        }
        if let Some(other) = self.macros.get(&replacement.spelling) {
            if other.enabled.get() && replacement.spelling != identifier.spelling {
                return false;
            }
        }
        !definition.function_like || definition.parameter_index(&replacement.spelling).is_none()
    }

    fn read_macro_arguments(
        &mut self,
        name: &Token,
        definition: &Rc<MacroDefinition>,
        end_site: &mut Place,
    ) -> Result<MacroArguments, Option<Token>> {
        let parameter_count = definition.parameters.len();
        let mut fixed_left = parameter_count;
        let variadic = definition.is_variadic();
        let mut token = self.lex_unexpanded();
        let mut groups: Vec<Vec<Token>> = Vec::new();
        let mut found_elided_comma = false;
        let mut too_many_place: Option<Place> = None;
        while token.kind != TokenKind::RightParen {
            let start_place = token.site;
            let mut current: Vec<Token> = Vec::new();
            let mut depth = 0usize;
            loop {
                token = self.lex_unexpanded();
                match token.kind {
                    TokenKind::EndOfFile | TokenKind::EndOfDirective => {
                        self.report_token(Message::UnterminatedMacroInvocation, name, Vec::new());
                        self.report(
                            Message::MacroDefinedHere,
                            definition.defined_at,
                            vec![quoted(&definition.name)],
                        );
                        return Err(Some(token));
                    }
                    TokenKind::RightParen => {
                        if depth == 0 {
                            *end_site = token.site;
                            if current
                                .last()
                                .is_some_and(|last| last.has(COMMA_AFTER_ELIDED))
                            {
                                found_elided_comma = true;
                            }
                            break;
                        }
                        depth -= 1;
                    }
                    TokenKind::LeftParen => depth += 1,
                    TokenKind::Comma => {
                        if depth == 0 && (!variadic || fixed_left > 1) {
                            break;
                        }
                    }
                    TokenKind::Comment if !self.options.keep_macro_comments => continue,
                    TokenKind::Identifier => {
                        if let Some(other) = self.macros.get(&token.spelling) {
                            if !other.enabled.get() {
                                token.flags |= DISABLE_EXPAND;
                            }
                        }
                    }
                    _ => {}
                }
                current.push(token.clone());
            }
            if current.is_empty() && groups.is_empty() && token.kind == TokenKind::RightParen {
                break;
            }
            if !variadic && fixed_left == 0 && too_many_place.is_none() {
                too_many_place = Some(
                    current
                        .first()
                        .map(|first| first.site)
                        .unwrap_or(start_place),
                );
            }
            if current.is_empty() && !self.language.at_least(CStandard::C99) {
                self.report_token(Message::EmptyMacroArgumentExtension, &token, Vec::new());
            }
            groups.push(current);
            fixed_left = fixed_left.saturating_sub(1);
        }
        let actual_count = groups.len();
        if !variadic && actual_count > parameter_count {
            let place = too_many_place.unwrap_or(name.site);
            self.report(Message::TooManyArguments, place, Vec::new());
            self.report(
                Message::MacroDefinedHere,
                definition.defined_at,
                vec![quoted(&definition.name)],
            );
            return Err(None);
        }
        let mut varargs_elided = false;
        if actual_count < parameter_count {
            if actual_count == 0 && parameter_count == 1 {
                varargs_elided = variadic;
            } else if (found_elided_comma || variadic)
                && (actual_count + 1 == parameter_count
                    || (actual_count == 0 && parameter_count == 2))
            {
                if !definition.comma_pasting {
                    let message = if self.language.at_least(CStandard::C23) {
                        Message::MissingVariadicArgumentCompatibility
                    } else {
                        Message::MissingVariadicArgumentExtension
                    };
                    self.report_token(message, &token, Vec::new());
                    self.report(
                        Message::MacroDefinedHere,
                        definition.defined_at,
                        vec![quoted(&definition.name)],
                    );
                }
                varargs_elided = true;
            } else {
                self.report_token(Message::TooFewArguments, &token, Vec::new());
                self.report(
                    Message::MacroDefinedHere,
                    definition.defined_at,
                    vec![quoted(&definition.name)],
                );
                return Err(None);
            }
            while groups.len() < parameter_count {
                groups.push(Vec::new());
            }
        }
        let count = groups.len();
        Ok(MacroArguments {
            unexpanded: groups,
            expanded: vec![None; count],
            varargs_elided,
        })
    }

    fn preexpanded_argument(&mut self, arguments: &mut MacroArguments, index: usize) -> Vec<Token> {
        if let Some(found) = &arguments.expanded[index] {
            return found.clone();
        }
        let mut stream = arguments.unexpanded[index].clone();
        let end_place = stream.last().map(|last| last.site).unwrap_or_default();
        let mut end = Token::marker(TokenKind::EndOfFile, end_place);
        end.flags |= PLACEHOLDER;
        stream.push(end);
        let depth = self.stack.len();
        let saved = self.in_argument_preexpansion;
        self.in_argument_preexpansion = true;
        self.push_tokens(stream, false);
        let mut result = Vec::new();
        loop {
            let token = self.lex();
            if token.kind == TokenKind::EndOfFile && token.has(PLACEHOLDER) {
                break;
            }
            if token.kind == TokenKind::EndOfFile && self.stack.len() <= depth {
                break;
            }
            result.push(token);
        }
        self.in_argument_preexpansion = saved;
        while self.stack.len() > depth {
            if let Some(Source::Tokens(source)) = self.stack.pop() {
                if let Some(definition) = source.macro_definition {
                    definition.enabled.set(true);
                }
            }
        }
        arguments.expanded[index] = Some(result.clone());
        result
    }

    fn argument_needs_preexpansion(&self, tokens: &[Token]) -> bool {
        tokens.iter().any(|token| {
            token.kind == TokenKind::Identifier && self.macros.contains_key(&token.spelling)
        })
    }

    fn stringify(
        &mut self,
        tokens: &[Token],
        charify: bool,
        site: (Place, Place),
        report_at: Option<&Token>,
    ) -> Token {
        let mut result: Vec<u8> = vec![b'"'];
        for (index, token) in tokens.iter().enumerate() {
            if index != 0 && (token.has_leading_space() || token.at_start_of_line()) {
                result.push(b' ');
            }
            if token.kind.is_string_or_character() {
                result.extend_from_slice(&stringify_text(&token.spelling, b'"'));
            } else {
                result.extend_from_slice(&token.spelling);
            }
        }
        if result.len() > 1 && *result.last().unwrap_or(&0) == b'\\' {
            let mut slashes = 0usize;
            let mut at = result.len();
            while at > 1 && result[at - 1] == b'\\' {
                slashes += 1;
                at -= 1;
            }
            if slashes % 2 == 1 {
                if let Some(last) = tokens.last() {
                    let last = last.clone();
                    self.report_token(Message::InvalidStringLiteral, &last, Vec::new());
                }
                result.pop();
            }
        }
        result.push(b'"');
        let mut kind = TokenKind::String(LiteralPrefix::Plain);
        if charify {
            kind = TokenKind::Character(LiteralPrefix::Plain);
            let length = result.len();
            result[0] = b'\'';
            result[length - 1] = b'\'';
            let bad = if result.len() == 3 {
                result[1] == b'\''
            } else {
                result.len() != 4 || result[1] != b'\\'
            };
            if bad {
                if let Some(first) = tokens.first().or(report_at) {
                    let first = first.clone();
                    self.report_token(Message::InvalidCharacterToCharify, &first, Vec::new());
                }
                result = b"' '".to_vec();
            }
        }
        let mut token = self.make_token(kind, result, site);
        token.flags |= STRINGIFIED_IN_MACRO;
        token
    }

    fn expand_function_arguments(
        &mut self,
        definition: &Rc<MacroDefinition>,
        arguments: &mut MacroArguments,
    ) -> (Vec<Token>, bool) {
        let body: Vec<Token> = definition.body.to_vec();
        let count = body.len();
        let mut result: Vec<Token> = Vec::with_capacity(count + 8);
        let mut next_gets_space = false;
        let mut called_with_variadic: Option<bool> = None;
        let mut vaopt = VaOptState::default();
        let site = (Place::default(), Place::default());
        let mut index = 0usize;
        while index < count {
            let current = body[index].clone();
            if index != 0
                && body[index - 1].kind != TokenKind::HashHash
                && current.has_leading_space()
            {
                next_gets_space = true;
            }
            if current.is_identifier("__VA_OPT__") {
                index += 1;
                vaopt.open(result.len());
                index += 1;
                continue;
            }
            if vaopt.active() {
                if current.kind == TokenKind::LeftParen {
                    vaopt.depth += 1;
                }
                let closes = current.kind == TokenKind::RightParen && {
                    vaopt.depth -= 1;
                    vaopt.depth == 0
                };
                if !closes {
                    if called_with_variadic.is_none() {
                        let variadic_index = arguments.unexpanded.len().saturating_sub(1);
                        let value = definition.is_variadic()
                            && !arguments.unexpanded.is_empty()
                            && !self
                                .preexpanded_argument(arguments, variadic_index)
                                .is_empty();
                        called_with_variadic = Some(value);
                    }
                    if called_with_variadic == Some(false) {
                        index += 1;
                        continue;
                    }
                } else {
                    if vaopt.stringify || vaopt.charify {
                        let prior = vaopt.prior;
                        let mut contents: Vec<Token> = result[prior..].to_vec();
                        let mut joined: Vec<Token> = Vec::new();
                        let mut position = 0usize;
                        while position < contents.len() {
                            if contents[position].kind == TokenKind::HashHash && position > 0 {
                                let mut left = joined
                                    .pop()
                                    .unwrap_or_else(|| contents[position - 1].clone());
                                position += 1;
                                while position < contents.len() {
                                    let right = contents[position].clone();
                                    left = self.paste_pair(left, &right, None).0;
                                    position += 1;
                                    if position < contents.len()
                                        && contents[position].kind == TokenKind::HashHash
                                    {
                                        position += 1;
                                        continue;
                                    }
                                    break;
                                }
                                joined.push(left);
                                continue;
                            }
                            joined.push(contents[position].clone());
                            position += 1;
                        }
                        contents = joined;
                        let mut stringified =
                            self.stringify(&contents, vaopt.charify, site, Some(&current));
                        if vaopt.leading_space_for_string {
                            stringified.flags |= LEADING_SPACE;
                        }
                        result.truncate(prior);
                        result.push(stringified);
                    } else if result.len() == vaopt.prior {
                        if result
                            .last()
                            .is_some_and(|last| last.kind == TokenKind::HashHash)
                        {
                            result.pop();
                        } else if index + 1 != count && body[index + 1].kind == TokenKind::HashHash
                        {
                            index += 1;
                        }
                    } else {
                        if vaopt.begins_with_placeholder
                            && vaopt.prior > 0
                            && result.len() >= vaopt.prior
                        {
                            result.remove(vaopt.prior - 1);
                        }
                        if vaopt.ends_with_placeholder
                            && index + 1 != count
                            && body[index + 1].kind == TokenKind::HashHash
                        {
                            index += 1;
                        }
                    }
                    vaopt = VaOptState::default();
                    index += 1;
                    continue;
                }
            }
            if current.kind == TokenKind::Hash || current.kind == TokenKind::HashAt {
                let next = &body[index + 1];
                let parameter = if next.kind == TokenKind::Identifier {
                    definition.parameter_index(&next.spelling)
                } else {
                    None
                };
                let Some(parameter) = parameter else {
                    vaopt.stringify = current.kind == TokenKind::Hash;
                    vaopt.charify = current.kind == TokenKind::HashAt;
                    vaopt.leading_space_for_string = next_gets_space;
                    index += 1;
                    continue;
                };
                let charify = current.kind == TokenKind::HashAt;
                let unexpanded = arguments.unexpanded[parameter].clone();
                let mut stringified = self.stringify(&unexpanded, charify, site, Some(&current));
                if next_gets_space {
                    stringified.flags |= LEADING_SPACE;
                }
                result.push(stringified);
                index += 2;
                next_gets_space = false;
                continue;
            }
            let non_empty_paste_before = result
                .last()
                .is_some_and(|last| last.kind == TokenKind::HashHash);
            let paste_before = index != 0 && body[index - 1].kind == TokenKind::HashHash;
            let paste_after = index + 1 != count && body[index + 1].kind == TokenKind::HashHash;
            let right_paren_after =
                index + 1 != count && body[index + 1].kind == TokenKind::RightParen;
            let parameter = if current.kind == TokenKind::Identifier {
                definition.parameter_index(&current.spelling)
            } else {
                None
            };
            let Some(parameter) = parameter else {
                let mut token = current.clone();
                if next_gets_space {
                    token.flags |= LEADING_SPACE;
                    next_gets_space = false;
                } else if paste_before && !non_empty_paste_before {
                    token.flags &= !LEADING_SPACE;
                }
                result.push(token);
                index += 1;
                continue;
            };
            if !paste_before
                && arguments.varargs_elided
                && self.remove_comma_before_varargs(
                    &mut result,
                    false,
                    definition,
                    parameter,
                    &mut next_gets_space,
                )
            {
                index += 1;
                continue;
            }
            if !paste_before && !paste_after {
                let unexpanded = arguments.unexpanded[parameter].clone();
                let tokens = if self.argument_needs_preexpansion(&unexpanded) {
                    self.preexpanded_argument(arguments, parameter)
                } else {
                    unexpanded
                };
                if !tokens.is_empty() {
                    let first = result.len();
                    for token in tokens {
                        let mut token = token;
                        if token.kind == TokenKind::HashHash {
                            token.kind = TokenKind::Unknown;
                        }
                        result.push(token);
                    }
                    result[first].set(LEADING_SPACE, next_gets_space);
                    result[first].set(START_OF_LINE, false);
                    next_gets_space = false;
                } else if non_empty_paste_before {
                    vaopt.begins_with_placeholder = true;
                } else if right_paren_after && vaopt.at_top_level() {
                    vaopt.ends_with_placeholder = true;
                }
                index += 1;
                continue;
            }
            let unexpanded = arguments.unexpanded[parameter].clone();
            if !unexpanded.is_empty() {
                let mut pseudo_paste = false;
                if non_empty_paste_before
                    && result.len() >= 2
                    && result[result.len() - 2].kind == TokenKind::Comma
                    && parameter + 1 == definition.parameters.len()
                    && definition.is_variadic()
                {
                    pseudo_paste = true;
                    if let Some(removed) = result.pop() {
                        let support = if self.language.at_least(CStandard::C23) {
                            "1"
                        } else {
                            "0"
                        };
                        self.report_token(Message::PasteComma, &removed, vec![support.to_string()]);
                    }
                }
                let first = result.len();
                let length = unexpanded.len();
                for token in unexpanded {
                    let mut token = token;
                    if token.kind == TokenKind::HashHash {
                        token.kind = TokenKind::Unknown;
                    }
                    result.push(token);
                }
                if !pseudo_paste && length > 0 {
                    result[first].set(START_OF_LINE, false);
                    result[first].set(LEADING_SPACE, next_gets_space);
                }
                next_gets_space = false;
                index += 1;
                continue;
            }
            if paste_after {
                index += 2;
                continue;
            }
            if right_paren_after && !non_empty_paste_before && vaopt.at_top_level() {
                vaopt.ends_with_placeholder = true;
            }
            if non_empty_paste_before {
                if !vaopt.active() || result.len() > vaopt.prior {
                    result.pop();
                } else {
                    vaopt.begins_with_placeholder = true;
                }
            }
            if arguments.varargs_elided {
                self.remove_comma_before_varargs(
                    &mut result,
                    true,
                    definition,
                    parameter,
                    &mut next_gets_space,
                );
            }
            index += 1;
        }
        (result, next_gets_space)
    }

    fn remove_comma_before_varargs(
        &mut self,
        result: &mut Vec<Token>,
        has_paste: bool,
        definition: &MacroDefinition,
        parameter: usize,
        next_gets_space: &mut bool,
    ) -> bool {
        if !definition.is_variadic() || parameter + 1 != definition.parameters.len() {
            return false;
        }
        if !has_paste && !self.language.microsoft_extensions {
            return false;
        }
        if self.language.at_least(CStandard::C99)
            && !self.language.gnu
            && definition.parameters.len() < 2
        {
            return false;
        }
        if result
            .last()
            .is_none_or(|last| last.kind != TokenKind::Comma)
        {
            return false;
        }
        if has_paste {
            if let Some(last) = result.last() {
                let last = last.clone();
                let support = if self.language.at_least(CStandard::C23) {
                    "1"
                } else {
                    "0"
                };
                self.report_token(Message::PasteComma, &last, vec![support.to_string()]);
            }
        }
        result.pop();
        if !result.is_empty() {
            if result
                .last()
                .is_some_and(|last| last.kind == TokenKind::HashHash)
            {
                result.pop();
            }
            if let Some(last) = result.last_mut() {
                last.flags |= COMMA_AFTER_ELIDED;
            }
        }
        *next_gets_space = false;
        true
    }

    fn paste_tokens(&mut self, top: usize, left: Token) -> Token {
        let mut left = left;
        loop {
            let right = {
                let Some(Source::Tokens(source)) = self.stack.get_mut(top) else {
                    return left;
                };
                if source.index >= source.tokens.len()
                    || source.tokens[source.index].kind != TokenKind::HashHash
                {
                    break;
                }
                let mut operator = source.tokens[source.index].clone();
                if let Some((begin, _)) = source.site {
                    operator.site = begin;
                }
                source.index += 1;
                if source.index >= source.tokens.len() {
                    return left;
                }
                let right = source.tokens[source.index].clone();
                (operator, right)
            };
            let (operator, right) = right;
            let (pasted, failed) = self.paste_pair(left.clone(), &right, Some(&operator));
            if failed {
                break;
            }
            if let Some(Source::Tokens(source)) = self.stack.get_mut(top) {
                source.index += 1;
            }
            left = pasted;
        }
        left
    }

    fn paste_pair(
        &mut self,
        left: Token,
        right: &Token,
        operator: Option<&Token>,
    ) -> (Token, bool) {
        let mut spelling = left.spelling.to_vec();
        spelling.extend_from_slice(&right.spelling);
        let site = (left.site, left.site);
        let mut result;
        if left.kind == TokenKind::Identifier && right.kind == TokenKind::Identifier {
            result = self.make_token(TokenKind::Identifier, spelling, site);
        } else {
            let bytes: Rc<[u8]> = Rc::from(spelling.clone());
            let mut lexer = Lexer::new(bytes, self.scratch, &self.language);
            lexer.raw = true;
            lexer.at_start_of_line = false;
            lexer.at_physical_start_of_line = false;
            let token = lexer.lex();
            let invalid = token.kind == TokenKind::EndOfFile || lexer.position != spelling.len();
            if invalid && self.language.assembly {
                return (left, true);
            }
            if invalid {
                if let Some(operator) = operator {
                    let place = operator.site;
                    self.report(
                        Message::BadPaste,
                        place,
                        vec![String::from_utf8_lossy(&spelling).into_owned()],
                    );
                } else {
                    self.report_token(
                        Message::BadPaste,
                        &left,
                        vec![String::from_utf8_lossy(&spelling).into_owned()],
                    );
                }
                return (left, true);
            }
            let mut kind = token.kind;
            if kind == TokenKind::HashHash {
                kind = TokenKind::Unknown;
            }
            result = self.make_token(kind, token.spelling.to_vec(), site);
        }
        result.set(START_OF_LINE, left.at_start_of_line());
        result.set(LEADING_SPACE, left.has_leading_space());
        if result.kind == TokenKind::Identifier && (left.has(HAS_UCN) || right.has(HAS_UCN)) {
            result.spelling = Rc::from(expand_ucns(&result.spelling));
        }
        (result, false)
    }
}

#[derive(Default)]
struct VaOptState {
    open: bool,
    depth: usize,
    prior: usize,
    stringify: bool,
    charify: bool,
    leading_space_for_string: bool,
    begins_with_placeholder: bool,
    ends_with_placeholder: bool,
}

impl VaOptState {
    fn open(&mut self, prior: usize) {
        self.open = true;
        self.depth = 1;
        self.prior = prior;
    }

    fn active(&self) -> bool {
        self.open && self.depth > 0
    }

    fn at_top_level(&self) -> bool {
        self.open && self.depth == 1
    }
}

fn stringify_text(text: &[u8], quote: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len() + 4);
    let mut at = 0usize;
    while at < text.len() {
        let byte = text[at];
        if byte == b'\\' || byte == quote {
            out.push(b'\\');
            out.push(byte);
        } else if byte == b'\n' || byte == b'\r' {
            out.push(b'\\');
            out.push(b'n');
            if at + 1 < text.len()
                && (text[at + 1] == b'\n' || text[at + 1] == b'\r')
                && text[at + 1] != byte
            {
                at += 1;
            }
        } else {
            out.push(byte);
        }
        at += 1;
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MacroUse {
    Define,
    Undefine,
    Other,
}

#[derive(Clone, Debug)]
struct ExpressionValue {
    bits: u64,
    unsigned: bool,
    identifier: Option<Token>,
}

impl ExpressionValue {
    fn signed(value: i64) -> ExpressionValue {
        ExpressionValue {
            bits: value as u64,
            unsigned: false,
            identifier: None,
        }
    }

    fn of(bits: u64, unsigned: bool) -> ExpressionValue {
        ExpressionValue {
            bits,
            unsigned,
            identifier: None,
        }
    }

    fn truth(value: bool) -> ExpressionValue {
        ExpressionValue::signed(i64::from(value))
    }

    fn is_zero(&self) -> bool {
        self.bits == 0
    }

    fn is_negative(&self) -> bool {
        !self.unsigned && (self.bits as i64) < 0
    }

    fn decimal(&self, unsigned: bool) -> String {
        if unsigned {
            self.bits.to_string()
        } else {
            (self.bits as i64).to_string()
        }
    }
}

enum NumberValue {
    Integer {
        value: u64,
        unsigned: bool,
        radix: u32,
        long_long: bool,
        overflow: bool,
    },
    Floating,
}

const C_KEYWORDS: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else",
    "enum", "extern", "float", "for", "goto", "if", "int", "long", "register", "return", "short",
    "signed", "sizeof", "static", "struct", "switch", "typedef", "union", "unsigned", "void",
    "volatile", "while",
];

const C99_KEYWORDS: &[&str] = &["_Bool", "_Complex", "_Imaginary", "inline", "restrict"];

const C11_KEYWORDS: &[&str] = &[
    "_Alignas",
    "_Alignof",
    "_Atomic",
    "_Generic",
    "_Noreturn",
    "_Static_assert",
    "_Thread_local",
];

const C23_KEYWORDS: &[&str] = &[
    "alignas",
    "alignof",
    "bool",
    "constexpr",
    "false",
    "nullptr",
    "static_assert",
    "thread_local",
    "true",
    "typeof",
    "typeof_unqual",
    "_BitInt",
    "_Decimal128",
    "_Decimal32",
    "_Decimal64",
];

const FEATURE_TEST_MACROS: &[&str] = &[
    "_ATFILE_SOURCE",
    "_BSD_SOURCE",
    "_CRT_NONSTDC_NO_WARNINGS",
    "_CRT_SECURE_CPP_OVERLOAD_STANDARD_NAMES",
    "_CRT_SECURE_NO_WARNINGS",
    "_FILE_OFFSET_BITS",
    "_FORTIFY_SOURCE",
    "_GLIBCXX_ASSERTIONS",
    "_GLIBCXX_CONCEPT_CHECKS",
    "_GLIBCXX_DEBUG",
    "_GLIBCXX_DEBUG_PEDANTIC",
    "_GLIBCXX_PARALLEL",
    "_GLIBCXX_PARALLEL_ASSERTIONS",
    "_GLIBCXX_SANITIZE_VECTOR",
    "_GLIBCXX_USE_CXX11_ABI",
    "_GLIBCXX_USE_DEPRECATED",
    "_GNU_SOURCE",
    "_ISOC11_SOURCE",
    "_ISOC95_SOURCE",
    "_ISOC99_SOURCE",
    "_LARGEFILE64_SOURCE",
    "_POSIX_C_SOURCE",
    "_REENTRANT",
    "_SVID_SOURCE",
    "_THREAD_SAFE",
    "_XOPEN_SOURCE",
    "_XOPEN_SOURCE_EXTENDED",
    "__STDCPP_WANT_MATH_SPEC_FUNCS__",
    "__STDC_FORMAT_MACROS",
];

fn token_kind_name(token: &Token) -> String {
    match token.kind.punctuator_spelling() {
        Some(spelling) => format!("'{spelling}'"),
        None => match token.kind {
            TokenKind::Identifier => quoted(&token.spelling),
            TokenKind::Number => "numeric_constant".to_string(),
            TokenKind::String(_) => "string_literal".to_string(),
            TokenKind::Character(_) => "char_constant".to_string(),
            TokenKind::EndOfDirective => "end of directive".to_string(),
            TokenKind::EndOfFile => "end of file".to_string(),
            _ => "unknown".to_string(),
        },
    }
}

fn hex_value(byte: u8) -> Option<u32> {
    (byte as char).to_digit(16)
}

impl<'files> Preprocessor<'files> {
    fn is_keyword(&self, name: &[u8]) -> bool {
        let Ok(text) = std::str::from_utf8(name) else {
            return false;
        };
        if C_KEYWORDS.contains(&text) {
            return true;
        }
        if (self.language.at_least(CStandard::C99) || self.language.gnu) && text == "inline" {
            return true;
        }
        if self.language.at_least(CStandard::C99) && C99_KEYWORDS.contains(&text) {
            return true;
        }
        if C99_KEYWORDS.contains(&text) && text.starts_with('_') {
            return true;
        }
        if C11_KEYWORDS.contains(&text) {
            return true;
        }
        if self.language.at_least(CStandard::C23) && C23_KEYWORDS.contains(&text) {
            return true;
        }
        if self.language.gnu && (text == "asm" || text == "typeof") {
            return true;
        }
        false
    }

    fn handle_end_of_file(&mut self, top: usize, token: Token) -> Option<Token> {
        let (conditionals, buffer, pragma) = match &mut self.stack[top] {
            Source::File(file) => (
                std::mem::take(&mut file.conditionals),
                file.buffer,
                file.lexer.pragma_lexer,
            ),
            Source::Tokens(_) => return Some(token),
        };
        if pragma {
            self.stack.pop();
            self.propagate(token.at_start_of_line(), token.has_leading_space());
            return None;
        }
        for conditional in conditionals.iter().rev() {
            self.report(
                Message::UnterminatedConditional,
                conditional.place,
                Vec::new(),
            );
        }
        let bytes = self.sources.buffer(buffer).bytes.clone();
        if let Some(last) = bytes.last() {
            if *last != b'\n' && *last != b'\r' {
                self.report(
                    Message::NoNewlineAtEndOfFile,
                    Place {
                        buffer,
                        offset: bytes.len() as u32,
                    },
                    Vec::new(),
                );
            }
        }
        if top == 0 {
            let unused = std::mem::take(&mut self.unused_macros);
            for place in unused {
                self.report(Message::MacroNotUsed, place, Vec::new());
            }
            return Some(token);
        }
        let is_builtin = Some(buffer) == self.builtin_buffer;
        if let Some(Source::File(file)) = self.stack.pop() {
            if !file.guard.read_any {
                if let (Some(key), Some(controlling)) = (file.guard_key, file.guard.controlling) {
                    self.include_guards.insert(key, controlling);
                }
            }
        }
        if !is_builtin {
            self.include_depth = self.include_depth.saturating_sub(1);
        }
        self.propagate(token.at_start_of_line(), token.has_leading_space());
        if let Some(index) = self.current_file_index() {
            if let Source::File(file) = &self.stack[index] {
                let place = file.lexer.place(file.lexer.position);
                self.file_changed(place, ChangeReason::Exit);
            }
        }
        None
    }

    fn enter_buffer(
        &mut self,
        name: &str,
        bytes: Rc<[u8]>,
        kind: BufferKind,
        included_at: Option<Place>,
        character: FileCharacter,
        origin: SearchOrigin,
    ) -> u32 {
        let SearchOrigin {
            directory,
            search_index,
            found_by_relative,
        } = origin;
        let mut buffer = SourceBuffer::new(name, bytes.clone(), kind);
        buffer.included_at = included_at;
        buffer.character = character;
        let index = self.sources.add(buffer);
        let mut lexer = Lexer::new(bytes, index, &self.language);
        lexer.keep_comments = self.options.keep_comments;
        self.stack.push(Source::File(FileSource {
            lexer,
            conditionals: Vec::new(),
            directory,
            search_index,
            found_by_relative,
            path: Rc::from(name),
            buffer: index,
            guard: GuardState::default(),
            guard_key: None,
        }));
        self.file_changed(
            Place {
                buffer: index,
                offset: 0,
            },
            ChangeReason::Enter,
        );
        index
    }

    fn read_file(&mut self, path: &str) -> Option<Rc<[u8]>> {
        if let Some(found) = self.read_cache.get(path) {
            return found.clone();
        }
        let found = if self.files.is_directory(path) {
            None
        } else {
            self.files.read(path).map(Rc::from)
        };
        self.read_cache.insert(path.to_string(), found.clone());
        found
    }

    fn join_path(directory: &str, name: &str) -> String {
        if directory.is_empty() {
            name.to_string()
        } else if directory.ends_with('/') {
            format!("{directory}{name}")
        } else {
            format!("{directory}/{name}")
        }
    }

    fn directory_of(path: &str) -> String {
        match path.rfind('/') {
            Some(0) => "/".to_string(),
            Some(at) => path[..at].to_string(),
            None => String::new(),
        }
    }

    fn lookup_file(
        &mut self,
        name: &str,
        angled: bool,
        start_index: Option<usize>,
        include_next: bool,
    ) -> Option<FoundFile> {
        if name.starts_with('/') {
            let bytes = self.read_file(name)?;
            return Some((name.to_string(), bytes, None, true, FileCharacter::User));
        }
        if !angled && !include_next {
            if let Some(index) = self.current_file_index() {
                let (directory, character) = match &self.stack[index] {
                    Source::File(file) => (
                        file.directory.clone(),
                        self.sources.buffer(file.buffer).character,
                    ),
                    _ => (String::new(), FileCharacter::User),
                };
                let path = Self::join_path(&directory, name);
                let full = if path.starts_with('/') || self.options.working_directory.is_empty() {
                    path.clone()
                } else {
                    Self::join_path(&self.options.working_directory, &path)
                };
                if let Some(bytes) = self.read_file(&full) {
                    return Some((path, bytes, None, true, character));
                }
            }
        }
        let quoted_count = self.options.quoted_directories.len();
        let total = quoted_count + self.options.directories.len();
        let mut index = start_index.unwrap_or(if angled { quoted_count } else { 0 });
        while index < total {
            let (directory, character) = if index < quoted_count {
                (
                    self.options.quoted_directories[index].clone(),
                    FileCharacter::User,
                )
            } else {
                let entry = &self.options.directories[index - quoted_count];
                (entry.path.clone(), entry.character)
            };
            let path = Self::join_path(&directory, name);
            let full = if path.starts_with('/') || self.options.working_directory.is_empty() {
                path.clone()
            } else {
                Self::join_path(&self.options.working_directory, &path)
            };
            if let Some(bytes) = self.read_file(&full) {
                return Some((path, bytes, Some(index), false, character));
            }
            index += 1;
        }
        None
    }

    fn handle_directive(&mut self, hash: Token) {
        let mut read_before = true;
        if let Some(Source::File(file)) = self.stack.last_mut() {
            file.lexer.in_directive = true;
            read_before = file.guard.read_any;
        }
        self.directive_read_before = read_before;
        let directive = self.lex_unexpanded();
        if self.in_macro_arguments {
            if directive.kind == TokenKind::Identifier {
                let name = directive.text().to_string();
                if matches!(
                    name.as_str(),
                    "include" | "import" | "include_next" | "__include_macros" | "pragma" | "embed"
                ) {
                    self.report_token(
                        Message::EmbeddedDirectiveError,
                        &directive,
                        vec!["1".to_string(), name],
                    );
                    if let Some(macro_name) = self.argument_macro_name.clone() {
                        self.report_token(
                            Message::MacroExpansionRequestedHere,
                            &macro_name,
                            vec![quoted(&macro_name.spelling)],
                        );
                    }
                    self.discard_until_end_of_directive();
                    return;
                }
            }
            self.report_token(Message::EmbeddedDirective, &directive, Vec::new());
        }
        match directive.kind {
            TokenKind::EndOfDirective => {
                if let Some(file) = self.current_file() {
                    file.guard.read_any = read_before;
                }
                return;
            }
            TokenKind::Number => {
                if !self.language.assembly || Some(hash.place.buffer) == self.builtin_buffer {
                    self.handle_digit_directive(directive);
                    return;
                }
            }
            TokenKind::Identifier => {
                let name = directive.text().to_string();
                match name.as_str() {
                    "if" => return self.handle_if(&directive, &hash),
                    "ifdef" => return self.handle_ifdef(&directive, false),
                    "ifndef" => return self.handle_ifdef(&directive, true),
                    "elif" => return self.handle_elif_family(&directive, 0),
                    "elifdef" => return self.handle_elif_family(&directive, 1),
                    "elifndef" => return self.handle_elif_family(&directive, 2),
                    "else" => return self.handle_else(&directive),
                    "endif" => return self.handle_endif(&directive),
                    "include" => {
                        return self.handle_include(&hash, &directive, IncludeKind::Include)
                    }
                    "include_next" => {
                        self.report_token(Message::IncludeNextDirective, &directive, Vec::new());
                        return self.handle_include(&hash, &directive, IncludeKind::IncludeNext);
                    }
                    "import" => {
                        self.report_token(Message::ImportDirective, &directive, Vec::new());
                        return self.handle_include(&hash, &directive, IncludeKind::Import);
                    }
                    "define" => return self.handle_define(),
                    "undef" => return self.handle_undefine(),
                    "line" => return self.handle_line(),
                    "error" => return self.handle_user_diagnostic(&directive, false),
                    "warning" => {
                        let message = if self.language.at_least(CStandard::C23) {
                            Message::WarningDirectiveCompatibility
                        } else {
                            Message::WarningDirectiveExtension
                        };
                        self.report_token(message, &directive, vec!["0".to_string()]);
                        return self.handle_user_diagnostic(&directive, true);
                    }
                    "pragma" => return self.handle_pragma_directive(&hash),
                    "ident" | "sccs" => return self.handle_ident(&directive),
                    "assert" | "unassert" => {}
                    _ => {}
                }
            }
            _ => {}
        }
        if self.language.assembly {
            let mut directive = directive;
            if directive.kind == TokenKind::HashHash {
                directive.kind = TokenKind::Unknown;
            }
            self.push_tokens(vec![hash, directive], false);
            return;
        }
        self.report_token(Message::InvalidDirective, &directive, vec!["0".to_string()]);
        self.discard_until_end_of_directive();
    }

    fn discard_until_end_of_directive(&mut self) -> Token {
        loop {
            let token = self.lex_unexpanded();
            if matches!(token.kind, TokenKind::EndOfDirective | TokenKind::EndOfFile) {
                return token;
            }
        }
    }

    fn check_end_of_directive(&mut self, name: &str, enable_macros: bool) {
        let mut token = if enable_macros {
            self.lex()
        } else {
            self.lex_unexpanded()
        };
        while token.kind == TokenKind::Comment {
            token = self.lex_unexpanded();
        }
        if matches!(token.kind, TokenKind::EndOfDirective | TokenKind::EndOfFile) {
            return;
        }
        self.report_token(
            Message::ExtraTokensAtEndOfDirective,
            &token,
            vec![name.to_string()],
        );
        self.discard_until_end_of_directive();
    }

    fn check_macro_name(&mut self, token: &Token, usage: MacroUse) -> (bool, bool) {
        if token.kind == TokenKind::EndOfDirective {
            self.report_token(Message::MissingMacroName, token, Vec::new());
            return (true, false);
        }
        if token.kind != TokenKind::Identifier {
            self.report_token(Message::MacroNotIdentifier, token, Vec::new());
            return (true, false);
        }
        if usage != MacroUse::Other && &*token.spelling == b"defined" {
            self.report_token(Message::DefinedMacroName, token, Vec::new());
            return (true, false);
        }
        let mut shadows_keyword = false;
        if usage != MacroUse::Other {
            let name = token.text();
            let reserved = name.starts_with("__")
                || (name.len() >= 2
                    && name.starts_with('_')
                    && name.as_bytes()[1].is_ascii_uppercase());
            let in_system = self.in_system_header(token.site);
            let in_builtin = Some(token.site.buffer) == self.builtin_buffer;
            if !in_system && !in_builtin {
                if reserved {
                    if usage == MacroUse::Undefine || !FEATURE_TEST_MACROS.contains(&name) {
                        self.report_token(Message::MacroReservedIdentifier, token, Vec::new());
                    }
                } else if usage == MacroUse::Define && self.is_keyword(&token.spelling) {
                    shadows_keyword = true;
                }
            }
        }
        (false, shadows_keyword)
    }

    fn read_macro_name(&mut self, usage: MacroUse) -> Option<(Token, bool)> {
        let token = self.lex_unexpanded();
        let (failed, shadows) = self.check_macro_name(&token, usage);
        if !failed {
            return Some((token, shadows));
        }
        if token.kind != TokenKind::EndOfDirective {
            self.discard_until_end_of_directive();
        }
        None
    }

    fn set_directive_comment_retention(&mut self, keep: bool) {
        if let Some(Source::File(file)) = self.stack.last_mut() {
            file.lexer.keep_comments = keep;
        }
    }

    fn in_directive(&self) -> bool {
        match self.stack.last() {
            Some(Source::File(file)) => file.lexer.in_directive,
            _ => false,
        }
    }

    fn handle_define(&mut self) {
        let Some((name, shadows_keyword)) = self.read_macro_name(MacroUse::Define) else {
            return;
        };
        let keep_macro_comments = self.options.keep_macro_comments;
        self.set_directive_comment_retention(keep_macro_comments);
        let definition = self.read_macro_definition(&name);
        let keep = self.options.keep_comments;
        self.set_directive_comment_retention(keep);
        if self.in_directive() {
            self.discard_until_end_of_directive();
        }
        let Some(definition) = definition else {
            return;
        };
        if shadows_keyword && !self.is_configuration_pattern(&name, &definition) {
            self.report_token(Message::MacroHidesKeyword, &name, Vec::new());
        }
        if let Some(first) = definition.body.first() {
            if first.kind == TokenKind::HashHash {
                let first = first.clone();
                self.report_token(Message::PasteAtStart, &first, Vec::new());
                return;
            }
        }
        if let Some(last) = definition.body.last() {
            if last.kind == TokenKind::HashHash {
                let last = last.clone();
                self.report_token(Message::PasteAtEnd, &last, Vec::new());
                return;
            }
        }
        let definition = Rc::new(definition);
        if let Some(existing) = self.macros.get(&name.spelling).cloned() {
            if !self.in_system_header(name.site) {
                if !existing.used.get() && existing.warn_if_unused.get() {
                    self.report(Message::MacroNotUsed, existing.defined_at, Vec::new());
                }
                if self.is_language_builtin(&existing) {
                    self.report_token(Message::RedefBuiltinMacro, &name, Vec::new());
                } else if !self.allow_redefinition(&existing)
                    && !definition.identical_to(&existing, self.language.microsoft_extensions)
                {
                    self.report(
                        Message::MacroRedefined,
                        definition.defined_at,
                        vec![quoted(&name.spelling)],
                    );
                    self.report(Message::PreviousDefinition, existing.defined_at, Vec::new());
                }
            }
            if existing.warn_if_unused.get() {
                let place = existing.defined_at;
                self.unused_macros.retain(|held| *held != place);
            }
        }
        let in_main = self.in_main_file(definition.defined_at);
        if in_main && !self.message_ignored(Message::MacroNotUsed, definition.defined_at) {
            definition.warn_if_unused.set(true);
            self.unused_macros.push(definition.defined_at);
        }
        self.macros
            .insert(name.spelling.clone(), definition.clone());
        self.print_macro_defined(&definition);
    }

    fn allow_redefinition(&self, definition: &Rc<MacroDefinition>) -> bool {
        self.pushed_macros.values().any(|stack| {
            stack.iter().any(|held| {
                held.as_ref()
                    .is_some_and(|held| Rc::ptr_eq(held, definition))
            })
        })
    }

    fn is_language_builtin(&self, definition: &MacroDefinition) -> bool {
        if definition.builtin.is_some() {
            return true;
        }
        if Some(definition.defined_at.buffer) != self.builtin_buffer {
            return false;
        }
        definition.name.starts_with(b"__STDC")
    }

    fn is_configuration_pattern(&self, name: &Token, definition: &MacroDefinition) -> bool {
        if definition.body.len() == 1 {
            let value = &definition.body[0];
            if value.kind == TokenKind::Identifier && value.spelling == name.spelling {
                return true;
            }
            if value.kind != TokenKind::Identifier || !self.is_keyword(&value.spelling) {
                return false;
            }
            let text = value.text();
            let trimmed = if let Some(rest) = text.strip_prefix("__") {
                rest.strip_suffix("__").unwrap_or(rest)
            } else if let Some(rest) = text.strip_prefix('_') {
                rest
            } else {
                return false;
            };
            return trimmed.as_bytes() == &*name.spelling;
        }
        matches!(
            &*name.spelling,
            b"extern" | b"inline" | b"static" | b"const"
        ) && definition.body.is_empty()
    }

    fn read_macro_parameters(&mut self) -> Option<MacroParameters> {
        let mut parameters: Vec<Rc<[u8]>> = Vec::new();
        loop {
            let token = self.lex_unexpanded_non_comment();
            match token.kind {
                TokenKind::RightParen => {
                    if parameters.is_empty() {
                        return Some((parameters, false, false));
                    }
                    self.report_token(
                        Message::ExpectedIdentifierInParameterList,
                        &token,
                        Vec::new(),
                    );
                    return None;
                }
                TokenKind::Ellipsis => {
                    if !self.language.at_least(CStandard::C99) {
                        self.report_token(Message::VariadicMacroExtension, &token, Vec::new());
                    }
                    let next = self.lex_unexpanded_non_comment();
                    if next.kind != TokenKind::RightParen {
                        self.report_token(
                            Message::MissingRightParenInMacroDefinition,
                            &next,
                            Vec::new(),
                        );
                        return None;
                    }
                    parameters.push(Rc::from(&b"__VA_ARGS__"[..]));
                    return Some((parameters, true, false));
                }
                TokenKind::EndOfDirective => {
                    self.report_token(
                        Message::MissingRightParenInMacroDefinition,
                        &token,
                        Vec::new(),
                    );
                    return None;
                }
                TokenKind::Identifier => {
                    if parameters.contains(&token.spelling) {
                        self.report_token(
                            Message::DuplicateParameterName,
                            &token,
                            vec![quoted(&token.spelling)],
                        );
                        return None;
                    }
                    parameters.push(token.spelling.clone());
                    let next = self.lex_unexpanded_non_comment();
                    match next.kind {
                        TokenKind::RightParen => return Some((parameters, false, false)),
                        TokenKind::Comma => {}
                        TokenKind::Ellipsis => {
                            self.report_token(Message::NamedVariadicMacro, &next, Vec::new());
                            let after = self.lex_unexpanded_non_comment();
                            if after.kind != TokenKind::RightParen {
                                self.report_token(
                                    Message::MissingRightParenInMacroDefinition,
                                    &after,
                                    Vec::new(),
                                );
                                return None;
                            }
                            return Some((parameters, false, true));
                        }
                        _ => {
                            self.report_token(
                                Message::ExpectedCommaInParameterList,
                                &next,
                                Vec::new(),
                            );
                            return None;
                        }
                    }
                }
                _ => {
                    self.report_token(Message::InvalidTokenInParameterList, &token, Vec::new());
                    return None;
                }
            }
        }
    }

    fn read_macro_definition(&mut self, name: &Token) -> Option<MacroDefinition> {
        let mut token = self.lex_unexpanded();
        let mut function_like = false;
        let mut parameters = Vec::new();
        let mut variadic = false;
        let mut gnu_variadic = false;
        if token.kind == TokenKind::EndOfDirective {
        } else if token.has_leading_space() {
            token.flags &= !LEADING_SPACE;
        } else if token.kind == TokenKind::LeftParen {
            function_like = true;
            let (read, c99, gnu) = self.read_macro_parameters()?;
            parameters = read;
            variadic = c99;
            gnu_variadic = gnu;
            if variadic {
                self.poisoned.remove(&b"__VA_ARGS__"[..]);
                self.poisoned.remove(&b"__VA_OPT__"[..]);
            }
            token = self.lex_unexpanded();
        } else if self.language.at_least(CStandard::C99) {
            self.report_token(Message::C99WhitespaceAfterMacroName, &token, Vec::new());
        } else if matches!(token.kind, TokenKind::At | TokenKind::Unknown) {
            self.report_token(Message::WhitespaceAfterMacroName, &token, Vec::new());
        } else {
            self.report_token(
                Message::WhitespaceRecommendedAfterMacroName,
                &token,
                Vec::new(),
            );
        }
        let restore = |preprocessor: &mut Preprocessor| {
            preprocessor.poisoned.insert(Rc::from(&b"__VA_ARGS__"[..]));
            preprocessor.poisoned.insert(Rc::from(&b"__VA_OPT__"[..]));
        };
        let mut body: Vec<Token> = Vec::new();
        let mut comma_pasting = false;
        if !function_like {
            while token.kind != TokenKind::EndOfDirective {
                body.push(token);
                token = self.lex_unexpanded();
            }
        } else {
            let mut vaopt_depth = 0usize;
            let mut vaopt_open: Option<Place> = None;
            let mut last = token.clone();
            while token.kind != TokenKind::EndOfDirective {
                last = token.clone();
                if !matches!(
                    token.kind,
                    TokenKind::Hash | TokenKind::HashAt | TokenKind::HashHash
                ) {
                    body.push(token.clone());
                    if token.is_identifier("__VA_OPT__") {
                        if vaopt_depth > 0 {
                            self.report_token(Message::NestedVaOpt, &token, Vec::new());
                            restore(self);
                            return None;
                        }
                        token = self.lex_unexpanded();
                        if token.kind != TokenKind::LeftParen {
                            self.report_token(
                                Message::MissingLeftParenAfterVaOpt,
                                &token,
                                Vec::new(),
                            );
                            restore(self);
                            return None;
                        }
                        body.push(token.clone());
                        vaopt_depth = 1;
                        vaopt_open = Some(token.site);
                        token = self.lex_unexpanded();
                        if token.kind == TokenKind::HashHash {
                            self.report_token(Message::VaOptPasteAtStart, &token, Vec::new());
                            restore(self);
                            return None;
                        }
                        continue;
                    } else if vaopt_depth > 0 {
                        if token.kind == TokenKind::RightParen {
                            vaopt_depth -= 1;
                            if vaopt_depth == 0
                                && body.len() >= 3
                                && body[body.len() - 2].kind == TokenKind::HashHash
                            {
                                self.report_token(Message::VaOptPasteAtEnd, &token, Vec::new());
                                restore(self);
                                return None;
                            }
                        } else if token.kind == TokenKind::LeftParen {
                            vaopt_depth += 1;
                        }
                    }
                    token = self.lex_unexpanded();
                    continue;
                }
                if self.language.traditional {
                    token.kind = TokenKind::Unknown;
                    body.push(token);
                    token = self.lex_unexpanded();
                    continue;
                }
                if token.kind == TokenKind::HashHash {
                    let operator = token.clone();
                    token = self.lex_unexpanded();
                    if token.kind == TokenKind::EndOfDirective {
                        body.push(operator);
                        break;
                    }
                    if !body.is_empty()
                        && token.is_identifier("__VA_ARGS__")
                        && body
                            .last()
                            .is_some_and(|previous| previous.kind == TokenKind::Comma)
                    {
                        comma_pasting = true;
                    }
                    body.push(operator);
                    continue;
                }
                let hash = token.clone();
                token = self.lex_unexpanded();
                let is_vaopt = token.is_identifier("__VA_OPT__");
                let is_parameter =
                    token.kind == TokenKind::Identifier && parameters.contains(&token.spelling);
                if !is_vaopt && !is_parameter {
                    if self.language.assembly && token.kind != TokenKind::EndOfDirective {
                        let mut hash = hash;
                        hash.kind = TokenKind::Unknown;
                        body.push(hash);
                        continue;
                    }
                    let charify = if hash.kind == TokenKind::HashAt {
                        "1"
                    } else {
                        "0"
                    };
                    self.report_token(
                        Message::StringizeNotParameter,
                        &token,
                        vec![charify.to_string()],
                    );
                    restore(self);
                    return None;
                }
                body.push(hash);
                if !is_vaopt {
                    body.push(token.clone());
                    last = token.clone();
                    token = self.lex_unexpanded();
                }
            }
            if vaopt_depth > 0 {
                self.report_token(
                    Message::MissingAfter,
                    &token,
                    vec![token_kind_name(&last), "')'".to_string()],
                );
                if let Some(open) = vaopt_open {
                    self.report(Message::ToMatch, open, vec!["'('".to_string()]);
                }
                restore(self);
                return None;
            }
        }
        restore(self);
        Some(MacroDefinition {
            name: name.spelling.clone(),
            parameters,
            function_like,
            variadic,
            gnu_variadic,
            body: Rc::from(body),
            defined_at: name.site,
            builtin: None,
            enabled: std::cell::Cell::new(true),
            used: std::cell::Cell::new(false),
            warn_if_unused: std::cell::Cell::new(false),
            comma_pasting,
        })
    }

    fn handle_undefine(&mut self) {
        let Some((name, _)) = self.read_macro_name(MacroUse::Undefine) else {
            return;
        };
        self.check_end_of_directive("undef", false);
        if let Some(existing) = self.macros.get(&name.spelling).cloned() {
            if !existing.used.get() && existing.warn_if_unused.get() {
                self.report(Message::MacroNotUsed, existing.defined_at, Vec::new());
            }
            if self.is_language_builtin(&existing) {
                self.report_token(Message::UndefBuiltinMacro, &name, Vec::new());
            }
            if existing.warn_if_unused.get() {
                let place = existing.defined_at;
                self.unused_macros.retain(|held| *held != place);
            }
            self.macros.remove(&name.spelling);
        }
        self.print_macro_undefined(&name);
    }

    fn push_conditional(
        &mut self,
        place: Place,
        was_skipping: bool,
        found_non_skip: bool,
        found_else: bool,
    ) {
        if let Some(file) = self.current_file() {
            file.conditionals.push(Conditional {
                place,
                was_skipping,
                found_non_skip,
                found_else,
            });
        }
    }

    fn pop_conditional(&mut self) -> Option<Conditional> {
        self.current_file()?.conditionals.pop()
    }

    fn handle_ifdef(&mut self, directive: &Token, negated: bool) {
        let Some((name, _)) = self.read_macro_name(MacroUse::Other) else {
            self.skip_excluded_block(directive.site, false, false);
            return;
        };
        self.check_end_of_directive(if negated { "ifndef" } else { "ifdef" }, false);
        let defined = self.macros.get(&name.spelling).cloned();
        let read_before = self.directive_read_before || !negated;
        if let Some(file) = self.current_file() {
            if file.conditionals.is_empty() {
                if !read_before && defined.is_none() {
                    file.guard.enter_ifndef(name.spelling.clone());
                } else {
                    file.guard.invalidate();
                }
            }
        }
        if let Some(definition) = &defined {
            self.mark_used(definition);
        }
        if defined.is_none() == negated {
            self.push_conditional(directive.site, false, true, false);
        } else {
            self.skip_excluded_block(directive.site, false, false);
        }
    }

    fn handle_if(&mut self, directive: &Token, _hash: &Token) {
        let read_before = self.directive_read_before;
        self.defined_state = None;
        let value = self.evaluate_directive_expression();
        let ifndef = match self.defined_state.take() {
            Some((true, name)) if self.last_expression_simple => Some(name),
            _ => None,
        };
        if let Some(file) = self.current_file() {
            if file.conditionals.is_empty() {
                match ifndef {
                    Some(name) if !read_before && value => file.guard.enter_ifndef(name),
                    _ => file.guard.invalidate(),
                }
            }
        }
        if value {
            self.push_conditional(directive.site, false, true, false);
        } else {
            self.skip_excluded_block(directive.site, false, false);
        }
    }

    fn handle_endif(&mut self, directive: &Token) {
        self.check_end_of_directive("endif", false);
        if self.pop_conditional().is_none() {
            self.report_token(Message::EndifWithoutIf, directive, Vec::new());
            return;
        }
        if let Some(file) = self.current_file() {
            if file.conditionals.is_empty() {
                file.guard.exit_conditional();
            }
        }
    }

    fn handle_else(&mut self, directive: &Token) {
        self.check_end_of_directive("else", false);
        let Some(conditional) = self.pop_conditional() else {
            self.report_token(Message::ElseWithoutIf, directive, Vec::new());
            return;
        };
        if let Some(file) = self.current_file() {
            if file.conditionals.is_empty() {
                file.guard.invalidate();
            }
        }
        if conditional.found_else {
            self.report_token(Message::ElseAfterElse, directive, Vec::new());
        }
        self.skip_excluded_block(conditional.place, true, true);
    }

    fn handle_elif_family(&mut self, directive: &Token, kind: usize) {
        if kind != 0 {
            let message = if self.language.at_least(CStandard::C23) {
                Message::C23DirectiveCompatibility
            } else {
                Message::C23DirectiveExtension
            };
            self.report_token(message, directive, vec![kind.to_string()]);
        }
        self.discard_until_end_of_directive();
        let Some(conditional) = self.pop_conditional() else {
            self.report_token(Message::ElifWithoutIf, directive, vec![kind.to_string()]);
            return;
        };
        if let Some(file) = self.current_file() {
            if file.conditionals.is_empty() {
                file.guard.invalidate();
            }
        }
        if conditional.found_else {
            self.report_token(Message::ElifAfterElse, directive, vec![kind.to_string()]);
        }
        self.skip_excluded_block(conditional.place, true, conditional.found_else);
    }

    fn file_lexer_mut(&mut self) -> Option<&mut Lexer> {
        match self.stack.last_mut() {
            Some(Source::File(file)) => Some(&mut file.lexer),
            _ => None,
        }
    }

    fn raw_discard_line(&mut self) {
        loop {
            let Some(lexer) = self.file_lexer_mut() else {
                return;
            };
            let token = lexer.lex();
            if matches!(token.kind, TokenKind::EndOfDirective | TokenKind::EndOfFile) {
                return;
            }
        }
    }

    fn set_raw(&mut self, raw: bool) {
        if let Some(lexer) = self.file_lexer_mut() {
            lexer.raw = raw;
        }
    }

    fn suggest_directive(&mut self, token: &Token, name: &str) {
        if self.language.assembly {
            return;
        }
        let mut candidates = vec!["if", "ifdef", "ifndef", "elif", "else", "endif"];
        if self.language.at_least(CStandard::C23) {
            candidates.extend(["elifdef", "elifndef"]);
        }
        let mut best: Option<(&str, usize)> = None;
        for candidate in &candidates {
            if candidate.eq_ignore_ascii_case(name) {
                best = Some((candidate, 0));
                break;
            }
        }
        if best.is_none() {
            let length = name.len();
            let maximum = if length < 3 {
                length.saturating_sub(1)
            } else {
                length / 3
            };
            for candidate in &candidates {
                let distance = edit_distance(name, candidate);
                if distance <= maximum && best.is_none_or(|(_, held)| distance < held) {
                    best = Some((candidate, distance));
                }
            }
        }
        if let Some((suggestion, _)) = best {
            self.report_token(
                Message::UnknownDirectiveSuggestion,
                token,
                vec!["1".to_string(), suggestion.to_string()],
            );
        }
    }

    fn skip_excluded_block(&mut self, if_place: Place, found_non_skip: bool, found_else: bool) {
        self.push_conditional(if_place, false, found_non_skip, found_else);
        self.set_raw(true);
        loop {
            let hash = loop {
                let Some(lexer) = self.file_lexer_mut() else {
                    return;
                };
                let token = lexer.lex();
                if token.kind == TokenKind::EndOfFile {
                    break None;
                }
                if token.kind == TokenKind::Hash && token.at_start_of_line() {
                    break Some(token);
                }
            };
            if hash.is_none() {
                break;
            }
            let directive = {
                let Some(lexer) = self.file_lexer_mut() else {
                    return;
                };
                lexer.in_directive = true;
                lexer.lex()
            };
            if directive.kind != TokenKind::Identifier {
                if let Some(lexer) = self.file_lexer_mut() {
                    lexer.in_directive = false;
                }
                if directive.kind == TokenKind::EndOfDirective {
                    if let Some(lexer) = self.file_lexer_mut() {
                        lexer.at_start_of_line = true;
                        lexer.at_physical_start_of_line = true;
                    }
                }
                continue;
            }
            let name = directive.text().to_string();
            let first = name.as_bytes()[0];
            if first.is_ascii_lowercase() && first != b'i' && first != b'e' {
                if let Some(lexer) = self.file_lexer_mut() {
                    lexer.in_directive = false;
                }
                continue;
            }
            if name.len() >= 20 {
                if let Some(lexer) = self.file_lexer_mut() {
                    lexer.in_directive = false;
                }
                continue;
            }
            let mut finished = false;
            if let Some(rest) = name.strip_prefix("if") {
                if rest.is_empty() || rest == "def" || rest == "ndef" {
                    self.raw_discard_line();
                    self.push_conditional(directive.site, true, false, false);
                    continue;
                }
                self.set_raw(false);
                self.suggest_directive(&directive, &name);
                self.set_raw(true);
            } else if first == b'e' {
                match &name[1..] {
                    "ndif" => {
                        let conditional = self.pop_conditional();
                        if conditional.is_some_and(|held| !held.was_skipping) {
                            if let Some(file) = self.current_file() {
                                if file.conditionals.is_empty() {
                                    file.guard.exit_conditional();
                                }
                            }
                            self.set_raw(false);
                            self.check_end_of_directive("endif", false);
                            self.set_raw(true);
                            finished = true;
                        } else {
                            self.raw_discard_line();
                            continue;
                        }
                    }
                    "lse" => {
                        let state = self
                            .current_file()
                            .and_then(|file| file.conditionals.last().copied());
                        let Some(state) = state else {
                            self.raw_discard_line();
                            continue;
                        };
                        if state.found_else {
                            self.report_token(Message::ElseAfterElse, &directive, Vec::new());
                        }
                        if let Some(file) = self.current_file() {
                            if let Some(last) = file.conditionals.last_mut() {
                                last.found_else = true;
                            }
                        }
                        if !state.was_skipping && !state.found_non_skip {
                            if let Some(file) = self.current_file() {
                                if let Some(last) = file.conditionals.last_mut() {
                                    last.found_non_skip = true;
                                }
                            }
                            self.set_raw(false);
                            self.check_end_of_directive("else", false);
                            self.set_raw(true);
                            finished = true;
                        } else {
                            self.raw_discard_line();
                            continue;
                        }
                    }
                    "lif" => {
                        let state = self
                            .current_file()
                            .and_then(|file| file.conditionals.last().copied());
                        let Some(state) = state else {
                            self.raw_discard_line();
                            continue;
                        };
                        if state.found_else {
                            self.report_token(
                                Message::ElifAfterElse,
                                &directive,
                                vec!["0".to_string()],
                            );
                        }
                        if state.was_skipping || state.found_non_skip {
                            self.raw_discard_line();
                            continue;
                        }
                        self.set_raw(false);
                        let value = self.evaluate_directive_expression();
                        self.set_raw(true);
                        if value {
                            if let Some(file) = self.current_file() {
                                if let Some(last) = file.conditionals.last_mut() {
                                    last.found_non_skip = true;
                                }
                            }
                            finished = true;
                        } else {
                            continue;
                        }
                    }
                    "lifdef" | "lifndef" => {
                        let is_elifdef = name == "elifdef";
                        let state = self
                            .current_file()
                            .and_then(|file| file.conditionals.last().copied());
                        let message = if self.language.at_least(CStandard::C23) {
                            Message::C23DirectiveCompatibility
                        } else {
                            Message::C23DirectiveExtension
                        };
                        let kind = if is_elifdef { "1" } else { "2" };
                        self.report_token(message, &directive, vec![kind.to_string()]);
                        let Some(state) = state else {
                            self.raw_discard_line();
                            continue;
                        };
                        if state.found_else {
                            self.report_token(
                                Message::ElifAfterElse,
                                &directive,
                                vec![kind.to_string()],
                            );
                        }
                        if state.was_skipping || state.found_non_skip {
                            self.raw_discard_line();
                            continue;
                        }
                        self.set_raw(false);
                        let read = self.read_macro_name(MacroUse::Other);
                        self.set_raw(true);
                        let Some((macro_name, _)) = read else {
                            continue;
                        };
                        self.set_raw(false);
                        self.check_end_of_directive(
                            if is_elifdef { "elifdef" } else { "elifndef" },
                            false,
                        );
                        self.set_raw(true);
                        let defined = self.macros.contains_key(&macro_name.spelling);
                        if defined == is_elifdef {
                            if let Some(file) = self.current_file() {
                                if let Some(last) = file.conditionals.last_mut() {
                                    last.found_non_skip = true;
                                }
                            }
                            finished = true;
                        } else {
                            continue;
                        }
                    }
                    _ => {
                        self.set_raw(false);
                        self.suggest_directive(&directive, &name);
                        self.set_raw(true);
                    }
                }
            } else {
                self.set_raw(false);
                self.suggest_directive(&directive, &name);
                self.set_raw(true);
            }
            if finished {
                break;
            }
            if let Some(lexer) = self.file_lexer_mut() {
                lexer.in_directive = false;
            }
        }
        self.set_raw(false);
        if let Some(index) = self.current_file_index() {
            self.drain_lexer(index);
        }
    }

    fn evaluate_directive_expression(&mut self) -> bool {
        let saved_if = self.parsing_if_directive;
        let saved_disable = self.disable_expansion;
        self.last_expression_simple = false;
        self.parsing_if_directive = true;
        self.disable_expansion = false;
        let mut token = self.lex_non_comment();
        let result = match self.evaluate_value(&mut token, true) {
            Err(()) => {
                if token.kind != TokenKind::EndOfDirective {
                    self.discard_until_end_of_directive();
                }
                false
            }
            Ok(mut value) => {
                if token.kind == TokenKind::EndOfDirective {
                    self.last_expression_simple = true;
                    !value.is_zero()
                } else if self
                    .evaluate_subexpression(
                        &mut value,
                        precedence(TokenKind::Question),
                        &mut token,
                        true,
                    )
                    .is_err()
                {
                    if token.kind != TokenKind::EndOfDirective {
                        self.discard_until_end_of_directive();
                    }
                    false
                } else {
                    if token.kind != TokenKind::EndOfDirective {
                        self.report_token(
                            Message::ExpectedEndOfLineInExpression,
                            &token,
                            Vec::new(),
                        );
                        self.discard_until_end_of_directive();
                    }
                    !value.is_zero()
                }
            }
        };
        self.parsing_if_directive = saved_if;
        self.disable_expansion = saved_disable;
        result
    }

    fn evaluate_defined(&mut self, token: &mut Token, live: bool) -> Result<ExpressionValue, ()> {
        let begin = token.clone();
        let mut peek = self.lex_unexpanded_non_comment();
        let mut left_paren: Option<Token> = None;
        if peek.kind == TokenKind::LeftParen {
            left_paren = Some(peek.clone());
            peek = self.lex_unexpanded_non_comment();
        }
        if self.check_macro_name(&peek, MacroUse::Other).0 {
            *token = peek;
            return Err(());
        }
        let definition = self.macros.get(&peek.spelling).cloned();
        let value = ExpressionValue::truth(definition.is_some());
        self.defined_state = Some((false, peek.spelling.clone()));
        if let Some(definition) = &definition {
            if live {
                self.mark_used(definition);
            }
        }
        if let Some(open) = left_paren {
            let close = self.lex_unexpanded_non_comment();
            if close.kind != TokenKind::RightParen {
                self.report_token(
                    Message::MissingAfter,
                    &close,
                    vec!["'defined'".to_string(), "')'".to_string()],
                );
                self.report_token(Message::ToMatch, &open, vec!["'('".to_string()]);
                *token = close;
                return Err(());
            }
        }
        *token = self.lex_non_comment();
        if let Some(function_like) = self.token_from_macro(&begin) {
            let message = if function_like {
                Message::ExpansionToDefinedFunction
            } else {
                Message::ExpansionToDefinedObject
            };
            self.report_token(message, &begin, Vec::new());
        }
        Ok(value)
    }

    fn token_from_macro(&self, token: &Token) -> Option<bool> {
        if token.place.buffer == token.site.buffer && token.place == token.site {
            return None;
        }
        for source in self.stack.iter().rev() {
            if let Source::Tokens(tokens) = source {
                if let Some(definition) = &tokens.macro_definition {
                    if tokens.site.is_some_and(|(begin, _)| begin == token.site) {
                        return Some(definition.function_like);
                    }
                }
            }
        }
        if token.place != token.site {
            return Some(false);
        }
        None
    }

    fn evaluate_value(&mut self, token: &mut Token, live: bool) -> Result<ExpressionValue, ()> {
        self.defined_state = None;
        match token.kind {
            TokenKind::Identifier => {
                if &*token.spelling == b"defined" {
                    return self.evaluate_defined(token, live);
                }
                if self.language.at_least(CStandard::C23)
                    && (&*token.spelling == b"true" || &*token.spelling == b"false")
                {
                    let mut value = ExpressionValue::truth(&*token.spelling == b"true");
                    value.identifier = Some(token.clone());
                    *token = self.lex_non_comment();
                    return Ok(value);
                }
                let current = token.clone();
                if live {
                    let message = if &*token.spelling == b"true" {
                        Message::UndefTrueIdentifier
                    } else {
                        Message::UndefIdentifier
                    };
                    self.report_token(message, &current, vec![quoted(&current.spelling)]);
                }
                *token = self.lex_non_comment();
                let mut value = ExpressionValue::signed(0);
                value.identifier = Some(current);
                Ok(value)
            }
            TokenKind::EndOfDirective | TokenKind::RightParen => {
                let current = token.clone();
                self.report_token(Message::ExpectedValueInExpression, &current, Vec::new());
                Err(())
            }
            TokenKind::Number => {
                let current = token.clone();
                let parsed = self.parse_number(&current, true)?;
                match parsed {
                    NumberValue::Floating => {
                        self.report_token(
                            Message::FloatingLiteralInExpression,
                            &current,
                            Vec::new(),
                        );
                        Err(())
                    }
                    NumberValue::Integer {
                        value,
                        unsigned,
                        radix,
                        long_long,
                        overflow,
                    } => {
                        if !self.language.at_least(CStandard::C99) && long_long {
                            self.report_token(Message::LongLongExtension, &current, Vec::new());
                        }
                        let mut result = ExpressionValue::of(value, unsigned);
                        if overflow {
                            self.report_token(
                                Message::IntegerLiteralTooLarge,
                                &current,
                                vec!["1".to_string()],
                            );
                            result.unsigned = true;
                        } else if !unsigned && (value as i64) < 0 {
                            if live && radix == 10 {
                                self.report_token(
                                    Message::IntegerLiteralTooLargeForSigned,
                                    &current,
                                    Vec::new(),
                                );
                            }
                            result.unsigned = true;
                        }
                        *token = self.lex_non_comment();
                        Ok(result)
                    }
                }
            }
            TokenKind::Character(prefix) => {
                let current = token.clone();
                let value = self.parse_character(&current, prefix)?;
                *token = self.lex_non_comment();
                Ok(value)
            }
            TokenKind::LeftParen => {
                let open = token.clone();
                *token = self.lex_non_comment();
                let mut value = self.evaluate_value(token, live)?;
                if token.kind != TokenKind::RightParen {
                    self.evaluate_subexpression(&mut value, 1, token, live)?;
                    self.defined_state = None;
                    value.identifier = None;
                    if token.kind != TokenKind::RightParen {
                        let current = token.clone();
                        self.report_token(
                            Message::ExpectedRightParenInExpression,
                            &current,
                            Vec::new(),
                        );
                        self.report_token(Message::ToMatch, &open, vec!["'('".to_string()]);
                        return Err(());
                    }
                }
                *token = self.lex_non_comment();
                value.identifier = None;
                Ok(value)
            }
            TokenKind::Plus => {
                *token = self.lex_non_comment();
                let mut value = self.evaluate_value(token, live)?;
                value.identifier = None;
                Ok(value)
            }
            TokenKind::Minus => {
                let operator = token.clone();
                *token = self.lex_non_comment();
                let mut value = self.evaluate_value(token, live)?;
                let overflow = !value.unsigned && value.bits == i64::MIN as u64;
                value.bits = value.bits.wrapping_neg();
                value.identifier = None;
                self.defined_state = None;
                if overflow && live {
                    self.report_token(Message::ExpressionOverflow, &operator, Vec::new());
                }
                Ok(value)
            }
            TokenKind::Tilde => {
                *token = self.lex_non_comment();
                let mut value = self.evaluate_value(token, live)?;
                value.bits = !value.bits;
                value.identifier = None;
                self.defined_state = None;
                Ok(value)
            }
            TokenKind::Exclaim => {
                *token = self.lex_non_comment();
                let value = self.evaluate_value(token, live)?;
                if let Some((negated, name)) = self.defined_state.take() {
                    self.defined_state = Some((!negated, name));
                }
                Ok(ExpressionValue::truth(value.is_zero()))
            }
            _ => {
                let current = token.clone();
                self.report_token(Message::InvalidTokenAtStart, &current, Vec::new());
                Err(())
            }
        }
    }

    fn unexpected_operator(&mut self, left_identifier: Option<Token>, token: &Token) {
        if token.kind == TokenKind::LeftParen {
            if let Some(name) = left_identifier {
                self.report_token(
                    Message::FunctionLikeMacroNotDefined,
                    &name,
                    vec![quoted(&name.spelling)],
                );
                return;
            }
        }
        self.report_token(Message::BadBinaryOperator, token, Vec::new());
    }

    fn evaluate_subexpression(
        &mut self,
        left: &mut ExpressionValue,
        minimum: u32,
        token: &mut Token,
        live: bool,
    ) -> Result<(), ()> {
        let mut peek_precedence = precedence(token.kind);
        if peek_precedence == u32::MAX {
            let name = left.identifier.clone();
            let current = token.clone();
            self.unexpected_operator(name, &current);
            return Err(());
        }
        loop {
            if peek_precedence < minimum {
                return Ok(());
            }
            let operator = token.kind;
            let operator_token = token.clone();
            let right_live = match operator {
                TokenKind::AmpAmp if left.is_zero() => false,
                TokenKind::PipePipe if !left.is_zero() => false,
                TokenKind::Question if left.is_zero() => false,
                _ => live,
            };
            *token = self.lex_non_comment();
            let mut right = self.evaluate_value(token, right_live)?;
            let this_precedence = peek_precedence;
            peek_precedence = precedence(token.kind);
            if peek_precedence == u32::MAX {
                let name = right.identifier.clone();
                let current = token.clone();
                self.unexpected_operator(name, &current);
                return Err(());
            }
            let right_precedence = if operator == TokenKind::Question {
                precedence(TokenKind::Comma)
            } else {
                this_precedence + 1
            };
            if peek_precedence >= right_precedence {
                self.evaluate_subexpression(&mut right, right_precedence, token, right_live)?;
                peek_precedence = precedence(token.kind);
            }
            let mut result_unsigned = false;
            match operator {
                TokenKind::Question
                | TokenKind::LessLess
                | TokenKind::GreaterGreater
                | TokenKind::Comma
                | TokenKind::PipePipe
                | TokenKind::AmpAmp => {}
                _ => {
                    result_unsigned = left.unsigned || right.unsigned;
                    if live && result_unsigned {
                        if !left.unsigned && left.is_negative() {
                            let text = format!("{} to {}", left.decimal(false), left.decimal(true));
                            self.report_token(
                                Message::ConvertToPositive,
                                &operator_token,
                                vec!["0".to_string(), text],
                            );
                        }
                        if !right.unsigned && right.is_negative() {
                            let text =
                                format!("{} to {}", right.decimal(false), right.decimal(true));
                            self.report_token(
                                Message::ConvertToPositive,
                                &operator_token,
                                vec!["1".to_string(), text],
                            );
                        }
                    }
                    left.unsigned = result_unsigned;
                    right.unsigned = result_unsigned;
                }
            }
            let mut overflow = false;
            let result: ExpressionValue = match operator {
                TokenKind::Percent => {
                    if right.bits != 0 {
                        let bits = if result_unsigned {
                            left.bits % right.bits
                        } else {
                            (left.bits as i64).wrapping_rem(right.bits as i64) as u64
                        };
                        ExpressionValue::of(bits, result_unsigned)
                    } else if live {
                        self.report_token(Message::RemainderByZero, &operator_token, Vec::new());
                        return Err(());
                    } else {
                        ExpressionValue::of(0, result_unsigned)
                    }
                }
                TokenKind::Slash => {
                    if let Some(quotient) = left.bits.checked_div(right.bits) {
                        let bits = if result_unsigned {
                            quotient
                        } else {
                            let (value, wrapped) =
                                (left.bits as i64).overflowing_div(right.bits as i64);
                            overflow = wrapped;
                            value as u64
                        };
                        ExpressionValue::of(bits, result_unsigned)
                    } else if live {
                        self.report_token(Message::DivisionByZero, &operator_token, Vec::new());
                        return Err(());
                    } else {
                        ExpressionValue::of(0, result_unsigned)
                    }
                }
                TokenKind::Star => {
                    let bits = if result_unsigned {
                        left.bits.wrapping_mul(right.bits)
                    } else {
                        let (value, wrapped) =
                            (left.bits as i64).overflowing_mul(right.bits as i64);
                        overflow = wrapped;
                        value as u64
                    };
                    ExpressionValue::of(bits, result_unsigned)
                }
                TokenKind::LessLess => {
                    let amount = right.bits;
                    if left.unsigned {
                        if amount >= 64 {
                            overflow = true;
                            ExpressionValue::of(0, true)
                        } else {
                            let shifted = left.bits << amount;
                            if amount > 0 && (left.bits >> (64 - amount)) != 0 {
                                overflow = true;
                            }
                            ExpressionValue::of(shifted, true)
                        }
                    } else if amount >= 64 {
                        overflow = true;
                        ExpressionValue::of(0, false)
                    } else {
                        let value = left.bits as i64;
                        let shifted = value.wrapping_shl(amount as u32);
                        let leading = if value < 0 {
                            value.leading_ones()
                        } else {
                            value.leading_zeros()
                        };
                        if u64::from(leading) <= amount {
                            overflow = true;
                        }
                        ExpressionValue::of(shifted as u64, false)
                    }
                }
                TokenKind::GreaterGreater => {
                    let mut amount = if !right.unsigned && (right.bits as i64) < 0 {
                        u64::MAX
                    } else {
                        right.bits
                    };
                    if amount >= 64 {
                        overflow = true;
                        amount = 63;
                    }
                    let bits = if left.unsigned {
                        left.bits >> amount
                    } else {
                        ((left.bits as i64) >> amount) as u64
                    };
                    ExpressionValue::of(bits, left.unsigned)
                }
                TokenKind::Plus => {
                    if left.unsigned {
                        ExpressionValue::of(left.bits.wrapping_add(right.bits), true)
                    } else {
                        let (value, wrapped) =
                            (left.bits as i64).overflowing_add(right.bits as i64);
                        overflow = wrapped;
                        ExpressionValue::of(value as u64, false)
                    }
                }
                TokenKind::Minus => {
                    if left.unsigned {
                        ExpressionValue::of(left.bits.wrapping_sub(right.bits), true)
                    } else {
                        let (value, wrapped) =
                            (left.bits as i64).overflowing_sub(right.bits as i64);
                        overflow = wrapped;
                        ExpressionValue::of(value as u64, false)
                    }
                }
                TokenKind::LessEqual
                | TokenKind::Less
                | TokenKind::GreaterEqual
                | TokenKind::Greater => {
                    let ordering = if result_unsigned {
                        left.bits.cmp(&right.bits)
                    } else {
                        (left.bits as i64).cmp(&(right.bits as i64))
                    };
                    let value = match operator {
                        TokenKind::LessEqual => ordering.is_le(),
                        TokenKind::Less => ordering.is_lt(),
                        TokenKind::GreaterEqual => ordering.is_ge(),
                        _ => ordering.is_gt(),
                    };
                    ExpressionValue::truth(value)
                }
                TokenKind::ExclaimEqual => ExpressionValue::truth(left.bits != right.bits),
                TokenKind::EqualEqual => ExpressionValue::truth(left.bits == right.bits),
                TokenKind::Amp => ExpressionValue::of(left.bits & right.bits, result_unsigned),
                TokenKind::Caret => ExpressionValue::of(left.bits ^ right.bits, result_unsigned),
                TokenKind::Pipe => ExpressionValue::of(left.bits | right.bits, result_unsigned),
                TokenKind::AmpAmp => ExpressionValue::truth(!left.is_zero() && !right.is_zero()),
                TokenKind::PipePipe => ExpressionValue::truth(!left.is_zero() || !right.is_zero()),
                TokenKind::Comma => {
                    if !self.language.at_least(CStandard::C99) || live {
                        self.report_token(Message::CommaInIfOperand, &operator_token, Vec::new());
                    }
                    right
                }
                TokenKind::Question => {
                    if token.kind != TokenKind::Colon {
                        let current = token.clone();
                        self.report_token(Message::Expected, &current, vec!["':'".to_string()]);
                        self.report_token(
                            Message::ToMatch,
                            &operator_token,
                            vec!["'?'".to_string()],
                        );
                        return Err(());
                    }
                    *token = self.lex_non_comment();
                    let after_live = live && left.is_zero();
                    let mut after = self.evaluate_value(token, after_live)?;
                    self.evaluate_subexpression(&mut after, this_precedence, token, after_live)?;
                    let unsigned = right.unsigned || after.unsigned;
                    let chosen = if !left.is_zero() {
                        right.bits
                    } else {
                        after.bits
                    };
                    peek_precedence = precedence(token.kind);
                    ExpressionValue::of(chosen, unsigned)
                }
                TokenKind::Colon => {
                    self.report_token(Message::ColonWithoutQuestion, &operator_token, Vec::new());
                    return Err(());
                }
                _ => return Err(()),
            };
            if overflow && live {
                self.report_token(Message::ExpressionOverflow, &operator_token, Vec::new());
            }
            left.bits = result.bits;
            left.unsigned = result.unsigned;
        }
    }

    fn parse_number(&mut self, token: &Token, report: bool) -> Result<NumberValue, ()> {
        let text = token.spelling.clone();
        let bytes: &[u8] = &text;
        let length = bytes.len();
        let byte = |at: usize| -> u8 { bytes.get(at).copied().unwrap_or(0) };
        let mut at: usize;
        let mut radix: u32;
        let mut digits_begin = 0usize;
        let mut saw_period = false;
        let mut saw_exponent = false;
        let is_separator = |value: u8| value == b'\'';
        let skip_digits = |mut position: usize, radix: u32| -> usize {
            while position < length {
                let value = bytes[position];
                let ok = match radix {
                    2 => value == b'0' || value == b'1',
                    8 => (b'0'..=b'7').contains(&value),
                    16 => value.is_ascii_hexdigit(),
                    _ => value.is_ascii_digit(),
                };
                if !ok && !is_separator(value) {
                    break;
                }
                position += 1;
            }
            position
        };
        let fail = |preprocessor: &mut Preprocessor, message: Message, arguments: Vec<String>| {
            if report {
                preprocessor.report_token(message, token, arguments);
            }
        };
        if byte(0) == b'0' {
            at = 1;
            let marker = byte(1);
            if (marker == b'x' || marker == b'X')
                && (byte(2).is_ascii_hexdigit() || byte(2) == b'.')
            {
                at = 2;
                radix = 16;
                digits_begin = at;
                at = skip_digits(at, 16);
                let mut significand = at > digits_begin;
                if byte(at) == b'.' {
                    at += 1;
                    saw_period = true;
                    let fraction = at;
                    at = skip_digits(at, 16);
                    if at > fraction {
                        significand = true;
                    }
                }
                if !significand {
                    fail(
                        self,
                        Message::HexConstantRequires,
                        vec!["0".to_string(), "1".to_string()],
                    );
                    return Err(());
                }
                if byte(at) == b'p' || byte(at) == b'P' {
                    at += 1;
                    saw_exponent = true;
                    if byte(at) == b'+' || byte(at) == b'-' {
                        at += 1;
                    }
                    let first = at;
                    at = skip_digits(at, 10);
                    if at == first {
                        fail(self, Message::ExponentHasNoDigits, Vec::new());
                        return Err(());
                    }
                    if !self.language.at_least(CStandard::C99) && report {
                        self.report_token(Message::HexFloatExtension, token, Vec::new());
                    }
                } else if saw_period {
                    fail(
                        self,
                        Message::HexConstantRequires,
                        vec!["0".to_string(), "0".to_string()],
                    );
                    return Err(());
                }
            } else if (marker == b'b' || marker == b'B') && (byte(2) == b'0' || byte(2) == b'1') {
                if report {
                    let message = if self.language.at_least(CStandard::C23) {
                        Message::BinaryLiteralCompatibility
                    } else {
                        Message::BinaryLiteralExtension
                    };
                    self.report_token(message, token, Vec::new());
                }
                at = 2;
                radix = 2;
                digits_begin = at;
                at = skip_digits(at, 2);
                if at < length && byte(at).is_ascii_hexdigit() {
                    fail(
                        self,
                        Message::InvalidDigit,
                        vec![(byte(at) as char).to_string(), "2".to_string()],
                    );
                    return Err(());
                }
            } else if (marker == b'o' || marker == b'O') && (b'0'..=b'7').contains(&byte(2)) {
                at = 2;
                radix = 8;
                digits_begin = at;
                at = skip_digits(at, 8);
                if at < length
                    && byte(at).is_ascii_hexdigit()
                    && byte(at) != b'e'
                    && byte(at) != b'E'
                {
                    fail(
                        self,
                        Message::InvalidDigit,
                        vec![(byte(at) as char).to_string(), "1".to_string()],
                    );
                    return Err(());
                }
            } else {
                radix = 8;
                let start = at;
                at = skip_digits(at, 8);
                if at != start {
                    digits_begin = start;
                }
                if at < length && byte(at).is_ascii_digit() {
                    let end_decimal = skip_digits(at, 10);
                    let after = byte(end_decimal);
                    if after == b'.' || after == b'e' || after == b'E' {
                        at = end_decimal;
                        radix = 10;
                    }
                }
                if at < length
                    && !self.parse_decimal_or_octal_tail(
                        token,
                        bytes,
                        &mut at,
                        &mut radix,
                        &mut saw_period,
                        &mut saw_exponent,
                        report,
                    )?
                {
                    return Err(());
                }
            }
        } else {
            radix = 10;
            at = skip_digits(0, 10);
            if at < length
                && !self.parse_decimal_or_octal_tail(
                    token,
                    bytes,
                    &mut at,
                    &mut radix,
                    &mut saw_period,
                    &mut saw_exponent,
                    report,
                )?
            {
                return Err(());
            }
        }
        let suffix_begin = at;
        let floating = saw_period || saw_exponent;
        let mut unsigned = false;
        let mut has_size = false;
        let mut long_long = false;
        let mut position = at;
        while position < length {
            let value = bytes[position];
            match value {
                b'f' | b'F' if floating && !has_size => {
                    has_size = true;
                    position += 1;
                    continue;
                }
                b'u' | b'U' if !floating && !unsigned => {
                    unsigned = true;
                    position += 1;
                    continue;
                }
                b'l' | b'L' if !has_size => {
                    has_size = true;
                    if byte(position + 1) == value {
                        if floating {
                            break;
                        }
                        long_long = true;
                        position += 2;
                    } else {
                        position += 1;
                    }
                    continue;
                }
                b'z' | b'Z' if !floating && !has_size => {
                    has_size = true;
                    position += 1;
                    continue;
                }
                b'i' | b'I' | b'j' | b'J' => {
                    position += 1;
                    continue;
                }
                b'w' | b'W'
                    if !floating
                        && !has_size
                        && ((value == b'w' && byte(position + 1) == b'b')
                            || (value == b'W' && byte(position + 1) == b'B')) =>
                {
                    has_size = true;
                    position += 2;
                    continue;
                }
                _ => break,
            }
        }
        if position != length {
            let suffix = String::from_utf8_lossy(&bytes[suffix_begin..]).into_owned();
            fail(
                self,
                Message::InvalidSuffix,
                vec![suffix, if floating { "1" } else { "0" }.to_string()],
            );
            return Err(());
        }
        if floating {
            return Ok(NumberValue::Floating);
        }
        let mut value: u64 = 0;
        let mut overflow = false;
        for &digit in &bytes[digits_begin..suffix_begin] {
            if is_separator(digit) {
                continue;
            }
            let Some(number) = hex_value(digit) else {
                continue;
            };
            let (multiplied, first) = value.overflowing_mul(u64::from(radix));
            let (added, second) = multiplied.overflowing_add(u64::from(number));
            overflow |= first || second;
            value = added;
        }
        Ok(NumberValue::Integer {
            value,
            unsigned,
            radix,
            long_long,
            overflow,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn parse_decimal_or_octal_tail(
        &mut self,
        token: &Token,
        bytes: &[u8],
        at: &mut usize,
        radix: &mut u32,
        saw_period: &mut bool,
        saw_exponent: &mut bool,
        report: bool,
    ) -> Result<bool, ()> {
        let length = bytes.len();
        let byte = |position: usize| -> u8 { bytes.get(position).copied().unwrap_or(0) };
        let skip_decimal = |mut position: usize| -> usize {
            while position < length
                && (bytes[position].is_ascii_digit() || bytes[position] == b'\'')
            {
                position += 1;
            }
            position
        };
        let current = byte(*at);
        if current.is_ascii_hexdigit() && current != b'e' && current != b'E' {
            if report {
                self.report_token(
                    Message::InvalidDigit,
                    token,
                    vec![
                        (current as char).to_string(),
                        if *radix == 8 { "1" } else { "0" }.to_string(),
                    ],
                );
            }
            return Ok(false);
        }
        if byte(*at) == b'.' {
            *at += 1;
            *radix = 10;
            *saw_period = true;
            *at = skip_decimal(*at);
        }
        if byte(*at) == b'e' || byte(*at) == b'E' {
            *at += 1;
            *radix = 10;
            *saw_exponent = true;
            if byte(*at) == b'+' || byte(*at) == b'-' {
                *at += 1;
            }
            let first = *at;
            let end = skip_decimal(*at);
            if bytes[first..end].iter().any(|value| value.is_ascii_digit()) {
                *at = end;
            } else {
                if report {
                    self.report_token(Message::ExponentHasNoDigits, token, Vec::new());
                }
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn parse_character(
        &mut self,
        token: &Token,
        prefix: LiteralPrefix,
    ) -> Result<ExpressionValue, ()> {
        let spelling = token.spelling.clone();
        let mut begin = match prefix {
            LiteralPrefix::Plain => 0,
            LiteralPrefix::Utf8 => 2,
            _ => 1,
        };
        if spelling.get(begin) != Some(&b'\'') {
            return Err(());
        }
        begin += 1;
        let end = spelling.len().saturating_sub(1);
        let largest: u64 = match prefix {
            LiteralPrefix::Wide => 0xFFFF_FFFF,
            LiteralPrefix::Utf8 => 0x7F,
            LiteralPrefix::Utf16 => 0xFFFF,
            LiteralPrefix::Utf32 => 0x10FFFF,
            LiteralPrefix::Plain => 0x7F,
        };
        let width = match prefix {
            LiteralPrefix::Plain | LiteralPrefix::Utf8 => 8u32,
            LiteralPrefix::Utf16 => 16,
            LiteralPrefix::Wide | LiteralPrefix::Utf32 => 32,
        };
        let mut values: Vec<u64> = Vec::new();
        let mut failed = false;
        let mut at = begin;
        while at < end {
            if spelling[at] != b'\\' {
                let start = at;
                while at < end && spelling[at] != b'\\' {
                    at += 1;
                }
                let chunk = &spelling[start..at];
                match std::str::from_utf8(chunk) {
                    Ok(text) => {
                        for character in text.chars() {
                            let value = u64::from(u32::from(character));
                            if value > largest {
                                failed = true;
                                self.report_token(Message::CharacterTooLarge, token, Vec::new());
                            }
                            values.push(value);
                        }
                    }
                    Err(_) => {
                        if prefix == LiteralPrefix::Plain {
                            self.report_token(
                                Message::BadCharacterEncodingWarning,
                                token,
                                Vec::new(),
                            );
                            values.extend(chunk.iter().map(|value| u64::from(*value)));
                        } else {
                            self.report_token(Message::BadCharacterEncoding, token, Vec::new());
                            failed = true;
                        }
                    }
                }
                continue;
            }
            let next = spelling.get(at + 1).copied().unwrap_or(0);
            if next == b'u' || next == b'U' {
                let digits = if next == b'u' { 4 } else { 8 };
                let start = at + 2;
                let mut value = 0u64;
                let mut count = 0;
                while count < digits
                    && start + count < end
                    && spelling[start + count].is_ascii_hexdigit()
                {
                    value = value * 16 + u64::from(hex_value(spelling[start + count]).unwrap_or(0));
                    count += 1;
                }
                if count < digits {
                    self.report_token(Message::IncompleteUniversalCharacterName, token, Vec::new());
                    failed = true;
                } else if (0xD800..=0xDFFF).contains(&value) || value > 0x10FFFF {
                    self.report_token(Message::InvalidUniversalCharacter, token, Vec::new());
                    failed = true;
                } else if value < 0xA0 && value != 0x24 && value != 0x40 && value != 0x60 {
                    if value < 0x20 || (0x7F..0xA0).contains(&value) {
                        self.report_token(Message::UniversalCharacterControl, token, Vec::new());
                    } else {
                        self.report_token(
                            Message::UniversalCharacterBasicSet,
                            token,
                            vec![(value as u8 as char).to_string()],
                        );
                    }
                    failed = true;
                } else if value > largest {
                    self.report_token(Message::CharacterTooLarge, token, Vec::new());
                    failed = true;
                }
                values.push(value);
                at = start + count;
                continue;
            }
            at += 1;
            let escape = spelling.get(at).copied().unwrap_or(0);
            at += 1;
            let value: u64 = match escape {
                b'\\' | b'\'' | b'"' | b'?' => u64::from(escape),
                b'a' => 7,
                b'b' => 8,
                b'e' | b'E' => {
                    self.report_token(
                        Message::NonstandardEscape,
                        token,
                        vec![(escape as char).to_string()],
                    );
                    27
                }
                b'f' => 12,
                b'n' => 10,
                b'r' => 13,
                b't' => 9,
                b'v' => 11,
                b'x' => {
                    if at >= end || !spelling[at].is_ascii_hexdigit() {
                        self.report_token(Message::HexEscapeNoDigits, token, vec!["x".to_string()]);
                        values.push(0);
                        continue;
                    }
                    let mut value = 0u64;
                    let mut overflow = false;
                    while at < end && spelling[at].is_ascii_hexdigit() {
                        if value & 0xF000_0000 != 0 {
                            overflow = true;
                        }
                        value = ((value << 4) | u64::from(hex_value(spelling[at]).unwrap_or(0)))
                            & 0xFFFF_FFFF;
                        at += 1;
                    }
                    if width != 32 && (value >> width) != 0 {
                        overflow = true;
                        value &= (1u64 << width) - 1;
                    }
                    if overflow && !failed {
                        failed = true;
                        self.report_token(Message::EscapeTooLarge, token, vec!["0".to_string()]);
                    }
                    value
                }
                b'0'..=b'7' => {
                    let mut value = u64::from(escape - b'0');
                    let mut count = 1;
                    while count < 3 && at < end && (b'0'..=b'7').contains(&spelling[at]) {
                        value = (value << 3) | u64::from(spelling[at] - b'0');
                        at += 1;
                        count += 1;
                    }
                    if width != 32 && (value >> width) != 0 {
                        self.report_token(Message::EscapeTooLarge, token, vec!["1".to_string()]);
                        value &= (1u64 << width) - 1;
                    }
                    value
                }
                b'(' | b'{' | b'[' | b'%' => {
                    self.report_token(
                        Message::NonstandardEscape,
                        token,
                        vec![(escape as char).to_string()],
                    );
                    u64::from(escape)
                }
                _ => {
                    let shown = if (0x20..0x7f).contains(&escape) {
                        (escape as char).to_string()
                    } else {
                        format!("x{escape:X}")
                    };
                    self.report_token(Message::UnknownEscape, token, vec![shown]);
                    u64::from(escape)
                }
            };
            values.push(value);
        }
        if values.len() > 1 {
            if prefix == LiteralPrefix::Plain && values.len() == 4 {
                self.report_token(Message::FourCharacterConstant, token, Vec::new());
            } else if prefix == LiteralPrefix::Plain {
                self.report_token(Message::MultiCharacterConstant, token, Vec::new());
            } else {
                let which = if prefix == LiteralPrefix::Wide {
                    "0"
                } else {
                    "1"
                };
                self.report_token(
                    Message::MultipleCharactersInWide,
                    token,
                    vec![which.to_string()],
                );
                failed = true;
            }
        }
        if failed {
            return Err(());
        }
        let mut value: u64 = 0;
        let mut too_long = false;
        if prefix == LiteralPrefix::Plain && values.len() > 1 {
            for item in &values {
                if (value as u32).leading_zeros() < 8 {
                    too_long = true;
                }
                value = ((value << 8) + (item & 0xFF)) & 0xFFFF_FFFF;
            }
            if too_long {
                self.report_token(Message::CharacterConstantTooLong, token, Vec::new());
            }
            let signed = value as u32 as i32 as i64;
            return Ok(ExpressionValue::signed(signed));
        }
        if let Some(last) = values.last() {
            value = *last;
        }
        match prefix {
            LiteralPrefix::Plain => {
                if values.len() == 1 && value & 0x80 != 0 && self.language.char_is_signed {
                    return Ok(ExpressionValue::signed(value as u8 as i8 as i64));
                }
                Ok(ExpressionValue::of(value, !self.language.char_is_signed))
            }
            LiteralPrefix::Wide => {
                if self.options.target.wchar_signed() {
                    Ok(ExpressionValue::signed(value as u32 as i32 as i64))
                } else {
                    Ok(ExpressionValue::of(value, true))
                }
            }
            _ => Ok(ExpressionValue::of(value, true)),
        }
    }
}

fn precedence(kind: TokenKind) -> u32 {
    match kind {
        TokenKind::Percent | TokenKind::Slash | TokenKind::Star => 14,
        TokenKind::Plus | TokenKind::Minus => 13,
        TokenKind::LessLess | TokenKind::GreaterGreater => 12,
        TokenKind::LessEqual | TokenKind::Less | TokenKind::GreaterEqual | TokenKind::Greater => 11,
        TokenKind::ExclaimEqual | TokenKind::EqualEqual => 10,
        TokenKind::Amp => 9,
        TokenKind::Caret => 8,
        TokenKind::Pipe => 7,
        TokenKind::AmpAmp => 6,
        TokenKind::PipePipe => 5,
        TokenKind::Question => 4,
        TokenKind::Comma => 3,
        TokenKind::Colon => 2,
        TokenKind::RightParen | TokenKind::EndOfDirective => 0,
        _ => u32::MAX,
    }
}

fn edit_distance(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    for (index, a) in left.iter().enumerate() {
        let mut current = vec![index + 1];
        for (other, b) in right.iter().enumerate() {
            let cost = usize::from(a != b);
            let value = (previous[other] + cost)
                .min(previous[other + 1] + 1)
                .min(current[other] + 1);
            current.push(value);
        }
        previous = current;
    }
    previous[right.len()]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IncludeKind {
    Include,
    IncludeNext,
    Import,
}

const ALWAYS_FEATURES: &[&str] = &[
    "arc_cf_code_audited",
    "assume_nonnull",
    "attribute_analyzer_noreturn",
    "attribute_availability",
    "attribute_availability_app_extension",
    "attribute_availability_driverkit",
    "attribute_availability_in_templates",
    "attribute_availability_swift",
    "attribute_availability_tvos",
    "attribute_availability_watchos",
    "attribute_availability_with_message",
    "attribute_availability_with_replacement",
    "attribute_availability_with_strict",
    "attribute_availability_with_version_underscores",
    "attribute_cf_consumed",
    "attribute_cf_returns_not_retained",
    "attribute_cf_returns_on_parameters",
    "attribute_cf_returns_retained",
    "attribute_deprecated_with_message",
    "attribute_deprecated_with_replacement",
    "attribute_diagnose_if_objc",
    "attribute_ext_vector_type",
    "attribute_ns_consumed",
    "attribute_ns_consumes_self",
    "attribute_ns_returns_not_retained",
    "attribute_ns_returns_retained",
    "attribute_objc_ivar_unused",
    "attribute_objc_method_family",
    "attribute_overloadable",
    "attribute_unavailable_with_message",
    "attribute_unused_on_fields",
    "c_thread_safety_attributes",
    "clang_atomic_attributes",
    "enumerator_attributes",
    "ext_vector_type_boolean",
    "nullability",
    "nullability_nullable_result",
    "nullability_on_arrays",
    "nullability_on_classes",
    "objc_arc_fields",
    "objc_bool",
    "objc_bridge_id",
    "objc_bridge_id_on_typedefs",
    "objc_property_explicit_atomic",
    "objc_protocol_qualifier_mangling",
    "ownership_holds",
    "ownership_returns",
    "ownership_takes",
    "pragma_stdc_cx_limited_range",
    "tls",
    "undefined_behavior_sanitizer_finegrained_feature_checks",
];

const ALWAYS_EXTENSIONS: &[&str] = &[
    "c_alignas",
    "c_alignof",
    "c_atomic",
    "c_attributes",
    "c_fixed_enum",
    "c_generic_selection_with_controlling_type",
    "c_generic_selections",
    "c_static_assert",
    "c_thread_local",
    "cxx_binary_literals",
    "cxx_fixed_enum",
    "cxx_type_aware_allocators",
    "gnu_asm",
    "gnu_asm_goto_with_outputs",
    "gnu_asm_goto_with_outputs_full",
    "matrix_types_scalar_division",
    "objc_c_static_assert",
    "overloadable_unmarked",
    "pragma_clang_attribute_external_declaration",
    "pragma_clang_attribute_namespaces",
    "statement_attributes_with_gnu_syntax",
];

const GNU_ATTRIBUTES: &[&str] = &[
    "alias",
    "aligned",
    "alloc_align",
    "alloc_size",
    "always_inline",
    "annotate",
    "artificial",
    "assume_aligned",
    "availability",
    "blocks",
    "cleanup",
    "cold",
    "common",
    "const",
    "constructor",
    "deprecated",
    "destructor",
    "diagnose_if",
    "enable_if",
    "error",
    "external_source_symbol",
    "fallthrough",
    "flag_enum",
    "flatten",
    "format",
    "format_arg",
    "gnu_inline",
    "hot",
    "ifunc",
    "leaf",
    "malloc",
    "may_alias",
    "mode",
    "naked",
    "no_instrument_function",
    "no_sanitize",
    "no_stack_protector",
    "nocommon",
    "noduplicate",
    "noinline",
    "nonnull",
    "noreturn",
    "nothrow",
    "objc_boxable",
    "optnone",
    "overloadable",
    "packed",
    "pass_object_size",
    "pure",
    "regparm",
    "retain",
    "returns_nonnull",
    "returns_twice",
    "section",
    "sentinel",
    "transparent_union",
    "unavailable",
    "uninitialized",
    "unused",
    "used",
    "vector_size",
    "visibility",
    "warn_unused_result",
    "warning",
    "weak",
    "weakref",
    "zero_call_used_regs",
];

const C23_ATTRIBUTES: &[(&str, &str)] = &[
    ("_Noreturn", "202202"),
    ("deprecated", "201904"),
    ("fallthrough", "201910"),
    ("maybe_unused", "202106"),
    ("nodiscard", "202003"),
    ("noreturn", "202202"),
    ("reproducible", "202207"),
    ("unsequenced", "202207"),
];

const PARSER_PRAGMAS: &[&str] = &[
    "align",
    "comment",
    "detect_mismatch",
    "fenv_access",
    "float_control",
    "function",
    "init_seg",
    "intrinsic",
    "ms_struct",
    "nounroll",
    "nounroll_and_jam",
    "optimize",
    "options",
    "pack",
    "redefine_extname",
    "section",
    "unroll",
    "unroll_and_jam",
    "unused",
    "weak",
];

const CLANG_PARSER_PRAGMAS: &[&str] = &[
    "attribute",
    "fp",
    "loop",
    "max_tokens_here",
    "max_tokens_total",
    "optimize",
    "riscv",
    "section",
];

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = (shifted - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u32;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    } as u32;
    if month <= 2 {
        year += 1;
    }
    (year, month, day)
}

const MONTH_NAMES: &[&str] = &[
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const WEEKDAY_NAMES: &[&str] = &["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];

fn date_parts(seconds: u64) -> (String, String, String) {
    let days = (seconds / 86_400) as i64;
    let rest = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let month_name = MONTH_NAMES[(month as usize).saturating_sub(1).min(11)];
    let date = format!("\"{month_name} {day:>2} {year}\"");
    let time = format!(
        "\"{:02}:{:02}:{:02}\"",
        rest / 3600,
        (rest / 60) % 60,
        rest % 60
    );
    let weekday = WEEKDAY_NAMES[(days.rem_euclid(7)) as usize];
    let stamp = format!(
        "\"{weekday} {month_name} {day:>2} {:02}:{:02}:{:02} {year}\"",
        rest / 3600,
        (rest / 60) % 60,
        rest % 60
    );
    (date, time, stamp)
}

fn printable_escape(text: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    for &byte in text {
        if (0x20..0x7f).contains(&byte) && byte != b'\\' && byte != b'"' {
            out.push(byte);
        } else {
            out.push(b'\\');
            out.push(b'0' + ((byte >> 6) & 7));
            out.push(b'0' + ((byte >> 3) & 7));
            out.push(b'0' + (byte & 7));
        }
    }
    out
}

fn evaluate_string_literal(spelling: &[u8]) -> Option<Vec<u8>> {
    let start = spelling.iter().position(|byte| *byte == b'"')?;
    if spelling.len() < start + 2 || *spelling.last()? != b'"' {
        return None;
    }
    let body = &spelling[start + 1..spelling.len() - 1];
    let mut out = Vec::with_capacity(body.len());
    let mut at = 0usize;
    while at < body.len() {
        if body[at] != b'\\' || at + 1 >= body.len() {
            out.push(body[at]);
            at += 1;
            continue;
        }
        at += 1;
        let escape = body[at];
        at += 1;
        match escape {
            b'n' => out.push(b'\n'),
            b't' => out.push(b'\t'),
            b'r' => out.push(b'\r'),
            b'a' => out.push(7),
            b'b' => out.push(8),
            b'f' => out.push(12),
            b'v' => out.push(11),
            b'e' | b'E' => out.push(27),
            b'x' => {
                let mut value = 0u32;
                while at < body.len() && body[at].is_ascii_hexdigit() {
                    value = value
                        .wrapping_mul(16)
                        .wrapping_add(hex_value(body[at]).unwrap_or(0));
                    at += 1;
                }
                out.push(value as u8);
            }
            b'0'..=b'7' => {
                let mut value = u32::from(escape - b'0');
                let mut count = 1;
                while count < 3 && at < body.len() && (b'0'..=b'7').contains(&body[at]) {
                    value = value * 8 + u32::from(body[at] - b'0');
                    at += 1;
                    count += 1;
                }
                out.push(value as u8);
            }
            b'u' | b'U' => {
                let digits = if escape == b'u' { 4 } else { 8 };
                let mut value = 0u32;
                let mut count = 0;
                while count < digits && at < body.len() && body[at].is_ascii_hexdigit() {
                    value = value * 16 + hex_value(body[at]).unwrap_or(0);
                    at += 1;
                    count += 1;
                }
                encode_utf8(value, &mut out);
            }
            other => out.push(other),
        }
    }
    Some(out)
}

impl<'files> Preprocessor<'files> {
    fn lex_header_name(&mut self, allow_expansion: bool) -> Option<Token> {
        let top_is_file = self.top_is_file();
        if top_is_file {
            if let Some(Source::File(file)) = self.stack.last_mut() {
                file.lexer.parsing_filename = true;
            }
        }
        let depth = self.stack.len();
        let mut token = if allow_expansion {
            self.lex()
        } else {
            self.lex_unexpanded()
        };
        if top_is_file && self.stack.len() >= depth {
            if let Some(Source::File(file)) = self.stack.get_mut(depth - 1) {
                file.lexer.parsing_filename = false;
            }
        }
        if token.kind == TokenKind::Less && allow_expansion {
            let open = token.clone();
            let mut buffer = vec![b'<'];
            while token.kind != TokenKind::Greater {
                token = self.lex();
                if matches!(token.kind, TokenKind::EndOfDirective | TokenKind::EndOfFile) {
                    self.report_token(Message::Expected, &token, vec!["'>'".to_string()]);
                    self.report_token(Message::ToMatch, &open, vec!["'<'".to_string()]);
                    return Some(token);
                }
                if token.has_leading_space() {
                    buffer.push(b' ');
                }
                buffer.extend_from_slice(&token.spelling);
            }
            let mut header = self.make_token(TokenKind::HeaderName, buffer, (open.site, open.site));
            header.set(START_OF_LINE, open.at_start_of_line());
            header.set(LEADING_SPACE, open.has_leading_space());
            return Some(header);
        }
        if token.kind == TokenKind::String(LiteralPrefix::Plain)
            && allow_expansion
            && token.spelling.len() >= 2
        {
            token.kind = TokenKind::HeaderName;
        }
        Some(token)
    }

    fn include_filename(token: &Token) -> Option<(String, bool)> {
        let spelling = &token.spelling;
        if spelling.len() < 2 {
            return None;
        }
        let angled = match (spelling[0], spelling[spelling.len() - 1]) {
            (b'<', b'>') => true,
            (b'"', b'"') => false,
            _ => return None,
        };
        let inner = &spelling[1..spelling.len() - 1];
        Some((String::from_utf8_lossy(inner).into_owned(), angled))
    }

    fn handle_include(&mut self, _hash: &Token, directive: &Token, kind: IncludeKind) {
        let Some(filename_token) = self.lex_header_name(true) else {
            return;
        };
        if filename_token.kind != TokenKind::HeaderName {
            if filename_token.kind != TokenKind::EndOfDirective {
                self.report_token(Message::ExpectsFilename, &filename_token, Vec::new());
                self.discard_until_end_of_directive();
            } else {
                self.report_token(Message::ExpectsFilename, &filename_token, Vec::new());
            }
            return;
        }
        let Some((name, angled)) = Self::include_filename(&filename_token) else {
            self.report_token(Message::ExpectsFilename, &filename_token, Vec::new());
            self.discard_until_end_of_directive();
            return;
        };
        let directive_name = directive.text().to_string();
        self.check_end_of_directive(&directive_name, true);
        if name.is_empty() {
            self.report_token(Message::EmptyFilename, &filename_token, Vec::new());
            return;
        }
        let mut start_index = None;
        let mut include_next = false;
        if kind == IncludeKind::IncludeNext {
            include_next = true;
            let current = self.real_file_index();
            let primary = self.in_primary_file();
            let (found_relative, search) = current
                .and_then(|index| match &self.stack[index] {
                    Source::File(file) => Some((file.found_by_relative, file.search_index)),
                    _ => None,
                })
                .unwrap_or_default();
            if primary && self.language.header_file {
                include_next = false;
            } else if primary {
                self.report_token(Message::IncludeNextInPrimary, directive, Vec::new());
                include_next = false;
            } else if found_relative || search.is_none() {
                self.report_token(Message::IncludeNextAbsolutePath, directive, Vec::new());
                include_next = false;
            } else {
                start_index = search.map(|index| index + 1);
            }
        }
        let found = self.lookup_file(&name, angled, start_index, include_next);
        let Some((path, bytes, search_index, relative, character)) = found else {
            if angled && self.lookup_file(&name, false, None, false).is_some() {
                self.report_token(
                    Message::FileNotFoundAngled,
                    &filename_token,
                    vec![
                        name,
                        if kind == IncludeKind::Import {
                            "1"
                        } else {
                            "0"
                        }
                        .to_string(),
                    ],
                );
            } else {
                self.report_token(Message::FileNotFound, &filename_token, vec![name]);
            }
            return;
        };
        let key = self.canonical_key(&path);
        if self.reached_include_limit && self.included_files.contains(&key) {
            return;
        }
        if self.once_files.contains(&key) {
            return;
        }
        if kind == IncludeKind::Import {
            self.once_files.insert(key.clone());
        }
        if let Some(guard) = self.include_guards.get(&key).cloned() {
            if self.macros.contains_key(&guard) {
                return;
            }
        }
        let character = if relative {
            let current_character = self
                .current_file_index()
                .and_then(|index| match &self.stack[index] {
                    Source::File(file) => Some(
                        self.sources
                            .presumed(file.lexer.place(file.lexer.position))
                            .character,
                    ),
                    _ => None,
                })
                .unwrap_or(FileCharacter::User);
            if current_character != FileCharacter::User {
                current_character
            } else {
                character
            }
        } else {
            character
        };
        if self.include_depth + 1 >= self.options.maximum_include_depth {
            self.report_token(Message::IncludeTooDeep, &filename_token, Vec::new());
            self.reached_include_limit = true;
            return;
        }
        self.included_files.insert(key.clone());
        let directory = Self::directory_of(&path);
        self.include_depth += 1;
        let site = filename_token.site;
        self.enter_buffer(
            &path,
            bytes,
            BufferKind::File,
            Some(site),
            character,
            SearchOrigin {
                directory,
                search_index,
                found_by_relative: relative,
            },
        );
        if let Some(Source::File(file)) = self.stack.last_mut() {
            file.guard_key = Some(key);
        }
    }

    fn canonical_key(&self, path: &str) -> String {
        let full = if path.starts_with('/') || self.options.working_directory.is_empty() {
            path.to_string()
        } else {
            Self::join_path(&self.options.working_directory, path)
        };
        let mut parts: Vec<&str> = Vec::new();
        for part in full.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    parts.pop();
                }
                other => parts.push(other),
            }
        }
        format!("/{}", parts.join("/"))
    }

    fn line_value(&mut self, token: &Token, message: Message, gnu: bool) -> Option<u32> {
        if token.kind != TokenKind::Number {
            self.report_token(message, token, Vec::new());
            if token.kind != TokenKind::EndOfDirective {
                self.discard_until_end_of_directive();
            }
            return None;
        }
        let mut value: u32 = 0;
        for &byte in token.spelling.iter() {
            if byte == b'\'' {
                continue;
            }
            if !byte.is_ascii_digit() {
                self.report_token(
                    Message::LineDigitSequence,
                    token,
                    vec![if gnu { "1" } else { "0" }.to_string()],
                );
                self.discard_until_end_of_directive();
                return None;
            }
            let Some(next) = value
                .checked_mul(10)
                .and_then(|scaled| scaled.checked_add(u32::from(byte - b'0')))
            else {
                self.report_token(message, token, Vec::new());
                self.discard_until_end_of_directive();
                return None;
            };
            value = next;
        }
        if token.spelling.first() == Some(&b'0') && value != 0 {
            self.report_token(
                Message::LineDecimal,
                token,
                vec![if gnu { "1" } else { "0" }.to_string()],
            );
        }
        Some(value)
    }

    fn current_position(&self) -> Option<Place> {
        let index = self.current_file_index()?;
        match &self.stack[index] {
            Source::File(file) => Some(file.lexer.place(file.lexer.position)),
            _ => None,
        }
    }

    fn handle_line(&mut self) {
        let digit = self.lex();
        let Some(line) = self.line_value(&digit, Message::LineRequiresInteger, false) else {
            return;
        };
        if line == 0 {
            self.report_token(Message::LineZero, &digit, Vec::new());
        }
        let limit: u64 = if self.language.at_least(CStandard::C99) {
            2_147_483_648
        } else {
            32_768
        };
        if u64::from(line) >= limit {
            self.report_token(Message::LineTooBig, &digit, vec![limit.to_string()]);
        }
        let string = self.lex();
        let mut name: Option<Rc<str>> = None;
        if string.kind == TokenKind::EndOfDirective {
        } else if string.kind != TokenKind::String(LiteralPrefix::Plain) {
            self.report_token(Message::LineInvalidFilename, &string, Vec::new());
            self.discard_until_end_of_directive();
            return;
        } else {
            let Some(value) = evaluate_string_literal(&string.spelling) else {
                self.discard_until_end_of_directive();
                return;
            };
            name = Some(Rc::from(String::from_utf8_lossy(&value).as_ref()));
            self.check_end_of_directive("line", true);
        }
        let character = self.sources.presumed(digit.site).character;
        if let Some(place) = self.current_position() {
            self.sources
                .add_line_entry(place, line, name, 0, character, digit.site);
            self.file_changed(place, ChangeReason::Rename);
        }
    }

    fn handle_digit_directive(&mut self, digit: Token) {
        let Some(line) = self.line_value(&digit, Message::LinemarkerRequiresInteger, true) else {
            return;
        };
        let string = self.lex();
        let mut entry = false;
        let mut exit = false;
        let mut name: Option<Rc<str>> = None;
        let mut character = FileCharacter::User;
        if string.kind == TokenKind::EndOfDirective {
            self.report_token(Message::GnuLineMarker, &string, Vec::new());
            character = self.sources.presumed(digit.site).character;
        } else if string.kind != TokenKind::String(LiteralPrefix::Plain) {
            self.report_token(Message::LinemarkerInvalidFilename, &string, Vec::new());
            self.discard_until_end_of_directive();
            return;
        } else {
            let Some(value) = evaluate_string_literal(&string.spelling) else {
                self.discard_until_end_of_directive();
                return;
            };
            let mut flag = self.lex();
            if flag.kind != TokenKind::EndOfDirective {
                let Some(mut value_flag) =
                    self.line_value(&flag, Message::LinemarkerInvalidFlag, false)
                else {
                    return;
                };
                if value_flag == 1 {
                    entry = true;
                    flag = self.lex();
                    if flag.kind != TokenKind::EndOfDirective {
                        let Some(next) =
                            self.line_value(&flag, Message::LinemarkerInvalidFlag, false)
                        else {
                            return;
                        };
                        value_flag = next;
                    } else {
                        value_flag = 0;
                    }
                } else if value_flag == 2 {
                    exit = true;
                    let presumed = self.sources.presumed(flag.site);
                    let valid = presumed
                        .include
                        .is_some_and(|include| include.buffer == flag.site.buffer);
                    if !valid {
                        self.report_token(Message::LinemarkerInvalidPop, &flag, Vec::new());
                        self.discard_until_end_of_directive();
                        return;
                    }
                    flag = self.lex();
                    if flag.kind != TokenKind::EndOfDirective {
                        let Some(next) =
                            self.line_value(&flag, Message::LinemarkerInvalidFlag, false)
                        else {
                            return;
                        };
                        value_flag = next;
                    } else {
                        value_flag = 0;
                    }
                }
                if value_flag != 0 {
                    if value_flag != 3 {
                        self.report_token(Message::LinemarkerInvalidFlag, &flag, Vec::new());
                        self.discard_until_end_of_directive();
                        return;
                    }
                    character = FileCharacter::System;
                    flag = self.lex();
                    if flag.kind != TokenKind::EndOfDirective {
                        let Some(next) =
                            self.line_value(&flag, Message::LinemarkerInvalidFlag, false)
                        else {
                            return;
                        };
                        if next != 4 {
                            self.report_token(Message::LinemarkerInvalidFlag, &flag, Vec::new());
                            self.discard_until_end_of_directive();
                            return;
                        }
                        character = FileCharacter::ExternCSystem;
                        let after = self.lex();
                        if after.kind != TokenKind::EndOfDirective {
                            self.report_token(Message::LinemarkerInvalidFlag, &after, Vec::new());
                            self.discard_until_end_of_directive();
                            return;
                        }
                    }
                }
            }
            if Some(digit.site.buffer) != self.builtin_buffer {
                self.report_token(Message::GnuLineMarker, &string, Vec::new());
            }
            if !(exit && value.is_empty()) {
                name = Some(Rc::from(String::from_utf8_lossy(&value).as_ref()));
            }
        }
        let entry_exit = if entry {
            1
        } else if exit {
            2
        } else {
            0
        };
        if let Some(place) = self.current_position() {
            self.sources
                .add_line_entry(place, line, name, entry_exit, character, digit.site);
            let reason = if entry {
                ChangeReason::Enter
            } else if exit {
                ChangeReason::Exit
            } else {
                ChangeReason::Rename
            };
            self.file_changed(place, reason);
        }
    }

    fn handle_user_diagnostic(&mut self, directive: &Token, warning: bool) {
        let text = match self.stack.last_mut() {
            Some(Source::File(file)) => file.lexer.read_to_end_of_line(),
            _ => Vec::new(),
        };
        if let Some(index) = self.current_file_index() {
            self.drain_lexer(index);
        }
        let end = self.lex_unexpanded();
        let _ = end;
        let mut start = 0usize;
        while start < text.len() && text[start] == b' ' {
            start += 1;
        }
        let message = String::from_utf8_lossy(&text[start..]).into_owned();
        let kind = if warning {
            Message::HashWarning
        } else {
            Message::HashError
        };
        self.report_token(kind, directive, vec![message]);
    }

    fn handle_ident(&mut self, directive: &Token) {
        self.report_token(Message::IdentDirective, directive, Vec::new());
        let string = self.lex();
        if !matches!(
            string.kind,
            TokenKind::String(LiteralPrefix::Plain) | TokenKind::String(LiteralPrefix::Wide)
        ) {
            self.report_token(Message::MalformedIdent, &string, Vec::new());
            if string.kind != TokenKind::EndOfDirective {
                self.discard_until_end_of_directive();
            }
            return;
        }
        self.check_end_of_directive("ident", false);
        let mut line = b"#ident ".to_vec();
        line.extend_from_slice(&string.spelling);
        self.print_directive_line(directive.site, &line, true);
    }

    fn finish_string_literal(
        &mut self,
        token: &mut Token,
        context: &str,
        allow_expansion: bool,
    ) -> Option<Vec<u8>> {
        if !matches!(token.kind, TokenKind::String(_)) {
            let current = token.clone();
            self.report_token(
                Message::ExpectedStringLiteral,
                &current,
                vec![context.to_string()],
            );
            return None;
        }
        let mut pieces = Vec::new();
        while matches!(token.kind, TokenKind::String(_)) {
            pieces.push(token.clone());
            *token = if allow_expansion {
                self.lex()
            } else {
                self.lex_unexpanded()
            };
        }
        let mut out = Vec::new();
        for piece in pieces {
            if let Some(value) = evaluate_string_literal(&piece.spelling) {
                out.extend_from_slice(&value);
            }
        }
        Some(out)
    }

    fn handle_pragma_directive(&mut self, introducer: &Token) {
        self.handle_pragma(introducer.site);
        if self.in_directive() {
            self.discard_until_end_of_directive();
        }
    }

    fn handle_pragma(&mut self, introducer: Place) {
        let _ = introducer;
        let first = self.lex_unexpanded();
        let name = if first.kind == TokenKind::Identifier {
            first.text().to_string()
        } else {
            String::new()
        };
        match name.as_str() {
            "once" => {
                self.check_end_of_directive("pragma once", false);
                self.pragma_once(&first);
            }
            "mark" => {
                if let Some(Source::File(file)) = self.stack.last_mut() {
                    file.lexer.read_to_end_of_line();
                }
            }
            "push_macro" => self.pragma_push_macro(&first, true),
            "pop_macro" => self.pragma_push_macro(&first, false),
            "message" => self.pragma_message(&first, 0, ""),
            "region" | "endregion" => {}
            "GCC" | "clang" => {
                let second = self.lex_unexpanded();
                let sub = if second.kind == TokenKind::Identifier {
                    second.text().to_string()
                } else {
                    String::new()
                };
                match sub.as_str() {
                    "poison" => self.pragma_poison(),
                    "system_header" => self.pragma_system_header(&second),
                    "dependency" => self.pragma_dependency(),
                    "diagnostic" => self.pragma_diagnostic(&second, &name),
                    "warning" if name == "GCC" => self.pragma_message(&second, 1, "GCC"),
                    "error" if name == "GCC" => self.pragma_message(&second, 2, "GCC"),
                    "__debug" if name == "clang" => self.pragma_debug(&second),
                    "deprecated"
                    | "restrict_expansion"
                    | "final"
                    | "assume_nonnull"
                    | "arc_cf_code_audited"
                    | "unsafe_buffer_usage"
                    | "module"
                    | "__set_pp_state"
                        if name == "clang" =>
                    {
                        if self.in_directive() {
                            self.discard_until_end_of_directive();
                        }
                    }
                    _ => {
                        let prefix = if name == "GCC" {
                            "#pragma GCC"
                        } else {
                            "#pragma clang"
                        };
                        let known = name == "clang" && CLANG_PARSER_PRAGMAS.contains(&sub.as_str())
                            || name == "GCC" && sub == "visibility";
                        let expand = self.language.microsoft_extensions;
                        self.unknown_pragma(prefix, second, known, expand);
                    }
                }
            }
            "omp" => self.unknown_pragma("#pragma omp", first, true, true),
            "STDC" => {
                let known = true;
                self.unknown_pragma("#pragma", first, known, false);
            }
            _ => {
                let known = PARSER_PRAGMAS.contains(&name.as_str());
                let expand = self.language.microsoft_extensions;
                self.unknown_pragma("#pragma", first, known, expand);
            }
        }
    }

    fn unknown_pragma(&mut self, prefix: &str, first: Token, known: bool, expand: bool) {
        if self.printer.is_some() {
            let site = first.site;
            self.printer_move_to_line(site, true);
            if let Some(printer) = &mut self.printer {
                printer.out.extend_from_slice(prefix.as_bytes());
                printer.tokens_on_line = true;
            }
            let mut token = first;
            if expand && token.kind == TokenKind::Identifier {
                let depth = self.stack.len();
                self.push_tokens(vec![token.clone()], false);
                token = self.lex();
                let _ = depth;
            }
            let mut is_first = true;
            while !matches!(token.kind, TokenKind::EndOfDirective | TokenKind::EndOfFile) {
                self.print_whitespace_before(&token, is_first, true);
                is_first = false;
                if let Some(printer) = &mut self.printer {
                    printer.out.extend_from_slice(&token.spelling);
                    printer.tokens_on_line = true;
                }
                token = if expand {
                    self.lex()
                } else {
                    self.lex_unexpanded()
                };
            }
            if let Some(printer) = &mut self.printer {
                printer.directive_on_line = true;
            }
            return;
        }
        if self.options.syntax_only && !known {
            self.report_token(Message::UnknownPragma, &first, Vec::new());
        }
        if first.kind != TokenKind::EndOfDirective && self.in_directive() {
            self.discard_until_end_of_directive();
        }
    }

    fn pragma_once(&mut self, token: &Token) {
        if self.in_primary_file() && !self.language.header_file {
            self.report_token(Message::PragmaOnceInMainFile, token, Vec::new());
            return;
        }
        let key = match self.real_file_index().map(|index| &self.stack[index]) {
            Some(Source::File(file)) => self.canonical_key(&file.path),
            _ => return,
        };
        self.once_files.insert(key);
    }

    fn pragma_poison(&mut self) {
        loop {
            self.set_raw(true);
            let token = self.lex_unexpanded();
            self.set_raw(false);
            if token.kind == TokenKind::EndOfDirective || token.kind == TokenKind::EndOfFile {
                return;
            }
            if token.kind != TokenKind::Identifier {
                self.report_token(Message::InvalidPoison, &token, Vec::new());
                return;
            }
            if self.poisoned.contains(&token.spelling) {
                continue;
            }
            if self.macros.contains_key(&token.spelling) {
                self.report_token(Message::PoisoningExistingMacro, &token, Vec::new());
            }
            self.poisoned.insert(token.spelling.clone());
        }
    }

    fn pragma_system_header(&mut self, token: &Token) {
        if self.in_primary_file() {
            self.report_token(Message::PragmaSystemHeaderInMainFile, token, Vec::new());
            return;
        }
        let presumed = self.sources.presumed(token.site);
        self.file_changed(token.site, ChangeReason::SystemHeader);
        if self.in_directive() {
            self.discard_until_end_of_directive();
        }
        if let Some(place) = self.current_position() {
            self.sources.add_line_entry(
                place,
                presumed.line + 1,
                Some(presumed.name.clone()),
                0,
                FileCharacter::System,
                token.site,
            );
        }
    }

    fn pragma_dependency(&mut self) {
        let Some(token) = self.lex_header_name(false) else {
            return;
        };
        if token.kind != TokenKind::HeaderName {
            self.report_token(Message::ExpectsFilename, &token, Vec::new());
            return;
        }
        let Some((name, angled)) = Self::include_filename(&token) else {
            return;
        };
        if name.is_empty() {
            return;
        }
        if self.lookup_file(&name, angled, None, false).is_none() {
            self.report_token(Message::FileNotFound, &token, vec![name]);
        }
    }

    fn pragma_push_macro(&mut self, pragma: &Token, push: bool) {
        let malformed = |preprocessor: &mut Preprocessor| {
            preprocessor.report_token(
                Message::PushPopMacroMalformed,
                pragma,
                vec![pragma.text().to_string()],
            );
        };
        let open = self.lex();
        if open.kind != TokenKind::LeftParen {
            malformed(self);
            return;
        }
        let string = self.lex();
        if string.kind != TokenKind::String(LiteralPrefix::Plain) {
            malformed(self);
            return;
        }
        let close = self.lex();
        if close.kind != TokenKind::RightParen {
            malformed(self);
            return;
        }
        if string.spelling.len() <= 2 {
            self.report_token(
                Message::PushPopMacroEmptyString,
                &string,
                vec![if push { "0" } else { "1" }.to_string()],
            );
            return;
        }
        let name: Rc<[u8]> = Rc::from(&string.spelling[1..string.spelling.len() - 1]);
        if push {
            let current = self.macros.get(&name).cloned();
            self.pushed_macros.entry(name).or_default().push(current);
            return;
        }
        let popped = match self.pushed_macros.get_mut(&name) {
            Some(stack) => stack.pop(),
            None => None,
        };
        match popped {
            Some(restored) => {
                if self
                    .pushed_macros
                    .get(&name)
                    .is_some_and(|stack| stack.is_empty())
                {
                    self.pushed_macros.remove(&name);
                }
                if let Some(existing) = self.macros.get(&name).cloned() {
                    if existing.warn_if_unused.get() {
                        let place = existing.defined_at;
                        self.unused_macros.retain(|held| *held != place);
                    }
                    self.macros.remove(&name);
                }
                if let Some(restored) = restored {
                    self.macros.insert(name, restored);
                }
            }
            None => {
                self.report_token(
                    Message::PopMacroWithoutPush,
                    pragma,
                    vec![String::from_utf8_lossy(&name).into_owned()],
                );
            }
        }
    }

    fn pragma_message(&mut self, pragma: &Token, kind: usize, namespace: &str) {
        let mut token = self.lex();
        let mut expect_close = false;
        match token.kind {
            TokenKind::LeftParen => {
                expect_close = true;
                token = self.lex();
            }
            TokenKind::String(LiteralPrefix::Plain) => {}
            _ => {
                self.report_token(
                    Message::PragmaMessageMalformed,
                    pragma,
                    vec![kind.to_string()],
                );
                return;
            }
        }
        let context = match kind {
            0 => "pragma message",
            1 => "pragma warning",
            _ => "pragma error",
        };
        let Some(message) = self.finish_string_literal(&mut token, context, true) else {
            return;
        };
        if expect_close {
            if token.kind != TokenKind::RightParen {
                let current = token.clone();
                self.report_token(
                    Message::PragmaMessageMalformed,
                    &current,
                    vec![kind.to_string()],
                );
                return;
            }
            token = self.lex();
        }
        if token.kind != TokenKind::EndOfDirective {
            let current = token.clone();
            self.report_token(
                Message::PragmaMessageMalformed,
                &current,
                vec![kind.to_string()],
            );
            return;
        }
        let text = String::from_utf8_lossy(&message).into_owned();
        let diagnostic = if kind == 2 {
            Message::PragmaError
        } else {
            Message::PragmaMessage
        };
        self.report_token(diagnostic, pragma, vec![text]);
        let mut line = b"#pragma ".to_vec();
        if !namespace.is_empty() {
            line.extend_from_slice(namespace.as_bytes());
            line.push(b' ');
        }
        match kind {
            0 => line.extend_from_slice(b"message(\""),
            1 => line.extend_from_slice(b"warning \""),
            _ => line.extend_from_slice(b"error \""),
        }
        line.extend_from_slice(&printable_escape(&message));
        line.push(b'"');
        if kind == 0 {
            line.push(b')');
        }
        self.print_directive_line(pragma.site, &line, false);
    }

    fn pragma_debug(&mut self, _pragma: &Token) {
        let command = self.lex_unexpanded();
        if command.kind != TokenKind::Identifier {
            self.report_token(Message::PragmaDebugMissingCommand, &command, Vec::new());
            return;
        }
        let name = command.text().to_string();
        match name.as_str() {
            "dump" | "diag_mapping" | "llvm_fatal_error" | "llvm_unreachable" | "macro"
            | "module_map" | "module_lookup" | "overflow_stack" | "handle_crash"
            | "handle_assert" | "sloc_usage" | "assert" | "crash" | "parser_crash" | "captured" => {
                let mut line = b"#pragma clang __debug ".to_vec();
                line.extend_from_slice(name.as_bytes());
                self.print_directive_line(command.site, &line, false);
            }
            _ => {
                self.report_token(Message::PragmaDebugUnexpectedCommand, &command, vec![name]);
            }
        }
    }

    fn pragma_diagnostic(&mut self, pragma: &Token, namespace: &str) {
        let action = self.lex_unexpanded();
        if action.kind != TokenKind::Identifier {
            self.report_token(Message::PragmaDiagnosticInvalid, &action, Vec::new());
            return;
        }
        let name = action.text().to_string();
        let mut token = self.lex_unexpanded();
        if name == "pop" || name == "push" {
            if name == "pop" {
                if !self.engine.pop() {
                    let current = token.clone();
                    self.report_token(Message::PragmaDiagnosticCannotPop, &current, Vec::new());
                } else {
                    let line = format!("#pragma {namespace} diagnostic pop");
                    self.print_directive_line(pragma.site, line.as_bytes(), false);
                }
            } else {
                self.engine.push();
                let line = format!("#pragma {namespace} diagnostic push");
                self.print_directive_line(pragma.site, line.as_bytes(), false);
            }
            if token.kind != TokenKind::EndOfDirective {
                let current = token.clone();
                self.report_token(Message::PragmaDiagnosticInvalidToken, &current, Vec::new());
            }
            return;
        }
        let severity = match name.as_str() {
            "ignored" => CSeverity::Ignored,
            "warning" => CSeverity::Warning,
            "error" => CSeverity::Error,
            "fatal" => CSeverity::Fatal,
            _ => {
                let current = token.clone();
                self.report_token(Message::PragmaDiagnosticInvalid, &current, Vec::new());
                return;
            }
        };
        let string_token = token.clone();
        let Some(option) = self.finish_string_literal(&mut token, "pragma diagnostic", false)
        else {
            return;
        };
        if token.kind != TokenKind::EndOfDirective {
            let current = token.clone();
            self.report_token(Message::PragmaDiagnosticInvalidToken, &current, Vec::new());
            return;
        }
        let option = String::from_utf8_lossy(&option).into_owned();
        if option.len() < 3 || !(option.starts_with("-W") || option.starts_with("-R")) {
            self.report_token(
                Message::PragmaDiagnosticInvalidOption,
                &string_token,
                Vec::new(),
            );
            return;
        }
        let group = &option[2..];
        let known = if group == "everything" {
            self.engine.state.enable_all = severity != CSeverity::Ignored;
            if severity == CSeverity::Ignored {
                self.engine.state.ignore_all = true;
            }
            true
        } else {
            self.engine.set_group(group, Some(severity), None)
        };
        if !known {
            self.report_token(
                Message::PragmaDiagnosticUnknownWarning,
                &string_token,
                vec![option],
            );
            return;
        }
        let label = match severity {
            CSeverity::Ignored => "ignored",
            CSeverity::Warning => "warning",
            CSeverity::Error => "error",
            _ => "fatal",
        };
        let line = format!("#pragma {namespace} diagnostic {label} \"{option}\"");
        self.print_directive_line(pragma.site, line.as_bytes(), false);
    }

    fn handle_pragma_operator(&mut self, identifier: Token) -> Option<Token> {
        let mut collected: Vec<Token> = vec![identifier.clone()];
        let collect = self.in_argument_preexpansion;
        let mut token = self.lex();
        if token.kind != TokenKind::LeftParen {
            self.report_token(Message::PragmaOperatorMalformed, &identifier, Vec::new());
            return Some(token);
        }
        collected.push(token.clone());
        token = self.lex();
        if !matches!(token.kind, TokenKind::String(_)) {
            self.report_token(Message::PragmaOperatorMalformed, &identifier, Vec::new());
            if !matches!(
                token.kind,
                TokenKind::RightParen | TokenKind::EndOfFile | TokenKind::EndOfDirective
            ) {
                token = self.lex();
            }
            while token.kind != TokenKind::RightParen
                && !token.at_start_of_line()
                && !matches!(token.kind, TokenKind::EndOfFile | TokenKind::EndOfDirective)
            {
                token = self.lex();
            }
            if token.kind == TokenKind::RightParen {
                token = self.lex();
            }
            return Some(token);
        }
        let string = token.clone();
        collected.push(token.clone());
        token = self.lex();
        if token.kind != TokenKind::RightParen {
            self.report_token(Message::PragmaOperatorMalformed, &identifier, Vec::new());
            return Some(token);
        }
        if collect {
            collected.push(token);
            let first = collected.remove(0);
            let mut rest = collected;
            for item in rest.iter_mut() {
                item.flags |= DISABLE_EXPAND;
            }
            self.stack.push(Source::Tokens(TokenSource {
                tokens: Rc::from(rest),
                index: 0,
                macro_definition: None,
                disable_expansion: true,
                at_start_of_line: false,
                leading_space: false,
                next_gets_space: false,
                site: None,
            }));
            let mut first = first;
            first.flags |= DISABLE_EXPAND;
            return Some(first);
        }
        let mut text = string.spelling.to_vec();
        let mut start = 0usize;
        if text.first() == Some(&b'L')
            || text.first() == Some(&b'U')
            || (text.first() == Some(&b'u') && text.get(1) != Some(&b'8'))
        {
            start = 1;
        } else if text.first() == Some(&b'u') {
            start = 2;
        }
        text.drain(..start);
        let mut body = Vec::with_capacity(text.len());
        body.push(b' ');
        let inner = &text[1..text.len().saturating_sub(1).max(1)];
        let mut at = 0usize;
        while at < inner.len() {
            if inner[at] == b'\\'
                && at + 1 < inner.len()
                && (inner[at + 1] == b'\\' || inner[at + 1] == b'"')
            {
                at += 1;
            }
            body.push(inner[at]);
            at += 1;
        }
        body.push(b'\n');
        let bytes: Rc<[u8]> = Rc::from(body);
        let buffer = SourceBuffer::new("<scratch space>", bytes.clone(), BufferKind::Pragma);
        let index = self.sources.add(buffer);
        let mut lexer = Lexer::new(bytes, index, &self.language);
        lexer.in_directive = true;
        lexer.pragma_lexer = true;
        lexer.at_start_of_line = false;
        lexer.at_physical_start_of_line = false;
        let depth = self.stack.len();
        self.stack.push(Source::File(FileSource {
            lexer,
            conditionals: Vec::new(),
            directory: String::new(),
            search_index: None,
            found_by_relative: false,
            path: Rc::from("<scratch space>"),
            buffer: index,
            guard: GuardState::default(),
            guard_key: None,
        }));
        self.pragma_site = Some(identifier.site);
        self.handle_pragma(identifier.site);
        if self.in_directive() && self.stack.len() > depth {
            self.discard_until_end_of_directive();
        }
        self.pragma_site = None;
        while self.stack.len() > depth {
            if let Some(Source::File(file)) = self.stack.last() {
                if file.lexer.pragma_lexer {
                    self.stack.pop();
                    continue;
                }
            }
            break;
        }
        None
    }

    fn handle_microsoft_pragma(&mut self, identifier: Token) -> Option<Token> {
        let mut collected: Vec<Token> = vec![identifier.clone()];
        let mut token = self.lex();
        collected.push(token.clone());
        if token.kind != TokenKind::LeftParen {
            self.report_token(Message::PragmaOperatorMalformed, &identifier, Vec::new());
            return Some(token);
        }
        let mut pragma_tokens: Vec<Token> = Vec::new();
        let mut depth = 0usize;
        token = self.lex();
        while token.kind != TokenKind::EndOfFile {
            collected.push(token.clone());
            pragma_tokens.push(token.clone());
            if token.kind == TokenKind::LeftParen {
                depth += 1;
            } else if token.kind == TokenKind::RightParen {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            token = self.lex();
        }
        if token.kind == TokenKind::EndOfFile {
            self.report_token(
                Message::UnterminatedMicrosoftPragma,
                &identifier,
                Vec::new(),
            );
            return Some(token);
        }
        if self.in_argument_preexpansion {
            let first = collected.remove(0);
            for item in collected.iter_mut() {
                item.flags |= DISABLE_EXPAND;
            }
            self.push_tokens(collected, true);
            let mut first = first;
            first.flags |= DISABLE_EXPAND;
            return Some(first);
        }
        if let Some(first) = pragma_tokens.first_mut() {
            first.flags |= LEADING_SPACE;
        }
        if let Some(last) = pragma_tokens.last_mut() {
            last.kind = TokenKind::EndOfDirective;
        }
        let depth = self.stack.len();
        self.push_tokens(pragma_tokens, true);
        self.handle_pragma(identifier.site);
        while self.stack.len() > depth {
            if let Some(Source::Tokens(source)) = self.stack.last() {
                if source.index >= source.tokens.len() {
                    self.stack.pop();
                    continue;
                }
            }
            let token = self.lex_unexpanded();
            if matches!(token.kind, TokenKind::EndOfDirective | TokenKind::EndOfFile) {
                break;
            }
        }
        if let Some(Source::Tokens(source)) = self.stack.last() {
            if self.stack.len() > depth && source.index >= source.tokens.len() {
                self.stack.pop();
            }
        }
        None
    }

    fn feature_value(&self, name: &str, extension: bool) -> bool {
        let trimmed = if name.len() >= 4 && name.starts_with("__") && name.ends_with("__") {
            &name[2..name.len() - 2]
        } else {
            name
        };
        let c11 = self.language.at_least(CStandard::C11);
        let feature = match trimmed {
            "c_alignas"
            | "c_alignof"
            | "c_atomic"
            | "c_generic_selections"
            | "c_static_assert"
            | "c_thread_local"
            | "objc_c_static_assert" => c11,
            "c_fixed_enum" => self.language.at_least(CStandard::C23),
            "c_countof" => self.language.at_least(CStandard::C2y),
            other => ALWAYS_FEATURES.contains(&other),
        };
        if feature || !extension {
            return feature;
        }
        if self.engine.state.extension_behavior == Some(CSeverity::Error) {
            return false;
        }
        match trimmed {
            "c_countof" => !self.language.at_least(CStandard::C2y),
            other => ALWAYS_EXTENSIONS.contains(&other),
        }
    }

    fn evaluate_feature_like(
        &mut self,
        name: &Token,
        expand_arguments: bool,
        operation: &mut dyn FnMut(&mut Preprocessor<'files>, &mut Token, &mut bool) -> u64,
    ) -> (Option<Vec<u8>>, Token) {
        let mut token = self.lex_unexpanded();
        if token.kind != TokenKind::LeftParen {
            self.report_token(
                Message::MissingAfter,
                &token,
                vec![quoted(&name.spelling), "'('".to_string()],
            );
            if !matches!(token.kind, TokenKind::EndOfFile | TokenKind::EndOfDirective) {
                return (Some(b"0".to_vec()), token);
            }
            return (None, token);
        }
        let open = token.clone();
        let mut depth = 1usize;
        let mut result: Option<u64> = None;
        let mut result_token: Option<Token> = None;
        let mut suppress = false;
        let mut already = false;
        loop {
            if !already {
                token = if expand_arguments {
                    self.lex()
                } else {
                    self.lex_unexpanded()
                };
            }
            already = false;
            match token.kind {
                TokenKind::EndOfFile | TokenKind::EndOfDirective => {
                    self.report_token(Message::UnterminatedMacroInvocation, &token, Vec::new());
                    return (None, token);
                }
                TokenKind::Comma => {
                    if !suppress {
                        self.report_token(Message::TooManyArguments, &token, Vec::new());
                        suppress = true;
                    }
                    continue;
                }
                TokenKind::LeftParen => {
                    depth += 1;
                    if result.is_none() {
                        if !suppress {
                            self.report_token(
                                Message::NestedParentheses,
                                &token,
                                vec![quoted(&name.spelling)],
                            );
                            suppress = true;
                        }
                        continue;
                    }
                }
                TokenKind::RightParen => {
                    depth -= 1;
                    if depth > 0 {
                        continue;
                    }
                    let text = match result {
                        Some(value) => {
                            if value > 1 {
                                format!("{value}L")
                            } else {
                                value.to_string()
                            }
                        }
                        None => {
                            if !suppress {
                                self.report_token(Message::TooFewArguments, &token, Vec::new());
                            }
                            "0".to_string()
                        }
                    };
                    return (Some(text.into_bytes()), token);
                }
                _ => {
                    if result.is_none() {
                        let mut lexed_next = false;
                        let value = operation(self, &mut token, &mut lexed_next);
                        result = Some(value);
                        result_token = Some(token.clone());
                        if lexed_next {
                            already = true;
                        }
                        continue;
                    }
                }
            }
            if !suppress {
                let previous = result_token
                    .as_ref()
                    .map(token_kind_name)
                    .unwrap_or_default();
                self.report_token(
                    Message::MissingAfter,
                    &token,
                    vec![previous, "')'".to_string()],
                );
                self.report_token(Message::ToMatch, &open, vec!["'('".to_string()]);
                suppress = true;
            }
        }
    }

    fn expect_feature_identifier(&mut self, token: &Token) -> Option<String> {
        if token.kind == TokenKind::Identifier {
            return Some(token.text().to_string());
        }
        self.report_token(Message::FeatureCheckMalformed, token, Vec::new());
        None
    }

    fn expand_builtin(&mut self, identifier: Token, builtin: Builtin) -> Option<Token> {
        if builtin == Builtin::Pragma {
            return self.handle_pragma_operator(identifier);
        }
        if builtin == Builtin::MicrosoftPragma {
            return self.handle_microsoft_pragma(identifier);
        }
        let site = identifier.site;
        let mut kind = TokenKind::Number;
        let text: Vec<u8>;
        let mut last = identifier.clone();
        match builtin {
            Builtin::Line => {
                let end = self.site_end_of(&identifier);
                let line = if (end.buffer as usize) < self.sources.buffers.len()
                    && self.sources.buffer(end.buffer).kind != BufferKind::Scratch
                {
                    self.sources.presumed(end).line
                } else {
                    1
                };
                text = line.to_string().into_bytes();
            }
            Builtin::File | Builtin::BaseFile | Builtin::FileName => {
                let mut presumed = self.sources.presumed(site);
                if builtin == Builtin::BaseFile {
                    while let Some(include) = presumed.include {
                        presumed = self.sources.presumed(include);
                    }
                }
                let mut name = presumed.name.to_string();
                if builtin == Builtin::FileName {
                    if let Some(at) = name.rfind('/') {
                        if at + 1 < name.len() {
                            name = name[at + 1..].to_string();
                        }
                    }
                }
                let mut quoted_name = vec![b'"'];
                quoted_name.extend_from_slice(&stringify_text(name.as_bytes(), b'"'));
                quoted_name.push(b'"');
                text = quoted_name;
                kind = TokenKind::String(LiteralPrefix::Plain);
            }
            Builtin::Date | Builtin::Time | Builtin::Timestamp => {
                self.report_token(Message::DateTime, &identifier, Vec::new());
                let seconds = self.options.source_date_epoch.unwrap_or_else(|| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|elapsed| elapsed.as_secs())
                        .unwrap_or(0)
                });
                let (date, time, stamp) = date_parts(seconds);
                text = match builtin {
                    Builtin::Date => date,
                    Builtin::Time => time,
                    _ => stamp,
                }
                .into_bytes();
                kind = TokenKind::String(LiteralPrefix::Plain);
            }
            Builtin::IncludeLevel => {
                let mut depth = 0u32;
                let mut presumed = self.sources.presumed(site);
                while let Some(include) = presumed.include {
                    depth += 1;
                    presumed = self.sources.presumed(include);
                }
                text = depth.to_string().into_bytes();
            }
            Builtin::FloatEvaluationMethod => {
                let method = if self.options.target.architecture.starts_with('i')
                    && self.options.target.architecture.ends_with("86")
                {
                    "2"
                } else {
                    "0"
                };
                text = method.as_bytes().to_vec();
            }
            Builtin::Counter => {
                let message = if self.language.at_least(CStandard::C2y) {
                    Message::CounterCompatibility
                } else {
                    Message::CounterExtension
                };
                self.report_token(message, &identifier, Vec::new());
                text = self.counter.to_string().into_bytes();
                self.counter = self.counter.wrapping_add(1);
            }
            Builtin::HasInclude | Builtin::HasIncludeNext => {
                let (value, end) =
                    self.evaluate_has_include(&identifier, builtin == Builtin::HasIncludeNext);
                let Some(value) = value else {
                    return Some(end);
                };
                last = end;
                text = if value { b"1".to_vec() } else { b"0".to_vec() };
            }
            Builtin::HasEmbed => {
                let (value, end) = self.evaluate_has_embed(&identifier);
                let Some(value) = value else {
                    return Some(end);
                };
                last = end;
                text = value.to_string().into_bytes();
            }
            Builtin::HasWarning => {
                let (value, end) = self.evaluate_feature_like(
                    &identifier,
                    false,
                    &mut |preprocessor, token, lexed| {
                        let start = token.clone();
                        *lexed = matches!(token.kind, TokenKind::String(_));
                        let Some(name) =
                            preprocessor.finish_string_literal(token, "'__has_warning'", false)
                        else {
                            return 0;
                        };
                        let name = String::from_utf8_lossy(&name).into_owned();
                        if name.len() < 3 || !name.starts_with("-W") {
                            preprocessor.report_token(
                                Message::HasWarningInvalidOption,
                                &start,
                                Vec::new(),
                            );
                            return 0;
                        }
                        u64::from(warning_group_exists(&name[2..]))
                    },
                );
                let Some(value) = value else {
                    return Some(end);
                };
                last = end;
                text = value;
            }
            _ => {
                let expand = matches!(
                    builtin,
                    Builtin::HasAttribute | Builtin::HasCAttribute | Builtin::HasDeclspecAttribute
                );
                let (value, end) = self.evaluate_feature_like(
                    &identifier,
                    expand,
                    &mut |preprocessor, token, lexed| {
                        preprocessor.feature_operation(builtin, token, lexed)
                    },
                );
                let Some(value) = value else {
                    return Some(end);
                };
                last = end;
                text = value;
            }
        }
        let _ = last;
        let mut token = self.make_token(kind, text, (site, site));
        token.set(START_OF_LINE, identifier.at_start_of_line());
        token.set(LEADING_SPACE, identifier.has_leading_space());
        Some(token)
    }

    fn feature_operation(&mut self, builtin: Builtin, token: &mut Token, lexed: &mut bool) -> u64 {
        match builtin {
            Builtin::IsIdentifier => u64::from(token.kind == TokenKind::Identifier),
            Builtin::HasFeature | Builtin::HasExtension => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                u64::from(self.feature_value(&name, builtin == Builtin::HasExtension))
            }
            Builtin::HasBuiltinFunction => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                let known = name.starts_with("__builtin_")
                    || name.starts_with("__sync_")
                    || name.starts_with("__atomic_")
                    || name.starts_with("__c11_atomic_")
                    || matches!(
                        name.as_str(),
                        "__is_target_arch"
                            | "__is_target_vendor"
                            | "__is_target_os"
                            | "__is_target_environment"
                            | "__is_target_variant_os"
                            | "__is_target_variant_environment"
                            | "alloca"
                            | "__assume"
                            | "__debugbreak"
                    );
                u64::from(known)
            }
            Builtin::HasConstexprBuiltinFunction => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                u64::from(matches!(
                    name.as_str(),
                    "__builtin_fmax"
                        | "__builtin_fmin"
                        | "__builtin_abs"
                        | "__builtin_popcount"
                        | "__builtin_clz"
                        | "__builtin_ctz"
                        | "__builtin_bswap32"
                        | "__builtin_bswap64"
                        | "__builtin_isnan"
                        | "__builtin_isinf"
                        | "__builtin_inf"
                        | "__builtin_nan"
                        | "__builtin_memcpy"
                        | "__builtin_strlen"
                ))
            }
            Builtin::HasAttribute => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                let trimmed = if name.len() > 4 && name.starts_with("__") && name.ends_with("__") {
                    &name[2..name.len() - 2]
                } else {
                    name.as_str()
                };
                u64::from(GNU_ATTRIBUTES.contains(&trimmed))
            }
            Builtin::HasDeclspecAttribute => {
                let _ = self.expect_feature_identifier(token);
                0
            }
            Builtin::HasCAttribute => {
                let Some(first) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                let next = self.lex_unexpanded();
                let mut scope: Option<String> = None;
                let mut name = first;
                if next.kind != TokenKind::ColonColon {
                    *token = next;
                    *lexed = true;
                } else {
                    scope = Some(name.clone());
                    let after = self.lex();
                    let Some(attribute) = self.expect_feature_identifier(&after) else {
                        return 0;
                    };
                    name = attribute;
                }
                let trimmed = if name.len() > 4 && name.starts_with("__") && name.ends_with("__") {
                    name[2..name.len() - 2].to_string()
                } else {
                    name.clone()
                };
                match scope.as_deref() {
                    None => C23_ATTRIBUTES
                        .iter()
                        .find(|(held, _)| *held == trimmed)
                        .and_then(|(_, value)| value.parse().ok())
                        .unwrap_or(0),
                    Some("gnu") | Some("__gnu__") => {
                        u64::from(GNU_ATTRIBUTES.contains(&trimmed.as_str()))
                    }
                    Some("clang") | Some("_Clang") => u64::from(matches!(
                        trimmed.as_str(),
                        "fallthrough"
                            | "nodiscard"
                            | "unsequenced"
                            | "reproducible"
                            | "overloadable"
                            | "noderef"
                            | "annotate"
                            | "always_inline"
                            | "noinline"
                            | "flag_enum"
                            | "enum_extensibility"
                    )),
                    _ => 0,
                }
            }
            Builtin::IsTargetArchitecture => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                let architecture = self.options.target.architecture.to_ascii_lowercase();
                let wanted = name.to_ascii_lowercase();
                u64::from(
                    architecture == wanted
                        || (wanted == "x86"
                            && architecture.starts_with('i')
                            && architecture.ends_with("86")),
                )
            }
            Builtin::IsTargetVendor => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                u64::from(self.options.target.vendor.eq_ignore_ascii_case(&name))
            }
            Builtin::IsTargetSystem => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                let system = self.options.target.system.to_ascii_lowercase();
                u64::from(system.starts_with(&name.to_ascii_lowercase()))
            }
            Builtin::IsTargetEnvironment => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                u64::from(self.options.target.environment.eq_ignore_ascii_case(&name))
            }
            Builtin::IsTargetVariantSystem | Builtin::IsTargetVariantEnvironment => {
                let _ = self.expect_feature_identifier(token);
                0
            }
            Builtin::BuildingModule => {
                if token.kind != TokenKind::Identifier {
                    self.report_token(Message::FeatureCheckMalformed, token, Vec::new());
                }
                0
            }
            _ => 0,
        }
    }

    fn evaluate_has_include(&mut self, identifier: &Token, next: bool) -> (Option<bool>, Token) {
        if !self.parsing_if_directive {
            self.report_token(
                Message::DirectiveRequired,
                identifier,
                vec![quoted(&identifier.spelling)],
            );
            return (None, identifier.clone());
        }
        let mut start = None;
        let mut include_next = false;
        if next {
            let current = self.real_file_index();
            let primary = self.in_primary_file();
            let found = current.and_then(|index| match &self.stack[index] {
                Source::File(file) => Some((file.found_by_relative, file.search_index)),
                _ => None,
            });
            if primary && self.language.header_file {
            } else if primary {
                self.report_token(Message::IncludeNextInPrimary, identifier, Vec::new());
            } else if let Some((relative, search)) = found {
                if relative || search.is_none() {
                    self.report_token(Message::IncludeNextAbsolutePath, identifier, Vec::new());
                } else {
                    start = search.map(|index| index + 1);
                    include_next = true;
                }
            }
        }
        let mut token = match self.lex_header_name(false) {
            Some(token) => token,
            None => return (None, identifier.clone()),
        };
        while token.kind == TokenKind::Comment {
            token = self.lex_header_name(false).unwrap_or(token);
        }
        let open;
        if token.kind != TokenKind::LeftParen {
            self.report_token(
                Message::MissingAfter,
                identifier,
                vec![quoted(&identifier.spelling), "'('".to_string()],
            );
            if token.kind != TokenKind::HeaderName {
                return (None, token);
            }
            open = identifier.clone();
        } else {
            open = token.clone();
            token = match self.lex_header_name(true) {
                Some(token) => token,
                None => return (None, open),
            };
        }
        if token.kind != TokenKind::HeaderName {
            self.report_token(Message::ExpectsFilename, &token, Vec::new());
            return (None, token);
        }
        let filename = token.clone();
        let close = self.lex_non_comment();
        if close.kind != TokenKind::RightParen {
            self.report_token(
                Message::MissingAfter,
                &filename,
                vec![quoted(&identifier.spelling), "')'".to_string()],
            );
            self.report_token(Message::ToMatch, &open, vec!["'('".to_string()]);
            return (None, close);
        }
        let Some((name, angled)) = Self::include_filename(&filename) else {
            return (Some(false), close);
        };
        if name.is_empty() {
            return (Some(false), close);
        }
        let found = self
            .lookup_file(&name, angled, start, include_next)
            .is_some();
        (Some(found), close)
    }

    fn evaluate_has_embed(&mut self, identifier: &Token) -> (Option<u32>, Token) {
        if !self.parsing_if_directive {
            self.report_token(
                Message::DirectiveRequired,
                identifier,
                vec![quoted(&identifier.spelling)],
            );
            return (None, identifier.clone());
        }
        let open = self.lex_unexpanded();
        if open.kind != TokenKind::LeftParen {
            self.report_token(
                Message::MissingAfter,
                &open,
                vec![quoted(&identifier.spelling), "'('".to_string()],
            );
            return (None, open);
        }
        let Some(filename) = self.lex_header_name(true) else {
            return (None, open);
        };
        if filename.kind != TokenKind::HeaderName {
            self.report_token(Message::ExpectsFilename, &filename, Vec::new());
            return (None, filename);
        }
        let mut token = self.lex();
        let mut depth = 0usize;
        let mut unrecognized = false;
        let mut limit: Option<u64> = None;
        while !(token.kind == TokenKind::RightParen && depth == 0) {
            match token.kind {
                TokenKind::EndOfDirective | TokenKind::EndOfFile => {
                    self.report_token(
                        Message::MissingAfter,
                        &filename,
                        vec![quoted(&identifier.spelling), "')'".to_string()],
                    );
                    self.report_token(Message::ToMatch, &open, vec!["'('".to_string()]);
                    return (None, token);
                }
                TokenKind::LeftParen => depth += 1,
                TokenKind::RightParen => depth -= 1,
                TokenKind::Identifier if depth == 0 => {
                    let name = token.text().to_string();
                    let plain = name.trim_matches('_');
                    if plain == "limit" {
                        let next = self.lex();
                        if next.kind == TokenKind::LeftParen {
                            let value = self.lex();
                            if value.kind == TokenKind::Number {
                                limit = value.text().parse().ok();
                            }
                            token = self.lex();
                            continue;
                        }
                        token = next;
                        continue;
                    }
                    if !matches!(plain, "prefix" | "suffix" | "if_empty") && !name.contains("::") {
                        unrecognized = true;
                    }
                }
                _ => {}
            }
            token = self.lex();
        }
        if unrecognized {
            return (Some(0), token);
        }
        let Some((name, angled)) = Self::include_filename(&filename) else {
            return (Some(0), token);
        };
        if name.is_empty() {
            return (Some(2), token);
        }
        match self.lookup_file(&name, angled, None, false) {
            None => (Some(0), token),
            Some((_, bytes, _, _, _)) => {
                let mut size = bytes.len() as u64;
                if let Some(limit) = limit {
                    size = size.min(limit);
                }
                if size == 0 {
                    (Some(2), token)
                } else {
                    (Some(1), token)
                }
            }
        }
    }
}

impl Printer {
    fn start_new_line_if_needed(&mut self) {
        if self.tokens_on_line || self.directive_on_line {
            self.out.push(b'\n');
            self.tokens_on_line = false;
            self.directive_on_line = false;
        }
    }

    fn write_line_info(&mut self, line: u32, extra: &str) {
        self.start_new_line_if_needed();
        if self.line_directives {
            self.out
                .extend_from_slice(format!("#line {line} \"{}\"", self.name).as_bytes());
        } else {
            self.out
                .extend_from_slice(format!("# {line} \"{}\"", self.name).as_bytes());
            self.out.extend_from_slice(extra.as_bytes());
            match self.character {
                FileCharacter::System => self.out.extend_from_slice(b" 3"),
                FileCharacter::ExternCSystem => self.out.extend_from_slice(b" 3 4"),
                FileCharacter::User => {}
            }
        }
        self.out.push(b'\n');
    }

    fn move_to_line(&mut self, line: u32, require_start: bool) -> bool {
        let mut started = false;
        if (require_start && self.tokens_on_line) || self.directive_on_line {
            self.out.push(b'\n');
            started = true;
            self.line = self.line.wrapping_add(1);
            self.tokens_on_line = false;
            self.directive_on_line = false;
        }
        let unchanged = self.line == line || (self.minimize && !self.line_markers);
        if unchanged {
        } else if !started && line.wrapping_sub(self.line) == 1 {
            self.out.push(b'\n');
            started = true;
        } else if self.line_markers {
            let difference = line.wrapping_sub(self.line);
            if difference <= 8 {
                for _ in 0..difference {
                    self.out.push(b'\n');
                }
            } else {
                self.write_line_info(line, "");
            }
            started = true;
        } else if self.tokens_on_line {
            self.out.push(b'\n');
            started = true;
        }
        if started {
            self.tokens_on_line = false;
            self.directive_on_line = false;
        }
        self.line = line;
        started
    }
}

fn spelling_adjacent(previous: &Token, token: &Token) -> bool {
    previous.place.buffer == token.place.buffer
        && previous.place.offset.wrapping_add(previous.length) == token.place.offset
}

fn avoid_concatenation(
    language: &LanguageOptions,
    before_previous: Option<&Token>,
    previous: &Token,
    token: &Token,
) -> bool {
    if previous.kind == TokenKind::Embed {
        return true;
    }
    if spelling_adjacent(previous, token) {
        return false;
    }
    let custom = matches!(previous.kind, TokenKind::Identifier);
    let first_character = matches!(
        previous.kind,
        TokenKind::Number
            | TokenKind::Period
            | TokenKind::Amp
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Slash
            | TokenKind::Less
            | TokenKind::Greater
            | TokenKind::Pipe
            | TokenKind::Percent
            | TokenKind::Colon
            | TokenKind::Hash
            | TokenKind::Arrow
    );
    let avoid_equal = matches!(
        previous.kind,
        TokenKind::Amp
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Slash
            | TokenKind::Less
            | TokenKind::Greater
            | TokenKind::Pipe
            | TokenKind::Percent
            | TokenKind::Star
            | TokenKind::Exclaim
            | TokenKind::LessLess
            | TokenKind::GreaterGreater
            | TokenKind::Caret
            | TokenKind::Equal
    );
    if !custom && !first_character && !avoid_equal {
        return false;
    }
    if avoid_equal && matches!(token.kind, TokenKind::Equal | TokenKind::EqualEqual) {
        return true;
    }
    if token.kind == TokenKind::Embed {
        return true;
    }
    if !custom && !first_character {
        return false;
    }
    let first = token.spelling.first().copied().unwrap_or(0);
    match previous.kind {
        TokenKind::Identifier => {
            if token.kind == TokenKind::Number {
                return first != b'.';
            }
            if token.kind == TokenKind::Identifier
                || matches!(
                    token.kind,
                    TokenKind::String(LiteralPrefix::Wide)
                        | TokenKind::String(LiteralPrefix::Utf8)
                        | TokenKind::String(LiteralPrefix::Utf16)
                        | TokenKind::String(LiteralPrefix::Utf32)
                        | TokenKind::Character(LiteralPrefix::Wide)
                        | TokenKind::Character(LiteralPrefix::Utf8)
                        | TokenKind::Character(LiteralPrefix::Utf16)
                        | TokenKind::Character(LiteralPrefix::Utf32)
                )
            {
                return true;
            }
            if !matches!(
                token.kind,
                TokenKind::Character(LiteralPrefix::Plain)
                    | TokenKind::String(LiteralPrefix::Plain)
            ) {
                return false;
            }
            &*previous.spelling == b"L"
        }
        TokenKind::Number => {
            is_number_body(first)
                || first == b'+'
                || first == b'-'
                || (language.dollar_identifiers && first == b'$')
        }
        TokenKind::Period => {
            (first == b'.' && before_previous.is_some_and(|held| held.kind == TokenKind::Period))
                || first.is_ascii_digit()
        }
        TokenKind::Amp => first == b'&',
        TokenKind::Plus => first == b'+',
        TokenKind::Minus => first == b'-' || first == b'>',
        TokenKind::Slash => first == b'*' || first == b'/',
        TokenKind::Less => first == b'<' || first == b':' || first == b'%',
        TokenKind::Greater => first == b'>',
        TokenKind::Pipe => first == b'|',
        TokenKind::Percent => first == b'>' || first == b':',
        TokenKind::Colon => first == b'>',
        TokenKind::Hash => first == b'#' || first == b'@' || first == b'%',
        TokenKind::Arrow => false,
        _ => false,
    }
}

impl<'files> Preprocessor<'files> {
    fn file_changed(&mut self, place: Place, reason: ChangeReason) {
        if self.printer.is_none() {
            return;
        }
        let presumed = self.sources.presumed(place);
        let include_line = if reason == ChangeReason::Enter {
            presumed
                .include
                .map(|include| self.sources.presumed(include).line)
        } else {
            None
        };
        let Some(printer) = &mut self.printer else {
            return;
        };
        let mut line = presumed.line;
        if let Some(include_line) = include_line {
            printer.move_to_line(include_line, false);
        } else if reason == ChangeReason::SystemHeader {
            line += 1;
        }
        printer.line = line;
        printer.name = presumed.name.clone();
        printer.character = if reason == ChangeReason::SystemHeader {
            FileCharacter::System
        } else {
            presumed.character
        };
        if !printer.line_markers {
            if !printer.minimize {
                printer.start_new_line_if_needed();
            }
            return;
        }
        if !printer.initialized {
            printer.write_line_info(line, "");
            printer.initialized = true;
        }
        if reason == ChangeReason::Enter && !printer.first_file_entered {
            printer.first_file_entered = true;
            return;
        }
        match reason {
            ChangeReason::Enter => printer.write_line_info(line, " 1"),
            ChangeReason::Exit => printer.write_line_info(line, " 2"),
            ChangeReason::Rename | ChangeReason::SystemHeader => printer.write_line_info(line, ""),
        }
    }

    fn presumed_line_of_site(&self, site: Place) -> Option<u32> {
        if site.buffer as usize >= self.sources.buffers.len() {
            return None;
        }
        if self.sources.buffer(site.buffer).kind == BufferKind::Scratch {
            return None;
        }
        Some(self.sources.presumed(site).line)
    }

    fn printer_move_to_line(&mut self, site: Place, require_start: bool) -> bool {
        let line = self.presumed_line_of_site(site);
        let Some(printer) = &mut self.printer else {
            return false;
        };
        let line = line.unwrap_or(printer.line);
        printer.move_to_line(line, require_start)
    }

    fn print_whitespace_before(
        &mut self,
        token: &Token,
        require_space: bool,
        require_same_line: bool,
    ) {
        if token.kind == TokenKind::EndOfFile {
            return;
        }
        let line = self.presumed_line_of_site(token.site);
        let column = if token.site.buffer as usize >= self.sources.buffers.len()
            || self.sources.buffer(token.site.buffer).kind == BufferKind::Scratch
        {
            1
        } else {
            self.sources
                .buffer(token.site.buffer)
                .physical_column(token.site.offset)
        };
        let language = self.language.clone();
        let Some(printer) = &mut self.printer else {
            return;
        };
        let mut moved = false;
        if !require_same_line || printer.directive_on_line {
            let target = line.unwrap_or(printer.line);
            let first_in_file = token.at_start_of_line() && line == Some(1);
            let require_start = printer.directive_on_line;
            moved = printer.move_to_line(target, require_start) || first_in_file;
        }
        if moved {
            if printer.minimize {
                if token.kind == TokenKind::Hash {
                    printer.out.push(b' ');
                }
            } else {
                let mut column = column;
                if column == 1 && token.has_leading_space() {
                    column = 2;
                }
                if column <= 1 && token.kind == TokenKind::Hash {
                    printer.out.push(b' ');
                }
                for _ in 1..column {
                    printer.out.push(b' ');
                }
            }
        } else {
            let concatenation = (printer.tokens_on_line || printer.directive_on_line)
                && printer.previous.as_ref().is_some_and(|previous| {
                    avoid_concatenation(
                        &language,
                        printer.before_previous.as_ref(),
                        previous,
                        token,
                    )
                });
            if require_space || (!printer.minimize && token.has_leading_space()) || concatenation {
                printer.out.push(b' ');
            }
        }
        printer.before_previous = printer.previous.take();
        printer.previous = Some(token.clone());
    }

    fn print_directive_line(&mut self, site: Place, line: &[u8], as_tokens: bool) {
        if self.printer.is_none() {
            return;
        }
        self.printer_move_to_line(site, true);
        if let Some(printer) = &mut self.printer {
            printer.out.extend_from_slice(line);
            if as_tokens {
                printer.tokens_on_line = true;
            } else {
                printer.directive_on_line = true;
            }
        }
    }

    fn macro_definition_text(definition: &MacroDefinition) -> Vec<u8> {
        let mut out = b"#define ".to_vec();
        out.extend_from_slice(&definition.name);
        if definition.function_like {
            out.push(b'(');
            let count = definition.parameters.len();
            for (index, parameter) in definition.parameters.iter().enumerate() {
                if index + 1 == count && &**parameter == b"__VA_ARGS__" {
                    out.extend_from_slice(b"...");
                } else {
                    out.extend_from_slice(parameter);
                }
                if index + 1 != count {
                    out.push(b',');
                }
            }
            if definition.gnu_variadic {
                out.extend_from_slice(b"...");
            }
            out.push(b')');
        }
        if definition
            .body
            .first()
            .is_none_or(|first| !first.has_leading_space())
        {
            out.push(b' ');
        }
        for token in definition.body.iter() {
            if token.has_leading_space() {
                out.push(b' ');
            }
            out.extend_from_slice(&token.spelling);
        }
        out
    }

    fn print_macro_defined(&mut self, definition: &MacroDefinition) {
        if !self
            .printer
            .as_ref()
            .is_some_and(|printer| printer.dump_defines)
        {
            return;
        }
        let text = Self::macro_definition_text(definition);
        self.print_directive_line(definition.defined_at, &text, false);
    }

    fn print_macro_undefined(&mut self, name: &Token) {
        if !self
            .printer
            .as_ref()
            .is_some_and(|printer| printer.dump_defines)
        {
            return;
        }
        let mut text = b"#undef ".to_vec();
        text.extend_from_slice(&name.spelling);
        self.print_directive_line(name.site, &text, false);
    }

    fn print_token(&mut self, token: &Token) {
        let Some(printer) = &mut self.printer else {
            return;
        };
        printer.out.extend_from_slice(&token.spelling);
        if token.kind == TokenKind::Comment || token.kind == TokenKind::Unknown {
            let mut newlines = 0u32;
            let bytes = &token.spelling;
            let mut at = 0usize;
            while at < bytes.len() {
                if bytes[at] == b'\n' || bytes[at] == b'\r' {
                    newlines += 1;
                    if at + 1 < bytes.len()
                        && (bytes[at + 1] == b'\n' || bytes[at + 1] == b'\r')
                        && bytes[at] != bytes[at + 1]
                    {
                        at += 1;
                    }
                }
                at += 1;
            }
            printer.line = printer.line.wrapping_add(newlines);
        }
        if token.kind == TokenKind::Comment && token.spelling.starts_with(b"//") {
            printer.directive_on_line = true;
        }
        printer.tokens_on_line = true;
    }

    pub fn macro_dump(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for definition in self.macros.values() {
            if definition.builtin.is_some() {
                continue;
            }
            out.extend_from_slice(&Self::macro_definition_text(definition));
            out.push(b'\n');
        }
        out
    }

    pub fn run(mut self, main_name: &str, main_bytes: Vec<u8>) -> PreprocessOutcome {
        for flag in self.engine.unknown_flags.clone() {
            let nowhere = Place {
                buffer: u32::MAX,
                offset: 0,
            };
            self.report(
                Message::UnknownWarningOption,
                nowhere,
                vec![format!("-W{flag}")],
            );
        }
        let bytes: Rc<[u8]> = Rc::from(main_bytes);
        let directory = Self::directory_of(main_name);
        let main = self.enter_buffer(
            main_name,
            bytes,
            BufferKind::File,
            None,
            FileCharacter::User,
            SearchOrigin {
                directory,
                search_index: None,
                found_by_relative: true,
            },
        );
        self.main_buffer = Some(main);
        let predefines = self.predefines_text();
        let builtin = self.enter_buffer(
            "<built-in>",
            Rc::from(predefines),
            BufferKind::BuiltIn,
            None,
            FileCharacter::User,
            SearchOrigin {
                directory: ".".to_string(),
                search_index: None,
                found_by_relative: false,
            },
        );
        self.builtin_buffer = Some(builtin);
        let mut tokens = Vec::new();
        match self.options.output {
            OutputMode::Preprocessed => {
                let mut token = self.lex();
                let mut start_of_line = false;
                loop {
                    start_of_line = start_of_line || token.at_start_of_line();
                    self.print_whitespace_before(&token, false, !start_of_line);
                    if token.kind == TokenKind::Comment
                        && self.language.traditional
                        && !self.options.keep_comments
                    {
                        token = self.lex();
                        continue;
                    }
                    if token.kind == TokenKind::EndOfDirective {
                        token = self.lex();
                        start_of_line = true;
                        continue;
                    }
                    self.print_token(&token);
                    start_of_line = false;
                    if token.kind == TokenKind::EndOfFile {
                        break;
                    }
                    token = self.lex();
                }
                if let Some(printer) = &mut self.printer {
                    printer.out.push(b'\n');
                }
            }
            OutputMode::Nothing | OutputMode::MacrosOnly => loop {
                let token = self.lex();
                if token.kind == TokenKind::EndOfFile {
                    break;
                }
                if self.options.syntax_only && token.kind != TokenKind::Comment {
                    tokens.push(token);
                }
            },
        }
        let output = match self.options.output {
            OutputMode::MacrosOnly => self.macro_dump(),
            _ => self
                .printer
                .take()
                .map(|printer| printer.out)
                .unwrap_or_default(),
        };
        PreprocessOutcome {
            output,
            tokens,
            diagnostics: std::mem::take(&mut self.engine.diagnostics),
            comments: std::mem::take(&mut self.comments),
            errors: self.engine.errors,
            warnings: self.engine.warnings,
            main_buffer: self.main_buffer,
            fatal: self.engine.fatal_occurred,
            unknown_warning_flags: std::mem::take(&mut self.engine.unknown_flags),
            sources: std::mem::take(&mut self.sources),
        }
    }

    fn predefines_text(&self) -> Vec<u8> {
        let mut text = String::from("# 1 \"<built-in>\" 3\n");
        if !self.options.undefine_predefined {
            for (name, value) in predefined_macros(&self.options) {
                text.push_str("#define ");
                text.push_str(&name);
                text.push(' ');
                text.push_str(&value);
                text.push('\n');
            }
        }
        for (name, value) in standard_macros(&self.options) {
            text.push_str("#define ");
            text.push_str(&name);
            text.push(' ');
            text.push_str(&value);
            text.push('\n');
        }
        text.push_str("# 1 \"<command line>\" 1\n");
        for option in &self.options.macros {
            match option {
                MacroOption::Define(definition) => {
                    match definition.split_once('=') {
                        Some((name, body)) => {
                            let body = body.split(['\n', '\r']).next().unwrap_or("");
                            text.push_str("#define ");
                            text.push_str(name);
                            text.push(' ');
                            text.push_str(body);
                            let trailing =
                                body.bytes().rev().take_while(|byte| *byte == b'\\').count();
                            if trailing % 2 == 1 {
                                text.push_str("\\\n");
                            }
                        }
                        None => {
                            text.push_str("#define ");
                            text.push_str(definition);
                            text.push_str(" 1");
                        }
                    }
                    text.push('\n');
                }
                MacroOption::Undefine(name) => {
                    text.push_str("#undef ");
                    text.push_str(name);
                    text.push('\n');
                }
            }
        }
        text.push_str("# 1 \"<built-in>\" 2\n");
        for include in &self.options.forced_includes {
            text.push_str("#include \"");
            text.push_str(&include.replace('\\', "\\\\").replace('"', "\\\""));
            text.push_str("\"\n");
        }
        text.into_bytes()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IntegerType {
    SignedChar,
    UnsignedChar,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    Long,
    UnsignedLong,
    LongLong,
    UnsignedLongLong,
}

impl IntegerType {
    fn name(self) -> &'static str {
        match self {
            IntegerType::SignedChar => "signed char",
            IntegerType::UnsignedChar => "unsigned char",
            IntegerType::Short => "short",
            IntegerType::UnsignedShort => "unsigned short",
            IntegerType::Int => "int",
            IntegerType::UnsignedInt => "unsigned int",
            IntegerType::Long => "long int",
            IntegerType::UnsignedLong => "long unsigned int",
            IntegerType::LongLong => "long long int",
            IntegerType::UnsignedLongLong => "long long unsigned int",
        }
    }

    fn signed(self) -> bool {
        matches!(
            self,
            IntegerType::SignedChar
                | IntegerType::Short
                | IntegerType::Int
                | IntegerType::Long
                | IntegerType::LongLong
        )
    }

    fn suffix(self) -> &'static str {
        match self {
            IntegerType::UnsignedInt => "U",
            IntegerType::Long => "L",
            IntegerType::UnsignedLong => "UL",
            IntegerType::LongLong => "LL",
            IntegerType::UnsignedLongLong => "ULL",
            _ => "",
        }
    }

    fn format_modifier(self) -> &'static str {
        match self {
            IntegerType::SignedChar | IntegerType::UnsignedChar => "hh",
            IntegerType::Short | IntegerType::UnsignedShort => "h",
            IntegerType::Long | IntegerType::UnsignedLong => "l",
            IntegerType::LongLong | IntegerType::UnsignedLongLong => "ll",
            _ => "",
        }
    }

    fn unsigned_of(self) -> IntegerType {
        match self {
            IntegerType::SignedChar => IntegerType::UnsignedChar,
            IntegerType::Short => IntegerType::UnsignedShort,
            IntegerType::Int => IntegerType::UnsignedInt,
            IntegerType::Long => IntegerType::UnsignedLong,
            IntegerType::LongLong => IntegerType::UnsignedLongLong,
            other => other,
        }
    }
}

struct TypeModel {
    long_width: u32,
    pointer_width: u32,
    wchar: IntegerType,
    wint: IntegerType,
    size: IntegerType,
    ptrdiff: IntegerType,
    intptr: IntegerType,
    intmax: IntegerType,
    int64: IntegerType,
    long_double: u32,
    biggest_alignment: u32,
}

impl TypeModel {
    fn for_target(target: &TargetInfo) -> TypeModel {
        let pointer_width = target.pointer_width();
        let long_width = target.long_width();
        let lp64 = long_width == 64;
        let wchar = if target.is_windows() {
            IntegerType::UnsignedShort
        } else if target.wchar_signed() {
            IntegerType::Int
        } else {
            IntegerType::UnsignedInt
        };
        let size = if pointer_width == 64 {
            if lp64 {
                IntegerType::UnsignedLong
            } else {
                IntegerType::UnsignedLongLong
            }
        } else {
            IntegerType::UnsignedInt
        };
        let ptrdiff = match size {
            IntegerType::UnsignedLong => IntegerType::Long,
            IntegerType::UnsignedLongLong => IntegerType::LongLong,
            _ => IntegerType::Int,
        };
        let long_double = match target.architecture.as_str() {
            "x86_64" => 128,
            "i386" | "i486" | "i586" | "i686" => 96,
            "aarch64" | "arm64" if !target.system.starts_with("darwin") && !target.is_windows() => {
                128
            }
            _ => 64,
        };
        TypeModel {
            long_width,
            pointer_width,
            wchar,
            wint: IntegerType::UnsignedInt,
            size,
            ptrdiff,
            intptr: ptrdiff,
            intmax: if lp64 {
                IntegerType::Long
            } else {
                IntegerType::LongLong
            },
            int64: if lp64 {
                IntegerType::Long
            } else {
                IntegerType::LongLong
            },
            long_double,
            biggest_alignment: if target.architecture == "x86_64"
                || target.architecture.starts_with("aarch64")
                || target.architecture.starts_with('i')
            {
                16
            } else {
                8
            },
        }
    }

    fn width(&self, ty: IntegerType) -> u32 {
        match ty {
            IntegerType::SignedChar | IntegerType::UnsignedChar => 8,
            IntegerType::Short | IntegerType::UnsignedShort => 16,
            IntegerType::Int | IntegerType::UnsignedInt => 32,
            IntegerType::Long | IntegerType::UnsignedLong => self.long_width,
            IntegerType::LongLong | IntegerType::UnsignedLongLong => 64,
        }
    }

    fn maximum(&self, ty: IntegerType) -> String {
        let width = self.width(ty);
        let value: u64 = if ty.signed() {
            (1u64 << (width - 1)) - 1
        } else if width == 64 {
            u64::MAX
        } else {
            (1u64 << width) - 1
        };
        format!("{value}{}", ty.suffix())
    }

    fn least(&self, width: u32) -> IntegerType {
        match width {
            8 => IntegerType::SignedChar,
            16 => IntegerType::Short,
            32 => IntegerType::Int,
            _ => self.int64,
        }
    }
}

fn define_formats(out: &mut Vec<(String, String)>, prefix: &str, ty: IntegerType, c23: bool) {
    let letters: &[char] = if ty.signed() {
        &['d', 'i']
    } else {
        &['o', 'u', 'x', 'X']
    };
    for letter in letters {
        out.push((
            format!("{prefix}_FMT{letter}__"),
            format!("\"{}{letter}\"", ty.format_modifier()),
        ));
    }
    if c23 && !ty.signed() {
        for letter in ['b', 'B'] {
            out.push((
                format!("{prefix}_FMT{letter}__"),
                format!("\"{}{letter}\"", ty.format_modifier()),
            ));
        }
    }
}

fn define_float(out: &mut Vec<(String, String)>, prefix: &str, width: u32, suffix: &str) {
    let (
        denorm,
        norm,
        digits,
        decimal,
        epsilon,
        mantissa,
        min10,
        max10,
        min_exp,
        max_exp,
        minimum,
        maximum,
    ) = match width {
        32 => (
            "1.40129846e-45",
            "3.40282347e+38",
            6,
            9,
            "1.19209290e-7",
            24,
            -37,
            38,
            -125,
            128,
            "1.17549435e-38",
            "3.40282347e+38",
        ),
        64 => (
            "4.9406564584124654e-324",
            "1.7976931348623157e+308",
            15,
            17,
            "2.2204460492503131e-16",
            53,
            -307,
            308,
            -1021,
            1024,
            "2.2250738585072014e-308",
            "1.7976931348623157e+308",
        ),
        80 | 96 => (
            "3.64519953188247460253e-4951",
            "1.18973149535723176502e+4932",
            18,
            21,
            "1.08420217248550443401e-19",
            64,
            -4931,
            4932,
            -16381,
            16384,
            "3.36210314311209350626e-4932",
            "1.18973149535723176502e+4932",
        ),
        _ => (
            "6.47517511943802511092443895822764655e-4966",
            "1.18973149535723176508575932662800702e+4932",
            33,
            36,
            "1.92592994438723585305597794258492732e-34",
            113,
            -4931,
            4932,
            -16381,
            16384,
            "3.36210314311209350626267781732175260e-4932",
            "1.18973149535723176508575932662800702e+4932",
        ),
    };
    let head = format!("__{prefix}_");
    out.push((format!("{head}DENORM_MIN__"), format!("{denorm}{suffix}")));
    out.push((format!("{head}NORM_MAX__"), format!("{norm}{suffix}")));
    out.push((format!("{head}HAS_DENORM__"), "1".to_string()));
    out.push((format!("{head}DIG__"), digits.to_string()));
    out.push((format!("{head}DECIMAL_DIG__"), decimal.to_string()));
    out.push((format!("{head}EPSILON__"), format!("{epsilon}{suffix}")));
    out.push((format!("{head}HAS_INFINITY__"), "1".to_string()));
    out.push((format!("{head}HAS_QUIET_NAN__"), "1".to_string()));
    out.push((format!("{head}MANT_DIG__"), mantissa.to_string()));
    out.push((format!("{head}MAX_10_EXP__"), max10.to_string()));
    out.push((format!("{head}MAX_EXP__"), max_exp.to_string()));
    out.push((format!("{head}MAX__"), format!("{maximum}{suffix}")));
    out.push((format!("{head}MIN_10_EXP__"), format!("({min10})")));
    out.push((format!("{head}MIN_EXP__"), format!("({min_exp})")));
    out.push((format!("{head}MIN__"), format!("{minimum}{suffix}")));
}

pub fn predefined_macros(options: &PreprocessorOptions) -> Vec<(String, String)> {
    let target = &options.target;
    let language = &options.language;
    let model = TypeModel::for_target(target);
    let c23 = language.at_least(CStandard::C23);
    let mut out: Vec<(String, String)> = Vec::new();
    let mut define = |name: &str, value: &str| out.push((name.to_string(), value.to_string()));
    define("__llvm__", "1");
    define("__clang__", "1");
    define("__clang_major__", "24");
    define("__clang_minor__", "0");
    define("__clang_patchlevel__", "0");
    define("__clang_version__", "\"24.0.0 \"");
    if let Some((major, minor, patch)) = options.gnuc_version {
        out.push(("__GNUC__".to_string(), major.to_string()));
        out.push(("__GNUC_MINOR__".to_string(), minor.to_string()));
        out.push(("__GNUC_PATCHLEVEL__".to_string(), patch.to_string()));
        out.push(("__GXX_ABI_VERSION".to_string(), "1002".to_string()));
    }
    let mut define = |name: &str, value: &str| out.push((name.to_string(), value.to_string()));
    define("__ATOMIC_RELAXED", "0");
    define("__ATOMIC_CONSUME", "1");
    define("__ATOMIC_ACQUIRE", "2");
    define("__ATOMIC_RELEASE", "3");
    define("__ATOMIC_ACQ_REL", "4");
    define("__ATOMIC_SEQ_CST", "5");
    define("__MEMORY_SCOPE_SYSTEM", "0");
    define("__MEMORY_SCOPE_DEVICE", "1");
    define("__MEMORY_SCOPE_WRKGRP", "2");
    define("__MEMORY_SCOPE_WVFRNT", "3");
    define("__MEMORY_SCOPE_SINGLE", "4");
    define("__MEMORY_SCOPE_CLUSTR", "5");
    define("__OPENCL_MEMORY_SCOPE_WORK_ITEM", "0");
    define("__OPENCL_MEMORY_SCOPE_WORK_GROUP", "1");
    define("__OPENCL_MEMORY_SCOPE_DEVICE", "2");
    define("__OPENCL_MEMORY_SCOPE_ALL_SVM_DEVICES", "3");
    define("__OPENCL_MEMORY_SCOPE_SUB_GROUP", "4");
    define("__FPCLASS_SNAN", "0x0001");
    define("__FPCLASS_QNAN", "0x0002");
    define("__FPCLASS_NEGINF", "0x0004");
    define("__FPCLASS_NEGNORMAL", "0x0008");
    define("__FPCLASS_NEGSUBNORMAL", "0x0010");
    define("__FPCLASS_NEGZERO", "0x0020");
    define("__FPCLASS_POSZERO", "0x0040");
    define("__FPCLASS_POSSUBNORMAL", "0x0080");
    define("__FPCLASS_POSNORMAL", "0x0100");
    define("__FPCLASS_POSINF", "0x0200");
    define("__PRAGMA_REDEFINE_EXTNAME", "1");
    define("__VERSION__", "\"Clang 24.0.0\"");
    if !language.gnu {
        define("__STRICT_ANSI__", "1");
    }
    define("__OBJC_BOOL_IS_BOOL", "0");
    define("__CONSTANT_CFSTRINGS__", "1");
    define("__clang_literal_encoding__", "\"UTF-8\"");
    define(
        "__clang_wide_literal_encoding__",
        if model.width(model.wchar) >= 32 {
            "\"UTF-32\""
        } else {
            "\"UTF-16\""
        },
    );
    if options.optimize > 0 {
        define("__OPTIMIZE__", "1");
    }
    if options.optimize_size {
        define("__OPTIMIZE_SIZE__", "1");
    }
    define("__ORDER_LITTLE_ENDIAN__", "1234");
    define("__ORDER_BIG_ENDIAN__", "4321");
    define("__ORDER_PDP_ENDIAN__", "3412");
    define("__BYTE_ORDER__", "__ORDER_LITTLE_ENDIAN__");
    define("__LITTLE_ENDIAN__", "1");
    if model.pointer_width == 64 && model.long_width == 64 {
        define("_LP64", "1");
        define("__LP64__", "1");
    }
    if model.pointer_width == 32 && model.long_width == 32 {
        define("_ILP32", "1");
        define("__ILP32__", "1");
    }
    define("__CHAR_BIT__", "8");
    define("__BOOL_WIDTH__", "1");
    define("__SHRT_WIDTH__", "16");
    define("__INT_WIDTH__", "32");
    let long_width = model.long_width.to_string();
    define("__LONG_WIDTH__", &long_width);
    define("__LLONG_WIDTH__", "64");
    define(
        "__BITINT_MAXWIDTH__",
        if model.pointer_width == 64 {
            "8388608"
        } else {
            "128"
        },
    );
    define("__SCHAR_MAX__", "127");
    define("__SHRT_MAX__", "32767");
    define("__INT_MAX__", "2147483647");
    let long_max = model.maximum(IntegerType::Long);
    define("__LONG_MAX__", &long_max);
    define("__LONG_LONG_MAX__", "9223372036854775807LL");
    let sized = |prefix: &str, ty: IntegerType, minimum: bool, out: &mut Vec<(String, String)>| {
        out.push((format!("{prefix}_MAX__"), model.maximum(ty)));
        out.push((format!("{prefix}_WIDTH__"), model.width(ty).to_string()));
        if minimum {
            let value = if ty.signed() {
                format!("(-{prefix}_MAX__ - 1)")
            } else {
                format!("0{}", ty.suffix())
            };
            out.push((format!("{prefix}_MIN__"), value));
        }
    };
    sized("__WCHAR", model.wchar, true, &mut out);
    sized("__WINT", model.wint, true, &mut out);
    sized("__INTMAX", model.intmax, false, &mut out);
    sized("__SIZE", model.size, false, &mut out);
    sized("__UINTMAX", model.intmax.unsigned_of(), false, &mut out);
    sized("__PTRDIFF", model.ptrdiff, false, &mut out);
    sized("__INTPTR", model.intptr, false, &mut out);
    sized("__UINTPTR", model.intptr.unsigned_of(), false, &mut out);
    let mut define = |name: &str, value: String| out.push((name.to_string(), value));
    define("__SIZEOF_DOUBLE__", "8".to_string());
    define("__SIZEOF_FLOAT__", "4".to_string());
    define("__SIZEOF_INT__", "4".to_string());
    define("__SIZEOF_LONG__", (model.long_width / 8).to_string());
    define(
        "__SIZEOF_LONG_DOUBLE__",
        (model.long_double / 8).to_string(),
    );
    define("__SIZEOF_LONG_LONG__", "8".to_string());
    define("__SIZEOF_POINTER__", (model.pointer_width / 8).to_string());
    define("__SIZEOF_SHORT__", "2".to_string());
    define(
        "__SIZEOF_PTRDIFF_T__",
        (model.width(model.ptrdiff) / 8).to_string(),
    );
    define(
        "__SIZEOF_SIZE_T__",
        (model.width(model.size) / 8).to_string(),
    );
    define(
        "__SIZEOF_WCHAR_T__",
        (model.width(model.wchar) / 8).to_string(),
    );
    define(
        "__SIZEOF_WINT_T__",
        (model.width(model.wint) / 8).to_string(),
    );
    if model.pointer_width == 64 {
        define("__SIZEOF_INT128__", "16".to_string());
    }
    define("__INTMAX_TYPE__", model.intmax.name().to_string());
    define_formats(&mut out, "__INTMAX", model.intmax, c23);
    let mut define = |name: &str, value: String| out.push((name.to_string(), value));
    define("__INTMAX_C_SUFFIX__", model.intmax.suffix().to_string());
    define("__INTMAX_C(c)", format!("c##{}", model.intmax.suffix()));
    define(
        "__UINTMAX_TYPE__",
        model.intmax.unsigned_of().name().to_string(),
    );
    define_formats(&mut out, "__UINTMAX", model.intmax.unsigned_of(), c23);
    let mut define = |name: &str, value: String| out.push((name.to_string(), value));
    define(
        "__UINTMAX_C_SUFFIX__",
        model.intmax.unsigned_of().suffix().to_string(),
    );
    define(
        "__UINTMAX_C(c)",
        format!("c##{}", model.intmax.unsigned_of().suffix()),
    );
    define("__PTRDIFF_TYPE__", model.ptrdiff.name().to_string());
    define_formats(&mut out, "__PTRDIFF", model.ptrdiff, c23);
    out.push((
        "__INTPTR_TYPE__".to_string(),
        model.intptr.name().to_string(),
    ));
    define_formats(&mut out, "__INTPTR", model.intptr, c23);
    out.push(("__SIZE_TYPE__".to_string(), model.size.name().to_string()));
    define_formats(&mut out, "__SIZE", model.size, c23);
    out.push(("__WCHAR_TYPE__".to_string(), model.wchar.name().to_string()));
    out.push(("__WINT_TYPE__".to_string(), model.wint.name().to_string()));
    out.push(("__SIG_ATOMIC_TYPE__".to_string(), "int".to_string()));
    out.push(("__SIG_ATOMIC_MAX__".to_string(), "2147483647".to_string()));
    out.push(("__SIG_ATOMIC_WIDTH__".to_string(), "32".to_string()));
    out.push((
        "__SIG_ATOMIC_MIN__".to_string(),
        "(-__SIG_ATOMIC_MAX__ - 1)".to_string(),
    ));
    if c23 {
        out.push(("__CHAR8_TYPE__".to_string(), "unsigned char".to_string()));
    }
    out.push(("__CHAR16_TYPE__".to_string(), "unsigned short".to_string()));
    out.push(("__CHAR32_TYPE__".to_string(), "unsigned int".to_string()));
    out.push((
        "__UINTPTR_TYPE__".to_string(),
        model.intptr.unsigned_of().name().to_string(),
    ));
    define_formats(&mut out, "__UINTPTR", model.intptr.unsigned_of(), c23);
    define_float(&mut out, "FLT", 32, "F");
    define_float(&mut out, "DBL", 64, "");
    let long_double_format = match model.long_double {
        64 => 64,
        96 => 80,
        128 if target.architecture == "x86_64" => 80,
        _ => 128,
    };
    define_float(&mut out, "LDBL", long_double_format, "L");
    out.push((
        "__POINTER_WIDTH__".to_string(),
        model.pointer_width.to_string(),
    ));
    out.push((
        "__BIGGEST_ALIGNMENT__".to_string(),
        model.biggest_alignment.to_string(),
    ));
    if !language.char_is_signed {
        out.push(("__CHAR_UNSIGNED__".to_string(), "1".to_string()));
    }
    if !model.wchar.signed() {
        out.push(("__WCHAR_UNSIGNED__".to_string(), "1".to_string()));
    }
    out.push(("__WINT_UNSIGNED__".to_string(), "1".to_string()));
    let exact = |out: &mut Vec<(String, String)>, ty: IntegerType| {
        let width = model.width(ty);
        let actual = if width == 64 {
            if ty.signed() {
                model.int64
            } else {
                model.int64.unsigned_of()
            }
        } else {
            ty
        };
        let prefix = if ty.signed() { "__INT" } else { "__UINT" };
        out.push((format!("{prefix}{width}_TYPE__"), actual.name().to_string()));
        define_formats(out, &format!("{prefix}{width}"), actual, c23);
        out.push((
            format!("{prefix}{width}_C_SUFFIX__"),
            actual.suffix().to_string(),
        ));
        let body = if actual.suffix().is_empty() {
            "c".to_string()
        } else {
            format!("c##{}", actual.suffix())
        };
        out.push((format!("{prefix}{width}_C(c)"), body));
    };
    let exact_size = |out: &mut Vec<(String, String)>, ty: IntegerType| {
        let width = model.width(ty);
        let actual = if width == 64 {
            if ty.signed() {
                model.int64
            } else {
                model.int64.unsigned_of()
            }
        } else {
            ty
        };
        let prefix = if ty.signed() { "__INT" } else { "__UINT" };
        out.push((format!("{prefix}{width}_MAX__"), model.maximum(actual)));
    };
    exact(&mut out, IntegerType::SignedChar);
    exact(&mut out, IntegerType::Short);
    exact(&mut out, IntegerType::Int);
    if model.long_width > 32 {
        exact(&mut out, IntegerType::Long);
    } else {
        exact(&mut out, IntegerType::LongLong);
    }
    exact(&mut out, IntegerType::UnsignedChar);
    exact_size(&mut out, IntegerType::UnsignedChar);
    exact_size(&mut out, IntegerType::SignedChar);
    exact(&mut out, IntegerType::UnsignedShort);
    exact_size(&mut out, IntegerType::UnsignedShort);
    exact_size(&mut out, IntegerType::Short);
    exact(&mut out, IntegerType::UnsignedInt);
    exact_size(&mut out, IntegerType::UnsignedInt);
    exact_size(&mut out, IntegerType::Int);
    let wide = if model.long_width > 32 {
        IntegerType::Long
    } else {
        IntegerType::LongLong
    };
    exact(&mut out, wide.unsigned_of());
    exact_size(&mut out, wide.unsigned_of());
    exact_size(&mut out, wide);
    for (kind, label) in [("LEAST", "__INT_LEAST"), ("FAST", "__INT_FAST")] {
        let _ = kind;
        for width in [8u32, 16, 32, 64] {
            for signed in [true, false] {
                let base = model.least(width);
                let ty = if signed { base } else { base.unsigned_of() };
                let prefix = if signed {
                    label.to_string()
                } else {
                    label.replacen("__INT", "__UINT", 1)
                };
                out.push((format!("{prefix}{width}_TYPE__"), ty.name().to_string()));
                out.push((format!("{prefix}{width}_MAX__"), model.maximum(ty)));
                if signed {
                    out.push((
                        format!("{prefix}{width}_WIDTH__"),
                        model.width(ty).to_string(),
                    ));
                }
                define_formats(&mut out, &format!("{prefix}{width}"), ty, c23);
            }
        }
    }
    let mut define = |name: &str, value: &str| out.push((name.to_string(), value.to_string()));
    define("__USER_LABEL_PREFIX__", "");
    define("__FINITE_MATH_ONLY__", "0");
    if options.gnuc_version.is_some() {
        define("__GNUC_STDC_INLINE__", "1");
        define("__GCC_ATOMIC_TEST_AND_SET_TRUEVAL", "1");
    }
    define("__GCC_DESTRUCTIVE_SIZE", "64");
    define("__GCC_CONSTRUCTIVE_SIZE", "64");
    for prefix in ["__GCC_ATOMIC_", "__CLANG_ATOMIC_"] {
        for name in [
            "BOOL", "CHAR", "CHAR16_T", "CHAR32_T", "WCHAR_T", "SHORT", "INT", "LONG", "LLONG",
            "POINTER",
        ] {
            out.push((format!("{prefix}{name}_LOCK_FREE"), "2".to_string()));
        }
        if c23 {
            out.push((format!("{prefix}CHAR8_T_LOCK_FREE"), "2".to_string()));
        }
    }
    let mut define = |name: &str, value: &str| out.push((name.to_string(), value.to_string()));
    if options.optimize == 0 || options.no_inline {
        define("__NO_INLINE__", "1");
    }
    if options.pic_level > 0 {
        let level = options.pic_level.to_string();
        define("__PIC__", &level);
        define("__pic__", &level);
        if options.pie {
            define("__PIE__", &level);
            define("__pie__", &level);
        }
    }
    define("__FLT_RADIX__", "2");
    define("__DECIMAL_DIG__", "__LDBL_DECIMAL_DIG__");
    define("__GCC_IEC_559", "2");
    define("__GCC_IEC_559_COMPLEX", "2");
    let system = target.system.as_str();
    let binary_elf = !(system.starts_with("darwin")
        || system.starts_with("macos")
        || system.starts_with("ios")
        || target.is_windows());
    if binary_elf {
        define("__ELF__", "1");
    }
    target_macros(target, &mut out);
    out
}

fn target_macros(target: &TargetInfo, out: &mut Vec<(String, String)>) {
    let mut define = |name: &str, value: &str| out.push((name.to_string(), value.to_string()));
    let system = target.system.as_str();
    if system.starts_with("linux") || target.environment.starts_with("android") {
        define("__gnu_linux__", "1");
        define("__linux", "1");
        define("__linux__", "1");
        define("__unix", "1");
        define("__unix__", "1");
        define("linux", "1");
        define("unix", "1");
        if target.environment.starts_with("android") {
            define("__ANDROID__", "1");
        }
        define("_GNU_SOURCE", "1");
    }
    match target.architecture.as_str() {
        "x86_64" => {
            define("__amd64", "1");
            define("__amd64__", "1");
            define("__x86_64", "1");
            define("__x86_64__", "1");
            define("__SEG_FS", "1");
            define("__SEG_GS", "1");
            define("__seg_fs", "__attribute__((address_space(257)))");
            define("__seg_gs", "__attribute__((address_space(256)))");
            define("__k8", "1");
            define("__k8__", "1");
            define("__tune_k8__", "1");
            define("__REGISTER_PREFIX__", "");
            define("__NO_MATH_INLINES", "1");
            define("__MMX__", "1");
            define("__SSE__", "1");
            define("__SSE2__", "1");
            define("__SSE_MATH__", "1");
            define("__SSE2_MATH__", "1");
            define("__FXSR__", "1");
            define("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_1", "1");
            define("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_2", "1");
            define("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_4", "1");
            define("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_8", "1");
            define("__SIZEOF_FLOAT128__", "16");
        }
        "i386" | "i486" | "i586" | "i686" => {
            define("i386", "1");
            define("__i386", "1");
            define("__i386__", "1");
            define("__REGISTER_PREFIX__", "");
            define("__NO_MATH_INLINES", "1");
            define("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_1", "1");
            define("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_2", "1");
            define("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_4", "1");
        }
        "aarch64" | "arm64" => {
            define("__aarch64__", "1");
            define("__AARCH64EL__", "1");
            define("__ARM_64BIT_STATE", "1");
            define("__ARM_ARCH", "8");
            define("__ARM_ARCH_ISA_A64", "1");
            define("__ARM_ARCH_PROFILE", "'A'");
            define("__ARM_FEATURE_CLZ", "1");
            define("__ARM_FEATURE_FMA", "1");
            define("__ARM_FEATURE_IDIV", "1");
            define("__ARM_FEATURE_UNALIGNED", "1");
            define("__ARM_FP", "0xE");
            define("__ARM_NEON", "1");
            define("__ARM_PCS_AAPCS64", "1");
            define("__ARM_SIZEOF_MINIMAL_ENUM", "4");
            define("__ARM_SIZEOF_WCHAR_T", "4");
            define("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_1", "1");
            define("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_2", "1");
            define("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_4", "1");
            define("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_8", "1");
        }
        architecture if architecture.starts_with("arm") || architecture.starts_with("thumb") => {
            define("__arm", "1");
            define("__arm__", "1");
            define("__ARMEL__", "1");
            define("__ARM_ARCH", "7");
            define("__ARM_ARCH_7A__", "1");
            define("__ARM_ARCH_PROFILE", "'A'");
            define("__ARM_EABI__", "1");
            define("__ARM_FEATURE_CLZ", "1");
            define("__ARM_SIZEOF_MINIMAL_ENUM", "4");
            define("__ARM_SIZEOF_WCHAR_T", "4");
            define("__APCS_32__", "1");
            define("__REGISTER_PREFIX__", "");
            if architecture.starts_with("thumb") {
                define("__thumb__", "1");
                define("__THUMBEL__", "1");
            }
        }
        _ => {}
    }
}

pub fn standard_macros(options: &PreprocessorOptions) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if !options.language.traditional {
        out.push(("__STDC__".to_string(), "1".to_string()));
    }
    out.push((
        "__STDC_HOSTED__".to_string(),
        if options.freestanding { "0" } else { "1" }.to_string(),
    ));
    if let Some(version) = options.language.standard.version_macro() {
        out.push(("__STDC_VERSION__".to_string(), version.to_string()));
    }
    out.push(("__STDC_UTF_16__".to_string(), "1".to_string()));
    out.push(("__STDC_UTF_32__".to_string(), "1".to_string()));
    out.push(("__STDC_EMBED_NOT_FOUND__".to_string(), "0".to_string()));
    out.push(("__STDC_EMBED_FOUND__".to_string(), "1".to_string()));
    out.push(("__STDC_EMBED_EMPTY__".to_string(), "2".to_string()));
    if options.language.assembly {
        out.push(("__ASSEMBLER__".to_string(), "1".to_string()));
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontendAction {
    Preprocess,
    PreprocessOnly,
    SyntaxOnly,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputLanguage {
    C,
    CHeader,
    PreprocessedC,
    Assembler,
    CPlusPlus,
    ObjectiveC,
    Other,
}

#[derive(Clone, Debug)]
pub struct CompilerInvocation {
    pub inputs: Vec<String>,
    pub output: Option<String>,
    pub action: FrontendAction,
    pub language_kind: Option<InputLanguage>,
    pub preprocessor: PreprocessorOptions,
    pub verify: Option<Vec<String>>,
    pub unsupported: Vec<String>,
    pub show_source: bool,
    pub dump_macros: bool,
}

impl CompilerInvocation {
    pub fn input_language(&self, path: &str) -> InputLanguage {
        if let Some(kind) = self.language_kind {
            return kind;
        }
        let extension = path
            .rsplit_once('.')
            .map(|(_, extension)| extension)
            .unwrap_or("");
        match extension {
            "c" => InputLanguage::C,
            "h" => InputLanguage::CHeader,
            "i" => InputLanguage::PreprocessedC,
            "S" | "s" | "sx" => InputLanguage::Assembler,
            "cpp" | "cc" | "cxx" | "C" | "hpp" | "hh" | "ii" | "cppm" | "cu" | "hip" => {
                InputLanguage::CPlusPlus
            }
            "m" | "mm" | "mi" => InputLanguage::ObjectiveC,
            _ => InputLanguage::C,
        }
    }
}

fn parse_gnuc_version(text: &str) -> Option<(u32, u32, u32)> {
    let mut parts = text.split('.').map(|part| part.parse::<u32>().ok());
    let major = parts.next()??;
    let minor = parts.next().flatten().unwrap_or(0);
    let patch = parts.next().flatten().unwrap_or(0);
    if major == 0 {
        return None;
    }
    Some((major, minor, patch))
}

pub fn parse_compiler_arguments(arguments: &[String]) -> CompilerInvocation {
    let mut invocation = CompilerInvocation {
        inputs: Vec::new(),
        output: None,
        action: FrontendAction::SyntaxOnly,
        language_kind: None,
        preprocessor: PreprocessorOptions::default(),
        verify: None,
        unsupported: Vec::new(),
        show_source: true,
        dump_macros: false,
    };
    let mut standard: Option<String> = None;
    let mut trigraphs: Option<bool> = None;
    let mut dollars: Option<bool> = None;
    let mut signed_char: Option<bool> = None;
    let mut microsoft = false;
    let mut traditional = false;
    let mut after_directories: Vec<SearchDirectory> = Vec::new();
    let options = &mut invocation.preprocessor;
    let mut index = 0usize;
    let take_value = |index: &mut usize, flag: &str, inline: &str| -> Option<String> {
        if !inline.is_empty() {
            return Some(inline.to_string());
        }
        *index += 1;
        let _ = flag;
        arguments.get(*index).cloned()
    };
    while index < arguments.len() {
        let argument = arguments[index].as_str();
        if !argument.starts_with('-') || argument == "-" {
            invocation.inputs.push(argument.to_string());
            index += 1;
            continue;
        }
        match argument {
            "-cc1" => {}
            "-E" => invocation.action = FrontendAction::Preprocess,
            "-Eonly" => invocation.action = FrontendAction::PreprocessOnly,
            "-fsyntax-only" => invocation.action = FrontendAction::SyntaxOnly,
            "-P" => options.line_markers = false,
            "-C" => options.keep_comments = true,
            "-CC" => {
                options.keep_comments = true;
                options.keep_macro_comments = true;
            }
            "-dD" => options.dump_defines = true,
            "-dM" => invocation.dump_macros = true,
            "-fuse-line-directives" => options.use_line_directives = true,
            "-fminimize-whitespace" => options.minimize_whitespace = true,
            "-w" => options.warnings.suppress_all = true,
            "-pedantic" | "-Wpedantic" => options.warnings.pedantic = true,
            "-pedantic-errors" => options.warnings.pedantic_errors = true,
            "-Werror" => options.warnings.warnings_as_errors = true,
            "-Weverything" => options.warnings.everything = true,
            "-Wsystem-headers" => options.warnings.show_in_system_headers = true,
            "-ffreestanding" => options.freestanding = true,
            "-fms-extensions" => microsoft = true,
            "-fdollars-in-identifiers" => dollars = Some(true),
            "-fno-dollars-in-identifiers" => dollars = Some(false),
            "-ftrigraphs" | "-trigraphs" => trigraphs = Some(true),
            "-fno-trigraphs" => trigraphs = Some(false),
            "-traditional-cpp" => traditional = true,
            "-undef" => options.undefine_predefined = true,
            "-fsigned-char" => signed_char = Some(true),
            "-funsigned-char" | "-fno-signed-char" => signed_char = Some(false),
            "-fno-operator-names" => {}
            "-nostdsysteminc" | "-nobuiltininc" | "-nostdinc" | "-nostdinc++" => {}
            "-fcolor-diagnostics"
            | "-fno-color-diagnostics"
            | "-fdiagnostics-show-option"
            | "-fno-diagnostics-show-option"
            | "-fno-show-column"
            | "-fshow-column"
            | "-fno-builtin"
            | "-ffile-reproducible"
            | "-fno-file-reproducible"
            | "-fgnu-keywords"
            | "-fno-gnu-keywords"
            | "-fno-signed-wchar"
            | "-fsigned-wchar"
            | "-fno-spell-checking"
            | "-fno-diagnostics-fixit-info"
            | "-fdiagnostics-parseable-fixits"
            | "-fno-common"
            | "-fcommon"
            | "-disable-free"
            | "-fexceptions"
            | "-fno-exceptions"
            | "-fgnu89-inline"
            | "-fno-math-errno"
            | "-fmath-errno"
            | "-ffast-math"
            | "-fno-rtti"
            | "-fno-elide-type"
            | "-fno-integrated-cc1"
            | "-fgnuc-version=0" => {}
            "-fno-caret-diagnostics" => invocation.show_source = false,
            "-pic-is-pie" => options.pie = true,
            "-fno-inline" | "-fno-inline-functions" => options.no_inline = true,
            "-finline-functions" | "-finline-hint-functions" => options.no_inline = false,
            "-fcaret-diagnostics" => invocation.show_source = true,
            "-verify" => invocation.verify = Some(vec!["expected".to_string()]),
            _ => {
                if let Some(rest) = argument.strip_prefix("-verify=") {
                    let mut prefixes: Vec<String> = rest.split(',').map(str::to_string).collect();
                    prefixes.sort();
                    prefixes.dedup();
                    let current = invocation.verify.get_or_insert_with(Vec::new);
                    current.retain(|held| {
                        held != "expected" || rest.split(',').any(|item| item == "expected")
                    });
                    current.extend(prefixes);
                    current.sort();
                    current.dedup();
                } else if let Some(rest) = argument.strip_prefix("-D") {
                    if let Some(value) = take_value(&mut index, "-D", rest) {
                        options.macros.push(MacroOption::Define(value));
                    }
                } else if let Some(rest) = argument.strip_prefix("-U") {
                    if let Some(value) = take_value(&mut index, "-U", rest) {
                        options.macros.push(MacroOption::Undefine(value));
                    }
                } else if argument == "-iquote" || argument.starts_with("-iquote") {
                    let rest = argument.strip_prefix("-iquote").unwrap_or("");
                    if let Some(value) = take_value(&mut index, "-iquote", rest) {
                        options.quoted_directories.push(value);
                    }
                } else if argument == "-isystem"
                    || argument == "-internal-isystem"
                    || argument == "-isystem-after"
                {
                    if let Some(value) = take_value(&mut index, argument, "") {
                        options.directories.push(SearchDirectory {
                            path: value,
                            character: FileCharacter::System,
                        });
                    }
                } else if argument == "-internal-externc-isystem" {
                    if let Some(value) = take_value(&mut index, argument, "") {
                        options.directories.push(SearchDirectory {
                            path: value,
                            character: FileCharacter::ExternCSystem,
                        });
                    }
                } else if argument == "-idirafter" {
                    if let Some(value) = take_value(&mut index, argument, "") {
                        after_directories.push(SearchDirectory {
                            path: value,
                            character: FileCharacter::System,
                        });
                    }
                } else if let Some(rest) = argument.strip_prefix("-isystem") {
                    options.directories.push(SearchDirectory {
                        path: rest.to_string(),
                        character: FileCharacter::System,
                    });
                } else if let Some(rest) = argument.strip_prefix("-I") {
                    if let Some(value) = take_value(&mut index, "-I", rest) {
                        options.directories.push(SearchDirectory {
                            path: value,
                            character: FileCharacter::User,
                        });
                    }
                } else if argument == "-include" {
                    if let Some(value) = take_value(&mut index, argument, "") {
                        options.forced_includes.push(value);
                    }
                } else if argument == "-o" || (argument.starts_with("-o") && argument.len() > 2) {
                    let rest = argument.strip_prefix("-o").unwrap_or("");
                    invocation.output = take_value(&mut index, "-o", rest);
                } else if let Some(rest) = argument.strip_prefix("-std=") {
                    standard = Some(rest.to_string());
                } else if argument == "-x" {
                    let value = take_value(&mut index, "-x", "").unwrap_or_default();
                    invocation.language_kind = Some(match value.as_str() {
                        "c" => InputLanguage::C,
                        "c-header" => InputLanguage::CHeader,
                        "cpp-output" | "c-cpp-output" => InputLanguage::PreprocessedC,
                        "assembler-with-cpp" => InputLanguage::Assembler,
                        "c++" | "c++-header" | "c++-cpp-output" | "cuda" | "hip" => {
                            InputLanguage::CPlusPlus
                        }
                        "objective-c"
                        | "objective-c++"
                        | "objective-c-header"
                        | "objective-c++-header" => InputLanguage::ObjectiveC,
                        _ => InputLanguage::Other,
                    });
                } else if argument == "-triple" || argument == "-target" {
                    if let Some(value) = take_value(&mut index, argument, "") {
                        options.target = TargetInfo::from_triple(&value);
                    }
                } else if let Some(rest) = argument.strip_prefix("-triple=") {
                    options.target = TargetInfo::from_triple(rest);
                } else if let Some(rest) = argument.strip_prefix("--target=") {
                    options.target = TargetInfo::from_triple(rest);
                } else if argument == "-ferror-limit" || argument.starts_with("-ferror-limit=") {
                    let rest = argument.strip_prefix("-ferror-limit=").unwrap_or("");
                    let value = take_value(&mut index, "-ferror-limit", rest);
                    options.warnings.error_limit =
                        value.and_then(|value| value.parse().ok()).unwrap_or(0);
                } else if let Some(level) = argument.strip_prefix("-O") {
                    let (optimize, size) = match level {
                        "0" => (0, false),
                        "fast" | "4" => (3, false),
                        "s" | "z" => (2, true),
                        "g" | "" => (1, false),
                        other => (other.parse::<u32>().unwrap_or(0).min(3), false),
                    };
                    options.optimize = optimize;
                    options.optimize_size = size;
                } else if argument == "-pic-level" {
                    let value = take_value(&mut index, argument, "");
                    options.pic_level = value.and_then(|value| value.parse().ok()).unwrap_or(0);
                } else if argument == "-main-file-name"
                    || argument == "-fmacro-backtrace-limit"
                    || argument == "-target-cpu"
                    || argument == "-target-abi"
                    || argument == "-target-feature"
                    || argument == "-mrelocation-model"
                    || argument == "-fmessage-length"
                    || argument == "-ftabstop"
                {
                    let _ = take_value(&mut index, argument, "");
                } else if let Some(rest) = argument.strip_prefix("-fgnuc-version=") {
                    options.gnuc_version = parse_gnuc_version(rest);
                } else if let Some(rest) = argument.strip_prefix("-source-date-epoch") {
                    let value = take_value(&mut index, argument, rest.trim_start_matches('='));
                    options.source_date_epoch = value.and_then(|value| value.parse().ok());
                } else if argument.starts_with("-fmacro-backtrace-limit=")
                    || argument.starts_with("-fmessage-length=")
                    || argument.starts_with("-fdiagnostics-")
                    || argument.starts_with("-g")
                    || argument.starts_with("-Wundef-prefix=")
                {
                } else if let Some(rest) = argument.strip_prefix("-W") {
                    if !rest.is_empty() {
                        options.warnings.flags.push(rest.to_string());
                    }
                } else {
                    invocation.unsupported.push(argument.to_string());
                }
            }
        }
        index += 1;
    }
    if !after_directories.is_empty() {
        options.directories.extend(after_directories);
    }
    let target_char_signed = options.target.char_signed();
    let input_kind = invocation
        .inputs
        .first()
        .map(|path| invocation.input_language(path))
        .unwrap_or(InputLanguage::C);
    let mut language = match &standard {
        Some(name) => match LanguageOptions::for_standard(name) {
            Some(language) => language,
            None => {
                invocation.unsupported.push(format!("-std={name}"));
                LanguageOptions::default()
            }
        },
        None => LanguageOptions::default(),
    };
    language.header_file = input_kind == InputLanguage::CHeader;
    if input_kind == InputLanguage::Assembler {
        language.assembly = true;
        language.dollar_identifiers = false;
    }
    if let Some(value) = trigraphs {
        language.trigraphs = value;
    }
    if let Some(value) = dollars {
        language.dollar_identifiers = value;
    }
    language.microsoft_extensions = microsoft;
    language.traditional = traditional;
    language.char_is_signed = signed_char.unwrap_or(target_char_signed);
    language.preprocessed_output = invocation.action == FrontendAction::PreprocessOnly
        || (invocation.action == FrontendAction::Preprocess && !invocation.dump_macros);
    let options = &mut invocation.preprocessor;
    options.language = language;
    if invocation.action == FrontendAction::Preprocess {
        options.output = if invocation.dump_macros {
            OutputMode::MacrosOnly
        } else {
            OutputMode::Preprocessed
        };
    } else {
        options.output = OutputMode::Nothing;
    }
    options.syntax_only = invocation.action == FrontendAction::SyntaxOnly;
    invocation
}
