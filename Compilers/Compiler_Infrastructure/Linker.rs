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
    pub initializer: Option<String>,
    pub finalizer: Option<String>,
    pub version_script: Vec<String>,
    pub export_dynamic_symbols: Vec<String>,
    pub dynamic_list: Vec<String>,
    pub pack_relative_relocations: bool,
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
            initializer: None,
            finalizer: None,
            version_script: Vec::new(),
            export_dynamic_symbols: Vec::new(),
            dynamic_list: Vec::new(),
            pack_relative_relocations: false,
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
        if let Some(value) = option_value(&mut cursor, argument, &["--build-id"])? {
            options.build_identifier = parse_build_identifier(&value)?;
            continue;
        }
        if argument == "--build-id" {
            options.build_identifier = BuildIdentifier::Fast;
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
    pub elf_symbols: Vec<elf::Symbol>,
    pub executable_stack_requested: bool,
    pub and_features: u32,
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
    pub got_constants: Vec<(u64, Expression, u32, i64, SymbolIndex)>,
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
    pub gnu_hash_bucket_count: usize,
    pub input_order: Vec<SectionIndex>,
    pub exception_frame_inputs: Vec<SectionIndex>,
    pub and_features: u32,
    pub elf_header_output: OutputIndex,
    pub program_header_output: OutputIndex,
    pub string_table: Vec<u8>,
    pub symbol_string_offsets: Vec<u32>,
    pub section_names: StringTable,
    pub synthesized: Vec<SymbolIndex>,
    pub relocation_read_only_segment: Option<usize>,
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
            got_constants: Vec::new(),
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
            gnu_hash_bucket_count: 1,
            input_order: Vec::new(),
            exception_frame_inputs: Vec::new(),
            and_features: 0,
            elf_header_output: usize::MAX,
            program_header_output: usize::MAX,
            string_table: vec![0],
            symbol_string_offsets: Vec::new(),
            section_names: StringTable::new(),
            synthesized: Vec::new(),
            relocation_read_only_segment: None,
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
        self.error(not_supported("linker script").with_context(format!("File: {display}")));
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
                let already = self.synthetic.contains_key(".ARM.attributes");
                if already || relocatable {
                    self.files[index].discarded[position] = true;
                    if relocatable && !already {
                        continue;
                    }
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
                    word,
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
                if self.target.architecture == Architecture::X86_64 {
                    section_type::X86_64_UNWIND
                } else {
                    section_type::PROGRAM_BITS
                },
                section_flag::ALLOCATE,
                word,
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
            Synthetic::ExceptionFrame => !self.exception_frame_inputs.is_empty(),
            Synthetic::ExceptionFrameHeader => !self.exception_frame_inputs.is_empty(),
            Synthetic::VersionNeeds => self.version_need_count() > 0,
            Synthetic::VersionSymbols => self.version_need_count() > 0,
            Synthetic::ArmExceptionIndex => false,
            _ => true,
        }
    }

    pub fn version_need_count(&self) -> usize {
        0
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
            Architecture::Arm => arm_implicit_addend(kind, data, at, encoding),
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

    pub fn add_relative_relocation(
        &mut self,
        section: SectionIndex,
        offset: u64,
        symbol: SymbolIndex,
        addend: i64,
        expression: Expression,
        kind: u32,
    ) {
        let relocation = DynamicRelocation {
            kind: self.target.relative_relocation,
            section,
            offset,
            symbol: Some(symbol),
            use_symbol_value: false,
            addend,
            expression,
        };
        if self.options.pack_relative_relocations
            && self.sections[section].alignment >= 2
            && offset.is_multiple_of(2)
        {
            self.relative_relocations.push(relocation);
            return;
        }
        self.dynamic_relocations.push(relocation);
        if !self.target.uses_addends {
            self.add_relocation(section, expression, kind, offset, addend, symbol);
        }
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
        self.dynamic_relocations.push(DynamicRelocation {
            kind: dynamic_kind,
            section,
            offset,
            symbol: Some(symbol),
            use_symbol_value: true,
            addend,
            expression: Expression::Addend,
        });
        if !self.target.uses_addends {
            self.add_relocation(section, Expression::Addend, kind, offset, addend, symbol);
        }
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
                self.dynamic_relocations.push(DynamicRelocation {
                    kind: self.target.relative_relocation,
                    section,
                    offset,
                    symbol: Some(symbol),
                    use_symbol_value: false,
                    addend,
                    expression,
                });
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
        let frames = self.exception_frame_inputs.clone();
        for section in frames {
            if self.sections[section].live {
                self.scan_section(section, &mut undefined);
            }
        }
        undefined
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

fn arm_implicit_addend(kind: u32, data: &[u8], at: usize, encoding: Encoding) -> i64 {
    let word = u64::from(encoding.read_u32(data, at).unwrap_or(0));
    match kind {
        2 | 3 | 9 | 13 | 17 | 18 | 19 | 21 | 22 | 23 | 24 | 25 | 26 | 38 | 41 | 55 | 56 | 96
        | 104 | 105 | 106 | 107 | 108 | 160 | 31 => sign_extend(word, 32),
        42 => sign_extend(word, 31),
        1 | 27 | 28 | 29 => sign_extend((word & 0x00ff_ffff) << 2, 26),
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
                self.got_constants.push((
                    offset,
                    Expression::Addend,
                    self.target.symbolic_relocation,
                    1,
                    usize::MAX,
                ));
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
                self.add_addend_only_if_non_preemptible(kind, got, offset, id);
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
                        self.got_constants.push((
                            offset,
                            Expression::Addend,
                            self.target.symbolic_relocation,
                            1,
                            id,
                        ));
                    }
                    self.got_constants.push((
                        offset + word,
                        Expression::Absolute,
                        self.target.offset_relocation,
                        0,
                        id,
                    ));
                }
            }
            if flags & needs::GOT_DYNAMIC_OFFSET != 0 {
                self.allocate_got_slot(id);
                let offset = self.symbols[id].got_index.unwrap_or(0) * self.target.got_entry_size;
                self.got_constants.push((
                    offset,
                    Expression::Absolute,
                    self.target.offset_relocation,
                    0,
                    id,
                ));
            }
            if flags & needs::TLS_INITIAL_EXECUTABLE != 0 {
                self.allocate_got_slot(id);
                let offset = self.symbols[id].got_index.unwrap_or(0) * self.target.got_entry_size;
                if !self.symbols[id].preemptible && !self.is_shared() {
                    self.got_constants.push((
                        offset,
                        Expression::ThreadPointerRelative,
                        self.target.symbolic_relocation,
                        0,
                        id,
                    ));
                } else {
                    let kind = self.target.tls_got_relocation;
                    self.add_addend_only_if_non_preemptible(kind, got, offset, id);
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
    ) {
        if self.symbols[id].preemptible {
            self.add_symbol_relocation_plain(kind, section, offset, id);
        } else {
            self.dynamic_relocations.push(DynamicRelocation {
                kind,
                section,
                offset,
                symbol: Some(id),
                use_symbol_value: false,
                addend: 0,
                expression: Expression::Absolute,
            });
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
            self.got_constants.push((
                offset,
                Expression::Absolute,
                self.target.symbolic_relocation,
                0,
                id,
            ));
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

    fn add_copy_relocation(&mut self, id: SymbolIndex) {
        let size = self.symbols[id].size;
        let alignment = self.symbols[id].shared_alignment.max(1);
        let read_only = false;
        let name = if read_only { ".bss.rel.ro" } else { ".bss" };
        let Some(section) = self.synthetic_section(name) else {
            return;
        };
        let offset = align_to(self.sections[section].size, alignment);
        self.sections[section].size = offset + size;
        self.sections[section].alignment = self.sections[section].alignment.max(alignment);
        let file = self.symbols[id].file;
        let value = self.symbols[id].value;
        let aliases: Vec<SymbolIndex> = self
            .global_symbols()
            .into_iter()
            .filter(|other| {
                let symbol = &self.symbols[*other];
                symbol.is_shared()
                    && symbol.file == file
                    && symbol.value == value
                    && symbol.is_object()
            })
            .collect();
        let mut targets = aliases;
        if !targets.contains(&id) {
            targets.push(id);
        }
        for target in targets {
            let entry = &mut self.symbols[target];
            entry.state = SymbolState::Defined;
            entry.section = Some(section);
            entry.value = offset;
            entry.used_in_regular_object = true;
            entry.needs &= !needs::COPY;
            entry.exported = true;
        }
        self.dynamic_relocations.push(DynamicRelocation {
            kind: self.target.copy_relocation,
            section,
            offset,
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
                self.dynamic_symbols.push(id);
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
                self.sort_symbol_table(Some(id));
                return;
            }
        }
        self.sort_symbol_table(None);
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
            if Some(output) == padding {
                dot = align_to(dot, common);
            }
            self.outputs[output].size = dot - start;
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
        ];
        for (name, size) in sizes {
            if let Some(section) = self.synthetic_section(name) {
                self.sections[section].size = size;
            }
        }
    }

    pub fn section_address(&self, section: SectionIndex, offset: u64) -> u64 {
        let data = &self.sections[section];
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
    fn scan_x86(
        &mut self,
        section: SectionIndex,
        relocations: &[RawRelocation],
        undefined: &mut Vec<UndefinedReference>,
    ) {
        let _ = (section, relocations, undefined);
        self.error(not_supported("i386 relocations"));
    }

    fn scan_aarch64(
        &mut self,
        section: SectionIndex,
        relocations: &[RawRelocation],
        undefined: &mut Vec<UndefinedReference>,
    ) {
        let _ = (section, relocations, undefined);
        self.error(not_supported("AArch64 relocations"));
    }

    fn scan_arm(
        &mut self,
        section: SectionIndex,
        relocations: &[RawRelocation],
        undefined: &mut Vec<UndefinedReference>,
    ) {
        let _ = (section, relocations, undefined);
        self.error(not_supported("ARM relocations"));
    }

    fn relocate(
        &mut self,
        buffer: &mut [u8],
        at: usize,
        relocation: &Relocation,
        value: u64,
        location: &str,
    ) {
        if self.target.architecture == Architecture::X86_64 {
            self.relocate_x86_64(buffer, at, relocation, value, location);
        }
    }

    fn apply_relocations(&mut self, section: SectionIndex, buffer: &mut [u8]) {
        let relocations = std::mem::take(&mut self.sections[section].relocations);
        let base = self.section_address(section, 0);
        for relocation in &relocations {
            if relocation.expression == Expression::None {
                continue;
            }
            let at = relocation.offset as usize;
            let place = base.wrapping_add(relocation.offset);
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

    fn write_got(&mut self, section: SectionIndex, buffer: &mut [u8]) {
        let constants = self.got_constants.clone();
        for (offset, expression, kind, addend, symbol) in constants {
            let relocation = Relocation {
                expression,
                kind,
                offset,
                addend,
                symbol: if symbol == usize::MAX { 0 } else { symbol },
            };
            let place = self.section_address(section, offset);
            let value = if symbol == usize::MAX {
                addend as u64
            } else {
                self.evaluate(&relocation, place)
            };
            if (offset as usize) < buffer.len() {
                self.write_word(buffer, offset as usize, value);
            }
        }
    }

    fn write_got_plt(&self, buffer: &mut [u8]) {
        let entry = self.target.got_entry_size as usize;
        if self.target.architecture == Architecture::X86_64
            || self.target.architecture == Architecture::X86
        {
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
        for (position, id) in self.got_plt_entries.iter().enumerate() {
            let at = (position + self.target.got_plt_header_entries as usize) * entry;
            let value = match self.target.architecture {
                Architecture::X86_64 | Architecture::X86 => self.plt_address(*id) + 6,
                _ => self.synthetic_address(".plt"),
            };
            if at + entry <= buffer.len() {
                self.write_word(buffer, at, value);
            }
        }
    }

    fn write_plt(&self, buffer: &mut [u8]) {
        if self.target.architecture != Architecture::X86_64 {
            return;
        }
        let plt = self.synthetic_address(".plt");
        let got_plt = self.synthetic_address(".got.plt");
        if buffer.len() >= 16 {
            buffer[..16].copy_from_slice(&[
                0xff, 0x35, 0, 0, 0, 0, 0xff, 0x25, 0, 0, 0, 0, 0x0f, 0x1f, 0x40, 0x00,
            ]);
            buffer[2..6].copy_from_slice(
                &((got_plt.wrapping_sub(plt).wrapping_add(2)) as u32).to_le_bytes(),
            );
            buffer[8..12].copy_from_slice(
                &((got_plt.wrapping_sub(plt).wrapping_add(4)) as u32).to_le_bytes(),
            );
        }
        for (position, id) in self.plt_entries.iter().enumerate() {
            let at = 16 + position * 16;
            let entry_address = plt + at as u64;
            self.write_x86_64_plt_entry(
                &mut buffer[at..at + 16],
                *id,
                entry_address,
                position as u64,
            );
        }
    }

    fn write_x86_64_plt_entry(
        &self,
        out: &mut [u8],
        id: SymbolIndex,
        entry_address: u64,
        index: u64,
    ) {
        out.copy_from_slice(&[0xff, 0x25, 0, 0, 0, 0, 0x68, 0, 0, 0, 0, 0xe9, 0, 0, 0, 0]);
        let got_plt = self.got_plt_address(id);
        out[2..6].copy_from_slice(
            &((got_plt.wrapping_sub(entry_address).wrapping_sub(6)) as u32).to_le_bytes(),
        );
        out[7..11].copy_from_slice(&(index as u32).to_le_bytes());
        let plt = self.synthetic_address(".plt");
        out[12..16].copy_from_slice(
            &((plt.wrapping_sub(entry_address).wrapping_sub(16)) as u32).to_le_bytes(),
        );
    }

    fn write_indirect_plt(&self, buffer: &mut [u8]) {
        if self.target.architecture != Architecture::X86_64 {
            return;
        }
        let iplt = self.synthetic_address(".iplt");
        for (position, id) in self.indirect_plt_entries.iter().enumerate() {
            let at = position * 16;
            let entry_address = iplt + at as u64;
            self.write_x86_64_plt_entry(
                &mut buffer[at..at + 16],
                *id,
                entry_address,
                position as u64,
            );
        }
    }

    fn encode_dynamic_relocations(&self, entries: &[DynamicRelocation]) -> Vec<u8> {
        let encoding = self.encoding();
        let rela = self.target.uses_addends;
        let mut out = Vec::new();
        for entry in entries {
            let offset = self.section_address(entry.section, entry.offset);
            let symbol_index = match (entry.use_symbol_value, entry.symbol) {
                (true, Some(id)) => self.symbols[id].dynamic_symbol_index as u32,
                _ => 0,
            };
            let addend = self.dynamic_addend(entry);
            elf::encode_relocation(
                encoding,
                &elf::Relocation {
                    offset,
                    kind: entry.kind,
                    symbol: symbol_index,
                    addend: addend as i64,
                },
                rela,
                &mut out,
            );
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
        if !symbol.is_defined() || symbol.needs & needs::COPY != 0 && !symbol.is_defined() {
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
            Synthetic::Got => self.write_got(section, &mut buffer),
            Synthetic::GotPlt => self.write_got_plt(&mut buffer),
            Synthetic::Plt => self.write_plt(&mut buffer),
            Synthetic::IndirectPlt => self.write_indirect_plt(&mut buffer),
            Synthetic::DynamicRelocations => {
                let entries = self.dynamic_relocations.clone();
                buffer = self.encode_dynamic_relocations(&entries);
            }
            Synthetic::PltRelocations => {
                let entries = self.plt_relocations.clone();
                buffer = self.encode_dynamic_relocations(&entries);
            }
            Synthetic::SymbolTable => buffer = self.encode_symbol_table(),
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
        out
    }

    pub fn header_flags(&self) -> u32 {
        match self.target.architecture {
            Architecture::Arm => 0x0500_0000,
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
                    self.outputs[output].link = index_of(".dynstr", self)
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
        self.create_synthetic_sections();
        self.create_output_sections();
        self.finalize_input_sections();
        if self.failed() {
            return None;
        }
        self.add_start_end_symbols();
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
        self.finalize_synthetic_sizes();
        self.fix_section_alignments();
        self.assign_addresses();
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
