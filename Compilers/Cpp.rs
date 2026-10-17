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
    pub declspec_keyword: bool,
    pub gnu_keywords: bool,
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
            declspec_keyword: false,
            gnu_keywords: gnu,
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
            declspec_keyword: false,
            gnu_keywords: true,
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
    PragmaAnnotation,
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
    base_keep_comments: bool,
    keep_whitespace: bool,
    base_keep_whitespace: bool,
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
            base_keep_comments: false,
            keep_whitespace: false,
            base_keep_whitespace: false,
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

    fn retains_comments(&self) -> bool {
        self.keep_comments || self.keep_whitespace
    }

    fn reset_extended_mode(&mut self) {
        if self.base_keep_whitespace {
            self.keep_whitespace = true;
        } else {
            self.keep_comments = self.base_keep_comments;
        }
    }

    fn skip_whitespace(&mut self, start: usize, mut at: usize, flags: &mut u16) -> Option<Token> {
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
        if self.keep_whitespace {
            let token = self.form(start, at, TokenKind::Unknown, *flags);
            if saw_newline {
                self.at_start_of_line = true;
                self.at_physical_start_of_line = true;
            }
            return Some(token);
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
        if self.retains_comments() {
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
        if self.retains_comments() {
            let token = self.form(start, at, TokenKind::Comment, *flags);
            self.at_physical_start_of_line = token.has(PHYSICAL_START_OF_LINE);
            return Ok(Some(token));
        }
        if !self.at_end(at) && is_horizontal_space(self.byte(at)) {
            self.skip_whitespace(at, at + 1, flags);
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
                let begin = at;
                while !self.at_end(at) && is_horizontal_space(self.byte(at)) {
                    at += 1;
                }
                if self.keep_whitespace {
                    return self.form(begin, at, TokenKind::Unknown, flags);
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
                    if self.at_end(start) || after > self.bytes.len() {
                        return self.lex_end_of_buffer(start, flags);
                    }
                    self.report(Message::NullInFile, start, Vec::new());
                    flags |= LEADING_SPACE;
                    if let Some(token) = self.skip_whitespace(start, at, &mut flags) {
                        return token;
                    }
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
                        self.reset_extended_mode();
                        self.at_start_of_line = true;
                        self.at_physical_start_of_line = true;
                        let token = self.form(start, at, TokenKind::EndOfDirective, flags);
                        return token;
                    }
                    flags &= !LEADING_SPACE;
                    if let Some(token) = self.skip_whitespace(start, at, &mut flags) {
                        return token;
                    }
                    flags &= !NEEDS_CLEANING;
                    continue 'restart;
                }
                b' ' | b'\t' | 0x0c | 0x0b => {
                    flags |= LEADING_SPACE;
                    if let Some(token) = self.skip_whitespace(start, at, &mut flags) {
                        return token;
                    }
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
            self.reset_extended_mode();
            let mut token = Token::marker(TokenKind::EndOfDirective, self.place(at));
            token.flags = flags;
            self.position = at;
            return token;
        }
        self.position = self.bytes.len();
        let mut end = self.bytes.len();
        if end > 0 && matches!(self.bytes[end - 1], b'\n' | b'\r') {
            end -= 1;
            if end > 0
                && matches!(self.bytes[end - 1], b'\n' | b'\r')
                && self.bytes[end - 1] != self.bytes[end]
            {
                end -= 1;
            }
        }
        let mut token = Token::marker(TokenKind::EndOfFile, self.place(end));
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
    WarningOffInSystemHeaders,
    WarningNoWerror,
    WarningInSystemHeaders,
    Extension,
    ExtWarn,
    ExtWarnInSystemHeaders,
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
    UndefPrefixIdentifier,
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
    StdcPragmaIgnored,
    StdcFenvRoundNotSupported,
    StdcUnknownRoundingMode,
    PragmaFloatingPointIgnored,
    PragmaExpectedIdentifier,
    PragmaExtraTokensAtEnd,
    PragmaCommentMalformed,
    PragmaCommentUnknownKind,
    PragmaCommentIgnored,
    PragmaCommentOnce,
    PragmaDeprecatedMacroUse,
    PragmaRestrictExpansionMacroUse,
    MacroAnnotationNote,
    PragmaFinalMacro,
    VisibilityNonMacro,
    NanInfinityDisabled,
    EmptyTranslationUnit,
    ExtraSemi,
    ExtraneousClosingBrace,
    ExpectedExternalDeclaration,
    PragmaFenvRequiresPrecise,
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
    FileNotFoundTypo,
    HeaderShadowing,
    SuggestParenthesesForMacro,
    InitializerListAtArgumentStart,
    AuditSyntax,
    AuditDoubleBegin,
    AuditUnmatchedEnd,
    AuditInclude,
    AuditEndOfFile,
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
            Message::UndefPrefixIdentifier => {
                (WarningOff, "undef-prefix", "%0 is not defined, evaluates to 0")
            }
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
            Message::SuggestParenthesesForMacro => (
                Note,
                "",
                "parentheses are required around macro argument containing braced initializer list",
            ),
            Message::InitializerListAtArgumentStart => (
                Note,
                "",
                "cannot use initializer list at the beginning of a macro argument",
            ),
            Message::HeaderShadowing => (
                WarningOff,
                "shadow-header",
                "multiple candidates for header '%0' found; directory '%1' chosen, ignoring others including '%2'",
            ),
            Message::FileNotFoundTypo => (Error, "", "'%0' file not found, did you mean '%1'?"),
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
            Message::StdcPragmaIgnored => {
                (ExtWarn, "unknown-pragmas", "unknown pragma in STDC namespace")
            }
            Message::StdcFenvRoundNotSupported => {
                (Warning, "unknown-pragmas", "pragma STDC FENV_ROUND is not supported")
            }
            Message::StdcUnknownRoundingMode => (
                Warning,
                "ignored-pragmas",
                "invalid or unsupported rounding mode in '#pragma STDC FENV_ROUND' - ignored",
            ),
            Message::PragmaFloatingPointIgnored => (
                Warning,
                "ignored-pragmas",
                "'#pragma %0' is not supported on this target - ignored",
            ),
            Message::PragmaExpectedIdentifier => (
                Warning,
                "ignored-pragmas",
                "expected identifier in '#pragma %0' - ignored",
            ),
            Message::PragmaExtraTokensAtEnd => (
                Warning,
                "ignored-pragmas",
                "extra tokens at end of '#pragma %0' - ignored",
            ),
            Message::PragmaCommentMalformed => (
                Error,
                "",
                "pragma comment requires parenthesized identifier and optional string",
            ),
            Message::PragmaCommentUnknownKind => (Error, "", "unknown kind of pragma comment"),
            Message::PragmaDeprecatedMacroUse => (
                ExtWarn,
                "deprecated-pragma",
                "macro %0 has been marked as deprecated%select{|: %2}1",
            ),
            Message::PragmaRestrictExpansionMacroUse => (
                ExtWarn,
                "restrict-expansion",
                "macro %0 has been marked as unsafe for use in headers%select{|: %2}1",
            ),
            Message::MacroAnnotationNote => (
                Note,
                "",
                "macro marked '%select{deprecated|restrict_expansion|final}0' here",
            ),
            Message::PragmaFinalMacro => (
                ExtWarnInSystemHeaders,
                "final-macro",
                "macro %0 has been marked as final and should not be %select{undefined|redefined}1",
            ),
            Message::VisibilityNonMacro => (Error, "", "no macro named %0"),
            Message::NanInfinityDisabled => (
                Warning,
                "nan-infinity-disabled",
                "use of %select{infinity|NaN}0%select{| via a macro}1 is undefined behavior due to the currently enabled floating-point options",
            ),
            Message::EmptyTranslationUnit => (
                Extension,
                "empty-translation-unit",
                "ISO C requires a translation unit to contain at least one declaration",
            ),
            Message::ExtraSemi => (
                Extension,
                "extra-semi",
                "extra ';' %select{outside of a function|inside a %1|inside instance variable list|after member function definition}0",
            ),
            Message::ExtraneousClosingBrace => (Error, "", "extraneous closing brace ('}')"),
            Message::ExpectedExternalDeclaration => (Error, "", "expected external declaration"),
            Message::PragmaFenvRequiresPrecise => (
                Error,
                "",
                "'#pragma STDC FENV_ACCESS ON' is illegal when precise is disabled",
            ),
            Message::PragmaCommentIgnored => {
                (Warning, "ignored-pragmas", "'#pragma comment %0' ignored")
            }
            Message::PragmaCommentOnce => (
                Warning,
                "ignored-pragmas",
                "'#pragma comment %0' ignored: it can be specified only once per translation unit",
            ),
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
                WarningOffInSystemHeaders,
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
            Message::AuditSyntax => (Error, "", "expected 'begin' or 'end'"),
            Message::AuditDoubleBegin => (Error, "", "already inside '#pragma clang %0'"),
            Message::AuditUnmatchedEnd => (Error, "", "not currently inside '#pragma clang %0'"),
            Message::AuditInclude => (
                Error,
                "",
                "cannot %select{#include files|import headers}0 inside '#pragma clang %1'",
            ),
            Message::AuditEndOfFile => (Error, "", "'#pragma clang %0' was not ended within this file"),
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
            MessageClass::Extension
                | MessageClass::ExtWarn
                | MessageClass::ExtWarnInSystemHeaders
                | MessageClass::ExtWarnError
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
    pub undef_prefixes: Vec<String>,
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
            | MessageClass::ExtWarn
            | MessageClass::ExtWarnInSystemHeaders => CSeverity::Warning,
            MessageClass::WarningOff
            | MessageClass::WarningOffInSystemHeaders
            | MessageClass::Extension => CSeverity::Ignored,
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
            && class != MessageClass::ExtWarnInSystemHeaders
            && class != MessageClass::WarningOffInSystemHeaders
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
    DiagnosticRenderer::default().render(diagnostic, map, show_source)
}

#[derive(Clone, Debug, Default)]
pub struct DiagnosticRenderer {
    last_include: Option<Place>,
    last_place: Option<Place>,
    last_severity: Option<CSeverity>,
}

impl DiagnosticRenderer {
    pub fn render(
        &mut self,
        diagnostic: &CDiagnostic,
        map: &SourceMap,
        show_source: bool,
    ) -> String {
        let mut out = String::new();
        let include = diagnostic.presumed.as_ref().and_then(|place| place.include);
        if include != self.last_include {
            self.last_include = include;
            if diagnostic.severity != CSeverity::Note {
                for entry in diagnostic.include_stack.iter() {
                    out.push_str(&format!(
                        "In file included from {}:{}:\n",
                        entry.name, entry.line
                    ));
                }
            }
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
        let repeated = diagnostic.place.is_some()
            && diagnostic.place == self.last_place
            && (self.last_severity != Some(CSeverity::Note)
                || Some(diagnostic.severity) == self.last_severity);
        if show_source && !repeated {
            if let Some(place) = diagnostic.place {
                if (place.buffer as usize) < map.buffers.len() {
                    let buffer = map.buffer(place.buffer);
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
        self.last_place = diagnostic.place;
        self.last_severity = Some(diagnostic.severity);
        out
    }
}

static KEYWORD_TABLE: [(&str, &str, u32); 418] = [
    ("auto", "auto", 0x3bffe7ff),
    ("break", "break", 0x3bffe7ff),
    ("case", "case", 0x3bffe7ff),
    ("char", "char", 0x3bffe7ff),
    ("const", "const", 0x3bffe7ff),
    ("continue", "continue", 0x3bffe7ff),
    ("default", "default", 0x3bffe7ff),
    ("do", "do", 0x3bffe7ff),
    ("double", "double", 0x3bffe7ff),
    ("else", "else", 0x3bffe7ff),
    ("enum", "enum", 0x3bffe7ff),
    ("extern", "extern", 0x3bffe7ff),
    ("float", "float", 0x3bffe7ff),
    ("for", "for", 0x3bffe7ff),
    ("goto", "goto", 0x3bffe7ff),
    ("if", "if", 0x3bffe7ff),
    ("int", "int", 0x3bffe7ff),
    ("_ExtInt", "_ExtInt", 0x3bffe7ff),
    ("_BitInt", "_BitInt", 0x3bffe7ff),
    ("long", "long", 0x3bffe7ff),
    ("register", "register", 0x3bffe7ff),
    ("return", "return", 0x3bffe7ff),
    ("short", "short", 0x3bffe7ff),
    ("signed", "signed", 0x3bffe7ff),
    ("static", "static", 0x3bffe7ff),
    ("struct", "struct", 0x3bffe7ff),
    ("switch", "switch", 0x3bffe7ff),
    ("typedef", "typedef", 0x3bffe7ff),
    ("union", "union", 0x3bffe7ff),
    ("unsigned", "unsigned", 0x3bffe7ff),
    ("void", "void", 0x3bffe7ff),
    ("volatile", "volatile", 0x7bffe7ff),
    ("while", "while", 0x3bffe7ff),
    ("_Alignas", "_Alignas", 0x3bffe7ff),
    ("_Alignof", "_Alignof", 0x3bffe7ff),
    ("_Atomic", "_Atomic", 0x3bfff7ff),
    ("_Bool", "_Bool", 0x80),
    ("_Complex", "_Complex", 0x3bffe7ff),
    ("_Generic", "_Generic", 0x3bffe7ff),
    ("_Imaginary", "_Imaginary", 0x3bffe7ff),
    ("_Noreturn", "_Noreturn", 0x3bffe7ff),
    ("_Static_assert", "_Static_assert", 0x3bffe7ff),
    ("_Thread_local", "_Thread_local", 0x3bffe7ff),
    ("__func__", "__func__", 0x3bffe7ff),
    ("__objc_yes", "__objc_yes", 0x3bffe7ff),
    ("__objc_no", "__objc_no", 0x3bffe7ff),
    ("__ptrauth", "__ptrauth", 0x3bffe7ff),
    ("__ob_wrap", "__ob_wrap", 0x3bffe7ff),
    ("__ob_trap", "__ob_trap", 0x3bffe7ff),
    ("asm", "asm", 0xa),
    ("bool", "bool", 0x420),
    ("catch", "catch", 0x2),
    ("class", "class", 0x2),
    ("const_cast", "const_cast", 0x2),
    ("delete", "delete", 0x2),
    ("dynamic_cast", "dynamic_cast", 0x2),
    ("explicit", "explicit", 0x2),
    ("export", "export", 0x2),
    ("false", "false", 0x420),
    ("friend", "friend", 0x2),
    ("mutable", "mutable", 0x2),
    ("namespace", "namespace", 0x2),
    ("new", "new", 0x2),
    ("operator", "operator", 0x2),
    ("private", "private", 0x2),
    ("protected", "protected", 0x2),
    ("public", "public", 0x2),
    ("reinterpret_cast", "reinterpret_cast", 0x2),
    ("static_cast", "static_cast", 0x2),
    ("template", "template", 0x2),
    ("this", "this", 0x2),
    ("throw", "throw", 0x2),
    ("true", "true", 0x420),
    ("try", "try", 0x2),
    ("typename", "typename", 0x2),
    ("typeid", "typeid", 0x2),
    ("using", "using", 0x2),
    ("virtual", "virtual", 0x2),
    ("wchar_t", "wchar_t", 0x2000),
    ("restrict", "restrict", 0x1),
    ("inline", "inline", 0xb),
    ("alignas", "alignas", 0x404),
    ("char16_t", "char16_t", 0x804),
    ("char32_t", "char32_t", 0x804),
    ("constexpr", "constexpr", 0x404),
    ("decltype", "decltype", 0x4),
    ("noexcept", "noexcept", 0x4),
    ("nullptr", "nullptr", 0x404),
    ("static_assert", "static_assert", 0x400404),
    ("thread_local", "thread_local", 0x404),
    ("co_await", "co_await", 0x140000),
    ("co_return", "co_return", 0x140000),
    ("co_yield", "co_yield", 0x140000),
    ("module", "module", 0x80000),
    ("import", "import", 0x80000),
    ("consteval", "consteval", 0x100000),
    ("constinit", "constinit", 0x100000),
    ("concept", "concept", 0x100000),
    ("requires", "requires", 0x100000),
    ("char8_t", "char8_t", 0x8000),
    ("_Float16", "_Float16", 0x3bffe7ff),
    ("typeof", "typeof", 0x408),
    ("typeof_unqual", "typeof_unqual", 0x400),
    ("_Defer", "_Defer", 0x20000000),
    ("_Accum", "_Accum", 0x10000000),
    ("_Fract", "_Fract", 0x10000000),
    ("_Sat", "_Sat", 0x10000000),
    ("_Decimal32", "_Decimal32", 0x3bffe7ff),
    ("_Decimal64", "_Decimal64", 0x3bffe7ff),
    ("_Decimal128", "_Decimal128", 0x3bffe7ff),
    ("__null", "__null", 0x2),
    ("__attribute", "__attribute", 0x3bffe7ff),
    ("__builtin_choose_expr", "__builtin_choose_expr", 0x3bffe7ff),
    ("__builtin_offsetof", "__builtin_offsetof", 0x3bffe7ff),
    ("__builtin_FILE", "__builtin_FILE", 0x3bffe7ff),
    ("__builtin_FILE_NAME", "__builtin_FILE_NAME", 0x3bffe7ff),
    ("__builtin_FUNCTION", "__builtin_FUNCTION", 0x3bffe7ff),
    ("__builtin_FUNCSIG", "__builtin_FUNCSIG", 0x10),
    ("__builtin_LINE", "__builtin_LINE", 0x3bffe7ff),
    ("__builtin_COLUMN", "__builtin_COLUMN", 0x3bffe7ff),
    (
        "__builtin_source_location",
        "__builtin_source_location",
        0x2,
    ),
    ("__builtin_va_arg", "__builtin_va_arg", 0x3bffe7ff),
    ("__extension__", "__extension__", 0x3bffe7ff),
    ("__float128", "__float128", 0x3bffe7ff),
    ("__ibm128", "__ibm128", 0x3bffe7ff),
    ("__imag", "__imag", 0x3bffe7ff),
    ("__int128", "__int128", 0x3bffe7ff),
    ("__label__", "__label__", 0x3bffe7ff),
    ("__real", "__real", 0x3bffe7ff),
    ("__thread", "__thread", 0x3bffe7ff),
    ("__FUNCTION__", "__FUNCTION__", 0x3bffe7ff),
    ("__PRETTY_FUNCTION__", "__PRETTY_FUNCTION__", 0x3bffe7ff),
    ("__auto_type", "__auto_type", 0x3bffe7ff),
    ("__FUNCDNAME__", "__FUNCDNAME__", 0x10),
    ("__FUNCSIG__", "__FUNCSIG__", 0x10),
    ("L__FUNCTION__", "L__FUNCTION__", 0x10),
    ("L__FUNCSIG__", "L__FUNCSIG__", 0x10),
    ("__private_extern__", "__private_extern__", 0x3bffe7ff),
    ("__module_private__", "__module_private__", 0x3bffe7ff),
    ("__declspec", "__declspec", 0x0),
    ("__cdecl", "__cdecl", 0x3bffe7ff),
    ("__stdcall", "__stdcall", 0x3bffe7ff),
    ("__fastcall", "__fastcall", 0x3bffe7ff),
    ("__thiscall", "__thiscall", 0x3bffe7ff),
    ("__regcall", "__regcall", 0x3bffe7ff),
    ("__vectorcall", "__vectorcall", 0x3bffe7ff),
    ("__forceinline", "__forceinline", 0x10),
    ("__unaligned", "__unaligned", 0x10),
    ("__super", "__super", 0x10),
    ("__global", "__global", 0x200200),
    ("__local", "__local", 0x200200),
    ("__constant", "__constant", 0x200200),
    ("__private", "__private", 0x200200),
    ("__generic", "__generic", 0x200200),
    ("global", "__global", 0x200200),
    ("local", "__local", 0x200200),
    ("constant", "__constant", 0x200200),
    ("private", "__private", 0x200),
    ("generic", "__generic", 0x200200),
    ("__kernel", "__kernel", 0x200200),
    ("kernel", "__kernel", 0x200200),
    ("__read_only", "__read_only", 0x200200),
    ("__write_only", "__write_only", 0x200200),
    ("__read_write", "__read_write", 0x200200),
    ("read_only", "__read_only", 0x200200),
    ("write_only", "__write_only", 0x200200),
    ("read_write", "__read_write", 0x200200),
    ("__builtin_astype", "__builtin_astype", 0x200200),
    ("image1d_t", "image1d_t", 0x200200),
    ("image1d_array_t", "image1d_array_t", 0x200200),
    ("image1d_buffer_t", "image1d_buffer_t", 0x200200),
    ("image2d_t", "image2d_t", 0x200200),
    ("image2d_array_t", "image2d_array_t", 0x200200),
    ("image2d_depth_t", "image2d_depth_t", 0x200200),
    ("image2d_array_depth_t", "image2d_array_depth_t", 0x200200),
    ("image2d_msaa_t", "image2d_msaa_t", 0x200200),
    ("image2d_array_msaa_t", "image2d_array_msaa_t", 0x200200),
    ("image2d_msaa_depth_t", "image2d_msaa_depth_t", 0x200200),
    (
        "image2d_array_msaa_depth_t",
        "image2d_array_msaa_depth_t",
        0x200200,
    ),
    ("image3d_t", "image3d_t", 0x200200),
    ("pipe", "pipe", 0x200200),
    ("addrspace_cast", "addrspace_cast", 0x200000),
    ("__noinline__", "__noinline__", 0x1000000),
    ("cbuffer", "cbuffer", 0x8000000),
    ("tbuffer", "tbuffer", 0x8000000),
    ("groupshared", "groupshared", 0x8000000),
    ("in", "in", 0x8000000),
    ("inout", "inout", 0x8000000),
    ("out", "out", 0x8000000),
    ("row_major", "row_major", 0x8000000),
    ("column_major", "column_major", 0x8000000),
    ("__hlsl_resource_t", "__hlsl_resource_t", 0x8000000),
    ("__pascal", "__pascal", 0x3bffe7ff),
    ("__vector", "__vector", 0x20040),
    ("__pixel", "__pixel", 0x40),
    ("__bool", "__bool", 0x20040),
    ("__fp16", "half", 0x3bffe7ff),
    ("__bf16", "__bf16", 0x3bffe7ff),
    ("half", "half", 0x4000),
    ("__bridge", "__bridge", 0x10000),
    ("__bridge_transfer", "__bridge_transfer", 0x10000),
    ("__bridge_retained", "__bridge_retained", 0x10000),
    ("__bridge_retain", "__bridge_retain", 0x10000),
    ("__covariant", "__covariant", 0x10000),
    ("__contravariant", "__contravariant", 0x10000),
    ("__kindof", "__kindof", 0x10000),
    ("__asm", "asm", 0x3bffe7ff),
    ("__asm__", "asm", 0x3bffe7ff),
    ("__attribute__", "__attribute", 0x3bffe7ff),
    ("__complex", "_Complex", 0x3bffe7ff),
    ("__complex__", "_Complex", 0x3bffe7ff),
    ("__const", "const", 0x3bffe7ff),
    ("__const__", "const", 0x3bffe7ff),
    ("__decltype", "decltype", 0x2),
    ("__imag__", "__imag", 0x3bffe7ff),
    ("__inline", "inline", 0x3bffe7ff),
    ("__inline__", "inline", 0x3bffe7ff),
    ("__nullptr", "nullptr", 0x3bffe7ff),
    ("__real__", "__real", 0x3bffe7ff),
    ("__restrict", "restrict", 0x3bffe7ff),
    ("__restrict__", "restrict", 0x3bffe7ff),
    ("__signed", "signed", 0x3bffe7ff),
    ("__signed__", "signed", 0x3bffe7ff),
    ("__typeof", "typeof", 0x3bffe7ff),
    ("__typeof__", "typeof", 0x3bffe7ff),
    ("__typeof_unqual", "typeof_unqual", 0x3bffe7ff),
    ("__typeof_unqual__", "typeof_unqual", 0x3bffe7ff),
    ("__volatile", "volatile", 0x3bffe7ff),
    ("__volatile__", "volatile", 0x3bffe7ff),
    ("_Nonnull", "_Nonnull", 0x3bffe7ff),
    ("_Nullable", "_Nullable", 0x3bffe7ff),
    ("_Nullable_result", "_Nullable_result", 0x3bffe7ff),
    ("_Null_unspecified", "_Null_unspecified", 0x3bffe7ff),
    ("__funcref", "__funcref", 0x3bffe7ff),
    ("__ptr64", "__ptr64", 0x10),
    ("__ptr32", "__ptr32", 0x2000010),
    ("__sptr", "__sptr", 0x10),
    ("__uptr", "__uptr", 0x10),
    ("__w64", "__w64", 0x10),
    ("__uuidof", "__uuidof", 0x110),
    ("__try", "__try", 0x110),
    ("__finally", "__finally", 0x110),
    ("__leave", "__leave", 0x110),
    ("__int64", "__int64", 0x10),
    ("__if_exists", "__if_exists", 0x10),
    ("__if_not_exists", "__if_not_exists", 0x10),
    ("__single_inheritance", "__single_inheritance", 0x10),
    ("__multiple_inheritance", "__multiple_inheritance", 0x10),
    ("__virtual_inheritance", "__virtual_inheritance", 0x10),
    ("__interface", "__interface", 0x10),
    ("__int8", "char", 0x10),
    ("__int16", "short", 0x10),
    ("__int32", "int", 0x10),
    ("__wchar_t", "wchar_t", 0x10),
    ("_asm", "asm", 0x10),
    ("_cdecl", "__cdecl", 0x110),
    ("_declspec", "__declspec", 0x10),
    ("_fastcall", "__fastcall", 0x110),
    ("_finally", "__finally", 0x400000),
    ("_forceinline", "__forceinline", 0x400000),
    ("_inline", "inline", 0x10),
    ("_int8", "char", 0x10),
    ("_int16", "short", 0x10),
    ("_int32", "int", 0x10),
    ("_int64", "__int64", 0x10),
    ("_leave", "__leave", 0x400000),
    ("_multiple_inheritance", "__multiple_inheritance", 0x400000),
    ("_ptr32", "__ptr32", 0x400000),
    ("_ptr64", "__ptr64", 0x400000),
    ("_restrict", "restrict", 0x400000),
    ("_stdcall", "__stdcall", 0x110),
    ("_thiscall", "__thiscall", 0x10),
    ("_try", "__try", 0x400000),
    ("_vectorcall", "__vectorcall", 0x10),
    ("_unaligned", "__unaligned", 0x400000),
    ("_uptr", "__uptr", 0x400000),
    ("_uuidof", "__uuidof", 0x110),
    ("_virtual_inheritance", "__virtual_inheritance", 0x400000),
    ("_w64", "__w64", 0x400000),
    ("_pascal", "__pascal", 0x100),
    (
        "__builtin_convertvector",
        "__builtin_convertvector",
        0x3bffe7ff,
    ),
    ("__char16_t", "char16_t", 0x2),
    ("__char32_t", "char32_t", 0x2),
    ("__builtin_bit_cast", "__builtin_bit_cast", 0x3bffe7ff),
    ("__builtin_available", "__builtin_available", 0x3bffe7ff),
    (
        "__builtin_sycl_unique_stable_name",
        "__builtin_sycl_unique_stable_name",
        0x800000,
    ),
    ("__arm_agnostic", "__arm_agnostic", 0x3bffe7ff),
    ("__arm_in", "__arm_in", 0x3bffe7ff),
    ("__arm_inout", "__arm_inout", 0x3bffe7ff),
    (
        "__arm_locally_streaming",
        "__arm_locally_streaming",
        0x3bffe7ff,
    ),
    ("__arm_new", "__arm_new", 0x3bffe7ff),
    ("__arm_out", "__arm_out", 0x3bffe7ff),
    ("__arm_preserves", "__arm_preserves", 0x3bffe7ff),
    ("__arm_streaming", "__arm_streaming", 0x3bffe7ff),
    (
        "__arm_streaming_compatible",
        "__arm_streaming_compatible",
        0x3bffe7ff,
    ),
    ("sizeof", "sizeof", 0x3bffe7ff),
    ("__datasizeof", "__datasizeof", 0x2),
    ("_Countof", "_Countof", 0x80),
    ("alignof", "alignof", 0x404),
    ("__alignof", "__alignof", 0x3bffe7ff),
    (
        "__builtin_types_compatible_p",
        "__builtin_types_compatible_p",
        0x80,
    ),
    ("__builtin_type_order", "__builtin_type_order", 0x100000),
    ("__is_interface_class", "__is_interface_class", 0x10),
    ("__is_sealed", "__is_sealed", 0x10),
    ("__is_destructible", "__is_destructible", 0x3bffe7ff),
    (
        "__is_trivially_destructible",
        "__is_trivially_destructible",
        0x2,
    ),
    (
        "__is_nothrow_destructible",
        "__is_nothrow_destructible",
        0x3bffe7ff,
    ),
    ("__is_nothrow_assignable", "__is_nothrow_assignable", 0x2),
    ("__is_constructible", "__is_constructible", 0x2),
    (
        "__is_nothrow_constructible",
        "__is_nothrow_constructible",
        0x2,
    ),
    ("__is_assignable", "__is_assignable", 0x2),
    (
        "__has_nothrow_move_assign",
        "__has_nothrow_move_assign",
        0x2,
    ),
    (
        "__has_trivial_move_assign",
        "__has_trivial_move_assign",
        0x2,
    ),
    (
        "__has_trivial_move_constructor",
        "__has_trivial_move_constructor",
        0x2,
    ),
    (
        "__builtin_is_implicit_lifetime",
        "__builtin_is_implicit_lifetime",
        0x2,
    ),
    (
        "__builtin_is_virtual_base_of",
        "__builtin_is_virtual_base_of",
        0x2,
    ),
    ("__has_nothrow_assign", "__has_nothrow_assign", 0x2),
    ("__has_nothrow_copy", "__has_nothrow_copy", 0x2),
    (
        "__has_nothrow_constructor",
        "__has_nothrow_constructor",
        0x2,
    ),
    ("__has_trivial_assign", "__has_trivial_assign", 0x2),
    ("__has_trivial_copy", "__has_trivial_copy", 0x2),
    (
        "__has_trivial_constructor",
        "__has_trivial_constructor",
        0x2,
    ),
    ("__has_trivial_destructor", "__has_trivial_destructor", 0x2),
    ("__has_virtual_destructor", "__has_virtual_destructor", 0x2),
    ("__is_abstract", "__is_abstract", 0x2),
    ("__is_aggregate", "__is_aggregate", 0x2),
    ("__is_base_of", "__is_base_of", 0x2),
    ("__is_class", "__is_class", 0x2),
    ("__is_convertible_to", "__is_convertible_to", 0x2),
    ("__is_empty", "__is_empty", 0x2),
    ("__is_enum", "__is_enum", 0x2),
    ("__is_final", "__is_final", 0x2),
    ("__is_literal", "__is_literal", 0x2),
    ("__is_pod", "__is_pod", 0x2),
    ("__is_polymorphic", "__is_polymorphic", 0x2),
    ("__is_standard_layout", "__is_standard_layout", 0x2),
    ("__is_trivial", "__is_trivial", 0x2),
    (
        "__is_trivially_assignable",
        "__is_trivially_assignable",
        0x2,
    ),
    (
        "__is_trivially_constructible",
        "__is_trivially_constructible",
        0x2,
    ),
    ("__is_trivially_copyable", "__is_trivially_copyable", 0x2),
    ("__is_union", "__is_union", 0x2),
    (
        "__has_unique_object_representations",
        "__has_unique_object_representations",
        0x2,
    ),
    ("__is_layout_compatible", "__is_layout_compatible", 0x2),
    (
        "__is_pointer_interconvertible_base_of",
        "__is_pointer_interconvertible_base_of",
        0x2,
    ),
    ("__add_lvalue_reference", "__add_lvalue_reference", 0x2),
    ("__add_pointer", "__add_pointer", 0x2),
    ("__add_rvalue_reference", "__add_rvalue_reference", 0x2),
    ("__decay", "__decay", 0x2),
    ("__make_signed", "__make_signed", 0x2),
    ("__make_unsigned", "__make_unsigned", 0x2),
    ("__remove_all_extents", "__remove_all_extents", 0x2),
    ("__remove_const", "__remove_const", 0x2),
    ("__remove_cv", "__remove_cv", 0x2),
    ("__remove_cvref", "__remove_cvref", 0x2),
    ("__remove_extent", "__remove_extent", 0x2),
    ("__remove_pointer", "__remove_pointer", 0x2),
    ("__remove_reference_t", "__remove_reference_t", 0x2),
    ("__remove_restrict", "__remove_restrict", 0x2),
    ("__remove_volatile", "__remove_volatile", 0x2),
    ("__underlying_type", "__underlying_type", 0x2),
    (
        "__is_trivially_equality_comparable",
        "__is_trivially_equality_comparable",
        0x2,
    ),
    ("__is_bounded_array", "__is_bounded_array", 0x2),
    ("__is_unbounded_array", "__is_unbounded_array", 0x2),
    ("__is_scoped_enum", "__is_scoped_enum", 0x2),
    ("__can_pass_in_regs", "__can_pass_in_regs", 0x2),
    (
        "__reference_binds_to_temporary",
        "__reference_binds_to_temporary",
        0x2,
    ),
    (
        "__reference_constructs_from_temporary",
        "__reference_constructs_from_temporary",
        0x2,
    ),
    (
        "__reference_converts_from_temporary",
        "__reference_converts_from_temporary",
        0x2,
    ),
    (
        "__builtin_lt_synthesizes_from_spaceship",
        "__builtin_lt_synthesizes_from_spaceship",
        0x2,
    ),
    (
        "__builtin_le_synthesizes_from_spaceship",
        "__builtin_le_synthesizes_from_spaceship",
        0x2,
    ),
    (
        "__builtin_gt_synthesizes_from_spaceship",
        "__builtin_gt_synthesizes_from_spaceship",
        0x2,
    ),
    (
        "__builtin_ge_synthesizes_from_spaceship",
        "__builtin_ge_synthesizes_from_spaceship",
        0x2,
    ),
    (
        "__builtin_is_cpp_trivially_relocatable",
        "__builtin_is_cpp_trivially_relocatable",
        0x2,
    ),
    (
        "__is_trivially_relocatable",
        "__is_trivially_relocatable",
        0x2,
    ),
    (
        "__is_bitwise_cloneable",
        "__is_bitwise_cloneable",
        0x3bffe7ff,
    ),
    (
        "__builtin_structured_binding_size",
        "__builtin_structured_binding_size",
        0x2,
    ),
    ("__is_lvalue_expr", "__is_lvalue_expr", 0x2),
    ("__is_rvalue_expr", "__is_rvalue_expr", 0x2),
    ("__is_arithmetic", "__is_arithmetic", 0x2),
    ("__is_floating_point", "__is_floating_point", 0x2),
    ("__is_integral", "__is_integral", 0x2),
    ("__is_complete_type", "__is_complete_type", 0x2),
    ("__is_void", "__is_void", 0x2),
    ("__is_array", "__is_array", 0x2),
    ("__is_function", "__is_function", 0x2),
    ("__is_reference", "__is_reference", 0x2),
    ("__is_lvalue_reference", "__is_lvalue_reference", 0x2),
    ("__is_rvalue_reference", "__is_rvalue_reference", 0x2),
    ("__is_fundamental", "__is_fundamental", 0x2),
    ("__is_object", "__is_object", 0x2),
    ("__is_scalar", "__is_scalar", 0x2),
    ("__is_compound", "__is_compound", 0x2),
    ("__is_pointer", "__is_pointer", 0x2),
    (
        "__is_member_object_pointer",
        "__is_member_object_pointer",
        0x2,
    ),
    (
        "__is_member_function_pointer",
        "__is_member_function_pointer",
        0x2,
    ),
    ("__is_member_pointer", "__is_member_pointer", 0x2),
    ("__is_const", "__is_const", 0x2),
    ("__is_volatile", "__is_volatile", 0x2),
    ("__is_signed", "__is_signed", 0x2),
    ("__is_unsigned", "__is_unsigned", 0x2),
    ("__is_same", "__is_same", 0x2),
    ("__is_convertible", "__is_convertible", 0x2),
    ("__is_nothrow_convertible", "__is_nothrow_convertible", 0x2),
    ("__array_rank", "__array_rank", 0x2),
    ("__array_extent", "__array_extent", 0x2),
    (
        "__builtin_ptrauth_type_discriminator",
        "__builtin_ptrauth_type_discriminator",
        0x3bffe7ff,
    ),
    ("vec_step", "vec_step", 0x220240),
    (
        "__builtin_hlsl_is_scalarized_layout_compatible",
        "__builtin_hlsl_is_scalarized_layout_compatible",
        0x8000000,
    ),
    (
        "__builtin_hlsl_is_intangible",
        "__builtin_hlsl_is_intangible",
        0x8000000,
    ),
    (
        "__builtin_hlsl_is_typed_resource_element_compatible",
        "__builtin_hlsl_is_typed_resource_element_compatible",
        0x8000000,
    ),
    (
        "__builtin_hlsl_is_constant_buffer_element_compatible",
        "__builtin_hlsl_is_constant_buffer_element_compatible",
        0x8000000,
    ),
    (
        "__builtin_omp_required_simd_align",
        "__builtin_omp_required_simd_align",
        0x3bffe7ff,
    ),
    (
        "__builtin_vectorelements",
        "__builtin_vectorelements",
        0x3bffe7ff,
    ),
    ("__alignof__", "__alignof", 0x3bffe7ff),
    ("__builtin_alignof", "__alignof", 0x10),
    ("_alignof", "__alignof", 0x10),
    ("__is_literal_type", "__is_literal", 0x2),
    ("__is_same_as", "__is_same", 0x2),
];

const KEYWORD_C99: u32 = 0x1;
const KEYWORD_GNU: u32 = 0x8;
const KEYWORD_MICROSOFT: u32 = 0x10;
const KEYWORD_BOOL_SUPPORT: u32 = 0x20;
const KEYWORD_NOT_CPLUSPLUS: u32 = 0x80;
const KEYWORD_C23: u32 = 0x400;
const KEYWORD_FIXED_POINT: u32 = 0x10000000;
const KEYWORD_DEFER: u32 = 0x20000000;
const KEYWORD_EVERYWHERE: u32 = 0x3bffe7ff;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum KeywordStatus {
    Unknown,
    Disabled,
    Future,
    Extension,
    Enabled,
}

fn keyword_flag_status(language: &LanguageOptions, flag: u32) -> KeywordStatus {
    let c99 = language.at_least(CStandard::C99);
    let c23 = language.at_least(CStandard::C23);
    let introduced = |present: bool| {
        if present {
            KeywordStatus::Enabled
        } else {
            KeywordStatus::Future
        }
    };
    match flag {
        KEYWORD_C99 => introduced(c99),
        KEYWORD_C23 | KEYWORD_BOOL_SUPPORT => introduced(c23),
        KEYWORD_GNU if language.gnu_keywords => KeywordStatus::Extension,
        KEYWORD_MICROSOFT if language.microsoft_extensions => KeywordStatus::Extension,
        KEYWORD_NOT_CPLUSPLUS => KeywordStatus::Enabled,
        KEYWORD_FIXED_POINT | KEYWORD_DEFER => KeywordStatus::Disabled,
        _ => KeywordStatus::Unknown,
    }
}

pub fn keyword_status(language: &LanguageOptions, flags: u32) -> KeywordStatus {
    if flags == KEYWORD_EVERYWHERE {
        return KeywordStatus::Enabled;
    }
    let mut status = KeywordStatus::Unknown;
    let mut remaining = flags;
    while remaining != 0 {
        let flag = remaining & remaining.wrapping_neg();
        remaining &= !flag;
        status = status.max(keyword_flag_status(language, flag));
    }
    if status == KeywordStatus::Unknown {
        KeywordStatus::Disabled
    } else {
        status
    }
}

pub fn keyword_kind(language: &LanguageOptions, spelling: &[u8]) -> Option<&'static str> {
    let mut kind = None;
    for (name, token, flags) in KEYWORD_TABLE.iter() {
        if name.as_bytes() != spelling {
            continue;
        }
        match keyword_status(language, *flags) {
            KeywordStatus::Enabled | KeywordStatus::Extension => kind = Some(*token),
            KeywordStatus::Future => kind = None,
            _ => {}
        }
    }
    if spelling == b"__declspec" && language.declspec_keyword {
        kind = Some("__declspec");
    }
    kind
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerType {
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
    pub fn name(self) -> &'static str {
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

    pub fn signed(self) -> bool {
        matches!(
            self,
            IntegerType::SignedChar
                | IntegerType::Short
                | IntegerType::Int
                | IntegerType::Long
                | IntegerType::LongLong
        )
    }

    pub fn format_modifier(self) -> &'static str {
        match self {
            IntegerType::SignedChar | IntegerType::UnsignedChar => "hh",
            IntegerType::Short | IntegerType::UnsignedShort => "h",
            IntegerType::Int | IntegerType::UnsignedInt => "",
            IntegerType::Long | IntegerType::UnsignedLong => "l",
            IntegerType::LongLong | IntegerType::UnsignedLongLong => "ll",
        }
    }

    pub fn unsigned_counterpart(self) -> IntegerType {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatFormat {
    Half,
    Single,
    Double,
    ExtendedPrecision,
    DoubleDouble,
    Quadruple,
}

impl FloatFormat {
    fn pick<T>(self, values: [T; 6]) -> T {
        let [half, single, double, extended, double_double, quadruple] = values;
        match self {
            FloatFormat::Half => half,
            FloatFormat::Single => single,
            FloatFormat::Double => double,
            FloatFormat::ExtendedPrecision => extended,
            FloatFormat::DoubleDouble => double_double,
            FloatFormat::Quadruple => quadruple,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetOptions {
    pub triple: String,
    pub cpu: String,
    pub abi: String,
    pub features: Vec<String>,
    pub code_model: String,
    pub wchar_size: u32,
    pub wchar_signed: bool,
}

impl Default for TargetOptions {
    fn default() -> TargetOptions {
        TargetOptions {
            triple: "x86_64-unknown-linux-gnu".to_string(),
            cpu: String::new(),
            abi: String::new(),
            features: Vec::new(),
            code_model: "default".to_string(),
            wchar_size: 0,
            wchar_signed: true,
        }
    }
}

struct X86Processor {
    name: &'static str,
    kind: usize,
    dispatch_only: bool,
    features: &'static [u16],
}

struct ArmFloatingPointUnit {
    version: u8,
    neon: u8,
    restriction: u8,
}

struct ArmArchitecture {
    name: &'static str,
    kind: &'static str,
    cpu_attribute: &'static str,
    sub_architecture: &'static str,
    default_fpu: usize,
    base_extensions: u64,
}

struct ArmExtension {
    identifier: u64,
    feature: &'static str,
    negative_feature: &'static str,
}

struct ArmProcessor {
    name: &'static str,
    architecture: usize,
    default_fpu: usize,
    extensions: u64,
}
const TRIPLE_OSES: &[(&str, &str)] = &[
    ("darwin", "darwin"),
    ("dragonfly", "dragonfly"),
    ("freebsd", "freebsd"),
    ("fuchsia", "fuchsia"),
    ("ios", "ios"),
    ("kfreebsd", "kfreebsd"),
    ("linux", "linux"),
    ("lv2", "lv2"),
    ("macosx", "macosx"),
    ("macos", "macosx"),
    ("managarm", "managarm"),
    ("netbsd", "netbsd"),
    ("openbsd", "openbsd"),
    ("solaris", "solaris"),
    ("uefi", "uefi"),
    ("windows", "windows"),
    ("win32", "windows"),
    ("zos", "zos"),
    ("haiku", "haiku"),
    ("rtems", "rtems"),
    ("aix", "aix"),
    ("cuda", "cuda"),
    ("nvcl", "nvcl"),
    ("amdhsa", "amdhsa"),
    ("ps4", "ps4"),
    ("ps5", "ps5"),
    ("elfiamcu", "elfiamcu"),
    ("tvos", "tvos"),
    ("watchos", "watchos"),
    ("bridgeos", "bridgeos"),
    ("driverkit", "driverkit"),
    ("xros", "xros"),
    ("visionos", "xros"),
    ("mesa3d", "mesa3d"),
    ("amdpal", "amdpal"),
    ("hermit", "hermit"),
    ("hurd", "hurd"),
    ("wasip1", "wasip1"),
    ("wasip2", "wasip2"),
    ("wasip3", "wasip3"),
    ("wasi", "wasi"),
    ("emscripten", "emscripten"),
    ("shadermodel", "shadermodel"),
    ("liteos", "liteos"),
    ("serenity", "serenity"),
    ("vulkan", "vulkan"),
    ("cheriotrtos", "cheriotrtos"),
    ("opencl", "opencl"),
    ("chipstar", "chipstar"),
    ("firmware", "firmware"),
    ("qurt", "qurt"),
    ("h2", "h2"),
];

const TRIPLE_ENVIRONMENTS: &[&str] = &[
    "eabihf",
    "eabi",
    "gnuabin32",
    "gnuabi64",
    "gnueabihft64",
    "gnueabihf",
    "gnueabit64",
    "gnueabi",
    "gnuf32",
    "gnuf64",
    "gnusf",
    "gnux32",
    "gnu_ilp32",
    "code16",
    "gnut64",
    "gnu",
    "android",
    "muslabin32",
    "muslabi64",
    "musleabihf",
    "musleabi",
    "muslf32",
    "muslsf",
    "muslx32",
    "muslwali",
    "musl",
    "msvc",
    "itanium",
    "cygnus",
    "coreclr",
    "simulator",
    "macabi",
    "pixel",
    "vertex",
    "geometry",
    "hull",
    "domain",
    "compute",
    "library",
    "raygeneration",
    "intersection",
    "anyhit",
    "closesthit",
    "miss",
    "callable",
    "mesh",
    "amplification",
    "rootsignature",
    "ohos",
    "pauthtest",
    "llvm",
    "mlibc",
    "mtia",
];

const TRIPLE_VENDORS: &[(&str, &str)] = &[
    ("amd", "amd"),
    ("apple", "apple"),
    ("csr", "csr"),
    ("fsl", "fsl"),
    ("ibm", "ibm"),
    ("img", "img"),
    ("intel", "intel"),
    ("mesa", "mesa"),
    ("mti", "mti"),
    ("nvidia", "nvidia"),
    ("oe", "oe"),
    ("pc", "pc"),
    ("scei", "scei"),
    ("sie", "scei"),
    ("suse", "suse"),
    ("meta", "meta"),
];

fn arm_canonical_arch_name(arch: &str) -> Option<&str> {
    let mut offset: Option<usize> = None;
    let mut name = arch;
    if arch.starts_with("arm64_32") {
        offset = Some(8);
    } else if arch.starts_with("arm64e") {
        offset = Some(6);
    } else if arch.starts_with("arm64") {
        offset = Some(5);
    } else if arch.starts_with("aarch64_32") {
        offset = Some(10);
    } else if arch.starts_with("arm") {
        offset = Some(3);
    } else if arch.starts_with("thumb") {
        offset = Some(5);
    } else if arch.starts_with("aarch64") {
        if arch.contains("eb") {
            return None;
        }
        offset = Some(if arch.get(7..10) == Some("_be") {
            10
        } else {
            7
        });
    }
    match offset {
        Some(at) if name.get(at..at + 2) == Some("eb") => offset = Some(at + 2),
        _ => {
            if let Some(stripped) = name.strip_suffix("eb") {
                name = stripped;
            }
        }
    }
    if let Some(at) = offset {
        name = name.get(at..).unwrap_or("");
    }
    if name.is_empty() {
        return Some(arch);
    }
    if offset.is_some() {
        let bytes = name.as_bytes();
        if bytes.len() >= 2 && (bytes[0] != b'v' || !bytes[1].is_ascii_digit()) {
            return None;
        }
        if name.contains("eb") {
            return None;
        }
    }
    Some(name)
}

fn arm_arch_synonym(arch: &str) -> &str {
    match arch {
        "v5" => "v5t",
        "v5e" => "v5te",
        "v6j" => "v6",
        "v6hl" => "v6k",
        "v6m" | "v6sm" | "v6s-m" => "v6-m",
        "v6z" | "v6zk" => "v6kz",
        "v7" | "v7a" | "v7hl" | "v7l" => "v7-a",
        "v7r" => "v7-r",
        "v7m" => "v7-m",
        "v7em" => "v7e-m",
        "v8" | "v8a" | "v8l" | "aarch64" | "arm64" => "v8-a",
        "v8.1a" => "v8.1-a",
        "v8.2a" => "v8.2-a",
        "v8.3a" => "v8.3-a",
        "v8.4a" => "v8.4-a",
        "v8.5a" => "v8.5-a",
        "v8.6a" => "v8.6-a",
        "v8.7a" => "v8.7-a",
        "v8.8a" => "v8.8-a",
        "v8.9a" => "v8.9-a",
        "v8r" => "v8-r",
        "v9" | "v9a" => "v9-a",
        "v9.1a" => "v9.1-a",
        "v9.2a" => "v9.2-a",
        "v9.3a" => "v9.3-a",
        "v9.4a" => "v9.4-a",
        "v9.5a" => "v9.5-a",
        "v9.6a" => "v9.6-a",
        "v9.7a" => "v9.7-a",
        "v8m.base" => "v8-m.base",
        "v8m.main" => "v8-m.main",
        "v8.1m.main" => "v8.1-m.main",
        other => other,
    }
}

fn arm_parse_architecture(name: &str) -> usize {
    let canonical = arm_canonical_arch_name(name).unwrap_or("");
    let synonym = arm_arch_synonym(canonical);
    ARM_ARCHITECTURES
        .iter()
        .position(|entry| entry.name.ends_with(synonym))
        .unwrap_or(0)
}

fn arm_kind_version(kind: &str) -> u32 {
    match kind {
        "ARMV4" | "ARMV4T" => 4,
        "ARMV5T" | "ARMV5TE" | "IWMMXT" | "IWMMXT2" | "XSCALE" | "ARMV5TEJ" => 5,
        "ARMV6" | "ARMV6K" | "ARMV6T2" | "ARMV6KZ" | "ARMV6M" => 6,
        "ARMV7A" | "ARMV7VE" | "ARMV7R" | "ARMV7M" | "ARMV7S" | "ARMV7EM" | "ARMV7K" => 7,
        other if other.starts_with("ARMV8") => 8,
        other if other.starts_with("ARMV9") => 9,
        _ => 0,
    }
}

fn arm_kind_profile(kind: &str) -> char {
    match kind {
        "ARMV6M" | "ARMV7M" | "ARMV7EM" | "ARMV8MMainline" | "ARMV8MBaseline"
        | "ARMV8_1MMainline" => 'M',
        "ARMV7R" | "ARMV8R" => 'R',
        "ARMV7A" | "ARMV7VE" | "ARMV7K" | "ARMV8A" => 'A',
        other if other.starts_with("ARMV8_") && other.ends_with('A') => 'A',
        other if other.starts_with("ARMV9") => 'A',
        _ => ' ',
    }
}

fn arm_arch_profile(arch: &str) -> char {
    arm_kind_profile(ARM_ARCHITECTURES[arm_parse_architecture(arch)].kind)
}

fn arm_arch_version(arch: &str) -> u32 {
    arm_kind_version(ARM_ARCHITECTURES[arm_parse_architecture(arch)].kind)
}

fn parse_arm_arch(arch_name: &str) -> &'static str {
    let isa = if arch_name.starts_with("aarch64") || arch_name.starts_with("arm64") {
        "aarch64"
    } else if arch_name.starts_with("thumb") {
        "thumb"
    } else if arch_name.starts_with("arm") {
        "arm"
    } else {
        ""
    };
    let big = if arch_name.starts_with("armeb")
        || arch_name.starts_with("thumbeb")
        || arch_name.starts_with("aarch64_be")
    {
        Some(true)
    } else if arch_name.starts_with("arm") || arch_name.starts_with("thumb") {
        Some(arch_name.ends_with("eb"))
    } else if arch_name.starts_with("aarch64") {
        Some(false)
    } else {
        None
    };
    let arch = match (big, isa) {
        (Some(false), "arm") => "arm",
        (Some(false), "thumb") => "thumb",
        (Some(false), "aarch64") => "aarch64",
        (Some(true), "arm") => "armeb",
        (Some(true), "thumb") => "thumbeb",
        (Some(true), "aarch64") => "aarch64_be",
        _ => "unknown",
    };
    let Some(canonical) = arm_canonical_arch_name(arch_name) else {
        return "unknown";
    };
    if canonical.is_empty() {
        return "unknown";
    }
    if isa == "thumb" && (canonical.starts_with("v2") || canonical.starts_with("v3")) {
        return "unknown";
    }
    if arm_arch_profile(canonical) == 'M' && arm_arch_version(canonical) == 6 {
        return if big == Some(true) {
            "thumbeb"
        } else {
            "thumb"
        };
    }
    arch
}

fn parse_triple_arch(name: &str) -> &'static str {
    let arch = match name {
        "i386" | "i486" | "i586" | "i686" | "i786" | "i886" | "i986" => "x86",
        "amd64" | "x86_64" | "x86_64h" | "x86_64_lfi" => "x86_64",
        "powerpc" | "powerpcspe" | "ppc" | "ppc32" => "powerpc",
        "powerpcle" | "ppcle" | "ppc32le" => "powerpcle",
        "powerpc64" | "ppu" | "ppc64" => "powerpc64",
        "powerpc64le" | "ppc64le" => "powerpc64le",
        "xscale" => "arm",
        "xscaleeb" => "armeb",
        "aarch64" | "aarch64_lfi" | "arm64" | "arm64e" | "arm64e.x1" | "arm64ec" => "aarch64",
        "aarch64_be" => "aarch64_be",
        "aarch64_32" | "arm64_32" => "aarch64_32",
        "arc" => "arc",
        "arm" => "arm",
        "armeb" => "armeb",
        "thumb" => "thumb",
        "thumbeb" => "thumbeb",
        "avr" => "avr",
        "m68k" => "m68k",
        "msp430" => "msp430",
        "mips" | "mipseb" | "mipsallegrex" | "mipsisa32r6" | "mipsr6" => "mips",
        "mipsel" | "mipsallegrexel" | "mipsisa32r6el" | "mipsr6el" => "mipsel",
        "mips64" | "mips64eb" | "mipsn32" | "mipsisa64r6" | "mips64r6" | "mipsn32r6" => "mips64",
        "mips64el" | "mipsn32el" | "mipsisa64r6el" | "mips64r6el" | "mipsn32r6el" => "mips64el",
        _ if name.starts_with("amdgpu") => "amdgcn",
        "amdgcn" => "amdgcn",
        "r600" => "r600",
        "riscv32" => "riscv32",
        "riscv64" => "riscv64",
        "riscv32be" => "riscv32be",
        "riscv64be" => "riscv64be",
        "hexagon" => "hexagon",
        "s390x" | "systemz" => "s390x",
        "sparc" => "sparc",
        "sparcel" => "sparcel",
        "sparcv9" | "sparc64" => "sparcv9",
        "tce" => "tce",
        "tcele" => "tcele",
        "tcele64" => "tcele64",
        "xcore" => "xcore",
        "nvptx" => "nvptx",
        "nvptx64" => "nvptx64",
        "amdil" => "amdil",
        "amdil64" => "amdil64",
        "hsail" => "hsail",
        "hsail64" => "hsail64",
        "spir" => "spir",
        "spir64" => "spir64",
        _ if name == "spirv" || name.starts_with("spirv1.") => "spirv",
        _ if name.starts_with("spirv32") => "spirv32",
        _ if name.starts_with("spirv64") => "spirv64",
        _ if name.starts_with("kalimba") => "kalimba",
        "lanai" => "lanai",
        "renderscript32" => "renderscript32",
        "renderscript64" => "renderscript64",
        "shave" => "shave",
        "ve" => "ve",
        "wasm32" => "wasm32",
        "wasm64" => "wasm64",
        "csky" => "csky",
        "loongarch32" => "loongarch32",
        "loongarch64" => "loongarch64",
        _ if name.starts_with("dxil") => "dxil",
        "xtensa" => "xtensa",
        _ => "unknown",
    };
    if arch == "unknown" {
        if name.starts_with("arm") || name.starts_with("thumb") || name.starts_with("aarch64") {
            return parse_arm_arch(name);
        }
        if name.starts_with("bpf") {
            return match name {
                "bpf" | "bpf_le" | "bpfel" => "bpfel",
                "bpf_be" | "bpfeb" => "bpfeb",
                _ => "unknown",
            };
        }
    }
    arch
}

fn parse_triple_vendor(name: &str) -> &'static str {
    TRIPLE_VENDORS
        .iter()
        .find(|(spelled, _)| *spelled == name)
        .map(|(_, vendor)| *vendor)
        .unwrap_or("unknown")
}

fn parse_triple_os(name: &str) -> &'static str {
    TRIPLE_OSES
        .iter()
        .find(|(spelled, _)| name.starts_with(spelled))
        .map(|(_, os)| *os)
        .unwrap_or("unknown")
}

fn parse_triple_environment(name: &str) -> &'static str {
    TRIPLE_ENVIRONMENTS
        .iter()
        .find(|spelled| name.starts_with(**spelled))
        .copied()
        .unwrap_or("unknown")
}

fn parse_triple_format(name: &str) -> &'static str {
    if name.ends_with("xcoff") {
        "xcoff"
    } else if name.ends_with("coff") {
        "coff"
    } else if name.ends_with("elf") {
        "elf"
    } else if name.ends_with("goff") {
        "goff"
    } else if name.ends_with("macho") {
        "macho"
    } else if name.ends_with("wasm") {
        "wasm"
    } else if name.ends_with("spirv") {
        "spirv"
    } else {
        "unknown"
    }
}

fn arm_sub_arch(name: &str) -> &'static str {
    let Some(canonical) = arm_canonical_arch_name(name) else {
        return "";
    };
    if canonical.is_empty() {
        return "";
    }
    match ARM_ARCHITECTURES[arm_parse_architecture(canonical)].name {
        "armv4t" => "v4t",
        "armv5t" => "v5",
        "armv5te" | "iwmmxt" | "iwmmxt2" | "xscale" | "armv5tej" => "v5te",
        "armv6" => "v6",
        "armv6k" | "armv6kz" => "v6k",
        "armv6t2" => "v6t2",
        "armv6-m" => "v6m",
        "armv7-a" | "armv7-r" => "v7",
        "armv7ve" => "v7ve",
        "armv7k" => "v7k",
        "armv7-m" => "v7m",
        "armv7s" => "v7s",
        "armv7e-m" => "v7em",
        "armv8-a" => "v8",
        "armv8.1-a" => "v8_1a",
        "armv8.2-a" => "v8_2a",
        "armv8.3-a" => "v8_3a",
        "armv8.4-a" => "v8_4a",
        "armv8.5-a" => "v8_5a",
        "armv8.6-a" => "v8_6a",
        "armv8.7-a" => "v8_7a",
        "armv8.8-a" => "v8_8a",
        "armv8.9-a" => "v8_9a",
        "armv9-a" => "v9",
        "armv9.1-a" => "v9_1a",
        "armv9.2-a" => "v9_2a",
        "armv9.3-a" => "v9_3a",
        "armv9.4-a" => "v9_4a",
        "armv9.5-a" => "v9_5a",
        "armv9.6-a" => "v9_6a",
        "armv9.7-a" => "v9_7a",
        "armv8-r" => "v8r",
        "armv8-m.base" => "v8m_baseline",
        "armv8-m.main" => "v8m_mainline",
        "armv8.1-m.main" => "v8_1m_mainline",
        _ => "",
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Triple {
    pub text: String,
    pub arch: &'static str,
    pub sub_arch: &'static str,
    pub vendor: &'static str,
    pub os: &'static str,
    pub environment: &'static str,
    pub object_format: &'static str,
    components: Vec<String>,
}

impl Triple {
    pub fn parse(text: &str) -> Triple {
        let components: Vec<String> = text.splitn(4, '-').map(str::to_string).collect();
        let mut arch = "unknown";
        let mut sub_arch = "";
        let mut vendor = "unknown";
        let mut os = "unknown";
        let mut environment = "unknown";
        let mut object_format = "unknown";
        if let Some(first) = components.first() {
            arch = parse_triple_arch(first);
            sub_arch = match first.as_str() {
                "arm64e" => "arm64e",
                "arm64e.x1" => "arm64e_x1",
                "arm64ec" => "arm64ec",
                "aarch64_lfi" | "x86_64_lfi" => "lfi",
                name if name.starts_with("arm")
                    || name.starts_with("thumb")
                    || name.starts_with("aarch64") =>
                {
                    arm_sub_arch(name)
                }
                _ => "",
            };
            if components.len() > 1 {
                vendor = parse_triple_vendor(&components[1]);
                if components.len() > 2 {
                    os = parse_triple_os(&components[2]);
                    if components.len() > 3 {
                        environment = parse_triple_environment(&components[3]);
                        object_format = parse_triple_format(&components[3]);
                    }
                }
            } else {
                environment = if first.starts_with("mipsn32") {
                    "gnuabin32"
                } else if first.starts_with("mips64") || first.starts_with("mipsisa64") {
                    "gnuabi64"
                } else if first.starts_with("mipsisa32")
                    || matches!(first.as_str(), "mips" | "mipsel" | "mipsr6" | "mipsr6el")
                {
                    "gnu"
                } else {
                    "unknown"
                };
            }
        }
        let mut triple = Triple {
            text: text.to_string(),
            arch,
            sub_arch,
            vendor,
            os,
            environment,
            object_format,
            components,
        };
        if triple.object_format == "unknown" {
            triple.object_format = triple.default_object_format();
        }
        triple
    }

    pub fn normalize(text: &str) -> String {
        let mut components: Vec<String> = text.split('-').map(str::to_string).collect();
        let component = |components: &Vec<String>, index: usize| -> String {
            components.get(index).cloned().unwrap_or_default()
        };
        let mut arch = if !components.is_empty() {
            parse_triple_arch(&components[0])
        } else {
            "unknown"
        };
        let mut vendor = if components.len() > 1 {
            parse_triple_vendor(&components[1])
        } else {
            "unknown"
        };
        let mut os = "unknown";
        let mut is_cygwin = false;
        let mut is_mingw = false;
        if components.len() > 2 {
            os = parse_triple_os(&components[2]);
            is_cygwin = components[2].starts_with("cygwin") || components[2].starts_with("msys");
            is_mingw = components[2].starts_with("mingw");
        }
        let mut environment = if components.len() > 3 {
            parse_triple_environment(&components[3])
        } else {
            "unknown"
        };
        let mut object_format = if components.len() > 4 {
            parse_triple_format(&components[4])
        } else {
            "unknown"
        };
        let mut found = [
            arch != "unknown",
            vendor != "unknown",
            os != "unknown",
            environment != "unknown",
        ];
        for position in 0..4 {
            if found[position] {
                continue;
            }
            let mut index = 0usize;
            while index < components.len() {
                if index < 4 && found[index] {
                    index += 1;
                    continue;
                }
                let piece = components[index].clone();
                let valid = match position {
                    0 => {
                        arch = parse_triple_arch(&piece);
                        arch != "unknown"
                    }
                    1 => {
                        vendor = parse_triple_vendor(&piece);
                        vendor != "unknown"
                    }
                    2 => {
                        os = parse_triple_os(&piece);
                        is_cygwin = piece.starts_with("cygwin") || piece.starts_with("msys");
                        is_mingw = piece.starts_with("mingw");
                        os != "unknown" || is_cygwin || is_mingw
                    }
                    _ => {
                        environment = parse_triple_environment(&piece);
                        if environment != "unknown" {
                            true
                        } else {
                            object_format = parse_triple_format(&piece);
                            object_format != "unknown"
                        }
                    }
                };
                if !valid {
                    index += 1;
                    continue;
                }
                if position < index {
                    let mut current = std::mem::take(&mut components[index]);
                    let mut slot = position;
                    while !current.is_empty() {
                        while slot < 4 && found[slot] {
                            slot += 1;
                        }
                        if slot >= components.len() {
                            components.resize(slot + 1, String::new());
                        }
                        std::mem::swap(&mut current, &mut components[slot]);
                        slot += 1;
                    }
                } else if position > index {
                    let mut moving = index;
                    loop {
                        let mut current = String::new();
                        let mut slot = moving;
                        while slot < components.len() {
                            std::mem::swap(&mut current, &mut components[slot]);
                            if current.is_empty() {
                                break;
                            }
                            slot += 1;
                            while slot < 4 && found[slot] {
                                slot += 1;
                            }
                        }
                        if !current.is_empty() {
                            components.push(current);
                        }
                        moving += 1;
                        while moving < 4 && found[moving] {
                            moving += 1;
                        }
                        if moving >= position {
                            break;
                        }
                    }
                }
                found[position] = true;
                break;
            }
        }
        if found[0]
            && !found[1]
            && !found[2]
            && found[3]
            && component(&components, 1) == "none"
            && component(&components, 2).is_empty()
        {
            components.swap(1, 2);
        }
        for piece in components.iter_mut() {
            if piece.is_empty() {
                *piece = "unknown".to_string();
            }
        }
        if environment == "android" && component(&components, 3).starts_with("androideabi") {
            let version = component(&components, 3)["androideabi".len()..].to_string();
            components[3] = format!("android{version}");
        }
        if vendor == "suse" && environment == "gnueabi" {
            components[3] = "gnueabihf".to_string();
        }
        if os == "windows" {
            components.resize(4, "unknown".to_string());
            components[2] = "windows".to_string();
            if environment == "unknown" {
                components[3] = if object_format == "unknown" || object_format == "coff" {
                    "msvc".to_string()
                } else {
                    object_format.to_string()
                };
            }
        } else if is_mingw {
            components.resize(4, "unknown".to_string());
            components[2] = "windows".to_string();
            components[3] = "gnu".to_string();
        } else if is_cygwin {
            components.resize(4, "unknown".to_string());
            components[2] = "windows".to_string();
            components[3] = "cygnus".to_string();
        }
        if (is_mingw || is_cygwin || (os == "windows" && environment != "unknown"))
            && object_format != "unknown"
            && object_format != "coff"
        {
            components.resize(5, "unknown".to_string());
            components[4] = object_format.to_string();
        }
        components.join("-")
    }

    fn default_object_format(&self) -> &'static str {
        match self.arch {
            "unknown" | "aarch64" | "aarch64_32" | "arm" | "thumb" | "x86" | "x86_64" => {
                if self.os == "windows" || self.os == "uefi" {
                    "coff"
                } else if self.is_darwin() {
                    "macho"
                } else {
                    "elf"
                }
            }
            "mipsel" => {
                if self.os == "windows" {
                    "coff"
                } else {
                    "elf"
                }
            }
            "powerpc64" | "powerpc" => {
                if self.os == "aix" {
                    "xcoff"
                } else if self.is_darwin() {
                    "macho"
                } else {
                    "elf"
                }
            }
            "s390x" => {
                if self.os == "zos" {
                    "goff"
                } else {
                    "elf"
                }
            }
            "wasm32" | "wasm64" => "wasm",
            "spirv" | "spirv32" | "spirv64" => "spirv",
            "dxil" => "dxcontainer",
            _ => "elf",
        }
    }

    pub fn arch_name(&self) -> &str {
        self.components.first().map(String::as_str).unwrap_or("")
    }

    pub fn vendor_name(&self) -> &str {
        self.components.get(1).map(String::as_str).unwrap_or("")
    }

    pub fn os_name(&self) -> &str {
        self.components.get(2).map(String::as_str).unwrap_or("")
    }

    pub fn environment_name(&self) -> &str {
        self.components.get(3).map(String::as_str).unwrap_or("")
    }

    pub fn environment_version(&self) -> u32 {
        let name = self.environment_name();
        if name == "none" {
            return 0;
        }
        let rest = name.strip_prefix(self.environment).unwrap_or(name);
        rest.split('.')
            .next()
            .and_then(|major| major.parse().ok())
            .unwrap_or(0)
    }

    pub fn is_darwin(&self) -> bool {
        matches!(
            self.os,
            "darwin"
                | "macosx"
                | "ios"
                | "tvos"
                | "watchos"
                | "bridgeos"
                | "driverkit"
                | "xros"
                | "firmware"
        )
    }

    pub fn is_windows(&self) -> bool {
        self.os == "windows"
    }

    pub fn is_android(&self) -> bool {
        self.environment == "android"
    }

    pub fn is_gnu_environment(&self) -> bool {
        matches!(
            self.environment,
            "gnu"
                | "gnut64"
                | "gnuabin32"
                | "gnuabi64"
                | "gnueabi"
                | "gnueabit64"
                | "gnueabihf"
                | "gnueabihft64"
                | "gnuf32"
                | "gnuf64"
                | "gnusf"
                | "gnux32"
        )
    }

    pub fn is_musl(&self) -> bool {
        self.environment.starts_with("musl")
    }

    pub fn is_ohos(&self) -> bool {
        self.environment == "ohos" || self.os == "liteos"
    }

    pub fn is_x32(&self) -> bool {
        matches!(self.environment, "gnux32" | "muslx32")
    }

    pub fn is_time64_abi(&self) -> bool {
        matches!(self.environment, "gnut64" | "gnueabit64" | "gnueabihft64")
    }

    pub fn is_elf(&self) -> bool {
        self.object_format == "elf"
    }

    pub fn is_thumb(&self) -> bool {
        matches!(self.arch, "thumb" | "thumbeb")
    }

    pub fn is_arm(&self) -> bool {
        matches!(self.arch, "arm" | "armeb")
    }

    pub fn is_aarch64(&self) -> bool {
        matches!(self.arch, "aarch64" | "aarch64_be" | "aarch64_32")
    }

    pub fn is_arch64_bit(&self) -> bool {
        self.pointer_bit_width() == 64
    }

    pub fn is_arch32_bit(&self) -> bool {
        self.pointer_bit_width() == 32
    }

    pub fn pointer_bit_width(&self) -> u32 {
        match self.arch {
            "unknown" => 0,
            "avr" | "msp430" => 16,
            "aarch64_32" | "amdil" | "arc" | "arm" | "armeb" | "csky" | "dxil" | "hexagon"
            | "hsail" | "kalimba" | "lanai" | "loongarch32" | "m68k" | "mips" | "mipsel"
            | "nvptx" | "powerpc" | "powerpcle" | "r600" | "renderscript32" | "riscv32"
            | "riscv32be" | "shave" | "sparc" | "sparcel" | "spir" | "spirv32" | "tce"
            | "tcele" | "thumb" | "thumbeb" | "wasm32" | "x86" | "xcore" | "xtensa" => 32,
            _ => 64,
        }
    }

    pub fn is_little_endian(&self) -> bool {
        !matches!(
            self.arch,
            "aarch64_be"
                | "armeb"
                | "thumbeb"
                | "bpfeb"
                | "lanai"
                | "m68k"
                | "mips"
                | "mips64"
                | "powerpc"
                | "powerpc64"
                | "s390x"
                | "sparc"
                | "sparcv9"
                | "tce"
                | "riscv32be"
                | "riscv64be"
        )
    }
}
const X86_FEATURES: [&str; 125] = [
    "cmov",
    "mmx",
    "popcnt",
    "sse",
    "sse2",
    "sse3",
    "ssse3",
    "sse4.1",
    "sse4.2",
    "avx",
    "avx2",
    "sse4a",
    "fma4",
    "xop",
    "fma",
    "avx512f",
    "bmi",
    "bmi2",
    "aes",
    "pclmul",
    "avx512vl",
    "avx512bw",
    "avx512dq",
    "avx512cd",
    "avx512vbmi",
    "avx512ifma",
    "avx512vpopcntdq",
    "avx512vbmi2",
    "gfni",
    "vpclmulqdq",
    "avx512vnni",
    "avx512bitalg",
    "avx512bf16",
    "avx512vp2intersect",
    "adx",
    "cldemote",
    "clflushopt",
    "clwb",
    "clzero",
    "cx16",
    "enqcmd",
    "f16c",
    "fsgsbase",
    "sahf",
    "64bit",
    "lwp",
    "lzcnt",
    "movbe",
    "movdir64b",
    "movdiri",
    "mwaitx",
    "pconfig",
    "pku",
    "prfchw",
    "ptwrite",
    "rdpid",
    "rdrnd",
    "rdseed",
    "rtm",
    "serialize",
    "sgx",
    "sha",
    "shstk",
    "tbm",
    "tsxldtrk",
    "vaes",
    "waitpkg",
    "wbnoinvd",
    "xsave",
    "xsavec",
    "xsaveopt",
    "xsaves",
    "amx-tile",
    "amx-int8",
    "amx-bf16",
    "uintr",
    "hreset",
    "kl",
    "widekl",
    "avxvnni",
    "avx512fp16",
    "avxifma",
    "avxvnniint8",
    "avxneconvert",
    "cmpccxadd",
    "amx-fp16",
    "prefetchi",
    "raoint",
    "amx-complex",
    "avxvnniint16",
    "sm3",
    "sha512",
    "sm4",
    "apxf",
    "usermsr",
    "avx10.1",
    "avx10.2",
    "amx-avx512",
    "amx-fp8",
    "movrs",
    "amx-movrs",
    "avx512bmm",
    "avx10v2aux",
    "nf",
    "cf",
    "cx8",
    "crc32",
    "invpcid",
    "rdpru",
    "vzeroupper",
    "x87",
    "evex512",
    "fxsr",
    "ccmp",
    "push2pop2",
    "ppx",
    "ndd",
    "egpr",
    "zu",
    "jmpabs",
    "retpoline-external-thunk",
    "retpoline-indirect-branches",
    "retpoline-indirect-calls",
    "lvi-cfi",
    "lvi-load-hardening",
];

const X86_IMPLIED_FEATURES: [&[u16]; 125] = [
    &[],
    &[],
    &[],
    &[],
    &[3],
    &[4],
    &[5],
    &[6],
    &[7],
    &[8],
    &[9],
    &[5],
    &[9, 11],
    &[12],
    &[9],
    &[10, 14, 41],
    &[],
    &[],
    &[4],
    &[4],
    &[15],
    &[15],
    &[15],
    &[15],
    &[21],
    &[15],
    &[15],
    &[21],
    &[4],
    &[9, 19],
    &[15],
    &[21],
    &[21],
    &[15],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[9],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[4],
    &[],
    &[],
    &[],
    &[10, 18],
    &[],
    &[],
    &[],
    &[68],
    &[68],
    &[68],
    &[],
    &[72],
    &[72],
    &[],
    &[],
    &[4],
    &[77],
    &[10],
    &[21],
    &[10],
    &[10],
    &[10],
    &[],
    &[72],
    &[],
    &[],
    &[72],
    &[10],
    &[9],
    &[10],
    &[10],
    &[],
    &[],
    &[20, 22, 23, 24, 25, 26, 27, 30, 31, 32, 80],
    &[95],
    &[72, 96],
    &[72],
    &[],
    &[72],
    &[21],
    &[95],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
];

const X86_CPU_KINDS: [&str; 91] = [
    "None",
    "i386",
    "i486",
    "WinChipC6",
    "WinChip2",
    "C3",
    "i586",
    "Pentium",
    "PentiumMMX",
    "PentiumPro",
    "i686",
    "Pentium2",
    "Pentium3",
    "PentiumM",
    "C3_2",
    "Yonah",
    "Pentium4",
    "Prescott",
    "Nocona",
    "Core2",
    "Penryn",
    "Bonnell",
    "Silvermont",
    "Goldmont",
    "GoldmontPlus",
    "Tremont",
    "Gracemont",
    "Nehalem",
    "Westmere",
    "SandyBridge",
    "IvyBridge",
    "Haswell",
    "Broadwell",
    "SkylakeClient",
    "SkylakeServer",
    "Cascadelake",
    "Cooperlake",
    "Cannonlake",
    "IcelakeClient",
    "Rocketlake",
    "IcelakeServer",
    "Tigerlake",
    "SapphireRapids",
    "Alderlake",
    "Raptorlake",
    "Meteorlake",
    "Arrowlake",
    "ArrowlakeS",
    "Lunarlake",
    "Pantherlake",
    "Wildcatlake",
    "Novalake",
    "Sierraforest",
    "Grandridge",
    "Graniterapids",
    "GraniterapidsD",
    "Emeraldrapids",
    "Clearwaterforest",
    "Diamondrapids",
    "KNL",
    "KNM",
    "Lakemont",
    "K6",
    "K6_2",
    "K6_3",
    "Athlon",
    "AthlonXP",
    "K8",
    "K8SSE3",
    "AMDFAM10",
    "BTVER1",
    "BTVER2",
    "BDVER1",
    "BDVER2",
    "BDVER3",
    "BDVER4",
    "ZNVER1",
    "ZNVER2",
    "ZNVER3",
    "ZNVER4",
    "ZNVER5",
    "ZNVER6",
    "C86_4G_M4",
    "C86_4G_M6",
    "C86_4G_M7",
    "C86_4G_M8",
    "x86_64",
    "x86_64_v2",
    "x86_64_v3",
    "x86_64_v4",
    "Geode",
];

const X86_PROCESSORS: [X86Processor; 137] = [
    X86Processor {
        name: "",
        kind: 0,
        dispatch_only: false,
        features: &[105, 110],
    },
    X86Processor {
        name: "generic",
        kind: 0,
        dispatch_only: true,
        features: &[44, 105, 110],
    },
    X86Processor {
        name: "i386",
        kind: 1,
        dispatch_only: false,
        features: &[110],
    },
    X86Processor {
        name: "i486",
        kind: 2,
        dispatch_only: false,
        features: &[110],
    },
    X86Processor {
        name: "winchip-c6",
        kind: 3,
        dispatch_only: false,
        features: &[1, 105, 110],
    },
    X86Processor {
        name: "winchip2",
        kind: 4,
        dispatch_only: false,
        features: &[1, 53, 105, 110],
    },
    X86Processor {
        name: "c3",
        kind: 5,
        dispatch_only: false,
        features: &[1, 53, 105, 110],
    },
    X86Processor {
        name: "i586",
        kind: 6,
        dispatch_only: false,
        features: &[105, 110],
    },
    X86Processor {
        name: "pentium",
        kind: 7,
        dispatch_only: false,
        features: &[105, 110],
    },
    X86Processor {
        name: "pentium-mmx",
        kind: 8,
        dispatch_only: false,
        features: &[1, 105, 110],
    },
    X86Processor {
        name: "pentium_mmx",
        kind: 8,
        dispatch_only: true,
        features: &[1, 105, 110],
    },
    X86Processor {
        name: "pentiumpro",
        kind: 9,
        dispatch_only: false,
        features: &[0, 105, 110],
    },
    X86Processor {
        name: "pentium_pro",
        kind: 9,
        dispatch_only: true,
        features: &[0, 105, 110],
    },
    X86Processor {
        name: "i686",
        kind: 10,
        dispatch_only: false,
        features: &[0, 105, 110],
    },
    X86Processor {
        name: "pentium2",
        kind: 11,
        dispatch_only: false,
        features: &[0, 1, 105, 110, 112],
    },
    X86Processor {
        name: "pentium_ii",
        kind: 11,
        dispatch_only: true,
        features: &[0, 1, 105, 110, 112],
    },
    X86Processor {
        name: "pentium3",
        kind: 12,
        dispatch_only: false,
        features: &[0, 1, 3, 105, 110, 112],
    },
    X86Processor {
        name: "pentium3m",
        kind: 12,
        dispatch_only: false,
        features: &[0, 1, 3, 105, 110, 112],
    },
    X86Processor {
        name: "pentium_iii",
        kind: 12,
        dispatch_only: true,
        features: &[0, 1, 3, 105, 110, 112],
    },
    X86Processor {
        name: "pentium_iii_no_xmm_regs",
        kind: 12,
        dispatch_only: true,
        features: &[0, 1, 3, 105, 110, 112],
    },
    X86Processor {
        name: "pentium-m",
        kind: 13,
        dispatch_only: false,
        features: &[0, 1, 3, 4, 105, 110, 112],
    },
    X86Processor {
        name: "pentium_m",
        kind: 13,
        dispatch_only: true,
        features: &[0, 1, 3, 4, 105, 110, 112],
    },
    X86Processor {
        name: "c3-2",
        kind: 14,
        dispatch_only: false,
        features: &[0, 1, 3, 105, 110, 112],
    },
    X86Processor {
        name: "yonah",
        kind: 15,
        dispatch_only: false,
        features: &[0, 1, 3, 4, 5, 105, 110, 112],
    },
    X86Processor {
        name: "pentium4",
        kind: 16,
        dispatch_only: false,
        features: &[0, 1, 3, 4, 105, 110, 112],
    },
    X86Processor {
        name: "pentium4m",
        kind: 16,
        dispatch_only: false,
        features: &[0, 1, 3, 4, 105, 110, 112],
    },
    X86Processor {
        name: "pentium_4",
        kind: 16,
        dispatch_only: true,
        features: &[0, 1, 3, 4, 105, 110, 112],
    },
    X86Processor {
        name: "pentium_4_sse3",
        kind: 17,
        dispatch_only: true,
        features: &[0, 1, 3, 4, 5, 105, 110, 112],
    },
    X86Processor {
        name: "prescott",
        kind: 17,
        dispatch_only: false,
        features: &[0, 1, 3, 4, 5, 105, 110, 112],
    },
    X86Processor {
        name: "nocona",
        kind: 18,
        dispatch_only: false,
        features: &[0, 1, 3, 4, 5, 39, 44, 105, 110, 112],
    },
    X86Processor {
        name: "core2",
        kind: 19,
        dispatch_only: false,
        features: &[0, 1, 3, 4, 5, 6, 39, 43, 44, 105, 110, 112],
    },
    X86Processor {
        name: "core_2_duo_ssse3",
        kind: 19,
        dispatch_only: true,
        features: &[0, 1, 3, 4, 5, 6, 39, 43, 44, 105, 110, 112],
    },
    X86Processor {
        name: "penryn",
        kind: 20,
        dispatch_only: false,
        features: &[0, 1, 3, 4, 5, 6, 7, 39, 43, 44, 105, 110, 112],
    },
    X86Processor {
        name: "core_2_duo_sse4_1",
        kind: 20,
        dispatch_only: true,
        features: &[0, 1, 3, 4, 5, 6, 7, 39, 43, 44, 105, 110, 112],
    },
    X86Processor {
        name: "bonnell",
        kind: 21,
        dispatch_only: false,
        features: &[0, 1, 3, 4, 5, 6, 39, 43, 44, 47, 105, 110, 112],
    },
    X86Processor {
        name: "atom",
        kind: 21,
        dispatch_only: false,
        features: &[0, 1, 3, 4, 5, 6, 39, 43, 44, 47, 105, 110, 112],
    },
    X86Processor {
        name: "silvermont",
        kind: 22,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 19, 39, 43, 44, 47, 53, 56, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "slm",
        kind: 22,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 19, 39, 43, 44, 47, 53, 56, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "atom_sse4_2",
        kind: 27,
        dispatch_only: true,
        features: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 39, 43, 44, 105, 106, 110, 112],
    },
    X86Processor {
        name: "atom_sse4_2_movbe",
        kind: 23,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 18, 19, 36, 39, 42, 43, 44, 47, 53, 56, 57, 61, 68, 69, 70,
            71, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "goldmont",
        kind: 23,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 18, 19, 36, 39, 42, 43, 44, 47, 53, 56, 57, 61, 68, 69, 70,
            71, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "goldmont-plus",
        kind: 24,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 18, 19, 36, 39, 42, 43, 44, 47, 53, 54, 55, 56, 57, 60, 61,
            68, 69, 70, 71, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "goldmont_plus",
        kind: 24,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 18, 19, 36, 39, 42, 43, 44, 47, 53, 54, 55, 56, 57, 60, 61,
            68, 69, 70, 71, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "tremont",
        kind: 25,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 18, 19, 28, 36, 37, 39, 42, 43, 44, 47, 53, 54, 55, 56, 57,
            60, 61, 68, 69, 70, 71, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "nehalem",
        kind: 27,
        dispatch_only: false,
        features: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 39, 43, 44, 105, 106, 110, 112],
    },
    X86Processor {
        name: "core_i7_sse4_2",
        kind: 27,
        dispatch_only: true,
        features: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 39, 43, 44, 105, 106, 110, 112],
    },
    X86Processor {
        name: "corei7",
        kind: 27,
        dispatch_only: false,
        features: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 39, 43, 44, 105, 106, 110, 112],
    },
    X86Processor {
        name: "westmere",
        kind: 28,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 19, 39, 43, 44, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "core_aes_pclmulqdq",
        kind: 27,
        dispatch_only: true,
        features: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 39, 43, 44, 105, 106, 110, 112],
    },
    X86Processor {
        name: "sandybridge",
        kind: 29,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 19, 39, 43, 44, 68, 70, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "core_2nd_gen_avx",
        kind: 29,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 19, 39, 43, 44, 68, 70, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "corei7-avx",
        kind: 29,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 19, 39, 43, 44, 68, 70, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "ivybridge",
        kind: 30,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 19, 39, 41, 42, 43, 44, 56, 68, 70, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "core_3rd_gen_avx",
        kind: 30,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 19, 39, 41, 42, 43, 44, 56, 68, 70, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "core-avx-i",
        kind: 30,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 19, 39, 41, 42, 43, 44, 56, 68, 70, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "haswell",
        kind: 31,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 16, 17, 19, 39, 41, 42, 43, 44, 46, 47, 56, 68,
            70, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "core-avx2",
        kind: 31,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 16, 17, 19, 39, 41, 42, 43, 44, 46, 47, 56, 68,
            70, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "core_4th_gen_avx",
        kind: 31,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 16, 17, 19, 39, 41, 42, 43, 44, 46, 47, 56, 68,
            70, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "core_4th_gen_avx_tsx",
        kind: 31,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 16, 17, 19, 39, 41, 42, 43, 44, 46, 47, 56, 68,
            70, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "broadwell",
        kind: 32,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 16, 17, 19, 34, 39, 41, 42, 43, 44, 46, 47, 53,
            56, 57, 68, 70, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "core_5th_gen_avx",
        kind: 32,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 16, 17, 19, 34, 39, 41, 42, 43, 44, 46, 47, 53,
            56, 57, 68, 70, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "core_5th_gen_avx_tsx",
        kind: 32,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 16, 17, 19, 34, 39, 41, 42, 43, 44, 46, 47, 53,
            56, 57, 68, 70, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "skylake",
        kind: 33,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 16, 17, 18, 19, 34, 36, 39, 41, 42, 43, 44, 46,
            47, 53, 56, 57, 60, 68, 69, 70, 71, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "skylake-avx512",
        kind: 34,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 34, 36, 37,
            39, 41, 42, 43, 44, 46, 47, 52, 53, 56, 57, 68, 69, 70, 71, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "skx",
        kind: 34,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 34, 36, 37,
            39, 41, 42, 43, 44, 46, 47, 52, 53, 56, 57, 68, 69, 70, 71, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "skylake_avx512",
        kind: 34,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 34, 36, 37,
            39, 41, 42, 43, 44, 46, 47, 52, 53, 56, 57, 68, 69, 70, 71, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "cascadelake",
        kind: 35,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 30, 34, 36,
            37, 39, 41, 42, 43, 44, 46, 47, 52, 53, 56, 57, 68, 69, 70, 71, 105, 106, 107, 110,
            112,
        ],
    },
    X86Processor {
        name: "cooperlake",
        kind: 36,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 30, 32, 34,
            36, 37, 39, 41, 42, 43, 44, 46, 47, 52, 53, 56, 57, 68, 69, 70, 71, 105, 106, 107, 110,
            112,
        ],
    },
    X86Processor {
        name: "cannonlake",
        kind: 37,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 34,
            36, 39, 41, 42, 43, 44, 46, 47, 52, 53, 56, 57, 60, 61, 68, 69, 70, 71, 105, 106, 107,
            110, 112,
        ],
    },
    X86Processor {
        name: "icelake-client",
        kind: 38,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 34, 36, 39, 41, 42, 43, 44, 46, 47, 52, 53, 55, 56, 57, 60, 61, 65,
            68, 69, 70, 71, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "icelake_client",
        kind: 38,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 34, 36, 39, 41, 42, 43, 44, 46, 47, 52, 53, 55, 56, 57, 60, 61, 65,
            68, 69, 70, 71, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "rocketlake",
        kind: 39,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 34, 36, 39, 41, 42, 43, 44, 46, 47, 52, 53, 55, 56, 57, 61, 65, 68,
            69, 70, 71, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "icelake-server",
        kind: 40,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 34, 36, 37, 39, 41, 42, 43, 44, 46, 47, 51, 52, 53, 55, 56, 57, 60,
            61, 65, 67, 68, 69, 70, 71, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "icelake_server",
        kind: 40,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 34, 36, 37, 39, 41, 42, 43, 44, 46, 47, 51, 52, 53, 55, 56, 57, 60,
            61, 65, 67, 68, 69, 70, 71, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "tigerlake",
        kind: 41,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 33, 34, 36, 37, 39, 41, 42, 43, 44, 46, 47, 48, 49, 52, 53, 55, 56,
            57, 60, 61, 62, 65, 68, 69, 70, 71, 77, 78, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "sapphirerapids",
        kind: 42,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 32, 34, 35, 36, 37, 39, 40, 41, 42, 43, 44, 46, 47, 48, 49, 51, 52,
            53, 54, 55, 56, 57, 59, 60, 61, 62, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 79,
            80, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "alderlake",
        kind: 43,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 36, 37, 39, 41, 42, 43, 44,
            46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70, 71, 76,
            78, 79, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "raptorlake",
        kind: 44,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 36, 37, 39, 41, 42, 43, 44,
            46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70, 71, 76,
            78, 79, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "meteorlake",
        kind: 45,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 36, 37, 39, 41, 42, 43, 44,
            46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70, 71, 76,
            78, 79, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "arrowlake",
        kind: 46,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 36, 37, 39, 40, 41, 42, 43,
            44, 46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70, 71,
            75, 76, 78, 79, 81, 82, 83, 84, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "arrowlake-s",
        kind: 47,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 36, 37, 39, 40, 41, 42, 43,
            44, 46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70, 71,
            75, 76, 78, 79, 81, 82, 83, 84, 89, 90, 91, 92, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "arrowlake_s",
        kind: 47,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 36, 37, 39, 40, 41, 42, 43,
            44, 46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70, 71,
            75, 76, 78, 79, 81, 82, 83, 84, 89, 90, 91, 92, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "lunarlake",
        kind: 48,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 36, 37, 39, 40, 41, 42, 43,
            44, 46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70, 71,
            75, 76, 78, 79, 81, 82, 83, 84, 89, 90, 91, 92, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "gracemont",
        kind: 26,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 36, 37, 39, 41, 42, 43, 44,
            46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70, 71, 76,
            78, 79, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "pantherlake",
        kind: 48,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 36, 37, 39, 40, 41, 42, 43,
            44, 46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70, 71,
            75, 76, 79, 81, 82, 83, 84, 89, 90, 91, 92, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "wildcatlake",
        kind: 48,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 36, 37, 39, 40, 41, 42, 43,
            44, 46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70, 71,
            75, 76, 79, 81, 82, 83, 84, 89, 90, 91, 92, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "novalake",
        kind: 51,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 36, 37, 39, 40, 41, 42, 43,
            44, 46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70, 71,
            75, 76, 79, 81, 82, 83, 84, 86, 89, 90, 91, 92, 96, 99, 103, 105, 106, 107, 110, 112,
            113, 114, 115, 116, 117, 118, 119,
        ],
    },
    X86Processor {
        name: "sierraforest",
        kind: 52,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 35, 36, 37, 39, 40, 41, 42,
            43, 44, 46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70,
            71, 75, 76, 78, 79, 81, 82, 83, 84, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "grandridge",
        kind: 53,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 35, 36, 37, 39, 40, 41, 42,
            43, 44, 46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70,
            71, 75, 76, 78, 79, 81, 82, 83, 84, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "graniterapids",
        kind: 54,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 32, 34, 35, 36, 37, 39, 40, 41, 42, 43, 44, 46, 47, 48, 49, 51, 52,
            53, 54, 55, 56, 57, 59, 60, 61, 62, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 79,
            80, 85, 86, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "graniterapids-d",
        kind: 55,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 32, 34, 35, 36, 37, 39, 40, 41, 42, 43, 44, 46, 47, 48, 49, 51, 52,
            53, 54, 55, 56, 57, 59, 60, 61, 62, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 79,
            80, 85, 86, 88, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "graniterapids_d",
        kind: 55,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 32, 34, 35, 36, 37, 39, 40, 41, 42, 43, 44, 46, 47, 48, 49, 51, 52,
            53, 54, 55, 56, 57, 59, 60, 61, 62, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 79,
            80, 85, 86, 88, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "emeraldrapids",
        kind: 56,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 32, 34, 35, 36, 37, 39, 40, 41, 42, 43, 44, 46, 47, 48, 49, 51, 52,
            53, 54, 55, 56, 57, 59, 60, 61, 62, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 79,
            80, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "clearwaterforest",
        kind: 48,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 14, 16, 17, 18, 19, 28, 29, 34, 35, 36, 37, 39, 40, 41, 42,
            43, 44, 46, 47, 48, 49, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 65, 66, 68, 69, 70,
            71, 75, 76, 79, 81, 82, 83, 84, 86, 89, 90, 91, 92, 94, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "diamondrapids",
        kind: 58,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 32, 34, 35, 36, 37, 39, 40, 41, 42, 43, 44, 46, 47, 48, 49, 51, 52,
            53, 54, 55, 56, 57, 59, 60, 61, 62, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 79,
            80, 81, 82, 83, 84, 85, 86, 88, 89, 90, 91, 92, 96, 97, 98, 99, 100, 103, 105, 106,
            107, 110, 112, 113, 114, 115, 116, 117, 118, 119,
        ],
    },
    X86Processor {
        name: "knl",
        kind: 59,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 23, 34, 39, 41, 42, 43, 44,
            46, 47, 53, 56, 57, 68, 70, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "mic_avx512",
        kind: 59,
        dispatch_only: true,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 23, 34, 39, 41, 42, 43, 44,
            46, 47, 53, 56, 57, 68, 70, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "knm",
        kind: 60,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 16, 17, 18, 19, 23, 26, 34, 39, 41, 42, 43,
            44, 46, 47, 53, 56, 57, 68, 70, 105, 106, 107, 110, 112,
        ],
    },
    X86Processor {
        name: "lakemont",
        kind: 61,
        dispatch_only: false,
        features: &[105],
    },
    X86Processor {
        name: "k6",
        kind: 62,
        dispatch_only: false,
        features: &[1, 105, 110],
    },
    X86Processor {
        name: "k6-2",
        kind: 63,
        dispatch_only: false,
        features: &[1, 53, 105, 110],
    },
    X86Processor {
        name: "k6-3",
        kind: 64,
        dispatch_only: false,
        features: &[1, 53, 105, 110],
    },
    X86Processor {
        name: "athlon",
        kind: 65,
        dispatch_only: false,
        features: &[1, 53, 105, 110],
    },
    X86Processor {
        name: "athlon-tbird",
        kind: 65,
        dispatch_only: false,
        features: &[1, 53, 105, 110],
    },
    X86Processor {
        name: "athlon-xp",
        kind: 66,
        dispatch_only: false,
        features: &[1, 3, 53, 105, 110, 112],
    },
    X86Processor {
        name: "athlon-mp",
        kind: 66,
        dispatch_only: false,
        features: &[1, 3, 53, 105, 110, 112],
    },
    X86Processor {
        name: "athlon-4",
        kind: 66,
        dispatch_only: false,
        features: &[1, 3, 53, 105, 110, 112],
    },
    X86Processor {
        name: "k8",
        kind: 67,
        dispatch_only: false,
        features: &[1, 3, 4, 44, 53, 105, 110, 112],
    },
    X86Processor {
        name: "athlon64",
        kind: 67,
        dispatch_only: false,
        features: &[1, 3, 4, 44, 53, 105, 110, 112],
    },
    X86Processor {
        name: "athlon-fx",
        kind: 67,
        dispatch_only: false,
        features: &[1, 3, 4, 44, 53, 105, 110, 112],
    },
    X86Processor {
        name: "opteron",
        kind: 67,
        dispatch_only: false,
        features: &[1, 3, 4, 44, 53, 105, 110, 112],
    },
    X86Processor {
        name: "k8-sse3",
        kind: 68,
        dispatch_only: false,
        features: &[1, 3, 4, 5, 44, 53, 105, 110, 112],
    },
    X86Processor {
        name: "athlon64-sse3",
        kind: 68,
        dispatch_only: false,
        features: &[1, 3, 4, 5, 44, 53, 105, 110, 112],
    },
    X86Processor {
        name: "opteron-sse3",
        kind: 68,
        dispatch_only: false,
        features: &[1, 3, 4, 5, 44, 53, 105, 110, 112],
    },
    X86Processor {
        name: "amdfam10",
        kind: 69,
        dispatch_only: false,
        features: &[1, 2, 3, 4, 5, 11, 39, 43, 44, 46, 53, 105, 110, 112],
    },
    X86Processor {
        name: "barcelona",
        kind: 69,
        dispatch_only: false,
        features: &[1, 2, 3, 4, 5, 11, 39, 43, 44, 46, 53, 105, 110, 112],
    },
    X86Processor {
        name: "btver1",
        kind: 70,
        dispatch_only: false,
        features: &[1, 2, 3, 4, 5, 6, 11, 39, 43, 44, 46, 53, 105, 110, 112],
    },
    X86Processor {
        name: "btver2",
        kind: 71,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 9, 11, 16, 18, 19, 39, 41, 43, 44, 46, 47, 53, 68, 70, 105, 106, 110,
            112,
        ],
    },
    X86Processor {
        name: "bdver1",
        kind: 72,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 18, 19, 39, 43, 44, 45, 46, 53, 68, 105, 106,
            110, 112,
        ],
    },
    X86Processor {
        name: "bdver2",
        kind: 73,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 16, 18, 19, 39, 41, 43, 44, 45, 46, 53, 63,
            68, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "bdver3",
        kind: 74,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 16, 18, 19, 39, 41, 42, 43, 44, 45, 46, 53,
            63, 68, 70, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "bdver4",
        kind: 75,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 18, 19, 39, 41, 42, 43, 44, 45,
            46, 47, 50, 53, 56, 63, 68, 70, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "znver1",
        kind: 76,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 14, 16, 17, 18, 19, 34, 36, 38, 39, 41, 42, 43, 44,
            46, 47, 50, 53, 56, 57, 61, 68, 69, 70, 71, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "znver2",
        kind: 77,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 14, 16, 17, 18, 19, 34, 36, 37, 38, 39, 41, 42, 43,
            44, 46, 47, 50, 53, 55, 56, 57, 61, 67, 68, 69, 70, 71, 105, 106, 108, 110, 112,
        ],
    },
    X86Processor {
        name: "znver3",
        kind: 78,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 14, 16, 17, 18, 19, 29, 34, 36, 37, 38, 39, 41, 42,
            43, 44, 46, 47, 50, 52, 53, 55, 56, 57, 61, 65, 67, 68, 69, 70, 71, 105, 106, 107, 108,
            110, 112,
        ],
    },
    X86Processor {
        name: "znver4",
        kind: 79,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 32, 34, 36, 37, 38, 39, 41, 42, 43, 44, 46, 47, 50, 52, 53, 55, 56,
            57, 61, 62, 65, 67, 68, 69, 70, 71, 105, 106, 107, 108, 110, 112,
        ],
    },
    X86Processor {
        name: "znver5",
        kind: 80,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 32, 33, 34, 36, 37, 38, 39, 41, 42, 43, 44, 46, 47, 48, 49, 50, 52,
            53, 55, 56, 57, 61, 62, 65, 67, 68, 69, 70, 71, 79, 86, 105, 106, 107, 108, 110, 112,
        ],
    },
    X86Processor {
        name: "znver6",
        kind: 81,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 32, 33, 34, 36, 37, 38, 39, 41, 42, 43, 44, 46, 47, 48, 49, 50, 52,
            53, 55, 56, 57, 61, 62, 65, 67, 68, 69, 70, 71, 79, 80, 81, 82, 83, 86, 101, 105, 106,
            107, 108, 110, 112,
        ],
    },
    X86Processor {
        name: "c86-4g-m4",
        kind: 82,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 14, 16, 17, 18, 19, 34, 36, 38, 39, 41, 42, 43, 44,
            46, 47, 50, 53, 56, 57, 61, 68, 69, 70, 71, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "c86-4g-m6",
        kind: 83,
        dispatch_only: false,
        features: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 14, 16, 17, 18, 19, 34, 36, 38, 39, 41, 42, 43, 44,
            46, 47, 50, 53, 56, 57, 61, 68, 69, 70, 71, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "c86-4g-m7",
        kind: 84,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
            26, 27, 28, 29, 30, 31, 32, 34, 36, 37, 38, 39, 41, 42, 43, 44, 46, 47, 50, 53, 56, 57,
            61, 65, 67, 68, 69, 70, 71, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "c86-4g-m8",
        kind: 85,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
            26, 27, 28, 29, 30, 31, 32, 34, 36, 37, 38, 39, 41, 42, 43, 44, 46, 47, 50, 53, 56, 57,
            61, 62, 65, 67, 68, 69, 70, 71, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "x86-64",
        kind: 86,
        dispatch_only: false,
        features: &[0, 1, 3, 4, 44, 105, 110, 112],
    },
    X86Processor {
        name: "x86-64-v2",
        kind: 87,
        dispatch_only: false,
        features: &[0, 1, 2, 3, 4, 8, 39, 43, 44, 105, 106, 110, 112],
    },
    X86Processor {
        name: "x86-64-v3",
        kind: 88,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 8, 10, 14, 16, 17, 39, 41, 43, 44, 46, 47, 68, 105, 106, 110, 112,
        ],
    },
    X86Processor {
        name: "x86-64-v4",
        kind: 89,
        dispatch_only: false,
        features: &[
            0, 1, 2, 3, 4, 8, 10, 14, 16, 17, 20, 21, 22, 23, 39, 41, 43, 44, 46, 47, 68, 105, 106,
            110, 112,
        ],
    },
    X86Processor {
        name: "geode",
        kind: 90,
        dispatch_only: false,
        features: &[1, 53, 105, 110],
    },
];
const ARM_FLOATING_POINT_UNITS: [ArmFloatingPointUnit; 24] = [
    ArmFloatingPointUnit {
        version: 0,
        neon: 0,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 0,
        neon: 0,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 1,
        neon: 0,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 1,
        neon: 0,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 2,
        neon: 0,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 3,
        neon: 0,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 2,
        neon: 0,
        restriction: 1,
    },
    ArmFloatingPointUnit {
        version: 3,
        neon: 0,
        restriction: 1,
    },
    ArmFloatingPointUnit {
        version: 2,
        neon: 0,
        restriction: 2,
    },
    ArmFloatingPointUnit {
        version: 3,
        neon: 0,
        restriction: 2,
    },
    ArmFloatingPointUnit {
        version: 4,
        neon: 0,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 4,
        neon: 0,
        restriction: 1,
    },
    ArmFloatingPointUnit {
        version: 4,
        neon: 0,
        restriction: 2,
    },
    ArmFloatingPointUnit {
        version: 5,
        neon: 0,
        restriction: 1,
    },
    ArmFloatingPointUnit {
        version: 5,
        neon: 0,
        restriction: 2,
    },
    ArmFloatingPointUnit {
        version: 5,
        neon: 0,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 6,
        neon: 0,
        restriction: 1,
    },
    ArmFloatingPointUnit {
        version: 6,
        neon: 0,
        restriction: 2,
    },
    ArmFloatingPointUnit {
        version: 2,
        neon: 1,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 3,
        neon: 1,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 4,
        neon: 1,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 5,
        neon: 1,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 5,
        neon: 2,
        restriction: 0,
    },
    ArmFloatingPointUnit {
        version: 0,
        neon: 0,
        restriction: 0,
    },
];

const ARM_ARCHITECTURES: [ArmArchitecture; 43] = [
    ArmArchitecture {
        name: "invalid",
        kind: "INVALID",
        cpu_attribute: "",
        sub_architecture: "",
        default_fpu: 1,
        base_extensions: 1,
    },
    ArmArchitecture {
        name: "armv4",
        kind: "ARMV4",
        cpu_attribute: "4",
        sub_architecture: "v4",
        default_fpu: 1,
        base_extensions: 1,
    },
    ArmArchitecture {
        name: "armv4t",
        kind: "ARMV4T",
        cpu_attribute: "4T",
        sub_architecture: "v4t",
        default_fpu: 1,
        base_extensions: 1,
    },
    ArmArchitecture {
        name: "armv5t",
        kind: "ARMV5T",
        cpu_attribute: "5T",
        sub_architecture: "v5",
        default_fpu: 1,
        base_extensions: 1,
    },
    ArmArchitecture {
        name: "armv5te",
        kind: "ARMV5TE",
        cpu_attribute: "5TE",
        sub_architecture: "v5e",
        default_fpu: 1,
        base_extensions: 1024,
    },
    ArmArchitecture {
        name: "armv5tej",
        kind: "ARMV5TEJ",
        cpu_attribute: "5TEJ",
        sub_architecture: "v5e",
        default_fpu: 1,
        base_extensions: 1024,
    },
    ArmArchitecture {
        name: "armv6",
        kind: "ARMV6",
        cpu_attribute: "6",
        sub_architecture: "v6",
        default_fpu: 3,
        base_extensions: 1024,
    },
    ArmArchitecture {
        name: "armv6k",
        kind: "ARMV6K",
        cpu_attribute: "6K",
        sub_architecture: "v6k",
        default_fpu: 3,
        base_extensions: 1024,
    },
    ArmArchitecture {
        name: "armv6t2",
        kind: "ARMV6T2",
        cpu_attribute: "6T2",
        sub_architecture: "v6t2",
        default_fpu: 1,
        base_extensions: 1024,
    },
    ArmArchitecture {
        name: "armv6kz",
        kind: "ARMV6KZ",
        cpu_attribute: "6KZ",
        sub_architecture: "v6kz",
        default_fpu: 3,
        base_extensions: 1280,
    },
    ArmArchitecture {
        name: "armv6-m",
        kind: "ARMV6M",
        cpu_attribute: "6-M",
        sub_architecture: "v6m",
        default_fpu: 1,
        base_extensions: 1,
    },
    ArmArchitecture {
        name: "armv7-a",
        kind: "ARMV7A",
        cpu_attribute: "7-A",
        sub_architecture: "v7",
        default_fpu: 18,
        base_extensions: 1024,
    },
    ArmArchitecture {
        name: "armv7ve",
        kind: "ARMV7VE",
        cpu_attribute: "7VE",
        sub_architecture: "v7ve",
        default_fpu: 18,
        base_extensions: 1904,
    },
    ArmArchitecture {
        name: "armv7-r",
        kind: "ARMV7R",
        cpu_attribute: "7-R",
        sub_architecture: "v7r",
        default_fpu: 1,
        base_extensions: 1040,
    },
    ArmArchitecture {
        name: "armv7-m",
        kind: "ARMV7M",
        cpu_attribute: "7-M",
        sub_architecture: "v7m",
        default_fpu: 1,
        base_extensions: 16,
    },
    ArmArchitecture {
        name: "armv7e-m",
        kind: "ARMV7EM",
        cpu_attribute: "7E-M",
        sub_architecture: "v7em",
        default_fpu: 1,
        base_extensions: 1040,
    },
    ArmArchitecture {
        name: "armv8-a",
        kind: "ARMV8A",
        cpu_attribute: "8-A",
        sub_architecture: "v8a",
        default_fpu: 22,
        base_extensions: 1906,
    },
    ArmArchitecture {
        name: "armv8.1-a",
        kind: "ARMV8_1A",
        cpu_attribute: "8.1-A",
        sub_architecture: "v8.1a",
        default_fpu: 22,
        base_extensions: 1906,
    },
    ArmArchitecture {
        name: "armv8.2-a",
        kind: "ARMV8_2A",
        cpu_attribute: "8.2-A",
        sub_architecture: "v8.2a",
        default_fpu: 22,
        base_extensions: 6002,
    },
    ArmArchitecture {
        name: "armv8.3-a",
        kind: "ARMV8_3A",
        cpu_attribute: "8.3-A",
        sub_architecture: "v8.3a",
        default_fpu: 22,
        base_extensions: 6002,
    },
    ArmArchitecture {
        name: "armv8.4-a",
        kind: "ARMV8_4A",
        cpu_attribute: "8.4-A",
        sub_architecture: "v8.4a",
        default_fpu: 22,
        base_extensions: 14194,
    },
    ArmArchitecture {
        name: "armv8.5-a",
        kind: "ARMV8_5A",
        cpu_attribute: "8.5-A",
        sub_architecture: "v8.5a",
        default_fpu: 22,
        base_extensions: 14194,
    },
    ArmArchitecture {
        name: "armv8.6-a",
        kind: "ARMV8_6A",
        cpu_attribute: "8.6-A",
        sub_architecture: "v8.6a",
        default_fpu: 22,
        base_extensions: 3159922,
    },
    ArmArchitecture {
        name: "armv8.7-a",
        kind: "ARMV8_7A",
        cpu_attribute: "8.7-A",
        sub_architecture: "v8.7a",
        default_fpu: 22,
        base_extensions: 3159922,
    },
    ArmArchitecture {
        name: "armv8.8-a",
        kind: "ARMV8_8A",
        cpu_attribute: "8.8-A",
        sub_architecture: "v8.8a",
        default_fpu: 22,
        base_extensions: 3209074,
    },
    ArmArchitecture {
        name: "armv8.9-a",
        kind: "ARMV8_9A",
        cpu_attribute: "8.9-A",
        sub_architecture: "v8.9a",
        default_fpu: 22,
        base_extensions: 3209074,
    },
    ArmArchitecture {
        name: "armv9-a",
        kind: "ARMV9A",
        cpu_attribute: "9-A",
        sub_architecture: "v9a",
        default_fpu: 21,
        base_extensions: 14194,
    },
    ArmArchitecture {
        name: "armv9.1-a",
        kind: "ARMV9_1A",
        cpu_attribute: "9.1-A",
        sub_architecture: "v9.1a",
        default_fpu: 21,
        base_extensions: 3159922,
    },
    ArmArchitecture {
        name: "armv9.2-a",
        kind: "ARMV9_2A",
        cpu_attribute: "9.2-A",
        sub_architecture: "v9.2a",
        default_fpu: 21,
        base_extensions: 3159922,
    },
    ArmArchitecture {
        name: "armv9.3-a",
        kind: "ARMV9_3A",
        cpu_attribute: "9.3-A",
        sub_architecture: "v9.3a",
        default_fpu: 22,
        base_extensions: 3159922,
    },
    ArmArchitecture {
        name: "armv9.4-a",
        kind: "ARMV9_4A",
        cpu_attribute: "9.4-A",
        sub_architecture: "v9.4a",
        default_fpu: 21,
        base_extensions: 3159922,
    },
    ArmArchitecture {
        name: "armv9.5-a",
        kind: "ARMV9_5A",
        cpu_attribute: "9.5-A",
        sub_architecture: "v9.5a",
        default_fpu: 21,
        base_extensions: 3159922,
    },
    ArmArchitecture {
        name: "armv9.6-a",
        kind: "ARMV9_6A",
        cpu_attribute: "9.6-A",
        sub_architecture: "v9.6a",
        default_fpu: 21,
        base_extensions: 3159922,
    },
    ArmArchitecture {
        name: "armv9.7-a",
        kind: "ARMV9_7A",
        cpu_attribute: "9.7-A",
        sub_architecture: "v9.7a",
        default_fpu: 21,
        base_extensions: 3159922,
    },
    ArmArchitecture {
        name: "armv8-r",
        kind: "ARMV8R",
        cpu_attribute: "8-R",
        sub_architecture: "v8r",
        default_fpu: 14,
        base_extensions: 1650,
    },
    ArmArchitecture {
        name: "armv8-m.base",
        kind: "ARMV8MBaseline",
        cpu_attribute: "8-M.Baseline",
        sub_architecture: "v8m.base",
        default_fpu: 1,
        base_extensions: 16,
    },
    ArmArchitecture {
        name: "armv8-m.main",
        kind: "ARMV8MMainline",
        cpu_attribute: "8-M.Mainline",
        sub_architecture: "v8m.main",
        default_fpu: 13,
        base_extensions: 16,
    },
    ArmArchitecture {
        name: "armv8.1-m.main",
        kind: "ARMV8_1MMainline",
        cpu_attribute: "8.1-M.Mainline",
        sub_architecture: "v8.1m.main",
        default_fpu: 17,
        base_extensions: 528400,
    },
    ArmArchitecture {
        name: "iwmmxt",
        kind: "IWMMXT",
        cpu_attribute: "iwmmxt",
        sub_architecture: "",
        default_fpu: 1,
        base_extensions: 1,
    },
    ArmArchitecture {
        name: "iwmmxt2",
        kind: "IWMMXT2",
        cpu_attribute: "iwmmxt2",
        sub_architecture: "",
        default_fpu: 1,
        base_extensions: 1,
    },
    ArmArchitecture {
        name: "xscale",
        kind: "XSCALE",
        cpu_attribute: "xscale",
        sub_architecture: "v5e",
        default_fpu: 1,
        base_extensions: 1,
    },
    ArmArchitecture {
        name: "armv7s",
        kind: "ARMV7S",
        cpu_attribute: "7-S",
        sub_architecture: "v7s",
        default_fpu: 20,
        base_extensions: 1024,
    },
    ArmArchitecture {
        name: "armv7k",
        kind: "ARMV7K",
        cpu_attribute: "7-K",
        sub_architecture: "v7k",
        default_fpu: 1,
        base_extensions: 1024,
    },
];

const ARM_EXTENSIONS: [ArmExtension; 38] = [
    ArmExtension {
        identifier: 0,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 1,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 2,
        feature: "+crc",
        negative_feature: "-crc",
    },
    ArmExtension {
        identifier: 4,
        feature: "+crypto",
        negative_feature: "-crypto",
    },
    ArmExtension {
        identifier: 16384,
        feature: "+sha2",
        negative_feature: "-sha2",
    },
    ArmExtension {
        identifier: 32768,
        feature: "+aes",
        negative_feature: "-aes",
    },
    ArmExtension {
        identifier: 8192,
        feature: "+dotprod",
        negative_feature: "-dotprod",
    },
    ArmExtension {
        identifier: 1024,
        feature: "+dsp",
        negative_feature: "-dsp",
    },
    ArmExtension {
        identifier: 8,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 262144,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 2147484672,
        feature: "+mve",
        negative_feature: "-mve",
    },
    ArmExtension {
        identifier: 2147484680,
        feature: "+mve.fp",
        negative_feature: "-mve.fp",
    },
    ArmExtension {
        identifier: 48,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 64,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 128,
        feature: "+neon",
        negative_feature: "-neon",
    },
    ArmExtension {
        identifier: 256,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 512,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 2048,
        feature: "+fullfp16",
        negative_feature: "-fullfp16",
    },
    ArmExtension {
        identifier: 4096,
        feature: "+ras",
        negative_feature: "-ras",
    },
    ArmExtension {
        identifier: 576460752303423488,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 1152921504606846976,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 2305843009213693952,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 4611686018427387904,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 9223372036854775808,
        feature: "",
        negative_feature: "",
    },
    ArmExtension {
        identifier: 65536,
        feature: "+fp16fml",
        negative_feature: "-fp16fml",
    },
    ArmExtension {
        identifier: 1048576,
        feature: "+bf16",
        negative_feature: "-bf16",
    },
    ArmExtension {
        identifier: 131072,
        feature: "+sb",
        negative_feature: "-sb",
    },
    ArmExtension {
        identifier: 2097152,
        feature: "+i8mm",
        negative_feature: "-i8mm",
    },
    ArmExtension {
        identifier: 524288,
        feature: "+lob",
        negative_feature: "-lob",
    },
    ArmExtension {
        identifier: 4194304,
        feature: "+cdecp0",
        negative_feature: "-cdecp0",
    },
    ArmExtension {
        identifier: 8388608,
        feature: "+cdecp1",
        negative_feature: "-cdecp1",
    },
    ArmExtension {
        identifier: 16777216,
        feature: "+cdecp2",
        negative_feature: "-cdecp2",
    },
    ArmExtension {
        identifier: 33554432,
        feature: "+cdecp3",
        negative_feature: "-cdecp3",
    },
    ArmExtension {
        identifier: 67108864,
        feature: "+cdecp4",
        negative_feature: "-cdecp4",
    },
    ArmExtension {
        identifier: 134217728,
        feature: "+cdecp5",
        negative_feature: "-cdecp5",
    },
    ArmExtension {
        identifier: 268435456,
        feature: "+cdecp6",
        negative_feature: "-cdecp6",
    },
    ArmExtension {
        identifier: 536870912,
        feature: "+cdecp7",
        negative_feature: "-cdecp7",
    },
    ArmExtension {
        identifier: 1073741824,
        feature: "+pacbti",
        negative_feature: "-pacbti",
    },
];

const ARM_PROCESSORS: [ArmProcessor; 96] = [
    ArmProcessor {
        name: "arm8",
        architecture: 1,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm810",
        architecture: 1,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "strongarm",
        architecture: 1,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "strongarm110",
        architecture: 1,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "strongarm1100",
        architecture: 1,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "strongarm1110",
        architecture: 1,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm7tdmi",
        architecture: 2,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm7tdmi-s",
        architecture: 2,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm710t",
        architecture: 2,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm720t",
        architecture: 2,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm9",
        architecture: 2,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm9tdmi",
        architecture: 2,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm920",
        architecture: 2,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm920t",
        architecture: 2,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm922t",
        architecture: 2,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm940t",
        architecture: 2,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "ep9312",
        architecture: 2,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm10tdmi",
        architecture: 3,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm1020t",
        architecture: 3,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm9e",
        architecture: 4,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm946e-s",
        architecture: 4,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm966e-s",
        architecture: 4,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm968e-s",
        architecture: 4,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm10e",
        architecture: 4,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm1020e",
        architecture: 4,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm1022e",
        architecture: 4,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm926ej-s",
        architecture: 5,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm1136j-s",
        architecture: 6,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm1136jf-s",
        architecture: 6,
        default_fpu: 3,
        extensions: 1,
    },
    ArmProcessor {
        name: "mpcore",
        architecture: 7,
        default_fpu: 3,
        extensions: 1,
    },
    ArmProcessor {
        name: "mpcorenovfp",
        architecture: 7,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm1176jz-s",
        architecture: 9,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm1176jzf-s",
        architecture: 9,
        default_fpu: 3,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm1156t2-s",
        architecture: 8,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "arm1156t2f-s",
        architecture: 8,
        default_fpu: 3,
        extensions: 1,
    },
    ArmProcessor {
        name: "cortex-m0",
        architecture: 10,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "cortex-m0plus",
        architecture: 10,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "cortex-m1",
        architecture: 10,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "sc000",
        architecture: 10,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "cortex-a5",
        architecture: 11,
        default_fpu: 20,
        extensions: 320,
    },
    ArmProcessor {
        name: "cortex-a7",
        architecture: 11,
        default_fpu: 20,
        extensions: 880,
    },
    ArmProcessor {
        name: "cortex-a8",
        architecture: 11,
        default_fpu: 18,
        extensions: 256,
    },
    ArmProcessor {
        name: "cortex-a9",
        architecture: 11,
        default_fpu: 19,
        extensions: 320,
    },
    ArmProcessor {
        name: "cortex-a12",
        architecture: 11,
        default_fpu: 20,
        extensions: 880,
    },
    ArmProcessor {
        name: "cortex-a15",
        architecture: 11,
        default_fpu: 20,
        extensions: 880,
    },
    ArmProcessor {
        name: "cortex-a17",
        architecture: 11,
        default_fpu: 20,
        extensions: 880,
    },
    ArmProcessor {
        name: "krait",
        architecture: 11,
        default_fpu: 20,
        extensions: 48,
    },
    ArmProcessor {
        name: "cortex-r4",
        architecture: 13,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "cortex-r4f",
        architecture: 13,
        default_fpu: 6,
        extensions: 1,
    },
    ArmProcessor {
        name: "cortex-r5",
        architecture: 13,
        default_fpu: 6,
        extensions: 96,
    },
    ArmProcessor {
        name: "cortex-r7",
        architecture: 13,
        default_fpu: 7,
        extensions: 96,
    },
    ArmProcessor {
        name: "cortex-r8",
        architecture: 13,
        default_fpu: 7,
        extensions: 96,
    },
    ArmProcessor {
        name: "cortex-r52",
        architecture: 34,
        default_fpu: 21,
        extensions: 1,
    },
    ArmProcessor {
        name: "cortex-r52plus",
        architecture: 34,
        default_fpu: 21,
        extensions: 1,
    },
    ArmProcessor {
        name: "sc300",
        architecture: 14,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "cortex-m3",
        architecture: 14,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "cortex-m4",
        architecture: 15,
        default_fpu: 12,
        extensions: 1,
    },
    ArmProcessor {
        name: "cortex-m7",
        architecture: 15,
        default_fpu: 13,
        extensions: 1,
    },
    ArmProcessor {
        name: "cortex-m23",
        architecture: 35,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "cortex-m33",
        architecture: 36,
        default_fpu: 14,
        extensions: 1024,
    },
    ArmProcessor {
        name: "star-mc1",
        architecture: 36,
        default_fpu: 14,
        extensions: 1024,
    },
    ArmProcessor {
        name: "cortex-m35p",
        architecture: 36,
        default_fpu: 14,
        extensions: 1024,
    },
    ArmProcessor {
        name: "cortex-m55",
        architecture: 37,
        default_fpu: 16,
        extensions: 2147486728,
    },
    ArmProcessor {
        name: "cortex-m85",
        architecture: 37,
        default_fpu: 16,
        extensions: 3221232648,
    },
    ArmProcessor {
        name: "cortex-m52",
        architecture: 37,
        default_fpu: 16,
        extensions: 3221232648,
    },
    ArmProcessor {
        name: "star-mc3",
        architecture: 37,
        default_fpu: 16,
        extensions: 3221232648,
    },
    ArmProcessor {
        name: "cortex-a32",
        architecture: 16,
        default_fpu: 22,
        extensions: 2,
    },
    ArmProcessor {
        name: "cortex-a35",
        architecture: 16,
        default_fpu: 22,
        extensions: 2,
    },
    ArmProcessor {
        name: "cortex-a53",
        architecture: 16,
        default_fpu: 22,
        extensions: 2,
    },
    ArmProcessor {
        name: "cortex-a55",
        architecture: 18,
        default_fpu: 22,
        extensions: 10240,
    },
    ArmProcessor {
        name: "cortex-a57",
        architecture: 16,
        default_fpu: 22,
        extensions: 2,
    },
    ArmProcessor {
        name: "cortex-a72",
        architecture: 16,
        default_fpu: 22,
        extensions: 2,
    },
    ArmProcessor {
        name: "cortex-a73",
        architecture: 16,
        default_fpu: 22,
        extensions: 2,
    },
    ArmProcessor {
        name: "cortex-a75",
        architecture: 18,
        default_fpu: 22,
        extensions: 10240,
    },
    ArmProcessor {
        name: "cortex-a76",
        architecture: 18,
        default_fpu: 22,
        extensions: 10240,
    },
    ArmProcessor {
        name: "cortex-a76ae",
        architecture: 18,
        default_fpu: 22,
        extensions: 10240,
    },
    ArmProcessor {
        name: "cortex-a77",
        architecture: 18,
        default_fpu: 22,
        extensions: 10240,
    },
    ArmProcessor {
        name: "cortex-a78",
        architecture: 18,
        default_fpu: 22,
        extensions: 10240,
    },
    ArmProcessor {
        name: "cortex-a78ae",
        architecture: 18,
        default_fpu: 22,
        extensions: 12288,
    },
    ArmProcessor {
        name: "cortex-a78c",
        architecture: 18,
        default_fpu: 22,
        extensions: 10240,
    },
    ArmProcessor {
        name: "cortex-a510",
        architecture: 26,
        default_fpu: 21,
        extensions: 3350528,
    },
    ArmProcessor {
        name: "cortex-a710",
        architecture: 26,
        default_fpu: 21,
        extensions: 3350528,
    },
    ArmProcessor {
        name: "cortex-x1",
        architecture: 18,
        default_fpu: 22,
        extensions: 10240,
    },
    ArmProcessor {
        name: "cortex-x1c",
        architecture: 18,
        default_fpu: 22,
        extensions: 10240,
    },
    ArmProcessor {
        name: "neoverse-n1",
        architecture: 18,
        default_fpu: 22,
        extensions: 10240,
    },
    ArmProcessor {
        name: "neoverse-n2",
        architecture: 26,
        default_fpu: 21,
        extensions: 3354624,
    },
    ArmProcessor {
        name: "neoverse-v1",
        architecture: 20,
        default_fpu: 22,
        extensions: 1062912,
    },
    ArmProcessor {
        name: "cyclone",
        architecture: 16,
        default_fpu: 22,
        extensions: 2,
    },
    ArmProcessor {
        name: "exynos-m3",
        architecture: 16,
        default_fpu: 22,
        extensions: 2,
    },
    ArmProcessor {
        name: "exynos-m4",
        architecture: 18,
        default_fpu: 22,
        extensions: 10240,
    },
    ArmProcessor {
        name: "exynos-m5",
        architecture: 18,
        default_fpu: 22,
        extensions: 10240,
    },
    ArmProcessor {
        name: "kryo",
        architecture: 16,
        default_fpu: 22,
        extensions: 2,
    },
    ArmProcessor {
        name: "iwmmxt",
        architecture: 38,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "xscale",
        architecture: 40,
        default_fpu: 1,
        extensions: 1,
    },
    ArmProcessor {
        name: "swift",
        architecture: 41,
        default_fpu: 20,
        extensions: 48,
    },
    ArmProcessor {
        name: "invalid",
        architecture: 0,
        default_fpu: 0,
        extensions: 0,
    },
];
const ARM_FLOATING_POINT_VFP2: u32 = 1;
const ARM_FLOATING_POINT_VFP3: u32 = 1 << 1;
const ARM_FLOATING_POINT_VFP4: u32 = 1 << 2;
const ARM_FLOATING_POINT_NEON: u32 = 1 << 3;
const ARM_FLOATING_POINT_ARMV8: u32 = 1 << 4;
const ARM_HARDWARE_HALF: u32 = 1 << 1;
const ARM_HARDWARE_SINGLE: u32 = 1 << 2;
const ARM_HARDWARE_DOUBLE: u32 = 1 << 3;
const ARM_DIVIDE_THUMB: u32 = 1;
const ARM_DIVIDE_ARM: u32 = 1 << 1;
const ARM_VECTOR_INTEGER: u32 = 1;
const ARM_VECTOR_FLOAT: u32 = 1 << 1;
const ARM_EXCLUSIVE_BYTE: u32 = 1;
const ARM_EXCLUSIVE_HALF: u32 = 1 << 1;
const ARM_EXCLUSIVE_WORD: u32 = 1 << 2;
const ARM_EXCLUSIVE_DOUBLE: u32 = 1 << 3;
const ARM_EXTENSION_HARDWARE_DIVIDE_THUMB: u64 = 1 << 4;
const ARM_EXTENSION_HARDWARE_DIVIDE_ARM: u64 = 1 << 5;
const AARCH64_FLOATING_POINT: u32 = 1;
const AARCH64_NEON: u32 = 1 << 1;
const AARCH64_SCALABLE_VECTOR: u32 = 1 << 2;
const MAXIMUM_INTEGER_BITS: u32 = 1 << 23;

const AARCH64_FEATURE_EFFECTS: &[(&str, u32, &[&str])] = &[
    ("+neon", AARCH64_NEON, &[]),
    ("+fp-armv8", AARCH64_NEON, &[]),
    ("+jscvt", AARCH64_NEON, &["jscvt"]),
    ("+fcma", AARCH64_NEON, &["fcma"]),
    (
        "+sve",
        AARCH64_NEON | AARCH64_SCALABLE_VECTOR,
        &["fullfp16"],
    ),
    (
        "+sve2",
        AARCH64_NEON | AARCH64_SCALABLE_VECTOR,
        &["fullfp16", "sve2"],
    ),
    (
        "+sve2p1",
        AARCH64_NEON | AARCH64_SCALABLE_VECTOR,
        &["fullfp16", "sve2", "sve2p1"],
    ),
    ("+sve-aes", AARCH64_NEON, &["fullfp16", "sve-aes"]),
    (
        "+sve2-sha3",
        AARCH64_NEON | AARCH64_SCALABLE_VECTOR,
        &["fullfp16", "sve2", "sve2-sha3"],
    ),
    (
        "+sve2-sm4",
        AARCH64_NEON | AARCH64_SCALABLE_VECTOR,
        &["fullfp16", "sve2", "sve2-sm4"],
    ),
    ("+sve-b16b16", 0, &["sve-b16b16"]),
    ("+sve-bitperm", AARCH64_NEON, &["fullfp16", "sve-bitperm"]),
    (
        "+f32mm",
        AARCH64_NEON | AARCH64_SCALABLE_VECTOR,
        &["fullfp16", "f32mm"],
    ),
    (
        "+f64mm",
        AARCH64_NEON | AARCH64_SCALABLE_VECTOR,
        &["fullfp16", "f64mm"],
    ),
    ("+sme", 0, &["sme", "bf16", "fullfp16"]),
    ("+sme2", 0, &["sme", "sme2", "bf16", "fullfp16"]),
    ("+sme2p1", 0, &["sme", "sme2", "sme2p1", "bf16", "fullfp16"]),
    ("+sme-f64f64", 0, &["sme", "sme-f64f64", "bf16", "fullfp16"]),
    ("+sme-i16i64", 0, &["sme", "sme-i16i64", "bf16", "fullfp16"]),
    (
        "+sme-fa64",
        AARCH64_NEON | AARCH64_SCALABLE_VECTOR,
        &["sme", "sve2", "sme-fa64"],
    ),
    (
        "+sme-f16f16",
        0,
        &["sme", "sme2", "bf16", "fullfp16", "sme-f16f16"],
    ),
    (
        "+sme-b16b16",
        0,
        &[
            "sme",
            "sme2",
            "bf16",
            "fullfp16",
            "sve-b16b16",
            "sme-b16b16",
        ],
    ),
    ("+fp8", 0, &["fp8"]),
    ("+fp8fma", 0, &["fp8fma"]),
    ("+fp8dot2", 0, &["fp8dot2"]),
    ("+fp8dot4", 0, &["fp8dot4"]),
    ("+ssve-fp8dot2", 0, &["ssve-fp8dot2"]),
    ("+ssve-fp8dot4", 0, &["ssve-fp8dot4"]),
    ("+ssve-fp8fma", 0, &["ssve-fp8fma"]),
    ("+sme-f8f32", 0, &["sme-f8f32"]),
    ("+sme-f8f16", 0, &["sme-f8f16"]),
    ("+sb", 0, &["sb"]),
    ("+predres", 0, &["predres"]),
    ("+ssbs", 0, &["ssbs"]),
    ("+bti", 0, &["bti"]),
    ("+wfxt", 0, &["wfxt"]),
    ("+crc", 0, &["crc"]),
    ("+rcpc", 0, &["rcpc"]),
    ("+aes", AARCH64_NEON, &["aes"]),
    ("+sha2", AARCH64_NEON, &["sha2"]),
    ("+sha3", AARCH64_NEON, &["sha2", "sha3"]),
    ("+rdm", AARCH64_NEON, &["rdm"]),
    ("+dit", 0, &["dit"]),
    ("+cccp", 0, &["ccpp"]),
    ("+ccdp", 0, &["ccpp", "ccdp"]),
    ("+fptoint", 0, &["fptoint"]),
    ("+sm4", AARCH64_NEON, &["sm4"]),
    ("+fprcvt", 0, &["fprcvt"]),
    ("+f8f16mm", 0, &["f8f16mm"]),
    ("+f8f32mm", 0, &["f8f32mm"]),
    ("+sve-f16f32mm", 0, &["sve-f16f32mm"]),
    ("+sve-b16mm", 0, &["sve-b16mm"]),
    ("+f16mm", 0, &["f16mm"]),
    ("+f16f32dot", 0, &["f16f32dot"]),
    ("+f16f32mm", 0, &["f16f32mm"]),
    ("+sve-bfscale", 0, &["sve-bfscale"]),
    ("+sve-aes2", 0, &["sve-aes2"]),
    ("+ssve-aes", 0, &["ssve-aes"]),
    ("+sve2p2", 0, &["sve2p2"]),
    ("+sve2p3", 0, &["sve2p3"]),
    ("+sme2p2", 0, &["sme2p2"]),
    ("+sme2p3", 0, &["sme2p3"]),
    ("+fullfp16", AARCH64_NEON, &["fullfp16"]),
    ("+dotprod", AARCH64_NEON, &["dotprod"]),
    ("+fp16fml", AARCH64_NEON, &["fullfp16", "fp16fml"]),
    ("+mte", 0, &["mte"]),
    ("+pauth", 0, &["pauth"]),
    ("+i8mm", 0, &["i8mm"]),
    ("+bf16", 0, &["bf16"]),
    ("+lse", 0, &["lse"]),
    ("+ls64", 0, &["ls64"]),
    ("+rand", 0, &["rand"]),
    ("+flagm", 0, &["flagm"]),
    ("+altnzcv", 0, &["flagm", "altnzcv"]),
    ("+mops", 0, &["mops"]),
    ("+d128", 0, &["d128"]),
    ("+gcs", 0, &["gcs"]),
    ("+rcpc3", 0, &["rcpc3"]),
    ("+pauth-lr", 0, &["pauth-lr", "pauth"]),
    ("+cssc", 0, &["cssc"]),
];

const X86_FEATURE_MACROS_FIRST: &[(&str, &str)] = &[
    ("aes", "__AES__"),
    ("vaes", "__VAES__"),
    ("pclmul", "__PCLMUL__"),
    ("vpclmulqdq", "__VPCLMULQDQ__"),
];

const X86_FEATURE_MACROS_SECOND: &[(&str, &str)] = &[
    ("lzcnt", "__LZCNT__"),
    ("rdrnd", "__RDRND__"),
    ("fsgsbase", "__FSGSBASE__"),
    ("bmi", "__BMI__"),
    ("bmi2", "__BMI2__"),
    ("popcnt", "__POPCNT__"),
    ("rtm", "__RTM__"),
    ("prfchw", "__PRFCHW__"),
    ("rdseed", "__RDSEED__"),
    ("adx", "__ADX__"),
    ("tbm", "__TBM__"),
    ("lwp", "__LWP__"),
    ("mwaitx", "__MWAITX__"),
    ("movbe", "__MOVBE__"),
];

const X86_FEATURE_MACROS_THIRD: &[(&str, &str)] = &[
    ("fma", "__FMA__"),
    ("f16c", "__F16C__"),
    ("gfni", "__GFNI__"),
];

const X86_FEATURE_MACROS_FOURTH: &[(&str, &str)] = &[
    ("avx10v2aux", "__AVX10_V2_AUX__"),
    ("avx512cd", "__AVX512CD__"),
    ("avx512vpopcntdq", "__AVX512VPOPCNTDQ__"),
    ("avx512vnni", "__AVX512VNNI__"),
    ("avx512bf16", "__AVX512BF16__"),
    ("avx512fp16", "__AVX512FP16__"),
    ("avx512dq", "__AVX512DQ__"),
    ("avx512bitalg", "__AVX512BITALG__"),
    ("avx512bmm", "__AVX512BMM__"),
    ("avx512bw", "__AVX512BW__"),
    ("avx512vl", "__AVX512VL__"),
    ("avx512vbmi", "__AVX512VBMI__"),
    ("avx512vbmi2", "__AVX512VBMI2__"),
    ("avx512ifma", "__AVX512IFMA__"),
    ("avx512vp2intersect", "__AVX512VP2INTERSECT__"),
    ("sha", "__SHA__"),
    ("sha512", "__SHA512__"),
    ("fxsr", "__FXSR__"),
    ("xsave", "__XSAVE__"),
    ("xsaveopt", "__XSAVEOPT__"),
    ("xsavec", "__XSAVEC__"),
    ("xsaves", "__XSAVES__"),
    ("pku", "__PKU__"),
    ("clflushopt", "__CLFLUSHOPT__"),
    ("clwb", "__CLWB__"),
    ("wbnoinvd", "__WBNOINVD__"),
    ("shstk", "__SHSTK__"),
    ("sgx", "__SGX__"),
    ("sm3", "__SM3__"),
    ("sm4", "__SM4__"),
    ("prefetchi", "__PREFETCHI__"),
    ("clzero", "__CLZERO__"),
    ("kl", "__KL__"),
    ("widekl", "__WIDEKL__"),
    ("rdpid", "__RDPID__"),
    ("rdpru", "__RDPRU__"),
    ("cldemote", "__CLDEMOTE__"),
    ("waitpkg", "__WAITPKG__"),
    ("movdiri", "__MOVDIRI__"),
    ("movdir64b", "__MOVDIR64B__"),
    ("movrs", "__MOVRS__"),
    ("pconfig", "__PCONFIG__"),
    ("ptwrite", "__PTWRITE__"),
    ("invpcid", "__INVPCID__"),
    ("enqcmd", "__ENQCMD__"),
    ("hreset", "__HRESET__"),
    ("amx-tile", "__AMX_TILE__"),
    ("amx-int8", "__AMX_INT8__"),
    ("amx-bf16", "__AMX_BF16__"),
    ("amx-fp16", "__AMX_FP16__"),
    ("amx-complex", "__AMX_COMPLEX__"),
    ("amx-fp8", "__AMX_FP8__"),
    ("amx-movrs", "__AMX_MOVRS__"),
    ("amx-avx512", "__AMX_AVX512__"),
    ("cmpccxadd", "__CMPCCXADD__"),
    ("raoint", "__RAOINT__"),
    ("avxifma", "__AVXIFMA__"),
    ("avxneconvert", "__AVXNECONVERT__"),
    ("avxvnni", "__AVXVNNI__"),
    ("avxvnniint16", "__AVXVNNIINT16__"),
    ("avxvnniint8", "__AVXVNNIINT8__"),
    ("serialize", "__SERIALIZE__"),
    ("tsxldtrk", "__TSXLDTRK__"),
    ("uintr", "__UINTR__"),
    ("usermsr", "__USERMSR__"),
    ("crc32", "__CRC32__"),
    ("egpr", "__EGPR__"),
    ("push2pop2", "__PUSH2POP2__"),
    ("ppx", "__PPX__"),
    ("ndd", "__NDD__"),
    ("ccmp", "__CCMP__"),
    ("nf", "__NF__"),
    ("cf", "__CF__"),
    ("zu", "__ZU__"),
    ("jmpabs", "__JMPABS__"),
];

const X86_SSE_LEVELS: &[(&str, u32)] = &[
    ("+avx512f", 9),
    ("+avx2", 8),
    ("+avx", 7),
    ("+sse4.2", 6),
    ("+sse4.1", 5),
    ("+ssse3", 4),
    ("+sse3", 3),
    ("+sse2", 2),
    ("+sse", 1),
];

const X86_SSE_MACROS: &[(u32, &[&str])] = &[
    (9, &["__AVX512F__"]),
    (8, &["__AVX2__"]),
    (7, &["__AVX__"]),
    (6, &["__SSE4_2__"]),
    (5, &["__SSE4_1__"]),
    (4, &["__SSSE3__"]),
    (3, &["__SSE3__"]),
    (2, &["__SSE2__", "__SSE2_MATH__"]),
    (1, &["__SSE__", "__SSE_MATH__"]),
];

pub struct MacroBuilder {
    pub text: String,
}

impl MacroBuilder {
    fn define(&mut self, name: &str, value: &str) {
        self.text.push_str("#define ");
        self.text.push_str(name);
        self.text.push(' ');
        self.text.push_str(value);
        self.text.push('\n');
    }

    fn define_one(&mut self, name: &str) {
        self.define(name, "1");
    }

    fn append(&mut self, line: &str) {
        self.text.push_str(line);
        self.text.push('\n');
    }

    fn define_standard(&mut self, name: &str, gnu: bool) {
        if gnu {
            self.define_one(name);
        }
        self.define_one(&format!("__{name}"));
        self.define_one(&format!("__{name}__"));
    }

    fn define_processor(&mut self, name: &str, tuning: bool) {
        self.define_one(&format!("__{name}"));
        self.define_one(&format!("__{name}__"));
        if tuning {
            self.define_one(&format!("__tune_{name}__"));
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct X86Target {
    pub cpu: usize,
    pub features: BTreeSet<String>,
    pub sse_level: u32,
    pub xop_level: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AArch64Target {
    pub floating_point: u32,
    pub flags: BTreeSet<&'static str>,
    pub unaligned_access: bool,
    pub multi_versioning: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArmTarget {
    pub architecture: usize,
    pub thumb: bool,
    pub abi: String,
    pub floating_point: u32,
    pub hardware_floating_point: u32,
    pub load_exclusive: u32,
    pub hardware_divide: u32,
    pub vector_extension: u32,
    pub coprocessor_mask: u32,
    pub soft_float: bool,
    pub soft_float_abi: bool,
    pub crc: bool,
    pub sha2: bool,
    pub aes: bool,
    pub dsp: bool,
    pub dot_product: bool,
    pub matrix_multiply: bool,
    pub pointer_authentication: bool,
    pub branch_target: bool,
    pub bfloat16: bool,
    pub fast_half: bool,
    pub unaligned_access: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetFamily {
    X86(X86Target),
    AArch64(AArch64Target),
    Arm(ArmTarget),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetInfo {
    pub triple: Triple,
    pub options: TargetOptions,
    pub pointer_width: u32,
    pub bool_width: u32,
    pub short_width: u32,
    pub int_width: u32,
    pub long_width: u32,
    pub long_long_width: u32,
    pub float_width: u32,
    pub double_width: u32,
    pub long_double_width: u32,
    pub suitable_align: u32,
    pub size_type: IntegerType,
    pub ptrdiff_type: IntegerType,
    pub intmax_type: IntegerType,
    pub intptr_type: IntegerType,
    pub wchar_type: IntegerType,
    pub wint_type: IntegerType,
    pub char16_type: IntegerType,
    pub char32_type: IntegerType,
    pub int64_type: IntegerType,
    pub int16_type: IntegerType,
    pub sig_atomic_type: IntegerType,
    pub half_format: FloatFormat,
    pub float_format: FloatFormat,
    pub double_format: FloatFormat,
    pub long_double_format: FloatFormat,
    pub max_atomic_inline_width: u32,
    pub max_atomic_promote_width: u32,
    pub has_float16: bool,
    pub has_float128: bool,
    pub user_label_prefix: &'static str,
    pub family: TargetFamily,
}

impl Default for TargetInfo {
    fn default() -> TargetInfo {
        match TargetInfo::create(&TargetOptions::default()) {
            Ok(target) => target,
            Err(reason) => panic!("{reason}"),
        }
    }
}

fn x86_feature_index(name: &str) -> Option<usize> {
    X86_FEATURES.iter().position(|item| *item == name)
}

fn x86_bits(indices: &[u16]) -> u128 {
    indices
        .iter()
        .fold(0u128, |bits, index| bits | (1u128 << *index))
}

fn x86_set_feature(features: &mut BTreeMap<String, bool>, name: &str, enabled: bool) {
    let name = match (name, enabled) {
        ("sse4", true) => "sse4.2",
        ("sse4", false) => "sse4.1",
        _ => name,
    };
    features.insert(name.to_string(), enabled);
    let Some(index) = x86_feature_index(name) else {
        return;
    };
    let count = X86_FEATURES.len();
    let mut bits = 0u128;
    if enabled {
        let implied = x86_bits(X86_IMPLIED_FEATURES[index]);
        if implied != 0 {
            bits |= implied;
            loop {
                let previous = bits;
                for position in (0..count).rev() {
                    if bits >> position & 1 == 1 {
                        bits |= x86_bits(X86_IMPLIED_FEATURES[position]);
                    }
                }
                if previous == bits {
                    break;
                }
            }
        }
    } else {
        bits |= 1u128 << index;
        loop {
            let previous = bits;
            for (position, implied) in X86_IMPLIED_FEATURES.iter().enumerate() {
                if x86_bits(implied) & bits != 0 {
                    bits |= 1u128 << position;
                }
            }
            if previous == bits {
                break;
            }
        }
    }
    for (position, feature) in X86_FEATURES.iter().enumerate() {
        if bits >> position & 1 == 1 && !feature.is_empty() {
            features.insert(feature.to_string(), enabled);
        }
    }
}

fn x86_parse_processor(name: &str, only_64_bit: bool) -> usize {
    let wide = x86_feature_index("64bit").unwrap_or(0) as u16;
    X86_PROCESSORS
        .iter()
        .find(|processor| {
            !processor.dispatch_only
                && processor.name == name
                && (processor.features.contains(&wide) || !only_64_bit)
        })
        .map(|processor| processor.kind)
        .unwrap_or(0)
}

fn x86_kind_index(name: &str) -> usize {
    X86_CPU_KINDS
        .iter()
        .position(|kind| *kind == name)
        .unwrap_or(0)
}

fn arm_processor(name: &str) -> Option<&'static ArmProcessor> {
    ARM_PROCESSORS
        .iter()
        .find(|processor| processor.name == name)
}

fn arm_default_fpu(cpu: &str, architecture: usize) -> usize {
    if cpu == "generic" {
        return ARM_ARCHITECTURES[architecture].default_fpu;
    }
    arm_processor(cpu)
        .map(|processor| processor.default_fpu)
        .unwrap_or(0)
}

fn arm_default_extensions(cpu: &str, architecture: usize) -> u64 {
    if cpu == "generic" {
        return ARM_ARCHITECTURES[architecture].base_extensions;
    }
    arm_processor(cpu)
        .map(|processor| {
            ARM_ARCHITECTURES[processor.architecture].base_extensions | processor.extensions
        })
        .unwrap_or(0)
}

fn arm_fpu_features(kind: usize, features: &mut Vec<&'static str>) {
    if kind == 0 || kind >= ARM_FLOATING_POINT_UNITS.len() {
        return;
    }
    let unit = &ARM_FLOATING_POINT_UNITS[kind];
    let table: [(&str, &str, u8, u8); 18] = [
        ("+vfp2", "-vfp2", 1, 1),
        ("+vfp2sp", "-vfp2sp", 1, 2),
        ("+vfp3", "-vfp3", 2, 0),
        ("+vfp3d16", "-vfp3d16", 2, 1),
        ("+vfp3d16sp", "-vfp3d16sp", 2, 2),
        ("+vfp3sp", "-vfp3sp", 2, 0),
        ("+fp16", "-fp16", 3, 2),
        ("+vfp4", "-vfp4", 4, 0),
        ("+vfp4d16", "-vfp4d16", 4, 1),
        ("+vfp4d16sp", "-vfp4d16sp", 4, 2),
        ("+vfp4sp", "-vfp4sp", 4, 0),
        ("+fp-armv8", "-fp-armv8", 5, 0),
        ("+fp-armv8d16", "-fp-armv8d16", 5, 1),
        ("+fp-armv8d16sp", "-fp-armv8d16sp", 5, 2),
        ("+fp-armv8sp", "-fp-armv8sp", 5, 0),
        ("+fullfp16", "-fullfp16", 6, 2),
        ("+fp64", "-fp64", 1, 1),
        ("+d32", "-d32", 2, 0),
    ];
    for (plus, minus, version, restriction) in table {
        if unit.version >= version && unit.restriction <= restriction {
            features.push(plus);
        } else {
            features.push(minus);
        }
    }
    for (plus, minus, level) in [
        ("+neon", "-neon", 1),
        ("+sha2", "-sha2", 2),
        ("+aes", "-aes", 2),
    ] {
        if unit.neon >= level {
            features.push(plus);
        } else {
            features.push(minus);
        }
    }
}

fn arm_extension_features(extensions: u64, features: &mut Vec<&'static str>) {
    if extensions == 0 {
        return;
    }
    for extension in ARM_EXTENSIONS.iter() {
        if extensions & extension.identifier == extension.identifier
            && !extension.feature.is_empty()
        {
            features.push(extension.feature);
        } else if !extension.negative_feature.is_empty() {
            features.push(extension.negative_feature);
        }
    }
    features.push(if extensions & ARM_EXTENSION_HARDWARE_DIVIDE_ARM != 0 {
        "+hwdiv-arm"
    } else {
        "-hwdiv-arm"
    });
    features.push(if extensions & ARM_EXTENSION_HARDWARE_DIVIDE_THUMB != 0 {
        "+hwdiv"
    } else {
        "-hwdiv"
    });
}

fn arm_cpu_attribute(architecture: usize) -> &'static str {
    let entry = &ARM_ARCHITECTURES[architecture];
    match entry.kind {
        "ARMV6M" => "6M",
        "ARMV7S" => "7S",
        "ARMV7A" => "7A",
        "ARMV7R" => "7R",
        "ARMV7M" => "7M",
        "ARMV7EM" => "7EM",
        "ARMV7VE" => "7VE",
        "ARMV8A" => "8A",
        "ARMV8_1A" => "8_1A",
        "ARMV8_2A" => "8_2A",
        "ARMV8_3A" => "8_3A",
        "ARMV8_4A" => "8_4A",
        "ARMV8_5A" => "8_5A",
        "ARMV8_6A" => "8_6A",
        "ARMV8_7A" => "8_7A",
        "ARMV8_8A" => "8_8A",
        "ARMV8_9A" => "8_9A",
        "ARMV9A" => "9A",
        "ARMV9_1A" => "9_1A",
        "ARMV9_2A" => "9_2A",
        "ARMV9_3A" => "9_3A",
        "ARMV9_4A" => "9_4A",
        "ARMV9_5A" => "9_5A",
        "ARMV9_6A" => "9_6A",
        "ARMV9_7A" => "9_7A",
        "ARMV8MBaseline" => "8M_BASE",
        "ARMV8MMainline" => "8M_MAIN",
        "ARMV8R" => "8R",
        "ARMV8_1MMainline" => "8_1M_MAIN",
        _ => entry.cpu_attribute,
    }
}

impl ArmTarget {
    fn kind(&self) -> &'static str {
        ARM_ARCHITECTURES[self.architecture].kind
    }

    fn sub_architecture(&self) -> &'static str {
        ARM_ARCHITECTURES[self.architecture].sub_architecture
    }

    fn profile(&self) -> char {
        arm_arch_profile(self.sub_architecture())
    }

    fn version(&self) -> u32 {
        arm_arch_version(self.sub_architecture())
    }

    fn cpu_profile(&self) -> &'static str {
        match self.profile() {
            'A' => "A",
            'R' => "R",
            'M' => "M",
            _ => "",
        }
    }

    fn supports_thumb(&self) -> bool {
        arm_cpu_attribute(self.architecture).contains('T') || self.version() >= 6
    }

    fn supports_thumb2(&self) -> bool {
        let attribute = arm_cpu_attribute(self.architecture);
        attribute == "6T2" || (self.version() >= 7 && attribute != "8M_BASE")
    }

    fn has_vector_extension(&self) -> bool {
        self.kind() == "ARMV8_1MMainline" && self.vector_extension != 0
    }
}

impl TargetInfo {
    fn base(triple: Triple, options: TargetOptions, family: TargetFamily) -> TargetInfo {
        TargetInfo {
            triple,
            options,
            pointer_width: 32,
            bool_width: 8,
            short_width: 16,
            int_width: 32,
            long_width: 32,
            long_long_width: 64,
            float_width: 32,
            double_width: 64,
            long_double_width: 64,
            suitable_align: 64,
            size_type: IntegerType::UnsignedLong,
            ptrdiff_type: IntegerType::Long,
            intmax_type: IntegerType::LongLong,
            intptr_type: IntegerType::Long,
            wchar_type: IntegerType::Int,
            wint_type: IntegerType::Int,
            char16_type: IntegerType::UnsignedShort,
            char32_type: IntegerType::UnsignedInt,
            int64_type: IntegerType::LongLong,
            int16_type: IntegerType::Short,
            sig_atomic_type: IntegerType::Int,
            half_format: FloatFormat::Half,
            float_format: FloatFormat::Single,
            double_format: FloatFormat::Double,
            long_double_format: FloatFormat::Double,
            max_atomic_inline_width: 0,
            max_atomic_promote_width: 0,
            has_float16: false,
            has_float128: false,
            user_label_prefix: "",
            family,
        }
    }

    pub fn create(options: &TargetOptions) -> Result<TargetInfo, String> {
        let triple = Triple::parse(&Triple::normalize(&options.triple));
        let architecture = triple.arch;
        if !matches!(
            architecture,
            "x86" | "x86_64" | "aarch64" | "aarch64_be" | "arm" | "armeb" | "thumb" | "thumbeb"
        ) {
            return Err(format!("target architecture {}", triple.arch_name()));
        }
        if triple.object_format != "elf" || triple.is_darwin() {
            return Err(format!("target object format {}", triple.object_format));
        }
        let linux = triple.os == "linux";
        if !(linux || triple.os == "unknown") || triple.environment == "ohos" {
            return Err(format!("target system {}", triple.os_name()));
        }
        let mut target = match architecture {
            "x86" | "x86_64" => TargetInfo::create_x86(triple, options.clone())?,
            "aarch64" | "aarch64_be" => TargetInfo::create_aarch64(triple, options.clone())?,
            _ => TargetInfo::create_arm(triple, options.clone())?,
        };
        if linux {
            target.wint_type = IntegerType::UnsignedInt;
            if matches!(architecture, "x86" | "x86_64") {
                target.has_float128 = true;
            }
            if target.triple.is_android() && architecture == "x86" {
                target.suitable_align = 32;
                target.long_double_width = 64;
                target.long_double_format = FloatFormat::Double;
            }
            if target.triple.is_android() && architecture == "x86_64" {
                target.long_double_format = FloatFormat::Quadruple;
            }
        }
        target.select_features()?;
        match options.wchar_size {
            1 => {
                target.wchar_type = if options.wchar_signed {
                    IntegerType::SignedChar
                } else {
                    IntegerType::UnsignedChar
                }
            }
            2 => {
                target.wchar_type = if options.wchar_signed {
                    IntegerType::Short
                } else {
                    IntegerType::UnsignedShort
                }
            }
            4 => {
                target.wchar_type = if options.wchar_signed {
                    IntegerType::Int
                } else {
                    IntegerType::UnsignedInt
                }
            }
            _ => {}
        }
        Ok(target)
    }

    fn create_x86(triple: Triple, options: TargetOptions) -> Result<TargetInfo, String> {
        let wide = triple.arch == "x86_64";
        if triple.os == "elfiamcu" {
            return Err("target system elfiamcu".to_string());
        }
        let family = TargetFamily::X86(X86Target {
            cpu: 0,
            features: BTreeSet::new(),
            sse_level: 0,
            xop_level: 0,
        });
        let mut target = TargetInfo::base(triple, options, family);
        target.long_double_format = FloatFormat::ExtendedPrecision;
        if wide {
            let x32 = target.triple.is_x32();
            let width = if x32 { 32 } else { 64 };
            target.long_width = width;
            target.pointer_width = width;
            target.long_double_width = 128;
            target.suitable_align = 128;
            target.size_type = if x32 {
                IntegerType::UnsignedInt
            } else {
                IntegerType::UnsignedLong
            };
            target.ptrdiff_type = if x32 {
                IntegerType::Int
            } else {
                IntegerType::Long
            };
            target.intptr_type = target.ptrdiff_type;
            target.intmax_type = if x32 {
                IntegerType::LongLong
            } else {
                IntegerType::Long
            };
            target.int64_type = target.intmax_type;
            target.max_atomic_promote_width = 128;
            target.max_atomic_inline_width = 64;
        } else {
            target.long_double_width = 96;
            target.suitable_align = 128;
            target.size_type = IntegerType::UnsignedInt;
            target.ptrdiff_type = IntegerType::Int;
            target.intptr_type = IntegerType::Int;
            target.max_atomic_promote_width = 64;
            target.max_atomic_inline_width = 32;
        }
        if !target.options.cpu.is_empty() {
            let kind = x86_parse_processor(&target.options.cpu, wide);
            if kind == 0 {
                return Err(format!("target cpu {}", target.options.cpu));
            }
            if let TargetFamily::X86(state) = &mut target.family {
                state.cpu = kind;
            }
        }
        if !target.options.abi.is_empty() {
            return Err(format!("target abi {}", target.options.abi));
        }
        Ok(target)
    }

    fn create_aarch64(triple: Triple, options: TargetOptions) -> Result<TargetInfo, String> {
        let family = TargetFamily::AArch64(AArch64Target {
            floating_point: AARCH64_FLOATING_POINT,
            flags: BTreeSet::new(),
            unaligned_access: true,
            multi_versioning: true,
        });
        let mut target = TargetInfo::base(triple, options, family);
        target.wchar_type = IntegerType::UnsignedInt;
        target.int64_type = IntegerType::Long;
        target.intmax_type = IntegerType::Long;
        target.has_float16 = true;
        target.long_width = 64;
        target.pointer_width = 64;
        target.max_atomic_inline_width = 128;
        target.max_atomic_promote_width = 128;
        target.long_double_width = 128;
        target.suitable_align = 128;
        target.long_double_format = FloatFormat::Quadruple;
        if !target.options.cpu.is_empty() && target.options.cpu != "generic" {
            return Err(format!("target cpu {}", target.options.cpu));
        }
        if !target.options.abi.is_empty() && target.options.abi != "aapcs" {
            return Err(format!("target abi {}", target.options.abi));
        }
        Ok(target)
    }

    fn create_arm(triple: Triple, options: TargetOptions) -> Result<TargetInfo, String> {
        let soft_float_abi = options
            .features
            .iter()
            .any(|item| item == "+soft-float-abi");
        let name = triple.arch_name().to_string();
        let family = TargetFamily::Arm(ArmTarget {
            architecture: ARM_ARCHITECTURES
                .iter()
                .position(|entry| entry.kind == "ARMV4T")
                .unwrap_or(0),
            thumb: name.starts_with("thumb"),
            abi: String::new(),
            floating_point: 0,
            hardware_floating_point: 0,
            load_exclusive: 0,
            hardware_divide: 0,
            vector_extension: 0,
            coprocessor_mask: 0,
            soft_float: false,
            soft_float_abi,
            crc: false,
            sha2: false,
            aes: false,
            dsp: false,
            dot_product: false,
            matrix_multiply: false,
            pointer_authentication: false,
            branch_target: false,
            bfloat16: false,
            fast_half: false,
            unaligned_access: true,
        });
        let mut target = TargetInfo::base(triple, options, family);
        target.ptrdiff_type = IntegerType::Int;
        target.intptr_type = IntegerType::Int;
        target.size_type = IntegerType::UnsignedInt;
        let parsed = arm_parse_architecture(&name);
        if parsed != 0 {
            target.arm_mut().architecture = parsed;
        }
        let abi = match target.triple.environment {
            "android" | "gnueabi" | "gnueabit64" | "gnueabihf" | "gnueabihft64" | "musleabi"
            | "musleabihf" => "aapcs-linux",
            "eabihf" | "eabi" => "aapcs",
            "gnu" => "apcs-gnu",
            _ => "aapcs",
        };
        target.arm_set_abi(abi);
        target.arm_set_atomic();
        let cpu = target.options.cpu.clone();
        if !cpu.is_empty() {
            if cpu != "generic" {
                let architecture = arm_processor(&cpu)
                    .map(|processor| processor.architecture)
                    .unwrap_or(0);
                target.arm_mut().architecture = architecture;
            }
            if target.arm().architecture == 0 {
                return Err(format!("target cpu {cpu}"));
            }
            target.arm_set_atomic();
        }
        let requested = target.options.abi.clone();
        if !requested.is_empty() && !target.arm_set_abi(&requested) {
            return Err(format!("target abi {requested}"));
        }
        Ok(target)
    }

    fn arm(&self) -> &ArmTarget {
        match &self.family {
            TargetFamily::Arm(state) => state,
            _ => unreachable!(),
        }
    }

    fn arm_mut(&mut self) -> &mut ArmTarget {
        match &mut self.family {
            TargetFamily::Arm(state) => state,
            _ => unreachable!(),
        }
    }

    fn arm_set_abi(&mut self, name: &str) -> bool {
        match name {
            "apcs-gnu" | "aapcs16" => {
                let width = if name == "aapcs16" { 64 } else { 32 };
                self.suitable_align = width;
                self.wchar_type = IntegerType::Int;
            }
            "aapcs" | "aapcs-vfp" | "aapcs-linux" => {
                self.suitable_align = 64;
                self.wchar_type = IntegerType::UnsignedInt;
            }
            _ => return false,
        }
        self.arm_mut().abi = name.to_string();
        true
    }

    fn arm_set_atomic(&mut self) {
        let linux = self.triple.os == "linux";
        let state = self.arm();
        let (profile, version) = (state.profile(), state.version());
        if profile == 'M' {
            self.max_atomic_promote_width = 32;
            if version >= 7 {
                self.max_atomic_inline_width = 32;
            }
        } else {
            self.max_atomic_promote_width = 64;
            if linux || version >= 6 {
                self.max_atomic_inline_width = 64;
            }
        }
    }

    fn select_features(&mut self) -> Result<(), String> {
        let written = self.options.features.clone();
        for feature in &written {
            if !feature.starts_with('+') && !feature.starts_with('-') {
                return Err(format!("target feature {feature}"));
            }
        }
        let mut map: BTreeMap<String, bool> = BTreeMap::new();
        let cpu = self.options.cpu.clone();
        match &self.family {
            TargetFamily::X86(_) => {
                if self.triple.arch == "x86_64" {
                    x86_set_feature(&mut map, "sse2", true);
                }
                let wide = x86_feature_index("64bit").unwrap_or(0) as u16;
                if let Some(processor) = X86_PROCESSORS.iter().find(|item| item.name == cpu) {
                    for index in processor.features {
                        let name = X86_FEATURES[*index as usize];
                        if *index != wide && !name.is_empty() {
                            x86_set_feature(&mut map, name, true);
                        }
                    }
                }
                let mut updated: Vec<String> = Vec::new();
                for feature in &written {
                    if feature == "+general-regs-only" {
                        updated.extend(["-x87", "-mmx", "-sse"].map(str::to_string));
                    } else if feature == "+apxf" || feature == "-apxf" {
                        let sign = &feature[..1];
                        for part in [
                            "egpr",
                            "push2pop2",
                            "ppx",
                            "ndd",
                            "ccmp",
                            "nf",
                            "zu",
                            "jmpabs",
                        ] {
                            updated.push(format!("{sign}{part}"));
                        }
                    } else {
                        updated.push(feature.clone());
                    }
                }
                for feature in &updated {
                    x86_set_feature(&mut map, &feature[1..], feature.starts_with('+'));
                }
                let written_has = |name: &str| updated.iter().any(|item| item == name);
                for (trigger, implied, blocker) in [
                    ("sse4.2", "popcnt", "-popcnt"),
                    ("sse", "mmx", "-mmx"),
                    ("avx", "xsave", "-xsave"),
                    ("sse4.2", "crc32", "-crc32"),
                ] {
                    if map.get(trigger) == Some(&true) && !written_has(blocker) {
                        map.insert(implied.to_string(), true);
                    }
                }
            }
            TargetFamily::AArch64(_) => {
                for feature in &written {
                    let name = &feature[1..];
                    let enabled = feature.starts_with('+');
                    map.insert(name.to_string(), enabled);
                    if name == "v8a" {
                        if enabled {
                            map.insert("fp-armv8".to_string(), true);
                            map.insert("neon".to_string(), true);
                        }
                    } else if (name.starts_with("v8") || name.starts_with("v9"))
                        && name.ends_with(['a', 'r'])
                    {
                        return Err(format!("target feature {feature}"));
                    }
                }
            }
            TargetFamily::Arm(state) => {
                let architecture = arm_parse_architecture(self.triple.arch_name());
                let mut defaults: Vec<&'static str> = Vec::new();
                arm_fpu_features(arm_default_fpu(&cpu, architecture), &mut defaults);
                arm_extension_features(arm_default_extensions(&cpu, architecture), &mut defaults);
                for feature in defaults {
                    if let Some(name) = feature.strip_prefix('+') {
                        map.insert(name.to_string(), true);
                    }
                }
                map.insert("thumb-mode".to_string(), state.thumb);
                for feature in &written {
                    let fixed = match feature.as_str() {
                        "+soft-float-abi" => continue,
                        "+arm" => "-thumb-mode",
                        "+thumb" => "+thumb-mode",
                        other => other,
                    };
                    map.insert(fixed[1..].to_string(), fixed.starts_with('+'));
                }
            }
        }
        let features: Vec<String> = map
            .iter()
            .map(|(name, enabled)| format!("{}{name}", if *enabled { '+' } else { '-' }))
            .collect();
        match &mut self.family {
            TargetFamily::X86(state) => {
                for feature in &features {
                    let Some(name) = feature.strip_prefix('+') else {
                        continue;
                    };
                    state.features.insert(name.to_string());
                    if let Some((_, level)) =
                        X86_SSE_LEVELS.iter().find(|(item, _)| item == feature)
                    {
                        state.sse_level = state.sse_level.max(*level);
                    }
                    let xop = match name {
                        "xop" => 3,
                        "fma4" => 2,
                        "sse4a" => 1,
                        _ => 0,
                    };
                    state.xop_level = state.xop_level.max(xop);
                }
                self.has_float16 = state.sse_level >= 2;
                if self.triple.arch == "x86" && state.features.contains("cx8") {
                    self.max_atomic_inline_width = 64;
                }
                if self.triple.arch == "x86_64" && state.features.contains("cx16") {
                    self.max_atomic_inline_width = 128;
                }
            }
            TargetFamily::AArch64(state) => {
                let mut no_floating_point = false;
                let mut no_neon = false;
                let mut no_scalable_vector = false;
                for feature in &features {
                    match feature.as_str() {
                        "-fp-armv8" => no_floating_point = true,
                        "-neon" => no_neon = true,
                        "-sve" => no_scalable_vector = true,
                        "-fmv" => state.multi_versioning = false,
                        "+strict-align" => state.unaligned_access = false,
                        _ => {}
                    }
                    if let Some((_, floating_point, flags)) = AARCH64_FEATURE_EFFECTS
                        .iter()
                        .find(|(name, _, _)| name == feature)
                    {
                        state.floating_point |= floating_point;
                        state.flags.extend(flags.iter().copied());
                    }
                }
                if features.iter().any(|feature| feature == "-d128") {
                    state.flags.remove("d128");
                }
                if no_floating_point {
                    state.floating_point &=
                        !(AARCH64_FLOATING_POINT | AARCH64_NEON | AARCH64_SCALABLE_VECTOR);
                }
                if no_neon {
                    state.floating_point &= !(AARCH64_NEON | AARCH64_SCALABLE_VECTOR);
                }
                if no_scalable_vector {
                    state.floating_point &= !AARCH64_SCALABLE_VECTOR;
                }
            }
            TargetFamily::Arm(state) => {
                self.has_float16 = true;
                for feature in &features {
                    match feature.as_str() {
                        "+soft-float" => state.soft_float = true,
                        "+vfp2sp" | "+vfp2" => {
                            state.floating_point |= ARM_FLOATING_POINT_VFP2;
                            state.hardware_floating_point |= ARM_HARDWARE_SINGLE;
                            if feature == "+vfp2" {
                                state.hardware_floating_point |= ARM_HARDWARE_DOUBLE;
                            }
                        }
                        "+vfp3sp" | "+vfp3d16sp" | "+vfp3" | "+vfp3d16" => {
                            state.floating_point |= ARM_FLOATING_POINT_VFP3;
                            state.hardware_floating_point |= ARM_HARDWARE_SINGLE;
                            if feature == "+vfp3" || feature == "+vfp3d16" {
                                state.hardware_floating_point |= ARM_HARDWARE_DOUBLE;
                            }
                        }
                        "+vfp4sp" | "+vfp4d16sp" | "+vfp4" | "+vfp4d16" => {
                            state.floating_point |= ARM_FLOATING_POINT_VFP4;
                            state.hardware_floating_point |=
                                ARM_HARDWARE_SINGLE | ARM_HARDWARE_HALF;
                            if feature == "+vfp4" || feature == "+vfp4d16" {
                                state.hardware_floating_point |= ARM_HARDWARE_DOUBLE;
                            }
                        }
                        "+fp-armv8sp" | "+fp-armv8d16sp" | "+fp-armv8" | "+fp-armv8d16" => {
                            state.floating_point |= ARM_FLOATING_POINT_ARMV8;
                            state.hardware_floating_point |=
                                ARM_HARDWARE_SINGLE | ARM_HARDWARE_HALF;
                            if feature == "+fp-armv8" || feature == "+fp-armv8d16" {
                                state.hardware_floating_point |= ARM_HARDWARE_DOUBLE;
                            }
                        }
                        "+neon" => {
                            state.floating_point |= ARM_FLOATING_POINT_NEON;
                            state.hardware_floating_point |= ARM_HARDWARE_SINGLE;
                        }
                        "+hwdiv" => state.hardware_divide |= ARM_DIVIDE_THUMB,
                        "+hwdiv-arm" => state.hardware_divide |= ARM_DIVIDE_ARM,
                        "+crc" => state.crc = true,
                        "+sha2" => state.sha2 = true,
                        "+aes" => state.aes = true,
                        "+dsp" => state.dsp = true,
                        "+fp64" => state.hardware_floating_point |= ARM_HARDWARE_DOUBLE,
                        "+8msecext" => {
                            if state.cpu_profile() != "M" || state.version() != 8 {
                                return Err("target feature +8msecext".to_string());
                            }
                        }
                        "+strict-align" => state.unaligned_access = false,
                        "+fp16" => state.hardware_floating_point |= ARM_HARDWARE_HALF,
                        "+fullfp16" => state.fast_half = true,
                        "+dotprod" => state.dot_product = true,
                        "+mve" => state.vector_extension |= ARM_VECTOR_INTEGER,
                        "+mve.fp" => {
                            state.fast_half = true;
                            state.floating_point |= ARM_FLOATING_POINT_ARMV8;
                            state.vector_extension |= ARM_VECTOR_INTEGER | ARM_VECTOR_FLOAT;
                            state.hardware_floating_point |=
                                ARM_HARDWARE_SINGLE | ARM_HARDWARE_HALF;
                        }
                        "+i8mm" => state.matrix_multiply = true,
                        "+bf16" => state.bfloat16 = true,
                        "+pacbti" => {
                            state.pointer_authentication = true;
                            state.branch_target = true;
                        }
                        other => {
                            if let Some(digit) = other.strip_prefix("+cdecp") {
                                if let Ok(coprocessor) = digit.parse::<u32>() {
                                    if digit.len() == 1 && coprocessor <= 7 {
                                        state.coprocessor_mask |= 1 << coprocessor;
                                    }
                                }
                            }
                        }
                    }
                }
                let profile = state.profile();
                let kind = state.kind();
                state.load_exclusive = match state.version() {
                    6 if profile == 'M' => 0,
                    6 if kind == "ARMV6K" || kind == "ARMV6KZ" => {
                        ARM_EXCLUSIVE_DOUBLE
                            | ARM_EXCLUSIVE_WORD
                            | ARM_EXCLUSIVE_HALF
                            | ARM_EXCLUSIVE_BYTE
                    }
                    6 => ARM_EXCLUSIVE_WORD,
                    7 | 8 if profile == 'M' => {
                        ARM_EXCLUSIVE_WORD | ARM_EXCLUSIVE_HALF | ARM_EXCLUSIVE_BYTE
                    }
                    7..=9 => {
                        ARM_EXCLUSIVE_DOUBLE
                            | ARM_EXCLUSIVE_WORD
                            | ARM_EXCLUSIVE_HALF
                            | ARM_EXCLUSIVE_BYTE
                    }
                    _ => 0,
                };
            }
        }
        Ok(())
    }

    pub fn char_width(&self) -> u32 {
        8
    }

    pub fn type_width(&self, ty: IntegerType) -> u32 {
        match ty {
            IntegerType::SignedChar | IntegerType::UnsignedChar => self.char_width(),
            IntegerType::Short | IntegerType::UnsignedShort => self.short_width,
            IntegerType::Int | IntegerType::UnsignedInt => self.int_width,
            IntegerType::Long | IntegerType::UnsignedLong => self.long_width,
            IntegerType::LongLong | IntegerType::UnsignedLongLong => self.long_long_width,
        }
    }

    pub fn type_constant_suffix(&self, ty: IntegerType) -> &'static str {
        match ty {
            IntegerType::SignedChar | IntegerType::Short | IntegerType::Int => "",
            IntegerType::Long => "L",
            IntegerType::LongLong => "LL",
            IntegerType::UnsignedChar if self.char_width() < self.int_width => "",
            IntegerType::UnsignedChar | IntegerType::UnsignedShort
                if self.short_width < self.int_width =>
            {
                ""
            }
            IntegerType::UnsignedChar | IntegerType::UnsignedShort | IntegerType::UnsignedInt => {
                "U"
            }
            IntegerType::UnsignedLong => "UL",
            IntegerType::UnsignedLongLong => "ULL",
        }
    }

    pub fn least_integer_type(&self, width: u32, signed: bool) -> Option<IntegerType> {
        let pick = |signed_type: IntegerType| {
            if signed {
                signed_type
            } else {
                signed_type.unsigned_counterpart()
            }
        };
        if self.char_width() >= width {
            Some(pick(IntegerType::SignedChar))
        } else if self.short_width >= width {
            Some(pick(IntegerType::Short))
        } else if self.int_width >= width {
            Some(pick(IntegerType::Int))
        } else if self.long_width >= width {
            Some(pick(IntegerType::Long))
        } else if self.long_long_width >= width {
            Some(pick(IntegerType::LongLong))
        } else {
            None
        }
    }

    pub fn has_builtin_atomic(&self, size: u32, alignment: u32) -> bool {
        size <= alignment
            && size <= self.max_atomic_inline_width
            && (size <= self.char_width() || (size / self.char_width()).is_power_of_two())
    }

    pub fn has_int128(&self) -> bool {
        self.triple.arch == "x86_64" || self.pointer_width >= 64
    }

    pub fn max_bit_int_width(&self) -> u32 {
        match self.family {
            TargetFamily::X86(_) => MAXIMUM_INTEGER_BITS,
            _ => 128,
        }
    }

    pub fn interference_sizes(&self) -> (u32, u32) {
        match self.family {
            TargetFamily::AArch64(_) => (256, 64),
            _ => (64, 64),
        }
    }

    pub fn is_big_endian(&self) -> bool {
        !self.triple.is_little_endian()
    }

    pub fn wchar_signed(&self) -> bool {
        self.wchar_type.signed()
    }

    pub fn is_windows(&self) -> bool {
        self.triple.is_windows()
    }

    pub fn has_strict_floating_point(&self) -> bool {
        matches!(self.family, TargetFamily::X86(_) | TargetFamily::AArch64(_))
    }

    pub fn supports_cpu_is(&self) -> bool {
        matches!(self.family, TargetFamily::X86(_))
    }

    pub fn supports_cpu_init(&self) -> bool {
        matches!(self.family, TargetFamily::X86(_))
    }

    pub fn supports_cpu_supports(&self) -> bool {
        matches!(self.family, TargetFamily::X86(_) | TargetFamily::AArch64(_))
    }

    pub fn floating_point_evaluation_method(&self) -> u32 {
        match &self.family {
            TargetFamily::X86(state) if state.sse_level == 0 => 2,
            _ => 0,
        }
    }

    pub fn is_target_architecture(&self, name: &str) -> bool {
        let wanted = Triple::parse(&format!("{}--", name.to_ascii_lowercase()));
        let sub_architecture_matches =
            wanted.sub_arch.is_empty() || wanted.sub_arch == self.triple.sub_arch;
        if self.triple.is_thumb()
            && sub_architecture_matches
            && ((self.triple.arch == "thumb" && wanted.arch == "arm")
                || (self.triple.arch == "thumbeb" && wanted.arch == "armeb"))
        {
            return true;
        }
        sub_architecture_matches && wanted.arch == self.triple.arch
    }

    pub fn is_target_vendor(&self, name: &str) -> bool {
        let vendor = match self.triple.vendor_name() {
            "" => "unknown",
            other => other,
        };
        vendor.eq_ignore_ascii_case(name)
    }

    pub fn is_target_system(&self, name: &str) -> bool {
        let wanted = Triple::parse(&format!("unknown-unknown-{}", name.to_ascii_lowercase()));
        if wanted.os == "darwin" {
            return self.triple.is_darwin();
        }
        self.triple.os == wanted.os
    }

    pub fn is_target_environment(&self, name: &str) -> bool {
        let wanted = Triple::parse(&format!("---{}", name.to_ascii_lowercase()));
        if wanted.environment == "unknown" && wanted.environment_name() != "unknown" {
            return false;
        }
        self.triple.environment == wanted.environment
    }

    fn target_defines(
        &self,
        language: &LanguageOptions,
        posix_threads: bool,
        builder: &mut MacroBuilder,
    ) {
        match &self.family {
            TargetFamily::X86(state) => self.x86_defines(state, language, builder),
            TargetFamily::AArch64(state) => self.aarch64_defines(state, builder),
            TargetFamily::Arm(state) => self.arm_defines(state, builder),
        }
        if self.triple.os == "linux" {
            builder.define_standard("unix", language.gnu);
            builder.define_standard("linux", language.gnu);
            if self.triple.is_android() {
                builder.define("__ANDROID__", "1");
                let major = self.triple.environment_version();
                if major != 0 {
                    builder.define("__ANDROID_MIN_SDK_VERSION__", &major.to_string());
                    builder.define("__ANDROID_API__", "__ANDROID_MIN_SDK_VERSION__");
                }
            } else {
                builder.define_one("__gnu_linux__");
            }
            if posix_threads {
                builder.define_one("_REENTRANT");
            }
            if self.has_float128 {
                builder.define_one("__FLOAT128__");
            }
            if self.triple.is_time64_abi() {
                builder.define("_FILE_OFFSET_BITS", "64");
                builder.define("_TIME_BITS", "64");
            }
        }
    }

    fn code_model(&self) -> String {
        match self.options.code_model.as_str() {
            "default" => "small".to_string(),
            other => other.to_string(),
        }
    }

    fn x86_defines(
        &self,
        state: &X86Target,
        language: &LanguageOptions,
        builder: &mut MacroBuilder,
    ) {
        let has = |name: &str| state.features.contains(name);
        builder.define_one("__GCC_ASM_FLAG_OUTPUTS__");
        builder.define_one(&format!("__code_model_{}__", self.code_model()));
        if self.triple.arch == "x86_64" {
            builder.define_one("__amd64__");
            builder.define_one("__amd64");
            builder.define_one("__x86_64");
            builder.define_one("__x86_64__");
            if self.triple.arch_name() == "x86_64h" {
                builder.define_one("__x86_64h");
                builder.define_one("__x86_64h__");
            }
        } else {
            builder.define_standard("i386", language.gnu);
        }
        if self.triple.sub_arch == "lfi" {
            builder.define_one("__LFI__");
        }
        builder.define_one("__SEG_GS");
        builder.define_one("__SEG_FS");
        builder.define("__seg_gs", "__attribute__((address_space(256)))");
        builder.define("__seg_fs", "__attribute__((address_space(257)))");
        let kind = X86_CPU_KINDS[state.cpu];
        match kind {
            "i386" => builder.define_one("__tune_i386__"),
            "i486" | "WinChipC6" | "WinChip2" | "C3" => builder.define_processor("i486", true),
            "PentiumMMX" | "i586" | "Pentium" => {
                if kind == "PentiumMMX" {
                    builder.define_one("__pentium_mmx__");
                    builder.define_one("__tune_pentium_mmx__");
                }
                builder.define_processor("i586", true);
                builder.define_processor("pentium", true);
            }
            "Pentium3" | "PentiumM" | "Pentium2" | "C3_2" | "PentiumPro" | "i686" => {
                if matches!(kind, "Pentium3" | "PentiumM") {
                    builder.define_one("__tune_pentium3__");
                }
                if matches!(kind, "Pentium3" | "PentiumM" | "Pentium2" | "C3_2") {
                    builder.define_one("__tune_pentium2__");
                }
                builder.define_processor("i686", true);
                builder.define_processor("pentiumpro", true);
            }
            "Pentium4" => builder.define_processor("pentium4", true),
            "Yonah" | "Prescott" | "Nocona" => builder.define_processor("nocona", true),
            "Core2" | "Penryn" => builder.define_processor("core2", true),
            "Bonnell" => builder.define_processor("atom", true),
            "Silvermont" => builder.define_processor("slm", true),
            "Goldmont" => builder.define_processor("goldmont", true),
            "GoldmontPlus" => builder.define_processor("goldmont_plus", true),
            "Tremont" => builder.define_processor("tremont", true),
            "Gracemont" | "Nehalem" | "Westmere" | "SandyBridge" | "IvyBridge" | "Haswell"
            | "Broadwell" | "SkylakeClient" | "SkylakeServer" | "Cascadelake" | "Cooperlake"
            | "Cannonlake" | "IcelakeClient" | "Rocketlake" | "IcelakeServer" | "Tigerlake"
            | "SapphireRapids" | "Alderlake" | "Raptorlake" | "Meteorlake" | "Arrowlake"
            | "ArrowlakeS" | "Lunarlake" | "Pantherlake" | "Wildcatlake" | "Novalake"
            | "Sierraforest" | "Grandridge" | "Graniterapids" | "GraniterapidsD"
            | "Emeraldrapids" | "Clearwaterforest" | "Diamondrapids" => {
                builder.define_processor("corei7", true)
            }
            "KNL" => builder.define_processor("knl", true),
            "Lakemont" => {
                builder.define_processor("i586", false);
                builder.define_processor("pentium", false);
                builder.define_one("__tune_lakemont__");
            }
            "K6_2" | "K6_3" | "K6" => {
                if kind == "K6_2" {
                    builder.define_one("__k6_2__");
                    builder.define_one("__tune_k6_2__");
                }
                if kind == "K6_3" {
                    builder.define_one("__k6_3__");
                    builder.define_one("__tune_k6_3__");
                }
                builder.define_processor("k6", true);
            }
            "Athlon" | "AthlonXP" => {
                builder.define_processor("athlon", true);
                if state.sse_level != 0 {
                    builder.define_one("__athlon_sse__");
                    builder.define_one("__tune_athlon_sse__");
                }
            }
            "K8" | "K8SSE3" | "x86_64" => builder.define_processor("k8", true),
            "AMDFAM10" => builder.define_processor("amdfam10", true),
            "BTVER1" => builder.define_processor("btver1", true),
            "BTVER2" => builder.define_processor("btver2", true),
            "BDVER1" => builder.define_processor("bdver1", true),
            "BDVER2" => builder.define_processor("bdver2", true),
            "BDVER3" => builder.define_processor("bdver3", true),
            "BDVER4" => builder.define_processor("bdver4", true),
            "ZNVER1" => builder.define_processor("znver1", true),
            "ZNVER2" => builder.define_processor("znver2", true),
            "ZNVER3" => builder.define_processor("znver3", true),
            "ZNVER4" => builder.define_processor("znver4", true),
            "ZNVER5" => builder.define_processor("znver5", true),
            "ZNVER6" => builder.define_processor("znver6", true),
            "Geode" => builder.define_processor("geode", true),
            "C86_4G_M4" => builder.define_processor("c86_4g_m4", true),
            "C86_4G_M6" => builder.define_processor("c86_4g_m6", true),
            "C86_4G_M7" => builder.define_processor("c86_4g_m7", true),
            "C86_4G_M8" => builder.define_processor("c86_4g_m8", true),
            _ => {}
        }
        builder.define("__REGISTER_PREFIX__", "");
        builder.define_one("__NO_MATH_INLINES");
        for (feature, name) in X86_FEATURE_MACROS_FIRST {
            if has(feature) {
                builder.define_one(name);
            }
        }
        if has("sahf") || self.triple.arch == "x86" {
            builder.define_one("__LAHF_SAHF__");
        }
        for (feature, name) in X86_FEATURE_MACROS_SECOND {
            if has(feature) {
                builder.define_one(name);
            }
        }
        if state.xop_level >= 3 {
            builder.define_one("__XOP__");
        }
        if state.xop_level >= 2 {
            builder.define_one("__FMA4__");
        }
        if state.xop_level >= 1 {
            builder.define_one("__SSE4A__");
        }
        for (feature, name) in X86_FEATURE_MACROS_THIRD {
            if has(feature) {
                builder.define_one(name);
            }
        }
        if has("avx10.1") {
            builder.define_one("__AVX10_1__");
            builder.define_one("__AVX10_1_512__");
        }
        if has("avx10.2") {
            builder.define_one("__AVX10_2__");
            builder.define_one("__AVX10_2_512__");
        }
        for (feature, name) in X86_FEATURE_MACROS_FOURTH {
            if has(feature) {
                builder.define_one(name);
            }
        }
        if [
            "egpr",
            "push2pop2",
            "ppx",
            "ndd",
            "ccmp",
            "nf",
            "zu",
            "jmpabs",
        ]
        .iter()
        .all(|feature| has(feature))
        {
            builder.define_one("__APX_F__");
        }
        if has("egpr") && has("inline-asm-use-gpr32") {
            builder.define_one("__APX_INLINE_ASM_USE_GPR32__");
        }
        for (level, names) in X86_SSE_MACROS {
            if state.sse_level >= *level {
                for name in names.iter() {
                    builder.define_one(name);
                }
            }
        }
        if has("mmx") {
            builder.define_one("__MMX__");
        }
        if state.cpu >= x86_kind_index("i486") || state.cpu == 0 {
            builder.define_one("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_1");
            builder.define_one("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_2");
            builder.define_one("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_4");
        }
        if has("cx8") {
            builder.define_one("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_8");
        }
        if has("cx16") && self.triple.arch == "x86_64" {
            builder.define_one("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_16");
        }
        if self.has_float128 {
            builder.define("__SIZEOF_FLOAT128__", "16");
        }
    }

    fn aarch64_defines(&self, state: &AArch64Target, builder: &mut MacroBuilder) {
        let has = |name: &str| state.flags.contains(name);
        let scalable = state.floating_point & AARCH64_SCALABLE_VECTOR != 0;
        let neon = state.floating_point & AARCH64_NEON != 0;
        if self.is_big_endian() {
            builder.define_one("__AARCH64EB__");
            builder.define_one("__AARCH_BIG_ENDIAN");
            builder.define_one("__ARM_BIG_ENDIAN");
        } else {
            builder.define_one("__AARCH64EL__");
        }
        builder.define_one("__aarch64__");
        if self.triple.sub_arch == "lfi" {
            builder.define_one("__LFI__");
        }
        builder.define_one("__GCC_ASM_FLAG_OUTPUTS__");
        builder.define_one(&format!(
            "__AARCH64_CMODEL_{}__",
            self.code_model().to_ascii_uppercase()
        ));
        builder.define(
            "__ARM_ACLE_VERSION(year, quarter, patch)",
            "(100 * (year) + 10 * (quarter) + (patch))",
        );
        builder.define("__ARM_ACLE", "202420");
        builder.define("__FUNCTION_MULTI_VERSIONING_SUPPORT_LEVEL", "202430");
        builder.define("__ARM_ARCH", "8");
        builder.define("__ARM_ARCH_PROFILE", "'A'");
        builder.define("__ARM_64BIT_STATE", "1");
        builder.define("__ARM_PCS_AAPCS64", "1");
        builder.define("__ARM_ARCH_ISA_A64", "1");
        builder.define("__ARM_FEATURE_CLZ", "1");
        builder.define("__ARM_FEATURE_FMA", "1");
        builder.define("__ARM_FEATURE_LDREX", "0xF");
        builder.define("__ARM_FEATURE_IDIV", "1");
        builder.define_one("__ARM_FEATURE_DIV");
        builder.define("__ARM_FEATURE_NUMERIC_MAXMIN", "1");
        builder.define("__ARM_FEATURE_DIRECTED_ROUNDING", "1");
        builder.define("__ARM_ALIGN_MAX_STACK_PWR", "4");
        builder.define("__ARM_STATE_ZA", "1");
        builder.define("__ARM_STATE_ZT0", "1");
        if state.floating_point & AARCH64_FLOATING_POINT != 0 {
            builder.define("__ARM_FP", "0xE");
        }
        builder.define("__ARM_FP16_FORMAT_IEEE", "1");
        builder.define("__ARM_FP16_ARGS", "1");
        builder.define("__ARM_NEON_SVE_BRIDGE", "1");
        builder.define(
            "__ARM_SIZEOF_WCHAR_T",
            &self.options.wchar_size_or(4).to_string(),
        );
        builder.define("__ARM_SIZEOF_MINIMAL_ENUM", "4");
        builder.define("__ARM_PREFETCH_RANGE", "1");
        if neon {
            builder.define("__ARM_NEON", "1");
            builder.define("__ARM_NEON_FP", "0xE");
        }
        if scalable {
            builder.define("__ARM_FEATURE_SVE", "1");
        }
        let sve2 = has("sve2");
        let conditional: [(bool, &str, &str); 61] = [
            (sve2, "__ARM_FEATURE_SVE2", "1"),
            (has("sve2p1"), "__ARM_FEATURE_SVE2p1", "1"),
            (sve2 && has("sve-aes"), "__ARM_FEATURE_SVE2_AES", "1"),
            (
                sve2 && has("sve-bitperm"),
                "__ARM_FEATURE_SVE2_BITPERM",
                "1",
            ),
            (sve2 && has("sve2-sha3"), "__ARM_FEATURE_SVE2_SHA3", "1"),
            (sve2 && has("sve2-sm4"), "__ARM_FEATURE_SVE2_SM4", "1"),
            (has("sve-b16b16"), "__ARM_FEATURE_SVE_B16B16", "1"),
            (has("sme"), "__ARM_FEATURE_SME", "1"),
            (has("sme"), "__ARM_FEATURE_LOCALLY_STREAMING", "1"),
            (has("sme2"), "__ARM_FEATURE_SME2", "1"),
            (has("sme2p1"), "__ARM_FEATURE_SME2p1", "1"),
            (has("sme-f16f16"), "__ARM_FEATURE_SME_F16F16", "1"),
            (has("sme-b16b16"), "__ARM_FEATURE_SME_B16B16", "1"),
            (has("fp8"), "__ARM_FEATURE_FP8", "1"),
            (has("fp8fma"), "__ARM_FEATURE_FP8FMA", "1"),
            (has("fp8dot2"), "__ARM_FEATURE_FP8DOT2", "1"),
            (has("fp8dot4"), "__ARM_FEATURE_FP8DOT4", "1"),
            (has("ssve-fp8dot2"), "__ARM_FEATURE_SSVE_FP8DOT2", "1"),
            (has("ssve-fp8dot4"), "__ARM_FEATURE_SSVE_FP8DOT4", "1"),
            (has("ssve-fp8fma"), "__ARM_FEATURE_SSVE_FP8FMA", "1"),
            (has("sme-f8f32"), "__ARM_FEATURE_SME_F8F32", "1"),
            (has("sme-f8f16"), "__ARM_FEATURE_SME_F8F16", "1"),
            (has("crc"), "__ARM_FEATURE_CRC32", "1"),
            (has("cssc"), "__ARM_FEATURE_CSSC", "1"),
            (has("rcpc3"), "__ARM_FEATURE_RCPC", "3"),
            (!has("rcpc3") && has("rcpc"), "__ARM_FEATURE_RCPC", "1"),
            (has("fprcvt"), "__ARM_FEATURE_FPRCVT", "1"),
            (has("f8f16mm"), "__ARM_FEATURE_F8F16MM", "1"),
            (has("f8f32mm"), "__ARM_FEATURE_F8F32MM", "1"),
            (has("sve-f16f32mm"), "__ARM_FEATURE_SVE_F16F32MM", "1"),
            (has("sve-bfscale"), "__ARM_FEATURE_SVE_BFSCALE", "1"),
            (has("sve-b16mm"), "__ARM_FEATURE_SVE_B16MM", "1"),
            (has("f16mm"), "__ARM_FEATURE_F16MM", "1"),
            (has("f16f32dot"), "__ARM_FEATURE_F16F32DOT", "1"),
            (has("f16f32mm"), "__ARM_FEATURE_F16F32MM", "1"),
            (has("sve-aes2"), "__ARM_FEATURE_SVE_AES2", "1"),
            (has("ssve-aes"), "__ARM_FEATURE_SSVE_AES", "1"),
            (has("sve2p2"), "__ARM_FEATURE_SVE2p2", "1"),
            (has("sve2p3"), "__ARM_FEATURE_SVE2p3", "1"),
            (has("sme2p2"), "__ARM_FEATURE_SME2p2", "1"),
            (has("sme2p3"), "__ARM_FEATURE_SME2p3", "1"),
            (
                state.multi_versioning,
                "__HAVE_FUNCTION_MULTI_VERSIONING",
                "1",
            ),
            (has("aes") && has("sha2"), "__ARM_FEATURE_CRYPTO", "1"),
            (has("aes"), "__ARM_FEATURE_AES", "1"),
            (has("sha2"), "__ARM_FEATURE_SHA2", "1"),
            (has("sha3"), "__ARM_FEATURE_SHA3", "1"),
            (has("sha3"), "__ARM_FEATURE_SHA512", "1"),
            (has("sm4"), "__ARM_FEATURE_SM3", "1"),
            (has("sm4"), "__ARM_FEATURE_SM4", "1"),
            (has("pauth"), "__ARM_FEATURE_PAUTH", "1"),
            (has("pauth-lr"), "__ARM_FEATURE_PAUTH_LR", "1"),
            (has("bti"), "__ARM_FEATURE_BTI", "1"),
            (state.unaligned_access, "__ARM_FEATURE_UNALIGNED", "1"),
            (
                neon && has("fullfp16"),
                "__ARM_FEATURE_FP16_VECTOR_ARITHMETIC",
                "1",
            ),
            (has("fullfp16"), "__ARM_FEATURE_FP16_SCALAR_ARITHMETIC", "1"),
            (has("dotprod"), "__ARM_FEATURE_DOTPROD", "1"),
            (has("mte"), "__ARM_FEATURE_MEMORY_TAGGING", "1"),
            (has("i8mm"), "__ARM_FEATURE_MATMUL_INT8", "1"),
            (has("lse"), "__ARM_FEATURE_ATOMICS", "1"),
            (has("bf16"), "__ARM_FEATURE_BF16", "1"),
            (has("bf16"), "__ARM_FEATURE_BF16_VECTOR_ARITHMETIC", "1"),
        ];
        for (enabled, name, value) in conditional {
            if enabled {
                builder.define(name, value);
            }
        }
        let later: [(bool, &str); 11] = [
            (has("bf16"), "__ARM_BF16_FORMAT_ALTERNATIVE"),
            (has("bf16"), "__ARM_FEATURE_BF16_SCALAR_ARITHMETIC"),
            (scalable && has("bf16"), "__ARM_FEATURE_SVE_BF16"),
            (scalable && has("f64mm"), "__ARM_FEATURE_SVE_MATMUL_FP64"),
            (scalable && has("f32mm"), "__ARM_FEATURE_SVE_MATMUL_FP32"),
            (scalable && has("i8mm"), "__ARM_FEATURE_SVE_MATMUL_INT8"),
            (neon && has("fp16fml"), "__ARM_FEATURE_FP16_FML"),
            (has("ls64"), "__ARM_FEATURE_LS64"),
            (has("rand"), "__ARM_FEATURE_RNG"),
            (has("mops"), "__ARM_FEATURE_MOPS"),
            (has("d128"), "__ARM_FEATURE_SYSREG128"),
        ];
        for (enabled, name) in later {
            if enabled {
                builder.define(name, "1");
            }
        }
        if has("gcs") {
            builder.define("__ARM_FEATURE_GCS", "1");
        }
        for width in ["1", "2", "4", "8", "16"] {
            builder.define_one(&format!("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_{width}"));
        }
        builder.define("__FP_FAST_FMA", "1");
        builder.define("__FP_FAST_FMAF", "1");
        if scalable {
            builder.define("__ARM_FEATURE_SVE_VECTOR_OPERATORS", "2");
        }
    }

    fn arm_defines(&self, state: &ArmTarget, builder: &mut MacroBuilder) {
        let version = state.version();
        let profile = state.profile();
        let cpu_profile = state.cpu_profile();
        let cpu_attribute = arm_cpu_attribute(state.architecture);
        let kind = state.kind();
        if self.is_big_endian() {
            builder.define_one("__ARMEB__");
            builder.define_one("__ARM_BIG_ENDIAN");
        } else {
            builder.define_one("__ARMEL__");
        }
        builder.define_one("__arm");
        builder.define_one("__arm__");
        builder.define("__REGISTER_PREFIX__", "");
        if !cpu_attribute.is_empty() {
            builder.define_one(&format!("__ARM_ARCH_{cpu_attribute}__"));
        }
        builder.define("__ARM_ARCH", &version.to_string());
        if version >= 8 {
            if state.sha2 && state.aes {
                builder.define("__ARM_FEATURE_CRYPTO", "1");
            }
            if state.sha2 {
                builder.define("__ARM_FEATURE_SHA2", "1");
            }
            if state.aes {
                builder.define("__ARM_FEATURE_AES", "1");
            }
            if state.crc {
                builder.define("__ARM_FEATURE_CRC32", "1");
            }
            builder.define("__ARM_FEATURE_NUMERIC_MAXMIN", "1");
            builder.define("__ARM_FEATURE_DIRECTED_ROUNDING", "1");
        }
        if cpu_profile.is_empty() || profile != 'M' {
            builder.define("__ARM_ARCH_ISA_ARM", "1");
        }
        if state.supports_thumb2() {
            builder.define("__ARM_ARCH_ISA_THUMB", "2");
        } else if state.supports_thumb() {
            builder.define("__ARM_ARCH_ISA_THUMB", "1");
        }
        builder.define("__ARM_32BIT_STATE", "1");
        if !cpu_profile.is_empty() {
            builder.define("__ARM_ARCH_PROFILE", &format!("'{cpu_profile}'"));
        }
        if state.unaligned_access {
            builder.define("__ARM_FEATURE_UNALIGNED", "1");
        }
        if state.load_exclusive != 0 {
            builder.define(
                "__ARM_FEATURE_LDREX",
                &format!("0x{:x}", state.load_exclusive),
            );
        }
        if version == 5 || (version == 6 && cpu_profile != "M") || version > 6 {
            builder.define("__ARM_FEATURE_CLZ", "1");
        }
        if state.hardware_floating_point != 0 {
            builder.define(
                "__ARM_FP",
                &format!("0x{:x}", state.hardware_floating_point),
            );
        }
        builder.define("__ARM_ACLE", "200");
        builder.define("__ARM_FP16_FORMAT_IEEE", "1");
        builder.define("__ARM_FP16_ARGS", "1");
        if version >= 7 && state.floating_point & ARM_FLOATING_POINT_VFP4 != 0 {
            builder.define("__ARM_FEATURE_FMA", "1");
        }
        if (5..=8).contains(&version) {
            builder.define_one("__THUMB_INTERWORK__");
        }
        if matches!(state.abi.as_str(), "aapcs" | "aapcs-linux" | "aapcs-vfp") {
            builder.define_one("__ARM_EABI__");
            builder.define("__ARM_PCS", "1");
        }
        if (!state.soft_float && !state.soft_float_abi)
            || state.abi == "aapcs-vfp"
            || state.abi == "aapcs16"
        {
            builder.define("__ARM_PCS_VFP", "1");
        }
        if state.soft_float || (state.soft_float_abi && state.floating_point == 0) {
            builder.define_one("__SOFTFP__");
        }
        let all = 0b1111u32;
        let coprocessor = match kind {
            "ARMV4" | "ARMV4T" => {
                if state.thumb {
                    0
                } else {
                    1
                }
            }
            "ARMV5T" => {
                if state.thumb {
                    0
                } else {
                    0b11
                }
            }
            "ARMV5TE" | "ARMV5TEJ" => {
                if state.thumb {
                    0
                } else {
                    0b111
                }
            }
            "ARMV6" | "ARMV6K" | "ARMV6KZ" | "ARMV6T2" => {
                if !state.thumb || kind == "ARMV6T2" {
                    all
                } else {
                    0
                }
            }
            "ARMV7A" | "ARMV7R" | "ARMV7M" | "ARMV7S" | "ARMV7EM" => all,
            "ARMV8MMainline" | "ARMV8_1MMainline" => all,
            other if other.starts_with("ARMV8") || other.starts_with("ARMV9") => {
                if matches!(other, "ARMV8MBaseline") {
                    0
                } else {
                    0b101
                }
            }
            _ => 0,
        };
        builder.define("__ARM_FEATURE_COPROC", &format!("0x{coprocessor:x}"));
        if kind == "XSCALE" {
            builder.define_one("__XSCALE__");
        }
        if state.thumb {
            builder.define_one("__THUMBEL__");
            builder.define_one("__thumb__");
            if state.supports_thumb2() {
                builder.define_one("__thumb2__");
            }
        }
        if (cpu_profile != "M" && version >= 6) || (cpu_profile == "M" && state.dsp) {
            builder.define("__ARM_FEATURE_SIMD32", "1");
        }
        if (state.hardware_divide & ARM_DIVIDE_THUMB != 0 && state.thumb)
            || (state.hardware_divide & ARM_DIVIDE_ARM != 0 && !state.thumb)
        {
            builder.define("__ARM_FEATURE_IDIV", "1");
            builder.define("__ARM_ARCH_EXT_IDIV__", "1");
        }
        builder.define_one("__APCS_32__");
        builder.define_one("__VFP_FP__");
        if state.floating_point != 0 {
            if state.floating_point & ARM_FLOATING_POINT_VFP2 != 0 {
                builder.define_one("__ARM_VFPV2__");
            }
            if state.floating_point & ARM_FLOATING_POINT_VFP3 != 0 {
                builder.define_one("__ARM_VFPV3__");
            }
            if state.floating_point & ARM_FLOATING_POINT_VFP4 != 0 {
                builder.define_one("__ARM_VFPV4__");
            }
            if state.floating_point & ARM_FLOATING_POINT_ARMV8 != 0 {
                builder.define_one("__ARM_FPV5__");
            }
        }
        if state.floating_point & ARM_FLOATING_POINT_NEON != 0 && !state.soft_float && version >= 7
        {
            builder.define("__ARM_NEON", "1");
            builder.define_one("__ARM_NEON__");
            builder.define(
                "__ARM_NEON_FP",
                &format!(
                    "0x{:x}",
                    state.hardware_floating_point & !ARM_HARDWARE_DOUBLE
                ),
            );
        }
        if state.has_vector_extension() {
            builder.define(
                "__ARM_FEATURE_MVE",
                if state.vector_extension & ARM_VECTOR_FLOAT != 0 {
                    "3"
                } else {
                    "1"
                },
            );
        }
        if state.coprocessor_mask != 0 {
            builder.define("__ARM_FEATURE_CDE", "1");
            builder.define(
                "__ARM_FEATURE_CDE_COPROC",
                &format!("0x{:x}", state.coprocessor_mask),
            );
        }
        builder.define(
            "__ARM_SIZEOF_WCHAR_T",
            &self.options.wchar_size_or(4).to_string(),
        );
        builder.define("__ARM_SIZEOF_MINIMAL_ENUM", "4");
        if version == 8 && profile == 'M' {
            builder.define("__ARM_FEATURE_CMSE", "1");
        }
        if version >= 6 && cpu_attribute != "6M" && cpu_attribute != "8M_BASE" {
            for width in ["1", "2", "4", "8"] {
                builder.define_one(&format!("__GCC_HAVE_SYNC_COMPARE_AND_SWAP_{width}"));
            }
        }
        if state.dsp {
            builder.define("__ARM_FEATURE_DSP", "1");
        }
        let saturating = (version == 6 && cpu_profile != "M") || version > 6;
        if saturating {
            builder.define("__ARM_FEATURE_SAT", "1");
        }
        if state.dsp || saturating {
            builder.define("__ARM_FEATURE_QBIT", "1");
        }
        if state.floating_point & ARM_FLOATING_POINT_NEON != 0 && state.fast_half {
            builder.define("__ARM_FEATURE_FP16_VECTOR_ARITHMETIC", "1");
        }
        if state.fast_half {
            builder.define("__ARM_FEATURE_FP16_SCALAR_ARITHMETIC", "1");
        }
        if state.dot_product {
            builder.define("__ARM_FEATURE_DOTPROD", "1");
        }
        if state.matrix_multiply {
            builder.define("__ARM_FEATURE_MATMUL_INT8", "1");
        }
        if state.pointer_authentication {
            builder.define("__ARM_FEATURE_PAUTH", "1");
        }
        if state.branch_target {
            builder.define("__ARM_FEATURE_BTI", "1");
        }
        if state.bfloat16 {
            builder.define("__ARM_FEATURE_BF16", "1");
            builder.define("__ARM_FEATURE_BF16_VECTOR_ARITHMETIC", "1");
            builder.define("__ARM_BF16_FORMAT_ALTERNATIVE", "1");
        }
        match kind {
            "ARMV8_1A" | "ARMV8_2A" => builder.define("__ARM_FEATURE_QRDMX", "1"),
            other
                if (other.starts_with("ARMV8_") && other.ends_with('A'))
                    || other.starts_with("ARMV9") =>
            {
                builder.define("__ARM_FEATURE_COMPLEX", "1");
                builder.define("__ARM_FEATURE_QRDMX", "1");
            }
            _ => {}
        }
    }
}

impl TargetOptions {
    fn wchar_size_or(&self, fallback: u32) -> u32 {
        if self.wchar_size == 0 {
            fallback
        } else {
            self.wchar_size
        }
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
    pub embed_directories: Vec<String>,
    pub spell_checking: bool,
    pub fast_math: bool,
    pub math_errno: bool,
    pub no_honor_infinities: bool,
    pub no_honor_nans: bool,
    pub gnu_inline: Option<bool>,
    pub exceptions: bool,
    pub posix_threads: bool,
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
            embed_directories: Vec::new(),
            spell_checking: true,
            fast_math: false,
            math_errno: false,
            no_honor_infinities: false,
            no_honor_nans: false,
            gnu_inline: None,
            exceptions: false,
            posix_threads: false,
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
    pub linker_comments: Vec<(String, Vec<u8>)>,
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

#[derive(Default)]
struct EmbedParameters {
    limit: Option<u64>,
    offset: Option<u64>,
    prefix: Option<Vec<Token>>,
    suffix: Option<Vec<Token>>,
    if_empty: Option<Vec<Token>>,
    unrecognized: usize,
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
    evaluated_defined: bool,
    assume_nonnull: Option<Place>,
    shadow_checked: BTreeSet<(String, String)>,
    arc_cf_audited: Option<Place>,
    directive_read_before: bool,
    last_expression_simple: bool,
    seen_copyright_comment: bool,
    linker_comments: Vec<(String, Vec<u8>)>,
    macro_annotations: BTreeMap<Rc<[u8]>, MacroAnnotations>,
}

#[derive(Clone, Debug, Default)]
struct MacroAnnotations {
    deprecated: Option<(Vec<u8>, Place)>,
    restricted: Option<(Vec<u8>, Place)>,
    final_place: Option<Place>,
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
            evaluated_defined: false,
            assume_nonnull: None,
            shadow_checked: BTreeSet::new(),
            arc_cf_audited: None,
            directive_read_before: true,
            last_expression_simple: false,
            seen_copyright_comment: false,
            linker_comments: Vec::new(),
            macro_annotations: BTreeMap::new(),
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
        } else if message == Message::UndefPrefixIdentifier {
            self.engine
                .option_text(message, severity, as_error)
                .map(|text| format!("{text}={}", self.options.warnings.undef_prefixes.join(",")))
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
        let raw = matches!(self.stack.last(), Some(Source::File(file)) if file.lexer.raw);
        if from_file && !raw && self.poisoned.contains(&token.spelling) {
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
        self.emit_macro_expansion_warnings(&identifier, false);
        if let Some(builtin) = definition.builtin {
            return self.expand_builtin(identifier, builtin);
        }
        let mut arguments = None;
        let mut end_site = identifier.site;
        if definition.function_like {
            self.argument_macro_name = Some(identifier.clone());
            self.in_macro_arguments = true;
            let read = self.read_macro_arguments(&identifier, &definition, &mut end_site);
            self.in_macro_arguments = false;
            self.argument_macro_name = None;
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

    fn regroup_braced_arguments(
        &mut self,
        groups: &[Vec<Token>],
    ) -> (Option<Vec<Vec<Token>>>, bool) {
        let mut brackets: Vec<TokenKind> = Vec::new();
        for token in groups.iter().flatten() {
            let opener = match token.kind {
                TokenKind::LeftParen | TokenKind::LeftBrace => {
                    brackets.push(token.kind);
                    continue;
                }
                TokenKind::RightParen => TokenKind::LeftParen,
                TokenKind::RightBrace => TokenKind::LeftBrace,
                _ => continue,
            };
            if brackets.pop() != Some(opener) {
                return (None, false);
            }
        }
        if !brackets.is_empty() {
            return (None, false);
        }
        let mut braces = 0usize;
        let mut current: Vec<Token> = Vec::new();
        let mut merged = false;
        let mut hints = 0usize;
        let mut initializer_lists = 0usize;
        let mut result: Vec<Vec<Token>> = Vec::new();
        for group in groups {
            for token in group {
                match token.kind {
                    TokenKind::LeftBrace => braces += 1,
                    TokenKind::RightBrace => braces -= 1,
                    _ => {}
                }
                current.push(token.clone());
            }
            if braces != 0 {
                merged = true;
                let site = current.last().map(|token| token.site).unwrap_or_default();
                current.push(self.make_token(TokenKind::Comma, b",".to_vec(), (site, site)));
                continue;
            }
            let mut argument = std::mem::take(&mut current);
            if merged {
                if argument
                    .first()
                    .is_some_and(|first| first.kind == TokenKind::LeftBrace)
                {
                    initializer_lists += 1;
                }
                let start = argument.first().map(|token| token.site).unwrap_or_default();
                let end = argument.last().map(|token| token.site).unwrap_or_default();
                argument.insert(
                    0,
                    self.make_token(TokenKind::LeftParen, Vec::new(), (start, start)),
                );
                argument.push(self.make_token(TokenKind::RightParen, Vec::new(), (end, end)));
                hints += 1;
            }
            result.push(argument);
            merged = false;
        }
        if hints > 0 && initializer_lists == 0 {
            (Some(result), false)
        } else {
            (None, initializer_lists > 0)
        }
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
        let mut actual_count = groups.len();
        if !variadic && actual_count > parameter_count {
            let place = too_many_place.unwrap_or(name.site);
            self.report(Message::TooManyArguments, place, Vec::new());
            self.report(
                Message::MacroDefinedHere,
                definition.defined_at,
                vec![quoted(&definition.name)],
            );
            let (regrouped, initializer_list) = self.regroup_braced_arguments(&groups);
            let Some(regrouped) = regrouped else {
                if initializer_list {
                    self.report_token(Message::InitializerListAtArgumentStart, name, Vec::new());
                }
                return Err(None);
            };
            if regrouped.len() != parameter_count {
                return Err(None);
            }
            self.report_token(Message::SuggestParenthesesForMacro, name, Vec::new());
            groups = regrouped;
            actual_count = groups.len();
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
        if let Some(place) = self.arc_cf_audited.take() {
            self.report(
                Message::AuditEndOfFile,
                place,
                vec!["arc_cf_code_audited".to_string()],
            );
        }
        if let Some(place) = self.assume_nonnull.take() {
            self.report(
                Message::AuditEndOfFile,
                place,
                vec!["assume_nonnull".to_string()],
            );
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
        lexer.base_keep_comments = self.options.keep_comments;
        lexer.keep_whitespace = self.language.traditional;
        lexer.base_keep_whitespace = self.language.traditional;
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

    fn diagnose_header_shadowing(
        &mut self,
        name: &str,
        found: &str,
        site: Place,
        angled: bool,
        start_index: Option<usize>,
        main_index: Option<usize>,
    ) {
        if self.message_ignored(Message::HeaderShadowing, site) {
            return;
        }
        let quoted_count = self.options.quoted_directories.len();
        let character_of = |preprocessor: &Self, index: usize| {
            if index < quoted_count {
                FileCharacter::User
            } else {
                preprocessor.options.directories[index - quoted_count].character
            }
        };
        if let Some(index) = main_index {
            if character_of(self, index) != FileCharacter::User {
                return;
            }
        }
        if !self
            .shadow_checked
            .insert((name.to_string(), self.canonical_key(found)))
        {
            return;
        }
        let total = quoted_count + self.options.directories.len();
        let mut index = if let Some(main) = main_index {
            main + 1
        } else if let Some(start) = start_index {
            start
        } else if angled {
            quoted_count
        } else {
            0
        };
        while index < total {
            if character_of(self, index) == FileCharacter::User {
                let directory = if index < quoted_count {
                    self.options.quoted_directories[index].clone()
                } else {
                    self.options.directories[index - quoted_count].path.clone()
                };
                let candidate = Self::join_path(&directory, name);
                let full =
                    if candidate.starts_with('/') || self.options.working_directory.is_empty() {
                        candidate.clone()
                    } else {
                        Self::join_path(&self.options.working_directory, &candidate)
                    };
                if self.canonical_key(&full) != self.canonical_key(found)
                    && self.read_file(&full).is_some()
                {
                    let chosen = Self::directory_of(found);
                    self.report(
                        Message::HeaderShadowing,
                        site,
                        vec![name.to_string(), chosen, directory],
                    );
                    return;
                }
            }
            index += 1;
        }
    }

    fn lookup_file(
        &mut self,
        name: &str,
        angled: bool,
        start_index: Option<usize>,
        include_next: bool,
        site: Place,
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
                    self.diagnose_header_shadowing(name, &full, site, angled, start_index, None);
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
                self.diagnose_header_shadowing(name, &full, site, angled, start_index, Some(index));
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
            file.lexer.keep_comments = false;
            file.lexer.keep_whitespace = false;
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
                    "embed" => return self.handle_embed(&hash, &directive),
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
        if !self.macros.contains_key(&name.spelling) {
            self.emit_final_macro_warning(&name, false);
        }
        let keep_macro_comments = self.options.keep_macro_comments;
        self.set_directive_comment_retention(keep_macro_comments);
        let definition = self.read_macro_definition(&name);
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
            self.emit_final_macro_warning(&name, false);
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
        self.emit_final_macro_warning(&name, true);
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
        self.emit_macro_expansion_warnings(&name, true);
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
                lexer.keep_whitespace = false;
                lexer.lex()
            };
            if directive.kind != TokenKind::Identifier {
                if let Some(lexer) = self.file_lexer_mut() {
                    lexer.in_directive = false;
                    lexer.reset_extended_mode();
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
                    lexer.reset_extended_mode();
                }
                continue;
            }
            if name.len() >= 20 {
                if let Some(lexer) = self.file_lexer_mut() {
                    lexer.in_directive = false;
                    lexer.reset_extended_mode();
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
                        self.emit_macro_expansion_warnings(&macro_name, false);
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

    fn evaluate_embed_expression(&mut self) -> (Option<ExpressionValue>, Token) {
        let saved_if = self.parsing_if_directive;
        let saved_disable = self.disable_expansion;
        self.parsing_if_directive = true;
        self.disable_expansion = false;
        let mut token = self.lex_non_comment();
        let result = match self.evaluate_value(&mut token, true) {
            Err(()) => {
                if token.kind != TokenKind::EndOfDirective {
                    token = self.discard_until_end_of_directive();
                }
                None
            }
            Ok(mut value) => {
                if token.kind == TokenKind::EndOfDirective {
                    Some(value)
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
                        token = self.discard_until_end_of_directive();
                    }
                    None
                } else {
                    Some(value)
                }
            }
        };
        self.parsing_if_directive = saved_if;
        self.disable_expansion = saved_disable;
        (result, token)
    }

    fn skip_to_end_of_directive(&mut self, current: &Token) -> Token {
        if current.kind == TokenKind::EndOfDirective {
            current.clone()
        } else {
            self.discard_until_end_of_directive()
        }
    }

    fn expect_embed_token(&mut self, current: &mut Token, kind: TokenKind, text: &str) -> bool {
        if current.kind == kind {
            return true;
        }
        self.report_token(Message::Expected, current, vec![text.to_string()]);
        *current = self.skip_to_end_of_directive(current);
        false
    }

    fn embed_integer(&mut self, current: &mut Token, end_kind: TokenKind) -> Option<u64> {
        if !self.expect_embed_token(current, TokenKind::LeftParen, "'('") {
            return None;
        }
        self.evaluated_defined = false;
        let (value, token) = self.evaluate_embed_expression();
        *current = token;
        let value = value?;
        if !self.expect_embed_token(current, TokenKind::RightParen, "')'") {
            return None;
        }
        *current = self.lex_non_comment();
        if self.evaluated_defined {
            self.report_token(Message::DefinedInEmbed, current, Vec::new());
            *current = self.skip_to_end_of_directive(current);
            return None;
        }
        if value.is_negative() {
            let text = (value.bits as i64).to_string();
            self.report_token(
                Message::RequiresPositiveValue,
                current,
                vec![text, "0".to_string()],
            );
            if current.kind != end_kind {
                *current = self.skip_to_end_of_directive(current);
            }
            return None;
        }
        Some(value.bits)
    }

    fn embed_token_soup(&mut self, current: &mut Token, tokens: &mut Vec<Token>) -> bool {
        if !self.expect_embed_token(current, TokenKind::LeftParen, "'('") {
            return false;
        }
        *current = self.lex_non_comment();
        let mut brackets: Vec<Token> = Vec::new();
        let mut waiting_for_inner = false;
        while current.kind != TokenKind::EndOfDirective
            && (waiting_for_inner || current.kind != TokenKind::RightParen)
        {
            match current.kind {
                TokenKind::LeftParen => {
                    waiting_for_inner = true;
                    brackets.push(current.clone());
                }
                TokenKind::LeftBrace | TokenKind::LeftSquare => brackets.push(current.clone()),
                TokenKind::RightParen | TokenKind::RightBrace | TokenKind::RightSquare => {
                    if current.kind == TokenKind::RightParen {
                        waiting_for_inner = false;
                    }
                    let Some(open) = brackets.last().cloned() else {
                        self.expect_embed_token(current, TokenKind::RightParen, "')'");
                        return false;
                    };
                    let (matching, text) = match open.kind {
                        TokenKind::LeftParen => (TokenKind::RightParen, "')'"),
                        TokenKind::LeftBrace => (TokenKind::RightBrace, "'}'"),
                        _ => (TokenKind::RightSquare, "']'"),
                    };
                    if current.kind != matching {
                        self.report_token(Message::Expected, current, vec![text.to_string()]);
                        let opener = format!("'{}'", open.text());
                        self.report_token(Message::ToMatch, &open, vec![opener]);
                        *current = self.skip_to_end_of_directive(current);
                        return false;
                    }
                    brackets.pop();
                }
                _ => {}
            }
            tokens.push(current.clone());
            *current = self.lex_non_comment();
        }
        if !self.expect_embed_token(current, TokenKind::RightParen, "')'") {
            return false;
        }
        *current = self.lex_non_comment();
        true
    }

    fn lex_embed_parameters(&mut self, for_has_embed: bool) -> (Option<EmbedParameters>, Token) {
        let end_kind = if for_has_embed {
            TokenKind::RightParen
        } else {
            TokenKind::EndOfDirective
        };
        let mut result = EmbedParameters::default();
        let mut current = self.lex_non_comment();
        while current.kind != end_kind && current.kind != TokenKind::EndOfDirective {
            let start = current.clone();
            if !self.expect_embed_token(&mut current, TokenKind::Identifier, "identifier") {
                return (None, current);
            }
            let mut name = current.text().to_string();
            current = self.lex_non_comment();
            if current.kind == TokenKind::ColonColon {
                current = self.lex_non_comment();
                if !self.expect_embed_token(&mut current, TokenKind::Identifier, "identifier") {
                    return (None, current);
                }
                name = format!("{name}::{}", current.text());
                current = self.lex_non_comment();
            }
            let parameter = if name.len() > 4 && name.starts_with("__") && name.ends_with("__") {
                name[2..name.len() - 2].to_string()
            } else {
                name.clone()
            };
            match parameter.as_str() {
                "limit" | "clang::offset" => {
                    let held = if parameter == "limit" {
                        result.limit.is_some()
                    } else {
                        result.offset.is_some()
                    };
                    if held {
                        self.report_token(
                            Message::EmbedDuplicateParameter,
                            &current,
                            vec![parameter.clone()],
                        );
                    }
                    let Some(value) = self.embed_integer(&mut current, end_kind) else {
                        return (None, current);
                    };
                    if parameter == "limit" {
                        result.limit = Some(value);
                    } else {
                        result.offset = Some(value);
                    }
                }
                "prefix" | "suffix" | "if_empty" => {
                    let held = match parameter.as_str() {
                        "prefix" => result.prefix.is_some(),
                        "suffix" => result.suffix.is_some(),
                        _ => result.if_empty.is_some(),
                    };
                    if held {
                        self.report_token(
                            Message::EmbedDuplicateParameter,
                            &current,
                            vec![parameter.clone()],
                        );
                    }
                    let mut tokens = Vec::new();
                    if !self.embed_token_soup(&mut current, &mut tokens) {
                        return (None, current);
                    }
                    match parameter.as_str() {
                        "prefix" => result.prefix = Some(tokens),
                        "suffix" => result.suffix = Some(tokens),
                        _ => result.if_empty = Some(tokens),
                    }
                }
                _ => {
                    result.unrecognized += 1;
                    if current.kind == TokenKind::LeftParen {
                        let mut ignored = Vec::new();
                        if !self.embed_token_soup(&mut current, &mut ignored) {
                            return (None, current);
                        }
                    }
                    if !for_has_embed {
                        self.report_token(
                            Message::UnknownParameter,
                            &start,
                            vec!["1".to_string(), parameter],
                        );
                        if current.kind != end_kind {
                            current = self.skip_to_end_of_directive(&current);
                        }
                        return (None, current);
                    }
                }
            }
        }
        (Some(result), current)
    }

    fn lookup_embed_file(&mut self, name: &str, angled: bool) -> Option<Rc<[u8]>> {
        if name.starts_with('/') {
            return self.read_file(name);
        }
        let mut candidates = Vec::new();
        if !angled {
            if let Some(index) = self.real_file_index() {
                if let Source::File(file) = &self.stack[index] {
                    if self.sources.buffer(file.buffer).kind == BufferKind::File {
                        candidates.push(Self::join_path(&file.directory, name));
                    }
                }
            }
            if !self.options.working_directory.is_empty() {
                candidates.push(Self::join_path(&self.options.working_directory, name));
            }
        }
        for directory in &self.options.embed_directories {
            candidates.push(Self::join_path(directory, name));
        }
        for candidate in candidates {
            let full = if candidate.starts_with('/') || self.options.working_directory.is_empty() {
                candidate
            } else {
                Self::join_path(&self.options.working_directory, &candidate)
            };
            if name.is_empty() || full.ends_with('/') {
                continue;
            }
            if let Some(bytes) = self.read_file(&full) {
                return Some(bytes);
            }
        }
        None
    }

    fn handle_embed(&mut self, hash: &Token, directive: &Token) {
        if self.language.at_least(CStandard::C23) {
            self.report_token(Message::EmbedDirectiveCompatibility, directive, Vec::new());
        } else {
            self.report_token(
                Message::EmbedDirectiveExtension,
                directive,
                vec!["0".to_string()],
            );
        }
        let Ok(filename) = self.lex_header_name(true) else {
            return;
        };
        if filename.kind != TokenKind::HeaderName {
            self.report_token(Message::ExpectsFilename, &filename, Vec::new());
            if filename.kind != TokenKind::EndOfDirective {
                self.discard_until_end_of_directive();
            }
            return;
        }
        let (parameters, _) = self.lex_embed_parameters(false);
        let Some(parameters) = parameters else {
            return;
        };
        let Some((name, angled)) = Self::include_filename(&filename) else {
            return;
        };
        if name.is_empty() {
            self.report_token(Message::EmptyFilename, &filename, Vec::new());
            return;
        }
        let Some(bytes) = self.lookup_embed_file(&name, angled) else {
            self.report_token(Message::FileNotFound, &filename, vec![name]);
            return;
        };
        let mut contents: &[u8] = &bytes;
        if let Some(offset) = parameters.offset {
            contents = &contents[(offset.min(contents.len() as u64)) as usize..];
        }
        if let Some(limit) = parameters.limit {
            contents = &contents[..(limit.min(contents.len() as u64)) as usize];
        }
        let mut tokens: Vec<Token> = Vec::new();
        if contents.is_empty() {
            match parameters.if_empty {
                Some(empty) => tokens = empty,
                None => return,
            }
        } else {
            if let Some(prefix) = parameters.prefix {
                tokens.extend(prefix);
            }
            let text = contents
                .iter()
                .map(|byte| byte.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            tokens.push(self.make_token(
                TokenKind::Embed,
                text.into_bytes(),
                (hash.site, hash.site),
            ));
            if let Some(suffix) = parameters.suffix {
                tokens.extend(suffix);
            }
        }
        if tokens.is_empty() {
            return;
        }
        if let Some(index) = self.real_file_index() {
            if let Source::File(file) = &mut self.stack[index] {
                file.lexer.at_start_of_line = false;
            }
        }
        self.push_tokens(tokens, true);
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
        self.evaluated_defined = true;
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
        let special = matches!(&*peek.spelling, b"INFINITY" | b"NAN");
        self.emit_macro_expansion_warnings(&peek, special);
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
                    if message == Message::UndefIdentifier
                        && self.message_ignored(Message::UndefIdentifier, current.site)
                        && self
                            .options
                            .warnings
                            .undef_prefixes
                            .iter()
                            .any(|prefix| current.spelling.starts_with(prefix.as_bytes()))
                    {
                        self.report_token(
                            Message::UndefPrefixIdentifier,
                            &current,
                            vec![quoted(&current.spelling)],
                        );
                    }
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

const ATTRIBUTE_SPELLINGS: &str = "\
C23 - _Noreturn 202202:-\n\
C23 - deprecated 201904:-\n\
C23 - fallthrough 201910:-\n\
C23 - maybe_unused 202106:-\n\
C23 - nodiscard 202003:-\n\
C23 - noreturn 202202:-\n\
C23 clang NSObject 1:-\n\
C23 clang __clang_arm_builtin_alias 1:a=arm,thumb,armeb,thumbeb,aarch64,aarch64_be,aarch64_32\n\
C23 clang __clang_arm_mve_strict_polymorphism 1:a=arm,thumb,armeb,thumbeb\n\
C23 clang aarch64_sve_pcs 1:-\n\
C23 clang aarch64_vector_pcs 1:-\n\
C23 clang acquire_handle 1:-\n\
C23 clang address_space 1:-\n\
C23 clang allocating 1:-\n\
C23 clang always_inline 1:-\n\
C23 clang amdgpu_av 1:a=amdgpu,r600\n\
C23 clang amdgpu_kernel 1:-\n\
C23 clang annotate 1:-\n\
C23 clang annotate_type 1:-\n\
C23 clang argument_with_type_tag 1:-\n\
C23 clang assume 1:-\n\
C23 clang atomic 1:-\n\
C23 clang availability 1:-\n\
C23 clang available_only_in_default_eval_method 1:-\n\
C23 clang blocking 1:-\n\
C23 clang blocks 1:-\n\
C23 clang bpf_fastcall 1:a=bpfel,bpfeb\n\
C23 clang btf_decl_tag 1:-\n\
C23 clang btf_type_tag 1:-\n\
C23 clang builtin_alias 1:-\n\
C23 clang callback 1:-\n\
C23 clang called_once 1:-\n\
C23 clang cf_audited_transfer 1:-\n\
C23 clang cf_consumed 1:-\n\
C23 clang cf_returns_not_retained 1:-\n\
C23 clang cf_returns_retained 1:-\n\
C23 clang cf_unknown_transfer 1:-\n\
C23 clang cfi_canonical_jump_table 1:-\n\
C23 clang cfi_salt 1:-\n\
C23 clang cfi_unchecked_callee 1:-\n\
C23 clang clspv_libclc_builtin 1:-\n\
C23 clang code_align 1:-\n\
C23 clang convergent 1:-\n\
C23 clang coro_await_elidable 1:-\n\
C23 clang coro_await_elidable_argument 1:-\n\
C23 clang coro_disable_lifetimebound 1:-\n\
C23 clang coro_lifetimebound 1:-\n\
C23 clang coro_only_destroy_when_complete 1:-\n\
C23 clang coro_return_type 1:-\n\
C23 clang coro_wrapper 1:-\n\
C23 clang counted_by 1:-\n\
C23 clang counted_by_or_null 1:-\n\
C23 clang cpu_dispatch 1:-\n\
C23 clang cpu_specific 1:-\n\
C23 clang device_kernel 1:-\n\
C23 clang diagnose_as_builtin 1:-\n\
C23 clang disable_sanitizer_instrumentation 1:-\n\
C23 clang disable_tail_calls 1:-\n\
C23 clang enforce_read_only_placement 1:-\n\
C23 clang enforce_tcb 1:-\n\
C23 clang enforce_tcb_leaf 1:-\n\
C23 clang enum_extensibility 1:-\n\
C23 clang exclude_from_explicit_instantiation 1:-\n\
C23 clang export_name 1:a=wasm32,wasm64\n\
C23 clang ext_vector_type 1:-\n\
C23 clang external_source_symbol 20230206:-\n\
C23 clang flag_enum 1:-\n\
C23 clang guard 1:o=Win32\n\
C23 clang hybrid_patchable 1:c=arm64ec\n\
C23 clang ibaction 1:-\n\
C23 clang iboutlet 1:-\n\
C23 clang iboutletcollection 1:-\n\
C23 clang import_module 1:a=wasm32,wasm64\n\
C23 clang import_name 1:a=wasm32,wasm64\n\
C23 clang internal_linkage 1:-\n\
C23 clang lifetime_capture_by 1:-\n\
C23 clang lifetime_capture_by_global 1:-\n\
C23 clang lifetime_capture_by_this 1:-\n\
C23 clang lifetime_capture_by_unknown 1:-\n\
C23 clang lifetimebound 1:-\n\
C23 clang likely 1:-\n\
C23 clang loader_uninitialized 1:-\n\
C23 clang lto_visibility_public 1:-\n\
C23 clang m68k_rtd 1:-\n\
C23 clang malloc_span 1:-\n\
C23 clang matrix_type 1:-\n\
C23 clang maybe_undef 1:-\n\
C23 clang mig_server_routine 1:-\n\
C23 clang min_vector_width 1:-\n\
C23 clang minsize 1:-\n\
C23 clang modular_format 1:-\n\
C23 clang musttail 1:c=musttail\n\
C23 clang neon_polyvector_type 1:-\n\
C23 clang neon_vector_type 1:-\n\
C23 clang no_builtin 1:-\n\
C23 clang no_field_protection 1:-\n\
C23 clang no_outline 1:-\n\
C23 clang no_sanitize 1:-\n\
C23 clang no_sanitize_memory 1:-\n\
C23 clang no_speculative_load_hardening 1:-\n\
C23 clang no_stack_protector 1:-\n\
C23 clang no_thread_safety_analysis 1:-\n\
C23 clang noconvergent 1:-\n\
C23 clang noderef 1:-\n\
C23 clang noduplicate 1:-\n\
C23 clang noescape 1:-\n\
C23 clang noinline 1:-\n\
C23 clang nomerge 1:-\n\
C23 clang nonallocating 1:-\n\
C23 clang nonblocking 1:-\n\
C23 clang not_tail_called 1:-\n\
C23 clang nouwtable 1:-\n\
C23 clang ns_consumed 1:-\n\
C23 clang ns_consumes_self 1:-\n\
C23 clang ns_returns_autoreleased 1:-\n\
C23 clang ns_returns_not_retained 1:-\n\
C23 clang ns_returns_retained 1:-\n\
C23 clang nvptx_kernel 1:-\n\
C23 clang objc_arc_weak_reference_unavailable 1:-\n\
C23 clang objc_boxable 1:-\n\
C23 clang objc_bridge 1:-\n\
C23 clang objc_bridge_mutable 1:-\n\
C23 clang objc_bridge_related 1:-\n\
C23 clang objc_class_stub 1:-\n\
C23 clang objc_designated_initializer 1:-\n\
C23 clang objc_direct 1:-\n\
C23 clang objc_direct_members 1:-\n\
C23 clang objc_exception 1:-\n\
C23 clang objc_externally_retained 1:-\n\
C23 clang objc_gc 1:-\n\
C23 clang objc_independent_class 1:-\n\
C23 clang objc_method_family 1:-\n\
C23 clang objc_non_runtime_protocol 1:-\n\
C23 clang objc_nonlazy_class 1:-\n\
C23 clang objc_ownership 1:-\n\
C23 clang objc_precise_lifetime 1:-\n\
C23 clang objc_protocol_requires_explicit_implementation 1:-\n\
C23 clang objc_requires_property_definitions 1:-\n\
C23 clang objc_requires_super 1:-\n\
C23 clang objc_returns_inner_pointer 1:-\n\
C23 clang objc_root_class 1:-\n\
C23 clang objc_runtime_name 1:-\n\
C23 clang objc_runtime_visible 1:-\n\
C23 clang objc_subclassing_restricted 1:-\n\
C23 clang opencl_constant 1:-\n\
C23 clang opencl_generic 1:-\n\
C23 clang opencl_global 1:-\n\
C23 clang opencl_global_device 1:-\n\
C23 clang opencl_global_host 1:-\n\
C23 clang opencl_local 1:-\n\
C23 clang opencl_private 1:-\n\
C23 clang optnone 1:-\n\
C23 clang os_consumed 1:-\n\
C23 clang os_consumes_this 1:-\n\
C23 clang os_returns_not_retained 1:-\n\
C23 clang os_returns_retained 1:-\n\
C23 clang os_returns_retained_on_non_zero 1:-\n\
C23 clang os_returns_retained_on_zero 1:-\n\
C23 clang overflow_behavior 1:-\n\
C23 clang overloadable 1:-\n\
C23 clang ownership_holds 1:-\n\
C23 clang ownership_returns 1:-\n\
C23 clang ownership_takes 1:-\n\
C23 clang pascal 1:-\n\
C23 clang pass_dynamic_object_size 1:-\n\
C23 clang pass_object_size 1:-\n\
C23 clang personality 1:-\n\
C23 clang pointer_field_protection 1:-\n\
C23 clang pointer_with_type_tag 1:-\n\
C23 clang preferred_type 1:-\n\
C23 clang preserve_access_index 1:a=bpfel,bpfeb\n\
C23 clang preserve_all 1:-\n\
C23 clang preserve_most 1:-\n\
C23 clang preserve_none 1:a=aarch64,aarch64_be,aarch64_32,x86,x86_64\n\
C23 clang preserve_static_offset 1:a=bpfel,bpfeb\n\
C23 clang ptrauth_vtable_pointer 1:-\n\
C23 clang reentrant_capability 1:-\n\
C23 clang release_handle 1:-\n\
C23 clang require_explicit_initialization 1:-\n\
C23 clang riscv_vector_cc 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
C23 clang riscv_vls_cc 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
C23 clang sized_by 1:-\n\
C23 clang sized_by_or_null 1:-\n\
C23 clang speculative_load_hardening 1:-\n\
C23 clang stack_protector_ignore 1:-\n\
C23 clang suppress 1:-\n\
C23 clang swift_async 1:-\n\
C23 clang swift_async_context 1:-\n\
C23 clang swift_async_error 1:-\n\
C23 clang swift_context 1:-\n\
C23 clang swift_error_result 1:-\n\
C23 clang swift_indirect_result 1:-\n\
C23 clang swiftasynccall 1:-\n\
C23 clang swiftcall 1:-\n\
C23 clang sycl_kernel 1:-\n\
C23 clang sycl_special_class 1:-\n\
C23 clang type_tag_for_datatype 1:-\n\
C23 clang type_visibility 1:-\n\
C23 clang unavailable 1:-\n\
C23 clang unlikely 1:-\n\
C23 clang unsafe_buffer_usage 1:-\n\
C23 clang use_handle 1:-\n\
C23 clang vectorcall 1:-\n\
C23 clang weak_import 1:-\n\
C23 clang xray_always_instrument 1:-\n\
C23 clang xray_log_args 1:-\n\
C23 clang xray_never_instrument 1:-\n\
C23 gnu __const 1:-\n\
C23 gnu alias 1:-\n\
C23 gnu aligned 1:-\n\
C23 gnu alloc_align 1:-\n\
C23 gnu alloc_size 1:-\n\
C23 gnu always_inline 1:-\n\
C23 gnu artificial 1:-\n\
C23 gnu assume_aligned 1:-\n\
C23 gnu cdecl 1:-\n\
C23 gnu cleanup 1:-\n\
C23 gnu cold 1:-\n\
C23 gnu common 1:-\n\
C23 gnu const 1:-\n\
C23 gnu constructor 1:-\n\
C23 gnu deprecated 1:-\n\
C23 gnu destructor 1:-\n\
C23 gnu dllexport 1:c=dllimport\n\
C23 gnu dllimport 1:c=dllimport\n\
C23 gnu error 1:-\n\
C23 gnu fallthrough 1:-\n\
C23 gnu far 1:a=mips,mipsel,mips64,mips64el\n\
C23 gnu fastcall 1:-\n\
C23 gnu flag_enum 1:-\n\
C23 gnu flatten 1:-\n\
C23 gnu force_align_arg_pointer 1:a=x86,x86_64\n\
C23 gnu format 1:-\n\
C23 gnu format_arg 1:-\n\
C23 gnu format_matches 1:-\n\
C23 gnu function_return 1:a=x86,x86_64\n\
C23 gnu gcc_struct 1:-\n\
C23 gnu gnu_inline 1:-\n\
C23 gnu hot 1:-\n\
C23 gnu ifunc 1:c=ifunc\n\
C23 gnu interrupt 1:a=arm,thumb,armeb,thumbeb 1:a=avr 1:a=x86,x86_64 1:a=msp430 1:a=mips,mipsel 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
C23 gnu leaf 1:-\n\
C23 gnu long_call 1:a=mips,mipsel,mips64,mips64el\n\
C23 gnu malloc 1:-\n\
C23 gnu may_alias 1:-\n\
C23 gnu micromips 1:a=mips,mipsel\n\
C23 gnu mips16 1:a=mips,mipsel\n\
C23 gnu mode 1:-\n\
C23 gnu model 1:a=loongarch32,loongarch64,x86_64,nvptx,nvptx64,amdgpu,r600,spirv,spirv32,spirv64\n\
C23 gnu ms_abi 1:-\n\
C23 gnu ms_struct 1:-\n\
C23 gnu naked 1:-\n\
C23 gnu near 1:a=mips,mipsel,mips64,mips64el\n\
C23 gnu no_address_safety_analysis 1:-\n\
C23 gnu no_caller_saved_registers 1:a=x86,x86_64\n\
C23 gnu no_instrument_function 1:-\n\
C23 gnu no_profile_instrument_function 1:-\n\
C23 gnu no_randomize_layout 1:-\n\
C23 gnu no_sanitize 1:-\n\
C23 gnu no_sanitize_address 1:-\n\
C23 gnu no_sanitize_thread 1:-\n\
C23 gnu no_split_stack 1:-\n\
C23 gnu no_stack_protector 1:-\n\
C23 gnu nocf_check 1:a=x86,x86_64\n\
C23 gnu nocommon 1:-\n\
C23 gnu nodebug 1:-\n\
C23 gnu noinline 1:-\n\
C23 gnu nomicromips 1:a=mips,mipsel\n\
C23 gnu nomips16 1:a=mips,mipsel\n\
C23 gnu nonnull 1:-\n\
C23 gnu nonstring 1:-\n\
C23 gnu noreturn 1:-\n\
C23 gnu nothrow 1:-\n\
C23 gnu packed 1:-\n\
C23 gnu patchable_function_entry 1:a=aarch64,aarch64_be,loongarch32,loongarch64,riscv32,riscv64,x86,x86_64,ppc,ppc64,ppc64le,systemz\n\
C23 gnu pcs 1:-\n\
C23 gnu pure 1:-\n\
C23 gnu randomize_layout 1:-\n\
C23 gnu regcall 1:-\n\
C23 gnu regparm 1:-\n\
C23 gnu retain 1:-\n\
C23 gnu returns_nonnull 1:-\n\
C23 gnu returns_twice 1:-\n\
C23 gnu section 1:-\n\
C23 gnu selectany 1:-\n\
C23 gnu sentinel 1:-\n\
C23 gnu short_call 1:a=mips,mipsel,mips64,mips64el\n\
C23 gnu signal 1:a=avr\n\
C23 gnu stdcall 1:-\n\
C23 gnu sysv_abi 1:-\n\
C23 gnu target 1:-\n\
C23 gnu target_clones 1:-\n\
C23 gnu target_version 1:a=aarch64,aarch64_be,aarch64_32,riscv32,riscv64,riscv32be,riscv64be\n\
C23 gnu thiscall 1:-\n\
C23 gnu tls_model 1:-\n\
C23 gnu transparent_union 1:-\n\
C23 gnu unused 1:-\n\
C23 gnu used 1:-\n\
C23 gnu vector_size 1:-\n\
C23 gnu visibility 1:-\n\
C23 gnu warn_unused 1:-\n\
C23 gnu warn_unused_result 1:-\n\
C23 gnu warning 1:-\n\
C23 gnu weak 1:-\n\
C23 gnu weakref 1:-\n\
C23 gnu zero_call_used_regs 1:-\n\
C23 msvc forceinline 1:-\n\
C23 msvc forceinline_calls 1:-\n\
C23 msvc noinline 1:-\n\
C23 riscv vector_cc 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
C23 riscv vls_cc 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
CXX11 - assume 202207:-\n\
CXX11 - deprecated 201309:-\n\
CXX11 - fallthrough 201603:-\n\
CXX11 - likely 201803:-\n\
CXX11 - maybe_unused 201603:-\n\
CXX11 - no_unique_address 201803:c=itanium\n\
CXX11 - nodiscard 201907:-\n\
CXX11 - noreturn 200809:-\n\
CXX11 - unlikely 201803:-\n\
CXX11 clang NSObject 1:-\n\
CXX11 clang __clang_arm_builtin_alias 1:a=arm,thumb,armeb,thumbeb,aarch64,aarch64_be,aarch64_32\n\
CXX11 clang __clang_arm_mve_strict_polymorphism 1:a=arm,thumb,armeb,thumbeb\n\
CXX11 clang aarch64_sve_pcs 1:-\n\
CXX11 clang aarch64_vector_pcs 1:-\n\
CXX11 clang acquire_capability 1:-\n\
CXX11 clang acquire_handle 1:-\n\
CXX11 clang acquire_shared_capability 1:-\n\
CXX11 clang address_space 1:-\n\
CXX11 clang allocating 1:-\n\
CXX11 clang always_destroy 1:-\n\
CXX11 clang always_inline 1:-\n\
CXX11 clang amdgpu_av 1:a=amdgpu,r600\n\
CXX11 clang amdgpu_flat_work_group_size 1:-\n\
CXX11 clang amdgpu_kernel 1:-\n\
CXX11 clang amdgpu_max_num_work_groups 1:-\n\
CXX11 clang amdgpu_num_sgpr 1:-\n\
CXX11 clang amdgpu_num_vgpr 1:-\n\
CXX11 clang amdgpu_waves_per_eu 1:-\n\
CXX11 clang annotate 1:-\n\
CXX11 clang annotate_type 1:-\n\
CXX11 clang argument_with_type_tag 1:-\n\
CXX11 clang assert_capability 1:-\n\
CXX11 clang assert_shared_capability 1:-\n\
CXX11 clang assume 1:-\n\
CXX11 clang atomic 1:-\n\
CXX11 clang availability 1:-\n\
CXX11 clang available_only_in_default_eval_method 1:-\n\
CXX11 clang blocking 1:-\n\
CXX11 clang blocks 1:-\n\
CXX11 clang bpf_fastcall 1:a=bpfel,bpfeb\n\
CXX11 clang btf_decl_tag 1:-\n\
CXX11 clang btf_type_tag 1:-\n\
CXX11 clang builtin_alias 1:-\n\
CXX11 clang callable_when 1:-\n\
CXX11 clang callback 1:-\n\
CXX11 clang called_once 1:-\n\
CXX11 clang capability 1:-\n\
CXX11 clang cf_audited_transfer 1:-\n\
CXX11 clang cf_consumed 1:-\n\
CXX11 clang cf_returns_not_retained 1:-\n\
CXX11 clang cf_returns_retained 1:-\n\
CXX11 clang cf_unknown_transfer 1:-\n\
CXX11 clang cfi_canonical_jump_table 1:-\n\
CXX11 clang cfi_salt 1:-\n\
CXX11 clang cfi_unchecked_callee 1:-\n\
CXX11 clang clspv_libclc_builtin 1:-\n\
CXX11 clang code_align 1:-\n\
CXX11 clang consumable 1:-\n\
CXX11 clang consumable_auto_cast_state 1:-\n\
CXX11 clang consumable_set_state_on_read 1:-\n\
CXX11 clang convergent 1:-\n\
CXX11 clang coro_await_elidable 1:-\n\
CXX11 clang coro_await_elidable_argument 1:-\n\
CXX11 clang coro_disable_lifetimebound 1:-\n\
CXX11 clang coro_lifetimebound 1:-\n\
CXX11 clang coro_only_destroy_when_complete 1:-\n\
CXX11 clang coro_return_type 1:-\n\
CXX11 clang coro_wrapper 1:-\n\
CXX11 clang counted_by 1:-\n\
CXX11 clang counted_by_or_null 1:-\n\
CXX11 clang cpu_dispatch 1:-\n\
CXX11 clang cpu_specific 1:-\n\
CXX11 clang device_kernel 1:-\n\
CXX11 clang diagnose_as_builtin 1:-\n\
CXX11 clang disable_sanitizer_instrumentation 1:-\n\
CXX11 clang disable_tail_calls 1:-\n\
CXX11 clang enforce_read_only_placement 1:-\n\
CXX11 clang enforce_tcb 1:-\n\
CXX11 clang enforce_tcb_leaf 1:-\n\
CXX11 clang enum_extensibility 1:-\n\
CXX11 clang exclude_from_explicit_instantiation 1:-\n\
CXX11 clang exclusive_locks_required 1:-\n\
CXX11 clang export_name 1:a=wasm32,wasm64\n\
CXX11 clang ext_vector_type 1:-\n\
CXX11 clang external_source_symbol 20230206:-\n\
CXX11 clang fallthrough 1:-\n\
CXX11 clang flag_enum 1:-\n\
CXX11 clang guard 1:o=Win32\n\
CXX11 clang guarded_var 1:-\n\
CXX11 clang hybrid_patchable 1:c=arm64ec\n\
CXX11 clang ibaction 1:-\n\
CXX11 clang iboutlet 1:-\n\
CXX11 clang iboutletcollection 1:-\n\
CXX11 clang import_module 1:a=wasm32,wasm64\n\
CXX11 clang import_name 1:a=wasm32,wasm64\n\
CXX11 clang intel_ocl_bicc 1:-\n\
CXX11 clang internal_linkage 1:-\n\
CXX11 clang lifetime_capture_by 1:-\n\
CXX11 clang lifetime_capture_by_global 1:-\n\
CXX11 clang lifetime_capture_by_this 1:-\n\
CXX11 clang lifetime_capture_by_unknown 1:-\n\
CXX11 clang lifetimebound 1:-\n\
CXX11 clang loader_uninitialized 1:-\n\
CXX11 clang lto_visibility_public 1:-\n\
CXX11 clang m68k_rtd 1:-\n\
CXX11 clang malloc_span 1:-\n\
CXX11 clang matrix_type 1:-\n\
CXX11 clang maybe_undef 1:-\n\
CXX11 clang mig_server_routine 1:-\n\
CXX11 clang min_vector_width 1:-\n\
CXX11 clang minsize 1:-\n\
CXX11 clang modular_format 1:-\n\
CXX11 clang musttail 1:c=musttail\n\
CXX11 clang neon_polyvector_type 1:-\n\
CXX11 clang neon_vector_type 1:-\n\
CXX11 clang no_builtin 1:-\n\
CXX11 clang no_destroy 1:-\n\
CXX11 clang no_field_protection 1:-\n\
CXX11 clang no_outline 1:-\n\
CXX11 clang no_sanitize 1:-\n\
CXX11 clang no_sanitize_memory 1:-\n\
CXX11 clang no_specializations 1:-\n\
CXX11 clang no_speculative_load_hardening 1:-\n\
CXX11 clang no_stack_protector 1:-\n\
CXX11 clang no_thread_safety_analysis 1:-\n\
CXX11 clang noconvergent 1:-\n\
CXX11 clang noderef 1:-\n\
CXX11 clang noduplicate 1:-\n\
CXX11 clang noescape 1:-\n\
CXX11 clang noinline 1:-\n\
CXX11 clang nomerge 1:-\n\
CXX11 clang nonallocating 1:-\n\
CXX11 clang nonblocking 1:-\n\
CXX11 clang not_tail_called 1:-\n\
CXX11 clang nouwtable 1:-\n\
CXX11 clang ns_consumed 1:-\n\
CXX11 clang ns_consumes_self 1:-\n\
CXX11 clang ns_returns_autoreleased 1:-\n\
CXX11 clang ns_returns_not_retained 1:-\n\
CXX11 clang ns_returns_retained 1:-\n\
CXX11 clang nvptx_kernel 1:-\n\
CXX11 clang objc_arc_weak_reference_unavailable 1:-\n\
CXX11 clang objc_boxable 1:-\n\
CXX11 clang objc_bridge 1:-\n\
CXX11 clang objc_bridge_mutable 1:-\n\
CXX11 clang objc_bridge_related 1:-\n\
CXX11 clang objc_class_stub 1:-\n\
CXX11 clang objc_designated_initializer 1:-\n\
CXX11 clang objc_direct 1:-\n\
CXX11 clang objc_direct_members 1:-\n\
CXX11 clang objc_exception 1:-\n\
CXX11 clang objc_externally_retained 1:-\n\
CXX11 clang objc_gc 1:-\n\
CXX11 clang objc_independent_class 1:-\n\
CXX11 clang objc_method_family 1:-\n\
CXX11 clang objc_non_runtime_protocol 1:-\n\
CXX11 clang objc_nonlazy_class 1:-\n\
CXX11 clang objc_ownership 1:-\n\
CXX11 clang objc_precise_lifetime 1:-\n\
CXX11 clang objc_protocol_requires_explicit_implementation 1:-\n\
CXX11 clang objc_requires_property_definitions 1:-\n\
CXX11 clang objc_requires_super 1:-\n\
CXX11 clang objc_returns_inner_pointer 1:-\n\
CXX11 clang objc_root_class 1:-\n\
CXX11 clang objc_runtime_name 1:-\n\
CXX11 clang objc_runtime_visible 1:-\n\
CXX11 clang objc_subclassing_restricted 1:-\n\
CXX11 clang opencl_constant 1:-\n\
CXX11 clang opencl_generic 1:-\n\
CXX11 clang opencl_global 1:-\n\
CXX11 clang opencl_global_device 1:-\n\
CXX11 clang opencl_global_host 1:-\n\
CXX11 clang opencl_local 1:-\n\
CXX11 clang opencl_private 1:-\n\
CXX11 clang optnone 1:-\n\
CXX11 clang os_consumed 1:-\n\
CXX11 clang os_consumes_this 1:-\n\
CXX11 clang os_returns_not_retained 1:-\n\
CXX11 clang os_returns_retained 1:-\n\
CXX11 clang os_returns_retained_on_non_zero 1:-\n\
CXX11 clang os_returns_retained_on_zero 1:-\n\
CXX11 clang overflow_behavior 1:-\n\
CXX11 clang overloadable 1:-\n\
CXX11 clang ownership_holds 1:-\n\
CXX11 clang ownership_returns 1:-\n\
CXX11 clang ownership_takes 1:-\n\
CXX11 clang param_typestate 1:-\n\
CXX11 clang pascal 1:-\n\
CXX11 clang pass_dynamic_object_size 1:-\n\
CXX11 clang pass_object_size 1:-\n\
CXX11 clang personality 1:-\n\
CXX11 clang pointer_field_protection 1:-\n\
CXX11 clang pointer_with_type_tag 1:-\n\
CXX11 clang preferred_name 1:-\n\
CXX11 clang preferred_type 1:-\n\
CXX11 clang preserve_access_index 1:a=bpfel,bpfeb\n\
CXX11 clang preserve_all 1:-\n\
CXX11 clang preserve_most 1:-\n\
CXX11 clang preserve_none 1:a=aarch64,aarch64_be,aarch64_32,x86,x86_64\n\
CXX11 clang preserve_static_offset 1:a=bpfel,bpfeb\n\
CXX11 clang pt_guarded_var 1:-\n\
CXX11 clang ptrauth_vtable_pointer 1:-\n\
CXX11 clang reentrant_capability 1:-\n\
CXX11 clang reinitializes 1:-\n\
CXX11 clang release_capability 1:-\n\
CXX11 clang release_generic_capability 1:-\n\
CXX11 clang release_handle 1:-\n\
CXX11 clang release_shared_capability 1:-\n\
CXX11 clang require_constant_initialization 1:-\n\
CXX11 clang require_explicit_initialization 1:-\n\
CXX11 clang requires_capability 1:-\n\
CXX11 clang requires_shared_capability 1:-\n\
CXX11 clang return_typestate 1:-\n\
CXX11 clang riscv_vector_cc 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
CXX11 clang riscv_vls_cc 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
CXX11 clang scoped_lockable 1:-\n\
CXX11 clang set_typestate 1:-\n\
CXX11 clang shared_capability 1:-\n\
CXX11 clang shared_locks_required 1:-\n\
CXX11 clang sized_by 1:-\n\
CXX11 clang sized_by_or_null 1:-\n\
CXX11 clang speculative_load_hardening 1:-\n\
CXX11 clang stack_protector_ignore 1:-\n\
CXX11 clang standalone_debug 1:-\n\
CXX11 clang suppress 1:-\n\
CXX11 clang swift_async 1:-\n\
CXX11 clang swift_async_context 1:-\n\
CXX11 clang swift_async_error 1:-\n\
CXX11 clang swift_context 1:-\n\
CXX11 clang swift_error_result 1:-\n\
CXX11 clang swift_indirect_result 1:-\n\
CXX11 clang swiftasynccall 1:-\n\
CXX11 clang swiftcall 1:-\n\
CXX11 clang sycl_constant 1:-\n\
CXX11 clang sycl_external 1:-\n\
CXX11 clang sycl_generic 1:-\n\
CXX11 clang sycl_global 1:-\n\
CXX11 clang sycl_kernel 1:-\n\
CXX11 clang sycl_kernel_entry_point 1:-\n\
CXX11 clang sycl_local 1:-\n\
CXX11 clang sycl_private 1:-\n\
CXX11 clang sycl_special_class 1:-\n\
CXX11 clang test_typestate 1:-\n\
CXX11 clang trivial_abi 1:-\n\
CXX11 clang try_acquire_capability 1:-\n\
CXX11 clang try_acquire_shared_capability 1:-\n\
CXX11 clang type_tag_for_datatype 1:-\n\
CXX11 clang type_visibility 1:-\n\
CXX11 clang unavailable 1:-\n\
CXX11 clang uninitialized 1:-\n\
CXX11 clang unlock_function 1:-\n\
CXX11 clang unsafe_buffer_usage 1:-\n\
CXX11 clang use_handle 1:-\n\
CXX11 clang using_if_exists 1:-\n\
CXX11 clang vecreturn 1:-\n\
CXX11 clang vectorcall 1:-\n\
CXX11 clang warn_unused_result 1:-\n\
CXX11 clang weak_import 1:-\n\
CXX11 clang xray_always_instrument 1:-\n\
CXX11 clang xray_log_args 1:-\n\
CXX11 clang xray_never_instrument 1:-\n\
CXX11 gnu __const 1:-\n\
CXX11 gnu abi_tag 1:-\n\
CXX11 gnu alias 1:-\n\
CXX11 gnu aligned 1:-\n\
CXX11 gnu alloc_align 1:-\n\
CXX11 gnu alloc_size 1:-\n\
CXX11 gnu always_inline 1:-\n\
CXX11 gnu artificial 1:-\n\
CXX11 gnu assume_aligned 1:-\n\
CXX11 gnu cdecl 1:-\n\
CXX11 gnu cleanup 1:-\n\
CXX11 gnu cold 1:-\n\
CXX11 gnu common 1:-\n\
CXX11 gnu const 1:-\n\
CXX11 gnu constructor 1:-\n\
CXX11 gnu deprecated 1:-\n\
CXX11 gnu destructor 1:-\n\
CXX11 gnu dllexport 1:c=dllimport\n\
CXX11 gnu dllimport 1:c=dllimport\n\
CXX11 gnu error 1:-\n\
CXX11 gnu fallthrough 1:-\n\
CXX11 gnu far 1:a=mips,mipsel,mips64,mips64el\n\
CXX11 gnu fastcall 1:-\n\
CXX11 gnu flag_enum 1:-\n\
CXX11 gnu flatten 1:-\n\
CXX11 gnu force_align_arg_pointer 1:a=x86,x86_64\n\
CXX11 gnu format 1:-\n\
CXX11 gnu format_arg 1:-\n\
CXX11 gnu format_matches 1:-\n\
CXX11 gnu function_return 1:a=x86,x86_64\n\
CXX11 gnu gcc_struct 1:-\n\
CXX11 gnu gnu_inline 1:-\n\
CXX11 gnu hot 1:-\n\
CXX11 gnu ifunc 1:c=ifunc\n\
CXX11 gnu init_priority 1:-\n\
CXX11 gnu interrupt 1:a=arm,thumb,armeb,thumbeb 1:a=avr 1:a=x86,x86_64 1:a=msp430 1:a=mips,mipsel 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
CXX11 gnu leaf 1:-\n\
CXX11 gnu long_call 1:a=mips,mipsel,mips64,mips64el\n\
CXX11 gnu malloc 1:-\n\
CXX11 gnu may_alias 1:-\n\
CXX11 gnu micromips 1:a=mips,mipsel\n\
CXX11 gnu mips16 1:a=mips,mipsel\n\
CXX11 gnu mode 1:-\n\
CXX11 gnu model 1:a=loongarch32,loongarch64,x86_64,nvptx,nvptx64,amdgpu,r600,spirv,spirv32,spirv64\n\
CXX11 gnu ms_abi 1:-\n\
CXX11 gnu ms_struct 1:-\n\
CXX11 gnu naked 1:-\n\
CXX11 gnu near 1:a=mips,mipsel,mips64,mips64el\n\
CXX11 gnu no_address_safety_analysis 1:-\n\
CXX11 gnu no_caller_saved_registers 1:a=x86,x86_64\n\
CXX11 gnu no_instrument_function 1:-\n\
CXX11 gnu no_profile_instrument_function 1:-\n\
CXX11 gnu no_randomize_layout 1:-\n\
CXX11 gnu no_sanitize 1:-\n\
CXX11 gnu no_sanitize_address 1:-\n\
CXX11 gnu no_sanitize_thread 1:-\n\
CXX11 gnu no_split_stack 1:-\n\
CXX11 gnu no_stack_protector 1:-\n\
CXX11 gnu nocf_check 1:a=x86,x86_64\n\
CXX11 gnu nocommon 1:-\n\
CXX11 gnu nodebug 1:-\n\
CXX11 gnu noinline 1:-\n\
CXX11 gnu nomicromips 1:a=mips,mipsel\n\
CXX11 gnu nomips16 1:a=mips,mipsel\n\
CXX11 gnu nonnull 1:-\n\
CXX11 gnu nonstring 1:-\n\
CXX11 gnu noreturn 1:-\n\
CXX11 gnu nothrow 1:-\n\
CXX11 gnu packed 1:-\n\
CXX11 gnu patchable_function_entry 1:a=aarch64,aarch64_be,loongarch32,loongarch64,riscv32,riscv64,x86,x86_64,ppc,ppc64,ppc64le,systemz\n\
CXX11 gnu pcs 1:-\n\
CXX11 gnu pure 1:-\n\
CXX11 gnu randomize_layout 1:-\n\
CXX11 gnu regcall 1:-\n\
CXX11 gnu regparm 1:-\n\
CXX11 gnu retain 1:-\n\
CXX11 gnu returns_nonnull 1:-\n\
CXX11 gnu returns_twice 1:-\n\
CXX11 gnu section 1:-\n\
CXX11 gnu selectany 1:-\n\
CXX11 gnu sentinel 1:-\n\
CXX11 gnu short_call 1:a=mips,mipsel,mips64,mips64el\n\
CXX11 gnu signal 1:a=avr\n\
CXX11 gnu stdcall 1:-\n\
CXX11 gnu sysv_abi 1:-\n\
CXX11 gnu target 1:-\n\
CXX11 gnu target_clones 1:-\n\
CXX11 gnu target_version 1:a=aarch64,aarch64_be,aarch64_32,riscv32,riscv64,riscv32be,riscv64be\n\
CXX11 gnu thiscall 1:-\n\
CXX11 gnu tls_model 1:-\n\
CXX11 gnu transparent_union 1:-\n\
CXX11 gnu unused 1:-\n\
CXX11 gnu used 1:-\n\
CXX11 gnu vector_size 1:-\n\
CXX11 gnu visibility 1:-\n\
CXX11 gnu warn_unused 1:-\n\
CXX11 gnu warn_unused_result 1:-\n\
CXX11 gnu warning 1:-\n\
CXX11 gnu weak 1:-\n\
CXX11 gnu weakref 1:-\n\
CXX11 gnu zero_call_used_regs 1:-\n\
CXX11 gsl Owner 1:-\n\
CXX11 gsl Pointer 1:-\n\
CXX11 gsl suppress 1:-\n\
CXX11 hlsl contained_type 1:-\n\
CXX11 hlsl dimension 1:-\n\
CXX11 hlsl is_array 1:-\n\
CXX11 hlsl is_counter 1:-\n\
CXX11 hlsl is_ms 1:-\n\
CXX11 hlsl is_rov 1:-\n\
CXX11 hlsl raw_buffer 1:-\n\
CXX11 hlsl resource_class 1:-\n\
CXX11 msvc constexpr 1:-\n\
CXX11 msvc forceinline 1:-\n\
CXX11 msvc forceinline_calls 1:-\n\
CXX11 msvc no_unique_address 201803:a=x86,x86_64,arm,thumb,aarch64,amdgpu,nvptx,nvptx64,spirv,spirv32,spirv64/c=msrecord\n\
CXX11 msvc noinline 1:-\n\
CXX11 omp assume 1:-\n\
CXX11 riscv vector_cc 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
CXX11 riscv vls_cc 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
CXX11 vk binding 1:-\n\
CXX11 vk constant_id 1:-\n\
CXX11 vk ext_builtin_input 1:-\n\
CXX11 vk ext_builtin_output 1:-\n\
CXX11 vk location 1:-\n\
CXX11 vk push_constant 1:-\n\
Declspec - __constant__ 1:-\n\
Declspec - __cudart_builtin__ 1:-\n\
Declspec - __device__ 1:-\n\
Declspec - __device_builtin__ 1:-\n\
Declspec - __device_builtin_surface_type__ 1:-\n\
Declspec - __device_builtin_texture_type__ 1:-\n\
Declspec - __global__ 1:-\n\
Declspec - __grid_constant__ 1:-\n\
Declspec - __host__ 1:-\n\
Declspec - __launch_bounds__ 1:-\n\
Declspec - __managed__ 1:-\n\
Declspec - __shared__ 1:-\n\
Declspec - align 1:-\n\
Declspec - allocate 1:-\n\
Declspec - allocator 1:-\n\
Declspec - code_seg 1:-\n\
Declspec - cpu_dispatch 1:-\n\
Declspec - cpu_specific 1:-\n\
Declspec - deprecated 1:-\n\
Declspec - dllexport 1:c=dllimport\n\
Declspec - dllimport 1:c=dllimport\n\
Declspec - empty_bases 1:a=x86,x86_64,arm,thumb,aarch64,amdgpu,nvptx,nvptx64,spirv,spirv32,spirv64/c=msrecord\n\
Declspec - guard 1:o=Win32\n\
Declspec - hybrid_patchable 1:c=arm64ec\n\
Declspec - layout_version 1:a=x86,x86_64,arm,thumb,aarch64,amdgpu,nvptx,nvptx64,spirv,spirv32,spirv64/c=msrecord\n\
Declspec - naked 1:-\n\
Declspec - no_init_all 1:-\n\
Declspec - noalias 1:-\n\
Declspec - noconvergent 1:-\n\
Declspec - noinline 1:-\n\
Declspec - noreturn 1:-\n\
Declspec - nothrow 1:-\n\
Declspec - novtable 1:a=x86,x86_64,arm,thumb,aarch64/c=msabi\n\
Declspec - property 1:-\n\
Declspec - restrict 1:-\n\
Declspec - safebuffers 1:-\n\
Declspec - selectany 1:-\n\
Declspec - strict_gs_check 1:-\n\
Declspec - thread 1:-\n\
Declspec - uuid 1:-\n\
GNU - NSObject 1:-\n\
GNU - __clang_arm_builtin_alias 1:a=arm,thumb,armeb,thumbeb,aarch64,aarch64_be,aarch64_32\n\
GNU - __clang_arm_mve_strict_polymorphism 1:a=arm,thumb,armeb,thumbeb\n\
GNU - __const 1:-\n\
GNU - aarch64_sve_pcs 1:-\n\
GNU - aarch64_vector_pcs 1:-\n\
GNU - abi_tag 1:-\n\
GNU - acquire_capability 1:-\n\
GNU - acquire_handle 1:-\n\
GNU - acquire_shared_capability 1:-\n\
GNU - acquired_after 1:-\n\
GNU - acquired_before 1:-\n\
GNU - address_space 1:-\n\
GNU - alias 1:-\n\
GNU - align_value 1:-\n\
GNU - aligned 1:-\n\
GNU - alloc_align 1:-\n\
GNU - alloc_size 1:-\n\
GNU - allocating 1:-\n\
GNU - always_destroy 1:-\n\
GNU - always_inline 1:-\n\
GNU - amdgpu_av 1:a=amdgpu,r600\n\
GNU - amdgpu_flat_work_group_size 1:-\n\
GNU - amdgpu_kernel 1:-\n\
GNU - amdgpu_max_num_work_groups 1:-\n\
GNU - amdgpu_num_sgpr 1:-\n\
GNU - amdgpu_num_vgpr 1:-\n\
GNU - amdgpu_waves_per_eu 1:-\n\
GNU - analyzer_noreturn 1:-\n\
GNU - annotate 1:-\n\
GNU - argument_with_type_tag 1:-\n\
GNU - arm_sve_vector_bits 1:-\n\
GNU - artificial 1:-\n\
GNU - assert_capability 1:-\n\
GNU - assert_exclusive_lock 1:-\n\
GNU - assert_shared_capability 1:-\n\
GNU - assert_shared_lock 1:-\n\
GNU - assume 1:-\n\
GNU - assume_aligned 1:-\n\
GNU - atomic 1:-\n\
GNU - availability 1:-\n\
GNU - available_only_in_default_eval_method 1:-\n\
GNU - blocking 1:-\n\
GNU - blocks 1:-\n\
GNU - bounded 1:-\n\
GNU - bpf_fastcall 1:a=bpfel,bpfeb\n\
GNU - btf_decl_tag 1:-\n\
GNU - btf_type_tag 1:-\n\
GNU - callable_when 1:-\n\
GNU - callback 1:-\n\
GNU - called_once 1:-\n\
GNU - capability 1:-\n\
GNU - cdecl 1:-\n\
GNU - cf_audited_transfer 1:-\n\
GNU - cf_consumed 1:-\n\
GNU - cf_returns_not_retained 1:-\n\
GNU - cf_returns_retained 1:-\n\
GNU - cf_unknown_transfer 1:-\n\
GNU - cfi_canonical_jump_table 1:-\n\
GNU - cfi_salt 1:-\n\
GNU - cfi_unchecked_callee 1:-\n\
GNU - clang_builtin_alias 1:-\n\
GNU - cleanup 1:-\n\
GNU - clspv_libclc_builtin 1:-\n\
GNU - cluster_dims 1:-\n\
GNU - cmse_nonsecure_call 1:a=arm,thumb,armeb,thumbeb\n\
GNU - cmse_nonsecure_entry 1:a=arm,thumb,armeb,thumbeb\n\
GNU - code_align 1:-\n\
GNU - cold 1:-\n\
GNU - common 1:-\n\
GNU - const 1:-\n\
GNU - constant 1:-\n\
GNU - constructor 1:-\n\
GNU - consumable 1:-\n\
GNU - consumable_auto_cast_state 1:-\n\
GNU - consumable_set_state_on_read 1:-\n\
GNU - convergent 1:-\n\
GNU - coro_await_elidable 1:-\n\
GNU - coro_await_elidable_argument 1:-\n\
GNU - coro_disable_lifetimebound 1:-\n\
GNU - coro_lifetimebound 1:-\n\
GNU - coro_only_destroy_when_complete 1:-\n\
GNU - coro_return_type 1:-\n\
GNU - coro_wrapper 1:-\n\
GNU - counted_by 1:-\n\
GNU - counted_by_or_null 1:-\n\
GNU - cpu_dispatch 1:-\n\
GNU - cpu_specific 1:-\n\
GNU - cudart_builtin 1:-\n\
GNU - deprecated 1:-\n\
GNU - destructor 1:-\n\
GNU - device 1:-\n\
GNU - device_builtin 1:-\n\
GNU - device_builtin_surface_type 1:-\n\
GNU - device_builtin_texture_type 1:-\n\
GNU - device_kernel 1:-\n\
GNU - diagnose_as_builtin 1:-\n\
GNU - diagnose_if 1:-\n\
GNU - disable_sanitizer_instrumentation 1:-\n\
GNU - disable_tail_calls 1:-\n\
GNU - dllexport 1:c=dllimport\n\
GNU - dllimport 1:c=dllimport\n\
GNU - enable_if 1:-\n\
GNU - enforce_read_only_placement 1:-\n\
GNU - enforce_tcb 1:-\n\
GNU - enforce_tcb_leaf 1:-\n\
GNU - enum_extensibility 1:-\n\
GNU - error 1:-\n\
GNU - exclude_from_explicit_instantiation 1:-\n\
GNU - exclusive_lock_function 1:-\n\
GNU - exclusive_locks_required 1:-\n\
GNU - exclusive_trylock_function 1:-\n\
GNU - export_name 1:a=wasm32,wasm64\n\
GNU - ext_vector_type 1:-\n\
GNU - external_source_symbol 20230206:-\n\
GNU - fallthrough 1:-\n\
GNU - far 1:a=mips,mipsel,mips64,mips64el\n\
GNU - fastcall 1:-\n\
GNU - flag_enum 1:-\n\
GNU - flatten 1:-\n\
GNU - force_align_arg_pointer 1:a=x86,x86_64\n\
GNU - format 1:-\n\
GNU - format_arg 1:-\n\
GNU - format_matches 1:-\n\
GNU - function_return 1:a=x86,x86_64\n\
GNU - gcc_struct 1:-\n\
GNU - global 1:-\n\
GNU - gnu_inline 1:-\n\
GNU - grid_constant 1:-\n\
GNU - guard 1:o=Win32\n\
GNU - guarded_by 1:-\n\
GNU - guarded_var 1:-\n\
GNU - host 1:-\n\
GNU - hot 1:-\n\
GNU - hybrid_patchable 1:c=arm64ec\n\
GNU - ibaction 1:-\n\
GNU - iboutlet 1:-\n\
GNU - iboutletcollection 1:-\n\
GNU - ifunc 1:c=ifunc\n\
GNU - import_module 1:a=wasm32,wasm64\n\
GNU - import_name 1:a=wasm32,wasm64\n\
GNU - init_priority 1:-\n\
GNU - intel_ocl_bicc 1:-\n\
GNU - intel_reqd_sub_group_size 1:-\n\
GNU - internal_linkage 1:-\n\
GNU - interrupt 1:a=arm,thumb,armeb,thumbeb 1:a=avr 1:a=x86,x86_64 1:a=m68k 1:a=msp430 1:a=mips,mipsel 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
GNU - interrupt_save_fp 1:a=arm,thumb,armeb,thumbeb\n\
GNU - launch_bounds 1:-\n\
GNU - leaf 1:-\n\
GNU - lifetime_capture_by 1:-\n\
GNU - lifetime_capture_by_global 1:-\n\
GNU - lifetime_capture_by_this 1:-\n\
GNU - lifetime_capture_by_unknown 1:-\n\
GNU - lifetimebound 1:-\n\
GNU - loader_uninitialized 1:-\n\
GNU - lock_returned 1:-\n\
GNU - lockable 1:-\n\
GNU - locks_excluded 1:-\n\
GNU - long_call 1:a=mips,mipsel,mips64,mips64el\n\
GNU - lto_visibility_public 1:-\n\
GNU - m68k_rtd 1:-\n\
GNU - malloc 1:-\n\
GNU - malloc_span 1:-\n\
GNU - managed 1:-\n\
GNU - matrix_type 1:-\n\
GNU - may_alias 1:-\n\
GNU - maybe_undef 1:-\n\
GNU - micromips 1:a=mips,mipsel\n\
GNU - mig_server_routine 1:-\n\
GNU - min_vector_width 1:-\n\
GNU - minsize 1:-\n\
GNU - mips16 1:a=mips,mipsel\n\
GNU - mode 1:-\n\
GNU - model 1:a=loongarch32,loongarch64,x86_64,nvptx,nvptx64,amdgpu,r600,spirv,spirv32,spirv64\n\
GNU - modular_format 1:-\n\
GNU - ms_abi 1:-\n\
GNU - ms_struct 1:-\n\
GNU - musttail 1:c=musttail\n\
GNU - naked 1:-\n\
GNU - near 1:a=mips,mipsel,mips64,mips64el\n\
GNU - neon_polyvector_type 1:-\n\
GNU - neon_vector_type 1:-\n\
GNU - no_address_safety_analysis 1:-\n\
GNU - no_builtin 1:-\n\
GNU - no_caller_saved_registers 1:a=x86,x86_64\n\
GNU - no_cluster 1:-\n\
GNU - no_destroy 1:-\n\
GNU - no_field_protection 1:-\n\
GNU - no_instrument_function 1:-\n\
GNU - no_outline 1:-\n\
GNU - no_profile_instrument_function 1:-\n\
GNU - no_randomize_layout 1:-\n\
GNU - no_sanitize 1:-\n\
GNU - no_sanitize_address 1:-\n\
GNU - no_sanitize_memory 1:-\n\
GNU - no_sanitize_thread 1:-\n\
GNU - no_specializations 1:-\n\
GNU - no_speculative_load_hardening 1:-\n\
GNU - no_split_stack 1:-\n\
GNU - no_stack_protector 1:-\n\
GNU - no_thread_safety_analysis 1:-\n\
GNU - nocf_check 1:a=x86,x86_64\n\
GNU - nocommon 1:-\n\
GNU - noconvergent 1:-\n\
GNU - nodebug 1:-\n\
GNU - noderef 1:-\n\
GNU - noduplicate 1:-\n\
GNU - noescape 1:-\n\
GNU - noinline 1:-\n\
GNU - nomerge 1:-\n\
GNU - nomicromips 1:a=mips,mipsel\n\
GNU - nomips16 1:a=mips,mipsel\n\
GNU - nonallocating 1:-\n\
GNU - nonblocking 1:-\n\
GNU - nonnull 1:-\n\
GNU - nonstring 1:-\n\
GNU - noreturn 1:-\n\
GNU - nosvm 1:-\n\
GNU - not_tail_called 1:-\n\
GNU - nothrow 1:-\n\
GNU - nouwtable 1:-\n\
GNU - ns_consumed 1:-\n\
GNU - ns_consumes_self 1:-\n\
GNU - ns_error_domain 1:-\n\
GNU - ns_returns_autoreleased 1:-\n\
GNU - ns_returns_not_retained 1:-\n\
GNU - ns_returns_retained 1:-\n\
GNU - nv_weak 1:-\n\
GNU - nvptx_kernel 1:-\n\
GNU - objc_arc_weak_reference_unavailable 1:-\n\
GNU - objc_boxable 1:-\n\
GNU - objc_bridge 1:-\n\
GNU - objc_bridge_mutable 1:-\n\
GNU - objc_bridge_related 1:-\n\
GNU - objc_class_stub 1:-\n\
GNU - objc_designated_initializer 1:-\n\
GNU - objc_direct 1:-\n\
GNU - objc_direct_members 1:-\n\
GNU - objc_exception 1:-\n\
GNU - objc_externally_retained 1:-\n\
GNU - objc_gc 1:-\n\
GNU - objc_independent_class 1:-\n\
GNU - objc_method_family 1:-\n\
GNU - objc_non_runtime_protocol 1:-\n\
GNU - objc_nonlazy_class 1:-\n\
GNU - objc_ownership 1:-\n\
GNU - objc_precise_lifetime 1:-\n\
GNU - objc_protocol_requires_explicit_implementation 1:-\n\
GNU - objc_requires_property_definitions 1:-\n\
GNU - objc_requires_super 1:-\n\
GNU - objc_returns_inner_pointer 1:-\n\
GNU - objc_root_class 1:-\n\
GNU - objc_runtime_name 1:-\n\
GNU - objc_runtime_visible 1:-\n\
GNU - objc_subclassing_restricted 1:-\n\
GNU - opencl_constant 1:-\n\
GNU - opencl_generic 1:-\n\
GNU - opencl_global 1:-\n\
GNU - opencl_global_device 1:-\n\
GNU - opencl_global_host 1:-\n\
GNU - opencl_local 1:-\n\
GNU - opencl_private 1:-\n\
GNU - opencl_unroll_hint 1:-\n\
GNU - optnone 1:-\n\
GNU - os_consumed 1:-\n\
GNU - os_consumes_this 1:-\n\
GNU - os_returns_not_retained 1:-\n\
GNU - os_returns_retained 1:-\n\
GNU - os_returns_retained_on_non_zero 1:-\n\
GNU - os_returns_retained_on_zero 1:-\n\
GNU - overflow_behavior 1:-\n\
GNU - overloadable 1:-\n\
GNU - ownership_holds 1:-\n\
GNU - ownership_returns 1:-\n\
GNU - ownership_takes 1:-\n\
GNU - packed 1:-\n\
GNU - param_typestate 1:-\n\
GNU - pascal 1:-\n\
GNU - pass_dynamic_object_size 1:-\n\
GNU - pass_object_size 1:-\n\
GNU - patchable_function_entry 1:a=aarch64,aarch64_be,loongarch32,loongarch64,riscv32,riscv64,x86,x86_64,ppc,ppc64,ppc64le,systemz\n\
GNU - pcs 1:-\n\
GNU - personality 1:-\n\
GNU - pointer_field_protection 1:-\n\
GNU - pointer_with_type_tag 1:-\n\
GNU - preferred_name 1:-\n\
GNU - preferred_type 1:-\n\
GNU - preserve_access_index 1:a=bpfel,bpfeb\n\
GNU - preserve_all 1:-\n\
GNU - preserve_most 1:-\n\
GNU - preserve_none 1:a=aarch64,aarch64_be,aarch64_32,x86,x86_64\n\
GNU - preserve_static_offset 1:a=bpfel,bpfeb\n\
GNU - pt_guarded_by 1:-\n\
GNU - pt_guarded_var 1:-\n\
GNU - ptrauth_vtable_pointer 1:-\n\
GNU - pure 1:-\n\
GNU - randomize_layout 1:-\n\
GNU - reentrant_capability 1:-\n\
GNU - regcall 1:-\n\
GNU - regparm 1:-\n\
GNU - reinitializes 1:-\n\
GNU - release_capability 1:-\n\
GNU - release_generic_capability 1:-\n\
GNU - release_handle 1:-\n\
GNU - release_shared_capability 1:-\n\
GNU - reqd_work_group_size 1:-\n\
GNU - require_constant_initialization 1:-\n\
GNU - require_explicit_initialization 1:-\n\
GNU - requires_capability 1:-\n\
GNU - requires_shared_capability 1:-\n\
GNU - retain 1:-\n\
GNU - return_typestate 1:-\n\
GNU - returns_nonnull 1:-\n\
GNU - returns_twice 1:-\n\
GNU - riscv_rvv_vector_bits 1:-\n\
GNU - riscv_vector_cc 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
GNU - riscv_vls_cc 1:a=riscv32,riscv64,riscv32be,riscv64be\n\
GNU - scoped_lockable 1:-\n\
GNU - section 1:-\n\
GNU - selectany 1:-\n\
GNU - sentinel 1:-\n\
GNU - set_typestate 1:-\n\
GNU - shared 1:-\n\
GNU - shared_capability 1:-\n\
GNU - shared_lock_function 1:-\n\
GNU - shared_locks_required 1:-\n\
GNU - shared_trylock_function 1:-\n\
GNU - short_call 1:a=mips,mipsel,mips64,mips64el\n\
GNU - signal 1:a=avr\n\
GNU - sized_by 1:-\n\
GNU - sized_by_or_null 1:-\n\
GNU - speculative_load_hardening 1:-\n\
GNU - stack_protector_ignore 1:-\n\
GNU - standalone_debug 1:-\n\
GNU - stdcall 1:-\n\
GNU - suppress 1:-\n\
GNU - swift_async 1:-\n\
GNU - swift_async_context 1:-\n\
GNU - swift_async_error 1:-\n\
GNU - swift_async_name 1:-\n\
GNU - swift_attr 1:-\n\
GNU - swift_bridge 1:-\n\
GNU - swift_bridged_typedef 1:-\n\
GNU - swift_context 1:-\n\
GNU - swift_error 1:-\n\
GNU - swift_error_result 1:-\n\
GNU - swift_indirect_result 1:-\n\
GNU - swift_name 1:-\n\
GNU - swift_newtype 1:-\n\
GNU - swift_objc_members 1:-\n\
GNU - swift_private 1:-\n\
GNU - swift_wrapper 1:-\n\
GNU - swiftasynccall 1:-\n\
GNU - swiftcall 1:-\n\
GNU - sycl_kernel 1:-\n\
GNU - sycl_special_class 1:-\n\
GNU - sysv_abi 1:-\n\
GNU - target 1:-\n\
GNU - target_clones 1:-\n\
GNU - target_version 1:a=aarch64,aarch64_be,aarch64_32,riscv32,riscv64,riscv32be,riscv64be\n\
GNU - test_typestate 1:-\n\
GNU - thiscall 1:-\n\
GNU - tls_model 1:-\n\
GNU - transparent_union 1:-\n\
GNU - trivial_abi 1:-\n\
GNU - try_acquire_capability 1:-\n\
GNU - try_acquire_shared_capability 1:-\n\
GNU - type_tag_for_datatype 1:-\n\
GNU - type_visibility 1:-\n\
GNU - unavailable 1:-\n\
GNU - uninitialized 1:-\n\
GNU - unlock_function 1:-\n\
GNU - unsafe_buffer_usage 1:-\n\
GNU - unused 1:-\n\
GNU - use_handle 1:-\n\
GNU - used 1:-\n\
GNU - using_if_exists 1:-\n\
GNU - vec_type_hint 1:-\n\
GNU - vecreturn 1:-\n\
GNU - vector_size 1:-\n\
GNU - vectorcall 1:-\n\
GNU - visibility 1:-\n\
GNU - warn_unused 1:-\n\
GNU - warn_unused_result 1:-\n\
GNU - warning 1:-\n\
GNU - weak 1:-\n\
GNU - weak_import 1:-\n\
GNU - weakref 1:-\n\
GNU - work_group_size_hint 1:-\n\
GNU - xray_always_instrument 1:-\n\
GNU - xray_log_args 1:-\n\
GNU - xray_never_instrument 1:-\n\
GNU - zero_call_used_regs 1:-\n\
HLSLAnnotation - packoffset 1:-\n\
HLSLAnnotation - register 1:-\n\
Microsoft - RootSignature 1:-\n\
Microsoft - WaveSize 1:-\n\
Microsoft - branch 1:-\n\
Microsoft - flatten 1:-\n\
Microsoft - loop 1:-\n\
Microsoft - numthreads 1:-\n\
Microsoft - shader 1:-\n\
Microsoft - unroll 1:-\n\
Microsoft - uuid 1:-\n\
Pragma - init_seg 1:-\n\
Pragma - nounroll 1:-\n\
Pragma - nounroll_and_jam 1:-\n\
Pragma - unroll 1:-\n\
Pragma - unroll_and_jam 1:-\n\
Pragma acc routine 1:-\n\
Pragma clang loop 1:-\n\
Pragma omp declare simd 1:-\n\
Pragma omp declare target 1:-\n\
Pragma omp declare variant 1:-\n\
";

type AttributeKey = (&'static str, &'static str, &'static str);

type AttributeTable = BTreeMap<AttributeKey, Vec<(u64, &'static str)>>;

fn attribute_table() -> &'static AttributeTable {
    static TABLE: std::sync::OnceLock<AttributeTable> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = BTreeMap::new();
        for line in ATTRIBUTE_SPELLINGS.lines() {
            let mut words = line.split(' ');
            let (Some(variety), Some(scope), Some(name)) =
                (words.next(), words.next(), words.next())
            else {
                continue;
            };
            let alternatives = words
                .filter_map(|word| word.split_once(':'))
                .map(|(version, condition)| (version.parse().unwrap_or(1), condition))
                .collect();
            table.insert((variety, scope, name), alternatives);
        }
        table
    })
}

fn llvm_architecture(target: &TargetInfo) -> &'static str {
    target.triple.arch
}

fn object_format(target: &TargetInfo) -> &'static str {
    match target.triple.object_format {
        "coff" => "COFF",
        "macho" => "MachO",
        "xcoff" => "XCOFF",
        "goff" => "GOFF",
        "wasm" => "Wasm",
        _ => "ELF",
    }
}

fn target_condition_holds(condition: &str, target: &TargetInfo) -> bool {
    if condition == "-" {
        return true;
    }
    let architecture = llvm_architecture(target);
    let microsoft = target.is_windows() && !target.triple.environment.starts_with("gnu");
    condition.split('/').all(|part| match part.split_once('=') {
        Some(("a", list)) => list.split(',').any(|item| item == architecture),
        Some(("o", list)) => list
            .split(',')
            .any(|item| item == "Win32" && target.is_windows()),
        Some(("f", list)) => list.split(',').any(|item| item == object_format(target)),
        Some(("c", "msrecord")) | Some(("c", "msabi")) => microsoft,
        Some(("c", "itanium")) => !microsoft,
        Some(("c", "dllimport")) => target.is_windows() || target.triple.os.starts_with("ps"),
        Some(("c", "arm64ec")) => architecture == "arm64ec",
        Some(("c", "musttail")) => true,
        Some(("c", "ifunc")) => {
            object_format(target) == "MachO"
                || (target.is_windows() && architecture == "aarch64")
                || architecture == "avr"
                || (object_format(target) == "ELF"
                    && ((target.triple.os == "linux" && !target.triple.is_musl())
                        || target.triple.os == "freebsd"))
        }
        _ => false,
    })
}

fn attribute_value(variety: &str, scope: &str, name: &str, target: &TargetInfo) -> u64 {
    let scope = match scope {
        "__gnu__" => "gnu",
        "_Clang" => "clang",
        "" => "-",
        other => other,
    };
    let name = if name.len() >= 4 && name.starts_with("__") && name.ends_with("__") {
        &name[2..name.len() - 2]
    } else {
        name
    };
    let Some(alternatives) = attribute_table().get(&(variety, scope, name)) else {
        return 0;
    };
    if let [(version, condition)] = alternatives.as_slice() {
        return if target_condition_holds(condition, target) {
            *version
        } else {
            0
        };
    }
    u64::from(
        alternatives
            .iter()
            .any(|(_, condition)| target_condition_holds(condition, target)),
    )
}

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
    fn lex_header_name(&mut self, allow_expansion: bool) -> Result<Token, Token> {
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
                    return Err(token);
                }
                if token.has_leading_space() {
                    buffer.push(b' ');
                }
                buffer.extend_from_slice(&token.spelling);
            }
            let mut header = self.make_token(TokenKind::HeaderName, buffer, (open.site, open.site));
            header.set(START_OF_LINE, open.at_start_of_line());
            header.set(LEADING_SPACE, open.has_leading_space());
            return Ok(header);
        }
        if token.kind == TokenKind::String(LiteralPrefix::Plain)
            && allow_expansion
            && token.spelling.len() >= 2
        {
            token.kind = TokenKind::HeaderName;
        }
        Ok(token)
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

    fn handle_include(&mut self, hash: &Token, directive: &Token, kind: IncludeKind) {
        let Ok(filename_token) = self.lex_header_name(true) else {
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
        if let Some(place) = self.arc_cf_audited.take() {
            self.report_token(
                Message::AuditInclude,
                hash,
                vec!["0".to_string(), "arc_cf_code_audited".to_string()],
            );
            self.report(Message::PragmaEnteredHere, place, Vec::new());
        }
        if let Some(place) = self.assume_nonnull.take() {
            self.report_token(
                Message::AuditInclude,
                hash,
                vec!["0".to_string(), "assume_nonnull".to_string()],
            );
            self.report(Message::PragmaEnteredHere, place, Vec::new());
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
        let mut found = self.lookup_file(
            &name,
            angled,
            start_index,
            include_next,
            filename_token.site,
        );
        if found.is_none() && angled {
            found = self.lookup_file(&name, false, start_index, include_next, filename_token.site);
            if found.is_some() {
                let import = if kind == IncludeKind::Import {
                    "1"
                } else {
                    "0"
                };
                self.report_token(
                    Message::FileNotFoundAngled,
                    &filename_token,
                    vec![name.clone(), import.to_string()],
                );
            }
        }
        if found.is_none() && self.options.spell_checking {
            let corrected = name
                .trim_start_matches(|character: char| !character.is_ascii_alphanumeric())
                .trim_end_matches(|character: char| !character.is_ascii_alphanumeric())
                .to_string();
            found = self.lookup_file(
                &corrected,
                angled,
                start_index,
                include_next,
                filename_token.site,
            );
            if found.is_some() {
                self.report_token(
                    Message::FileNotFoundTypo,
                    &filename_token,
                    vec![name.clone(), corrected],
                );
            }
        }
        let Some((path, bytes, search_index, relative, character)) = found else {
            self.report_token(Message::FileNotFound, &filename_token, vec![name]);
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
        let string = self.lex_header_name(true).unwrap_or_else(|end| end);
        let mut name: Option<Rc<str>> = None;
        if string.kind == TokenKind::EndOfDirective {
        } else if string.kind != TokenKind::HeaderName {
            self.report_token(Message::LineInvalidFilename, &string, Vec::new());
            self.discard_until_end_of_directive();
            return;
        } else {
            name = Some(self.line_directive_filename(&string));
            self.check_end_of_directive("line", true);
        }
        let character = self.sources.presumed(digit.site).character;
        if let Some(place) = self.current_position() {
            self.sources
                .add_line_entry(place, line, name, 0, character, digit.site);
            self.file_changed(place, ChangeReason::Rename);
        }
    }

    fn line_directive_filename(&mut self, token: &Token) -> Rc<str> {
        let spelling = &token.spelling;
        if spelling.len() < 2 || spelling[0] != b'"' || spelling[spelling.len() - 1] != b'"' {
            self.report_token(Message::LineInvalidFilename, token, Vec::new());
            return Rc::from("");
        }
        Rc::from(String::from_utf8_lossy(&spelling[1..spelling.len() - 1]).as_ref())
    }

    fn handle_digit_directive(&mut self, digit: Token) {
        let Some(line) = self.line_value(&digit, Message::LinemarkerRequiresInteger, true) else {
            return;
        };
        let string = self.lex_header_name(true).unwrap_or_else(|end| end);
        let mut entry = false;
        let mut exit = false;
        let mut name: Option<Rc<str>> = None;
        let mut character = FileCharacter::User;
        if string.kind == TokenKind::EndOfDirective {
            self.report_token(Message::GnuLineMarker, &string, Vec::new());
            character = self.sources.presumed(digit.site).character;
        } else if string.kind != TokenKind::HeaderName {
            self.report_token(Message::LinemarkerInvalidFilename, &string, Vec::new());
            self.discard_until_end_of_directive();
            return;
        } else {
            let value = self.line_directive_filename(&string);
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
                name = Some(value);
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
                    "assume_nonnull" | "arc_cf_code_audited" if name == "clang" => {
                        self.pragma_audit(&second)
                    }
                    "deprecated" | "restrict_expansion" | "final" if name == "clang" => {
                        self.pragma_macro_annotation(&sub)
                    }
                    "unsafe_buffer_usage" | "module" | "__set_pp_state" if name == "clang" => {
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
            "omp" => {
                let second = self.lex_unexpanded();
                self.unknown_pragma("#pragma omp", second, true, true)
            }
            "STDC" if self.parser_pragmas_active() => self.pragma_stdc(),
            "STDC" => {
                let known = true;
                self.unknown_pragma("#pragma", first, known, false);
            }
            "comment" if self.parser_pragmas_active() => self.pragma_comment(&first),
            _ => {
                let known = PARSER_PRAGMAS.contains(&name.as_str());
                let expand = self.language.microsoft_extensions;
                self.unknown_pragma("#pragma", first, known, expand);
            }
        }
    }

    fn pragma_macro_annotation(&mut self, kind: &str) {
        let open = self.lex();
        if open.kind != TokenKind::LeftParen {
            self.report_token(Message::Expected, &open, vec!["(".to_string()]);
            return;
        }
        let name = self.lex_unexpanded();
        if name.kind != TokenKind::Identifier {
            self.report_token(Message::Expected, &name, vec!["identifier".to_string()]);
            return;
        }
        if !self.macros.contains_key(&name.spelling) {
            self.report_token(
                Message::VisibilityNonMacro,
                &name,
                vec![quoted(&name.spelling)],
            );
            return;
        }
        let mut token = self.lex();
        let mut message = Vec::new();
        if kind != "final" && token.kind == TokenKind::Comma {
            token = self.lex();
            let context = format!("#pragma clang {kind}");
            match self.finish_string_literal(&mut token, &context, true) {
                Some(value) => message = value,
                None => return,
            }
        }
        if token.kind != TokenKind::RightParen {
            self.report_token(Message::Expected, &token, vec![")".to_string()]);
            return;
        }
        let annotations = self
            .macro_annotations
            .entry(name.spelling.clone())
            .or_default();
        match kind {
            "deprecated" => annotations.deprecated = Some((message, token.site)),
            "restrict_expansion" => annotations.restricted = Some((message, token.site)),
            _ => annotations.final_place = Some(token.site),
        }
    }

    fn emit_macro_expansion_warnings(&mut self, identifier: &Token, is_ifndef: bool) {
        let annotations = self.macro_annotations.get(&identifier.spelling).cloned();
        if let Some(annotations) = annotations {
            if let Some((message, place)) = annotations.deprecated {
                self.report_annotated_use(
                    Message::PragmaDeprecatedMacroUse,
                    identifier,
                    &message,
                    place,
                    "0",
                );
            }
            if let Some((message, place)) = annotations.restricted {
                if !self.in_main_file(identifier.site) {
                    self.report_annotated_use(
                        Message::PragmaRestrictExpansionMacroUse,
                        identifier,
                        &message,
                        place,
                        "1",
                    );
                }
            }
        }
        if !is_ifndef {
            if &*identifier.spelling == b"INFINITY" && self.options.no_honor_infinities {
                self.report_token(
                    Message::NanInfinityDisabled,
                    identifier,
                    vec!["0".to_string(), "1".to_string()],
                );
            }
            if &*identifier.spelling == b"NAN" && self.options.no_honor_nans {
                self.report_token(
                    Message::NanInfinityDisabled,
                    identifier,
                    vec!["1".to_string(), "1".to_string()],
                );
            }
        }
    }

    fn report_annotated_use(
        &mut self,
        message: Message,
        identifier: &Token,
        text: &[u8],
        place: Place,
        selection: &str,
    ) {
        let arguments = if text.is_empty() {
            vec![quoted(&identifier.spelling), "0".to_string()]
        } else {
            vec![
                quoted(&identifier.spelling),
                "1".to_string(),
                String::from_utf8_lossy(text).into_owned(),
            ]
        };
        self.report_token(message, identifier, arguments);
        self.report(
            Message::MacroAnnotationNote,
            place,
            vec![selection.to_string()],
        );
    }

    fn emit_final_macro_warning(&mut self, identifier: &Token, undefining: bool) {
        let Some(place) = self
            .macro_annotations
            .get(&identifier.spelling)
            .and_then(|annotations| annotations.final_place)
        else {
            return;
        };
        self.report_token(
            Message::PragmaFinalMacro,
            identifier,
            vec![
                quoted(&identifier.spelling),
                if undefining { "0" } else { "1" }.to_string(),
            ],
        );
        self.report(Message::MacroAnnotationNote, place, vec!["2".to_string()]);
    }

    fn parser_pragmas_active(&self) -> bool {
        self.options.syntax_only && self.printer.is_none()
    }

    fn push_pragma_annotation(&mut self, at: &Token, value: &str) {
        let mut annotation = Token::new(
            TokenKind::PragmaAnnotation,
            Rc::from(value.as_bytes()),
            at.site,
            0,
        );
        annotation.flags = at.flags & (START_OF_LINE | LEADING_SPACE);
        self.push_tokens(vec![annotation], true);
    }

    fn lex_on_off_switch(&mut self) -> Option<&'static str> {
        let token = self.lex_unexpanded();
        let value = if token.kind != TokenKind::Identifier {
            None
        } else {
            match token.text() {
                "ON" => Some("ON"),
                "OFF" => Some("OFF"),
                "DEFAULT" => Some("DEFAULT"),
                _ => None,
            }
        };
        let Some(value) = value else {
            self.report_token(Message::OnOffSwitchSyntax, &token, Vec::new());
            return None;
        };
        let next = self.lex_unexpanded();
        if next.kind != TokenKind::EndOfDirective {
            self.report_token(Message::PragmaSyntaxEndOfDirective, &next, Vec::new());
        }
        Some(value)
    }

    fn pragma_stdc(&mut self) {
        let name = self.lex_unexpanded();
        let spelled = if name.kind == TokenKind::Identifier {
            name.text().to_string()
        } else {
            String::new()
        };
        match spelled.as_str() {
            "FP_CONTRACT" | "CX_LIMITED_RANGE" => {
                if let Some(value) = self.lex_on_off_switch() {
                    let annotation = format!("{} {value}", spelled.to_ascii_lowercase());
                    self.push_pragma_annotation(&name, &annotation);
                }
            }
            "FENV_ACCESS" => {
                if !self.options.target.has_strict_floating_point() {
                    self.report_token(
                        Message::PragmaFloatingPointIgnored,
                        &name,
                        vec![spelled.clone()],
                    );
                    return;
                }
                if let Some(value) = self.lex_on_off_switch() {
                    self.push_pragma_annotation(&name, &format!("fenv_access {value}"));
                }
            }
            "FENV_ROUND" => self.pragma_stdc_rounding(&name),
            _ => self.report_token(Message::StdcPragmaIgnored, &name, Vec::new()),
        }
    }

    fn pragma_stdc_rounding(&mut self, name: &Token) {
        if !self.options.target.has_strict_floating_point() {
            self.report_token(
                Message::PragmaFloatingPointIgnored,
                name,
                vec!["FENV_ROUND".to_string()],
            );
            return;
        }
        let mode = self.lex();
        if mode.kind != TokenKind::Identifier {
            self.report_token(
                Message::PragmaExpectedIdentifier,
                &mode,
                vec!["FENV_ROUND".to_string()],
            );
            return;
        }
        if !matches!(
            mode.text(),
            "FE_TOWARDZERO"
                | "FE_TONEAREST"
                | "FE_UPWARD"
                | "FE_DOWNWARD"
                | "FE_TONEARESTFROMZERO"
                | "FE_DYNAMIC"
        ) {
            self.report_token(Message::StdcUnknownRoundingMode, &mode, Vec::new());
            return;
        }
        let end = self.lex();
        if end.kind != TokenKind::EndOfDirective {
            self.report_token(
                Message::PragmaExtraTokensAtEnd,
                &end,
                vec!["STDC FENV_ROUND".to_string()],
            );
            return;
        }
        self.report_token(Message::StdcFenvRoundNotSupported, &end, Vec::new());
        let annotation = format!("fenv_round {}", mode.text());
        self.push_pragma_annotation(&end, &annotation);
    }

    fn pragma_comment(&mut self, introducer: &Token) {
        let open = self.lex();
        if open.kind != TokenKind::LeftParen {
            self.report_token(Message::PragmaCommentMalformed, introducer, Vec::new());
            return;
        }
        let kind = self.lex();
        if kind.kind != TokenKind::Identifier {
            self.report_token(Message::PragmaCommentMalformed, introducer, Vec::new());
            return;
        }
        let name = kind.text().to_string();
        if !matches!(
            name.as_str(),
            "linker" | "lib" | "compiler" | "exestr" | "user" | "copyright"
        ) {
            self.report_token(Message::PragmaCommentUnknownKind, &kind, Vec::new());
            return;
        }
        let triple = &self.options.target.triple;
        if (triple.os == "aix" && name != "copyright") || (triple.is_elf() && name != "lib") {
            self.report_token(Message::PragmaCommentIgnored, &kind, vec![name]);
            return;
        }
        if name == "copyright" {
            if self.seen_copyright_comment {
                self.report_token(Message::PragmaCommentOnce, &kind, vec![name]);
                return;
            }
            self.seen_copyright_comment = true;
        }
        let mut token = self.lex();
        let mut argument = Vec::new();
        if token.kind == TokenKind::Comma {
            token = self.lex();
            match self.finish_string_literal(&mut token, "pragma comment", true) {
                Some(value) => argument = value,
                None => return,
            }
        }
        if token.kind != TokenKind::RightParen {
            self.report_token(Message::PragmaCommentMalformed, &token, Vec::new());
            return;
        }
        let end = self.lex();
        if end.kind != TokenKind::EndOfDirective {
            self.report_token(Message::PragmaCommentMalformed, &end, Vec::new());
            return;
        }
        if name == "copyright" && argument.is_empty() {
            return;
        }
        self.linker_comments.push((name, argument));
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
        if self.parser_pragmas_active() && known {
            let annotation = format!("unsupported {prefix} {}", first.text());
            if first.kind != TokenKind::EndOfDirective && self.in_directive() {
                self.discard_until_end_of_directive();
            }
            self.push_pragma_annotation(&first, &annotation);
            return;
        }
        if self.options.syntax_only && !known {
            self.report_token(Message::UnknownPragma, &first, Vec::new());
        }
        if first.kind != TokenKind::EndOfDirective && self.in_directive() {
            self.discard_until_end_of_directive();
        }
    }

    fn pragma_audit(&mut self, name: &Token) {
        let kind = name.text().to_string();
        let token = self.lex_unexpanded();
        let begin = if token.is_identifier("begin") {
            true
        } else if token.is_identifier("end") {
            false
        } else {
            self.report_token(Message::AuditSyntax, &token, Vec::new());
            if token.kind != TokenKind::EndOfDirective && self.in_directive() {
                self.discard_until_end_of_directive();
            }
            return;
        };
        let after = self.lex_unexpanded();
        if after.kind != TokenKind::EndOfDirective {
            self.report_token(
                Message::ExtraTokensAtEndOfDirective,
                &after,
                vec!["pragma".to_string()],
            );
            if self.in_directive() {
                self.discard_until_end_of_directive();
            }
        }
        let held = if kind == "assume_nonnull" {
            self.assume_nonnull
        } else {
            self.arc_cf_audited
        };
        let replacement = if begin {
            if let Some(previous) = held {
                self.report_token(Message::AuditDoubleBegin, name, vec![kind.clone()]);
                self.report(Message::PragmaEnteredHere, previous, Vec::new());
            }
            Some(name.site)
        } else {
            if held.is_none() {
                self.report_token(Message::AuditUnmatchedEnd, name, vec![kind.clone()]);
                return;
            }
            None
        };
        if kind == "assume_nonnull" {
            self.assume_nonnull = replacement;
            if self.printer.is_some() {
                self.printer_move_to_line(name.site, true);
                if let Some(printer) = &mut self.printer {
                    let text = if begin {
                        "#pragma clang assume_nonnull begin"
                    } else {
                        "#pragma clang assume_nonnull end"
                    };
                    printer.out.extend_from_slice(text.as_bytes());
                    printer.directive_on_line = true;
                }
            }
        } else {
            self.arc_cf_audited = replacement;
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
        let Ok(token) = self.lex_header_name(false) else {
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
        if self
            .lookup_file(&name, angled, None, false, token.site)
            .is_none()
        {
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
                if matches!(token.kind, TokenKind::EndOfFile | TokenKind::EndOfDirective) {
                    return (None, token);
                }
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
                text = self
                    .options
                    .target
                    .floating_point_evaluation_method()
                    .to_string()
                    .into_bytes();
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
                let value = match value {
                    Some(value) => value,
                    None if end.kind == TokenKind::RightParen => false,
                    None => return Some(end),
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
            Builtin::IsIdentifier => u64::from(
                token.kind == TokenKind::Identifier
                    && keyword_kind(&self.language, &token.spelling).is_none(),
            ),
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
                let target = &self.options.target;
                match name.as_str() {
                    "__builtin_cpu_is" => return u64::from(target.supports_cpu_is()),
                    "__builtin_cpu_init" => return u64::from(target.supports_cpu_init()),
                    "__builtin_cpu_supports" => return u64::from(target.supports_cpu_supports()),
                    _ => {}
                }
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
                attribute_value("GNU", "", &name, &self.options.target)
            }
            Builtin::HasDeclspecAttribute => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                u64::from(
                    self.language.declspec_keyword
                        && attribute_value("Declspec", "", &name, &self.options.target) != 0,
                )
            }
            Builtin::HasCAttribute => {
                let Some(first) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                let next = self.lex_unexpanded();
                let mut scope = String::new();
                let mut name = first;
                if next.kind != TokenKind::ColonColon {
                    *token = next;
                    *lexed = true;
                } else {
                    scope = name.clone();
                    let after = self.lex();
                    *token = after.clone();
                    let Some(attribute) = self.expect_feature_identifier(&after) else {
                        return 0;
                    };
                    name = attribute;
                }
                attribute_value("C23", &scope, &name, &self.options.target)
            }
            Builtin::IsTargetArchitecture => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                u64::from(self.options.target.is_target_architecture(&name))
            }
            Builtin::IsTargetVendor => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                u64::from(self.options.target.is_target_vendor(&name))
            }
            Builtin::IsTargetSystem => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                u64::from(self.options.target.is_target_system(&name))
            }
            Builtin::IsTargetEnvironment => {
                let Some(name) = self.expect_feature_identifier(token) else {
                    return 0;
                };
                u64::from(self.options.target.is_target_environment(&name))
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
        if !self.parsing_if_directive {
            self.report_token(
                Message::DirectiveRequired,
                identifier,
                vec![quoted(&identifier.spelling)],
            );
            return (None, identifier.clone());
        }
        let mut token = match self.lex_header_name(true) {
            Ok(token) => token,
            Err(end) => return (None, end),
        };
        while token.kind == TokenKind::Comment {
            token = match self.lex_header_name(true) {
                Ok(token) => token,
                Err(end) => return (None, end),
            };
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
                Ok(token) => token,
                Err(end) => return (None, end),
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
            self.report_token(Message::EmptyFilename, &filename, Vec::new());
            return (Some(false), close);
        }
        let found = self
            .lookup_file(&name, angled, start, include_next, filename.site)
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
        let filename = match self.lex_header_name(true) {
            Ok(filename) => filename,
            Err(end) => return (None, end),
        };
        if filename.kind != TokenKind::HeaderName {
            self.report_token(Message::ExpectsFilename, &filename, Vec::new());
            return (None, filename);
        }
        let (parameters, current) = self.lex_embed_parameters(true);
        let Some(parameters) = parameters else {
            return (None, current);
        };
        if current.kind != TokenKind::RightParen {
            let mut end = filename.clone();
            if end.site == end.place {
                end.site.offset = end.site.offset.wrapping_add(filename.length);
            }
            self.report_token(
                Message::MissingAfter,
                &end,
                vec![quoted(&identifier.spelling), "')'".to_string()],
            );
            self.report_token(Message::ToMatch, &open, vec!["'('".to_string()]);
            let current = self.skip_to_end_of_directive(&current);
            return (None, current);
        }
        if parameters.unrecognized > 0 {
            return (Some(0), current);
        }
        let Some((name, angled)) = Self::include_filename(&filename) else {
            return (Some(0), current);
        };
        if name.is_empty() {
            self.report_token(Message::EmptyFilename, &filename, Vec::new());
        }
        let Some(bytes) = self.lookup_embed_file(&name, angled) else {
            return (Some(0), current);
        };
        let mut size = bytes.len() as u64;
        if let Some(offset) = parameters.offset {
            size = size.saturating_sub(offset);
        }
        if let Some(limit) = parameters.limit {
            size = if limit > size { 0 } else { limit };
        }
        if size == 0 {
            (Some(2), current)
        } else {
            (Some(1), current)
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

    pub fn begin(&mut self, main_name: &str, main_bytes: Vec<u8>) {
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
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            let token = self.lex();
            if token.kind != TokenKind::Comment {
                return token;
            }
        }
    }

    pub fn run(mut self, main_name: &str, main_bytes: Vec<u8>) -> PreprocessOutcome {
        self.begin(main_name, main_bytes);
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
        let mut outcome = self.finish();
        outcome.tokens = tokens;
        outcome
    }

    pub fn finish(mut self) -> PreprocessOutcome {
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
            tokens: Vec::new(),
            diagnostics: std::mem::take(&mut self.engine.diagnostics),
            comments: std::mem::take(&mut self.comments),
            errors: self.engine.errors,
            warnings: self.engine.warnings,
            main_buffer: self.main_buffer,
            fatal: self.engine.fatal_occurred,
            unknown_warning_flags: std::mem::take(&mut self.engine.unknown_flags),
            sources: std::mem::take(&mut self.sources),
            linker_comments: std::mem::take(&mut self.linker_comments),
        }
    }

    fn predefines_text(&self) -> Vec<u8> {
        let mut text = String::from("# 1 \"<built-in>\" 3\n");
        text.push_str(&predefined_macro_text(&self.options));
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

fn define_float_macros(
    builder: &mut MacroBuilder,
    prefix: &str,
    format: FloatFormat,
    suffix: &str,
) {
    let normal_maximum = format.pick([
        "6.5504e+4",
        "3.40282347e+38",
        "1.7976931348623157e+308",
        "1.18973149535723176502e+4932",
        "8.98846567431157953864652595394501e+307",
        "1.18973149535723176508575932662800702e+4932",
    ]);
    let denormal_minimum = format.pick([
        "5.9604644775390625e-8",
        "1.40129846e-45",
        "4.9406564584124654e-324",
        "3.64519953188247460253e-4951",
        "4.94065645841246544176568792868221e-324",
        "6.47517511943802511092443895822764655e-4966",
    ]);
    let digits = format.pick([3, 6, 15, 18, 31, 33]);
    let decimal_digits = format.pick([5, 9, 17, 21, 33, 36]);
    let epsilon = format.pick([
        "9.765625e-4",
        "1.19209290e-7",
        "2.2204460492503131e-16",
        "1.08420217248550443401e-19",
        "4.94065645841246544176568792868221e-324",
        "1.92592994438723585305597794258492732e-34",
    ]);
    let mantissa_digits = format.pick([11, 24, 53, 64, 106, 113]);
    let minimum_decimal_exponent = format.pick([-4, -37, -307, -4931, -291, -4931]);
    let maximum_decimal_exponent = format.pick([4, 38, 308, 4932, 308, 4932]);
    let minimum_exponent = format.pick([-13, -125, -1021, -16381, -968, -16381]);
    let maximum_exponent = format.pick([16, 128, 1024, 16384, 1024, 16384]);
    let minimum = format.pick([
        "6.103515625e-5",
        "1.17549435e-38",
        "2.2250738585072014e-308",
        "3.36210314311209350626e-4932",
        "2.00416836000897277799610805135016e-292",
        "3.36210314311209350626267781732175260e-4932",
    ]);
    let maximum = format.pick([
        "6.5504e+4",
        "3.40282347e+38",
        "1.7976931348623157e+308",
        "1.18973149535723176502e+4932",
        "1.79769313486231580793728971405301e+308",
        "1.18973149535723176508575932662800702e+4932",
    ]);
    let head = format!("__{prefix}_");
    builder.define(
        &format!("{head}DENORM_MIN__"),
        &format!("{denormal_minimum}{suffix}"),
    );
    builder.define(
        &format!("{head}NORM_MAX__"),
        &format!("{normal_maximum}{suffix}"),
    );
    builder.define_one(&format!("{head}HAS_DENORM__"));
    builder.define(&format!("{head}DIG__"), &digits.to_string());
    builder.define(&format!("{head}DECIMAL_DIG__"), &decimal_digits.to_string());
    builder.define(&format!("{head}EPSILON__"), &format!("{epsilon}{suffix}"));
    builder.define_one(&format!("{head}HAS_INFINITY__"));
    builder.define_one(&format!("{head}HAS_QUIET_NAN__"));
    builder.define(&format!("{head}MANT_DIG__"), &mantissa_digits.to_string());
    builder.define(
        &format!("{head}MAX_10_EXP__"),
        &maximum_decimal_exponent.to_string(),
    );
    builder.define(&format!("{head}MAX_EXP__"), &maximum_exponent.to_string());
    builder.define(&format!("{head}MAX__"), &format!("{maximum}{suffix}"));
    builder.define(
        &format!("{head}MIN_10_EXP__"),
        &format!("({minimum_decimal_exponent})"),
    );
    builder.define(
        &format!("{head}MIN_EXP__"),
        &format!("({minimum_exponent})"),
    );
    builder.define(&format!("{head}MIN__"), &format!("{minimum}{suffix}"));
}

fn type_maximum(width: u32, signed: bool) -> u128 {
    if signed {
        (1u128 << (width - 1)) - 1
    } else {
        (1u128 << width) - 1
    }
}

fn define_type_size(builder: &mut MacroBuilder, name: &str, ty: IntegerType, target: &TargetInfo) {
    let value = type_maximum(target.type_width(ty), ty.signed());
    builder.define(name, &format!("{value}{}", target.type_constant_suffix(ty)));
}

fn define_type_minimum(
    builder: &mut MacroBuilder,
    prefix: &str,
    ty: IntegerType,
    target: &TargetInfo,
) {
    let value = if ty.signed() {
        format!("(-{prefix}_MAX__ - 1)")
    } else {
        format!("0{}", target.type_constant_suffix(ty))
    };
    builder.define(&format!("{prefix}_MIN__"), &value);
}

fn define_format(builder: &mut MacroBuilder, c23: bool, prefix: &str, ty: IntegerType) {
    let modifier = ty.format_modifier();
    let letters = if ty.signed() { "di" } else { "ouxX" };
    let mut emit = |letter: char| {
        builder.define(
            &format!("{prefix}_FMT{letter}__"),
            &format!("\"{modifier}{letter}\""),
        );
    };
    letters.chars().for_each(&mut emit);
    if c23 && !ty.signed() {
        "bB".chars().for_each(&mut emit);
    }
}

fn define_type_size_and_width(
    builder: &mut MacroBuilder,
    prefix: &str,
    ty: IntegerType,
    target: &TargetInfo,
) {
    define_type_size(builder, &format!("{prefix}_MAX__"), ty, target);
    builder.define(
        &format!("{prefix}_WIDTH__"),
        &target.type_width(ty).to_string(),
    );
}

fn define_constant_suffix(builder: &mut MacroBuilder, prefix: &str, suffix: &str) {
    builder.define(&format!("{prefix}_C_SUFFIX__"), suffix);
    let body = if suffix.is_empty() {
        "c".to_string()
    } else {
        format!("c##{suffix}")
    };
    builder.define(&format!("{prefix}_C(c)"), &body);
}

fn define_exact_width_type(
    builder: &mut MacroBuilder,
    c23: bool,
    ty: IntegerType,
    target: &TargetInfo,
) {
    let width = target.type_width(ty);
    let signed = ty.signed();
    let mut actual = ty;
    if width == 64 {
        actual = if signed {
            target.int64_type
        } else {
            target.int64_type.unsigned_counterpart()
        };
    }
    if width == 16 {
        actual = if signed {
            target.int16_type
        } else {
            target.int16_type.unsigned_counterpart()
        };
    }
    let prefix = format!("{}{width}", if signed { "__INT" } else { "__UINT" });
    builder.define(&format!("{prefix}_TYPE__"), actual.name());
    define_format(builder, c23, &prefix, actual);
    define_constant_suffix(builder, &prefix, target.type_constant_suffix(actual));
}

fn define_exact_width_type_size(builder: &mut MacroBuilder, ty: IntegerType, target: &TargetInfo) {
    let width = target.type_width(ty);
    let signed = ty.signed();
    let actual = if width == 64 {
        if signed {
            target.int64_type
        } else {
            target.int64_type.unsigned_counterpart()
        }
    } else {
        ty
    };
    let prefix = if signed { "__INT" } else { "__UINT" };
    define_type_size(builder, &format!("{prefix}{width}_MAX__"), actual, target);
}

fn define_least_width_type(
    builder: &mut MacroBuilder,
    c23: bool,
    head: &str,
    width: u32,
    signed: bool,
    target: &TargetInfo,
) {
    let Some(ty) = target.least_integer_type(width, signed) else {
        return;
    };
    let prefix = format!("{}{head}{width}", if signed { "__INT_" } else { "__UINT_" });
    builder.define(&format!("{prefix}_TYPE__"), ty.name());
    if signed {
        define_type_size_and_width(builder, &prefix, ty, target);
    } else {
        define_type_size(builder, &format!("{prefix}_MAX__"), ty, target);
    }
    define_format(builder, c23, &prefix, ty);
}

fn lock_free_value(width: u32, target: &TargetInfo) -> &'static str {
    if target.has_builtin_atomic(width, width) {
        "2"
    } else {
        "1"
    }
}

fn initialize_predefined_macros(options: &PreprocessorOptions, builder: &mut MacroBuilder) {
    let target = &options.target;
    let language = &options.language;
    let c23 = language.at_least(CStandard::C23);
    builder.define_one("__llvm__");
    builder.define_one("__clang__");
    builder.define("__clang_major__", "24");
    builder.define("__clang_minor__", "0");
    builder.define("__clang_patchlevel__", "0");
    builder.define("__clang_version__", "\"24.0.0 \"");
    if let Some((major, minor, patch)) = options.gnuc_version {
        builder.define("__GNUC__", &major.to_string());
        builder.define("__GNUC_MINOR__", &minor.to_string());
        builder.define("__GNUC_PATCHLEVEL__", &patch.to_string());
        builder.define("__GXX_ABI_VERSION", "1002");
    }
    for (name, value) in [
        ("__ATOMIC_RELAXED", "0"),
        ("__ATOMIC_CONSUME", "1"),
        ("__ATOMIC_ACQUIRE", "2"),
        ("__ATOMIC_RELEASE", "3"),
        ("__ATOMIC_ACQ_REL", "4"),
        ("__ATOMIC_SEQ_CST", "5"),
        ("__MEMORY_SCOPE_SYSTEM", "0"),
        ("__MEMORY_SCOPE_DEVICE", "1"),
        ("__MEMORY_SCOPE_WRKGRP", "2"),
        ("__MEMORY_SCOPE_WVFRNT", "3"),
        ("__MEMORY_SCOPE_SINGLE", "4"),
        ("__MEMORY_SCOPE_CLUSTR", "5"),
        ("__OPENCL_MEMORY_SCOPE_WORK_ITEM", "0"),
        ("__OPENCL_MEMORY_SCOPE_WORK_GROUP", "1"),
        ("__OPENCL_MEMORY_SCOPE_DEVICE", "2"),
        ("__OPENCL_MEMORY_SCOPE_ALL_SVM_DEVICES", "3"),
        ("__OPENCL_MEMORY_SCOPE_SUB_GROUP", "4"),
        ("__FPCLASS_SNAN", "0x0001"),
        ("__FPCLASS_QNAN", "0x0002"),
        ("__FPCLASS_NEGINF", "0x0004"),
        ("__FPCLASS_NEGNORMAL", "0x0008"),
        ("__FPCLASS_NEGSUBNORMAL", "0x0010"),
        ("__FPCLASS_NEGZERO", "0x0020"),
        ("__FPCLASS_POSZERO", "0x0040"),
        ("__FPCLASS_POSSUBNORMAL", "0x0080"),
        ("__FPCLASS_POSNORMAL", "0x0100"),
        ("__FPCLASS_POSINF", "0x0200"),
        ("__PRAGMA_REDEFINE_EXTNAME", "1"),
        ("__VERSION__", "\"Clang 24.0.0\""),
    ] {
        builder.define(name, value);
    }
    if !language.gnu {
        builder.define_one("__STRICT_ANSI__");
    }
    builder.define("__OBJC_BOOL_IS_BOOL", "0");
    builder.define_one("__CONSTANT_CFSTRINGS__");
    if options.exceptions {
        builder.define_one("__EXCEPTIONS");
    }
    builder.define("__clang_literal_encoding__", "\"UTF-8\"");
    if target.type_width(target.wchar_type) >= 32 {
        builder.define("__clang_wide_literal_encoding__", "\"UTF-32\"");
    } else {
        builder.define("__clang_wide_literal_encoding__", "\"UTF-16\"");
    }
    if options.optimize != 0 {
        builder.define_one("__OPTIMIZE__");
    }
    if options.optimize_size {
        builder.define_one("__OPTIMIZE_SIZE__");
    }
    if options.fast_math {
        builder.define_one("__FAST_MATH__");
    }
    builder.define("__ORDER_LITTLE_ENDIAN__", "1234");
    builder.define("__ORDER_BIG_ENDIAN__", "4321");
    builder.define("__ORDER_PDP_ENDIAN__", "3412");
    if target.is_big_endian() {
        builder.define("__BYTE_ORDER__", "__ORDER_BIG_ENDIAN__");
        builder.define_one("__BIG_ENDIAN__");
    } else {
        builder.define("__BYTE_ORDER__", "__ORDER_LITTLE_ENDIAN__");
        builder.define_one("__LITTLE_ENDIAN__");
    }
    if target.pointer_width == 64 && target.long_width == 64 && target.int_width == 32 {
        builder.define_one("_LP64");
        builder.define_one("__LP64__");
    }
    if target.pointer_width == 32 && target.long_width == 32 && target.int_width == 32 {
        builder.define_one("_ILP32");
        builder.define_one("__ILP32__");
    }
    builder.define("__CHAR_BIT__", &target.char_width().to_string());
    builder.define("__BOOL_WIDTH__", "1");
    builder.define("__SHRT_WIDTH__", &target.short_width.to_string());
    builder.define("__INT_WIDTH__", &target.int_width.to_string());
    builder.define("__LONG_WIDTH__", &target.long_width.to_string());
    builder.define("__LLONG_WIDTH__", &target.long_long_width.to_string());
    builder.define(
        "__BITINT_MAXWIDTH__",
        &target.max_bit_int_width().to_string(),
    );
    define_type_size(builder, "__SCHAR_MAX__", IntegerType::SignedChar, target);
    define_type_size(builder, "__SHRT_MAX__", IntegerType::Short, target);
    define_type_size(builder, "__INT_MAX__", IntegerType::Int, target);
    define_type_size(builder, "__LONG_MAX__", IntegerType::Long, target);
    define_type_size(builder, "__LONG_LONG_MAX__", IntegerType::LongLong, target);
    define_type_size_and_width(builder, "__WCHAR", target.wchar_type, target);
    define_type_minimum(builder, "__WCHAR", target.wchar_type, target);
    define_type_size_and_width(builder, "__WINT", target.wint_type, target);
    define_type_minimum(builder, "__WINT", target.wint_type, target);
    define_type_size_and_width(builder, "__INTMAX", target.intmax_type, target);
    define_type_size_and_width(builder, "__SIZE", target.size_type, target);
    let unsigned_intmax = target.intmax_type.unsigned_counterpart();
    let unsigned_intptr = target.intptr_type.unsigned_counterpart();
    define_type_size_and_width(builder, "__UINTMAX", unsigned_intmax, target);
    define_type_size_and_width(builder, "__PTRDIFF", target.ptrdiff_type, target);
    define_type_size_and_width(builder, "__INTPTR", target.intptr_type, target);
    define_type_size_and_width(builder, "__UINTPTR", unsigned_intptr, target);
    let size_of = |width: u32| (width / target.char_width()).to_string();
    builder.define("__SIZEOF_DOUBLE__", &size_of(target.double_width));
    builder.define("__SIZEOF_FLOAT__", &size_of(target.float_width));
    builder.define("__SIZEOF_INT__", &size_of(target.int_width));
    builder.define("__SIZEOF_LONG__", &size_of(target.long_width));
    builder.define("__SIZEOF_LONG_DOUBLE__", &size_of(target.long_double_width));
    builder.define("__SIZEOF_LONG_LONG__", &size_of(target.long_long_width));
    builder.define("__SIZEOF_POINTER__", &size_of(target.pointer_width));
    builder.define("__SIZEOF_SHORT__", &size_of(target.short_width));
    builder.define(
        "__SIZEOF_PTRDIFF_T__",
        &size_of(target.type_width(target.ptrdiff_type)),
    );
    builder.define(
        "__SIZEOF_SIZE_T__",
        &size_of(target.type_width(target.size_type)),
    );
    builder.define(
        "__SIZEOF_WCHAR_T__",
        &size_of(target.type_width(target.wchar_type)),
    );
    builder.define(
        "__SIZEOF_WINT_T__",
        &size_of(target.type_width(target.wint_type)),
    );
    if target.has_int128() {
        builder.define("__SIZEOF_INT128__", &size_of(128));
    }
    builder.define("__INTMAX_TYPE__", target.intmax_type.name());
    define_format(builder, c23, "__INTMAX", target.intmax_type);
    define_constant_suffix(
        builder,
        "__INTMAX",
        target.type_constant_suffix(target.intmax_type),
    );
    builder.define("__UINTMAX_TYPE__", unsigned_intmax.name());
    define_format(builder, c23, "__UINTMAX", unsigned_intmax);
    define_constant_suffix(
        builder,
        "__UINTMAX",
        target.type_constant_suffix(unsigned_intmax),
    );
    builder.define("__PTRDIFF_TYPE__", target.ptrdiff_type.name());
    define_format(builder, c23, "__PTRDIFF", target.ptrdiff_type);
    builder.define("__INTPTR_TYPE__", target.intptr_type.name());
    define_format(builder, c23, "__INTPTR", target.intptr_type);
    builder.define("__SIZE_TYPE__", target.size_type.name());
    define_format(builder, c23, "__SIZE", target.size_type);
    builder.define("__WCHAR_TYPE__", target.wchar_type.name());
    builder.define("__WINT_TYPE__", target.wint_type.name());
    builder.define("__SIG_ATOMIC_TYPE__", target.sig_atomic_type.name());
    define_type_size_and_width(builder, "__SIG_ATOMIC", target.sig_atomic_type, target);
    define_type_minimum(builder, "__SIG_ATOMIC", target.sig_atomic_type, target);
    if c23 {
        builder.define("__CHAR8_TYPE__", IntegerType::UnsignedChar.name());
    }
    builder.define("__CHAR16_TYPE__", target.char16_type.name());
    builder.define("__CHAR32_TYPE__", target.char32_type.name());
    builder.define("__UINTPTR_TYPE__", unsigned_intptr.name());
    define_format(builder, c23, "__UINTPTR", unsigned_intptr);
    if target.has_float16 {
        define_float_macros(builder, "FLT16", target.half_format, "F16");
    }
    define_float_macros(builder, "FLT", target.float_format, "F");
    define_float_macros(builder, "DBL", target.double_format, "");
    define_float_macros(builder, "LDBL", target.long_double_format, "L");
    builder.define("__POINTER_WIDTH__", &target.pointer_width.to_string());
    builder.define(
        "__BIGGEST_ALIGNMENT__",
        &(target.suitable_align / target.char_width()).to_string(),
    );
    if !language.char_is_signed {
        builder.define_one("__CHAR_UNSIGNED__");
    }
    if !target.wchar_type.signed() {
        builder.define_one("__WCHAR_UNSIGNED__");
    }
    if !target.wint_type.signed() {
        builder.define_one("__WINT_UNSIGNED__");
    }
    define_exact_width_type(builder, c23, IntegerType::SignedChar, target);
    if target.short_width > target.char_width() {
        define_exact_width_type(builder, c23, IntegerType::Short, target);
    }
    if target.int_width > target.short_width {
        define_exact_width_type(builder, c23, IntegerType::Int, target);
    }
    if target.long_width > target.int_width {
        define_exact_width_type(builder, c23, IntegerType::Long, target);
    }
    if target.long_long_width > target.long_width {
        define_exact_width_type(builder, c23, IntegerType::LongLong, target);
    }
    define_exact_width_type(builder, c23, IntegerType::UnsignedChar, target);
    define_exact_width_type_size(builder, IntegerType::UnsignedChar, target);
    define_exact_width_type_size(builder, IntegerType::SignedChar, target);
    for (narrow, wide, unsigned_wide, signed_wide) in [
        (
            target.char_width(),
            target.short_width,
            IntegerType::UnsignedShort,
            IntegerType::Short,
        ),
        (
            target.short_width,
            target.int_width,
            IntegerType::UnsignedInt,
            IntegerType::Int,
        ),
        (
            target.int_width,
            target.long_width,
            IntegerType::UnsignedLong,
            IntegerType::Long,
        ),
        (
            target.long_width,
            target.long_long_width,
            IntegerType::UnsignedLongLong,
            IntegerType::LongLong,
        ),
    ] {
        if wide > narrow {
            define_exact_width_type(builder, c23, unsigned_wide, target);
            define_exact_width_type_size(builder, unsigned_wide, target);
            define_exact_width_type_size(builder, signed_wide, target);
        }
    }
    for head in ["LEAST", "FAST"] {
        for width in [8, 16, 32, 64] {
            define_least_width_type(builder, c23, head, width, true, target);
            define_least_width_type(builder, c23, head, width, false, target);
        }
    }
    builder.define("__USER_LABEL_PREFIX__", target.user_label_prefix);
    if !options.math_errno {
        builder.define_one("__NO_MATH_ERRNO__");
    }
    if options.fast_math || (options.no_honor_infinities && options.no_honor_nans) {
        builder.define("__FINITE_MATH_ONLY__", "1");
    } else {
        builder.define("__FINITE_MATH_ONLY__", "0");
    }
    if options.gnuc_version.is_some() {
        let gnu_inline = options
            .gnu_inline
            .unwrap_or(!language.at_least(CStandard::C99));
        if gnu_inline {
            builder.define_one("__GNUC_GNU_INLINE__");
        } else {
            builder.define_one("__GNUC_STDC_INLINE__");
        }
        builder.define("__GCC_ATOMIC_TEST_AND_SET_TRUEVAL", "1");
    }
    let (destructive, constructive) = target.interference_sizes();
    builder.define("__GCC_DESTRUCTIVE_SIZE", &destructive.to_string());
    builder.define("__GCC_CONSTRUCTIVE_SIZE", &constructive.to_string());
    builder.append("#pragma push_macro(\"__GCC_DESTRUCTIVE_SIZE\")");
    builder.append("#pragma push_macro(\"__GCC_CONSTRUCTIVE_SIZE\")");
    let mut prefixes = vec!["__CLANG_ATOMIC_"];
    if options.gnuc_version.is_some() {
        prefixes.push("__GCC_ATOMIC_");
    }
    for prefix in prefixes {
        let char_width = target.char_width();
        let mut entries = vec![("BOOL", target.bool_width), ("CHAR", char_width)];
        if c23 {
            entries.push(("CHAR8_T", char_width));
        }
        entries.extend([
            ("CHAR16_T", target.type_width(target.char16_type)),
            ("CHAR32_T", target.type_width(target.char32_type)),
            ("WCHAR_T", target.type_width(target.wchar_type)),
            ("SHORT", target.short_width),
            ("INT", target.int_width),
            ("LONG", target.long_width),
            ("LLONG", target.long_long_width),
            ("POINTER", target.pointer_width),
        ]);
        for (name, width) in entries {
            builder.define(
                &format!("{prefix}{name}_LOCK_FREE"),
                lock_free_value(width, target),
            );
        }
    }
    if options.optimize == 0 || options.no_inline {
        builder.define_one("__NO_INLINE__");
    }
    if options.pic_level > 0 {
        let level = options.pic_level.to_string();
        builder.define("__PIC__", &level);
        builder.define("__pic__", &level);
        if options.pie {
            builder.define("__PIE__", &level);
            builder.define("__pie__", &level);
        }
    }
    builder.define("__FLT_RADIX__", "2");
    builder.define("__DECIMAL_DIG__", "__LDBL_DECIMAL_DIG__");
    if target.triple.is_elf() {
        builder.define_one("__ELF__");
    }
    target.target_defines(language, options.posix_threads, builder);
}

fn initialize_standard_predefined_macros(
    options: &PreprocessorOptions,
    builder: &mut MacroBuilder,
) {
    let language = &options.language;
    if !language.traditional {
        builder.define_one("__STDC__");
    }
    if options.freestanding {
        builder.define("__STDC_HOSTED__", "0");
    } else {
        builder.define_one("__STDC_HOSTED__");
    }
    if let Some(version) = language.standard.version_macro() {
        builder.define("__STDC_VERSION__", version);
    }
    builder.define("__STDC_UTF_16__", "1");
    builder.define("__STDC_UTF_32__", "1");
    builder.define("__STDC_EMBED_NOT_FOUND__", "0");
    builder.define("__STDC_EMBED_FOUND__", "1");
    builder.define("__STDC_EMBED_EMPTY__", "2");
    if language.assembly {
        builder.define_one("__ASSEMBLER__");
    }
}

pub fn predefined_macro_text(options: &PreprocessorOptions) -> String {
    let mut builder = MacroBuilder {
        text: String::new(),
    };
    if !options.undefine_predefined {
        initialize_predefined_macros(options, &mut builder);
    }
    initialize_standard_predefined_macros(options, &mut builder);
    builder.text
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
    pub target_problem: Option<String>,
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
        target_problem: None,
        show_source: true,
        dump_macros: false,
    };
    let mut target_options = TargetOptions::default();
    let mut standard: Option<String> = None;
    let mut trigraphs: Option<bool> = None;
    let mut dollars: Option<bool> = None;
    let mut signed_char: Option<bool> = None;
    let mut microsoft = false;
    let mut declspec: Option<bool> = None;
    let mut traditional = false;
    let mut gnu_keywords: Option<bool> = None;
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
            "-fdeclspec" => declspec = Some(true),
            "-fno-declspec" => declspec = Some(false),
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
            | "-fno-diagnostics-fixit-info"
            | "-fdiagnostics-parseable-fixits"
            | "-fno-common"
            | "-fcommon"
            | "-disable-free"
            | "-fno-rtti"
            | "-fno-elide-type"
            | "-fno-integrated-cc1"
            | "-fgnuc-version=0" => {}
            "-fno-caret-diagnostics" => invocation.show_source = false,
            "-fgnu-keywords" => gnu_keywords = Some(true),
            "-fno-gnu-keywords" => gnu_keywords = Some(false),
            "-fno-signed-wchar" => target_options.wchar_signed = false,
            "-fsigned-wchar" => target_options.wchar_signed = true,
            "-fexceptions" => options.exceptions = true,
            "-fno-exceptions" => options.exceptions = false,
            "-fgnu89-inline" => options.gnu_inline = Some(true),
            "-fno-gnu89-inline" => options.gnu_inline = Some(false),
            "-fmath-errno" => options.math_errno = true,
            "-fno-math-errno" => options.math_errno = false,
            "-ffast-math" => {
                options.fast_math = true;
                options.no_honor_infinities = true;
                options.no_honor_nans = true;
            }
            "-fno-fast-math" => options.fast_math = false,
            "-menable-no-infs" => options.no_honor_infinities = true,
            "-menable-no-nans" => options.no_honor_nans = true,
            "-pthread" => options.posix_threads = true,
            "-fno-spell-checking" => options.spell_checking = false,
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
                } else if let Some(rest) = argument.strip_prefix("-x") {
                    let value = take_value(&mut index, "-x", rest).unwrap_or_default();
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
                        target_options.triple = value;
                    }
                } else if let Some(rest) = argument.strip_prefix("--embed-dir=") {
                    options.embed_directories.push(rest.to_string());
                } else if let Some(rest) = argument.strip_prefix("-triple=") {
                    target_options.triple = rest.to_string();
                } else if let Some(rest) = argument.strip_prefix("--target=") {
                    target_options.triple = rest.to_string();
                } else if argument == "-target-cpu" {
                    if let Some(value) = take_value(&mut index, argument, "") {
                        target_options.cpu = value;
                    }
                } else if argument == "-target-feature" {
                    if let Some(value) = take_value(&mut index, argument, "") {
                        target_options.features.push(value);
                    }
                } else if argument == "-target-abi" {
                    if let Some(value) = take_value(&mut index, argument, "") {
                        target_options.abi = value;
                    }
                } else if let Some(rest) = argument.strip_prefix("-mcmodel=") {
                    target_options.code_model = rest.to_string();
                } else if let Some(rest) = argument.strip_prefix("-fwchar-type=") {
                    match rest {
                        "char" => target_options.wchar_size = 1,
                        "short" => target_options.wchar_size = 2,
                        "int" => target_options.wchar_size = 4,
                        _ => invocation.unsupported.push(argument.to_string()),
                    }
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
                {
                } else if let Some(rest) = argument.strip_prefix("-Wundef-prefix=") {
                    options.warnings.flags.push("undef-prefix".to_string());
                    options.warnings.undef_prefixes.extend(
                        rest.split(',')
                            .filter(|item| !item.is_empty())
                            .map(str::to_string),
                    );
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
    match TargetInfo::create(&target_options) {
        Ok(target) => invocation.preprocessor.target = target,
        Err(reason) => invocation.target_problem = Some(reason),
    }
    let input_kind = invocation
        .inputs
        .first()
        .map(|path| invocation.input_language(path))
        .unwrap_or(invocation.language_kind.unwrap_or(InputLanguage::C));
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
    language.declspec_keyword = declspec.unwrap_or(microsoft);
    language.traditional = traditional;
    if let Some(value) = gnu_keywords {
        language.gnu_keywords = value;
    }
    language.char_is_signed = signed_char.unwrap_or(true);
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

pub struct Parser<'session, 'files> {
    preprocessor: &'session mut Preprocessor<'files>,
    token: Token,
    unsupported: Option<String>,
}

impl<'session, 'files> Parser<'session, 'files> {
    pub fn new(preprocessor: &'session mut Preprocessor<'files>) -> Parser<'session, 'files> {
        let token = preprocessor.next_token();
        Parser {
            preprocessor,
            token,
            unsupported: None,
        }
    }

    fn consume(&mut self) -> Place {
        let place = self.token.site;
        self.token = self.preprocessor.next_token();
        place
    }

    fn keyword(&self) -> Option<&'static str> {
        if self.token.kind != TokenKind::Identifier {
            return None;
        }
        keyword_kind(&self.preprocessor.language, &self.token.spelling)
    }

    fn unsupported(&mut self, what: String) {
        if self.unsupported.is_none() {
            self.unsupported = Some(what);
        }
    }

    fn report(&mut self, message: Message, place: Place, arguments: Vec<String>) {
        self.preprocessor.report(message, place, arguments);
    }

    pub fn parse_translation_unit(mut self) -> Option<String> {
        let mut at_end = self.parse_top_level_declaration();
        if at_end && !self.preprocessor.language.header_file {
            let place = self.token.site;
            self.report(Message::EmptyTranslationUnit, place, Vec::new());
        }
        while !at_end && self.unsupported.is_none() {
            at_end = self.parse_top_level_declaration();
        }
        self.unsupported
    }

    fn parse_top_level_declaration(&mut self) -> bool {
        if self.token.kind == TokenKind::EndOfFile {
            return true;
        }
        self.parse_external_declaration();
        false
    }

    fn parse_external_declaration(&mut self) {
        match self.token.kind {
            TokenKind::PragmaAnnotation => self.handle_pragma_annotation(),
            TokenKind::Semi => self.consume_extra_semicolon(0),
            TokenKind::RightBrace => {
                let place = self.consume();
                self.report(Message::ExtraneousClosingBrace, place, Vec::new());
            }
            TokenKind::EndOfFile => {
                let place = self.token.site;
                self.report(Message::ExpectedExternalDeclaration, place, Vec::new());
            }
            _ => {
                let what = match self.keyword() {
                    Some(keyword) => format!("declaration starting with '{keyword}'"),
                    None => format!("declaration starting with {:?}", self.token.kind),
                };
                self.unsupported(what);
            }
        }
    }

    fn handle_pragma_annotation(&mut self) {
        let value = String::from_utf8_lossy(&self.token.spelling).into_owned();
        let place = self.consume();
        let mut words = value.split(' ');
        match (words.next(), words.next()) {
            (Some("fp_contract" | "cx_limited_range" | "fenv_round"), _) => {}
            (Some("fenv_access"), Some(state)) => {
                if state == "ON" && self.preprocessor.options.fast_math {
                    self.report(Message::PragmaFenvRequiresPrecise, place, Vec::new());
                }
            }
            _ => self.unsupported(value),
        }
    }

    fn consume_extra_semicolon(&mut self, kind: usize) {
        let start = self.consume();
        while self.token.kind == TokenKind::Semi && !self.token.at_start_of_line() {
            self.consume();
        }
        self.report(
            Message::ExtraSemi,
            start,
            vec![kind.to_string(), String::new()],
        );
    }
}
