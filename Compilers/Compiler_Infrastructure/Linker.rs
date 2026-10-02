use crate::compilers::compiler_infrastructure::fail;
use crate::compilers::elf::{
    self, align_to, file_type, machine, section_flag, section_index, section_type, segment_flag,
    segment_type, symbol_binding, symbol_type, visibility, ElfFile, Encoding, Endian,
    ProgramHeader, Section, StringTable,
};
use crate::diag::{Diagnostic, Severity};
use crate::FailureClass;
use std::collections::{BTreeMap, BTreeSet};

pub const LINKER_IDENTITY: &str = "Linker: Omni Builder 1.0.0";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OutputKind {
    Executable,
    PositionIndependentExecutable,
    SharedLibrary,
    Relocatable,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Discard {
    Default,
    Locals,
    All,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Strip {
    None,
    Debug,
    All,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SeparateCode {
    None,
    Code,
    Loadable,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnresolvedPolicy {
    ReportError,
    Warn,
    Ignore,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BuildIdentifier {
    None,
    Fast,
    Md5,
    Sha1,
    Uuid,
    Hexadecimal(Vec<u8>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InputItem {
    File(String),
    Library(String),
    StartGroup,
    EndGroup,
    StartLibrary,
    EndLibrary,
    WholeArchive(bool),
    AsNeeded(bool),
    Static(bool),
    PushState,
    PopState,
}

#[derive(Clone, Debug)]
pub struct Options {
    pub output: String,
    pub kind: OutputKind,
    pub emulation: Option<String>,
    pub entry: Option<String>,
    pub image_base: Option<u64>,
    pub maximum_page_size: Option<u64>,
    pub common_page_size: Option<u64>,
    pub inputs: Vec<InputItem>,
    pub library_paths: Vec<String>,
    pub sysroot: Option<String>,
    pub gc_sections: bool,
    pub print_gc_sections: bool,
    pub relocation_read_only: bool,
    pub bind_now: bool,
    pub text_relocations_forbidden: bool,
    pub separate_code: SeparateCode,
    pub executable_stack: bool,
    pub gnu_hash: Option<bool>,
    pub system_five_hash: Option<bool>,
    pub exception_frame_header: bool,
    pub build_identifier: BuildIdentifier,
    pub shared_object_name: Option<String>,
    pub run_paths: Vec<String>,
    pub export_dynamic: bool,
    pub strip: Strip,
    pub discard: Discard,
    pub undefined: Vec<String>,
    pub defined_symbols: Vec<(String, String)>,
    pub wrapped: Vec<String>,
    pub unresolved_symbols: Option<UnresolvedPolicy>,
    pub no_undefined: bool,
    pub allow_shared_library_undefined: Option<bool>,
    pub fatal_warnings: bool,
    pub no_inhibit_executable: bool,
    pub section_start: BTreeMap<String, u64>,
    pub emit_relocations: bool,
    pub relax: bool,
    pub dynamic_linker: Option<String>,
    pub no_dynamic_linker: bool,
    pub enable_new_dynamic_tags: bool,
    pub symbolic: bool,
    pub symbolic_functions: bool,
    pub allow_multiple_definition: bool,
    pub warn_common: bool,
    pub optimize: u32,
    pub z_keywords: Vec<String>,
    pub copy_relocations: bool,
    pub origin: bool,
    pub no_delete: bool,
    pub load_globally: bool,
    pub initialize_first: bool,
    pub interpose: bool,
    pub no_default_library: bool,
    pub no_dynamic_open: bool,
    pub read_only_dynamic: bool,
    pub combine_relocations: bool,
    pub initializer: Option<String>,
    pub finalizer: Option<String>,
    pub version_script: Vec<String>,
    pub export_dynamic_symbols: Vec<String>,
    pub dynamic_list: Vec<String>,
    pub pack_relative_relocations: bool,
    pub apply_dynamic_relocations: bool,
    pub arm_target1_relative: bool,
    pub arm_target2: String,
    pub android_page_size_16_kilobytes: bool,
    pub threads: Option<usize>,
    pub error_limit: Option<usize>,
    pub warnings: Vec<Diagnostic>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            output: "a.out".to_string(),
            kind: OutputKind::Executable,
            emulation: None,
            entry: None,
            image_base: None,
            maximum_page_size: None,
            common_page_size: None,
            inputs: Vec::new(),
            library_paths: Vec::new(),
            sysroot: None,
            gc_sections: false,
            print_gc_sections: false,
            relocation_read_only: true,
            bind_now: false,
            text_relocations_forbidden: true,
            separate_code: SeparateCode::None,
            executable_stack: false,
            gnu_hash: None,
            system_five_hash: None,
            exception_frame_header: false,
            build_identifier: BuildIdentifier::None,
            shared_object_name: None,
            run_paths: Vec::new(),
            export_dynamic: false,
            strip: Strip::None,
            discard: Discard::Default,
            undefined: Vec::new(),
            defined_symbols: Vec::new(),
            wrapped: Vec::new(),
            unresolved_symbols: None,
            no_undefined: false,
            allow_shared_library_undefined: None,
            fatal_warnings: false,
            no_inhibit_executable: false,
            section_start: BTreeMap::new(),
            emit_relocations: false,
            relax: true,
            dynamic_linker: None,
            no_dynamic_linker: false,
            enable_new_dynamic_tags: true,
            symbolic: false,
            symbolic_functions: false,
            allow_multiple_definition: false,
            warn_common: false,
            optimize: 1,
            z_keywords: Vec::new(),
            copy_relocations: true,
            origin: false,
            no_delete: false,
            load_globally: false,
            initialize_first: false,
            interpose: false,
            no_default_library: false,
            no_dynamic_open: false,
            read_only_dynamic: false,
            combine_relocations: true,
            initializer: None,
            finalizer: None,
            version_script: Vec::new(),
            export_dynamic_symbols: Vec::new(),
            dynamic_list: Vec::new(),
            pack_relative_relocations: false,
            apply_dynamic_relocations: false,
            arm_target1_relative: false,
            arm_target2: "got-rel".to_string(),
            android_page_size_16_kilobytes: false,
            threads: None,
            error_limit: None,
            warnings: Vec::new(),
        }
    }
}

pub fn problem(code: &str, message: &str) -> Diagnostic {
    fail(code, message).with_class(FailureClass::UserError)
}

pub fn caution(code: &str, message: &str) -> Diagnostic {
    let mut diagnostic = fail(code, message).with_class(FailureClass::UserError);
    diagnostic.severity = Severity::Warning;
    diagnostic
}

fn unknown_option(option: &str) -> Diagnostic {
    problem("EI202", "The linker does not recognize this option.")
        .with_context(format!("Option: {option}"))
}

fn missing_value(option: &str) -> Diagnostic {
    problem("EI203", "An option is missing its value.").with_context(format!("Option: {option}"))
}

fn invalid_value(option: &str, value: &str) -> Diagnostic {
    problem("EI204", "An option value is not valid.")
        .with_context(format!("Option: {option}"))
        .with_context(format!("Value: {value}"))
}

pub fn not_supported(feature: &str) -> Diagnostic {
    problem(
        "EI212",
        "The linker does not support what this input asks for yet.",
    )
    .with_context(format!("Feature: {feature}"))
}

pub fn script_tokens(text: &str) -> Result<Vec<String>, &'static str> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut at = 0usize;
    let word = |byte: u8| byte.is_ascii_alphanumeric() || b"_.$/\\~=+[]*?-!^:".contains(&byte);
    while at < bytes.len() {
        let byte = bytes[at];
        if byte.is_ascii_whitespace() {
            at += 1;
            continue;
        }
        if bytes[at..].starts_with(b"/*") {
            match text[at + 2..].find("*/") {
                Some(end) => at += end + 4,
                None => return Err("unclosed comment in a linker script"),
            }
            continue;
        }
        if byte == b'#' {
            while at < bytes.len() && bytes[at] != b'\n' {
                at += 1;
            }
            continue;
        }
        if byte == b'"' {
            match text[at + 1..].find('"') {
                Some(end) => {
                    tokens.push(text[at..at + end + 2].to_string());
                    at += end + 2;
                }
                None => return Err("unclosed quote"),
            }
            continue;
        }
        let mut end = at;
        while end < bytes.len() && word(bytes[end]) {
            end += 1;
        }
        if end == at {
            end = at + 1;
        }
        tokens.push(text[at..end].to_string());
        at = end;
    }
    Ok(tokens)
}

fn unquote(token: &str) -> String {
    if token.len() >= 2 && token.starts_with('"') && token.ends_with('"') {
        token[1..token.len() - 1].to_string()
    } else {
        token.to_string()
    }
}

fn skip_parenthesized(tokens: &[String], at: usize) -> usize {
    if tokens.get(at).map(String::as_str) != Some("(") {
        return at;
    }
    let mut depth = 0usize;
    let mut position = at;
    while position < tokens.len() {
        match tokens[position].as_str() {
            "(" => depth += 1,
            ")" => {
                depth -= 1;
                if depth == 0 {
                    return position + 1;
                }
            }
            _ => {}
        }
        position += 1;
    }
    position
}

pub fn parse_number(text: &str) -> Option<u64> {
    let text = text.trim();
    if let Some(hexadecimal) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        return u64::from_str_radix(hexadecimal, 16).ok();
    }
    if let Some(hexadecimal) = text.strip_suffix('h').or_else(|| text.strip_suffix('H')) {
        return u64::from_str_radix(hexadecimal, 16).ok();
    }
    text.parse::<u64>().ok()
}

struct ArgumentCursor<'a> {
    arguments: &'a [String],
    at: usize,
}

impl ArgumentCursor<'_> {
    fn next(&mut self) -> Option<String> {
        let found = self.arguments.get(self.at).cloned();
        self.at += 1;
        found
    }
}

fn option_value(
    cursor: &mut ArgumentCursor,
    argument: &str,
    names: &[&str],
) -> Result<Option<String>, Diagnostic> {
    for name in names {
        if argument == *name {
            return cursor.next().map(Some).ok_or_else(|| missing_value(name));
        }
        if let Some(rest) = argument.strip_prefix(name) {
            if let Some(value) = rest.strip_prefix('=') {
                return Ok(Some(value.to_string()));
            }
        }
    }
    Ok(None)
}

fn joined_value(argument: &str, names: &[&str]) -> Option<String> {
    for name in names {
        if let Some(rest) = argument.strip_prefix(name) {
            if !rest.is_empty() {
                return Some(rest.trim_start_matches('=').to_string());
            }
        }
    }
    None
}

const IGNORED_FLAGS: &[&str] = &[
    "--color-diagnostics",
    "--no-color-diagnostics",
    "--no-add-needed",
    "--add-needed",
    "--eh-frame-hdr",
    "--no-eh-frame-hdr",
    "--no-copy-dt-needed-entries",
    "--copy-dt-needed-entries",
    "--no-ctors-in-init-array",
    "--no-keep-memory",
    "--no-mmap-output-file",
    "--no-pipeline-knowledge",
    "--no-warn-mismatch",
    "--rpath-link",
    "--sort-common",
    "--stats",
    "--warn-execstack",
    "--warn-once",
    "--warn-shared-textrel",
    "--no-warn-execstack",
    "-g",
    "--long-plt",
    "--secure-plt",
    "--qmagic",
    "--Qy",
    "--dc",
    "--dp",
    "--fatal-warnings",
    "--no-fatal-warnings",
    "--disable-new-dtags",
    "--enable-new-dtags",
    "--relax",
    "--no-relax",
    "--gc-sections",
    "--no-gc-sections",
    "--print-gc-sections",
    "--no-print-gc-sections",
    "--export-dynamic",
    "--no-export-dynamic",
    "--strip-all",
    "--strip-debug",
    "--discard-all",
    "--discard-locals",
    "--discard-none",
    "--Bsymbolic",
    "--Bsymbolic-functions",
    "--allow-multiple-definition",
    "--no-allow-multiple-definition",
    "--warn-common",
    "--no-warn-common",
    "--noinhibit-exec",
    "--no-undefined",
    "--allow-shlib-undefined",
    "--no-allow-shlib-undefined",
    "--emit-relocs",
    "--no-dynamic-linker",
    "--pie",
    "--no-pie",
    "--shared",
    "--relocatable",
    "--static",
    "--whole-archive",
    "--no-whole-archive",
    "--as-needed",
    "--no-as-needed",
    "--start-group",
    "--end-group",
    "--start-lib",
    "--end-lib",
    "--push-state",
    "--pop-state",
];

fn normalize_dashes(argument: &str) -> String {
    if argument.starts_with("--") || !argument.starts_with('-') || argument.len() <= 2 {
        return argument.to_string();
    }
    let single = &argument[1..];
    let long_forms = [
        "shared",
        "static",
        "pie",
        "no-pie",
        "soname",
        "rpath",
        "entry",
        "wrap",
        "defsym",
        "script",
        "version-script",
        "dynamic-linker",
        "gc-sections",
        "no-gc-sections",
        "export-dynamic",
        "hash-style",
        "build-id",
        "image-base",
        "Bsymbolic",
        "Bsymbolic-functions",
        "Bstatic",
        "Bdynamic",
        "Bshareable",
        "init",
        "fini",
        "whole-archive",
        "no-whole-archive",
        "as-needed",
        "no-as-needed",
        "start-group",
        "end-group",
        "start-lib",
        "end-lib",
        "push-state",
        "pop-state",
        "strip-all",
        "strip-debug",
        "discard-all",
        "discard-locals",
        "eh-frame-hdr",
        "no-eh-frame-hdr",
        "relocatable",
        "emit-relocs",
        "noinhibit-exec",
        "fatal-warnings",
        "no-undefined",
        "allow-shlib-undefined",
        "no-allow-shlib-undefined",
        "sysroot",
        "section-start",
        "undefined",
        "threads",
        "pack-dyn-relocs",
        "warn-common",
        "error-limit",
        "unresolved-symbols",
        "export-dynamic-symbol",
        "dynamic-list",
        "no-dynamic-linker",
        "enable-new-dtags",
        "disable-new-dtags",
        "allow-multiple-definition",
        "relax",
        "no-relax",
        "color-diagnostics",
        "print-gc-sections",
        "max-page-size",
        "Ttext",
        "Tdata",
        "Tbss",
        "trace",
        "Map",
        "verbose",
        "library",
        "library-path",
        "output",
        "nostdlib",
        "no-warn-mismatch",
        "m",
        "o",
        "e",
        "u",
        "l",
        "L",
        "T",
        "z",
        "y",
        "O",
    ];
    for long in long_forms {
        if single == long || single.starts_with(&format!("{long}=")) {
            if long.len() == 1 {
                return argument.to_string();
            }
            return format!("-{argument}");
        }
    }
    argument.to_string()
}

pub fn parse_command_line(arguments: &[String]) -> Result<Options, Diagnostic> {
    let mut options = Options::default();
    let mut cursor = ArgumentCursor { arguments, at: 0 };
    let mut pie: Option<bool> = None;
    let mut shared = false;
    let mut relocatable = false;
    let mut nostdlib = false;
    while let Some(raw) = cursor.next() {
        if raw == "-" || !raw.starts_with('-') || raw.len() == 1 {
            options.inputs.push(InputItem::File(raw));
            continue;
        }
        let argument = normalize_dashes(&raw);
        let argument = argument.as_str();
        if let Some(value) = option_value(&mut cursor, argument, &["--output"])? {
            options.output = value;
            continue;
        }
        if argument == "-o" {
            options.output = cursor.next().ok_or_else(|| missing_value("-o"))?;
            continue;
        }
        if let Some(value) = argument.strip_prefix("-o") {
            options.output = value.to_string();
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--entry"])? {
            options.entry = Some(value);
            continue;
        }
        if argument == "-e" {
            options.entry = Some(cursor.next().ok_or_else(|| missing_value("-e"))?);
            continue;
        }
        if let Some(value) = argument.strip_prefix("-e") {
            options.entry = Some(value.to_string());
            continue;
        }
        if argument == "-m" {
            options.emulation = Some(cursor.next().ok_or_else(|| missing_value("-m"))?);
            continue;
        }
        if let Some(value) = argument.strip_prefix("-m") {
            options.emulation = Some(value.to_string());
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--library-path"])? {
            options.library_paths.push(value);
            continue;
        }
        if argument == "-L" {
            options
                .library_paths
                .push(cursor.next().ok_or_else(|| missing_value("-L"))?);
            continue;
        }
        if let Some(value) = argument.strip_prefix("-L") {
            options.library_paths.push(value.to_string());
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--library"])? {
            options.inputs.push(InputItem::Library(value));
            continue;
        }
        if argument == "-l" {
            let value = cursor.next().ok_or_else(|| missing_value("-l"))?;
            options.inputs.push(InputItem::Library(value));
            continue;
        }
        if let Some(value) = argument.strip_prefix("-l") {
            options.inputs.push(InputItem::Library(value.to_string()));
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--undefined"])? {
            options.undefined.push(value);
            continue;
        }
        if argument == "-u" {
            options
                .undefined
                .push(cursor.next().ok_or_else(|| missing_value("-u"))?);
            continue;
        }
        if let Some(value) = argument.strip_prefix("-u") {
            options.undefined.push(value.to_string());
            continue;
        }
        if argument == "-z" {
            let value = cursor.next().ok_or_else(|| missing_value("-z"))?;
            apply_z_keyword(&mut options, &value)?;
            continue;
        }
        if let Some(value) = argument.strip_prefix("-z") {
            apply_z_keyword(&mut options, value)?;
            continue;
        }
        if argument == "-O" {
            let value = cursor.next().ok_or_else(|| missing_value("-O"))?;
            options.optimize = value.parse().map_err(|_| invalid_value("-O", &value))?;
            continue;
        }
        if let Some(value) = argument.strip_prefix("-O") {
            options.optimize = value.parse().map_err(|_| invalid_value("-O", value))?;
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--soname", "-h"])? {
            options.shared_object_name = Some(value);
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--rpath", "-R"])? {
            options.run_paths.push(value);
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--wrap"])? {
            options.wrapped.push(value);
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--defsym"])? {
            let (name, expression) = value
                .split_once('=')
                .ok_or_else(|| invalid_value("--defsym", &value))?;
            options
                .defined_symbols
                .push((name.to_string(), expression.to_string()));
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--image-base"])? {
            options.image_base =
                Some(parse_number(&value).ok_or_else(|| invalid_value("--image-base", &value))?);
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--hash-style"])? {
            match value.as_str() {
                "sysv" => {
                    options.system_five_hash = Some(true);
                    options.gnu_hash.get_or_insert(false);
                }
                "gnu" => {
                    options.gnu_hash = Some(true);
                    options.system_five_hash.get_or_insert(false);
                }
                "both" => {
                    options.gnu_hash = Some(true);
                    options.system_five_hash = Some(true);
                }
                _ => return Err(invalid_value("--hash-style", &value)),
            }
            continue;
        }
        if argument == "--build-id" || argument == "-build-id" {
            options.build_identifier = BuildIdentifier::Fast;
            continue;
        }
        if let Some(value) = argument
            .strip_prefix("--build-id=")
            .or_else(|| argument.strip_prefix("-build-id="))
        {
            options.build_identifier = parse_build_identifier(value)?;
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--dynamic-linker", "-I"])? {
            options.dynamic_linker = Some(value);
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--sysroot"])? {
            options.sysroot = Some(value);
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--threads"])? {
            options.threads = value.parse().ok();
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--error-limit"])? {
            options.error_limit = value.parse().ok();
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--init"])? {
            options.initializer = Some(value);
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--fini"])? {
            options.finalizer = Some(value);
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--unresolved-symbols"])? {
            options.unresolved_symbols = Some(match value.as_str() {
                "ignore-all" | "ignore-in-object-files" | "ignore-in-shared-libs" => {
                    UnresolvedPolicy::Ignore
                }
                "report-all" => UnresolvedPolicy::ReportError,
                _ => return Err(invalid_value("--unresolved-symbols", &value)),
            });
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--section-start"])? {
            let (name, address) = value
                .split_once('=')
                .ok_or_else(|| invalid_value("--section-start", &value))?;
            let address =
                parse_number(address).ok_or_else(|| invalid_value("--section-start", &value))?;
            options.section_start.insert(name.to_string(), address);
            continue;
        }
        let mut segment_start = None;
        for (flag, section) in [("-Ttext", ".text"), ("-Tdata", ".data"), ("-Tbss", ".bss")] {
            if let Some(value) = option_value(&mut cursor, argument, &[flag, &format!("-{flag}")])?
            {
                segment_start = Some((section, value));
            } else if let Some(value) = argument.strip_prefix(flag) {
                if !value.is_empty() && !value.starts_with('=') {
                    segment_start = Some((section, value.to_string()));
                }
            }
        }
        if let Some((section, value)) = segment_start {
            let address = parse_number(&value).ok_or_else(|| invalid_value(&raw, &value))?;
            options.section_start.insert(section.to_string(), address);
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--target2"])? {
            if !matches!(value.as_str(), "rel" | "abs" | "got-rel") {
                return Err(invalid_value("--target2", &value));
            }
            options.arm_target2 = value;
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--export-dynamic-symbol"])? {
            options.export_dynamic_symbols.push(value);
            continue;
        }
        if let Some(value) = option_value(&mut cursor, argument, &["--pack-dyn-relocs"])? {
            match value.as_str() {
                "none" => {}
                "relr" => options.pack_relative_relocations = true,
                _ => return Err(not_supported(&format!("--pack-dyn-relocs={value}"))),
            }
            continue;
        }
        if let Some(value) = joined_value(argument, &["--plugin-opt", "-plugin-opt"]) {
            let _ = value;
            continue;
        }
        if argument.starts_with("--plugin") || argument.starts_with("-plugin") {
            if !argument.contains('=') {
                cursor.next();
            }
            continue;
        }
        match argument {
            "--shared" | "-shared" | "-Bshareable" | "--Bshareable" => shared = true,
            "--pie" | "-pie" | "--pic-executable" => pie = Some(true),
            "--no-pie" | "-no-pie" | "--no-pic-executable" => pie = Some(false),
            "--relocatable" | "-r" | "-i" => relocatable = true,
            "--static" | "-static" | "-Bstatic" | "--Bstatic" | "-dn" | "-non_shared" => {
                options.inputs.push(InputItem::Static(true))
            }
            "-Bdynamic" | "--Bdynamic" | "-dy" | "-call_shared" => {
                options.inputs.push(InputItem::Static(false))
            }
            "--whole-archive" => options.inputs.push(InputItem::WholeArchive(true)),
            "--no-whole-archive" => options.inputs.push(InputItem::WholeArchive(false)),
            "--as-needed" => options.inputs.push(InputItem::AsNeeded(true)),
            "--no-as-needed" => options.inputs.push(InputItem::AsNeeded(false)),
            "--start-group" | "-(" => options.inputs.push(InputItem::StartGroup),
            "--end-group" | "-)" => options.inputs.push(InputItem::EndGroup),
            "--start-lib" => options.inputs.push(InputItem::StartLibrary),
            "--end-lib" => options.inputs.push(InputItem::EndLibrary),
            "--push-state" => options.inputs.push(InputItem::PushState),
            "--pop-state" => options.inputs.push(InputItem::PopState),
            "--gc-sections" => options.gc_sections = true,
            "--no-gc-sections" => options.gc_sections = false,
            "--print-gc-sections" => options.print_gc_sections = true,
            "--no-print-gc-sections" => options.print_gc_sections = false,
            "--export-dynamic" | "-E" => options.export_dynamic = true,
            "--no-export-dynamic" => options.export_dynamic = false,
            "--strip-all" | "-s" => options.strip = Strip::All,
            "--strip-debug" | "-S" => options.strip = Strip::Debug,
            "--discard-all" | "-x" => options.discard = Discard::All,
            "--discard-locals" | "-X" => options.discard = Discard::Locals,
            "--discard-none" => options.discard = Discard::None,
            "--eh-frame-hdr" => options.exception_frame_header = true,
            "--no-eh-frame-hdr" => options.exception_frame_header = false,
            "--emit-relocs" | "-q" => options.emit_relocations = true,
            "--noinhibit-exec" => options.no_inhibit_executable = true,
            "--fatal-warnings" => options.fatal_warnings = true,
            "--no-fatal-warnings" => options.fatal_warnings = false,
            "--no-undefined" => options.no_undefined = true,
            "--allow-shlib-undefined" => options.allow_shared_library_undefined = Some(true),
            "--no-allow-shlib-undefined" => options.allow_shared_library_undefined = Some(false),
            "--relax" => options.relax = true,
            "--no-relax" => options.relax = false,
            "--no-dynamic-linker" => options.no_dynamic_linker = true,
            "--apply-dynamic-relocs" => options.apply_dynamic_relocations = true,
            "--target1-rel" => options.arm_target1_relative = true,
            "--target1-abs" => options.arm_target1_relative = false,
            "--no-apply-dynamic-relocs" => options.apply_dynamic_relocations = false,
            "--enable-new-dtags" => options.enable_new_dynamic_tags = true,
            "--disable-new-dtags" => options.enable_new_dynamic_tags = false,
            "--Bsymbolic" => options.symbolic = true,
            "--Bsymbolic-functions" => options.symbolic_functions = true,
            "--allow-multiple-definition" => options.allow_multiple_definition = true,
            "--no-allow-multiple-definition" => options.allow_multiple_definition = false,
            "--warn-common" => options.warn_common = true,
            "--no-warn-common" => options.warn_common = false,
            "-nostdlib" | "--nostdlib" => nostdlib = true,
            "--color-diagnostics"
            | "--no-color-diagnostics"
            | "--no-add-needed"
            | "--add-needed"
            | "--no-copy-dt-needed-entries"
            | "--copy-dt-needed-entries"
            | "--no-ctors-in-init-array"
            | "--no-keep-memory"
            | "--no-mmap-output-file"
            | "--no-warn-mismatch"
            | "--sort-common"
            | "--stats"
            | "--warn-execstack"
            | "--warn-once"
            | "--warn-shared-textrel"
            | "--no-warn-execstack"
            | "-g"
            | "--long-plt"
            | "--secure-plt"
            | "--qmagic"
            | "--Qy"
            | "--dc"
            | "--dp"
            | "-dc"
            | "-dp"
            | "--eh-frame-hdr=" => {}
            _ => {
                if argument == "--rpath-link" || argument == "-rpath-link" {
                    cursor.next();
                    continue;
                }
                if argument.starts_with("--rpath-link=") || argument.starts_with("-rpath-link=") {
                    continue;
                }
                if IGNORED_FLAGS.contains(&argument) {
                    continue;
                }
                return Err(unknown_option(&raw));
            }
        }
    }
    let _ = nostdlib;
    options.kind = if relocatable {
        OutputKind::Relocatable
    } else if shared {
        OutputKind::SharedLibrary
    } else if pie == Some(true) {
        OutputKind::PositionIndependentExecutable
    } else {
        OutputKind::Executable
    };
    if relocatable && (shared || pie == Some(true)) {
        return Err(invalid_value("-r", if shared { "-shared" } else { "-pie" }));
    }
    Ok(options)
}

fn parse_build_identifier(value: &str) -> Result<BuildIdentifier, Diagnostic> {
    Ok(match value {
        "fast" | "tree" => BuildIdentifier::Fast,
        "md5" => BuildIdentifier::Md5,
        "sha1" => BuildIdentifier::Sha1,
        "uuid" => BuildIdentifier::Uuid,
        "none" => BuildIdentifier::None,
        _ => {
            let digits = value
                .strip_prefix("0x")
                .ok_or_else(|| invalid_value("--build-id", value))?;
            if digits.len() % 2 != 0 {
                return Err(invalid_value("--build-id", value));
            }
            let mut bytes = Vec::new();
            for at in (0..digits.len()).step_by(2) {
                bytes.push(
                    u8::from_str_radix(&digits[at..at + 2], 16)
                        .map_err(|_| invalid_value("--build-id", value))?,
                );
            }
            BuildIdentifier::Hexadecimal(bytes)
        }
    })
}

fn apply_z_keyword(options: &mut Options, keyword: &str) -> Result<(), Diagnostic> {
    match keyword {
        "relro" => options.relocation_read_only = true,
        "norelro" => options.relocation_read_only = false,
        "now" => options.bind_now = true,
        "lazy" => options.bind_now = false,
        "text" => options.text_relocations_forbidden = true,
        "notext" | "textoff" => options.text_relocations_forbidden = false,
        "execstack" => options.executable_stack = true,
        "noexecstack" => options.executable_stack = false,
        "separate-code" => options.separate_code = SeparateCode::Code,
        "separate-loadable-segments" => options.separate_code = SeparateCode::Loadable,
        "noseparate-code" => options.separate_code = SeparateCode::None,
        "copyreloc" => options.copy_relocations = true,
        "nocopyreloc" => options.copy_relocations = false,
        "origin" => options.origin = true,
        "nodelete" => options.no_delete = true,
        "global" => options.load_globally = true,
        "initfirst" => options.initialize_first = true,
        "interpose" => options.interpose = true,
        "nodefaultlib" => options.no_default_library = true,
        "nodlopen" => options.no_dynamic_open = true,
        "rodynamic" => options.read_only_dynamic = true,
        "combreloc" => options.combine_relocations = true,
        "nocombreloc" => options.combine_relocations = false,
        "defs" => options.no_undefined = true,
        "undefs" => options.no_undefined = false,
        "nognustack" | "noexecstack-ignored" => {}
        "pack-relative-relocs" => options.pack_relative_relocations = true,
        "nopack-relative-relocs" => options.pack_relative_relocations = false,
        _ => {
            if let Some(value) = keyword.strip_prefix("max-page-size=") {
                options.maximum_page_size =
                    Some(parse_number(value).ok_or_else(|| invalid_value("-z", keyword))?);
            } else if let Some(value) = keyword.strip_prefix("common-page-size=") {
                options.common_page_size =
                    Some(parse_number(value).ok_or_else(|| invalid_value("-z", keyword))?);
            } else {
                return Err(not_supported(&format!("-z {keyword}")));
            }
        }
    }
    options.z_keywords.push(keyword.to_string());
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Architecture {
    X86,
    X86_64,
    Arm,
    Aarch64,
}

impl Architecture {
    pub fn of_machine(machine_number: u16) -> Option<Architecture> {
        match machine_number {
            machine::X86 => Some(Architecture::X86),
            machine::X86_64 => Some(Architecture::X86_64),
            machine::ARM => Some(Architecture::Arm),
            machine::AARCH64 => Some(Architecture::Aarch64),
            _ => None,
        }
    }

    pub fn machine(self) -> u16 {
        match self {
            Architecture::X86 => machine::X86,
            Architecture::X86_64 => machine::X86_64,
            Architecture::Arm => machine::ARM,
            Architecture::Aarch64 => machine::AARCH64,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Target {
    pub architecture: Architecture,
    pub encoding: Encoding,
    pub default_image_base: u64,
    pub default_maximum_page_size: u64,
    pub default_common_page_size: u64,
    pub uses_addends: bool,
    pub got_entry_size: u64,
    pub got_plt_header_entries: u64,
    pub plt_header_size: u64,
    pub plt_entry_size: u64,
    pub indirect_plt_entry_size: u64,
    pub trap: [u8; 4],
    pub symbolic_relocation: u32,
    pub relative_relocation: u32,
    pub global_data_relocation: u32,
    pub jump_slot_relocation: u32,
    pub copy_relocation: u32,
    pub indirect_relative_relocation: u32,
    pub module_index_relocation: u32,
    pub offset_relocation: u32,
    pub tls_got_relocation: u32,
    pub tls_descriptor_relocation: u32,
    pub got_base_in_got_plt: bool,
    pub thread_control_block_size: u64,
}

impl Target {
    pub fn for_architecture(architecture: Architecture, endian: Endian) -> Target {
        let encoding = |class| Encoding { class, endian };
        match architecture {
            Architecture::X86_64 => Target {
                architecture,
                encoding: encoding(elf::Class::Elf64),
                default_image_base: 0x20_0000,
                default_maximum_page_size: 4096,
                default_common_page_size: 4096,
                uses_addends: true,
                got_entry_size: 8,
                got_plt_header_entries: 3,
                plt_header_size: 16,
                plt_entry_size: 16,
                indirect_plt_entry_size: 16,
                trap: [0xcc; 4],
                symbolic_relocation: 1,
                relative_relocation: 8,
                global_data_relocation: 6,
                jump_slot_relocation: 7,
                copy_relocation: 5,
                indirect_relative_relocation: 37,
                module_index_relocation: 16,
                offset_relocation: 17,
                tls_got_relocation: 18,
                tls_descriptor_relocation: 36,
                got_base_in_got_plt: true,
                thread_control_block_size: 0,
            },
            Architecture::X86 => Target {
                architecture,
                encoding: encoding(elf::Class::Elf32),
                default_image_base: 0x40_0000,
                default_maximum_page_size: 4096,
                default_common_page_size: 4096,
                uses_addends: false,
                got_entry_size: 4,
                got_plt_header_entries: 3,
                plt_header_size: 16,
                plt_entry_size: 16,
                indirect_plt_entry_size: 16,
                trap: [0xcc; 4],
                symbolic_relocation: 1,
                relative_relocation: 8,
                global_data_relocation: 6,
                jump_slot_relocation: 7,
                copy_relocation: 5,
                indirect_relative_relocation: 42,
                module_index_relocation: 35,
                offset_relocation: 36,
                tls_got_relocation: 14,
                tls_descriptor_relocation: 41,
                got_base_in_got_plt: true,
                thread_control_block_size: 0,
            },
            Architecture::Aarch64 => Target {
                architecture,
                encoding: encoding(elf::Class::Elf64),
                default_image_base: 0x20_0000,
                default_maximum_page_size: 65536,
                default_common_page_size: 4096,
                uses_addends: true,
                got_entry_size: 8,
                got_plt_header_entries: 3,
                plt_header_size: 32,
                plt_entry_size: 16,
                indirect_plt_entry_size: 16,
                trap: if endian == Endian::Little {
                    [0x00, 0x00, 0x20, 0xd4]
                } else {
                    [0xd4, 0x20, 0x00, 0x00]
                },
                symbolic_relocation: 257,
                relative_relocation: 1027,
                global_data_relocation: 1025,
                jump_slot_relocation: 1026,
                copy_relocation: 1024,
                indirect_relative_relocation: 1032,
                module_index_relocation: 1028,
                offset_relocation: 1029,
                tls_got_relocation: 1030,
                tls_descriptor_relocation: 1031,
                got_base_in_got_plt: true,
                thread_control_block_size: 16,
            },
            Architecture::Arm => Target {
                architecture,
                encoding: encoding(elf::Class::Elf32),
                default_image_base: 0x1_0000,
                default_maximum_page_size: 65536,
                default_common_page_size: 4096,
                uses_addends: false,
                got_entry_size: 4,
                got_plt_header_entries: 3,
                plt_header_size: 32,
                plt_entry_size: 16,
                indirect_plt_entry_size: 16,
                trap: [0xd4, 0xd4, 0xd4, 0xd4],
                symbolic_relocation: 2,
                relative_relocation: 23,
                global_data_relocation: 21,
                jump_slot_relocation: 22,
                copy_relocation: 20,
                indirect_relative_relocation: 160,
                module_index_relocation: 17,
                offset_relocation: 18,
                tls_got_relocation: 19,
                tls_descriptor_relocation: 13,
                got_base_in_got_plt: true,
                thread_control_block_size: 8,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArmFeatures {
    pub has_exchange: bool,
    pub j1_j2_branch_encoding: bool,
    pub has_move_wide: bool,
    pub has_arm_instruction_set: bool,
    pub has_thumb2_instruction_set: bool,
    pub floating_point_arguments: Option<u64>,
}

pub type FileIndex = usize;
pub type SectionIndex = usize;
pub type SymbolIndex = usize;
pub type OutputIndex = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolState {
    Placeholder,
    Undefined,
    Defined,
    Common { alignment: u64 },
    Shared,
    Lazy { source: LazySource },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LazySource {
    ArchiveMember { archive: usize, member: usize },
    LazyObject { file: FileIndex },
}

pub mod needs {
    pub const GOT: u32 = 1;
    pub const PLT: u32 = 2;
    pub const COPY: u32 = 4;
    pub const TLS_GENERAL_DYNAMIC: u32 = 8;
    pub const TLS_INITIAL_EXECUTABLE: u32 = 16;
    pub const TLS_DESCRIPTOR: u32 = 32;
    pub const GOT_DYNAMIC_OFFSET: u32 = 64;
    pub const DIRECT_RELOCATION: u32 = 128;
}

#[derive(Clone, Debug)]
pub struct SymbolData {
    pub name: String,
    pub file: Option<FileIndex>,
    pub state: SymbolState,
    pub binding: u8,
    pub kind: u8,
    pub other: u8,
    pub value: u64,
    pub size: u64,
    pub section: Option<SectionIndex>,
    pub is_local: bool,
    pub used_in_regular_object: bool,
    pub referenced: bool,
    pub exported: bool,
    pub preemptible: bool,
    pub needs: u32,
    pub got_index: Option<u64>,
    pub plt_index: Option<u64>,
    pub in_indirect_plt: bool,
    pub got_in_indirect_got: bool,
    pub tls_general_dynamic_index: Option<u64>,
    pub tls_descriptor_index: Option<u64>,
    pub dynamic_symbol_index: usize,
    pub version: u16,
    pub discarded_section: u32,
    pub shared_alignment: u64,
    pub shared_protected: bool,
    pub defined_in_shared: bool,
    pub script_defined: bool,
    pub output_section_symbol: Option<OutputIndex>,
    pub output_relative_end: bool,
    pub header_relative: bool,
}

impl SymbolData {
    fn new(name: &str) -> SymbolData {
        SymbolData {
            name: name.to_string(),
            file: None,
            state: SymbolState::Placeholder,
            binding: symbol_binding::GLOBAL,
            kind: symbol_type::NO_TYPE,
            other: 0,
            value: 0,
            size: 0,
            section: None,
            is_local: false,
            used_in_regular_object: false,
            referenced: false,
            exported: false,
            preemptible: false,
            needs: 0,
            got_index: None,
            plt_index: None,
            in_indirect_plt: false,
            got_in_indirect_got: false,
            tls_general_dynamic_index: None,
            tls_descriptor_index: None,
            dynamic_symbol_index: 0,
            version: 1,
            discarded_section: 0,
            shared_alignment: 0,
            shared_protected: false,
            defined_in_shared: false,
            script_defined: false,
            output_section_symbol: None,
            output_relative_end: false,
            header_relative: false,
        }
    }

    pub fn visibility(&self) -> u8 {
        self.other & 3
    }

    pub fn set_visibility(&mut self, value: u8) {
        self.other = (self.other & !3) | (value & 3);
    }

    pub fn is_defined(&self) -> bool {
        self.state == SymbolState::Defined
    }

    pub fn is_undefined(&self) -> bool {
        self.state == SymbolState::Undefined
    }

    pub fn is_shared(&self) -> bool {
        self.state == SymbolState::Shared
    }

    pub fn is_lazy(&self) -> bool {
        matches!(self.state, SymbolState::Lazy { .. })
    }

    pub fn is_common(&self) -> bool {
        matches!(self.state, SymbolState::Common { .. })
    }

    pub fn is_weak(&self) -> bool {
        self.binding == symbol_binding::WEAK
    }

    pub fn is_thread_local(&self) -> bool {
        self.kind == symbol_type::THREAD_LOCAL
    }

    pub fn is_function(&self) -> bool {
        self.kind == symbol_type::FUNCTION
    }

    pub fn is_object(&self) -> bool {
        self.kind == symbol_type::OBJECT
    }

    pub fn is_indirect_function(&self) -> bool {
        self.kind == symbol_type::GNU_INDIRECT_FUNCTION
    }

    pub fn is_section(&self) -> bool {
        self.kind == symbol_type::SECTION
    }

    pub fn is_undefined_weak(&self) -> bool {
        self.is_weak() && self.is_undefined()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Expression {
    None,
    Absolute,
    Addend,
    AddendNegative,
    ProgramCounter,
    Got,
    GotOffset,
    GotProgramCounter,
    GotOnlyProgramCounter,
    GotPltOnlyProgramCounter,
    GotRelative,
    GotPltRelative,
    GotPlt,
    GotPltGotRelative,
    GotPltProgramCounter,
    Plt,
    PltProgramCounter,
    PltGotPlt,
    PltGotRelative,
    RelaxGotProgramCounter,
    RelaxGotProgramCounterNoPic,
    ThreadPointerRelative,
    ThreadPointerRelativeNegative,
    DynamicThreadRelative,
    Size,
    TlsGeneralDynamicProgramCounter,
    TlsGeneralDynamicGot,
    TlsGeneralDynamicGotPlt,
    TlsLocalDynamicProgramCounter,
    TlsLocalDynamicGot,
    TlsLocalDynamicGotPlt,
    TlsLocalDynamicGotOffset,
    TlsDescriptor,
    TlsDescriptorProgramCounter,
    TlsDescriptorGotPlt,
    TlsDescriptorCall,
    Aarch64PageProgramCounter,
    Aarch64GotPageProgramCounter,
    Aarch64GotPage,
    Aarch64TlsDescriptorPage,
    ArmProgramCounterAligned,
    ArmStaticBaseRelative,
    RelaxHint,
}

#[derive(Clone, Debug)]
pub struct Relocation {
    pub expression: Expression,
    pub kind: u32,
    pub offset: u64,
    pub addend: i64,
    pub symbol: SymbolIndex,
}

#[derive(Clone, Debug)]
pub struct DynamicRelocation {
    pub kind: u32,
    pub section: SectionIndex,
    pub offset: u64,
    pub symbol: Option<SymbolIndex>,
    pub use_symbol_value: bool,
    pub addend: i64,
    pub expression: Expression,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Synthetic {
    Interpreter,
    Bss,
    BssRelocationReadOnly,
    BuildIdentifier,
    DynamicStringTable,
    DynamicSymbolTable,
    Dynamic,
    DynamicRelocations,
    RelativeRelocations,
    VersionSymbols,
    VersionDefinitions,
    VersionNeeds,
    GnuHash,
    SystemFiveHash,
    ExceptionFrameHeader,
    ExceptionFrame,
    ArmExceptionIndex,
    Got,
    GotPlt,
    IndirectGotPlt,
    RelocationReadOnlyPadding,
    PltRelocations,
    Plt,
    IndirectPlt,
    GnuProperty,
    SymbolTable,
    SectionNameTable,
    StringTable,
    Comment,
    Merge,
    Common,
    ArmAttributes,
    Thunks,
}

#[derive(Clone, Debug, Default)]
pub struct MergePiece {
    pub input_offset: u64,
    pub length: u64,
    pub output_offset: u64,
    pub live: bool,
    pub hash: u64,
}

#[derive(Clone, Debug)]
pub struct InputSection {
    pub file: Option<FileIndex>,
    pub elf_index: usize,
    pub name: String,
    pub kind: u32,
    pub flags: u64,
    pub alignment: u64,
    pub entry_size: u64,
    pub data: Vec<u8>,
    pub size: u64,
    pub info: u32,
    pub link: u32,
    pub relocation_section: Option<usize>,
    pub live: bool,
    pub output: Option<OutputIndex>,
    pub offset_in_output: u64,
    pub synthetic: Option<Synthetic>,
    pub relocations: Vec<Relocation>,
    pub pieces: Vec<MergePiece>,
    pub merged_into: Option<SectionIndex>,
    pub link_order_dependency: Option<SectionIndex>,
    pub dependents: Vec<SectionIndex>,
    pub is_merge: bool,
    pub is_exception_frame: bool,
    pub common_symbol: Option<SymbolIndex>,
    pub merged_members: Vec<SectionIndex>,
}

impl InputSection {
    fn synthetic(
        name: &str,
        kind: u32,
        flags: u64,
        alignment: u64,
        which: Synthetic,
    ) -> InputSection {
        InputSection {
            file: None,
            elf_index: 0,
            name: name.to_string(),
            kind,
            flags,
            alignment,
            entry_size: 0,
            data: Vec::new(),
            size: 0,
            info: 0,
            link: 0,
            relocation_section: None,
            live: true,
            output: None,
            offset_in_output: 0,
            synthetic: Some(which),
            relocations: Vec::new(),
            pieces: Vec::new(),
            merged_into: None,
            link_order_dependency: None,
            dependents: Vec::new(),
            is_merge: false,
            is_exception_frame: false,
            common_symbol: None,
            merged_members: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum FileState {
    Object,
    Shared,
    LazyObject,
}

#[derive(Clone, Debug)]
pub struct InputFile {
    pub name: String,
    pub archive_name: Option<String>,
    pub elf: ElfFile,
    pub state: FileState,
    pub symbols: Vec<SymbolIndex>,
    pub first_global: usize,
    pub sections: Vec<Option<SectionIndex>>,
    pub discarded: Vec<bool>,
    pub kept_groups: Vec<usize>,
    pub group: usize,
    pub shared_object_name: String,
    pub needed: bool,
    pub as_needed: bool,
    pub needed_libraries: Vec<String>,
    pub version_names: Vec<String>,
    pub version_definitions: Vec<Option<(String, u32)>>,
    pub version_need_information: Vec<(u16, bool)>,
    pub elf_symbols: Vec<elf::Symbol>,
    pub executable_stack_requested: bool,
    pub and_features: u32,
}

#[derive(Clone, Debug)]
pub struct FramePiece {
    pub input_offset: u64,
    pub size: u64,
    pub is_common_information: bool,
    pub first_relocation: Option<usize>,
    pub output_offset: Option<u64>,
    pub record: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct FrameRecord {
    pub common_information: (SectionIndex, usize),
    pub descriptions: Vec<(SectionIndex, usize)>,
}

#[derive(Clone, Debug)]
pub struct VersionNeedOutput {
    pub file_name: u32,
    pub entries: Vec<(u32, u16, bool, u32)>,
}

#[derive(Clone, Debug)]
pub struct ArchiveMember {
    pub name: String,
    pub bytes: Vec<u8>,
    pub extracted: bool,
}

#[derive(Clone, Debug)]
pub struct Archive {
    pub name: String,
    pub members: Vec<ArchiveMember>,
    pub group: usize,
}

fn archive_problem(name: &str, reason: &str) -> Diagnostic {
    problem("EI201", "The archive could not be read.")
        .with_context(format!("File: {name}"))
        .with_context(format!("Reason: {reason}"))
}

pub fn is_archive(bytes: &[u8]) -> bool {
    bytes.starts_with(b"!<arch>\n") || bytes.starts_with(b"!<thin>\n")
}

pub fn read_archive(
    name: &str,
    bytes: &[u8],
    source: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<Vec<ArchiveMember>, Diagnostic> {
    let thin = bytes.starts_with(b"!<thin>\n");
    let mut at = 8usize;
    let mut long_names: Vec<u8> = Vec::new();
    let mut members = Vec::new();
    let folder = name
        .rsplit_once('/')
        .map(|(folder, _)| folder)
        .unwrap_or("");
    while at + 60 <= bytes.len() {
        let header = &bytes[at..at + 60];
        if &header[58..60] != b"`\n" {
            return Err(archive_problem(name, "a member header is malformed"));
        }
        let raw_name = String::from_utf8_lossy(&header[0..16])
            .trim_end()
            .to_string();
        let size: usize = String::from_utf8_lossy(&header[48..58])
            .trim()
            .parse()
            .map_err(|_| archive_problem(name, "a member size is not a number"))?;
        let start = at + 60;
        let is_special = raw_name == "/" || raw_name == "/SYM64/" || raw_name == "//";
        let stored = if thin && !is_special { 0 } else { size };
        if start + stored > bytes.len() {
            return Err(archive_problem(
                name,
                "a member runs past the end of the archive",
            ));
        }
        let body = &bytes[start..start + stored];
        if raw_name == "//" {
            long_names = body.to_vec();
        } else if raw_name == "/" || raw_name == "/SYM64/" || raw_name == "__.SYMDEF" {
        } else {
            let member_name = if let Some(offset) = raw_name.strip_prefix('/') {
                let offset: usize = offset
                    .trim()
                    .parse()
                    .map_err(|_| archive_problem(name, "a long member name is malformed"))?;
                let rest = long_names
                    .get(offset..)
                    .ok_or_else(|| archive_problem(name, "a long member name is out of range"))?;
                let end = rest
                    .iter()
                    .position(|&byte| byte == b'\n')
                    .unwrap_or(rest.len());
                String::from_utf8_lossy(&rest[..end])
                    .trim_end_matches('/')
                    .to_string()
            } else if let Some(length) = raw_name.strip_prefix("#1/") {
                let length: usize = length
                    .trim()
                    .parse()
                    .map_err(|_| archive_problem(name, "a member name is malformed"))?;
                String::from_utf8_lossy(&body[..length.min(body.len())])
                    .trim_end_matches('\0')
                    .to_string()
            } else {
                raw_name.trim_end_matches('/').to_string()
            };
            let member_bytes = if thin {
                let path = if member_name.starts_with('/') || folder.is_empty() {
                    member_name.clone()
                } else {
                    format!("{folder}/{member_name}")
                };
                source(&path).ok_or_else(|| {
                    archive_problem(name, "a thin archive member could not be found")
                        .with_context(format!("Member: {member_name}"))
                })?
            } else {
                body.to_vec()
            };
            members.push(ArchiveMember {
                name: member_name,
                bytes: member_bytes,
                extracted: false,
            });
        }
        at = start + stored;
        if at % 2 == 1 {
            at += 1;
        }
    }
    Ok(members)
}

#[derive(Clone, Debug, Default)]
pub struct LinkOutcome {
    pub outputs: Vec<(String, Vec<u8>)>,
    pub diagnostics: Vec<Diagnostic>,
    pub messages: Vec<String>,
}

impl LinkOutcome {
    pub fn succeeded(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity.is_blocking())
    }
}

#[derive(Clone, Debug)]
pub struct OutputSection {
    pub name: String,
    pub kind: u32,
    pub flags: u64,
    pub alignment: u64,
    pub entry_size: u64,
    pub address: u64,
    pub offset: u64,
    pub size: u64,
    pub link: u32,
    pub info: u32,
    pub members: Vec<SectionIndex>,
    pub has_input_sections: bool,
    pub index: usize,
    pub name_offset: u32,
    pub rank: u32,
    pub relocation_read_only: bool,
    pub load_segment: Option<usize>,
    pub address_fixed: Option<u64>,
    pub page_align: Option<PageAlign>,
    pub used_in_expression: bool,
    pub live: bool,
    pub relocation_for: Option<OutputIndex>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageAlign {
    Congruent,
    Aligned,
    ThreadLocal,
}

impl OutputSection {
    fn new(name: &str) -> OutputSection {
        OutputSection {
            name: name.to_string(),
            kind: section_type::PROGRAM_BITS,
            flags: 0,
            alignment: 1,
            entry_size: 0,
            address: 0,
            offset: 0,
            size: 0,
            link: 0,
            info: 0,
            members: Vec::new(),
            has_input_sections: false,
            index: 0,
            name_offset: 0,
            rank: 0,
            relocation_read_only: false,
            load_segment: None,
            address_fixed: None,
            page_align: None,
            used_in_expression: false,
            live: true,
            relocation_for: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub kind: u32,
    pub flags: u32,
    pub first: Option<OutputIndex>,
    pub last: Option<OutputIndex>,
    pub alignment: u64,
    pub memory_size_override: Option<u64>,
    pub includes_headers: bool,
    pub header_only: bool,
}

impl Segment {
    fn new(kind: u32, flags: u32) -> Segment {
        Segment {
            kind,
            flags,
            first: None,
            last: None,
            alignment: 1,
            memory_size_override: None,
            includes_headers: false,
            header_only: false,
        }
    }
}

pub const HEADER_SECTION: OutputIndex = usize::MAX;

pub struct Context<'s> {
    pub options: Options,
    pub source: &'s dyn Fn(&str) -> Option<Vec<u8>>,
    pub target: Target,
    pub files: Vec<InputFile>,
    pub object_files: Vec<FileIndex>,
    pub shared_files: Vec<FileIndex>,
    pub symbols: Vec<SymbolData>,
    pub names: BTreeMap<String, SymbolIndex>,
    pub sections: Vec<InputSection>,
    pub outputs: Vec<OutputSection>,
    pub order: Vec<OutputIndex>,
    pub segments: Vec<Segment>,
    pub diagnostics: Vec<Diagnostic>,
    pub messages: Vec<String>,
    pub comdat_groups: BTreeMap<String, FileIndex>,
    pub synthetic: BTreeMap<String, SectionIndex>,
    pub dynamic_relocations: Vec<DynamicRelocation>,
    pub relative_relocations: Vec<DynamicRelocation>,
    pub plt_relocations: Vec<DynamicRelocation>,
    pub got_entries: Vec<SymbolIndex>,
    pub dummy: Option<SymbolIndex>,
    pub got_plt_entries: Vec<SymbolIndex>,
    pub indirect_got_plt_entries: Vec<SymbolIndex>,
    pub plt_entries: Vec<SymbolIndex>,
    pub indirect_plt_entries: Vec<SymbolIndex>,
    pub got_size: u64,
    pub tls_index_offset: Option<u64>,
    pub needs_tls_local_dynamic: bool,
    pub has_got_offset_relocation: bool,
    pub has_got_plt_offset_relocation: bool,
    pub got_has_deferred_entries: bool,
    pub symbol_table: Vec<SymbolIndex>,
    pub symbol_table_first_global: usize,
    pub dynamic_symbols: Vec<SymbolIndex>,
    pub dynamic_symbol_name_offsets: Vec<u32>,
    pub dynamic_strings: Vec<u8>,
    pub dynamic_string_offsets: BTreeMap<String, u32>,
    pub gnu_hash_bucket_count: usize,
    pub gnu_hash_symbol_count: usize,
    pub version_auxiliary_count: usize,
    pub version_needs: Vec<VersionNeedOutput>,
    pub dynamic_entry_count: usize,
    pub has_tls_initial_executable: bool,
    pub internal_file: FileIndex,
    pub has_dynamic_symbols: bool,
    pub maximum_page_size: u64,
    pub common_page_size: u64,
    pub image_base: u64,
    pub entry_name: String,
    pub tls_segment: Option<usize>,
    pub header_size: u64,
    pub section_header_offset: u64,
    pub file_size: u64,
    pub reserved: BTreeMap<String, SymbolIndex>,
    pub executable_stack_requested: bool,
    pub archives_seen: BTreeSet<String>,
    pub shared_names_seen: BTreeSet<String>,
    pub dynamic_string_table: StringTable,
    pub input_order: Vec<SectionIndex>,
    pub exception_frame_inputs: Vec<SectionIndex>,
    pub frame_pieces: BTreeMap<SectionIndex, Vec<FramePiece>>,
    pub frame_relocations: BTreeMap<SectionIndex, Vec<RawRelocation>>,
    pub frame_records: Vec<FrameRecord>,
    pub frame_header_entries: Vec<(i64, i64)>,
    pub frame_header_large: bool,
    pub frame_header_size: u64,
    pub and_features: u32,
    pub elf_header_output: OutputIndex,
    pub program_header_output: OutputIndex,
    pub string_table: Vec<u8>,
    pub symbol_string_offsets: Vec<u32>,
    pub section_names: StringTable,
    pub synthesized: Vec<SymbolIndex>,
    pub relocation_read_only_segment: Option<usize>,
    pub arm_features: ArmFeatures,
    pub thunk_state: ThunkState,
    pub synthetic_file_symbol: Option<SymbolIndex>,
}

fn file_display(context: &Context, file: Option<FileIndex>) -> String {
    match file {
        Some(index) if index < context.files.len() => {
            let file = &context.files[index];
            match &file.archive_name {
                Some(archive) => format!("{archive}({})", file.name),
                None => file.name.clone(),
            }
        }
        _ => "<internal>".to_string(),
    }
}

impl<'s> Context<'s> {
    pub fn new(options: Options, source: &'s dyn Fn(&str) -> Option<Vec<u8>>) -> Context<'s> {
        Context {
            options,
            source,
            target: Target::for_architecture(Architecture::X86_64, Endian::Little),
            files: Vec::new(),
            object_files: Vec::new(),
            shared_files: Vec::new(),
            symbols: Vec::new(),
            names: BTreeMap::new(),
            sections: Vec::new(),
            outputs: Vec::new(),
            order: Vec::new(),
            segments: Vec::new(),
            diagnostics: Vec::new(),
            messages: Vec::new(),
            comdat_groups: BTreeMap::new(),
            synthetic: BTreeMap::new(),
            dynamic_relocations: Vec::new(),
            relative_relocations: Vec::new(),
            plt_relocations: Vec::new(),
            got_entries: Vec::new(),
            dummy: None,
            got_plt_entries: Vec::new(),
            indirect_got_plt_entries: Vec::new(),
            plt_entries: Vec::new(),
            indirect_plt_entries: Vec::new(),
            got_size: 0,
            tls_index_offset: None,
            needs_tls_local_dynamic: false,
            has_got_offset_relocation: false,
            has_got_plt_offset_relocation: false,
            got_has_deferred_entries: false,
            symbol_table: Vec::new(),
            symbol_table_first_global: 0,
            dynamic_symbols: Vec::new(),
            dynamic_symbol_name_offsets: Vec::new(),
            dynamic_strings: vec![0],
            dynamic_string_offsets: BTreeMap::from([(String::new(), 0)]),
            gnu_hash_bucket_count: 0,
            gnu_hash_symbol_count: 0,
            version_auxiliary_count: 0,
            version_needs: Vec::new(),
            dynamic_entry_count: 0,
            has_tls_initial_executable: false,
            internal_file: usize::MAX,
            has_dynamic_symbols: false,
            maximum_page_size: 4096,
            common_page_size: 4096,
            image_base: 0,
            entry_name: String::new(),
            tls_segment: None,
            header_size: 0,
            section_header_offset: 0,
            file_size: 0,
            reserved: BTreeMap::new(),
            executable_stack_requested: false,
            archives_seen: BTreeSet::new(),
            shared_names_seen: BTreeSet::new(),
            dynamic_string_table: StringTable::new(),
            input_order: Vec::new(),
            exception_frame_inputs: Vec::new(),
            frame_pieces: BTreeMap::new(),
            frame_relocations: BTreeMap::new(),
            frame_records: Vec::new(),
            frame_header_entries: Vec::new(),
            frame_header_large: false,
            frame_header_size: 0,
            and_features: 0,
            elf_header_output: usize::MAX,
            program_header_output: usize::MAX,
            string_table: vec![0],
            symbol_string_offsets: Vec::new(),
            section_names: StringTable::new(),
            synthesized: Vec::new(),
            relocation_read_only_segment: None,
            arm_features: ArmFeatures::default(),
            thunk_state: ThunkState::default(),
            synthetic_file_symbol: None,
        }
    }

    pub fn error(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn warn(&mut self, mut diagnostic: Diagnostic) {
        if self.options.fatal_warnings {
            diagnostic.severity = Severity::Error;
        } else {
            diagnostic.severity = Severity::Warning;
        }
        self.diagnostics.push(diagnostic);
    }

    pub fn failed(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity.is_blocking())
    }

    pub fn is_pic(&self) -> bool {
        matches!(
            self.options.kind,
            OutputKind::PositionIndependentExecutable | OutputKind::SharedLibrary
        )
    }

    pub fn is_shared(&self) -> bool {
        self.options.kind == OutputKind::SharedLibrary
    }

    pub fn is_relocatable(&self) -> bool {
        self.options.kind == OutputKind::Relocatable
    }

    pub fn encoding(&self) -> Encoding {
        self.target.encoding
    }

    pub fn word_size(&self) -> u64 {
        self.target.encoding.word_size() as u64
    }

    pub fn file_name(&self, file: Option<FileIndex>) -> String {
        file_display(self, file)
    }

    pub fn insert(&mut self, name: &str) -> SymbolIndex {
        if let Some(found) = self.names.get(name) {
            return *found;
        }
        let index = self.symbols.len();
        self.symbols.push(SymbolData::new(name));
        self.names.insert(name.to_string(), index);
        index
    }

    pub fn find(&self, name: &str) -> Option<SymbolIndex> {
        self.names.get(name).copied()
    }

    fn merge_visibility(&mut self, symbol: SymbolIndex, other: u8) {
        let incoming = other & 3;
        if incoming != visibility::DEFAULT {
            let current = self.symbols[symbol].visibility();
            let chosen = if current == visibility::DEFAULT {
                incoming
            } else {
                current.min(incoming)
            };
            self.symbols[symbol].set_visibility(chosen);
        }
    }

    fn overwrite(
        &mut self,
        symbol: SymbolIndex,
        state: SymbolState,
        file: Option<FileIndex>,
        binding: u8,
        other: u8,
        kind: u8,
    ) {
        let entry = &mut self.symbols[symbol];
        if entry.file.is_some_and(|index| {
            matches!(
                self.files.get(index).map(|file| &file.state),
                Some(FileState::Shared)
            )
        }) && state == SymbolState::Defined
        {
            entry.version = elf::version_index::GLOBAL;
        }
        entry.state = state;
        entry.file = file;
        entry.binding = binding;
        entry.kind = kind;
        entry.other = (entry.other & 3) | (other & !3);
    }

    pub fn resolve_undefined(
        &mut self,
        symbol: SymbolIndex,
        file: FileIndex,
        binding: u8,
        other: u8,
        kind: u8,
        discarded_section: u32,
    ) {
        self.merge_visibility(symbol, other);
        let file_is_shared = matches!(self.files[file].state, FileState::Shared);
        let current = self.symbols[symbol].state.clone();
        if current == SymbolState::Placeholder
            || (current == SymbolState::Shared && (other & 3) != visibility::DEFAULT)
            || (current == SymbolState::Undefined
                && binding != symbol_binding::WEAK
                && discarded_section != 0)
        {
            self.overwrite(
                symbol,
                SymbolState::Undefined,
                Some(file),
                binding,
                other,
                kind,
            );
            self.symbols[symbol].discarded_section = discarded_section;
            return;
        }
        if let SymbolState::Lazy { source } = current {
            if binding == symbol_binding::WEAK {
                self.symbols[symbol].binding = symbol_binding::WEAK;
                self.symbols[symbol].kind = kind;
                return;
            }
            self.extract(source);
            return;
        }
        if file_is_shared {
            return;
        }
        if current == SymbolState::Undefined || current == SymbolState::Shared {
            if binding != symbol_binding::WEAK || !self.symbols[symbol].referenced {
                self.symbols[symbol].binding = binding;
            }
            if self.symbols[symbol].file == Some(self.internal_file) {
                self.symbols[symbol].file = Some(file);
                self.symbols[symbol].kind = kind;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn resolve_defined(
        &mut self,
        symbol: SymbolIndex,
        file: Option<FileIndex>,
        binding: u8,
        other: u8,
        kind: u8,
        value: u64,
        size: u64,
        section: Option<SectionIndex>,
    ) {
        self.merge_visibility(symbol, other);
        let replace = match &self.symbols[symbol].state {
            SymbolState::Common { .. } => {
                if self.options.warn_common {
                    let name = self.symbols[symbol].name.clone();
                    self.warn(
                        caution("EI251", "A common symbol is overridden.")
                            .with_context(format!("Symbol: {name}")),
                    );
                }
                binding != symbol_binding::WEAK
            }
            SymbolState::Defined => {
                self.symbols[symbol].binding != symbol_binding::GLOBAL
                    && binding == symbol_binding::GLOBAL
            }
            _ => true,
        };
        if replace {
            self.overwrite(symbol, SymbolState::Defined, file, binding, other, kind);
            let entry = &mut self.symbols[symbol];
            entry.value = value;
            entry.size = size;
            entry.section = section;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn resolve_common(
        &mut self,
        symbol: SymbolIndex,
        file: FileIndex,
        binding: u8,
        other: u8,
        kind: u8,
        alignment: u64,
        size: u64,
    ) {
        self.merge_visibility(symbol, other);
        let current = self.symbols[symbol].state.clone();
        if current == SymbolState::Defined && !self.symbols[symbol].is_weak() {
            if self.options.warn_common {
                let name = self.symbols[symbol].name.clone();
                self.warn(
                    caution("EI251", "A common symbol is overridden.")
                        .with_context(format!("Symbol: {name}")),
                );
            }
            return;
        }
        if let SymbolState::Common { alignment: held } = current {
            if self.options.warn_common {
                let name = self.symbols[symbol].name.clone();
                self.warn(
                    caution("EI252", "A common symbol is declared more than once.")
                        .with_context(format!("Symbol: {name}")),
                );
            }
            self.symbols[symbol].state = SymbolState::Common {
                alignment: held.max(alignment),
            };
            if self.symbols[symbol].size < size {
                self.symbols[symbol].file = Some(file);
                self.symbols[symbol].size = size;
            }
            return;
        }
        let previous_size = if current == SymbolState::Shared {
            self.symbols[symbol].size
        } else {
            0
        };
        self.overwrite(
            symbol,
            SymbolState::Common { alignment },
            Some(file),
            binding,
            other,
            kind,
        );
        self.symbols[symbol].size = size.max(previous_size);
        self.symbols[symbol].section = None;
    }

    pub fn resolve_lazy(&mut self, symbol: SymbolIndex, file: FileIndex) {
        let current = self.symbols[symbol].state.clone();
        if current == SymbolState::Placeholder {
            self.symbols[symbol].state = SymbolState::Lazy {
                source: LazySource::LazyObject { file },
            };
            self.symbols[symbol].file = Some(file);
            self.symbols[symbol].binding = symbol_binding::GLOBAL;
            return;
        }
        if current != SymbolState::Undefined {
            return;
        }
        if self.symbols[symbol].is_weak() {
            let kind = self.symbols[symbol].kind;
            self.symbols[symbol].state = SymbolState::Lazy {
                source: LazySource::LazyObject { file },
            };
            self.symbols[symbol].file = Some(file);
            self.symbols[symbol].kind = kind;
            self.symbols[symbol].binding = symbol_binding::WEAK;
            return;
        }
        self.extract(LazySource::LazyObject { file });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn resolve_shared(
        &mut self,
        symbol: SymbolIndex,
        file: FileIndex,
        binding: u8,
        other: u8,
        kind: u8,
        value: u64,
        size: u64,
        alignment: u64,
        version: u16,
    ) {
        self.symbols[symbol].exported = true;
        let current = self.symbols[symbol].state.clone();
        if current == SymbolState::Placeholder {
            self.set_shared(
                symbol, file, binding, other, kind, value, size, alignment, version,
            );
            return;
        }
        if let SymbolState::Common { .. } = current {
            if size > self.symbols[symbol].size {
                self.symbols[symbol].size = size;
            }
            return;
        }
        if self.symbols[symbol].visibility() == visibility::DEFAULT
            && (current == SymbolState::Undefined || self.symbols[symbol].is_lazy())
        {
            let keep = self.symbols[symbol].binding;
            self.set_shared(
                symbol, file, binding, other, kind, value, size, alignment, version,
            );
            self.symbols[symbol].binding = keep;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn set_shared(
        &mut self,
        symbol: SymbolIndex,
        file: FileIndex,
        binding: u8,
        other: u8,
        kind: u8,
        value: u64,
        size: u64,
        alignment: u64,
        version: u16,
    ) {
        self.overwrite(
            symbol,
            SymbolState::Shared,
            Some(file),
            binding,
            other,
            kind,
        );
        let entry = &mut self.symbols[symbol];
        entry.value = value;
        entry.size = size;
        entry.shared_alignment = alignment;
        entry.version = version;
        entry.section = None;
        entry.shared_protected = other & 3 == visibility::PROTECTED;
        if entry.other & 3 == visibility::PROTECTED {
            entry.set_visibility(visibility::DEFAULT);
        }
    }

    pub fn extract(&mut self, source: LazySource) {
        let LazySource::LazyObject { file } = source else {
            return;
        };
        if !matches!(self.files[file].state, FileState::LazyObject) {
            return;
        }
        self.files[file].state = FileState::Object;
        self.parse_object(file);
    }
}

fn file_missing(path: &str) -> Diagnostic {
    problem("EI205", "An input file could not be found.").with_context(format!("File: {path}"))
}

fn incompatible(file: &str, reference: &str) -> Diagnostic {
    problem(
        "EI209",
        "The input targets a different machine than the output.",
    )
    .with_context(format!("File: {file}"))
    .with_context(format!("Reference: {reference}"))
}

pub fn emulation_target(emulation: &str) -> Option<(Architecture, Endian)> {
    let trimmed = emulation
        .strip_suffix("_fbsd")
        .or_else(|| emulation.strip_suffix("_freebsd"))
        .unwrap_or(emulation);
    Some(match trimmed {
        "elf_x86_64" | "elf_amd64" => (Architecture::X86_64, Endian::Little),
        "elf_i386" | "elf_iamcu" => (Architecture::X86, Endian::Little),
        "aarch64linux" | "aarch64elf" => (Architecture::Aarch64, Endian::Little),
        "aarch64linuxb" | "aarch64elfb" => (Architecture::Aarch64, Endian::Big),
        "armelf" | "armelf_linux_eabi" => (Architecture::Arm, Endian::Little),
        "armelfb" | "armelfb_linux_eabi" => (Architecture::Arm, Endian::Big),
        _ => return None,
    })
}

#[derive(Clone, Copy)]
struct LoadState {
    whole_archive: bool,
    as_needed: bool,
    is_static: bool,
    in_library: bool,
}

impl<'s> Context<'s> {
    fn set_target_from(&mut self, encoding: Encoding, machine_number: u16, file: &str) -> bool {
        let Some(architecture) = Architecture::of_machine(machine_number) else {
            self.error(
                not_supported(&format!("machine {machine_number}"))
                    .with_context(format!("File: {file}")),
            );
            return false;
        };
        let target = Target::for_architecture(architecture, encoding.endian);
        if target.encoding.class != encoding.class {
            self.error(
                not_supported("ELF class for machine").with_context(format!("File: {file}")),
            );
            return false;
        }
        self.target = target;
        true
    }

    pub fn load_inputs(&mut self) {
        let mut chosen_by_emulation = false;
        if let Some(emulation) = self.options.emulation.clone() {
            match emulation_target(&emulation) {
                Some((architecture, endian)) => {
                    self.target = Target::for_architecture(architecture, endian);
                    chosen_by_emulation = true;
                }
                None => {
                    self.error(invalid_value("-m", &emulation));
                    return;
                }
            }
        }
        let mut target_reference: Option<String> = if chosen_by_emulation {
            self.options.emulation.clone()
        } else {
            None
        };
        let mut state = LoadState {
            whole_archive: false,
            as_needed: false,
            is_static: false,
            in_library: false,
        };
        let mut stack: Vec<LoadState> = Vec::new();
        let mut group = 0usize;
        let items = self.options.inputs.clone();
        let mut files_seen = 0usize;
        for item in items {
            match item {
                InputItem::WholeArchive(value) => state.whole_archive = value,
                InputItem::AsNeeded(value) => state.as_needed = value,
                InputItem::Static(value) => state.is_static = value,
                InputItem::StartLibrary => state.in_library = true,
                InputItem::EndLibrary => state.in_library = false,
                InputItem::StartGroup | InputItem::EndGroup => group += 1,
                InputItem::PushState => stack.push(state),
                InputItem::PopState => {
                    if let Some(previous) = stack.pop() {
                        state = previous;
                    }
                }
                InputItem::File(path) => {
                    files_seen += 1;
                    let resolved = self.resolve_path(&path);
                    match (self.source)(&resolved) {
                        Some(bytes) => self.add_file(
                            &path,
                            &resolved,
                            bytes,
                            state,
                            group,
                            false,
                            &mut target_reference,
                        ),
                        None => self.error(file_missing(&path)),
                    }
                }
                InputItem::Library(name) => {
                    files_seen += 1;
                    match self.find_library(&name, state.is_static) {
                        Some((path, bytes)) => self.add_file(
                            &path,
                            &path,
                            bytes,
                            state,
                            group,
                            true,
                            &mut target_reference,
                        ),
                        None => self.error(
                            problem("EI205", "An input file could not be found.")
                                .with_context(format!("Library: -l{name}")),
                        ),
                    }
                }
            }
        }
        if files_seen == 0 && !self.failed() {
            self.error(problem("EI213", "No input files were given."));
        }
    }

    fn resolve_path(&self, path: &str) -> String {
        if let (Some(root), Some(rest)) = (&self.options.sysroot, path.strip_prefix('=')) {
            return format!("{root}{rest}");
        }
        path.to_string()
    }

    fn find_library(&self, name: &str, is_static: bool) -> Option<(String, Vec<u8>)> {
        let mut candidates: Vec<String> = Vec::new();
        for folder in &self.options.library_paths {
            let folder = self.resolve_path(folder);
            if let Some(exact) = name.strip_prefix(':') {
                candidates.push(format!("{folder}/{exact}"));
                continue;
            }
            if !is_static {
                candidates.push(format!("{folder}/lib{name}.so"));
            }
            candidates.push(format!("{folder}/lib{name}.a"));
        }
        for candidate in candidates {
            if let Some(bytes) = (self.source)(&candidate) {
                return Some((candidate, bytes));
            }
        }
        None
    }

    #[allow(clippy::too_many_arguments)]
    fn add_file(
        &mut self,
        display: &str,
        path: &str,
        bytes: Vec<u8>,
        state: LoadState,
        group: usize,
        from_library: bool,
        target_reference: &mut Option<String>,
    ) {
        if is_archive(&bytes) {
            let members = match read_archive(path, &bytes, self.source) {
                Ok(members) => members,
                Err(diagnostic) => {
                    self.error(diagnostic);
                    return;
                }
            };
            for member in members {
                if !elf::is_elf(&member.bytes) {
                    continue;
                }
                let lazy = !state.whole_archive;
                self.add_elf(
                    &member.name,
                    Some(display.to_string()),
                    member.bytes,
                    state,
                    group,
                    lazy,
                    from_library,
                    target_reference,
                );
            }
            return;
        }
        if elf::is_elf(&bytes) {
            self.add_elf(
                display,
                None,
                bytes,
                state,
                group,
                state.in_library,
                from_library,
                target_reference,
            );
            return;
        }
        self.read_input_script(display, path, &bytes, state, group, target_reference);
    }

    fn read_input_script(
        &mut self,
        display: &str,
        path: &str,
        bytes: &[u8],
        state: LoadState,
        group: usize,
        target_reference: &mut Option<String>,
    ) {
        let text = String::from_utf8_lossy(bytes).to_string();
        let tokens = match script_tokens(&text) {
            Ok(tokens) => tokens,
            Err(reason) => {
                self.error(
                    problem("EI239", "The linker script could not be read.")
                        .with_context(format!("File: {display}"))
                        .with_context(format!("Reason: {reason}")),
                );
                return;
            }
        };
        let mut at = 0usize;
        while at < tokens.len() {
            let command = tokens[at].clone();
            at += 1;
            match command.as_str() {
                ";" => {}
                "GROUP" | "INPUT" => {
                    if tokens.get(at).map(String::as_str) != Some("(") {
                        self.script_error(display, "expected (");
                        return;
                    }
                    at += 1;
                    let mut inner = state;
                    let mut as_needed_depth = 0usize;
                    while at < tokens.len() {
                        let token = tokens[at].clone();
                        at += 1;
                        match token.as_str() {
                            ")" if as_needed_depth > 0 => {
                                as_needed_depth -= 1;
                                inner.as_needed = state.as_needed;
                            }
                            ")" => break,
                            "," => {}
                            "AS_NEEDED" => {
                                if tokens.get(at).map(String::as_str) == Some("(") {
                                    at += 1;
                                }
                                as_needed_depth += 1;
                                inner.as_needed = true;
                            }
                            _ => {
                                let name = unquote(&token);
                                self.add_script_file(&name, path, inner, group, target_reference);
                            }
                        }
                    }
                }
                "OUTPUT_FORMAT" | "OUTPUT_ARCH" | "TARGET" => {
                    at = skip_parenthesized(&tokens, at);
                }
                "SEARCH_DIR" => {
                    if tokens.get(at).map(String::as_str) == Some("(") {
                        if let Some(folder) = tokens.get(at + 1) {
                            self.options.library_paths.push(unquote(folder));
                        }
                    }
                    at = skip_parenthesized(&tokens, at);
                }
                "ENTRY" => {
                    if tokens.get(at).map(String::as_str) == Some("(") {
                        if let Some(name) = tokens.get(at + 1) {
                            if self.options.entry.is_none() {
                                self.options.entry = Some(unquote(name));
                            }
                        }
                    }
                    at = skip_parenthesized(&tokens, at);
                }
                "EXTERN" => {
                    let start = at + 1;
                    at = skip_parenthesized(&tokens, at);
                    for token in tokens.iter().take(at.saturating_sub(1)).skip(start) {
                        if token != "," {
                            let id = self.add_unused_undefined(&unquote(token));
                            self.symbols[id].referenced = true;
                            self.options.undefined.push(unquote(token));
                        }
                    }
                }
                other => {
                    self.error(
                        not_supported("linker script command")
                            .with_context(format!("Command: {other}"))
                            .with_context(format!("File: {display}")),
                    );
                    return;
                }
            }
        }
    }

    fn script_error(&mut self, display: &str, reason: &str) {
        self.error(
            problem("EI239", "The linker script could not be read.")
                .with_context(format!("File: {display}"))
                .with_context(format!("Reason: {reason}")),
        );
    }

    fn add_script_file(
        &mut self,
        name: &str,
        script: &str,
        state: LoadState,
        group: usize,
        target_reference: &mut Option<String>,
    ) {
        if let Some(library) = name.strip_prefix("-l") {
            match self.find_library(library, state.is_static) {
                Some((path, bytes)) => {
                    self.add_file(&path, &path, bytes, state, group, true, target_reference)
                }
                None => self.error(
                    problem("EI205", "An input file could not be found.")
                        .with_context(format!("Library: -l{library}")),
                ),
            }
            return;
        }
        let mut candidates: Vec<(String, bool)> = Vec::new();
        if let Some(root) = self.options.sysroot.clone() {
            if script.starts_with(&root) && name.starts_with('/') {
                candidates.push((format!("{root}{name}"), false));
            }
        }
        if name.starts_with('/') {
            candidates.push((name.to_string(), false));
        } else if let Some(rest) = name.strip_prefix('=') {
            match &self.options.sysroot {
                Some(root) => candidates.push((format!("{root}/{rest}"), false)),
                None => candidates.push((rest.to_string(), false)),
            }
        } else {
            if let Some((folder, _)) = script.rsplit_once('/') {
                candidates.push((format!("{folder}/{name}"), false));
            }
            candidates.push((name.to_string(), false));
            for folder in self.options.library_paths.clone() {
                let folder = self.resolve_path(&folder);
                candidates.push((format!("{folder}/{name}"), true));
            }
        }
        for (candidate, from_library) in candidates {
            if let Some(bytes) = (self.source)(&candidate) {
                self.add_file(
                    &candidate,
                    &candidate,
                    bytes,
                    state,
                    group,
                    from_library,
                    target_reference,
                );
                return;
            }
        }
        self.error(file_missing(name));
    }

    #[allow(clippy::too_many_arguments)]
    fn add_elf(
        &mut self,
        name: &str,
        archive: Option<String>,
        bytes: Vec<u8>,
        state: LoadState,
        group: usize,
        lazy: bool,
        from_library: bool,
        target_reference: &mut Option<String>,
    ) {
        let display = match &archive {
            Some(archive) => format!("{archive}({name})"),
            None => name.to_string(),
        };
        let elf = match ElfFile::parse(&bytes) {
            Ok(elf) => elf,
            Err(diagnostic) => {
                self.error(diagnostic.with_context(format!("File: {display}")));
                return;
            }
        };
        let encoding = elf.encoding();
        match target_reference {
            None => {
                if !self.set_target_from(encoding, elf.header.machine, &display) {
                    return;
                }
                *target_reference = Some(display.clone());
            }
            Some(reference) => {
                if elf.header.machine != self.target.architecture.machine()
                    || encoding != self.target.encoding
                {
                    let reference = reference.clone();
                    self.error(incompatible(&display, &reference));
                    return;
                }
            }
        }
        let is_shared = elf.header.file_type == file_type::SHARED;
        if is_shared && state.is_static {
            self.error(
                problem("EI214", "A shared library cannot be linked statically.")
                    .with_context(format!("File: {display}")),
            );
            return;
        }
        if elf.header.file_type != file_type::RELOCATABLE && !is_shared {
            self.error(
                problem(
                    "EI215",
                    "The input file is neither an object nor a shared library.",
                )
                .with_context(format!("File: {display}")),
            );
            return;
        }
        let index = self.files.len();
        let shared_name = if from_library {
            name.rsplit('/').next().unwrap_or(name).to_string()
        } else {
            name.to_string()
        };
        self.files.push(InputFile {
            name: name.to_string(),
            archive_name: archive,
            elf,
            state: if is_shared {
                FileState::Shared
            } else if lazy {
                FileState::LazyObject
            } else {
                FileState::Object
            },
            symbols: Vec::new(),
            first_global: 0,
            sections: Vec::new(),
            discarded: Vec::new(),
            kept_groups: Vec::new(),
            group,
            shared_object_name: shared_name,
            needed: !state.as_needed,
            as_needed: state.as_needed,
            needed_libraries: Vec::new(),
            version_names: Vec::new(),
            version_definitions: Vec::new(),
            version_need_information: Vec::new(),
            elf_symbols: Vec::new(),
            executable_stack_requested: false,
            and_features: 0,
        });
        if is_shared {
            self.parse_shared(index);
        } else if lazy {
            self.parse_lazy(index);
        } else {
            self.parse_object(index);
        }
    }

    fn read_symbols(&mut self, index: FileIndex) -> Option<Vec<elf::Symbol>> {
        let elf = &self.files[index].elf;
        let Some(table) = elf.symbol_table() else {
            return Some(Vec::new());
        };
        let first_global = elf.sections[table].info as usize;
        match elf.symbols(table) {
            Ok(symbols) => {
                self.files[index].first_global = first_global.min(symbols.len());
                Some(symbols)
            }
            Err(diagnostic) => {
                let display = self.file_name(Some(index));
                self.error(diagnostic.with_context(format!("File: {display}")));
                None
            }
        }
    }

    fn parse_lazy(&mut self, index: FileIndex) {
        let Some(symbols) = self.read_symbols(index) else {
            return;
        };
        let first_global = self.files[index].first_global;
        self.files[index].elf_symbols = symbols.clone();
        for symbol in symbols.iter().skip(first_global) {
            if symbol.is_undefined() {
                continue;
            }
            let id = self.insert(&symbol.name);
            self.resolve_lazy(id, index);
            if !matches!(self.files[index].state, FileState::LazyObject) {
                return;
            }
        }
    }

    pub fn parse_object(&mut self, index: FileIndex) {
        self.object_files.push(index);
        let Some(symbols) = (if self.files[index].elf_symbols.is_empty() {
            self.read_symbols(index)
        } else {
            Some(std::mem::take(&mut self.files[index].elf_symbols))
        }) else {
            return;
        };
        let section_count = self.files[index].elf.sections.len();
        self.files[index].discarded = vec![false; section_count];
        let mut kept = Vec::new();
        for position in 0..section_count {
            if self.files[index].elf.sections[position].kind != section_type::GROUP {
                continue;
            }
            let group = match self.files[index].elf.group(position) {
                Ok(group) => group,
                Err(diagnostic) => {
                    self.error(diagnostic);
                    continue;
                }
            };
            let signature = symbols
                .get(group.signature as usize)
                .map(|symbol| symbol.name.clone())
                .unwrap_or_default();
            let keep = group.flags == 0 || !self.comdat_groups.contains_key(&signature);
            if group.flags != 0 && keep {
                self.comdat_groups.insert(signature, index);
            }
            if keep {
                kept.push(position);
            } else {
                for member in group.members {
                    if let Some(slot) = self.files[index].discarded.get_mut(member as usize) {
                        *slot = true;
                    }
                }
            }
        }
        self.files[index].kept_groups = kept;
        let first_global = self.files[index].first_global;
        let mut handles = vec![usize::MAX; symbols.len()];
        for position in first_global..symbols.len() {
            handles[position] = self.insert(&symbols[position].name);
        }
        self.files[index].symbols = handles.clone();
        let mut undefined = Vec::new();
        for position in first_global..symbols.len() {
            let symbol = &symbols[position];
            if symbol.is_undefined() {
                undefined.push(position);
                continue;
            }
            let id = handles[position];
            self.symbols[id].used_in_regular_object = true;
            if symbol.is_common() {
                self.resolve_common(
                    id,
                    index,
                    symbol.binding,
                    symbol.other,
                    symbol.kind,
                    symbol.value,
                    symbol.size,
                );
                continue;
            }
            self.resolve_defined(
                id,
                Some(index),
                symbol.binding,
                symbol.other,
                symbol.kind,
                symbol.value,
                symbol.size,
                None,
            );
        }
        for position in undefined {
            let symbol = symbols[position].clone();
            let id = handles[position];
            self.resolve_undefined(id, index, symbol.binding, symbol.other, symbol.kind, 0);
            self.symbols[id].used_in_regular_object = true;
            self.symbols[id].referenced = true;
        }
        self.files[index].elf_symbols = symbols;
    }

    fn parse_shared(&mut self, index: FileIndex) {
        let elf = self.files[index].elf.clone();
        if let Ok(Some(name)) = elf.shared_object_name() {
            self.files[index].shared_object_name = name;
        }
        self.files[index].needed_libraries = elf.needed_libraries().unwrap_or_default();
        let shared_name = self.files[index].shared_object_name.clone();
        if !self.shared_names_seen.insert(shared_name.clone()) {
            if self.files[index].needed {
                if let Some(other) = self
                    .shared_files
                    .iter()
                    .copied()
                    .find(|other| self.files[*other].shared_object_name == shared_name)
                {
                    self.files[other].needed = true;
                }
            }
            return;
        }
        self.shared_files.push(index);
        let Some(table) = elf.dynamic_symbol_table() else {
            return;
        };
        let symbols = match elf.symbols(table) {
            Ok(symbols) => symbols,
            Err(diagnostic) => {
                self.error(diagnostic);
                return;
            }
        };
        let first_global = (elf.sections[table].info as usize).min(symbols.len());
        self.files[index].first_global = first_global;
        let versions = elf.version_symbols();
        let definitions = elf.version_definitions().unwrap_or_default();
        let mut definition_names: BTreeMap<u16, String> = BTreeMap::new();
        for definition in &definitions {
            if let Some(name) = definition.names.first() {
                definition_names.insert(definition.index, name.clone());
                let slot = definition.index as usize;
                let table = &mut self.files[index].version_definitions;
                if table.len() <= slot {
                    table.resize(slot + 1, None);
                }
                table[slot] = Some((name.clone(), definition.hash));
            }
        }
        let mut need_names: BTreeMap<u16, String> = BTreeMap::new();
        for need in elf.version_needs().unwrap_or_default() {
            for entry in need.entries {
                need_names.insert(entry.other & 0x7fff, entry.name);
            }
        }
        let mut handles = vec![usize::MAX; symbols.len()];
        for position in first_global..symbols.len() {
            let symbol = &symbols[position];
            let version = versions
                .get(position)
                .copied()
                .unwrap_or(elf::version_index::GLOBAL);
            let bare = version & 0x7fff;
            if symbol.is_undefined() {
                let mut name = symbol.name.clone();
                if version != elf::version_index::LOCAL && version != elf::version_index::GLOBAL {
                    if let Some(version_name) = need_names.get(&bare) {
                        name = format!("{name}@{version_name}");
                    }
                }
                let id = self.insert(&name);
                handles[position] = id;
                self.resolve_undefined(id, index, symbol.binding, symbol.other, symbol.kind, 0);
                self.symbols[id].exported = true;
                continue;
            }
            if version == elf::version_index::LOCAL {
                continue;
            }
            let alignment = {
                let mut value = if symbol.value != 0 {
                    1u64 << symbol.value.trailing_zeros()
                } else {
                    u64::MAX
                };
                if symbol.section > 0 && (symbol.section as usize) < elf.sections.len() {
                    value = value.min(elf.sections[symbol.section as usize].alignment);
                }
                if value > u64::from(u32::MAX) {
                    0
                } else {
                    value
                }
            };
            if version == bare {
                let id = self.insert(&symbol.name);
                handles[position] = id;
                self.resolve_shared(
                    id,
                    index,
                    symbol.binding,
                    symbol.other,
                    symbol.kind,
                    symbol.value,
                    symbol.size,
                    alignment,
                    version,
                );
                self.symbols[id].defined_in_shared = true;
                if self.symbols[id].file == Some(index) {
                    self.symbols[id].version = version;
                }
            }
            if version == elf::version_index::GLOBAL {
                continue;
            }
            let Some(version_name) = definition_names.get(&bare) else {
                continue;
            };
            let id = self.insert(&format!("{}@{version_name}", symbol.name));
            self.resolve_shared(
                id,
                index,
                symbol.binding,
                symbol.other,
                symbol.kind,
                symbol.value,
                symbol.size,
                alignment,
                bare,
            );
            self.symbols[id].defined_in_shared = true;
            if self.symbols[id].file == Some(index) {
                self.symbols[id].version = bare;
            }
        }
        self.files[index].symbols = handles;
        self.files[index].elf_symbols = symbols;
    }
}

const XXH_SECRET: [u8; 192] = [
    0xb8, 0xfe, 0x6c, 0x39, 0x23, 0xa4, 0x4b, 0xbe, 0x7c, 0x01, 0x81, 0x2c, 0xf7, 0x21, 0xad, 0x1c,
    0xde, 0xd4, 0x6d, 0xe9, 0x83, 0x90, 0x97, 0xdb, 0x72, 0x40, 0xa4, 0xa4, 0xb7, 0xb3, 0x67, 0x1f,
    0xcb, 0x79, 0xe6, 0x4e, 0xcc, 0xc0, 0xe5, 0x78, 0x82, 0x5a, 0xd0, 0x7d, 0xcc, 0xff, 0x72, 0x21,
    0xb8, 0x08, 0x46, 0x74, 0xf7, 0x43, 0x24, 0x8e, 0xe0, 0x35, 0x90, 0xe6, 0x81, 0x3a, 0x26, 0x4c,
    0x3c, 0x28, 0x52, 0xbb, 0x91, 0xc3, 0x00, 0xcb, 0x88, 0xd0, 0x65, 0x8b, 0x1b, 0x53, 0x2e, 0xa3,
    0x71, 0x64, 0x48, 0x97, 0xa2, 0x0d, 0xf9, 0x4e, 0x38, 0x19, 0xef, 0x46, 0xa9, 0xde, 0xac, 0xd8,
    0xa8, 0xfa, 0x76, 0x3f, 0xe3, 0x9c, 0x34, 0x3f, 0xf9, 0xdc, 0xbb, 0xc7, 0xc7, 0x0b, 0x4f, 0x1d,
    0x8a, 0x51, 0xe0, 0x4b, 0xcd, 0xb4, 0x59, 0x31, 0xc8, 0x9f, 0x7e, 0xc9, 0xd9, 0x78, 0x73, 0x64,
    0xea, 0xc5, 0xac, 0x83, 0x34, 0xd3, 0xeb, 0xc3, 0xc5, 0x81, 0xa0, 0xff, 0xfa, 0x13, 0x63, 0xeb,
    0x17, 0x0d, 0xdd, 0x51, 0xb7, 0xf0, 0xda, 0x49, 0xd3, 0x16, 0x55, 0x26, 0x29, 0xd4, 0x68, 0x9e,
    0x2b, 0x16, 0xbe, 0x58, 0x7d, 0x47, 0xa1, 0xfc, 0x8f, 0xf8, 0xb8, 0xd1, 0x7a, 0xd0, 0x31, 0xce,
    0x45, 0xcb, 0x3a, 0x8f, 0x95, 0x16, 0x04, 0x28, 0xaf, 0xd7, 0xfb, 0xca, 0xbb, 0x4b, 0x40, 0x7e,
];

const PRIME32_1: u64 = 0x9E37_79B1;
const PRIME32_2: u64 = 0x85EB_CA77;
const PRIME32_3: u64 = 0xC2B2_AE3D;
const PRIME64_1: u64 = 0x9E37_79B1_85EB_CA87;
const PRIME64_2: u64 = 0xC2B2_AE3D_27D4_EB4F;
const PRIME64_3: u64 = 0x1656_67B1_9E37_79F9;
const PRIME64_4: u64 = 0x85EB_CA77_C2B2_AE63;
const PRIME64_5: u64 = 0x27D4_EB2F_1656_67C5;

fn little_64(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap_or([0; 8]))
}

fn little_32(bytes: &[u8], at: usize) -> u64 {
    u64::from(u32::from_le_bytes(
        bytes[at..at + 4].try_into().unwrap_or([0; 4]),
    ))
}

fn multiply_fold(left: u64, right: u64) -> u64 {
    let product = u128::from(left) * u128::from(right);
    (product as u64) ^ ((product >> 64) as u64)
}

fn xxh3_avalanche(mut hash: u64) -> u64 {
    hash ^= hash >> 37;
    hash = hash.wrapping_mul(0x1656_6791_9E37_79F9);
    hash ^ (hash >> 32)
}

fn xxh64_avalanche(mut hash: u64) -> u64 {
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(PRIME64_2);
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(PRIME64_3);
    hash ^ (hash >> 32)
}

fn xxh3_mix16(input: &[u8], at: usize, secret_at: usize) -> u64 {
    multiply_fold(
        little_64(input, at) ^ little_64(&XXH_SECRET, secret_at),
        little_64(input, at + 8) ^ little_64(&XXH_SECRET, secret_at + 8),
    )
}

fn xxh3_accumulate_stripe(accumulators: &mut [u64; 8], input: &[u8], at: usize, secret_at: usize) {
    for lane in 0..8 {
        let value = little_64(input, at + 8 * lane);
        let key = value ^ little_64(&XXH_SECRET, secret_at + 8 * lane);
        accumulators[lane ^ 1] = accumulators[lane ^ 1].wrapping_add(value);
        accumulators[lane] =
            accumulators[lane].wrapping_add((key & 0xffff_ffff).wrapping_mul(key >> 32));
    }
}

pub fn xxh3_64(input: &[u8]) -> u64 {
    let length = input.len();
    let length_64 = length as u64;
    if length == 0 {
        return xxh64_avalanche(little_64(&XXH_SECRET, 56) ^ little_64(&XXH_SECRET, 64));
    }
    if length <= 3 {
        let first = u64::from(input[0]);
        let middle = u64::from(input[length >> 1]);
        let last = u64::from(input[length - 1]);
        let combined = (first << 16) | (middle << 24) | last | (length_64 << 8);
        let flip = (little_32(&XXH_SECRET, 0) ^ little_32(&XXH_SECRET, 4)) & 0xffff_ffff;
        return xxh64_avalanche(combined ^ flip);
    }
    if length <= 8 {
        let low = little_32(input, 0);
        let high = little_32(input, length - 4);
        let flip = little_64(&XXH_SECRET, 8) ^ little_64(&XXH_SECRET, 16);
        let combined = high.wrapping_add(low << 32);
        let mut hash = combined ^ flip;
        hash ^= hash.rotate_left(49) ^ hash.rotate_left(24);
        hash = hash.wrapping_mul(0x9FB2_1C65_1E98_DF25);
        hash ^= (hash >> 35).wrapping_add(length_64);
        hash = hash.wrapping_mul(0x9FB2_1C65_1E98_DF25);
        return hash ^ (hash >> 28);
    }
    if length <= 16 {
        let flip_low = little_64(&XXH_SECRET, 24) ^ little_64(&XXH_SECRET, 32);
        let flip_high = little_64(&XXH_SECRET, 40) ^ little_64(&XXH_SECRET, 48);
        let low = little_64(input, 0) ^ flip_low;
        let high = little_64(input, length - 8) ^ flip_high;
        let accumulator = length_64
            .wrapping_add(low.swap_bytes())
            .wrapping_add(high)
            .wrapping_add(multiply_fold(low, high));
        return xxh3_avalanche(accumulator);
    }
    if length <= 128 {
        let mut accumulator = length_64.wrapping_mul(PRIME64_1);
        if length > 32 {
            if length > 64 {
                if length > 96 {
                    accumulator = accumulator.wrapping_add(xxh3_mix16(input, 48, 96));
                    accumulator = accumulator.wrapping_add(xxh3_mix16(input, length - 64, 112));
                }
                accumulator = accumulator.wrapping_add(xxh3_mix16(input, 32, 64));
                accumulator = accumulator.wrapping_add(xxh3_mix16(input, length - 48, 80));
            }
            accumulator = accumulator.wrapping_add(xxh3_mix16(input, 16, 32));
            accumulator = accumulator.wrapping_add(xxh3_mix16(input, length - 32, 48));
        }
        accumulator = accumulator.wrapping_add(xxh3_mix16(input, 0, 0));
        accumulator = accumulator.wrapping_add(xxh3_mix16(input, length - 16, 16));
        return xxh3_avalanche(accumulator);
    }
    if length <= 240 {
        let mut accumulator = length_64.wrapping_mul(PRIME64_1);
        let rounds = length / 16;
        for round in 0..8 {
            accumulator = accumulator.wrapping_add(xxh3_mix16(input, 16 * round, 16 * round));
        }
        accumulator = xxh3_avalanche(accumulator);
        for round in 8..rounds {
            accumulator =
                accumulator.wrapping_add(xxh3_mix16(input, 16 * round, 16 * (round - 8) + 3));
        }
        accumulator = accumulator.wrapping_add(xxh3_mix16(input, length - 16, 119));
        return xxh3_avalanche(accumulator);
    }
    let mut accumulators = [
        PRIME32_3, PRIME64_1, PRIME64_2, PRIME64_3, PRIME64_4, PRIME32_2, PRIME64_5, PRIME32_1,
    ];
    let stripes_per_block = (XXH_SECRET.len() - 64) / 8;
    let block_length = 64 * stripes_per_block;
    let blocks = (length - 1) / block_length;
    for block in 0..blocks {
        for stripe in 0..stripes_per_block {
            xxh3_accumulate_stripe(
                &mut accumulators,
                input,
                block * block_length + stripe * 64,
                stripe * 8,
            );
        }
        for (lane, accumulator) in accumulators.iter_mut().enumerate() {
            let key = little_64(&XXH_SECRET, XXH_SECRET.len() - 64 + 8 * lane);
            let mut value = *accumulator;
            value ^= value >> 47;
            value ^= key;
            *accumulator = value.wrapping_mul(PRIME32_1);
        }
    }
    let stripes = ((length - 1) - block_length * blocks) / 64;
    for stripe in 0..stripes {
        xxh3_accumulate_stripe(
            &mut accumulators,
            input,
            blocks * block_length + stripe * 64,
            stripe * 8,
        );
    }
    xxh3_accumulate_stripe(
        &mut accumulators,
        input,
        length - 64,
        XXH_SECRET.len() - 64 - 7,
    );
    let mut result = length_64.wrapping_mul(PRIME64_1);
    for pair in 0..4 {
        result = result.wrapping_add(multiply_fold(
            accumulators[2 * pair] ^ little_64(&XXH_SECRET, 11 + 16 * pair),
            accumulators[2 * pair + 1] ^ little_64(&XXH_SECRET, 11 + 16 * pair + 8),
        ));
    }
    xxh3_avalanche(result)
}

pub fn chunked_hash(data: &[u8], size: usize, hash: &dyn Fn(&[u8]) -> Vec<u8>) -> Vec<u8> {
    let mut hashes = Vec::new();
    for chunk in data.chunks(1024 * 1024) {
        let mut value = hash(chunk);
        value.resize(size, 0);
        hashes.extend_from_slice(&value);
    }
    let mut value = hash(&hashes);
    value.resize(size, 0);
    value
}

const BLAKE3_INITIAL: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

const BLAKE3_PERMUTATION: [usize; 16] = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8];

fn blake3_compress(
    chaining: &[u32; 8],
    block: &[u32; 16],
    counter: u64,
    length: u32,
    flags: u32,
) -> [u32; 16] {
    let mut state = [
        chaining[0],
        chaining[1],
        chaining[2],
        chaining[3],
        chaining[4],
        chaining[5],
        chaining[6],
        chaining[7],
        BLAKE3_INITIAL[0],
        BLAKE3_INITIAL[1],
        BLAKE3_INITIAL[2],
        BLAKE3_INITIAL[3],
        counter as u32,
        (counter >> 32) as u32,
        length,
        flags,
    ];
    let mut message = *block;
    let mix = |state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize, x: u32, y: u32| {
        state[a] = state[a].wrapping_add(state[b]).wrapping_add(x);
        state[d] = (state[d] ^ state[a]).rotate_right(16);
        state[c] = state[c].wrapping_add(state[d]);
        state[b] = (state[b] ^ state[c]).rotate_right(12);
        state[a] = state[a].wrapping_add(state[b]).wrapping_add(y);
        state[d] = (state[d] ^ state[a]).rotate_right(8);
        state[c] = state[c].wrapping_add(state[d]);
        state[b] = (state[b] ^ state[c]).rotate_right(7);
    };
    for round in 0..7 {
        mix(&mut state, 0, 4, 8, 12, message[0], message[1]);
        mix(&mut state, 1, 5, 9, 13, message[2], message[3]);
        mix(&mut state, 2, 6, 10, 14, message[4], message[5]);
        mix(&mut state, 3, 7, 11, 15, message[6], message[7]);
        mix(&mut state, 0, 5, 10, 15, message[8], message[9]);
        mix(&mut state, 1, 6, 11, 12, message[10], message[11]);
        mix(&mut state, 2, 7, 8, 13, message[12], message[13]);
        mix(&mut state, 3, 4, 9, 14, message[14], message[15]);
        if round < 6 {
            let previous = message;
            for (index, source) in BLAKE3_PERMUTATION.iter().enumerate() {
                message[index] = previous[*source];
            }
        }
    }
    for index in 0..8 {
        state[index] ^= state[index + 8];
        state[index + 8] ^= chaining[index];
    }
    state
}

fn blake3_words(bytes: &[u8]) -> [u32; 16] {
    let mut padded = [0u8; 64];
    padded[..bytes.len()].copy_from_slice(bytes);
    let mut words = [0u32; 16];
    for (index, word) in words.iter_mut().enumerate() {
        *word = u32::from_le_bytes([
            padded[4 * index],
            padded[4 * index + 1],
            padded[4 * index + 2],
            padded[4 * index + 3],
        ]);
    }
    words
}

pub fn blake3(data: &[u8]) -> [u8; 32] {
    const CHUNK_START: u32 = 1;
    const CHUNK_END: u32 = 2;
    const PARENT: u32 = 4;
    const ROOT: u32 = 8;
    let chunks: Vec<&[u8]> = if data.is_empty() {
        vec![data]
    } else {
        data.chunks(1024).collect()
    };
    let mut stack: Vec<[u32; 8]> = Vec::new();
    let mut last_output: ([u32; 8], [u32; 16], u64, u32, u32) =
        (BLAKE3_INITIAL, [0; 16], 0, 0, CHUNK_START | CHUNK_END);
    for (counter, chunk) in chunks.iter().enumerate() {
        let blocks: Vec<&[u8]> = if chunk.is_empty() {
            vec![*chunk]
        } else {
            chunk.chunks(64).collect()
        };
        let mut chaining = BLAKE3_INITIAL;
        for (index, block) in blocks.iter().enumerate() {
            let mut flags = 0;
            if index == 0 {
                flags |= CHUNK_START;
            }
            if index + 1 == blocks.len() {
                flags |= CHUNK_END;
                last_output = (
                    chaining,
                    blake3_words(block),
                    counter as u64,
                    block.len() as u32,
                    flags,
                );
            } else {
                let words = blake3_words(block);
                let state = blake3_compress(&chaining, &words, counter as u64, 64, flags);
                chaining.copy_from_slice(&state[..8]);
            }
        }
        if counter + 1 < chunks.len() {
            let (input, words, count, length, flags) = last_output;
            let state = blake3_compress(&input, &words, count, length, flags);
            let mut value = [0u32; 8];
            value.copy_from_slice(&state[..8]);
            let mut total = counter as u64 + 1;
            while total & 1 == 0 {
                let Some(left) = stack.pop() else {
                    break;
                };
                let mut block = [0u32; 16];
                block[..8].copy_from_slice(&left);
                block[8..].copy_from_slice(&value);
                let state = blake3_compress(&BLAKE3_INITIAL, &block, 0, 64, PARENT);
                value.copy_from_slice(&state[..8]);
                total >>= 1;
            }
            stack.push(value);
        }
    }
    while let Some(left) = stack.pop() {
        let (input, words, count, length, flags) = last_output;
        let state = blake3_compress(&input, &words, count, length, flags);
        let mut right = [0u32; 8];
        right.copy_from_slice(&state[..8]);
        let mut block = [0u32; 16];
        block[..8].copy_from_slice(&left);
        block[8..].copy_from_slice(&right);
        last_output = (BLAKE3_INITIAL, block, 0, 64, PARENT);
    }
    let (input, words, _, length, flags) = last_output;
    let state = blake3_compress(&input, &words, 0, length, flags | ROOT);
    let mut out = [0u8; 32];
    for index in 0..8 {
        out[4 * index..4 * index + 4].copy_from_slice(&state[index].to_le_bytes());
    }
    out
}

pub fn message_digest_5(data: &[u8]) -> [u8; 16] {
    const SHIFTS: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    const CONSTANTS: [u32; 64] = [
        0xd76a_a478,
        0xe8c7_b756,
        0x2420_70db,
        0xc1bd_ceee,
        0xf57c_0faf,
        0x4787_c62a,
        0xa830_4613,
        0xfd46_9501,
        0x6980_98d8,
        0x8b44_f7af,
        0xffff_5bb1,
        0x895c_d7be,
        0x6b90_1122,
        0xfd98_7193,
        0xa679_438e,
        0x49b4_0821,
        0xf61e_2562,
        0xc040_b340,
        0x265e_5a51,
        0xe9b6_c7aa,
        0xd62f_105d,
        0x0244_1453,
        0xd8a1_e681,
        0xe7d3_fbc8,
        0x21e1_cde6,
        0xc337_07d6,
        0xf4d5_0d87,
        0x455a_14ed,
        0xa9e3_e905,
        0xfcef_a3f8,
        0x676f_02d9,
        0x8d2a_4c8a,
        0xfffa_3942,
        0x8771_f681,
        0x6d9d_6122,
        0xfde5_380c,
        0xa4be_ea44,
        0x4bde_cfa9,
        0xf6bb_4b60,
        0xbebf_bc70,
        0x289b_7ec6,
        0xeaa1_27fa,
        0xd4ef_3085,
        0x0488_1d05,
        0xd9d4_d039,
        0xe6db_99e5,
        0x1fa2_7cf8,
        0xc4ac_5665,
        0xf429_2244,
        0x432a_ff97,
        0xab94_23a7,
        0xfc93_a039,
        0x655b_59c3,
        0x8f0c_cc92,
        0xffef_f47d,
        0x8584_5dd1,
        0x6fa8_7e4f,
        0xfe2c_e6e0,
        0xa301_4314,
        0x4e08_11a1,
        0xf753_7e82,
        0xbd3a_f235,
        0x2ad7_d2bb,
        0xeb86_d391,
    ];
    let mut message = data.to_vec();
    let bits = (data.len() as u64).wrapping_mul(8);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bits.to_le_bytes());
    let mut state: [u32; 4] = [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476];
    for block in message.chunks(64) {
        let words: Vec<u32> = block
            .chunks(4)
            .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
            .collect();
        let [mut a, mut b, mut c, mut d] = state;
        for index in 0..64 {
            let (mixed, word) = match index / 16 {
                0 => ((b & c) | (!b & d), index),
                1 => ((d & b) | (!d & c), (5 * index + 1) % 16),
                2 => (b ^ c ^ d, (3 * index + 5) % 16),
                _ => (c ^ (b | !d), (7 * index) % 16),
            };
            let rotated = a
                .wrapping_add(mixed)
                .wrapping_add(CONSTANTS[index])
                .wrapping_add(words[word])
                .rotate_left(SHIFTS[index]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(rotated);
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
    }
    let mut out = [0u8; 16];
    for (index, word) in state.iter().enumerate() {
        out[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    out
}

pub fn secure_hash_1(data: &[u8]) -> [u8; 20] {
    let mut message = data.to_vec();
    let bits = (data.len() as u64).wrapping_mul(8);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bits.to_be_bytes());
    let mut state: [u32; 5] = [
        0x6745_2301,
        0xefcd_ab89,
        0x98ba_dcfe,
        0x1032_5476,
        0xc3d2_e1f0,
    ];
    for block in message.chunks(64) {
        let mut words = [0u32; 80];
        for (index, word) in block.chunks(4).enumerate() {
            words[index] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for index in 16..80 {
            words[index] =
                (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16])
                    .rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = state;
        for (index, word) in words.iter().enumerate() {
            let (mixed, constant) = match index / 20 {
                0 => ((b & c) | (!b & d), 0x5a82_7999),
                1 => (b ^ c ^ d, 0x6ed9_eba1),
                2 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
                _ => (b ^ c ^ d, 0xca62_c1d6u32),
            };
            let next = a
                .rotate_left(5)
                .wrapping_add(mixed)
                .wrapping_add(e)
                .wrapping_add(constant)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = next;
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
    }
    let mut out = [0u8; 20];
    for (index, word) in state.iter().enumerate() {
        out[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn read_unsigned_leb(data: &[u8], at: &mut usize) -> Option<u64> {
    let mut value = 0u64;
    let mut shift = 0;
    loop {
        let byte = *data.get(*at)?;
        *at += 1;
        if shift < 64 {
            value |= u64::from(byte & 0x7f) << shift;
        }
        shift += 7;
        if byte & 0x80 == 0 {
            return Some(value);
        }
    }
}

pub fn parse_arm_attributes(data: &[u8], encoding: Encoding) -> Option<BTreeMap<u64, u64>> {
    if data.first() != Some(&b'A') {
        return None;
    }
    let mut values = BTreeMap::new();
    let mut at = 1usize;
    while at < data.len() {
        let length = encoding.read_u32(data, at)? as usize;
        if length < 4 || at + length > data.len() {
            return None;
        }
        let end = at + length;
        let mut cursor = at + 4;
        let name_end = data[cursor..end].iter().position(|&byte| byte == 0)? + cursor;
        let vendor = &data[cursor..name_end];
        cursor = name_end + 1;
        if vendor == b"aeabi" {
            while cursor < end {
                let tag = read_unsigned_leb(data, &mut cursor)?;
                let size = encoding.read_u32(data, cursor)? as usize;
                let sub_end = (cursor - 1 + size).min(end);
                cursor += 4;
                if tag != 1 {
                    cursor = sub_end;
                    continue;
                }
                while cursor < sub_end {
                    let attribute = read_unsigned_leb(data, &mut cursor)?;
                    let is_string = matches!(attribute, 4 | 5 | 65 | 67)
                        || (attribute > 32
                            && attribute % 2 == 1
                            && attribute != 65
                            && attribute != 67);
                    if attribute == 32 {
                        let flag = read_unsigned_leb(data, &mut cursor)?;
                        let _ = flag;
                        let stop = data[cursor..sub_end].iter().position(|&byte| byte == 0)?;
                        cursor += stop + 1;
                    } else if is_string {
                        let stop = data[cursor..sub_end].iter().position(|&byte| byte == 0)?;
                        cursor += stop + 1;
                    } else {
                        let value = read_unsigned_leb(data, &mut cursor)?;
                        values.insert(attribute, value);
                    }
                }
                cursor = sub_end;
            }
        }
        at = end;
    }
    Some(values)
}

pub fn is_debug_section(name: &str) -> bool {
    name.starts_with(".debug") || name.starts_with(".zdebug")
}

impl<'s> Context<'s> {
    #[allow(clippy::needless_range_loop)]
    pub fn initialize_sections(&mut self, index: FileIndex) {
        let count = self.files[index].elf.sections.len();
        let mut map: Vec<Option<SectionIndex>> = vec![None; count];
        let relocatable = self.is_relocatable();
        for position in 1..count {
            if self.files[index].discarded[position] {
                continue;
            }
            let header = self.files[index].elf.sections[position].clone();
            if header.flags & section_flag::EXCLUDE != 0 && !relocatable {
                self.files[index].discarded[position] = true;
                continue;
            }
            match header.kind {
                section_type::GROUP => {
                    if !relocatable {
                        self.files[index].discarded[position] = true;
                    }
                    continue;
                }
                section_type::SYMBOL_TABLE_INDICES
                | section_type::SYMBOL_TABLE
                | section_type::STRING_TABLE
                | section_type::RELOCATIONS
                | section_type::RELOCATIONS_WITH_ADDENDS
                | section_type::NULL => continue,
                _ => {}
            }
            if header.kind == section_type::AARCH64_ATTRIBUTES
                && self.target.architecture == Architecture::Aarch64
            {
                self.files[index].discarded[position] = true;
                continue;
            }
            if header.kind == section_type::ARM_ATTRIBUTES
                && self.target.architecture == Architecture::Arm
            {
                self.read_arm_attributes(index, &header.data);
                if self.synthetic.contains_key(".ARM.attributes") {
                    self.files[index].discarded[position] = true;
                    continue;
                }
            }
            if header.name.starts_with(".n") {
                match header.name.as_str() {
                    ".note.GNU-stack" => {
                        if header.flags & section_flag::EXECUTE != 0 && !relocatable {
                            self.files[index].executable_stack_requested = true;
                        }
                        self.files[index].discarded[position] = true;
                        continue;
                    }
                    ".note.gnu.property" => {
                        self.read_gnu_property(index, &header);
                        self.files[index].discarded[position] = true;
                        continue;
                    }
                    ".note.GNU-split-stack" | ".note.GNU-no-split-stack" | ".note.gnu.build-id" => {
                        self.files[index].discarded[position] = true;
                        continue;
                    }
                    _ => {}
                }
            }
            let is_exception_frame = header.name == ".eh_frame" && !relocatable;
            let is_merge = header.flags & section_flag::MERGE != 0
                && (self.options.optimize > 0 || relocatable)
                && header.size > 0
                && header.entry_size > 0
                && header.flags & section_flag::WRITE == 0
                && header.size.is_multiple_of(header.entry_size);
            let size = if header.kind == section_type::NO_BITS {
                header.size
            } else {
                header.data.len() as u64
            };
            let section = InputSection {
                file: Some(index),
                elf_index: position,
                name: header.name.clone(),
                kind: header.kind,
                flags: header.flags,
                alignment: header.alignment.max(1),
                entry_size: header.entry_size,
                data: if header.kind == section_type::NO_BITS {
                    Vec::new()
                } else {
                    header.data.clone()
                },
                size,
                info: header.info,
                link: header.link,
                relocation_section: None,
                live: true,
                output: None,
                offset_in_output: 0,
                synthetic: None,
                relocations: Vec::new(),
                pieces: Vec::new(),
                merged_into: None,
                link_order_dependency: None,
                dependents: Vec::new(),
                is_merge,
                is_exception_frame,
                common_symbol: None,
                merged_members: Vec::new(),
            };
            map[position] = Some(self.sections.len());
            if header.kind == section_type::ARM_ATTRIBUTES
                && self.target.architecture == Architecture::Arm
            {
                self.synthetic
                    .insert(".ARM.attributes".to_string(), self.sections.len());
            }
            self.sections.push(section);
        }
        for position in 1..count {
            if self.files[index].discarded[position] {
                continue;
            }
            let header = &self.files[index].elf.sections[position];
            let kind = header.kind;
            if kind == section_type::RELOCATIONS || kind == section_type::RELOCATIONS_WITH_ADDENDS {
                let info = header.info as usize;
                let target = match map.get(info) {
                    Some(Some(target)) => *target,
                    _ => {
                        if (info >= count || !self.files[index].discarded[info])
                            && (info >= count || map.get(info).is_none())
                        {
                            let display = self.file_name(Some(index));
                            self.error(
                                problem(
                                    "EI216",
                                    "A relocation section points at a section that does not exist.",
                                )
                                .with_context(format!("File: {display}"))
                                .with_context(format!("Index: {position}"))
                                .with_context(format!("Information: {info}")),
                            );
                        }
                        continue;
                    }
                };
                if self.sections[target].is_merge {
                    self.sections[target].is_merge = false;
                }
                self.sections[target].relocation_section = Some(position);
                continue;
            }
            let flags = header.flags;
            let link = header.link as usize;
            if link == 0 || flags & section_flag::LINK_ORDER == 0 {
                continue;
            }
            let Some(own) = map[position] else {
                continue;
            };
            match map.get(link).copied().flatten() {
                Some(dependency) => {
                    self.sections[own].link_order_dependency = Some(dependency);
                    self.sections[dependency].dependents.push(own);
                }
                None => {
                    if link >= count || !self.files[index].discarded[link] {
                        let display = self.file_name(Some(index));
                        self.error(
                            problem("EI217", "A section links to a section that does not exist.")
                                .with_context(format!("File: {display}"))
                                .with_context(format!("Link: {link}")),
                        );
                    } else {
                        self.sections[own].live = false;
                        self.files[index].discarded[position] = true;
                        map[position] = None;
                    }
                }
            }
        }
        self.files[index].sections = map;
        let symbols = self.files[index].elf_symbols.clone();
        let first_global = self.files[index].first_global;
        for (position, symbol) in symbols.iter().enumerate().take(first_global) {
            let mut section_number = symbol.section;
            if section_number >= section_index::RESERVED_LOW {
                section_number = 0;
            }
            let section = self.files[index]
                .sections
                .get(section_number as usize)
                .copied()
                .flatten();
            let discarded = (section_number as usize) < count
                && section_number != 0
                && self.files[index].discarded[section_number as usize];
            let mut data = SymbolData::new(&symbol.name);
            data.file = Some(index);
            data.is_local = true;
            data.binding = symbol_binding::LOCAL;
            data.kind = symbol.kind;
            data.other = symbol.other;
            data.used_in_regular_object = true;
            if symbol.section == section_index::UNDEFINED || discarded {
                data.state = SymbolState::Undefined;
                data.discarded_section = section_number;
            } else {
                data.state = SymbolState::Defined;
                data.value = symbol.value;
                data.size = symbol.size;
                data.section = section;
            }
            let id = self.symbols.len();
            self.symbols.push(data);
            if self.files[index].symbols.len() <= position {
                self.files[index].symbols.resize(position + 1, usize::MAX);
            }
            self.files[index].symbols[position] = id;
        }
    }

    fn read_arm_attributes(&mut self, index: FileIndex, data: &[u8]) {
        let Some(attributes) = parse_arm_attributes(data, self.encoding()) else {
            let display = self.file_name(Some(index));
            self.warn(
                caution("EI254", "The ARM attributes of an input could not be read.")
                    .with_context(format!("File: {display}")),
            );
            return;
        };
        if let Some(architecture) = attributes.get(&6).copied() {
            match architecture {
                0..=2 => {}
                3..=7 | 9 => self.arm_features.has_exchange = true,
                _ => {
                    self.arm_features.has_exchange = true;
                    self.arm_features.j1_j2_branch_encoding = true;
                    if architecture != 11 && architecture != 12 {
                        self.arm_features.has_move_wide = true;
                    }
                }
            }
        }
        if attributes.get(&8).is_some_and(|value| *value >= 1) {
            self.arm_features.has_arm_instruction_set = true;
        }
        if attributes.get(&9).is_some_and(|value| *value >= 2) {
            self.arm_features.has_thumb2_instruction_set = true;
        }
        if let Some(arguments) = attributes.get(&28).copied() {
            if arguments != 3 {
                match self.arm_features.floating_point_arguments {
                    None => self.arm_features.floating_point_arguments = Some(arguments),
                    Some(held) if held != arguments => {
                        let display = self.file_name(Some(index));
                        self.error(
                            problem(
                                "EI236",
                                "Inputs disagree on how floating point arguments are passed.",
                            )
                            .with_context(format!("File: {display}")),
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    fn read_gnu_property(&mut self, index: FileIndex, header: &Section) {
        let feature = match self.target.architecture {
            Architecture::X86 | Architecture::X86_64 => 0xc000_0002u32,
            Architecture::Aarch64 => 0xc000_0000u32,
            Architecture::Arm => return,
        };
        let encoding = self.encoding();
        let Ok(notes) = elf::decode_notes(encoding, &header.data, header.alignment) else {
            return;
        };
        for note in notes {
            if note.kind != elf::note_type::GNU_PROPERTY || note.name != b"GNU" {
                continue;
            }
            let mut at = 0usize;
            let data = &note.description;
            while at + 8 <= data.len() {
                let kind = encoding.read_u32(data, at).unwrap_or(0);
                let size = encoding.read_u32(data, at + 4).unwrap_or(0) as usize;
                at += 8;
                if at + size > data.len() {
                    break;
                }
                if kind == feature && size >= 4 {
                    self.files[index].and_features |= encoding.read_u32(data, at).unwrap_or(0);
                }
                at += align_to(size as u64, encoding.word_size() as u64) as usize;
            }
        }
    }

    pub fn post_parse(&mut self, index: FileIndex) {
        let symbols = self.files[index].elf_symbols.clone();
        let first_global = self.files[index].first_global;
        for (position, symbol) in symbols.iter().enumerate().skip(first_global) {
            let id = self.files[index].symbols[position];
            if symbol.binding != symbol_binding::GLOBAL
                && symbol.binding != symbol_binding::WEAK
                && symbol.binding != symbol_binding::GNU_UNIQUE
            {
                let display = self.file_name(Some(index));
                self.error(
                    problem("EI218", "A symbol has a binding the linker does not know.")
                        .with_context(format!("File: {display}"))
                        .with_context(format!("Index: {position}")),
                );
            }
            if !self.symbols[id].is_defined() || symbol.section == section_index::UNDEFINED {
                continue;
            }
            let mut section_number = symbol.section;
            if section_number >= section_index::RESERVED_LOW {
                if section_number == section_index::COMMON {
                    continue;
                }
                section_number = 0;
            }
            let count = self.files[index].elf.sections.len();
            if section_number as usize >= count {
                let display = self.file_name(Some(index));
                self.error(
                    problem("EI219", "A symbol points at a section that does not exist.")
                        .with_context(format!("File: {display}"))
                        .with_context(format!("Index: {section_number}")),
                );
                continue;
            }
            if section_number != 0 && self.files[index].discarded[section_number as usize] {
                if self.symbols[id].file == Some(index) {
                    let entry = &mut self.symbols[id];
                    entry.state = SymbolState::Undefined;
                    entry.discarded_section = section_number;
                    entry.section = None;
                }
                continue;
            }
            let section = self.files[index].sections[section_number as usize];
            if self.symbols[id].file == Some(index) {
                self.symbols[id].section = section;
                continue;
            }
            if self.symbols[id].binding == symbol_binding::WEAK
                || symbol.binding == symbol_binding::WEAK
            {
                continue;
            }
            if self.options.allow_multiple_definition {
                continue;
            }
            let name = self.symbols[id].name.clone();
            let first = self.definition_place(id);
            let second = match section {
                Some(section) => format!(
                    "{}:({}+0x{:x})",
                    self.file_name(Some(index)),
                    self.sections[section].name,
                    symbol.value
                ),
                None => self.file_name(Some(index)),
            };
            self.error(
                problem("EI206", "A symbol is defined more than once.")
                    .with_context(format!("Symbol: {name}"))
                    .with_context(format!("Definition: {first}"))
                    .with_context(format!("Definition: {second}")),
            );
        }
    }

    pub fn definition_place(&self, id: SymbolIndex) -> String {
        let symbol = &self.symbols[id];
        match symbol.section {
            Some(section) => format!(
                "{}:({}+0x{:x})",
                self.file_name(symbol.file),
                self.sections[section].name,
                symbol.value
            ),
            None => self.file_name(symbol.file),
        }
    }

    pub fn split_merge_sections(&mut self) {
        let gc = self.options.gc_sections;
        for section in self.sections.iter_mut() {
            if !section.is_merge || !section.live || !section.pieces.is_empty() {
                continue;
            }
            let live = section.flags & section_flag::ALLOCATE == 0 || !gc;
            let entry = section.entry_size.max(1) as usize;
            let data = &section.data;
            let mut pieces = Vec::new();
            if section.flags & section_flag::STRINGS != 0 {
                let mut at = 0usize;
                while at < data.len() {
                    let mut end = at;
                    while end + entry <= data.len()
                        && data[end..end + entry].iter().any(|&b| b != 0)
                    {
                        end += entry;
                    }
                    let content_end = end.min(data.len());
                    let piece_end = (end + entry).min(data.len());
                    pieces.push(MergePiece {
                        input_offset: at as u64,
                        length: (piece_end - at) as u64,
                        output_offset: 0,
                        live,
                        hash: (xxh3_64(&data[at..content_end]) as u32 >> 1) as u64,
                    });
                    at = piece_end;
                }
            } else {
                let mut at = 0usize;
                while at + entry <= data.len() {
                    pieces.push(MergePiece {
                        input_offset: at as u64,
                        length: entry as u64,
                        output_offset: 0,
                        live,
                        hash: (xxh3_64(&data[at..at + entry]) as u32 >> 1) as u64,
                    });
                    at += entry;
                }
            }
            section.pieces = pieces;
        }
    }

    pub fn replace_common_symbols(&mut self) {
        for position in 0..self.object_files.len() {
            let file = self.object_files[position];
            let handles = self.files[file].symbols.clone();
            let first_global = self.files[file].first_global;
            for id in handles.into_iter().skip(first_global) {
                if id == usize::MAX {
                    continue;
                }
                let SymbolState::Common { alignment } = self.symbols[id].state else {
                    continue;
                };
                let mut section = InputSection::synthetic(
                    "COMMON",
                    section_type::NO_BITS,
                    section_flag::WRITE | section_flag::ALLOCATE,
                    alignment.max(1),
                    Synthetic::Common,
                );
                section.file = Some(file);
                section.size = self.symbols[id].size;
                section.common_symbol = Some(id);
                section.file = self.symbols[id].file;
                let section_id = self.sections.len();
                self.sections.push(section);
                self.input_order.push(section_id);
                let entry = &mut self.symbols[id];
                entry.state = SymbolState::Defined;
                entry.section = Some(section_id);
                entry.value = 0;
            }
        }
    }
}

pub const INTERNAL: FileIndex = usize::MAX;

type MergeShard = (Vec<u8>, BTreeMap<Vec<u8>, u64>);
type UndefinedGroup = (SymbolIndex, bool, Vec<(SectionIndex, u64)>);
type SymbolGroup = (Option<FileIndex>, Vec<(SymbolIndex, u32)>);

pub mod rank {
    pub const NOT_ADDRESS_SET: u32 = 1 << 27;
    pub const NOT_ALLOCATED: u32 = 1 << 26;
    pub const LARGE_EXECUTE_WRITE: u32 = 1 << 16;
    pub const LARGE_ALTERNATE: u32 = 1 << 15;
    pub const WRITE: u32 = 1 << 14;
    pub const EXECUTE_WRITE: u32 = 1 << 13;
    pub const EXECUTE: u32 = 1 << 12;
    pub const READ_ONLY_DATA: u32 = 1 << 11;
    pub const LARGE_EXECUTE: u32 = 1 << 10;
    pub const LARGE: u32 = 1 << 9;
    pub const NOT_RELOCATION_READ_ONLY: u32 = 1 << 8;
    pub const NOT_THREAD_LOCAL: u32 = 1 << 7;
    pub const BSS: u32 = 1 << 6;
}

fn is_section_prefix(prefix: &str, name: &str) -> bool {
    match name.strip_prefix(prefix) {
        Some(rest) => rest.is_empty() || rest.starts_with('.'),
        None => false,
    }
}

pub fn output_section_name(name: &str) -> String {
    if name == "COMMON" {
        return ".bss".to_string();
    }
    if is_section_prefix(".text", name) {
        return ".text".to_string();
    }
    if is_section_prefix(".ltext", name) {
        return ".ltext".to_string();
    }
    for prefix in [
        ".data.rel.ro",
        ".data",
        ".rodata",
        ".bss.rel.ro",
        ".bss",
        ".ldata",
        ".lrodata",
        ".lbss",
        ".gcc_except_table",
        ".init_array",
        ".fini_array",
        ".tbss",
        ".tdata",
        ".ARM.exidx",
        ".ARM.extab",
        ".ctors",
        ".dtors",
        ".sbss",
        ".sdata",
        ".srodata",
        ".gnu.build.attributes",
    ] {
        if is_section_prefix(prefix, name) {
            return prefix.to_string();
        }
    }
    name.to_string()
}

fn is_c_identifier(name: &str) -> bool {
    let mut characters = name.chars();
    match characters.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn can_merge_to_program_bits(kind: u32, architecture: Architecture) -> bool {
    kind == section_type::NO_BITS
        || kind == section_type::PROGRAM_BITS
        || kind == section_type::INIT_ARRAY
        || kind == section_type::PREINIT_ARRAY
        || kind == section_type::FINI_ARRAY
        || kind == section_type::NOTE
        || (kind == section_type::X86_64_UNWIND && architecture == Architecture::X86_64)
}

pub fn priority_of(name: &str) -> i64 {
    let Some(position) = name.rfind('.') else {
        return 65536;
    };
    let mut value = 65536i64;
    if let Ok(number) = name[position + 1..].parse::<i64>() {
        value = number;
        if position == 6 && (name.starts_with(".ctors") || name.starts_with(".dtors")) {
            value = 65535 - value;
        }
    }
    value
}

impl<'s> Context<'s> {
    pub fn add_synthetic(
        &mut self,
        name: &str,
        kind: u32,
        flags: u64,
        alignment: u64,
        which: Synthetic,
    ) -> SectionIndex {
        let index = self.sections.len();
        self.sections
            .push(InputSection::synthetic(name, kind, flags, alignment, which));
        self.input_order.push(index);
        self.synthetic.insert(name.to_string(), index);
        index
    }

    pub fn synthetic_section(&self, name: &str) -> Option<SectionIndex> {
        self.synthetic.get(name).copied()
    }

    pub fn create_synthetic_sections(&mut self) {
        let word = self.word_size();
        let encoding = self.encoding();
        let relocatable = self.is_relocatable();
        let rela = self.target.uses_addends;
        if !relocatable && !self.is_shared() && !self.options.no_dynamic_linker {
            if let Some(interpreter) = self.options.dynamic_linker.clone() {
                let index = self.add_synthetic(
                    ".interp",
                    section_type::PROGRAM_BITS,
                    section_flag::ALLOCATE,
                    1,
                    Synthetic::Interpreter,
                );
                let mut data = interpreter.into_bytes();
                data.push(0);
                self.sections[index].size = data.len() as u64;
                self.sections[index].data = data;
            }
        }
        self.add_synthetic(
            ".bss",
            section_type::NO_BITS,
            section_flag::ALLOCATE | section_flag::WRITE,
            1,
            Synthetic::Bss,
        );
        self.add_synthetic(
            ".bss.rel.ro",
            section_type::NO_BITS,
            section_flag::ALLOCATE | section_flag::WRITE,
            1,
            Synthetic::BssRelocationReadOnly,
        );
        if self.options.build_identifier != BuildIdentifier::None {
            let index = self.add_synthetic(
                ".note.gnu.build-id",
                section_type::NOTE,
                section_flag::ALLOCATE,
                4,
                Synthetic::BuildIdentifier,
            );
            let length = match &self.options.build_identifier {
                BuildIdentifier::Fast => 8,
                BuildIdentifier::Md5 | BuildIdentifier::Uuid => 16,
                BuildIdentifier::Sha1 => 20,
                BuildIdentifier::Hexadecimal(bytes) => bytes.len() as u64,
                BuildIdentifier::None => 0,
            };
            self.sections[index].size = 16 + length;
        }
        if !relocatable {
            if self.has_dynamic_symbols {
                let index = self.add_synthetic(
                    ".dynsym",
                    section_type::DYNAMIC_SYMBOLS,
                    section_flag::ALLOCATE,
                    word,
                    Synthetic::DynamicSymbolTable,
                );
                self.sections[index].entry_size = encoding.symbol_size() as u64;
                let index = self.add_synthetic(
                    ".gnu.version",
                    section_type::GNU_VERSION_SYMBOLS,
                    section_flag::ALLOCATE,
                    2,
                    Synthetic::VersionSymbols,
                );
                self.sections[index].entry_size = 2;
                let index = self.add_synthetic(
                    ".gnu.version_r",
                    section_type::GNU_VERSION_NEEDS,
                    section_flag::ALLOCATE,
                    4,
                    Synthetic::VersionNeeds,
                );
                let _ = index;
                if self.options.gnu_hash.unwrap_or(true) {
                    self.add_synthetic(
                        ".gnu.hash",
                        section_type::GNU_HASH,
                        section_flag::ALLOCATE,
                        word,
                        Synthetic::GnuHash,
                    );
                }
                if self.options.system_five_hash.unwrap_or(true) {
                    let index = self.add_synthetic(
                        ".hash",
                        section_type::HASH,
                        section_flag::ALLOCATE,
                        4,
                        Synthetic::SystemFiveHash,
                    );
                    self.sections[index].entry_size = 4;
                }
                let index = self.add_synthetic(
                    ".dynamic",
                    section_type::DYNAMIC,
                    section_flag::ALLOCATE | section_flag::WRITE,
                    word,
                    Synthetic::Dynamic,
                );
                self.sections[index].entry_size = encoding.dynamic_entry_size() as u64;
                self.add_synthetic(
                    ".dynstr",
                    section_type::STRING_TABLE,
                    section_flag::ALLOCATE,
                    1,
                    Synthetic::DynamicStringTable,
                );
            }
            let name = if rela { ".rela.dyn" } else { ".rel.dyn" };
            let index = self.add_synthetic(
                name,
                if rela {
                    section_type::RELOCATIONS_WITH_ADDENDS
                } else {
                    section_type::RELOCATIONS
                },
                section_flag::ALLOCATE,
                word,
                Synthetic::DynamicRelocations,
            );
            self.sections[index].entry_size = encoding.relocation_size(rela) as u64;
            if self.options.pack_relative_relocations {
                let index = self.add_synthetic(
                    ".relr.dyn",
                    section_type::RELATIVE_RELOCATIONS,
                    section_flag::ALLOCATE,
                    word,
                    Synthetic::RelativeRelocations,
                );
                self.sections[index].entry_size = word;
            }
            if self.options.exception_frame_header {
                self.add_synthetic(
                    ".eh_frame_hdr",
                    section_type::PROGRAM_BITS,
                    section_flag::ALLOCATE,
                    4,
                    Synthetic::ExceptionFrameHeader,
                );
            }
            self.add_synthetic(
                ".eh_frame",
                section_type::PROGRAM_BITS,
                section_flag::ALLOCATE,
                1,
                Synthetic::ExceptionFrame,
            );
            if self.target.architecture == Architecture::Arm {
                self.add_synthetic(
                    ".ARM.exidx",
                    section_type::ARM_EXCEPTION_INDEX,
                    section_flag::ALLOCATE | section_flag::LINK_ORDER,
                    4,
                    Synthetic::ArmExceptionIndex,
                );
            }
        }
        let got_entry = self.target.got_entry_size;
        self.add_synthetic(
            ".got",
            section_type::PROGRAM_BITS,
            section_flag::ALLOCATE | section_flag::WRITE,
            got_entry,
            Synthetic::Got,
        );
        self.add_synthetic(
            ".got.plt",
            section_type::PROGRAM_BITS,
            section_flag::ALLOCATE | section_flag::WRITE,
            got_entry,
            Synthetic::GotPlt,
        );
        let indirect_name = if self.target.architecture == Architecture::Arm {
            ".got"
        } else {
            ".got.plt"
        };
        let index = self.sections.len();
        self.sections.push(InputSection::synthetic(
            indirect_name,
            section_type::PROGRAM_BITS,
            section_flag::ALLOCATE | section_flag::WRITE,
            got_entry,
            Synthetic::IndirectGotPlt,
        ));
        self.input_order.push(index);
        self.synthetic.insert("igot".to_string(), index);
        if self.options.relocation_read_only && !relocatable {
            self.add_synthetic(
                ".relro_padding",
                section_type::NO_BITS,
                section_flag::ALLOCATE | section_flag::WRITE,
                1,
                Synthetic::RelocationReadOnlyPadding,
            );
        }
        let name = if rela { ".rela.plt" } else { ".rel.plt" };
        let index = self.add_synthetic(
            name,
            if rela {
                section_type::RELOCATIONS_WITH_ADDENDS
            } else {
                section_type::RELOCATIONS
            },
            section_flag::ALLOCATE,
            word,
            Synthetic::PltRelocations,
        );
        self.sections[index].entry_size = encoding.relocation_size(rela) as u64;
        let mut plt_flags = section_flag::ALLOCATE | section_flag::EXECUTE;
        if self.target.architecture == Architecture::Aarch64 {
            plt_flags |= section_flag::AARCH64_PURE_CODE;
        }
        self.add_synthetic(
            ".plt",
            section_type::PROGRAM_BITS,
            plt_flags,
            16,
            Synthetic::Plt,
        );
        self.add_synthetic(
            ".iplt",
            section_type::PROGRAM_BITS,
            section_flag::ALLOCATE | section_flag::EXECUTE,
            16,
            Synthetic::IndirectPlt,
        );
        if self.and_features != 0 {
            self.add_synthetic(
                ".note.gnu.property",
                section_type::NOTE,
                section_flag::ALLOCATE,
                word,
                Synthetic::GnuProperty,
            );
        }
        if self.options.strip != Strip::All {
            let index = self.add_synthetic(
                ".symtab",
                section_type::SYMBOL_TABLE,
                0,
                word,
                Synthetic::SymbolTable,
            );
            self.sections[index].entry_size = encoding.symbol_size() as u64;
        }
        self.add_synthetic(
            ".shstrtab",
            section_type::STRING_TABLE,
            0,
            1,
            Synthetic::SectionNameTable,
        );
        if self.options.strip != Strip::All {
            self.add_synthetic(
                ".strtab",
                section_type::STRING_TABLE,
                0,
                1,
                Synthetic::StringTable,
            );
        }
    }

    pub fn create_comment_section(&mut self) {
        let mut data = LINKER_IDENTITY.as_bytes().to_vec();
        data.push(0);
        let mut section = InputSection::synthetic(
            ".comment",
            section_type::PROGRAM_BITS,
            section_flag::MERGE | section_flag::STRINGS,
            1,
            Synthetic::Comment,
        );
        section.synthetic = None;
        section.entry_size = 1;
        section.size = data.len() as u64;
        section.data = data;
        section.is_merge = true;
        let index = self.sections.len();
        self.sections.push(section);
        self.input_order.push(index);
    }

    pub fn aggregate_sections(&mut self) {
        let objects = self.object_files.clone();
        for file in objects {
            let sections: Vec<SectionIndex> = self.files[file]
                .sections
                .iter()
                .flatten()
                .copied()
                .collect();
            for section in sections {
                if self.sections[section].is_exception_frame {
                    self.exception_frame_inputs.push(section);
                } else {
                    self.input_order.push(section);
                }
            }
        }
        if self.options.strip != Strip::None {
            let sections = &self.sections;
            self.input_order
                .retain(|section| !is_debug_section(&sections[*section].name));
        }
    }

    pub fn is_needed(&self, section: SectionIndex) -> bool {
        let data = &self.sections[section];
        let Some(which) = data.synthetic else {
            return true;
        };
        match which {
            Synthetic::Bss | Synthetic::BssRelocationReadOnly => data.size != 0,
            Synthetic::Got => {
                self.has_got_offset_relocation || !self.got_entries.is_empty() || self.got_size > 0
            }
            Synthetic::GotPlt => {
                !self.got_plt_entries.is_empty() || self.has_got_plt_offset_relocation
            }
            Synthetic::IndirectGotPlt => !self.indirect_got_plt_entries.is_empty(),
            Synthetic::Plt => !self.plt_entries.is_empty(),
            Synthetic::IndirectPlt => !self.indirect_plt_entries.is_empty(),
            Synthetic::PltRelocations => !self.plt_relocations.is_empty(),
            Synthetic::DynamicRelocations => !self.dynamic_relocations.is_empty(),
            Synthetic::RelativeRelocations => !self.relative_relocations.is_empty(),
            Synthetic::ExceptionFrame => !self.frame_pieces.is_empty(),
            Synthetic::ExceptionFrameHeader => !self.frame_pieces.is_empty(),
            Synthetic::VersionNeeds => self.version_need_count() > 0,
            Synthetic::VersionSymbols => self.version_need_count() > 0,
            Synthetic::DynamicSymbolTable
            | Synthetic::DynamicStringTable
            | Synthetic::GnuHash
            | Synthetic::SystemFiveHash
            | Synthetic::Dynamic => self.has_dynamic_symbols,
            Synthetic::ArmExceptionIndex => false,
            _ => true,
        }
    }

    pub fn version_need_count(&self) -> usize {
        self.version_auxiliary_count
    }

    pub fn version_definition_count(&self) -> usize {
        1
    }

    fn is_shared_file(&self, file: Option<FileIndex>) -> bool {
        file.is_some_and(|file| file < self.files.len() && self.shared_files.contains(&file))
    }

    pub fn dynamic_symbol_name(&self, id: SymbolIndex) -> String {
        let symbol = &self.symbols[id];
        if symbol.version == elf::version_index::LOCAL {
            return symbol.name.clone();
        }
        match symbol.name.find('@') {
            Some(at) => symbol.name[..at].to_string(),
            None => symbol.name.clone(),
        }
    }

    fn add_dynamic_symbol(&mut self, id: SymbolIndex) {
        let name = self.dynamic_symbol_name(id);
        let offset = self.dynamic_strings.len() as u32;
        if !name.is_empty() {
            self.dynamic_strings.extend_from_slice(name.as_bytes());
            self.dynamic_strings.push(0);
        }
        self.dynamic_symbols.push(id);
        self.dynamic_symbol_name_offsets
            .push(if name.is_empty() { 0 } else { offset });
        let file = self.symbols[id].file;
        if self.is_shared_file(file) && !self.symbols[id].is_undefined() {
            let file = file.unwrap_or(0);
            if self.files[file].needed {
                self.add_version_need(id, file);
            }
        }
    }

    fn add_version_need(&mut self, id: SymbolIndex, file: FileIndex) {
        let version = self.symbols[id].version;
        if version == elf::version_index::GLOBAL {
            return;
        }
        let slot = version as usize;
        if self.files[file].version_need_information.is_empty() {
            let count = self.files[file].version_definitions.len();
            self.files[file]
                .version_need_information
                .resize(count, (0, true));
        }
        if slot >= self.files[file].version_need_information.len() {
            return;
        }
        if self.files[file].version_need_information[slot].0 == 0 {
            self.version_auxiliary_count += 1;
            let identifier =
                (self.version_auxiliary_count + self.version_definition_count()) as u16;
            self.files[file].version_need_information[slot].0 = identifier;
        }
        let weak = self.symbols[id].binding == symbol_binding::WEAK;
        let information = &mut self.files[file].version_need_information[slot];
        information.1 &= weak;
        self.symbols[id].version = information.0;
    }

    pub fn add_dynamic_string(&mut self, text: &str) -> u32 {
        if let Some(offset) = self.dynamic_string_offsets.get(text) {
            return *offset;
        }
        let offset = self.dynamic_strings.len() as u32;
        self.dynamic_strings.extend_from_slice(text.as_bytes());
        self.dynamic_strings.push(0);
        self.dynamic_string_offsets.insert(text.to_string(), offset);
        offset
    }

    pub fn finalize_dynamic_tables(&mut self) {
        if !self.has_dynamic_symbols {
            return;
        }
        if self.synthetic_section(".gnu.hash").is_some() {
            let mut undefined = Vec::new();
            let mut defined = Vec::new();
            for (id, offset) in self
                .dynamic_symbols
                .iter()
                .copied()
                .zip(self.dynamic_symbol_name_offsets.iter().copied())
            {
                if self.symbols[id].is_defined() {
                    defined.push((id, offset));
                } else {
                    undefined.push((id, offset));
                }
            }
            let buckets = (defined.len() / 4).max(1);
            self.gnu_hash_bucket_count = buckets;
            self.gnu_hash_symbol_count = defined.len();
            let mut hashed: Vec<(u32, u32, SymbolIndex, u32)> = defined
                .into_iter()
                .map(|(id, offset)| {
                    let hash = elf::gnu_hash(self.dynamic_symbol_name(id).as_bytes());
                    (hash % buckets as u32, offset, id, hash)
                })
                .collect();
            hashed.sort_by_key(|entry| (entry.0, entry.1));
            self.dynamic_symbols = undefined.iter().map(|entry| entry.0).collect();
            self.dynamic_symbol_name_offsets = undefined.iter().map(|entry| entry.1).collect();
            for (_, offset, id, _) in hashed {
                self.dynamic_symbols.push(id);
                self.dynamic_symbol_name_offsets.push(offset);
            }
        }
        for (position, id) in self.dynamic_symbols.clone().into_iter().enumerate() {
            self.symbols[id].dynamic_symbol_index = position + 1;
        }
        self.version_needs.clear();
        for file in self.shared_files.clone() {
            if self.files[file].version_need_information.is_empty() {
                continue;
            }
            let file_name = self.files[file].shared_object_name.clone();
            let file_name = self.add_dynamic_string(&file_name);
            let mut entries = Vec::new();
            for slot in 0..self.files[file].version_need_information.len() {
                let (identifier, weak) = self.files[file].version_need_information[slot];
                if identifier == 0 {
                    continue;
                }
                let (name, hash) = self.files[file].version_definitions[slot]
                    .clone()
                    .unwrap_or_default();
                let name = self.add_dynamic_string(&name);
                entries.push((hash, identifier, weak, name));
            }
            self.version_needs
                .push(VersionNeedOutput { file_name, entries });
        }
        self.dynamic_entry_count = self.dynamic_entries().len();
    }

    fn dynamic_tables_section_address(&self, name: &str) -> u64 {
        self.synthetic_section(name)
            .map(|section| self.section_address(section, 0))
            .unwrap_or(0)
    }

    fn live_synthetic(&self, name: &str) -> bool {
        self.synthetic_section(name).is_some_and(|section| {
            self.sections[section]
                .output
                .is_some_and(|output| self.outputs[output].live)
        })
    }

    pub fn dynamic_entries(&mut self) -> Vec<elf::DynamicEntry> {
        use elf::dynamic_tag as tag;
        let mut entries: Vec<elf::DynamicEntry> = Vec::new();
        let mut add = |tag: i64, value: u64| entries.push(elf::DynamicEntry { tag, value });
        let encoding = self.encoding();
        let rela = self.target.uses_addends;
        if !self.options.run_paths.is_empty() {
            let joined = self.options.run_paths.join(":");
            let offset = self.add_dynamic_string(&joined);
            add(
                if self.options.enable_new_dynamic_tags {
                    tag::RUN_PATH
                } else {
                    tag::RUN_PATH_OLD
                },
                u64::from(offset),
            );
        }
        for file in self.shared_files.clone() {
            if self.files[file].needed {
                let name = self.files[file].shared_object_name.clone();
                let offset = self.add_dynamic_string(&name);
                add(tag::NEEDED, u64::from(offset));
            }
        }
        if let Some(name) = self.options.shared_object_name.clone() {
            if !name.is_empty() {
                let offset = self.add_dynamic_string(&name);
                add(tag::SHARED_OBJECT_NAME, u64::from(offset));
            }
        }
        let mut flags = 0u64;
        let mut flags_1 = 0u64;
        if self.options.symbolic {
            flags |= elf::dynamic_flag::SYMBOLIC;
        }
        if self.options.load_globally {
            flags_1 |= elf::dynamic_flag_1::GLOBAL;
        }
        if self.options.initialize_first {
            flags_1 |= elf::dynamic_flag_1::INITIALIZE_FIRST;
        }
        if self.options.interpose {
            flags_1 |= elf::dynamic_flag_1::INTERPOSE;
        }
        if self.options.no_default_library {
            flags_1 |= elf::dynamic_flag_1::NO_DEFAULT_LIBRARY;
        }
        if self.options.no_delete {
            flags_1 |= elf::dynamic_flag_1::NO_DELETE;
        }
        if self.options.no_dynamic_open {
            flags_1 |= elf::dynamic_flag_1::NO_OPEN;
        }
        if self.options.kind == OutputKind::PositionIndependentExecutable {
            flags_1 |= elf::dynamic_flag_1::PIE;
        }
        if self.options.bind_now {
            flags |= elf::dynamic_flag::BIND_NOW;
            flags_1 |= elf::dynamic_flag_1::NOW;
        }
        if self.options.origin {
            flags |= elf::dynamic_flag::ORIGIN;
            flags_1 |= elf::dynamic_flag_1::ORIGIN;
        }
        if !self.options.text_relocations_forbidden {
            flags |= elf::dynamic_flag::TEXT_RELOCATIONS;
        }
        if self.has_tls_initial_executable && self.is_shared() {
            flags |= elf::dynamic_flag::STATIC_THREAD_LOCAL;
        }
        if flags != 0 {
            add(tag::FLAGS, flags);
        }
        if flags_1 != 0 {
            add(tag::FLAGS_1, flags_1);
        }
        if !self.is_shared() && !self.options.read_only_dynamic {
            add(tag::DEBUG, 0);
        }
        let dynamic_name = if rela { ".rela.dyn" } else { ".rel.dyn" };
        if self.live_synthetic(dynamic_name) && !self.dynamic_relocations.is_empty() {
            add(
                if rela {
                    tag::RELOCATIONS_WITH_ADDENDS
                } else {
                    tag::RELOCATIONS
                },
                self.dynamic_tables_section_address(dynamic_name),
            );
            add(
                if rela {
                    tag::RELOCATIONS_WITH_ADDENDS_SIZE
                } else {
                    tag::RELOCATIONS_SIZE
                },
                self.dynamic_relocations.len() as u64 * encoding.relocation_size(rela) as u64,
            );
            add(
                if rela {
                    tag::RELOCATION_WITH_ADDEND_ENTRY
                } else {
                    tag::RELOCATION_ENTRY
                },
                encoding.relocation_size(rela) as u64,
            );
            let relative = self
                .dynamic_relocations
                .iter()
                .filter(|entry| entry.kind == self.target.relative_relocation)
                .count();
            if self.options.combine_relocations && relative > 0 {
                add(
                    if rela {
                        tag::RELATIVE_WITH_ADDEND_COUNT
                    } else {
                        tag::RELATIVE_COUNT
                    },
                    relative as u64,
                );
            }
        }
        if self.live_synthetic(".relr.dyn") && !self.relative_relocations.is_empty() {
            let size = self
                .synthetic_section(".relr.dyn")
                .and_then(|section| self.sections[section].output)
                .map(|output| self.outputs[output].size)
                .unwrap_or(0);
            add(
                tag::RELATIVE_RELOCATIONS,
                self.dynamic_tables_section_address(".relr.dyn"),
            );
            add(tag::RELATIVE_RELOCATIONS_SIZE, size);
            add(tag::RELATIVE_RELOCATION_ENTRY, self.word_size());
        }
        let plt_name = if rela { ".rela.plt" } else { ".rel.plt" };
        if self.live_synthetic(plt_name) && !self.plt_relocations.is_empty() {
            add(
                tag::JUMP_RELOCATIONS,
                self.dynamic_tables_section_address(plt_name),
            );
            add(
                tag::PLT_RELOCATIONS_SIZE,
                self.plt_relocations.len() as u64 * encoding.relocation_size(rela) as u64,
            );
            add(
                tag::PLT_GOT,
                self.dynamic_tables_section_address(".got.plt"),
            );
            add(
                tag::PLT_RELOCATION_KIND,
                if rela {
                    tag::RELOCATIONS_WITH_ADDENDS as u64
                } else {
                    tag::RELOCATIONS as u64
                },
            );
        }
        add(
            tag::SYMBOL_TABLE,
            self.dynamic_tables_section_address(".dynsym"),
        );
        add(tag::SYMBOL_ENTRY, encoding.symbol_size() as u64);
        add(
            tag::STRING_TABLE,
            self.dynamic_tables_section_address(".dynstr"),
        );
        add(tag::STRING_TABLE_SIZE, self.dynamic_strings.len() as u64);
        if !self.options.text_relocations_forbidden {
            add(tag::TEXT_RELOCATIONS, 0);
        }
        if self.live_synthetic(".gnu.hash") {
            add(
                tag::GNU_HASH,
                self.dynamic_tables_section_address(".gnu.hash"),
            );
        }
        if self.live_synthetic(".hash") {
            add(tag::HASH, self.dynamic_tables_section_address(".hash"));
        }
        for (name, start, size) in [
            (
                ".preinit_array",
                tag::PREINIT_ARRAY,
                tag::PREINIT_ARRAY_SIZE,
            ),
            (".init_array", tag::INIT_ARRAY, tag::INIT_ARRAY_SIZE),
            (".fini_array", tag::FINI_ARRAY, tag::FINI_ARRAY_SIZE),
        ] {
            if let Some(output) = self.find_output_any(name) {
                add(start, self.outputs[output].address);
                add(size, self.outputs[output].size);
            }
        }
        let initializer = self
            .options
            .initializer
            .clone()
            .unwrap_or_else(|| "_init".to_string());
        if let Some(id) = self.find(&initializer) {
            if self.symbols[id].is_defined() {
                add(tag::INIT, self.symbol_address(id, 0));
            }
        }
        let finalizer = self
            .options
            .finalizer
            .clone()
            .unwrap_or_else(|| "_fini".to_string());
        if let Some(id) = self.find(&finalizer) {
            if self.symbols[id].is_defined() {
                add(tag::FINI, self.symbol_address(id, 0));
            }
        }
        if self.live_synthetic(".gnu.version") && self.version_need_count() > 0 {
            add(
                tag::VERSION_SYMBOLS,
                self.dynamic_tables_section_address(".gnu.version"),
            );
        }
        if self.live_synthetic(".gnu.version_r") && self.version_need_count() > 0 {
            add(
                tag::VERSION_NEEDS,
                self.dynamic_tables_section_address(".gnu.version_r"),
            );
            let count = self
                .shared_files
                .iter()
                .filter(|file| !self.files[**file].version_need_information.is_empty())
                .count();
            add(tag::VERSION_NEED_COUNT, count as u64);
        }
        add(tag::NULL, 0);
        entries
    }

    fn encode_dynamic_symbol_table(&self) -> Vec<u8> {
        let encoding = self.encoding();
        let mut out = Vec::new();
        elf::encode_symbol(encoding, &elf::Symbol::default(), &mut out);
        for (id, name_offset) in self
            .dynamic_symbols
            .iter()
            .zip(&self.dynamic_symbol_name_offsets)
        {
            let symbol = &self.symbols[*id];
            let section_number = self.output_index_of_symbol(*id);
            let value = if symbol.is_defined() {
                self.symbol_address(*id, 0)
            } else {
                0
            };
            let size = if section_number != section_index::UNDEFINED {
                symbol.size
            } else {
                0
            };
            elf::encode_symbol(
                encoding,
                &elf::Symbol {
                    name: String::new(),
                    name_offset: *name_offset,
                    value,
                    size,
                    binding: symbol.binding,
                    kind: symbol.kind,
                    other: symbol.other,
                    section: section_number,
                },
                &mut out,
            );
        }
        out
    }

    fn encode_gnu_hash(&self) -> Vec<u8> {
        let start = self.dynamic_symbols.len() - self.gnu_hash_symbol_count;
        let hashes: Vec<u32> = self.dynamic_symbols[start..]
            .iter()
            .map(|id| elf::gnu_hash(self.dynamic_symbol_name(*id).as_bytes()))
            .collect();
        elf::gnu_hash_table(
            self.encoding(),
            start + 1,
            &hashes,
            self.gnu_hash_bucket_count,
        )
    }

    fn encode_system_five_hash(&self) -> Vec<u8> {
        let mut names: Vec<Vec<u8>> = vec![Vec::new()];
        for id in &self.dynamic_symbols {
            names.push(self.dynamic_symbol_name(*id).into_bytes());
        }
        let references: Vec<&[u8]> = names.iter().map(|name| name.as_slice()).collect();
        elf::system_five_hash_table(self.encoding(), &references, references.len())
    }

    fn encode_version_symbols(&self) -> Vec<u8> {
        let encoding = self.encoding();
        let mut out = Vec::new();
        encoding.put_u16(&mut out, 0);
        for id in &self.dynamic_symbols {
            let symbol = &self.symbols[*id];
            encoding.put_u16(
                &mut out,
                if symbol.is_undefined() {
                    0
                } else {
                    symbol.version
                },
            );
        }
        out
    }

    fn encode_version_needs(&self) -> Vec<u8> {
        let encoding = self.encoding();
        let mut out = Vec::new();
        let needs = self.version_needs.len();
        let mut auxiliary_at = needs * 16;
        for (position, need) in self.version_needs.iter().enumerate() {
            encoding.put_u16(&mut out, 1);
            encoding.put_u16(&mut out, need.entries.len() as u16);
            encoding.put_u32(&mut out, need.file_name);
            encoding.put_u32(&mut out, (auxiliary_at - position * 16) as u32);
            encoding.put_u32(&mut out, if position + 1 == needs { 0 } else { 16 });
            auxiliary_at += need.entries.len() * 16;
        }
        for need in &self.version_needs {
            for (position, (hash, identifier, weak, name)) in need.entries.iter().enumerate() {
                encoding.put_u32(&mut out, *hash);
                encoding.put_u16(&mut out, if *weak { elf::version_flag::WEAK } else { 0 });
                encoding.put_u16(&mut out, *identifier);
                encoding.put_u32(&mut out, *name);
                encoding.put_u32(
                    &mut out,
                    if position + 1 == need.entries.len() {
                        0
                    } else {
                        16
                    },
                );
            }
        }
        out
    }

    pub fn create_output_sections(&mut self) {
        let mut by_name: BTreeMap<String, Vec<OutputIndex>> = BTreeMap::new();
        let order = self.input_order.clone();
        for section in order {
            if !self.sections[section].live || self.sections[section].output.is_some() {
                continue;
            }
            let name = output_section_name(&self.sections[section].name);
            let kind = self.sections[section].kind;
            let flags = self.sections[section].flags;
            let is_group = kind == section_type::GROUP || flags & section_flag::GROUP != 0;
            let output = if is_group && self.is_relocatable() {
                None
            } else {
                by_name
                    .get(&name)
                    .and_then(|candidates| candidates.first().copied())
            };
            let output = match output {
                Some(found) => found,
                None => {
                    let index = self.outputs.len();
                    self.outputs.push(OutputSection::new(&name));
                    by_name.entry(name).or_default().push(index);
                    self.order.push(index);
                    index
                }
            };
            self.sections[section].output = Some(output);
            self.outputs[output].members.push(section);
        }
    }

    pub fn finalize_input_sections(&mut self) {
        for output in 0..self.outputs.len() {
            let members = std::mem::take(&mut self.outputs[output].members);
            let mut kept: Vec<SectionIndex> = Vec::new();
            let mut merges: Vec<SectionIndex> = Vec::new();
            for member in members {
                if !self.sections[member].is_merge {
                    kept.push(member);
                    continue;
                }
                if !self.sections[member].live {
                    continue;
                }
                let flags = self.sections[member].flags;
                let entry_size = self.sections[member].entry_size;
                let alignment = self.sections[member].alignment;
                let found = merges.iter().copied().find(|candidate| {
                    let held = &self.sections[*candidate];
                    held.flags == flags
                        && held.entry_size == entry_size
                        && (held.alignment == alignment || flags & section_flag::STRINGS == 0)
                });
                let target = match found {
                    Some(target) => target,
                    None => {
                        let name = self.sections[member].name.clone();
                        let kind = self.sections[member].kind;
                        let index = self.sections.len();
                        let mut merged = InputSection::synthetic(
                            &name,
                            kind,
                            flags,
                            alignment,
                            Synthetic::Merge,
                        );
                        merged.entry_size = entry_size;
                        merged.output = Some(output);
                        self.sections.push(merged);
                        merges.push(index);
                        kept.push(index);
                        index
                    }
                };
                self.sections[member].merged_into = Some(target);
                self.sections[target].alignment = self.sections[target].alignment.max(alignment);
                self.sections[target].merged_members.push(member);
            }
            for member in &kept {
                self.commit_section(output, *member);
            }
            self.outputs[output].members = kept;
            for merge in merges {
                self.finalize_merge(merge);
            }
        }
    }

    fn commit_section(&mut self, output: OutputIndex, section: SectionIndex) {
        let kind = self.sections[section].kind;
        let flags = self.sections[section].flags;
        let alignment = self.sections[section].alignment;
        let entry_size = self.sections[section].entry_size;
        let architecture = self.target.architecture;
        let target = &mut self.outputs[output];
        if target.kind != kind {
            if !target.has_input_sections {
                target.kind = kind;
            } else if !can_merge_to_program_bits(target.kind, architecture)
                || !can_merge_to_program_bits(kind, architecture)
            {
                if target.kind != section_type::NO_BITS {
                    let name = self.sections[section].name.clone();
                    let place = self.section_place(section);
                    let output_name = self.outputs[output].name.clone();
                    self.error(
                        problem(
                            "EI220",
                            "Sections of different types are placed in one output section.",
                        )
                        .with_context(format!("Section: {name}"))
                        .with_context(format!("File: {place}"))
                        .with_context(format!("Output: {output_name}")),
                    );
                }
                self.outputs[output].kind = section_type::PROGRAM_BITS;
            } else {
                target.kind = section_type::PROGRAM_BITS;
            }
        }
        let target = &mut self.outputs[output];
        if !target.has_input_sections {
            target.has_input_sections = true;
            target.entry_size = entry_size;
            target.flags = flags;
        } else if (target.flags ^ flags) & section_flag::THREAD_LOCAL != 0 {
            let name = target.name.clone();
            self.error(
                problem(
                    "EI221",
                    "Thread-local and ordinary sections are placed in one output section.",
                )
                .with_context(format!("Output: {name}")),
            );
        }
        let target = &mut self.outputs[output];
        let and_mask = match architecture {
            Architecture::Arm => section_flag::ARM_PURE_CODE,
            Architecture::Aarch64 => section_flag::AARCH64_PURE_CODE,
            _ => 0,
        };
        let and_flags = (target.flags & flags) & and_mask;
        let or_flags = (target.flags | flags) & !and_mask;
        target.flags = and_flags | or_flags;
        target.alignment = target.alignment.max(alignment);
        if target.entry_size != entry_size {
            target.entry_size = 0;
        }
        self.sections[section].output = Some(output);
    }

    pub fn section_place(&self, section: SectionIndex) -> String {
        let data = &self.sections[section];
        format!("{}:({})", self.file_name(data.file), data.name)
    }

    fn finalize_merge(&mut self, merge: SectionIndex) {
        let alignment = self.sections[merge].alignment.max(1);
        let members = self.sections[merge].merged_members.clone();
        let shard_count = 32usize;
        let mut shards: Vec<MergeShard> = (0..shard_count)
            .map(|_| (Vec::new(), BTreeMap::new()))
            .collect();
        let mut placements: Vec<(SectionIndex, usize, usize, u64)> = Vec::new();
        for member in &members {
            for position in 0..self.sections[*member].pieces.len() {
                let piece = self.sections[*member].pieces[position].clone();
                if !piece.live {
                    continue;
                }
                let start = piece.input_offset as usize;
                let bytes =
                    self.sections[*member].data[start..start + piece.length as usize].to_vec();
                let shard = (piece.hash >> 26) as usize & (shard_count - 1);
                let (content, known) = &mut shards[shard];
                let offset = match known.get(&bytes) {
                    Some(offset) => *offset,
                    None => {
                        let offset = align_to(content.len() as u64, alignment);
                        content.resize(offset as usize, 0);
                        content.extend_from_slice(&bytes);
                        known.insert(bytes, offset);
                        offset
                    }
                };
                placements.push((*member, position, shard, offset));
            }
        }
        let mut shard_offsets = vec![0u64; shard_count];
        let mut at = 0u64;
        let mut data = Vec::new();
        for (shard, (content, _)) in shards.iter().enumerate() {
            if !content.is_empty() {
                at = align_to(at, alignment);
            }
            shard_offsets[shard] = at;
            data.resize(at as usize, 0);
            data.extend_from_slice(content);
            at += content.len() as u64;
        }
        for (member, position, shard, offset) in placements {
            self.sections[member].pieces[position].output_offset = offset + shard_offsets[shard];
        }
        self.sections[merge].size = at;
        self.sections[merge].data = data;
    }

    pub fn rank_of(&self, output: OutputIndex) -> u32 {
        let section = &self.outputs[output];
        let mut value = 0u32;
        if self.options.section_start.contains_key(&section.name) {
            return value;
        }
        value |= rank::NOT_ADDRESS_SET;
        if section.flags & section_flag::ALLOCATE == 0 {
            return value | rank::NOT_ALLOCATED;
        }
        let executable = section.flags & section_flag::EXECUTE != 0;
        let writable = section.flags & section_flag::WRITE != 0;
        let large = section.flags & section_flag::X86_64_LARGE != 0
            && self.target.architecture == Architecture::X86_64;
        if !writable && !executable {
            if !large {
                value |= rank::LARGE;
            }
            if section.name == ".interp" {
                value |= 1;
            } else if section.kind == section_type::NOTE {
                value |= 2;
            } else if section.kind != section_type::PROGRAM_BITS {
                value |= 3;
            } else {
                value |= rank::READ_ONLY_DATA;
            }
        } else if executable {
            if large {
                value |= if writable {
                    rank::LARGE_EXECUTE_WRITE
                } else {
                    rank::LARGE_EXECUTE
                };
            } else {
                value |= if writable {
                    rank::EXECUTE_WRITE
                } else {
                    rank::EXECUTE
                };
            }
        } else {
            value |= rank::WRITE;
            if section.flags & section_flag::THREAD_LOCAL == 0 {
                value |= rank::NOT_THREAD_LOCAL;
            }
            if !self.is_relocation_read_only(output) {
                value |= rank::NOT_RELOCATION_READ_ONLY;
            }
            if large {
                value |= rank::LARGE;
            }
        }
        if section.kind == section_type::NO_BITS {
            value |= rank::BSS;
        }
        value
    }

    pub fn is_relocation_read_only(&self, output: OutputIndex) -> bool {
        if !self.options.relocation_read_only {
            return false;
        }
        let section = &self.outputs[output];
        if section.relocation_read_only {
            return true;
        }
        let flags = section.flags;
        if flags & section_flag::ALLOCATE == 0 || flags & section_flag::WRITE == 0 {
            return false;
        }
        if flags & section_flag::THREAD_LOCAL != 0 {
            return true;
        }
        if matches!(
            section.kind,
            section_type::INIT_ARRAY | section_type::FINI_ARRAY | section_type::PREINIT_ARRAY
        ) {
            return true;
        }
        let holds = |which: &str| {
            self.synthetic_section(which)
                .is_some_and(|index| self.sections[index].output == Some(output))
        };
        if holds(".got") {
            return true;
        }
        if section.name == ".toc" {
            return true;
        }
        if holds(".got.plt") {
            return self.options.bind_now;
        }
        if holds(".relro_padding") {
            return true;
        }
        if section.name == ".dynamic" {
            return true;
        }
        let name = section.name.as_str();
        name == ".data.rel.ro"
            || name == ".bss.rel.ro"
            || name == ".ctors"
            || name == ".dtors"
            || name == ".jcr"
            || name == ".eh_frame"
            || name == ".fini_array"
            || name == ".init_array"
            || name == ".preinit_array"
    }
}

#[derive(Clone, Debug)]
pub struct UndefinedReference {
    pub symbol: SymbolIndex,
    pub section: SectionIndex,
    pub offset: u64,
    pub warning: bool,
}

impl<'s> Context<'s> {
    pub fn add_unused_undefined(&mut self, name: &str) -> SymbolIndex {
        let id = self.insert(name);
        if self.symbols[id].state == SymbolState::Placeholder {
            let entry = &mut self.symbols[id];
            entry.state = SymbolState::Undefined;
            entry.file = Some(INTERNAL);
            entry.binding = symbol_binding::GLOBAL;
        }
        id
    }

    pub fn handle_undefined(&mut self, id: SymbolIndex) {
        self.symbols[id].used_in_regular_object = true;
        if let SymbolState::Lazy { source } = self.symbols[id].state.clone() {
            self.extract(source);
        }
    }

    fn add_optional_regular(
        &mut self,
        name: &str,
        place: SymbolPlace,
        value: u64,
        other: u8,
    ) -> Option<SymbolIndex> {
        let id = self.find(name)?;
        let state = self.symbols[id].state.clone();
        if state == SymbolState::Defined || matches!(state, SymbolState::Common { .. }) {
            return None;
        }
        self.define_synthesized(
            id,
            place,
            value,
            other,
            symbol_type::NO_TYPE,
            symbol_binding::GLOBAL,
        );
        self.symbols[id].used_in_regular_object = true;
        self.synthesized.push(id);
        Some(id)
    }

    pub fn define_synthesized(
        &mut self,
        id: SymbolIndex,
        place: SymbolPlace,
        value: u64,
        other: u8,
        kind: u8,
        binding: u8,
    ) {
        self.merge_visibility(id, other);
        let entry = &mut self.symbols[id];
        entry.state = SymbolState::Defined;
        entry.file = Some(INTERNAL);
        entry.binding = binding;
        entry.kind = kind;
        entry.value = value;
        entry.size = 0;
        entry.section = None;
        entry.header_relative = false;
        entry.output_section_symbol = None;
        entry.output_relative_end = false;
        match place {
            SymbolPlace::Absolute => {}
            SymbolPlace::Header => entry.header_relative = true,
            SymbolPlace::HeaderEnd => {
                entry.header_relative = true;
                entry.output_relative_end = true;
            }
            SymbolPlace::Section(section) => entry.section = Some(section),
            SymbolPlace::OutputStart(output) => entry.output_section_symbol = Some(output),
            SymbolPlace::OutputEnd(output) => {
                entry.output_section_symbol = Some(output);
                entry.output_relative_end = true;
            }
        }
        self.reserved.insert(self.symbols[id].name.clone(), id);
    }

    pub fn add_reserved_symbols(&mut self) {
        if let Some(id) = self.find("_GLOBAL_OFFSET_TABLE_") {
            if self.symbols[id].is_defined() {
                let place = self.definition_place(id);
                self.error(
                    problem("EI222", "An input redefines a symbol the linker defines.")
                        .with_context("Symbol: _GLOBAL_OFFSET_TABLE_".to_string())
                        .with_context(format!("File: {place}")),
                );
                return;
            }
            self.define_synthesized(
                id,
                SymbolPlace::Header,
                0,
                visibility::HIDDEN,
                symbol_type::NO_TYPE,
                symbol_binding::GLOBAL,
            );
            self.reserved
                .insert("_GLOBAL_OFFSET_TABLE_".to_string(), id);
        }
        for name in ["__ehdr_start", "__executable_start", "__dso_handle"] {
            self.add_optional_regular(name, SymbolPlace::Header, 0, visibility::HIDDEN);
        }
        for name in [
            "__bss_start",
            "end",
            "_end",
            "etext",
            "_etext",
            "edata",
            "_edata",
        ] {
            let place = if name == "__bss_start" {
                SymbolPlace::Header
            } else {
                SymbolPlace::HeaderEnd
            };
            self.add_optional_regular(name, place, 0, visibility::DEFAULT);
        }
    }

    pub fn compute_binding(&self, id: SymbolIndex) -> u8 {
        let symbol = &self.symbols[id];
        let shown = symbol.visibility();
        if (shown != visibility::DEFAULT && shown != visibility::PROTECTED)
            || symbol.version == elf::version_index::LOCAL
        {
            return symbol_binding::LOCAL;
        }
        if symbol.binding == symbol_binding::GNU_UNIQUE {
            return symbol_binding::GLOBAL;
        }
        symbol.binding
    }

    pub fn dynamic_undefined_weak(&self) -> bool {
        self.has_dynamic_symbols && (!self.shared_files.is_empty() || self.is_shared())
    }

    pub fn compute_is_preemptible(&self, id: SymbolIndex) -> bool {
        let symbol = &self.symbols[id];
        if symbol.visibility() != visibility::DEFAULT {
            return false;
        }
        if !symbol.is_defined() {
            return !symbol.is_undefined() || self.dynamic_undefined_weak();
        }
        if !self.is_shared() {
            return false;
        }
        if self.options.symbolic || (self.options.symbolic_functions && symbol.is_function()) {
            return false;
        }
        true
    }

    pub fn global_symbols(&self) -> Vec<SymbolIndex> {
        self.names
            .values()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn parse_versions_and_compute_preemptible(&mut self) {
        for id in self.global_symbols() {
            if self.compute_binding(id) == symbol_binding::LOCAL {
                self.symbols[id].exported = false;
                continue;
            }
            let symbol = &self.symbols[id];
            if !symbol.is_defined() && !symbol.is_common() {
                self.symbols[id].preemptible = self.compute_is_preemptible(id);
            } else if self.options.export_dynamic && symbol.used_in_regular_object {
                self.symbols[id].exported = true;
                self.symbols[id].preemptible = self.compute_is_preemptible(id);
            }
        }
    }

    pub fn demote_symbols_and_compute_preemptible(&mut self) {
        for id in self.global_symbols() {
            let state = self.symbols[id].state.clone();
            if state == SymbolState::Defined {
                if let Some(section) = self.symbols[id].section {
                    if !self.sections[section].live {
                        let entry = &mut self.symbols[id];
                        if entry.binding == symbol_binding::WEAK {
                            entry.binding = symbol_binding::GLOBAL;
                        }
                        entry.state = SymbolState::Undefined;
                        entry.discarded_section = self.sections[section].elf_index as u32;
                        entry.section = None;
                        entry.used_in_regular_object = false;
                    }
                }
            } else {
                let unneeded_shared = state == SymbolState::Shared
                    && self.symbols[id]
                        .file
                        .is_some_and(|file| file < self.files.len() && !self.files[file].needed);
                if self.symbols[id].is_lazy() || unneeded_shared {
                    let entry = &mut self.symbols[id];
                    let binding = if matches!(state, SymbolState::Lazy { .. }) {
                        entry.binding
                    } else {
                        symbol_binding::WEAK
                    };
                    entry.state = SymbolState::Undefined;
                    entry.file = Some(INTERNAL);
                    entry.binding = binding;
                    entry.version = elf::version_index::GLOBAL;
                }
            }
            let entry = &self.symbols[id];
            let preemptible =
                (entry.is_undefined() || entry.exported) && self.compute_is_preemptible(id);
            self.symbols[id].preemptible = preemptible;
        }
    }

    pub fn object_message(&self, section: SectionIndex, offset: u64) -> String {
        let data = &self.sections[section];
        let Some(file) = data.file.filter(|file| *file < self.files.len()) else {
            return format!("<internal>:({}+0x{:x})", data.name, offset);
        };
        let file_data = &self.files[file];
        let archive = match &file_data.archive_name {
            Some(archive) => format!(" in archive {archive}"),
            None => String::new(),
        };
        for id in &file_data.symbols {
            if *id == usize::MAX {
                continue;
            }
            let symbol = &self.symbols[*id];
            if symbol.is_defined()
                && symbol.section == Some(section)
                && symbol.value <= offset
                && offset < symbol.value + symbol.size
                && !symbol.is_section()
            {
                return format!("{}:({}){archive}", file_data.name, symbol.name);
            }
        }
        format!("{}:({}+0x{:x}){archive}", file_data.name, data.name, offset)
    }

    pub fn error_location(&self, section: SectionIndex, offset: u64) -> String {
        let data = &self.sections[section];
        let file = self.file_name(data.file);
        if let Some(function) = self.enclosing_function(section, offset) {
            return format!("{file}:(function {function}: {}+0x{:x})", data.name, offset);
        }
        format!("{file}:({}+0x{:x})", data.name, offset)
    }

    fn enclosing_function(&self, section: SectionIndex, offset: u64) -> Option<String> {
        let file = self.sections[section]
            .file
            .filter(|file| *file < self.files.len())?;
        for id in &self.files[file].symbols {
            if *id == usize::MAX {
                continue;
            }
            let symbol = &self.symbols[*id];
            if symbol.is_defined()
                && symbol.is_function()
                && symbol.section == Some(section)
                && symbol.value <= offset
                && offset < symbol.value + symbol.size
            {
                return Some(symbol.name.clone());
            }
        }
        None
    }

    pub fn unresolved_policy(&self) -> UnresolvedPolicy {
        if let Some(policy) = self.options.unresolved_symbols {
            return policy;
        }
        if self.is_shared() && !self.options.no_undefined {
            UnresolvedPolicy::Ignore
        } else {
            UnresolvedPolicy::ReportError
        }
    }

    pub fn maybe_report_undefined(
        &mut self,
        id: SymbolIndex,
        section: SectionIndex,
        offset: u64,
        undefined: &mut Vec<UndefinedReference>,
    ) -> bool {
        let symbol = &self.symbols[id];
        if symbol.is_weak() {
            return false;
        }
        let can_be_external = !symbol.is_local && symbol.visibility() == visibility::DEFAULT;
        let policy = self.unresolved_policy();
        if policy == UnresolvedPolicy::Ignore && can_be_external {
            return false;
        }
        let warning = (policy == UnresolvedPolicy::Warn && can_be_external)
            || self.options.no_inhibit_executable;
        undefined.push(UndefinedReference {
            symbol: id,
            section,
            offset,
            warning,
        });
        !warning
    }

    pub fn report_undefined(&mut self, undefined: Vec<UndefinedReference>) {
        let mut grouped: Vec<UndefinedGroup> = Vec::new();
        for reference in undefined {
            match grouped.iter_mut().find(|entry| entry.0 == reference.symbol) {
                Some(entry) => entry.2.push((reference.section, reference.offset)),
                None => grouped.push((
                    reference.symbol,
                    reference.warning,
                    vec![(reference.section, reference.offset)],
                )),
            }
        }
        for (id, warning, places) in grouped {
            let symbol = &self.symbols[id];
            let name = symbol.name.clone();
            let shown = match symbol.visibility() {
                visibility::INTERNAL => "internal",
                visibility::HIDDEN => "hidden",
                visibility::PROTECTED => "protected",
                _ => "",
            };
            let mut diagnostic = if symbol.discarded_section != 0
                && symbol.file.is_some_and(|file| file < self.files.len())
            {
                let file = symbol.file.unwrap_or(0);
                let section_name = self.files[file]
                    .elf
                    .sections
                    .get(symbol.discarded_section as usize)
                    .map(|section| section.name.clone())
                    .unwrap_or_default();
                let group = self.comdat_groups.iter().find_map(|(signature, holder)| {
                    let _ = holder;
                    let elf = &self.files[file].elf;
                    for (position, section) in elf.sections.iter().enumerate() {
                        if section.kind == section_type::GROUP {
                            if let Ok(found) = elf.group(position) {
                                if found.members.contains(&symbol.discarded_section) {
                                    let symbols = &self.files[file].elf_symbols;
                                    if symbols.get(found.signature as usize).map(|s| &s.name)
                                        == Some(signature)
                                    {
                                        return Some((signature.clone(), *holder));
                                    }
                                }
                            }
                        }
                    }
                    None
                });
                let mut diagnostic = problem(
                    "EI223",
                    "A symbol is defined in a section the linker discarded.",
                )
                .with_context(format!("Symbol: {name}"))
                .with_context(format!("Section: {section_name}"))
                .with_context(format!("File: {}", self.file_name(Some(file))));
                if let Some((signature, holder)) = group {
                    diagnostic = diagnostic
                        .with_context(format!("Group: {signature}"))
                        .with_context(format!("Prevailing: {}", self.file_name(Some(holder))));
                }
                diagnostic
            } else {
                let mut diagnostic = problem("EI207", "A symbol is referenced but never defined.")
                    .with_context(format!("Symbol: {name}"));
                if !shown.is_empty() {
                    diagnostic = diagnostic.with_context(format!("Visibility: {shown}"));
                }
                diagnostic
            };
            for (section, offset) in places.iter().take(3) {
                diagnostic = diagnostic.with_context(format!(
                    "Reference: {}",
                    self.object_message(*section, *offset)
                ));
            }
            if places.len() > 3 {
                diagnostic =
                    diagnostic.with_context(format!("More references: {}", places.len() - 3));
            }
            if warning {
                self.warn(diagnostic);
            } else {
                self.error(diagnostic);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolPlace {
    Absolute,
    Header,
    HeaderEnd,
    Section(SectionIndex),
    OutputStart(OutputIndex),
    OutputEnd(OutputIndex),
}

fn sign_extend(value: u64, bits: u32) -> i64 {
    let shift = 64 - bits;
    ((value << shift) as i64) >> shift
}

pub fn relocation_name(architecture: Architecture, kind: u32) -> String {
    let table: &[(u32, &str)] = match architecture {
        Architecture::X86_64 => X86_64_RELOCATION_NAMES,
        Architecture::X86 => X86_RELOCATION_NAMES,
        Architecture::Aarch64 => AARCH64_RELOCATION_NAMES,
        Architecture::Arm => ARM_RELOCATION_NAMES,
    };
    table
        .iter()
        .find(|(number, _)| *number == kind)
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| format!("Unknown ({kind})"))
}

pub const X86_64_RELOCATION_NAMES: &[(u32, &str)] = &[
    (0, "R_X86_64_NONE"),
    (1, "R_X86_64_64"),
    (2, "R_X86_64_PC32"),
    (3, "R_X86_64_GOT32"),
    (4, "R_X86_64_PLT32"),
    (5, "R_X86_64_COPY"),
    (6, "R_X86_64_GLOB_DAT"),
    (7, "R_X86_64_JUMP_SLOT"),
    (8, "R_X86_64_RELATIVE"),
    (9, "R_X86_64_GOTPCREL"),
    (10, "R_X86_64_32"),
    (11, "R_X86_64_32S"),
    (12, "R_X86_64_16"),
    (13, "R_X86_64_PC16"),
    (14, "R_X86_64_8"),
    (15, "R_X86_64_PC8"),
    (16, "R_X86_64_DTPMOD64"),
    (17, "R_X86_64_DTPOFF64"),
    (18, "R_X86_64_TPOFF64"),
    (19, "R_X86_64_TLSGD"),
    (20, "R_X86_64_TLSLD"),
    (21, "R_X86_64_DTPOFF32"),
    (22, "R_X86_64_GOTTPOFF"),
    (23, "R_X86_64_TPOFF32"),
    (24, "R_X86_64_PC64"),
    (25, "R_X86_64_GOTOFF64"),
    (26, "R_X86_64_GOTPC32"),
    (27, "R_X86_64_GOT64"),
    (28, "R_X86_64_GOTPCREL64"),
    (29, "R_X86_64_GOTPC64"),
    (30, "R_X86_64_GOTPLT64"),
    (31, "R_X86_64_PLTOFF64"),
    (32, "R_X86_64_SIZE32"),
    (33, "R_X86_64_SIZE64"),
    (34, "R_X86_64_GOTPC32_TLSDESC"),
    (35, "R_X86_64_TLSDESC_CALL"),
    (36, "R_X86_64_TLSDESC"),
    (37, "R_X86_64_IRELATIVE"),
    (38, "R_X86_64_RELATIVE64"),
    (41, "R_X86_64_GOTPCRELX"),
    (42, "R_X86_64_REX_GOTPCRELX"),
    (43, "R_X86_64_CODE_4_GOTPCRELX"),
    (44, "R_X86_64_CODE_4_GOTTPOFF"),
    (45, "R_X86_64_CODE_4_GOTPC32_TLSDESC"),
    (46, "R_X86_64_CODE_5_GOTPCRELX"),
    (47, "R_X86_64_CODE_5_GOTTPOFF"),
    (48, "R_X86_64_CODE_5_GOTPC32_TLSDESC"),
    (49, "R_X86_64_CODE_6_GOTPCRELX"),
    (50, "R_X86_64_CODE_6_GOTTPOFF"),
    (51, "R_X86_64_CODE_6_GOTPC32_TLSDESC"),
];

pub const X86_RELOCATION_NAMES: &[(u32, &str)] = &[
    (0, "R_386_NONE"),
    (1, "R_386_32"),
    (2, "R_386_PC32"),
    (3, "R_386_GOT32"),
    (4, "R_386_PLT32"),
    (5, "R_386_COPY"),
    (6, "R_386_GLOB_DAT"),
    (7, "R_386_JUMP_SLOT"),
    (8, "R_386_RELATIVE"),
    (9, "R_386_GOTOFF"),
    (10, "R_386_GOTPC"),
    (11, "R_386_32PLT"),
    (14, "R_386_TLS_TPOFF"),
    (15, "R_386_TLS_IE"),
    (16, "R_386_TLS_GOTIE"),
    (17, "R_386_TLS_LE"),
    (18, "R_386_TLS_GD"),
    (19, "R_386_TLS_LDM"),
    (20, "R_386_16"),
    (21, "R_386_PC16"),
    (22, "R_386_8"),
    (23, "R_386_PC8"),
    (32, "R_386_TLS_LDO_32"),
    (35, "R_386_TLS_DTPMOD32"),
    (36, "R_386_TLS_DTPOFF32"),
    (37, "R_386_TLS_TPOFF32"),
    (38, "R_386_SIZE32"),
    (39, "R_386_TLS_GOTDESC"),
    (40, "R_386_TLS_DESC_CALL"),
    (41, "R_386_TLS_DESC"),
    (42, "R_386_IRELATIVE"),
    (43, "R_386_GOT32X"),
];

pub const AARCH64_RELOCATION_NAMES: &[(u32, &str)] = &[
    (0, "R_AARCH64_NONE"),
    (257, "R_AARCH64_ABS64"),
    (258, "R_AARCH64_ABS32"),
    (259, "R_AARCH64_ABS16"),
    (260, "R_AARCH64_PREL64"),
    (261, "R_AARCH64_PREL32"),
    (262, "R_AARCH64_PREL16"),
    (263, "R_AARCH64_MOVW_UABS_G0"),
    (264, "R_AARCH64_MOVW_UABS_G0_NC"),
    (265, "R_AARCH64_MOVW_UABS_G1"),
    (266, "R_AARCH64_MOVW_UABS_G1_NC"),
    (267, "R_AARCH64_MOVW_UABS_G2"),
    (268, "R_AARCH64_MOVW_UABS_G2_NC"),
    (269, "R_AARCH64_MOVW_UABS_G3"),
    (270, "R_AARCH64_MOVW_SABS_G0"),
    (271, "R_AARCH64_MOVW_SABS_G1"),
    (272, "R_AARCH64_MOVW_SABS_G2"),
    (273, "R_AARCH64_LD_PREL_LO19"),
    (274, "R_AARCH64_ADR_PREL_LO21"),
    (275, "R_AARCH64_ADR_PREL_PG_HI21"),
    (276, "R_AARCH64_ADR_PREL_PG_HI21_NC"),
    (277, "R_AARCH64_ADD_ABS_LO12_NC"),
    (278, "R_AARCH64_LDST8_ABS_LO12_NC"),
    (279, "R_AARCH64_TSTBR14"),
    (280, "R_AARCH64_CONDBR19"),
    (282, "R_AARCH64_JUMP26"),
    (283, "R_AARCH64_CALL26"),
    (284, "R_AARCH64_LDST16_ABS_LO12_NC"),
    (285, "R_AARCH64_LDST32_ABS_LO12_NC"),
    (286, "R_AARCH64_LDST64_ABS_LO12_NC"),
    (287, "R_AARCH64_MOVW_PREL_G0"),
    (288, "R_AARCH64_MOVW_PREL_G0_NC"),
    (289, "R_AARCH64_MOVW_PREL_G1"),
    (290, "R_AARCH64_MOVW_PREL_G1_NC"),
    (291, "R_AARCH64_MOVW_PREL_G2"),
    (292, "R_AARCH64_MOVW_PREL_G2_NC"),
    (293, "R_AARCH64_MOVW_PREL_G3"),
    (299, "R_AARCH64_LDST128_ABS_LO12_NC"),
    (300, "R_AARCH64_MOVW_GOTOFF_G0"),
    (307, "R_AARCH64_GOTREL64"),
    (308, "R_AARCH64_GOTREL32"),
    (309, "R_AARCH64_GOT_LD_PREL19"),
    (310, "R_AARCH64_LD64_GOTOFF_LO15"),
    (311, "R_AARCH64_ADR_GOT_PAGE"),
    (312, "R_AARCH64_LD64_GOT_LO12_NC"),
    (313, "R_AARCH64_LD64_GOTPAGE_LO15"),
    (314, "R_AARCH64_PLT32"),
    (315, "R_AARCH64_GOTPCREL32"),
    (512, "R_AARCH64_TLSGD_ADR_PREL21"),
    (513, "R_AARCH64_TLSGD_ADR_PAGE21"),
    (514, "R_AARCH64_TLSGD_ADD_LO12_NC"),
    (529, "R_AARCH64_TLSLD_ADD_DTPREL_HI12"),
    (530, "R_AARCH64_TLSLD_ADD_DTPREL_LO12"),
    (531, "R_AARCH64_TLSLD_ADD_DTPREL_LO12_NC"),
    (539, "R_AARCH64_TLSIE_MOVW_GOTTPREL_G1"),
    (540, "R_AARCH64_TLSIE_MOVW_GOTTPREL_G0_NC"),
    (541, "R_AARCH64_TLSIE_ADR_GOTTPREL_PAGE21"),
    (542, "R_AARCH64_TLSIE_LD64_GOTTPREL_LO12_NC"),
    (543, "R_AARCH64_TLSIE_LD_GOTTPREL_PREL19"),
    (544, "R_AARCH64_TLSLE_MOVW_TPREL_G2"),
    (545, "R_AARCH64_TLSLE_MOVW_TPREL_G1"),
    (546, "R_AARCH64_TLSLE_MOVW_TPREL_G1_NC"),
    (547, "R_AARCH64_TLSLE_MOVW_TPREL_G0"),
    (548, "R_AARCH64_TLSLE_MOVW_TPREL_G0_NC"),
    (549, "R_AARCH64_TLSLE_ADD_TPREL_HI12"),
    (550, "R_AARCH64_TLSLE_ADD_TPREL_LO12"),
    (551, "R_AARCH64_TLSLE_ADD_TPREL_LO12_NC"),
    (552, "R_AARCH64_TLSLE_LDST8_TPREL_LO12"),
    (553, "R_AARCH64_TLSLE_LDST8_TPREL_LO12_NC"),
    (554, "R_AARCH64_TLSLE_LDST16_TPREL_LO12"),
    (555, "R_AARCH64_TLSLE_LDST16_TPREL_LO12_NC"),
    (556, "R_AARCH64_TLSLE_LDST32_TPREL_LO12"),
    (557, "R_AARCH64_TLSLE_LDST32_TPREL_LO12_NC"),
    (558, "R_AARCH64_TLSLE_LDST64_TPREL_LO12"),
    (559, "R_AARCH64_TLSLE_LDST64_TPREL_LO12_NC"),
    (560, "R_AARCH64_TLSDESC_LD_PREL19"),
    (561, "R_AARCH64_TLSDESC_ADR_PREL21"),
    (562, "R_AARCH64_TLSDESC_ADR_PAGE21"),
    (563, "R_AARCH64_TLSDESC_LD64_LO12"),
    (564, "R_AARCH64_TLSDESC_ADD_LO12"),
    (565, "R_AARCH64_TLSDESC_OFF_G1"),
    (566, "R_AARCH64_TLSDESC_OFF_G0_NC"),
    (567, "R_AARCH64_TLSDESC_LDR"),
    (568, "R_AARCH64_TLSDESC_ADD"),
    (569, "R_AARCH64_TLSDESC_CALL"),
    (570, "R_AARCH64_TLSLE_LDST128_TPREL_LO12"),
    (571, "R_AARCH64_TLSLE_LDST128_TPREL_LO12_NC"),
    (1024, "R_AARCH64_COPY"),
    (1025, "R_AARCH64_GLOB_DAT"),
    (1026, "R_AARCH64_JUMP_SLOT"),
    (1027, "R_AARCH64_RELATIVE"),
    (1028, "R_AARCH64_TLS_DTPMOD64"),
    (1029, "R_AARCH64_TLS_DTPREL64"),
    (1030, "R_AARCH64_TLS_TPREL64"),
    (1031, "R_AARCH64_TLSDESC"),
    (1032, "R_AARCH64_IRELATIVE"),
];

pub const ARM_RELOCATION_NAMES: &[(u32, &str)] = &[
    (0, "R_ARM_NONE"),
    (1, "R_ARM_PC24"),
    (2, "R_ARM_ABS32"),
    (3, "R_ARM_REL32"),
    (4, "R_ARM_LDR_PC_G0"),
    (5, "R_ARM_ABS16"),
    (6, "R_ARM_ABS12"),
    (7, "R_ARM_THM_ABS5"),
    (8, "R_ARM_ABS8"),
    (9, "R_ARM_SBREL32"),
    (10, "R_ARM_THM_CALL"),
    (11, "R_ARM_THM_PC8"),
    (13, "R_ARM_TLS_DESC"),
    (17, "R_ARM_TLS_DTPMOD32"),
    (18, "R_ARM_TLS_DTPOFF32"),
    (19, "R_ARM_TLS_TPOFF32"),
    (20, "R_ARM_COPY"),
    (21, "R_ARM_GLOB_DAT"),
    (22, "R_ARM_JUMP_SLOT"),
    (23, "R_ARM_RELATIVE"),
    (24, "R_ARM_GOTOFF32"),
    (25, "R_ARM_BASE_PREL"),
    (26, "R_ARM_GOT_BREL"),
    (27, "R_ARM_PLT32"),
    (28, "R_ARM_CALL"),
    (29, "R_ARM_JUMP24"),
    (30, "R_ARM_THM_JUMP24"),
    (31, "R_ARM_BASE_ABS"),
    (38, "R_ARM_TARGET1"),
    (40, "R_ARM_V4BX"),
    (41, "R_ARM_TARGET2"),
    (42, "R_ARM_PREL31"),
    (43, "R_ARM_MOVW_ABS_NC"),
    (44, "R_ARM_MOVT_ABS"),
    (45, "R_ARM_MOVW_PREL_NC"),
    (46, "R_ARM_MOVT_PREL"),
    (47, "R_ARM_THM_MOVW_ABS_NC"),
    (48, "R_ARM_THM_MOVT_ABS"),
    (49, "R_ARM_THM_MOVW_PREL_NC"),
    (50, "R_ARM_THM_MOVT_PREL"),
    (51, "R_ARM_THM_JUMP19"),
    (52, "R_ARM_THM_JUMP6"),
    (53, "R_ARM_THM_ALU_PREL_11_0"),
    (54, "R_ARM_THM_PC12"),
    (55, "R_ARM_ABS32_NOI"),
    (56, "R_ARM_REL32_NOI"),
    (57, "R_ARM_ALU_PC_G0_NC"),
    (58, "R_ARM_ALU_PC_G0"),
    (59, "R_ARM_ALU_PC_G1_NC"),
    (60, "R_ARM_ALU_PC_G1"),
    (61, "R_ARM_ALU_PC_G2"),
    (62, "R_ARM_LDR_PC_G1"),
    (63, "R_ARM_LDR_PC_G2"),
    (64, "R_ARM_LDRS_PC_G0"),
    (65, "R_ARM_LDRS_PC_G1"),
    (66, "R_ARM_LDRS_PC_G2"),
    (67, "R_ARM_LDC_PC_G0"),
    (68, "R_ARM_LDC_PC_G1"),
    (69, "R_ARM_LDC_PC_G2"),
    (96, "R_ARM_GOT_PREL"),
    (102, "R_ARM_THM_JUMP11"),
    (103, "R_ARM_THM_JUMP8"),
    (104, "R_ARM_TLS_GD32"),
    (105, "R_ARM_TLS_LDM32"),
    (106, "R_ARM_TLS_LDO32"),
    (107, "R_ARM_TLS_IE32"),
    (108, "R_ARM_TLS_LE32"),
    (109, "R_ARM_TLS_LDO12"),
    (110, "R_ARM_TLS_LE12"),
    (111, "R_ARM_TLS_IE12GP"),
    (129, "R_ARM_THM_TLS_CALL"),
    (130, "R_ARM_TLS_DESCSEQ"),
    (131, "R_ARM_THM_TLS_DESCSEQ16"),
    (132, "R_ARM_THM_TLS_DESCSEQ32"),
    (160, "R_ARM_IRELATIVE"),
];

#[derive(Clone, Copy, Debug)]
pub struct RawRelocation {
    pub offset: u64,
    pub kind: u32,
    pub symbol: u32,
    pub addend: i64,
}

impl<'s> Context<'s> {
    pub fn raw_relocations(&mut self, section: SectionIndex) -> Vec<RawRelocation> {
        let Some(file) = self.sections[section]
            .file
            .filter(|file| *file < self.files.len())
        else {
            return Vec::new();
        };
        let Some(relocation_index) = self.sections[section].relocation_section else {
            return Vec::new();
        };
        let entries = match self.files[file].elf.relocations(relocation_index) {
            Ok(entries) => entries,
            Err(diagnostic) => {
                self.error(diagnostic);
                return Vec::new();
            }
        };
        let with_addend = self.files[file].elf.sections[relocation_index].kind
            == section_type::RELOCATIONS_WITH_ADDENDS;
        let mut raw = Vec::with_capacity(entries.len());
        for entry in entries {
            let addend = if with_addend {
                entry.addend
            } else {
                self.implicit_addend(&self.sections[section].data, entry.offset, entry.kind)
            };
            raw.push(RawRelocation {
                offset: entry.offset,
                kind: entry.kind,
                symbol: entry.symbol,
                addend,
            });
        }
        raw
    }

    pub fn implicit_addend(&self, data: &[u8], offset: u64, kind: u32) -> i64 {
        let at = offset as usize;
        let encoding = self.encoding();
        let read32 = || encoding.read_u32(data, at).unwrap_or(0) as u64;
        let read16 = || encoding.read_u16(data, at).unwrap_or(0) as u64;
        let read64 = || encoding.read_u64(data, at).unwrap_or(0);
        match self.target.architecture {
            Architecture::X86_64 => match kind {
                14 | 15 => sign_extend(u64::from(data.get(at).copied().unwrap_or(0)), 8),
                12 | 13 => sign_extend(read16(), 16),
                1 | 16 | 17 | 18 | 24 | 25 | 27 | 29 | 31 | 33 | 6 | 37 | 8 => read64() as i64,
                36 => encoding.read_u64(data, at + 8).unwrap_or(0) as i64,
                7 | 0 => 0,
                _ => sign_extend(read32(), 32),
            },
            Architecture::X86 => x86_implicit_addend(kind, data, at, encoding),
            Architecture::Arm => arm_implicit_addend(
                kind,
                data,
                at,
                encoding,
                self.arm_features.j1_j2_branch_encoding,
            ),
            Architecture::Aarch64 => match kind {
                257 | 260 | 1027 | 1032 | 1030 | 1025 | 1026 => read64() as i64,
                258 | 261 => sign_extend(read32(), 32),
                259 | 262 => sign_extend(read16(), 16),
                _ => 0,
            },
        }
    }

    pub fn symbol_of(&self, section: SectionIndex, index: u32) -> Option<SymbolIndex> {
        let file = self.sections[section].file?;
        self.files
            .get(file)?
            .symbols
            .get(index as usize)
            .copied()
            .filter(|id| *id != usize::MAX)
    }

    pub fn is_absolute(&self, id: SymbolIndex) -> bool {
        let symbol = &self.symbols[id];
        if symbol.is_undefined() {
            return true;
        }
        symbol.is_defined()
            && symbol.section.is_none()
            && !symbol.header_relative
            && symbol.output_section_symbol.is_none()
    }

    pub fn add_relocation(
        &mut self,
        section: SectionIndex,
        expression: Expression,
        kind: u32,
        offset: u64,
        addend: i64,
        symbol: SymbolIndex,
    ) {
        self.sections[section].relocations.push(Relocation {
            expression,
            kind,
            offset,
            addend,
            symbol,
        });
    }

    pub fn symbolic_dynamic_relocation(&self, kind: u32) -> u32 {
        match self.target.architecture {
            Architecture::X86_64 => {
                if kind == self.target.symbolic_relocation || kind == 32 || kind == 33 {
                    kind
                } else {
                    0
                }
            }
            Architecture::X86 => {
                if kind == 1 || kind == 14 || kind == 35 || kind == 36 || kind == 37 {
                    kind
                } else {
                    0
                }
            }
            Architecture::Aarch64 => {
                if kind == 257 {
                    kind
                } else {
                    0
                }
            }
            Architecture::Arm => {
                if kind == 2 || kind == 38 {
                    2
                } else {
                    0
                }
            }
        }
    }

    pub fn uses_only_low_page_bits(&self, kind: u32) -> bool {
        match self.target.architecture {
            Architecture::Aarch64 => matches!(
                kind,
                277 | 278 | 284 | 285 | 286 | 299 | 312 | 514 | 542 | 563 | 564
            ),
            _ => false,
        }
    }

    pub fn dummy_symbol(&mut self) -> SymbolIndex {
        if let Some(dummy) = self.dummy {
            return dummy;
        }
        let mut data = SymbolData::new("");
        data.is_local = true;
        data.binding = symbol_binding::LOCAL;
        data.state = SymbolState::Undefined;
        data.file = Some(INTERNAL);
        let id = self.symbols.len();
        self.symbols.push(data);
        self.dummy = Some(id);
        id
    }

    pub fn writes_addends(&self) -> bool {
        !self.target.uses_addends || self.options.apply_dynamic_relocations
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_dynamic_relocation(
        &mut self,
        against_symbol: bool,
        dynamic_kind: u32,
        section: SectionIndex,
        offset: u64,
        symbol: SymbolIndex,
        addend: i64,
        expression: Expression,
        addend_kind: u32,
    ) {
        if self.writes_addends() && (expression != Expression::Addend || addend != 0) {
            self.add_relocation(section, expression, addend_kind, offset, addend, symbol);
        }
        self.dynamic_relocations.push(DynamicRelocation {
            kind: dynamic_kind,
            section,
            offset,
            symbol: Some(symbol),
            use_symbol_value: against_symbol,
            addend,
            expression,
        });
    }

    pub fn add_relative_relocation(
        &mut self,
        section: SectionIndex,
        offset: u64,
        symbol: SymbolIndex,
        addend: i64,
        expression: Expression,
        kind: u32,
    ) {
        if self.options.pack_relative_relocations
            && self.sections[section].alignment >= 2
            && offset.is_multiple_of(2)
        {
            self.relative_relocations.push(DynamicRelocation {
                kind: self.target.relative_relocation,
                section,
                offset,
                symbol: Some(symbol),
                use_symbol_value: false,
                addend,
                expression,
            });
            self.add_relocation(section, expression, kind, offset, addend, symbol);
            return;
        }
        let relative = self.target.relative_relocation;
        self.add_dynamic_relocation(
            false, relative, section, offset, symbol, addend, expression, kind,
        );
    }

    pub fn add_symbol_relocation(
        &mut self,
        dynamic_kind: u32,
        section: SectionIndex,
        offset: u64,
        symbol: SymbolIndex,
        addend: i64,
        kind: u32,
    ) {
        self.add_dynamic_relocation(
            true,
            dynamic_kind,
            section,
            offset,
            symbol,
            addend,
            Expression::Addend,
            kind,
        );
    }

    pub fn is_static_link_time_constant(
        &mut self,
        expression: Expression,
        kind: u32,
        symbol: SymbolIndex,
        section: SectionIndex,
        offset: u64,
    ) -> bool {
        use Expression::*;
        if matches!(
            expression,
            GotPlt
                | GotOffset
                | RelaxHint
                | Aarch64GotPageProgramCounter
                | GotProgramCounter
                | GotOnlyProgramCounter
                | GotPltOnlyProgramCounter
                | PltProgramCounter
                | PltGotRelative
                | PltGotPlt
                | GotPltGotRelative
                | GotPltProgramCounter
                | Aarch64GotPage
        ) {
            return true;
        }
        if expression == Got || expression == Plt {
            return self.uses_only_low_page_bits(kind) || !self.is_pic();
        }
        if kind == self.indirect_symbolic_relocation() && kind != 0 {
            return false;
        }
        let data = &self.symbols[symbol];
        if data.preemptible {
            return data.is_undefined() && !self.is_pic();
        }
        if !self.is_pic() {
            return true;
        }
        if expression == Size {
            return true;
        }
        let absolute = self.is_absolute(symbol) || data.is_thread_local();
        let relative = matches!(
            expression,
            ProgramCounter
                | GotRelative
                | GotPltRelative
                | ArmProgramCounterAligned
                | Aarch64PageProgramCounter
                | RelaxGotProgramCounter
        );
        if absolute && !relative {
            return true;
        }
        if !absolute && relative {
            return true;
        }
        if !absolute && !relative {
            return self.uses_only_low_page_bits(kind);
        }
        if data.is_undefined() || data.script_defined {
            return true;
        }
        let name = self.symbols[symbol].name.clone();
        let relocation = relocation_name(self.target.architecture, kind);
        let place = self.object_message(section, offset);
        self.error(
            problem(
                "EI224",
                "A relocation cannot refer to an absolute symbol here.",
            )
            .with_context(format!("Relocation: {relocation}"))
            .with_context(format!("Symbol: {name}"))
            .with_context(format!("Reference: {place}")),
        );
        true
    }

    pub fn indirect_symbolic_relocation(&self) -> u32 {
        0
    }

    pub fn process(
        &mut self,
        section: SectionIndex,
        mut expression: Expression,
        kind: u32,
        offset: u64,
        symbol: SymbolIndex,
        addend: i64,
    ) {
        let indirect = self.symbols[symbol].is_indirect_function();
        if !self.symbols[symbol].preemptible && !indirect {
            if expression != Expression::GotProgramCounter {
                expression = from_plt(expression);
            } else if !(self.is_absolute(symbol) || self.symbols[symbol].is_thread_local()) {
                expression = self.adjust_got_program_counter(section, kind, addend, offset);
                if matches!(
                    expression,
                    Expression::RelaxGotProgramCounter | Expression::RelaxGotProgramCounterNoPic
                ) {
                    self.got_has_deferred_entries = true;
                }
            }
        }
        if needs_got(expression) {
            self.symbols[symbol].needs |= needs::GOT;
        } else if needs_plt(expression) {
            self.symbols[symbol].needs |= needs::PLT;
        } else if indirect {
            self.symbols[symbol].needs |= needs::DIRECT_RELOCATION;
        }
        self.process_auxiliary(section, expression, kind, offset, symbol, addend);
    }

    pub fn process_auxiliary(
        &mut self,
        section: SectionIndex,
        expression: Expression,
        kind: u32,
        offset: u64,
        symbol: SymbolIndex,
        addend: i64,
    ) {
        if self.is_static_link_time_constant(expression, kind, symbol, section, offset) {
            self.add_relocation(section, expression, kind, offset, addend, symbol);
            return;
        }
        let flags = self.sections[section].flags;
        let can_write =
            flags & section_flag::WRITE != 0 || !self.options.text_relocations_forbidden;
        let preemptible = self.symbols[symbol].preemptible;
        if can_write {
            let dynamic_kind = self.symbolic_dynamic_relocation(kind);
            if expression == Expression::Got
                || (dynamic_kind == self.target.symbolic_relocation && !preemptible)
            {
                self.add_relative_relocation(section, offset, symbol, addend, expression, kind);
                return;
            }
            if dynamic_kind != 0 {
                self.add_symbol_relocation(dynamic_kind, section, offset, symbol, addend, kind);
                return;
            }
        }
        if !self.is_shared() && self.symbols[symbol].is_shared() {
            if self.symbols[symbol].shared_protected {
                let name = self.symbols[symbol].name.clone();
                self.error(
                    problem(
                        "EI225",
                        "A protected symbol of a shared library cannot be preempted.",
                    )
                    .with_context(format!("Symbol: {name}"))
                    .with_context(format!(
                        "Reference: {}",
                        self.object_message(section, offset)
                    )),
                );
                return;
            }
            if self.symbols[symbol].is_object() {
                if !self.options.copy_relocations {
                    let name = self.symbols[symbol].name.clone();
                    self.error(
                        problem("EI226", "A relocation against a shared library object needs a copy relocation, and copy relocations are turned off.")
                            .with_context(format!("Relocation: {}", relocation_name(self.target.architecture, kind)))
                            .with_context(format!("Symbol: {name}")),
                    );
                }
                self.symbols[symbol].needs |= needs::COPY;
                self.add_relocation(section, expression, kind, offset, addend, symbol);
                return;
            }
            if self.symbols[symbol].is_function() {
                if self.options.kind == OutputKind::PositionIndependentExecutable
                    && self.target.architecture == Architecture::X86
                {
                    let name = self.symbols[symbol].name.clone();
                    self.error(
                        problem("EI227", "A function of a shared library cannot be preempted here; it needs position independent code.")
                            .with_context(format!("Symbol: {name}"))
                            .with_context(format!("Reference: {}", self.object_message(section, offset))),
                    );
                }
                self.symbols[symbol].needs |= needs::COPY | needs::PLT;
                self.add_relocation(section, expression, kind, offset, addend, symbol);
                return;
            }
        }
        let name = self.symbols[symbol].name.clone();
        let mut diagnostic = problem("EI228", "A relocation cannot be used against this symbol; the code needs to be position independent.")
            .with_context(format!("Relocation: {}", relocation_name(self.target.architecture, kind)));
        if name.is_empty() {
            diagnostic = diagnostic.with_context("Symbol: local symbol".to_string());
        } else {
            diagnostic = diagnostic.with_context(format!("Symbol: {name}"));
        }
        let defined_in = self.file_name(self.symbols[symbol].file);
        diagnostic = diagnostic
            .with_context(format!("Defined: {defined_in}"))
            .with_context(format!(
                "Reference: {}",
                self.object_message(section, offset)
            ));
        self.error(diagnostic);
    }

    pub fn process_program_counter(
        &mut self,
        section: SectionIndex,
        kind: u32,
        offset: u64,
        addend: i64,
        symbol: SymbolIndex,
    ) {
        if self.symbols[symbol].is_indirect_function() {
            self.symbols[symbol].needs |= needs::DIRECT_RELOCATION;
        }
        if self.symbols[symbol].preemptible || (self.is_absolute(symbol) && self.is_pic()) {
            self.process_auxiliary(
                section,
                Expression::ProgramCounter,
                kind,
                offset,
                symbol,
                addend,
            );
        } else {
            self.add_relocation(
                section,
                Expression::ProgramCounter,
                kind,
                offset,
                addend,
                symbol,
            );
        }
    }

    pub fn process_plt_program_counter(
        &mut self,
        section: SectionIndex,
        kind: u32,
        offset: u64,
        addend: i64,
        symbol: SymbolIndex,
    ) {
        if self.symbols[symbol].is_indirect_function() {
            self.process(
                section,
                Expression::PltProgramCounter,
                kind,
                offset,
                symbol,
                addend,
            );
            return;
        }
        if self.symbols[symbol].preemptible {
            self.symbols[symbol].needs |= needs::PLT;
            self.add_relocation(
                section,
                Expression::PltProgramCounter,
                kind,
                offset,
                addend,
                symbol,
            );
        } else if !(self.is_absolute(symbol) && self.is_pic()) {
            self.add_relocation(
                section,
                Expression::ProgramCounter,
                kind,
                offset,
                addend,
                symbol,
            );
        } else {
            self.process_auxiliary(
                section,
                Expression::ProgramCounter,
                kind,
                offset,
                symbol,
                addend,
            );
        }
    }

    pub fn check_tls_local_executable(
        &mut self,
        section: SectionIndex,
        offset: u64,
        symbol: SymbolIndex,
        kind: u32,
    ) -> bool {
        if !self.is_shared() {
            return false;
        }
        let name = self.symbols[symbol].name.clone();
        self.error(
            problem(
                "EI229",
                "A thread-local relocation for executables cannot be used in a shared library.",
            )
            .with_context(format!(
                "Relocation: {}",
                relocation_name(self.target.architecture, kind)
            ))
            .with_context(format!("Symbol: {name}"))
            .with_context(format!(
                "Reference: {}",
                self.object_message(section, offset)
            )),
        );
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub fn handle_tls_initial_executable(
        &mut self,
        section: SectionIndex,
        expression: Expression,
        kind: u32,
        offset: u64,
        addend: i64,
        symbol: SymbolIndex,
        relax: bool,
    ) {
        if relax && !self.is_shared() && !self.symbols[symbol].preemptible {
            self.add_relocation(
                section,
                Expression::ThreadPointerRelative,
                kind,
                offset,
                addend,
                symbol,
            );
        } else {
            self.symbols[symbol].needs |= needs::TLS_INITIAL_EXECUTABLE;
            if expression == Expression::Got && self.is_pic() && !self.uses_only_low_page_bits(kind)
            {
                let relative = self.target.relative_relocation;
                self.add_dynamic_relocation(
                    false, relative, section, offset, symbol, addend, expression, kind,
                );
            } else {
                self.add_relocation(section, expression, kind, offset, addend, symbol);
            }
        }
    }

    pub fn handle_tls_local_dynamic(
        &mut self,
        section: SectionIndex,
        shared_expression: Expression,
        kind: u32,
        offset: u64,
        addend: i64,
        symbol: SymbolIndex,
    ) -> bool {
        if self.is_shared() {
            self.needs_tls_local_dynamic = true;
            self.add_relocation(section, shared_expression, kind, offset, addend, symbol);
            return false;
        }
        self.add_relocation(
            section,
            Expression::ThreadPointerRelative,
            kind,
            offset,
            addend,
            symbol,
        );
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub fn handle_tls_general_dynamic(
        &mut self,
        section: SectionIndex,
        shared_expression: Expression,
        initial_expression: Expression,
        local_expression: Expression,
        kind: u32,
        offset: u64,
        addend: i64,
        symbol: SymbolIndex,
    ) -> bool {
        if !self.is_shared() && initial_expression != Expression::None {
            if self.symbols[symbol].preemptible {
                self.symbols[symbol].needs |= needs::TLS_INITIAL_EXECUTABLE;
                self.add_relocation(section, initial_expression, kind, offset, addend, symbol);
            } else {
                self.add_relocation(section, local_expression, kind, offset, addend, symbol);
            }
            return true;
        }
        self.symbols[symbol].needs |= needs::TLS_GENERAL_DYNAMIC;
        self.add_relocation(section, shared_expression, kind, offset, addend, symbol);
        false
    }

    #[allow(clippy::too_many_arguments)]
    pub fn handle_tls_descriptor(
        &mut self,
        section: SectionIndex,
        shared_expression: Expression,
        initial_expression: Expression,
        kind: u32,
        offset: u64,
        addend: i64,
        symbol: SymbolIndex,
    ) {
        if self.is_shared() {
            self.symbols[symbol].needs |= needs::TLS_DESCRIPTOR;
            self.add_relocation(section, shared_expression, kind, offset, addend, symbol);
        } else if self.symbols[symbol].preemptible {
            self.symbols[symbol].needs |= needs::TLS_INITIAL_EXECUTABLE;
            self.add_relocation(section, initial_expression, kind, offset, addend, symbol);
        } else {
            self.add_relocation(
                section,
                Expression::ThreadPointerRelative,
                kind,
                offset,
                addend,
                symbol,
            );
        }
    }

    fn adjust_got_program_counter(
        &self,
        section: SectionIndex,
        kind: u32,
        addend: i64,
        offset: u64,
    ) -> Expression {
        match self.target.architecture {
            Architecture::X86_64 => {
                if !self.options.relax || addend != -4 || !(kind == 41 || kind == 42 || kind == 43)
                {
                    return Expression::GotProgramCounter;
                }
                let data = &self.sections[section].data;
                let at = offset as usize;
                if at < 2 {
                    return Expression::GotProgramCounter;
                }
                let operation = data[at - 2];
                let modifier = data[at - 1];
                if operation == 0x8b {
                    return Expression::RelaxGotProgramCounter;
                }
                if operation == 0xff && (modifier == 0x15 || modifier == 0x25) {
                    return Expression::RelaxGotProgramCounter;
                }
                if kind == 41 {
                    return Expression::GotProgramCounter;
                }
                if self.is_pic() {
                    Expression::GotProgramCounter
                } else {
                    Expression::RelaxGotProgramCounterNoPic
                }
            }
            Architecture::X86 => Expression::GotProgramCounter,
            _ => Expression::GotProgramCounter,
        }
    }

    pub fn scan_relocations(&mut self) -> Vec<UndefinedReference> {
        let mut undefined = Vec::new();
        let objects = self.object_files.clone();
        for file in objects {
            let sections: Vec<SectionIndex> = self.files[file]
                .sections
                .iter()
                .flatten()
                .copied()
                .collect();
            for section in sections {
                let data = &self.sections[section];
                if !data.live || data.flags & section_flag::ALLOCATE == 0 || data.is_exception_frame
                {
                    continue;
                }
                if data.kind == section_type::ARM_EXCEPTION_INDEX
                    && self.target.architecture == Architecture::Arm
                {
                    continue;
                }
                self.scan_section(section, &mut undefined);
            }
        }
        let frames: Vec<SectionIndex> = self.frame_pieces.keys().copied().collect();
        for section in frames {
            let relocations: Vec<RawRelocation> = self
                .frame_relocations
                .get(&section)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|relocation| {
                    relocation.kind != 0
                        && self
                            .frame_output_offset(section, relocation.offset)
                            .is_some()
                })
                .collect();
            match self.target.architecture {
                Architecture::X86_64 => self.scan_x86_64(section, &relocations, &mut undefined),
                Architecture::X86 => self.scan_x86(section, &relocations, &mut undefined),
                Architecture::Aarch64 => self.scan_aarch64(section, &relocations, &mut undefined),
                Architecture::Arm => self.scan_arm(section, &relocations, &mut undefined),
            }
        }
        undefined
    }

    fn frame_piece_index(&self, section: SectionIndex, offset: u64) -> Option<usize> {
        let pieces = self.frame_pieces.get(&section)?;
        let position = pieces.partition_point(|piece| piece.input_offset <= offset);
        if position == 0 {
            return None;
        }
        let piece = &pieces[position - 1];
        if piece.input_offset + piece.size <= offset {
            return None;
        }
        Some(position - 1)
    }

    pub fn frame_output_offset(&self, section: SectionIndex, offset: u64) -> Option<u64> {
        let index = self.frame_piece_index(section, offset)?;
        let piece = &self.frame_pieces[&section][index];
        piece
            .output_offset
            .map(|output| output + (offset - piece.input_offset))
    }

    fn frame_parent_offset(&self, section: SectionIndex, offset: u64) -> u64 {
        match self.frame_piece_index(section, offset) {
            Some(index) => {
                let piece = &self.frame_pieces[&section][index];
                match piece.output_offset {
                    Some(output) => output + (offset - piece.input_offset),
                    None => offset - piece.input_offset,
                }
            }
            None => offset,
        }
    }

    pub fn mark_live(&mut self) {
        for id in self.global_symbols() {
            let symbol = &self.symbols[id];
            if symbol.is_shared()
                && symbol.used_in_regular_object
                && symbol.binding != symbol_binding::WEAK
            {
                if let Some(file) = symbol.file.filter(|file| *file < self.files.len()) {
                    self.files[file].needed = true;
                }
            }
        }
    }

    pub fn combine_exception_frames(&mut self) {
        let Some(synthetic) = self.synthetic_section(".eh_frame") else {
            return;
        };
        let encoding = self.encoding();
        let mut common_keys: BTreeMap<(Vec<u8>, Option<SymbolIndex>), usize> = BTreeMap::new();
        for section in self.exception_frame_inputs.clone() {
            if !self.sections[section].live {
                continue;
            }
            let mut relocations = self.raw_relocations(section);
            relocations.sort_by_key(|relocation| relocation.offset);
            let data = self.sections[section].data.clone();
            let mut pieces: Vec<FramePiece> = Vec::new();
            let mut at = 0usize;
            let mut relocation_index = 0usize;
            let mut problem_reason: Option<&str> = None;
            while at < data.len() {
                if data.len() - at < 4 {
                    problem_reason = Some("CIE/FDE too small");
                    break;
                }
                let length = u64::from(encoding.read_u32(&data, at).unwrap_or(0));
                if length == 0 {
                    break;
                }
                let size = length + 4;
                if size > (data.len() - at) as u64 {
                    problem_reason = Some(if size == u64::from(u32::MAX) + 4 {
                        "CIE/FDE too large"
                    } else {
                        "CIE/FDE ends past the end of the section"
                    });
                    break;
                }
                let identifier = if data.len() - at >= 8 {
                    encoding.read_u32(&data, at + 4).unwrap_or(1)
                } else {
                    1
                };
                while relocation_index < relocations.len()
                    && relocations[relocation_index].offset < at as u64
                {
                    relocation_index += 1;
                }
                let first_relocation = (relocation_index < relocations.len()
                    && relocations[relocation_index].offset < at as u64 + size)
                    .then_some(relocation_index);
                pieces.push(FramePiece {
                    input_offset: at as u64,
                    size,
                    is_common_information: identifier == 0,
                    first_relocation,
                    output_offset: None,
                    record: None,
                });
                at += size as usize;
            }
            if let Some(reason) = problem_reason {
                let place = self.object_message(section, at as u64);
                self.error(
                    problem("EI240", "The exception frame section is malformed.")
                        .with_context(format!("Reason: {reason}"))
                        .with_context(format!("Location: {place}")),
                );
            }
            let mut offset_to_record: BTreeMap<u64, usize> = BTreeMap::new();
            for (index, piece) in pieces.iter().enumerate() {
                if !piece.is_common_information {
                    continue;
                }
                let personality = piece
                    .first_relocation
                    .and_then(|first| self.symbol_of(section, relocations[first].symbol));
                let start = piece.input_offset as usize;
                let key = (
                    data[start..start + piece.size as usize].to_vec(),
                    personality,
                );
                let record = match common_keys.get(&key) {
                    Some(record) => *record,
                    None => {
                        self.frame_records.push(FrameRecord {
                            common_information: (section, index),
                            descriptions: Vec::new(),
                        });
                        common_keys.insert(key, self.frame_records.len() - 1);
                        self.frame_records.len() - 1
                    }
                };
                offset_to_record.insert(piece.input_offset, record);
            }
            let mut assigned: Vec<Option<usize>> = vec![None; pieces.len()];
            for (index, piece) in pieces.iter().enumerate() {
                if piece.is_common_information {
                    continue;
                }
                let identifier = u64::from(
                    encoding
                        .read_u32(&data, piece.input_offset as usize + 4)
                        .unwrap_or(0),
                );
                let target = (piece.input_offset + 4).wrapping_sub(identifier);
                let Some(record) = offset_to_record.get(&target).copied() else {
                    let place = self.object_message(section, piece.input_offset);
                    self.error(
                        problem("EI240", "The exception frame section is malformed.")
                            .with_context("Reason: invalid CIE reference".to_string())
                            .with_context(format!("Location: {place}")),
                    );
                    continue;
                };
                if !self.frame_description_is_live(section, piece, &relocations) {
                    continue;
                }
                self.frame_records[record]
                    .descriptions
                    .push((section, index));
                assigned[index] = Some(record);
            }
            for (index, record) in assigned.into_iter().enumerate() {
                pieces[index].record = record;
            }
            self.sections[section].merged_into = Some(synthetic);
            self.frame_pieces.insert(section, pieces);
            self.frame_relocations.insert(section, relocations);
        }
        let mut offset = 0u64;
        for record in self.frame_records.clone() {
            let (section, index) = record.common_information;
            if let Some(pieces) = self.frame_pieces.get_mut(&section) {
                pieces[index].output_offset = Some(offset);
                offset += pieces[index].size;
            }
            for (section, index) in record.descriptions {
                if let Some(pieces) = self.frame_pieces.get_mut(&section) {
                    pieces[index].output_offset = Some(offset);
                    offset += pieces[index].size;
                }
            }
        }
        self.sections[synthetic].size = offset + 4;
        let descriptions: usize = self
            .frame_records
            .iter()
            .map(|record| record.descriptions.len())
            .sum();
        self.frame_header_size = 12 + descriptions as u64 * 8;
        if let Some(header) = self.synthetic_section(".eh_frame_hdr") {
            self.sections[header].size = self.frame_header_size;
        }
    }

    fn frame_description_is_live(
        &self,
        section: SectionIndex,
        piece: &FramePiece,
        relocations: &[RawRelocation],
    ) -> bool {
        let Some(first) = piece.first_relocation else {
            return false;
        };
        let Some(symbol) = self.symbol_of(section, relocations[first].symbol) else {
            return false;
        };
        let symbol = &self.symbols[symbol];
        if !symbol.is_defined() {
            return false;
        }
        symbol
            .section
            .is_some_and(|target| self.sections[target].live)
    }

    fn frame_encoding_of(&self, data: &[u8]) -> Option<u8> {
        let mut at = 8usize;
        let version = *data.get(at)?;
        at += 1;
        if version != 1 && version != 3 {
            return None;
        }
        let end = data[at..].iter().position(|byte| *byte == 0)? + at;
        let augmentation = data[at..end].to_vec();
        at = end + 1;
        let skip_leb = |at: &mut usize| -> Option<()> {
            loop {
                let byte = *data.get(*at)?;
                *at += 1;
                if byte & 0x80 == 0 {
                    return Some(());
                }
            }
        };
        skip_leb(&mut at)?;
        skip_leb(&mut at)?;
        if version == 1 {
            at += 1;
        } else {
            skip_leb(&mut at)?;
        }
        for character in augmentation {
            match character {
                b'R' => return data.get(at).copied(),
                b'z' => skip_leb(&mut at)?,
                b'L' => at += 1,
                b'P' => {
                    let encoding = *data.get(at)?;
                    at += 1;
                    let size = match encoding & 0x0f {
                        0x00 | 0x08 => self.word_size() as usize,
                        0x02 | 0x0a => 2,
                        0x03 | 0x0b => 4,
                        0x04 | 0x0c => 8,
                        _ => return None,
                    };
                    at += size;
                }
                b'B' | b'S' | b'G' => {}
                _ => return None,
            }
        }
        Some(0)
    }

    pub fn update_frame_header(&mut self) -> bool {
        let (Some(header), Some(frame)) = (
            self.synthetic_section(".eh_frame_hdr"),
            self.synthetic_section(".eh_frame"),
        ) else {
            return false;
        };
        if !self.live_synthetic(".eh_frame_hdr") {
            return false;
        }
        let header_address = self.section_address(header, 0);
        let frame_address = self.section_address(frame, 0);
        let frame_pointer = frame_address.wrapping_sub(header_address).wrapping_sub(4) as i64;
        let mut large = frame_pointer != i64::from(frame_pointer as i32);
        let mut entries: Vec<(i64, i64)> = Vec::new();
        for record in self.frame_records.clone() {
            let (cie_section, cie_index) = record.common_information;
            let cie = self.frame_pieces[&cie_section][cie_index].clone();
            let start = cie.input_offset as usize;
            let data = self.sections[cie_section].data[start..start + cie.size as usize].to_vec();
            let encoding = self.frame_encoding_of(&data).unwrap_or(0);
            if encoding & 0x70 != 0x00 && encoding & 0x70 != 0x10 {
                self.error(
                    problem("EI240", "The exception frame section is malformed.")
                        .with_context("Reason: unknown FDE size encoding".to_string()),
                );
                continue;
            }
            for (section, index) in record.descriptions {
                let piece = self.frame_pieces[&section][index].clone();
                let Some(first) = piece.first_relocation else {
                    continue;
                };
                let relocation = self.frame_relocations[&section][first];
                let Some(symbol) = self.symbol_of(section, relocation.symbol) else {
                    continue;
                };
                let program_counter = self
                    .symbol_address(symbol, relocation.addend)
                    .wrapping_sub(header_address) as i64;
                let description = frame_address
                    .wrapping_add(piece.output_offset.unwrap_or(0))
                    .wrapping_sub(header_address) as i64;
                large |= program_counter != i64::from(program_counter as i32)
                    || description != i64::from(description as i32);
                entries.push((program_counter, description));
            }
        }
        entries.sort_by_key(|entry| entry.0);
        entries.dedup_by_key(|entry| entry.0);
        let entry_size = if large { 16 } else { 8 };
        let size = 4 + if large { 8 } else { 4 } + 4 + entries.len() as u64 * entry_size;
        self.frame_header_entries = entries;
        self.frame_header_large = large;
        let old = self.frame_header_size;
        self.frame_header_size = size.max(old);
        self.sections[header].size = self.frame_header_size;
        self.frame_header_size != old
    }

    fn write_exception_frame(&mut self, buffer: &mut [u8]) {
        let encoding = self.encoding();
        for section in self.frame_pieces.keys().copied().collect::<Vec<_>>() {
            let mut data = self.sections[section].data.clone();
            self.apply_relocations(section, &mut data);
            let pieces = self.frame_pieces[&section].clone();
            for piece in pieces {
                let Some(output) = piece.output_offset else {
                    continue;
                };
                let start = piece.input_offset as usize;
                let end = start + piece.size as usize;
                let at = output as usize;
                buffer[at..at + piece.size as usize].copy_from_slice(&data[start..end]);
                let mut length = Vec::new();
                encoding.put_u32(&mut length, (piece.size - 4) as u32);
                buffer[at..at + 4].copy_from_slice(&length);
                if let Some(record) = piece.record {
                    let (cie_section, cie_index) = self.frame_records[record].common_information;
                    let cie_output = self.frame_pieces[&cie_section][cie_index]
                        .output_offset
                        .unwrap_or(0);
                    let mut pointer = Vec::new();
                    encoding.put_u32(&mut pointer, (output + 4 - cie_output) as u32);
                    buffer[at + 4..at + 8].copy_from_slice(&pointer);
                }
            }
        }
    }

    fn write_frame_header(&self, buffer: &mut [u8]) {
        let encoding = self.encoding();
        let (Some(header), Some(frame)) = (
            self.synthetic_section(".eh_frame_hdr"),
            self.synthetic_section(".eh_frame"),
        ) else {
            return;
        };
        let large = self.frame_header_large;
        let frame_pointer = self
            .section_address(frame, 0)
            .wrapping_sub(self.section_address(header, 0))
            .wrapping_sub(4);
        let mut out = vec![
            1u8,
            0x10 | if large { 0x0c } else { 0x0b },
            0x03,
            0x30 | if large { 0x0c } else { 0x0b },
        ];
        let field = |out: &mut Vec<u8>, value: u64| {
            if large {
                encoding.put_u64(out, value);
            } else {
                encoding.put_u32(out, value as u32);
            }
        };
        field(&mut out, frame_pointer);
        encoding.put_u32(&mut out, self.frame_header_entries.len() as u32);
        for (program_counter, description) in &self.frame_header_entries {
            field(&mut out, *program_counter as u64);
            field(&mut out, *description as u64);
        }
        let length = out.len().min(buffer.len());
        buffer[..length].copy_from_slice(&out[..length]);
    }

    pub fn scan_section(&mut self, section: SectionIndex, undefined: &mut Vec<UndefinedReference>) {
        let relocations = self.raw_relocations(section);
        match self.target.architecture {
            Architecture::X86_64 => self.scan_x86_64(section, &relocations, undefined),
            Architecture::X86 => self.scan_x86(section, &relocations, undefined),
            Architecture::Aarch64 => self.scan_aarch64(section, &relocations, undefined),
            Architecture::Arm => self.scan_arm(section, &relocations, undefined),
        }
    }

    pub fn reject_unknown_relocation(
        &mut self,
        section: SectionIndex,
        offset: u64,
        kind: u32,
        symbol: SymbolIndex,
    ) {
        let name = self.symbols[symbol].name.clone();
        let place = self.error_location(section, offset);
        self.error(
            problem("EI210", "The linker does not support this relocation.")
                .with_context(format!("Relocation: {kind}"))
                .with_context(format!("Symbol: {name}"))
                .with_context(format!("Location: {place}")),
        );
    }

    fn scan_x86_64(
        &mut self,
        section: SectionIndex,
        relocations: &[RawRelocation],
        undefined: &mut Vec<UndefinedReference>,
    ) {
        let mut position = 0;
        while position < relocations.len() {
            let relocation = relocations[position];
            position += 1;
            let Some(symbol) = self.symbol_of(section, relocation.symbol) else {
                self.error(
                    problem("EI230", "A relocation names a symbol that does not exist.")
                        .with_context(format!(
                            "Location: {}",
                            self.error_location(section, relocation.offset)
                        )),
                );
                continue;
            };
            let kind = relocation.kind;
            let offset = relocation.offset;
            let addend = relocation.addend;
            if self.symbols[symbol].is_undefined()
                && relocation.symbol != 0
                && self.maybe_report_undefined(symbol, section, offset, undefined)
            {
                continue;
            }
            let expression = match kind {
                0 => continue,
                14 | 12 | 10 | 11 | 1 => Expression::Absolute,
                15 | 13 | 2 | 24 => {
                    self.process_program_counter(section, kind, offset, addend, symbol);
                    continue;
                }
                26 | 29 => {
                    self.has_got_plt_offset_relocation = true;
                    Expression::GotPltOnlyProgramCounter
                }
                25 => {
                    self.has_got_plt_offset_relocation = true;
                    Expression::GotPltRelative
                }
                3 | 27 => {
                    self.has_got_plt_offset_relocation = true;
                    Expression::GotPlt
                }
                31 => {
                    self.has_got_plt_offset_relocation = true;
                    Expression::PltGotPlt
                }
                9 | 41 | 42 | 43 => Expression::GotProgramCounter,
                4 => {
                    self.process_plt_program_counter(section, kind, offset, addend, symbol);
                    continue;
                }
                23 | 18 => {
                    if self.check_tls_local_executable(section, offset, symbol, kind) {
                        continue;
                    }
                    Expression::ThreadPointerRelative
                }
                22 | 44 | 50 => {
                    self.handle_tls_initial_executable(
                        section,
                        Expression::GotProgramCounter,
                        kind,
                        offset,
                        addend,
                        symbol,
                        true,
                    );
                    continue;
                }
                19 => {
                    if self.handle_tls_general_dynamic(
                        section,
                        Expression::TlsGeneralDynamicProgramCounter,
                        Expression::GotProgramCounter,
                        Expression::ThreadPointerRelative,
                        kind,
                        offset,
                        addend,
                        symbol,
                    ) {
                        position += 1;
                    }
                    continue;
                }
                20 => {
                    if self.handle_tls_local_dynamic(
                        section,
                        Expression::TlsLocalDynamicProgramCounter,
                        kind,
                        offset,
                        addend,
                        symbol,
                    ) {
                        position += 1;
                    }
                    continue;
                }
                21 | 17 => {
                    let expression = if self.is_shared() {
                        Expression::DynamicThreadRelative
                    } else {
                        Expression::ThreadPointerRelative
                    };
                    self.add_relocation(section, expression, kind, offset, addend, symbol);
                    continue;
                }
                35 => {
                    if !self.is_shared() {
                        self.add_relocation(
                            section,
                            Expression::ThreadPointerRelative,
                            kind,
                            offset,
                            addend,
                            symbol,
                        );
                    }
                    continue;
                }
                34 | 45 => {
                    self.handle_tls_descriptor(
                        section,
                        Expression::TlsDescriptorProgramCounter,
                        Expression::GotProgramCounter,
                        kind,
                        offset,
                        addend,
                        symbol,
                    );
                    continue;
                }
                32 | 33 => Expression::Size,
                _ => {
                    self.reject_unknown_relocation(section, offset, kind, symbol);
                    continue;
                }
            };
            self.process(section, expression, kind, offset, symbol, addend);
        }
    }
}

fn needs_got(expression: Expression) -> bool {
    matches!(
        expression,
        Expression::Got
            | Expression::GotOffset
            | Expression::Aarch64GotPageProgramCounter
            | Expression::GotProgramCounter
            | Expression::GotPlt
            | Expression::Aarch64GotPage
    )
}

fn needs_plt(expression: Expression) -> bool {
    matches!(
        expression,
        Expression::Plt
            | Expression::PltProgramCounter
            | Expression::PltGotRelative
            | Expression::PltGotPlt
            | Expression::GotPltGotRelative
            | Expression::GotPltProgramCounter
    )
}

fn from_plt(expression: Expression) -> Expression {
    match expression {
        Expression::PltProgramCounter => Expression::ProgramCounter,
        Expression::Plt => Expression::Absolute,
        Expression::PltGotPlt => Expression::GotPltRelative,
        Expression::PltGotRelative => Expression::GotRelative,
        other => other,
    }
}

fn x86_implicit_addend(kind: u32, data: &[u8], at: usize, encoding: Encoding) -> i64 {
    match kind {
        22 | 23 => sign_extend(u64::from(data.get(at).copied().unwrap_or(0)), 8),
        20 | 21 => sign_extend(u64::from(encoding.read_u16(data, at).unwrap_or(0)), 16),
        1 | 2 | 3 | 4 | 8 | 9 | 10 | 11 | 15 | 16 | 17 | 18 | 19 | 32 | 36 | 37 | 38 | 39 | 42
        | 43 | 14 | 35 => sign_extend(u64::from(encoding.read_u32(data, at).unwrap_or(0)), 32),
        41 => sign_extend(u64::from(encoding.read_u32(data, at + 4).unwrap_or(0)), 32),
        _ => 0,
    }
}

fn arm_implicit_addend(kind: u32, data: &[u8], at: usize, encoding: Encoding, j1_j2: bool) -> i64 {
    let word = u64::from(encoding.read_u32(data, at).unwrap_or(0));
    let high = u64::from(encoding.read_u16(data, at).unwrap_or(0));
    let low = u64::from(encoding.read_u16(data, at + 2).unwrap_or(0));
    match kind {
        2 | 25 | 21 | 24 | 26 | 96 | 160 | 3 | 23 | 9 | 38 | 41 | 17 | 18 | 104 | 107 | 105
        | 108 | 106 | 19 => sign_extend(word, 32),
        42 => sign_extend(word, 31),
        28 | 29 | 1 | 27 => sign_extend((word << 2) & 0xffff_ffff, 26),
        103 => sign_extend(high << 1, 9),
        102 => sign_extend(high << 1, 12),
        51 => sign_extend(
            ((high & 0x0400) << 10)
                | ((low & 0x0800) << 8)
                | ((low & 0x2000) << 5)
                | ((high & 0x003f) << 12)
                | ((low & 0x07ff) << 1),
            20,
        ),
        10 if !j1_j2 => sign_extend(((high & 0x7ff) << 12) | ((low & 0x7ff) << 1), 22),
        10 | 30 => sign_extend(
            ((high & 0x0400) << 14)
                | (!((low ^ (high << 3)) << 10) & 0x0080_0000)
                | (!((low ^ (high << 1)) << 11) & 0x0040_0000)
                | ((high & 0x003ff) << 12)
                | ((low & 0x007ff) << 1),
            24,
        ),
        43 | 44 | 45 | 46 | 84 | 85 => {
            let value = word & 0x000f_0fff;
            sign_extend(((value & 0x000f_0000) >> 4) | (value & 0x00fff), 16)
        }
        47 | 48 | 49 | 50 | 87 | 88 => sign_extend(
            ((high & 0x000f) << 12)
                | ((high & 0x0400) << 1)
                | ((low & 0x7000) >> 4)
                | (low & 0x00ff),
            16,
        ),
        132..=135 => (high & 0xff) as i64,
        57..=61 => {
            let value = ((word & 0xff) as u32).rotate_right((((word & 0xf00) >> 8) * 2) as u32);
            if word & 0x0040_0000 != 0 {
                -i64::from(value)
            } else {
                i64::from(value)
            }
        }
        4 | 62 | 63 => {
            let immediate = (word & 0xfff) as i64;
            if word & 0x0080_0000 != 0 {
                immediate
            } else {
                -immediate
            }
        }
        64..=66 => {
            let immediate = ((word & 0xf00) >> 4 | (word & 0xf)) as i64;
            if word & 0x0080_0000 != 0 {
                immediate
            } else {
                -immediate
            }
        }
        53 => {
            let immediate = ((high & 0x0400) << 1 | (low & 0x7000) >> 4 | (low & 0x00ff)) as i64;
            if high & 0x00f0 != 0 {
                -immediate
            } else {
                immediate
            }
        }
        11 => ((((high & 0xff) << 2) + 4) & 0x3ff) as i64 - 4,
        54 => {
            let immediate = (low & 0x0fff) as i64;
            if high & 0x0080 != 0 {
                immediate
            } else {
                -immediate
            }
        }
        _ => 0,
    }
}

impl<'s> Context<'s> {
    pub fn post_scan_relocations(&mut self) {
        let mut order = self.global_symbols();
        for file in self.object_files.clone() {
            let first_global = self.files[file].first_global;
            for (position, id) in self.files[file].symbols.iter().enumerate() {
                if position == 0 || position >= first_global || *id == usize::MAX {
                    continue;
                }
                order.push(*id);
            }
        }
        if self.needs_tls_local_dynamic && self.tls_index_offset.is_none() {
            let entries = self.got_size / self.target.got_entry_size;
            self.tls_index_offset = Some(entries * self.word_size());
            self.got_size += 2 * self.target.got_entry_size;
            let offset = self.tls_index_offset.unwrap_or(0);
            let got = self.synthetic_section(".got").unwrap_or(0);
            if self.is_shared() {
                self.dynamic_relocations.push(DynamicRelocation {
                    kind: self.target.module_index_relocation,
                    section: got,
                    offset,
                    symbol: None,
                    use_symbol_value: false,
                    addend: 0,
                    expression: Expression::Addend,
                });
            } else {
                {
                    let got_section = self.synthetic_section(".got").unwrap_or(0);
                    let symbol_for_got = self.dummy_symbol();
                    self.add_relocation(
                        got_section,
                        Expression::Addend,
                        self.target.symbolic_relocation,
                        offset,
                        1,
                        symbol_for_got,
                    );
                }
            }
        }
        for id in order {
            let flags = self.symbols[id].needs;
            if self.handle_non_preemptible_indirect(id, flags) {
                continue;
            }
            if flags == 0 {
                continue;
            }
            if flags & needs::GOT != 0 {
                self.add_got_entry(id);
            }
            if flags & needs::PLT != 0 {
                self.add_plt_entry(id, false);
            }
            if flags & needs::COPY != 0 {
                if self.symbols[id].is_object() {
                    self.add_copy_relocation(id);
                } else if !self.symbols[id].is_defined() {
                    let plt = self.synthetic_section(".plt").unwrap_or(0);
                    let index = self.symbols[id].plt_index.unwrap_or(0);
                    let value = self.target.plt_header_size + self.target.plt_entry_size * index;
                    let entry = &mut self.symbols[id];
                    entry.state = SymbolState::Defined;
                    entry.section = Some(plt);
                    entry.value = value;
                    entry.size = 0;
                    entry.needs |= needs::COPY;
                }
            }
            if !self.symbols[id].is_thread_local() {
                continue;
            }
            let got = self.synthetic_section(".got").unwrap_or(0);
            let word = self.word_size();
            if flags & needs::TLS_DESCRIPTOR != 0 {
                let entries = self.got_size / self.target.got_entry_size;
                self.symbols[id].tls_descriptor_index = Some(entries);
                self.got_size += 2 * self.target.got_entry_size;
                let offset = entries * self.target.got_entry_size;
                let kind = self.target.tls_descriptor_relocation;
                self.add_addend_only_if_non_preemptible(kind, got, offset, id, kind);
            }
            if flags & needs::TLS_GENERAL_DYNAMIC != 0 {
                let entries = self.got_size / self.target.got_entry_size;
                self.symbols[id].tls_general_dynamic_index = Some(entries);
                self.got_size += 2 * self.target.got_entry_size;
                let offset = entries * self.target.got_entry_size;
                if self.symbols[id].preemptible {
                    let module = self.target.module_index_relocation;
                    let dtp = self.target.offset_relocation;
                    self.add_symbol_relocation_plain(module, got, offset, id);
                    self.add_symbol_relocation_plain(dtp, got, offset + word, id);
                } else {
                    if self.is_shared() {
                        self.dynamic_relocations.push(DynamicRelocation {
                            kind: self.target.module_index_relocation,
                            section: got,
                            offset,
                            symbol: None,
                            use_symbol_value: false,
                            addend: 0,
                            expression: Expression::Addend,
                        });
                    } else {
                        {
                            let got_section = self.synthetic_section(".got").unwrap_or(0);
                            let symbol_for_got = id;
                            self.add_relocation(
                                got_section,
                                Expression::Addend,
                                self.target.symbolic_relocation,
                                offset,
                                1,
                                symbol_for_got,
                            );
                        }
                    }
                    {
                        let got_section = self.synthetic_section(".got").unwrap_or(0);
                        let symbol_for_got = id;
                        self.add_relocation(
                            got_section,
                            Expression::Absolute,
                            self.target.offset_relocation,
                            offset + word,
                            0,
                            symbol_for_got,
                        );
                    }
                }
            }
            if flags & needs::GOT_DYNAMIC_OFFSET != 0 {
                self.allocate_got_slot(id);
                let offset = self.symbols[id].got_index.unwrap_or(0) * self.target.got_entry_size;
                {
                    let got_section = self.synthetic_section(".got").unwrap_or(0);
                    let symbol_for_got = id;
                    self.add_relocation(
                        got_section,
                        Expression::Absolute,
                        self.target.offset_relocation,
                        offset,
                        0,
                        symbol_for_got,
                    );
                }
            }
            if flags & needs::TLS_INITIAL_EXECUTABLE != 0 {
                self.has_tls_initial_executable = true;
                self.allocate_got_slot(id);
                let offset = self.symbols[id].got_index.unwrap_or(0) * self.target.got_entry_size;
                if !self.symbols[id].preemptible && !self.is_shared() {
                    {
                        let got_section = self.synthetic_section(".got").unwrap_or(0);
                        let symbol_for_got = id;
                        self.add_relocation(
                            got_section,
                            Expression::ThreadPointerRelative,
                            self.target.symbolic_relocation,
                            offset,
                            0,
                            symbol_for_got,
                        );
                    }
                } else {
                    let kind = self.target.tls_got_relocation;
                    let symbolic = self.target.symbolic_relocation;
                    self.add_addend_only_if_non_preemptible(kind, got, offset, id, symbolic);
                }
            }
        }
    }

    fn add_symbol_relocation_plain(
        &mut self,
        kind: u32,
        section: SectionIndex,
        offset: u64,
        id: SymbolIndex,
    ) {
        self.dynamic_relocations.push(DynamicRelocation {
            kind,
            section,
            offset,
            symbol: Some(id),
            use_symbol_value: true,
            addend: 0,
            expression: Expression::Addend,
        });
    }

    fn add_addend_only_if_non_preemptible(
        &mut self,
        kind: u32,
        section: SectionIndex,
        offset: u64,
        id: SymbolIndex,
        addend_kind: u32,
    ) {
        if self.symbols[id].preemptible {
            self.add_symbol_relocation_plain(kind, section, offset, id);
        } else {
            self.add_dynamic_relocation(
                false,
                kind,
                section,
                offset,
                id,
                0,
                Expression::Absolute,
                addend_kind,
            );
        }
    }

    fn allocate_got_slot(&mut self, id: SymbolIndex) {
        if self.symbols[id].got_index.is_some() {
            return;
        }
        let entries = self.got_size / self.target.got_entry_size;
        self.symbols[id].got_index = Some(entries);
        self.got_size += self.target.got_entry_size;
        self.got_entries.push(id);
    }

    pub fn add_got_entry(&mut self, id: SymbolIndex) {
        self.allocate_got_slot(id);
        let offset = self.symbols[id].got_index.unwrap_or(0) * self.target.got_entry_size;
        let got = self.synthetic_section(".got").unwrap_or(0);
        if self.symbols[id].preemptible {
            let kind = self.target.global_data_relocation;
            self.add_symbol_relocation_plain(kind, got, offset, id);
            return;
        }
        if !self.is_pic() || self.is_absolute(id) {
            {
                let got_section = self.synthetic_section(".got").unwrap_or(0);
                let symbol_for_got = id;
                self.add_relocation(
                    got_section,
                    Expression::Absolute,
                    self.target.symbolic_relocation,
                    offset,
                    0,
                    symbol_for_got,
                );
            }
        } else {
            let kind = self.target.symbolic_relocation;
            self.add_relative_relocation(got, offset, id, 0, Expression::Absolute, kind);
        }
    }

    pub fn add_plt_entry(&mut self, id: SymbolIndex, indirect: bool) {
        if indirect {
            let index = self.indirect_plt_entries.len() as u64;
            self.indirect_plt_entries.push(id);
            self.indirect_got_plt_entries.push(id);
            self.symbols[id].plt_index = Some(index);
            self.symbols[id].in_indirect_plt = true;
            let section = self.synthetic_section("igot").unwrap_or(0);
            let offset = index * self.target.got_entry_size;
            self.plt_or_dynamic_indirect(section, offset, id);
            return;
        }
        let index = self.plt_entries.len() as u64;
        self.plt_entries.push(id);
        self.got_plt_entries.push(id);
        self.symbols[id].plt_index = Some(index);
        let section = self.synthetic_section(".got.plt").unwrap_or(0);
        let offset = (index + self.target.got_plt_header_entries) * self.target.got_entry_size;
        let preemptible = self.symbols[id].preemptible;
        self.plt_relocations.push(DynamicRelocation {
            kind: self.target.jump_slot_relocation,
            section,
            offset,
            symbol: Some(id),
            use_symbol_value: preemptible,
            addend: 0,
            expression: if preemptible {
                Expression::Addend
            } else {
                Expression::Absolute
            },
        });
    }

    fn plt_or_dynamic_indirect(&mut self, section: SectionIndex, offset: u64, id: SymbolIndex) {
        let relocation = DynamicRelocation {
            kind: self.target.indirect_relative_relocation,
            section,
            offset,
            symbol: Some(id),
            use_symbol_value: false,
            addend: 0,
            expression: Expression::Absolute,
        };
        self.dynamic_relocations.push(relocation);
    }

    fn handle_non_preemptible_indirect(&mut self, id: SymbolIndex, flags: u32) -> bool {
        let symbol = &self.symbols[id];
        if !symbol.is_indirect_function() || symbol.preemptible || !symbol.is_defined() {
            return false;
        }
        if flags & (needs::GOT | needs::PLT | needs::DIRECT_RELOCATION) == 0 {
            return true;
        }
        if flags & needs::DIRECT_RELOCATION != 0 {
            let mut copy = self.symbols[id].clone();
            copy.is_local = true;
            copy.binding = symbol_binding::LOCAL;
            copy.needs = 0;
            let local = self.symbols.len();
            self.symbols.push(copy);
            self.add_plt_entry(local, true);
            self.symbols[id].in_indirect_plt = true;
            self.symbols[id].plt_index = self.symbols[local].plt_index;
            let iplt = self.synthetic_section(".iplt").unwrap_or(0);
            let value =
                self.symbols[id].plt_index.unwrap_or(0) * self.target.indirect_plt_entry_size;
            let entry = &mut self.symbols[id];
            entry.section = Some(iplt);
            entry.value = value;
            entry.size = 0;
            entry.kind = symbol_type::FUNCTION;
            if flags & needs::GOT != 0 {
                self.add_got_entry(id);
            }
        } else {
            self.add_plt_entry(id, true);
            if flags & needs::GOT != 0 {
                self.symbols[id].got_in_indirect_got = true;
            }
        }
        true
    }

    fn shared_symbol_read_only(&self, id: SymbolIndex) -> bool {
        let symbol = &self.symbols[id];
        let Some(file) = symbol.file.filter(|file| *file < self.files.len()) else {
            return false;
        };
        self.files[file].elf.program_headers.iter().any(|header| {
            (header.kind == segment_type::LOAD || header.kind == segment_type::GNU_RELRO)
                && header.flags & segment_flag::WRITE == 0
                && symbol.value >= header.virtual_address
                && symbol.value < header.virtual_address + header.memory_size
        })
    }

    fn add_copy_relocation(&mut self, id: SymbolIndex) {
        let size = self.symbols[id].size;
        let alignment = self.symbols[id].shared_alignment;
        if size == 0 || alignment == 0 {
            let name = self.symbols[id].name.clone();
            self.error(
                problem("EI241", "A copy relocation cannot be made for this symbol.")
                    .with_context(format!("Symbol: {name}")),
            );
        }
        let read_only = self.shared_symbol_read_only(id);
        let name = if read_only { ".bss.rel.ro" } else { ".bss" };
        let Some(parent) = self.synthetic_section(name) else {
            return;
        };
        let Some(output) = self.sections[parent].output else {
            return;
        };
        let section = self.sections.len();
        let mut data = InputSection::synthetic(
            name,
            section_type::NO_BITS,
            section_flag::ALLOCATE | section_flag::WRITE,
            alignment.max(1),
            Synthetic::Bss,
        );
        data.size = size;
        data.output = Some(output);
        self.sections.push(data);
        self.outputs[output].members.push(section);
        self.commit_section(output, section);
        let file = self.symbols[id].file;
        let value = self.symbols[id].value;
        let mut targets: Vec<SymbolIndex> = Vec::new();
        if let Some(file) = file.filter(|file| *file < self.files.len()) {
            let first_global = self.files[file].first_global;
            let candidates: Vec<String> = self.files[file]
                .elf_symbols
                .iter()
                .skip(first_global)
                .filter(|symbol| {
                    symbol.section != section_index::UNDEFINED
                        && symbol.section != section_index::ABSOLUTE
                        && symbol.kind != symbol_type::THREAD_LOCAL
                        && symbol.value == value
                })
                .map(|symbol| symbol.name.clone())
                .collect();
            for name in candidates {
                if let Some(alias) = self.find(&name) {
                    if self.symbols[alias].is_shared() && !targets.contains(&alias) {
                        targets.push(alias);
                    }
                }
            }
        }
        if !targets.contains(&id) {
            targets.push(id);
        }
        targets.sort_unstable();
        for target in targets {
            let entry = &mut self.symbols[target];
            entry.state = SymbolState::Defined;
            entry.section = Some(section);
            entry.value = 0;
            entry.used_in_regular_object = true;
            entry.needs &= needs::GOT;
        }
        self.dynamic_relocations.push(DynamicRelocation {
            kind: self.target.copy_relocation,
            section,
            offset: 0,
            symbol: Some(id),
            use_symbol_value: true,
            addend: 0,
            expression: Expression::Addend,
        });
    }

    pub fn include_in_symbol_table(&self, id: SymbolIndex) -> bool {
        let symbol = &self.symbols[id];
        if symbol.is_defined() {
            let Some(section) = symbol.section else {
                return true;
            };
            if !self.sections[section].live {
                return false;
            }
            if self.sections[section].is_merge && self.sections[section].merged_into.is_some() {
                return self
                    .merge_piece(section, symbol.value)
                    .is_none_or(|piece| piece.live);
            }
            return true;
        }
        symbol.referenced || !self.options.gc_sections
    }

    fn merge_piece(&self, section: SectionIndex, offset: u64) -> Option<&MergePiece> {
        let pieces = &self.sections[section].pieces;
        let position = pieces.partition_point(|piece| piece.input_offset <= offset);
        if position == 0 {
            return None;
        }
        pieces.get(position - 1)
    }

    fn keep_local_in_symbol_table(&self, id: SymbolIndex) -> bool {
        let symbol = &self.symbols[id];
        if symbol.is_section() {
            return false;
        }
        if self.target.architecture == Architecture::Arm {
            if let Some(section) = symbol.section {
                if self.sections[section].kind == section_type::ARM_EXCEPTION_INDEX {
                    return false;
                }
            }
        }
        match self.options.discard {
            Discard::None => return true,
            Discard::All => return false,
            _ => {}
        }
        if symbol.name.starts_with(".L") {
            let in_merge = symbol
                .section
                .is_some_and(|section| self.sections[section].flags & section_flag::MERGE != 0);
            if self.options.discard == Discard::Locals || in_merge {
                return false;
            }
        }
        true
    }

    fn add_to_symbol_table(&mut self, id: SymbolIndex) {
        let name = self.symbols[id].name.clone();
        let offset = if name.is_empty() {
            0
        } else {
            let offset = self.string_table.len() as u32;
            self.string_table.extend_from_slice(name.as_bytes());
            self.string_table.push(0);
            offset
        };
        self.symbol_table.push(id);
        self.symbol_string_offsets.push(offset);
    }

    pub fn build_symbol_tables(&mut self) {
        let has_table = self.options.strip != Strip::All;
        if has_table {
            for file in self.object_files.clone() {
                let first_global = self.files[file].first_global;
                let handles = self.files[file].symbols.clone();
                for (position, id) in handles.into_iter().enumerate() {
                    if position == 0 || position >= first_global || id == usize::MAX {
                        continue;
                    }
                    if !self.symbols[id].is_defined() {
                        continue;
                    }
                    if let Some(section) = self.symbols[id].section {
                        if !self.sections[section].live {
                            self.symbols[id].state = SymbolState::Undefined;
                            continue;
                        }
                    }
                    if self.include_in_symbol_table(id) && self.keep_local_in_symbol_table(id) {
                        self.add_to_symbol_table(id);
                    }
                }
            }
        }
        self.symbol_table_first_global = self.symbol_table.len();
        for id in self.global_symbols() {
            if !self.symbols[id].used_in_regular_object || !self.include_in_symbol_table(id) {
                continue;
            }
            if !self.is_relocatable() {
                let binding = self.compute_binding(id);
                self.symbols[id].binding = binding;
            }
            if has_table {
                self.add_to_symbol_table(id);
            }
            let symbol = &self.symbols[id];
            if (symbol.exported || symbol.preemptible)
                && symbol.binding != symbol_binding::LOCAL
                && self.has_dynamic_symbols
            {
                self.add_dynamic_symbol(id);
            }
        }
        if has_table && !self.is_relocatable() {
            let has_file = self.symbol_table[..self.symbol_table_first_global]
                .iter()
                .any(|id| self.symbols[*id].kind == symbol_type::FILE);
            let localized = self.symbol_table[self.symbol_table_first_global..]
                .iter()
                .any(|id| self.symbols[*id].binding == symbol_binding::LOCAL);
            if has_file && localized {
                let mut data = SymbolData::new("");
                data.state = SymbolState::Defined;
                data.is_local = true;
                data.binding = symbol_binding::LOCAL;
                data.kind = symbol_type::FILE;
                data.file = Some(INTERNAL);
                let id = self.symbols.len();
                self.symbols.push(data);
                self.add_to_symbol_table(id);
                self.synthetic_file_symbol = Some(id);
            }
        }
    }

    pub fn finalize_symbol_table(&mut self) {
        let synthetic = self.synthetic_file_symbol;
        self.sort_symbol_table(synthetic);
    }

    fn sort_symbol_table(&mut self, synthetic_file: Option<SymbolIndex>) {
        let mut groups: Vec<SymbolGroup> = Vec::new();
        let mut localized = Vec::new();
        let mut globals = Vec::new();
        let mut file_entry = None;
        for (position, (id, offset)) in self
            .symbol_table
            .iter()
            .copied()
            .zip(self.symbol_string_offsets.iter().copied())
            .enumerate()
        {
            let local =
                self.symbols[id].is_local || self.symbols[id].binding == symbol_binding::LOCAL;
            if !local {
                globals.push((id, offset));
            } else if Some(id) == synthetic_file {
                file_entry = Some((id, offset));
            } else if synthetic_file.is_some() && position >= self.symbol_table_first_global {
                localized.push((id, offset));
            } else {
                let file = self.symbols[id].file;
                match groups.iter_mut().find(|group| group.0 == file) {
                    Some(group) => group.1.push((id, offset)),
                    None => groups.push((file, vec![(id, offset)])),
                }
            }
        }
        let mut sorted: Vec<(SymbolIndex, u32)> =
            groups.into_iter().flat_map(|group| group.1).collect();
        if let Some(entry) = file_entry {
            sorted.push(entry);
            sorted.extend(localized);
        }
        self.symbol_table_first_global = sorted.len();
        sorted.extend(globals);
        self.symbol_table = sorted.iter().map(|entry| entry.0).collect();
        self.symbol_string_offsets = sorted.iter().map(|entry| entry.1).collect();
    }

    pub fn remove_unused_synthetic_sections(&mut self) {
        for section in 0..self.sections.len() {
            let Some(which) = self.sections[section].synthetic else {
                continue;
            };
            if matches!(which, Synthetic::Merge | Synthetic::Common) {
                continue;
            }
            let Some(output) = self.sections[section].output else {
                continue;
            };
            let may_grow_late = self.is_pic()
                && self.got_has_deferred_entries
                && which
                    == if self.options.pack_relative_relocations {
                        Synthetic::RelativeRelocations
                    } else {
                        Synthetic::DynamicRelocations
                    };
            if self.is_needed(section) || may_grow_late {
                continue;
            }
            self.sections[section].live = false;
            self.outputs[output]
                .members
                .retain(|member| *member != section);
        }
    }

    pub fn sort_output_sections(&mut self) {
        for output in self.order.clone() {
            let rank = self.rank_of(output);
            self.outputs[output].rank = rank;
            self.outputs[output].relocation_read_only = self.is_relocation_read_only(output);
        }
        let outputs = &self.outputs;
        let starts = &self.options.section_start;
        self.order.sort_by(|left, right| {
            let a = &outputs[*left];
            let b = &outputs[*right];
            if a.rank != b.rank {
                return a.rank.cmp(&b.rank);
            }
            if a.rank & rank::NOT_ADDRESS_SET == 0 {
                return starts.get(&a.name).cmp(&starts.get(&b.name));
            }
            std::cmp::Ordering::Equal
        });
        let mut seen_relocation_read_only = false;
        let relocation_padding = self
            .synthetic_section(".relro_padding")
            .and_then(|section| self.sections[section].output);
        let mut kept = Vec::new();
        for output in self.order.clone() {
            let empty = self.outputs[output].members.is_empty();
            let mut discardable = empty && !self.outputs[output].used_in_expression;
            if Some(output) == relocation_padding && !seen_relocation_read_only {
                discardable = true;
            }
            if discardable {
                self.outputs[output].live = false;
                for member in self.outputs[output].members.clone() {
                    self.sections[member].live = false;
                }
                continue;
            }
            let section = &self.outputs[output];
            if section.relocation_read_only
                && !(section.kind == section_type::NO_BITS
                    && section.flags & section_flag::THREAD_LOCAL != 0)
            {
                seen_relocation_read_only = true;
            }
            kept.push(output);
        }
        self.order = kept;
        for output in self.order.clone() {
            let name = self.outputs[output].name.clone();
            if name == ".init_array" || name == ".fini_array" {
                self.sort_by_priority(output);
            } else if name == ".ctors" || name == ".dtors" {
                self.sort_constructors(output);
            }
        }
    }

    fn sort_by_priority(&mut self, output: OutputIndex) {
        let mut members: Vec<(i64, SectionIndex)> = self.outputs[output]
            .members
            .iter()
            .map(|member| (priority_of(&self.sections[*member].name), *member))
            .collect();
        members.sort_by_key(|entry| entry.0);
        self.outputs[output].members = members.into_iter().map(|entry| entry.1).collect();
    }

    fn sort_constructors(&mut self, output: OutputIndex) {
        let is_runtime = |name: &str, which: &str| {
            let base = name.rsplit('/').next().unwrap_or(name);
            let Some(stem) = base.strip_suffix(".o") else {
                return false;
            };
            if let Some(rest) = stem.strip_prefix("clang_rt.") {
                return rest.starts_with(which);
            }
            match stem.strip_prefix(which) {
                Some(rest) => rest.len() <= 1,
                None => false,
            }
        };
        let mut members: Vec<(bool, bool, i64, SectionIndex)> = self.outputs[output]
            .members
            .iter()
            .map(|member| {
                let file = self.file_name(self.sections[*member].file);
                let begin = is_runtime(&file, "crtbegin");
                let end = is_runtime(&file, "crtend");
                (
                    begin,
                    end,
                    priority_of(&self.sections[*member].name),
                    *member,
                )
            })
            .collect();
        members.sort_by(|left, right| {
            if left.0 != right.0 {
                return right.0.cmp(&left.0);
            }
            if left.1 != right.1 {
                return left.1.cmp(&right.1);
            }
            right.2.cmp(&left.2)
        });
        self.outputs[output].members = members.into_iter().map(|entry| entry.3).collect();
    }

    pub fn assign_section_indices(&mut self) {
        let mut index = 0;
        self.section_names = StringTable::new();
        for output in self.order.clone() {
            index += 1;
            self.outputs[output].index = index;
            let name = self.outputs[output].name.clone();
            self.outputs[output].name_offset = self.section_names.add(&name);
        }
        if let Some(section) = self.synthetic_section(".shstrtab") {
            self.sections[section].size = self.section_names.len() as u64;
        }
    }
}

impl<'s> Context<'s> {
    fn segment_add(&mut self, segment: usize, output: OutputIndex) {
        let alignment = self.outputs[output].alignment;
        let entry = &mut self.segments[segment];
        entry.last = Some(output);
        if entry.first.is_none() {
            entry.first = Some(output);
        }
        entry.alignment = entry.alignment.max(alignment);
        if entry.kind == segment_type::LOAD {
            self.outputs[output].load_segment = Some(segment);
        }
    }

    fn add_segment(&mut self, kind: u32, flags: u32) -> usize {
        let mut segment = Segment::new(kind, flags);
        if kind == segment_type::LOAD {
            segment.alignment = self.maximum_page_size;
        } else {
            segment.alignment = 0;
        }
        self.segments.push(segment);
        self.segments.len() - 1
    }

    fn needs_load(&self, output: OutputIndex) -> bool {
        let section = &self.outputs[output];
        if section.flags & section_flag::ALLOCATE == 0 {
            return false;
        }
        !(section.flags & section_flag::THREAD_LOCAL != 0 && section.kind == section_type::NO_BITS)
    }

    pub fn segment_flags_of(&self, output: OutputIndex) -> u32 {
        let flags = self.outputs[output].flags;
        let pure = (self.target.architecture == Architecture::Arm
            && flags & section_flag::ARM_PURE_CODE != 0)
            || (self.target.architecture == Architecture::Aarch64
                && flags & section_flag::AARCH64_PURE_CODE != 0);
        let mut result = 0;
        if !pure {
            result |= segment_flag::READ;
        }
        if flags & section_flag::WRITE != 0 {
            result |= segment_flag::WRITE;
        }
        if flags & section_flag::EXECUTE != 0 {
            result |= segment_flag::EXECUTE;
        }
        result
    }

    fn find_output(&self, name: &str) -> Option<OutputIndex> {
        self.order
            .iter()
            .copied()
            .find(|output| self.outputs[*output].name == name)
    }

    pub fn create_segments(&mut self) {
        let header = self.elf_header_output;
        let programs = self.program_header_output;
        let phdr = self.add_segment(segment_type::PROGRAM_HEADERS, segment_flag::READ);
        self.segment_add(phdr, programs);
        if let Some(interpreter) = self.find_output(".interp") {
            let flags = self.segment_flags_of(interpreter);
            let segment = self.add_segment(segment_type::INTERPRETER, flags);
            self.segment_add(segment, interpreter);
        }
        let mut flags = segment_flag::READ;
        let mut load = Some(self.add_segment(segment_type::LOAD, flags));
        if let Some(segment) = load {
            self.segment_add(segment, header);
            self.segment_add(segment, programs);
        }
        let mut relocation_segments: Vec<Segment> = Vec::new();
        let mut relocation_ends: BTreeSet<OutputIndex> = BTreeSet::new();
        let mut active = false;
        for output in self.order.clone() {
            if !self.needs_load(output) {
                continue;
            }
            if self.outputs[output].relocation_read_only {
                if !active {
                    relocation_segments
                        .push(Segment::new(segment_type::GNU_RELRO, segment_flag::READ));
                    active = true;
                }
                let alignment = self.outputs[output].alignment;
                let segment = relocation_segments.last_mut().expect("an active segment");
                segment.last = Some(output);
                if segment.first.is_none() {
                    segment.first = Some(output);
                }
                segment.alignment = segment.alignment.max(alignment);
            } else if active {
                active = false;
                relocation_ends.insert(output);
            }
        }
        for output in self.order.clone() {
            if !self.needs_load(output) {
                continue;
            }
            let new_flags = self.segment_flags_of(output);
            let incompatible = flags ^ new_flags;
            if incompatible != 0 {
                load = None;
            }
            let joins = match load {
                Some(segment) => {
                    let last = self.segments[segment].last;
                    !relocation_ends.contains(&output)
                        && (self.outputs[output].kind == section_type::NO_BITS
                            || last.is_none_or(|last| {
                                last == programs
                                    || last == header
                                    || self.outputs[last].kind != section_type::NO_BITS
                            }))
                }
                None => false,
            };
            if joins {
                let segment = load.expect("a load segment");
                self.segments[segment].flags |= new_flags;
            } else {
                load = Some(self.add_segment(segment_type::LOAD, new_flags));
                flags = new_flags;
            }
            let segment = load.expect("a load segment");
            self.segment_add(segment, output);
        }
        let mut thread_local = Segment::new(segment_type::THREAD_LOCAL, segment_flag::READ);
        for output in self.order.clone() {
            if self.outputs[output].flags & section_flag::THREAD_LOCAL != 0 {
                thread_local.last = Some(output);
                if thread_local.first.is_none() {
                    thread_local.first = Some(output);
                }
                thread_local.alignment = thread_local.alignment.max(self.outputs[output].alignment);
            }
        }
        if thread_local.first.is_some() {
            thread_local.alignment = thread_local.alignment.max(1);
            self.segments.push(thread_local);
            self.tls_segment = Some(self.segments.len() - 1);
        }
        if let Some(dynamic) = self
            .synthetic_section(".dynamic")
            .filter(|section| self.sections[*section].live)
            .and_then(|section| self.sections[section].output)
        {
            let flags = self.segment_flags_of(dynamic);
            let segment = self.add_segment(segment_type::DYNAMIC, flags);
            self.segment_add(segment, dynamic);
        }
        for mut segment in relocation_segments {
            segment.alignment = 1;
            self.segments.push(segment);
            self.relocation_read_only_segment = Some(self.segments.len() - 1);
        }
        if let Some(header) = self
            .synthetic_section(".eh_frame_hdr")
            .filter(|section| self.sections[*section].live)
            .and_then(|section| self.sections[section].output)
        {
            let flags = self.segment_flags_of(header);
            let segment = self.add_segment(segment_type::GNU_EXCEPTION_FRAME, flags);
            self.segment_add(segment, header);
        }
        let mut stack_flags = segment_flag::READ | segment_flag::WRITE;
        if self.options.executable_stack {
            stack_flags |= segment_flag::EXECUTE;
        }
        let stack = self.add_segment(segment_type::GNU_STACK, stack_flags);
        self.segments[stack].memory_size_override = Some(0);
        if let Some(property) = self.find_output(".note.gnu.property") {
            let segment = self.add_segment(segment_type::GNU_PROPERTY, segment_flag::READ);
            self.segment_add(segment, property);
        }
        let mut note: Option<usize> = None;
        for output in self.order.clone() {
            let section = &self.outputs[output];
            if section.kind == section_type::NOTE && section.flags & section_flag::ALLOCATE != 0 {
                let alignment = section.alignment;
                let start_new = match note {
                    None => true,
                    Some(segment) => self.segments[segment]
                        .last
                        .is_none_or(|last| self.outputs[last].alignment != alignment),
                };
                if start_new {
                    note = Some(self.add_segment(segment_type::NOTE, segment_flag::READ));
                }
                let segment = note.expect("a note segment");
                self.segment_add(segment, output);
            } else {
                note = None;
            }
        }
        if self.target.architecture == Architecture::Arm {
            if let Some(index) = self
                .order
                .iter()
                .copied()
                .find(|output| self.outputs[*output].kind == section_type::ARM_EXCEPTION_INDEX)
            {
                let segment =
                    self.add_segment(segment_type::ARM_EXCEPTION_INDEX, segment_flag::READ);
                self.segment_add(segment, index);
            }
        }
        let header_size = (self.segments.len() * self.encoding().program_header_size()) as u64;
        self.outputs[programs].size = header_size;
    }

    pub fn remove_empty_loads(&mut self) {
        let mut kept = Vec::new();
        let mut removed = Vec::new();
        for (index, segment) in self.segments.iter().enumerate() {
            let keep = if segment.kind != segment_type::LOAD {
                true
            } else {
                match (segment.first, segment.last) {
                    (Some(first), Some(last)) => {
                        let first = &self.outputs[first];
                        let last = &self.outputs[last];
                        last.address + last.size - first.address != 0
                    }
                    _ => false,
                }
            };
            if keep {
                kept.push(index);
            } else {
                removed.push(index);
            }
        }
        if removed.is_empty() {
            return;
        }
        let mut renumber = BTreeMap::new();
        let mut segments = Vec::new();
        for (new, old) in kept.iter().enumerate() {
            renumber.insert(*old, new);
            segments.push(self.segments[*old].clone());
        }
        for output in self.outputs.iter_mut() {
            output.load_segment = output
                .load_segment
                .and_then(|old| renumber.get(&old).copied());
        }
        self.tls_segment = self.tls_segment.and_then(|old| renumber.get(&old).copied());
        self.relocation_read_only_segment = self
            .relocation_read_only_segment
            .and_then(|old| renumber.get(&old).copied());
        self.segments = segments;
    }

    pub fn set_reserved_symbol_sections(&mut self) {
        if let Some(id) = self.find("_GLOBAL_OFFSET_TABLE_") {
            if self.reserved.contains_key("_GLOBAL_OFFSET_TABLE_") {
                let holder = if self.target.got_base_in_got_plt {
                    self.synthetic_section(".got.plt")
                } else {
                    self.synthetic_section(".got")
                };
                if let Some(section) = holder {
                    self.symbols[id].header_relative = false;
                    self.symbols[id].section = Some(section);
                    self.symbols[id].value = 0;
                }
            }
        }
        let mut last: Option<usize> = None;
        let mut last_read_only: Option<OutputIndex> = None;
        for (index, segment) in self.segments.iter().enumerate() {
            if segment.kind != segment_type::LOAD {
                continue;
            }
            last = Some(index);
            if segment.flags & segment_flag::WRITE == 0 {
                if let Some(output) = segment.last {
                    if output != self.elf_header_output && output != self.program_header_output {
                        last_read_only = Some(output);
                    }
                }
            }
        }
        let set_end = |context: &mut Context, name: &str, output: OutputIndex| {
            if let Some(id) = context.reserved.get(name).copied() {
                let entry = &mut context.symbols[id];
                entry.header_relative = false;
                entry.output_section_symbol = Some(output);
                entry.output_relative_end = true;
                entry.value = 0;
            }
        };
        if let Some(output) = last_read_only {
            set_end(self, "etext", output);
            set_end(self, "_etext", output);
        }
        if let Some(segment) = last {
            let final_section = self.segments[segment].last;
            let mut data_end = None;
            for output in self.order.clone() {
                if self.outputs[output].kind != section_type::NO_BITS {
                    data_end = Some(output);
                }
                if Some(output) == final_section {
                    break;
                }
            }
            if let Some(output) = data_end {
                set_end(self, "edata", output);
                set_end(self, "_edata", output);
            }
            if let Some(output) = final_section {
                if output != self.elf_header_output && output != self.program_header_output {
                    set_end(self, "end", output);
                    set_end(self, "_end", output);
                }
            }
        }
        if let Some(id) = self.reserved.get("__bss_start").copied() {
            if let Some(output) = self.find_output(".bss") {
                let entry = &mut self.symbols[id];
                entry.header_relative = false;
                entry.output_section_symbol = Some(output);
                entry.output_relative_end = false;
                entry.value = 0;
            }
        }
    }

    pub fn add_start_end_symbols(&mut self) {
        let define = |context: &mut Context,
                      start: &str,
                      end: &str,
                      output: Option<OutputIndex>| {
            match output {
                Some(output) => {
                    let first = context.add_optional_regular(
                        start,
                        SymbolPlace::OutputStart(output),
                        0,
                        visibility::HIDDEN,
                    );
                    let second = context.add_optional_regular(
                        end,
                        SymbolPlace::OutputEnd(output),
                        0,
                        visibility::HIDDEN,
                    );
                    if first.is_some() || second.is_some() {
                        context.outputs[output].used_in_expression = true;
                    }
                }
                None => {
                    context.add_optional_regular(start, SymbolPlace::Header, 0, visibility::HIDDEN);
                    context.add_optional_regular(end, SymbolPlace::Header, 0, visibility::HIDDEN);
                }
            }
        };
        let preinit = self.find_output_any(".preinit_array");
        let init = self.find_output_any(".init_array");
        let fini = self.find_output_any(".fini_array");
        define(
            self,
            "__preinit_array_start",
            "__preinit_array_end",
            preinit,
        );
        define(self, "__init_array_start", "__init_array_end", init);
        define(self, "__fini_array_start", "__fini_array_end", fini);
        if let Some(exception) = self.find_output_any(".ARM.exidx") {
            define(self, "__exidx_start", "__exidx_end", Some(exception));
        }
        for output in self.order.clone() {
            let name = self.outputs[output].name.clone();
            if !is_c_identifier(&name) {
                continue;
            }
            let first = self.add_optional_regular(
                &format!("__start_{name}"),
                SymbolPlace::OutputStart(output),
                0,
                visibility::PROTECTED,
            );
            let second = self.add_optional_regular(
                &format!("__stop_{name}"),
                SymbolPlace::OutputEnd(output),
                0,
                visibility::PROTECTED,
            );
            if first.is_some() || second.is_some() {
                self.outputs[output].used_in_expression = true;
            }
        }
    }

    fn find_output_any(&self, name: &str) -> Option<OutputIndex> {
        self.order
            .iter()
            .copied()
            .find(|output| self.outputs[*output].name == name)
    }

    pub fn fix_section_alignments(&mut self) {
        let mut previous: Option<usize> = None;
        for index in 0..self.segments.len() {
            if self.segments[index].kind != segment_type::LOAD {
                continue;
            }
            let Some(first) = self.segments[index].first else {
                continue;
            };
            if first != self.elf_header_output && self.outputs[first].address_fixed.is_none() {
                let separate = match self.options.separate_code {
                    SeparateCode::Loadable => true,
                    SeparateCode::Code => previous.is_some_and(|previous| {
                        (self.segments[previous].flags & segment_flag::EXECUTE)
                            != (self.segments[index].flags & segment_flag::EXECUTE)
                    }),
                    SeparateCode::None => false,
                };
                self.outputs[first].page_align = Some(if separate {
                    PageAlign::Aligned
                } else if self
                    .tls_segment
                    .is_some_and(|segment| self.segments[segment].first == Some(first))
                {
                    PageAlign::ThreadLocal
                } else {
                    PageAlign::Congruent
                });
            }
            previous = Some(index);
        }
    }

    pub fn assign_addresses(&mut self) -> bool {
        let header = self.elf_header_output;
        let programs = self.program_header_output;
        let mut dot = self.image_base;
        if self.is_relocatable() {
            dot = 0;
        }
        self.outputs[header].address = dot;
        self.outputs[programs].address = dot + self.outputs[header].size;
        if !self.is_relocatable() {
            dot += self.outputs[header].size + self.outputs[programs].size;
        }
        let mut changed = false;
        let mut thread_local_bss: Option<u64> = None;
        let page = self.maximum_page_size;
        let common = self.common_page_size;
        let padding = self
            .synthetic_section(".relro_padding")
            .and_then(|section| self.sections[section].output);
        for output in self.order.clone() {
            let saved = dot;
            let flags = self.outputs[output].flags;
            let is_thread_bss = flags & section_flag::THREAD_LOCAL != 0
                && self.outputs[output].kind == section_type::NO_BITS;
            if flags & section_flag::ALLOCATE == 0 {
                dot = 0;
            } else if is_thread_bss && self.outputs[output].address_fixed.is_none() {
                match thread_local_bss {
                    None => thread_local_bss = Some(dot),
                    Some(address) => dot = address,
                }
            } else if let Some(address) = self.outputs[output].address_fixed {
                dot = address;
            } else if let Some(mode) = self.outputs[output].page_align {
                dot = match mode {
                    PageAlign::Aligned => align_to(dot, page),
                    PageAlign::Congruent => align_to(dot, page) + dot % page,
                    PageAlign::ThreadLocal => {
                        let alignment = self
                            .tls_segment
                            .map(|segment| self.segments[segment].alignment)
                            .unwrap_or(1);
                        align_to(dot, page) + align_to(dot % page, alignment)
                    }
                };
            }
            dot = align_to(dot, self.outputs[output].alignment);
            if self.outputs[output].address != dot {
                changed = true;
            }
            self.outputs[output].address = dot;
            let start = dot;
            for member in self.outputs[output].members.clone() {
                dot = align_to(dot, self.sections[member].alignment);
                self.sections[member].offset_in_output = dot - start;
                dot += self.sections[member].size;
            }
            self.outputs[output].size = dot - start;
            if Some(output) == padding {
                self.outputs[output].size += align_to(dot, common) - dot;
            }
            if flags & section_flag::ALLOCATE == 0 {
                dot = saved;
            } else if is_thread_bss {
                thread_local_bss = Some(dot);
                dot = saved;
            }
        }
        changed
    }

    pub fn assign_file_offsets(&mut self) {
        let header = self.elf_header_output;
        let programs = self.program_header_output;
        self.outputs[header].offset = 0;
        self.outputs[programs].offset = self.outputs[header].size;
        let mut offset = if self.is_relocatable() {
            self.outputs[header].size
        } else {
            self.outputs[header].size + self.outputs[programs].size
        };
        let mut last_executable: Option<usize> = None;
        for (index, segment) in self.segments.iter().enumerate() {
            if segment.kind == segment_type::LOAD && segment.flags & segment_flag::EXECUTE != 0 {
                last_executable = Some(index);
            }
        }
        let mut bss_load: Option<usize> = None;
        for output in self.order.clone() {
            if self.outputs[output].flags & section_flag::ALLOCATE == 0 {
                continue;
            }
            offset = self.compute_file_offset(output, offset, bss_load);
            self.outputs[output].offset = offset;
            if self.outputs[output].kind != section_type::NO_BITS {
                offset += self.outputs[output].size;
            } else if self.outputs[output].load_segment.is_some() {
                bss_load = self.outputs[output].load_segment;
            }
            if self.options.separate_code != SeparateCode::None {
                if let Some(segment) = last_executable {
                    if self.segments[segment].last == Some(output) {
                        offset = align_to(offset, self.maximum_page_size);
                    }
                }
            }
        }
        for output in self.order.clone() {
            if self.outputs[output].flags & section_flag::ALLOCATE != 0 {
                continue;
            }
            offset = align_to(offset, self.outputs[output].alignment);
            self.outputs[output].offset = offset;
            if self.outputs[output].kind != section_type::NO_BITS {
                offset += self.outputs[output].size;
            }
        }
        self.section_header_offset = align_to(offset, self.word_size());
        self.file_size = self.section_header_offset
            + (self.order.len() as u64 + 1) * self.encoding().section_header_size() as u64;
    }

    fn compute_file_offset(
        &self,
        output: OutputIndex,
        offset: u64,
        bss_load: Option<usize>,
    ) -> u64 {
        let section = &self.outputs[output];
        if let Some(segment) = section.load_segment {
            if self.segments[segment].first == Some(output) {
                let alignment = self.segments[segment].alignment.max(1);
                let skew = section.address % alignment;
                let base = offset - offset % alignment + skew;
                return if base < offset {
                    base + alignment
                } else {
                    base
                };
            }
        }
        let is_tls_first = self
            .tls_segment
            .is_some_and(|segment| self.segments[segment].first == Some(output));
        if section.kind == section_type::NO_BITS && !is_tls_first {
            return offset;
        }
        let Some(segment) = section.load_segment else {
            return align_to(offset, section.alignment);
        };
        if section.size == 0 && Some(segment) == bss_load {
            return offset;
        }
        let first = self.segments[segment]
            .first
            .expect("a load segment has a first section");
        self.outputs[first].offset + section.address - self.outputs[first].address
    }

    pub fn program_headers(&self) -> Vec<ProgramHeader> {
        let mut headers = Vec::new();
        for segment in &self.segments {
            let mut header = ProgramHeader {
                kind: segment.kind,
                flags: segment.flags,
                alignment: segment.alignment,
                ..ProgramHeader::default()
            };
            if let (Some(first), Some(last)) = (segment.first, segment.last) {
                let first = &self.outputs[first];
                let last = &self.outputs[last];
                header.file_size = last.offset - first.offset;
                if last.kind != section_type::NO_BITS {
                    header.file_size += last.size;
                }
                header.memory_size = last.address + last.size - first.address;
                header.offset = first.offset;
                header.virtual_address = first.address;
                header.physical_address = first.address;
            }
            if let Some(size) = segment.memory_size_override {
                header.memory_size = size;
            }
            headers.push(header);
        }
        headers
    }
}

impl<'s> Context<'s> {
    pub fn finalize_synthetic_sizes(&mut self) {
        let entry = self.target.got_entry_size;
        let encoding = self.encoding();
        let rela = self.target.uses_addends;
        let sizes: Vec<(&str, u64)> = vec![
            (".got", self.got_size),
            (
                ".got.plt",
                (self.target.got_plt_header_entries + self.got_plt_entries.len() as u64) * entry,
            ),
            ("igot", self.indirect_got_plt_entries.len() as u64 * entry),
            (
                ".plt",
                if self.plt_entries.is_empty() {
                    0
                } else {
                    self.target.plt_header_size
                        + self.plt_entries.len() as u64 * self.target.plt_entry_size
                },
            ),
            (
                ".iplt",
                self.indirect_plt_entries.len() as u64 * self.target.indirect_plt_entry_size,
            ),
            (
                if rela { ".rela.dyn" } else { ".rel.dyn" },
                self.dynamic_relocations.len() as u64 * encoding.relocation_size(rela) as u64,
            ),
            (
                if rela { ".rela.plt" } else { ".rel.plt" },
                self.plt_relocations.len() as u64 * encoding.relocation_size(rela) as u64,
            ),
            (
                ".symtab",
                (self.symbol_table.len() as u64 + 1) * encoding.symbol_size() as u64,
            ),
            (".strtab", self.string_table.len() as u64),
            (
                ".dynsym",
                (self.dynamic_symbols.len() as u64 + 1) * encoding.symbol_size() as u64,
            ),
            (".dynstr", self.dynamic_strings.len() as u64),
            (
                ".gnu.hash",
                16 + encoding.word_size() as u64
                    * elf::gnu_hash_mask_words(encoding, self.gnu_hash_symbol_count) as u64
                    + self.gnu_hash_bucket_count as u64 * 4
                    + self.gnu_hash_symbol_count as u64 * 4,
            ),
            (
                ".hash",
                (2 + 2 * (self.dynamic_symbols.len() as u64 + 1)) * 4,
            ),
            (".gnu.version", (self.dynamic_symbols.len() as u64 + 1) * 2),
            (
                ".gnu.version_r",
                self.version_needs.len() as u64 * 16 + self.version_auxiliary_count as u64 * 16,
            ),
            (
                ".dynamic",
                self.dynamic_entry_count as u64 * encoding.dynamic_entry_size() as u64,
            ),
        ];
        for (name, size) in sizes {
            if let Some(section) = self.synthetic_section(name) {
                self.sections[section].size = size;
            }
        }
    }

    pub fn section_address(&self, section: SectionIndex, offset: u64) -> u64 {
        let data = &self.sections[section];
        if data.is_exception_frame {
            if let Some(parent) = data.merged_into {
                return self
                    .section_address(parent, 0)
                    .wrapping_add(self.frame_parent_offset(section, offset));
            }
        }
        if let Some(merged) = data.merged_into {
            let piece = self.merge_piece(section, offset);
            let (output_offset, input_offset) = piece
                .map(|piece| (piece.output_offset, piece.input_offset))
                .unwrap_or((0, 0));
            return self
                .section_address(merged, 0)
                .wrapping_add(output_offset)
                .wrapping_add(offset.wrapping_sub(input_offset));
        }
        let base = data
            .output
            .map(|output| self.outputs[output].address)
            .unwrap_or(0);
        base.wrapping_add(data.offset_in_output)
            .wrapping_add(offset)
    }

    pub fn symbol_address(&self, id: SymbolIndex, addend: i64) -> u64 {
        let symbol = &self.symbols[id];
        if !symbol.is_defined() {
            return (addend as u64).wrapping_add(0);
        }
        let address = if symbol.header_relative {
            let header = &self.outputs[self.elf_header_output];
            if symbol.output_relative_end {
                header.address.wrapping_add(u64::MAX)
            } else {
                header.address.wrapping_add(symbol.value)
            }
        } else if let Some(output) = symbol.output_section_symbol {
            let section = &self.outputs[output];
            let base = if symbol.output_relative_end {
                section.address + section.size
            } else {
                section.address
            };
            base.wrapping_add(symbol.value)
        } else if let Some(section) = symbol.section {
            if symbol.is_section() {
                self.section_address(section, symbol.value.wrapping_add(addend as u64))
                    .wrapping_sub(addend as u64)
            } else {
                self.section_address(section, symbol.value)
            }
        } else {
            symbol.value
        };
        let address = if symbol.is_thread_local() && !self.is_relocatable() {
            match self
                .tls_segment
                .and_then(|segment| self.segments[segment].first)
            {
                Some(first) => address.wrapping_sub(self.outputs[first].address),
                None => address,
            }
        } else {
            address
        };
        address.wrapping_add(addend as u64)
    }

    fn tls_segment_header(&self) -> Option<ProgramHeader> {
        let segment = self.tls_segment?;
        let headers = self.program_headers();
        headers.get(segment).copied()
    }

    pub fn thread_pointer_offset(&self, id: SymbolIndex) -> i64 {
        if self.reserved.get("_TLS_MODULE_BASE_") == Some(&id) {
            return 0;
        }
        let Some(tls) = self.tls_segment_header() else {
            return 0;
        };
        let address = self.symbol_address(id, 0);
        let alignment = tls.alignment.max(1);
        let word = self.word_size();
        match self.target.architecture {
            Architecture::Arm | Architecture::Aarch64 => {
                (address
                    .wrapping_add(word * 2)
                    .wrapping_add(tls.virtual_address.wrapping_sub(word * 2) & (alignment - 1)))
                    as i64
            }
            Architecture::X86 | Architecture::X86_64 => {
                address.wrapping_sub(tls.memory_size).wrapping_sub(
                    (0u64
                        .wrapping_sub(tls.virtual_address)
                        .wrapping_sub(tls.memory_size))
                        & (alignment - 1),
                ) as i64
            }
        }
    }

    pub fn got_address(&self, id: SymbolIndex) -> u64 {
        let symbol = &self.symbols[id];
        if symbol.got_in_indirect_got {
            let igot = self.synthetic_section("igot").unwrap_or(0);
            return self.section_address(
                igot,
                symbol.plt_index.unwrap_or(0) * self.target.got_entry_size,
            );
        }
        let got = self.synthetic_section(".got").unwrap_or(0);
        self.section_address(
            got,
            symbol.got_index.unwrap_or(0) * self.target.got_entry_size,
        )
    }

    pub fn got_plt_address(&self, id: SymbolIndex) -> u64 {
        let symbol = &self.symbols[id];
        let index = symbol.plt_index.unwrap_or(0);
        if symbol.in_indirect_plt {
            let igot = self.synthetic_section("igot").unwrap_or(0);
            return self.section_address(igot, index * self.target.got_entry_size);
        }
        let got_plt = self.synthetic_section(".got.plt").unwrap_or(0);
        self.section_address(
            got_plt,
            (index + self.target.got_plt_header_entries) * self.target.got_entry_size,
        )
    }

    pub fn plt_address(&self, id: SymbolIndex) -> u64 {
        let symbol = &self.symbols[id];
        let index = symbol.plt_index.unwrap_or(0);
        if symbol.in_indirect_plt {
            let iplt = self.synthetic_section(".iplt").unwrap_or(0);
            return self.section_address(iplt, index * self.target.indirect_plt_entry_size);
        }
        let plt = self.synthetic_section(".plt").unwrap_or(0);
        self.section_address(
            plt,
            self.target.plt_header_size + index * self.target.plt_entry_size,
        )
    }

    fn synthetic_address(&self, name: &str) -> u64 {
        self.synthetic_section(name)
            .map(|section| self.section_address(section, 0))
            .unwrap_or(0)
    }

    pub fn evaluate(&self, relocation: &Relocation, place: u64) -> u64 {
        use Expression::*;
        let addend = relocation.addend;
        let symbol = relocation.symbol;
        let got = || self.synthetic_address(".got");
        let got_plt = || self.synthetic_address(".got.plt");
        match relocation.expression {
            None => 0,
            Absolute | DynamicThreadRelative | RelaxGotProgramCounterNoPic => {
                self.symbol_address(symbol, addend)
            }
            Addend => addend as u64,
            AddendNegative => (addend as u64).wrapping_neg(),
            RelaxHint => 0,
            Got => self.got_address(symbol).wrapping_add(addend as u64),
            GotOnlyProgramCounter => got().wrapping_add(addend as u64).wrapping_sub(place),
            GotPltOnlyProgramCounter => got_plt().wrapping_add(addend as u64).wrapping_sub(place),
            GotRelative => self.symbol_address(symbol, addend).wrapping_sub(got()),
            GotPltRelative => self.symbol_address(symbol, addend).wrapping_sub(got_plt()),
            GotPlt => self
                .got_address(symbol)
                .wrapping_add(addend as u64)
                .wrapping_sub(got_plt()),
            GotOffset | TlsLocalDynamicGotOffset => (self.symbols[symbol].got_index.unwrap_or(0)
                * self.target.got_entry_size)
                .wrapping_add(addend as u64),
            Aarch64GotPageProgramCounter => {
                aarch64_page(self.got_address(symbol).wrapping_add(addend as u64))
                    .wrapping_sub(aarch64_page(place))
            }
            Aarch64GotPage => self
                .got_address(symbol)
                .wrapping_add(addend as u64)
                .wrapping_sub(aarch64_page(got())),
            GotProgramCounter => self
                .got_address(symbol)
                .wrapping_add(addend as u64)
                .wrapping_sub(place),
            GotPltGotRelative => self
                .got_plt_address(symbol)
                .wrapping_add(addend as u64)
                .wrapping_sub(got()),
            GotPltProgramCounter => self
                .got_plt_address(symbol)
                .wrapping_add(addend as u64)
                .wrapping_sub(place),
            Aarch64PageProgramCounter => {
                let value = if self.symbols[symbol].is_undefined_weak() {
                    place.wrapping_add(addend as u64)
                } else {
                    self.symbol_address(symbol, addend)
                };
                aarch64_page(value).wrapping_sub(aarch64_page(place))
            }
            ProgramCounter | ArmProgramCounterAligned => {
                let place = if relocation.expression == ArmProgramCounterAligned {
                    place & !3
                } else {
                    place
                };
                let destination = if self.symbols[symbol].is_undefined() {
                    match self.target.architecture {
                        Architecture::Arm => {
                            arm_undefined_weak_destination(relocation.kind, addend, place)
                        }
                        Architecture::Aarch64 => {
                            aarch64_undefined_weak_destination(relocation.kind, place)
                                .wrapping_add(addend as u64)
                        }
                        _ => self.symbol_address(symbol, addend),
                    }
                } else {
                    self.symbol_address(symbol, addend)
                };
                destination.wrapping_sub(place)
            }
            Plt => self.plt_address(symbol).wrapping_add(addend as u64),
            PltProgramCounter => self
                .plt_address(symbol)
                .wrapping_add(addend as u64)
                .wrapping_sub(place),
            PltGotPlt => self
                .plt_address(symbol)
                .wrapping_add(addend as u64)
                .wrapping_sub(got_plt()),
            PltGotRelative => self
                .plt_address(symbol)
                .wrapping_add(addend as u64)
                .wrapping_sub(got()),
            RelaxGotProgramCounter => self.symbol_address(symbol, addend).wrapping_sub(place),
            ThreadPointerRelative => {
                if self.symbols[symbol].is_undefined() {
                    return addend as u64;
                }
                (self.thread_pointer_offset(symbol) as u64).wrapping_add(addend as u64)
            }
            ThreadPointerRelativeNegative => {
                if self.symbols[symbol].is_undefined() {
                    return addend as u64;
                }
                (self.thread_pointer_offset(symbol) as u64)
                    .wrapping_neg()
                    .wrapping_add(addend as u64)
            }
            Size => self.symbols[symbol].size.wrapping_add(addend as u64),
            TlsDescriptor => self
                .tls_descriptor_address(symbol)
                .wrapping_add(addend as u64),
            TlsDescriptorProgramCounter => self
                .tls_descriptor_address(symbol)
                .wrapping_add(addend as u64)
                .wrapping_sub(place),
            TlsDescriptorGotPlt => self
                .tls_descriptor_address(symbol)
                .wrapping_add(addend as u64)
                .wrapping_sub(got_plt()),
            Aarch64TlsDescriptorPage => aarch64_page(
                self.tls_descriptor_address(symbol)
                    .wrapping_add(addend as u64),
            )
            .wrapping_sub(aarch64_page(place)),
            TlsDescriptorCall => 0,
            TlsGeneralDynamicGot => (self.symbols[symbol].tls_general_dynamic_index.unwrap_or(0)
                * self.target.got_entry_size)
                .wrapping_add(addend as u64),
            TlsGeneralDynamicGotPlt => self
                .global_dynamic_address(symbol)
                .wrapping_add(addend as u64)
                .wrapping_sub(got_plt()),
            TlsGeneralDynamicProgramCounter => self
                .global_dynamic_address(symbol)
                .wrapping_add(addend as u64)
                .wrapping_sub(place),
            TlsLocalDynamicGotPlt => got()
                .wrapping_add(self.tls_index_offset.unwrap_or(0))
                .wrapping_add(addend as u64)
                .wrapping_sub(got_plt()),
            TlsLocalDynamicGot => self
                .tls_index_offset
                .unwrap_or(0)
                .wrapping_add(addend as u64),
            TlsLocalDynamicProgramCounter => got()
                .wrapping_add(self.tls_index_offset.unwrap_or(0))
                .wrapping_add(addend as u64)
                .wrapping_sub(place),
            ArmStaticBaseRelative => self.symbol_address(symbol, addend),
        }
    }

    fn tls_descriptor_address(&self, id: SymbolIndex) -> u64 {
        let got = self.synthetic_address(".got");
        got + self.symbols[id].tls_descriptor_index.unwrap_or(0) * self.target.got_entry_size
    }

    fn global_dynamic_address(&self, id: SymbolIndex) -> u64 {
        let got = self.synthetic_address(".got");
        got + self.symbols[id].tls_general_dynamic_index.unwrap_or(0) * self.target.got_entry_size
    }

    pub fn range_error(
        &mut self,
        location: &str,
        relocation: &Relocation,
        value: i64,
        minimum: i64,
        maximum: u64,
    ) {
        let name = relocation_name(self.target.architecture, relocation.kind);
        let symbol = &self.symbols[relocation.symbol];
        let mut diagnostic = problem("EI208", "A relocation value does not fit in its field.")
            .with_context(format!("Location: {location}"))
            .with_context(format!("Relocation: {name}"))
            .with_context(format!("Value: {value}"))
            .with_context(format!("Range: [{minimum}, {maximum}]"));
        if !symbol.is_section() {
            diagnostic = diagnostic.with_context(format!("Symbol: {}", symbol.name));
        } else if let Some(section) = symbol.section {
            diagnostic =
                diagnostic.with_context(format!("Section: {}", self.sections[section].name));
        }
        if !symbol.is_section() {
            diagnostic =
                diagnostic.with_context(format!("Defined: {}", self.file_name(symbol.file)));
        }
        self.error(diagnostic);
    }

    pub fn check_signed(&mut self, location: &str, relocation: &Relocation, value: u64, bits: u32) {
        let signed = value as i64;
        let minimum = -(1i64 << (bits - 1));
        let maximum = (1i64 << (bits - 1)) - 1;
        if signed < minimum || signed > maximum {
            self.range_error(location, relocation, signed, minimum, maximum as u64);
        }
    }

    pub fn check_unsigned(
        &mut self,
        location: &str,
        relocation: &Relocation,
        value: u64,
        bits: u32,
    ) {
        let maximum = if bits >= 64 {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        };
        if value > maximum {
            self.range_error(location, relocation, value as i64, 0, maximum);
        }
    }

    pub fn check_either(&mut self, location: &str, relocation: &Relocation, value: u64, bits: u32) {
        let signed = value as i64;
        let minimum = -(1i64 << (bits - 1));
        let maximum = (1u64 << bits) - 1;
        if signed < minimum || (signed >= 0 && value > maximum) {
            self.range_error(location, relocation, signed, minimum, maximum);
        }
    }

    fn write_word(&self, buffer: &mut [u8], at: usize, value: u64) {
        self.encoding().write_word(buffer, at, value);
    }

    pub fn relocate_x86_64(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        relocation: &Relocation,
        value: u64,
        location: &str,
    ) {
        let encoding = self.encoding();
        match relocation.kind {
            14 => {
                self.check_either(location, relocation, value, 8);
                buffer[at] = value as u8;
            }
            15 => {
                self.check_signed(location, relocation, value, 8);
                buffer[at] = value as u8;
            }
            12 => {
                self.check_either(location, relocation, value, 16);
                encoding.write_u16(buffer, at, value as u16);
            }
            13 => {
                self.check_signed(location, relocation, value, 16);
                encoding.write_u16(buffer, at, value as u16);
            }
            10 => {
                self.check_unsigned(location, relocation, value, 32);
                encoding.write_u32(buffer, at, value as u32);
            }
            11 | 3 | 26 | 9 | 2 | 4 | 21 | 32 | 23 => {
                self.check_signed(location, relocation, value, 32);
                encoding.write_u32(buffer, at, value as u32);
            }
            1 | 17 | 18 | 24 | 25 | 27 | 29 | 31 | 33 => encoding.write_u64(buffer, at, value),
            41..=43 => {
                if relocation.expression != Expression::GotProgramCounter {
                    x86_64_relax_got(buffer, at, relocation, value);
                } else {
                    self.check_signed(location, relocation, value, 32);
                    encoding.write_u32(buffer, at, value as u32);
                }
            }
            34 | 45 | 35 | 19 => {
                if relocation.expression == Expression::ThreadPointerRelative {
                    self.x86_64_relax_general_dynamic_to_local(
                        buffer, at, relocation, value, location,
                    );
                } else if relocation.expression == Expression::GotProgramCounter {
                    self.x86_64_relax_general_dynamic_to_initial(
                        buffer, at, relocation, value, location,
                    );
                } else {
                    self.check_signed(location, relocation, value, 32);
                    encoding.write_u32(buffer, at, value as u32);
                }
            }
            20 => {
                if relocation.expression == Expression::ThreadPointerRelative {
                    self.x86_64_relax_local_dynamic_to_local(buffer, at, location);
                } else {
                    self.check_signed(location, relocation, value, 32);
                    encoding.write_u32(buffer, at, value as u32);
                }
            }
            22 | 44 | 50 => {
                if relocation.expression == Expression::ThreadPointerRelative {
                    self.x86_64_relax_initial_to_local(buffer, at, relocation, value, location);
                } else {
                    self.check_signed(location, relocation, value, 32);
                    encoding.write_u32(buffer, at, value as u32);
                }
            }
            36 => encoding.write_u64(buffer, at + 8, value),
            _ => {}
        }
    }

    fn x86_64_relax_general_dynamic_to_local(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        relocation: &Relocation,
        value: u64,
        location: &str,
    ) {
        let encoding = self.encoding();
        match relocation.kind {
            19 => {
                if buffer.get(at + 4) == Some(&0x48) && buffer.get(at + 5) == Some(&0xb8) && at >= 3
                {
                    let code: [u8; 22] = [
                        0x64, 0x48, 0x8b, 0x04, 0x25, 0, 0, 0, 0, 0x48, 0x8d, 0x80, 0, 0, 0, 0,
                        0x66, 0x0f, 0x1f, 0x44, 0, 0,
                    ];
                    buffer[at - 3..at - 3 + 22].copy_from_slice(&code);
                    encoding.write_u32(buffer, at + 9, value.wrapping_add(4) as u32);
                    return;
                }
                if at < 4 || at + 12 > buffer.len() {
                    return;
                }
                let code: [u8; 16] = [
                    0x64, 0x48, 0x8b, 0x04, 0x25, 0, 0, 0, 0, 0x48, 0x8d, 0x80, 0, 0, 0, 0,
                ];
                buffer[at - 4..at + 12].copy_from_slice(&code);
                encoding.write_u32(buffer, at + 8, value.wrapping_add(4) as u32);
            }
            34 | 45 => {
                if at < 3
                    || (buffer[at - 3] & 0xfb) != 0x48
                    || buffer[at - 2] != 0x8d
                    || (buffer[at - 1] & 0xc7) != 0x05
                {
                    self.error(
                        problem("EI231", "A thread-local descriptor relocation is used with an instruction it does not fit.")
                            .with_context(format!("Location: {location}")),
                    );
                    return;
                }
                if relocation.kind == 34 {
                    buffer[at - 3] = 0x48 | ((buffer[at - 3] >> 2) & 1);
                } else {
                    buffer[at - 3] = (buffer[at - 3] & !0x44) | ((buffer[at - 3] & 0x44) >> 2);
                }
                buffer[at - 2] = 0xc7;
                buffer[at - 1] = 0xc0 | ((buffer[at - 1] >> 3) & 7);
                encoding.write_u32(buffer, at, value.wrapping_add(4) as u32);
            }
            _ => {
                buffer[at] = 0x66;
                buffer[at + 1] = 0x90;
            }
        }
    }

    fn x86_64_relax_general_dynamic_to_initial(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        relocation: &Relocation,
        value: u64,
        location: &str,
    ) {
        let encoding = self.encoding();
        if relocation.kind == 19 {
            if buffer.get(at + 4) == Some(&0x48) && buffer.get(at + 5) == Some(&0xb8) && at >= 3 {
                let code: [u8; 22] = [
                    0x64, 0x48, 0x8b, 0x04, 0x25, 0, 0, 0, 0, 0x48, 0x03, 0x05, 0, 0, 0, 0, 0x66,
                    0x0f, 0x1f, 0x44, 0, 0,
                ];
                buffer[at - 3..at - 3 + 22].copy_from_slice(&code);
                let adjusted = value.wrapping_sub(9);
                self.check_signed(location, relocation, adjusted, 32);
                encoding.write_u32(buffer, at + 9, adjusted as u32);
                return;
            }
            if at < 4 {
                return;
            }
            let code: [u8; 16] = [
                0x64, 0x48, 0x8b, 0x04, 0x25, 0, 0, 0, 0, 0x48, 0x03, 0x05, 0, 0, 0, 0,
            ];
            buffer[at - 4..at + 12].copy_from_slice(&code);
            let adjusted = value.wrapping_sub(8);
            self.check_signed(location, relocation, adjusted, 32);
            encoding.write_u32(buffer, at + 8, adjusted as u32);
        } else if relocation.kind == 34 || relocation.kind == 45 {
            if at < 3
                || (buffer[at - 3] & 0xfb) != 0x48
                || buffer[at - 2] != 0x8d
                || (buffer[at - 1] & 0xc7) != 0x05
            {
                self.error(
                    problem("EI231", "A thread-local descriptor relocation is used with an instruction it does not fit.")
                        .with_context(format!("Location: {location}")),
                );
                return;
            }
            buffer[at - 2] = 0x8b;
            self.check_signed(location, relocation, value, 32);
            encoding.write_u32(buffer, at, value as u32);
        }
    }

    fn x86_64_relax_initial_to_local(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        relocation: &Relocation,
        value: u64,
        location: &str,
    ) {
        if at < 3 {
            return;
        }
        let register = buffer[at - 1] >> 3;
        let start = at - 3;
        let mut fits = true;
        if relocation.kind == 22 {
            let head = &buffer[start..at];
            if head == [0x48, 0x03, 0x25] {
                buffer[start..at].copy_from_slice(&[0x48, 0x81, 0xc4]);
            } else if head == [0x4c, 0x03, 0x25] {
                buffer[start..at].copy_from_slice(&[0x49, 0x81, 0xc4]);
            } else if head[..2] == [0x4c, 0x03] {
                buffer[start] = 0x4d;
                buffer[start + 1] = 0x8d;
                buffer[at - 1] = 0x80 | (register << 3) | register;
            } else if head[..2] == [0x48, 0x03] {
                buffer[start] = 0x48;
                buffer[start + 1] = 0x8d;
                buffer[at - 1] = 0x80 | (register << 3) | register;
            } else if head[..2] == [0x4c, 0x8b] {
                buffer[start] = 0x49;
                buffer[start + 1] = 0xc7;
                buffer[at - 1] = 0xc0 | register;
            } else if head[..2] == [0x48, 0x8b] {
                buffer[start] = 0x48;
                buffer[start + 1] = 0xc7;
                buffer[at - 1] = 0xc0 | register;
            } else {
                fits = false;
            }
        } else if relocation.kind == 44 {
            if at < 4 || buffer[at - 4] != 0xd5 {
                fits = false;
            } else {
                let prefix = buffer[at - 3];
                buffer[at - 3] = (prefix & !0x44) | ((prefix & 0x44) >> 2);
                buffer[at - 1] = 0xc0 | register;
                if buffer[at - 2] == 0x8b {
                    buffer[at - 2] = 0xc7;
                } else if buffer[at - 2] == 0x03 {
                    buffer[at - 2] = 0x81;
                } else {
                    fits = false;
                }
            }
        } else {
            fits = false;
        }
        if !fits {
            self.error(
                problem("EI232", "An initial executable thread-local relocation is used with an instruction it does not fit.")
                    .with_context(format!("Location: {location}"))
                    .with_context(format!("Relocation: {}", relocation_name(self.target.architecture, relocation.kind))),
            );
            return;
        }
        self.encoding()
            .write_u32(buffer, at, value.wrapping_add(4) as u32);
    }

    fn x86_64_relax_local_dynamic_to_local(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        location: &str,
    ) {
        let code: [u8; 12] = [0x66, 0x66, 0x66, 0x64, 0x48, 0x8b, 0x04, 0x25, 0, 0, 0, 0];
        if buffer.get(at + 4) == Some(&0xe8) && at >= 3 {
            buffer[at - 3..at + 9].copy_from_slice(&code);
            return;
        }
        if buffer.get(at + 4) == Some(&0xff) && buffer.get(at + 5) == Some(&0x15) && at >= 3 {
            buffer[at - 3] = 0x66;
            buffer[at - 2..at + 10].copy_from_slice(&code);
            return;
        }
        if buffer.get(at + 4) == Some(&0x48) && buffer.get(at + 5) == Some(&0xb8) && at >= 3 {
            let long: [u8; 22] = [
                0x66, 0x66, 0x66, 0x66, 0x2e, 0x0f, 0x1f, 0x84, 0, 0, 0, 0, 0, 0x64, 0x48, 0x8b,
                0x04, 0x25, 0, 0, 0, 0,
            ];
            buffer[at - 3..at + 19].copy_from_slice(&long);
            return;
        }
        self.error(
            problem(
                "EI233",
                "A local dynamic thread-local relocation is not followed by the call it needs.",
            )
            .with_context(format!("Location: {location}")),
        );
    }
}

fn x86_64_relax_got(buffer: &mut [u8], at: usize, relocation: &Relocation, value: u64) {
    let operation = buffer[at - 2];
    let modifier = buffer[at - 1];
    if operation == 0x8b {
        buffer[at - 2] = 0x8d;
        buffer[at..at + 4].copy_from_slice(&(value as u32).to_le_bytes());
        return;
    }
    if operation != 0xff {
        let prefix = buffer[at - 3];
        let rex2 = relocation.kind == 43;
        let value = value.wrapping_add(4);
        if operation == 0x85 {
            buffer[at - 1] = 0xc0 | ((modifier & 0x38) >> 3);
            buffer[at - 2] = 0xf7;
        } else {
            buffer[at - 1] = 0xc0 | ((modifier & 0x38) >> 3) | (operation & 0x3c);
            buffer[at - 2] = 0x81;
        }
        buffer[at - 3] = if rex2 {
            (prefix & !0x44) | ((prefix & 0x44) >> 2)
        } else {
            (prefix & !0x4) | ((prefix & 0x4) >> 2)
        };
        buffer[at..at + 4].copy_from_slice(&(value as u32).to_le_bytes());
        return;
    }
    if modifier == 0x15 {
        buffer[at - 2] = 0x67;
        buffer[at - 1] = 0xe8;
        buffer[at..at + 4].copy_from_slice(&(value as u32).to_le_bytes());
        return;
    }
    buffer[at - 2] = 0xe9;
    buffer[at + 3] = 0x90;
    buffer[at - 1..at + 3].copy_from_slice(&(value.wrapping_add(1) as u32).to_le_bytes());
}

fn aarch64_encode_page(out: &mut [u8], at: usize, value: u64) {
    let immediate = value >> 12;
    let low = ((immediate & 0x3) << 29) as u32;
    let high = ((immediate & 0x1f_fffc) << 3) as u32;
    let mask = (0x3u32 << 29) | (0x1f_fffcu32 << 3);
    let word = u32::from_le_bytes(out[at..at + 4].try_into().unwrap_or([0; 4]));
    out[at..at + 4].copy_from_slice(&((word & !mask) | low | high).to_le_bytes());
}

fn aarch64_encode_low12(out: &mut [u8], at: usize, value: u64) {
    let word = u32::from_le_bytes(out[at..at + 4].try_into().unwrap_or([0; 4]));
    let updated = (word & !(0xfff << 10)) | (((value & 0xfff) as u32) << 10);
    out[at..at + 4].copy_from_slice(&updated.to_le_bytes());
}

pub fn aarch64_page(value: u64) -> u64 {
    value & !0xfff
}

fn aarch64_undefined_weak_destination(kind: u32, place: u64) -> u64 {
    match kind {
        279 | 280 | 282 | 283 | 274 | 273 | 287 | 288 | 289 | 290 | 291 | 292 | 293 => place,
        _ => 0,
    }
}

fn arm_undefined_weak_destination(kind: u32, addend: i64, place: u64) -> u64 {
    let mut destination = 0u64;
    match kind {
        1 | 28 | 29 | 27 => destination = place.wrapping_add(8).wrapping_add(addend as u64),
        10 | 30 | 51 | 102 | 103 | 52 => {
            destination = place.wrapping_add(4).wrapping_add(addend as u64)
        }
        _ => {}
    }
    if destination == 0 {
        return addend as u64;
    }
    destination
}

impl<'s> Context<'s> {
    fn next_symbol(
        &mut self,
        section: SectionIndex,
        relocation: &RawRelocation,
        undefined: &mut Vec<UndefinedReference>,
    ) -> Option<SymbolIndex> {
        let Some(symbol) = self.symbol_of(section, relocation.symbol) else {
            self.error(
                problem("EI230", "A relocation names a symbol that does not exist.").with_context(
                    format!(
                        "Location: {}",
                        self.error_location(section, relocation.offset)
                    ),
                ),
            );
            return None;
        };
        if self.symbols[symbol].is_undefined()
            && relocation.symbol != 0
            && self.maybe_report_undefined(symbol, section, relocation.offset, undefined)
        {
            return None;
        }
        Some(symbol)
    }

    fn scan_x86(
        &mut self,
        section: SectionIndex,
        relocations: &[RawRelocation],
        undefined: &mut Vec<UndefinedReference>,
    ) {
        let mut position = 0;
        while position < relocations.len() {
            let relocation = relocations[position];
            position += 1;
            let Some(symbol) = self.next_symbol(section, &relocation, undefined) else {
                continue;
            };
            let kind = relocation.kind;
            let offset = relocation.offset;
            let addend = relocation.addend;
            let expression = match kind {
                0 => continue,
                22 | 20 | 1 => Expression::Absolute,
                23 | 21 | 2 => {
                    self.process_program_counter(section, kind, offset, addend, symbol);
                    continue;
                }
                4 => {
                    self.process_plt_program_counter(section, kind, offset, addend, symbol);
                    continue;
                }
                10 => {
                    self.has_got_plt_offset_relocation = true;
                    Expression::GotPltOnlyProgramCounter
                }
                9 => {
                    self.has_got_plt_offset_relocation = true;
                    Expression::GotPltRelative
                }
                3 | 43 => {
                    let at = offset as usize;
                    let data = &self.sections[section].data;
                    let expression =
                        if offset != 0 && at <= data.len() && (data[at - 1] & 0xc7) == 0x5 {
                            Expression::Got
                        } else {
                            Expression::GotPlt
                        };
                    if expression == Expression::GotPlt {
                        self.has_got_plt_offset_relocation = true;
                    }
                    expression
                }
                17 => {
                    if self.check_tls_local_executable(section, offset, symbol, kind) {
                        continue;
                    }
                    Expression::ThreadPointerRelative
                }
                34 => {
                    if self.check_tls_local_executable(section, offset, symbol, kind) {
                        continue;
                    }
                    Expression::ThreadPointerRelativeNegative
                }
                15 => {
                    self.handle_tls_initial_executable(
                        section,
                        Expression::Got,
                        kind,
                        offset,
                        addend,
                        symbol,
                        true,
                    );
                    continue;
                }
                16 => {
                    self.has_got_plt_offset_relocation = true;
                    self.handle_tls_initial_executable(
                        section,
                        Expression::GotPlt,
                        kind,
                        offset,
                        addend,
                        symbol,
                        true,
                    );
                    continue;
                }
                18 => {
                    self.has_got_plt_offset_relocation = true;
                    if self.handle_tls_general_dynamic(
                        section,
                        Expression::TlsGeneralDynamicGotPlt,
                        Expression::GotPlt,
                        Expression::ThreadPointerRelativeNegative,
                        kind,
                        offset,
                        addend,
                        symbol,
                    ) {
                        position += 1;
                    }
                    continue;
                }
                19 => {
                    self.has_got_plt_offset_relocation = true;
                    if self.handle_tls_local_dynamic(
                        section,
                        Expression::TlsLocalDynamicGotPlt,
                        kind,
                        offset,
                        addend,
                        symbol,
                    ) {
                        position += 1;
                    }
                    continue;
                }
                32 => {
                    let expression = if self.is_shared() {
                        Expression::DynamicThreadRelative
                    } else {
                        Expression::ThreadPointerRelative
                    };
                    self.add_relocation(section, expression, kind, offset, addend, symbol);
                    continue;
                }
                39 => {
                    self.has_got_plt_offset_relocation = true;
                    self.handle_tls_descriptor(
                        section,
                        Expression::TlsDescriptorGotPlt,
                        Expression::GotPlt,
                        kind,
                        offset,
                        addend,
                        symbol,
                    );
                    continue;
                }
                40 => {
                    if !self.is_shared() {
                        self.add_relocation(
                            section,
                            Expression::ThreadPointerRelative,
                            kind,
                            offset,
                            addend,
                            symbol,
                        );
                    }
                    continue;
                }
                _ => {
                    self.reject_unknown_relocation(section, offset, kind, symbol);
                    continue;
                }
            };
            self.process(section, expression, kind, offset, symbol, addend);
        }
    }

    pub fn relocate_x86(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        relocation: &Relocation,
        value: u64,
        location: &str,
    ) {
        let value = sign_extend(value, 32) as u64;
        let little = Encoding::LITTLE_32;
        match relocation.kind {
            18 | 39 | 40 => {
                if matches!(
                    relocation.expression,
                    Expression::ThreadPointerRelative | Expression::ThreadPointerRelativeNegative
                ) {
                    self.x86_relax_general_dynamic_to_local(
                        buffer, at, relocation, value, location,
                    );
                } else if relocation.expression == Expression::GotPlt {
                    self.x86_relax_general_dynamic_to_initial(
                        buffer, at, relocation, value, location,
                    );
                } else {
                    self.relocate_x86_plain(buffer, at, relocation, value, location);
                }
            }
            19 | 32 => {
                if relocation.expression == Expression::ThreadPointerRelative {
                    if relocation.kind == 32 {
                        little.write_u32(buffer, at, value as u32);
                        return;
                    }
                    let code: &[u8] = if buffer.get(at + 4) == Some(&0xe8) {
                        &[0x65, 0xa1, 0, 0, 0, 0, 0x90, 0x8d, 0x74, 0x26, 0x00]
                    } else {
                        &[0x65, 0xa1, 0, 0, 0, 0, 0x8d, 0xb6, 0, 0, 0, 0]
                    };
                    if at >= 2 && at - 2 + code.len() <= buffer.len() {
                        buffer[at - 2..at - 2 + code.len()].copy_from_slice(code);
                    }
                    return;
                }
                self.relocate_x86_plain(buffer, at, relocation, value, location);
            }
            15 | 16 => {
                if relocation.expression == Expression::ThreadPointerRelative && at >= 2 {
                    let register = (buffer[at - 1] >> 3) & 7;
                    if relocation.kind == 15 {
                        if buffer[at - 1] == 0xa1 {
                            buffer[at - 1] = 0xb8;
                        } else if buffer[at - 2] == 0x8b {
                            buffer[at - 2] = 0xc7;
                            buffer[at - 1] = 0xc0 | register;
                        } else {
                            buffer[at - 2] = 0x81;
                            buffer[at - 1] = 0xc0 | register;
                        }
                    } else if buffer[at - 2] == 0x8b {
                        buffer[at - 2] = 0xc7;
                        buffer[at - 1] = 0xc0 | register;
                    } else {
                        buffer[at - 2] = 0x8d;
                        buffer[at - 1] = 0x80 | (register << 3) | register;
                    }
                    little.write_u32(buffer, at, value as u32);
                    return;
                }
                self.relocate_x86_plain(buffer, at, relocation, value, location);
            }
            _ => self.relocate_x86_plain(buffer, at, relocation, value, location),
        }
    }

    fn relocate_x86_plain(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        relocation: &Relocation,
        value: u64,
        location: &str,
    ) {
        let little = Encoding::LITTLE_32;
        match relocation.kind {
            22 => {
                self.check_either(location, relocation, value, 8);
                buffer[at] = value as u8;
            }
            23 => {
                self.check_signed(location, relocation, value, 8);
                buffer[at] = value as u8;
            }
            20 => {
                self.check_either(location, relocation, value, 16);
                little.write_u16(buffer, at, value as u16);
            }
            21 => {
                self.check_signed(location, relocation, value, 17);
                little.write_u16(buffer, at, value as u16);
            }
            1 | 3 | 43 | 9 | 10 | 2 | 4 | 8 | 39 | 40 | 35 | 36 | 18 | 16 | 15 | 19 | 32 | 17
            | 34 | 14 | 37 => {
                self.check_signed(location, relocation, value, 32);
                little.write_u32(buffer, at, value as u32);
            }
            41 => little.write_u32(buffer, at + 4, value as u32),
            _ => {}
        }
    }

    fn x86_relax_general_dynamic_to_local(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        relocation: &Relocation,
        value: u64,
        location: &str,
    ) {
        let little = Encoding::LITTLE_32;
        match relocation.kind {
            18 => {
                let start = if at >= 2 && buffer[at - 2] == 0x04 {
                    at - 3
                } else {
                    at.saturating_sub(2)
                };
                let code = [0x65, 0xa1, 0, 0, 0, 0, 0x81, 0xe8, 0, 0, 0, 0];
                if start + code.len() <= buffer.len() {
                    buffer[start..start + code.len()].copy_from_slice(&code);
                    little.write_u32(buffer, start + 8, value as u32);
                }
            }
            39 => {
                if at < 2 || buffer[at - 2..at] != [0x8d, 0x83] {
                    self.error(
                        problem("EI231", "A thread-local descriptor relocation is used with an instruction it does not fit.")
                            .with_context(format!("Location: {location}")),
                    );
                    return;
                }
                buffer[at - 1] = 0x05;
                little.write_u32(buffer, at, value as u32);
            }
            _ => {
                buffer[at] = 0x66;
                buffer[at + 1] = 0x90;
            }
        }
    }

    fn x86_relax_general_dynamic_to_initial(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        relocation: &Relocation,
        value: u64,
        location: &str,
    ) {
        let little = Encoding::LITTLE_32;
        if relocation.kind == 18 {
            let start = if at >= 2 && buffer[at - 2] == 0x04 {
                at - 3
            } else {
                at.saturating_sub(2)
            };
            let code = [0x65, 0xa1, 0, 0, 0, 0, 0x03, 0x83, 0, 0, 0, 0];
            if start + code.len() <= buffer.len() {
                buffer[start..start + code.len()].copy_from_slice(&code);
                little.write_u32(buffer, start + 8, value as u32);
            }
        } else if relocation.kind == 39 {
            if at < 2 || buffer[at - 2..at] != [0x8d, 0x83] {
                self.error(
                    problem("EI231", "A thread-local descriptor relocation is used with an instruction it does not fit.")
                        .with_context(format!("Location: {location}")),
                );
                return;
            }
            buffer[at - 2] = 0x8b;
            little.write_u32(buffer, at, value as u32);
        }
    }

    fn scan_aarch64(
        &mut self,
        section: SectionIndex,
        relocations: &[RawRelocation],
        undefined: &mut Vec<UndefinedReference>,
    ) {
        for relocation in relocations {
            let Some(symbol) = self.next_symbol(section, relocation, undefined) else {
                continue;
            };
            let kind = relocation.kind;
            let offset = relocation.offset;
            let addend = relocation.addend;
            let expression = match kind {
                0 => continue,
                259 | 258 | 257 | 277 | 299 | 284 | 285 | 286 | 278 | 270 | 271 | 272 | 263
                | 264 | 265 | 266 | 267 | 268 | 269 => Expression::Absolute,
                262 | 261 | 260 | 274 | 273 | 287 | 288 | 289 | 290 | 291 | 292 | 293 => {
                    self.process_program_counter(section, kind, offset, addend, symbol);
                    continue;
                }
                275 | 276 => Expression::Aarch64PageProgramCounter,
                314 | 283 | 280 | 282 | 279 => {
                    self.process_plt_program_counter(section, kind, offset, addend, symbol);
                    continue;
                }
                311 => Expression::Aarch64GotPageProgramCounter,
                312 => Expression::Got,
                313 => Expression::Aarch64GotPage,
                315 | 309 => Expression::GotProgramCounter,
                549 | 551 | 553 | 555 | 557 | 559 | 571 | 547 | 548 | 545 | 546 | 544 => {
                    if self.check_tls_local_executable(section, offset, symbol, kind) {
                        continue;
                    }
                    Expression::ThreadPointerRelative
                }
                541 => {
                    self.handle_tls_initial_executable(
                        section,
                        Expression::Aarch64GotPageProgramCounter,
                        kind,
                        offset,
                        addend,
                        symbol,
                        true,
                    );
                    continue;
                }
                542 => {
                    self.handle_tls_initial_executable(
                        section,
                        Expression::Got,
                        kind,
                        offset,
                        addend,
                        symbol,
                        true,
                    );
                    continue;
                }
                562 => {
                    self.handle_tls_descriptor(
                        section,
                        Expression::Aarch64TlsDescriptorPage,
                        Expression::Aarch64GotPageProgramCounter,
                        kind,
                        offset,
                        addend,
                        symbol,
                    );
                    continue;
                }
                563 | 564 => {
                    self.handle_tls_descriptor(
                        section,
                        Expression::TlsDescriptor,
                        Expression::Got,
                        kind,
                        offset,
                        addend,
                        symbol,
                    );
                    continue;
                }
                569 => {
                    if !self.is_shared() {
                        self.add_relocation(
                            section,
                            Expression::ThreadPointerRelative,
                            kind,
                            offset,
                            addend,
                            symbol,
                        );
                    }
                    continue;
                }
                _ => {
                    self.reject_unknown_relocation(section, offset, kind, symbol);
                    continue;
                }
            };
            self.process(section, expression, kind, offset, symbol, addend);
        }
    }

    pub fn relocate_aarch64(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        relocation: &Relocation,
        value: u64,
        location: &str,
    ) {
        let encoding = self.encoding();
        let instruction =
            |buffer: &[u8]| u32::from_le_bytes(buffer[at..at + 4].try_into().unwrap_or([0; 4]));
        let store =
            |buffer: &mut [u8], word: u32| buffer[at..at + 4].copy_from_slice(&word.to_le_bytes());
        let masked = |buffer: &mut [u8], bits: u32, mask: u32| {
            let word = u32::from_le_bytes(buffer[at..at + 4].try_into().unwrap_or([0; 4]));
            buffer[at..at + 4].copy_from_slice(&((word & !mask) | bits).to_le_bytes());
        };
        let address = |buffer: &mut [u8], immediate: u64| {
            let low = ((immediate & 0x3) << 29) as u32;
            let high = ((immediate & 0x1f_fffc) << 3) as u32;
            let mask = (0x3u32 << 29) | (0x1f_fffcu32 << 3);
            let word = u32::from_le_bytes(buffer[at..at + 4].try_into().unwrap_or([0; 4]));
            buffer[at..at + 4].copy_from_slice(&((word & !mask) | low | high).to_le_bytes());
        };
        let immediate12 = |buffer: &mut [u8], immediate: u64| {
            let word = u32::from_le_bytes(buffer[at..at + 4].try_into().unwrap_or([0; 4]));
            buffer[at..at + 4].copy_from_slice(
                &((word & !(0xfff << 10)) | (((immediate & 0xfff) as u32) << 10)).to_le_bytes(),
            );
        };
        let signed_move = |buffer: &mut [u8], mut immediate: u32| {
            let mut word = u32::from_le_bytes(buffer[at..at + 4].try_into().unwrap_or([0; 4]));
            if word & (1 << 29) == 0 {
                if immediate & 0x10000 != 0 {
                    immediate ^= 0xffff;
                    word &= !(1 << 30);
                } else {
                    word |= 1 << 30;
                }
            }
            buffer[at..at + 4].copy_from_slice(&(word | ((immediate & 0xffff) << 5)).to_le_bytes());
        };
        let bits =
            |value: u64, start: u32, end: u32| (value >> start) & ((1u64 << (end + 1 - start)) - 1);
        match relocation.kind {
            259 => {
                self.check_either(location, relocation, value, 16);
                encoding.write_u16(buffer, at, value as u16);
            }
            262 => {
                self.check_signed(location, relocation, value, 16);
                encoding.write_u16(buffer, at, value as u16);
            }
            258 => {
                self.check_either(location, relocation, value, 32);
                encoding.write_u32(buffer, at, value as u32);
            }
            261 | 314 | 315 => {
                self.check_signed(location, relocation, value, 32);
                encoding.write_u32(buffer, at, value as u32);
            }
            257 | 260 | 1029 => encoding.write_u64(buffer, at, value),
            277 => immediate12(buffer, value),
            311 | 275 | 541 | 562 => {
                self.check_signed(location, relocation, value, 33);
                address(buffer, value >> 12);
            }
            276 => address(buffer, value >> 12),
            274 => {
                self.check_signed(location, relocation, value, 21);
                address(buffer, value);
            }
            282 | 283 => {
                if relocation.kind == 282 {
                    store(buffer, 0x1400_0000);
                }
                self.check_signed(location, relocation, value, 28);
                masked(
                    buffer,
                    ((value & 0x0fff_fffc) >> 2) as u32,
                    0x0fff_fffc >> 2,
                );
            }
            280 | 273 | 309 => {
                self.check_alignment(location, relocation, value, 4);
                self.check_signed(location, relocation, value, 21);
                masked(buffer, ((value & 0x1f_fffc) << 3) as u32, 0x1f_fffc << 3);
            }
            278 | 553 => immediate12(buffer, bits(value, 0, 11)),
            284 | 555 => {
                self.check_alignment(location, relocation, value, 2);
                immediate12(buffer, bits(value, 1, 11));
            }
            285 | 557 => {
                self.check_alignment(location, relocation, value, 4);
                immediate12(buffer, bits(value, 2, 11));
            }
            286 | 312 | 542 | 559 | 563 => {
                self.check_alignment(location, relocation, value, 8);
                immediate12(buffer, bits(value, 3, 11));
            }
            299 | 571 => {
                self.check_alignment(location, relocation, value, 16);
                immediate12(buffer, bits(value, 4, 11));
            }
            313 => {
                self.check_alignment(location, relocation, value, 8);
                immediate12(buffer, bits(value, 3, 14));
            }
            263 | 264 => {
                if relocation.kind == 263 {
                    self.check_unsigned(location, relocation, value, 16);
                }
                masked(buffer, ((value & 0xffff) << 5) as u32, 0xffff << 5);
            }
            265 | 266 => {
                if relocation.kind == 265 {
                    self.check_unsigned(location, relocation, value, 32);
                }
                masked(
                    buffer,
                    ((value & 0xffff_0000) >> 11) as u32,
                    0xffff_0000u32 >> 11,
                );
            }
            267 | 268 => {
                if relocation.kind == 267 {
                    self.check_unsigned(location, relocation, value, 48);
                }
                masked(
                    buffer,
                    ((value & 0xffff_0000_0000) >> 27) as u32,
                    (0xffff_0000_0000u64 >> 27) as u32,
                );
            }
            269 => masked(
                buffer,
                ((value & 0xffff_0000_0000_0000) >> 43) as u32,
                (0xffff_0000_0000_0000u64 >> 43) as u32,
            ),
            287 | 270 | 547 | 288 | 548 => {
                if matches!(relocation.kind, 287 | 270 | 547) {
                    self.check_signed(location, relocation, value, 17);
                }
                signed_move(buffer, value as u32);
            }
            289 | 271 | 545 | 290 | 546 => {
                if matches!(relocation.kind, 289 | 271 | 545) {
                    self.check_signed(location, relocation, value, 33);
                }
                signed_move(buffer, (value >> 16) as u32);
            }
            291 | 272 | 544 | 292 => {
                if matches!(relocation.kind, 291 | 272 | 544) {
                    self.check_signed(location, relocation, value, 49);
                }
                signed_move(buffer, (value >> 32) as u32);
            }
            293 => signed_move(buffer, (value >> 48) as u32),
            279 => {
                self.check_signed(location, relocation, value, 16);
                masked(buffer, ((value & 0xfffc) << 3) as u32, 0xfffc << 3);
            }
            549 => {
                self.check_unsigned(location, relocation, value, 24);
                if self.options.relax && (value >> 12) == 0 {
                    let word = instruction(buffer);
                    if word & (1 << 31) != 0 && (word & 0x1f) == ((word >> 5) & 0x1f) {
                        store(buffer, 0xd503_201f);
                        return;
                    }
                }
                immediate12(buffer, value >> 12);
            }
            551 | 564 => immediate12(buffer, value),
            1031 => encoding.write_u64(buffer, at + 8, value),
            _ => {}
        }
    }

    pub fn check_alignment(
        &mut self,
        location: &str,
        relocation: &Relocation,
        value: u64,
        alignment: u64,
    ) {
        if value & (alignment - 1) != 0 {
            let name = relocation_name(self.target.architecture, relocation.kind);
            self.error(
                problem(
                    "EI234",
                    "A relocation value is not aligned the way its field needs.",
                )
                .with_context(format!("Location: {location}"))
                .with_context(format!("Relocation: {name}"))
                .with_context(format!("Value: 0x{value:x}"))
                .with_context(format!("Alignment: {alignment}")),
            );
        }
    }

    fn aarch64_relocate_section(&mut self, section: SectionIndex, buffer: &mut [u8]) {
        let relocations = std::mem::take(&mut self.sections[section].relocations);
        let base = self.section_address(section, 0);
        let mut unsafe_symbols: BTreeSet<SymbolIndex> = BTreeSet::new();
        if self.options.relax {
            let mut index = 0;
            while index < relocations.len() {
                let relocation = &relocations[index];
                if relocation.kind == 311 {
                    if index + 1 < relocations.len()
                        && relocations[index + 1].kind == 312
                        && !unsafe_symbols.contains(&relocation.symbol)
                        && self.aarch64_legal_page_load(
                            relocation,
                            &relocations[index + 1],
                            base,
                            buffer,
                        )
                    {
                        index += 2;
                        continue;
                    }
                    unsafe_symbols.insert(relocation.symbol);
                } else if relocation.kind == 312 {
                    unsafe_symbols.insert(relocation.symbol);
                }
                index += 1;
            }
        }
        let mut index = 0;
        while index < relocations.len() {
            let relocation = relocations[index].clone();
            index += 1;
            if relocation.expression == Expression::None {
                continue;
            }
            let at = relocation.offset as usize;
            if at + 4 > buffer.len() && !matches!(relocation.kind, 257..=262) {
                continue;
            }
            let place = base.wrapping_add(relocation.offset);
            let value = self.evaluate(&relocation, place);
            let location = self.error_location(section, relocation.offset);
            match relocation.kind {
                311 => {
                    if index < relocations.len()
                        && self.options.relax
                        && relocations[index].kind == 312
                        && !unsafe_symbols.contains(&relocation.symbol)
                    {
                        let next = relocations[index].clone();
                        self.aarch64_relax_page_load(&relocation, &next, base, buffer, &location);
                        index += 1;
                        continue;
                    }
                }
                275 => {
                    if index < relocations.len() {
                        let next = relocations[index].clone();
                        if self.aarch64_relax_page_add(&relocation, &next, base, buffer, &location)
                        {
                            index += 1;
                            continue;
                        }
                    }
                }
                562 | 563 | 564 | 569 => {
                    if relocation.expression == Expression::ThreadPointerRelative {
                        self.check_unsigned(&location, &relocation, value, 32);
                        let word: u32 = match relocation.kind {
                            564 | 569 => 0xd503_201f,
                            562 => 0xd2a0_0000 | ((((value >> 16) & 0xffff) as u32) << 5),
                            _ => 0xf280_0000 | (((value & 0xffff) as u32) << 5),
                        };
                        buffer[at..at + 4].copy_from_slice(&word.to_le_bytes());
                        continue;
                    }
                    if matches!(
                        relocation.expression,
                        Expression::Aarch64GotPageProgramCounter | Expression::Got
                    ) {
                        match relocation.kind {
                            564 | 569 => {
                                buffer[at..at + 4].copy_from_slice(&0xd503_201fu32.to_le_bytes())
                            }
                            562 => {
                                buffer[at..at + 4].copy_from_slice(&0x9000_0000u32.to_le_bytes());
                                let relaxed = Relocation {
                                    kind: 541,
                                    ..relocation.clone()
                                };
                                self.relocate_aarch64(buffer, at, &relaxed, value, &location);
                            }
                            _ => {
                                buffer[at..at + 4].copy_from_slice(&0xf940_0000u32.to_le_bytes());
                                let relaxed = Relocation {
                                    kind: 542,
                                    ..relocation.clone()
                                };
                                self.relocate_aarch64(buffer, at, &relaxed, value, &location);
                            }
                        }
                        continue;
                    }
                }
                541 | 542 if relocation.expression == Expression::ThreadPointerRelative => {
                    self.check_unsigned(&location, &relocation, value, 32);
                    let register =
                        u32::from_le_bytes(buffer[at..at + 4].try_into().unwrap_or([0; 4])) & 0x1f;
                    let word = if relocation.kind == 541 {
                        (0xd2a0_0000 | register) | ((((value >> 16) & 0xffff) as u32) << 5)
                    } else {
                        (0xf280_0000 | register) | (((value & 0xffff) as u32) << 5)
                    };
                    buffer[at..at + 4].copy_from_slice(&word.to_le_bytes());
                    continue;
                }
                _ => {}
            }
            self.relocate_aarch64(buffer, at, &relocation, value, &location);
        }
        self.sections[section].relocations = relocations;
    }

    fn read_le32(buffer: &[u8], at: usize) -> u32 {
        u32::from_le_bytes(
            buffer
                .get(at..at + 4)
                .and_then(|bytes| bytes.try_into().ok())
                .unwrap_or([0; 4]),
        )
    }

    fn aarch64_legal_page_load(
        &self,
        page: &Relocation,
        load: &Relocation,
        base: u64,
        buffer: &[u8],
    ) -> bool {
        if page.offset + 4 != load.offset || page.symbol != load.symbol {
            return false;
        }
        let symbol = &self.symbols[page.symbol];
        if !symbol.is_defined() || symbol.preemptible || symbol.is_indirect_function() {
            return false;
        }
        if page.addend != 0 || load.addend != 0 {
            return false;
        }
        let page_word = Self::read_le32(buffer, page.offset as usize);
        let load_word = Self::read_le32(buffer, load.offset as usize);
        if (page_word & 0x9f00_0000) != 0x9000_0000 || (load_word & 0x3b00_0000) != 0x3900_0000 {
            return false;
        }
        if load_word >> 31 == 0 {
            return false;
        }
        let destination = page_word & 0x1f;
        if destination != load_word & 0x1f || destination != (load_word >> 5) & 0x1f {
            return false;
        }
        if self.is_pic() && self.is_absolute(page.symbol) {
            return false;
        }
        let value = aarch64_page(self.symbol_address(page.symbol, 0))
            .wrapping_sub(aarch64_page(base.wrapping_add(page.offset))) as i64;
        value == sign_extend(value as u64, 33)
    }

    fn aarch64_relax_page_load(
        &mut self,
        page: &Relocation,
        load: &Relocation,
        base: u64,
        buffer: &mut [u8],
        location: &str,
    ) {
        let page_at = page.offset as usize;
        let load_at = load.offset as usize;
        let register = Self::read_le32(buffer, page_at) & 0x1f;
        let symbol = page.symbol;
        let page_relocation = Relocation {
            expression: Expression::Aarch64PageProgramCounter,
            kind: 275,
            offset: page.offset,
            addend: 0,
            symbol,
        };
        let add_relocation = Relocation {
            expression: Expression::Absolute,
            kind: 277,
            offset: load.offset,
            addend: 0,
            symbol,
        };
        buffer[page_at..page_at + 4].copy_from_slice(&(0x9000_0000 | register).to_le_bytes());
        buffer[load_at..load_at + 4]
            .copy_from_slice(&(0x9100_0000 | register | (register << 5)).to_le_bytes());
        let address = self.symbol_address(symbol, 0);
        let page_value =
            aarch64_page(address).wrapping_sub(aarch64_page(base.wrapping_add(page.offset)));
        self.relocate_aarch64(buffer, page_at, &page_relocation, page_value, location);
        self.relocate_aarch64(buffer, load_at, &add_relocation, address, location);
        self.aarch64_relax_page_add(&page_relocation, &add_relocation, base, buffer, location);
    }

    fn aarch64_relax_page_add(
        &mut self,
        page: &Relocation,
        add: &Relocation,
        base: u64,
        buffer: &mut [u8],
        location: &str,
    ) -> bool {
        if !self.options.relax || add.kind != 277 {
            return false;
        }
        if page.offset + 4 != add.offset
            || page.symbol != add.symbol
            || page.addend != 0
            || add.addend != 0
        {
            return false;
        }
        let page_word = Self::read_le32(buffer, page.offset as usize);
        let add_word = Self::read_le32(buffer, add.offset as usize);
        if (page_word & 0x9f00_0000) != 0x9000_0000 || (add_word & 0xffc0_0000) != 0x9100_0000 {
            return false;
        }
        let destination = page_word & 0x1f;
        if destination != add_word & 0x1f || destination != (add_word >> 5) & 0x1f {
            return false;
        }
        let value = self
            .symbol_address(page.symbol, 0)
            .wrapping_sub(base.wrapping_add(add.offset)) as i64;
        if !(-1024 * 1024..1024 * 1024).contains(&value) {
            return false;
        }
        let page_at = page.offset as usize;
        let add_at = add.offset as usize;
        buffer[page_at..page_at + 4].copy_from_slice(&0xd503_201fu32.to_le_bytes());
        buffer[add_at..add_at + 4].copy_from_slice(&(0x1000_0000 | destination).to_le_bytes());
        let relocation = Relocation {
            expression: Expression::Absolute,
            kind: 274,
            offset: add.offset,
            addend: 0,
            symbol: page.symbol,
        };
        self.relocate_aarch64(buffer, add_at, &relocation, value as u64, location);
        true
    }

    fn scan_arm(
        &mut self,
        section: SectionIndex,
        relocations: &[RawRelocation],
        undefined: &mut Vec<UndefinedReference>,
    ) {
        for relocation in relocations {
            if relocation.kind == 0 {
                continue;
            }
            let Some(symbol) = self.next_symbol(section, relocation, undefined) else {
                continue;
            };
            let kind = relocation.kind;
            let offset = relocation.offset;
            let addend = relocation.addend;
            let expression = match kind {
                40 => continue,
                2 | 43 | 44 | 47 | 48 | 132 | 133 | 134 | 135 => Expression::Absolute,
                103 | 102 | 45 | 46 | 3 | 49 | 50 => {
                    self.process_program_counter(section, kind, offset, addend, symbol);
                    continue;
                }
                58 | 57 | 60 | 59 | 61 | 4 | 62 | 63 | 64 | 65 | 66 | 53 | 11 | 54 => {
                    Expression::ArmProgramCounterAligned
                }
                28 | 29 | 1 | 27 | 42 | 51 | 30 | 10 => {
                    self.process_plt_program_counter(section, kind, offset, addend, symbol);
                    continue;
                }
                26 => Expression::GotOffset,
                96 => Expression::GotProgramCounter,
                24 => {
                    self.has_got_offset_relocation = true;
                    Expression::GotRelative
                }
                25 => {
                    self.has_got_offset_relocation = true;
                    Expression::GotOnlyProgramCounter
                }
                9 | 84 | 86 | 85 | 87 | 89 | 88 => Expression::ArmStaticBaseRelative,
                38 => {
                    if self.options.arm_target1_relative {
                        Expression::ProgramCounter
                    } else {
                        Expression::Absolute
                    }
                }
                41 => match self.options.arm_target2.as_str() {
                    "rel" => Expression::ProgramCounter,
                    "abs" => Expression::Absolute,
                    _ => Expression::GotProgramCounter,
                },
                108 => {
                    if self.check_tls_local_executable(section, offset, symbol, kind) {
                        continue;
                    }
                    Expression::ThreadPointerRelative
                }
                107 => {
                    self.handle_tls_initial_executable(
                        section,
                        Expression::GotProgramCounter,
                        kind,
                        offset,
                        addend,
                        symbol,
                        false,
                    );
                    continue;
                }
                104 => {
                    self.handle_tls_general_dynamic(
                        section,
                        Expression::TlsGeneralDynamicProgramCounter,
                        Expression::None,
                        Expression::None,
                        kind,
                        offset,
                        addend,
                        symbol,
                    );
                    continue;
                }
                105 => {
                    self.needs_tls_local_dynamic = true;
                    self.add_relocation(
                        section,
                        Expression::TlsLocalDynamicProgramCounter,
                        kind,
                        offset,
                        addend,
                        symbol,
                    );
                    continue;
                }
                106 => Expression::DynamicThreadRelative,
                _ => {
                    self.reject_unknown_relocation(section, offset, kind, symbol);
                    continue;
                }
            };
            self.process(section, expression, kind, offset, symbol, addend);
        }
    }

    fn arm_read16(&self, buffer: &[u8], at: usize) -> u32 {
        u32::from(self.encoding().read_u16(buffer, at).unwrap_or(0))
    }

    fn arm_write16(&self, buffer: &mut [u8], at: usize, value: u32) {
        self.encoding().write_u16(buffer, at, value as u16);
    }

    fn arm_read32(&self, buffer: &[u8], at: usize) -> u32 {
        self.encoding().read_u32(buffer, at).unwrap_or(0)
    }

    fn arm_write32(&self, buffer: &mut [u8], at: usize, value: u32) {
        self.encoding().write_u32(buffer, at, value);
    }

    fn arm_state_change_warning(&mut self, location: &str, relocation: &Relocation) {
        let symbol = &self.symbols[relocation.symbol];
        let name = relocation_name(Architecture::Arm, relocation.kind);
        let mut diagnostic = caution(
            "EI253",
            "A branch and link relocation does not switch between ARM and Thumb state.",
        )
        .with_context(format!("Location: {location}"))
        .with_context(format!("Relocation: {name}"));
        if symbol.is_section() {
            if let Some(section) = symbol.section {
                diagnostic =
                    diagnostic.with_context(format!("Section: {}", self.sections[section].name));
            }
        } else {
            diagnostic = diagnostic.with_context(format!("Symbol: {}", symbol.name));
        }
        self.warn(diagnostic);
    }

    fn arm_group_remainder(group: u32, mut value: u32) -> (u32, u32) {
        let mut remainder;
        let mut leading;
        let mut remaining = group as i64;
        loop {
            leading = value.leading_zeros() & !1;
            remainder = value;
            if leading == 32 {
                break;
            }
            value &= 0x00ff_ffff >> leading;
            if remaining == 0 {
                break;
            }
            remaining -= 1;
        }
        (remainder, leading)
    }

    pub fn relocate_arm(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        relocation: &Relocation,
        value: u64,
        location: &str,
    ) {
        let symbol_is_function = self.symbols[relocation.symbol].is_function();
        let in_plt = self.symbols[relocation.symbol].plt_index.is_some();
        match relocation.kind {
            2 | 25 | 24 | 26 | 96 | 3 | 23 | 9 | 38 | 41 | 104 | 107 | 105 | 106 | 108 | 19
            | 18 => self.arm_write32(buffer, at, value as u32),
            42 => {
                self.check_signed(location, relocation, value, 31);
                let word = self.arm_read32(buffer, at);
                self.arm_write32(
                    buffer,
                    at,
                    (word & 0x8000_0000) | (value as u32 & !0x8000_0000),
                );
            }
            28 | 29 | 1 | 27 => {
                if relocation.kind == 28 {
                    let thumb_target = value & 1 != 0;
                    let is_exchange = (self.arm_read32(buffer, at) & 0xfe00_0000) == 0xfa00_0000;
                    if !symbol_is_function && is_exchange != thumb_target {
                        self.arm_state_change_warning(location, relocation);
                    }
                    if if symbol_is_function {
                        thumb_target
                    } else {
                        is_exchange
                    } {
                        self.check_signed(location, relocation, value, 26);
                        let word = 0xfa00_0000
                            | (((value & 2) as u32) << 23)
                            | ((value >> 2) as u32 & 0x00ff_ffff);
                        self.arm_write32(buffer, at, word);
                        return;
                    }
                    let word = self.arm_read32(buffer, at);
                    self.arm_write32(buffer, at, 0xeb00_0000 | (word & 0x00ff_ffff));
                }
                self.check_signed(location, relocation, value, 26);
                let word = self.arm_read32(buffer, at);
                self.arm_write32(
                    buffer,
                    at,
                    (word & !0x00ff_ffff) | ((value >> 2) as u32 & 0x00ff_ffff),
                );
            }
            103 => {
                self.check_signed(location, relocation, value, 9);
                let half = self.arm_read16(buffer, at);
                self.arm_write16(buffer, at, (half & 0xff00) | ((value >> 1) as u32 & 0x00ff));
            }
            102 => {
                self.check_signed(location, relocation, value, 12);
                let half = self.arm_read16(buffer, at);
                self.arm_write16(buffer, at, (half & 0xf800) | ((value >> 1) as u32 & 0x07ff));
            }
            51 => {
                self.check_signed(location, relocation, value, 21);
                let v = value as u32;
                let half = self.arm_read16(buffer, at);
                self.arm_write16(
                    buffer,
                    at,
                    (half & 0xfbc0) | ((v >> 10) & 0x0400) | ((v >> 12) & 0x003f),
                );
                self.arm_write16(
                    buffer,
                    at + 2,
                    0x8000 | ((v >> 8) & 0x0800) | ((v >> 5) & 0x2000) | ((v >> 1) & 0x07ff),
                );
            }
            10 | 30 => {
                let mut value = value;
                if relocation.kind == 10 {
                    let thumb_target = value & 1 != 0;
                    let use_thumb = thumb_target || self.use_thumb_plt();
                    let is_exchange = (self.arm_read16(buffer, at + 2) & 0x1000) == 0;
                    if !symbol_is_function && !in_plt && is_exchange == use_thumb {
                        self.arm_state_change_warning(location, relocation);
                    }
                    let low = self.arm_read16(buffer, at + 2);
                    if if symbol_is_function || in_plt {
                        !use_thumb
                    } else {
                        is_exchange
                    } {
                        value = align_to(value, 4);
                        self.arm_write16(buffer, at + 2, low & !0x1000);
                    } else {
                        self.arm_write16(buffer, at + 2, (low & !0x1000) | (1 << 12));
                    }
                    if !self.arm_features.j1_j2_branch_encoding {
                        self.check_signed(location, relocation, value, 23);
                        let v = value as u32;
                        self.arm_write16(buffer, at, 0xf000 | ((v >> 12) & 0x07ff));
                        let low = self.arm_read16(buffer, at + 2);
                        self.arm_write16(
                            buffer,
                            at + 2,
                            (low & 0xd000) | 0x2800 | ((v >> 1) & 0x07ff),
                        );
                        return;
                    }
                }
                self.check_signed(location, relocation, value, 25);
                let v = value as u32;
                self.arm_write16(
                    buffer,
                    at,
                    0xf000 | ((v >> 14) & 0x0400) | ((v >> 12) & 0x03ff),
                );
                let low = self.arm_read16(buffer, at + 2);
                self.arm_write16(
                    buffer,
                    at + 2,
                    (low & 0xd000)
                        | (((!(v >> 10)) ^ (v >> 11)) & 0x2000)
                        | (((!(v >> 11)) ^ (v >> 13)) & 0x0800)
                        | ((v >> 1) & 0x07ff),
                );
            }
            43 | 45 | 84 => {
                let word = self.arm_read32(buffer, at);
                let v = value as u32;
                self.arm_write32(
                    buffer,
                    at,
                    (word & !0x000f_0fff) | ((v & 0xf000) << 4) | (v & 0x0fff),
                );
            }
            44 | 46 | 85 => {
                let word = self.arm_read32(buffer, at);
                let v = value as u32;
                self.arm_write32(
                    buffer,
                    at,
                    (word & !0x000f_0fff) | (((v >> 16) & 0xf000) << 4) | ((v >> 16) & 0xfff),
                );
            }
            48 | 50 | 88 => {
                let v = value as u32;
                self.arm_write16(
                    buffer,
                    at,
                    0xf2c0 | ((v >> 17) & 0x0400) | ((v >> 28) & 0x000f),
                );
                let low = self.arm_read16(buffer, at + 2);
                self.arm_write16(
                    buffer,
                    at + 2,
                    (low & 0x8f00) | ((v >> 12) & 0x7000) | ((v >> 16) & 0x00ff),
                );
            }
            47 | 49 | 87 => {
                let v = value as u32;
                self.arm_write16(
                    buffer,
                    at,
                    0xf240 | ((v >> 1) & 0x0400) | ((v >> 12) & 0x000f),
                );
                let low = self.arm_read16(buffer, at + 2);
                self.arm_write16(
                    buffer,
                    at + 2,
                    (low & 0x8f00) | ((v << 4) & 0x7000) | (v & 0x00ff),
                );
            }
            132..=135 => {
                let shift = match relocation.kind {
                    135 => 24,
                    134 => 16,
                    133 => 8,
                    _ => 0,
                };
                let half = self.arm_read16(buffer, at);
                self.arm_write16(
                    buffer,
                    at,
                    (half & !0x00ff) | ((value >> shift) as u32 & 0x00ff),
                );
            }
            57..=61 => {
                let (group, check) = match relocation.kind {
                    58 => (0, true),
                    57 => (0, false),
                    60 => (1, true),
                    59 => (1, false),
                    _ => (2, true),
                };
                let mut value = value;
                let mut operation = 0x0080_0000u32;
                if value >> 63 != 0 {
                    operation = 0x0040_0000;
                    value = value.wrapping_neg();
                }
                let (mut immediate, leading) = Self::arm_group_remainder(group, value as u32);
                let mut rotation = 0u32;
                if leading < 24 {
                    immediate = immediate.rotate_right(24 - leading);
                    rotation = (leading + 8) << 7;
                }
                if check && immediate > 0xff {
                    let name = relocation_name(Architecture::Arm, relocation.kind);
                    self.error(
                        problem(
                            "EI235",
                            "A relocation value cannot be encoded as an immediate.",
                        )
                        .with_context(format!("Location: {location}"))
                        .with_context(format!("Value: {value}"))
                        .with_context(format!("Relocation: {name}")),
                    );
                }
                let word = self.arm_read32(buffer, at);
                self.arm_write32(
                    buffer,
                    at,
                    (word & 0xff3f_f000) | operation | rotation | (immediate & 0xff),
                );
            }
            4 | 62 | 63 | 64 | 65 | 66 => {
                let group = match relocation.kind {
                    4 | 64 => 0,
                    62 | 65 => 1,
                    _ => 2,
                };
                let mut value = value;
                if symbol_is_function {
                    value &= !1;
                }
                let mut operation = 0x0080_0000u32;
                if value >> 63 != 0 {
                    operation = 0;
                    value = value.wrapping_neg();
                }
                let immediate = Self::arm_group_remainder(group, value as u32).0;
                let word = self.arm_read32(buffer, at);
                if matches!(relocation.kind, 4 | 62 | 63) {
                    self.check_unsigned(location, relocation, u64::from(immediate), 12);
                    self.arm_write32(buffer, at, (word & 0xff7f_f000) | operation | immediate);
                } else {
                    self.check_unsigned(location, relocation, u64::from(immediate), 8);
                    self.arm_write32(
                        buffer,
                        at,
                        (word & 0xff7f_f0f0)
                            | operation
                            | ((immediate & 0xf0) << 4)
                            | (immediate & 0xf),
                    );
                }
            }
            53 => {
                let mut immediate = value as i64;
                let mut subtract = 0u32;
                if immediate < 0 {
                    immediate = -immediate;
                    subtract = 0x00a0;
                }
                self.check_unsigned(location, relocation, immediate as u64, 12);
                let immediate = immediate as u32;
                let half = self.arm_read16(buffer, at);
                self.arm_write16(
                    buffer,
                    at,
                    (half & 0xfb0f) | subtract | ((immediate & 0x800) >> 1),
                );
                let low = self.arm_read16(buffer, at + 2);
                self.arm_write16(
                    buffer,
                    at + 2,
                    (low & 0x8f00) | ((immediate & 0x700) << 4) | (immediate & 0xff),
                );
            }
            11 => {
                let mut value = value;
                if symbol_is_function {
                    value &= !1;
                }
                self.check_unsigned(location, relocation, value, 10);
                self.check_alignment(location, relocation, value, 4);
                let half = self.arm_read16(buffer, at);
                self.arm_write16(buffer, at, (half & 0xff00) | ((value as u32 & 0x3fc) >> 2));
            }
            54 => {
                let mut value = value;
                if symbol_is_function {
                    value &= !1;
                }
                let mut immediate = value as i64;
                let mut up = 0x0080u32;
                if immediate < 0 {
                    immediate = -immediate;
                    up = 0;
                }
                self.check_unsigned(location, relocation, immediate as u64, 12);
                let half = self.arm_read16(buffer, at);
                self.arm_write16(buffer, at, half | up);
                let low = self.arm_read16(buffer, at + 2);
                self.arm_write16(buffer, at + 2, (low & 0xf000) | immediate as u32);
            }
            _ => {}
        }
    }

    pub fn use_thumb_plt(&self) -> bool {
        self.arm_features.has_thumb2_instruction_set && !self.arm_features.has_arm_instruction_set
    }

    fn relocate(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        relocation: &Relocation,
        value: u64,
        location: &str,
    ) {
        match self.target.architecture {
            Architecture::X86_64 => self.relocate_x86_64(buffer, at, relocation, value, location),
            Architecture::X86 => self.relocate_x86(buffer, at, relocation, value, location),
            Architecture::Aarch64 => self.relocate_aarch64(buffer, at, relocation, value, location),
            Architecture::Arm => self.relocate_arm(buffer, at, relocation, value, location),
        }
    }

    fn apply_relocations(&mut self, section: SectionIndex, buffer: &mut [u8]) {
        if self.target.architecture == Architecture::Aarch64
            && !self.sections[section].is_exception_frame
        {
            self.aarch64_relocate_section(section, buffer);
            return;
        }
        let relocations = std::mem::take(&mut self.sections[section].relocations);
        let base = self.section_address(section, 0);
        for relocation in &relocations {
            if relocation.expression == Expression::None {
                continue;
            }
            let at = relocation.offset as usize;
            let place = if self.sections[section].is_exception_frame {
                self.section_address(section, relocation.offset)
            } else {
                base.wrapping_add(relocation.offset)
            };
            let value = self.evaluate(relocation, place);
            if at >= buffer.len() {
                continue;
            }
            let location = self.error_location(section, relocation.offset);
            self.relocate(buffer, at, relocation, value, &location);
        }
        self.sections[section].relocations = relocations;
    }

    fn apply_non_allocated_relocations(&mut self, section: SectionIndex, buffer: &mut [u8]) {
        let raw = self.raw_relocations(section);
        let name = self.sections[section].name.clone();
        let debug = is_debug_section(&name);
        let tombstone: u64 = if name == ".debug_loc" || name == ".debug_ranges" {
            1
        } else {
            0
        };
        let base = self.section_address(section, 0);
        for entry in raw {
            let Some(symbol) = self.symbol_of(section, entry.symbol) else {
                continue;
            };
            if entry.kind == 0 {
                continue;
            }
            let relocation = Relocation {
                expression: Expression::Absolute,
                kind: entry.kind,
                offset: entry.offset,
                addend: entry.addend,
                symbol,
            };
            let defined_dead = self.symbols[symbol]
                .section
                .is_some_and(|held| !self.sections[held].live)
                || (self.symbols[symbol].is_undefined()
                    && self.symbols[symbol].discarded_section != 0);
            let value = if defined_dead && debug {
                tombstone
            } else {
                match self.non_allocated_expression(entry.kind) {
                    Some(Expression::Size) => {
                        self.symbols[symbol].size.wrapping_add(entry.addend as u64)
                    }
                    Some(Expression::ProgramCounter) => self
                        .symbol_address(symbol, entry.addend)
                        .wrapping_sub(base.wrapping_add(entry.offset)),
                    Some(Expression::DynamicThreadRelative) => {
                        self.symbol_address(symbol, entry.addend)
                    }
                    Some(_) => self.symbol_address(symbol, entry.addend),
                    None => continue,
                }
            };
            let at = entry.offset as usize;
            if at >= buffer.len() {
                continue;
            }
            let location = self.error_location(section, entry.offset);
            let bits = self.word_size() * 8;
            let value = if bits == 32 {
                value & 0xffff_ffff
            } else {
                value
            };
            self.relocate(buffer, at, &relocation, value, &location);
        }
    }

    fn non_allocated_expression(&self, kind: u32) -> Option<Expression> {
        match self.target.architecture {
            Architecture::X86_64 => match kind {
                1 | 10 | 11 | 12 | 14 => Some(Expression::Absolute),
                2 | 13 | 15 | 24 => Some(Expression::ProgramCounter),
                17 | 21 => Some(Expression::DynamicThreadRelative),
                32 | 33 => Some(Expression::Size),
                25 => Some(Expression::GotPltRelative),
                _ => None,
            },
            _ => Some(Expression::Absolute),
        }
    }

    fn write_got_plt(&self, buffer: &mut [u8]) {
        let entry = self.target.got_entry_size as usize;
        if matches!(
            self.target.architecture,
            Architecture::X86_64 | Architecture::X86
        ) {
            let dynamic = self
                .synthetic_section(".dynamic")
                .filter(|section| {
                    self.sections[*section].live && self.sections[*section].output.is_some()
                })
                .map(|section| self.section_address(section, 0))
                .unwrap_or(0);
            if buffer.len() >= entry {
                self.write_word(buffer, 0, dynamic);
            }
        }
        let plt = self.synthetic_address(".plt");
        for (position, id) in self.got_plt_entries.iter().enumerate() {
            let at = (position + self.target.got_plt_header_entries as usize) * entry;
            let value = match self.target.architecture {
                Architecture::X86_64 | Architecture::X86 => self.plt_address(*id) + 6,
                _ => plt,
            };
            if at + entry <= buffer.len() {
                self.write_word(buffer, at, value);
            }
        }
    }

    fn write_indirect_got_plt(&self, buffer: &mut [u8]) {
        let entry = self.target.got_entry_size as usize;
        let writes = match self.target.architecture {
            Architecture::X86 | Architecture::Arm => true,
            _ => self.writes_addends(),
        };
        if !writes {
            return;
        }
        for (position, id) in self.indirect_got_plt_entries.iter().enumerate() {
            let at = position * entry;
            if at + entry <= buffer.len() {
                self.write_word(buffer, at, self.symbol_address(*id, 0));
            }
        }
    }

    fn write_plt(&mut self, buffer: &mut [u8]) {
        let plt = self.synthetic_address(".plt");
        let got_plt = self.synthetic_address(".got.plt");
        let header = self.target.plt_header_size as usize;
        let size = self.target.plt_entry_size as usize;
        if buffer.len() >= header {
            self.write_plt_header(&mut buffer[..header], plt, got_plt);
        }
        for (position, id) in self.plt_entries.clone().into_iter().enumerate() {
            let at = header + position * size;
            let entry_address = plt + at as u64;
            self.write_plt_entry(
                &mut buffer[at..at + size],
                id,
                entry_address,
                position as u64,
            );
        }
    }

    fn write_plt_header(&self, out: &mut [u8], plt: u64, got_plt: u64) {
        let encoding = self.encoding();
        match self.target.architecture {
            Architecture::X86_64 => {
                out.copy_from_slice(&[
                    0xff, 0x35, 0, 0, 0, 0, 0xff, 0x25, 0, 0, 0, 0, 0x0f, 0x1f, 0x40, 0x00,
                ]);
                out[2..6].copy_from_slice(
                    &((got_plt.wrapping_sub(plt).wrapping_add(2)) as u32).to_le_bytes(),
                );
                out[8..12].copy_from_slice(
                    &((got_plt.wrapping_sub(plt).wrapping_add(4)) as u32).to_le_bytes(),
                );
            }
            Architecture::X86 => {
                if self.is_pic() {
                    out.copy_from_slice(&[
                        0xff, 0xb3, 0x04, 0, 0, 0, 0xff, 0xa3, 0x08, 0, 0, 0, 0x90, 0x90, 0x90,
                        0x90,
                    ]);
                } else {
                    out.copy_from_slice(&[
                        0xff, 0x35, 0, 0, 0, 0, 0xff, 0x25, 0, 0, 0, 0, 0x90, 0x90, 0x90, 0x90,
                    ]);
                    out[2..6].copy_from_slice(&((got_plt + 4) as u32).to_le_bytes());
                    out[8..12].copy_from_slice(&((got_plt + 8) as u32).to_le_bytes());
                }
            }
            Architecture::Aarch64 => {
                let words: [u32; 8] = [
                    0xa9bf_7bf0,
                    0x9000_0010,
                    0xf940_0211,
                    0x9100_0210,
                    0xd61f_0220,
                    0xd503_201f,
                    0xd503_201f,
                    0xd503_201f,
                ];
                for (index, word) in words.iter().enumerate() {
                    out[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
                }
                let target = got_plt + 16;
                aarch64_encode_page(
                    out,
                    4,
                    aarch64_page(target).wrapping_sub(aarch64_page(plt + 4)),
                );
                aarch64_encode_low12(out, 8, (target >> 3) & 0x1ff);
                aarch64_encode_low12(out, 12, target & 0xfff);
            }
            Architecture::Arm => {
                let offset = got_plt.wrapping_sub(plt).wrapping_sub(4);
                let trap = u32::from_le_bytes(self.target.trap);
                if offset >= (1 << 27) {
                    let words = [0xe52d_e004u32, 0xe59f_e004, 0xe08f_e00e, 0xe5be_f008];
                    for (index, word) in words.iter().enumerate() {
                        encoding.write_u32(out, index * 4, *word);
                    }
                    encoding.write_u32(
                        out,
                        16,
                        got_plt.wrapping_sub(plt + 8).wrapping_sub(8) as u32,
                    );
                } else {
                    encoding.write_u32(out, 0, 0xe52d_e004);
                    encoding.write_u32(out, 4, 0xe28f_e600 | ((offset >> 20) & 0xff) as u32);
                    encoding.write_u32(out, 8, 0xe28e_ea00 | ((offset >> 12) & 0xff) as u32);
                    encoding.write_u32(out, 12, 0xe5be_f000 | (offset & 0xfff) as u32);
                    encoding.write_u32(out, 16, trap);
                }
                for at in [20, 24, 28] {
                    encoding.write_u32(out, at, trap);
                }
            }
        }
    }

    fn write_plt_entry(&self, out: &mut [u8], id: SymbolIndex, entry_address: u64, index: u64) {
        let encoding = self.encoding();
        let got_plt_entry = self.got_plt_address(id);
        let plt = self.synthetic_address(".plt");
        match self.target.architecture {
            Architecture::X86_64 => {
                out.copy_from_slice(&[0xff, 0x25, 0, 0, 0, 0, 0x68, 0, 0, 0, 0, 0xe9, 0, 0, 0, 0]);
                out[2..6].copy_from_slice(
                    &((got_plt_entry.wrapping_sub(entry_address).wrapping_sub(6)) as u32)
                        .to_le_bytes(),
                );
                out[7..11].copy_from_slice(&(index as u32).to_le_bytes());
                out[12..16].copy_from_slice(
                    &((plt.wrapping_sub(entry_address).wrapping_sub(16)) as u32).to_le_bytes(),
                );
            }
            Architecture::X86 => {
                if self.is_pic() {
                    out.copy_from_slice(&[
                        0xff, 0xa3, 0, 0, 0, 0, 0x68, 0, 0, 0, 0, 0xe9, 0, 0, 0, 0,
                    ]);
                    let got_plt = self.synthetic_address(".got.plt");
                    out[2..6].copy_from_slice(
                        &(got_plt_entry.wrapping_sub(got_plt) as u32).to_le_bytes(),
                    );
                } else {
                    out.copy_from_slice(&[
                        0xff, 0x25, 0, 0, 0, 0, 0x68, 0, 0, 0, 0, 0xe9, 0, 0, 0, 0,
                    ]);
                    out[2..6].copy_from_slice(&(got_plt_entry as u32).to_le_bytes());
                }
                out[7..11].copy_from_slice(&((8 * index) as u32).to_le_bytes());
                out[12..16].copy_from_slice(
                    &((plt.wrapping_sub(entry_address).wrapping_sub(16)) as u32).to_le_bytes(),
                );
            }
            Architecture::Aarch64 => {
                let words: [u32; 4] = [0x9000_0010, 0xf940_0211, 0x9100_0210, 0xd61f_0220];
                for (position, word) in words.iter().enumerate() {
                    out[position * 4..position * 4 + 4].copy_from_slice(&word.to_le_bytes());
                }
                aarch64_encode_page(
                    out,
                    0,
                    aarch64_page(got_plt_entry).wrapping_sub(aarch64_page(entry_address)),
                );
                aarch64_encode_low12(out, 4, (got_plt_entry >> 3) & 0x1ff);
                aarch64_encode_low12(out, 8, got_plt_entry & 0xfff);
            }
            Architecture::Arm => {
                let offset = got_plt_entry.wrapping_sub(entry_address).wrapping_sub(8);
                if offset >= (1 << 27) {
                    encoding.write_u32(out, 0, 0xe59f_c004);
                    encoding.write_u32(out, 4, 0xe08c_c00f);
                    encoding.write_u32(out, 8, 0xe59c_f000);
                    encoding.write_u32(
                        out,
                        12,
                        got_plt_entry
                            .wrapping_sub(entry_address + 4)
                            .wrapping_sub(8) as u32,
                    );
                } else {
                    encoding.write_u32(out, 0, 0xe28f_c600 | ((offset >> 20) & 0xff) as u32);
                    encoding.write_u32(out, 4, 0xe28c_ca00 | ((offset >> 12) & 0xff) as u32);
                    encoding.write_u32(out, 8, 0xe5bc_f000 | (offset & 0xfff) as u32);
                    encoding.write_u32(out, 12, u32::from_le_bytes(self.target.trap));
                }
            }
        }
    }

    fn write_indirect_plt(&mut self, buffer: &mut [u8]) {
        let iplt = self.synthetic_address(".iplt");
        let size = self.target.indirect_plt_entry_size as usize;
        for (position, id) in self.indirect_plt_entries.clone().into_iter().enumerate() {
            let at = position * size;
            let entry_address = iplt + at as u64;
            self.write_plt_entry(
                &mut buffer[at..at + size],
                id,
                entry_address,
                position as u64,
            );
        }
    }

    pub fn finalized_dynamic_relocations(
        &self,
        entries: &[DynamicRelocation],
        combine: bool,
    ) -> Vec<elf::Relocation> {
        let relative_kind = self.target.relative_relocation;
        let indirect_kind = self.target.indirect_relative_relocation;
        let finalize = |entry: &DynamicRelocation| {
            let offset = self.section_address(entry.section, entry.offset);
            let symbol_index = match (entry.use_symbol_value, entry.symbol) {
                (true, Some(id)) => self.symbols[id].dynamic_symbol_index as u32,
                _ => 0,
            };
            let mut addend = self.dynamic_addend(entry);
            if !self.encoding().is_64() {
                addend = sign_extend(addend, 32) as u64;
            }
            elf::Relocation {
                offset,
                kind: entry.kind,
                symbol: symbol_index,
                addend: addend as i64,
            }
        };
        let mut relative: Vec<elf::Relocation> = Vec::new();
        let mut others: Vec<elf::Relocation> = Vec::new();
        for entry in entries {
            let finalized = finalize(entry);
            if entry.kind == relative_kind {
                relative.push(finalized);
            } else {
                others.push(finalized);
            }
        }
        relative.sort_by_key(|entry| entry.offset);
        let (mut plain, indirect): (Vec<_>, Vec<_>) = others
            .into_iter()
            .partition(|entry| entry.kind != indirect_kind);
        if combine {
            plain.sort_by_key(|entry| (entry.symbol, entry.offset));
        }
        relative.extend(plain);
        relative.extend(indirect);
        relative
    }

    fn encode_dynamic_relocations(&self, entries: &[DynamicRelocation], combine: bool) -> Vec<u8> {
        let encoding = self.encoding();
        let rela = self.target.uses_addends;
        let mut out = Vec::new();
        for relocation in self.finalized_dynamic_relocations(entries, combine) {
            elf::encode_relocation(encoding, &relocation, rela, &mut out);
        }
        out
    }

    fn dynamic_addend(&self, entry: &DynamicRelocation) -> u64 {
        if entry.use_symbol_value {
            return entry.addend as u64;
        }
        let Some(symbol) = entry.symbol else {
            return entry.addend as u64;
        };
        let relocation = Relocation {
            expression: entry.expression,
            kind: entry.kind,
            offset: entry.offset,
            addend: entry.addend,
            symbol,
        };
        let place = self.section_address(entry.section, entry.offset);
        self.evaluate(&relocation, place)
    }

    fn encode_symbol_table(&self) -> Vec<u8> {
        let encoding = self.encoding();
        let mut out = Vec::new();
        elf::encode_symbol(encoding, &elf::Symbol::default(), &mut out);
        for (id, name_offset) in self.symbol_table.iter().zip(&self.symbol_string_offsets) {
            let symbol = &self.symbols[*id];
            let section_number = self.output_index_of_symbol(*id);
            let value = if symbol.is_defined() {
                self.symbol_address(*id, 0)
            } else {
                0
            };
            let size = if section_number != section_index::UNDEFINED {
                symbol.size
            } else {
                0
            };
            let binding = if symbol.is_local {
                symbol_binding::LOCAL
            } else {
                symbol.binding
            };
            elf::encode_symbol(
                encoding,
                &elf::Symbol {
                    name: String::new(),
                    name_offset: *name_offset,
                    value,
                    size,
                    binding,
                    kind: symbol.kind,
                    other: symbol.other,
                    section: section_number,
                },
                &mut out,
            );
        }
        out
    }

    pub fn output_index_of_symbol(&self, id: SymbolIndex) -> u32 {
        let symbol = &self.symbols[id];
        if !symbol.is_defined() || symbol.needs & needs::COPY != 0 {
            return section_index::UNDEFINED;
        }
        let output = if symbol.header_relative {
            self.order.first().copied()
        } else if let Some(output) = symbol.output_section_symbol {
            Some(output)
        } else if let Some(section) = symbol.section {
            let target = self.sections[section].merged_into.unwrap_or(section);
            self.sections[target].output
        } else {
            None
        };
        match output {
            Some(output) if self.outputs[output].live && self.outputs[output].index > 0 => {
                let index = self.outputs[output].index as u32;
                if index >= section_index::RESERVED_LOW {
                    section_index::EXTENDED
                } else {
                    index
                }
            }
            _ => {
                if symbol.header_relative {
                    1
                } else {
                    section_index::ABSOLUTE
                }
            }
        }
    }

    fn section_contents(&mut self, section: SectionIndex) -> Vec<u8> {
        let size = self.sections[section].size as usize;
        let Some(which) = self.sections[section].synthetic else {
            let mut data = self.sections[section].data.clone();
            data.resize(size.max(data.len()), 0);
            if self.sections[section].flags & section_flag::ALLOCATE != 0 {
                self.apply_relocations(section, &mut data);
            } else {
                self.apply_non_allocated_relocations(section, &mut data);
            }
            return data;
        };
        let mut buffer = vec![0u8; size];
        match which {
            Synthetic::Interpreter | Synthetic::Merge => {
                let data = &self.sections[section].data;
                buffer[..data.len().min(size)].copy_from_slice(&data[..data.len().min(size)]);
            }
            Synthetic::GotPlt => self.write_got_plt(&mut buffer),
            Synthetic::IndirectGotPlt => self.write_indirect_got_plt(&mut buffer),
            Synthetic::Plt => self.write_plt(&mut buffer),
            Synthetic::IndirectPlt => self.write_indirect_plt(&mut buffer),
            Synthetic::Thunks => self.write_thunks(section, &mut buffer),
            Synthetic::ExceptionFrame => self.write_exception_frame(&mut buffer),
            Synthetic::ExceptionFrameHeader => self.write_frame_header(&mut buffer),
            Synthetic::DynamicRelocations => {
                let entries = self.dynamic_relocations.clone();
                buffer = self.encode_dynamic_relocations(&entries, true);
            }
            Synthetic::PltRelocations => {
                let entries = self.plt_relocations.clone();
                buffer = self.encode_dynamic_relocations(&entries, false);
            }
            Synthetic::SymbolTable => buffer = self.encode_symbol_table(),
            Synthetic::DynamicSymbolTable => buffer = self.encode_dynamic_symbol_table(),
            Synthetic::DynamicStringTable => buffer = self.dynamic_strings.clone(),
            Synthetic::GnuHash => buffer = self.encode_gnu_hash(),
            Synthetic::SystemFiveHash => buffer = self.encode_system_five_hash(),
            Synthetic::VersionSymbols => buffer = self.encode_version_symbols(),
            Synthetic::VersionNeeds => buffer = self.encode_version_needs(),
            Synthetic::Dynamic => {
                let entries = self.dynamic_entries();
                buffer.clear();
                elf::encode_dynamic(self.encoding(), &entries, &mut buffer);
            }
            Synthetic::StringTable => buffer = self.string_table.clone(),
            Synthetic::SectionNameTable => buffer = self.section_names.bytes().to_vec(),
            Synthetic::BuildIdentifier => {
                let encoding = self.encoding();
                let length = size - 16;
                let mut note = Vec::new();
                encoding.put_u32(&mut note, 4);
                encoding.put_u32(&mut note, length as u32);
                encoding.put_u32(&mut note, elf::note_type::GNU_BUILD_IDENTIFIER);
                note.extend_from_slice(b"GNU\0");
                note.resize(size, 0);
                buffer = note;
            }
            _ => {}
        }
        buffer.resize(size, 0);
        if !self.sections[section].relocations.is_empty() {
            self.apply_relocations(section, &mut buffer);
        }
        buffer
    }

    pub fn write_output(&mut self) -> Vec<u8> {
        let encoding = self.encoding();
        let mut out = vec![0u8; self.file_size as usize];
        let headers = self.program_headers();
        let mut header = Vec::new();
        out[..4].copy_from_slice(b"\x7fELF");
        header.extend_from_slice(b"\x7fELF");
        header.push(if encoding.is_64() { 2 } else { 1 });
        header.push(if encoding.endian == Endian::Little {
            1
        } else {
            2
        });
        header.push(1);
        header.push(0);
        header.push(0);
        header.extend_from_slice(&[0; 7]);
        let kind = if self.is_pic() {
            file_type::SHARED
        } else if self.is_relocatable() {
            file_type::RELOCATABLE
        } else {
            file_type::EXECUTABLE
        };
        encoding.put_u16(&mut header, kind);
        encoding.put_u16(&mut header, self.target.architecture.machine());
        encoding.put_u32(&mut header, 1);
        encoding.put_word(&mut header, self.entry_address());
        encoding.put_word(
            &mut header,
            if self.is_relocatable() {
                0
            } else {
                encoding.header_size() as u64
            },
        );
        encoding.put_word(&mut header, self.section_header_offset);
        encoding.put_u32(&mut header, self.header_flags());
        encoding.put_u16(&mut header, encoding.header_size() as u16);
        encoding.put_u16(
            &mut header,
            if self.is_relocatable() {
                0
            } else {
                encoding.program_header_size() as u16
            },
        );
        encoding.put_u16(&mut header, headers.len() as u16);
        encoding.put_u16(&mut header, encoding.section_header_size() as u16);
        let count = self.order.len() + 1;
        encoding.put_u16(
            &mut header,
            if count >= section_index::RESERVED_LOW as usize {
                0
            } else {
                count as u16
            },
        );
        let names = self
            .synthetic_section(".shstrtab")
            .and_then(|section| self.sections[section].output)
            .map(|output| self.outputs[output].index)
            .unwrap_or(0);
        encoding.put_u16(
            &mut header,
            if names >= section_index::RESERVED_LOW as usize {
                section_index::EXTENDED as u16
            } else {
                names as u16
            },
        );
        out[..header.len()].copy_from_slice(&header);
        let mut table = Vec::new();
        for program in &headers {
            elf::encode_program_header(encoding, program, &mut table);
        }
        let at = encoding.header_size();
        if !self.is_relocatable() {
            out[at..at + table.len()].copy_from_slice(&table);
        }
        for output in self.order.clone() {
            if self.outputs[output].kind == section_type::NO_BITS {
                continue;
            }
            let start = self.outputs[output].offset as usize;
            let size = self.outputs[output].size as usize;
            let executable = self.outputs[output].flags & section_flag::EXECUTE != 0;
            let filler = if executable { self.target.trap } else { [0; 4] };
            let members = self.outputs[output].members.clone();
            let mut cursor = 0usize;
            for member in members {
                let begin = self.sections[member].offset_in_output as usize;
                if executable && filler != [0; 4] {
                    for position in cursor..begin.min(size) {
                        out[start + position] = filler[(position - cursor) % 4];
                    }
                }
                let contents = self.section_contents(member);
                let length = contents.len().min(size.saturating_sub(begin));
                if self.sections[member].kind != section_type::NO_BITS {
                    out[start + begin..start + begin + length].copy_from_slice(&contents[..length]);
                }
                cursor = begin + self.sections[member].size as usize;
            }
            if executable && filler != [0; 4] {
                for position in cursor..size {
                    out[start + position] = filler[(position - cursor) % 4];
                }
            }
        }
        let mut section_headers = Vec::new();
        elf::encode_section_header(encoding, &Section::default(), &mut section_headers);
        for output in self.order.clone() {
            let data = &self.outputs[output];
            let section = Section {
                name_offset: data.name_offset,
                kind: data.kind,
                flags: data.flags,
                address: data.address,
                offset: data.offset,
                size: data.size,
                link: data.link,
                info: data.info,
                alignment: data.alignment,
                entry_size: data.entry_size,
                ..Section::default()
            };
            elf::encode_section_header(encoding, &section, &mut section_headers);
        }
        let at = self.section_header_offset as usize;
        out[at..at + section_headers.len()].copy_from_slice(&section_headers);
        self.write_build_identifier(&mut out);
        out
    }

    fn write_build_identifier(&self, out: &mut [u8]) {
        let Some(section) = self.synthetic_section(".note.gnu.build-id") else {
            return;
        };
        let Some(output) = self.sections[section].output else {
            return;
        };
        let at =
            (self.outputs[output].offset + self.sections[section].offset_in_output + 16) as usize;
        let length = (self.sections[section].size as usize).saturating_sub(16);
        let identifier: Vec<u8> = match &self.options.build_identifier {
            BuildIdentifier::None => return,
            BuildIdentifier::Hexadecimal(bytes) => bytes.clone(),
            BuildIdentifier::Fast => chunked_hash(out, 8, &|chunk: &[u8]| {
                xxh3_64(chunk).to_le_bytes().to_vec()
            }),
            BuildIdentifier::Md5 | BuildIdentifier::Uuid => {
                chunked_hash(out, 16, &|chunk: &[u8]| blake3(chunk)[..16].to_vec())
            }
            BuildIdentifier::Sha1 => {
                chunked_hash(out, 20, &|chunk: &[u8]| blake3(chunk)[..20].to_vec())
            }
        };
        let length = length.min(identifier.len());
        if at + length <= out.len() {
            out[at..at + length].copy_from_slice(&identifier[..length]);
        }
    }

    pub fn header_flags(&self) -> u32 {
        match self.target.architecture {
            Architecture::Arm => {
                let float = match self.arm_features.floating_point_arguments {
                    Some(1) => 0x400,
                    Some(0) | None => 0x200,
                    _ => 0,
                };
                0x0500_0000 | float
            }
            _ => 0,
        }
    }

    pub fn entry_address(&self) -> u64 {
        if self.is_relocatable() {
            return 0;
        }
        if let Some(id) = self.find(&self.entry_name) {
            return self.symbol_address(id, 0);
        }
        parse_number(&self.entry_name).unwrap_or(0)
    }

    pub fn finalize_output_headers(&mut self) {
        for output in self.order.clone() {
            let first = self.outputs[output].members.first().copied();
            if self.outputs[output].flags & section_flag::LINK_ORDER != 0 {
                if let Some(dependency) =
                    first.and_then(|first| self.sections[first].link_order_dependency)
                {
                    if let Some(target) = self.sections[dependency].output {
                        self.outputs[output].link = self.outputs[target].index as u32;
                    }
                }
            }
            let which = first.and_then(|first| self.sections[first].synthetic);
            let index_of = |name: &str, context: &Context| {
                context
                    .synthetic_section(name)
                    .and_then(|section| context.sections[section].output)
                    .map(|output| context.outputs[output].index as u32)
                    .unwrap_or(0)
            };
            match which {
                Some(Synthetic::SymbolTable) => {
                    self.outputs[output].link = index_of(".strtab", self);
                    self.outputs[output].info = self.symbol_table_first_global as u32 + 1;
                }
                Some(Synthetic::DynamicSymbolTable) => {
                    self.outputs[output].link = index_of(".dynstr", self);
                    self.outputs[output].info = 1;
                }
                Some(Synthetic::DynamicRelocations) => {
                    self.outputs[output].link = index_of(".dynsym", self);
                }
                Some(Synthetic::PltRelocations) => {
                    self.outputs[output].link = index_of(".dynsym", self);
                    let target = index_of(".got.plt", self);
                    if target != 0 && self.has_dynamic_symbols {
                        self.outputs[output].info = target;
                        self.outputs[output].flags |= section_flag::INFO_LINK;
                    }
                }
                Some(Synthetic::Dynamic) => self.outputs[output].link = index_of(".dynstr", self),
                Some(Synthetic::GnuHash)
                | Some(Synthetic::SystemFiveHash)
                | Some(Synthetic::VersionSymbols) => {
                    self.outputs[output].link = index_of(".dynsym", self)
                }
                Some(Synthetic::VersionNeeds) => {
                    self.outputs[output].link = index_of(".dynstr", self);
                    self.outputs[output].info = self.version_needs.len() as u32;
                }
                _ => {}
            }
        }
    }

    pub fn run(&mut self) -> Option<Vec<u8>> {
        for name in self.options.undefined.clone() {
            let id = self.add_unused_undefined(&name);
            self.symbols[id].referenced = true;
        }
        self.load_inputs();
        if self.failed() {
            return None;
        }
        self.has_dynamic_symbols = !self.shared_files.is_empty() || self.is_pic();
        self.options.export_dynamic |= self.is_shared();
        self.options.export_dynamic &= self.has_dynamic_symbols;
        self.entry_name = match &self.options.entry {
            Some(entry) => entry.clone(),
            None if self.is_relocatable() => String::new(),
            None => "_start".to_string(),
        };
        if let Some(id) = self.find(&self.entry_name.clone()) {
            self.handle_undefined(id);
        }
        for name in self.options.undefined.clone() {
            if let Some(id) = self.find(&name) {
                if self.symbols[id].is_defined() {
                    self.symbols[id].used_in_regular_object = true;
                }
            }
        }
        for file in self.object_files.clone() {
            self.initialize_sections(file);
        }
        for file in self.object_files.clone() {
            self.post_parse(file);
        }
        if self.failed() {
            return None;
        }
        if self.is_relocatable() {
            self.error(not_supported("relocatable output"));
            return None;
        }
        self.and_features = self
            .object_files
            .iter()
            .map(|file| self.files[*file].and_features)
            .fold(u32::MAX, |left, right| left & right);
        if self.object_files.is_empty() {
            self.and_features = 0;
        }
        self.and_features = 0;
        self.add_reserved_symbols();
        self.parse_versions_and_compute_preemptible();
        self.replace_common_symbols();
        self.aggregate_sections();
        self.maximum_page_size = self
            .options
            .maximum_page_size
            .unwrap_or(self.target.default_maximum_page_size);
        self.common_page_size = self
            .options
            .common_page_size
            .unwrap_or(self.target.default_common_page_size)
            .min(self.maximum_page_size);
        self.image_base = self.options.image_base.unwrap_or(if self.is_pic() {
            0
        } else {
            self.target.default_image_base
        });
        self.create_comment_section();
        self.split_merge_sections();
        self.mark_live();
        self.create_synthetic_sections();
        if let Some(synthetic) = self.synthetic_section(".eh_frame") {
            for section in self.exception_frame_inputs.clone() {
                let alignment = self.sections[synthetic]
                    .alignment
                    .max(self.sections[section].alignment);
                self.sections[synthetic].alignment = alignment;
            }
        }
        self.create_output_sections();
        self.finalize_input_sections();
        if self.failed() {
            return None;
        }
        self.add_start_end_symbols();
        if self.has_dynamic_symbols {
            if let Some(dynamic) = self.synthetic_section(".dynamic") {
                let id = self.insert("_DYNAMIC");
                let state = self.symbols[id].state.clone();
                if state != SymbolState::Defined {
                    self.define_synthesized(
                        id,
                        SymbolPlace::Section(dynamic),
                        0,
                        visibility::HIDDEN,
                        symbol_type::NO_TYPE,
                        symbol_binding::WEAK,
                    );
                    self.symbols[id].used_in_regular_object = true;
                }
            }
        }
        if self.options.export_dynamic {
            for id in self.synthesized.clone() {
                if self.compute_binding(id) != symbol_binding::LOCAL {
                    self.symbols[id].exported = true;
                }
            }
        }
        self.combine_exception_frames();
        self.demote_symbols_and_compute_preemptible();
        let undefined = self.scan_relocations();
        self.report_undefined(undefined);
        if self.failed() {
            return None;
        }
        self.post_scan_relocations();
        self.build_symbol_tables();
        self.remove_unused_synthetic_sections();
        self.sort_output_sections();
        self.assign_section_indices();
        for output in self.order.clone() {
            if let Some(address) = self.options.section_start.get(&self.outputs[output].name) {
                self.outputs[output].address_fixed = Some(*address);
            }
        }
        let mut header = OutputSection::new("");
        header.flags = section_flag::ALLOCATE;
        header.size = self.encoding().header_size() as u64;
        self.elf_header_output = self.outputs.len();
        self.outputs.push(header);
        let mut programs = OutputSection::new("");
        programs.flags = section_flag::ALLOCATE;
        programs.alignment = self.word_size();
        self.program_header_output = self.outputs.len();
        self.outputs.push(programs);
        self.create_segments();
        self.set_reserved_symbol_sections();
        self.finalize_dynamic_tables();
        self.finalize_synthetic_sizes();
        self.fix_section_alignments();
        self.assign_addresses();
        let mut pass = 0u32;
        loop {
            let mut changed = if self.needs_thunks() {
                self.create_thunks(pass)
            } else {
                false
            };
            pass += 1;
            if changed && pass >= 30 {
                self.error(problem("EI237", "Address assignment did not settle."));
                break;
            }
            self.finalize_synthetic_sizes();
            changed |= self.update_frame_header();
            changed |= self.assign_addresses();
            if !changed {
                break;
            }
        }
        self.finalize_symbol_table();
        self.finalize_synthetic_sizes();
        self.assign_addresses();
        self.finalize_output_headers();
        if self.failed() {
            return None;
        }
        self.remove_empty_loads();
        self.assign_file_offsets();
        let bytes = self.write_output();
        if self.failed() {
            return None;
        }
        Some(bytes)
    }
}

pub fn link(arguments: &[String], source: &dyn Fn(&str) -> Option<Vec<u8>>) -> LinkOutcome {
    let options = match parse_command_line(arguments) {
        Ok(options) => options,
        Err(diagnostic) => {
            return LinkOutcome {
                outputs: Vec::new(),
                diagnostics: vec![diagnostic],
                messages: Vec::new(),
            }
        }
    };
    let output = options.output.clone();
    let mut context = Context::new(options, source);
    let bytes = context.run();
    let mut outcome = LinkOutcome {
        outputs: Vec::new(),
        diagnostics: context.diagnostics,
        messages: context.messages,
    };
    if let Some(bytes) = bytes {
        outcome.outputs.push((output, bytes));
    }
    outcome
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThunkKind {
    Aarch64AbsoluteLong,
    Aarch64PagedLong,
    ArmV7AbsoluteLong,
    ArmV7PositionIndependentLong,
    ThumbV7AbsoluteLong,
    ThumbV7PositionIndependentLong,
    ThumbV6MAbsoluteLong,
    ThumbV6MPositionIndependentLong,
    ArmV5LongLoadProgramCounter,
    ArmV4AbsoluteLongExchange,
    ArmV4PositionIndependentLongExchange,
    ArmV4PositionIndependentLong,
    ThumbV4AbsoluteLongExchange,
    ThumbV4AbsoluteLong,
    ThumbV4PositionIndependentLongExchange,
    ThumbV4PositionIndependentLong,
}

impl ThunkKind {
    fn is_thumb(self) -> bool {
        matches!(
            self,
            ThunkKind::ThumbV7AbsoluteLong
                | ThunkKind::ThumbV7PositionIndependentLong
                | ThunkKind::ThumbV6MAbsoluteLong
                | ThunkKind::ThumbV6MPositionIndependentLong
                | ThunkKind::ThumbV4AbsoluteLongExchange
                | ThunkKind::ThumbV4AbsoluteLong
                | ThunkKind::ThumbV4PositionIndependentLongExchange
                | ThunkKind::ThumbV4PositionIndependentLong
        )
    }

    fn is_aarch64(self) -> bool {
        matches!(
            self,
            ThunkKind::Aarch64AbsoluteLong | ThunkKind::Aarch64PagedLong
        )
    }

    fn long_size(self) -> u64 {
        match self {
            ThunkKind::Aarch64AbsoluteLong => 16,
            ThunkKind::Aarch64PagedLong => 12,
            ThunkKind::ArmV7AbsoluteLong => 12,
            ThunkKind::ArmV7PositionIndependentLong => 16,
            ThunkKind::ThumbV7AbsoluteLong => 10,
            ThunkKind::ThumbV7PositionIndependentLong => 12,
            ThunkKind::ThumbV6MAbsoluteLong => 12,
            ThunkKind::ThumbV6MPositionIndependentLong => 16,
            ThunkKind::ArmV5LongLoadProgramCounter => 8,
            ThunkKind::ArmV4AbsoluteLongExchange => 12,
            ThunkKind::ArmV4PositionIndependentLongExchange => 16,
            ThunkKind::ArmV4PositionIndependentLong => 12,
            ThunkKind::ThumbV4AbsoluteLongExchange => 12,
            ThunkKind::ThumbV4AbsoluteLong => 16,
            ThunkKind::ThumbV4PositionIndependentLongExchange => 16,
            ThunkKind::ThumbV4PositionIndependentLong => 20,
        }
    }

    fn name_prefix(self) -> &'static str {
        match self {
            ThunkKind::Aarch64AbsoluteLong => "__AArch64AbsLongThunk_",
            ThunkKind::Aarch64PagedLong => "__AArch64ADRPThunk_",
            ThunkKind::ArmV7AbsoluteLong => "__ARMv7ABSLongThunk_",
            ThunkKind::ArmV7PositionIndependentLong => "__ARMV7PILongThunk_",
            ThunkKind::ThumbV7AbsoluteLong => "__Thumbv7ABSLongThunk_",
            ThunkKind::ThumbV7PositionIndependentLong => "__ThumbV7PILongThunk_",
            ThunkKind::ThumbV6MAbsoluteLong => "__Thumbv6MABSLongThunk_",
            ThunkKind::ThumbV6MPositionIndependentLong => "__Thumbv6MPILongThunk_",
            ThunkKind::ArmV5LongLoadProgramCounter => "__ARMv5LongLdrPcThunk_",
            ThunkKind::ArmV4AbsoluteLongExchange => "__ARMv4ABSLongBXThunk_",
            ThunkKind::ArmV4PositionIndependentLongExchange => "__ARMv4PILongBXThunk_",
            ThunkKind::ArmV4PositionIndependentLong => "__ARMv4PILongThunk_",
            ThunkKind::ThumbV4AbsoluteLongExchange => "__Thumbv4ABSLongBXThunk_",
            ThunkKind::ThumbV4AbsoluteLong => "__Thumbv4ABSLongThunk_",
            ThunkKind::ThumbV4PositionIndependentLongExchange => "__Thumbv4PILongBXThunk_",
            ThunkKind::ThumbV4PositionIndependentLong => "__Thumbv4PILongThunk_",
        }
    }

    fn long_mapping_symbols(self) -> &'static [(&'static str, u64)] {
        match self {
            ThunkKind::Aarch64AbsoluteLong => &[("$d", 8)],
            ThunkKind::ThumbV6MAbsoluteLong => &[("$d", 8)],
            ThunkKind::ThumbV6MPositionIndependentLong => &[("$d", 12)],
            ThunkKind::ArmV5LongLoadProgramCounter => &[("$d", 4)],
            ThunkKind::ArmV4AbsoluteLongExchange => &[("$d", 8)],
            ThunkKind::ThumbV4AbsoluteLongExchange => &[("$a", 4), ("$d", 8)],
            ThunkKind::ThumbV4AbsoluteLong => &[("$a", 4), ("$d", 12)],
            ThunkKind::ArmV4PositionIndependentLongExchange => &[("$d", 12)],
            ThunkKind::ArmV4PositionIndependentLong => &[("$d", 8)],
            ThunkKind::ThumbV4PositionIndependentLongExchange => &[("$a", 4), ("$d", 12)],
            ThunkKind::ThumbV4PositionIndependentLong => &[("$a", 4), ("$d", 16)],
            _ => &[],
        }
    }

    fn checks_short_when_created(self) -> bool {
        !matches!(
            self,
            ThunkKind::Aarch64PagedLong
                | ThunkKind::ArmV7AbsoluteLong
                | ThunkKind::ArmV7PositionIndependentLong
                | ThunkKind::ThumbV7AbsoluteLong
                | ThunkKind::ThumbV7PositionIndependentLong
        )
    }
}

#[derive(Clone, Debug)]
pub struct Thunk {
    pub kind: ThunkKind,
    pub destination: SymbolIndex,
    pub addend: i64,
    pub offset: u64,
    pub alignment: u64,
    pub symbols: Vec<SymbolIndex>,
    pub may_use_short: bool,
    pub section: SectionIndex,
}

#[derive(Clone, Debug, Default)]
pub struct ThunkState {
    pub thunks: Vec<Thunk>,
    pub sections: BTreeMap<OutputIndex, Vec<(SectionIndex, u32)>>,
    pub by_place: BTreeMap<(SectionIndex, u64, i64), Vec<usize>>,
    pub by_symbol: BTreeMap<(SymbolIndex, i64), Vec<usize>>,
    pub by_target: BTreeMap<SymbolIndex, usize>,
    pub thunk_of_section: BTreeMap<SectionIndex, Vec<usize>>,
}

impl<'s> Context<'s> {
    pub fn needs_thunks(&self) -> bool {
        matches!(
            self.target.architecture,
            Architecture::Arm | Architecture::Aarch64
        )
    }

    fn thunk_spacing(&self) -> u64 {
        match self.target.architecture {
            Architecture::Aarch64 => 128 * 1024 * 1024 - 0x30000,
            Architecture::Arm => {
                if self.arm_features.j1_j2_branch_encoding {
                    0x100_0000 - 0x30000
                } else {
                    0x40_0000 - 0x7500
                }
            }
            _ => 0,
        }
    }

    fn program_counter_bias(&self, kind: u32) -> i64 {
        if self.target.architecture != Architecture::Arm {
            return 0;
        }
        match kind {
            51 | 30 | 10 => 4,
            _ => 8,
        }
    }

    pub fn in_branch_range(&self, kind: u32, source: u64, destination: u64) -> bool {
        match self.target.architecture {
            Architecture::Aarch64 => {
                if kind != 283 && kind != 282 && kind != 314 {
                    return true;
                }
                let mut range: u64 = if kind == 314 {
                    1 << 31
                } else {
                    128 * 1024 * 1024
                };
                if destination > source {
                    range -= 4;
                    return destination - source <= range;
                }
                source - destination <= range
            }
            Architecture::Arm => {
                let (mut source, mut destination) = (source, destination);
                if destination & 1 == 0 {
                    source &= !3;
                } else {
                    destination &= !1;
                }
                let offset = sign_extend(destination.wrapping_sub(source), 32);
                let fits =
                    |bits: u32| offset >= -(1i64 << (bits - 1)) && offset < (1i64 << (bits - 1));
                match kind {
                    1 | 27 | 29 | 28 => fits(26),
                    51 => fits(21),
                    30 | 10 => {
                        if self.arm_features.j1_j2_branch_encoding {
                            fits(25)
                        } else {
                            fits(23)
                        }
                    }
                    _ => true,
                }
            }
            _ => true,
        }
    }

    fn arm_destination(&self, symbol: SymbolIndex) -> u64 {
        let value = if self.symbols[symbol].plt_index.is_some() {
            self.plt_address(symbol)
        } else {
            self.symbol_address(symbol, 0)
        };
        sign_extend(value, 32) as u64
    }

    fn needs_thunk(&self, relocation: &Relocation, source: u64) -> bool {
        let symbol = &self.symbols[relocation.symbol];
        let in_plt = symbol.plt_index.is_some();
        if symbol.is_undefined() && !in_plt {
            return false;
        }
        let destination = if relocation.expression == Expression::PltProgramCounter {
            self.plt_address(relocation.symbol)
        } else {
            self.symbol_address(relocation.symbol, 0)
        };
        match self.target.architecture {
            Architecture::Aarch64 => {
                if !matches!(relocation.kind, 283 | 282 | 314) {
                    return false;
                }
                let destination = if relocation.expression == Expression::PltProgramCounter {
                    destination
                } else {
                    self.symbol_address(relocation.symbol, relocation.addend)
                };
                !self.in_branch_range(relocation.kind, source, destination)
            }
            Architecture::Arm => {
                let address = self.symbol_address(relocation.symbol, 0);
                match relocation.kind {
                    1 | 27 | 29 | 28 => {
                        if relocation.kind != 28
                            && symbol.is_function()
                            && relocation.expression == Expression::ProgramCounter
                            && address & 1 != 0
                        {
                            return true;
                        }
                        !self.in_branch_range(
                            relocation.kind,
                            source,
                            destination.wrapping_add(relocation.addend as u64),
                        ) || (!self.arm_features.has_exchange && address & 1 != 0)
                    }
                    51 | 30 | 10 => {
                        if relocation.kind != 10
                            && ((relocation.expression == Expression::PltProgramCounter
                                && !self.use_thumb_plt())
                                || (symbol.is_function() && address & 1 == 0))
                        {
                            return true;
                        }
                        !self.in_branch_range(
                            relocation.kind,
                            source,
                            destination.wrapping_add(relocation.addend as u64),
                        ) || (!self.arm_features.has_exchange && address & 1 == 0)
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn thunk_supported(&self, kind: u32) -> bool {
        if self.target.architecture != Architecture::Arm || self.arm_features.has_move_wide {
            return true;
        }
        if self.arm_features.j1_j2_branch_encoding {
            return matches!(kind, 51 | 30 | 10);
        }
        matches!(kind, 1 | 27 | 29 | 28 | 10)
    }

    fn choose_thunk(
        &self,
        kind: u32,
        symbol: SymbolIndex,
        addend: i64,
        section: SectionIndex,
    ) -> ThunkKind {
        let position_independent = self.is_pic();
        if self.target.architecture == Architecture::Aarch64 {
            return if position_independent {
                ThunkKind::Aarch64PagedLong
            } else {
                ThunkKind::Aarch64AbsoluteLong
            };
        }
        let _ = section;
        let features = self.arm_features;
        let arm_branch = matches!(kind, 1 | 27 | 29 | 28);
        if !features.has_move_wide {
            if features.j1_j2_branch_encoding {
                return if position_independent {
                    ThunkKind::ThumbV6MPositionIndependentLong
                } else {
                    ThunkKind::ThumbV6MAbsoluteLong
                };
            }
            if features.has_exchange {
                return if position_independent {
                    ThunkKind::ArmV4PositionIndependentLongExchange
                } else {
                    ThunkKind::ArmV5LongLoadProgramCounter
                };
            }
            let thumb_target = self.symbol_address(symbol, addend) & 1 != 0;
            if arm_branch {
                return match (position_independent, thumb_target) {
                    (true, true) => ThunkKind::ArmV4PositionIndependentLongExchange,
                    (true, false) => ThunkKind::ArmV4PositionIndependentLong,
                    (false, true) => ThunkKind::ArmV4AbsoluteLongExchange,
                    (false, false) => ThunkKind::ArmV5LongLoadProgramCounter,
                };
            }
            return match (position_independent, thumb_target) {
                (true, true) => ThunkKind::ThumbV4PositionIndependentLong,
                (true, false) => ThunkKind::ThumbV4PositionIndependentLongExchange,
                (false, true) => ThunkKind::ThumbV4AbsoluteLong,
                (false, false) => ThunkKind::ThumbV4AbsoluteLongExchange,
            };
        }
        if arm_branch {
            if position_independent {
                ThunkKind::ArmV7PositionIndependentLong
            } else {
                ThunkKind::ArmV7AbsoluteLong
            }
        } else if position_independent {
            ThunkKind::ThumbV7PositionIndependentLong
        } else {
            ThunkKind::ThumbV7AbsoluteLong
        }
    }

    fn thunk_compatible(&self, thunk: &Thunk, kind: u32) -> bool {
        if thunk.kind.is_aarch64() {
            return true;
        }
        if thunk.kind.is_thumb() {
            if !self.arm_features.has_exchange && kind == 28 {
                return false;
            }
            return !matches!(kind, 29 | 1 | 27);
        }
        if !self.arm_features.has_exchange && kind == 10 {
            return false;
        }
        !matches!(kind, 51 | 30)
    }

    fn thunk_target_address(&self, thunk: usize) -> u64 {
        self.symbol_address(self.thunk_state.thunks[thunk].symbols[0], 0)
    }

    fn thunk_destination(&self, thunk: usize) -> u64 {
        let data = &self.thunk_state.thunks[thunk];
        if self.symbols[data.destination].plt_index.is_some() {
            self.plt_address(data.destination)
        } else {
            self.symbol_address(data.destination, data.addend)
        }
    }

    fn add_thunk_local(&mut self, thunk: usize, name: &str, kind: u8, value: u64) {
        let section = self.thunk_state.thunks[thunk].section;
        let offset = self.thunk_state.thunks[thunk].offset;
        let mut data = SymbolData::new(name);
        data.state = SymbolState::Defined;
        data.is_local = true;
        data.binding = symbol_binding::LOCAL;
        data.kind = kind;
        data.file = Some(INTERNAL);
        data.section = Some(section);
        data.value = value + offset;
        data.used_in_regular_object = true;
        let id = self.symbols.len();
        self.symbols.push(data);
        self.thunk_state.thunks[thunk].symbols.push(id);
        if self.options.strip != Strip::All {
            self.add_to_symbol_table(id);
        }
    }

    fn may_use_short_thunk(&mut self, thunk: usize) -> bool {
        if !self.thunk_state.thunks[thunk].may_use_short {
            return false;
        }
        let kind = self.thunk_state.thunks[thunk].kind;
        let destination_symbol = self.thunk_state.thunks[thunk].destination;
        let short = if kind.is_aarch64() {
            let destination = self.thunk_destination(thunk);
            let place = self.thunk_target_address(thunk);
            let offset = destination.wrapping_sub(place) as i64;
            (-(1i64 << 27)..(1i64 << 27)).contains(&offset)
        } else if kind.is_thumb() {
            let destination = self.arm_destination(destination_symbol);
            if destination & 1 == 0
                || !self.arm_features.j1_j2_branch_encoding
                || !self.arm_features.has_move_wide
            {
                false
            } else {
                let place = self.thunk_target_address(thunk) & !1;
                let offset = destination.wrapping_sub(place).wrapping_sub(4) as i64;
                (-(1i64 << 24)..(1i64 << 24)).contains(&offset)
            }
        } else {
            let destination = self.arm_destination(destination_symbol);
            if destination & 1 != 0 {
                false
            } else {
                let place = self.thunk_target_address(thunk);
                let offset = destination.wrapping_sub(place).wrapping_sub(8) as i64;
                (-(1i64 << 25)..(1i64 << 25)).contains(&offset)
            }
        };
        if !short {
            self.thunk_state.thunks[thunk].may_use_short = false;
            for (name, value) in kind.long_mapping_symbols() {
                self.add_thunk_local(thunk, name, symbol_type::NO_TYPE, *value);
            }
            if kind == ThunkKind::Aarch64AbsoluteLong {
                self.thunk_state.thunks[thunk].alignment = 8;
            }
        }
        short
    }

    fn thunk_size(&mut self, thunk: usize) -> u64 {
        if self.may_use_short_thunk(thunk) {
            4
        } else {
            self.thunk_state.thunks[thunk].kind.long_size()
        }
    }

    fn add_thunk_section(&mut self, output: OutputIndex, offset: u64, pass: u32) -> SectionIndex {
        let mut section = InputSection::synthetic(
            ".text.thunk",
            section_type::PROGRAM_BITS,
            section_flag::ALLOCATE | section_flag::EXECUTE,
            4,
            Synthetic::Thunks,
        );
        section.output = Some(output);
        section.offset_in_output = offset;
        let index = self.sections.len();
        self.sections.push(section);
        self.thunk_state
            .sections
            .entry(output)
            .or_default()
            .push((index, pass));
        self.thunk_state.thunk_of_section.insert(index, Vec::new());
        index
    }

    fn create_initial_thunk_sections(&mut self) {
        let spacing = self.thunk_spacing();
        for output in self.order.clone() {
            let flags = self.outputs[output].flags;
            if flags & section_flag::ALLOCATE == 0 || flags & section_flag::EXECUTE == 0 {
                continue;
            }
            let members = self.outputs[output].members.clone();
            let (Some(first), Some(last)) = (members.first(), members.last()) else {
                continue;
            };
            let begin = self.sections[*first].offset_in_output;
            let end = self.sections[*last].offset_in_output + self.sections[*last].size;
            let mut last_lower_bound = u64::from(u32::MAX);
            if end - begin > spacing * 2 {
                last_lower_bound = end - spacing;
            }
            let mut limit = begin;
            let mut previous_limit = begin;
            let mut upper_bound = begin + spacing;
            for member in &members {
                limit = self.sections[*member].offset_in_output + self.sections[*member].size;
                if limit > upper_bound {
                    self.add_thunk_section(output, previous_limit, 0);
                    upper_bound = previous_limit + spacing;
                }
                if limit > last_lower_bound {
                    break;
                }
                previous_limit = limit;
            }
            self.add_thunk_section(output, limit, 0);
        }
    }

    fn thunk_section_for(
        &mut self,
        output: OutputIndex,
        section: SectionIndex,
        relocation: &Relocation,
        source: u64,
        pass: u32,
    ) -> SectionIndex {
        let bias = self.program_counter_bias(relocation.kind);
        let base_address = self.outputs[output].address;
        let candidates = self
            .thunk_state
            .sections
            .get(&output)
            .cloned()
            .unwrap_or_default();
        for (thunk_section, _) in candidates {
            let base = (base_address + self.sections[thunk_section].offset_in_output)
                .wrapping_sub(bias as u64);
            let limit = base + self.sections[thunk_section].size;
            let probe = if source > limit { base } else { limit };
            if self.in_branch_range(relocation.kind, source, probe) {
                return thunk_section;
            }
        }
        let mut offset = self.sections[section].offset_in_output;
        if !self.in_branch_range(
            relocation.kind,
            source,
            (base_address + offset).wrapping_add(relocation.addend as u64),
        ) {
            offset += self.sections[section].size;
        }
        self.add_thunk_section(output, offset, pass)
    }

    pub fn assign_thunk_offsets(&mut self, thunk_section: SectionIndex) -> bool {
        let thunks = self
            .thunk_state
            .thunk_of_section
            .get(&thunk_section)
            .cloned()
            .unwrap_or_default();
        let mut offset = 0u64;
        let mut changed = false;
        for thunk in thunks {
            let alignment = self.thunk_state.thunks[thunk].alignment;
            if alignment > self.sections[thunk_section].alignment {
                self.sections[thunk_section].alignment = alignment;
                changed = true;
            }
            offset = align_to(offset, alignment);
            let previous = self.thunk_state.thunks[thunk].offset;
            for symbol in self.thunk_state.thunks[thunk].symbols.clone() {
                self.symbols[symbol].value = self.symbols[symbol].value - previous + offset;
            }
            self.thunk_state.thunks[thunk].offset = offset;
            let size = self.thunk_size(thunk);
            let target = self.thunk_state.thunks[thunk].symbols[0];
            self.symbols[target].size = size;
            offset += size;
        }
        if offset != self.sections[thunk_section].size {
            changed = true;
        }
        self.sections[thunk_section].size = offset;
        changed
    }

    fn sort_thunks_by_destination(&mut self, thunk_section: SectionIndex) {
        let base = self.section_address(thunk_section, 0);
        let thunks = self
            .thunk_state
            .thunk_of_section
            .get(&thunk_section)
            .cloned()
            .unwrap_or_default();
        let keyed: Vec<(u64, usize)> = thunks
            .iter()
            .map(|thunk| (self.thunk_destination(*thunk), *thunk))
            .collect();
        let (mut backward, mut forward): (Vec<_>, Vec<_>) =
            keyed.into_iter().partition(|entry| entry.0 <= base);
        forward.sort_by_key(|entry| std::cmp::Reverse(entry.0));
        backward.extend(forward);
        self.thunk_state.thunk_of_section.insert(
            thunk_section,
            backward.into_iter().map(|entry| entry.1).collect(),
        );
    }

    pub fn create_thunks(&mut self, pass: u32) -> bool {
        let mut changed = false;
        if pass == 0 && self.thunk_spacing() != 0 {
            self.create_initial_thunk_sections();
        }
        for output in self.order.clone() {
            let flags = self.outputs[output].flags;
            if flags & section_flag::ALLOCATE == 0 || flags & section_flag::EXECUTE == 0 {
                continue;
            }
            for member in self.outputs[output].members.clone() {
                let count = self.sections[member].relocations.len();
                for position in 0..count {
                    let relocation = self.sections[member].relocations[position].clone();
                    if relocation.expression == Expression::None {
                        continue;
                    }
                    let source = self.section_address(member, relocation.offset);
                    if pass > 0 {
                        if let Some(thunk) =
                            self.thunk_state.by_target.get(&relocation.symbol).copied()
                        {
                            if self.in_branch_range(
                                relocation.kind,
                                source,
                                self.symbol_address(relocation.symbol, relocation.addend),
                            ) {
                                continue;
                            }
                            let destination = self.thunk_state.thunks[thunk].destination;
                            let entry = &mut self.sections[member].relocations[position];
                            entry.symbol = destination;
                            entry.addend = self.thunk_state.thunks[thunk].addend;
                            if self.symbols[destination].plt_index.is_some() {
                                entry.expression = match entry.expression {
                                    Expression::ProgramCounter => Expression::PltProgramCounter,
                                    Expression::Absolute => Expression::Plt,
                                    other => other,
                                };
                            }
                        }
                    }
                    let relocation = self.sections[member].relocations[position].clone();
                    if !self.needs_thunk(&relocation, source) {
                        continue;
                    }
                    if !self.thunk_supported(relocation.kind) {
                        let name = relocation_name(Architecture::Arm, relocation.kind);
                        let symbol = self.symbols[relocation.symbol].name.clone();
                        self.error(
                            problem("EI238", "This branch cannot reach its target on the architecture the inputs describe.")
                                .with_context(format!("Relocation: {name}"))
                                .with_context(format!("Symbol: {symbol}")),
                        );
                        self.sections[member].relocations[position].expression = Expression::None;
                        continue;
                    }
                    let bias = self.program_counter_bias(relocation.kind);
                    let key_addend = relocation.addend + bias;
                    let symbol = &self.symbols[relocation.symbol];
                    let place_key = if symbol.is_defined() && symbol.plt_index.is_none() {
                        symbol
                            .section
                            .map(|section| (section, symbol.value, key_addend))
                    } else {
                        None
                    };
                    let existing = match place_key {
                        Some(key) => self
                            .thunk_state
                            .by_place
                            .get(&key)
                            .cloned()
                            .unwrap_or_default(),
                        None => self
                            .thunk_state
                            .by_symbol
                            .get(&(relocation.symbol, key_addend))
                            .cloned()
                            .unwrap_or_default(),
                    };
                    let mut chosen = None;
                    for thunk in existing {
                        if self.thunk_compatible(&self.thunk_state.thunks[thunk], relocation.kind)
                            && self.in_branch_range(
                                relocation.kind,
                                source,
                                self.symbol_address(
                                    self.thunk_state.thunks[thunk].symbols[0],
                                    -bias,
                                ),
                            )
                        {
                            chosen = Some(thunk);
                            break;
                        }
                    }
                    let thunk = match chosen {
                        Some(thunk) => thunk,
                        None => {
                            let kind = self.choose_thunk(
                                relocation.kind,
                                relocation.symbol,
                                relocation.addend,
                                member,
                            );
                            let thunk = self.thunk_state.thunks.len();
                            let thunk_section =
                                self.thunk_section_for(output, member, &relocation, source, pass);
                            self.thunk_state.thunks.push(Thunk {
                                kind,
                                destination: relocation.symbol,
                                addend: relocation.addend,
                                offset: 0,
                                alignment: if kind.is_thumb() { 2 } else { 4 },
                                symbols: Vec::new(),
                                may_use_short: true,
                                section: thunk_section,
                            });
                            match place_key {
                                Some(key) => self
                                    .thunk_state
                                    .by_place
                                    .entry(key)
                                    .or_default()
                                    .push(thunk),
                                None => self
                                    .thunk_state
                                    .by_symbol
                                    .entry((relocation.symbol, key_addend))
                                    .or_default()
                                    .push(thunk),
                            }
                            self.thunk_state
                                .thunk_of_section
                                .entry(thunk_section)
                                .or_default()
                                .push(thunk);
                            let name = format!(
                                "{}{}",
                                kind.name_prefix(),
                                self.symbols[relocation.symbol].name
                            );
                            let start = if kind.is_thumb() { 1 } else { 0 };
                            self.add_thunk_local(thunk, &name, symbol_type::FUNCTION, start);
                            let mapping = if kind.is_aarch64() {
                                "$x"
                            } else if kind.is_thumb() {
                                "$t"
                            } else {
                                "$a"
                            };
                            self.add_thunk_local(thunk, mapping, symbol_type::NO_TYPE, 0);
                            if kind.checks_short_when_created() {
                                self.may_use_short_thunk(thunk);
                            }
                            let target = self.thunk_state.thunks[thunk].symbols[0];
                            self.thunk_state.by_target.insert(target, thunk);
                            thunk
                        }
                    };
                    let target = self.thunk_state.thunks[thunk].symbols[0];
                    let entry = &mut self.sections[member].relocations[position];
                    entry.symbol = target;
                    entry.expression = from_plt(entry.expression);
                    entry.addend = -bias;
                }
            }
            let thunk_sections = self
                .thunk_state
                .sections
                .get(&output)
                .cloned()
                .unwrap_or_default();
            for (thunk_section, _) in thunk_sections {
                if pass == 0 {
                    self.sort_thunks_by_destination(thunk_section);
                }
                changed |= self.assign_thunk_offsets(thunk_section);
            }
        }
        self.merge_thunks(pass);
        changed
    }

    fn merge_thunks(&mut self, pass: u32) {
        for output in self.order.clone() {
            let Some(list) = self.thunk_state.sections.get_mut(&output) else {
                continue;
            };
            let sections = &self.sections;
            list.retain(|(section, _)| sections[*section].size != 0);
            let mut fresh: Vec<SectionIndex> = list
                .iter()
                .filter(|(_, created)| *created == pass)
                .map(|(section, _)| *section)
                .collect();
            fresh.sort_by_key(|section| self.sections[*section].offset_in_output);
            if fresh.is_empty() {
                continue;
            }
            let members = std::mem::take(&mut self.outputs[output].members);
            let mut merged = Vec::with_capacity(members.len() + fresh.len());
            let mut left = members.into_iter().peekable();
            let mut right = fresh.into_iter().peekable();
            loop {
                match (left.peek(), right.peek()) {
                    (Some(member), Some(thunk)) => {
                        let thunk_offset = self.sections[*thunk].offset_in_output;
                        let member_offset = self.sections[*member].offset_in_output;
                        if thunk_offset < member_offset {
                            merged.push(right.next().unwrap_or(0));
                        } else {
                            merged.push(left.next().unwrap_or(0));
                        }
                    }
                    (Some(_), None) => merged.push(left.next().unwrap_or(0)),
                    (None, Some(_)) => merged.push(right.next().unwrap_or(0)),
                    (None, None) => break,
                }
            }
            self.outputs[output].members = merged;
        }
    }

    fn write_thunks(&mut self, section: SectionIndex, buffer: &mut [u8]) {
        let thunks = self
            .thunk_state
            .thunk_of_section
            .get(&section)
            .cloned()
            .unwrap_or_default();
        for thunk in thunks {
            let at = self.thunk_state.thunks[thunk].offset as usize;
            let size = self.thunk_size(thunk) as usize;
            if at + size > buffer.len() {
                continue;
            }
            let short = self.thunk_state.thunks[thunk].may_use_short;
            let mut piece = vec![0u8; size];
            self.write_thunk(thunk, short, &mut piece);
            buffer[at..at + size].copy_from_slice(&piece);
        }
    }

    fn encode_no_symbol(&mut self, out: &mut [u8], at: usize, kind: u32, value: u64) {
        let relocation = Relocation {
            expression: Expression::Absolute,
            kind,
            offset: 0,
            addend: 0,
            symbol: self.dummy_symbol(),
        };
        let location = String::new();
        match self.target.architecture {
            Architecture::Aarch64 => self.relocate_aarch64(out, at, &relocation, value, &location),
            _ => self.relocate_arm(out, at, &relocation, value, &location),
        }
    }

    fn write_thunk(&mut self, thunk: usize, short: bool, out: &mut [u8]) {
        let data = self.thunk_state.thunks[thunk].clone();
        let place = self.thunk_target_address(thunk);
        let encoding = self.encoding();
        if data.kind.is_aarch64() {
            let destination = self.thunk_destination(thunk);
            if short {
                out[0..4].copy_from_slice(&0x1400_0000u32.to_le_bytes());
                self.encode_no_symbol(out, 0, 283, destination.wrapping_sub(place));
                return;
            }
            match data.kind {
                ThunkKind::Aarch64AbsoluteLong => {
                    out[0..8].copy_from_slice(&[0x50, 0x00, 0x00, 0x58, 0x00, 0x02, 0x1f, 0xd6]);
                    self.encode_no_symbol(out, 8, 257, destination);
                }
                _ => {
                    out[0..12].copy_from_slice(&[
                        0x10, 0, 0, 0x90, 0x10, 0x02, 0x00, 0x91, 0x00, 0x02, 0x1f, 0xd6,
                    ]);
                    self.encode_no_symbol(
                        out,
                        0,
                        275,
                        aarch64_page(destination).wrapping_sub(aarch64_page(place)),
                    );
                    self.encode_no_symbol(out, 4, 277, destination);
                }
            }
            return;
        }
        let destination = self.arm_destination(data.destination);
        let half =
            |out: &mut [u8], at: usize, value: u32| encoding.write_u16(out, at, value as u16);
        let word = |out: &mut [u8], at: usize, value: u32| encoding.write_u32(out, at, value);
        if short {
            if data.kind.is_thumb() {
                half(out, 0, 0xf000);
                half(out, 2, 0xb000);
                self.encode_no_symbol(out, 0, 30, destination.wrapping_sub(place).wrapping_sub(4));
            } else {
                word(out, 0, 0xea00_0000);
                self.encode_no_symbol(out, 0, 29, destination.wrapping_sub(place).wrapping_sub(8));
            }
            return;
        }
        let aligned_place = place & !1;
        match data.kind {
            ThunkKind::ArmV7AbsoluteLong => {
                word(out, 0, 0xe300_c000);
                word(out, 4, 0xe340_c000);
                word(out, 8, 0xe12f_ff1c);
                self.encode_no_symbol(out, 0, 43, destination);
                self.encode_no_symbol(out, 4, 44, destination);
            }
            ThunkKind::ThumbV7AbsoluteLong => {
                half(out, 0, 0xf240);
                half(out, 2, 0x0c00);
                half(out, 4, 0xf2c0);
                half(out, 6, 0x0c00);
                half(out, 8, 0x4760);
                self.encode_no_symbol(out, 0, 47, destination);
                self.encode_no_symbol(out, 4, 48, destination);
            }
            ThunkKind::ArmV7PositionIndependentLong => {
                word(out, 0, 0xe30f_cff0);
                word(out, 4, 0xe340_c000);
                word(out, 8, 0xe08c_c00f);
                word(out, 12, 0xe12f_ff1c);
                let offset = destination.wrapping_sub(place).wrapping_sub(16);
                self.encode_no_symbol(out, 0, 45, offset);
                self.encode_no_symbol(out, 4, 46, offset);
            }
            ThunkKind::ThumbV7PositionIndependentLong => {
                for (at, value) in [
                    (0, 0xf64f),
                    (2, 0x7cf4),
                    (4, 0xf2c0),
                    (6, 0x0c00),
                    (8, 0x44fc),
                    (10, 0x4760),
                ] {
                    half(out, at, value);
                }
                let offset = destination.wrapping_sub(aligned_place).wrapping_sub(12);
                self.encode_no_symbol(out, 0, 49, offset);
                self.encode_no_symbol(out, 4, 50, offset);
            }
            ThunkKind::ThumbV6MAbsoluteLong => {
                for (at, value) in [(0, 0xb403), (2, 0x4801), (4, 0x9001), (6, 0xbd01)] {
                    half(out, at, value);
                }
                word(out, 8, destination as u32);
            }
            ThunkKind::ThumbV6MPositionIndependentLong => {
                for (at, value) in [
                    (0, 0xb401),
                    (2, 0x4802),
                    (4, 0x4684),
                    (6, 0xbc01),
                    (8, 0x44e7),
                    (10, 0x46c0),
                ] {
                    half(out, at, value);
                }
                word(
                    out,
                    12,
                    destination.wrapping_sub(aligned_place).wrapping_sub(12) as u32,
                );
            }
            ThunkKind::ArmV5LongLoadProgramCounter => {
                word(out, 0, 0xe51f_f004);
                word(out, 4, destination as u32);
            }
            ThunkKind::ArmV4AbsoluteLongExchange => {
                word(out, 0, 0xe59f_c000);
                word(out, 4, 0xe12f_ff1c);
                word(out, 8, destination as u32);
            }
            ThunkKind::ThumbV4AbsoluteLongExchange => {
                half(out, 0, 0x4778);
                half(out, 2, 0xe7fd);
                word(out, 4, 0xe51f_f004);
                word(out, 8, destination as u32);
            }
            ThunkKind::ThumbV4AbsoluteLong => {
                half(out, 0, 0x4778);
                half(out, 2, 0xe7fd);
                word(out, 4, 0xe59f_c000);
                word(out, 8, 0xe12f_ff1c);
                word(out, 12, destination as u32);
            }
            ThunkKind::ArmV4PositionIndependentLongExchange => {
                word(out, 0, 0xe59f_c004);
                word(out, 4, 0xe08f_c00c);
                word(out, 8, 0xe12f_ff1c);
                word(
                    out,
                    12,
                    destination.wrapping_sub(aligned_place).wrapping_sub(12) as u32,
                );
            }
            ThunkKind::ArmV4PositionIndependentLong => {
                word(out, 0, 0xe59f_c000);
                word(out, 4, 0xe08f_f00c);
                word(
                    out,
                    8,
                    destination.wrapping_sub(aligned_place).wrapping_sub(12) as u32,
                );
            }
            ThunkKind::ThumbV4PositionIndependentLongExchange => {
                half(out, 0, 0x4778);
                half(out, 2, 0xe7fd);
                word(out, 4, 0xe59f_c000);
                word(out, 8, 0xe08c_f00f);
                word(
                    out,
                    12,
                    destination.wrapping_sub(aligned_place).wrapping_sub(16) as u32,
                );
            }
            ThunkKind::ThumbV4PositionIndependentLong => {
                half(out, 0, 0x4778);
                half(out, 2, 0xe7fd);
                word(out, 4, 0xe59f_c004);
                word(out, 8, 0xe08f_c00c);
                word(out, 12, 0xe12f_ff1c);
                word(
                    out,
                    16,
                    destination.wrapping_sub(aligned_place).wrapping_sub(16) as u32,
                );
            }
            ThunkKind::Aarch64AbsoluteLong | ThunkKind::Aarch64PagedLong => {}
        }
    }
}
