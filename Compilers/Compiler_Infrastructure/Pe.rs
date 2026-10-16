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

pub mod relocation_arm {
    pub const ABSOLUTE: u16 = 0;
    pub const ADDRESS_32: u16 = 1;
    pub const ADDRESS_32_IMAGE_RELATIVE: u16 = 2;
    pub const BRANCH_24: u16 = 3;
    pub const BRANCH_11: u16 = 4;
    pub const RELATIVE_32: u16 = 0xa;
    pub const SECTION: u16 = 0xe;
    pub const SECTION_RELATIVE: u16 = 0xf;
    pub const MOVE_32: u16 = 0x10;
    pub const THUMB_MOVE_32: u16 = 0x11;
    pub const THUMB_BRANCH_20: u16 = 0x12;
    pub const THUMB_BRANCH_24: u16 = 0x14;
    pub const THUMB_BRANCH_EXCHANGE_23: u16 = 0x15;
    pub const PAIR: u16 = 0x16;
}

pub mod relocation_arm64 {
    pub const ABSOLUTE: u16 = 0;
    pub const ADDRESS_32: u16 = 1;
    pub const ADDRESS_32_IMAGE_RELATIVE: u16 = 2;
    pub const BRANCH_26: u16 = 3;
    pub const PAGE_BASE_RELATIVE_21: u16 = 4;
    pub const RELATIVE_21: u16 = 5;
    pub const PAGE_OFFSET_12_ADD: u16 = 6;
    pub const PAGE_OFFSET_12_LOAD: u16 = 7;
    pub const SECTION_RELATIVE: u16 = 8;
    pub const SECTION_RELATIVE_LOW_12_ADD: u16 = 9;
    pub const SECTION_RELATIVE_HIGH_12_ADD: u16 = 0xa;
    pub const SECTION_RELATIVE_LOW_12_LOAD: u16 = 0xb;
    pub const TOKEN: u16 = 0xc;
    pub const SECTION: u16 = 0xd;
    pub const ADDRESS_64: u16 = 0xe;
    pub const BRANCH_19: u16 = 0xf;
    pub const BRANCH_14: u16 = 0x10;
    pub const RELATIVE_32: u16 = 0x11;
}

pub fn read_thumb_move(bytes: &[u8], at: usize, top: bool) -> Result<u16, &'static str> {
    let first = read_u16(bytes, at).unwrap_or(0);
    let second = read_u16(bytes, at + 2).unwrap_or(0);
    let expected = if top { 0xf2c0 } else { 0xf240 };
    if first & 0xfbf0 != expected || second & 0x8000 != 0 {
        return Err(if top { "MOVT" } else { "MOVW" });
    }
    Ok((second & 0x00ff)
        | ((second >> 4) & 0x0700)
        | ((first << 1) & 0x0800)
        | ((first & 0x000f) << 12))
}

pub fn write_thumb_move(bytes: &mut [u8], at: usize, value: u16) {
    let first = read_u16(bytes, at).unwrap_or(0);
    let second = read_u16(bytes, at + 2).unwrap_or(0);
    let first = (first & 0xfbf0) | ((value & 0x800) >> 1) | ((value >> 12) & 0xf);
    let second = (second & 0x8f00) | ((value & 0x700) << 4) | (value & 0xff);
    bytes[at..at + 2].copy_from_slice(&first.to_le_bytes());
    bytes[at + 2..at + 4].copy_from_slice(&second.to_le_bytes());
}

pub fn apply_thumb_move_32(bytes: &mut [u8], at: usize, value: u32) -> Result<(), &'static str> {
    let low = read_thumb_move(bytes, at, false);
    let high = read_thumb_move(bytes, at + 4, true);
    let immediate = u32::from(low.unwrap_or(0)) | (u32::from(high.unwrap_or(0)) << 16);
    let value = value.wrapping_add(immediate);
    write_thumb_move(bytes, at, value as u16);
    write_thumb_move(bytes, at + 4, (value >> 16) as u16);
    low.and(high).map(|_| ())
}

pub fn apply_thumb_branch_20(bytes: &mut [u8], at: usize, value: i32) -> bool {
    let fits = (-(1 << 20)..(1 << 20)).contains(&value);
    let sign = u16::from(value < 0);
    let jump_one = ((value >> 19) & 1) as u16;
    let jump_two = ((value >> 18) & 1) as u16;
    let first = read_u16(bytes, at).unwrap_or(0) | (sign << 10) | ((value >> 12) & 0x3f) as u16;
    let second = read_u16(bytes, at + 2).unwrap_or(0)
        | (jump_one << 13)
        | (jump_two << 11)
        | ((value >> 1) & 0x7ff) as u16;
    bytes[at..at + 2].copy_from_slice(&first.to_le_bytes());
    bytes[at + 2..at + 4].copy_from_slice(&second.to_le_bytes());
    fits
}

pub fn apply_thumb_branch_24(bytes: &mut [u8], at: usize, value: i32) -> bool {
    let fits = (-(1 << 24)..(1 << 24)).contains(&value);
    let sign = u16::from(value < 0);
    let jump_one = (((!value >> 23) & 1) as u16) ^ sign;
    let jump_two = (((!value >> 22) & 1) as u16) ^ sign;
    let first = read_u16(bytes, at).unwrap_or(0) | (sign << 10) | ((value >> 12) & 0x3ff) as u16;
    let second = (read_u16(bytes, at + 2).unwrap_or(0) & 0xd000)
        | (jump_one << 13)
        | (jump_two << 11)
        | ((value >> 1) & 0x7ff) as u16;
    bytes[at..at + 2].copy_from_slice(&first.to_le_bytes());
    bytes[at + 2..at + 4].copy_from_slice(&second.to_le_bytes());
    fits
}

pub fn apply_arm64_address(bytes: &mut [u8], at: usize, target: u64, place: u64, shift: u32) {
    let original = read_u32(bytes, at).unwrap_or(0);
    let raw = ((original >> 29) & 0x3) | ((original >> 3) & 0x1f_fffc);
    let immediate = ((i64::from(raw) << 43) >> 43) as u64;
    let target = target.wrapping_add(immediate);
    let delta = (target >> shift).wrapping_sub(place >> shift) as u32;
    let low = (delta & 0x3) << 29;
    let high = (delta & 0x1f_fffc) << 3;
    let mask = (0x3u32 << 29) | (0x1f_fffcu32 << 3);
    write_u32(bytes, at, (original & !mask) | low | high);
}

pub fn apply_arm64_immediate(bytes: &mut [u8], at: usize, immediate: u64, range_limit: u32) {
    let original = read_u32(bytes, at).unwrap_or(0);
    let immediate = immediate.wrapping_add(u64::from((original >> 10) & 0xfff));
    let cleared = original & !(0xfff << 10);
    write_u32(
        bytes,
        at,
        cleared | (((immediate as u32) & (0xfff >> range_limit)) << 10),
    );
}

pub fn apply_arm64_load(bytes: &mut [u8], at: usize, immediate: u64) -> bool {
    let original = read_u32(bytes, at).unwrap_or(0);
    let mut size = original >> 30;
    if original & 0x480_0000 == 0x480_0000 {
        size += 4;
    }
    let aligned = immediate & ((1u64 << size) - 1) == 0;
    apply_arm64_immediate(bytes, at, immediate >> size, size);
    aligned
}

