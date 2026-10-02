use crate::compilers::compiler_infrastructure::fail;
use crate::diag::Diagnostic;

pub mod machine {
    pub const UNKNOWN: u16 = 0;
    pub const I386: u16 = 0x14c;
    pub const AMD64: u16 = 0x8664;
    pub const ARM_THUMB: u16 = 0x1c4;
    pub const ARM64: u16 = 0xaa64;
    pub const ARM64_EMULATION_COMPATIBLE: u16 = 0xa641;
    pub const ARM64_HYBRID: u16 = 0xa64e;
}

pub mod file_characteristic {
    pub const RELOCATIONS_STRIPPED: u16 = 0x1;
    pub const EXECUTABLE_IMAGE: u16 = 0x2;
    pub const LINE_NUMBERS_STRIPPED: u16 = 0x4;
    pub const LOCAL_SYMBOLS_STRIPPED: u16 = 0x8;
    pub const LARGE_ADDRESS_AWARE: u16 = 0x20;
    pub const MACHINE_32_BIT: u16 = 0x100;
    pub const DEBUG_STRIPPED: u16 = 0x200;
    pub const REMOVABLE_RUN_FROM_SWAP: u16 = 0x400;
    pub const NET_RUN_FROM_SWAP: u16 = 0x800;
    pub const SYSTEM: u16 = 0x1000;
    pub const DYNAMIC_LIBRARY: u16 = 0x2000;
    pub const UNIPROCESSOR_ONLY: u16 = 0x4000;
}

pub mod section_characteristic {
    pub const TYPE_NO_PAD: u32 = 0x8;
    pub const CODE: u32 = 0x20;
    pub const INITIALIZED_DATA: u32 = 0x40;
    pub const UNINITIALIZED_DATA: u32 = 0x80;
    pub const LINK_OTHER: u32 = 0x100;
    pub const LINK_INFORMATION: u32 = 0x200;
    pub const LINK_REMOVE: u32 = 0x800;
    pub const LINK_COMDAT: u32 = 0x1000;
    pub const GLOBAL_POINTER_RELATIVE: u32 = 0x8000;
    pub const ALIGNMENT_MASK: u32 = 0x00f0_0000;
    pub const LINK_RELOCATION_OVERFLOW: u32 = 0x0100_0000;
    pub const DISCARDABLE: u32 = 0x0200_0000;
    pub const NOT_CACHED: u32 = 0x0400_0000;
    pub const NOT_PAGED: u32 = 0x0800_0000;
    pub const SHARED: u32 = 0x1000_0000;
    pub const EXECUTE: u32 = 0x2000_0000;
    pub const READ: u32 = 0x4000_0000;
    pub const WRITE: u32 = 0x8000_0000;
    pub const PERMISSIONS: u32 = 0xfe00_0000;
}

pub mod storage_class {
    pub const END_OF_FUNCTION: u8 = 0xff;
    pub const NULL: u8 = 0;
    pub const AUTOMATIC: u8 = 1;
    pub const EXTERNAL: u8 = 2;
    pub const STATIC: u8 = 3;
    pub const REGISTER: u8 = 4;
    pub const EXTERNAL_DEFINITION: u8 = 5;
    pub const LABEL: u8 = 6;
    pub const UNDEFINED_LABEL: u8 = 7;
    pub const MEMBER_OF_STRUCTURE: u8 = 8;
    pub const ARGUMENT: u8 = 9;
    pub const STRUCTURE_TAG: u8 = 10;
    pub const BLOCK: u8 = 100;
    pub const FUNCTION: u8 = 101;
    pub const END_OF_STRUCTURE: u8 = 102;
    pub const FILE: u8 = 103;
    pub const SECTION: u8 = 104;
    pub const WEAK_EXTERNAL: u8 = 105;
    pub const COMMON_LANGUAGE_TOKEN: u8 = 107;
}

pub mod section_number {
    pub const UNDEFINED: i32 = 0;
    pub const ABSOLUTE: i32 = -1;
    pub const DEBUG: i32 = -2;
}

pub mod comdat_selection {
    pub const NONE: u8 = 0;
    pub const NO_DUPLICATES: u8 = 1;
    pub const ANY: u8 = 2;
    pub const SAME_SIZE: u8 = 3;
    pub const EXACT_MATCH: u8 = 4;
    pub const ASSOCIATIVE: u8 = 5;
    pub const LARGEST: u8 = 6;
    pub const NEWEST: u8 = 7;
}

pub mod weak_search {
    pub const NO_LIBRARY: u32 = 1;
    pub const LIBRARY: u32 = 2;
    pub const ALIAS: u32 = 3;
    pub const ANTI_DEPENDENCY: u32 = 4;
}

pub mod relocation_amd64 {
    pub const ABSOLUTE: u16 = 0;
    pub const ADDRESS_64: u16 = 1;
    pub const ADDRESS_32: u16 = 2;
    pub const ADDRESS_32_IMAGE_RELATIVE: u16 = 3;
    pub const RELATIVE_32: u16 = 4;
    pub const RELATIVE_32_1: u16 = 5;
    pub const RELATIVE_32_2: u16 = 6;
    pub const RELATIVE_32_3: u16 = 7;
    pub const RELATIVE_32_4: u16 = 8;
    pub const RELATIVE_32_5: u16 = 9;
    pub const SECTION: u16 = 0xa;
    pub const SECTION_RELATIVE: u16 = 0xb;
    pub const SECTION_RELATIVE_7: u16 = 0xc;
    pub const TOKEN: u16 = 0xd;
    pub const SPAN_RELATIVE_32: u16 = 0xe;
    pub const PAIR: u16 = 0xf;
    pub const SPAN_32: u16 = 0x10;
}

pub mod relocation_i386 {
    pub const ABSOLUTE: u16 = 0;
    pub const DIRECT_16: u16 = 1;
    pub const RELATIVE_16: u16 = 2;
    pub const DIRECT_32: u16 = 6;
    pub const DIRECT_32_IMAGE_RELATIVE: u16 = 7;
    pub const SEGMENT_12: u16 = 9;
    pub const SECTION: u16 = 0xa;
    pub const SECTION_RELATIVE: u16 = 0xb;
    pub const TOKEN: u16 = 0xc;
    pub const SECTION_RELATIVE_7: u16 = 0xd;
    pub const RELATIVE_32: u16 = 0x14;
}

pub mod base_relocation {
    pub const ABSOLUTE: u8 = 0;
    pub const HIGH: u8 = 1;
    pub const LOW: u8 = 2;
    pub const HIGH_LOW: u8 = 3;
    pub const HIGH_ADJUST: u8 = 4;
    pub const ARM_MOVE_32: u8 = 5;
    pub const THUMB_MOVE_32: u8 = 7;
    pub const DIRECT_64: u8 = 10;
}

