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
    pub symbolic_non_weak: bool,
    pub symbolic_non_weak_functions: bool,
    pub undefined_version: bool,
    pub dynamic_list_files: Vec<String>,
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
    pub start_stop_garbage_collection: bool,
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
            symbolic_non_weak: false,
            symbolic_non_weak_functions: false,
            undefined_version: false,
            dynamic_list_files: Vec::new(),
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
            start_stop_garbage_collection: true,
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

fn unquote(token: &str) -> String {
    if token.len() >= 2 && token.starts_with('"') && token.ends_with('"') {
        token[1..token.len() - 1].to_string()
    } else {
        token.to_string()
    }
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
        "Bsymbolic-non-weak",
        "Bsymbolic-non-weak-functions",
        "Bno-symbolic",
        "undefined-version",
        "no-undefined-version",
        "dynamic-list",
        "export-dynamic-symbol",
        "export-dynamic-symbol-list",
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
        if let Some(value) = option_value(&mut cursor, argument, &["--version-script"])? {
            options.version_script.push(value);
            continue;
        }
        if let Some(value) = option_value(
            &mut cursor,
            argument,
            &["--dynamic-list", "--export-dynamic-symbol-list"],
        )? {
            options.dynamic_list_files.push(value);
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
            "--Bsymbolic" => {
                options.symbolic = true;
                options.symbolic_functions = false;
                options.symbolic_non_weak = false;
                options.symbolic_non_weak_functions = false;
            }
            "--Bsymbolic-functions" => {
                options.symbolic = false;
                options.symbolic_functions = true;
                options.symbolic_non_weak = false;
                options.symbolic_non_weak_functions = false;
            }
            "--Bsymbolic-non-weak" => {
                options.symbolic = false;
                options.symbolic_functions = false;
                options.symbolic_non_weak = true;
                options.symbolic_non_weak_functions = false;
            }
            "--Bsymbolic-non-weak-functions" => {
                options.symbolic = false;
                options.symbolic_functions = false;
                options.symbolic_non_weak = false;
                options.symbolic_non_weak_functions = true;
            }
            "--Bno-symbolic" => {
                options.symbolic = false;
                options.symbolic_functions = false;
                options.symbolic_non_weak = false;
                options.symbolic_non_weak_functions = false;
            }
            "--undefined-version" => options.undefined_version = true,
            "--no-undefined-version" => options.undefined_version = false,
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
        "start-stop-gc" => options.start_stop_garbage_collection = true,
        "nostart-stop-gc" => options.start_stop_garbage_collection = false,
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
    pub used: bool,
    pub version_suffix_at: Option<usize>,
    pub version_script_assigned: bool,
    pub in_dynamic_list: bool,
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
            used: false,
            version_suffix_at: None,
            version_script_assigned: false,
            in_dynamic_list: false,
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
    pub next_in_group: Option<SectionIndex>,
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
            next_in_group: None,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolPattern {
    pub name: String,
    pub extern_cpp: bool,
    pub wildcard: bool,
}

#[derive(Clone, Debug)]
pub struct VersionDefinition {
    pub name: String,
    pub identifier: u16,
    pub non_local: Vec<SymbolPattern>,
    pub local: Vec<SymbolPattern>,
}

pub fn has_wildcard(text: &str) -> bool {
    text.contains(['?', '*', '['])
}

pub fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    let mut pattern_at = 0usize;
    let mut text_at = 0usize;
    let mut star: Option<(usize, usize)> = None;
    let class_end = |start: usize| -> Option<usize> {
        let mut at = start + 1;
        if at < pattern.len() && (pattern[at] == '!' || pattern[at] == '^') {
            at += 1;
        }
        if at < pattern.len() && pattern[at] == ']' {
            at += 1;
        }
        while at < pattern.len() && pattern[at] != ']' {
            at += 1;
        }
        (at < pattern.len()).then_some(at)
    };
    let class_matches = |start: usize, end: usize, character: char| -> bool {
        let mut at = start + 1;
        let negated = pattern[at] == '!' || pattern[at] == '^';
        if negated {
            at += 1;
        }
        let mut found = false;
        let mut first = true;
        while at < end {
            if !first && pattern[at] == ']' {
                break;
            }
            first = false;
            let low = pattern[at];
            if at + 2 < end && pattern[at + 1] == '-' {
                let high = pattern[at + 2];
                if low <= character && character <= high {
                    found = true;
                }
                at += 3;
            } else {
                if low == character {
                    found = true;
                }
                at += 1;
            }
        }
        found != negated
    };
    while text_at < text.len() {
        let mut advanced = false;
        if pattern_at < pattern.len() {
            match pattern[pattern_at] {
                '*' => {
                    star = Some((pattern_at, text_at));
                    pattern_at += 1;
                    continue;
                }
                '?' => {
                    pattern_at += 1;
                    text_at += 1;
                    advanced = true;
                }
                '[' => {
                    if let Some(end) = class_end(pattern_at) {
                        if class_matches(pattern_at, end, text[text_at]) {
                            pattern_at = end + 1;
                            text_at += 1;
                            advanced = true;
                        }
                    } else if text[text_at] == '[' {
                        pattern_at += 1;
                        text_at += 1;
                        advanced = true;
                    }
                }
                '\\' if pattern_at + 1 < pattern.len() => {
                    if pattern[pattern_at + 1] == text[text_at] {
                        pattern_at += 2;
                        text_at += 1;
                        advanced = true;
                    }
                }
                character => {
                    if character == text[text_at] {
                        pattern_at += 1;
                        text_at += 1;
                        advanced = true;
                    }
                }
            }
        }
        if advanced {
            continue;
        }
        match star {
            Some((star_pattern, star_text)) => {
                pattern_at = star_pattern + 1;
                text_at = star_text + 1;
                star = Some((star_pattern, star_text + 1));
            }
            None => return false,
        }
    }
    while pattern_at < pattern.len() && pattern[pattern_at] == '*' {
        pattern_at += 1;
    }
    pattern_at == pattern.len()
}

pub fn pattern_matches(pattern: &SymbolPattern, name: &str) -> bool {
    if pattern.wildcard {
        glob_match(&pattern.name, name)
    } else {
        pattern.name == name
    }
}

pub struct ScriptLexer<'t> {
    text: &'t str,
    at: usize,
    pub version_mode: bool,
}

