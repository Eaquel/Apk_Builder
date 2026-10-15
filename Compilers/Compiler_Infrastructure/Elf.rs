use crate::compilers::compiler_infrastructure::fail;
use crate::diag::Diagnostic;
use std::collections::BTreeMap;

pub mod machine {
    pub const NONE: u16 = 0;
    pub const X86: u16 = 3;
    pub const ARM: u16 = 40;
    pub const X86_64: u16 = 62;
    pub const AARCH64: u16 = 183;
}

pub mod file_type {
    pub const NONE: u16 = 0;
    pub const RELOCATABLE: u16 = 1;
    pub const EXECUTABLE: u16 = 2;
    pub const SHARED: u16 = 3;
    pub const CORE: u16 = 4;
}

pub mod section_type {
    pub const NULL: u32 = 0;
    pub const PROGRAM_BITS: u32 = 1;
    pub const SYMBOL_TABLE: u32 = 2;
    pub const STRING_TABLE: u32 = 3;
    pub const RELOCATIONS_WITH_ADDENDS: u32 = 4;
    pub const HASH: u32 = 5;
    pub const DYNAMIC: u32 = 6;
    pub const NOTE: u32 = 7;
    pub const NO_BITS: u32 = 8;
    pub const RELOCATIONS: u32 = 9;
    pub const SHARED_LIBRARY: u32 = 10;
    pub const DYNAMIC_SYMBOLS: u32 = 11;
    pub const INIT_ARRAY: u32 = 14;
    pub const FINI_ARRAY: u32 = 15;
    pub const PREINIT_ARRAY: u32 = 16;
    pub const GROUP: u32 = 17;
    pub const SYMBOL_TABLE_INDICES: u32 = 18;
    pub const RELATIVE_RELOCATIONS: u32 = 19;
    pub const ANDROID_RELOCATIONS: u32 = 0x6000_0001;
    pub const ANDROID_RELOCATIONS_WITH_ADDENDS: u32 = 0x6000_0002;
    pub const LLVM_ODR_TABLE: u32 = 0x6fff_4c00;
    pub const LLVM_LINKER_OPTIONS: u32 = 0x6fff_4c01;
    pub const LLVM_ADDRESS_SIGNIFICANCE: u32 = 0x6fff_4c03;
    pub const LLVM_DEPENDENT_LIBRARIES: u32 = 0x6fff_4c04;
    pub const LLVM_SYMBOL_PARTITION: u32 = 0x6fff_4c05;
    pub const LLVM_PART_EHDR: u32 = 0x6fff_4c06;
    pub const LLVM_PART_PHDR: u32 = 0x6fff_4c07;
    pub const LLVM_CALL_GRAPH_PROFILE: u32 = 0x6fff_4c09;
    pub const ANDROID_RELATIVE_RELOCATIONS: u32 = 0x6fff_ff00;
    pub const GNU_ATTRIBUTES: u32 = 0x6fff_fff5;
    pub const GNU_HASH: u32 = 0x6fff_fff6;
    pub const GNU_VERSION_DEFINITIONS: u32 = 0x6fff_fffd;
    pub const GNU_VERSION_NEEDS: u32 = 0x6fff_fffe;
    pub const GNU_VERSION_SYMBOLS: u32 = 0x6fff_ffff;
    pub const ARM_EXCEPTION_INDEX: u32 = 0x7000_0001;
    pub const ARM_ATTRIBUTES: u32 = 0x7000_0003;
    pub const AARCH64_ATTRIBUTES: u32 = 0x7000_0003;
    pub const AARCH64_MEMORY_TAG_GLOBALS_DYNAMIC: u32 = 0x7000_0007;
    pub const X86_64_UNWIND: u32 = 0x7000_0001;
}

pub mod section_flag {
    pub const WRITE: u64 = 0x1;
    pub const ALLOCATE: u64 = 0x2;
    pub const EXECUTE: u64 = 0x4;
    pub const MERGE: u64 = 0x10;
    pub const STRINGS: u64 = 0x20;
    pub const INFO_LINK: u64 = 0x40;
    pub const LINK_ORDER: u64 = 0x80;
    pub const OPERATING_SYSTEM_NONCONFORMING: u64 = 0x100;
    pub const GROUP: u64 = 0x200;
    pub const THREAD_LOCAL: u64 = 0x400;
    pub const COMPRESSED: u64 = 0x800;
    pub const GNU_RETAIN: u64 = 0x20_0000;
    pub const X86_64_LARGE: u64 = 0x1000_0000;
    pub const ARM_PURE_CODE: u64 = 0x2000_0000;
    pub const AARCH64_PURE_CODE: u64 = 0x2000_0000;
    pub const EXCLUDE: u64 = 0x8000_0000;
}

pub mod segment_type {
    pub const NULL: u32 = 0;
    pub const LOAD: u32 = 1;
    pub const DYNAMIC: u32 = 2;
    pub const INTERPRETER: u32 = 3;
    pub const NOTE: u32 = 4;
    pub const SHARED_LIBRARY: u32 = 5;
    pub const PROGRAM_HEADERS: u32 = 6;
    pub const THREAD_LOCAL: u32 = 7;
    pub const GNU_EXCEPTION_FRAME: u32 = 0x6474_e550;
    pub const GNU_STACK: u32 = 0x6474_e551;
    pub const GNU_RELRO: u32 = 0x6474_e552;
    pub const GNU_PROPERTY: u32 = 0x6474_e553;
    pub const OPENBSD_MUTABLE: u32 = 0x65a3_dbe5;
    pub const OPENBSD_RANDOMIZE: u32 = 0x65a3_dbe6;
    pub const OPENBSD_WRITE_XOR_EXECUTE_NEEDED: u32 = 0x65a3_dbe7;
    pub const OPENBSD_NO_BRANCH_TARGET: u32 = 0x65a3_dbe8;
    pub const OPENBSD_SYSTEM_CALLS: u32 = 0x65a3_dbe9;
    pub const OPENBSD_BOOT_DATA: u32 = 0x65a4_1be6;
    pub const ARM_EXCEPTION_INDEX: u32 = 0x7000_0001;
    pub const AARCH64_MEMORY_TAG_EXTENSION: u32 = 0x7000_0002;
}

pub mod segment_flag {
    pub const EXECUTE: u32 = 1;
    pub const WRITE: u32 = 2;
    pub const READ: u32 = 4;
}

pub mod dynamic_tag {
    pub const NULL: i64 = 0;
    pub const NEEDED: i64 = 1;
    pub const PLT_RELOCATIONS_SIZE: i64 = 2;
    pub const PLT_GOT: i64 = 3;
    pub const HASH: i64 = 4;
    pub const STRING_TABLE: i64 = 5;
    pub const SYMBOL_TABLE: i64 = 6;
    pub const RELOCATIONS_WITH_ADDENDS: i64 = 7;
    pub const RELOCATIONS_WITH_ADDENDS_SIZE: i64 = 8;
    pub const RELOCATION_WITH_ADDEND_ENTRY: i64 = 9;
    pub const STRING_TABLE_SIZE: i64 = 10;
    pub const SYMBOL_ENTRY: i64 = 11;
    pub const INIT: i64 = 12;
    pub const FINI: i64 = 13;
    pub const SHARED_OBJECT_NAME: i64 = 14;
    pub const RUN_PATH_OLD: i64 = 15;
    pub const SYMBOLIC: i64 = 16;
    pub const RELOCATIONS: i64 = 17;
    pub const RELOCATIONS_SIZE: i64 = 18;
    pub const RELOCATION_ENTRY: i64 = 19;
    pub const PLT_RELOCATION_KIND: i64 = 20;
    pub const DEBUG: i64 = 21;
    pub const TEXT_RELOCATIONS: i64 = 22;
    pub const JUMP_RELOCATIONS: i64 = 23;
    pub const BIND_NOW: i64 = 24;
    pub const INIT_ARRAY: i64 = 25;
    pub const FINI_ARRAY: i64 = 26;
    pub const INIT_ARRAY_SIZE: i64 = 27;
    pub const FINI_ARRAY_SIZE: i64 = 28;
    pub const RUN_PATH: i64 = 29;
    pub const FLAGS: i64 = 30;
    pub const PREINIT_ARRAY: i64 = 32;
    pub const PREINIT_ARRAY_SIZE: i64 = 33;
    pub const SYMBOL_TABLE_INDICES: i64 = 34;
    pub const RELATIVE_RELOCATIONS_SIZE: i64 = 35;
    pub const RELATIVE_RELOCATIONS: i64 = 36;
    pub const RELATIVE_RELOCATION_ENTRY: i64 = 37;
    pub const ANDROID_RELOCATIONS: i64 = 0x6000_000f;
    pub const ANDROID_RELOCATIONS_SIZE: i64 = 0x6000_0010;
    pub const ANDROID_RELOCATIONS_WITH_ADDENDS: i64 = 0x6000_0011;
    pub const ANDROID_RELOCATIONS_WITH_ADDENDS_SIZE: i64 = 0x6000_0012;
    pub const ANDROID_RELATIVE_RELOCATIONS: i64 = 0x6fff_e000;
    pub const ANDROID_RELATIVE_RELOCATIONS_SIZE: i64 = 0x6fff_e001;
    pub const ANDROID_RELATIVE_RELOCATION_ENTRY: i64 = 0x6fff_e003;
    pub const ANDROID_RELATIVE_RELOCATION_COUNT: i64 = 0x6fff_e005;
    pub const GNU_HASH: i64 = 0x6fff_fef5;
    pub const RELATIVE_WITH_ADDEND_COUNT: i64 = 0x6fff_fff9;
    pub const RELATIVE_COUNT: i64 = 0x6fff_fffa;
    pub const FLAGS_1: i64 = 0x6fff_fffb;
    pub const VERSION_SYMBOLS: i64 = 0x6fff_fff0;
    pub const VERSION_DEFINITIONS: i64 = 0x6fff_fffc;
    pub const VERSION_DEFINITION_COUNT: i64 = 0x6fff_fffd;
    pub const VERSION_NEEDS: i64 = 0x6fff_fffe;
    pub const VERSION_NEED_COUNT: i64 = 0x6fff_ffff;
    pub const AARCH64_BRANCH_TARGET_PLT: i64 = 0x7000_0001;
    pub const AARCH64_POINTER_AUTHENTICATION_PLT: i64 = 0x7000_0003;
    pub const AARCH64_VARIANT_PROCEDURE_CALL: i64 = 0x7000_0005;
    pub const AARCH64_MEMORY_TAG_MODE: i64 = 0x7000_0009;
    pub const AARCH64_MEMORY_TAG_HEAP: i64 = 0x7000_000b;
    pub const AARCH64_MEMORY_TAG_STACK: i64 = 0x7000_000c;
    pub const AARCH64_MEMORY_TAG_GLOBALS: i64 = 0x7000_000d;
    pub const AARCH64_MEMORY_TAG_GLOBALS_SIZE: i64 = 0x7000_000f;
}

pub mod dynamic_flag {
    pub const ORIGIN: u64 = 0x1;
    pub const SYMBOLIC: u64 = 0x2;
    pub const TEXT_RELOCATIONS: u64 = 0x4;
    pub const BIND_NOW: u64 = 0x8;
    pub const STATIC_THREAD_LOCAL: u64 = 0x10;
}

pub mod dynamic_flag_1 {
    pub const NOW: u64 = 0x1;
    pub const GLOBAL: u64 = 0x2;
    pub const GROUP: u64 = 0x4;
    pub const NO_DELETE: u64 = 0x8;
    pub const LOAD_FILTER: u64 = 0x10;
    pub const INITIALIZE_FIRST: u64 = 0x20;
    pub const NO_OPEN: u64 = 0x40;
    pub const ORIGIN: u64 = 0x80;
    pub const DIRECT: u64 = 0x100;
    pub const INTERPOSE: u64 = 0x400;
    pub const NO_DEFAULT_LIBRARY: u64 = 0x800;
    pub const NO_DUMP: u64 = 0x1000;
    pub const NO_DIRECT: u64 = 0x2_0000;
    pub const PIE: u64 = 0x800_0000;
}

pub mod symbol_binding {
    pub const LOCAL: u8 = 0;
    pub const GLOBAL: u8 = 1;
    pub const WEAK: u8 = 2;
    pub const GNU_UNIQUE: u8 = 10;
}

pub mod symbol_type {
    pub const NO_TYPE: u8 = 0;
    pub const OBJECT: u8 = 1;
    pub const FUNCTION: u8 = 2;
    pub const SECTION: u8 = 3;
    pub const FILE: u8 = 4;
    pub const COMMON: u8 = 5;
    pub const THREAD_LOCAL: u8 = 6;
    pub const GNU_INDIRECT_FUNCTION: u8 = 10;
}