pub mod data_directory {
    pub const EXPORT: usize = 0;
    pub const IMPORT: usize = 1;
    pub const RESOURCE: usize = 2;
    pub const EXCEPTION: usize = 3;
    pub const CERTIFICATE: usize = 4;
    pub const BASE_RELOCATION: usize = 5;
    pub const DEBUG: usize = 6;
    pub const ARCHITECTURE: usize = 7;
    pub const GLOBAL_POINTER: usize = 8;
    pub const THREAD_LOCAL_STORAGE: usize = 9;
    pub const LOAD_CONFIGURATION: usize = 10;
    pub const BOUND_IMPORT: usize = 11;
    pub const IMPORT_ADDRESS_TABLE: usize = 12;
    pub const DELAY_IMPORT: usize = 13;
    pub const COMMON_LANGUAGE_RUNTIME: usize = 14;
    pub const COUNT: usize = 16;
}

pub mod subsystem {
    pub const UNKNOWN: u16 = 0;
    pub const NATIVE: u16 = 1;
    pub const WINDOWS_GRAPHICAL: u16 = 2;
    pub const WINDOWS_CONSOLE: u16 = 3;
    pub const OS2_CONSOLE: u16 = 5;
    pub const POSIX_CONSOLE: u16 = 7;
    pub const NATIVE_WINDOWS: u16 = 8;
    pub const WINDOWS_CE_GRAPHICAL: u16 = 9;
    pub const EFI_APPLICATION: u16 = 10;
    pub const EFI_BOOT_SERVICE_DRIVER: u16 = 11;
    pub const EFI_RUNTIME_DRIVER: u16 = 12;
    pub const EFI_ROM: u16 = 13;
    pub const XBOX: u16 = 14;
    pub const WINDOWS_BOOT_APPLICATION: u16 = 16;
}

pub mod dll_characteristic {
    pub const HIGH_ENTROPY_VIRTUAL_ADDRESS: u16 = 0x20;
    pub const DYNAMIC_BASE: u16 = 0x40;
    pub const FORCE_INTEGRITY: u16 = 0x80;
    pub const NX_COMPATIBLE: u16 = 0x100;
    pub const NO_ISOLATION: u16 = 0x200;
    pub const NO_STRUCTURED_EXCEPTION_HANDLING: u16 = 0x400;
    pub const NO_BIND: u16 = 0x800;
    pub const APPLICATION_CONTAINER: u16 = 0x1000;
    pub const WDM_DRIVER: u16 = 0x2000;
    pub const GUARD_CONTROL_FLOW: u16 = 0x4000;
    pub const TERMINAL_SERVER_AWARE: u16 = 0x8000;
}

pub mod import_type {
    pub const CODE: u16 = 0;
    pub const DATA: u16 = 1;
    pub const CONSTANT: u16 = 2;
}

pub mod import_name_type {
    pub const ORDINAL: u16 = 0;
    pub const NAME: u16 = 1;
    pub const NAME_NO_PREFIX: u16 = 2;
    pub const NAME_UNDECORATE: u16 = 3;
    pub const NAME_EXPORT_AS: u16 = 4;
}

pub const BIG_OBJECT_MAGIC: [u8; 16] = [
    0xc7, 0xa1, 0xba, 0xd1, 0xee, 0xba, 0xa9, 0x4b, 0xaf, 0x20, 0xfa, 0xf6, 0x6a, 0xa4, 0xdc, 0xb8,
];

pub const DOS_PROGRAM: [u8; 56] = [
    0x0e, 0x1f, 0xba, 0x0e, 0x00, 0xb4, 0x09, 0xcd, 0x21, 0xb8, 0x01, 0x4c, 0xcd, 0x21, 0x54, 0x68,
    0x69, 0x73, 0x20, 0x70, 0x72, 0x6f, 0x67, 0x72, 0x61, 0x6d, 0x20, 0x63, 0x61, 0x6e, 0x6e, 0x6f,
    0x74, 0x20, 0x62, 0x65, 0x20, 0x72, 0x75, 0x6e, 0x20, 0x69, 0x6e, 0x20, 0x44, 0x4f, 0x53, 0x20,
    0x6d, 0x6f, 0x64, 0x65, 0x2e, 0x24, 0x00, 0x00,
];

pub const DOS_HEADER_SIZE: usize = 64;
pub const COFF_HEADER_SIZE: usize = 20;
pub const SECTION_HEADER_SIZE: usize = 40;
pub const PE32_HEADER_SIZE: usize = 96;
pub const PE32_PLUS_HEADER_SIZE: usize = 112;
pub const SYMBOL_SIZE: usize = 18;
pub const BIG_SYMBOL_SIZE: usize = 20;
pub const RELOCATION_SIZE: usize = 10;

pub fn is_64_bit(machine_number: u16) -> bool {
    matches!(
        machine_number,
        machine::AMD64
            | machine::ARM64
            | machine::ARM64_EMULATION_COMPATIBLE
            | machine::ARM64_HYBRID
    )
}

pub fn machine_name(machine_number: u16) -> &'static str {
    match machine_number {
        machine::I386 => "x86",
        machine::AMD64 => "x64",
        machine::ARM_THUMB => "arm",
        machine::ARM64 => "arm64",
        machine::ARM64_EMULATION_COMPATIBLE => "arm64ec",
        machine::ARM64_HYBRID => "arm64x",
        _ => "unknown",
    }
}

pub fn section_alignment(characteristics: u32) -> u64 {
    let shift = (characteristics >> 20) & 0xf;
    if shift == 0 {
        1
    } else {
        1u64 << (shift - 1)
    }
}

pub fn alignment_characteristics(alignment: u64) -> u32 {
    if alignment <= 1 {
        return 0x0010_0000;
    }
    let shift = 63 - alignment.leading_zeros();
    (shift + 1).min(14) << 20
}

fn unreadable(offset: usize, reason: &str) -> Diagnostic {
    fail("EI200", "The object file could not be read.")
        .with_context(format!("Offset: {offset}"))
        .with_context(format!("Reason: {reason}"))
}