pub fn or_u32(bytes: &mut [u8], at: usize, value: u32) {
    let original = read_u32(bytes, at).unwrap_or(0);
    write_u32(bytes, at, original | value);
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
        let appdomain_global = self.storage_class == storage_class::EXTERNAL
            && self.section_number == section_number::ABSOLUTE;
        self.auxiliary_count > 0
            && (appdomain_global || self.storage_class == storage_class::STATIC)
    }

    pub fn section_definition(&self, big: bool) -> Option<SectionDefinition> {
        let aux = &self.auxiliary;
        if aux.len() < 18 || self.storage_class != storage_class::STATIC {
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
    if bytes.len() >= COFF_HEADER_SIZE && bytes[0] == 0 && bytes[1] == 0 && !is_import_object(bytes)
    {
        return read_u16(bytes, 16) == Some(0);
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
                            .with_context(format!(
                                "Cause: getSectionName failed: #{}: invalid string table offset",
                                index + 1
                            ))
                    })?
            } else if let Some(rest) = raw_name.strip_prefix('/') {
                rest.parse::<u32>()
                    .ok()
                    .and_then(string_at)
                    .ok_or_else(|| {
                        unreadable(at, "a section name points outside the string table")
                            .with_context(format!(
                                "Cause: getSectionName failed: #{}: invalid string table offset",
                                index + 1
                            ))
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShortExport {
    pub name: String,
    pub external_name: String,
    pub symbol_name: String,
    pub import_name: String,
    pub export_as: String,
    pub ordinal: u16,
    pub no_name: bool,
    pub data: bool,
    pub private: bool,
    pub constant: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ArchiveMember {
    pub name: String,
    pub data: Vec<u8>,
}

const IMPORT_DESCRIPTOR_PREFIX: &str = "__IMPORT_DESCRIPTOR_";
const NULL_IMPORT_DESCRIPTOR: &str = "__NULL_IMPORT_DESCRIPTOR";

fn image_relative_relocation(machine_number: u16) -> u16 {
    match machine_number {
        machine::I386 => relocation_i386::DIRECT_32_IMAGE_RELATIVE,
        machine::ARM_THUMB => relocation_arm::ADDRESS_32_IMAGE_RELATIVE,
        machine::ARM64 => relocation_arm64::ADDRESS_32_IMAGE_RELATIVE,
        _ => relocation_amd64::ADDRESS_32_IMAGE_RELATIVE,
    }
}

fn apply_name_type(name_type: u16, name: &str) -> String {
    let trimmed = name.strip_prefix(['?', '@', '_']).unwrap_or(name);
    match name_type {
        import_name_type::NAME_NO_PREFIX => trimmed.to_string(),
        import_name_type::NAME_UNDECORATE => trimmed.split('@').next().unwrap_or("").to_string(),
        _ => name.to_string(),
    }
}

fn short_name_type(symbol: &str, external: &str, machine_number: u16, mingw: bool) -> u16 {
    if external.starts_with('_') && external.contains('@') && !mingw {
        return import_name_type::NAME;
    }
    if symbol != external {
        return import_name_type::NAME_UNDECORATE;
    }
    if machine_number == machine::I386 && symbol.starts_with('_') {
        return import_name_type::NAME_NO_PREFIX;
    }
    import_name_type::NAME
}

fn replace_export_name(text: &str, from: &str, to: &str) -> Option<String> {
    let (mut from, mut to) = (from, to);
    let mut position = text.find(from);
    if position.is_none() && from.starts_with('_') && to.starts_with('_') {
        from = &from[1..];
        to = &to[1..];
        position = text.find(from);
    }
    let position = position?;
    Some(format!(
        "{}{}{}",
        &text[..position],
        to,
        &text[position + from.len()..]
    ))
}

fn put_coff_header(
    out: &mut Vec<u8>,
    machine_number: u16,
    sections: u16,
    symbol_table: u32,
    symbols: u32,
    characteristics: u16,
) {
    put_u16(out, machine_number);
    put_u16(out, sections);
    put_u32(out, 0);
    put_u32(out, symbol_table);
    put_u32(out, symbols);
    put_u16(out, 0);
    put_u16(out, characteristics);
}

#[allow(clippy::too_many_arguments)]
fn put_raw_section(
    out: &mut Vec<u8>,
    name: &[u8; 8],
    size: u32,
    data: u32,
    relocations: u32,
    relocation_count: u16,
    characteristics: u32,
) {
    out.extend_from_slice(name);
    put_u32(out, 0);
    put_u32(out, 0);
    put_u32(out, size);
    put_u32(out, data);
    put_u32(out, relocations);
    put_u32(out, 0);
    put_u16(out, relocation_count);
    put_u16(out, 0);
    put_u32(out, characteristics);
}

fn put_raw_symbol(out: &mut Vec<u8>, name: [u8; 8], section: u16, class: u8, auxiliary: u8) {
    out.extend_from_slice(&name);
    put_u32(out, 0);
    put_u16(out, section);
    put_u16(out, 0);
    out.push(class);
    out.push(auxiliary);
}

fn string_offset_name(offset: u32) -> [u8; 8] {
    let mut name = [0u8; 8];
    name[4..].copy_from_slice(&offset.to_le_bytes());
    name
}

fn put_raw_strings(out: &mut Vec<u8>, strings: &[&str]) {
    let start = out.len();
    put_u32(out, 0);
    for text in strings {
        out.extend_from_slice(text.as_bytes());
        out.push(0);
    }
    let length = (out.len() - start) as u32;
    write_u32(out, start, length);
}

fn word_characteristics(machine_number: u16) -> u16 {
    if is_64_bit(machine_number) {
        0
    } else {
        file_characteristic::MACHINE_32_BIT
    }
}

fn import_descriptor_object(import_name: &str, library: &str, machine_number: u16) -> Vec<u8> {
    use section_characteristic as character;
    let descriptor_symbol = format!("{IMPORT_DESCRIPTOR_PREFIX}{library}");
    let thunk_symbol = format!("\u{7f}{library}_NULL_THUNK_DATA");
    let headers = (COFF_HEADER_SIZE + 2 * SECTION_HEADER_SIZE) as u32;
    let mut out = Vec::new();
    let symbol_table = headers + 20 + 3 * RELOCATION_SIZE as u32 + import_name.len() as u32 + 1;
    put_coff_header(
        &mut out,
        machine_number,
        2,
        symbol_table,
        7,
        word_characteristics(machine_number),
    );
    let data = character::INITIALIZED_DATA | character::READ | character::WRITE;
    put_raw_section(
        &mut out,
        b".idata$2",
        20,
        headers,
        headers + 20,
        3,
        alignment_characteristics(4) | data,
    );
    put_raw_section(
        &mut out,
        b".idata$6",
        import_name.len() as u32 + 1,
        headers + 20 + 3 * RELOCATION_SIZE as u32,
        0,
        0,
        alignment_characteristics(2) | data,
    );
    out.extend_from_slice(&[0u8; 20]);
    let relocation = image_relative_relocation(machine_number);
    for (offset, symbol) in [(12u32, 2u32), (0, 3), (16, 4)] {
        put_u32(&mut out, offset);
        put_u32(&mut out, symbol);
        put_u16(&mut out, relocation);
    }
    out.extend_from_slice(import_name.as_bytes());
    out.push(0);
    put_raw_symbol(
        &mut out,
        string_offset_name(4),
        1,
        storage_class::EXTERNAL,
        0,
    );
    put_raw_symbol(&mut out, *b".idata$2", 1, storage_class::SECTION, 0);
    put_raw_symbol(&mut out, *b".idata$6", 2, storage_class::STATIC, 0);
    put_raw_symbol(&mut out, *b".idata$4", 0, storage_class::SECTION, 0);
    put_raw_symbol(&mut out, *b".idata$5", 0, storage_class::SECTION, 0);
    let second = 4 + descriptor_symbol.len() as u32 + 1;
    put_raw_symbol(
        &mut out,
        string_offset_name(second),
        0,
        storage_class::EXTERNAL,
        0,
    );
    let third = second + NULL_IMPORT_DESCRIPTOR.len() as u32 + 1;
    put_raw_symbol(
        &mut out,
        string_offset_name(third),
        0,
        storage_class::EXTERNAL,
        0,
    );
    put_raw_strings(
        &mut out,
        &[&descriptor_symbol, NULL_IMPORT_DESCRIPTOR, &thunk_symbol],
    );
    out
}

fn null_import_descriptor_object(machine_number: u16) -> Vec<u8> {
    use section_characteristic as character;
    let headers = (COFF_HEADER_SIZE + SECTION_HEADER_SIZE) as u32;
    let mut out = Vec::new();
    put_coff_header(
        &mut out,
        machine_number,
        1,
        headers + 20,
        1,
        word_characteristics(machine_number),
    );
    put_raw_section(
        &mut out,
        b".idata$3",
        20,
        headers,
        0,
        0,
        alignment_characteristics(4)
            | character::INITIALIZED_DATA
            | character::READ
            | character::WRITE,
    );
    out.extend_from_slice(&[0u8; 20]);
    put_raw_symbol(
        &mut out,
        string_offset_name(4),
        1,
        storage_class::EXTERNAL,
        0,
    );
    put_raw_strings(&mut out, &[NULL_IMPORT_DESCRIPTOR]);
    out
}

fn null_thunk_object(library: &str, machine_number: u16) -> Vec<u8> {
    use section_characteristic as character;
    let thunk_symbol = format!("\u{7f}{library}_NULL_THUNK_DATA");
    let headers = (COFF_HEADER_SIZE + 2 * SECTION_HEADER_SIZE) as u32;
    let word = if is_64_bit(machine_number) { 8 } else { 4 };
    let mut out = Vec::new();
    put_coff_header(
        &mut out,
        machine_number,
        2,
        headers + 2 * word,
        1,
        word_characteristics(machine_number),
    );
    let alignment = if is_64_bit(machine_number) {
        alignment_characteristics(8)
    } else {
        alignment_characteristics(4)
    };
    let flags = alignment | character::INITIALIZED_DATA | character::READ | character::WRITE;
    put_raw_section(&mut out, b".idata$5", word, headers, 0, 0, flags);
    put_raw_section(&mut out, b".idata$4", word, headers + word, 0, 0, flags);
    out.extend(std::iter::repeat_n(0u8, 2 * word as usize));
    put_raw_symbol(
        &mut out,
        string_offset_name(4),
        1,
        storage_class::EXTERNAL,
        0,
    );
    put_raw_strings(&mut out, &[&thunk_symbol]);
    out
}

fn weak_external_object(symbol: &str, weak: &str, import: bool, machine_number: u16) -> Vec<u8> {
    use section_characteristic as character;
    let mut out = Vec::new();
    put_coff_header(
        &mut out,
        machine_number,
        1,
        (COFF_HEADER_SIZE + SECTION_HEADER_SIZE) as u32,
        5,
        0,
    );
    put_raw_section(
        &mut out,
        b".drectve",
        0,
        0,
        0,
        0,
        character::LINK_INFORMATION | character::LINK_REMOVE,
    );
    put_raw_symbol(&mut out, *b"@comp.id", 0xffff, storage_class::STATIC, 0);
    put_raw_symbol(&mut out, *b"@feat.00", 0xffff, storage_class::STATIC, 0);
    let prefix = if import { "__imp_" } else { "" };
    put_raw_symbol(
        &mut out,
        string_offset_name(4),
        0,
        storage_class::EXTERNAL,
        0,
    );
    let second = 4 + (prefix.len() + symbol.len()) as u32 + 1;
    put_raw_symbol(
        &mut out,
        string_offset_name(second),
        0,
        storage_class::WEAK_EXTERNAL,
        1,
    );
    put_raw_symbol(
        &mut out,
        [2, 0, 0, 0, weak_search::ALIAS as u8, 0, 0, 0],
        0,
        0,
        0,
    );
    let strong = format!("{prefix}{symbol}");
    let weak = format!("{prefix}{weak}");
    put_raw_strings(&mut out, &[&strong, &weak]);
    out
}

fn short_import_object(
    symbol: &str,
    library: &str,
    ordinal: u16,
    kind: u16,
    name_type: u16,
    export_name: &str,
    machine_number: u16,
) -> Vec<u8> {
    ImportObject {
        machine: machine_number,
        time_stamp: 0,
        size_of_data: 0,
        ordinal_or_hint: ordinal,
        kind,
        name_type,
        symbol_name: symbol.to_string(),
        library_name: library.to_string(),
        export_name: (!export_name.is_empty()).then(|| export_name.to_string()),
    }
    .to_bytes()
}

pub fn write_import_library(
    import_name: &str,
    exports: &[ShortExport],
    machine_number: u16,
    mingw: bool,
) -> Result<Vec<u8>, Diagnostic> {
    let file_name = import_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(import_name);
    let library = match file_name.rfind('.') {
        Some(position) if position > 0 => &file_name[..position],
        _ => file_name,
    };
    let member = |data: Vec<u8>| ArchiveMember {
        name: file_name.to_string(),
        data,
    };
    let mut members = vec![
        member(import_descriptor_object(file_name, library, machine_number)),
        member(null_import_descriptor_object(machine_number)),
        member(null_thunk_object(library, machine_number)),
    ];
    let mut regular: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut renames: Vec<(String, u16, &ShortExport)> = Vec::new();
    for export in exports {
        if export.private {
            continue;
        }
        let kind = if export.constant {
            import_type::CONSTANT
        } else if export.data {
            import_type::DATA
        } else {
            import_type::CODE
        };
        let symbol_name = if export.symbol_name.is_empty() {
            &export.name
        } else {
            &export.symbol_name
        };
        let name = if export.external_name.is_empty() {
            symbol_name.clone()
        } else {
            match replace_export_name(symbol_name, &export.name, &export.external_name) {
                Some(name) => name,
                None => {
                    return Err(
                        fail("EI246", "The module definition file could not be read.")
                            .with_context(format!("Symbol: {symbol_name}"))
                            .with_context(format!("Export: {}", export.external_name)),
                    )
                }
            }
        };
        let mut export_name = String::new();
        let name_type = if export.no_name {
            import_name_type::ORDINAL
        } else if !export.export_as.is_empty() {
            export_name = export.export_as.clone();
            import_name_type::NAME_EXPORT_AS
        } else if !export.import_name.is_empty() {
            if machine_number == machine::I386
                && apply_name_type(import_name_type::NAME_UNDECORATE, &name) == export.import_name
            {
                import_name_type::NAME_UNDECORATE
            } else if machine_number == machine::I386
                && apply_name_type(import_name_type::NAME_NO_PREFIX, &name) == export.import_name
            {
                import_name_type::NAME_NO_PREFIX
            } else if name == export.import_name {
                import_name_type::NAME
            } else {
                renames.push((name, kind, export));
                continue;
            }
        } else {
            short_name_type(symbol_name, &export.name, machine_number, mingw)
        };
        regular.insert(apply_name_type(name_type, &name), name.clone());
        members.push(member(short_import_object(
            &name,
            file_name,
            export.ordinal,
            kind,
            name_type,
            &export_name,
            machine_number,
        )));
    }
    for (name, kind, export) in renames {
        match regular.get(&export.import_name) {
            Some(symbol) => {
                if kind == import_type::CODE {
                    members.push(member(weak_external_object(
                        symbol,
                        &name,
                        false,
                        machine_number,
                    )));
                }
                members.push(member(weak_external_object(
                    symbol,
                    &name,
                    true,
                    machine_number,
                )));
            }
            None => members.push(member(short_import_object(
                &name,
                file_name,
                export.ordinal,
                kind,
                import_name_type::NAME_EXPORT_AS,
                &export.import_name,
                machine_number,
            ))),
        }
    }
    Ok(write_coff_archive(&members))
}

fn archive_symbols(data: &[u8]) -> Vec<String> {
    if let Ok(import) = ImportObject::parse(data) {
        let mut names = vec![format!("__imp_{}", import.symbol_name)];
        if import.kind != import_type::DATA {
            names.push(import.symbol_name);
        }
        return names;
    }
    let Ok(object) = CoffObject::parse(data) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for symbol in &object.symbols {
        let global = symbol.storage_class == storage_class::EXTERNAL
            || symbol.storage_class == storage_class::WEAK_EXTERNAL;
        if !global || symbol.file_name().is_some() || symbol.is_section_definition() {
            continue;
        }
        if symbol.storage_class == storage_class::EXTERNAL && symbol.is_undefined() {
            continue;
        }
        if let Some((_, search)) = symbol.weak_external() {
            if search != weak_search::ALIAS {
                continue;
            }
        }
        names.push(symbol.name.clone());
    }
    names
}

fn padded(text: &str, width: usize) -> String {
    format!("{text:<width$}")
}

fn archive_header(name: &str, mode: &str, size: usize) -> String {
    format!(
        "{}{}{}{}{}{}`\n",
        padded(name, 16),
        padded("0", 12),
        padded("0", 6),
        padded("0", 6),
        padded(mode, 8),
        padded(&size.to_string(), 10)
    )
}

pub fn write_coff_archive(members: &[ArchiveMember]) -> Vec<u8> {
    let mut long_names: Vec<u8> = Vec::new();
    let mut long_name_offsets: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    let mut headers: Vec<String> = Vec::new();
    let mut symbol_names: Vec<u8> = Vec::new();
    let mut member_symbols: Vec<Vec<usize>> = Vec::new();
    let mut symbol_map: std::collections::BTreeMap<Vec<u8>, u16> =
        std::collections::BTreeMap::new();
    for (index, member) in members.iter().enumerate() {
        let name = if member.name.len() >= 16 || member.name.contains('/') {
            let offset = *long_name_offsets
                .entry(member.name.clone())
                .or_insert_with(|| {
                    let offset = long_names.len();
                    long_names.extend_from_slice(member.name.as_bytes());
                    long_names.push(0);
                    offset
                });
            format!("/{offset}")
        } else {
            format!("{}/", member.name)
        };
        headers.push(archive_header(&name, "644", member.data.len()));
        let mut offsets = Vec::new();
        for symbol in archive_symbols(&member.data) {
            let key = symbol.as_bytes().to_vec();
            if symbol_map.contains_key(&key) {
                continue;
            }
            symbol_map.insert(key, (index + 1) as u16);
            offsets.push(symbol_names.len());
            symbol_names.extend_from_slice(symbol.as_bytes());
            symbol_names.push(0);
        }
        member_symbols.push(offsets);
    }
    let symbol_count: usize = member_symbols.iter().map(Vec::len).sum();
    let mut table_size = 4 + symbol_count * 4 + symbol_names.len();
    let table_padding = table_size % 2;
    table_size += table_padding;
    let mut map_size = 8
        + members.len() * 4
        + symbol_map
            .keys()
            .map(|name| 2 + name.len() + 1)
            .sum::<usize>();
    let map_padding = map_size % 2;
    map_size += map_padding;
    let long_padding = long_names.len() % 2;
    let long_size = if long_names.is_empty() {
        0
    } else {
        60 + long_names.len() + long_padding
    };
    let headers_size = 8 + 60 + table_size + 60 + map_size + long_size;
    let mut positions = Vec::new();
    let mut position = headers_size;
    for (header, member) in headers.iter().zip(members) {
        positions.push(position as u32);
        position += header.len() + member.data.len() + member.data.len() % 2;
    }
    let mut out = b"!<arch>\n".to_vec();
    out.extend_from_slice(archive_header("/", "0", table_size).as_bytes());
    out.extend_from_slice(&(symbol_count as u32).to_be_bytes());
    for (offsets, member_position) in member_symbols.iter().zip(&positions) {
        for _ in offsets {
            out.extend_from_slice(&member_position.to_be_bytes());
        }
    }
    out.extend_from_slice(&symbol_names);
    out.extend(std::iter::repeat_n(0u8, table_padding));
    out.extend_from_slice(archive_header("/", "0", map_size).as_bytes());
    put_u32(&mut out, members.len() as u32);
    for member_position in &positions {
        put_u32(&mut out, *member_position);
    }
    put_u32(&mut out, symbol_map.len() as u32);
    for index in symbol_map.values() {
        put_u16(&mut out, *index);
    }
    for name in symbol_map.keys() {
        out.extend_from_slice(name);
        out.push(0);
    }
    out.extend(std::iter::repeat_n(0u8, map_padding));
    if !long_names.is_empty() {
        out.extend_from_slice(
            format!(
                "{}{}`\n",
                padded("//", 48),
                padded(&(long_names.len() + long_padding).to_string(), 10)
            )
            .as_bytes(),
        );
        out.extend_from_slice(&long_names);
        if long_padding == 1 {
            out.push(b'\n');
        }
    }
    for (header, member) in headers.iter().zip(members) {
        out.extend_from_slice(header.as_bytes());
        out.extend_from_slice(&member.data);
        if member.data.len() % 2 == 1 {
            out.push(b'\n');
        }
    }
    out
}

pub fn write_library_archive(members: &[ArchiveMember], thin: bool, index: bool) -> Vec<u8> {
    if !thin && index {
        return write_coff_archive(members);
    }
    let mut table: Vec<u8> = Vec::new();
    let mut table_offsets: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    let mut headers: Vec<String> = Vec::new();
    for member in members {
        let name = if thin || member.name.len() >= 16 || member.name.contains('/') {
            let offset = if thin {
                let offset = table.len();
                table.extend_from_slice(member.name.as_bytes());
                table.extend_from_slice(b"/\n");
                offset
            } else {
                *table_offsets.entry(member.name.clone()).or_insert_with(|| {
                    let offset = table.len();
                    table.extend_from_slice(member.name.as_bytes());
                    table.extend_from_slice(b"/\n");
                    offset
                })
            };
            format!("/{offset}")
        } else {
            format!("{}/", member.name)
        };
        headers.push(archive_header(&name, "644", member.data.len()));
    }
    let mut symbol_members: Vec<usize> = Vec::new();
    let mut symbol_names: Vec<u8> = Vec::new();
    if index {
        for (position, member) in members.iter().enumerate() {
            for symbol in archive_symbols(&member.data) {
                symbol_members.push(position);
                symbol_names.extend_from_slice(symbol.as_bytes());
                symbol_names.push(0);
            }
        }
    }
    let mut symbol_size = 4 + symbol_members.len() * 4 + symbol_names.len();
    let symbol_padding = symbol_size % 2;
    symbol_size += symbol_padding;
    let table_padding = table.len() % 2;
    let mut position = 8;
    if index {
        position += 60 + symbol_size;
    }
    if !table.is_empty() {
        position += 60 + table.len() + table_padding;
    }
    let mut positions = Vec::new();
    for (header, member) in headers.iter().zip(members) {
        positions.push(position as u32);
        position += header.len();
        if !thin {
            position += member.data.len() + member.data.len() % 2;
        }
    }
    let mut out = if thin {
        b"!<thin>\n".to_vec()
    } else {
        b"!<arch>\n".to_vec()
    };
    if index {
        out.extend_from_slice(archive_header("/", "0", symbol_size).as_bytes());
        out.extend_from_slice(&(symbol_members.len() as u32).to_be_bytes());
        for member in &symbol_members {
            out.extend_from_slice(&positions[*member].to_be_bytes());
        }
        out.extend_from_slice(&symbol_names);
        out.extend(std::iter::repeat_n(0u8, symbol_padding));
    }
    if !table.is_empty() {
        out.extend_from_slice(
            format!(
                "{}{}`\n",
                padded("//", 48),
                padded(&(table.len() + table_padding).to_string(), 10)
            )
            .as_bytes(),
        );
        out.extend_from_slice(&table);
        if table_padding == 1 {
            out.push(b'\n');
        }
    }
    for (header, member) in headers.iter().zip(members) {
        out.extend_from_slice(header.as_bytes());
        if !thin {
            out.extend_from_slice(&member.data);
            if member.data.len() % 2 == 1 {
                out.push(b'\n');
            }
        }
    }
    out
}

pub mod program_database {
    use super::{put_u16, put_u32};
    use std::collections::{BTreeMap, BTreeSet};

    pub const INVALID_STREAM: u16 = 0xffff;
    pub const STREAM_INFORMATION: usize = 1;
    pub const STREAM_TYPES: usize = 2;
    pub const STREAM_DEBUG_INFORMATION: usize = 3;
    pub const STREAM_IDENTIFIERS: usize = 4;
    pub const SPECIAL_STREAM_COUNT: usize = 5;
    pub const FIRST_TYPE_INDEX: u32 = 0x1000;
    pub const DEBUG_STREAM_SECTION_HEADERS: usize = 5;
    pub const DEBUG_STREAM_FRAME_OLD: usize = 0;
    pub const DEBUG_STREAM_FRAME_NEW: usize = 9;
    pub const DEBUG_STREAM_COUNT: usize = 11;
    const HASH_BUCKET_COUNT: usize = 4096;
    const TYPE_HASH_BUCKETS: u32 = 0x3ffff;
    const MAGIC: &[u8; 32] = b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0";

    pub fn hash_string_first(text: &[u8]) -> u32 {
        let mut result = 0u32;
        let whole = text.len() / 4;
        for index in 0..whole {
            result ^= u32::from_le_bytes([
                text[index * 4],
                text[index * 4 + 1],
                text[index * 4 + 2],
                text[index * 4 + 3],
            ]);
        }
        let mut at = whole * 4;
        let mut remaining = text.len() % 4;
        if remaining >= 2 {
            result ^= u32::from(u16::from_le_bytes([text[at], text[at + 1]]));
            at += 2;
            remaining -= 2;
        }
        if remaining == 1 {
            result ^= u32::from(text[at]);
        }
        result |= 0x2020_2020;
        result ^= result >> 11;
        result ^ (result >> 16)
    }

    pub fn hash_string_second(text: &[u8]) -> u32 {
        let mut hash = 0xb170_a1bfu32;
        let whole = text.len() / 4;
        for index in 0..whole {
            let item = u32::from_le_bytes([
                text[index * 4],
                text[index * 4 + 1],
                text[index * 4 + 2],
                text[index * 4 + 3],
            ]);
            hash = hash.wrapping_add(item);
            hash = hash.wrapping_add(hash << 10);
            hash ^= hash >> 6;
        }
        for byte in &text[whole * 4..] {
            hash = hash.wrapping_add(u32::from(*byte));
            hash = hash.wrapping_add(hash << 10);
            hash ^= hash >> 6;
        }
        hash.wrapping_mul(1_664_525).wrapping_add(1_013_904_223)
    }

    pub fn jam_checksum(data: &[u8]) -> u32 {
        let mut table = [0u32; 256];
        for (index, slot) in table.iter_mut().enumerate() {
            let mut value = index as u32;
            for _ in 0..8 {
                value = if value & 1 != 0 {
                    0xedb8_8320 ^ (value >> 1)
                } else {
                    value >> 1
                };
            }
            *slot = value;
        }
        let mut state = 0u32;
        for byte in data {
            state = table[((state ^ u32::from(*byte)) & 0xff) as usize] ^ (state >> 8);
        }
        state
    }

    fn align_four(out: &mut Vec<u8>) {
        while !out.len().is_multiple_of(4) {
            out.push(0);
        }
    }

    fn put_c_string(out: &mut Vec<u8>, text: &[u8]) {
        out.extend_from_slice(text);
        out.push(0);
    }

    #[derive(Clone, Debug)]
    pub struct StringTable {
        pub data: Vec<u8>,
        pub offsets: BTreeMap<Vec<u8>, u32>,
        pub order: Vec<Vec<u8>>,
    }

    impl Default for StringTable {
        fn default() -> StringTable {
            StringTable::new()
        }
    }

    impl StringTable {
        pub fn new() -> StringTable {
            StringTable {
                data: vec![0],
                offsets: BTreeMap::new(),
                order: Vec::new(),
            }
        }

        pub fn insert(&mut self, text: &[u8]) -> u32 {
            if let Some(offset) = self.offsets.get(text) {
                return *offset;
            }
            let offset = self.data.len() as u32;
            self.data.extend_from_slice(text);
            self.data.push(0);
            self.offsets.insert(text.to_vec(), offset);
            self.order.push(text.to_vec());
            offset
        }

        pub fn len(&self) -> usize {
            self.order.len()
        }

        pub fn is_empty(&self) -> bool {
            self.order.is_empty()
        }

        pub fn serialized(&self) -> Vec<u8> {
            let mut out = Vec::new();
            put_u32(&mut out, 0xeffe_effe);
            put_u32(&mut out, 1);
            put_u32(&mut out, self.data.len() as u32);
            out.extend_from_slice(&self.data);
            let bucket_count = string_bucket_count(self.order.len() as u32);
            put_u32(&mut out, bucket_count);
            let mut buckets = vec![0u32; bucket_count as usize];
            for text in &self.order {
                let offset = self.offsets[text];
                let hash = hash_string_first(text);
                for step in 0..bucket_count {
                    let slot = (hash.wrapping_add(step) % bucket_count) as usize;
                    if buckets[slot] != 0 {
                        continue;
                    }
                    buckets[slot] = offset;
                    break;
                }
            }
            for bucket in buckets {
                put_u32(&mut out, bucket);
            }
            put_u32(&mut out, self.order.len() as u32);
            out
        }
    }

    fn string_bucket_count(count: u32) -> u32 {
        let mut strings = 0u32;
        let mut buckets = 1u32;
        let mut table: Vec<(u32, u32)> = vec![(0, 1)];
        while buckets <= u32::MAX / 3 {
            strings += 1;
            if buckets * 3 / 4 < strings {
                buckets = buckets * 3 / 2 + 1;
                table.push((strings, buckets));
            }
        }
        table
            .iter()
            .find(|(threshold, _)| *threshold >= count)
            .map(|(_, buckets)| *buckets)
            .unwrap_or(buckets)
    }

    #[derive(Clone, Debug)]
    pub struct NamedStreams {
        names: Vec<u8>,
        buckets: Vec<Option<(u32, u32)>>,
    }

    impl Default for NamedStreams {
        fn default() -> NamedStreams {
            NamedStreams {
                names: Vec::new(),
                buckets: vec![None],
            }
        }
    }

    impl NamedStreams {
        fn name_at(&self, offset: u32) -> &[u8] {
            let start = offset as usize;
            let end = self.names[start..]
                .iter()
                .position(|byte| *byte == 0)
                .map(|length| start + length)
                .unwrap_or(self.names.len());
            &self.names[start..end]
        }

        fn locate(&self, name: &[u8]) -> (usize, bool) {
            let capacity = self.buckets.len();
            let start = (u32::from(hash_string_first(name) as u16) as usize) % capacity;
            let mut index = start;
            loop {
                match self.buckets[index] {
                    Some((offset, _)) => {
                        if self.name_at(offset) == name {
                            return (index, true);
                        }
                    }
                    None => return (index, false),
                }
                index = (index + 1) % capacity;
                if index == start {
                    return (index, false);
                }
            }
        }

        pub fn get(&self, name: &str) -> Option<u32> {
            let (index, found) = self.locate(name.as_bytes());
            if found {
                self.buckets[index].map(|(_, stream)| stream)
            } else {
                None
            }
        }

        pub fn set(&mut self, name: &str, stream: u32) {
            let (index, found) = self.locate(name.as_bytes());
            if found {
                if let Some(bucket) = self.buckets[index].as_mut() {
                    bucket.1 = stream;
                }
                return;
            }
            let offset = self.names.len() as u32;
            self.names.extend_from_slice(name.as_bytes());
            self.names.push(0);
            self.buckets[index] = Some((offset, stream));
            self.grow();
        }

        fn grow(&mut self) {
            let size = self
                .buckets
                .iter()
                .filter(|bucket| bucket.is_some())
                .count() as u32;
            let capacity = self.buckets.len() as u32;
            let maximum = capacity * 2 / 3 + 1;
            if size < maximum {
                return;
            }
            let entries: Vec<(u32, u32)> = self.buckets.iter().flatten().copied().collect();
            self.buckets = vec![None; (maximum * 2) as usize];
            for (offset, stream) in entries {
                let name = self.name_at(offset).to_vec();
                let (index, _) = self.locate(&name);
                self.buckets[index] = Some((offset, stream));
            }
        }

        pub fn serialized(&self) -> Vec<u8> {
            let mut out = Vec::new();
            put_u32(&mut out, self.names.len() as u32);
            out.extend_from_slice(&self.names);
            let size = self
                .buckets
                .iter()
                .filter(|bucket| bucket.is_some())
                .count();
            put_u32(&mut out, size as u32);
            put_u32(&mut out, self.buckets.len() as u32);
            let last = self.buckets.iter().rposition(|bucket| bucket.is_some());
            let words = last.map(|position| position / 32 + 1).unwrap_or(0);
            put_u32(&mut out, words as u32);
            for word in 0..words {
                let mut value = 0u32;
                for bit in 0..32 {
                    let index = word * 32 + bit;
                    if index < self.buckets.len() && self.buckets[index].is_some() {
                        value |= 1 << bit;
                    }
                }
                put_u32(&mut out, value);
            }
            put_u32(&mut out, 0);
            for (offset, stream) in self.buckets.iter().flatten() {
                put_u32(&mut out, *offset);
                put_u32(&mut out, *stream);
            }
            put_u32(&mut out, 0);
            out
        }
    }

    #[derive(Clone, Debug)]
    pub struct StreamFile {
        pub block_size: u32,
        free: Vec<bool>,
        pub sizes: Vec<u32>,
        pub blocks: Vec<Vec<u32>>,
    }

    impl StreamFile {
        pub fn new(block_size: u32) -> StreamFile {
            StreamFile {
                block_size,
                free: vec![false, false, false, false],
                sizes: Vec::new(),
                blocks: Vec::new(),
            }
        }

        fn block_count(&self, size: u32) -> u32 {
            size.div_ceil(self.block_size)
        }

        fn allocate(&mut self, count: u32) -> Vec<u32> {
            if count == 0 {
                return Vec::new();
            }
            let free_count = self.free.iter().filter(|free| **free).count() as u32;
            if free_count < count {
                let old = self.free.len() as u32;
                let mut new = old + (count - free_count);
                let mut next = old.div_ceil(self.block_size) * self.block_size + 1;
                self.free.resize(new as usize, true);
                while next < new {
                    new += 2;
                    self.free.resize(new as usize, true);
                    self.free[next as usize] = false;
                    self.free[next as usize + 1] = false;
                    next += self.block_size;
                }
            }
            let mut result = Vec::new();
            for index in 0..self.free.len() {
                if result.len() as u32 == count {
                    break;
                }
                if self.free[index] {
                    self.free[index] = false;
                    result.push(index as u32);
                }
            }
            result
        }

        pub fn add_stream(&mut self, size: u32) -> usize {
            let count = self.block_count(size);
            let blocks = self.allocate(count);
            self.sizes.push(size);
            self.blocks.push(blocks);
            self.sizes.len() - 1
        }

        pub fn set_stream_size(&mut self, index: usize, size: u32) {
            let old = self.sizes[index];
            if old == size {
                return;
            }
            let new_count = self.block_count(size);
            let old_count = self.block_count(old);
            if new_count > old_count {
                let added = self.allocate(new_count - old_count);
                self.blocks[index].extend(added);
            } else if old_count > new_count {
                for block in self.blocks[index].split_off(new_count as usize) {
                    self.free[block as usize] = true;
                }
            }
            self.sizes[index] = size;
        }

        pub fn commit(&mut self, contents: &[Vec<u8>]) -> Result<Vec<u8>, String> {
            let mut directory_size = 4 + 4 * self.sizes.len() as u32;
            for size in &self.sizes {
                directory_size += 4 * self.block_count(*size);
            }
            let directory_count = self.block_count(directory_size);
            let directory_blocks = self.allocate(directory_count);
            let block_count = self.free.len() as u32;
            let block_size = self.block_size;
            let file_size = u64::from(block_size) * u64::from(block_count);
            let limit = match block_size {
                8192 => u64::from(u32::MAX) * 2,
                16384 => u64::from(u32::MAX) * 3,
                32768 => u64::from(u32::MAX) * 4,
                _ => u64::from(u32::MAX),
            };
            if file_size > limit {
                return Err(format!(
                    "File size {} too large for current PDB page size {block_size}",
                    group_digits(file_size)
                ));
            }
            if u64::from(directory_count) * 4 > u64::from(block_size) {
                return Err(format!(
                    "The directory block map ({} bytes) doesn't fit in a block ({block_size} bytes)",
                    directory_count * 4
                ));
            }
            let mut out = vec![0u8; file_size as usize];
            out[..32].copy_from_slice(MAGIC);
            let header = [block_size, 2, block_count, directory_size, 0, 3];
            for (index, value) in header.iter().enumerate() {
                out[32 + index * 4..36 + index * 4].copy_from_slice(&value.to_le_bytes());
            }
            for map in [1u32, 2] {
                let intervals = (block_count - map).div_ceil(block_size);
                for interval in 0..intervals {
                    let block = (map + interval * block_size) as usize;
                    let start = block * block_size as usize;
                    if start < out.len() {
                        let end = (start + block_size as usize).min(out.len());
                        out[start..end].fill(0xff);
                    }
                }
            }
            let map_bytes = block_count.div_ceil(8) as usize;
            for byte_index in 0..map_bytes {
                let mut value = 0u8;
                for bit in 0..8 {
                    let block = byte_index * 8 + bit;
                    let free = block >= block_count as usize || self.free[block];
                    if free {
                        value |= 1 << bit;
                    }
                }
                let interval = byte_index / block_size as usize;
                let within = byte_index % block_size as usize;
                let block = 2 + interval * block_size as usize;
                out[block * block_size as usize + within] = value;
            }
            let map_at = 3 * block_size as usize;
            for (index, block) in directory_blocks.iter().enumerate() {
                out[map_at + index * 4..map_at + index * 4 + 4]
                    .copy_from_slice(&block.to_le_bytes());
            }
            let mut directory = Vec::new();
            put_u32(&mut directory, self.sizes.len() as u32);
            for size in &self.sizes {
                put_u32(&mut directory, *size);
            }
            for blocks in &self.blocks {
                for block in blocks {
                    put_u32(&mut directory, *block);
                }
            }
            write_blocks(&mut out, block_size, &directory_blocks, &directory);
            for (index, data) in contents.iter().enumerate() {
                if index >= self.blocks.len() {
                    break;
                }
                let length = (self.sizes[index] as usize).min(data.len());
                write_blocks(&mut out, block_size, &self.blocks[index], &data[..length]);
            }
            Ok(out)
        }
    }

    pub fn group_digits(value: u64) -> String {
        let text = value.to_string();
        let mut out = String::new();
        for (index, character) in text.chars().enumerate() {
            if index > 0 && (text.len() - index).is_multiple_of(3) {
                out.push(',');
            }
            out.push(character);
        }
        out
    }

    fn write_blocks(out: &mut [u8], block_size: u32, blocks: &[u32], data: &[u8]) {
        for (index, chunk) in data.chunks(block_size as usize).enumerate() {
            if index >= blocks.len() {
                break;
            }
            let start = blocks[index] as usize * block_size as usize;
            out[start..start + chunk.len()].copy_from_slice(chunk);
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct SectionContribution {
        pub section: u16,
        pub offset: i32,
        pub size: i32,
        pub characteristics: u32,
        pub module: u16,
        pub data_checksum: u32,
        pub relocation_checksum: u32,
    }

    impl SectionContribution {
        pub fn encode(&self, out: &mut Vec<u8>) {
            put_u16(out, self.section);
            put_u16(out, 0);
            put_u32(out, self.offset as u32);
            put_u32(out, self.size as u32);
            put_u32(out, self.characteristics);
            put_u16(out, self.module);
            put_u16(out, 0);
            put_u32(out, self.data_checksum);
            put_u32(out, self.relocation_checksum);
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct Module {
        pub name: String,
        pub object_name: String,
        pub first_contribution: SectionContribution,
        pub symbols: Vec<u8>,
        pub subsections: Vec<u8>,
        pub source_files: Vec<String>,
        pub path_name_index: u32,
        pub stream: u16,
    }

    #[derive(Clone, Debug, Default)]
    pub struct SectionMapEntry {
        pub flags: u16,
        pub frame: u16,
        pub length: u32,
    }

    #[derive(Clone, Debug)]
    pub struct DebugInformation {
        pub age: u32,
        pub build_number: u16,
        pub machine: u16,
        pub modules: Vec<Module>,
        pub contributions: Vec<SectionContribution>,
        pub section_map: Vec<SectionMapEntry>,
        pub source_names: Vec<String>,
        pub edit_names: StringTable,
        pub debug_streams: Vec<Option<Vec<u8>>>,
        pub debug_stream_numbers: Vec<u16>,
    }

    impl Default for DebugInformation {
        fn default() -> DebugInformation {
            DebugInformation {
                age: 1,
                build_number: 0,
                machine: 0x14c,
                modules: Vec::new(),
                contributions: Vec::new(),
                section_map: Vec::new(),
                source_names: Vec::new(),
                edit_names: StringTable::new(),
                debug_streams: vec![None; DEBUG_STREAM_COUNT],
                debug_stream_numbers: vec![INVALID_STREAM; DEBUG_STREAM_COUNT],
            }
        }
    }

    impl DebugInformation {
        pub fn add_module(&mut self, name: &str) -> usize {
            self.modules.push(Module {
                name: name.to_string(),
                stream: INVALID_STREAM,
                ..Module::default()
            });
            self.modules.len() - 1
        }

        pub fn add_source_file(&mut self, module: usize, path: &str) {
            if !self.source_names.iter().any(|name| name == path) {
                self.source_names.push(path.to_string());
            }
            self.modules[module].source_files.push(path.to_string());
        }

        pub fn create_section_map(&mut self, headers: &[(u32, u32)]) {
            self.section_map.clear();
            for (index, (characteristics, virtual_size)) in headers.iter().enumerate() {
                let mut flags = 0u16;
                if characteristics & 0x4000_0000 != 0 {
                    flags |= 1;
                }
                if characteristics & 0x8000_0000 != 0 {
                    flags |= 2;
                }
                if characteristics & 0x2000_0000 != 0 {
                    flags |= 4;
                }
                if characteristics & 0x0002_0000 == 0 {
                    flags |= 8;
                }
                flags |= 0x100;
                self.section_map.push(SectionMapEntry {
                    flags,
                    frame: index as u16 + 1,
                    length: *virtual_size,
                });
            }
            self.section_map.push(SectionMapEntry {
                flags: 8 | 0x200,
                frame: headers.len() as u16 + 1,
                length: u32::MAX,
            });
        }

        fn module_stream(module: &Module) -> Vec<u8> {
            let mut out = Vec::new();
            put_u32(&mut out, 4);
            out.extend_from_slice(&module.symbols);
            align_four(&mut out);
            out.extend_from_slice(&module.subsections);
            put_u32(&mut out, 0);
            out
        }

        fn file_information(&self) -> Vec<u8> {
            let module_count = self.modules.len().min(usize::from(u16::MAX));
            let mut out = Vec::new();
            put_u16(&mut out, module_count as u16);
            put_u16(
                &mut out,
                self.source_names.len().min(usize::from(u16::MAX)) as u16,
            );
            for index in 0..module_count {
                put_u16(&mut out, index as u16);
            }
            for module in &self.modules {
                put_u16(&mut out, module.source_files.len() as u16);
            }
            let mut name_offsets: BTreeMap<&str, u32> = BTreeMap::new();
            let mut names = Vec::new();
            for name in &self.source_names {
                name_offsets.insert(name.as_str(), names.len() as u32);
                put_c_string(&mut names, name.as_bytes());
            }
            for module in &self.modules {
                for name in &module.source_files {
                    put_u32(
                        &mut out,
                        name_offsets.get(name.as_str()).copied().unwrap_or(0),
                    );
                }
            }
            out.extend_from_slice(&names);
            align_four(&mut out);
            out
        }

        fn serialized(&self, globals: u16, publics: u16, records: u16) -> Vec<u8> {
            let mut modules = Vec::new();
            for (index, module) in self.modules.iter().enumerate() {
                put_u32(&mut modules, index as u32);
                module.first_contribution.encode(&mut modules);
                put_u16(&mut modules, 0);
                put_u16(&mut modules, module.stream);
                let symbol_bytes = if module.stream == INVALID_STREAM {
                    0
                } else {
                    4 + module.symbols.len().div_ceil(4) * 4
                };
                put_u32(&mut modules, symbol_bytes as u32);
                put_u32(&mut modules, 0);
                put_u32(&mut modules, module.subsections.len() as u32);
                put_u16(&mut modules, module.source_files.len() as u16);
                put_u16(&mut modules, 0);
                put_u32(&mut modules, 0);
                put_u32(&mut modules, 0);
                put_u32(&mut modules, module.path_name_index);
                put_c_string(&mut modules, module.name.as_bytes());
                put_c_string(&mut modules, module.object_name.as_bytes());
                align_four(&mut modules);
            }
            let mut contributions = Vec::new();
            if !self.contributions.is_empty() {
                put_u32(&mut contributions, 0xeffe_0000u32.wrapping_add(19_970_605));
                for contribution in &self.contributions {
                    contribution.encode(&mut contributions);
                }
            }
            let mut section_map = Vec::new();
            if !self.section_map.is_empty() {
                put_u16(&mut section_map, self.section_map.len() as u16);
                put_u16(&mut section_map, self.section_map.len() as u16);
                for entry in &self.section_map {
                    put_u16(&mut section_map, entry.flags);
                    put_u16(&mut section_map, 0);
                    put_u16(&mut section_map, 0);
                    put_u16(&mut section_map, entry.frame);
                    put_u16(&mut section_map, u16::MAX);
                    put_u16(&mut section_map, u16::MAX);
                    put_u32(&mut section_map, 0);
                    put_u32(&mut section_map, entry.length);
                }
            }
            let files = self.file_information();
            let edit_names = self.edit_names.serialized();
            let mut out = Vec::new();
            put_u32(&mut out, u32::MAX);
            put_u32(&mut out, 19_990_903);
            put_u32(&mut out, self.age);
            put_u16(&mut out, globals);
            put_u16(&mut out, self.build_number);
            put_u16(&mut out, publics);
            put_u16(&mut out, 0);
            put_u16(&mut out, records);
            put_u16(&mut out, 0);
            put_u32(&mut out, modules.len() as u32);
            put_u32(&mut out, contributions.len() as u32);
            put_u32(&mut out, section_map.len() as u32);
            put_u32(&mut out, files.len() as u32);
            put_u32(&mut out, 0);
            put_u32(&mut out, 0);
            put_u32(&mut out, (DEBUG_STREAM_COUNT * 2) as u32);
            put_u32(&mut out, edit_names.len() as u32);
            put_u16(&mut out, 0);
            put_u16(&mut out, self.machine);
            put_u32(&mut out, 0);
            out.extend_from_slice(&modules);
            out.extend_from_slice(&contributions);
            out.extend_from_slice(&section_map);
            out.extend_from_slice(&files);
            out.extend_from_slice(&edit_names);
            for number in &self.debug_stream_numbers {
                put_u16(&mut out, *number);
            }
            out
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct TypeStream {
        pub records: Vec<Vec<u8>>,
        pub hashes: Vec<u32>,
    }

    impl TypeStream {
        pub fn add(&mut self, record: Vec<u8>, hash: u32) {
            self.records.push(record);
            self.hashes.push(hash);
        }

        fn index_offsets(&self) -> Vec<(u32, u32)> {
            let mut out = Vec::new();
            let mut bytes = 0usize;
            for (count, record) in self.records.iter().enumerate() {
                let next = bytes + record.len();
                if next / 8192 > bytes / 8192 || count == 0 {
                    out.push((FIRST_TYPE_INDEX + count as u32, bytes as u32));
                }
                bytes = next;
            }
            out
        }

        fn hash_stream(&self) -> Vec<u8> {
            let mut out = Vec::new();
            for hash in &self.hashes {
                put_u32(&mut out, hash % TYPE_HASH_BUCKETS);
            }
            for (index, offset) in self.index_offsets() {
                put_u32(&mut out, index);
                put_u32(&mut out, offset);
            }
            out
        }

        fn serialized(&self, hash_stream: u16) -> Vec<u8> {
            let bytes: usize = self.records.iter().map(|record| record.len()).sum();
            let hashes = (self.hashes.len() * 4) as u32;
            let offsets = (self.index_offsets().len() * 8) as u32;
            let mut out = Vec::new();
            put_u32(&mut out, 20_040_203);
            put_u32(&mut out, 56);
            put_u32(&mut out, FIRST_TYPE_INDEX);
            put_u32(&mut out, FIRST_TYPE_INDEX + self.records.len() as u32);
            put_u32(&mut out, bytes as u32);
            put_u16(&mut out, hash_stream);
            put_u16(&mut out, INVALID_STREAM);
            put_u32(&mut out, 4);
            put_u32(&mut out, TYPE_HASH_BUCKETS);
            put_u32(&mut out, 0);
            put_u32(&mut out, hashes);
            put_u32(&mut out, hashes);
            put_u32(&mut out, offsets);
            put_u32(&mut out, hashes);
            put_u32(&mut out, 0);
            for record in &self.records {
                out.extend_from_slice(record);
            }
            out
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct Public {
        pub name: Vec<u8>,
        pub flags: u32,
        pub offset: u32,
        pub segment: u16,
        pub record_offset: u32,
    }

    fn public_size(public: &Public) -> usize {
        let length = public.name.len().min(0xff00 - 14 - 1);
        (14 + length + 1).div_ceil(4) * 4
    }

    fn compare_hash_names(left: &[u8], right: &[u8]) -> std::cmp::Ordering {
        if left.len() != right.len() {
            return left.len().cmp(&right.len());
        }
        if !left.is_ascii() || !right.is_ascii() {
            return left.cmp(right);
        }
        left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase())
    }

    fn hash_table(records: &[(Vec<u8>, u32)]) -> Vec<u8> {
        let buckets: Vec<usize> = records
            .iter()
            .map(|(name, _)| hash_string_first(name) as usize % HASH_BUCKET_COUNT)
            .collect();
        let mut starts = vec![0u32; HASH_BUCKET_COUNT];
        for bucket in &buckets {
            starts[*bucket] += 1;
        }
        let mut sum = 0u32;
        for start in starts.iter_mut() {
            let size = *start;
            *start = sum;
            sum += size;
        }
        let mut cursors = starts.clone();
        let mut entries = vec![0usize; records.len()];
        for (index, bucket) in buckets.iter().enumerate() {
            entries[cursors[*bucket] as usize] = index;
            cursors[*bucket] += 1;
        }
        for bucket in 0..HASH_BUCKET_COUNT {
            let begin = starts[bucket] as usize;
            let end = cursors[bucket] as usize;
            entries[begin..end].sort_by(|left, right| {
                compare_hash_names(&records[*left].0, &records[*right].0)
                    .then(records[*left].1.cmp(&records[*right].1))
            });
        }
        let mut bitmap = vec![0u32; (HASH_BUCKET_COUNT + 32) / 32];
        let mut chains = Vec::new();
        for (word, slot) in bitmap.iter_mut().enumerate() {
            for bit in 0..32 {
                let bucket = word * 32 + bit;
                if bucket >= HASH_BUCKET_COUNT || starts[bucket] == cursors[bucket] {
                    continue;
                }
                *slot |= 1 << bit;
                chains.push(starts[bucket] * 12);
            }
        }
        let mut out = Vec::new();
        put_u32(&mut out, u32::MAX);
        put_u32(&mut out, 0xeffe_0000u32.wrapping_add(19_990_810));
        put_u32(&mut out, (entries.len() * 8) as u32);
        put_u32(&mut out, ((bitmap.len() + chains.len()) * 4) as u32);
        for entry in entries {
            put_u32(&mut out, records[entry].1 + 1);
            put_u32(&mut out, 1);
        }
        for word in bitmap {
            put_u32(&mut out, word);
        }
        for chain in chains {
            put_u32(&mut out, chain);
        }
        out
    }

    #[derive(Clone, Debug, Default)]
    pub struct InjectedSource {
        pub stream_name: String,
        pub content: Vec<u8>,
        pub name_index: u32,
        pub virtual_name_index: u32,
    }

    #[derive(Clone, Debug)]
    pub struct ProgramDatabase {
        pub block_size: u32,
        pub named_data: Vec<(String, Vec<u8>)>,
        pub information: DebugInformation,
        pub types: TypeStream,
        pub identifiers: TypeStream,
        pub strings: StringTable,
        pub publics: Vec<Public>,
        pub globals: Vec<(Vec<u8>, Vec<u8>)>,
        globals_seen: BTreeSet<Vec<u8>>,
        pub injected: Vec<InjectedSource>,
    }

    impl ProgramDatabase {
        pub fn new(block_size: u32) -> ProgramDatabase {
            ProgramDatabase {
                block_size,
                named_data: Vec::new(),
                information: DebugInformation::default(),
                types: TypeStream::default(),
                identifiers: TypeStream::default(),
                strings: StringTable::new(),
                publics: Vec::new(),
                globals: Vec::new(),
                globals_seen: BTreeSet::new(),
                injected: Vec::new(),
            }
        }

        pub fn add_global(&mut self, record: Vec<u8>, name: Vec<u8>) {
            let kind = u16::from_le_bytes([record[2], record[3]]);
            if (kind == 0x1108 || kind == 0x1107) && !self.globals_seen.insert(record.clone()) {
                return;
            }
            self.globals.push((record, name));
        }

        pub fn set_publics(&mut self, mut publics: Vec<Public>) {
            publics.sort_by(|left, right| left.name.cmp(&right.name));
            let mut offset = 0u32;
            for public in publics.iter_mut() {
                public.record_offset = offset;
                offset += public_size(public) as u32;
            }
            self.publics = publics;
        }

        pub fn commit(&mut self) -> Result<(Vec<u8>, usize), String> {
            let mut file = StreamFile::new(self.block_size);
            for _ in 0..SPECIAL_STREAM_COUNT {
                file.add_stream(0);
            }
            let mut named = NamedStreams::default();
            let mut contents: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
            for (name, data) in &self.named_data {
                let stream = file.add_stream(data.len() as u32);
                named.set(name, stream as u32);
                contents.insert(stream, data.clone());
            }
            let mut features = Vec::new();
            if !self.identifiers.records.is_empty() {
                features.push(20_140_508u32);
            }
            let link_information = file.add_stream(0);
            named.set("/LinkInfo", link_information as u32);
            let public_bytes: u32 = self
                .publics
                .iter()
                .map(|public| public_size(public) as u32)
                .sum();
            let mut global_records = Vec::new();
            let mut global_offset = public_bytes;
            for (record, name) in &self.globals {
                global_records.push((name.clone(), global_offset));
                global_offset += record.len() as u32;
            }
            let globals_hash = hash_table(&global_records);
            let public_records: Vec<(Vec<u8>, u32)> = self
                .publics
                .iter()
                .map(|public| (public.name.clone(), public.record_offset))
                .collect();
            let publics_hash = hash_table(&public_records);
            let globals_stream = file.add_stream(globals_hash.len() as u32);
            let publics_size = 28 + publics_hash.len() + self.publics.len() * 4;
            let publics_stream = file.add_stream(publics_size as u32);
            let records_stream = file.add_stream(global_offset);
            let type_stream = self.types.serialized(INVALID_STREAM);
            file.set_stream_size(STREAM_TYPES, type_stream.len() as u32);
            let type_hashes = self.types.hash_stream();
            let mut type_hash_stream = INVALID_STREAM;
            if !type_hashes.is_empty() {
                type_hash_stream = file.add_stream(type_hashes.len() as u32) as u16;
                contents.insert(type_hash_stream as usize, type_hashes);
            }
            for index in 0..DEBUG_STREAM_COUNT {
                if let Some(data) = self.information.debug_streams[index].clone() {
                    let stream = file.add_stream(data.len() as u32);
                    self.information.debug_stream_numbers[index] = stream as u16;
                    contents.insert(stream, data);
                }
            }
            for module in 0..self.information.modules.len() {
                self.information.modules[module].stream = INVALID_STREAM;
                if self.information.modules[module].symbols.is_empty()
                    && self.information.modules[module].subsections.is_empty()
                {
                    continue;
                }
                let data = DebugInformation::module_stream(&self.information.modules[module]);
                let stream = file.add_stream(data.len() as u32);
                self.information.modules[module].stream = stream as u16;
                contents.insert(stream, data);
            }
            let debug_stream = self.information.serialized(
                globals_stream as u16,
                publics_stream as u16,
                records_stream as u16,
            );
            file.set_stream_size(STREAM_DEBUG_INFORMATION, debug_stream.len() as u32);
            let names = self.strings.serialized();
            let names_stream = file.add_stream(names.len() as u32);
            named.set("/names", names_stream as u32);
            contents.insert(names_stream, names);
            let identifier_stream_hashes = self.identifiers.hash_stream();
            let mut identifier_hash_stream = INVALID_STREAM;
            let identifier_size = self.identifiers.serialized(INVALID_STREAM).len();
            file.set_stream_size(STREAM_IDENTIFIERS, identifier_size as u32);
            if !identifier_stream_hashes.is_empty() {
                identifier_hash_stream =
                    file.add_stream(identifier_stream_hashes.len() as u32) as u16;
                contents.insert(identifier_hash_stream as usize, identifier_stream_hashes);
            }
            let information_size =
                |named: &NamedStreams| 28 + named.serialized().len() + features.len() * 4;
            file.set_stream_size(STREAM_INFORMATION, information_size(&named) as u32);
            if !self.injected.is_empty() {
                let mut table: Vec<Option<(u32, Vec<u8>)>> = vec![None; 2];
                let mut entries: Vec<(u32, Vec<u8>)> = Vec::new();
                for source in &self.injected {
                    let mut entry = Vec::new();
                    put_u32(&mut entry, 40);
                    put_u32(&mut entry, 19_980_827);
                    put_u32(&mut entry, jam_checksum(&source.content));
                    put_u32(&mut entry, source.content.len() as u32);
                    put_u32(&mut entry, source.name_index);
                    put_u32(&mut entry, 1);
                    put_u32(&mut entry, source.virtual_name_index);
                    entry.extend_from_slice(&[0u8; 12]);
                    entries.push((source.virtual_name_index, entry));
                }
                for (key, entry) in entries {
                    let capacity = table.len();
                    let mut index = (u32::from(key as u16) as usize) % capacity;
                    while let Some((existing, _)) = &table[index] {
                        if *existing == key {
                            break;
                        }
                        index = (index + 1) % capacity;
                    }
                    table[index] = Some((key, entry));
                    let size = table.iter().filter(|slot| slot.is_some()).count() as u32;
                    let maximum = table.len() as u32 * 2 / 3 + 1;
                    if size >= maximum {
                        let old: Vec<(u32, Vec<u8>)> = table.iter().flatten().cloned().collect();
                        table = vec![None; (maximum * 2) as usize];
                        for (old_key, old_entry) in old {
                            let capacity = table.len();
                            let mut slot = (u32::from(old_key as u16) as usize) % capacity;
                            while table[slot].is_some() {
                                slot = (slot + 1) % capacity;
                            }
                            table[slot] = Some((old_key, old_entry));
                        }
                    }
                }
                let mut body = Vec::new();
                let size = table.iter().filter(|slot| slot.is_some()).count();
                put_u32(&mut body, size as u32);
                put_u32(&mut body, table.len() as u32);
                let last = table.iter().rposition(|slot| slot.is_some());
                let words = last.map(|position| position / 32 + 1).unwrap_or(0);
                put_u32(&mut body, words as u32);
                for word in 0..words {
                    let mut value = 0u32;
                    for bit in 0..32 {
                        let index = word * 32 + bit;
                        if index < table.len() && table[index].is_some() {
                            value |= 1 << bit;
                        }
                    }
                    put_u32(&mut body, value);
                }
                put_u32(&mut body, 0);
                for (key, entry) in table.iter().flatten() {
                    put_u32(&mut body, *key);
                    body.extend_from_slice(entry);
                }
                let mut header_block = Vec::new();
                put_u32(&mut header_block, 19_980_827);
                put_u32(&mut header_block, (64 + body.len()) as u32);
                header_block.extend_from_slice(&[0u8; 56]);
                header_block.extend_from_slice(&body);
                let header_stream = file.add_stream(header_block.len() as u32);
                named.set("/src/headerblock", header_stream as u32);
                contents.insert(header_stream, header_block);
                for source in &self.injected {
                    let stream = file.add_stream(source.content.len() as u32);
                    named.set(&source.stream_name, stream as u32);
                    contents.insert(stream, source.content.clone());
                }
                file.set_stream_size(STREAM_INFORMATION, information_size(&named) as u32);
            }
            let mut information = Vec::new();
            put_u32(&mut information, 20_000_404);
            information.extend_from_slice(&[0u8; 24]);
            information.extend_from_slice(&named.serialized());
            for feature in &features {
                put_u32(&mut information, *feature);
            }
            contents.insert(STREAM_INFORMATION, information);
            contents.insert(STREAM_DEBUG_INFORMATION, debug_stream);
            contents.insert(STREAM_TYPES, self.types.serialized(type_hash_stream));
            contents.insert(
                STREAM_IDENTIFIERS,
                self.identifiers.serialized(identifier_hash_stream),
            );
            let mut record_bytes = Vec::new();
            for public in &self.publics {
                let length = public.name.len().min(0xff00 - 14 - 1);
                let size = public_size(public);
                put_u16(&mut record_bytes, (size - 2) as u16);
                put_u16(&mut record_bytes, 0x110e);
                put_u32(&mut record_bytes, public.flags);
                put_u32(&mut record_bytes, public.offset);
                put_u16(&mut record_bytes, public.segment);
                record_bytes.extend_from_slice(&public.name[..length]);
                record_bytes.extend(std::iter::repeat_n(0u8, size - 14 - length));
            }
            for (record, _) in &self.globals {
                record_bytes.extend_from_slice(record);
            }
            contents.insert(records_stream, record_bytes);
            contents.insert(globals_stream, globals_hash);
            let mut publics_data = Vec::new();
            put_u32(&mut publics_data, publics_hash.len() as u32);
            put_u32(&mut publics_data, (self.publics.len() * 4) as u32);
            put_u32(&mut publics_data, 0);
            put_u32(&mut publics_data, 0);
            put_u16(&mut publics_data, 0);
            put_u16(&mut publics_data, 0);
            put_u32(&mut publics_data, 0);
            put_u32(&mut publics_data, 0);
            publics_data.extend_from_slice(&publics_hash);
            let mut address_order: Vec<usize> = (0..self.publics.len()).collect();
            address_order.sort_by(|left, right| {
                let left = &self.publics[*left];
                let right = &self.publics[*right];
                left.segment
                    .cmp(&right.segment)
                    .then(left.offset.cmp(&right.offset))
                    .then(left.name.cmp(&right.name))
            });
            for index in address_order {
                put_u32(&mut publics_data, self.publics[index].record_offset);
            }
            contents.insert(publics_stream, publics_data);
            let stream_count = file.sizes.len();
            let mut ordered = vec![Vec::new(); stream_count];
            for (index, data) in contents {
                if index < stream_count {
                    ordered[index] = data;
                }
            }
            let bytes = file.commit(&ordered)?;
            let information_offset = file.blocks[STREAM_INFORMATION]
                .first()
                .map(|block| *block as usize * self.block_size as usize)
                .unwrap_or(0);
            Ok((bytes, information_offset))
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct ServerDatabase {
        pub guid: [u8; 16],
        pub age: u32,
        pub types: Vec<Vec<u8>>,
        pub type_bytes: usize,
        pub identifiers: Option<Vec<Vec<u8>>>,
        pub identifier_bytes: usize,
    }

    const CORRUPT_FILE: &str = "The PDB file is corrupt.";
    const UNEXPECTED_FORMAT: &str = "The data is in an unexpected format.";
    const STREAM_TOO_SHORT: &str =
        "Stream Error: The stream is too short to perform the requested operation.";
    const MISSING_STREAM: &str = "The specified stream could not be loaded.";
    const MINIMUM_TYPE_BUCKETS: u32 = 0x1000;
    const MAXIMUM_TYPE_BUCKETS: u32 = 0x40000;

    fn word_at(bytes: &[u8], at: usize) -> Option<u32> {
        let slice = bytes.get(at..at.checked_add(4)?)?;
        Some(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    fn split_type_records(data: &[u8]) -> Result<Vec<Vec<u8>>, String> {
        let mut records = Vec::new();
        let mut at = 0usize;
        while at < data.len() {
            if at + 4 > data.len() {
                return Err(STREAM_TOO_SHORT.to_string());
            }
            let length = u16::from_le_bytes([data[at], data[at + 1]]) as usize + 2;
            if length < 4 || at + length > data.len() {
                return Err(STREAM_TOO_SHORT.to_string());
            }
            records.push(data[at..at + length].to_vec());
            at += length;
        }
        Ok(records)
    }

    fn read_type_stream(data: &[u8]) -> Result<(Vec<Vec<u8>>, usize), String> {
        if data.len() < 56 {
            return Err(format!(
                "{CORRUPT_FILE} TPI Stream does not contain a header."
            ));
        }
        let field = |at: usize| word_at(data, at).unwrap_or(0);
        if field(0) != 20_040_203 {
            return Err(format!("{CORRUPT_FILE} Unsupported TPI Version."));
        }
        if field(4) != 56 {
            return Err(format!("{CORRUPT_FILE} Corrupt TPI Header size."));
        }
        if field(24) != 4 {
            return Err(format!(
                "{CORRUPT_FILE} TPI Stream expected 4 byte hash key size."
            ));
        }
        let buckets = field(28);
        if !(MINIMUM_TYPE_BUCKETS..=MAXIMUM_TYPE_BUCKETS).contains(&buckets) {
            return Err(format!(
                "{CORRUPT_FILE} TPI Stream Invalid number of hash buckets."
            ));
        }
        let length = field(16) as usize;
        let Some(records) = data.get(56..56 + length) else {
            return Err(STREAM_TOO_SHORT.to_string());
        };
        Ok((split_type_records(records)?, length))
    }

    fn skip_bit_vector(data: &[u8], at: &mut usize) -> Result<Vec<u32>, String> {
        let count = word_at(data, *at).ok_or(STREAM_TOO_SHORT)? as usize;
        *at += 4;
        let mut words = Vec::new();
        for _ in 0..count {
            words.push(word_at(data, *at).ok_or(STREAM_TOO_SHORT)?);
            *at += 4;
        }
        Ok(words)
    }

    pub fn read_server_database(bytes: &[u8]) -> Result<ServerDatabase, String> {
        if bytes.len() < 56 {
            return Err(format!("{CORRUPT_FILE} MSF superblock is missing"));
        }
        if bytes[..32] != MAGIC[..] {
            return Err(format!(
                "{UNEXPECTED_FORMAT} MSF magic header doesn't match"
            ));
        }
        let field = |at: usize| word_at(bytes, at).unwrap_or(0);
        let block_size = field(32);
        let free_block_map = field(36);
        let block_count = field(40);
        let directory_bytes = field(44);
        let block_map_address = field(52);
        if !matches!(block_size, 512 | 1024 | 2048 | 4096 | 8192 | 16384 | 32768) {
            return Err(format!("{UNEXPECTED_FORMAT} Unsupported block size."));
        }
        if !directory_bytes.is_multiple_of(4) {
            return Err(format!(
                "{UNEXPECTED_FORMAT} Directory size is not multiple of 4."
            ));
        }
        let directory_blocks = directory_bytes.div_ceil(block_size);
        if directory_blocks > block_size / 4 {
            return Err(format!("{UNEXPECTED_FORMAT} Too many directory blocks."));
        }
        if block_map_address == 0 {
            return Err(format!("{UNEXPECTED_FORMAT} Block 0 is reserved"));
        }
        if block_map_address >= block_count {
            return Err(format!("{UNEXPECTED_FORMAT} Block map address is invalid."));
        }
        if free_block_map != 1 && free_block_map != 2 {
            return Err(format!(
                "{UNEXPECTED_FORMAT} The free block map isn't at block 1 or block 2."
            ));
        }
        let block_size = block_size as usize;
        if !bytes.len().is_multiple_of(block_size) {
            return Err(format!(
                "{CORRUPT_FILE} File size is not a multiple of block size"
            ));
        }
        let block = |index: u32| -> Result<&[u8], String> {
            let start = index as usize * block_size;
            bytes
                .get(start..start + block_size)
                .ok_or_else(|| STREAM_TOO_SHORT.to_string())
        };
        let map_start = block_map_address as usize * block_size;
        let mut directory = Vec::new();
        for position in 0..directory_blocks as usize {
            let index = word_at(bytes, map_start + position * 4).ok_or(STREAM_TOO_SHORT)?;
            directory.extend_from_slice(block(index)?);
        }
        directory.truncate(directory_bytes as usize);
        let stream_count = word_at(&directory, 0).ok_or(STREAM_TOO_SHORT)? as usize;
        let mut sizes = Vec::new();
        for index in 0..stream_count {
            sizes.push(word_at(&directory, 4 + index * 4).ok_or(STREAM_TOO_SHORT)?);
        }
        let mut at = 4 + stream_count * 4;
        let mut streams: Vec<Vec<u8>> = Vec::new();
        for size in &sizes {
            let length = if *size == u32::MAX { 0 } else { *size };
            let mut data = Vec::new();
            for _ in 0..length.div_ceil(block_size as u32) {
                let index = word_at(&directory, at).ok_or(STREAM_TOO_SHORT)?;
                at += 4;
                if (index as u64 + 1) * block_size as u64 > bytes.len() as u64 {
                    return Err(format!("{CORRUPT_FILE} Stream block map is corrupt."));
                }
                data.extend_from_slice(block(index)?);
            }
            data.truncate(length as usize);
            streams.push(data);
        }
        let Some(information) = streams.get(STREAM_INFORMATION) else {
            return Err(MISSING_STREAM.to_string());
        };
        if information.len() < 28 {
            return Err(format!(
                "{STREAM_TOO_SHORT}\n{CORRUPT_FILE} PDB Stream does not contain a header."
            ));
        }
        let version = word_at(information, 0).unwrap_or(0);
        if !matches!(version, 20_000_404 | 20_030_901 | 20_091_201 | 20_140_508) {
            return Err(format!("{CORRUPT_FILE} Unsupported PDB stream version."));
        }
        let mut database = ServerDatabase {
            age: word_at(information, 8).unwrap_or(0),
            ..ServerDatabase::default()
        };
        database.guid.copy_from_slice(&information[12..28]);
        let mut at = 28usize;
        let buffer = word_at(information, at).ok_or(STREAM_TOO_SHORT)? as usize;
        at += 4 + buffer;
        at += 8;
        let present = skip_bit_vector(information, &mut at)?;
        skip_bit_vector(information, &mut at)?;
        let entries: u32 = present.iter().map(|word| word.count_ones()).sum();
        at += entries as usize * 8;
        let mut identifiers = false;
        while let Some(signature) = word_at(information, at) {
            at += 4;
            if signature == 20_091_201 {
                identifiers = true;
                break;
            }
            if signature == 20_140_508 {
                identifiers = true;
            }
        }
        let Some(types) = streams.get(STREAM_TYPES).filter(|data| !data.is_empty()) else {
            return Err(MISSING_STREAM.to_string());
        };
        let (records, length) = read_type_stream(types)?;
        database.types = records;
        database.type_bytes = length;
        if identifiers {
            if let Some(items) = streams.get(STREAM_IDENTIFIERS) {
                let (records, length) = read_type_stream(items)?;
                database.identifiers = Some(records);
                database.identifier_bytes = length;
            }
        }
        Ok(database)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DllExports {
    pub machine: u16,
    pub dll_name: String,
    pub exports: Vec<(String, bool)>,
}

pub fn read_dll_exports(bytes: &[u8]) -> Result<DllExports, String> {
    let header = read_u32(bytes, 0x3c).ok_or("is not a COFF file")? as usize;
    if bytes.get(header..header + 4) != Some(b"PE\0\0".as_slice()) {
        return Err("is not a COFF file".to_string());
    }
    let coff = header + 4;
    let machine_number = read_u16(bytes, coff).ok_or("is not a COFF file")?;
    let section_count = read_u16(bytes, coff + 2).ok_or("is not a COFF file")? as usize;
    let optional_size = read_u16(bytes, coff + 16).ok_or("is not a COFF file")? as usize;
    let optional = coff + 20;
    let magic = read_u16(bytes, optional).unwrap_or(0);
    let directories = match magic {
        0x10b => optional + 96,
        0x20b => optional + 112,
        _ => return Err("is not a PE-COFF executable".to_string()),
    };
    let mut sections: Vec<(u32, u32, u32, u32)> = Vec::new();
    let table = optional + optional_size;
    for index in 0..section_count {
        let at = table + index * 40;
        let virtual_size = read_u32(bytes, at + 8).unwrap_or(0);
        let virtual_address = read_u32(bytes, at + 12).unwrap_or(0);
        let raw_pointer = read_u32(bytes, at + 20).unwrap_or(0);
        let characteristics = read_u32(bytes, at + 36).unwrap_or(0);
        sections.push((virtual_address, virtual_size, raw_pointer, characteristics));
    }
    let offset_of = |rva: u32| -> Option<usize> {
        sections.iter().find_map(|(address, size, raw, _)| {
            if rva >= *address && rva < address.wrapping_add((*size).max(1)) {
                Some((*raw + (rva - address)) as usize)
            } else {
                None
            }
        })
    };
    let string_at = |rva: u32| -> String {
        match offset_of(rva) {
            Some(at) if at < bytes.len() => {
                let end = bytes[at..]
                    .iter()
                    .position(|byte| *byte == 0)
                    .map_or(bytes.len(), |length| at + length);
                String::from_utf8_lossy(&bytes[at..end]).to_string()
            }
            _ => String::new(),
        }
    };
    let mut result = DllExports {
        machine: machine_number,
        ..DllExports::default()
    };
    let export_rva = read_u32(bytes, directories).unwrap_or(0);
    if export_rva == 0 {
        return Ok(result);
    }
    let Some(directory) = offset_of(export_rva) else {
        return Ok(result);
    };
    result.dll_name = string_at(read_u32(bytes, directory + 12).unwrap_or(0));
    let function_count = read_u32(bytes, directory + 20).unwrap_or(0) as usize;
    let name_count = read_u32(bytes, directory + 24).unwrap_or(0) as usize;
    let functions = read_u32(bytes, directory + 28).unwrap_or(0);
    let names = read_u32(bytes, directory + 32).unwrap_or(0);
    let ordinals = read_u32(bytes, directory + 36).unwrap_or(0);
    let mut names_by_index: std::collections::BTreeMap<usize, String> =
        std::collections::BTreeMap::new();
    if let (Some(names_at), Some(ordinals_at)) = (offset_of(names), offset_of(ordinals)) {
        for position in 0..name_count {
            let name_rva = read_u32(bytes, names_at + position * 4).unwrap_or(0);
            let index = read_u16(bytes, ordinals_at + position * 2).unwrap_or(0) as usize;
            names_by_index
                .entry(index)
                .or_insert_with(|| string_at(name_rva));
        }
    }
    let Some(functions_at) = offset_of(functions) else {
        return Ok(result);
    };
    for index in 0..function_count {
        let rva = read_u32(bytes, functions_at + index * 4).unwrap_or(0);
        let Some(name) = names_by_index.get(&index) else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let code = sections
            .iter()
            .find(|(address, size, _, _)| rva >= *address && rva <= address.wrapping_add(*size))
            .is_some_and(|(_, _, _, characteristics)| {
                characteristics & section_characteristic::CODE != 0
            });
        result.exports.push((name.clone(), code));
    }
    Ok(result)
}

pub const RESOURCE_MAGIC: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00,
];

pub fn is_windows_resource(bytes: &[u8]) -> bool {
    bytes.starts_with(&RESOURCE_MAGIC)
}

pub fn manifest_resource(manifest: &[u8], identifier: u32) -> Vec<u8> {
    let mut out = RESOURCE_MAGIC.to_vec();
    out.extend_from_slice(&[0u8; 16]);
    put_u32(&mut out, manifest.len() as u32);
    put_u32(&mut out, 32);
    put_u16(&mut out, 0xffff);
    put_u16(&mut out, 24);
    put_u16(&mut out, 0xffff);
    put_u16(&mut out, identifier as u16);
    put_u32(&mut out, 0);
    put_u16(&mut out, 0x30);
    put_u16(&mut out, 0x0409);
    put_u32(&mut out, 0);
    put_u32(&mut out, 0);
    out.extend_from_slice(manifest);
    while !out.len().is_multiple_of(4) {
        out.push(0);
    }
    out
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceKey {
    Text(Vec<u16>),
    Number(u16),
}

#[derive(Clone, Debug, Default)]
pub struct ResourceNode {
    pub children: std::collections::BTreeMap<ResourceKey, ResourceNode>,
    pub data: Option<usize>,
    pub string_index: Option<usize>,
    pub major_version: u16,
    pub minor_version: u16,
    pub characteristics: u32,
    pub origin: usize,
}

#[derive(Clone, Debug, Default)]
pub struct ResourceTree {
    pub root: ResourceNode,
    pub data: Vec<Vec<u8>>,
    pub strings: Vec<Vec<u16>>,
    pub files: Vec<String>,
}

fn resource_key_text(key: &ResourceKey) -> String {
    match key {
        ResourceKey::Number(number) => number.to_string(),
        ResourceKey::Text(units) => String::from_utf16_lossy(units),
    }
}

fn resource_type_text(key: &ResourceKey) -> String {
    let ResourceKey::Number(number) = key else {
        return resource_key_text(key);
    };
    let name = match number {
        1 => "CURSOR (ID 1)",
        2 => "BITMAP (ID 2)",
        3 => "ICON (ID 3)",
        4 => "MENU (ID 4)",
        5 => "DIALOG (ID 5)",
        6 => "STRINGTABLE (ID 6)",
        7 => "FONTDIR (ID 7)",
        8 => "FONT (ID 8)",
        9 => "ACCELERATOR (ID 9)",
        10 => "RCDATA (ID 10)",
        11 => "MESSAGETABLE (ID 11)",
        12 => "GROUP_CURSOR (ID 12)",
        14 => "GROUP_ICON (ID 14)",
        16 => "VERSIONINFO (ID 16)",
        17 => "DLGINCLUDE (ID 17)",
        19 => "PLUGPLAY (ID 19)",
        20 => "VXD (ID 20)",
        21 => "ANICURSOR (ID 21)",
        22 => "ANIICON (ID 22)",
        23 => "HTML (ID 23)",
        24 => "MANIFEST (ID 24)",
        _ => return format!("ID {number}"),
    };
    name.to_string()
}

fn read_resource_key(bytes: &[u8], at: &mut usize) -> Option<ResourceKey> {
    let first = read_u16(bytes, *at)?;
    if first == 0xffff {
        let number = read_u16(bytes, *at + 2)?;
        *at += 4;
        return Some(ResourceKey::Number(number));
    }
    let mut units = Vec::new();
    loop {
        let unit = read_u16(bytes, *at)?;
        *at += 2;
        if unit == 0 {
            break;
        }
        units.push(unit);
    }
    Some(ResourceKey::Text(units))
}

impl ResourceTree {
    pub fn parse(
        &mut self,
        name: &str,
        bytes: &[u8],
        duplicates: &mut Vec<String>,
    ) -> Result<(), String> {
        if bytes.len() < 32 {
            return Err(format!("{name}: too small to be a resource file"));
        }
        let origin = self.files.len();
        self.files.push(name.to_string());
        let mut at = 32usize;
        while at < bytes.len() {
            let data_size = read_u32(bytes, at)
                .ok_or_else(|| format!("{name}: unexpected end of file"))?
                as usize;
            let header_size = read_u32(bytes, at + 4)
                .ok_or_else(|| format!("{name}: unexpected end of file"))?
                as usize;
            if header_size < 32 {
                return Err(format!("{name}: header size too small"));
            }
            let mut cursor = at + 8;
            let kind = read_resource_key(bytes, &mut cursor)
                .ok_or_else(|| format!("{name}: unexpected end of file"))?;
            let entry = read_resource_key(bytes, &mut cursor)
                .ok_or_else(|| format!("{name}: unexpected end of file"))?;
            cursor = (cursor - at).div_ceil(4) * 4 + at;
            let language = read_u16(bytes, cursor + 6)
                .ok_or_else(|| format!("{name}: unexpected end of file"))?;
            let version = read_u32(bytes, cursor + 8)
                .ok_or_else(|| format!("{name}: unexpected end of file"))?;
            let characteristics = read_u32(bytes, cursor + 12)
                .ok_or_else(|| format!("{name}: unexpected end of file"))?;
            let start = at + header_size;
            let end = start + data_size;
            if end > bytes.len() {
                return Err(format!("{name}: unexpected end of file"));
            }
            let content = bytes[start..end].to_vec();
            at = end.div_ceil(4) * 4;
            if data_size == 0 && kind == ResourceKey::Number(0) && entry == ResourceKey::Number(0) {
                continue;
            }
            let strings = &mut self.strings;
            let mut add_child = |node: &mut ResourceNode, key: &ResourceKey| -> usize {
                let fresh = !node.children.contains_key(key);
                let child = node.children.entry(key.clone()).or_default();
                if fresh {
                    if let ResourceKey::Text(units) = key {
                        child.string_index = Some(strings.len());
                        strings.push(units.clone());
                    }
                }
                0
            };
            add_child(&mut self.root, &kind);
            let type_node = self
                .root
                .children
                .get_mut(&kind)
                .unwrap_or_else(|| unreachable!());
            add_child(type_node, &entry);
            let name_node = type_node
                .children
                .get_mut(&entry)
                .unwrap_or_else(|| unreachable!());
            let language_key = ResourceKey::Number(language);
            if let Some(existing) = name_node.children.get(&language_key) {
                let first = self.files[existing.origin].clone();
                duplicates.push(format!(
                    "duplicate resource: type {}/name {}/language {language}, in {first} and in {name}",
                    resource_type_text(&kind),
                    resource_key_text(&entry)
                ));
                continue;
            }
            let index = self.data.len();
            self.data.push(content);
            name_node.children.insert(
                language_key,
                ResourceNode {
                    data: Some(index),
                    major_version: (version >> 16) as u16,
                    minor_version: version as u16,
                    characteristics,
                    origin,
                    ..ResourceNode::default()
                },
            );
        }
        Ok(())
    }

    pub fn write_object(&self, machine_number: u16, time_stamp: u32) -> Vec<u8> {
        fn tree_size(node: &ResourceNode) -> usize {
            if node.data.is_some() {
                return 16;
            }
            16 + node.children.len() * 8 + node.children.values().map(tree_size).sum::<usize>()
        }
        let tree = tree_size(&self.root);
        let mut string_offsets = Vec::new();
        let mut current = tree;
        let mut strings_total = 0usize;
        for text in &self.strings {
            string_offsets.push(current as u32);
            let size = text.len() * 2 + 2;
            current += size;
            strings_total += size;
        }
        let section_one = tree + strings_total.div_ceil(4) * 4;
        let mut section_one_bytes: Vec<u8> = Vec::with_capacity(section_one);
        let mut relocation_addresses = vec![0u32; self.data.len()];
        let mut queue: std::collections::VecDeque<&ResourceNode> =
            std::collections::VecDeque::new();
        queue.push_back(&self.root);
        let mut next_level = 16 + self.root.children.len() * 8;
        let mut data_nodes: Vec<&ResourceNode> = Vec::new();
        while let Some(node) = queue.pop_front() {
            let names = node
                .children
                .keys()
                .filter(|key| matches!(key, ResourceKey::Text(_)))
                .count();
            put_u32(&mut section_one_bytes, node.characteristics);
            put_u32(&mut section_one_bytes, 0);
            put_u16(&mut section_one_bytes, node.major_version);
            put_u16(&mut section_one_bytes, node.minor_version);
            put_u16(&mut section_one_bytes, names as u16);
            put_u16(&mut section_one_bytes, (node.children.len() - names) as u16);
            let ordered = node
                .children
                .iter()
                .filter(|(key, _)| matches!(key, ResourceKey::Text(_)))
                .chain(
                    node.children
                        .iter()
                        .filter(|(key, _)| matches!(key, ResourceKey::Number(_))),
                );
            for (key, child) in ordered {
                let identifier = match key {
                    ResourceKey::Number(number) => u32::from(*number),
                    ResourceKey::Text(_) => {
                        string_offsets[child.string_index.unwrap_or(0)] | 0x8000_0000
                    }
                };
                put_u32(&mut section_one_bytes, identifier);
                if child.data.is_some() {
                    put_u32(&mut section_one_bytes, next_level as u32);
                    next_level += 16;
                    data_nodes.push(child);
                } else {
                    put_u32(&mut section_one_bytes, next_level as u32 | 0x8000_0000);
                    next_level += 16 + child.children.len() * 8;
                    queue.push_back(child);
                }
            }
        }
        for node in data_nodes {
            let index = node.data.unwrap_or(0);
            relocation_addresses[index] = section_one_bytes.len() as u32;
            put_u32(&mut section_one_bytes, 0);
            put_u32(&mut section_one_bytes, self.data[index].len() as u32);
            put_u32(&mut section_one_bytes, 0);
            put_u32(&mut section_one_bytes, 0);
        }
        for text in &self.strings {
            put_u16(&mut section_one_bytes, text.len() as u16);
            for unit in text {
                put_u16(&mut section_one_bytes, *unit);
            }
        }
        section_one_bytes.resize(section_one, 0);
        let mut data_offsets = Vec::new();
        let mut section_two_bytes: Vec<u8> = Vec::new();
        for content in &self.data {
            data_offsets.push(section_two_bytes.len() as u32);
            section_two_bytes.extend_from_slice(content);
            section_two_bytes.resize(section_two_bytes.len().div_ceil(8) * 8, 0);
        }
        let header = 20 + 2 * 40;
        let section_one_offset = header;
        let relocations_offset = section_one_offset + section_one;
        let section_two_offset = (relocations_offset + self.data.len() * 10).div_ceil(8) * 8;
        let symbol_offset = (section_two_offset + section_two_bytes.len()).div_ceil(8) * 8;
        let mut out = Vec::new();
        put_u16(&mut out, machine_number);
        put_u16(&mut out, 2);
        put_u32(&mut out, time_stamp);
        put_u32(&mut out, symbol_offset as u32);
        put_u32(&mut out, (self.data.len() + 5) as u32);
        put_u16(&mut out, 0);
        put_u16(&mut out, 0x0100);
        let section_header = |out: &mut Vec<u8>,
                              name: &[u8; 8],
                              size: usize,
                              pointer: usize,
                              relocations: usize,
                              count: usize| {
            out.extend_from_slice(name);
            put_u32(out, 0);
            put_u32(out, 0);
            put_u32(out, size as u32);
            put_u32(out, pointer as u32);
            put_u32(out, relocations as u32);
            put_u32(out, 0);
            put_u16(out, count as u16);
            put_u16(out, 0);
            put_u32(
                out,
                section_characteristic::INITIALIZED_DATA | section_characteristic::READ,
            );
        };
        section_header(
            &mut out,
            b".rsrc$01",
            section_one,
            section_one_offset,
            relocations_offset,
            self.data.len(),
        );
        section_header(
            &mut out,
            b".rsrc$02",
            section_two_bytes.len(),
            section_two_offset,
            0,
            0,
        );
        out.extend_from_slice(&section_one_bytes);
        let relocation_kind = match machine_number {
            machine::I386 => relocation_i386::DIRECT_32_IMAGE_RELATIVE,
            machine::ARM_THUMB => relocation_arm::ADDRESS_32_IMAGE_RELATIVE,
            machine::AMD64 => relocation_amd64::ADDRESS_32_IMAGE_RELATIVE,
            _ => relocation_arm64::ADDRESS_32_IMAGE_RELATIVE,
        };
        for (index, address) in relocation_addresses.iter().enumerate() {
            put_u32(&mut out, *address);
            put_u32(&mut out, (index + 5) as u32);
            put_u16(&mut out, relocation_kind);
        }
        out.resize(section_two_offset, 0);
        out.extend_from_slice(&section_two_bytes);
        out.resize(symbol_offset, 0);
        let symbol = |out: &mut Vec<u8>, name: &[u8], value: u32, section: u16, auxiliary: u8| {
            let mut short = [0u8; 8];
            short[..name.len()].copy_from_slice(name);
            out.extend_from_slice(&short);
            put_u32(out, value);
            put_u16(out, section);
            put_u16(out, 0);
            out.push(storage_class::STATIC);
            out.push(auxiliary);
        };
        symbol(&mut out, b"@feat.00", 0x11, 0xffff, 0);
        symbol(&mut out, b".rsrc$01", 0, 1, 1);
        put_u32(&mut out, section_one as u32);
        put_u16(&mut out, self.data.len() as u16);
        out.extend_from_slice(&[0u8; 12]);
        symbol(&mut out, b".rsrc$02", 0, 2, 1);
        put_u32(&mut out, section_two_bytes.len() as u32);
        out.extend_from_slice(&[0u8; 14]);
        for (index, offset) in data_offsets.iter().enumerate() {
            let name = format!("$R{:06X}", index & 0xff_ffff);
            symbol(&mut out, name.as_bytes(), *offset, 2, 0);
        }
        put_u32(&mut out, 0);
        out
    }
}