pub mod visibility {
    pub const DEFAULT: u8 = 0;
    pub const INTERNAL: u8 = 1;
    pub const HIDDEN: u8 = 2;
    pub const PROTECTED: u8 = 3;
}

pub mod section_index {
    pub const UNDEFINED: u32 = 0;
    pub const RESERVED_LOW: u32 = 0xff00;
    pub const ABSOLUTE: u32 = 0xfff1;
    pub const COMMON: u32 = 0xfff2;
    pub const EXTENDED: u32 = 0xffff;
}

pub mod group_flag {
    pub const COMDAT: u32 = 1;
}

pub mod note_type {
    pub const GNU_ABI_TAG: u32 = 1;
    pub const GNU_BUILD_IDENTIFIER: u32 = 3;
    pub const GNU_PROPERTY: u32 = 5;
    pub const ANDROID_IDENTIFIER: u32 = 1;
    pub const ANDROID_MEMORY_TAG: u32 = 4;
}

pub mod version_index {
    pub const LOCAL: u16 = 0;
    pub const GLOBAL: u16 = 1;
    pub const HIDDEN: u16 = 0x8000;
}

pub mod version_flag {
    pub const BASE: u16 = 1;
    pub const WEAK: u16 = 2;
}

pub const PAGE_SIZE_ANDROID_15: u64 = 16384;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Class {
    Elf32,
    Elf64,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Encoding {
    pub class: Class,
    pub endian: Endian,
}

impl Encoding {
    pub const LITTLE_32: Encoding = Encoding {
        class: Class::Elf32,
        endian: Endian::Little,
    };
    pub const LITTLE_64: Encoding = Encoding {
        class: Class::Elf64,
        endian: Endian::Little,
    };

    pub fn is_64(self) -> bool {
        self.class == Class::Elf64
    }

    pub fn word_size(self) -> usize {
        if self.is_64() {
            8
        } else {
            4
        }
    }

    pub fn header_size(self) -> usize {
        if self.is_64() {
            64
        } else {
            52
        }
    }

    pub fn program_header_size(self) -> usize {
        if self.is_64() {
            56
        } else {
            32
        }
    }

    pub fn section_header_size(self) -> usize {
        if self.is_64() {
            64
        } else {
            40
        }
    }

    pub fn symbol_size(self) -> usize {
        if self.is_64() {
            24
        } else {
            16
        }
    }

    pub fn relocation_size(self, with_addend: bool) -> usize {
        match (self.is_64(), with_addend) {
            (true, true) => 24,
            (true, false) => 16,
            (false, true) => 12,
            (false, false) => 8,
        }
    }

    pub fn dynamic_entry_size(self) -> usize {
        if self.is_64() {
            16
        } else {
            8
        }
    }

    pub fn read_u16(self, bytes: &[u8], at: usize) -> Option<u16> {
        let held: [u8; 2] = bytes.get(at..at.checked_add(2)?)?.try_into().ok()?;
        Some(match self.endian {
            Endian::Little => u16::from_le_bytes(held),
            Endian::Big => u16::from_be_bytes(held),
        })
    }

    pub fn read_u32(self, bytes: &[u8], at: usize) -> Option<u32> {
        let held: [u8; 4] = bytes.get(at..at.checked_add(4)?)?.try_into().ok()?;
        Some(match self.endian {
            Endian::Little => u32::from_le_bytes(held),
            Endian::Big => u32::from_be_bytes(held),
        })
    }

    pub fn read_u64(self, bytes: &[u8], at: usize) -> Option<u64> {
        let held: [u8; 8] = bytes.get(at..at.checked_add(8)?)?.try_into().ok()?;
        Some(match self.endian {
            Endian::Little => u64::from_le_bytes(held),
            Endian::Big => u64::from_be_bytes(held),
        })
    }

    pub fn read_word(self, bytes: &[u8], at: usize) -> Option<u64> {
        if self.is_64() {
            self.read_u64(bytes, at)
        } else {
            self.read_u32(bytes, at).map(u64::from)
        }
    }

    pub fn put_u16(self, out: &mut Vec<u8>, value: u16) {
        match self.endian {
            Endian::Little => out.extend_from_slice(&value.to_le_bytes()),
            Endian::Big => out.extend_from_slice(&value.to_be_bytes()),
        }
    }

    pub fn put_u32(self, out: &mut Vec<u8>, value: u32) {
        match self.endian {
            Endian::Little => out.extend_from_slice(&value.to_le_bytes()),
            Endian::Big => out.extend_from_slice(&value.to_be_bytes()),
        }
    }

    pub fn put_u64(self, out: &mut Vec<u8>, value: u64) {
        match self.endian {
            Endian::Little => out.extend_from_slice(&value.to_le_bytes()),
            Endian::Big => out.extend_from_slice(&value.to_be_bytes()),
        }
    }

    pub fn put_word(self, out: &mut Vec<u8>, value: u64) {
        if self.is_64() {
            self.put_u64(out, value);
        } else {
            self.put_u32(out, value as u32);
        }
    }

    pub fn write_u16(self, bytes: &mut [u8], at: usize, value: u16) {
        let encoded = match self.endian {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        };
        bytes[at..at + 2].copy_from_slice(&encoded);
    }

    pub fn write_u32(self, bytes: &mut [u8], at: usize, value: u32) {
        let encoded = match self.endian {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        };
        bytes[at..at + 4].copy_from_slice(&encoded);
    }

    pub fn write_u64(self, bytes: &mut [u8], at: usize, value: u64) {
        let encoded = match self.endian {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        };
        bytes[at..at + 8].copy_from_slice(&encoded);
    }

    pub fn write_word(self, bytes: &mut [u8], at: usize, value: u64) {
        if self.is_64() {
            self.write_u64(bytes, at, value);
        } else {
            self.write_u32(bytes, at, value as u32);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub encoding: Encoding,
    pub operating_system: u8,
    pub abi_version: u8,
    pub file_type: u16,
    pub machine: u16,
    pub version: u32,
    pub entry: u64,
    pub program_header_offset: u64,
    pub section_header_offset: u64,
    pub flags: u32,
    pub header_size: u16,
    pub program_header_entry_size: u16,
    pub program_header_count: u32,
    pub section_header_entry_size: u16,
    pub section_count: u32,
    pub section_name_table: u32,
}

impl Header {
    pub fn new(encoding: Encoding, file_type: u16, machine: u16) -> Header {
        Header {
            encoding,
            operating_system: 0,
            abi_version: 0,
            file_type,
            machine,
            version: 1,
            entry: 0,
            program_header_offset: 0,
            section_header_offset: 0,
            flags: 0,
            header_size: encoding.header_size() as u16,
            program_header_entry_size: encoding.program_header_size() as u16,
            program_header_count: 0,
            section_header_entry_size: encoding.section_header_size() as u16,
            section_count: 0,
            section_name_table: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Section {
    pub name: String,
    pub name_offset: u32,
    pub kind: u32,
    pub flags: u64,
    pub address: u64,
    pub offset: u64,
    pub size: u64,
    pub link: u32,
    pub info: u32,
    pub alignment: u64,
    pub entry_size: u64,
    pub data: Vec<u8>,
}

impl Section {
    pub fn occupies_file(&self) -> bool {
        self.kind != section_type::NO_BITS && self.kind != section_type::NULL
    }

    pub fn is_allocated(&self) -> bool {
        self.flags & section_flag::ALLOCATE != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ProgramHeader {
    pub kind: u32,
    pub flags: u32,
    pub offset: u64,
    pub virtual_address: u64,
    pub physical_address: u64,
    pub file_size: u64,
    pub memory_size: u64,
    pub alignment: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Symbol {
    pub name: String,
    pub name_offset: u32,
    pub value: u64,
    pub size: u64,
    pub binding: u8,
    pub kind: u8,
    pub other: u8,
    pub section: u32,
}

impl Symbol {
    pub fn info(&self) -> u8 {
        (self.binding << 4) | (self.kind & 0xf)
    }

    pub fn visibility(&self) -> u8 {
        self.other & 3
    }

    pub fn is_undefined(&self) -> bool {
        self.section == section_index::UNDEFINED
    }

    pub fn is_common(&self) -> bool {
        self.section == section_index::COMMON
    }

    pub fn is_absolute(&self) -> bool {
        self.section == section_index::ABSOLUTE
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Relocation {
    pub offset: u64,
    pub kind: u32,
    pub symbol: u32,
    pub addend: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynamicEntry {
    pub tag: i64,
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    pub name: Vec<u8>,
    pub kind: u32,
    pub description: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionDefinition {
    pub flags: u16,
    pub index: u16,
    pub hash: u32,
    pub names: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionNeedAuxiliary {
    pub hash: u32,
    pub flags: u16,
    pub other: u16,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionNeed {
    pub file: String,
    pub entries: Vec<VersionNeedAuxiliary>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub flags: u32,
    pub signature: u32,
    pub members: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElfFile {
    pub header: Header,
    pub sections: Vec<Section>,
    pub program_headers: Vec<ProgramHeader>,
}

const BUT_GOT: &str = ", but got ";

fn unreadable(offset: u64, reason: &str) -> Diagnostic {
    fail("EI200", "The object file could not be read.")
        .with_context(format!("Offset: {offset}"))
        .with_context(format!("Reason: {reason}"))
}

pub fn is_elf(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\x7fELF")
}

pub fn identify(bytes: &[u8]) -> Option<Encoding> {
    if !is_elf(bytes) || bytes.len() < 16 {
        return None;
    }
    let class = match bytes[4] {
        1 => Class::Elf32,
        2 => Class::Elf64,
        _ => return None,
    };
    let endian = match bytes[5] {
        1 => Endian::Little,
        2 => Endian::Big,
        _ => return None,
    };
    Some(Encoding { class, endian })
}

pub fn string_at(table: &[u8], offset: u32) -> Option<String> {
    let start = offset as usize;
    let rest = table.get(start..)?;
    let end = rest.iter().position(|&byte| byte == 0)?;
    Some(String::from_utf8_lossy(&rest[..end]).into_owned())
}

pub fn read_header(bytes: &[u8]) -> Result<Header, Diagnostic> {
    if !is_elf(bytes) {
        return Err(unreadable(0, "not an ELF file"));
    }
    if !matches!(bytes.get(5), Some(1) | Some(2)) {
        return Err(unreadable(0, "corrupted ELF file: invalid data encoding"));
    }
    if !matches!(bytes.get(4), Some(1) | Some(2)) {
        return Err(unreadable(0, "corrupted ELF file: invalid file class"));
    }
    let encoding = identify(bytes).ok_or_else(|| unreadable(0, "not an ELF file"))?;
    if bytes.len() < encoding.header_size() {
        return Err(unreadable(0, "the file header is truncated"));
    }
    let short = || unreadable(0, "the file header is truncated");
    let half = |at: usize| encoding.read_u16(bytes, at).ok_or_else(short);
    let full = |at: usize| encoding.read_u32(bytes, at).ok_or_else(short);
    let word = |at: usize| encoding.read_word(bytes, at).ok_or_else(short);
    let (entry, program, section, rest) = if encoding.is_64() {
        (word(24)?, word(32)?, word(40)?, 48)
    } else {
        (word(24)?, word(28)?, word(32)?, 36)
    };
    Ok(Header {
        encoding,
        operating_system: bytes[7],
        abi_version: bytes[8],
        file_type: half(16)?,
        machine: half(18)?,
        version: full(20)?,
        entry,
        program_header_offset: program,
        section_header_offset: section,
        flags: full(rest)?,
        header_size: half(rest + 4)?,
        program_header_entry_size: half(rest + 6)?,
        program_header_count: u32::from(half(rest + 8)?),
        section_header_entry_size: half(rest + 10)?,
        section_count: u32::from(half(rest + 12)?),
        section_name_table: u32::from(half(rest + 14)?),
    })
}

impl ElfFile {
    pub fn new(encoding: Encoding, file_type: u16, machine: u16) -> ElfFile {
        ElfFile {
            header: Header::new(encoding, file_type, machine),
            sections: vec![Section::default()],
            program_headers: Vec::new(),
        }
    }

    pub fn encoding(&self) -> Encoding {
        self.header.encoding
    }

    pub fn parse(bytes: &[u8]) -> Result<ElfFile, Diagnostic> {
        let mut header = read_header(bytes)?;
        let encoding = header.encoding;
        let mut sections = Vec::new();
        let section_size = encoding.section_header_size();
        let table = header.section_header_offset as usize;
        if header.section_header_offset != 0 {
            if usize::from(header.section_header_entry_size) != section_size {
                return Err(unreadable(
                    header.section_header_offset,
                    "the section header size is wrong",
                ));
            }
            let first = read_section_header(bytes, encoding, table)?;
            if header.section_count == 0 {
                header.section_count = first.size as u32;
            }
            if header.section_name_table == section_index::EXTENDED {
                header.section_name_table = first.link;
            }
            if header.program_header_count == 0xffff {
                header.program_header_count = first.info;
            }
            let count = header.section_count as usize;
            let end = count
                .checked_mul(section_size)
                .and_then(|length| length.checked_add(table));
            if end.is_none_or(|end| end > bytes.len()) {
                return Err(unreadable(
                    header.section_header_offset,
                    "the section header table runs past the end of the file",
                ));
            }
            for index in 0..count {
                let mut section =
                    read_section_header(bytes, encoding, table + index * section_size)?;
                if index > 0 && section.occupies_file() {
                    let start = section.offset as usize;
                    let stop = section
                        .offset
                        .checked_add(section.size)
                        .filter(|stop| *stop <= bytes.len() as u64)
                        .ok_or_else(|| {
                            unreadable(
                                section.offset,
                                &format!(
                                    "section [index {index}] has a sh_offset (0x{:x}) + sh_size (0x{:x}) that is greater than the file size (0x{:x})",
                                    section.offset,
                                    section.size,
                                    bytes.len()
                                ),
                            )
                        })?;
                    section.data = bytes[start..stop as usize].to_vec();
                }
                sections.push(section);
            }
            let names = header.section_name_table as usize;
            if names != 0 {
                let table_data = sections
                    .get(names)
                    .map(|section| section.data.clone())
                    .ok_or_else(|| {
                        unreadable(
                            header.section_header_offset,
                            "the section name table index is out of range",
                        )
                    })?;
                for section in sections.iter_mut() {
                    section.name =
                        string_at(&table_data, section.name_offset).ok_or_else(|| {
                            unreadable(
                                header.section_header_offset,
                                "a section name lies outside the name table",
                            )
                        })?;
                }
            }
        }
        let mut program_headers = Vec::new();
        if header.program_header_offset != 0 && header.program_header_count != 0 {
            let size = encoding.program_header_size();
            if usize::from(header.program_header_entry_size) != size {
                return Err(unreadable(
                    header.program_header_offset,
                    "the program header size is wrong",
                ));
            }
            for index in 0..header.program_header_count as usize {
                let at = header.program_header_offset as usize + index * size;
                program_headers.push(read_program_header(bytes, encoding, at)?);
            }
        }
        Ok(ElfFile {
            header,
            sections,
            program_headers,
        })
    }

    pub fn section_named(&self, name: &str) -> Option<usize> {
        self.sections
            .iter()
            .position(|section| section.name == name)
    }

    pub fn sections_of_kind(&self, kind: u32) -> Vec<usize> {
        (0..self.sections.len())
            .filter(|index| self.sections[*index].kind == kind)
            .collect()
    }

    pub fn symbol_table(&self) -> Option<usize> {
        self.sections_of_kind(section_type::SYMBOL_TABLE)
            .first()
            .copied()
    }

    pub fn dynamic_symbol_table(&self) -> Option<usize> {
        self.sections_of_kind(section_type::DYNAMIC_SYMBOLS)
            .first()
            .copied()
    }

    pub fn symbols(&self, table: usize) -> Result<Vec<Symbol>, Diagnostic> {
        let section = self
            .sections
            .get(table)
            .ok_or_else(|| unreadable(0, "the symbol table index is out of range"))?;
        let strings = self
            .sections
            .get(section.link as usize)
            .map(|strings| strings.data.as_slice())
            .unwrap_or(&[]);
        let extended = self.sections.iter().find(|candidate| {
            candidate.kind == section_type::SYMBOL_TABLE_INDICES && candidate.link as usize == table
        });
        decode_symbols(
            self.encoding(),
            &section.data,
            strings,
            extended.map(|found| found.data.as_slice()),
        )
        .map_err(|(at, reason)| unreadable(section.offset + at as u64, reason))
    }

    pub fn relocations(&self, index: usize) -> Result<Vec<Relocation>, Diagnostic> {
        let section = &self.sections[index];
        let with_addend = match section.kind {
            section_type::RELOCATIONS_WITH_ADDENDS => true,
            section_type::RELOCATIONS => false,
            _ => return Err(unreadable(section.offset, "not a relocation section")),
        };
        decode_relocations(self.encoding(), &section.data, with_addend)
            .map_err(|(at, reason)| unreadable(section.offset + at as u64, reason))
    }

    pub fn group(&self, index: usize) -> Result<Group, Diagnostic> {
        let section = &self.sections[index];
        let encoding = self.encoding();
        if section.data.len() < 4 || !section.data.len().is_multiple_of(4) {
            return Err(unreadable(section.offset, "a group section is malformed"));
        }
        let words: Vec<u32> = (0..section.data.len() / 4)
            .filter_map(|at| encoding.read_u32(&section.data, at * 4))
            .collect();
        Ok(Group {
            flags: words[0],
            signature: section.info,
            members: words[1..].to_vec(),
        })
    }

    pub fn dynamic_entries(&self) -> Result<Vec<DynamicEntry>, Diagnostic> {
        let Some(index) = self
            .sections_of_kind(section_type::DYNAMIC)
            .first()
            .copied()
        else {
            return Ok(Vec::new());
        };
        let section = &self.sections[index];
        let entry_size = (self.encoding().word_size() * 2) as u64;
        if section.entry_size != entry_size {
            return Err(unreadable(
                section.offset,
                &format!(
                    "section [index {index}] has invalid sh_entsize: expected {entry_size}{BUT_GOT}{}",
                    section.entry_size
                ),
            ));
        }
        if !section.size.is_multiple_of(entry_size) {
            return Err(unreadable(
                section.offset,
                &format!(
                    "section [index {index}] has an invalid sh_size ({}) which is not a multiple of its sh_entsize ({entry_size})",
                    section.size
                ),
            ));
        }
        Ok(decode_dynamic(self.encoding(), &section.data))
    }

    pub fn needed_libraries(&self) -> Result<Vec<String>, Diagnostic> {
        let Some(index) = self
            .sections_of_kind(section_type::DYNAMIC)
            .first()
            .copied()
        else {
            return Ok(Vec::new());
        };
        let strings = &self.sections[self.sections[index].link as usize].data;
        let mut names = Vec::new();
        for entry in self.dynamic_entries()? {
            if entry.tag != dynamic_tag::NEEDED {
                continue;
            }
            let name = string_at(strings, entry.value as u32)
                .filter(|_| entry.value < strings.len() as u64)
                .ok_or_else(|| {
                    unreadable(self.sections[index].offset, "invalid DT_NEEDED entry")
                })?;
            names.push(name);
        }
        Ok(names)
    }

    pub fn shared_object_name(&self) -> Result<Option<String>, Diagnostic> {
        let Some(index) = self
            .sections_of_kind(section_type::DYNAMIC)
            .first()
            .copied()
        else {
            return Ok(None);
        };
        let strings = &self.sections[self.sections[index].link as usize].data;
        let Some(entry) = self
            .dynamic_entries()?
            .into_iter()
            .find(|entry| entry.tag == dynamic_tag::SHARED_OBJECT_NAME)
        else {
            return Ok(None);
        };
        string_at(strings, entry.value as u32)
            .filter(|_| entry.value < strings.len() as u64)
            .map(Some)
            .ok_or_else(|| unreadable(self.sections[index].offset, "invalid DT_SONAME entry"))
    }

    pub fn notes(&self, index: usize) -> Result<Vec<Note>, Diagnostic> {
        let section = &self.sections[index];
        decode_notes(self.encoding(), &section.data, section.alignment.max(4))
            .map_err(|reason| unreadable(section.offset, reason))
    }

    pub fn version_symbols(&self) -> Vec<u16> {
        let encoding = self.encoding();
        self.sections_of_kind(section_type::GNU_VERSION_SYMBOLS)
            .first()
            .map(|index| {
                let data = &self.sections[*index].data;
                (0..data.len() / 2)
                    .filter_map(|at| encoding.read_u16(data, at * 2))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn version_definitions(&self) -> Result<Vec<VersionDefinition>, Diagnostic> {
        let Some(index) = self
            .sections_of_kind(section_type::GNU_VERSION_DEFINITIONS)
            .first()
            .copied()
        else {
            return Ok(Vec::new());
        };
        let section = &self.sections[index];
        let strings = &self.sections[section.link as usize].data;
        decode_version_definitions(self.encoding(), &section.data, strings)
            .map_err(|reason| unreadable(section.offset, reason))
    }

    pub fn version_needs(&self) -> Result<Vec<VersionNeed>, Diagnostic> {
        let Some(index) = self
            .sections_of_kind(section_type::GNU_VERSION_NEEDS)
            .first()
            .copied()
        else {
            return Ok(Vec::new());
        };
        let section = &self.sections[index];
        let strings = &self.sections[section.link as usize].data;
        decode_version_needs(self.encoding(), &section.data, strings)
            .map_err(|reason| unreadable(section.offset, reason))
    }

    pub fn version_need_problem(&self) -> Option<&'static str> {
        let index = self
            .sections_of_kind(section_type::GNU_VERSION_NEEDS)
            .first()
            .copied()?;
        let section = &self.sections[index];
        let encoding = self.encoding();
        let data = &section.data;
        let strings = self
            .dynamic_symbol_table()
            .and_then(|table| self.sections.get(self.sections[table].link as usize))
            .map_or(0, |strings| strings.data.len() as u64);
        let mut at = 0usize;
        for _ in 0..section.info {
            if at.checked_add(16).is_none_or(|end| end > data.len()) {
                return Some("has an invalid Verneed");
            }
            let count = encoding.read_u16(data, at + 2).unwrap_or(0);
            let auxiliary = encoding.read_u32(data, at + 8).unwrap_or(0) as usize;
            let next = encoding.read_u32(data, at + 12).unwrap_or(0) as usize;
            let mut cursor = at.saturating_add(auxiliary);
            for _ in 0..count {
                if cursor.checked_add(16).is_none_or(|end| end > data.len()) {
                    return Some("has an invalid Vernaux");
                }
                let name = encoding.read_u32(data, cursor + 8).unwrap_or(0);
                if u64::from(name) >= strings {
                    return Some("has a Vernaux with an invalid vna_name");
                }
                cursor = cursor
                    .saturating_add(encoding.read_u32(data, cursor + 12).unwrap_or(0) as usize);
            }
            at = at.saturating_add(next);
        }
        None
    }

    pub fn assign_object_layout(&mut self) {
        let encoding = self.encoding();
        let mut at = encoding.header_size() as u64;
        if !self.program_headers.is_empty() {
            self.header.program_header_offset = at;
            at += (self.program_headers.len() * encoding.program_header_size()) as u64;
        }
        for section in self.sections.iter_mut().skip(1) {
            let alignment = section.alignment.max(1);
            at = align_to(at, alignment);
            section.offset = at;
            if section.occupies_file() {
                section.size = section.data.len() as u64;
                at += section.size;
            }
        }
        at = align_to(at, encoding.word_size() as u64);
        self.header.section_header_offset = at;
    }

    pub fn rebuild_section_names(&mut self) {
        let index = self.header.section_name_table as usize;
        if index == 0 || index >= self.sections.len() {
            return;
        }
        let mut table = StringTable::new();
        let offsets: Vec<u32> = self
            .sections
            .iter()
            .map(|section| {
                if section.kind == section_type::NULL && section.name.is_empty() {
                    0
                } else {
                    table.add(&section.name)
                }
            })
            .collect();
        for (section, offset) in self.sections.iter_mut().zip(offsets) {
            section.name_offset = offset;
        }
        self.sections[index].data = table.into_bytes();
        self.sections[index].size = self.sections[index].data.len() as u64;
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let encoding = self.encoding();
        let mut end = encoding.header_size() as u64;
        if !self.program_headers.is_empty() {
            end = end.max(
                self.header.program_header_offset
                    + (self.program_headers.len() * encoding.program_header_size()) as u64,
            );
        }
        for section in self.sections.iter().skip(1) {
            if section.occupies_file() {
                end = end.max(section.offset + section.data.len() as u64);
            }
        }
        if self.sections.len() > 1 || self.header.section_header_offset != 0 {
            end = end.max(
                self.header.section_header_offset
                    + (self.sections.len() * encoding.section_header_size()) as u64,
            );
        }
        let mut out = vec![0u8; end as usize];
        let header = self.encode_header();
        out[..header.len()].copy_from_slice(&header);
        if !self.program_headers.is_empty() {
            let mut table = Vec::new();
            for program in &self.program_headers {
                encode_program_header(encoding, program, &mut table);
            }
            let at = self.header.program_header_offset as usize;
            out[at..at + table.len()].copy_from_slice(&table);
        }
        for section in self.sections.iter().skip(1) {
            if section.occupies_file() && !section.data.is_empty() {
                let at = section.offset as usize;
                out[at..at + section.data.len()].copy_from_slice(&section.data);
            }
        }
        if self.sections.len() > 1 || self.header.section_header_offset != 0 {
            let mut table = Vec::new();
            for (index, section) in self.sections.iter().enumerate() {
                let mut written = section.clone();
                if index == 0 {
                    if self.sections.len() >= section_index::RESERVED_LOW as usize {
                        written.size = self.sections.len() as u64;
                    }
                    if self.header.section_name_table >= section_index::RESERVED_LOW {
                        written.link = self.header.section_name_table;
                    }
                    if self.program_headers.len() >= 0xffff {
                        written.info = self.program_headers.len() as u32;
                    }
                } else if section.occupies_file() {
                    written.size = section.data.len() as u64;
                }
                encode_section_header(encoding, &written, &mut table);
            }
            let at = self.header.section_header_offset as usize;
            out[at..at + table.len()].copy_from_slice(&table);
        }
        out
    }

    fn encode_header(&self) -> Vec<u8> {
        let header = &self.header;
        let encoding = header.encoding;
        let mut out = Vec::with_capacity(encoding.header_size());
        out.extend_from_slice(b"\x7fELF");
        out.push(if encoding.is_64() { 2 } else { 1 });
        out.push(match encoding.endian {
            Endian::Little => 1,
            Endian::Big => 2,
        });
        out.push(1);
        out.push(header.operating_system);
        out.push(header.abi_version);
        out.extend_from_slice(&[0; 7]);
        encoding.put_u16(&mut out, header.file_type);
        encoding.put_u16(&mut out, header.machine);
        encoding.put_u32(&mut out, header.version);
        encoding.put_word(&mut out, header.entry);
        let program_count = self.program_headers.len();
        encoding.put_word(
            &mut out,
            if program_count == 0 {
                0
            } else {
                header.program_header_offset
            },
        );
        let has_sections = self.sections.len() > 1 || header.section_header_offset != 0;
        encoding.put_word(
            &mut out,
            if has_sections {
                header.section_header_offset
            } else {
                0
            },
        );
        encoding.put_u32(&mut out, header.flags);
        encoding.put_u16(&mut out, encoding.header_size() as u16);
        encoding.put_u16(
            &mut out,
            if program_count == 0 && header.program_header_entry_size == 0 {
                0
            } else {
                encoding.program_header_size() as u16
            },
        );
        encoding.put_u16(&mut out, program_count.min(0xffff) as u16);
        encoding.put_u16(
            &mut out,
            if has_sections || header.section_header_entry_size != 0 {
                encoding.section_header_size() as u16
            } else {
                0
            },
        );
        let count = if !has_sections || self.sections.len() >= section_index::RESERVED_LOW as usize
        {
            0
        } else {
            self.sections.len() as u16
        };
        encoding.put_u16(&mut out, count);
        let names = if header.section_name_table >= section_index::RESERVED_LOW {
            section_index::EXTENDED as u16
        } else {
            header.section_name_table as u16
        };
        encoding.put_u16(&mut out, names);
        out
    }
}

pub fn align_to(value: u64, alignment: u64) -> u64 {
    if alignment <= 1 {
        return value;
    }
    value.div_ceil(alignment) * alignment
}

fn read_section_header(bytes: &[u8], encoding: Encoding, at: usize) -> Result<Section, Diagnostic> {
    let short = || unreadable(at as u64, "a section header is truncated");
    let full = |offset: usize| encoding.read_u32(bytes, at + offset).ok_or_else(short);
    let word = |offset: usize| encoding.read_word(bytes, at + offset).ok_or_else(short);
    if encoding.is_64() {
        Ok(Section {
            name_offset: full(0)?,
            kind: full(4)?,
            flags: word(8)?,
            address: word(16)?,
            offset: word(24)?,
            size: word(32)?,
            link: full(40)?,
            info: full(44)?,
            alignment: word(48)?,
            entry_size: word(56)?,
            ..Section::default()
        })
    } else {
        Ok(Section {
            name_offset: full(0)?,
            kind: full(4)?,
            flags: word(8)?,
            address: word(12)?,
            offset: word(16)?,
            size: word(20)?,
            link: full(24)?,
            info: full(28)?,
            alignment: word(32)?,
            entry_size: word(36)?,
            ..Section::default()
        })
    }
}

fn read_program_header(
    bytes: &[u8],
    encoding: Encoding,
    at: usize,
) -> Result<ProgramHeader, Diagnostic> {
    let short = || unreadable(at as u64, "a program header is truncated");
    let full = |offset: usize| encoding.read_u32(bytes, at + offset).ok_or_else(short);
    let word = |offset: usize| encoding.read_word(bytes, at + offset).ok_or_else(short);
    if encoding.is_64() {
        Ok(ProgramHeader {
            kind: full(0)?,
            flags: full(4)?,
            offset: word(8)?,
            virtual_address: word(16)?,
            physical_address: word(24)?,
            file_size: word(32)?,
            memory_size: word(40)?,
            alignment: word(48)?,
        })
    } else {
        Ok(ProgramHeader {
            kind: full(0)?,
            offset: word(4)?,
            virtual_address: word(8)?,
            physical_address: word(12)?,
            file_size: word(16)?,
            memory_size: word(20)?,
            flags: full(24)?,
            alignment: word(28)?,
        })
    }
}

pub fn encode_section_header(encoding: Encoding, section: &Section, out: &mut Vec<u8>) {
    encoding.put_u32(out, section.name_offset);
    encoding.put_u32(out, section.kind);
    encoding.put_word(out, section.flags);
    encoding.put_word(out, section.address);
    encoding.put_word(out, section.offset);
    encoding.put_word(out, section.size);
    encoding.put_u32(out, section.link);
    encoding.put_u32(out, section.info);
    encoding.put_word(out, section.alignment);
    encoding.put_word(out, section.entry_size);
}

pub fn encode_program_header(encoding: Encoding, program: &ProgramHeader, out: &mut Vec<u8>) {
    encoding.put_u32(out, program.kind);
    if encoding.is_64() {
        encoding.put_u32(out, program.flags);
    }
    encoding.put_word(out, program.offset);
    encoding.put_word(out, program.virtual_address);
    encoding.put_word(out, program.physical_address);
    encoding.put_word(out, program.file_size);
    encoding.put_word(out, program.memory_size);
    if !encoding.is_64() {
        encoding.put_u32(out, program.flags);
    }
    encoding.put_word(out, program.alignment);
}

pub fn decode_symbols(
    encoding: Encoding,
    data: &[u8],
    strings: &[u8],
    extended: Option<&[u8]>,
) -> Result<Vec<Symbol>, (usize, &'static str)> {
    let size = encoding.symbol_size();
    if !data.len().is_multiple_of(size) {
        return Err((
            0,
            "the symbol table size is not a multiple of its entry size",
        ));
    }
    let mut symbols = Vec::with_capacity(data.len() / size);
    for index in 0..data.len() / size {
        let at = index * size;
        let short = (at, "a symbol is truncated");
        let full = |offset: usize| encoding.read_u32(data, at + offset).ok_or(short);
        let half = |offset: usize| encoding.read_u16(data, at + offset).ok_or(short);
        let word = |offset: usize| encoding.read_word(data, at + offset).ok_or(short);
        let (name_offset, value, symbol_size, info, other, section) = if encoding.is_64() {
            (
                full(0)?,
                word(8)?,
                word(16)?,
                data[at + 4],
                data[at + 5],
                half(6)?,
            )
        } else {
            (
                full(0)?,
                word(4)?,
                word(8)?,
                data[at + 12],
                data[at + 13],
                half(14)?,
            )
        };
        let mut section = u32::from(section);
        if section == section_index::EXTENDED {
            section = extended
                .and_then(|table| encoding.read_u32(table, index * 4))
                .ok_or((at, "an extended section index is missing"))?;
        }
        let name = string_at(strings, name_offset).ok_or((at, "invalid symbol name offset"))?;
        symbols.push(Symbol {
            name,
            name_offset,
            value,
            size: symbol_size,
            binding: info >> 4,
            kind: info & 0xf,
            other,
            section,
        });
    }
    Ok(symbols)
}

pub fn encode_symbol(encoding: Encoding, symbol: &Symbol, out: &mut Vec<u8>) {
    let section = if symbol.section >= section_index::RESERVED_LOW
        && symbol.section != section_index::ABSOLUTE
        && symbol.section != section_index::COMMON
        && symbol.section != section_index::EXTENDED
    {
        section_index::EXTENDED as u16
    } else {
        symbol.section as u16
    };
    encoding.put_u32(out, symbol.name_offset);
    if encoding.is_64() {
        out.push(symbol.info());
        out.push(symbol.other);
        encoding.put_u16(out, section);
        encoding.put_u64(out, symbol.value);
        encoding.put_u64(out, symbol.size);
    } else {
        encoding.put_u32(out, symbol.value as u32);
        encoding.put_u32(out, symbol.size as u32);
        out.push(symbol.info());
        out.push(symbol.other);
        encoding.put_u16(out, section);
    }
}

pub fn decode_relocations(
    encoding: Encoding,
    data: &[u8],
    with_addend: bool,
) -> Result<Vec<Relocation>, (usize, &'static str)> {
    let size = encoding.relocation_size(with_addend);
    if !data.len().is_multiple_of(size) {
        return Err((
            0,
            "the relocation section size is not a multiple of its entry size",
        ));
    }
    let mut relocations = Vec::with_capacity(data.len() / size);
    for index in 0..data.len() / size {
        let at = index * size;
        let short = (at, "a relocation is truncated");
        let word_size = encoding.word_size();
        let offset = encoding.read_word(data, at).ok_or(short)?;
        let information = encoding.read_word(data, at + word_size).ok_or(short)?;
        let addend = if with_addend {
            let raw = encoding.read_word(data, at + 2 * word_size).ok_or(short)?;
            if encoding.is_64() {
                raw as i64
            } else {
                i64::from(raw as u32 as i32)
            }
        } else {
            0
        };
        let (symbol, kind) = if encoding.is_64() {
            ((information >> 32) as u32, information as u32)
        } else {
            ((information >> 8) as u32, (information & 0xff) as u32)
        };
        relocations.push(Relocation {
            offset,
            kind,
            symbol,
            addend,
        });
    }
    Ok(relocations)
}

pub fn encode_relocation(
    encoding: Encoding,
    relocation: &Relocation,
    with_addend: bool,
    out: &mut Vec<u8>,
) {
    encoding.put_word(out, relocation.offset);
    let information = if encoding.is_64() {
        (u64::from(relocation.symbol) << 32) | u64::from(relocation.kind)
    } else {
        (u64::from(relocation.symbol) << 8) | u64::from(relocation.kind & 0xff)
    };
    encoding.put_word(out, information);
    if with_addend {
        encoding.put_word(out, relocation.addend as u64);
    }
}

pub fn decode_dynamic(encoding: Encoding, data: &[u8]) -> Vec<DynamicEntry> {
    let size = encoding.dynamic_entry_size();
    let mut entries = Vec::new();
    for index in 0..data.len() / size {
        let at = index * size;
        let (Some(tag), Some(value)) = (
            encoding.read_word(data, at),
            encoding.read_word(data, at + size / 2),
        ) else {
            break;
        };
        let tag = if encoding.is_64() {
            tag as i64
        } else {
            i64::from(tag as u32 as i32)
        };
        entries.push(DynamicEntry { tag, value });
        if tag == dynamic_tag::NULL {
            break;
        }
    }
    entries
}

pub fn encode_dynamic(encoding: Encoding, entries: &[DynamicEntry], out: &mut Vec<u8>) {
    for entry in entries {
        encoding.put_word(out, entry.tag as u64);
        encoding.put_word(out, entry.value);
    }
}

pub fn decode_notes(
    encoding: Encoding,
    data: &[u8],
    alignment: u64,
) -> Result<Vec<Note>, &'static str> {
    let alignment = if alignment == 8 { 8 } else { 4 };
    let mut notes = Vec::new();
    let mut at = 0usize;
    while at < data.len() {
        let short = "a note is truncated";
        let name_size = encoding.read_u32(data, at).ok_or(short)? as usize;
        let description_size = encoding.read_u32(data, at + 4).ok_or(short)? as usize;
        let kind = encoding.read_u32(data, at + 8).ok_or(short)?;
        let name_start = at + 12;
        let name_end = name_start.checked_add(name_size).ok_or(short)?;
        let description_start = align_to(name_end as u64, alignment) as usize;
        let description_end = description_start
            .checked_add(description_size)
            .ok_or(short)?;
        if description_end > data.len() || name_end > data.len() {
            return Err(short);
        }
        let mut name = data[name_start..name_end].to_vec();
        while name.last() == Some(&0) {
            name.pop();
        }
        notes.push(Note {
            name,
            kind,
            description: data[description_start..description_end].to_vec(),
        });
        at = align_to(description_end as u64, alignment) as usize;
    }
    Ok(notes)
}

pub fn encode_note(encoding: Encoding, note: &Note, alignment: u64, out: &mut Vec<u8>) {
    let alignment = if alignment == 8 { 8 } else { 4 };
    let name_size = if note.name.is_empty() {
        0
    } else {
        note.name.len() + 1
    };
    encoding.put_u32(out, name_size as u32);
    encoding.put_u32(out, note.description.len() as u32);
    encoding.put_u32(out, note.kind);
    let start = out.len();
    out.extend_from_slice(&note.name);
    if name_size > 0 {
        out.push(0);
    }
    while !(out.len() - start).is_multiple_of(alignment as usize) {
        out.push(0);
    }
    let start = out.len();
    out.extend_from_slice(&note.description);
    while !(out.len() - start).is_multiple_of(alignment as usize) {
        out.push(0);
    }
}

pub fn decode_version_definitions(
    encoding: Encoding,
    data: &[u8],
    strings: &[u8],
) -> Result<Vec<VersionDefinition>, &'static str> {
    let short = "a version definition is truncated";
    let mut definitions = Vec::new();
    let mut at = 0usize;
    loop {
        if at + 20 > data.len() {
            return Err(short);
        }
        let flags = encoding.read_u16(data, at + 2).ok_or(short)?;
        let index = encoding.read_u16(data, at + 4).ok_or(short)?;
        let count = encoding.read_u16(data, at + 6).ok_or(short)?;
        let hash = encoding.read_u32(data, at + 8).ok_or(short)?;
        let auxiliary = encoding.read_u32(data, at + 12).ok_or(short)? as usize;
        let next = encoding.read_u32(data, at + 16).ok_or(short)? as usize;
        let mut names = Vec::new();
        let mut cursor = at + auxiliary;
        for _ in 0..count {
            let name = encoding.read_u32(data, cursor).ok_or(short)?;
            names.push(string_at(strings, name).ok_or(short)?);
            let step = encoding.read_u32(data, cursor + 4).ok_or(short)? as usize;
            if step == 0 {
                break;
            }
            cursor += step;
        }
        definitions.push(VersionDefinition {
            flags,
            index,
            hash,
            names,
        });
        if next == 0 {
            break;
        }
        at += next;
    }
    Ok(definitions)
}

pub fn decode_version_needs(
    encoding: Encoding,
    data: &[u8],
    strings: &[u8],
) -> Result<Vec<VersionNeed>, &'static str> {
    let short = "a version need is truncated";
    let mut needs = Vec::new();
    let mut at = 0usize;
    loop {
        if at + 16 > data.len() {
            return Err(short);
        }
        let count = encoding.read_u16(data, at + 2).ok_or(short)?;
        let file = encoding.read_u32(data, at + 4).ok_or(short)?;
        let auxiliary = encoding.read_u32(data, at + 8).ok_or(short)? as usize;
        let next = encoding.read_u32(data, at + 12).ok_or(short)? as usize;
        let mut entries = Vec::new();
        let mut cursor = at + auxiliary;
        for _ in 0..count {
            let hash = encoding.read_u32(data, cursor).ok_or(short)?;
            let flags = encoding.read_u16(data, cursor + 4).ok_or(short)?;
            let other = encoding.read_u16(data, cursor + 6).ok_or(short)?;
            let name = encoding.read_u32(data, cursor + 8).ok_or(short)?;
            entries.push(VersionNeedAuxiliary {
                hash,
                flags,
                other,
                name: string_at(strings, name).ok_or(short)?,
            });
            let step = encoding.read_u32(data, cursor + 12).ok_or(short)? as usize;
            if step == 0 {
                break;
            }
            cursor += step;
        }
        needs.push(VersionNeed {
            file: string_at(strings, file).ok_or(short)?,
            entries,
        });
        if next == 0 {
            break;
        }
        at += next;
    }
    Ok(needs)
}

pub fn encode_version_definitions(
    encoding: Encoding,
    definitions: &[(VersionDefinition, Vec<u32>)],
    out: &mut Vec<u8>,
) {
    for (position, (definition, names)) in definitions.iter().enumerate() {
        encoding.put_u16(out, 1);
        encoding.put_u16(out, definition.flags);
        encoding.put_u16(out, definition.index);
        encoding.put_u16(out, names.len() as u16);
        encoding.put_u32(out, definition.hash);
        encoding.put_u32(out, 20);
        let next = if position + 1 == definitions.len() {
            0
        } else {
            20 + 8 * names.len() as u32
        };
        encoding.put_u32(out, next);
        for (offset_index, name) in names.iter().enumerate() {
            encoding.put_u32(out, *name);
            encoding.put_u32(
                out,
                if offset_index + 1 == names.len() {
                    0
                } else {
                    8
                },
            );
        }
    }
}

pub fn encode_version_needs(
    encoding: Encoding,
    needs: &[(u32, Vec<(VersionNeedAuxiliary, u32)>)],
    out: &mut Vec<u8>,
) {
    for (position, (file, entries)) in needs.iter().enumerate() {
        encoding.put_u16(out, 1);
        encoding.put_u16(out, entries.len() as u16);
        encoding.put_u32(out, *file);
        encoding.put_u32(out, 16);
        let next = if position + 1 == needs.len() {
            0
        } else {
            16 + 16 * entries.len() as u32
        };
        encoding.put_u32(out, next);
        for (entry_index, (entry, name)) in entries.iter().enumerate() {
            encoding.put_u32(out, entry.hash);
            encoding.put_u16(out, entry.flags);
            encoding.put_u16(out, entry.other);
            encoding.put_u32(out, *name);
            encoding.put_u32(
                out,
                if entry_index + 1 == entries.len() {
                    0
                } else {
                    16
                },
            );
        }
    }
}

pub fn system_five_hash(name: &[u8]) -> u32 {
    let mut hash: u32 = 0;
    for &byte in name {
        hash = (hash << 4).wrapping_add(u32::from(byte));
        let high = hash & 0xf000_0000;
        if high != 0 {
            hash ^= high >> 24;
        }
        hash &= !high;
    }
    hash
}

pub fn gnu_hash(name: &[u8]) -> u32 {
    let mut hash: u32 = 5381;
    for &byte in name {
        hash = hash.wrapping_mul(33).wrapping_add(u32::from(byte));
    }
    hash
}

pub fn system_five_hash_table(encoding: Encoding, names: &[&[u8]], bucket_count: usize) -> Vec<u8> {
    let buckets = bucket_count.max(1);
    let mut bucket_heads = vec![0u32; buckets];
    let mut chains = vec![0u32; names.len()];
    for (index, name) in names.iter().enumerate().skip(1) {
        let slot = system_five_hash(name) as usize % buckets;
        chains[index] = bucket_heads[slot];
        bucket_heads[slot] = index as u32;
    }
    let mut out = Vec::new();
    encoding.put_u32(&mut out, buckets as u32);
    encoding.put_u32(&mut out, names.len() as u32);
    for head in bucket_heads {
        encoding.put_u32(&mut out, head);
    }
    for link in chains {
        encoding.put_u32(&mut out, link);
    }
    out
}

pub fn gnu_hash_mask_words(encoding: Encoding, symbol_count: usize) -> usize {
    if symbol_count == 0 {
        return 1;
    }
    let quotient = (symbol_count * 12) / (encoding.word_size() * 8);
    let mut words = 1usize;
    while words <= quotient {
        words <<= 1;
    }
    words
}

pub fn gnu_hash_table(
    encoding: Encoding,
    symbol_offset: usize,
    hashes: &[u32],
    bucket_count: usize,
) -> Vec<u8> {
    let shift: u32 = 26;
    let bucket_count = bucket_count.max(1);
    let word_bits = encoding.word_size() as u32 * 8;
    let mask_words = gnu_hash_mask_words(encoding, hashes.len());
    let mut out = Vec::new();
    encoding.put_u32(&mut out, bucket_count as u32);
    encoding.put_u32(&mut out, symbol_offset as u32);
    encoding.put_u32(&mut out, mask_words as u32);
    encoding.put_u32(&mut out, shift);
    let mut bloom = vec![0u64; mask_words];
    for hash in hashes {
        let word = (*hash / word_bits) as usize & (mask_words - 1);
        bloom[word] |= 1u64 << (hash % word_bits);
        bloom[word] |= 1u64 << ((hash >> shift) % word_bits);
    }
    for word in bloom {
        encoding.put_word(&mut out, word);
    }
    let mut buckets = vec![0u32; bucket_count];
    for (position, hash) in hashes.iter().enumerate() {
        let bucket = *hash as usize % bucket_count;
        if buckets[bucket] == 0 {
            buckets[bucket] = (symbol_offset + position) as u32;
        }
    }
    for bucket in buckets {
        encoding.put_u32(&mut out, bucket);
    }
    for (position, hash) in hashes.iter().enumerate() {
        let bucket = *hash as usize % bucket_count;
        let last =
            position + 1 == hashes.len() || hashes[position + 1] as usize % bucket_count != bucket;
        let value = if last { hash | 1 } else { hash & !1 };
        encoding.put_u32(&mut out, value);
    }
    out
}

#[derive(Clone, Debug, Default)]
pub struct StringTable {
    bytes: Vec<u8>,
    known: BTreeMap<Vec<u8>, u32>,
}

impl StringTable {
    pub fn new() -> StringTable {
        StringTable {
            bytes: vec![0],
            known: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, text: &str) -> u32 {
        self.add_bytes(text.as_bytes())
    }

    pub fn add_bytes(&mut self, text: &[u8]) -> u32 {
        if text.is_empty() {
            return 0;
        }
        if let Some(found) = self.known.get(text) {
            return *found;
        }
        let offset = self.bytes.len() as u32;
        self.bytes.extend_from_slice(text);
        self.bytes.push(0);
        self.known.insert(text.to_vec(), offset);
        offset
    }

    pub fn add_unique(&mut self, text: &str) -> u32 {
        if text.is_empty() {
            return 0;
        }
        let offset = self.bytes.len() as u32;
        self.bytes.extend_from_slice(text.as_bytes());
        self.bytes.push(0);
        self.known.entry(text.as_bytes().to_vec()).or_insert(offset);
        offset
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.len() <= 1
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

pub fn machine_name(machine: u16) -> &'static str {
    match machine {
        machine::X86 => "i386",
        machine::X86_64 => "x86-64",
        machine::ARM => "ARM",
        machine::AARCH64 => "AArch64",
        _ => "unknown",
    }
}

pub fn segment_permissions(flags: u64) -> u32 {
    let mut permissions = segment_flag::READ;
    if flags & section_flag::WRITE != 0 {
        permissions |= segment_flag::WRITE;
    }
    if flags & section_flag::EXECUTE != 0 {
        permissions |= segment_flag::EXECUTE;
    }
    permissions
}

struct InflateBits<'a> {
    data: &'a [u8],
    at: usize,
    buffer: u64,
    count: u32,
}

impl InflateBits<'_> {
    fn need(&mut self, bits: u32) -> Result<(), &'static str> {
        while self.count < bits {
            let byte = *self.data.get(self.at).ok_or("Z_DATA_ERROR")?;
            self.at += 1;
            self.buffer |= u64::from(byte) << self.count;
            self.count += 8;
        }
        Ok(())
    }

    fn take(&mut self, bits: u32) -> Result<u32, &'static str> {
        if bits == 0 {
            return Ok(0);
        }
        self.need(bits)?;
        let value = (self.buffer & ((1u64 << bits) - 1)) as u32;
        self.buffer >>= bits;
        self.count -= bits;
        Ok(value)
    }

    fn align(&mut self) {
        let drop = self.count % 8;
        self.buffer >>= drop;
        self.count -= drop;
    }
}

struct HuffmanTable {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl HuffmanTable {
    fn build(lengths: &[u8]) -> Result<HuffmanTable, &'static str> {
        let mut counts = [0u16; 16];
        for length in lengths {
            counts[usize::from(*length)] += 1;
        }
        counts[0] = 0;
        let mut left = 1i32;
        for count in counts.iter().skip(1) {
            left <<= 1;
            left -= i32::from(*count);
            if left < 0 {
                return Err("Z_DATA_ERROR");
            }
        }
        let mut offsets = [0u16; 16];
        for length in 1..15 {
            offsets[length + 1] = offsets[length] + counts[length];
        }
        let mut symbols = vec![0u16; lengths.len()];
        for (symbol, length) in lengths.iter().enumerate() {
            if *length != 0 {
                symbols[usize::from(offsets[usize::from(*length)])] = symbol as u16;
                offsets[usize::from(*length)] += 1;
            }
        }
        Ok(HuffmanTable { counts, symbols })
    }

    fn decode(&self, bits: &mut InflateBits) -> Result<u16, &'static str> {
        let mut code = 0i32;
        let mut first = 0i32;
        let mut index = 0i32;
        for length in 1..16 {
            code |= bits.take(1)? as i32;
            let count = i32::from(self.counts[length]);
            if code - count < first {
                return self
                    .symbols
                    .get((index + (code - first)) as usize)
                    .copied()
                    .ok_or("Z_DATA_ERROR");
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        Err("Z_DATA_ERROR")
    }
}

const LENGTH_BASES: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_BASES: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

fn inflate_block(
    bits: &mut InflateBits,
    out: &mut Vec<u8>,
    literals: &HuffmanTable,
    distances: &HuffmanTable,
    limit: usize,
) -> Result<(), &'static str> {
    loop {
        let symbol = literals.decode(bits)?;
        if symbol < 256 {
            if out.len() >= limit {
                return Err("Z_BUF_ERROR");
            }
            out.push(symbol as u8);
            continue;
        }
        if symbol == 256 {
            return Ok(());
        }
        let index = usize::from(symbol - 257);
        if index >= LENGTH_BASES.len() {
            return Err("Z_DATA_ERROR");
        }
        let length =
            usize::from(LENGTH_BASES[index]) + bits.take(u32::from(LENGTH_EXTRA[index]))? as usize;
        let code = usize::from(distances.decode(bits)?);
        if code >= DISTANCE_BASES.len() {
            return Err("Z_DATA_ERROR");
        }
        let distance = usize::from(DISTANCE_BASES[code])
            + bits.take(u32::from(DISTANCE_EXTRA[code]))? as usize;
        if distance > out.len() {
            return Err("Z_DATA_ERROR");
        }
        if out.len() + length > limit {
            return Err("Z_BUF_ERROR");
        }
        let start = out.len() - distance;
        for offset in 0..length {
            let byte = out[start + offset];
            out.push(byte);
        }
    }
}

pub fn inflate_zlib(data: &[u8], limit: usize) -> Result<Vec<u8>, &'static str> {
    if data.len() < 2 {
        return Err("Z_BUF_ERROR");
    }
    let method = data[0];
    if method & 0x0f != 8
        || (u16::from(data[0]) << 8 | u16::from(data[1])) % 31 != 0
        || data[1] & 0x20 != 0
    {
        return Err("Z_DATA_ERROR");
    }
    let mut bits = InflateBits {
        data,
        at: 2,
        buffer: 0,
        count: 0,
    };
    let mut out: Vec<u8> = Vec::with_capacity(limit.min(1 << 26));
    loop {
        let last = bits.take(1)?;
        match bits.take(2)? {
            0 => {
                bits.align();
                let length = bits.take(16)?;
                let complement = bits.take(16)?;
                if length != !complement & 0xffff {
                    return Err("Z_DATA_ERROR");
                }
                for _ in 0..length {
                    if out.len() >= limit {
                        return Err("Z_BUF_ERROR");
                    }
                    out.push(bits.take(8)? as u8);
                }
            }
            1 => {
                let mut lengths = [0u8; 288];
                for (symbol, length) in lengths.iter_mut().enumerate() {
                    *length = match symbol {
                        0..=143 => 8,
                        144..=255 => 9,
                        256..=279 => 7,
                        _ => 8,
                    };
                }
                let literals = HuffmanTable::build(&lengths)?;
                let distances = HuffmanTable::build(&[5u8; 30])?;
                inflate_block(&mut bits, &mut out, &literals, &distances, limit)?;
            }
            2 => {
                let literal_count = bits.take(5)? as usize + 257;
                let distance_count = bits.take(5)? as usize + 1;
                let code_count = bits.take(4)? as usize + 4;
                let mut code_lengths = [0u8; 19];
                for position in CODE_LENGTH_ORDER.iter().take(code_count) {
                    code_lengths[*position] = bits.take(3)? as u8;
                }
                let codes = HuffmanTable::build(&code_lengths)?;
                let mut lengths: Vec<u8> = Vec::with_capacity(literal_count + distance_count);
                while lengths.len() < literal_count + distance_count {
                    let symbol = codes.decode(&mut bits)?;
                    match symbol {
                        0..=15 => lengths.push(symbol as u8),
                        16 => {
                            let previous = *lengths.last().ok_or("Z_DATA_ERROR")?;
                            let count = 3 + bits.take(2)? as usize;
                            lengths.extend(std::iter::repeat_n(previous, count));
                        }
                        17 => {
                            let count = 3 + bits.take(3)? as usize;
                            lengths.extend(std::iter::repeat_n(0, count));
                        }
                        _ => {
                            let count = 11 + bits.take(7)? as usize;
                            lengths.extend(std::iter::repeat_n(0, count));
                        }
                    }
                }
                if lengths.len() != literal_count + distance_count || lengths[256] == 0 {
                    return Err("Z_DATA_ERROR");
                }
                let literals = HuffmanTable::build(&lengths[..literal_count])?;
                let distances = HuffmanTable::build(&lengths[literal_count..])?;
                inflate_block(&mut bits, &mut out, &literals, &distances, limit)?;
            }
            _ => return Err("Z_DATA_ERROR"),
        }
        if last == 1 {
            break;
        }
    }
    bits.align();
    if let (Ok(high), Ok(low)) = (bits.take(16), bits.take(16)) {
        let stored = ((high & 0xff) << 24) | ((high >> 8) << 16) | ((low & 0xff) << 8) | (low >> 8);
        let mut first = 1u32;
        let mut second = 0u32;
        for byte in &out {
            first = (first + u32::from(*byte)) % 65521;
            second = (second + first) % 65521;
        }
        if stored != (second << 16) | first {
            return Err("Z_DATA_ERROR");
        }
    }
    Ok(out)
}

pub fn adler_checksum(data: &[u8]) -> u32 {
    let mut first = 1u32;
    let mut second = 0u32;
    for chunk in data.chunks(5552) {
        for byte in chunk {
            first += u32::from(*byte);
            second += first;
        }
        first %= 65521;
        second %= 65521;
    }
    (second << 16) | first
}

const DEFLATE_WINDOW: usize = 32768;
const DEFLATE_WINDOW_MASK: usize = DEFLATE_WINDOW - 1;
const DEFLATE_HASH_MASK: usize = 32767;
const DEFLATE_MINIMUM_MATCH: usize = 3;
const DEFLATE_MAXIMUM_MATCH: usize = 258;
const DEFLATE_MINIMUM_LOOKAHEAD: usize = DEFLATE_MAXIMUM_MATCH + DEFLATE_MINIMUM_MATCH + 1;
const DEFLATE_MAXIMUM_DISTANCE: usize = DEFLATE_WINDOW - DEFLATE_MINIMUM_LOOKAHEAD;
const DEFLATE_SYMBOL_END: usize = (16384 - 1) * 3;
const DEFLATE_TOO_FAR: usize = 4096;
const DEFLATE_LITERAL_CODES: usize = 286;
const DEFLATE_DISTANCE_CODES: usize = 30;
const DEFLATE_BIT_LENGTH_CODES: usize = 19;
const DEFLATE_HEAP_SIZE: usize = 2 * DEFLATE_LITERAL_CODES + 1;
const DEFLATE_BIT_LENGTH_EXTRA: [u8; 19] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 7];
const DEFLATE_LEVELS: [(usize, usize, usize, usize); 10] = [
    (0, 0, 0, 0),
    (4, 4, 8, 4),
    (4, 5, 16, 8),
    (4, 6, 32, 32),
    (4, 4, 16, 16),
    (8, 16, 32, 32),
    (8, 16, 128, 128),
    (8, 32, 128, 256),
    (32, 128, 258, 1024),
    (32, 258, 258, 4096),
];

struct DeflateTree {
    frequency: Vec<u16>,
    length: Vec<u16>,
    maximum_code: i32,
}

impl DeflateTree {
    fn new(size: usize) -> DeflateTree {
        DeflateTree {
            frequency: vec![0; size],
            length: vec![0; size],
            maximum_code: 0,
        }
    }
}

#[derive(Clone, Copy)]
enum DeflateTreeKind {
    Literal,
    Distance,
    BitLength,
}

struct Deflater<'a> {
    input: &'a [u8],
    input_position: usize,
    window: Vec<u8>,
    previous: Vec<u16>,
    head: Vec<u16>,
    hash: usize,
    start: usize,
    block_start: isize,
    lookahead: usize,
    insert: usize,
    match_length: usize,
    match_start: usize,
    previous_length: usize,
    previous_match: usize,
    match_available: bool,
    high_water: usize,
    good_match: usize,
    lazy_match: usize,
    nice_match: usize,
    chain_length: usize,
    symbols: Vec<u8>,
    trees: [DeflateTree; 3],
    heap: Vec<usize>,
    heap_length: usize,
    heap_maximum: usize,
    depth: Vec<u8>,
    bit_counts: [u16; 16],
    optimal_length: u64,
    static_length: u64,
    output: Vec<u8>,
    bit_buffer: u32,
    bit_valid: u32,
    length_codes: [u8; 256],
    distance_codes: [u8; 512],
    length_bases: [u16; 29],
    distance_bases: [u16; 30],
    static_literal_codes: [u16; 288],
    static_literal_lengths: [u16; 288],
}

fn reverse_bits(mut code: u32, mut length: u32) -> u16 {
    let mut result = 0u32;
    loop {
        result |= code & 1;
        code >>= 1;
        result <<= 1;
        length -= 1;
        if length == 0 {
            break;
        }
    }
    (result >> 1) as u16
}

fn generate_codes(codes: &mut [u16], lengths: &[u16], maximum_code: usize, counts: &[u16; 16]) {
    let mut next = [0u16; 16];
    let mut code = 0u32;
    for bits in 1..16 {
        code = (code + u32::from(counts[bits - 1])) << 1;
        next[bits] = code as u16;
    }
    for symbol in 0..=maximum_code {
        let length = usize::from(lengths[symbol]);
        if length == 0 {
            continue;
        }
        codes[symbol] = reverse_bits(u32::from(next[length]), length as u32);
        next[length] = next[length].wrapping_add(1);
    }
}

impl<'a> Deflater<'a> {
    fn new(input: &'a [u8], level: usize) -> Deflater<'a> {
        let (good_match, lazy_match, nice_match, chain_length) = DEFLATE_LEVELS[level];
        let mut length_codes = [0u8; 256];
        let mut length_bases = [0u16; 29];
        let mut length = 0usize;
        for code in 0..28 {
            length_bases[code] = length as u16;
            for _ in 0..(1usize << LENGTH_EXTRA[code]) {
                length_codes[length] = code as u8;
                length += 1;
            }
        }
        length_codes[255] = 28;
        let mut distance_codes = [0u8; 512];
        let mut distance_bases = [0u16; 30];
        let mut distance = 0usize;
        for code in 0..16 {
            distance_bases[code] = distance as u16;
            for _ in 0..(1usize << DISTANCE_EXTRA[code]) {
                distance_codes[distance] = code as u8;
                distance += 1;
            }
        }
        distance >>= 7;
        for code in 16..30 {
            distance_bases[code] = (distance << 7) as u16;
            for _ in 0..(1usize << (DISTANCE_EXTRA[code] - 7)) {
                distance_codes[256 + distance] = code as u8;
                distance += 1;
            }
        }
        let mut static_literal_lengths = [0u16; 288];
        let mut counts = [0u16; 16];
        for (symbol, slot) in static_literal_lengths.iter_mut().enumerate() {
            *slot = match symbol {
                0..=143 => 8,
                144..=255 => 9,
                256..=279 => 7,
                _ => 8,
            };
            counts[usize::from(*slot)] += 1;
        }
        let mut static_literal_codes = [0u16; 288];
        generate_codes(
            &mut static_literal_codes,
            &static_literal_lengths,
            287,
            &counts,
        );
        let mut deflater = Deflater {
            input,
            input_position: 0,
            window: vec![0; 2 * DEFLATE_WINDOW],
            previous: vec![0; DEFLATE_WINDOW],
            head: vec![0; DEFLATE_HASH_MASK + 1],
            hash: 0,
            start: 0,
            block_start: 0,
            lookahead: 0,
            insert: 0,
            match_length: DEFLATE_MINIMUM_MATCH - 1,
            match_start: 0,
            previous_length: DEFLATE_MINIMUM_MATCH - 1,
            previous_match: 0,
            match_available: false,
            high_water: 0,
            good_match,
            lazy_match,
            nice_match,
            chain_length,
            symbols: Vec::with_capacity(DEFLATE_SYMBOL_END),
            trees: [
                DeflateTree::new(DEFLATE_HEAP_SIZE),
                DeflateTree::new(2 * DEFLATE_DISTANCE_CODES + 1),
                DeflateTree::new(2 * DEFLATE_BIT_LENGTH_CODES + 1),
            ],
            heap: vec![0; DEFLATE_HEAP_SIZE],
            heap_length: 0,
            heap_maximum: 0,
            depth: vec![0; DEFLATE_HEAP_SIZE],
            bit_counts: [0; 16],
            optimal_length: 0,
            static_length: 0,
            output: Vec::new(),
            bit_buffer: 0,
            bit_valid: 0,
            length_codes,
            distance_codes,
            length_bases,
            distance_bases,
            static_literal_codes,
            static_literal_lengths,
        };
        deflater.initialize_block();
        deflater
    }

    fn initialize_block(&mut self) {
        let sizes = [
            DEFLATE_LITERAL_CODES,
            DEFLATE_DISTANCE_CODES,
            DEFLATE_BIT_LENGTH_CODES,
        ];
        for (tree, size) in self.trees.iter_mut().zip(sizes) {
            tree.frequency[..size].fill(0);
        }
        self.trees[0].frequency[256] = 1;
        self.optimal_length = 0;
        self.static_length = 0;
        self.symbols.clear();
    }

    fn update_hash(&mut self, byte: u8) {
        self.hash = ((self.hash << 5) ^ usize::from(byte)) & DEFLATE_HASH_MASK;
    }

    fn insert_string(&mut self, position: usize) -> usize {
        self.update_hash(self.window[position + DEFLATE_MINIMUM_MATCH - 1]);
        let head = self.head[self.hash];
        self.previous[position & DEFLATE_WINDOW_MASK] = head;
        self.head[self.hash] = position as u16;
        usize::from(head)
    }

    fn slide_hash(&mut self) {
        for entry in self.head.iter_mut().chain(self.previous.iter_mut()) {
            let value = usize::from(*entry);
            *entry = if value >= DEFLATE_WINDOW {
                (value - DEFLATE_WINDOW) as u16
            } else {
                0
            };
        }
    }

    fn fill_window(&mut self) {
        let window_size = 2 * DEFLATE_WINDOW;
        loop {
            let mut more = window_size - self.lookahead - self.start;
            if self.start >= DEFLATE_WINDOW + DEFLATE_MAXIMUM_DISTANCE {
                self.window
                    .copy_within(DEFLATE_WINDOW..DEFLATE_WINDOW + DEFLATE_WINDOW - more, 0);
                self.match_start = self.match_start.wrapping_sub(DEFLATE_WINDOW);
                self.start -= DEFLATE_WINDOW;
                self.block_start -= DEFLATE_WINDOW as isize;
                if self.insert > self.start {
                    self.insert = self.start;
                }
                self.slide_hash();
                more += DEFLATE_WINDOW;
            }
            let remaining = self.input.len() - self.input_position;
            if remaining == 0 {
                break;
            }
            let count = remaining.min(more);
            let at = self.start + self.lookahead;
            self.window[at..at + count]
                .copy_from_slice(&self.input[self.input_position..self.input_position + count]);
            self.input_position += count;
            self.lookahead += count;
            if self.lookahead + self.insert >= DEFLATE_MINIMUM_MATCH {
                let mut position = self.start - self.insert;
                self.hash = usize::from(self.window[position]);
                self.update_hash(self.window[position + 1]);
                while self.insert != 0 {
                    self.update_hash(self.window[position + DEFLATE_MINIMUM_MATCH - 1]);
                    self.previous[position & DEFLATE_WINDOW_MASK] = self.head[self.hash];
                    self.head[self.hash] = position as u16;
                    position += 1;
                    self.insert -= 1;
                    if self.lookahead + self.insert < DEFLATE_MINIMUM_MATCH {
                        break;
                    }
                }
            }
            if !(self.lookahead < DEFLATE_MINIMUM_LOOKAHEAD
                && self.input_position < self.input.len())
            {
                break;
            }
        }
        if self.high_water < window_size {
            let current = self.start + self.lookahead;
            if self.high_water < current {
                let initial = (window_size - current).min(DEFLATE_MAXIMUM_MATCH);
                self.window[current..current + initial].fill(0);
                self.high_water = current + initial;
            } else if self.high_water < current + DEFLATE_MAXIMUM_MATCH {
                let initial = (current + DEFLATE_MAXIMUM_MATCH - self.high_water)
                    .min(window_size - self.high_water);
                self.window[self.high_water..self.high_water + initial].fill(0);
                self.high_water += initial;
            }
        }
    }

    fn longest_match(&mut self, mut current: usize) -> usize {
        let mut chain = self.chain_length;
        let scan = self.start;
        let mut best = self.previous_length;
        let mut nice = self.nice_match;
        let limit = self.start.saturating_sub(DEFLATE_MAXIMUM_DISTANCE);
        let end = scan + DEFLATE_MAXIMUM_MATCH;
        let mut end_previous = self.window[scan + best - 1];
        let mut end_byte = self.window[scan + best];
        if self.previous_length >= self.good_match {
            chain >>= 2;
        }
        if nice > self.lookahead {
            nice = self.lookahead;
        }
        loop {
            let candidate = current;
            if self.window[candidate + best] == end_byte
                && self.window[candidate + best - 1] == end_previous
                && self.window[candidate] == self.window[scan]
                && self.window[candidate + 1] == self.window[scan + 1]
            {
                let mut offset = 2usize;
                let length = loop {
                    let mut mismatch = false;
                    for _ in 0..8 {
                        offset += 1;
                        if self.window[scan + offset] != self.window[candidate + offset] {
                            mismatch = true;
                            break;
                        }
                    }
                    if mismatch || scan + offset >= end {
                        break offset;
                    }
                };
                if length > best {
                    self.match_start = candidate;
                    best = length;
                    if length >= nice {
                        break;
                    }
                    end_previous = self.window[scan + best - 1];
                    end_byte = self.window[scan + best];
                }
            }
            current = usize::from(self.previous[current & DEFLATE_WINDOW_MASK]);
            if current <= limit {
                break;
            }
            chain -= 1;
            if chain == 0 {
                break;
            }
        }
        best.min(self.lookahead)
    }

    fn tally(&mut self, distance: usize, length_or_literal: usize) -> bool {
        self.symbols.push(distance as u8);
        self.symbols.push((distance >> 8) as u8);
        self.symbols.push(length_or_literal as u8);
        if distance == 0 {
            self.trees[0].frequency[length_or_literal] += 1;
        } else {
            let code = usize::from(self.length_codes[length_or_literal]);
            self.trees[0].frequency[code + 257] += 1;
            let code = self.distance_code(distance - 1);
            self.trees[1].frequency[code] += 1;
        }
        self.symbols.len() == DEFLATE_SYMBOL_END
    }

    fn distance_code(&self, distance: usize) -> usize {
        if distance < 256 {
            usize::from(self.distance_codes[distance])
        } else {
            usize::from(self.distance_codes[256 + (distance >> 7)])
        }
    }

    fn flush_block(&mut self, last: bool) {
        let stored = if self.block_start >= 0 {
            Some(self.block_start as usize)
        } else {
            None
        };
        let length = (self.start as isize - self.block_start) as usize;
        self.finish_block(stored, length, last);
        self.block_start = self.start as isize;
    }

    fn deflate_fast(&mut self, finish: bool) {
        loop {
            if self.lookahead < DEFLATE_MINIMUM_LOOKAHEAD {
                self.fill_window();
                if self.lookahead == 0 {
                    break;
                }
            }
            let mut head = 0usize;
            if self.lookahead >= DEFLATE_MINIMUM_MATCH {
                head = self.insert_string(self.start);
            }
            if head != 0 && self.start - head <= DEFLATE_MAXIMUM_DISTANCE {
                self.match_length = self.longest_match(head);
            }
            let flush;
            if self.match_length >= DEFLATE_MINIMUM_MATCH {
                flush = self.tally(
                    self.start - self.match_start,
                    self.match_length - DEFLATE_MINIMUM_MATCH,
                );
                self.lookahead -= self.match_length;
                if self.match_length <= self.lazy_match && self.lookahead >= DEFLATE_MINIMUM_MATCH {
                    self.match_length -= 1;
                    loop {
                        self.start += 1;
                        self.insert_string(self.start);
                        self.match_length -= 1;
                        if self.match_length == 0 {
                            break;
                        }
                    }
                    self.start += 1;
                } else {
                    self.start += self.match_length;
                    self.match_length = 0;
                    self.hash = usize::from(self.window[self.start]);
                    self.update_hash(self.window[self.start + 1]);
                }
            } else {
                flush = self.tally(0, usize::from(self.window[self.start]));
                self.lookahead -= 1;
                self.start += 1;
            }
            if flush {
                self.flush_block(false);
            }
        }
        self.insert = self.start.min(DEFLATE_MINIMUM_MATCH - 1);
        if finish {
            self.flush_block(true);
        } else if !self.symbols.is_empty() {
            self.flush_block(false);
        }
    }

    fn deflate_slow(&mut self, finish: bool) {
        loop {
            if self.lookahead < DEFLATE_MINIMUM_LOOKAHEAD {
                self.fill_window();
                if self.lookahead == 0 {
                    break;
                }
            }
            let mut head = 0usize;
            if self.lookahead >= DEFLATE_MINIMUM_MATCH {
                head = self.insert_string(self.start);
            }
            self.previous_length = self.match_length;
            self.previous_match = self.match_start;
            self.match_length = DEFLATE_MINIMUM_MATCH - 1;
            if head != 0
                && self.previous_length < self.lazy_match
                && self.start - head <= DEFLATE_MAXIMUM_DISTANCE
            {
                self.match_length = self.longest_match(head);
                if self.match_length == DEFLATE_MINIMUM_MATCH
                    && self.start - self.match_start > DEFLATE_TOO_FAR
                {
                    self.match_length = DEFLATE_MINIMUM_MATCH - 1;
                }
            }
            if self.previous_length >= DEFLATE_MINIMUM_MATCH
                && self.match_length <= self.previous_length
            {
                let maximum_insert = self.start + self.lookahead - DEFLATE_MINIMUM_MATCH;
                let flush = self.tally(
                    self.start - 1 - self.previous_match,
                    self.previous_length - DEFLATE_MINIMUM_MATCH,
                );
                self.lookahead -= self.previous_length - 1;
                self.previous_length -= 2;
                loop {
                    self.start += 1;
                    if self.start <= maximum_insert {
                        self.insert_string(self.start);
                    }
                    self.previous_length -= 1;
                    if self.previous_length == 0 {
                        break;
                    }
                }
                self.match_available = false;
                self.match_length = DEFLATE_MINIMUM_MATCH - 1;
                self.start += 1;
                if flush {
                    self.flush_block(false);
                }
            } else if self.match_available {
                let flush = self.tally(0, usize::from(self.window[self.start - 1]));
                if flush {
                    self.flush_block(false);
                }
                self.start += 1;
                self.lookahead -= 1;
            } else {
                self.match_available = true;
                self.start += 1;
                self.lookahead -= 1;
            }
        }
        if self.match_available {
            self.tally(0, usize::from(self.window[self.start - 1]));
            self.match_available = false;
        }
        self.insert = self.start.min(DEFLATE_MINIMUM_MATCH - 1);
        if finish {
            self.flush_block(true);
        } else if !self.symbols.is_empty() {
            self.flush_block(false);
        }
    }

    fn send_bits(&mut self, value: u32, length: u32) {
        if self.bit_valid > 16 - length {
            self.bit_buffer = (self.bit_buffer | (value << self.bit_valid)) & 0xffff;
            self.output.push(self.bit_buffer as u8);
            self.output.push((self.bit_buffer >> 8) as u8);
            self.bit_buffer = (value & 0xffff) >> (16 - self.bit_valid);
            self.bit_valid = self.bit_valid + length - 16;
        } else {
            self.bit_buffer = (self.bit_buffer | (value << self.bit_valid)) & 0xffff;
            self.bit_valid += length;
        }
    }

    fn send_code(&mut self, kind: usize, symbol: usize) {
        let code = u32::from(self.trees[kind].frequency[symbol]);
        let length = u32::from(self.trees[kind].length[symbol]);
        self.send_bits(code, length);
    }

    fn window_up(&mut self) {
        if self.bit_valid > 8 {
            self.output.push(self.bit_buffer as u8);
            self.output.push((self.bit_buffer >> 8) as u8);
        } else if self.bit_valid > 0 {
            self.output.push(self.bit_buffer as u8);
        }
        self.bit_buffer = 0;
        self.bit_valid = 0;
    }

    fn stored_block(&mut self, stored: Option<usize>, length: usize, last: bool) {
        self.send_bits(u32::from(last), 3);
        self.window_up();
        self.output.push(length as u8);
        self.output.push((length >> 8) as u8);
        self.output.push(!length as u8);
        self.output.push((!length >> 8) as u8);
        if let Some(at) = stored {
            self.output.extend_from_slice(&self.window[at..at + length]);
        }
    }

    fn smaller(&self, kind: usize, first: usize, second: usize) -> bool {
        let tree = &self.trees[kind];
        tree.frequency[first] < tree.frequency[second]
            || (tree.frequency[first] == tree.frequency[second]
                && self.depth[first] <= self.depth[second])
    }

    fn sift_down(&mut self, kind: usize, mut node: usize) {
        let value = self.heap[node];
        let mut child = node << 1;
        while child <= self.heap_length {
            if child < self.heap_length
                && self.smaller(kind, self.heap[child + 1], self.heap[child])
            {
                child += 1;
            }
            if self.smaller(kind, value, self.heap[child]) {
                break;
            }
            self.heap[node] = self.heap[child];
            node = child;
            child <<= 1;
        }
        self.heap[node] = value;
    }

    fn tree_parameters(kind: DeflateTreeKind) -> (usize, usize, usize, &'static [u8]) {
        match kind {
            DeflateTreeKind::Literal => (DEFLATE_LITERAL_CODES, 257, 15, &LENGTH_EXTRA),
            DeflateTreeKind::Distance => (DEFLATE_DISTANCE_CODES, 0, 15, &DISTANCE_EXTRA),
            DeflateTreeKind::BitLength => {
                (DEFLATE_BIT_LENGTH_CODES, 0, 7, &DEFLATE_BIT_LENGTH_EXTRA)
            }
        }
    }

    fn static_tree_length(&self, kind: DeflateTreeKind, symbol: usize) -> Option<u64> {
        match kind {
            DeflateTreeKind::Literal => Some(u64::from(self.static_literal_lengths[symbol])),
            DeflateTreeKind::Distance => Some(5),
            DeflateTreeKind::BitLength => None,
        }
    }

    fn generate_bit_lengths(&mut self, kind: DeflateTreeKind) {
        let index = kind as usize;
        let (_, base, maximum_length, extra) = Deflater::tree_parameters(kind);
        let maximum_code = self.trees[index].maximum_code;
        let mut overflow = 0i32;
        self.bit_counts = [0; 16];
        let root = self.heap[self.heap_maximum];
        self.trees[index].length[root] = 0;
        let mut position = self.heap_maximum + 1;
        while position < DEFLATE_HEAP_SIZE {
            let node = self.heap[position];
            position += 1;
            let parent = usize::from(self.trees[index].length[node]);
            let mut bits = usize::from(self.trees[index].length[parent]) + 1;
            if bits > maximum_length {
                bits = maximum_length;
                overflow += 1;
            }
            self.trees[index].length[node] = bits as u16;
            if node as i32 > maximum_code {
                continue;
            }
            self.bit_counts[bits] += 1;
            let extra_bits = if node >= base {
                u64::from(extra[node - base])
            } else {
                0
            };
            let frequency = u64::from(self.trees[index].frequency[node]);
            self.optimal_length = self
                .optimal_length
                .wrapping_add(frequency * (bits as u64 + extra_bits));
            if let Some(length) = self.static_tree_length(kind, node) {
                self.static_length = self
                    .static_length
                    .wrapping_add(frequency * (length + extra_bits));
            }
        }
        if overflow == 0 {
            return;
        }
        loop {
            let mut bits = maximum_length - 1;
            while self.bit_counts[bits] == 0 {
                bits -= 1;
            }
            self.bit_counts[bits] -= 1;
            self.bit_counts[bits + 1] += 2;
            self.bit_counts[maximum_length] -= 1;
            overflow -= 2;
            if overflow <= 0 {
                break;
            }
        }
        let mut position = DEFLATE_HEAP_SIZE;
        for bits in (1..=maximum_length).rev() {
            let mut count = self.bit_counts[bits];
            while count != 0 {
                position -= 1;
                let node = self.heap[position];
                if node as i32 > maximum_code {
                    continue;
                }
                let current = u64::from(self.trees[index].length[node]);
                if current != bits as u64 {
                    let frequency = u64::from(self.trees[index].frequency[node]);
                    self.optimal_length = self
                        .optimal_length
                        .wrapping_add((bits as u64).wrapping_sub(current).wrapping_mul(frequency));
                    self.trees[index].length[node] = bits as u16;
                }
                count -= 1;
            }
        }
    }

    fn build_tree(&mut self, kind: DeflateTreeKind) {
        let index = kind as usize;
        let (elements, _, _, _) = Deflater::tree_parameters(kind);
        self.heap_length = 0;
        self.heap_maximum = DEFLATE_HEAP_SIZE;
        let mut maximum_code = -1i32;
        for symbol in 0..elements {
            if self.trees[index].frequency[symbol] != 0 {
                self.heap_length += 1;
                self.heap[self.heap_length] = symbol;
                maximum_code = symbol as i32;
                self.depth[symbol] = 0;
            } else {
                self.trees[index].length[symbol] = 0;
            }
        }
        while self.heap_length < 2 {
            let node = if maximum_code < 2 {
                maximum_code += 1;
                maximum_code as usize
            } else {
                0
            };
            self.heap_length += 1;
            self.heap[self.heap_length] = node;
            self.trees[index].frequency[node] = 1;
            self.depth[node] = 0;
            self.optimal_length = self.optimal_length.wrapping_sub(1);
            if let Some(length) = self.static_tree_length(kind, node) {
                self.static_length = self.static_length.wrapping_sub(length);
            }
        }
        self.trees[index].maximum_code = maximum_code;
        for node in (1..=self.heap_length / 2).rev() {
            self.sift_down(index, node);
        }
        let mut next = elements;
        loop {
            let least = self.heap[1];
            self.heap[1] = self.heap[self.heap_length];
            self.heap_length -= 1;
            self.sift_down(index, 1);
            let second = self.heap[1];
            self.heap_maximum -= 1;
            self.heap[self.heap_maximum] = least;
            self.heap_maximum -= 1;
            self.heap[self.heap_maximum] = second;
            let tree = &mut self.trees[index];
            tree.frequency[next] = tree.frequency[least].wrapping_add(tree.frequency[second]);
            self.depth[next] = self.depth[least].max(self.depth[second]).wrapping_add(1);
            tree.length[least] = next as u16;
            tree.length[second] = next as u16;
            self.heap[1] = next;
            next += 1;
            self.sift_down(index, 1);
            if self.heap_length < 2 {
                break;
            }
        }
        self.heap_maximum -= 1;
        self.heap[self.heap_maximum] = self.heap[1];
        self.generate_bit_lengths(kind);
        let counts = self.bit_counts;
        let tree = &mut self.trees[index];
        generate_codes(
            &mut tree.frequency,
            &tree.length,
            maximum_code as usize,
            &counts,
        );
    }

    fn scan_tree(&mut self, index: usize) {
        let maximum_code = self.trees[index].maximum_code as usize;
        let mut previous = u16::MAX as i32 + 1;
        let mut next = self.trees[index].length[0];
        let mut count = 0usize;
        let (mut maximum_count, mut minimum_count) = if next == 0 { (138, 3) } else { (7, 4) };
        self.trees[index].length[maximum_code + 1] = 0xffff;
        for symbol in 0..=maximum_code {
            let current = next;
            next = self.trees[index].length[symbol + 1];
            count += 1;
            if count < maximum_count && current == next {
                continue;
            } else if count < minimum_count {
                self.trees[2].frequency[usize::from(current)] += count as u16;
            } else if current != 0 {
                if i32::from(current) != previous {
                    self.trees[2].frequency[usize::from(current)] += 1;
                }
                self.trees[2].frequency[16] += 1;
            } else if count <= 10 {
                self.trees[2].frequency[17] += 1;
            } else {
                self.trees[2].frequency[18] += 1;
            }
            count = 0;
            previous = i32::from(current);
            (maximum_count, minimum_count) = if next == 0 {
                (138, 3)
            } else if current == next {
                (6, 3)
            } else {
                (7, 4)
            };
        }
    }

    fn send_tree(&mut self, index: usize, maximum_code: usize) {
        let mut previous = u16::MAX as i32 + 1;
        let mut next = self.trees[index].length[0];
        let mut count = 0usize;
        let (mut maximum_count, mut minimum_count) = if next == 0 { (138, 3) } else { (7, 4) };
        for symbol in 0..=maximum_code {
            let current = next;
            next = self.trees[index].length[symbol + 1];
            count += 1;
            if count < maximum_count && current == next {
                continue;
            } else if count < minimum_count {
                while count != 0 {
                    self.send_code(2, usize::from(current));
                    count -= 1;
                }
            } else if current != 0 {
                if i32::from(current) != previous {
                    self.send_code(2, usize::from(current));
                    count -= 1;
                }
                self.send_code(2, 16);
                self.send_bits(count as u32 - 3, 2);
            } else if count <= 10 {
                self.send_code(2, 17);
                self.send_bits(count as u32 - 3, 3);
            } else {
                self.send_code(2, 18);
                self.send_bits(count as u32 - 11, 7);
            }
            count = 0;
            previous = i32::from(current);
            (maximum_count, minimum_count) = if next == 0 {
                (138, 3)
            } else if current == next {
                (6, 3)
            } else {
                (7, 4)
            };
        }
    }

    fn build_bit_length_tree(&mut self) -> usize {
        self.scan_tree(0);
        self.scan_tree(1);
        self.build_tree(DeflateTreeKind::BitLength);
        let mut maximum_index = DEFLATE_BIT_LENGTH_CODES - 1;
        while maximum_index >= 3 {
            if self.trees[2].length[CODE_LENGTH_ORDER[maximum_index]] != 0 {
                break;
            }
            maximum_index -= 1;
        }
        self.optimal_length = self
            .optimal_length
            .wrapping_add(3 * (maximum_index as u64 + 1) + 5 + 5 + 4);
        maximum_index
    }

    fn compress_block(&mut self, dynamic: bool) {
        let symbols = std::mem::take(&mut self.symbols);
        for entry in symbols.chunks_exact(3) {
            let mut distance = usize::from(entry[0]) | usize::from(entry[1]) << 8;
            let value = usize::from(entry[2]);
            if distance == 0 {
                self.send_literal(dynamic, value);
                continue;
            }
            let code = usize::from(self.length_codes[value]);
            self.send_literal(dynamic, code + 257);
            let extra = u32::from(LENGTH_EXTRA[code]);
            if extra != 0 {
                self.send_bits((value - usize::from(self.length_bases[code])) as u32, extra);
            }
            distance -= 1;
            let code = self.distance_code(distance);
            if dynamic {
                self.send_code(1, code);
            } else {
                self.send_bits(u32::from(reverse_bits(code as u32, 5)), 5);
            }
            let extra = u32::from(DISTANCE_EXTRA[code]);
            if extra != 0 {
                self.send_bits(
                    (distance - usize::from(self.distance_bases[code])) as u32,
                    extra,
                );
            }
        }
        self.symbols = symbols;
        self.send_literal(dynamic, 256);
    }

    fn send_literal(&mut self, dynamic: bool, symbol: usize) {
        if dynamic {
            self.send_code(0, symbol);
        } else {
            let code = u32::from(self.static_literal_codes[symbol]);
            let length = u32::from(self.static_literal_lengths[symbol]);
            self.send_bits(code, length);
        }
    }

    fn finish_block(&mut self, stored: Option<usize>, length: usize, last: bool) {
        self.build_tree(DeflateTreeKind::Literal);
        self.build_tree(DeflateTreeKind::Distance);
        let maximum_index = self.build_bit_length_tree();
        let mut optimal_bytes = (self.optimal_length.wrapping_add(3 + 7)) >> 3;
        let static_bytes = (self.static_length.wrapping_add(3 + 7)) >> 3;
        if static_bytes <= optimal_bytes {
            optimal_bytes = static_bytes;
        }
        if length as u64 + 4 <= optimal_bytes && stored.is_some() {
            self.stored_block(stored, length, last);
        } else if static_bytes == optimal_bytes {
            self.send_bits(2 + u32::from(last), 3);
            self.compress_block(false);
        } else {
            self.send_bits(4 + u32::from(last), 3);
            let literal_codes = self.trees[0].maximum_code as u32 + 1;
            let distance_codes = self.trees[1].maximum_code as u32 + 1;
            self.send_bits(literal_codes - 257, 5);
            self.send_bits(distance_codes - 1, 5);
            self.send_bits(maximum_index as u32 + 1 - 4, 4);
            for order in CODE_LENGTH_ORDER.iter().take(maximum_index + 1) {
                let length = u32::from(self.trees[2].length[*order]);
                self.send_bits(length, 3);
            }
            self.send_tree(0, literal_codes as usize - 1);
            self.send_tree(1, distance_codes as usize - 1);
            self.compress_block(true);
        }
        self.initialize_block();
        if last {
            self.window_up();
        }
    }
}

pub fn deflate_raw(data: &[u8], level: usize, finish: bool) -> Vec<u8> {
    let mut deflater = Deflater::new(data, level.clamp(1, 9));
    if level.clamp(1, 9) <= 3 {
        deflater.deflate_fast(finish);
    } else {
        deflater.deflate_slow(finish);
    }
    if !finish {
        deflater.stored_block(None, 0, false);
    }
    deflater.output
}