pub fn read_u16(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

pub fn read_u32(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

pub fn read_u64(bytes: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(bytes.get(at..at + 8)?.try_into().ok()?))
}

pub fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub fn write_u16(bytes: &mut [u8], at: usize, value: u16) {
    if let Some(slot) = bytes.get_mut(at..at + 2) {
        slot.copy_from_slice(&value.to_le_bytes());
    }
}

pub fn write_u32(bytes: &mut [u8], at: usize, value: u32) {
    if let Some(slot) = bytes.get_mut(at..at + 4) {
        slot.copy_from_slice(&value.to_le_bytes());
    }
}

pub fn write_u64(bytes: &mut [u8], at: usize, value: u64) {
    if let Some(slot) = bytes.get_mut(at..at + 8) {
        slot.copy_from_slice(&value.to_le_bytes());
    }
}

fn c_string(bytes: &[u8], at: usize) -> Option<String> {
    let rest = bytes.get(at..)?;
    let end = rest.iter().position(|byte| *byte == 0)?;
    Some(String::from_utf8_lossy(&rest[..end]).to_string())
}

fn fixed_name(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

pub fn decode_base64_offset(text: &str) -> Option<u32> {
    if text.len() > 6 {
        return None;
    }
    let mut value: u64 = 0;
    for character in text.bytes() {
        let digit = match character {
            b'A'..=b'Z' => character - b'A',
            b'a'..=b'z' => character - b'a' + 26,
            b'0'..=b'9' => character - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        value = value * 64 + u64::from(digit);
    }
    u32::try_from(value).ok()
}

pub fn encode_base64_offset(mut value: u32) -> String {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut digits = [b'A'; 6];
    for slot in digits.iter_mut().rev() {
        *slot = alphabet[(value % 64) as usize];
        value /= 64;
    }
    String::from_utf8_lossy(&digits).to_string()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoffRelocation {
    pub virtual_address: u32,
    pub symbol_index: u32,
    pub kind: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoffSection {
    pub name: String,
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub pointer_to_relocations: u32,
    pub pointer_to_line_numbers: u32,
    pub number_of_relocations: u32,
    pub number_of_line_numbers: u16,
    pub characteristics: u32,
    pub data: Vec<u8>,
    pub relocations: Vec<CoffRelocation>,
}

impl CoffSection {
    pub fn alignment(&self) -> u64 {
        section_alignment(self.characteristics)
    }

    pub fn is_uninitialized(&self) -> bool {
        self.characteristics & section_characteristic::UNINITIALIZED_DATA != 0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SectionDefinition {
    pub length: u32,
    pub relocation_count: u16,
    pub line_number_count: u16,
    pub check_sum: u32,
    pub number: u32,
    pub selection: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoffSymbol {
    pub name: String,
    pub value: u32,
    pub section_number: i32,
    pub kind: u16,
    pub storage_class: u8,
    pub auxiliary: Vec<u8>,
    pub auxiliary_count: u8,
    pub index: u32,
}

impl CoffSymbol {
    pub fn is_external(&self) -> bool {
        self.storage_class == storage_class::EXTERNAL
            || self.storage_class == storage_class::WEAK_EXTERNAL
    }

    pub fn is_undefined(&self) -> bool {
        self.section_number == section_number::UNDEFINED && self.value == 0
    }

    pub fn is_common(&self) -> bool {
        self.storage_class == storage_class::EXTERNAL
            && self.section_number == section_number::UNDEFINED
            && self.value != 0
    }

    pub fn is_function_definition(&self) -> bool {
        self.storage_class == storage_class::EXTERNAL
            && self.kind >> 4 == 2
            && self.section_number > 0
    }

    pub fn is_section_definition(&self) -> bool {
        self.storage_class == storage_class::STATIC && self.auxiliary_count > 0 && self.value == 0
    }

    pub fn section_definition(&self, big: bool) -> Option<SectionDefinition> {
        let aux = &self.auxiliary;
        if aux.len() < 18 {
            return None;
        }
        let low = read_u16(aux, 12)?;
        let high = if big { read_u16(aux, 16)? } else { 0 };
        Some(SectionDefinition {
            length: read_u32(aux, 0)?,
            relocation_count: read_u16(aux, 4)?,
            line_number_count: read_u16(aux, 6)?,
            check_sum: read_u32(aux, 8)?,
            number: u32::from(low) | (u32::from(high) << 16),
            selection: aux[14],
        })
    }

    pub fn weak_external(&self) -> Option<(u32, u32)> {
        if self.storage_class != storage_class::WEAK_EXTERNAL || self.auxiliary.len() < 8 {
            return None;
        }
        Some((read_u32(&self.auxiliary, 0)?, read_u32(&self.auxiliary, 4)?))
    }

    pub fn file_name(&self) -> Option<String> {
        if self.storage_class != storage_class::FILE {
            return None;
        }
        Some(fixed_name(&self.auxiliary))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoffObject {
    pub machine: u16,
    pub time_stamp: u32,
    pub characteristics: u16,
    pub big: bool,
    pub sections: Vec<CoffSection>,
    pub symbols: Vec<CoffSymbol>,
    pub symbol_count: u32,
    pub string_table: Vec<u8>,
}

pub fn is_big_object(bytes: &[u8]) -> bool {
    bytes.len() >= 56
        && read_u16(bytes, 0) == Some(0)
        && read_u16(bytes, 2) == Some(0xffff)
        && read_u16(bytes, 4).is_some_and(|version| version >= 2)
        && bytes[12..28] == BIG_OBJECT_MAGIC
}

pub fn is_coff_object(bytes: &[u8]) -> bool {
    if is_big_object(bytes) {
        return true;
    }
    matches!(
        read_u16(bytes, 0),
        Some(
            machine::I386
                | machine::AMD64
                | machine::ARM_THUMB
                | machine::ARM64
                | machine::ARM64_EMULATION_COMPATIBLE
                | machine::ARM64_HYBRID
        )
    ) && bytes.len() >= COFF_HEADER_SIZE
        && read_u16(bytes, 16) == Some(0)
}

pub fn is_import_object(bytes: &[u8]) -> bool {
    bytes.len() >= 20
        && read_u16(bytes, 0) == Some(0)
        && read_u16(bytes, 2) == Some(0xffff)
        && read_u16(bytes, 4) == Some(0)
}

impl CoffObject {
    pub fn parse(bytes: &[u8]) -> Result<CoffObject, Diagnostic> {
        let big = is_big_object(bytes);
        let (
            machine_number,
            time_stamp,
            section_count,
            symbol_offset,
            symbol_count,
            optional,
            characteristics,
            header_end,
        ) = if big {
            (
                read_u16(bytes, 6).unwrap_or(0),
                read_u32(bytes, 8).unwrap_or(0),
                read_u32(bytes, 44).unwrap_or(0) as usize,
                read_u32(bytes, 48).unwrap_or(0) as usize,
                read_u32(bytes, 52).unwrap_or(0) as usize,
                0usize,
                0u16,
                56usize,
            )
        } else {
            if bytes.len() < COFF_HEADER_SIZE {
                return Err(unreadable(0, "the COFF header is truncated"));
            }
            (
                read_u16(bytes, 0).unwrap_or(0),
                read_u32(bytes, 4).unwrap_or(0),
                read_u16(bytes, 2).unwrap_or(0) as usize,
                read_u32(bytes, 8).unwrap_or(0) as usize,
                read_u32(bytes, 12).unwrap_or(0) as usize,
                read_u16(bytes, 16).unwrap_or(0) as usize,
                read_u16(bytes, 18).unwrap_or(0),
                COFF_HEADER_SIZE,
            )
        };
        let symbol_size = if big { BIG_SYMBOL_SIZE } else { SYMBOL_SIZE };
        let mut string_table = Vec::new();
        if symbol_offset != 0 {
            let table_at = symbol_offset + symbol_count * symbol_size;
            if let Some(length) = read_u32(bytes, table_at) {
                let length = length as usize;
                if length >= 4 && table_at + length <= bytes.len() {
                    string_table = bytes[table_at..table_at + length].to_vec();
                } else if length > 4 {
                    return Err(unreadable(table_at, "the string table is truncated"));
                }
            }
        }
        let string_at =
            |offset: u32| -> Option<String> { c_string(&string_table, offset as usize) };
        let mut sections = Vec::with_capacity(section_count);
        let table = header_end + optional;
        for index in 0..section_count {
            let at = table + index * SECTION_HEADER_SIZE;
            let Some(header) = bytes.get(at..at + SECTION_HEADER_SIZE) else {
                return Err(unreadable(at, "the section table is truncated"));
            };
            let raw_name = fixed_name(&header[..8]);
            let name = if let Some(rest) = raw_name.strip_prefix("//") {
                decode_base64_offset(rest)
                    .and_then(string_at)
                    .ok_or_else(|| {
                        unreadable(at, "a section name points outside the string table")
                    })?
            } else if let Some(rest) = raw_name.strip_prefix('/') {
                rest.parse::<u32>()
                    .ok()
                    .and_then(string_at)
                    .ok_or_else(|| {
                        unreadable(at, "a section name points outside the string table")
                    })?
            } else {
                raw_name
            };
            let mut section = CoffSection {
                name,
                virtual_size: read_u32(header, 8).unwrap_or(0),
                virtual_address: read_u32(header, 12).unwrap_or(0),
                size_of_raw_data: read_u32(header, 16).unwrap_or(0),
                pointer_to_raw_data: read_u32(header, 20).unwrap_or(0),
                pointer_to_relocations: read_u32(header, 24).unwrap_or(0),
                pointer_to_line_numbers: read_u32(header, 28).unwrap_or(0),
                number_of_relocations: u32::from(read_u16(header, 32).unwrap_or(0)),
                number_of_line_numbers: read_u16(header, 34).unwrap_or(0),
                characteristics: read_u32(header, 36).unwrap_or(0),
                data: Vec::new(),
                relocations: Vec::new(),
            };
            if !section.is_uninitialized() && section.size_of_raw_data > 0 {
                let start = section.pointer_to_raw_data as usize;
                let end = start + section.size_of_raw_data as usize;
                section.data = bytes
                    .get(start..end)
                    .ok_or_else(|| unreadable(start, "section data runs past the end of the file"))?
                    .to_vec();
            }
            let mut relocation_at = section.pointer_to_relocations as usize;
            let mut count = section.number_of_relocations as usize;
            if section.characteristics & section_characteristic::LINK_RELOCATION_OVERFLOW != 0
                && count == 0xffff
            {
                count = read_u32(bytes, relocation_at)
                    .ok_or_else(|| unreadable(relocation_at, "the relocation count is truncated"))?
                    as usize;
                count = count.saturating_sub(1);
                relocation_at += RELOCATION_SIZE;
                section.number_of_relocations = count as u32;
            }
            for position in 0..count {
                let at = relocation_at + position * RELOCATION_SIZE;
                let Some(entry) = bytes.get(at..at + RELOCATION_SIZE) else {
                    return Err(unreadable(at, "a relocation runs past the end of the file"));
                };
                section.relocations.push(CoffRelocation {
                    virtual_address: read_u32(entry, 0).unwrap_or(0),
                    symbol_index: read_u32(entry, 4).unwrap_or(0),
                    kind: read_u16(entry, 8).unwrap_or(0),
                });
            }
            sections.push(section);
        }
        let mut symbols = Vec::new();
        let mut index = 0usize;
        while index < symbol_count {
            let at = symbol_offset + index * symbol_size;
            let Some(entry) = bytes.get(at..at + symbol_size) else {
                return Err(unreadable(at, "the symbol table is truncated"));
            };
            let name = if read_u32(entry, 0) == Some(0) {
                string_at(read_u32(entry, 4).unwrap_or(0)).ok_or_else(|| {
                    unreadable(at, "a symbol name points outside the string table")
                })?
            } else {
                fixed_name(&entry[..8])
            };
            let (section_number, kind, class, auxiliary_count) = if big {
                (
                    read_u32(entry, 12).unwrap_or(0) as i32,
                    read_u16(entry, 16).unwrap_or(0),
                    entry[18],
                    entry[19],
                )
            } else {
                (
                    i32::from(read_u16(entry, 12).unwrap_or(0) as i16),
                    read_u16(entry, 14).unwrap_or(0),
                    entry[16],
                    entry[17],
                )
            };
            let mut auxiliary = Vec::new();
            for extra in 0..auxiliary_count as usize {
                let aux_at = at + (extra + 1) * symbol_size;
                let Some(record) = bytes.get(aux_at..aux_at + symbol_size) else {
                    return Err(unreadable(
                        aux_at,
                        "an auxiliary symbol record is truncated",
                    ));
                };
                auxiliary.extend_from_slice(&record[..18.min(symbol_size)]);
                if symbol_size > 18 {
                    auxiliary.extend_from_slice(&record[18..]);
                }
            }
            symbols.push(CoffSymbol {
                name,
                value: read_u32(entry, 8).unwrap_or(0),
                section_number,
                kind,
                storage_class: class,
                auxiliary,
                auxiliary_count,
                index: index as u32,
            });
            index += 1 + auxiliary_count as usize;
        }
        Ok(CoffObject {
            machine: machine_number,
            time_stamp,
            characteristics,
            big,
            sections,
            symbols,
            symbol_count: symbol_count as u32,
            string_table,
        })
    }

    pub fn symbol_at(&self, index: u32) -> Option<&CoffSymbol> {
        let position = self.symbols.partition_point(|symbol| symbol.index < index);
        self.symbols
            .get(position)
            .filter(|symbol| symbol.index == index)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let symbol_size = if self.big {
            BIG_SYMBOL_SIZE
        } else {
            SYMBOL_SIZE
        };
        let header_size = if self.big { 56 } else { COFF_HEADER_SIZE };
        let mut strings: Vec<u8> = vec![0, 0, 0, 0];
        let mut add_string = |text: &str| -> u32 {
            let offset = strings.len() as u32;
            strings.extend_from_slice(text.as_bytes());
            strings.push(0);
            offset
        };
        let mut section_names: Vec<[u8; 8]> = Vec::new();
        for section in &self.sections {
            let mut name = [0u8; 8];
            if section.name.len() <= 8 {
                name[..section.name.len()].copy_from_slice(section.name.as_bytes());
            } else {
                let offset = add_string(&section.name);
                let text = if offset <= 9_999_999 {
                    format!("/{offset}")
                } else {
                    format!("//{}", encode_base64_offset(offset))
                };
                name[..text.len()].copy_from_slice(text.as_bytes());
            }
            section_names.push(name);
        }
        let mut symbol_names: Vec<[u8; 8]> = Vec::new();
        for symbol in &self.symbols {
            let mut name = [0u8; 8];
            if symbol.name.len() <= 8 {
                name[..symbol.name.len()].copy_from_slice(symbol.name.as_bytes());
            } else {
                let offset = add_string(&symbol.name);
                name[4..8].copy_from_slice(&offset.to_le_bytes());
            }
            symbol_names.push(name);
        }
        let length = strings.len() as u32;
        strings[..4].copy_from_slice(&length.to_le_bytes());
        let mut cursor = header_size + self.sections.len() * SECTION_HEADER_SIZE;
        let mut placements = Vec::new();
        for section in &self.sections {
            let data_at = if section.data.is_empty() { 0 } else { cursor };
            cursor += section.data.len();
            let relocation_at = if section.relocations.is_empty() {
                0
            } else {
                cursor
            };
            let overflow = section.relocations.len() >= 0xffff;
            cursor += (section.relocations.len() + usize::from(overflow)) * RELOCATION_SIZE;
            placements.push((data_at, relocation_at, overflow));
        }
        let symbol_at = cursor;
        let mut out = Vec::new();
        if self.big {
            put_u16(&mut out, 0);
            put_u16(&mut out, 0xffff);
            put_u16(&mut out, 2);
            put_u16(&mut out, self.machine);
            put_u32(&mut out, self.time_stamp);
            out.extend_from_slice(&BIG_OBJECT_MAGIC);
            out.extend_from_slice(&[0; 16]);
            put_u32(&mut out, self.sections.len() as u32);
            put_u32(
                &mut out,
                if self.symbols.is_empty() {
                    0
                } else {
                    symbol_at as u32
                },
            );
            put_u32(&mut out, self.symbol_count);
        } else {
            put_u16(&mut out, self.machine);
            put_u16(&mut out, self.sections.len() as u16);
            put_u32(&mut out, self.time_stamp);
            put_u32(
                &mut out,
                if self.symbols.is_empty() {
                    0
                } else {
                    symbol_at as u32
                },
            );
            put_u32(&mut out, self.symbol_count);
            put_u16(&mut out, 0);
            put_u16(&mut out, self.characteristics);
        }
        for ((section, name), (data_at, relocation_at, overflow)) in
            self.sections.iter().zip(&section_names).zip(&placements)
        {
            out.extend_from_slice(name);
            put_u32(&mut out, section.virtual_size);
            put_u32(&mut out, section.virtual_address);
            put_u32(
                &mut out,
                if section.is_uninitialized() {
                    section.size_of_raw_data
                } else {
                    section.data.len() as u32
                },
            );
            put_u32(&mut out, *data_at as u32);
            put_u32(&mut out, *relocation_at as u32);
            put_u32(&mut out, 0);
            put_u16(
                &mut out,
                if *overflow {
                    0xffff
                } else {
                    section.relocations.len() as u16
                },
            );
            put_u16(&mut out, 0);
            let mut characteristics = section.characteristics;
            if *overflow {
                characteristics |= section_characteristic::LINK_RELOCATION_OVERFLOW;
            }
            put_u32(&mut out, characteristics);
        }
        for (section, (_, _, overflow)) in self.sections.iter().zip(&placements) {
            out.extend_from_slice(&section.data);
            if *overflow {
                put_u32(&mut out, section.relocations.len() as u32 + 1);
                put_u32(&mut out, 0);
                put_u16(&mut out, 0);
            }
            for relocation in &section.relocations {
                put_u32(&mut out, relocation.virtual_address);
                put_u32(&mut out, relocation.symbol_index);
                put_u16(&mut out, relocation.kind);
            }
        }
        for (symbol, name) in self.symbols.iter().zip(&symbol_names) {
            out.extend_from_slice(name);
            put_u32(&mut out, symbol.value);
            if self.big {
                put_u32(&mut out, symbol.section_number as u32);
            } else {
                put_u16(&mut out, symbol.section_number as i16 as u16);
            }
            put_u16(&mut out, symbol.kind);
            out.push(symbol.storage_class);
            out.push(symbol.auxiliary_count);
            for record in symbol.auxiliary.chunks(symbol_size) {
                let mut padded = record.to_vec();
                padded.resize(symbol_size, 0);
                out.extend_from_slice(&padded);
            }
        }
        if !self.symbols.is_empty() || strings.len() > 4 {
            out.extend_from_slice(&strings);
        }
        out
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportObject {
    pub machine: u16,
    pub time_stamp: u32,
    pub size_of_data: u32,
    pub ordinal_or_hint: u16,
    pub kind: u16,
    pub name_type: u16,
    pub symbol_name: String,
    pub library_name: String,
    pub export_name: Option<String>,
}

impl ImportObject {
    pub fn parse(bytes: &[u8]) -> Result<ImportObject, Diagnostic> {
        if !is_import_object(bytes) {
            return Err(unreadable(0, "this is not an import object"));
        }
        let size = read_u32(bytes, 12).unwrap_or(0) as usize;
        if 20 + size > bytes.len() {
            return Err(unreadable(20, "the import object data is truncated"));
        }
        let information = read_u16(bytes, 18).unwrap_or(0);
        let data = &bytes[20..20 + size];
        let symbol_name = c_string(data, 0)
            .ok_or_else(|| unreadable(20, "the import symbol name is not terminated"))?;
        let library_at = symbol_name.len() + 1;
        let library_name = c_string(data, library_at).ok_or_else(|| {
            unreadable(20 + library_at, "the import library name is not terminated")
        })?;
        let export_at = library_at + library_name.len() + 1;
        let name_type = (information >> 2) & 0x7;
        let export_name = if name_type == import_name_type::NAME_EXPORT_AS {
            c_string(data, export_at)
        } else {
            None
        };
        Ok(ImportObject {
            machine: read_u16(bytes, 6).unwrap_or(0),
            time_stamp: read_u32(bytes, 8).unwrap_or(0),
            size_of_data: size as u32,
            ordinal_or_hint: read_u16(bytes, 16).unwrap_or(0),
            kind: information & 0x3,
            name_type,
            symbol_name,
            library_name,
            export_name,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.symbol_name.as_bytes());
        data.push(0);
        data.extend_from_slice(self.library_name.as_bytes());
        data.push(0);
        if let Some(export) = &self.export_name {
            data.extend_from_slice(export.as_bytes());
            data.push(0);
        }
        let mut out = Vec::new();
        put_u16(&mut out, 0);
        put_u16(&mut out, 0xffff);
        put_u16(&mut out, 0);
        put_u16(&mut out, self.machine);
        put_u32(&mut out, self.time_stamp);
        put_u32(&mut out, data.len() as u32);
        put_u16(&mut out, self.ordinal_or_hint);
        put_u16(&mut out, (self.kind & 0x3) | ((self.name_type & 0x7) << 2));
        out.extend_from_slice(&data);
        out
    }

    pub fn imported_name(&self) -> String {
        let name = self.symbol_name.as_str();
        match self.name_type {
            import_name_type::ORDINAL => String::new(),
            import_name_type::NAME => name.to_string(),
            import_name_type::NAME_EXPORT_AS => self.export_name.clone().unwrap_or_default(),
            import_name_type::NAME_NO_PREFIX => {
                let trimmed = name.strip_prefix(['?', '@', '_']).unwrap_or(name);
                trimmed.to_string()
            }
            _ => {
                let trimmed = name.strip_prefix(['?', '@', '_']).unwrap_or(name);
                trimmed.split('@').next().unwrap_or("").to_string()
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModuleExport {
    pub name: String,
    pub external_name: String,
    pub import_name: String,
    pub export_as: String,
    pub ordinal: u16,
    pub no_name: bool,
    pub data: bool,
    pub private: bool,
    pub constant: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModuleDefinition {
    pub exports: Vec<ModuleExport>,
    pub import_name: String,
    pub output_file: String,
    pub image_base: u64,
    pub stack_reserve: Option<u64>,
    pub stack_commit: Option<u64>,
    pub heap_reserve: Option<u64>,
    pub heap_commit: Option<u64>,
    pub major_image_version: u32,
    pub minor_image_version: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DefinitionToken {
    End,
    Word(String),
    Comma,
    Equal,
    EqualEqual,
}

struct DefinitionLexer<'t> {
    text: &'t str,
}

impl DefinitionLexer<'_> {
    fn lex(&mut self) -> DefinitionToken {
        loop {
            self.text = self
                .text
                .trim_matches(|character: char| character.is_ascii_whitespace());
            let Some(first) = self.text.chars().next() else {
                return DefinitionToken::End;
            };
            match first {
                '\0' => return DefinitionToken::End,
                ';' => {
                    self.text = match self.text.find('\n') {
                        Some(end) => &self.text[end..],
                        None => "",
                    };
                    continue;
                }
                '=' => {
                    self.text = &self.text[1..];
                    if let Some(rest) = self.text.strip_prefix('=') {
                        self.text = rest;
                        return DefinitionToken::EqualEqual;
                    }
                    return DefinitionToken::Equal;
                }
                ',' => {
                    self.text = &self.text[1..];
                    return DefinitionToken::Comma;
                }
                '"' => {
                    let rest = &self.text[1..];
                    let (word, after) = match rest.find('"') {
                        Some(end) => (&rest[..end], &rest[end + 1..]),
                        None => (rest, ""),
                    };
                    self.text = after;
                    return DefinitionToken::Word(word.to_string());
                }
                _ => {
                    let end = self
                        .text
                        .find(['=', ',', ';', '\r', '\n', ' ', '\t', '\x0b'])
                        .unwrap_or(self.text.len());
                    let word = self.text[..end].to_string();
                    self.text = &self.text[end..];
                    return DefinitionToken::Word(word);
                }
            }
        }
    }
}

fn keyword(token: &DefinitionToken) -> Option<&str> {
    match token {
        DefinitionToken::Word(word) => match word.as_str() {
            "BASE" | "CONSTANT" | "DATA" | "EXPORTS" | "EXPORTAS" | "HEAPSIZE" | "LIBRARY"
            | "NAME" | "NONAME" | "PRIVATE" | "STACKSIZE" | "VERSION" => Some(word.as_str()),
            _ => None,
        },
        _ => None,
    }
}

fn identifier(token: &DefinitionToken) -> Option<&str> {
    match token {
        DefinitionToken::Word(word) if keyword(token).is_none() => Some(word.as_str()),
        _ => None,
    }
}

pub fn parse_integer(text: &str) -> Option<u64> {
    if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        return u64::from_str_radix(rest, 16).ok();
    }
    if let Some(rest) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
        return u64::from_str_radix(rest, 2).ok();
    }
    if let Some(rest) = text.strip_prefix("0o") {
        return u64::from_str_radix(rest, 8).ok();
    }
    if text.len() > 1 && text.starts_with('0') {
        return u64::from_str_radix(&text[1..], 8).ok();
    }
    text.parse::<u64>().ok()
}

fn is_decorated(symbol: &str, mingw: bool) -> bool {
    symbol.starts_with('@')
        || symbol.contains("@@")
        || symbol.starts_with('?')
        || (!mingw && symbol.contains('@'))
}

pub fn parse_module_definition(
    text: &str,
    machine_number: u16,
    mingw: bool,
    add_underscores: bool,
) -> Result<ModuleDefinition, String> {
    let add_underscores = add_underscores && machine_number == machine::I386;
    let mut lexer = DefinitionLexer { text };
    let mut stack: Vec<DefinitionToken> = Vec::new();
    let mut token = DefinitionToken::End;
    let mut information = ModuleDefinition::default();
    let mut read = |stack: &mut Vec<DefinitionToken>, token: &mut DefinitionToken| {
        *token = stack.pop().unwrap_or_else(|| lexer.lex());
    };
    loop {
        read(&mut stack, &mut token);
        match keyword(&token) {
            None if token == DefinitionToken::End => break,
            Some("EXPORTS") => loop {
                read(&mut stack, &mut token);
                let Some(name) = identifier(&token).map(str::to_string) else {
                    stack.push(token.clone());
                    break;
                };
                let mut export = ModuleExport {
                    name,
                    ..ModuleExport::default()
                };
                read(&mut stack, &mut token);
                if token == DefinitionToken::Equal {
                    read(&mut stack, &mut token);
                    let Some(internal) = identifier(&token).map(str::to_string) else {
                        return Err(format!(
                            "identifier expected, but got {}",
                            match &token {
                                DefinitionToken::Word(word) => word.clone(),
                                _ => String::new(),
                            }
                        ));
                    };
                    export.external_name = export.name.clone();
                    export.name = internal;
                } else {
                    stack.push(token.clone());
                }
                if add_underscores {
                    if !is_decorated(&export.name, mingw)
                        && (export.external_name.is_empty() || !export.name.contains('.'))
                    {
                        export.name = format!("_{}", export.name);
                    }
                    if !export.external_name.is_empty()
                        && !is_decorated(&export.external_name, mingw)
                    {
                        export.external_name = format!("_{}", export.external_name);
                    }
                }
                loop {
                    read(&mut stack, &mut token);
                    if let Some(word) = identifier(&token).filter(|word| word.starts_with('@')) {
                        let word = word.to_string();
                        if word == "@" {
                            read(&mut stack, &mut token);
                            if let DefinitionToken::Word(number) = &token {
                                export.ordinal = number.parse().unwrap_or(0);
                            }
                        } else if let Ok(number) = word[1..].parse::<u16>() {
                            export.ordinal = number;
                        } else {
                            stack.push(token.clone());
                            break;
                        }
                        read(&mut stack, &mut token);
                        if keyword(&token) == Some("NONAME") {
                            export.no_name = true;
                        } else {
                            stack.push(token.clone());
                        }
                        continue;
                    }
                    match keyword(&token) {
                        Some("DATA") => {
                            export.data = true;
                            continue;
                        }
                        Some("CONSTANT") => {
                            export.constant = true;
                            continue;
                        }
                        Some("PRIVATE") => {
                            export.private = true;
                            continue;
                        }
                        _ => {}
                    }
                    if token == DefinitionToken::EqualEqual {
                        read(&mut stack, &mut token);
                        if let DefinitionToken::Word(word) = &token {
                            export.import_name = word.clone();
                        }
                        continue;
                    }
                    if keyword(&token) == Some("EXPORTAS") {
                        read(&mut stack, &mut token);
                        match &token {
                            DefinitionToken::End => {
                                return Err("unexpected end of file, EXPORTAS identifier expected"
                                    .to_string())
                            }
                            DefinitionToken::Word(word) => export.export_as = word.clone(),
                            _ => {}
                        }
                    } else {
                        stack.push(token.clone());
                    }
                    break;
                }
                information.exports.push(export);
            },
            Some(which @ ("HEAPSIZE" | "STACKSIZE")) => {
                let heap = which == "HEAPSIZE";
                read(&mut stack, &mut token);
                let reserve = identifier(&token)
                    .and_then(parse_integer)
                    .ok_or_else(|| "integer expected".to_string())?;
                let mut commit = None;
                read(&mut stack, &mut token);
                if token == DefinitionToken::Comma {
                    read(&mut stack, &mut token);
                    commit = Some(
                        identifier(&token)
                            .and_then(parse_integer)
                            .ok_or_else(|| "integer expected".to_string())?,
                    );
                } else {
                    stack.push(token.clone());
                }
                if heap {
                    information.heap_reserve = Some(reserve);
                    if commit.is_some() {
                        information.heap_commit = commit;
                    }
                } else {
                    information.stack_reserve = Some(reserve);
                    if commit.is_some() {
                        information.stack_commit = commit;
                    }
                }
            }
            Some(which @ ("LIBRARY" | "NAME")) => {
                let library = which == "LIBRARY";
                read(&mut stack, &mut token);
                let name = match identifier(&token) {
                    Some(name) => name.to_string(),
                    None => {
                        stack.push(token.clone());
                        String::new()
                    }
                };
                if !name.is_empty() {
                    read(&mut stack, &mut token);
                    if keyword(&token) == Some("BASE") {
                        read(&mut stack, &mut token);
                        if token != DefinitionToken::Equal {
                            return Err("'=' expected".to_string());
                        }
                        read(&mut stack, &mut token);
                        information.image_base = identifier(&token)
                            .and_then(parse_integer)
                            .ok_or_else(|| "integer expected".to_string())?;
                    } else {
                        stack.push(token.clone());
                        information.image_base = 0;
                    }
                }
                information.import_name = name.clone();
                if information.output_file.is_empty() {
                    information.output_file = name.clone();
                    let has_extension = name
                        .rsplit(['/', '\\'])
                        .next()
                        .is_some_and(|base| base.contains('.'));
                    if !has_extension {
                        information
                            .output_file
                            .push_str(if library { ".dll" } else { ".exe" });
                    }
                }
            }
            Some("VERSION") => {
                read(&mut stack, &mut token);
                let Some(value) = identifier(&token).map(str::to_string) else {
                    return Err("identifier expected".to_string());
                };
                let (major, minor) = value.split_once('.').unwrap_or((value.as_str(), ""));
                information.major_image_version = major
                    .parse()
                    .map_err(|_| format!("integer expected, but got {value}"))?;
                information.minor_image_version = if minor.is_empty() {
                    0
                } else {
                    minor
                        .parse()
                        .map_err(|_| format!("integer expected, but got {value}"))?
                };
            }
            _ => {
                let value = match &token {
                    DefinitionToken::Word(word) => word.clone(),
                    DefinitionToken::Comma => ",".to_string(),
                    DefinitionToken::Equal => "=".to_string(),
                    DefinitionToken::EqualEqual => "==".to_string(),
                    DefinitionToken::End => String::new(),
                };
                return Err(format!("unknown directive: {value}"));
            }
        }
    }
    Ok(information)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataDirectory {
    pub address: u32,
    pub size: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImageHeader {
    pub machine: u16,
    pub time_stamp: u32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols: u32,
    pub characteristics: u16,
    pub plus: bool,
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
    pub base_of_data: u32,
    pub image_base: u64,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub major_operating_system_version: u16,
    pub minor_operating_system_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub check_sum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: u64,
    pub size_of_stack_commit: u64,
    pub size_of_heap_reserve: u64,
    pub size_of_heap_commit: u64,
    pub loader_flags: u32,
    pub directories: Vec<DataDirectory>,
}

impl ImageHeader {
    pub fn optional_header_size(&self) -> usize {
        (if self.plus {
            PE32_PLUS_HEADER_SIZE
        } else {
            PE32_HEADER_SIZE
        }) + 8 * self.directories.len()
    }

    pub fn encode(&self, section_count: u16, out: &mut Vec<u8>) {
        put_u16(out, self.machine);
        put_u16(out, section_count);
        put_u32(out, self.time_stamp);
        put_u32(out, self.pointer_to_symbol_table);
        put_u32(out, self.number_of_symbols);
        put_u16(out, self.optional_header_size() as u16);
        put_u16(out, self.characteristics);
        put_u16(out, if self.plus { 0x20b } else { 0x10b });
        out.push(self.major_linker_version);
        out.push(self.minor_linker_version);
        put_u32(out, self.size_of_code);
        put_u32(out, self.size_of_initialized_data);
        put_u32(out, self.size_of_uninitialized_data);
        put_u32(out, self.address_of_entry_point);
        put_u32(out, self.base_of_code);
        if self.plus {
            put_u64(out, self.image_base);
        } else {
            put_u32(out, self.base_of_data);
            put_u32(out, self.image_base as u32);
        }
        put_u32(out, self.section_alignment);
        put_u32(out, self.file_alignment);
        put_u16(out, self.major_operating_system_version);
        put_u16(out, self.minor_operating_system_version);
        put_u16(out, self.major_image_version);
        put_u16(out, self.minor_image_version);
        put_u16(out, self.major_subsystem_version);
        put_u16(out, self.minor_subsystem_version);
        put_u32(out, self.win32_version_value);
        put_u32(out, self.size_of_image);
        put_u32(out, self.size_of_headers);
        put_u32(out, self.check_sum);
        put_u16(out, self.subsystem);
        put_u16(out, self.dll_characteristics);
        if self.plus {
            put_u64(out, self.size_of_stack_reserve);
            put_u64(out, self.size_of_stack_commit);
            put_u64(out, self.size_of_heap_reserve);
            put_u64(out, self.size_of_heap_commit);
        } else {
            put_u32(out, self.size_of_stack_reserve as u32);
            put_u32(out, self.size_of_stack_commit as u32);
            put_u32(out, self.size_of_heap_reserve as u32);
            put_u32(out, self.size_of_heap_commit as u32);
        }
        put_u32(out, self.loader_flags);
        put_u32(out, self.directories.len() as u32);
        for directory in &self.directories {
            put_u32(out, directory.address);
            put_u32(out, directory.size);
        }
    }
}

pub fn dos_stub() -> Vec<u8> {
    let size = DOS_HEADER_SIZE + DOS_PROGRAM.len();
    let mut out = vec![0u8; DOS_HEADER_SIZE];
    out[0] = b'M';
    out[1] = b'Z';
    write_u16(&mut out, 2, (size % 512) as u16);
    write_u16(&mut out, 4, size.div_ceil(512) as u16);
    write_u16(&mut out, 8, (DOS_HEADER_SIZE / 16) as u16);
    write_u16(&mut out, 24, DOS_HEADER_SIZE as u16);
    write_u32(&mut out, 60, size as u32);
    out.extend_from_slice(&DOS_PROGRAM);
    out
}

pub fn encode_section_header(section: &CoffSection, name: [u8; 8], out: &mut Vec<u8>) {
    out.extend_from_slice(&name);
    put_u32(out, section.virtual_size);
    put_u32(out, section.virtual_address);
    put_u32(out, section.size_of_raw_data);
    put_u32(out, section.pointer_to_raw_data);
    put_u32(out, section.pointer_to_relocations);
    put_u32(out, section.pointer_to_line_numbers);
    put_u16(out, section.number_of_relocations as u16);
    put_u16(out, section.number_of_line_numbers);
    put_u32(out, section.characteristics);
}

pub fn image_check_sum(bytes: &[u8]) -> u32 {
    let mut sum: u64 = 0;
    let mut chunks = bytes.chunks_exact(2);
    for pair in &mut chunks {
        sum += u64::from(u16::from_le_bytes([pair[0], pair[1]]));
    }
    if let [last] = chunks.remainder() {
        sum += u64::from(*last);
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    (sum + bytes.len() as u64) as u32
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PeImage {
    pub header: ImageHeader,
    pub sections: Vec<CoffSection>,
    pub header_offset: usize,
}

impl PeImage {
    pub fn parse(bytes: &[u8]) -> Result<PeImage, Diagnostic> {
        if bytes.get(..2) != Some(b"MZ") {
            return Err(unreadable(0, "the image does not start with a DOS header"));
        }
        let header_offset = read_u32(bytes, 60).unwrap_or(0) as usize;
        if bytes.get(header_offset..header_offset + 4) != Some(b"PE\0\0") {
            return Err(unreadable(header_offset, "the PE signature is missing"));
        }
        let at = header_offset + 4;
        let file = bytes
            .get(at..at + COFF_HEADER_SIZE)
            .ok_or_else(|| unreadable(at, "the COFF header is truncated"))?;
        let section_count = read_u16(file, 2).unwrap_or(0) as usize;
        let optional_size = read_u16(file, 16).unwrap_or(0) as usize;
        let optional_at = at + COFF_HEADER_SIZE;
        let optional = bytes
            .get(optional_at..optional_at + optional_size)
            .ok_or_else(|| unreadable(optional_at, "the optional header is truncated"))?;
        let plus = read_u16(optional, 0) == Some(0x20b);
        let word = |offset: usize| -> u64 {
            if plus {
                read_u64(optional, offset).unwrap_or(0)
            } else {
                u64::from(read_u32(optional, offset).unwrap_or(0))
            }
        };
        let (image_base, stack_at) = if plus { (word(24), 72) } else { (word(28), 72) };
        let step = if plus { 8 } else { 4 };
        let directories_at = stack_at + 4 * step + 8;
        let directory_count = read_u32(optional, directories_at - 4).unwrap_or(0) as usize;
        let mut directories = Vec::new();
        for index in 0..directory_count {
            let entry = directories_at + index * 8;
            directories.push(DataDirectory {
                address: read_u32(optional, entry).unwrap_or(0),
                size: read_u32(optional, entry + 4).unwrap_or(0),
            });
        }
        let header = ImageHeader {
            machine: read_u16(file, 0).unwrap_or(0),
            time_stamp: read_u32(file, 4).unwrap_or(0),
            pointer_to_symbol_table: read_u32(file, 8).unwrap_or(0),
            number_of_symbols: read_u32(file, 12).unwrap_or(0),
            characteristics: read_u16(file, 18).unwrap_or(0),
            plus,
            major_linker_version: optional.get(2).copied().unwrap_or(0),
            minor_linker_version: optional.get(3).copied().unwrap_or(0),
            size_of_code: read_u32(optional, 4).unwrap_or(0),
            size_of_initialized_data: read_u32(optional, 8).unwrap_or(0),
            size_of_uninitialized_data: read_u32(optional, 12).unwrap_or(0),
            address_of_entry_point: read_u32(optional, 16).unwrap_or(0),
            base_of_code: read_u32(optional, 20).unwrap_or(0),
            base_of_data: if plus {
                0
            } else {
                read_u32(optional, 24).unwrap_or(0)
            },
            image_base,
            section_alignment: read_u32(optional, 32).unwrap_or(0),
            file_alignment: read_u32(optional, 36).unwrap_or(0),
            major_operating_system_version: read_u16(optional, 40).unwrap_or(0),
            minor_operating_system_version: read_u16(optional, 42).unwrap_or(0),
            major_image_version: read_u16(optional, 44).unwrap_or(0),
            minor_image_version: read_u16(optional, 46).unwrap_or(0),
            major_subsystem_version: read_u16(optional, 48).unwrap_or(0),
            minor_subsystem_version: read_u16(optional, 50).unwrap_or(0),
            win32_version_value: read_u32(optional, 52).unwrap_or(0),
            size_of_image: read_u32(optional, 56).unwrap_or(0),
            size_of_headers: read_u32(optional, 60).unwrap_or(0),
            check_sum: read_u32(optional, 64).unwrap_or(0),
            subsystem: read_u16(optional, 68).unwrap_or(0),
            dll_characteristics: read_u16(optional, 70).unwrap_or(0),
            size_of_stack_reserve: word(stack_at),
            size_of_stack_commit: word(stack_at + step),
            size_of_heap_reserve: word(stack_at + 2 * step),
            size_of_heap_commit: word(stack_at + 3 * step),
            loader_flags: read_u32(optional, stack_at + 4 * step).unwrap_or(0),
            directories,
        };
        let table = optional_at + optional_size;
        let mut sections = Vec::new();
        for index in 0..section_count {
            let at = table + index * SECTION_HEADER_SIZE;
            let entry = bytes
                .get(at..at + SECTION_HEADER_SIZE)
                .ok_or_else(|| unreadable(at, "the section table is truncated"))?;
            let mut section = CoffSection {
                name: fixed_name(&entry[..8]),
                virtual_size: read_u32(entry, 8).unwrap_or(0),
                virtual_address: read_u32(entry, 12).unwrap_or(0),
                size_of_raw_data: read_u32(entry, 16).unwrap_or(0),
                pointer_to_raw_data: read_u32(entry, 20).unwrap_or(0),
                pointer_to_relocations: read_u32(entry, 24).unwrap_or(0),
                pointer_to_line_numbers: read_u32(entry, 28).unwrap_or(0),
                number_of_relocations: u32::from(read_u16(entry, 32).unwrap_or(0)),
                number_of_line_numbers: read_u16(entry, 34).unwrap_or(0),
                characteristics: read_u32(entry, 36).unwrap_or(0),
                data: Vec::new(),
                relocations: Vec::new(),
            };
            if section.size_of_raw_data > 0 {
                let start = section.pointer_to_raw_data as usize;
                let end = start + section.size_of_raw_data as usize;
                section.data = bytes
                    .get(start..end)
                    .ok_or_else(|| unreadable(start, "section data runs past the end of the file"))?
                    .to_vec();
            }
            sections.push(section);
        }
        Ok(PeImage {
            header,
            sections,
            header_offset,
        })
    }

    pub fn section_named(&self, name: &str) -> Option<&CoffSection> {
        self.sections.iter().find(|section| section.name == name)
    }

    pub fn read_at_address(&self, address: u32, length: usize) -> Option<Vec<u8>> {
        let section = self.sections.iter().find(|section| {
            address >= section.virtual_address
                && address
                    < section.virtual_address + section.virtual_size.max(section.size_of_raw_data)
        })?;
        let offset = (address - section.virtual_address) as usize;
        let mut out = vec![0u8; length];
        for (position, slot) in out.iter_mut().enumerate() {
            *slot = section.data.get(offset + position).copied().unwrap_or(0);
        }
        Some(out)
    }
}