impl<'t> ScriptLexer<'t> {
    pub fn new(text: &'t str, version_mode: bool) -> ScriptLexer<'t> {
        ScriptLexer {
            text,
            at: 0,
            version_mode,
        }
    }

    fn skip_space(&mut self) -> Result<(), String> {
        let bytes = self.text.as_bytes();
        loop {
            if self.text[self.at..].starts_with("/*") {
                match self.text[self.at + 2..].find("*/") {
                    Some(end) => self.at += end + 4,
                    None => return Err("unclosed comment in a linker script".to_string()),
                }
                continue;
            }
            if bytes.get(self.at) == Some(&b'#') {
                while self.at < bytes.len() && bytes[self.at] != b'\n' {
                    self.at += 1;
                }
                continue;
            }
            if bytes
                .get(self.at)
                .is_some_and(|byte| byte.is_ascii_whitespace())
            {
                self.at += 1;
                continue;
            }
            return Ok(());
        }
    }

    pub fn next_token(&mut self) -> Result<Option<String>, String> {
        self.skip_space()?;
        let bytes = self.text.as_bytes();
        if self.at >= bytes.len() {
            return Ok(None);
        }
        let start = self.at;
        if bytes[start] == b'"' {
            return match self.text[start + 1..].find('"') {
                Some(end) => {
                    self.at = start + end + 2;
                    Ok(Some(self.text[start..self.at].to_string()))
                }
                None => Err("unclosed quote".to_string()),
            };
        }
        let word = |byte: u8| byte.is_ascii_alphanumeric() || b"_.$/\\~=+[]*?-!^".contains(&byte);
        let mut end = start;
        while end < bytes.len() {
            let byte = bytes[end];
            if byte == b':' {
                if self.version_mode {
                    if bytes.get(end + 1) == Some(&b':') {
                        end += 2;
                        continue;
                    }
                    break;
                }
                end += 1;
                continue;
            }
            if !word(byte) {
                break;
            }
            end += 1;
        }
        if end == start {
            end = start + 1;
        }
        self.at = end;
        Ok(Some(self.text[start..end].to_string()))
    }

    pub fn peek(&mut self) -> Result<Option<String>, String> {
        let saved = self.at;
        let token = self.next_token();
        self.at = saved;
        token
    }

    pub fn consume(&mut self, expected: &str) -> Result<bool, String> {
        if self.peek()?.as_deref() == Some(expected) {
            self.next_token()?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn expect(&mut self, expected: &str) -> Result<(), String> {
        match self.next_token()? {
            Some(token) if token == expected => Ok(()),
            Some(token) => Err(format!("{expected} expected, but got {token}")),
            None => Err("unexpected EOF".to_string()),
        }
    }

    pub fn till(&mut self, end: &str) -> Result<Option<String>, String> {
        match self.next_token()? {
            Some(token) if token == end => Ok(None),
            Some(token) => Ok(Some(token)),
            None => Err("unexpected EOF".to_string()),
        }
    }
}

pub type SymbolPatterns = (Vec<SymbolPattern>, Vec<SymbolPattern>);

fn read_version_symbols(lexer: &mut ScriptLexer) -> Result<SymbolPatterns, String> {
    let mut locals = Vec::new();
    let mut globals = Vec::new();
    let mut into_locals = false;
    while let Some(token) = lexer.till("}")? {
        if token == "extern" {
            let language = lexer.next_token()?.unwrap_or_default();
            let extern_cpp = language == "\"C++\"";
            if !extern_cpp && language != "\"C\"" {
                return Err("Unknown language".to_string());
            }
            lexer.expect("{")?;
            let mut patterns = Vec::new();
            while let Some(token) = lexer.till("}")? {
                patterns.push(SymbolPattern {
                    name: unquote(&token),
                    extern_cpp,
                    wildcard: !token.starts_with('"') && has_wildcard(&token),
                });
                if lexer.consume("}")? {
                    break;
                }
                lexer.expect(";")?;
            }
            if into_locals {
                locals.extend(patterns);
            } else {
                globals.extend(patterns);
            }
        } else {
            if token == "local" && lexer.consume(":")? {
                into_locals = true;
                continue;
            }
            if token == "global" && lexer.consume(":")? {
                into_locals = false;
                continue;
            }
            let pattern = SymbolPattern {
                name: unquote(&token),
                extern_cpp: false,
                wildcard: has_wildcard(&token),
            };
            if into_locals {
                locals.push(pattern);
            } else {
                globals.push(pattern);
            }
        }
        lexer.expect(";")?;
    }
    Ok((locals, globals))
}

pub fn read_version_command(
    lexer: &mut ScriptLexer,
    definitions: &mut Vec<VersionDefinition>,
) -> Result<(), String> {
    let saved = lexer.version_mode;
    lexer.version_mode = true;
    let result = (|| {
        if lexer.consume("{")? {
            let (locals, globals) = read_version_symbols(lexer)?;
            definitions[0].local.extend(locals);
            definitions[1].non_local.extend(globals);
            lexer.expect(";")?;
            return Ok(());
        }
        if lexer.peek()?.is_none() {
            return Err("unexpected EOF".to_string());
        }
        while let Some(token) = lexer.peek()? {
            if token == "}" {
                break;
            }
            lexer.next_token()?;
            if token == "{" {
                return Err("anonymous version definition is used in combination with other version definitions".to_string());
            }
            lexer.expect("{")?;
            let (locals, globals) = read_version_symbols(lexer)?;
            let identifier = definitions.len() as u16;
            definitions.push(VersionDefinition {
                name: token,
                identifier,
                non_local: globals,
                local: locals,
            });
            if lexer.next_token()?.as_deref() != Some(";") {
                lexer.expect(";")?;
            }
        }
        Ok(())
    })();
    lexer.version_mode = saved;
    result
}

pub fn read_version_script(
    text: &str,
    definitions: &mut Vec<VersionDefinition>,
) -> Result<(), String> {
    let mut lexer = ScriptLexer::new(text, true);
    read_version_command(&mut lexer, definitions)?;
    match lexer.peek()? {
        Some(token) => Err(format!("EOF expected, but got {token}")),
        None => Ok(()),
    }
}

pub fn read_dynamic_list(text: &str) -> Result<Vec<SymbolPattern>, String> {
    let mut lexer = ScriptLexer::new(text, true);
    lexer.expect("{")?;
    let (locals, globals) = read_version_symbols(&mut lexer)?;
    lexer.expect(";")?;
    if let Some(token) = lexer.peek()? {
        return Err(format!("EOF expected, but got {token}"));
    }
    if !locals.is_empty() {
        return Err("\"local:\" scope not supported in --dynamic-list".to_string());
    }
    Ok(globals)
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
    pub version_definitions: Vec<VersionDefinition>,
    pub dynamic_list_patterns: Vec<SymbolPattern>,
    pub version_definition_name: u32,
    pub version_definition_names: Vec<u32>,
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
            version_definitions: vec![
                VersionDefinition {
                    name: "local".to_string(),
                    identifier: elf::version_index::LOCAL,
                    non_local: Vec::new(),
                    local: Vec::new(),
                },
                VersionDefinition {
                    name: "global".to_string(),
                    identifier: elf::version_index::GLOBAL,
                    non_local: Vec::new(),
                    local: Vec::new(),
                },
            ],
            dynamic_list_patterns: Vec::new(),
            version_definition_name: 0,
            version_definition_names: Vec::new(),
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
        let stem = match name.find('@') {
            Some(at) if name[at + 1..].starts_with('@') => &name[..at],
            _ => name,
        };
        if let Some(found) = self.names.get(stem).copied() {
            if stem.len() != name.len() {
                self.symbols[found].name = name.to_string();
            }
            return found;
        }
        let index = self.symbols.len();
        self.symbols.push(SymbolData::new(name));
        self.names.insert(stem.to_string(), index);
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
        if let Err(reason) = self.read_input_commands(&text, path, state, group, target_reference) {
            self.script_error(display, &reason);
        }
    }

    fn read_input_commands(
        &mut self,
        text: &str,
        path: &str,
        state: LoadState,
        group: usize,
        target_reference: &mut Option<String>,
    ) -> Result<(), String> {
        let mut lexer = ScriptLexer::new(text, false);
        let parenthesized = |lexer: &mut ScriptLexer| -> Result<Vec<String>, String> {
            lexer.expect("(")?;
            let mut values = Vec::new();
            while let Some(token) = lexer.till(")")? {
                if token != "," {
                    values.push(unquote(&token));
                }
            }
            Ok(values)
        };
        while let Some(command) = lexer.next_token()? {
            match command.as_str() {
                ";" => {}
                "GROUP" | "INPUT" => {
                    lexer.expect("(")?;
                    while let Some(token) = lexer.till(")")? {
                        match token.as_str() {
                            "," => {}
                            "AS_NEEDED" => {
                                let mut inner = state;
                                inner.as_needed = true;
                                for name in parenthesized(&mut lexer)? {
                                    self.add_script_file(
                                        &name,
                                        path,
                                        inner,
                                        group,
                                        target_reference,
                                    );
                                }
                            }
                            _ => {
                                let name = unquote(&token);
                                self.add_script_file(&name, path, state, group, target_reference);
                            }
                        }
                    }
                }
                "OUTPUT_FORMAT" | "OUTPUT_ARCH" | "TARGET" => {
                    parenthesized(&mut lexer)?;
                }
                "SEARCH_DIR" => {
                    if let Some(folder) = parenthesized(&mut lexer)?.first() {
                        self.options.library_paths.push(folder.clone());
                    }
                }
                "ENTRY" => {
                    if let Some(name) = parenthesized(&mut lexer)?.first() {
                        if self.options.entry.is_none() {
                            self.options.entry = Some(name.clone());
                        }
                    }
                }
                "EXTERN" => {
                    lexer.expect("(")?;
                    while let Some(token) = lexer.till(")")? {
                        let name = unquote(&token);
                        let id = self.add_unused_undefined(&name);
                        self.symbols[id].referenced = true;
                        self.options.undefined.push(name);
                    }
                }
                "VERSION" => {
                    lexer.expect("{")?;
                    let mut definitions = std::mem::take(&mut self.version_definitions);
                    let result = read_version_command(&mut lexer, &mut definitions);
                    self.version_definitions = definitions;
                    result?;
                    lexer.expect("}")?;
                }
                other => {
                    self.error(
                        not_supported("linker script command")
                            .with_context(format!("Command: {other}"))
                            .with_context(format!("File: {path}")),
                    );
                    return Ok(());
                }
            }
        }
        Ok(())
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
                flags: if relocatable {
                    header.flags
                } else {
                    header.flags & !section_flag::GROUP
                },
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
                next_in_group: None,
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
        if !relocatable {
            for group in self.files[index].kept_groups.clone() {
                let Ok(group) = self.files[index].elf.group(group) else {
                    continue;
                };
                let members: Vec<SectionIndex> = group
                    .members
                    .iter()
                    .filter_map(|member| {
                        self.files[index]
                            .sections
                            .get(*member as usize)
                            .copied()
                            .flatten()
                    })
                    .collect();
                if !members
                    .iter()
                    .any(|member| self.sections[*member].flags & section_flag::ALLOCATE != 0)
                {
                    continue;
                }
                for (position, member) in members.iter().enumerate() {
                    let next = members[(position + 1) % members.len()];
                    self.sections[*member].next_in_group = Some(next);
                }
            }
        }
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
                if self.has_named_versions() {
                    self.add_synthetic(
                        ".gnu.version_d",
                        section_type::GNU_VERSION_DEFINITIONS,
                        section_flag::ALLOCATE,
                        4,
                        Synthetic::VersionDefinitions,
                    );
                }
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
            Synthetic::VersionSymbols => self.version_need_count() > 0 || self.has_named_versions(),
            Synthetic::VersionDefinitions => self.has_named_versions(),
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
        self.version_definitions.len() - 1
    }

    pub fn has_named_versions(&self) -> bool {
        self.version_definitions.len() > 2
    }

    fn is_shared_file(&self, file: Option<FileIndex>) -> bool {
        file.is_some_and(|file| file < self.files.len() && self.shared_files.contains(&file))
    }

    pub fn dynamic_symbol_name(&self, id: SymbolIndex) -> String {
        let symbol = &self.symbols[id];
        match symbol.version_suffix_at {
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
        if self.has_named_versions() {
            let file_name = self
                .options
                .shared_object_name
                .clone()
                .unwrap_or_else(|| self.options.output.clone());
            self.version_definition_name = self.add_dynamic_string(&file_name);
            self.version_definition_names.clear();
            for definition in self.version_definitions.clone().iter().skip(2) {
                let offset = self.add_dynamic_string(&definition.name);
                self.version_definition_names.push(offset);
            }
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
        if self.live_synthetic(".gnu.version")
            && (self.version_need_count() > 0 || self.has_named_versions())
        {
            add(
                tag::VERSION_SYMBOLS,
                self.dynamic_tables_section_address(".gnu.version"),
            );
        }
        if self.live_synthetic(".gnu.version_d") {
            add(
                tag::VERSION_DEFINITIONS,
                self.dynamic_tables_section_address(".gnu.version_d"),
            );
            add(
                tag::VERSION_DEFINITION_COUNT,
                self.version_definition_count() as u64,
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

    fn encode_version_definitions(&self) -> Vec<u8> {
        let encoding = self.encoding();
        let mut out = Vec::new();
        let file_name = self
            .options
            .shared_object_name
            .clone()
            .unwrap_or_else(|| self.options.output.clone());
        let mut entries: Vec<(u16, String, u32)> =
            vec![(1, file_name, self.version_definition_name)];
        for (definition, offset) in self
            .version_definitions
            .iter()
            .skip(2)
            .zip(&self.version_definition_names)
        {
            entries.push((definition.identifier, definition.name.clone(), *offset));
        }
        let count = entries.len();
        for (position, (index, name, offset)) in entries.into_iter().enumerate() {
            encoding.put_u16(&mut out, 1);
            encoding.put_u16(
                &mut out,
                if index == 1 {
                    elf::version_flag::BASE
                } else {
                    0
                },
            );
            encoding.put_u16(&mut out, index);
            encoding.put_u16(&mut out, 1);
            encoding.put_u32(&mut out, elf::system_five_hash(name.as_bytes()));
            encoding.put_u32(&mut out, 20);
            encoding.put_u32(&mut out, if position + 1 == count { 0 } else { 28 });
            encoding.put_u32(&mut out, offset);
            encoding.put_u32(&mut out, 0);
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
        if self.options.symbolic
            || !self.options.dynamic_list_files.is_empty()
            || (self.options.symbolic_non_weak && symbol.binding != symbol_binding::WEAK)
            || (self.options.symbolic_functions && symbol.is_function())
            || (self.options.symbolic_non_weak_functions
                && symbol.is_function()
                && symbol.binding != symbol_binding::WEAK)
        {
            return symbol.in_dynamic_list;
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

    pub fn read_version_inputs(&mut self) {
        for path in self.options.version_script.clone() {
            let Some(bytes) = self.read_script_file(&path) else {
                self.error(file_missing(&path));
                continue;
            };
            let text = String::from_utf8_lossy(&bytes).to_string();
            let mut definitions = std::mem::take(&mut self.version_definitions);
            let result = read_version_script(&text, &mut definitions);
            self.version_definitions = definitions;
            if let Err(reason) = result {
                self.script_error(&path, &reason);
            }
        }
        for name in self.options.export_dynamic_symbols.clone() {
            self.dynamic_list_patterns.push(SymbolPattern {
                wildcard: has_wildcard(&name),
                name,
                extern_cpp: false,
            });
        }
        for path in self.options.dynamic_list_files.clone() {
            let Some(bytes) = self.read_script_file(&path) else {
                self.error(file_missing(&path));
                continue;
            };
            let text = String::from_utf8_lossy(&bytes).to_string();
            match read_dynamic_list(&text) {
                Ok(patterns) => self.dynamic_list_patterns.extend(patterns),
                Err(reason) => self.script_error(&path, &reason),
            }
        }
        let uses_cpp = self
            .version_definitions
            .iter()
            .flat_map(|definition| definition.non_local.iter().chain(&definition.local))
            .chain(&self.dynamic_list_patterns)
            .any(|pattern| pattern.extern_cpp);
        if uses_cpp {
            self.error(not_supported("extern \"C++\" symbol patterns"));
        }
    }

    fn read_script_file(&self, path: &str) -> Option<Vec<u8>> {
        if let Some(bytes) = (self.source)(&self.resolve_path(path)) {
            return Some(bytes);
        }
        for folder in &self.options.library_paths {
            let folder = self.resolve_path(folder);
            if let Some(bytes) = (self.source)(&format!("{folder}/{path}")) {
                return Some(bytes);
            }
        }
        None
    }

    fn can_be_versioned(&self, id: SymbolIndex) -> bool {
        let symbol = &self.symbols[id];
        symbol.is_defined() || symbol.is_common() || symbol.is_lazy()
    }

    fn find_by_version(&self, name: &str) -> Option<SymbolIndex> {
        self.find(name).filter(|id| self.can_be_versioned(*id))
    }

    fn version_label(&self, version: u16) -> String {
        match version {
            elf::version_index::LOCAL => "VER_NDX_LOCAL".to_string(),
            elf::version_index::GLOBAL => "VER_NDX_GLOBAL".to_string(),
            other => format!(
                "version '{}'",
                self.version_definitions
                    .get(other as usize)
                    .map(|definition| definition.name.as_str())
                    .unwrap_or("")
            ),
        }
    }

    fn assign_exact_version(&mut self, name: &str, version: u16) -> bool {
        let Some(id) = self.find_by_version(name) else {
            return false;
        };
        if self.symbols[id].name.contains('@') {
            return true;
        }
        if !self.symbols[id].version_script_assigned {
            self.symbols[id].version_script_assigned = true;
            self.symbols[id].version = version;
        }
        if self.symbols[id].version != version {
            let previous = self.version_label(self.symbols[id].version);
            let requested = self.version_label(version);
            self.warn(
                caution(
                    "EI244",
                    "A version script assigns a symbol to a second version.",
                )
                .with_context(format!("Symbol: {name}"))
                .with_context(format!("Previous: {previous}"))
                .with_context(format!("Version: {requested}")),
            );
        }
        true
    }

    pub fn scan_version_script(&mut self) {
        let definitions = self.version_definitions.clone();
        for definition in &definitions {
            let exact =
                |context: &mut Context, pattern: &SymbolPattern, version: u16, label: &str| {
                    let mut found = context.assign_exact_version(&pattern.name, version);
                    found |= context
                        .find_by_version(&format!("{}@{}", pattern.name, definition.name))
                        .is_some();
                    if !found && !context.options.undefined_version {
                        context.error(
                            problem(
                                "EI242",
                                "A version script names a symbol that is not defined.",
                            )
                            .with_context(format!("Version: {label}"))
                            .with_context(format!("Symbol: {}", pattern.name)),
                        );
                    }
                };
            for pattern in definition
                .non_local
                .iter()
                .filter(|pattern| !pattern.wildcard)
            {
                exact(self, pattern, definition.identifier, &definition.name);
            }
            for pattern in definition.local.iter().filter(|pattern| !pattern.wildcard) {
                exact(self, pattern, elf::version_index::LOCAL, "local");
            }
        }
        let mut patterns: Vec<(SymbolPattern, u16)> = Vec::new();
        let mut asterisks: Vec<(SymbolPattern, u16)> = Vec::new();
        let mut global_asterisk = false;
        let mut local_asterisk = false;
        let mut reported = false;
        for definition in definitions.iter().rev() {
            for local in [false, true] {
                let version = if local {
                    elf::version_index::LOCAL
                } else {
                    definition.identifier
                };
                let list = if local {
                    &definition.local
                } else {
                    &definition.non_local
                };
                for pattern in list.iter().filter(|pattern| pattern.wildcard) {
                    if pattern.name != "*" {
                        patterns.push((pattern.clone(), version));
                        continue;
                    }
                    if !reported {
                        if (local && global_asterisk) || (!local && local_asterisk) {
                            self.warn(
                                caution("EI245", "The wildcard pattern * is used more than once in the version script.")
                                    .with_context("Scope: both local and global".to_string()),
                            );
                            reported = true;
                        } else if !local && global_asterisk {
                            self.warn(
                                caution("EI245", "The wildcard pattern * is used more than once in the version script.")
                                    .with_context("Scope: multiple version definitions".to_string()),
                            );
                            reported = true;
                        } else {
                            local_asterisk = local;
                            global_asterisk = !local;
                        }
                    }
                    asterisks.push((pattern.clone(), version));
                }
            }
        }
        patterns.extend(asterisks);
        if !patterns.is_empty() {
            for id in self.global_symbols() {
                let symbol = &self.symbols[id];
                if symbol.version_script_assigned
                    || symbol.name.contains('@')
                    || !self.can_be_versioned(id)
                {
                    continue;
                }
                if let Some((_, version)) = patterns
                    .iter()
                    .find(|(pattern, _)| pattern_matches(pattern, &symbol.name))
                {
                    self.symbols[id].version = *version;
                }
            }
        }
        let list = self.dynamic_list_patterns.clone();
        for pattern in list.iter().filter(|pattern| !pattern.wildcard) {
            if let Some(id) = self.find_by_version(&pattern.name) {
                self.symbols[id].exported = true;
                self.symbols[id].in_dynamic_list = true;
            }
        }
        let wildcards: Vec<&SymbolPattern> =
            list.iter().filter(|pattern| pattern.wildcard).collect();
        if !wildcards.is_empty() {
            for id in self.global_symbols() {
                if !self.can_be_versioned(id) {
                    continue;
                }
                let name = self.symbols[id].name.clone();
                if wildcards
                    .iter()
                    .any(|pattern| pattern_matches(pattern, &name))
                {
                    self.symbols[id].exported = true;
                    self.symbols[id].in_dynamic_list = true;
                }
            }
        }
    }

    fn parse_symbol_version(&mut self, id: SymbolIndex) {
        if self.symbols[id].version == elf::version_index::LOCAL {
            return;
        }
        let name = self.symbols[id].name.clone();
        let Some(at) = name.find('@') else {
            return;
        };
        self.symbols[id].version_suffix_at = Some(at);
        let mut version = &name[at + 1..];
        if version.is_empty() || !self.symbols[id].is_defined() {
            return;
        }
        let default = version.starts_with('@');
        if default {
            version = &version[1..];
        }
        let base = &name[..at];
        for definition in self.version_definitions.iter().skip(2) {
            if definition.name != version {
                continue;
            }
            let matches =
                |list: &[SymbolPattern]| list.iter().any(|pattern| pattern_matches(pattern, base));
            let identifier = if !matches(&definition.non_local) && matches(&definition.local) {
                elf::version_index::LOCAL
            } else if default {
                definition.identifier
            } else {
                definition.identifier | 0x8000
            };
            self.symbols[id].version = identifier;
            return;
        }
        if self.is_shared() && self.symbols[id].version != elf::version_index::LOCAL {
            let file = self.file_name(self.symbols[id].file);
            self.error(
                problem("EI243", "A symbol names a version that is not defined.")
                    .with_context(format!("File: {file}"))
                    .with_context(format!("Symbol: {name}"))
                    .with_context(format!("Version: {version}")),
            );
        }
    }

    pub fn combine_versioned_symbols(&mut self) {
        if !self.has_named_versions() {
            return;
        }
        let mut map: BTreeMap<SymbolIndex, SymbolIndex> = BTreeMap::new();
        for id in self.global_symbols() {
            let Some(at) = self.symbols[id].version_suffix_at else {
                continue;
            };
            let name = self.symbols[id].name.clone();
            let suffix = &name[at..];
            if suffix.starts_with("@@") {
                continue;
            }
            let Some(other) = self.find(&name[..at]) else {
                continue;
            };
            if other == id || !self.symbols[other].is_defined() {
                continue;
            }
            let other_name = self.symbols[other].name.clone();
            let other_suffix = self.symbols[other]
                .version_suffix_at
                .map(|position| other_name[position..].to_string())
                .unwrap_or_default();
            if other_suffix.starts_with("@@") && other_suffix[2..] == suffix[1..] {
                map.insert(id, other);
                let symbol = self.symbols[id].clone();
                if symbol.is_defined() {
                    if !symbol.is_weak()
                        && !self.symbols[other].is_weak()
                        && !self.options.allow_multiple_definition
                    {
                        let place = self.definition_place(other);
                        self.error(
                            problem("EI206", "A symbol is defined more than once.")
                                .with_context(format!("Symbol: {other_name}"))
                                .with_context(format!("Definition: {place}")),
                        );
                    }
                } else if symbol.binding != symbol_binding::WEAK
                    && self.symbols[other].binding == symbol_binding::WEAK
                    && !self.symbols[other].is_defined()
                {
                    self.symbols[other].binding = symbol.binding;
                }
                if symbol.referenced {
                    self.symbols[other].referenced = true;
                }
                self.symbols[id].state = SymbolState::Placeholder;
                self.symbols[id].used_in_regular_object = false;
            } else if self.symbols[id].is_defined() {
                let other_version = self.symbols[other].version;
                let same = if other_version > elf::version_index::GLOBAL {
                    self.version_definitions
                        .get((other_version & 0x7fff) as usize)
                        .is_some_and(|definition| definition.name == suffix[1..])
                } else {
                    self.symbols[id].section == self.symbols[other].section
                        && self.symbols[id].value == self.symbols[other].value
                };
                if same {
                    map.insert(other, id);
                    self.symbols[other].state = SymbolState::Placeholder;
                    self.symbols[other].used_in_regular_object = false;
                }
            }
        }
        if map.is_empty() {
            return;
        }
        for file in self.object_files.clone() {
            for handle in self.files[file].symbols.iter_mut() {
                if let Some(target) = map.get(handle) {
                    *handle = *target;
                }
            }
        }
    }

    pub fn parse_versions_and_compute_preemptible(&mut self) {
        for id in self.global_symbols() {
            if self.symbols[id].name.contains('@') {
                self.parse_symbol_version(id);
            }
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
            if name.starts_with("_ZTV") {
                diagnostic = diagnostic
                    .with_context("Advice: the class may be missing its key function".to_string());
            }
            if self.options.gc_sections
                && self.options.start_stop_garbage_collection
                && name.starts_with("__start_")
            {
                diagnostic = diagnostic.with_context(
                    "Advice: retain the encapsulated section or use -z nostart-stop-gc".to_string(),
                );
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
        self.split_exception_frames();
        if !self.options.gc_sections {
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
            return;
        }
        let candidates: Vec<SectionIndex> = self
            .input_order
            .iter()
            .copied()
            .filter(|section| self.sections[*section].live)
            .collect();
        for section in &candidates {
            self.sections[*section].live = false;
        }
        let mut queue: Vec<SectionIndex> = Vec::new();
        let mut named: BTreeMap<String, Vec<SectionIndex>> = BTreeMap::new();
        for id in self.global_symbols() {
            if self.symbols[id].exported {
                self.mark_symbol_live(id, &mut queue);
            }
        }
        let mut roots: Vec<String> = vec![self.entry_name.clone()];
        roots.push(
            self.options
                .initializer
                .clone()
                .unwrap_or_else(|| "_init".to_string()),
        );
        roots.push(
            self.options
                .finalizer
                .clone()
                .unwrap_or_else(|| "_fini".to_string()),
        );
        roots.extend(self.options.undefined.iter().cloned());
        for name in roots {
            if let Some(id) = self.find(&name) {
                self.mark_symbol_live(id, &mut queue);
            }
        }
        for section in self.frame_pieces.keys().copied().collect::<Vec<_>>() {
            let pieces = self.frame_pieces[&section].clone();
            let relocations = self.frame_relocations[&section].clone();
            for piece in pieces {
                let Some(first) = piece.first_relocation else {
                    continue;
                };
                if piece.is_common_information {
                    self.resolve_live_reference(
                        section,
                        &relocations[first],
                        false,
                        &named,
                        &mut queue,
                    );
                    continue;
                }
                let end = piece.input_offset + piece.size;
                for relocation in relocations[first..]
                    .iter()
                    .take_while(|relocation| relocation.offset < end)
                {
                    self.resolve_live_reference(section, relocation, true, &named, &mut queue);
                }
            }
        }
        for section in candidates {
            let data = &self.sections[section];
            if data.flags & section_flag::GNU_RETAIN != 0 {
                self.enqueue_live(section, 0, &mut queue);
                continue;
            }
            if data.flags & section_flag::LINK_ORDER != 0 {
                continue;
            }
            if data.flags & section_flag::ALLOCATE == 0
                && data.kind != section_type::RELOCATIONS
                && data.kind != section_type::RELOCATIONS_WITH_ADDENDS
                && data.next_in_group.is_none()
            {
                self.sections[section].live = true;
                for dependent in self.sections[section].dependents.clone() {
                    self.sections[dependent].live = true;
                }
            }
            let data = &self.sections[section];
            let name = data.name.clone();
            let reserved = match data.kind {
                section_type::FINI_ARRAY
                | section_type::INIT_ARRAY
                | section_type::PREINIT_ARRAY => true,
                section_type::NOTE => data.next_in_group.is_none(),
                _ => {
                    name == ".init"
                        || name == ".fini"
                        || name.starts_with(".init_array")
                        || name == ".jcr"
                        || name.starts_with(".ctors")
                        || name.starts_with(".dtors")
                }
            };
            if reserved {
                self.enqueue_live(section, 0, &mut queue);
            } else if (!self.options.start_stop_garbage_collection || name.starts_with("__libc_"))
                && is_c_identifier(&name)
            {
                named
                    .entry(format!("__start_{name}"))
                    .or_default()
                    .push(section);
                named
                    .entry(format!("__stop_{name}"))
                    .or_default()
                    .push(section);
            }
        }
        while let Some(section) = queue.pop() {
            let relocations = self.raw_relocations(section);
            for relocation in &relocations {
                self.resolve_live_reference(section, relocation, false, &named, &mut queue);
            }
            for dependent in self.sections[section].dependents.clone() {
                self.enqueue_live(dependent, 0, &mut queue);
            }
            if let Some(next) = self.sections[section].next_in_group {
                self.enqueue_live(next, 0, &mut queue);
            }
        }
        for id in self.global_symbols() {
            let symbol = &self.symbols[id];
            if symbol.is_shared() && symbol.used && symbol.binding != symbol_binding::WEAK {
                if let Some(file) = symbol.file.filter(|file| *file < self.files.len()) {
                    self.files[file].needed = true;
                }
            }
        }
        if self.options.print_gc_sections {
            for section in self.input_order.clone() {
                if !self.sections[section].live && self.sections[section].file.is_some() {
                    let place = self.section_place(section);
                    self.messages
                        .push(format!("removing unused section {place}"));
                }
            }
        }
    }

    fn mark_symbol_live(&mut self, id: SymbolIndex, queue: &mut Vec<SectionIndex>) {
        let symbol = &self.symbols[id];
        if !symbol.is_defined() {
            return;
        }
        if let Some(section) = symbol.section {
            let value = symbol.value;
            self.enqueue_live(section, value, queue);
        }
    }

    fn enqueue_live(&mut self, section: SectionIndex, offset: u64, queue: &mut Vec<SectionIndex>) {
        if !self.sections[section].pieces.is_empty() {
            let pieces = &mut self.sections[section].pieces;
            let position = pieces.partition_point(|piece| piece.input_offset <= offset);
            if position > 0 {
                pieces[position - 1].live = true;
            }
        }
        if self.sections[section].live || self.sections[section].is_exception_frame {
            return;
        }
        self.sections[section].live = true;
        queue.push(section);
    }

    fn resolve_live_reference(
        &mut self,
        section: SectionIndex,
        relocation: &RawRelocation,
        from_description: bool,
        named: &BTreeMap<String, Vec<SectionIndex>>,
        queue: &mut Vec<SectionIndex>,
    ) {
        let Some(id) = self.symbol_of(section, relocation.symbol) else {
            return;
        };
        self.symbols[id].used = true;
        let symbol = &self.symbols[id];
        if symbol.is_defined() {
            let Some(target) = symbol.section else {
                return;
            };
            let mut offset = symbol.value;
            if symbol.is_section() {
                offset = offset.wrapping_add(relocation.addend as u64);
                if self.sections[target].is_merge
                    && offset >= self.sections[target].data.len() as u64
                {
                    return;
                }
            }
            let target_data = &self.sections[target];
            if from_description
                && (target_data.flags & (section_flag::EXECUTE | section_flag::LINK_ORDER) != 0
                    || target_data.next_in_group.is_some())
            {
                return;
            }
            self.enqueue_live(target, offset, queue);
            return;
        }
        let name = self.symbols[id].name.clone();
        for target in named.get(&name).cloned().unwrap_or_default() {
            self.enqueue_live(target, 0, queue);
        }
    }

    pub fn split_exception_frames(&mut self) {
        let encoding = self.encoding();
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
            self.frame_pieces.insert(section, pieces);
            self.frame_relocations.insert(section, relocations);
        }
    }

    pub fn combine_exception_frames(&mut self) {
        let Some(synthetic) = self.synthetic_section(".eh_frame") else {
            return;
        };
        let encoding = self.encoding();
        let mut common_keys: BTreeMap<(Vec<u8>, Option<SymbolIndex>), usize> = BTreeMap::new();
        for section in self.frame_pieces.keys().copied().collect::<Vec<_>>() {
            let mut pieces = self.frame_pieces[&section].clone();
            let relocations = self.frame_relocations[&section].clone();
            let data = self.sections[section].data.clone();
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
        symbol.used || !self.options.gc_sections
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
        let name = self.dynamic_symbol_name(id);
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
                ".gnu.version_d",
                28 * self.version_definition_count() as u64,
            ),
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
            Synthetic::VersionDefinitions => buffer = self.encode_version_definitions(),
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
        header.push(self.operating_system_abi());
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

    pub fn operating_system_abi(&self) -> u8 {
        let mut abi = 0;
        for file in &self.files {
            if file.archive_name.is_some() {
                continue;
            }
            abi = file.elf.header.operating_system;
            if abi != 0 {
                break;
            }
        }
        abi
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
                Some(Synthetic::VersionDefinitions) => {
                    self.outputs[output].link = index_of(".dynstr", self);
                    self.outputs[output].info = self.version_definition_count() as u32;
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
        self.read_version_inputs();
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
        self.scan_version_script();
        self.parse_versions_and_compute_preemptible();
        self.combine_versioned_symbols();
        if self.failed() {
            return None;
        }
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

pub fn tokenize_windows_command_line(text: &str) -> Vec<String> {
    let characters: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut at = 0usize;
    while at < characters.len() {
        while at < characters.len() && characters[at].is_whitespace() {
            at += 1;
        }
        if at >= characters.len() {
            break;
        }
        let mut token = String::new();
        let mut quoted = false;
        while at < characters.len() {
            let character = characters[at];
            if !quoted && character.is_whitespace() {
                break;
            }
            if character == '\\' {
                let mut count = 0usize;
                while at < characters.len() && characters[at] == '\\' {
                    count += 1;
                    at += 1;
                }
                if at < characters.len() && characters[at] == '"' {
                    for _ in 0..count / 2 {
                        token.push('\\');
                    }
                    if count % 2 == 1 {
                        token.push('"');
                        at += 1;
                    }
                } else {
                    for _ in 0..count {
                        token.push('\\');
                    }
                }
                continue;
            }
            if character == '"' {
                if quoted && at + 1 < characters.len() && characters[at + 1] == '"' {
                    token.push('"');
                    at += 2;
                    continue;
                }
                quoted = !quoted;
                at += 1;
                continue;
            }
            token.push(character);
            at += 1;
        }
        tokens.push(token);
    }
    tokens
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdenticalCodeFolding {
    None,
    Safe,
    All,
}

pub mod guard_level {
    pub const OFF: u32 = 0;
    pub const CONTROL_FLOW: u32 = 1;
    pub const LONG_JUMP: u32 = 2;
    pub const EXCEPTION_CONTINUATION: u32 = 4;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoffInput {
    File(String),
    WholeArchive(String),
    StartLibrary,
    EndLibrary,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoffExport {
    pub name: String,
    pub external_name: String,
    pub forward_to: String,
    pub export_as: String,
    pub import_name: String,
    pub symbol_name: String,
    pub export_name: String,
    pub ordinal: u16,
    pub no_name: bool,
    pub data: bool,
    pub private: bool,
    pub constant: bool,
    pub source: u8,
    pub symbol: Option<SymbolIndex>,
    pub forward_chunk: Option<usize>,
}

pub mod export_source {
    pub const DIRECTIVES: u8 = 0;
    pub const EXPORT: u8 = 1;
    pub const MODULE_DEFINITION: u8 = 2;
    pub const EXPORT_ALL: u8 = 3;
}

#[derive(Clone, Debug)]
pub struct CoffOptions {
    pub output: String,
    pub inputs: Vec<CoffInput>,
    pub library_paths: Vec<String>,
    pub default_libraries: Vec<String>,
    pub no_default_libraries: Vec<String>,
    pub no_default_all: bool,
    pub entry: Option<String>,
    pub no_entry: bool,
    pub dll: bool,
    pub machine: u16,
    pub subsystem: Option<(u16, Option<(u32, u32)>)>,
    pub os_version: Option<(u32, u32)>,
    pub image_version: (u32, u32),
    pub image_base: Option<u64>,
    pub section_alignment: Option<u64>,
    pub file_alignment: u64,
    pub stack: (u64, u64),
    pub heap: (u64, u64),
    pub dynamic_base: Option<bool>,
    pub fixed: bool,
    pub high_entropy: Option<bool>,
    pub large_address_aware: Option<bool>,
    pub nx_compatible: bool,
    pub application_container: bool,
    pub allow_bind: bool,
    pub allow_isolation: bool,
    pub terminal_server_aware: bool,
    pub integrity_check: bool,
    pub guard: u32,
    pub safe_seh: Option<bool>,
    pub no_seh: bool,
    pub release: bool,
    pub timestamp: Option<u32>,
    pub repro: bool,
    pub collect: Option<bool>,
    pub folding: Option<IdenticalCodeFolding>,
    pub tail_merge: u8,
    pub debug: bool,
    pub write_symbol_table: bool,
    pub exports: Vec<String>,
    pub definition_file: Option<String>,
    pub import_library: Option<String>,
    pub no_import_library: bool,
    pub includes: Vec<String>,
    pub alternate_names: Vec<String>,
    pub merges: Vec<String>,
    pub sections: Vec<String>,
    pub align_common: Vec<String>,
    pub force_unresolved: bool,
    pub force_multiple: bool,
    pub demangle: bool,
    pub mingw: bool,
    pub kill_at: bool,
    pub warnings_as_errors: bool,
    pub whole_archive_all: bool,
    pub whole_archive_files: Vec<String>,
    pub delay_loads: Vec<String>,
    pub discard_sections: Vec<String>,
    pub driver: bool,
    pub driver_unprocessor_only: bool,
    pub driver_wdm: bool,
    pub swap_run_cd: bool,
    pub swap_run_net: bool,
    pub function_pad_minimum: Option<u32>,
    pub build_identifier: bool,
    pub ignored_warnings: Vec<String>,
    pub verbose: bool,
    pub warnings: Vec<Diagnostic>,
}

impl Default for CoffOptions {
    fn default() -> CoffOptions {
        CoffOptions {
            output: String::new(),
            inputs: Vec::new(),
            library_paths: Vec::new(),
            default_libraries: Vec::new(),
            no_default_libraries: Vec::new(),
            no_default_all: false,
            entry: None,
            no_entry: false,
            dll: false,
            machine: 0,
            subsystem: None,
            os_version: None,
            image_version: (0, 0),
            image_base: None,
            section_alignment: None,
            file_alignment: 512,
            stack: (1024 * 1024, 4096),
            heap: (1024 * 1024, 4096),
            dynamic_base: None,
            fixed: false,
            high_entropy: None,
            large_address_aware: None,
            nx_compatible: true,
            application_container: false,
            allow_bind: true,
            allow_isolation: true,
            terminal_server_aware: true,
            integrity_check: false,
            guard: guard_level::OFF,
            safe_seh: None,
            no_seh: false,
            release: false,
            timestamp: None,
            repro: false,
            collect: None,
            folding: None,
            tail_merge: 1,
            debug: false,
            write_symbol_table: false,
            exports: Vec::new(),
            definition_file: None,
            import_library: None,
            no_import_library: false,
            includes: Vec::new(),
            alternate_names: Vec::new(),
            merges: Vec::new(),
            sections: Vec::new(),
            align_common: Vec::new(),
            force_unresolved: false,
            force_multiple: false,
            demangle: true,
            mingw: false,
            kill_at: false,
            warnings_as_errors: false,
            whole_archive_all: false,
            whole_archive_files: Vec::new(),
            delay_loads: Vec::new(),
            discard_sections: Vec::new(),
            driver: false,
            driver_unprocessor_only: false,
            driver_wdm: false,
            swap_run_cd: false,
            swap_run_net: false,
            function_pad_minimum: None,
            build_identifier: false,
            ignored_warnings: Vec::new(),
            verbose: false,
            warnings: Vec::new(),
        }
    }
}

pub fn coff_machine_from_name(name: &str) -> Option<u16> {
    match name.to_ascii_lowercase().as_str() {
        "x64" | "amd64" => Some(crate::compilers::pe::machine::AMD64),
        "x86" | "i386" => Some(crate::compilers::pe::machine::I386),
        "arm" => Some(crate::compilers::pe::machine::ARM_THUMB),
        "arm64" => Some(crate::compilers::pe::machine::ARM64),
        "arm64ec" => Some(crate::compilers::pe::machine::ARM64_EMULATION_COMPATIBLE),
        "arm64x" => Some(crate::compilers::pe::machine::ARM64_HYBRID),
        _ => None,
    }
}

pub fn coff_subsystem_from_name(name: &str) -> Option<u16> {
    use crate::compilers::pe::subsystem;
    Some(match name.to_ascii_lowercase().as_str() {
        "boot_application" => subsystem::WINDOWS_BOOT_APPLICATION,
        "console" => subsystem::WINDOWS_CONSOLE,
        "default" => subsystem::UNKNOWN,
        "efi_application" => subsystem::EFI_APPLICATION,
        "efi_boot_service_driver" => subsystem::EFI_BOOT_SERVICE_DRIVER,
        "efi_rom" => subsystem::EFI_ROM,
        "efi_runtime_driver" => subsystem::EFI_RUNTIME_DRIVER,
        "native" => subsystem::NATIVE,
        "posix" => subsystem::POSIX_CONSOLE,
        "windows" => subsystem::WINDOWS_GRAPHICAL,
        "xbox" => subsystem::XBOX,
        _ => return None,
    })
}

fn coff_number(text: &str) -> Option<u64> {
    crate::compilers::pe::parse_integer(text)
}

fn coff_version(option: &str, text: &str) -> Result<(u32, u32), Diagnostic> {
    let (major, minor) = text.split_once('.').unwrap_or((text, ""));
    let major = major
        .parse::<u32>()
        .map_err(|_| invalid_value(option, text))?;
    let minor = if minor.is_empty() {
        0
    } else {
        minor
            .parse::<u32>()
            .map_err(|_| invalid_value(option, text))?
    };
    Ok((major, minor))
}

fn coff_pair(option: &str, text: &str, current: (u64, u64)) -> Result<(u64, u64), Diagnostic> {
    let (first, second) = text.split_once(',').unwrap_or((text, ""));
    let first = coff_number(first).ok_or_else(|| invalid_value(option, text))?;
    let second = if second.is_empty() {
        current.1
    } else {
        coff_number(second).ok_or_else(|| invalid_value(option, text))?
    };
    Ok((first, second))
}

fn split_coff_option(argument: &str) -> Option<(String, Option<String>)> {
    let rest = argument
        .strip_prefix("/?")
        .or_else(|| argument.strip_prefix("-?"))
        .or_else(|| argument.strip_prefix('/'))
        .or_else(|| argument.strip_prefix('-'))?;
    if rest.is_empty() {
        return None;
    }
    let rest = rest.strip_prefix('-').unwrap_or(rest);
    match rest.split_once(':') {
        Some((name, value)) => Some((name.to_ascii_lowercase(), Some(value.to_string()))),
        None => Some((rest.to_ascii_lowercase(), None)),
    }
}

pub fn parse_coff_command_line(
    arguments: &[String],
    source: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<CoffOptions, Diagnostic> {
    let mut expanded: Vec<String> = Vec::new();
    for argument in arguments {
        if let Some(path) = argument.strip_prefix('@') {
            let bytes = source(path).ok_or_else(|| file_missing(path))?;
            expanded.extend(tokenize_windows_command_line(&String::from_utf8_lossy(
                &bytes,
            )));
        } else {
            expanded.push(argument.clone());
        }
    }
    let mut options = CoffOptions::default();
    let mut index = 0usize;
    while index < expanded.len() {
        let argument = expanded[index].clone();
        index += 1;
        if argument.starts_with("--") {
            let name = argument.trim_start_matches('-');
            if name.starts_with("color-diagnostics")
                || name == "no-color-diagnostics"
                || name.starts_with("time-trace")
                || name.starts_with("rsp-quoting")
                || name == "version"
            {
                continue;
            }
        }
        let parsed = if argument.starts_with('/') || argument.starts_with('-') {
            split_coff_option(&argument)
        } else {
            None
        };
        let Some((name, value)) = parsed else {
            options.inputs.push(CoffInput::File(argument));
            continue;
        };
        let value_text = value.clone().unwrap_or_default();
        let need = |value: &Option<String>| -> Result<String, Diagnostic> {
            value.clone().ok_or_else(|| missing_value(&argument))
        };
        let flag_pair = |value: &Option<String>| -> Option<bool> {
            match value.as_deref() {
                None => Some(true),
                Some(text) if text.eq_ignore_ascii_case("no") => Some(false),
                _ => None,
            }
        };
        match name.as_str() {
            "out" => options.output = need(&value)?,
            "libpath" => options.library_paths.push(need(&value)?),
            "defaultlib" => options.default_libraries.push(need(&value)?),
            "nodefaultlib" => match value {
                Some(library) => options.no_default_libraries.push(library),
                None => options.no_default_all = true,
            },
            "entry" => options.entry = Some(need(&value)?),
            "noentry" => options.no_entry = true,
            "dll" => options.dll = true,
            "machine" => {
                let text = need(&value)?;
                options.machine = coff_machine_from_name(&text)
                    .ok_or_else(|| invalid_value("/machine", &text))?;
            }
            "subsystem" => {
                let text = need(&value)?;
                let (system, version) = text.split_once(',').unwrap_or((text.as_str(), ""));
                let system = coff_subsystem_from_name(system)
                    .ok_or_else(|| invalid_value("/subsystem", &text))?;
                let version = if version.is_empty() {
                    None
                } else {
                    Some(coff_version("/subsystem", version)?)
                };
                options.subsystem = Some((system, version));
            }
            "osversion" => options.os_version = Some(coff_version("/osversion", &need(&value)?)?),
            "version" => options.image_version = coff_version("/version", &need(&value)?)?,
            "base" => {
                let text = need(&value)?;
                let (base, _) = text.split_once(',').unwrap_or((text.as_str(), ""));
                options.image_base =
                    Some(coff_number(base).ok_or_else(|| invalid_value("/base", &text))?);
            }
            "align" => {
                let text = need(&value)?;
                let number = coff_number(&text).ok_or_else(|| invalid_value("/align", &text))?;
                if !number.is_power_of_two() {
                    return Err(invalid_value("/align", &text));
                }
                options.section_alignment = Some(number);
            }
            "filealign" => {
                let text = need(&value)?;
                let number =
                    coff_number(&text).ok_or_else(|| invalid_value("/filealign", &text))?;
                if !number.is_power_of_two() {
                    return Err(invalid_value("/filealign", &text));
                }
                options.file_alignment = number;
            }
            "stack" => options.stack = coff_pair("/stack", &need(&value)?, options.stack)?,
            "heap" => options.heap = coff_pair("/heap", &need(&value)?, options.heap)?,
            "dynamicbase" => {
                options.dynamic_base =
                    Some(flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?)
            }
            "fixed" => {
                options.fixed =
                    flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?
            }
            "highentropyva" => {
                options.high_entropy =
                    Some(flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?)
            }
            "largeaddressaware" => {
                options.large_address_aware =
                    Some(flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?)
            }
            "nxcompat" => {
                options.nx_compatible =
                    flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?
            }
            "appcontainer" => {
                options.application_container =
                    flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?
            }
            "allowbind" => {
                options.allow_bind =
                    flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?
            }
            "allowisolation" => {
                options.allow_isolation =
                    flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?
            }
            "tsaware" => {
                options.terminal_server_aware =
                    flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?
            }
            "integritycheck" => {
                options.integrity_check =
                    flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?
            }
            "safeseh" => {
                options.safe_seh =
                    Some(flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?)
            }
            "noseh" => options.no_seh = true,
            "release" => options.release = true,
            "brepro" => {
                options.repro = true;
                options.timestamp = None;
            }
            "timestamp" => {
                let text = need(&value)?;
                let number = coff_number(&text)
                    .filter(|number| *number <= u64::from(u32::MAX))
                    .ok_or_else(|| invalid_value("/timestamp", &text))?;
                options.timestamp = Some(number as u32);
                options.repro = false;
            }
            "guard" => {
                for part in need(&value)?.split(',') {
                    match part.to_ascii_lowercase().as_str() {
                        "no" => options.guard = guard_level::OFF,
                        "nolongjmp" => options.guard &= !guard_level::LONG_JUMP,
                        "noehcont" => options.guard &= !guard_level::EXCEPTION_CONTINUATION,
                        "cf" | "longjmp" => {
                            options.guard |= guard_level::CONTROL_FLOW | guard_level::LONG_JUMP
                        }
                        "ehcont" => {
                            options.guard |=
                                guard_level::CONTROL_FLOW | guard_level::EXCEPTION_CONTINUATION
                        }
                        _ => return Err(invalid_value("/guard", part)),
                    }
                }
            }
            "opt" => {
                for part in need(&value)?.to_ascii_lowercase().split(',') {
                    match part {
                        "ref" => options.collect = Some(true),
                        "noref" => options.collect = Some(false),
                        "icf" => options.folding = Some(IdenticalCodeFolding::All),
                        "safeicf" => options.folding = Some(IdenticalCodeFolding::Safe),
                        "noicf" => options.folding = Some(IdenticalCodeFolding::None),
                        "lldtailmerge" => options.tail_merge = 2,
                        "nolldtailmerge" => options.tail_merge = 0,
                        "lbr" | "nolbr" | "ltodebugpassmanager" | "noltodebugpassmanager" => {}
                        other if other.starts_with("icf=") => {
                            options.folding = Some(IdenticalCodeFolding::All)
                        }
                        other if other.starts_with("lldlto") => {}
                        other => return Err(invalid_value("/opt", other)),
                    }
                }
            }
            "debug" => {
                let text = value.clone().unwrap_or_else(|| "full".to_string());
                for part in text.to_ascii_lowercase().split(',') {
                    match part {
                        "none" => {
                            options.debug = false;
                            options.write_symbol_table = false;
                        }
                        "symtab" => {
                            options.write_symbol_table = true;
                            options.debug = true;
                        }
                        "nosymtab" => options.write_symbol_table = false,
                        _ => return Err(not_supported(&format!("/debug:{part}"))),
                    }
                }
            }
            "export" => options.exports.push(need(&value)?),
            "def" => options.definition_file = Some(need(&value)?),
            "implib" => options.import_library = Some(need(&value)?),
            "noimplib" => options.no_import_library = true,
            "include" => options.includes.push(need(&value)?),
            "alternatename" => options.alternate_names.push(need(&value)?),
            "merge" => options.merges.push(need(&value)?),
            "section" => options.sections.push(need(&value)?),
            "aligncomm" => options.align_common.push(need(&value)?),
            "force" => match value.as_deref().map(str::to_ascii_lowercase).as_deref() {
                None => {
                    options.force_unresolved = true;
                    options.force_multiple = true;
                }
                Some("unresolved") => options.force_unresolved = true,
                Some("multiple") | Some("multipleres") => options.force_multiple = true,
                _ => return Err(invalid_value("/force", &value_text)),
            },
            "demangle" => {
                options.demangle =
                    flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?
            }
            "lldmingw" => options.mingw = true,
            "kill-at" => options.kill_at = true,
            "wx" => {
                options.warnings_as_errors =
                    flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?
            }
            "wholearchive" => match value {
                Some(path) => {
                    options.whole_archive_files.push(path.clone());
                    options.inputs.push(CoffInput::WholeArchive(path));
                }
                None => options.whole_archive_all = true,
            },
            "start-lib" => options.inputs.push(CoffInput::StartLibrary),
            "end-lib" => options.inputs.push(CoffInput::EndLibrary),
            "delayload" => options.delay_loads.push(need(&value)?.to_ascii_lowercase()),
            "discard-section" => options.discard_sections.push(need(&value)?),
            "driver" => match value.as_deref().map(str::to_ascii_lowercase).as_deref() {
                None => options.driver = true,
                Some("wdm") => options.driver_wdm = true,
                Some("uponly") => options.driver_unprocessor_only = true,
                Some("wdm,uponly") | Some("uponly,wdm") => {
                    options.driver_wdm = true;
                    options.driver_unprocessor_only = true;
                }
                _ => return Err(invalid_value("/driver", &value_text)),
            },
            "swaprun" => {
                for part in need(&value)?.to_ascii_lowercase().split(',') {
                    match part {
                        "cd" => options.swap_run_cd = true,
                        "net" => options.swap_run_net = true,
                        _ => return Err(invalid_value("/swaprun", part)),
                    }
                }
            }
            "functionpadmin" => {
                options.function_pad_minimum = Some(match &value {
                    Some(text) => coff_number(text)
                        .ok_or_else(|| invalid_value("/functionpadmin", text))?
                        as u32,
                    None => 0,
                })
            }
            "build-id" => {
                options.build_identifier =
                    flag_pair(&value).ok_or_else(|| invalid_value(&argument, &value_text))?
            }
            "ignore" => options
                .ignored_warnings
                .extend(need(&value)?.split(',').map(str::to_string)),
            "verbose" => options.verbose = true,
            "incremental"
            | "nologo"
            | "link"
            | "ignoreidl"
            | "ltcg"
            | "nocoffgrpinfo"
            | "noexp"
            | "novcfeature"
            | "throwingnew"
            | "editandcontinue"
            | "fastfail"
            | "kernel"
            | "pdbcompress"
            | "emitpogophaseinfo"
            | "emittoolversioninfo"
            | "delay"
            | "errorreport"
            | "idlout"
            | "ilk"
            | "ltcgout"
            | "maxilksize"
            | "tlbid"
            | "tlbout"
            | "guardsym"
            | "assemblydebug"
            | "lldignoreenv"
            | "nodbgdirmerge"
            | "manifest"
            | "manifestuac"
            | "manifestfile"
            | "manifestdependency"
            | "pdb"
            | "pdbaltpath"
            | "pdbsourcepath"
            | "natvis"
            | "mllvm"
            | "errorlimit"
            | "cetcompat"
            | "summary"
            | "time"
            | "inferasanlibs"
            | "prefetch-inputs"
            | "failifmismatch"
            | "profile" => {}
            _ => {
                options.warnings.push(
                    caution(
                        "EI247",
                        "The linker ignores an option it does not recognize.",
                    )
                    .with_context(format!("Option: {argument}")),
                );
            }
        }
    }
    Ok(options)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoffSymbolKind {
    Placeholder,
    Undefined {
        alias: Option<SymbolIndex>,
        anti_dependency: bool,
    },
    Regular {
        file: Option<usize>,
        chunk: Option<usize>,
        value: u32,
        comdat: bool,
        external: bool,
        weak: bool,
        kind: u16,
    },
    Common {
        file: usize,
        chunk: usize,
        size: u64,
    },
    Absolute {
        address: u64,
    },
    Synthetic {
        chunk: Option<usize>,
        offset: u32,
    },
    ImportData {
        import: usize,
    },
    ImportThunk {
        import: usize,
        chunk: usize,
    },
    LocalImport {
        target: SymbolIndex,
        chunk: usize,
    },
    LazyArchive {
        archive: usize,
        member: usize,
    },
    LazyObject {
        file: usize,
    },
}

#[derive(Clone, Debug)]
pub struct CoffSymbolData {
    pub name: String,
    pub kind: CoffSymbolKind,
    pub used_in_regular_object: bool,
    pub pending_archive_load: bool,
    pub gc_root: bool,
}

impl CoffSymbolData {
    fn new(name: &str) -> CoffSymbolData {
        CoffSymbolData {
            name: name.to_string(),
            kind: CoffSymbolKind::Placeholder,
            used_in_regular_object: false,
            pending_archive_load: false,
            gc_root: false,
        }
    }

    pub fn is_lazy(&self) -> bool {
        matches!(
            self.kind,
            CoffSymbolKind::LazyArchive { .. } | CoffSymbolKind::LazyObject { .. }
        )
    }

    pub fn is_undefined(&self) -> bool {
        matches!(self.kind, CoffSymbolKind::Undefined { .. })
    }

    pub fn is_defined(&self) -> bool {
        !matches!(
            self.kind,
            CoffSymbolKind::Placeholder
                | CoffSymbolKind::Undefined { .. }
                | CoffSymbolKind::LazyArchive { .. }
                | CoffSymbolKind::LazyObject { .. }
        )
    }

    pub fn is_defined_coff(&self) -> bool {
        matches!(
            self.kind,
            CoffSymbolKind::Regular { .. } | CoffSymbolKind::Common { .. }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoffChunkKind {
    Section,
    Common {
        symbol: SymbolIndex,
    },
    Merge,
    ImportThunk {
        import: usize,
    },
    LocalImport {
        symbol: SymbolIndex,
    },
    HintName {
        name: String,
        hint: u16,
    },
    Lookup {
        target: usize,
    },
    OrdinalOnly {
        ordinal: u16,
    },
    ImportDirectory {
        name: usize,
        lookup: usize,
        address: usize,
    },
    Null,
    Text {
        text: String,
    },
    ExportDirectory {
        base: u32,
        maximum: u32,
        names: u32,
        name: usize,
        addresses: usize,
        pointers: usize,
        ordinals: usize,
    },
    ExportAddresses {
        base: u32,
    },
    ExportNamePointers {
        names: Vec<usize>,
    },
    ExportOrdinals {
        base: u32,
    },
    BaseRelocations {
        data: Vec<u8>,
    },
    RvaTable {
        entries: Vec<(usize, u32)>,
        flags: Option<Vec<u8>>,
    },
    DebugDirectory {
        records: Vec<(u32, usize)>,
        repro: bool,
    },
    CodeViewRecord,
}

#[derive(Clone, Debug)]
pub struct CoffChunk {
    pub kind: CoffChunkKind,
    pub file: Option<usize>,
    pub section: usize,
    pub name: String,
    pub characteristics: u32,
    pub alignment: u64,
    pub size: u64,
    pub has_data: bool,
    pub live: bool,
    pub rva: u64,
    pub output: Option<usize>,
    pub comdat: bool,
    pub selection: u8,
    pub children: Vec<usize>,
    pub replacement: usize,
    pub leader: Option<SymbolIndex>,
    pub check_sum: u32,
    pub keep_unique: bool,
    pub classes: [u32; 2],
    pub merged_into: Option<usize>,
}

impl CoffChunk {
    fn synthetic(
        kind: CoffChunkKind,
        size: u64,
        alignment: u64,
        characteristics: u32,
    ) -> CoffChunk {
        CoffChunk {
            kind,
            file: None,
            section: 0,
            name: String::new(),
            characteristics,
            alignment: alignment.max(1),
            size,
            has_data: true,
            live: true,
            rva: 0,
            output: None,
            comdat: false,
            selection: 0,
            children: Vec::new(),
            replacement: usize::MAX,
            leader: None,
            check_sum: 0,
            keep_unique: false,
            classes: [0, 0],
            merged_into: None,
        }
    }

    pub fn output_characteristics(&self) -> u32 {
        match self.kind {
            CoffChunkKind::Section => {
                self.characteristics
                    & (crate::compilers::pe::section_characteristic::PERMISSIONS | 0xe0)
            }
            _ => self.characteristics,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CoffObjectInput {
    pub name: String,
    pub parent: String,
    pub object: crate::compilers::pe::CoffObject,
    pub symbols: Vec<Option<SymbolIndex>>,
    pub chunks: Vec<usize>,
    pub directives: String,
    pub feature_flags: u32,
    pub lazy: bool,
    pub safe_handler_chunks: Vec<usize>,
    pub guard_function_chunks: Vec<usize>,
    pub guard_import_chunks: Vec<usize>,
    pub guard_long_jump_chunks: Vec<usize>,
    pub guard_continuation_chunks: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct CoffImportInput {
    pub name: String,
    pub parent: String,
    pub object: crate::compilers::pe::ImportObject,
    pub live: bool,
    pub external_name: String,
    pub library: String,
    pub import_symbol: Option<SymbolIndex>,
    pub thunk_symbol: Option<SymbolIndex>,
    pub thunk_chunk: Option<usize>,
    pub location: Option<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct CoffArchiveMember {
    pub name: String,
    pub bytes: Vec<u8>,
    pub offset: usize,
}

#[derive(Clone, Debug, Default)]
pub struct CoffArchive {
    pub name: String,
    pub members: Vec<CoffArchiveMember>,
    pub symbols: Vec<(String, usize)>,
    pub seen: BTreeSet<usize>,
}

pub fn read_coff_archive(name: &str, bytes: &[u8]) -> Result<CoffArchive, Diagnostic> {
    if !bytes.starts_with(b"!<arch>\n") {
        return Err(archive_problem(name, "the archive signature is missing"));
    }
    let mut at = 8usize;
    let mut members = Vec::new();
    let mut long_names: Vec<u8> = Vec::new();
    let mut first_table: Option<Vec<u8>> = None;
    let mut second_table: Option<Vec<u8>> = None;
    let mut special = 0usize;
    while at + 60 <= bytes.len() {
        let header = &bytes[at..at + 60];
        if &header[58..60] != b"`\n" {
            return Err(archive_problem(name, "a member header is malformed"));
        }
        let raw_name = String::from_utf8_lossy(&header[..16])
            .trim_end()
            .to_string();
        let size = String::from_utf8_lossy(&header[48..58])
            .trim()
            .parse::<usize>()
            .map_err(|_| archive_problem(name, "a member size is not a number"))?;
        let data_at = at + 60;
        if data_at + size > bytes.len() {
            return Err(archive_problem(
                name,
                "a member runs past the end of the archive",
            ));
        }
        let data = &bytes[data_at..data_at + size];
        if raw_name == "/" || raw_name == "/SYM64/" {
            if special == 0 {
                first_table = Some(data.to_vec());
            } else {
                second_table = Some(data.to_vec());
            }
            special += 1;
        } else if raw_name == "//" {
            long_names = data.to_vec();
        } else if raw_name == "/<ECSYMBOLS>/" || raw_name == "/<HYBRIDMAP>/" {
        } else {
            let member_name = if let Some(offset) = raw_name.strip_prefix('/') {
                let offset = offset.parse::<usize>().unwrap_or(0);
                let rest = long_names.get(offset..).unwrap_or(&[]);
                let end = rest
                    .iter()
                    .position(|byte| *byte == b'\n' || *byte == 0)
                    .unwrap_or(rest.len());
                String::from_utf8_lossy(&rest[..end])
                    .trim_end_matches('/')
                    .to_string()
            } else {
                raw_name.trim_end_matches('/').to_string()
            };
            members.push(CoffArchiveMember {
                name: member_name,
                bytes: data.to_vec(),
                offset: at,
            });
        }
        at = data_at + size + (size & 1);
    }
    let member_at = |offset: usize, members: &[CoffArchiveMember]| -> Option<usize> {
        members.iter().position(|member| member.offset == offset)
    };
    let mut symbols = Vec::new();
    if let Some(table) = second_table {
        let count = read_u32_le(&table, 0) as usize;
        let mut cursor = 4;
        let mut offsets = Vec::with_capacity(count);
        for _ in 0..count {
            offsets.push(read_u32_le(&table, cursor) as usize);
            cursor += 4;
        }
        let symbol_count = read_u32_le(&table, cursor) as usize;
        cursor += 4;
        let mut indices = Vec::with_capacity(symbol_count);
        for _ in 0..symbol_count {
            indices.push(u16::from_le_bytes([
                table.get(cursor).copied().unwrap_or(0),
                table.get(cursor + 1).copied().unwrap_or(0),
            ]) as usize);
            cursor += 2;
        }
        for index in indices {
            let end = table[cursor.min(table.len())..]
                .iter()
                .position(|byte| *byte == 0)
                .map(|end| cursor + end)
                .unwrap_or(table.len());
            let symbol = String::from_utf8_lossy(&table[cursor.min(table.len())..end]).to_string();
            cursor = end + 1;
            if index == 0 || index > offsets.len() {
                continue;
            }
            if let Some(member) = member_at(offsets[index - 1], &members) {
                symbols.push((symbol, member));
            }
        }
    } else if let Some(table) = first_table {
        let count = read_u32_be(&table, 0) as usize;
        let mut cursor = 4;
        let mut offsets = Vec::with_capacity(count);
        for _ in 0..count {
            offsets.push(read_u32_be(&table, cursor) as usize);
            cursor += 4;
        }
        for offset in offsets {
            let end = table[cursor.min(table.len())..]
                .iter()
                .position(|byte| *byte == 0)
                .map(|end| cursor + end)
                .unwrap_or(table.len());
            let symbol = String::from_utf8_lossy(&table[cursor.min(table.len())..end]).to_string();
            cursor = end + 1;
            if let Some(member) = member_at(offset, &members) {
                symbols.push((symbol, member));
            }
        }
    }
    Ok(CoffArchive {
        name: name.to_string(),
        members,
        symbols,
        seen: BTreeSet::new(),
    })
}

fn read_u32_le(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([
        bytes.get(at).copied().unwrap_or(0),
        bytes.get(at + 1).copied().unwrap_or(0),
        bytes.get(at + 2).copied().unwrap_or(0),
        bytes.get(at + 3).copied().unwrap_or(0),
    ])
}

fn read_u32_be(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([
        bytes.get(at).copied().unwrap_or(0),
        bytes.get(at + 1).copied().unwrap_or(0),
        bytes.get(at + 2).copied().unwrap_or(0),
        bytes.get(at + 3).copied().unwrap_or(0),
    ])
}

#[derive(Clone, Debug)]
pub enum CoffTask {
    Path {
        path: String,
        lazy: bool,
        whole: bool,
    },
    Member {
        archive: usize,
        member: usize,
    },
}

#[derive(Clone, Debug, Default)]
pub struct CoffOutputSection {
    pub name: String,
    pub characteristics: u32,
    pub chunks: Vec<usize>,
    pub virtual_address: u64,
    pub virtual_size: u64,
    pub raw_size: u64,
    pub file_offset: u64,
    pub index: usize,
}

pub struct CoffContext<'s> {
    pub source: &'s dyn Fn(&str) -> Option<Vec<u8>>,
    pub options: CoffOptions,
    pub diagnostics: Vec<Diagnostic>,
    pub messages: Vec<String>,
    pub machine: u16,
    pub objects: Vec<CoffObjectInput>,
    pub object_order: Vec<usize>,
    pub imports: Vec<CoffImportInput>,
    pub archives: Vec<CoffArchive>,
    pub symbols: Vec<CoffSymbolData>,
    pub names: BTreeMap<String, SymbolIndex>,
    pub chunks: Vec<CoffChunk>,
    pub tasks: std::collections::VecDeque<CoffTask>,
    pub search_paths: Vec<String>,
    pub visited_files: BTreeSet<String>,
    pub visited_libraries: BTreeSet<String>,
    pub gc_roots: Vec<SymbolIndex>,
    pub entry: Option<SymbolIndex>,
    pub exports: Vec<CoffExport>,
    pub directive_exports: BTreeSet<String>,
    pub alternate_names: BTreeMap<String, String>,
    pub align_common: BTreeMap<String, u64>,
    pub merges: BTreeMap<String, String>,
    pub section_permissions: BTreeMap<String, u32>,
    pub local_import_chunks: Vec<usize>,
    pub merge_chunks: [Option<usize>; 16],
    pub merge_members: BTreeMap<usize, Vec<usize>>,
    pub merge_offsets: BTreeMap<usize, BTreeMap<Vec<u8>, u64>>,
    pub subsystem: u16,
    pub subsystem_version: (u32, u32),
    pub os_version: (u32, u32),
    pub image_base: u64,
    pub collect: bool,
    pub folding: IdenticalCodeFolding,
    pub tail_merge: bool,
    pub dynamic_base: bool,
    pub relocatable: bool,
    pub safe_seh: bool,
    pub load_config_symbol: Option<SymbolIndex>,
    pub load_config_size: u32,
    pub library_order: BTreeMap<String, usize>,
    pub had_explicit_exports: bool,
    pub write_check_sum: bool,
    pub outputs: Vec<CoffOutputSection>,
    pub size_of_headers: u64,
    pub size_of_image: u64,
    pub file_size: u64,
    pub import_table: Option<(usize, u64)>,
    pub import_address_table: Option<(usize, u64)>,
    pub export_range: Option<(usize, usize)>,
    pub exception_range: Option<(usize, usize)>,
    pub debug_directory: Option<usize>,
    pub set_no_seh: bool,
    pub thread_local_alignment: u64,
}

pub fn coff_display(object: &CoffObjectInput) -> String {
    let base =
        |path: &str| -> String { path.rsplit(['/', '\\']).next().unwrap_or(path).to_string() };
    if object.parent.is_empty() {
        object.name.clone()
    } else {
        format!("{}({})", base(&object.parent), base(&object.name))
    }
}

impl<'s> CoffContext<'s> {
    pub fn new(
        options: CoffOptions,
        source: &'s dyn Fn(&str) -> Option<Vec<u8>>,
    ) -> CoffContext<'s> {
        CoffContext {
            source,
            diagnostics: options.warnings.clone(),
            messages: Vec::new(),
            machine: options.machine,
            options,
            objects: Vec::new(),
            object_order: Vec::new(),
            imports: Vec::new(),
            archives: Vec::new(),
            symbols: Vec::new(),
            names: BTreeMap::new(),
            chunks: Vec::new(),
            tasks: std::collections::VecDeque::new(),
            search_paths: Vec::new(),
            visited_files: BTreeSet::new(),
            visited_libraries: BTreeSet::new(),
            gc_roots: Vec::new(),
            entry: None,
            exports: Vec::new(),
            directive_exports: BTreeSet::new(),
            alternate_names: BTreeMap::new(),
            align_common: BTreeMap::new(),
            merges: BTreeMap::new(),
            section_permissions: BTreeMap::new(),
            local_import_chunks: Vec::new(),
            merge_chunks: [None; 16],
            merge_members: BTreeMap::new(),
            merge_offsets: BTreeMap::new(),
            subsystem: 0,
            subsystem_version: (6, 0),
            os_version: (6, 0),
            image_base: 0,
            collect: true,
            folding: IdenticalCodeFolding::None,
            tail_merge: false,
            dynamic_base: true,
            relocatable: true,
            safe_seh: false,
            load_config_symbol: None,
            load_config_size: 0,
            library_order: BTreeMap::new(),
            had_explicit_exports: false,
            write_check_sum: false,
            outputs: Vec::new(),
            size_of_headers: 0,
            size_of_image: 0,
            file_size: 0,
            import_table: None,
            import_address_table: None,
            export_range: None,
            exception_range: None,
            debug_directory: None,
            set_no_seh: false,
            thread_local_alignment: 0,
        }
    }

    pub fn error(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn warn(&mut self, diagnostic: Diagnostic) {
        let mut diagnostic = diagnostic;
        if self.options.warnings_as_errors {
            diagnostic.severity = Severity::Error;
        }
        self.diagnostics.push(diagnostic);
    }

    pub fn failed(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    pub fn is_64(&self) -> bool {
        crate::compilers::pe::is_64_bit(self.machine)
    }

    pub fn word_size(&self) -> u64 {
        if self.is_64() {
            8
        } else {
            4
        }
    }

    pub fn mangle(&self, name: &str) -> String {
        if self.machine == crate::compilers::pe::machine::I386 {
            format!("_{name}")
        } else {
            name.to_string()
        }
    }

    pub fn find(&self, name: &str) -> Option<SymbolIndex> {
        self.names.get(name).copied()
    }

    pub fn find_underscore(&self, name: &str) -> Option<SymbolIndex> {
        self.find(&self.mangle(name))
    }

    fn insert(&mut self, name: &str, regular: bool) -> (SymbolIndex, bool) {
        if let Some(id) = self.names.get(name).copied() {
            if regular {
                self.symbols[id].used_in_regular_object = true;
            }
            return (id, false);
        }
        let id = self.symbols.len();
        let mut data = CoffSymbolData::new(name);
        data.used_in_regular_object = regular;
        self.symbols.push(data);
        self.names.insert(name.to_string(), id);
        (id, true)
    }

    fn local_symbol(&mut self, name: &str, kind: CoffSymbolKind) -> SymbolIndex {
        let id = self.symbols.len();
        let mut data = CoffSymbolData::new(name);
        data.kind = kind;
        data.used_in_regular_object = true;
        self.symbols.push(data);
        id
    }

    pub fn file_label(&self, file: Option<usize>) -> String {
        match file {
            Some(file) if file < self.objects.len() => coff_display(&self.objects[file]),
            _ => "<internal>".to_string(),
        }
    }

    pub fn symbol_file(&self, id: SymbolIndex) -> Option<usize> {
        match &self.symbols[id].kind {
            CoffSymbolKind::Regular { file, .. } => *file,
            CoffSymbolKind::Common { file, .. } => Some(*file),
            CoffSymbolKind::LazyObject { file } => Some(*file),
            _ => None,
        }
    }

    pub fn report_duplicate(&mut self, existing: SymbolIndex, new_file: Option<usize>) {
        let name = self.symbols[existing].name.clone();
        let old = self.file_label(self.symbol_file(existing));
        let new = self.file_label(new_file);
        let diagnostic = problem("EI206", "A symbol is defined more than once.")
            .with_context(format!("Symbol: {name}"))
            .with_context(format!("Definition: {old}"))
            .with_context(format!("Definition: {new}"));
        if self.options.force_multiple {
            self.warn(diagnostic);
        } else {
            self.error(diagnostic);
        }
    }

    fn force_lazy(&mut self, id: SymbolIndex) {
        self.symbols[id].pending_archive_load = true;
        match self.symbols[id].kind.clone() {
            CoffSymbolKind::LazyArchive { archive, member } => self.add_member(archive, member),
            CoffSymbolKind::LazyObject { file } => {
                if !self.objects[file].lazy {
                    return;
                }
                self.objects[file].lazy = false;
                self.add_object_file(file);
            }
            _ => {}
        }
    }

    fn add_member(&mut self, archive: usize, member: usize) {
        let offset = self.archives[archive].members[member].offset;
        if !self.archives[archive].seen.insert(offset) {
            return;
        }
        self.tasks.push_back(CoffTask::Member { archive, member });
    }

    pub fn add_undefined(&mut self, name: &str, regular: bool, override_lazy: bool) -> SymbolIndex {
        let (id, inserted) = self.insert(name, regular);
        if inserted || (self.symbols[id].is_lazy() && override_lazy) {
            self.symbols[id].kind = CoffSymbolKind::Undefined {
                alias: None,
                anti_dependency: false,
            };
            return id;
        }
        if self.symbols[id].is_lazy() {
            self.force_lazy(id);
        }
        id
    }

    pub fn add_gc_root(&mut self, name: &str) -> SymbolIndex {
        let id = self.add_undefined(name, true, false);
        if !self.symbols[id].gc_root {
            self.symbols[id].gc_root = true;
            self.gc_roots.push(id);
        }
        id
    }

    fn add_lazy_archive(&mut self, archive: usize, name: &str, member: usize) {
        let (id, inserted) = self.insert(name, false);
        if inserted {
            self.symbols[id].kind = CoffSymbolKind::LazyArchive { archive, member };
            return;
        }
        let CoffSymbolKind::Undefined { alias, .. } = self.symbols[id].kind.clone() else {
            return;
        };
        if alias.is_some() || self.symbols[id].pending_archive_load {
            return;
        }
        self.symbols[id].pending_archive_load = true;
        self.add_member(archive, member);
    }

    fn add_lazy_object(&mut self, file: usize, name: &str) {
        let (id, inserted) = self.insert(name, true);
        if inserted {
            self.symbols[id].kind = CoffSymbolKind::LazyObject { file };
            return;
        }
        let CoffSymbolKind::Undefined { alias, .. } = self.symbols[id].kind.clone() else {
            return;
        };
        if alias.is_some() || self.symbols[id].pending_archive_load {
            return;
        }
        self.symbols[id].pending_archive_load = true;
        self.objects[file].lazy = false;
        self.add_object_file(file);
    }

    pub fn add_absolute(&mut self, name: &str, address: u64) -> SymbolIndex {
        let (id, inserted) = self.insert(name, false);
        self.symbols[id].used_in_regular_object = true;
        let symbol = &self.symbols[id];
        if inserted || symbol.is_undefined() || symbol.is_lazy() {
            self.symbols[id].kind = CoffSymbolKind::Absolute { address };
        } else if let CoffSymbolKind::Absolute { address: held } = symbol.kind {
            if held != address {
                self.report_duplicate(id, None);
            }
        } else if !symbol.is_defined_coff() {
            self.report_duplicate(id, None);
        }
        id
    }

    pub fn add_synthetic(&mut self, name: &str, chunk: Option<usize>) -> SymbolIndex {
        let (id, inserted) = self.insert(name, false);
        self.symbols[id].used_in_regular_object = true;
        let symbol = &self.symbols[id];
        if inserted || symbol.is_undefined() || symbol.is_lazy() {
            self.symbols[id].kind = CoffSymbolKind::Synthetic { chunk, offset: 0 };
        } else if !symbol.is_defined_coff() {
            self.report_duplicate(id, None);
        }
        id
    }

    #[allow(clippy::too_many_arguments)]
    fn add_regular(
        &mut self,
        file: usize,
        name: &str,
        chunk: usize,
        value: u32,
        kind: u16,
        weak: bool,
    ) -> SymbolIndex {
        let (id, inserted) = self.insert(name, true);
        let replace = match &self.symbols[id].kind {
            CoffSymbolKind::Regular { weak: held, .. } => *held,
            _ => true,
        };
        if inserted || replace {
            self.symbols[id].kind = CoffSymbolKind::Regular {
                file: Some(file),
                chunk: Some(chunk),
                value,
                comdat: false,
                external: true,
                weak,
                kind,
            };
        } else if !weak {
            self.report_duplicate(id, Some(file));
        }
        id
    }

    fn add_comdat(
        &mut self,
        file: usize,
        name: &str,
        value: u32,
        kind: u16,
    ) -> (SymbolIndex, bool) {
        let (id, inserted) = self.insert(name, true);
        let is_regular = matches!(self.symbols[id].kind, CoffSymbolKind::Regular { .. });
        if inserted || !is_regular {
            self.symbols[id].kind = CoffSymbolKind::Regular {
                file: Some(file),
                chunk: None,
                value,
                comdat: true,
                external: true,
                weak: false,
                kind,
            };
            return (id, true);
        }
        if let CoffSymbolKind::Regular { comdat: false, .. } = self.symbols[id].kind {
            self.report_duplicate(id, Some(file));
        }
        (id, false)
    }

    fn add_common(&mut self, file: usize, name: &str, size: u64, chunk: usize) -> SymbolIndex {
        let (id, inserted) = self.insert(name, true);
        let replace = match &self.symbols[id].kind {
            CoffSymbolKind::Common { size: held, .. } => size > *held,
            other => !matches!(other, CoffSymbolKind::Regular { .. }),
        };
        if inserted || replace {
            self.symbols[id].kind = CoffSymbolKind::Common { file, chunk, size };
            self.chunks[chunk].live = true;
        }
        id
    }

    fn add_import_data(&mut self, name: &str, import: usize) -> Option<SymbolIndex> {
        let (id, inserted) = self.insert(name, false);
        self.symbols[id].used_in_regular_object = true;
        let symbol = &self.symbols[id];
        if inserted || symbol.is_undefined() || symbol.is_lazy() {
            self.symbols[id].kind = CoffSymbolKind::ImportData { import };
            return Some(id);
        }
        self.report_duplicate(id, None);
        None
    }

    fn add_import_thunk(&mut self, name: &str, import: usize, chunk: usize) -> Option<SymbolIndex> {
        let (id, inserted) = self.insert(name, false);
        self.symbols[id].used_in_regular_object = true;
        let symbol = &self.symbols[id];
        if inserted || symbol.is_undefined() || symbol.is_lazy() {
            self.symbols[id].kind = CoffSymbolKind::ImportThunk { import, chunk };
            return Some(id);
        }
        self.report_duplicate(id, None);
        None
    }

    pub fn find_file(&self, name: &str) -> String {
        if name.starts_with('/') || name.contains(":\\") || name.starts_with('\\') {
            return name.to_string();
        }
        let has_extension = name.contains('.');
        for folder in &self.search_paths {
            let path = if folder.is_empty() {
                name.to_string()
            } else {
                format!("{}/{name}", folder.trim_end_matches(['/', '\\']))
            };
            if (self.source)(&path).is_some() {
                return path;
            }
            if !has_extension {
                let with = format!("{path}.obj");
                if (self.source)(&with).is_some() {
                    return with;
                }
            }
        }
        name.to_string()
    }

    fn find_file_if_new(&mut self, name: &str) -> Option<String> {
        let path = self.find_file(name);
        if (self.source)(&path).is_some() && !self.visited_files.insert(path.clone()) {
            return None;
        }
        if path.to_ascii_lowercase().ends_with(".lib") {
            let base = path
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or(&path)
                .to_ascii_lowercase();
            self.visited_libraries.insert(base);
        }
        Some(path)
    }

    pub fn find_library(&self, name: &str) -> String {
        let name = if name.contains('.') {
            name.to_string()
        } else {
            format!("{name}.lib")
        };
        self.find_file(&name)
    }

    fn find_library_if_new(&mut self, name: &str) -> Option<String> {
        if self.options.no_default_all {
            return None;
        }
        if !self.visited_libraries.insert(name.to_ascii_lowercase()) {
            return None;
        }
        let path = self.find_library(name);
        let lowered = path.to_ascii_lowercase();
        if self
            .options
            .no_default_libraries
            .iter()
            .any(|library| self.find_library(library).to_ascii_lowercase() == lowered)
        {
            return None;
        }
        if (self.source)(&path).is_some() && !self.visited_files.insert(path.clone()) {
            return None;
        }
        Some(path)
    }

    pub fn enqueue_path(&mut self, path: &str, lazy: bool, whole: bool) {
        self.tasks.push_back(CoffTask::Path {
            path: path.to_string(),
            lazy,
            whole,
        });
    }

    pub fn run_tasks(&mut self) -> bool {
        let worked = !self.tasks.is_empty();
        while let Some(task) = self.tasks.pop_front() {
            match task {
                CoffTask::Path { path, lazy, whole } => match (self.source)(&path) {
                    Some(bytes) => self.add_buffer(&path, bytes, whole, lazy),
                    None => self.error(file_missing(&path)),
                },
                CoffTask::Member { archive, member } => {
                    let data = self.archives[archive].members[member].clone();
                    let parent = self.archives[archive].name.clone();
                    self.add_archive_buffer(&data.name, &parent, data.bytes, false);
                }
            }
        }
        worked
    }

    fn add_buffer(&mut self, path: &str, bytes: Vec<u8>, whole: bool, lazy: bool) {
        if bytes.starts_with(b"!<arch>\n") {
            let archive = match read_coff_archive(path, &bytes) {
                Ok(archive) => archive,
                Err(diagnostic) => {
                    self.error(diagnostic);
                    return;
                }
            };
            if whole {
                for member in archive.members {
                    self.add_archive_buffer(&member.name, path, member.bytes, false);
                }
                return;
            }
            let index = self.archives.len();
            let symbols = archive.symbols.clone();
            self.archives.push(archive);
            for (name, member) in symbols {
                self.add_lazy_archive(index, &name, member);
            }
            return;
        }
        if crate::compilers::pe::is_import_object(&bytes) {
            self.add_import_file(path, "", &bytes);
            return;
        }
        if crate::compilers::pe::is_coff_object(&bytes) {
            self.add_object_bytes(path, "", &bytes, lazy);
            return;
        }
        self.error(
            problem(
                "EI212",
                "The linker does not support what this input asks for yet.",
            )
            .with_context("Feature: this input file type".to_string())
            .with_context(format!("File: {path}")),
        );
    }

    fn add_archive_buffer(&mut self, name: &str, parent: &str, bytes: Vec<u8>, lazy: bool) {
        if crate::compilers::pe::is_import_object(&bytes) {
            self.add_import_file(name, parent, &bytes);
            return;
        }
        if crate::compilers::pe::is_coff_object(&bytes) {
            self.add_object_bytes(name, parent, &bytes, lazy);
            return;
        }
        self.error(
            problem(
                "EI212",
                "The linker does not support what this input asks for yet.",
            )
            .with_context("Feature: this archive member type".to_string())
            .with_context(format!("File: {parent}({name})")),
        );
    }

    fn add_object_bytes(&mut self, name: &str, parent: &str, bytes: &[u8], lazy: bool) {
        let object = match crate::compilers::pe::CoffObject::parse(bytes) {
            Ok(object) => object,
            Err(diagnostic) => {
                self.error(diagnostic.with_context(format!("File: {name}")));
                return;
            }
        };
        let index = self.objects.len();
        self.objects.push(CoffObjectInput {
            name: name.to_string(),
            parent: parent.to_string(),
            object,
            lazy,
            ..CoffObjectInput::default()
        });
        self.add_object_file(index);
    }

    fn add_object_file(&mut self, file: usize) {
        if self.objects[file].lazy {
            self.parse_lazy_object(file);
        } else {
            self.parse_object(file);
            self.object_order.push(file);
        }
        let machine_number = self.objects[file].object.machine;
        self.check_machine(machine_number, file);
        self.parse_directives(Some(file));
    }

    fn check_machine(&mut self, machine_number: u16, file: usize) {
        if machine_number == 0 {
            return;
        }
        if self.machine == 0 {
            self.machine = machine_number;
            return;
        }
        if self.machine != machine_number {
            let label = self.file_label(Some(file));
            let found = crate::compilers::pe::machine_name(machine_number);
            let expected = crate::compilers::pe::machine_name(self.machine);
            self.error(
                problem(
                    "EI209",
                    "The input targets a different machine than the output.",
                )
                .with_context(format!("File: {label}"))
                .with_context(format!("Machine: {found}"))
                .with_context(format!("Expected: {expected}")),
            );
        }
    }

    fn parse_lazy_object(&mut self, file: usize) {
        let symbols = self.objects[file].object.symbols.clone();
        for symbol in symbols {
            if symbol.is_undefined() || !symbol.is_external() || symbol.weak_external().is_some() {
                continue;
            }
            if symbol.section_number == crate::compilers::pe::section_number::ABSOLUTE
                && (symbol.name == "@feat.00" || symbol.name == "@comp.id")
            {
                continue;
            }
            self.add_lazy_object(file, &symbol.name);
            if !self.objects[file].lazy {
                return;
            }
        }
    }

    fn read_section(
        &mut self,
        file: usize,
        number: usize,
        check_sum: u32,
        leader_name: &str,
    ) -> Option<usize> {
        use crate::compilers::pe::section_characteristic as character;
        let section = self.objects[file].object.sections[number - 1].clone();
        let name = section.name.clone();
        if name == ".drectve" {
            self.objects[file].directives = String::from_utf8_lossy(&section.data).to_string();
            return None;
        }
        if name == ".llvm_addrsig" || name == ".llvm.call-graph-profile" {
            return None;
        }
        if self.options.discard_sections.contains(&name) {
            return None;
        }
        if name.starts_with(".debug_") {
            return None;
        }
        if section.characteristics & character::LINK_REMOVE != 0 {
            return None;
        }
        let alignment = if section.characteristics & character::TYPE_NO_PAD != 0 {
            1
        } else {
            let shift = (section.characteristics >> 20) & 0xf;
            if shift > 0 {
                1u64 << (shift - 1)
            } else {
                16
            }
        };
        let comdat = section.characteristics & character::LINK_COMDAT != 0;
        let index = self.chunks.len();
        self.chunks.push(CoffChunk {
            kind: CoffChunkKind::Section,
            file: Some(file),
            section: number,
            name: name.clone(),
            characteristics: section.characteristics,
            alignment,
            size: u64::from(section.size_of_raw_data),
            has_data: section.characteristics & character::UNINITIALIZED_DATA == 0,
            live: !self.collect || !comdat,
            rva: 0,
            output: None,
            comdat,
            selection: 0,
            children: Vec::new(),
            replacement: index,
            leader: None,
            check_sum,
            keep_unique: false,
            classes: [0, 0],
            merged_into: None,
        });
        let is_code_view =
            name == ".debug$S" || name == ".debug$T" || name == ".debug$P" || name == ".debug$H";
        if is_code_view {
        } else if name == ".gfids$y" {
            self.objects[file].guard_function_chunks.push(index);
        } else if name == ".giats$y" {
            self.objects[file].guard_import_chunks.push(index);
        } else if name == ".gljmp$y" {
            self.objects[file].guard_long_jump_chunks.push(index);
        } else if name == ".gehcont$y" {
            self.objects[file].guard_continuation_chunks.push(index);
        } else if name == ".sxdata" {
            self.objects[file].safe_handler_chunks.push(index);
        } else if self.tail_merge
            && section.relocations.is_empty()
            && name == ".rdata"
            && leader_name.starts_with("??_C@")
        {
            let log = alignment.trailing_zeros() as usize;
            let merge = match self.merge_chunks[log] {
                Some(merge) => merge,
                None => {
                    let merge = self.chunks.len();
                    let mut chunk = CoffChunk::synthetic(
                        CoffChunkKind::Merge,
                        0,
                        alignment,
                        character::READ | character::INITIALIZED_DATA,
                    );
                    chunk.name = ".rdata".to_string();
                    self.chunks.push(chunk);
                    self.merge_chunks[log] = Some(merge);
                    merge
                }
            };
            self.merge_members.entry(merge).or_default().push(index);
            self.chunks[index].merged_into = Some(merge);
        } else if name == ".rsrc"
            || name.starts_with(".rsrc$")
            || section.characteristics & character::LINK_INFORMATION == 0
        {
            self.objects[file].chunks.push(index);
        }
        Some(index)
    }

    fn parse_object(&mut self, file: usize) {
        use crate::compilers::pe::{comdat_selection, section_characteristic as character};
        let section_count = self.objects[file].object.sections.len();
        let mut sparse: Vec<Option<Option<usize>>> = vec![Some(None); section_count + 1];
        for (number, slot) in sparse.iter_mut().enumerate().skip(1) {
            let characteristics = self.objects[file].object.sections[number - 1].characteristics;
            if characteristics & character::LINK_COMDAT != 0 {
                *slot = None;
            } else {
                *slot = Some(self.read_section(file, number, 0, ""));
            }
        }
        let symbol_count = self.objects[file].object.symbol_count as usize;
        self.objects[file].symbols = vec![None; symbol_count];
        let symbols = self.objects[file].object.symbols.clone();
        let big = self.objects[file].object.big;
        let mut weak_aliases: Vec<(SymbolIndex, u32, bool)> = Vec::new();
        let mut pending: Vec<usize> = Vec::new();
        let mut comdat_definitions: Vec<Option<crate::compilers::pe::SectionDefinition>> =
            vec![None; section_count + 1];
        for (position, symbol) in symbols.iter().enumerate() {
            let index = symbol.index as usize;
            if symbol.storage_class == crate::compilers::pe::storage_class::EXTERNAL
                && symbol.section_number == 0
                && symbol.value == 0
            {
                let id = self.add_undefined(&symbol.name, true, false);
                self.objects[file].symbols[index] = Some(id);
            } else if let Some((tag, characteristics)) = symbol.weak_external() {
                let id = self.add_undefined(&symbol.name, true, true);
                self.objects[file].symbols[index] = Some(id);
                weak_aliases.push((id, tag, characteristics == 4));
            } else {
                match self.create_defined(file, position, &mut sparse, &mut comdat_definitions, big)
                {
                    Some(created) => self.objects[file].symbols[index] = created,
                    None => pending.push(position),
                }
            }
        }
        for position in pending {
            let symbol = symbols[position].clone();
            let number = symbol.section_number as usize;
            if let Some(definition) = symbol.section_definition(big) {
                if definition.selection == comdat_selection::ASSOCIATIVE {
                    let parent = definition.number as usize;
                    match sparse.get(parent).cloned().flatten() {
                        None => {
                            let label = self.file_label(Some(file));
                            self.error(
                                problem(
                                    "EI212",
                                    "The linker does not support what this input asks for yet.",
                                )
                                .with_context(
                                    "Feature: an associative comdat whose parent is not a leader"
                                        .to_string(),
                                )
                                .with_context(format!("File: {label}"))
                                .with_context(format!("Symbol: {}", symbol.name)),
                            );
                        }
                        Some(Some(parent_chunk)) => {
                            let chunk = self.read_section(file, number, definition.check_sum, "");
                            sparse[number] = Some(chunk);
                            if let Some(chunk) = chunk {
                                self.chunks[chunk].selection = comdat_selection::ASSOCIATIVE;
                                self.add_associative(parent_chunk, chunk);
                            }
                        }
                        Some(None) => sparse[number] = Some(None),
                    }
                }
            }
            if sparse.get(number).is_some_and(Option::is_none) {
                continue;
            }
            let created = self.create_regular(file, &symbol, &sparse);
            self.objects[file].symbols[symbol.index as usize] = created;
        }
        for (id, tag, anti) in weak_aliases {
            let target = self.objects[file]
                .symbols
                .get(tag as usize)
                .copied()
                .flatten();
            let Some(target) = target else {
                continue;
            };
            if let CoffSymbolKind::Undefined {
                alias,
                anti_dependency,
            } = self.symbols[id].kind.clone()
            {
                if let Some(existing) = alias {
                    if existing != target {
                        if anti {
                            continue;
                        }
                        if !anti_dependency {
                            self.report_duplicate(id, Some(file));
                        }
                    }
                }
                self.symbols[id].kind = CoffSymbolKind::Undefined {
                    alias: Some(target),
                    anti_dependency: anti,
                };
            }
        }
    }

    fn add_associative(&mut self, parent: usize, child: usize) {
        let name = self.chunks[child].name.clone();
        let children = &self.chunks[parent].children;
        let position = children
            .iter()
            .position(|existing| self.chunks[*existing].name <= name)
            .unwrap_or(children.len());
        self.chunks[parent].children.insert(position, child);
    }

    fn create_regular(
        &mut self,
        file: usize,
        symbol: &crate::compilers::pe::CoffSymbol,
        sparse: &[Option<Option<usize>>],
    ) -> Option<SymbolIndex> {
        let chunk = sparse
            .get(symbol.section_number as usize)
            .cloned()
            .flatten()
            .flatten();
        if symbol.is_external() {
            if let Some(chunk) = chunk {
                return Some(self.add_regular(
                    file,
                    &symbol.name,
                    chunk,
                    symbol.value,
                    symbol.kind,
                    false,
                ));
            }
            return Some(self.add_undefined(&symbol.name, true, false));
        }
        let chunk = chunk?;
        let value = if symbol.storage_class == crate::compilers::pe::storage_class::SECTION {
            0
        } else {
            symbol.value
        };
        Some(self.local_symbol(
            "",
            CoffSymbolKind::Regular {
                file: Some(file),
                chunk: Some(chunk),
                value,
                comdat: false,
                external: false,
                weak: false,
                kind: symbol.kind,
            },
        ))
    }

    fn create_defined(
        &mut self,
        file: usize,
        position: usize,
        sparse: &mut [Option<Option<usize>>],
        comdat_definitions: &mut [Option<crate::compilers::pe::SectionDefinition>],
        big: bool,
    ) -> Option<Option<SymbolIndex>> {
        use crate::compilers::pe::{
            comdat_selection, section_characteristic as character, section_number,
        };
        let symbol = self.objects[file].object.symbols[position].clone();
        if symbol.is_common() {
            let size = u64::from(symbol.value);
            let alignment = size.next_power_of_two().min(32);
            let index = self.chunks.len();
            let mut chunk = CoffChunk::synthetic(
                CoffChunkKind::Common { symbol: usize::MAX },
                size,
                alignment,
                character::UNINITIALIZED_DATA | character::READ | character::WRITE,
            );
            chunk.has_data = false;
            chunk.live = false;
            chunk.name = ".bss".to_string();
            self.chunks.push(chunk);
            self.objects[file].chunks.push(index);
            let id = self.add_common(file, &symbol.name, size, index);
            self.chunks[index].kind = CoffChunkKind::Common { symbol: id };
            return Some(Some(id));
        }
        if symbol.section_number == section_number::ABSOLUTE {
            if symbol.name == "@feat.00" {
                self.objects[file].feature_flags = symbol.value;
            }
            if symbol.name == "@feat.00" || symbol.name == "@comp.id" {
                return Some(None);
            }
            if symbol.is_external() {
                return Some(Some(
                    self.add_absolute(&symbol.name, u64::from(symbol.value)),
                ));
            }
            return Some(Some(self.local_symbol(
                &symbol.name,
                CoffSymbolKind::Absolute {
                    address: u64::from(symbol.value),
                },
            )));
        }
        if symbol.section_number == section_number::DEBUG {
            return Some(None);
        }
        if symbol.section_number == 0 {
            return Some(None);
        }
        let number = symbol.section_number as usize;
        if number >= sparse.len() {
            let label = self.file_label(Some(file));
            self.error(
                problem("EI230", "A relocation names a symbol that does not exist.")
                    .with_context(format!("File: {label}"))
                    .with_context(format!("Symbol: {}", symbol.name)),
            );
            return Some(None);
        }
        if let Some(definition) = comdat_definitions[number].take() {
            let (leader, mut prevailing) = if symbol.is_external() {
                self.add_comdat(file, &symbol.name, symbol.value, symbol.kind)
            } else {
                (
                    self.local_symbol(
                        "",
                        CoffSymbolKind::Regular {
                            file: Some(file),
                            chunk: None,
                            value: symbol.value,
                            comdat: false,
                            external: false,
                            weak: false,
                            kind: symbol.kind,
                        },
                    ),
                    true,
                )
            };
            if definition.selection < comdat_selection::NO_DUPLICATES
                || definition.selection > comdat_selection::LARGEST
            {
                let label = self.file_label(Some(file));
                self.error(
                    problem(
                        "EI212",
                        "The linker does not support what this input asks for yet.",
                    )
                    .with_context(format!(
                        "Feature: comdat selection {}",
                        definition.selection
                    ))
                    .with_context(format!("File: {label}")),
                );
                return Some(None);
            }
            let mut selection = definition.selection;
            let leader_is_comdat = matches!(
                self.symbols[leader].kind,
                CoffSymbolKind::Regular { comdat: true, .. }
            );
            if leader_is_comdat && !prevailing {
                self.handle_comdat_selection(
                    file,
                    &symbol,
                    &mut selection,
                    &mut prevailing,
                    leader,
                );
            }
            if prevailing {
                let chunk = self.read_section(file, number, definition.check_sum, &symbol.name);
                sparse[number] = Some(chunk);
                let Some(chunk) = chunk else {
                    return Some(None);
                };
                self.chunks[chunk].leader = Some(leader);
                self.chunks[chunk].selection = selection;
                if let CoffSymbolKind::Regular {
                    chunk: slot,
                    file: owner,
                    value,
                    ..
                } = &mut self.symbols[leader].kind
                {
                    *slot = Some(chunk);
                    *owner = Some(file);
                    *value = symbol.value;
                }
            } else {
                sparse[number] = Some(None);
            }
            return Some(Some(leader));
        }
        if sparse[number].is_none() {
            if let Some(definition) = symbol.section_definition(big) {
                if definition.selection != comdat_selection::ASSOCIATIVE {
                    comdat_definitions[number] = Some(definition);
                }
            }
            return None;
        }
        Some(self.create_regular(file, &symbol, sparse))
    }

    fn handle_comdat_selection(
        &mut self,
        file: usize,
        symbol: &crate::compilers::pe::CoffSymbol,
        selection: &mut u8,
        prevailing: &mut bool,
        leader: SymbolIndex,
    ) {
        use crate::compilers::pe::comdat_selection;
        let CoffSymbolKind::Regular {
            chunk: Some(leader_chunk),
            ..
        } = self.symbols[leader].kind.clone()
        else {
            return;
        };
        let mut leader_selection = self.chunks[leader_chunk].selection;
        if (*selection == comdat_selection::ANY && leader_selection == comdat_selection::LARGEST)
            || (*selection == comdat_selection::LARGEST
                && leader_selection == comdat_selection::ANY)
        {
            leader_selection = comdat_selection::LARGEST;
            *selection = comdat_selection::LARGEST;
        }
        if self.options.mingw
            && ((*selection == comdat_selection::ANY
                && leader_selection == comdat_selection::SAME_SIZE)
                || (*selection == comdat_selection::SAME_SIZE
                    && leader_selection == comdat_selection::ANY))
        {
            leader_selection = comdat_selection::SAME_SIZE;
            *selection = comdat_selection::SAME_SIZE;
        }
        if *selection != leader_selection {
            self.report_duplicate(leader, Some(file));
            return;
        }
        let new_section = &self.objects[file].object.sections[symbol.section_number as usize - 1];
        let new_size = u64::from(new_section.size_of_raw_data);
        match *selection {
            comdat_selection::NO_DUPLICATES => self.report_duplicate(leader, Some(file)),
            comdat_selection::ANY => {}
            comdat_selection::SAME_SIZE => {
                if self.chunks[leader_chunk].size != new_size {
                    self.report_duplicate(leader, Some(file));
                }
            }
            comdat_selection::EXACT_MATCH => {
                let new_data = new_section.data.clone();
                if self.section_data(leader_chunk) != new_data.as_slice() {
                    self.report_duplicate(leader, Some(file));
                }
            }
            comdat_selection::LARGEST if self.chunks[leader_chunk].size < new_size => {
                self.symbols[leader].kind = CoffSymbolKind::Regular {
                    file: Some(file),
                    chunk: None,
                    value: symbol.value,
                    comdat: true,
                    external: true,
                    weak: false,
                    kind: symbol.kind,
                };
                *prevailing = true;
            }
            _ => {}
        }
    }

    pub fn section_data(&self, chunk: usize) -> &[u8] {
        let data = &self.chunks[chunk];
        match data.file {
            Some(file) if data.kind == CoffChunkKind::Section => {
                &self.objects[file].object.sections[data.section - 1].data
            }
            _ => &[],
        }
    }

    pub fn section_relocations(&self, chunk: usize) -> &[crate::compilers::pe::CoffRelocation] {
        let data = &self.chunks[chunk];
        match data.file {
            Some(file) if data.kind == CoffChunkKind::Section => {
                &self.objects[file].object.sections[data.section - 1].relocations
            }
            _ => &[],
        }
    }

    fn add_import_file(&mut self, name: &str, parent: &str, bytes: &[u8]) {
        use crate::compilers::pe::{import_type, ImportObject};
        let object = match ImportObject::parse(bytes) {
            Ok(object) => object,
            Err(diagnostic) => {
                self.error(diagnostic.with_context(format!("File: {parent}({name})")));
                return;
            }
        };
        let index = self.imports.len();
        let external_name = object.imported_name();
        let library = object.library_name.clone();
        let symbol_name = object.symbol_name.clone();
        let kind = object.kind;
        let machine_number = object.machine;
        self.imports.push(CoffImportInput {
            name: name.to_string(),
            parent: parent.to_string(),
            object,
            live: !self.collect,
            external_name,
            library,
            ..CoffImportInput::default()
        });
        let import_symbol = self.add_import_data(&format!("__imp_{symbol_name}"), index);
        self.imports[index].import_symbol = import_symbol;
        if import_symbol.is_some() {
            if kind == import_type::CONSTANT {
                self.add_import_data(&symbol_name, index);
            }
            if kind == import_type::CODE {
                let alignment = if machine_number == crate::compilers::pe::machine::AMD64 {
                    16
                } else {
                    1
                };
                let chunk = self.chunks.len();
                let mut thunk = CoffChunk::synthetic(
                    CoffChunkKind::ImportThunk { import: index },
                    6,
                    alignment,
                    crate::compilers::pe::section_characteristic::READ
                        | crate::compilers::pe::section_characteristic::EXECUTE,
                );
                thunk.live = !self.collect;
                self.chunks.push(thunk);
                self.imports[index].thunk_chunk = Some(chunk);
                self.imports[index].thunk_symbol =
                    self.add_import_thunk(&symbol_name, index, chunk);
            }
        }
        if machine_number != 0 {
            if self.machine == 0 {
                self.machine = machine_number;
            } else if self.machine != machine_number {
                let found = crate::compilers::pe::machine_name(machine_number);
                let expected = crate::compilers::pe::machine_name(self.machine);
                self.error(
                    problem(
                        "EI209",
                        "The input targets a different machine than the output.",
                    )
                    .with_context(format!("File: {parent}({name})"))
                    .with_context(format!("Machine: {found}"))
                    .with_context(format!("Expected: {expected}")),
                );
            }
        }
    }

    fn parse_directives(&mut self, file: Option<usize>) {
        let Some(file) = file else {
            return;
        };
        let text = self.objects[file].directives.clone();
        if text.is_empty() {
            return;
        }
        for token in tokenize_windows_command_line(&text) {
            let lowered = token.to_ascii_lowercase();
            if lowered.starts_with("/export:") || lowered.starts_with("-export:") {
                let value = token[8..].to_string();
                if !self.directive_exports.insert(value.clone()) {
                    continue;
                }
                match parse_coff_export(&value) {
                    Ok(mut export) => {
                        export.source = export_source::DIRECTIVES;
                        self.exports.push(export);
                    }
                    Err(diagnostic) => self.error(diagnostic),
                }
                continue;
            }
            if lowered.starts_with("/include:") || lowered.starts_with("-include:") {
                let value = token[9..].to_string();
                self.add_gc_root(&value);
                continue;
            }
            if lowered.starts_with("/exclude-symbols:") || lowered.starts_with("-exclude-symbols:")
            {
                continue;
            }
            let Some((name, value)) = split_coff_option(&token) else {
                continue;
            };
            let value = value.unwrap_or_default();
            match name.as_str() {
                "aligncomm" => self.parse_align_common(&value),
                "alternatename" => self.parse_alternate_name(&value),
                "defaultlib" => {
                    if let Some(path) = self.find_library_if_new(&value) {
                        self.enqueue_path(&path, false, false);
                    }
                }
                "entry" => {
                    let mangled = self.mangle(&value);
                    let id = self.add_gc_root(&mangled);
                    self.entry = Some(id);
                }
                "include" => {
                    self.add_gc_root(&value);
                }
                "merge" => self.parse_merge(&value),
                "nodefaultlib" => self.options.no_default_libraries.push(value),
                "release" => self.write_check_sum = true,
                "section" => self.parse_section(&value),
                "stack" => {
                    if let Ok(pair) = coff_pair("/stack", &value, self.options.stack) {
                        self.options.stack = pair;
                    }
                }
                "subsystem" => {
                    let (system, version) = value.split_once(',').unwrap_or((value.as_str(), ""));
                    if let Some(system) = coff_subsystem_from_name(system) {
                        let version = if version.is_empty() {
                            None
                        } else {
                            coff_version("/subsystem", version).ok()
                        };
                        if let Some(version) = version {
                            self.options.os_version = Some(version);
                        }
                        self.options.subsystem = Some((system, version));
                    }
                }
                "failifmismatch" | "manifestdependency" | "editandcontinue" | "guardsym"
                | "throwingnew" | "inferasanlibs" => {}
                _ => {
                    let label = self.file_label(Some(file));
                    self.error(
                        problem("EI202", "The linker does not recognize this option.")
                            .with_context(format!("Option: {token}"))
                            .with_context(format!("File: {label}")),
                    );
                }
            }
        }
    }

    pub fn parse_align_common(&mut self, value: &str) {
        let Some((name, alignment)) = value.split_once(',') else {
            self.error(invalid_value("/aligncomm", value));
            return;
        };
        let Some(shift) = coff_number(alignment) else {
            self.error(invalid_value("/aligncomm", value));
            return;
        };
        let entry = self.align_common.entry(name.to_string()).or_insert(0);
        *entry = (*entry).max(1u64 << shift);
    }

    pub fn parse_alternate_name(&mut self, value: &str) {
        let Some((from, to)) = value.split_once('=') else {
            self.error(invalid_value("/alternatename", value));
            return;
        };
        if from.is_empty() || to.is_empty() {
            self.error(invalid_value("/alternatename", value));
            return;
        }
        if let Some(existing) = self.alternate_names.get(from) {
            if existing != to {
                self.error(invalid_value("/alternatename", value));
            }
            return;
        }
        self.alternate_names
            .insert(from.to_string(), to.to_string());
    }

    pub fn parse_merge(&mut self, value: &str) {
        let Some((from, to)) = value.split_once('=') else {
            self.error(invalid_value("/merge", value));
            return;
        };
        if from.is_empty()
            || to.is_empty()
            || from == ".rsrc"
            || to == ".rsrc"
            || from == ".reloc"
            || to == ".reloc"
        {
            self.error(invalid_value("/merge", value));
            return;
        }
        if let Some(existing) = self.merges.get(from) {
            if existing != to {
                self.warn(
                    caution(
                        "EI247",
                        "The linker ignores an option it does not recognize.",
                    )
                    .with_context(format!("Option: /merge:{value}")),
                );
            }
            return;
        }
        self.merges.insert(from.to_string(), to.to_string());
    }

    pub fn parse_section(&mut self, value: &str) {
        use crate::compilers::pe::section_characteristic as character;
        let Some((name, attributes)) = value.split_once(',') else {
            self.error(invalid_value("/section", value));
            return;
        };
        if name.is_empty() || attributes.is_empty() {
            self.error(invalid_value("/section", value));
            return;
        }
        let mut permissions = 0u32;
        for letter in attributes.to_ascii_lowercase().chars() {
            permissions |= match letter {
                'd' => character::DISCARDABLE,
                'e' => character::EXECUTE,
                'k' => character::NOT_CACHED,
                'p' => character::NOT_PAGED,
                'r' => character::READ,
                's' => character::SHARED,
                'w' => character::WRITE,
                _ => {
                    self.error(invalid_value("/section", value));
                    return;
                }
            };
        }
        self.section_permissions
            .insert(name.to_string(), permissions);
    }
}

pub fn parse_coff_export(argument: &str) -> Result<CoffExport, Diagnostic> {
    let failure = || invalid_value("/export", argument);
    let mut export = CoffExport {
        source: export_source::EXPORT,
        ..CoffExport::default()
    };
    let mut parts = argument.splitn(2, ',');
    let name = parts.next().unwrap_or("");
    let mut rest = parts.next().unwrap_or("").to_string();
    if name.is_empty() {
        return Err(failure());
    }
    export.name = name.to_string();
    if let Some((left, right)) = name.split_once('=') {
        if right.contains('.') {
            export.name = left.to_string();
            export.forward_to = right.to_string();
        } else {
            export.external_name = left.to_string();
            export.name = right.to_string();
            if export.name.is_empty() {
                return Err(failure());
            }
        }
    }
    while !rest.is_empty() {
        let (token, remainder) = match rest.split_once(',') {
            Some((token, remainder)) => (token.to_string(), remainder.to_string()),
            None => (rest.clone(), String::new()),
        };
        rest = remainder;
        let lowered = token.to_ascii_lowercase();
        if lowered == "noname" {
            if export.ordinal == 0 {
                return Err(failure());
            }
            export.no_name = true;
            continue;
        }
        if lowered == "data" {
            export.data = true;
            continue;
        }
        if lowered == "constant" {
            export.constant = true;
            continue;
        }
        if lowered == "private" {
            export.private = true;
            continue;
        }
        if lowered == "exportas" {
            if !rest.is_empty() && !rest.contains(',') {
                export.export_as = rest.clone();
            } else {
                return Err(failure());
            }
            break;
        }
        if let Some(number) = token.strip_prefix('@') {
            let ordinal = coff_number(number).ok_or_else(failure)?;
            if ordinal == 0 || ordinal > 65535 {
                return Err(failure());
            }
            export.ordinal = ordinal as u16;
            continue;
        }
        return Err(failure());
    }
    Ok(export)
}

impl<'s> CoffContext<'s> {
    pub fn run(&mut self) -> Option<Vec<u8>> {
        use crate::compilers::pe::{machine, subsystem};
        if self
            .options
            .inputs
            .iter()
            .all(|input| matches!(input, CoffInput::StartLibrary | CoffInput::EndLibrary))
        {
            self.error(problem("EI213", "No input files were given."));
            return None;
        }
        self.search_paths.push(String::new());
        self.search_paths.extend(self.options.library_paths.clone());
        let debug_collect = !self.options.debug;
        self.collect = self.options.collect.unwrap_or(debug_collect);
        self.folding = self.options.folding.unwrap_or(if self.collect {
            IdenticalCodeFolding::All
        } else {
            IdenticalCodeFolding::None
        });
        self.tail_merge = (self.options.tail_merge == 1
            && self.folding != IdenticalCodeFolding::None)
            || self.options.tail_merge == 2;
        if self.options.no_entry && !self.options.dll {
            self.error(
                problem("EI204", "An option value is not valid.")
                    .with_context("Option: /noentry must be specified with /dll".to_string()),
            );
        }
        self.dynamic_base = self.options.dynamic_base.unwrap_or(true);
        if self.options.fixed {
            if self.options.dynamic_base == Some(true) {
                self.error(
                    problem("EI204", "An option value is not valid.").with_context(
                        "Option: /fixed must not be specified with /dynamicbase".to_string(),
                    ),
                );
            } else {
                self.relocatable = false;
                self.dynamic_base = false;
            }
        }
        if let Some((system, version)) = self.options.subsystem {
            self.subsystem = system;
            if let Some(version) = version {
                self.subsystem_version = version;
            }
        }
        self.os_version = self.options.os_version.unwrap_or(self.subsystem_version);
        for value in self.options.alternate_names.clone() {
            self.parse_alternate_name(&value);
        }
        for value in self.options.includes.clone() {
            self.add_gc_root(&value);
        }
        for value in self.options.merges.clone() {
            self.parse_merge(&value);
        }
        for default in [
            ".idata=.rdata",
            ".didat=.rdata",
            ".edata=.rdata",
            ".xdata=.rdata",
            ".00cfg=.rdata",
            ".bss=.data",
        ] {
            self.parse_merge(default);
        }
        if self.options.mingw {
            for default in [
                ".ctors=.rdata",
                ".dtors=.rdata",
                ".CRT=.rdata",
                ".data_cygwin_nocopy=.data",
            ] {
                self.parse_merge(default);
            }
        }
        for value in self.options.sections.clone() {
            self.parse_section(&value);
        }
        for value in self.options.align_common.clone() {
            self.parse_align_common(&value);
        }
        self.write_check_sum = self.options.release;
        if self.failed() {
            return None;
        }
        let whole_files: BTreeSet<String> = self
            .options
            .whole_archive_files
            .iter()
            .map(|path| self.find_file(path))
            .collect();
        let mut in_library = false;
        for input in self.options.inputs.clone() {
            match input {
                CoffInput::StartLibrary => in_library = true,
                CoffInput::EndLibrary => in_library = false,
                CoffInput::WholeArchive(path) => {
                    if let Some(path) = self.find_file_if_new(&path) {
                        self.enqueue_path(&path, in_library, true);
                    }
                }
                CoffInput::File(path) => {
                    if let Some(path) = self.find_file_if_new(&path) {
                        let whole = self.options.whole_archive_all || whole_files.contains(&path);
                        self.enqueue_path(&path, in_library, whole);
                    }
                }
            }
        }
        self.run_tasks();
        if self.failed() {
            return None;
        }
        if self.machine == 0 {
            self.warn(caution(
                "EI258",
                "The target machine was not given, so x64 is assumed.",
            ));
            self.machine = machine::AMD64;
        }
        for library in self.options.default_libraries.clone() {
            if let Some(path) = self.find_library_if_new(&library) {
                self.enqueue_path(&path, false, false);
            }
        }
        self.run_tasks();
        if self.failed() {
            return None;
        }
        if self.machine == machine::I386 {
            self.safe_seh = self.options.safe_seh.unwrap_or(!self.options.mingw);
        }
        for value in self.options.exports.clone() {
            match parse_coff_export(&value) {
                Ok(mut export) => {
                    if self.machine == machine::I386 {
                        if !is_coff_decorated(&export.name, self.options.mingw) {
                            export.name = format!("_{}", export.name);
                        }
                        if !export.external_name.is_empty()
                            && !is_coff_decorated(&export.external_name, self.options.mingw)
                        {
                            export.external_name = format!("_{}", export.external_name);
                        }
                    }
                    self.exports.push(export);
                }
                Err(diagnostic) => self.error(diagnostic),
            }
        }
        if let Some(path) = self.options.definition_file.clone() {
            self.parse_module_definitions(&path);
        }
        if self.failed() {
            return None;
        }
        if self.subsystem == subsystem::UNKNOWN {
            self.subsystem = self.infer_subsystem();
            if self.subsystem == subsystem::UNKNOWN {
                self.error(problem(
                    "EI249",
                    "The subsystem cannot be inferred and must be given.",
                ));
                return None;
            }
        }
        if let Some(entry) = self.options.entry.clone() {
            if entry.is_empty() {
                self.error(missing_value("/entry"));
                return None;
            }
            let mangled = self.mangle(&entry);
            self.entry = Some(self.add_gc_root(&mangled));
        } else if self.entry.is_none() && !self.options.no_entry {
            if self.options.dll {
                let name = if self.machine == machine::I386 {
                    "__DllMainCRTStartup@12"
                } else {
                    "_DllMainCRTStartup"
                };
                self.entry = Some(self.add_gc_root(name));
            } else if self.options.driver_wdm {
                let name = self.mangle("_NtProcessStartup");
                self.entry = Some(self.add_gc_root(&name));
            } else {
                let name = self.find_default_entry();
                self.entry = Some(self.add_gc_root(&name));
            }
        }
        if self.options.output.is_empty() {
            let first = self.options.inputs.iter().find_map(|input| match input {
                CoffInput::File(path) | CoffInput::WholeArchive(path) => Some(path.clone()),
                _ => None,
            });
            let first = first.unwrap_or_default();
            let stem = match first.rsplit_once('.') {
                Some((stem, _)) if !stem.ends_with(['/', '\\']) => stem.to_string(),
                _ => first.clone(),
            };
            let extension = if self.options.dll {
                ".dll"
            } else if self.options.driver {
                ".sys"
            } else {
                ".exe"
            };
            self.options.output = format!("{stem}{extension}");
        }
        self.image_base = self.options.image_base.unwrap_or(if self.is_64() {
            if self.options.dll {
                0x1_8000_0000
            } else {
                0x1_4000_0000
            }
        } else if self.options.dll {
            0x1000_0000
        } else {
            0x40_0000
        });
        let image_base_name = self.mangle("__ImageBase");
        self.add_synthetic(&image_base_name, None);
        if self.machine == machine::I386 {
            self.add_absolute("___safe_se_handler_table", 0);
            self.add_absolute("___safe_se_handler_count", 0);
        }
        for name in [
            "__guard_fids_count",
            "__guard_fids_table",
            "__guard_flags",
            "__guard_iat_count",
            "__guard_iat_table",
            "__guard_longjmp_count",
            "__guard_longjmp_table",
            "__enclave_config",
            "__guard_eh_cont_count",
            "__guard_eh_cont_table",
        ] {
            let mangled = self.mangle(name);
            self.add_absolute(&mangled, 0);
        }
        loop {
            if let Some(entry) = self.entry {
                self.mangle_maybe(entry);
            }
            for position in 0..self.exports.len() {
                if !self.exports[position].forward_to.is_empty() {
                    continue;
                }
                let name = self.exports[position].name.clone();
                let symbol = self.add_gc_root(&name);
                self.exports[position].symbol = Some(symbol);
                if self.exports[position].source != export_source::DIRECTIVES {
                    let mangled = self.mangle_maybe(symbol);
                    self.exports[position].symbol_name = mangled;
                }
            }
            self.resolve_alternate_names();
            if self.find_underscore("_load_config_used").is_some() {
                let name = self.mangle("_load_config_used");
                self.add_gc_root(&name);
            }
            if !self.run_tasks() {
                break;
            }
        }
        self.had_explicit_exports = !self.exports.is_empty();
        if let Some(id) = self.find_underscore("_tls_used") {
            if self.symbols[id].is_defined() && !self.gc_roots.contains(&id) {
                self.gc_roots.push(id);
            }
        }
        self.run_tasks();
        let aliases = self.resolve_remaining_undefined();
        if self.failed() {
            return None;
        }
        self.initialize_load_config();
        if self.collect {
            self.mark_live();
        }
        for alias in aliases {
            self.resolve_weak_alias(alias);
        }
        if self.options.dll || !self.exports.is_empty() {
            self.fix_exports();
            self.assign_export_ordinals();
        }
        for (name, alignment) in self.align_common.clone() {
            match self.find(&name) {
                None => self.warn(
                    caution(
                        "EI247",
                        "The linker ignores an option it does not recognize.",
                    )
                    .with_context(format!("Option: /aligncomm:{name}")),
                ),
                Some(id) => {
                    if let CoffSymbolKind::Common { chunk, .. } = self.symbols[id].kind {
                        self.chunks[chunk].alignment = self.chunks[chunk].alignment.max(alignment);
                    }
                }
            }
        }
        if self.folding != IdenticalCodeFolding::None {
            self.fold_identical_sections();
        }
        if self.failed() {
            return None;
        }
        self.write_image()
    }

    fn infer_subsystem(&self) -> u16 {
        use crate::compilers::pe::subsystem;
        if self.options.dll {
            return subsystem::WINDOWS_GRAPHICAL;
        }
        if self.options.mingw {
            return subsystem::WINDOWS_CONSOLE;
        }
        let has = |name: &str| self.find_underscore_mangle(name);
        if has("main") || has("wmain") {
            return subsystem::WINDOWS_CONSOLE;
        }
        if has("WinMain") || has("wWinMain") {
            return subsystem::WINDOWS_GRAPHICAL;
        }
        subsystem::UNKNOWN
    }

    fn find_default_entry(&self) -> String {
        use crate::compilers::pe::subsystem;
        if self.options.mingw {
            return self.mangle(if self.subsystem == subsystem::WINDOWS_GRAPHICAL {
                "WinMainCRTStartup"
            } else {
                "mainCRTStartup"
            });
        }
        if self.subsystem == subsystem::WINDOWS_GRAPHICAL {
            if self.find_underscore_mangle("wWinMain") && !self.find_underscore_mangle("WinMain") {
                return self.mangle("wWinMainCRTStartup");
            }
            return self.mangle("WinMainCRTStartup");
        }
        if self.find_underscore_mangle("wmain") && !self.find_underscore_mangle("main") {
            return self.mangle("wmainCRTStartup");
        }
        self.mangle("mainCRTStartup")
    }

    fn find_mangle(&self, name: &str) -> Option<SymbolIndex> {
        if let Some(id) = self.find(name) {
            if let CoffSymbolKind::Undefined { alias, .. } = self.symbols[id].kind {
                if let Some(alias) = self.weak_alias_target(id) {
                    return Some(alias);
                }
                let _ = alias;
            } else {
                return Some(id);
            }
        }
        let by_prefix = |prefix: &str| -> Option<SymbolIndex> {
            self.names
                .iter()
                .find(|(key, _)| key.starts_with(prefix))
                .map(|(_, id)| *id)
        };
        if self.machine != crate::compilers::pe::machine::I386 {
            return by_prefix(&format!("?{name}@@Y"));
        }
        let stripped = name.strip_prefix('_')?;
        by_prefix(&format!("{name}@"))
            .or_else(|| by_prefix(&format!("@{stripped}@")))
            .or_else(|| by_prefix(&format!("{stripped}@@")))
            .or_else(|| by_prefix(&format!("?{stripped}@@Y")))
    }

    fn find_underscore_mangle(&self, name: &str) -> bool {
        let mangled = self.mangle(name);
        self.find_mangle(&mangled)
            .is_some_and(|id| !self.symbols[id].is_undefined())
    }

    fn mangle_maybe(&mut self, id: SymbolIndex) -> String {
        if !self.symbols[id].is_undefined() {
            return String::new();
        }
        let name = self.symbols[id].name.clone();
        let Some(mangled) = self.find_mangle(&name) else {
            return String::new();
        };
        let mangled_name = self.symbols[mangled].name.clone();
        let target = self.add_undefined(&mangled_name, true, false);
        if let CoffSymbolKind::Undefined { alias, .. } = &mut self.symbols[id].kind {
            *alias = Some(target);
        }
        mangled_name
    }

    fn weak_alias_target(&self, id: SymbolIndex) -> Option<SymbolIndex> {
        let mut seen = BTreeSet::new();
        let mut current = match &self.symbols[id].kind {
            CoffSymbolKind::Undefined { alias, .. } => *alias,
            _ => None,
        };
        while let Some(candidate) = current {
            match &self.symbols[candidate].kind {
                CoffSymbolKind::Undefined {
                    alias,
                    anti_dependency,
                } => {
                    if *anti_dependency || !seen.insert(candidate) {
                        return None;
                    }
                    current = *alias;
                }
                _ => {
                    if let CoffSymbolKind::Undefined {
                        anti_dependency: true,
                        ..
                    } = self.symbols[id].kind
                    {
                        return None;
                    }
                    return Some(candidate);
                }
            }
        }
        None
    }

    fn defined_target(&self, id: SymbolIndex) -> Option<SymbolIndex> {
        if self.symbols[id].is_defined() {
            return Some(id);
        }
        if self.symbols[id].is_undefined() {
            return self
                .weak_alias_target(id)
                .filter(|target| self.symbols[*target].is_defined());
        }
        None
    }

    fn resolve_alternate_names(&mut self) {
        for (from, to) in self.alternate_names.clone() {
            let Some(id) = self.find(&from) else {
                continue;
            };
            let CoffSymbolKind::Undefined { alias, .. } = self.symbols[id].kind.clone() else {
                continue;
            };
            if alias.is_some() {
                continue;
            }
            let Some(target) = self.find(&to) else {
                continue;
            };
            if let CoffSymbolKind::Undefined {
                alias: target_alias,
                anti_dependency,
            } = self.symbols[target].kind
            {
                if target_alias.is_none() || anti_dependency {
                    continue;
                }
            }
            self.symbols[target].used_in_regular_object = true;
            if self.symbols[target].is_lazy() {
                self.force_lazy(target);
            }
            self.symbols[id].kind = CoffSymbolKind::Undefined {
                alias: Some(target),
                anti_dependency: false,
            };
        }
    }

    fn resolve_remaining_undefined(&mut self) -> Vec<SymbolIndex> {
        let mut aliases = Vec::new();
        let mut undefined: BTreeSet<SymbolIndex> = BTreeSet::new();
        let mut local_imports: BTreeMap<SymbolIndex, SymbolIndex> = BTreeMap::new();
        let ids: Vec<SymbolIndex> = self
            .names
            .values()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        for id in ids {
            if !self.symbols[id].is_undefined() || !self.symbols[id].used_in_regular_object {
                continue;
            }
            if self.weak_alias_target(id).is_some() {
                aliases.push(id);
                continue;
            }
            let name = self.symbols[id].name.clone();
            if let Some(imported) = name.strip_prefix("__imp_") {
                if let Some(target) = self
                    .find(imported)
                    .and_then(|target| self.defined_target(target))
                {
                    let chunk = self.chunks.len();
                    let word = self.word_size();
                    let mut data = CoffChunk::synthetic(
                        CoffChunkKind::LocalImport { symbol: target },
                        word,
                        word,
                        0,
                    );
                    data.name = ".rdata".to_string();
                    self.chunks.push(data);
                    self.symbols[id].kind = CoffSymbolKind::LocalImport { target, chunk };
                    self.local_import_chunks.push(chunk);
                    local_imports.insert(id, target);
                    continue;
                }
            }
            if name.contains("_PchSym_") {
                continue;
            }
            if self.options.force_unresolved {
                self.symbols[id].kind = CoffSymbolKind::Absolute { address: 0 };
            }
            undefined.insert(id);
        }
        self.report_problem_symbols(&undefined, &local_imports);
        aliases
    }

    fn report_problem_symbols(
        &mut self,
        undefined: &BTreeSet<SymbolIndex>,
        local_imports: &BTreeMap<SymbolIndex, SymbolIndex>,
    ) {
        if undefined.is_empty() && local_imports.is_empty() {
            return;
        }
        let warn_local = !self
            .options
            .ignored_warnings
            .iter()
            .any(|code| code == "4217");
        for root in self.gc_roots.clone() {
            if undefined.contains(&root) {
                let name = self.symbols[root].name.clone();
                let diagnostic = problem("EI207", "A symbol is referenced but never defined.")
                    .with_context(format!("Symbol: {name}"))
                    .with_context("Reference: <root>".to_string());
                if self.options.force_unresolved {
                    self.warn(diagnostic);
                } else {
                    self.error(diagnostic);
                }
            }
            if warn_local {
                if let Some(target) = local_imports.get(&root) {
                    let name = self.symbols[*target].name.clone();
                    self.warn(
                        caution("EI259", "A locally defined symbol is imported.")
                            .with_context(format!("Symbol: {name}"))
                            .with_context("Reference: <root>".to_string()),
                    );
                }
            }
        }
        let mut order: Vec<SymbolIndex> = Vec::new();
        let mut references: BTreeMap<SymbolIndex, Vec<(usize, u32)>> = BTreeMap::new();
        for file in self.object_order.clone() {
            for (index, symbol) in self.objects[file].symbols.clone().into_iter().enumerate() {
                let Some(symbol) = symbol else {
                    continue;
                };
                if undefined.contains(&symbol) {
                    if !references.contains_key(&symbol) {
                        order.push(symbol);
                    }
                    references
                        .entry(symbol)
                        .or_default()
                        .push((file, index as u32));
                }
                if warn_local {
                    if let Some(target) = local_imports.get(&symbol) {
                        let name = self.symbols[*target].name.clone();
                        let label = self.file_label(Some(file));
                        self.warn(
                            caution("EI259", "A locally defined symbol is imported.")
                                .with_context(format!("Symbol: {name}"))
                                .with_context(format!("Reference: {label}")),
                        );
                    }
                }
            }
        }
        for symbol in order {
            let name = self.symbols[symbol].name.clone();
            let mut diagnostic = problem("EI207", "A symbol is referenced but never defined.")
                .with_context(format!("Symbol: {name}"));
            let mut shown = 0usize;
            let mut total = 0usize;
            for (file, index) in &references[&symbol] {
                let mut locations: Vec<String> = Vec::new();
                for chunk in self.objects[*file].chunks.clone() {
                    if self.chunks[chunk].kind != CoffChunkKind::Section {
                        continue;
                    }
                    for relocation in self.section_relocations(chunk).to_vec() {
                        if relocation.symbol_index != *index {
                            continue;
                        }
                        total += 1;
                        if shown + locations.len() >= 3 {
                            continue;
                        }
                        let label = self.file_label(Some(*file));
                        match self.enclosing_symbol(*file, chunk, relocation.virtual_address) {
                            Some(owner) => locations.push(format!("{label}:({owner})")),
                            None => locations.push(label),
                        }
                    }
                }
                if total == 0 && shown < 3 {
                    locations.push(self.file_label(Some(*file)));
                    total += 1;
                }
                shown += locations.len();
                for location in locations {
                    diagnostic = diagnostic.with_context(format!("Reference: {location}"));
                }
            }
            if shown < total {
                diagnostic = diagnostic.with_context(format!("More references: {}", total - shown));
            }
            if self.options.force_unresolved {
                self.warn(diagnostic);
            } else {
                self.error(diagnostic);
            }
        }
    }

    fn enclosing_symbol(&self, file: usize, chunk: usize, address: u32) -> Option<String> {
        let mut best: Option<(u32, String)> = None;
        for symbol in self.objects[file].symbols.iter().flatten() {
            if let CoffSymbolKind::Regular {
                file: Some(owner),
                chunk: Some(target),
                value,
                ..
            } = &self.symbols[*symbol].kind
            {
                if *owner != file || *target != chunk || *value > address {
                    continue;
                }
                if best.as_ref().is_some_and(|(held, _)| *value < *held) {
                    continue;
                }
                best = Some((*value, self.symbols[*symbol].name.clone()));
            }
        }
        best.map(|(_, name)| name).filter(|name| !name.is_empty())
    }

    fn resolve_weak_alias(&mut self, id: SymbolIndex) {
        let Some(target) = self.weak_alias_target(id) else {
            return;
        };
        if !self.symbols[target].is_defined() {
            return;
        }
        let kind = self.symbols[target].kind.clone();
        self.symbols[id].kind = kind;
    }

    fn initialize_load_config(&mut self) {
        let Some(id) = self.find_underscore("_load_config_used") else {
            return;
        };
        let CoffSymbolKind::Regular {
            chunk: Some(chunk),
            value,
            ..
        } = self.symbols[id].kind.clone()
        else {
            return;
        };
        if !self.chunks[chunk].has_data {
            self.error(
                problem(
                    "EI212",
                    "The linker does not support what this input asks for yet.",
                )
                .with_context("Feature: _load_config_used in uninitialized data".to_string()),
            );
            return;
        }
        let data = self.section_data(chunk).to_vec();
        let offset = value as usize;
        if offset + 4 > data.len() {
            self.error(
                problem(
                    "EI212",
                    "The linker does not support what this input asks for yet.",
                )
                .with_context("Feature: a _load_config_used section that is too small".to_string()),
            );
            return;
        }
        let size = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        if offset + size as usize > data.len() {
            self.error(
                problem(
                    "EI212",
                    "The linker does not support what this input asks for yet.",
                )
                .with_context(
                    "Feature: a _load_config_used size larger than its section".to_string(),
                ),
            );
            return;
        }
        self.load_config_size = size;
        self.load_config_symbol = Some(id);
    }

    fn mark_live(&mut self) {
        let mut worklist: Vec<usize> = Vec::new();
        for file in self.object_order.clone() {
            for chunk in self.objects[file].chunks.clone() {
                let data = &self.chunks[chunk];
                if data.kind == CoffChunkKind::Section
                    && data.live
                    && !data.name.starts_with(".debug_")
                {
                    worklist.push(chunk);
                }
            }
        }
        for root in self.gc_roots.clone() {
            self.mark_symbol_live(root, &mut worklist);
        }
        while let Some(chunk) = worklist.pop() {
            if let Some(file) = self.chunks[chunk].file {
                let relocations = self.section_relocations(chunk).to_vec();
                for relocation in relocations {
                    if let Some(symbol) = self.objects[file]
                        .symbols
                        .get(relocation.symbol_index as usize)
                        .copied()
                        .flatten()
                    {
                        self.mark_symbol_live(symbol, &mut worklist);
                    }
                }
            }
            for child in self.chunks[chunk].children.clone() {
                if !self.chunks[child].live {
                    self.chunks[child].live = true;
                    worklist.push(child);
                }
            }
        }
    }

    fn mark_symbol_live(&mut self, id: SymbolIndex, worklist: &mut Vec<usize>) {
        let Some(defined) = self.defined_target(id) else {
            return;
        };
        match self.symbols[defined].kind.clone() {
            CoffSymbolKind::Regular {
                chunk: Some(chunk), ..
            } => {
                let chunk = self.chunks[chunk].replacement;
                if self.chunks[chunk].kind == CoffChunkKind::Section && !self.chunks[chunk].live {
                    self.chunks[chunk].live = true;
                    worklist.push(chunk);
                }
            }
            CoffSymbolKind::ImportData { import } => self.imports[import].live = true,
            CoffSymbolKind::ImportThunk { import, chunk } => {
                self.imports[import].live = true;
                self.chunks[chunk].live = true;
            }
            _ => {}
        }
    }

    fn fold_identical_sections(&mut self) {
        use crate::compilers::pe::section_characteristic as character;
        let mut candidates: Vec<usize> = Vec::new();
        let mut next_class: u32 = 1;
        for file in self.object_order.clone() {
            for chunk in self.objects[file].chunks.clone() {
                if self.chunks[chunk].kind != CoffChunkKind::Section {
                    continue;
                }
                if self.folding_eligible(chunk) {
                    candidates.push(chunk);
                } else {
                    self.chunks[chunk].classes[0] = next_class;
                    next_class = next_class.wrapping_add(1);
                }
            }
        }
        for members in self.merge_members.clone().values() {
            for member in members {
                self.chunks[*member].classes[0] = next_class;
                next_class = next_class.wrapping_add(1);
            }
        }
        for chunk in &candidates {
            self.chunks[*chunk].classes[0] = xxh3_64(self.section_data(*chunk)) as u32;
        }
        for round in 0..2usize {
            let mut updates = Vec::new();
            for chunk in &candidates {
                let mut hash = self.chunks[*chunk].classes[round % 2];
                for target in self.relocation_targets(*chunk) {
                    if let CoffSymbolKind::Regular {
                        chunk: Some(target_chunk),
                        ..
                    } = self.symbols[target].kind
                    {
                        let target_chunk = self.chunks[target_chunk].replacement;
                        hash = hash.wrapping_add(self.chunks[target_chunk].classes[round % 2]);
                    }
                }
                updates.push((*chunk, hash | (1 << 31)));
            }
            for (chunk, hash) in updates {
                self.chunks[chunk].classes[(round + 1) % 2] = hash;
            }
        }
        candidates.sort_by_key(|chunk| self.chunks[*chunk].classes[0]);
        let mut count = 0usize;
        let each_class = |context: &mut CoffContext,
                          candidates: &mut Vec<usize>,
                          count: usize,
                          constant: bool|
         -> bool {
            let mut repeat = false;
            let mut begin = 0usize;
            while begin < candidates.len() {
                let class = context.chunks[candidates[begin]].classes[count % 2];
                let mut end = begin + 1;
                while end < candidates.len()
                    && context.chunks[candidates[end]].classes[count % 2] == class
                {
                    end += 1;
                }
                let mut start = begin;
                while start < end {
                    let head = candidates[start];
                    let (mut same, mut different): (Vec<usize>, Vec<usize>) =
                        (Vec::new(), Vec::new());
                    for candidate in &candidates[start + 1..end] {
                        if context.sections_equal(head, *candidate, count, constant) {
                            same.push(*candidate);
                        } else {
                            different.push(*candidate);
                        }
                    }
                    let middle = start + 1 + same.len();
                    for (position, candidate) in
                        (start + 1..).zip(same.into_iter().chain(different))
                    {
                        candidates[position] = candidate;
                    }
                    for candidate in &candidates[start..middle] {
                        context.chunks[*candidate].classes[(count + 1) % 2] = middle as u32;
                    }
                    if middle != end {
                        repeat = true;
                    }
                    start = middle;
                }
                begin = end;
            }
            repeat
        };
        each_class(self, &mut candidates, count, true);
        count += 1;
        loop {
            let repeat = each_class(self, &mut candidates, count, false);
            count += 1;
            if !repeat {
                break;
            }
        }
        let mut begin = 0usize;
        while begin < candidates.len() {
            let class = self.chunks[candidates[begin]].classes[count % 2];
            let mut end = begin + 1;
            while end < candidates.len() && self.chunks[candidates[end]].classes[count % 2] == class
            {
                end += 1;
            }
            let head = candidates[begin];
            for other in candidates[begin + 1..end].iter().copied() {
                let alignment = self.chunks[head]
                    .alignment
                    .max(self.chunks[other].alignment);
                self.chunks[head].alignment = alignment;
                self.chunks[other].replacement = self.chunks[head].replacement;
                self.chunks[other].live = false;
            }
            begin = end;
        }
        let _ = character::READ;
    }

    fn folding_eligible(&self, chunk: usize) -> bool {
        use crate::compilers::pe::section_characteristic as character;
        let data = &self.chunks[chunk];
        let output = data.output_characteristics();
        if !data.comdat || !data.live || output & character::WRITE != 0 {
            return false;
        }
        if self.folding == IdenticalCodeFolding::All && output & character::EXECUTE != 0 {
            return true;
        }
        let base = data.name.split('$').next().unwrap_or("");
        if base == ".pdata" || base == ".xdata" {
            return true;
        }
        let vtable = if self.machine == crate::compilers::pe::machine::I386 {
            "__ZTV"
        } else {
            "_ZTV"
        };
        if let Some(leader) = data.leader {
            let name = &self.symbols[leader].name;
            if name.starts_with("??_7") || name.starts_with(vtable) {
                return true;
            }
        }
        !data.keep_unique
    }

    fn relocation_targets(&self, chunk: usize) -> Vec<SymbolIndex> {
        let Some(file) = self.chunks[chunk].file else {
            return Vec::new();
        };
        self.section_relocations(chunk)
            .iter()
            .filter_map(|relocation| {
                self.objects[file]
                    .symbols
                    .get(relocation.symbol_index as usize)
                    .copied()
                    .flatten()
            })
            .collect()
    }

    fn sections_equal(&self, left: usize, right: usize, count: usize, constant: bool) -> bool {
        let class_of =
            |chunk: usize| self.chunks[self.chunks[chunk].replacement].classes[count % 2];
        let symbol_equal = |first: Option<SymbolIndex>, second: Option<SymbolIndex>| -> bool {
            if first == second {
                return true;
            }
            let (Some(first), Some(second)) = (first, second) else {
                return false;
            };
            match (&self.symbols[first].kind, &self.symbols[second].kind) {
                (
                    CoffSymbolKind::Regular {
                        chunk: Some(first_chunk),
                        value: first_value,
                        ..
                    },
                    CoffSymbolKind::Regular {
                        chunk: Some(second_chunk),
                        value: second_value,
                        ..
                    },
                ) => {
                    if constant && first_value != second_value {
                        return false;
                    }
                    class_of(*first_chunk) == class_of(*second_chunk)
                }
                _ => false,
            }
        };
        let left_relocations = self.section_relocations(left);
        let right_relocations = self.section_relocations(right);
        if left_relocations.len() != right_relocations.len() {
            return false;
        }
        let left_file = self.chunks[left].file.unwrap_or(0);
        let right_file = self.chunks[right].file.unwrap_or(0);
        for (first, second) in left_relocations.iter().zip(right_relocations) {
            if constant
                && (first.kind != second.kind || first.virtual_address != second.virtual_address)
            {
                return false;
            }
            let first_symbol = self.objects[left_file]
                .symbols
                .get(first.symbol_index as usize)
                .copied()
                .flatten();
            let second_symbol = self.objects[right_file]
                .symbols
                .get(second.symbol_index as usize)
                .copied()
                .flatten();
            if !symbol_equal(first_symbol, second_symbol) {
                return false;
            }
        }
        let associated_equal = || -> bool {
            let considered = |chunk: &usize| {
                let name = &self.chunks[*chunk].name;
                !(name.starts_with(".debug")
                    || name == ".gfids$y"
                    || name == ".giats$y"
                    || name == ".gljmp$y")
            };
            let first: Vec<usize> = self.chunks[left]
                .children
                .iter()
                .copied()
                .filter(considered)
                .collect();
            let second: Vec<usize> = self.chunks[right]
                .children
                .iter()
                .copied()
                .filter(considered)
                .collect();
            first.len() == second.len()
                && first.iter().zip(&second).all(|(a, b)| {
                    self.chunks[*a].classes[count % 2] == self.chunks[*b].classes[count % 2]
                })
        };
        if constant {
            let a = &self.chunks[left];
            let b = &self.chunks[right];
            return a.output_characteristics() == b.output_characteristics()
                && a.name == b.name
                && a.size == b.size
                && a.check_sum == b.check_sum
                && self.section_data(left) == self.section_data(right)
                && associated_equal();
        }
        associated_equal()
    }

    fn parse_module_definitions(&mut self, path: &str) {
        let Some(bytes) = (self.source)(path) else {
            self.error(file_missing(path));
            return;
        };
        let text = String::from_utf8_lossy(&bytes).to_string();
        let definition = match crate::compilers::pe::parse_module_definition(
            &text,
            self.machine,
            self.options.mingw,
            true,
        ) {
            Ok(definition) => definition,
            Err(reason) => {
                self.error(
                    problem("EI246", "The module definition file could not be read.")
                        .with_context(format!("File: {path}"))
                        .with_context(format!("Reason: {reason}")),
                );
                return;
            }
        };
        if self.options.output.is_empty() {
            self.options.output = definition.output_file.clone();
        }
        if definition.image_base != 0 {
            self.options.image_base = Some(definition.image_base);
        }
        if let Some(reserve) = definition.stack_reserve {
            self.options.stack.0 = reserve;
        }
        if let Some(commit) = definition.stack_commit {
            self.options.stack.1 = commit;
        }
        if let Some(reserve) = definition.heap_reserve {
            self.options.heap.0 = reserve;
        }
        if let Some(commit) = definition.heap_commit {
            self.options.heap.1 = commit;
        }
        if definition.major_image_version != 0 {
            self.options.image_version.0 = definition.major_image_version;
        }
        if definition.minor_image_version != 0 {
            self.options.image_version.1 = definition.minor_image_version;
        }
        for entry in definition.exports {
            let mut export = CoffExport {
                source: export_source::MODULE_DEFINITION,
                ..CoffExport::default()
            };
            if !entry.external_name.is_empty()
                && entry.external_name != entry.name
                && entry.name.contains('.')
            {
                export.name = entry.external_name.clone();
                export.forward_to = entry.name.clone();
            } else {
                export.name = entry.name.clone();
                export.external_name = entry.external_name.clone();
            }
            export.export_as = entry.export_as;
            export.import_name = entry.import_name;
            export.ordinal = entry.ordinal;
            export.no_name = entry.no_name;
            export.data = entry.data;
            export.private = entry.private;
            export.constant = entry.constant;
            self.exports.push(export);
        }
    }

    fn fix_exports(&mut self) {
        use crate::compilers::pe::machine;
        let mut ordinals = BTreeSet::new();
        for export in &self.exports {
            if export.ordinal != 0 && !ordinals.insert(export.ordinal) {
                let name = export.name.clone();
                self.diagnostics.push(
                    problem("EI255", "Two exports use the same ordinal.")
                        .with_context(format!("Symbol: {name}")),
                );
                return;
            }
        }
        for export in self.exports.iter_mut() {
            if !export.export_as.is_empty() {
                export.export_name = export.export_as.clone();
                continue;
            }
            let symbol = if !export.forward_to.is_empty() || export.external_name.is_empty() {
                export.name.clone()
            } else {
                export.external_name.clone()
            };
            if self.machine == machine::I386
                && symbol.starts_with('_')
                && (self.options.mingw || !symbol.contains('@'))
            {
                export.export_name = symbol[1..].to_string();
                continue;
            }
            export.export_name = symbol;
        }
        let priority = |source: u8| match source {
            export_source::DIRECTIVES => 1,
            export_source::EXPORT_ALL => 2,
            _ => 3,
        };
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        let mut kept: Vec<CoffExport> = Vec::new();
        for export in self.exports.clone() {
            match seen.get(&export.export_name).copied() {
                None => {
                    seen.insert(export.export_name.clone(), kept.len());
                    kept.push(export);
                }
                Some(position) => {
                    let existing = kept[position].clone();
                    let same = existing.name == export.name
                        && existing.external_name == export.external_name
                        && existing.ordinal == export.ordinal
                        && existing.no_name == export.no_name
                        && existing.data == export.data
                        && existing.private == export.private;
                    if same || existing.name != export.name {
                        continue;
                    }
                    if priority(existing.source) < priority(export.source) {
                        kept[position] = export;
                        continue;
                    }
                    if priority(existing.source) > priority(export.source) {
                        continue;
                    }
                    self.warn(
                        caution("EI256", "A symbol is exported more than once.")
                            .with_context(format!("Symbol: {}", export.name)),
                    );
                }
            }
        }
        kept.sort_by(|left, right| left.export_name.cmp(&right.export_name));
        self.exports = kept;
    }

    fn assign_export_ordinals(&mut self) {
        let mut maximum = self
            .exports
            .iter()
            .map(|export| u32::from(export.ordinal))
            .max()
            .unwrap_or(0);
        for export in self.exports.iter_mut() {
            if export.ordinal == 0 {
                maximum += 1;
                export.ordinal = maximum.min(u32::from(u16::MAX)) as u16;
            }
        }
        if maximum > u32::from(u16::MAX) {
            self.error(
                problem("EI255", "Two exports use the same ordinal.")
                    .with_context(format!("Count: {maximum}")),
            );
        }
    }
}

pub fn is_coff_decorated(symbol: &str, mingw: bool) -> bool {
    symbol.starts_with('@')
        || symbol.contains("@@")
        || symbol.starts_with('?')
        || (!mingw && symbol.contains('@'))
}

pub fn link_coff(arguments: &[String], source: &dyn Fn(&str) -> Option<Vec<u8>>) -> LinkOutcome {
    let options = match parse_coff_command_line(arguments, source) {
        Ok(options) => options,
        Err(diagnostic) => {
            return LinkOutcome {
                outputs: Vec::new(),
                diagnostics: vec![diagnostic],
                messages: Vec::new(),
            }
        }
    };
    let mut context = CoffContext::new(options, source);
    let bytes = context.run();
    let mut outcome = LinkOutcome {
        outputs: Vec::new(),
        diagnostics: context.diagnostics,
        messages: context.messages,
    };
    if let Some(bytes) = bytes {
        outcome
            .outputs
            .push((context.options.output.clone(), bytes));
    }
    outcome
}

impl<'s> CoffContext<'s> {
    fn new_chunk(
        &mut self,
        kind: CoffChunkKind,
        size: u64,
        alignment: u64,
        characteristics: u32,
    ) -> usize {
        let index = self.chunks.len();
        self.chunks
            .push(CoffChunk::synthetic(kind, size, alignment, characteristics));
        index
    }

    pub fn chunk_rva(&self, chunk: usize) -> u64 {
        let target = self.chunks[chunk].replacement;
        let target = if target == usize::MAX { chunk } else { target };
        self.chunks[target].rva
    }

    fn chunk_output(&self, chunk: usize) -> Option<usize> {
        let target = self.chunks[chunk].replacement;
        let target = if target == usize::MAX { chunk } else { target };
        let data = &self.chunks[target];
        match data.merged_into {
            Some(merge) => self.chunks[merge].output,
            None => data.output,
        }
    }

    fn symbol_chunk(&self, id: SymbolIndex) -> Option<usize> {
        match &self.symbols[id].kind {
            CoffSymbolKind::Regular { chunk, .. } => *chunk,
            CoffSymbolKind::Common { chunk, .. } => Some(*chunk),
            CoffSymbolKind::Synthetic { chunk, .. } => *chunk,
            CoffSymbolKind::ImportData { import } => self.imports[*import].location,
            CoffSymbolKind::ImportThunk { chunk, .. } => Some(*chunk),
            CoffSymbolKind::LocalImport { chunk, .. } => Some(*chunk),
            _ => None,
        }
    }

    pub fn symbol_rva(&self, id: SymbolIndex) -> u64 {
        match &self.symbols[id].kind {
            CoffSymbolKind::Regular {
                chunk: Some(chunk),
                value,
                ..
            } => self.chunk_rva(*chunk) + u64::from(*value),
            CoffSymbolKind::Common { chunk, .. } => self.chunk_rva(*chunk),
            CoffSymbolKind::Absolute { address } => address.wrapping_sub(self.image_base),
            CoffSymbolKind::Synthetic {
                chunk: Some(chunk),
                offset,
            } => self.chunk_rva(*chunk) + u64::from(*offset),
            CoffSymbolKind::ImportData { import } => self.imports[*import]
                .location
                .map(|chunk| self.chunk_rva(chunk))
                .unwrap_or(0),
            CoffSymbolKind::ImportThunk { chunk, .. }
            | CoffSymbolKind::LocalImport { chunk, .. } => self.chunk_rva(*chunk),
            _ => 0,
        }
    }

    fn create_import_tables(&mut self) -> Vec<SymbolIndex> {
        let mut symbols = Vec::new();
        for import in 0..self.imports.len() {
            if !self.imports[import].live {
                continue;
            }
            let library = self.imports[import].library.to_ascii_lowercase();
            let next = self.library_order.len();
            self.library_order.entry(library).or_insert(next);
            if let Some(symbol) = self.imports[import].import_symbol {
                symbols.push(symbol);
            }
        }
        symbols
    }

    fn create_import_chunks(&mut self, symbols: &[SymbolIndex]) -> [Vec<usize>; 5] {
        use crate::compilers::pe::section_characteristic as character;
        let read_data = character::INITIALIZED_DATA | character::READ;
        let word = self.word_size();
        let mut bins: BTreeMap<usize, Vec<SymbolIndex>> = BTreeMap::new();
        for symbol in symbols {
            let CoffSymbolKind::ImportData { import } = self.symbols[*symbol].kind else {
                continue;
            };
            let library = self.imports[import].library.to_ascii_lowercase();
            let order = self.library_order.get(&library).copied().unwrap_or(0);
            bins.entry(order).or_default().push(*symbol);
        }
        let mut directories = Vec::new();
        let mut lookups = Vec::new();
        let mut addresses = Vec::new();
        let mut hints = Vec::new();
        let mut names = Vec::new();
        for (_, mut list) in bins {
            list.sort_by(|left, right| {
                let base = |id: &SymbolIndex| {
                    let name = &self.symbols[*id].name;
                    name.strip_prefix("__imp_").unwrap_or(name).to_string()
                };
                base(left).cmp(&base(right))
            });
            let base = lookups.len();
            for symbol in &list {
                let CoffSymbolKind::ImportData { import } = self.symbols[*symbol].kind else {
                    continue;
                };
                let ordinal = self.imports[import].object.ordinal_or_hint;
                let external = self.imports[import].external_name.clone();
                let (lookup, address) = if external.is_empty() {
                    (
                        self.new_chunk(
                            CoffChunkKind::OrdinalOnly { ordinal },
                            word,
                            word,
                            read_data,
                        ),
                        self.new_chunk(
                            CoffChunkKind::OrdinalOnly { ordinal },
                            word,
                            word,
                            read_data,
                        ),
                    )
                } else {
                    let size = ((external.len() + 3) as u64).div_ceil(2) * 2;
                    let hint = self.new_chunk(
                        CoffChunkKind::HintName {
                            name: external,
                            hint: ordinal,
                        },
                        size,
                        1,
                        read_data,
                    );
                    hints.push(hint);
                    (
                        self.new_chunk(
                            CoffChunkKind::Lookup { target: hint },
                            word,
                            word,
                            read_data,
                        ),
                        self.new_chunk(
                            CoffChunkKind::Lookup { target: hint },
                            word,
                            word,
                            read_data,
                        ),
                    )
                };
                lookups.push(lookup);
                addresses.push(address);
            }
            lookups.push(self.new_chunk(CoffChunkKind::Null, word, word, read_data));
            addresses.push(self.new_chunk(CoffChunkKind::Null, word, word, read_data));
            for (position, symbol) in list.iter().enumerate() {
                if let CoffSymbolKind::ImportData { import } = self.symbols[*symbol].kind {
                    self.imports[import].location = Some(addresses[base + position]);
                }
            }
            let library = match self.symbols[list[0]].kind {
                CoffSymbolKind::ImportData { import } => self.imports[import].library.clone(),
                _ => String::new(),
            };
            let name_chunk = self.new_chunk(
                CoffChunkKind::Text {
                    text: library.clone(),
                },
                library.len() as u64 + 1,
                1,
                read_data,
            );
            names.push(name_chunk);
            let directory = self.new_chunk(
                CoffChunkKind::ImportDirectory {
                    name: name_chunk,
                    lookup: lookups[base],
                    address: addresses[base],
                },
                20,
                4,
                read_data,
            );
            directories.push(directory);
        }
        directories.push(self.new_chunk(CoffChunkKind::Null, 20, 4, read_data));
        [directories, lookups, addresses, hints, names]
    }

    fn output_named(&self, name: &str) -> Option<usize> {
        self.outputs.iter().position(|output| output.name == name)
    }

    fn create_output(
        &mut self,
        name: &str,
        characteristics: u32,
        keys: &mut BTreeMap<(String, u32), usize>,
    ) -> usize {
        if let Some(found) = keys.get(&(name.to_string(), characteristics)) {
            return *found;
        }
        let index = self.outputs.len();
        self.outputs.push(CoffOutputSection {
            name: name.to_string(),
            characteristics,
            ..CoffOutputSection::default()
        });
        keys.insert((name.to_string(), characteristics), index);
        index
    }

    fn write_image(&mut self) -> Option<Vec<u8>> {
        use crate::compilers::pe::section_characteristic as character;
        let data = character::INITIALIZED_DATA;
        let bss = character::UNINITIALIZED_DATA;
        let code = character::CODE;
        let read = character::READ;
        let write = character::WRITE;
        let execute = character::EXECUTE;
        let discardable = character::DISCARDABLE;
        let import_symbols = self.create_import_tables();
        let mut keys: BTreeMap<(String, u32), usize> = BTreeMap::new();
        let text = self.create_output(".text", code | read | execute, &mut keys);
        let _ = self.create_output(".bss", bss | read | write, &mut keys);
        let rdata = self.create_output(".rdata", data | read, &mut keys);
        let _ = self.create_output(".buildid", data | read, &mut keys);
        let _ = self.create_output(".cvinfo", data | read, &mut keys);
        let data_section = self.create_output(".data", data | read | write, &mut keys);
        let pdata = self.create_output(".pdata", data | read, &mut keys);
        let _ = self.create_output(".idata", data | read, &mut keys);
        let edata = self.create_output(".edata", data | read, &mut keys);
        let _ = self.create_output(".didat", data | read, &mut keys);
        let rsrc = self.create_output(".rsrc", data | read, &mut keys);
        let reloc = self.create_output(".reloc", data | discardable | read, &mut keys);
        let _ = self.create_output(".ctors", data | read | write, &mut keys);
        let _ = self.create_output(".dtors", data | read | write, &mut keys);
        let _ = data_section;
        let mut partial: BTreeMap<(String, u32), Vec<usize>> = BTreeMap::new();
        for file in self.object_order.clone() {
            for chunk in self.objects[file].chunks.clone() {
                let entry = &self.chunks[chunk];
                if !entry.live {
                    continue;
                }
                let name = entry.name.clone();
                if name.starts_with(".tls") {
                    self.thread_local_alignment = self.thread_local_alignment.max(entry.alignment);
                }
                partial
                    .entry((name, entry.output_characteristics()))
                    .or_default()
                    .push(chunk);
            }
        }
        for prefix in [".rsrc", ".edata"] {
            self.fix_partial_characteristics(&mut partial, prefix, data | read);
        }
        self.fix_partial_characteristics(&mut partial, ".idata", data | read);
        let mut has_import_data = false;
        let keys_with_idata: Vec<(String, u32)> = partial
            .keys()
            .filter(|(name, _)| name.starts_with(".idata"))
            .cloned()
            .collect();
        for key in keys_with_idata {
            let mut list = partial.remove(&key).unwrap_or_default();
            if !list.is_empty() {
                has_import_data = true;
            }
            list.sort_by(|left, right| {
                let label = |chunk: &usize| -> String {
                    match self.chunks[*chunk].file {
                        Some(file) => {
                            format!("{}/{}", self.objects[file].parent, self.objects[file].name)
                        }
                        None => String::new(),
                    }
                };
                let left_section = self.chunks[*left].file.is_some();
                let right_section = self.chunks[*right].file.is_some();
                if !left_section || !right_section {
                    return right_section.cmp(&left_section).reverse();
                }
                label(left).cmp(&label(right))
            });
            partial.insert(key, list);
        }
        if !import_symbols.is_empty() {
            has_import_data = true;
        }
        if has_import_data {
            let [directories, lookups, addresses, hints, names] =
                self.create_import_chunks(&import_symbols);
            let mut add = |name: &str, list: Vec<usize>| {
                partial
                    .entry((name.to_string(), data | read))
                    .or_default()
                    .extend(list);
            };
            add(".idata$2", directories);
            add(".idata$4", lookups);
            add(".idata$5", addresses);
            if !hints.is_empty() {
                add(".idata$6", hints);
            }
            add(".idata$7", names);
            if let Some(list) = partial.get(&(".idata$2".to_string(), data | read)) {
                if let Some(first) = list.first() {
                    let size = list.iter().map(|chunk| self.chunks[*chunk].size).sum();
                    self.import_table = Some((*first, size));
                }
            }
            if let Some(list) = partial.get(&(".idata$5".to_string(), data | read)) {
                if let Some(first) = list.first() {
                    let size = list.iter().map(|chunk| self.chunks[*chunk].size).sum();
                    self.import_address_table = Some((*first, size));
                }
            }
        }
        for ((name, characteristics), list) in partial {
            let base = name.split('$').next().unwrap_or("").to_string();
            let base = if self.options.mingw {
                match base[1.min(base.len())..].find('.') {
                    Some(position) => base[..position + 1].to_string(),
                    None => base,
                }
            } else {
                base
            };
            let mut characteristics = characteristics;
            let mut list = list;
            if base == ".CRT" {
                characteristics = data | read;
                list.sort_by(|left, right| {
                    let same = self.chunks[*left].file == self.chunks[*right].file;
                    if same {
                        self.chunks[*left].section.cmp(&self.chunks[*right].section)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
            }
            let output = self.create_output(&base, characteristics, &mut keys);
            self.outputs[output].chunks.extend(list);
        }
        let order_of = |output: &CoffOutputSection| -> u8 {
            if output.characteristics & discardable != 0 {
                if output.name.starts_with(".debug_") {
                    3
                } else {
                    2
                }
            } else if output.name == ".rsrc" {
                1
            } else {
                0
            }
        };
        let mut indexed: Vec<(usize, CoffOutputSection)> =
            self.outputs.drain(..).enumerate().collect();
        indexed.sort_by_key(|(_, output)| order_of(output));
        let mut remap: BTreeMap<usize, usize> = BTreeMap::new();
        for (position, (old, output)) in indexed.into_iter().enumerate() {
            remap.insert(old, position);
            self.outputs.push(output);
        }
        let text = remap[&text];
        let rdata = remap[&rdata];
        let pdata = remap[&pdata];
        let edata = remap[&edata];
        let rsrc_name = self.outputs[remap[&rsrc]].name.clone();
        let reloc = remap[&reloc];
        let data_output = remap[&data_section];
        for import in 0..self.imports.len() {
            if !self.imports[import].live {
                continue;
            }
            if let Some(chunk) = self.imports[import].thunk_chunk {
                if self.imports[import].thunk_symbol.is_some() && self.chunks[chunk].live {
                    self.outputs[text].chunks.push(chunk);
                }
            }
        }
        let merge_chunks: Vec<usize> = self.merge_chunks.iter().flatten().copied().collect();
        for merge in merge_chunks {
            self.finalize_merge_chunk(merge);
            self.outputs[rdata].chunks.push(merge);
        }
        for chunk in self.local_import_chunks.clone() {
            self.outputs[rdata].chunks.push(chunk);
        }
        if self.options.repro || self.options.build_identifier {
            let records: Vec<(u32, usize)> = if self.options.build_identifier {
                let record = self.new_chunk(CoffChunkKind::CodeViewRecord, 25, 4, data | read);
                vec![(2, record)]
            } else {
                Vec::new()
            };
            let size = (records.len() as u64 + u64::from(self.options.repro)) * 28;
            let directory = self.new_chunk(
                CoffChunkKind::DebugDirectory {
                    records: records.clone(),
                    repro: self.options.repro,
                },
                size,
                4,
                data | read,
            );
            let target = self.output_named(".rdata").unwrap_or(rdata);
            self.outputs[target].chunks.push(directory);
            for (_, record) in records {
                self.outputs[target].chunks.push(record);
            }
            self.debug_directory = Some(directory);
        }
        if self.safe_seh {
            self.create_safe_exception_handler_table(rdata);
        }
        self.create_guard_tables(rdata);
        if !self.outputs[edata].chunks.is_empty() {
            let first = self.outputs[edata].chunks[0];
            let last = *self.outputs[edata].chunks.last().unwrap_or(&first);
            self.export_range = Some((first, last));
            if self.had_explicit_exports {
                self.warn(caution(
                    "EI257",
                    "Literal .edata sections override the exports.",
                ));
            }
        } else if !self.exports.is_empty() {
            let chunks = self.create_export_chunks();
            let first = chunks[0];
            let last = *chunks.last().unwrap_or(&first);
            self.outputs[edata].chunks.extend(chunks);
            self.export_range = Some((first, last));
        }
        if let (Some(first), Some(last)) = (
            self.outputs[pdata].chunks.first().copied(),
            self.outputs[pdata].chunks.last().copied(),
        ) {
            self.exception_range = Some((first, last));
        }
        for (from, to) in self.merges.clone() {
            if from != ".bss" {
                self.merge_output(&from, &to);
            }
        }
        if let Some(to) = self.merges.get(".bss").cloned() {
            let destination = self.merge_destination(".bss", &to);
            let shared = self
                .section_permissions
                .get(&destination)
                .is_some_and(|permissions| permissions & character::SHARED != 0);
            if !shared {
                self.merge_output(".bss", &destination);
            }
        }
        let reloc_name = self.outputs[reloc].name.clone();
        self.outputs.retain(|output| {
            output.name == reloc_name && output.characteristics == (data | discardable | read)
                || !output.chunks.is_empty()
        });
        let _ = (rsrc_name, data_output);
        self.assign_addresses();
        self.outputs.retain(|output| output.virtual_size != 0);
        for (position, output) in self.outputs.iter_mut().enumerate() {
            output.index = position + 1;
        }
        for position in 0..self.outputs.len() {
            for chunk in self.outputs[position].chunks.clone() {
                self.chunks[chunk].output = Some(position);
            }
        }
        for (name, permissions) in self.section_permissions.clone() {
            for output in self.outputs.iter_mut() {
                if output.name == name {
                    output.characteristics =
                        (output.characteristics & !character::PERMISSIONS) | permissions;
                }
            }
        }
        if self.file_size > u64::from(u32::MAX) {
            self.error(
                problem(
                    "EI212",
                    "The linker does not support what this input asks for yet.",
                )
                .with_context("Feature: an image larger than 4 GiB".to_string()),
            );
            return None;
        }
        let mut bytes = self.write_headers();
        self.write_sections(&mut bytes);
        if self.failed() {
            return None;
        }
        self.sort_exception_table(&mut bytes);
        if self.thread_local_alignment != 0 {
            self.fix_thread_local_alignment(&mut bytes);
        }
        self.write_time_stamp(&mut bytes);
        if self.write_check_sum {
            let offset = self.coff_header_offset() + 20 + 64;
            crate::compilers::pe::write_u32(&mut bytes, offset, 0);
            let sum = crate::compilers::pe::image_check_sum(&bytes);
            crate::compilers::pe::write_u32(&mut bytes, offset, sum);
        }
        Some(bytes)
    }

    fn fix_partial_characteristics(
        &mut self,
        partial: &mut BTreeMap<(String, u32), Vec<usize>>,
        prefix: &str,
        wanted: u32,
    ) {
        let keys: Vec<(String, u32)> = partial.keys().cloned().collect();
        for (name, characteristics) in keys {
            let Some(rest) = name.strip_prefix(prefix) else {
                continue;
            };
            if !rest.is_empty() && !rest.starts_with('$') {
                continue;
            }
            if characteristics == wanted {
                continue;
            }
            let list = partial
                .remove(&(name.clone(), characteristics))
                .unwrap_or_default();
            partial.entry((name, wanted)).or_default().extend(list);
        }
    }

    fn merge_destination(&self, from: &str, to: &str) -> String {
        let mut seen = BTreeSet::new();
        let mut current = to.to_string();
        while seen.insert(current.clone()) {
            match self.merges.get(&current) {
                Some(next) => current = next.clone(),
                None => break,
            }
        }
        let _ = from;
        current
    }

    fn merge_output(&mut self, from: &str, to: &str) {
        if from == to {
            return;
        }
        let destination = self.merge_destination(from, to);
        let Some(source) = self.output_named(from) else {
            return;
        };
        let Some(target) = self.output_named(&destination) else {
            self.outputs[source].name = destination;
            return;
        };
        if source == target {
            return;
        }
        let moved = std::mem::take(&mut self.outputs[source].chunks);
        self.outputs[target].chunks.extend(moved);
        use crate::compilers::pe::section_characteristic as character;
        if self.outputs[source].characteristics & character::CODE != 0 {
            let output = &mut self.outputs[target];
            output.characteristics |= character::CODE;
            output.characteristics &=
                !(character::INITIALIZED_DATA | character::UNINITIALIZED_DATA);
        }
    }

    fn finalize_merge_chunk(&mut self, merge: usize) {
        let members = self.merge_members.get(&merge).cloned().unwrap_or_default();
        let alignment = self.chunks[merge].alignment;
        let mut strings: Vec<Vec<u8>> = Vec::new();
        for member in &members {
            if self.chunks[*member].live {
                strings.push(self.section_data(*member).to_vec());
            }
        }
        let (offsets, size) = tail_merged_layout(&strings, alignment);
        self.chunks[merge].size = size;
        self.merge_offsets.insert(merge, offsets);
    }

    fn assign_addresses(&mut self) {
        let header_size = self.coff_header_offset()
            + 20
            + if self.is_64() { 112 } else { 96 }
            + 16 * 8
            + 40 * self.outputs.len();
        let file_alignment = self.options.file_alignment;
        let section_alignment = self.options.section_alignment.unwrap_or(4096);
        self.size_of_headers = align_to(header_size as u64, file_alignment);
        let mut file_size = self.size_of_headers;
        let mut rva = align_to(self.size_of_headers, section_alignment);
        let reloc_name = ".reloc";
        let padding = self.options.function_pad_minimum.unwrap_or(0);
        for position in 0..self.outputs.len() {
            if self.outputs[position].name == reloc_name
                && self.outputs[position].characteristics
                    == (crate::compilers::pe::section_characteristic::INITIALIZED_DATA
                        | crate::compilers::pe::section_characteristic::DISCARDABLE
                        | crate::compilers::pe::section_characteristic::READ)
            {
                self.outputs[position].chunks.clear();
                let blocks = self.base_relocation_blocks();
                self.outputs[position].chunks.extend(blocks);
            }
            let mut raw_size = 0u64;
            let mut virtual_size = 0u64;
            self.outputs[position].virtual_address = rva;
            let is_code = self.outputs[position].characteristics
                & crate::compilers::pe::section_characteristic::CODE
                != 0;
            for chunk in self.outputs[position].chunks.clone() {
                let _ = (is_code, padding);
                virtual_size = align_to(virtual_size, self.chunks[chunk].alignment);
                self.chunks[chunk].rva = rva + virtual_size;
                virtual_size += self.chunks[chunk].size;
                if self.chunks[chunk].has_data {
                    raw_size = align_to(virtual_size, file_alignment);
                }
            }
            self.outputs[position].virtual_size = virtual_size;
            self.outputs[position].raw_size = raw_size;
            self.outputs[position].file_offset = if raw_size != 0 { file_size } else { 0 };
            rva += align_to(virtual_size, section_alignment);
            file_size += align_to(raw_size, file_alignment);
        }
        self.size_of_image = align_to(rva, section_alignment);
        self.file_size = file_size;
        for (merge, offsets) in self.merge_offsets.clone() {
            let base = self.chunks[merge].rva;
            for member in self.merge_members.get(&merge).cloned().unwrap_or_default() {
                if !self.chunks[member].live {
                    continue;
                }
                let key = self.section_data(member).to_vec();
                let offset = offsets.get(&key).copied().unwrap_or(0);
                self.chunks[member].rva = base + offset;
            }
        }
    }

    fn coff_header_offset(&self) -> usize {
        crate::compilers::pe::DOS_HEADER_SIZE + crate::compilers::pe::DOS_PROGRAM.len() + 4
    }

    fn base_relocation_blocks(&mut self) -> Vec<usize> {
        use crate::compilers::pe::{
            base_relocation, machine, relocation_amd64, relocation_i386,
            section_characteristic as character,
        };
        if !self.relocatable {
            return Vec::new();
        }
        let default_kind = if self.is_64() {
            base_relocation::DIRECT_64
        } else {
            base_relocation::HIGH_LOW
        };
        let mut blocks = Vec::new();
        for position in 0..self.outputs.len() {
            if self.outputs[position].characteristics & character::DISCARDABLE != 0 {
                continue;
            }
            let mut entries: Vec<(u32, u8)> = Vec::new();
            for chunk in self.outputs[position].chunks.clone() {
                match self.chunks[chunk].kind.clone() {
                    CoffChunkKind::Section => {
                        let Some(file) = self.chunks[chunk].file else {
                            continue;
                        };
                        for relocation in self.section_relocations(chunk).to_vec() {
                            let kind = match self.machine {
                                machine::AMD64 => match relocation.kind {
                                    relocation_amd64::ADDRESS_64 => base_relocation::DIRECT_64,
                                    relocation_amd64::ADDRESS_32 => base_relocation::HIGH_LOW,
                                    _ => continue,
                                },
                                machine::I386 => match relocation.kind {
                                    relocation_i386::DIRECT_32 => base_relocation::HIGH_LOW,
                                    _ => continue,
                                },
                                _ => continue,
                            };
                            let Some(target) = self.objects[file]
                                .symbols
                                .get(relocation.symbol_index as usize)
                                .copied()
                                .flatten()
                            else {
                                continue;
                            };
                            if !self.symbols[target].is_defined()
                                || matches!(
                                    self.symbols[target].kind,
                                    CoffSymbolKind::Absolute { .. }
                                )
                            {
                                continue;
                            }
                            entries.push((
                                (self.chunks[chunk].rva + u64::from(relocation.virtual_address))
                                    as u32,
                                kind,
                            ));
                        }
                    }
                    CoffChunkKind::ImportThunk { .. } if self.machine == machine::I386 => {
                        entries.push(((self.chunks[chunk].rva + 2) as u32, default_kind));
                    }
                    CoffChunkKind::LocalImport { .. } => {
                        entries.push((self.chunks[chunk].rva as u32, default_kind));
                    }
                    _ => {}
                }
            }
            if entries.is_empty() {
                continue;
            }
            let mask = !0xfffu32;
            let mut page = entries[0].0 & mask;
            entries.sort_by_key(|entry| entry.0);
            let mut start = 0usize;
            let mut index = 1usize;
            while index < entries.len() {
                let current = entries[index].0 & mask;
                if current != page {
                    blocks.push(self.base_relocation_chunk(page, &entries[start..index]));
                    start = index;
                    page = current;
                }
                index += 1;
            }
            if start != index {
                blocks.push(self.base_relocation_chunk(page, &entries[start..index]));
            }
        }
        blocks
    }

    fn base_relocation_chunk(&mut self, page: u32, entries: &[(u32, u8)]) -> usize {
        let size = align_to(entries.len() as u64 * 2 + 8, 4);
        let mut data = Vec::with_capacity(size as usize);
        crate::compilers::pe::put_u32(&mut data, page);
        crate::compilers::pe::put_u32(&mut data, size as u32);
        for (rva, kind) in entries {
            crate::compilers::pe::put_u16(
                &mut data,
                (u16::from(*kind) << 12) | (rva.wrapping_sub(page) as u16),
            );
        }
        data.resize(size as usize, 0);
        let characteristics = crate::compilers::pe::section_characteristic::INITIALIZED_DATA
            | crate::compilers::pe::section_characteristic::READ;
        self.new_chunk(
            CoffChunkKind::BaseRelocations { data },
            size,
            1,
            characteristics,
        )
    }

    fn write_headers(&mut self) -> Vec<u8> {
        use crate::compilers::pe::{
            data_directory, dll_characteristic as dll, file_characteristic as file, DataDirectory,
            ImageHeader,
        };
        let mut out = crate::compilers::pe::dos_stub();
        out.extend_from_slice(b"PE\0\0");
        let mut characteristics = file::EXECUTABLE_IMAGE;
        if self.options.large_address_aware.unwrap_or(self.is_64()) {
            characteristics |= file::LARGE_ADDRESS_AWARE;
        }
        if !self.is_64() {
            characteristics |= file::MACHINE_32_BIT;
        }
        if self.options.dll {
            characteristics |= file::DYNAMIC_LIBRARY;
        }
        if self.options.driver_unprocessor_only {
            characteristics |= file::UNIPROCESSOR_ONLY;
        }
        if !self.relocatable {
            characteristics |= file::RELOCATIONS_STRIPPED;
        }
        if self.options.swap_run_cd {
            characteristics |= file::REMOVABLE_RUN_FROM_SWAP;
        }
        if self.options.swap_run_net {
            characteristics |= file::NET_RUN_FROM_SWAP;
        }
        let mut dll_characteristics = 0u16;
        if self.options.application_container {
            dll_characteristics |= dll::APPLICATION_CONTAINER;
        }
        if self.options.driver_wdm {
            dll_characteristics |= dll::WDM_DRIVER;
        }
        if self.dynamic_base {
            dll_characteristics |= dll::DYNAMIC_BASE;
        }
        if self.is_64() && self.options.high_entropy.unwrap_or(true) {
            dll_characteristics |= dll::HIGH_ENTROPY_VIRTUAL_ADDRESS;
        }
        if !self.options.allow_bind {
            dll_characteristics |= dll::NO_BIND;
        }
        if self.options.nx_compatible {
            dll_characteristics |= dll::NX_COMPATIBLE;
        }
        if !self.options.allow_isolation {
            dll_characteristics |= dll::NO_ISOLATION;
        }
        if self.options.guard != guard_level::OFF {
            dll_characteristics |= dll::GUARD_CONTROL_FLOW;
        }
        if self.options.integrity_check {
            dll_characteristics |= dll::FORCE_INTEGRITY;
        }
        if self.set_no_seh
            || (self.machine == crate::compilers::pe::machine::I386 && self.options.no_seh)
        {
            dll_characteristics |= dll::NO_STRUCTURED_EXCEPTION_HANDLING;
        }
        if !self.options.dll && self.options.terminal_server_aware {
            dll_characteristics |= dll::TERMINAL_SERVER_AWARE;
        }
        let mut directories = vec![DataDirectory::default(); data_directory::COUNT];
        if let Some((first, last)) = self.export_range {
            let start = self.chunk_rva(first);
            directories[data_directory::EXPORT] = DataDirectory {
                address: start as u32,
                size: (self.chunk_rva(last) + self.chunks[last].size - start) as u32,
            };
        }
        if let Some((first, size)) = self.import_table {
            directories[data_directory::IMPORT] = DataDirectory {
                address: self.chunk_rva(first) as u32,
                size: size as u32,
            };
        }
        if let Some((first, size)) = self.import_address_table {
            directories[data_directory::IMPORT_ADDRESS_TABLE] = DataDirectory {
                address: self.chunk_rva(first) as u32,
                size: size as u32,
            };
        }
        if let Some(rsrc) = self.output_named(".rsrc") {
            if self.outputs[rsrc].virtual_size != 0 {
                directories[data_directory::RESOURCE] = DataDirectory {
                    address: self.outputs[rsrc].virtual_address as u32,
                    size: self.outputs[rsrc].virtual_size as u32,
                };
            }
        }
        if let Some((first, last)) = self.exception_range {
            let start = self.chunk_rva(first);
            directories[data_directory::EXCEPTION] = DataDirectory {
                address: start as u32,
                size: (self.chunk_rva(last) + self.chunks[last].size - start) as u32,
            };
        }
        if let Some(reloc) = self.outputs.iter().position(|output| {
            output.name == ".reloc"
                && output.characteristics
                    == (crate::compilers::pe::section_characteristic::INITIALIZED_DATA
                        | crate::compilers::pe::section_characteristic::DISCARDABLE
                        | crate::compilers::pe::section_characteristic::READ)
        }) {
            if self.outputs[reloc].virtual_size != 0 {
                directories[data_directory::BASE_RELOCATION] = DataDirectory {
                    address: self.outputs[reloc].virtual_address as u32,
                    size: self.outputs[reloc].virtual_size as u32,
                };
            }
        }
        if let Some(id) = self.find_underscore("_tls_used") {
            if self.symbols[id].is_defined() {
                directories[data_directory::THREAD_LOCAL_STORAGE] = DataDirectory {
                    address: self.symbol_rva(id) as u32,
                    size: if self.is_64() { 40 } else { 24 },
                };
            }
        }
        if let Some(directory) = self.debug_directory {
            directories[data_directory::DEBUG] = DataDirectory {
                address: self.chunk_rva(directory) as u32,
                size: self.chunks[directory].size as u32,
            };
        }
        if let Some(id) = self.load_config_symbol {
            directories[data_directory::LOAD_CONFIGURATION] = DataDirectory {
                address: self.symbol_rva(id) as u32,
                size: self.load_config_size,
            };
        }
        let text = self.output_named(".text");
        let (base_of_code, size_of_code) = match text {
            Some(text) if self.outputs[text].virtual_size != 0 => (
                self.outputs[text].virtual_address as u32,
                self.outputs[text].raw_size as u32,
            ),
            _ => (0, 0),
        };
        let initialized: u64 = self
            .outputs
            .iter()
            .filter(|output| {
                output.characteristics
                    & crate::compilers::pe::section_characteristic::INITIALIZED_DATA
                    != 0
            })
            .map(|output| output.raw_size)
            .sum();
        let entry = if self.options.no_entry {
            0
        } else {
            self.entry
                .and_then(|entry| self.defined_target(entry))
                .map(|entry| self.symbol_rva(entry) as u32)
                .unwrap_or(0)
        };
        let header = ImageHeader {
            machine: self.machine,
            time_stamp: 0,
            pointer_to_symbol_table: 0,
            number_of_symbols: 0,
            characteristics,
            plus: self.is_64(),
            major_linker_version: 14,
            minor_linker_version: 0,
            size_of_code,
            size_of_initialized_data: initialized as u32,
            size_of_uninitialized_data: 0,
            address_of_entry_point: entry,
            base_of_code,
            base_of_data: 0,
            image_base: self.image_base,
            section_alignment: self.options.section_alignment.unwrap_or(4096) as u32,
            file_alignment: self.options.file_alignment as u32,
            major_operating_system_version: self.os_version.0 as u16,
            minor_operating_system_version: self.os_version.1 as u16,
            major_image_version: self.options.image_version.0 as u16,
            minor_image_version: self.options.image_version.1 as u16,
            major_subsystem_version: self.subsystem_version.0 as u16,
            minor_subsystem_version: self.subsystem_version.1 as u16,
            win32_version_value: 0,
            size_of_image: self.size_of_image as u32,
            size_of_headers: self.size_of_headers as u32,
            check_sum: 0,
            subsystem: self.subsystem,
            dll_characteristics,
            size_of_stack_reserve: self.options.stack.0,
            size_of_stack_commit: self.options.stack.1,
            size_of_heap_reserve: self.options.heap.0,
            size_of_heap_commit: self.options.heap.1,
            loader_flags: 0,
            directories,
        };
        header.encode(self.outputs.len() as u16, &mut out);
        for output in &self.outputs {
            let mut name = [0u8; 8];
            let length = output.name.len().min(8);
            name[..length].copy_from_slice(&output.name.as_bytes()[..length]);
            let section = crate::compilers::pe::CoffSection {
                virtual_size: output.virtual_size as u32,
                virtual_address: output.virtual_address as u32,
                size_of_raw_data: output.raw_size as u32,
                pointer_to_raw_data: output.file_offset as u32,
                characteristics: output.characteristics,
                ..crate::compilers::pe::CoffSection::default()
            };
            crate::compilers::pe::encode_section_header(&section, name, &mut out);
        }
        out.resize(self.file_size as usize, 0);
        out
    }

    fn write_sections(&mut self, out: &mut [u8]) {
        use crate::compilers::pe::{machine, section_characteristic as character};
        for position in 0..self.outputs.len() {
            let output = self.outputs[position].clone();
            let start = output.file_offset as usize;
            if output.raw_size == 0 {
                continue;
            }
            if output.characteristics & character::CODE != 0
                && (self.machine == machine::AMD64 || self.machine == machine::I386)
            {
                let raw = output.raw_size as usize;
                let mut previous_end = 0usize;
                for chunk in &output.chunks {
                    let offset = (self.chunks[*chunk].rva - output.virtual_address) as usize;
                    if offset >= raw {
                        break;
                    }
                    out[start + previous_end..start + offset].fill(0xcc);
                    previous_end = (offset + self.chunks[*chunk].size as usize).min(raw);
                }
                out[start + previous_end..start + raw].fill(0xcc);
            }
            for chunk in output.chunks.clone() {
                if !self.chunks[chunk].has_data {
                    continue;
                }
                let offset = start + (self.chunks[chunk].rva - output.virtual_address) as usize;
                let bytes = self.chunk_contents(chunk);
                let end = (offset + bytes.len()).min(out.len());
                out[offset..end].copy_from_slice(&bytes[..end - offset]);
            }
        }
    }

    fn chunk_contents(&mut self, chunk: usize) -> Vec<u8> {
        use crate::compilers::pe::put_u32;
        let word = self.word_size();
        let size = self.chunks[chunk].size as usize;
        let mut out = vec![0u8; size];
        match self.chunks[chunk].kind.clone() {
            CoffChunkKind::Section => {
                let data = self.section_data(chunk).to_vec();
                let length = data.len().min(size);
                out[..length].copy_from_slice(&data[..length]);
                self.apply_coff_relocations(chunk, &mut out);
            }
            CoffChunkKind::Merge => {
                let offsets = self.merge_offsets.get(&chunk).cloned().unwrap_or_default();
                for (bytes, offset) in offsets {
                    let at = offset as usize;
                    let end = (at + bytes.len()).min(out.len());
                    out[at..end].copy_from_slice(&bytes[..end - at]);
                }
            }
            CoffChunkKind::ImportThunk { import } => {
                out[0] = 0xff;
                out[1] = 0x25;
                let target = self.imports[import]
                    .import_symbol
                    .map(|symbol| self.symbol_rva(symbol))
                    .unwrap_or(0);
                let value = if self.machine == crate::compilers::pe::machine::AMD64 {
                    target.wrapping_sub(self.chunks[chunk].rva).wrapping_sub(6) as u32
                } else {
                    target.wrapping_add(self.image_base) as u32
                };
                out[2..6].copy_from_slice(&value.to_le_bytes());
            }
            CoffChunkKind::LocalImport { symbol } => {
                let value = self.symbol_rva(symbol).wrapping_add(self.image_base);
                if word == 8 {
                    out[..8].copy_from_slice(&value.to_le_bytes());
                } else {
                    out[..4].copy_from_slice(&(value as u32).to_le_bytes());
                }
            }
            CoffChunkKind::HintName { name, hint } => {
                out[..2].copy_from_slice(&hint.to_le_bytes());
                out[2..2 + name.len()].copy_from_slice(name.as_bytes());
            }
            CoffChunkKind::Lookup { target } => {
                let value = self.chunk_rva(target);
                if word == 8 {
                    out[..8].copy_from_slice(&value.to_le_bytes());
                } else {
                    out[..4].copy_from_slice(&(value as u32).to_le_bytes());
                }
            }
            CoffChunkKind::OrdinalOnly { ordinal } => {
                if word == 8 {
                    out[..8].copy_from_slice(&((1u64 << 63) | u64::from(ordinal)).to_le_bytes());
                } else {
                    out[..4].copy_from_slice(&((1u32 << 31) | u32::from(ordinal)).to_le_bytes());
                }
            }
            CoffChunkKind::ImportDirectory {
                name,
                lookup,
                address,
            } => {
                let mut data = Vec::new();
                put_u32(&mut data, self.chunk_rva(lookup) as u32);
                put_u32(&mut data, 0);
                put_u32(&mut data, 0);
                put_u32(&mut data, self.chunk_rva(name) as u32);
                put_u32(&mut data, self.chunk_rva(address) as u32);
                out[..20].copy_from_slice(&data);
            }
            CoffChunkKind::Null => {}
            CoffChunkKind::Text { text } => {
                out[..text.len()].copy_from_slice(text.as_bytes());
            }
            CoffChunkKind::ExportDirectory {
                base,
                maximum,
                names,
                name,
                addresses,
                pointers,
                ordinals,
            } => {
                let mut data = Vec::new();
                put_u32(&mut data, 0);
                put_u32(&mut data, 0);
                put_u32(&mut data, 0);
                put_u32(&mut data, self.chunk_rva(name) as u32);
                put_u32(&mut data, base);
                put_u32(&mut data, maximum - base + 1);
                put_u32(&mut data, names);
                put_u32(&mut data, self.chunk_rva(addresses) as u32);
                put_u32(&mut data, self.chunk_rva(pointers) as u32);
                put_u32(&mut data, self.chunk_rva(ordinals) as u32);
                out[..40].copy_from_slice(&data);
            }
            CoffChunkKind::ExportAddresses { base } => {
                for export in self.exports.clone() {
                    let at = (u32::from(export.ordinal) - base) as usize * 4;
                    let value = match export.forward_chunk {
                        Some(forward) => self.chunk_rva(forward),
                        None => export
                            .symbol
                            .and_then(|symbol| self.defined_target(symbol))
                            .map(|symbol| self.symbol_rva(symbol))
                            .unwrap_or(0),
                    };
                    if at + 4 <= out.len() {
                        out[at..at + 4].copy_from_slice(&(value as u32).to_le_bytes());
                    }
                }
            }
            CoffChunkKind::ExportNamePointers { names } => {
                for (position, name) in names.iter().enumerate() {
                    let value = self.chunk_rva(*name) as u32;
                    out[position * 4..position * 4 + 4].copy_from_slice(&value.to_le_bytes());
                }
            }
            CoffChunkKind::ExportOrdinals { base } => {
                let mut at = 0usize;
                for export in self.exports.clone() {
                    if export.no_name {
                        continue;
                    }
                    let value = (u32::from(export.ordinal) - base) as u16;
                    out[at..at + 2].copy_from_slice(&value.to_le_bytes());
                    at += 2;
                }
            }
            CoffChunkKind::BaseRelocations { data } => {
                let length = data.len().min(size);
                out[..length].copy_from_slice(&data[..length]);
            }
            CoffChunkKind::RvaTable { entries, flags } => {
                let mut values: Vec<(u32, u8)> = entries
                    .iter()
                    .enumerate()
                    .map(|(position, (target, offset))| {
                        let flag = flags
                            .as_ref()
                            .and_then(|flags| flags.get(position).copied())
                            .unwrap_or(0);
                        ((self.chunk_rva(*target) + u64::from(*offset)) as u32, flag)
                    })
                    .collect();
                values.sort();
                let stride = if flags.is_some() { 5 } else { 4 };
                for (position, (value, flag)) in values.iter().enumerate() {
                    out[position * stride..position * stride + 4]
                        .copy_from_slice(&value.to_le_bytes());
                    if stride == 5 {
                        out[position * stride + 4] = *flag;
                    }
                }
            }
            CoffChunkKind::DebugDirectory { records, repro } => {
                let mut data = Vec::new();
                let entry = |data: &mut Vec<u8>, kind: u32, size: u32, rva: u32, offset: u32| {
                    put_u32(data, 0);
                    put_u32(data, 0);
                    put_u32(data, 0);
                    put_u32(data, kind);
                    put_u32(data, size);
                    put_u32(data, rva);
                    put_u32(data, offset);
                };
                for (kind, record) in records {
                    let rva = self.chunk_rva(record);
                    let offset = self
                        .chunk_output(record)
                        .map(|output| {
                            self.outputs[output].file_offset
                                + (rva - self.outputs[output].virtual_address)
                        })
                        .unwrap_or(0);
                    entry(
                        &mut data,
                        kind,
                        self.chunks[record].size as u32,
                        rva as u32,
                        offset as u32,
                    );
                }
                if repro {
                    entry(&mut data, 16, 0, 0, 0);
                }
                let length = data.len().min(size);
                out[..length].copy_from_slice(&data[..length]);
            }
            CoffChunkKind::CodeViewRecord | CoffChunkKind::Common { .. } => {}
        }
        out
    }

    fn apply_coff_relocations(&mut self, chunk: usize, out: &mut [u8]) {
        use crate::compilers::pe::{machine, relocation_amd64 as amd64, relocation_i386 as i386};
        let Some(file) = self.chunks[chunk].file else {
            return;
        };
        let relocations = self.section_relocations(chunk).to_vec();
        let size = self.chunks[chunk].size;
        let section_count = self.outputs.len() as u16;
        for relocation in relocations {
            if u64::from(relocation.virtual_address) >= size {
                let label = self.file_label(Some(file));
                self.error(
                    problem(
                        "EI248",
                        "A relocation points outside its section or at a discarded section.",
                    )
                    .with_context(format!("File: {label}"))
                    .with_context(format!("Offset: {}", relocation.virtual_address)),
                );
                continue;
            }
            let symbol = self.objects[file]
                .symbols
                .get(relocation.symbol_index as usize)
                .copied()
                .flatten()
                .and_then(|symbol| self.defined_target(symbol));
            let target_chunk = symbol.and_then(|symbol| self.symbol_chunk(symbol));
            let output = target_chunk.and_then(|chunk| self.chunk_output(chunk));
            let absolute = symbol.is_some_and(|symbol| {
                matches!(
                    self.symbols[symbol].kind,
                    CoffSymbolKind::Absolute { .. } | CoffSymbolKind::Synthetic { .. }
                )
            });
            let Some(symbol) = symbol.filter(|_| output.is_some() || absolute) else {
                let name = self.objects[file]
                    .object
                    .symbol_at(relocation.symbol_index)
                    .map(|symbol| symbol.name.clone())
                    .unwrap_or_default();
                if self.chunks[chunk].name.starts_with(".debug") {
                    continue;
                }
                let label = self.file_label(Some(file));
                self.error(
                    problem(
                        "EI248",
                        "A relocation points outside its section or at a discarded section.",
                    )
                    .with_context(format!("Symbol: {name}"))
                    .with_context(format!("File: {label}")),
                );
                continue;
            };
            let s = self.symbol_rva(symbol);
            let p = self.chunks[chunk].rva + u64::from(relocation.virtual_address);
            let at = relocation.virtual_address as usize;
            let add32 = |out: &mut [u8], value: u64| {
                let current = u32::from_le_bytes(out[at..at + 4].try_into().unwrap_or([0; 4]));
                out[at..at + 4].copy_from_slice(&current.wrapping_add(value as u32).to_le_bytes());
            };
            let add16 = |out: &mut [u8], value: u16| {
                let current = u16::from_le_bytes(out[at..at + 2].try_into().unwrap_or([0; 2]));
                out[at..at + 2].copy_from_slice(&current.wrapping_add(value).to_le_bytes());
            };
            let section_index = output
                .map(|output| self.outputs[output].index as u16)
                .unwrap_or(section_count + 1);
            let section_relative = |context: &mut CoffContext| -> Option<u64> {
                match output {
                    Some(output) => {
                        let relative = s.wrapping_sub(context.outputs[output].virtual_address);
                        if relative > u64::from(u32::MAX) {
                            None
                        } else {
                            Some(relative)
                        }
                    }
                    None => None,
                }
            };
            match self.machine {
                machine::AMD64 => match relocation.kind {
                    amd64::ADDRESS_32 => add32(out, s.wrapping_add(self.image_base)),
                    amd64::ADDRESS_64 => {
                        let current =
                            u64::from_le_bytes(out[at..at + 8].try_into().unwrap_or([0; 8]));
                        out[at..at + 8].copy_from_slice(
                            &current
                                .wrapping_add(s.wrapping_add(self.image_base))
                                .to_le_bytes(),
                        );
                    }
                    amd64::ADDRESS_32_IMAGE_RELATIVE => add32(out, s),
                    amd64::RELATIVE_32..=amd64::RELATIVE_32_5 => {
                        let extra = u64::from(relocation.kind - amd64::RELATIVE_32) + 4;
                        add32(out, s.wrapping_sub(p).wrapping_sub(extra));
                    }
                    amd64::SECTION => add16(out, section_index),
                    amd64::SECTION_RELATIVE => match section_relative(self) {
                        Some(value) => add32(out, value),
                        None => self.section_relative_failure(chunk),
                    },
                    other => self.unsupported_coff_relocation(file, other),
                },
                machine::I386 => match relocation.kind {
                    i386::ABSOLUTE => {}
                    i386::DIRECT_32 => add32(out, s.wrapping_add(self.image_base)),
                    i386::DIRECT_32_IMAGE_RELATIVE => add32(out, s),
                    i386::RELATIVE_32 => add32(out, s.wrapping_sub(p).wrapping_sub(4)),
                    i386::SECTION => add16(out, section_index),
                    i386::SECTION_RELATIVE => match section_relative(self) {
                        Some(value) => add32(out, value),
                        None => self.section_relative_failure(chunk),
                    },
                    other => self.unsupported_coff_relocation(file, other),
                },
                _ => self.unsupported_coff_relocation(file, relocation.kind),
            }
        }
    }

    fn section_relative_failure(&mut self, chunk: usize) {
        let name = self.chunks[chunk].name.clone();
        if name.starts_with(".debug$") {
            return;
        }
        self.error(
            problem("EI208", "A relocation value does not fit in its field.")
                .with_context(format!("Section: {name}"))
                .with_context("Relocation: SECREL".to_string()),
        );
    }

    fn unsupported_coff_relocation(&mut self, file: usize, kind: u16) {
        let label = self.file_label(Some(file));
        self.error(
            problem("EI210", "The linker does not support this relocation.")
                .with_context(format!("Relocation: {kind:#x}"))
                .with_context(format!("File: {label}")),
        );
    }

    fn fix_thread_local_alignment(&mut self, out: &mut [u8]) {
        let Some(id) = self.find_underscore("_tls_used") else {
            return;
        };
        if !self.symbols[id].is_defined() {
            return;
        }
        let Some(output) = self
            .symbol_chunk(id)
            .and_then(|chunk| self.chunk_output(chunk))
        else {
            return;
        };
        let section = &self.outputs[output];
        let offset = self.symbol_rva(id) - section.virtual_address;
        let directory_size = if self.is_64() { 40 } else { 24 };
        if offset + directory_size > section.raw_size {
            self.error(
                problem(
                    "EI248",
                    "A relocation points outside its section or at a discarded section.",
                )
                .with_context("Symbol: _tls_used".to_string()),
            );
            return;
        }
        let position = (section.file_offset + offset + directory_size - 4) as usize;
        let characteristics = u32::from_le_bytes([
            out[position],
            out[position + 1],
            out[position + 2],
            out[position + 3],
        ]);
        let encoded = (self.thread_local_alignment.trailing_zeros() + 1) << 20;
        let updated = (characteristics & !0x00f0_0000) | encoded;
        out[position..position + 4].copy_from_slice(&updated.to_le_bytes());
    }

    fn sort_exception_table(&mut self, out: &mut [u8]) {
        let Some((first, last)) = self.exception_range else {
            return;
        };
        if self.machine != crate::compilers::pe::machine::AMD64 {
            self.warn(
                caution(
                    "EI212",
                    "The linker does not support what this input asks for yet.",
                )
                .with_context("Feature: sorting .pdata for this machine".to_string()),
            );
            return;
        }
        let offset_of = |context: &CoffContext, chunk: usize| -> Option<usize> {
            let output = context.chunk_output(chunk)?;
            Some(
                (context.outputs[output].file_offset + context.chunk_rva(chunk)
                    - context.outputs[output].virtual_address) as usize,
            )
        };
        let (Some(begin), Some(end_start)) = (offset_of(self, first), offset_of(self, last)) else {
            return;
        };
        let end = end_start + self.chunks[last].size as usize;
        if !(end - begin).is_multiple_of(12) {
            self.error(
                problem(
                    "EI212",
                    "The linker does not support what this input asks for yet.",
                )
                .with_context("Feature: a .pdata size that is not a multiple of 12".to_string()),
            );
            return;
        }
        let mut entries: Vec<[u8; 12]> = out[begin..end]
            .chunks(12)
            .map(|entry| entry.try_into().unwrap_or([0; 12]))
            .collect();
        entries.sort_by_key(|entry| u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]));
        for (position, entry) in entries.iter().enumerate() {
            out[begin + position * 12..begin + position * 12 + 12].copy_from_slice(entry);
        }
    }

    fn write_time_stamp(&mut self, out: &mut [u8]) {
        let mut stamp = self.options.timestamp.unwrap_or(0);
        if self.options.repro || self.options.build_identifier {
            let hash = xxh3_64(out);
            if self.options.repro {
                stamp = hash as u32;
            }
            if self.options.build_identifier {
                if let Some(record) = self
                    .chunks
                    .iter()
                    .position(|chunk| chunk.kind == CoffChunkKind::CodeViewRecord)
                {
                    if let Some(output) = self.chunk_output(record) {
                        let at = (self.outputs[output].file_offset + self.chunk_rva(record)
                            - self.outputs[output].virtual_address)
                            as usize;
                        out[at..at + 4].copy_from_slice(b"RSDS");
                        out[at + 4..at + 12].copy_from_slice(&hash.to_le_bytes());
                        out[at + 12..at + 20].copy_from_slice(b"LLD PDB.");
                        out[at + 20..at + 24].copy_from_slice(&1u32.to_le_bytes());
                    }
                }
            }
        }
        if let Some(directory) = self.debug_directory {
            if let Some(output) = self.chunk_output(directory) {
                let at = (self.outputs[output].file_offset + self.chunk_rva(directory)
                    - self.outputs[output].virtual_address) as usize;
                let count = self.chunks[directory].size as usize / 28;
                for index in 0..count {
                    let slot = at + index * 28 + 4;
                    out[slot..slot + 4].copy_from_slice(&stamp.to_le_bytes());
                }
            }
        }
        let at = self.coff_header_offset() + 4;
        out[at..at + 4].copy_from_slice(&stamp.to_le_bytes());
    }

    fn create_export_chunks(&mut self) -> Vec<usize> {
        use crate::compilers::pe::section_characteristic as character;
        let read_data = character::INITIALIZED_DATA | character::READ;
        let mut base = 1u32 << 16;
        let mut maximum = 0u32;
        for export in &self.exports {
            base = base.min(u32::from(export.ordinal));
            maximum = maximum.max(u32::from(export.ordinal));
        }
        let file_name = self
            .options
            .output
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or("")
            .to_string();
        let name = self.new_chunk(
            CoffChunkKind::Text {
                text: file_name.clone(),
            },
            file_name.len() as u64 + 1,
            1,
            read_data,
        );
        let addresses = self.new_chunk(
            CoffChunkKind::ExportAddresses { base },
            u64::from(maximum - base + 1) * 4,
            1,
            read_data,
        );
        let mut names = Vec::new();
        for export in self.exports.clone() {
            if !export.no_name {
                let text = export.export_name.clone();
                names.push(self.new_chunk(
                    CoffChunkKind::Text { text: text.clone() },
                    text.len() as u64 + 1,
                    1,
                    read_data,
                ));
            }
        }
        let mut forwards = Vec::new();
        for position in 0..self.exports.len() {
            if self.exports[position].forward_to.is_empty() {
                continue;
            }
            let text = self.exports[position].forward_to.clone();
            let chunk = self.new_chunk(
                CoffChunkKind::Text { text: text.clone() },
                text.len() as u64 + 1,
                1,
                read_data,
            );
            self.exports[position].forward_chunk = Some(chunk);
            forwards.push(chunk);
        }
        let count = names.len();
        let pointers = self.new_chunk(
            CoffChunkKind::ExportNamePointers {
                names: names.clone(),
            },
            count as u64 * 4,
            1,
            read_data,
        );
        let ordinals = self.new_chunk(
            CoffChunkKind::ExportOrdinals { base },
            count as u64 * 2,
            1,
            read_data,
        );
        let directory = self.new_chunk(
            CoffChunkKind::ExportDirectory {
                base,
                maximum,
                names: count as u32,
                name,
                addresses,
                pointers,
                ordinals,
            },
            40,
            1,
            read_data,
        );
        let mut chunks = vec![directory, name, addresses, pointers, ordinals];
        chunks.extend(names);
        chunks.extend(forwards);
        chunks
    }

    fn create_safe_exception_handler_table(&mut self, rdata: usize) {
        let mut handlers: BTreeSet<(usize, u32)> = BTreeSet::new();
        for file in self.object_order.clone() {
            if self.objects[file].feature_flags & 1 == 0 {
                let label = self.file_label(Some(file));
                self.error(
                    problem(
                        "EI260",
                        "An input is not compatible with safe exception handlers.",
                    )
                    .with_context(format!("File: {label}")),
                );
            }
            for symbol in self
                .symbols_from_index_sections(file, &self.objects[file].safe_handler_chunks.clone())
            {
                if let Some(entry) = self.rva_set_entry(symbol) {
                    handlers.insert(entry);
                }
            }
        }
        self.set_no_seh =
            handlers.is_empty() || self.find_underscore("_load_config_used").is_none();
        self.add_rva_table(
            handlers,
            "__safe_se_handler_table",
            "__safe_se_handler_count",
            rdata,
        );
    }

    fn rva_set_entry(&self, symbol: SymbolIndex) -> Option<(usize, u32)> {
        let chunk = self.symbol_chunk(symbol)?;
        let chunk = match self.chunks[chunk].kind {
            CoffChunkKind::Section => self.chunks[chunk].replacement,
            _ => chunk,
        };
        let offset = match &self.symbols[symbol].kind {
            CoffSymbolKind::Regular { value, .. } => *value,
            CoffSymbolKind::Synthetic { offset, .. } => *offset,
            _ => 0,
        };
        Some((chunk, offset))
    }

    fn symbols_from_index_sections(&mut self, file: usize, chunks: &[usize]) -> Vec<SymbolIndex> {
        let mut found = Vec::new();
        for chunk in chunks {
            if !self.chunks[*chunk].live {
                continue;
            }
            let data = self.section_data(*chunk).to_vec();
            if !data.len().is_multiple_of(4) {
                continue;
            }
            for entry in data.chunks(4) {
                let index = u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]) as usize;
                if let Some(symbol) = self.objects[file].symbols.get(index).copied().flatten() {
                    if let Some(target) = self.defined_target(symbol) {
                        if self.symbol_is_live(target) {
                            found.push(target);
                        }
                    }
                }
            }
        }
        found
    }

    fn symbol_is_live(&self, symbol: SymbolIndex) -> bool {
        match &self.symbols[symbol].kind {
            CoffSymbolKind::Regular {
                chunk: Some(chunk), ..
            } => self.chunks[self.chunks[*chunk].replacement].live,
            CoffSymbolKind::ImportData { import } => self.imports[*import].live,
            CoffSymbolKind::ImportThunk { chunk, .. } => self.chunks[*chunk].live,
            _ => true,
        }
    }

    fn add_rva_table(
        &mut self,
        entries: BTreeSet<(usize, u32)>,
        table: &str,
        count: &str,
        rdata: usize,
    ) {
        if entries.is_empty() {
            return;
        }
        let list: Vec<(usize, u32)> = entries.into_iter().collect();
        let size = list.len() as u64 * 4;
        let chunk = self.new_chunk(
            CoffChunkKind::RvaTable {
                entries: list.clone(),
                flags: None,
            },
            size,
            4,
            crate::compilers::pe::section_characteristic::INITIALIZED_DATA
                | crate::compilers::pe::section_characteristic::READ,
        );
        self.outputs[rdata].chunks.push(chunk);
        if let Some(id) = self.find_underscore(table) {
            self.symbols[id].kind = CoffSymbolKind::Synthetic {
                chunk: Some(chunk),
                offset: 0,
            };
        }
        if let Some(id) = self.find_underscore(count) {
            self.symbols[id].kind = CoffSymbolKind::Absolute {
                address: list.len() as u64,
            };
        }
    }

    fn create_guard_tables(&mut self, rdata: usize) {
        if self.options.guard == guard_level::OFF {
            let instrumented = self
                .object_order
                .iter()
                .any(|file| self.objects[*file].feature_flags & 0x800 != 0);
            if instrumented {
                if let Some(id) = self.find_underscore("__guard_flags") {
                    self.symbols[id].kind = CoffSymbolKind::Absolute { address: 0x100 };
                }
            }
            return;
        }
        self.error(not_supported("/guard:cf"));
        let _ = rdata;
    }
}

pub fn tail_merged_layout(strings: &[Vec<u8>], alignment: u64) -> (BTreeMap<Vec<u8>, u64>, u64) {
    let mut unique: Vec<Vec<u8>> = Vec::new();
    let mut seen = BTreeSet::new();
    for string in strings {
        if seen.insert(string.clone()) {
            unique.push(string.clone());
        }
    }
    let mut order: Vec<usize> = (0..unique.len()).collect();
    order.sort_by(|left, right| {
        let a = &unique[*left];
        let b = &unique[*right];
        let mut position = 0usize;
        loop {
            let first = if position < a.len() {
                Some(a[a.len() - 1 - position])
            } else {
                None
            };
            let second = if position < b.len() {
                Some(b[b.len() - 1 - position])
            } else {
                None
            };
            match (first, second) {
                (None, None) => return std::cmp::Ordering::Equal,
                (None, Some(_)) => return std::cmp::Ordering::Greater,
                (Some(_), None) => return std::cmp::Ordering::Less,
                (Some(x), Some(y)) if x != y => return y.cmp(&x),
                _ => position += 1,
            }
        }
    });
    let mut offsets = BTreeMap::new();
    let mut size = 0u64;
    let mut previous: Vec<u8> = Vec::new();
    for index in order {
        let string = &unique[index];
        if previous.ends_with(string) {
            let position = size - string.len() as u64;
            if position.is_multiple_of(alignment.max(1)) {
                offsets.insert(string.clone(), position);
                continue;
            }
        }
        size = align_to(size, alignment.max(1));
        offsets.insert(string.clone(), size);
        size += string.len() as u64;
        previous = string.clone();
    }
    (offsets, size)
}
