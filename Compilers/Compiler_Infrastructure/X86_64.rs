use crate::compilers::compiler_infrastructure::fail;
use crate::compilers::table_gen::{read_table, TableRecord, TableValue};
use crate::diag::Diagnostic;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

pub const TABLE: &[u8] = include_bytes!("Tables/X86_64.bin");

pub type RegisterId = u16;

pub const NO_REGISTER: RegisterId = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Bits16,
    Bits32,
    Bits64,
}

impl Mode {
    pub fn feature(self) -> &'static str {
        match self {
            Mode::Bits16 => "Is16Bit",
            Mode::Bits32 => "Is32Bit",
            Mode::Bits64 => "Is64Bit",
        }
    }

    pub fn pointer_width(self) -> u32 {
        match self {
            Mode::Bits16 => 16,
            Mode::Bits32 => 32,
            Mode::Bits64 => 64,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Mode::Bits16 => 0,
            Mode::Bits32 => 1,
            Mode::Bits64 => 2,
        }
    }
}

pub const FORM_MASK: u64 = 127;
pub const PSEUDO: u64 = 0;
pub const RAW_FORM: u64 = 1;
pub const ADD_REGISTER_FORM: u64 = 2;
pub const RAW_FORM_MEMORY_OFFSET: u64 = 3;
pub const RAW_FORM_SOURCE: u64 = 4;
pub const RAW_FORM_DESTINATION: u64 = 5;
pub const RAW_FORM_DESTINATION_SOURCE: u64 = 6;
pub const RAW_FORM_IMMEDIATE_8: u64 = 7;
pub const RAW_FORM_IMMEDIATE_16: u64 = 8;
pub const ADD_CONDITION_FORM: u64 = 9;
pub const PREFIX_BYTE: u64 = 10;
pub const MODRM_DESTINATION_REGISTER_CONDITION: u64 = 18;
pub const MODRM_DESTINATION_MEMORY_CONDITION: u64 = 19;
pub const MODRM_DESTINATION_MEMORY_VEX_OPERAND_3_CONDITION: u64 = 20;
pub const MODRM_REGISTER_ZERO: u64 = 21;
pub const MODRM_SOURCE_MEMORY_FORCED_SIB: u64 = 22;
pub const MODRM_DESTINATION_MEMORY_FORCED_SIB: u64 = 23;
pub const MODRM_DESTINATION_MEMORY: u64 = 24;
pub const MODRM_SOURCE_MEMORY: u64 = 25;
pub const MODRM_SOURCE_MEMORY_VEX_OPERAND_3: u64 = 26;
pub const MODRM_SOURCE_MEMORY_OPERAND_4: u64 = 27;
pub const MODRM_SOURCE_MEMORY_CONDITION: u64 = 28;
pub const MODRM_ANY_MEMORY_CONDITION: u64 = 30;
pub const MODRM_ANY_MEMORY: u64 = 31;
pub const MODRM_MEMORY_0: u64 = 32;
pub const MODRM_MEMORY_7: u64 = 39;
pub const MODRM_DESTINATION_REGISTER: u64 = 40;
pub const MODRM_SOURCE_REGISTER: u64 = 41;
pub const MODRM_SOURCE_REGISTER_VEX_OPERAND_3: u64 = 42;
pub const MODRM_SOURCE_REGISTER_OPERAND_4: u64 = 43;
pub const MODRM_SOURCE_REGISTER_CONDITION: u64 = 44;
pub const MODRM_ANY_REGISTER_CONDITION: u64 = 46;
pub const MODRM_ANY_REGISTER: u64 = 47;
pub const MODRM_REGISTER_0: u64 = 48;
pub const MODRM_REGISTER_7: u64 = 55;
pub const MODRM_EXTENSION_0: u64 = 56;
pub const MODRM_EXTENSION_7: u64 = 63;
pub const MODRM_FIXED_FIRST: u64 = 64;
pub const MODRM_FIXED_LAST: u64 = 127;

pub const OPERAND_SIZE_SHIFT: u64 = 7;
pub const OPERAND_SIZE_MASK: u64 = 3 << OPERAND_SIZE_SHIFT;
pub const OPERAND_SIZE_16: u64 = 1 << OPERAND_SIZE_SHIFT;
pub const OPERAND_SIZE_32: u64 = 2 << OPERAND_SIZE_SHIFT;
pub const ADDRESS_SIZE_SHIFT: u64 = 9;
pub const ADDRESS_SIZE_MASK: u64 = 3 << ADDRESS_SIZE_SHIFT;
pub const ADDRESS_SIZE_16: u64 = 1 << ADDRESS_SIZE_SHIFT;
pub const ADDRESS_SIZE_32: u64 = 2 << ADDRESS_SIZE_SHIFT;
pub const ADDRESS_SIZE_64: u64 = 3 << ADDRESS_SIZE_SHIFT;
pub const OPERAND_PREFIX_SHIFT: u64 = 11;
pub const OPERAND_PREFIX_MASK: u64 = 3 << OPERAND_PREFIX_SHIFT;
pub const PREFIX_66: u64 = 1 << OPERAND_PREFIX_SHIFT;
pub const PREFIX_F3: u64 = 2 << OPERAND_PREFIX_SHIFT;
pub const PREFIX_F2: u64 = 3 << OPERAND_PREFIX_SHIFT;
pub const OPCODE_MAP_SHIFT: u64 = 13;
pub const OPCODE_MAP_MASK: u64 = 15 << OPCODE_MAP_SHIFT;
pub const MAP_ONE_BYTE: u64 = 0;
pub const MAP_0F: u64 = 1 << OPCODE_MAP_SHIFT;
pub const MAP_0F38: u64 = 2 << OPCODE_MAP_SHIFT;
pub const MAP_0F3A: u64 = 3 << OPCODE_MAP_SHIFT;
pub const MAP_XOP_8: u64 = 4 << OPCODE_MAP_SHIFT;
pub const MAP_XOP_9: u64 = 5 << OPCODE_MAP_SHIFT;
pub const MAP_XOP_A: u64 = 6 << OPCODE_MAP_SHIFT;
pub const MAP_THREE_D_NOW: u64 = 7 << OPCODE_MAP_SHIFT;
pub const MAP_4: u64 = 8 << OPCODE_MAP_SHIFT;
pub const MAP_5: u64 = 9 << OPCODE_MAP_SHIFT;
pub const MAP_6: u64 = 10 << OPCODE_MAP_SHIFT;
pub const MAP_7: u64 = 11 << OPCODE_MAP_SHIFT;
pub const REX_W: u64 = 1 << 17;
pub const IMMEDIATE_SHIFT: u64 = 18;
pub const IMMEDIATE_MASK: u64 = 15 << IMMEDIATE_SHIFT;
pub const IMMEDIATE_8: u64 = 1 << IMMEDIATE_SHIFT;
pub const IMMEDIATE_8_PC_RELATIVE: u64 = 2 << IMMEDIATE_SHIFT;
pub const IMMEDIATE_8_REGISTER: u64 = 3 << IMMEDIATE_SHIFT;
pub const IMMEDIATE_16: u64 = 4 << IMMEDIATE_SHIFT;
pub const IMMEDIATE_16_PC_RELATIVE: u64 = 5 << IMMEDIATE_SHIFT;
pub const IMMEDIATE_32: u64 = 6 << IMMEDIATE_SHIFT;
pub const IMMEDIATE_32_PC_RELATIVE: u64 = 7 << IMMEDIATE_SHIFT;
pub const IMMEDIATE_32_SIGNED: u64 = 8 << IMMEDIATE_SHIFT;
pub const IMMEDIATE_64: u64 = 9 << IMMEDIATE_SHIFT;
pub const FLOATING_POINT_TYPE_SHIFT: u64 = 22;
pub const FLOATING_POINT_TYPE_MASK: u64 = 7 << FLOATING_POINT_TYPE_SHIFT;
pub const LOCK: u64 = 1 << 25;
pub const REPEAT: u64 = 1 << 26;
pub const ENCODING_SHIFT: u64 = 29;
pub const ENCODING_MASK: u64 = 3 << ENCODING_SHIFT;
pub const ENCODING_VEX: u64 = 1 << ENCODING_SHIFT;
pub const ENCODING_XOP: u64 = 2 << ENCODING_SHIFT;
pub const ENCODING_EVEX: u64 = 3 << ENCODING_SHIFT;
pub const OPCODE_SHIFT: u64 = 31;
pub const VEX_4V: u64 = 1 << 39;
pub const VEX_L: u64 = 1 << 40;
pub const EVEX_K: u64 = 1 << 41;
pub const EVEX_Z: u64 = 1 << 42;
pub const EVEX_L2: u64 = 1 << 43;
pub const EVEX_B: u64 = 1 << 44;
pub const COMPRESSED_DISPLACEMENT_SHIFT: u64 = 45;
pub const COMPRESSED_DISPLACEMENT_MASK: u64 = 7 << COMPRESSED_DISPLACEMENT_SHIFT;
pub const EVEX_ROUNDING_CONTROL: u64 = 1 << 48;
pub const NO_TRACK: u64 = 1 << 49;
pub const EXPLICIT_PREFIX_SHIFT: u64 = 50;
pub const EXPLICIT_PREFIX_MASK: u64 = 3 << EXPLICIT_PREFIX_SHIFT;
pub const EXPLICIT_REX2_PREFIX: u64 = 1 << EXPLICIT_PREFIX_SHIFT;
pub const EXPLICIT_VEX_PREFIX: u64 = 2 << EXPLICIT_PREFIX_SHIFT;
pub const EXPLICIT_EVEX_PREFIX: u64 = 3 << EXPLICIT_PREFIX_SHIFT;
pub const EVEX_NO_FLAGS: u64 = 1 << 52;
pub const TWO_CONDITIONAL_OPERANDS: u64 = 1 << 53;
pub const EVEX_U: u64 = 1 << 54;

pub const PREFIX_OPERAND_SIZE: u32 = 1;
pub const PREFIX_ADDRESS_SIZE: u32 = 1 << 1;
pub const PREFIX_REPEAT_NOT_EQUAL: u32 = 1 << 2;
pub const PREFIX_REPEAT: u32 = 1 << 3;
pub const PREFIX_LOCK: u32 = 1 << 4;
pub const PREFIX_NO_TRACK: u32 = 1 << 5;
pub const PREFIX_USE_REX: u32 = 1 << 6;
pub const PREFIX_USE_REX2: u32 = 1 << 7;
pub const PREFIX_USE_VEX: u32 = 1 << 8;
pub const PREFIX_USE_VEX2: u32 = 1 << 9;
pub const PREFIX_USE_VEX3: u32 = 1 << 10;
pub const PREFIX_USE_EVEX: u32 = 1 << 11;
pub const PREFIX_USE_DISPLACEMENT_8: u32 = 1 << 12;
pub const PREFIX_USE_DISPLACEMENT_32: u32 = 1 << 13;

pub const ADDRESS_BASE: usize = 0;
pub const ADDRESS_SCALE: usize = 1;
pub const ADDRESS_INDEX: usize = 2;
pub const ADDRESS_DISPLACEMENT: usize = 3;
pub const ADDRESS_SEGMENT: usize = 4;
pub const ADDRESS_OPERAND_COUNT: usize = 5;

pub fn form_of(flags: u64) -> u64 {
    flags & FORM_MASK
}

pub fn base_opcode(flags: u64) -> u8 {
    (flags >> OPCODE_SHIFT) as u8
}

pub fn has_immediate(flags: u64) -> bool {
    flags & IMMEDIATE_MASK != 0
}

pub fn immediate_size(flags: u64) -> usize {
    match flags & IMMEDIATE_MASK {
        IMMEDIATE_8 | IMMEDIATE_8_PC_RELATIVE | IMMEDIATE_8_REGISTER => 1,
        IMMEDIATE_16 | IMMEDIATE_16_PC_RELATIVE => 2,
        IMMEDIATE_32 | IMMEDIATE_32_SIGNED | IMMEDIATE_32_PC_RELATIVE => 4,
        IMMEDIATE_64 => 8,
        _ => 0,
    }
}

pub fn immediate_is_pc_relative(flags: u64) -> bool {
    matches!(
        flags & IMMEDIATE_MASK,
        IMMEDIATE_8_PC_RELATIVE | IMMEDIATE_16_PC_RELATIVE | IMMEDIATE_32_PC_RELATIVE
    )
}

pub fn immediate_is_signed(flags: u64) -> bool {
    flags & IMMEDIATE_MASK == IMMEDIATE_32_SIGNED
}

pub fn has_new_data_destination(flags: u64) -> bool {
    flags & OPCODE_MAP_MASK == MAP_4 && flags & EVEX_B != 0 && flags & VEX_4V != 0
}

pub fn memory_operand_number(flags: u64) -> Option<usize> {
    let vex_4v = (flags & VEX_4V != 0) as usize;
    let evex_k = (flags & EVEX_K != 0) as usize;
    match form_of(flags) {
        MODRM_DESTINATION_MEMORY
        | MODRM_DESTINATION_MEMORY_FORCED_SIB
        | MODRM_DESTINATION_MEMORY_CONDITION => Some(has_new_data_destination(flags) as usize),
        MODRM_SOURCE_MEMORY | MODRM_SOURCE_MEMORY_FORCED_SIB => Some(1 + vex_4v + evex_k),
        MODRM_SOURCE_MEMORY_VEX_OPERAND_3 => Some(1 + evex_k),
        MODRM_SOURCE_MEMORY_OPERAND_4 => Some(3),
        MODRM_SOURCE_MEMORY_CONDITION => Some(1 + has_new_data_destination(flags) as usize),
        MODRM_DESTINATION_MEMORY_VEX_OPERAND_3_CONDITION => Some(1),
        MODRM_ANY_MEMORY_CONDITION | MODRM_ANY_MEMORY => Some(vex_4v + evex_k),
        form if (MODRM_MEMORY_0..=MODRM_MEMORY_7).contains(&form) => Some(vex_4v + evex_k),
        _ => None,
    }
}

pub fn compressed_displacement_scale(flags: u64) -> i64 {
    let scale = (flags & COMPRESSED_DISPLACEMENT_MASK) >> COMPRESSED_DISPLACEMENT_SHIFT;
    if scale == 0 {
        0
    } else {
        1 << (scale - 1)
    }
}

pub fn displacement_fits_eight_bits(flags: u64, value: i64) -> Option<i64> {
    let evex = flags & ENCODING_MASK == ENCODING_EVEX;
    let scale = compressed_displacement_scale(flags);
    if !evex || scale == 0 {
        return if (-128..=127).contains(&value) {
            Some(0)
        } else {
            None
        };
    }
    if value & (scale - 1) != 0 {
        return None;
    }
    let compressed = value / scale;
    if !(-128..=127).contains(&compressed) {
        return None;
    }
    Some(compressed - value)
}

#[derive(Clone, Debug)]
pub struct Register {
    pub record: String,
    pub assembly_name: String,
    pub encoding: u16,
    pub sub_registers: Vec<RegisterId>,
    pub super_registers: Vec<RegisterId>,
    pub alternate_names: Vec<String>,
    pub definition_order: u64,
}

#[derive(Clone, Debug, Default)]
pub struct RegisterClass {
    pub record: String,
    pub members: Vec<RegisterId>,
    pub member_set: BTreeSet<RegisterId>,
    pub definition_order: u64,
}

impl RegisterClass {
    pub fn contains(&self, register: RegisterId) -> bool {
        self.member_set.contains(&register)
    }
}

#[derive(Clone, Debug, Default)]
pub struct OperandRecord {
    pub record: String,
    pub print_method: String,
    pub operand_type: String,
    pub sub_operands: Vec<String>,
    pub match_class: String,
    pub register_class: String,
    pub is_register_operand: bool,
    pub is_register_class: bool,
    pub decoder_method: String,
    pub memory_size: i64,
}

#[derive(Clone, Debug)]
pub struct OperandGroup {
    pub name: String,
    pub record: String,
    pub first: usize,
    pub count: usize,
    pub output: bool,
    pub tied_to: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub record: String,
    pub flags: u64,
    pub groups: Vec<OperandGroup>,
    pub operand_records: Vec<String>,
    pub operand_groups: Vec<usize>,
    pub tied_to: Vec<Option<usize>>,
    pub definitions: usize,
    pub assembly: String,
    pub predicates: Vec<String>,
    pub code_generation_only: bool,
    pub assembly_parser_only: bool,
    pub pseudo: bool,
    pub variant_name: String,
    pub two_operand_alias_constraint: String,
    pub match_converter: String,
    pub definition_order: u64,
    pub size: i64,
    pub modes: [bool; 3],
    pub force_disassemble: bool,
    pub disassemble_only: bool,
    pub decoder_namespace: String,
    pub may_load: bool,
    pub may_store: bool,
    pub is_branch: bool,
    pub is_call: bool,
    pub is_return: bool,
    pub is_terminator: bool,
    pub is_indirect_branch: bool,
    pub has_position_order: bool,
}

impl Instruction {
    pub fn operand_count(&self) -> usize {
        self.operand_records.len()
    }

    pub fn operand_bias(&self) -> usize {
        let count = self.operand_count();
        match self.definitions {
            1 => {
                if (count > 1 && self.tied_to.get(1).copied().flatten() == Some(0))
                    || (count == 8 && self.tied_to.get(6).copied().flatten() == Some(0))
                {
                    1
                } else {
                    0
                }
            }
            2 => {
                let tied = |index: usize| self.tied_to.get(index).copied().flatten();
                if (count >= 4 && tied(2) == Some(0) && tied(3) == Some(1))
                    || (count == 9
                        && tied(2) == Some(0)
                        && (tied(3) == Some(1) || tied(8) == Some(1)))
                {
                    2
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    pub fn memory_operand_index(&self) -> Option<usize> {
        memory_operand_number(self.flags).map(|number| number + self.operand_bias())
    }

    pub fn valid_in(&self, mode: Mode) -> bool {
        self.modes[mode.index()]
    }

    pub fn named_group(&self, name: &str) -> Option<usize> {
        self.groups.iter().position(|group| group.name == name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliasArgument {
    Operand {
        name: String,
        record: String,
    },
    Immediate(i64),
    Register(RegisterId),
    Complex {
        record: String,
        arguments: Vec<AliasArgument>,
    },
}

#[derive(Clone, Debug)]
pub struct InstructionAlias {
    pub record: String,
    pub assembly: String,
    pub result: usize,
    pub arguments: Vec<AliasArgument>,
    pub emit_priority: i64,
    pub predicates: Vec<String>,
    pub variant_name: String,
    pub use_converter: bool,
    pub definition_order: u64,
    pub modes: [bool; 3],
}

#[derive(Clone, Debug)]
pub struct MnemonicAlias {
    pub from: String,
    pub to: String,
    pub modes: [bool; 3],
    pub has_predicates: bool,
    pub variant_name: String,
}

#[derive(Clone, Debug)]
pub struct AssemblyOperandClass {
    pub record: String,
    pub name: String,
    pub super_classes: Vec<String>,
    pub predicate_method: String,
    pub render_method: String,
}

pub struct Target {
    pub registers: Vec<Register>,
    pub register_by_record: BTreeMap<String, RegisterId>,
    pub register_by_name: BTreeMap<String, RegisterId>,
    pub register_classes: BTreeMap<String, RegisterClass>,
    pub instructions: Vec<Instruction>,
    pub instruction_by_record: BTreeMap<String, usize>,
    pub operand_records: BTreeMap<String, OperandRecord>,
    pub aliases: Vec<InstructionAlias>,
    pub mnemonic_aliases: Vec<MnemonicAlias>,
    pub assembly_operand_classes: Vec<AssemblyOperandClass>,
    pub token_aliases: Vec<(String, String)>,
    pub records: BTreeMap<String, TableRecord>,
}

pub fn table_text(record: &TableRecord, field: &str) -> String {
    match record.field(field) {
        Some(TableValue::String(text)) | Some(TableValue::Code(text)) => text.clone(),
        _ => String::new(),
    }
}

pub fn table_integer(record: &TableRecord, field: &str) -> i64 {
    match record.field(field) {
        Some(TableValue::Integer(value)) => *value,
        Some(TableValue::Bits(bits)) => bits_value(bits) as i64,
        Some(TableValue::Bit(bit)) => *bit as i64,
        _ => 0,
    }
}

pub fn table_bit(record: &TableRecord, field: &str) -> bool {
    match record.field(field) {
        Some(TableValue::Bit(bit)) => *bit,
        Some(TableValue::Integer(value)) => *value != 0,
        _ => false,
    }
}

pub fn bits_value(bits: &[TableValue]) -> u64 {
    let mut value = 0u64;
    for (index, bit) in bits.iter().enumerate().take(64) {
        if *bit == TableValue::Bit(true) {
            value |= 1u64 << index;
        }
    }
    value
}

pub fn table_record_name(record: &TableRecord, field: &str) -> String {
    match record.field(field) {
        Some(TableValue::Record(name)) => name.clone(),
        _ => String::new(),
    }
}

pub fn table_list(record: &TableRecord, field: &str) -> Vec<TableValue> {
    match record.field(field) {
        Some(TableValue::List(items)) => items.clone(),
        _ => Vec::new(),
    }
}

pub fn table_record_list(record: &TableRecord, field: &str) -> Vec<String> {
    table_list(record, field)
        .into_iter()
        .filter_map(|item| match item {
            TableValue::Record(name) => Some(name),
            _ => None,
        })
        .collect()
}

pub fn table_text_list(record: &TableRecord, field: &str) -> Vec<String> {
    table_list(record, field)
        .into_iter()
        .filter_map(|item| match item {
            TableValue::String(text) => Some(text),
            _ => None,
        })
        .collect()
}

fn dag_arguments(value: Option<&TableValue>) -> Vec<(TableValue, Option<String>)> {
    match value {
        Some(TableValue::Dag { arguments, .. }) => arguments.clone(),
        _ => Vec::new(),
    }
}

fn dag_operator(value: &TableValue) -> String {
    match value {
        TableValue::Dag { operator, .. } => match operator.as_ref() {
            TableValue::Record(name) => name.clone(),
            _ => String::new(),
        },
        _ => String::new(),
    }
}

fn condition_holds(condition: &TableValue, features: &BTreeSet<&str>) -> bool {
    match condition {
        TableValue::Record(feature) => features.contains(feature.as_str()),
        TableValue::Dag {
            operator,
            arguments,
            ..
        } => {
            let TableValue::Record(operator) = operator.as_ref() else {
                return true;
            };
            match operator.as_str() {
                "all_of" => arguments
                    .iter()
                    .all(|(argument, _)| condition_holds(argument, features)),
                "any_of" => arguments
                    .iter()
                    .any(|(argument, _)| condition_holds(argument, features)),
                "not" => !arguments
                    .iter()
                    .all(|(argument, _)| condition_holds(argument, features)),
                _ => true,
            }
        }
        _ => true,
    }
}

fn mode_validity(records: &BTreeMap<String, TableRecord>, predicates: &[String]) -> [bool; 3] {
    let mut modes = [true; 3];
    for (index, mode) in [Mode::Bits16, Mode::Bits32, Mode::Bits64]
        .iter()
        .enumerate()
    {
        let features: BTreeSet<&str> = [mode.feature()].into_iter().collect();
        for predicate in predicates {
            let Some(record) = records.get(predicate) else {
                continue;
            };
            if let Some(condition @ TableValue::Dag { .. }) = record.field("AssemblerCondDag") {
                if !condition_holds(condition, &features) {
                    modes[index] = false;
                }
            }
        }
    }
    modes
}

fn read_records() -> Result<BTreeMap<String, TableRecord>, Diagnostic> {
    let mut records = BTreeMap::new();
    for bytes in [crate::compilers::x86::TABLE, TABLE] {
        let table = read_table(bytes)?;
        for record in table.records {
            records.entry(record.name.clone()).or_insert(record);
        }
    }
    Ok(records)
}

fn sequence_names(format: &str, start: i64, end: i64, stride: i64) -> Vec<String> {
    let mut names = Vec::new();
    let stride = if stride == 0 { 1 } else { stride };
    let mut at = start;
    loop {
        if (stride > 0 && at > end) || (stride < 0 && at < end) {
            break;
        }
        names.push(format.replace("%u", &at.to_string()));
        at += stride;
    }
    names
}

impl Target {
    fn class_members(&self, value: &TableValue, depth: usize) -> Vec<RegisterId> {
        if depth > 64 {
            return Vec::new();
        }
        match value {
            TableValue::Record(name) => {
                if let Some(register) = self.register_by_record.get(name) {
                    return vec![*register];
                }
                if let Some(class) = self.register_classes.get(name) {
                    return class.members.clone();
                }
                match self
                    .records
                    .get(name)
                    .and_then(|record| record.field("MemberList"))
                {
                    Some(list) => self.class_members(&list.clone(), depth + 1),
                    None => Vec::new(),
                }
            }
            TableValue::Dag { arguments, .. } => {
                let operator = dag_operator(value);
                let sets: Vec<Vec<RegisterId>> = arguments
                    .iter()
                    .map(|(argument, _)| self.class_members(argument, depth + 1))
                    .collect();
                let integer = |index: usize| match arguments.get(index) {
                    Some((TableValue::Integer(value), _)) => *value,
                    _ => 0,
                };
                match operator.as_str() {
                    "add" => {
                        let mut out = Vec::new();
                        let mut seen = BTreeSet::new();
                        for set in sets {
                            for register in set {
                                if seen.insert(register) {
                                    out.push(register);
                                }
                            }
                        }
                        out
                    }
                    "sub" => {
                        let mut removed = BTreeSet::new();
                        for set in sets.iter().skip(1) {
                            removed.extend(set.iter().copied());
                        }
                        sets.first()
                            .map(|first| {
                                first
                                    .iter()
                                    .copied()
                                    .filter(|register| !removed.contains(register))
                                    .collect()
                            })
                            .unwrap_or_default()
                    }
                    "and" => {
                        let mut kept: Vec<RegisterId> = sets.first().cloned().unwrap_or_default();
                        for set in sets.iter().skip(1) {
                            let other: BTreeSet<RegisterId> = set.iter().copied().collect();
                            kept.retain(|register| other.contains(register));
                        }
                        kept
                    }
                    "sequence" => {
                        let format = match arguments.first() {
                            Some((TableValue::String(text), _)) => text.clone(),
                            _ => String::new(),
                        };
                        let stride = if arguments.len() > 3 { integer(3) } else { 1 };
                        sequence_names(&format, integer(1), integer(2), stride)
                            .into_iter()
                            .filter_map(|name| self.register_by_record.get(&name).copied())
                            .collect()
                    }
                    "trunc" => {
                        let count = integer(1).max(0) as usize;
                        sets.first()
                            .map(|first| first.iter().copied().take(count).collect())
                            .unwrap_or_default()
                    }
                    "shl" => {
                        let count = integer(1).max(0) as usize;
                        sets.first()
                            .map(|first| first.iter().copied().skip(count).collect())
                            .unwrap_or_default()
                    }
                    _ => Vec::new(),
                }
            }
            _ => Vec::new(),
        }
    }

    fn load() -> Result<Target, Diagnostic> {
        let records = read_records()?;
        let mut target = Target {
            registers: vec![Register {
                record: "NoRegister".to_string(),
                assembly_name: String::new(),
                encoding: 0,
                sub_registers: Vec::new(),
                super_registers: Vec::new(),
                alternate_names: Vec::new(),
                definition_order: 0,
            }],
            register_by_record: BTreeMap::new(),
            register_by_name: BTreeMap::new(),
            register_classes: BTreeMap::new(),
            instructions: Vec::new(),
            instruction_by_record: BTreeMap::new(),
            operand_records: BTreeMap::new(),
            aliases: Vec::new(),
            mnemonic_aliases: Vec::new(),
            assembly_operand_classes: Vec::new(),
            token_aliases: Vec::new(),
            records: BTreeMap::new(),
        };
        for record in records.values() {
            if record.is_a("Register") && !record.is_a("RegisterClass") {
                let identifier = target.registers.len() as RegisterId;
                target.registers.push(Register {
                    record: record.name.clone(),
                    assembly_name: table_text(record, "AsmName"),
                    encoding: table_integer(record, "HWEncoding") as u16,
                    sub_registers: Vec::new(),
                    super_registers: Vec::new(),
                    alternate_names: table_text_list(record, "AltNames"),
                    definition_order: record.definition_order,
                });
                target
                    .register_by_record
                    .insert(record.name.clone(), identifier);
            }
        }
        for identifier in 1..target.registers.len() {
            let record = &records[&target.registers[identifier].record];
            let subs: Vec<RegisterId> = table_record_list(record, "SubRegs")
                .iter()
                .filter_map(|name| target.register_by_record.get(name).copied())
                .collect();
            for sub in &subs {
                target.registers[*sub as usize]
                    .super_registers
                    .push(identifier as RegisterId);
            }
            target.registers[identifier].sub_registers = subs;
            let name = target.registers[identifier].assembly_name.clone();
            if !name.is_empty() {
                target
                    .register_by_name
                    .entry(name)
                    .or_insert(identifier as RegisterId);
            }
        }
        target.records = records;
        let class_names: Vec<String> = target
            .records
            .values()
            .filter(|record| record.is_a("RegisterClass"))
            .map(|record| record.name.clone())
            .collect();
        for _ in 0..4 {
            for name in &class_names {
                let Some(list) = target.records[name].field("MemberList").cloned() else {
                    continue;
                };
                let members = target.class_members(&list, 0);
                let member_set = members.iter().copied().collect();
                let definition_order = target.records[name].definition_order;
                target.register_classes.insert(
                    name.clone(),
                    RegisterClass {
                        record: name.clone(),
                        members,
                        member_set,
                        definition_order,
                    },
                );
            }
        }
        for record in target.records.values() {
            if record.is_a("Operand") || record.is_a("RegisterOperand") {
                let sub_operands = dag_arguments(record.field("MIOperandInfo"))
                    .into_iter()
                    .filter_map(|(argument, _)| match argument {
                        TableValue::Record(name) => Some(name),
                        _ => None,
                    })
                    .collect();
                target.operand_records.insert(
                    record.name.clone(),
                    OperandRecord {
                        record: record.name.clone(),
                        print_method: table_text(record, "PrintMethod"),
                        operand_type: table_text(record, "OperandType"),
                        sub_operands,
                        match_class: table_record_name(record, "ParserMatchClass"),
                        register_class: table_record_name(record, "RegClass"),
                        is_register_operand: record.is_a("RegisterOperand"),
                        is_register_class: false,
                        decoder_method: table_text(record, "DecoderMethod"),
                        memory_size: table_integer(record, "Size"),
                    },
                );
            } else if record.is_a("RegisterClass") {
                target.operand_records.insert(
                    record.name.clone(),
                    OperandRecord {
                        record: record.name.clone(),
                        print_method: "printOperand".to_string(),
                        operand_type: "OPERAND_REGISTER".to_string(),
                        sub_operands: Vec::new(),
                        match_class: String::new(),
                        register_class: record.name.clone(),
                        is_register_operand: false,
                        is_register_class: true,
                        decoder_method: String::new(),
                        memory_size: 0,
                    },
                );
            } else if record.is_a("AsmOperandClass") {
                target.assembly_operand_classes.push(AssemblyOperandClass {
                    record: record.name.clone(),
                    name: table_text(record, "Name"),
                    super_classes: table_record_list(record, "SuperClasses"),
                    predicate_method: table_text(record, "PredicateMethod"),
                    render_method: table_text(record, "RenderMethod"),
                });
            } else if record.is_a("TokenAlias") {
                target.token_aliases.push((
                    table_text(record, "FromToken"),
                    table_text(record, "ToToken"),
                ));
            }
        }
        let instruction_names: Vec<String> = target
            .records
            .values()
            .filter(|record| record.is_a("Instruction") && record.is_a("X86Inst"))
            .map(|record| record.name.clone())
            .collect();
        for name in instruction_names {
            let instruction = target.instruction_from(&target.records[&name]);
            target
                .instruction_by_record
                .insert(name, target.instructions.len());
            target.instructions.push(instruction);
        }
        let alias_names: Vec<String> = target
            .records
            .values()
            .filter(|record| record.is_a("InstAlias"))
            .map(|record| record.name.clone())
            .collect();
        for name in alias_names {
            if let Some(alias) = target.alias_from(&target.records[&name]) {
                target.aliases.push(alias);
            }
        }
        let mnemonic_names: Vec<String> = target
            .records
            .values()
            .filter(|record| record.is_a("MnemonicAlias"))
            .map(|record| record.name.clone())
            .collect();
        for name in mnemonic_names {
            let record = &target.records[&name];
            let predicates = table_record_list(record, "Predicates");
            let has_predicates = predicates.iter().any(|predicate| {
                matches!(
                    target
                        .records
                        .get(predicate)
                        .and_then(|held| held.field("AssemblerCondDag")),
                    Some(TableValue::Dag { .. })
                )
            });
            target.mnemonic_aliases.push(MnemonicAlias {
                from: table_text(record, "FromMnemonic"),
                to: table_text(record, "ToMnemonic"),
                modes: mode_validity(&target.records, &predicates),
                has_predicates,
                variant_name: table_text(record, "AsmVariantName"),
            });
        }
        Ok(target)
    }

    fn instruction_from(&self, record: &TableRecord) -> Instruction {
        let mut groups = Vec::new();
        let mut operand_records = Vec::new();
        let mut operand_groups = Vec::new();
        let mut definitions = 0;
        for (field, output) in [("OutOperandList", true), ("InOperandList", false)] {
            for (argument, name) in dag_arguments(record.field(field)) {
                let TableValue::Record(operand) = argument else {
                    continue;
                };
                let sub_operands = self
                    .operand_records
                    .get(&operand)
                    .map(|held| held.sub_operands.clone())
                    .unwrap_or_default();
                let first = operand_records.len();
                if sub_operands.is_empty() {
                    operand_records.push(operand.clone());
                    operand_groups.push(groups.len());
                } else {
                    for sub in &sub_operands {
                        operand_records.push(sub.clone());
                        operand_groups.push(groups.len());
                    }
                }
                let count = operand_records.len() - first;
                if output {
                    definitions += count;
                }
                groups.push(OperandGroup {
                    name: name.unwrap_or_default(),
                    record: operand,
                    first,
                    count,
                    output,
                    tied_to: None,
                });
            }
        }
        let mut tied_to = vec![None; operand_records.len()];
        for constraint in table_text(record, "Constraints").split(',') {
            let constraint = constraint.trim();
            let Some((left, right)) = constraint.split_once('=') else {
                continue;
            };
            let left = left.trim().trim_start_matches('$');
            let right = right.trim().trim_start_matches('$');
            let left_group = groups.iter().position(|group| group.name == left);
            let right_group = groups.iter().position(|group| group.name == right);
            if let (Some(left_group), Some(right_group)) = (left_group, right_group) {
                let (tied, target) = if groups[left_group].first > groups[right_group].first {
                    (left_group, right_group)
                } else {
                    (right_group, left_group)
                };
                groups[tied].tied_to = Some(target);
                let tied_index = groups[tied].first;
                tied_to[tied_index] = Some(groups[target].first);
            }
        }
        let predicates = table_record_list(record, "Predicates");
        let modes = mode_validity(&self.records, &predicates);
        let flags = match record.field("TSFlags") {
            Some(TableValue::Bits(bits)) => bits_value(bits),
            _ => 0,
        };
        Instruction {
            record: record.name.clone(),
            flags,
            groups,
            operand_records,
            operand_groups,
            tied_to,
            definitions,
            assembly: table_text(record, "AsmString"),
            predicates,
            code_generation_only: table_bit(record, "isCodeGenOnly"),
            assembly_parser_only: table_bit(record, "isAsmParserOnly"),
            pseudo: table_bit(record, "isPseudo"),
            variant_name: table_text(record, "AsmVariantName"),
            two_operand_alias_constraint: table_text(record, "TwoOperandAliasConstraint"),
            match_converter: table_text(record, "AsmMatchConverter"),
            definition_order: record.definition_order,
            size: table_integer(record, "Size"),
            modes,
            force_disassemble: table_bit(record, "ForceDisassemble"),
            disassemble_only: record.is_a("DisassembleOnly"),
            decoder_namespace: table_text(record, "DecoderNamespace"),
            may_load: table_bit(record, "mayLoad"),
            may_store: table_bit(record, "mayStore"),
            is_branch: table_bit(record, "isBranch"),
            is_call: table_bit(record, "isCall"),
            is_return: table_bit(record, "isReturn"),
            is_terminator: table_bit(record, "isTerminator"),
            is_indirect_branch: table_bit(record, "isIndirectBranch"),
            has_position_order: table_bit(record, "HasPositionOrder"),
        }
    }

    fn alias_argument(&self, value: &TableValue, name: &Option<String>) -> Option<AliasArgument> {
        match value {
            TableValue::Integer(number) => Some(AliasArgument::Immediate(*number)),
            TableValue::Bits(bits) => Some(AliasArgument::Immediate(bits_value(bits) as i64)),
            TableValue::Record(record) => {
                if let Some(name) = name {
                    return Some(AliasArgument::Operand {
                        name: name.clone(),
                        record: record.clone(),
                    });
                }
                if record == "zero_reg" {
                    return Some(AliasArgument::Register(NO_REGISTER));
                }
                self.register_by_record
                    .get(record)
                    .map(|register| AliasArgument::Register(*register))
            }
            TableValue::Dag { arguments, .. } => {
                let record = dag_operator(value);
                let mut converted = Vec::new();
                for (argument, argument_name) in arguments {
                    converted.push(self.alias_argument(argument, argument_name)?);
                }
                Some(AliasArgument::Complex {
                    record,
                    arguments: converted,
                })
            }
            _ => None,
        }
    }

    fn alias_from(&self, record: &TableRecord) -> Option<InstructionAlias> {
        let result_value = record.field("ResultInst")?;
        let result_name = dag_operator(result_value);
        let result = *self.instruction_by_record.get(&result_name)?;
        let mut arguments = Vec::new();
        for (argument, name) in dag_arguments(Some(result_value)) {
            arguments.push(self.alias_argument(&argument, &name)?);
        }
        let predicates = table_record_list(record, "Predicates");
        Some(InstructionAlias {
            record: record.name.clone(),
            assembly: table_text(record, "AsmString"),
            result,
            arguments,
            emit_priority: table_integer(record, "EmitPriority"),
            modes: mode_validity(&self.records, &predicates),
            predicates,
            variant_name: table_text(record, "AsmVariantName"),
            use_converter: table_bit(record, "UseInstAsmMatchConverter"),
            definition_order: record.definition_order,
        })
    }

    pub fn instruction(&self, record: &str) -> Option<usize> {
        self.instruction_by_record.get(record).copied()
    }

    pub fn register(&self, record: &str) -> RegisterId {
        self.register_by_record
            .get(record)
            .copied()
            .unwrap_or(NO_REGISTER)
    }

    pub fn register_name(&self, register: RegisterId) -> &str {
        &self.registers[register as usize].assembly_name
    }

    pub fn encoding(&self, register: RegisterId) -> u16 {
        self.registers[register as usize].encoding
    }

    pub fn class(&self, name: &str) -> Option<&RegisterClass> {
        self.register_classes.get(name)
    }

    pub fn in_class(&self, name: &str, register: RegisterId) -> bool {
        register != NO_REGISTER
            && self
                .register_classes
                .get(name)
                .is_some_and(|class| class.contains(register))
    }

    pub fn family(&self, register: RegisterId) -> BTreeSet<RegisterId> {
        let mut root = register;
        let mut guard = 0;
        while let Some(parent) = self.registers[root as usize].super_registers.first() {
            if !self.is_general_purpose(*parent) {
                break;
            }
            root = *parent;
            guard += 1;
            if guard > 8 {
                break;
            }
        }
        let mut found = BTreeSet::new();
        let mut stack = vec![root];
        while let Some(at) = stack.pop() {
            if found.insert(at) {
                stack.extend(self.registers[at as usize].sub_registers.iter().copied());
            }
        }
        found
    }

    pub fn is_general_purpose(&self, register: RegisterId) -> bool {
        ["GR8", "GR16", "GR32", "GR64"]
            .iter()
            .any(|class| self.in_class(class, register))
            || ["RIP", "EIP", "IP"].contains(&self.registers[register as usize].record.as_str())
    }

    pub fn sub_super_register(&self, register: RegisterId, size: u32, high: bool) -> RegisterId {
        if register == NO_REGISTER {
            return NO_REGISTER;
        }
        let class = match size {
            8 => "GR8",
            16 => "GR16",
            32 => "GR32",
            _ => "GR64",
        };
        let family = self.family(register);
        let mut candidates: Vec<RegisterId> = family
            .into_iter()
            .filter(|member| self.in_class(class, *member))
            .collect();
        if size == 8 {
            let high_names = ["AH", "BH", "CH", "DH"];
            candidates.retain(|member| {
                high_names.contains(&self.registers[*member as usize].record.as_str()) == high
            });
        }
        candidates.first().copied().unwrap_or(NO_REGISTER)
    }

    pub fn is_extended_apx(&self, register: RegisterId) -> bool {
        let record = &self.registers[register as usize].record;
        let digits: String = record
            .trim_start_matches('R')
            .chars()
            .take_while(|character| character.is_ascii_digit())
            .collect();
        record.starts_with('R')
            && digits
                .parse::<u32>()
                .is_ok_and(|number| (16..=31).contains(&number))
            && self.is_general_purpose(register)
    }

    pub fn is_extended(&self, register: RegisterId) -> bool {
        if register == NO_REGISTER {
            return false;
        }
        if self.is_extended_apx(register) {
            return true;
        }
        let record = self.registers[register as usize].record.as_str();
        for prefix in ["XMM", "YMM", "ZMM"] {
            if let Some(number) = record
                .strip_prefix(prefix)
                .and_then(|rest| rest.parse::<u32>().ok())
            {
                return number >= 8;
            }
        }
        for prefix in ["CR", "DR"] {
            if let Some(number) = record
                .strip_prefix(prefix)
                .and_then(|rest| rest.parse::<u32>().ok())
            {
                return (8..=15).contains(&number);
            }
        }
        if let Some(rest) = record.strip_prefix('R') {
            let digits: String = rest
                .chars()
                .take_while(|character| character.is_ascii_digit())
                .collect();
            let suffix = &rest[digits.len()..];
            if let Ok(number) = digits.parse::<u32>() {
                return (8..=15).contains(&number) && ["", "D", "W", "B"].contains(&suffix);
            }
        }
        false
    }

    pub fn can_use_apx_extended_register(&self, opcode: usize) -> bool {
        let instruction = &self.instructions[opcode];
        let flags = instruction.flags;
        let encoding = flags & ENCODING_MASK;
        if encoding == ENCODING_EVEX {
            return true;
        }
        let record = instruction.record.as_str();
        if form_of(flags) == PSEUDO {
            return matches!(
                record,
                "MOV32r0"
                    | "MOV32r1"
                    | "MOV32r_1"
                    | "MOV32ri64"
                    | "ADD8rr_DB"
                    | "ADD16rr_DB"
                    | "ADD32rr_DB"
                    | "ADD64rr_DB"
                    | "ADD8ri_DB"
                    | "ADD16ri_DB"
                    | "ADD32ri_DB"
                    | "ADD64ri32_DB"
            );
        }
        if matches!(
            record,
            "XSAVE"
                | "XSAVE64"
                | "XSAVEOPT"
                | "XSAVEOPT64"
                | "XSAVEC"
                | "XSAVEC64"
                | "XSAVES"
                | "XSAVES64"
                | "XRSTOR"
                | "XRSTOR64"
                | "XRSTORS"
                | "XRSTORS64"
        ) {
            return false;
        }
        let map = flags & OPCODE_MAP_MASK;
        encoding == 0 && (map == MAP_ONE_BYTE || map == MAP_0F)
    }

    pub fn is_low_byte_needing_rex(&self, register: RegisterId) -> bool {
        matches!(
            self.registers[register as usize].record.as_str(),
            "SPL" | "BPL" | "SIL" | "DIL"
        )
    }

    pub fn is_high_byte(&self, register: RegisterId) -> bool {
        matches!(
            self.registers[register as usize].record.as_str(),
            "AH" | "BH" | "CH" | "DH"
        )
    }

    pub fn vector_number(&self, register: RegisterId, prefix: &str) -> Option<u32> {
        self.registers[register as usize]
            .record
            .strip_prefix(prefix)
            .and_then(|rest| rest.parse::<u32>().ok())
    }
}

pub fn target() -> Result<&'static Target, Diagnostic> {
    static TARGET: OnceLock<Result<Target, Diagnostic>> = OnceLock::new();
    TARGET
        .get_or_init(Target::load)
        .as_ref()
        .map_err(|problem| problem.clone())
}

pub fn target_failure(reason: &str) -> Diagnostic {
    fail("EI300", "The x86 instruction tables could not be loaded.")
        .with_context(format!("Reason: {reason}"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
    LogicalNot,
    Plus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    And,
    Or,
    Xor,
    ShiftLeft,
    ArithmeticShiftRight,
    LogicalShiftRight,
    LogicalAnd,
    LogicalOr,
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    OrNot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Constant(i64),
    Symbol {
        name: String,
        specifier: Option<String>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Register(RegisterId),
}

impl Expression {
    pub fn constant_value(&self) -> Option<i64> {
        match self {
            Expression::Constant(value) => Some(*value),
            _ => None,
        }
    }

    pub fn symbol(name: &str) -> Expression {
        Expression::Symbol {
            name: name.to_string(),
            specifier: None,
        }
    }

    pub fn sum(left: Expression, right: Expression) -> Expression {
        Expression::Binary {
            operator: BinaryOperator::Add,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineOperand {
    Register(RegisterId),
    Immediate(i64),
    Expression(Expression),
}

impl MachineOperand {
    pub fn register(&self) -> RegisterId {
        match self {
            MachineOperand::Register(register) => *register,
            _ => NO_REGISTER,
        }
    }

    pub fn immediate(&self) -> i64 {
        match self {
            MachineOperand::Immediate(value) => *value,
            _ => 0,
        }
    }

    pub fn is_register(&self) -> bool {
        matches!(self, MachineOperand::Register(_))
    }

    pub fn is_immediate(&self) -> bool {
        matches!(self, MachineOperand::Immediate(_))
    }

    pub fn is_expression(&self) -> bool {
        matches!(self, MachineOperand::Expression(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineInstruction {
    pub opcode: usize,
    pub operands: Vec<MachineOperand>,
    pub flags: u32,
}

impl MachineInstruction {
    pub fn new(opcode: usize) -> MachineInstruction {
        MachineInstruction {
            opcode,
            operands: Vec::new(),
            flags: 0,
        }
    }

    pub fn operand(&self, index: usize) -> &MachineOperand {
        static EMPTY: MachineOperand = MachineOperand::Register(NO_REGISTER);
        self.operands.get(index).unwrap_or(&EMPTY)
    }

    pub fn register_at(&self, index: usize) -> RegisterId {
        self.operand(index).register()
    }

    pub fn immediate_at(&self, index: usize) -> i64 {
        self.operand(index).immediate()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FixupKind {
    None,
    Data1,
    Data2,
    Data4,
    Data8,
    SectionRelative4,
    RipRelative4,
    RipRelative4MoveLoad,
    RipRelative4MoveLoadRex2,
    RipRelative4Relax,
    RipRelative4RelaxRex,
    RipRelative4RelaxRex2,
    RipRelative4RelaxEvex,
    Signed4,
    Signed4Relax,
    GlobalOffsetTable,
    Branch4PcRelative,
    Literal(u32),
}

impl FixupKind {
    pub fn size(self) -> usize {
        match self {
            FixupKind::None => 0,
            FixupKind::Data1 => 1,
            FixupKind::Data2 => 2,
            FixupKind::Data8 => 8,
            FixupKind::Literal(42) => 8,
            _ => 4,
        }
    }

    pub fn name(self) -> String {
        match self {
            FixupKind::None => "FK_NONE".to_string(),
            FixupKind::Data1 => "FK_Data_1".to_string(),
            FixupKind::Data2 => "FK_Data_2".to_string(),
            FixupKind::Data4 => "FK_Data_4".to_string(),
            FixupKind::Data8 => "FK_Data_8".to_string(),
            FixupKind::SectionRelative4 => "FK_SecRel_4".to_string(),
            FixupKind::RipRelative4 => "reloc_riprel_4byte".to_string(),
            FixupKind::RipRelative4MoveLoad => "reloc_riprel_4byte_movq_load".to_string(),
            FixupKind::RipRelative4MoveLoadRex2 => "reloc_riprel_4byte_movq_load_rex2".to_string(),
            FixupKind::RipRelative4Relax => "reloc_riprel_4byte_relax".to_string(),
            FixupKind::RipRelative4RelaxRex => "reloc_riprel_4byte_relax_rex".to_string(),
            FixupKind::RipRelative4RelaxRex2 => "reloc_riprel_4byte_relax_rex2".to_string(),
            FixupKind::RipRelative4RelaxEvex => "reloc_riprel_4byte_relax_evex".to_string(),
            FixupKind::Signed4 => "reloc_signed_4byte".to_string(),
            FixupKind::Signed4Relax => "reloc_signed_4byte_relax".to_string(),
            FixupKind::GlobalOffsetTable => "reloc_global_offset_table".to_string(),
            FixupKind::Branch4PcRelative => "reloc_branch_4byte_pcrel".to_string(),
            FixupKind::Literal(number) => format!("fixup_{number}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fixup {
    pub offset: usize,
    pub expression: Expression,
    pub kind: FixupKind,
    pub pc_relative: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Encoded {
    pub bytes: Vec<u8>,
    pub fixups: Vec<Fixup>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrefixKind {
    None,
    Rex,
    Rex2,
    Xop,
    Vex2,
    Vex3,
    Evex,
}

#[derive(Clone, Copy, Debug)]
struct PrefixHelper {
    w: u8,
    r: u8,
    x: u8,
    b: u8,
    m: u8,
    r2: u8,
    x2: u8,
    b2: u8,
    vex_4v: u8,
    vex_l: u8,
    vex_pp: u8,
    vex_5m: u8,
    evex_z: u8,
    evex_l2: u8,
    evex_b: u8,
    evex_v2: u8,
    evex_aaa: u8,
    kind: PrefixKind,
}

impl PrefixHelper {
    fn new() -> PrefixHelper {
        PrefixHelper {
            w: 0,
            r: 0,
            x: 0,
            b: 0,
            m: 0,
            r2: 0,
            x2: 0,
            b2: 0,
            vex_4v: 0,
            vex_l: 0,
            vex_pp: 0,
            vex_5m: 0,
            evex_z: 0,
            evex_l2: 0,
            evex_b: 0,
            evex_v2: 0,
            evex_aaa: 0,
            kind: PrefixKind::None,
        }
    }

    fn set_r(&mut self, encoding: u16) {
        self.r = ((encoding >> 3) & 1) as u8;
    }

    fn set_r2(&mut self, encoding: u16) {
        self.r2 = ((encoding >> 4) & 1) as u8;
    }

    fn set_x(&mut self, encoding: u16) {
        self.x = ((encoding >> 3) & 1) as u8;
    }

    fn set_x2(&mut self, encoding: u16) {
        self.x2 = ((encoding >> 4) & 1) as u8;
    }

    fn set_b(&mut self, encoding: u16) {
        self.b = ((encoding >> 3) & 1) as u8;
    }

    fn set_b2(&mut self, encoding: u16) {
        self.b2 = ((encoding >> 4) & 1) as u8;
    }

    fn set_4v(&mut self, encoding: u16) {
        self.vex_4v = (encoding & 15) as u8;
    }

    fn set_v2(&mut self, encoding: u16) {
        self.evex_v2 = ((encoding >> 4) & 1) as u8;
    }

    fn determine_optimal_kind(&mut self) -> PrefixKind {
        self.kind = match self.kind {
            PrefixKind::None => {
                if self.r2 | self.x2 | self.b2 != 0 {
                    PrefixKind::Rex2
                } else if self.w | self.r | self.x | self.b != 0 {
                    PrefixKind::Rex
                } else {
                    PrefixKind::None
                }
            }
            PrefixKind::Rex => {
                if self.r2 | self.x2 | self.b2 != 0 {
                    PrefixKind::Rex2
                } else {
                    PrefixKind::Rex
                }
            }
            PrefixKind::Vex2 => {
                if self.w | self.x | self.b != 0 || self.vex_5m != 1 {
                    PrefixKind::Vex3
                } else {
                    PrefixKind::Vex2
                }
            }
            other => other,
        };
        self.kind
    }

    fn emit(&self, out: &mut Vec<u8>) {
        let first = ((!self.r & 1) << 7) | ((!self.x & 1) << 6) | ((!self.b & 1) << 5);
        let last = ((!self.vex_4v & 15) << 3) | (self.vex_l << 2) | self.vex_pp;
        match self.kind {
            PrefixKind::None => {}
            PrefixKind::Rex => {
                out.push(0x40 | (self.w << 3) | (self.r << 2) | (self.x << 1) | self.b)
            }
            PrefixKind::Rex2 => {
                out.push(0xd5);
                out.push(
                    (self.m << 7)
                        | (self.r2 << 6)
                        | (self.x2 << 5)
                        | (self.b2 << 4)
                        | (self.w << 3)
                        | (self.r << 2)
                        | (self.x << 1)
                        | self.b,
                );
            }
            PrefixKind::Vex2 => {
                out.push(0xc5);
                out.push(((!self.r & 1) << 7) | last);
            }
            PrefixKind::Vex3 | PrefixKind::Xop => {
                out.push(if self.kind == PrefixKind::Vex3 {
                    0xc4
                } else {
                    0x8f
                });
                out.push(first | self.vex_5m);
                out.push((self.w << 7) | last);
            }
            PrefixKind::Evex => {
                out.push(0x62);
                out.push(first | ((!self.r2 & 1) << 4) | (self.b2 << 3) | self.vex_5m);
                out.push(
                    (self.w << 7)
                        | ((!self.vex_4v & 15) << 3)
                        | ((!self.x2 & 1) << 2)
                        | self.vex_pp,
                );
                out.push(
                    (self.evex_z << 7)
                        | (self.evex_l2 << 6)
                        | (self.vex_l << 5)
                        | (self.evex_b << 4)
                        | ((!self.evex_v2 & 1) << 3)
                        | self.evex_aaa,
                );
            }
        }
    }
}

fn modrm_byte(mode: u8, register_field: u8, register_memory: u8) -> u8 {
    register_memory | (register_field << 3) | (mode << 6)
}

pub struct Encoder<'t> {
    pub target: &'t Target,
    pub mode: Mode,
}

pub fn encoding_failure(reason: &str) -> Diagnostic {
    fail("EI301", "The instruction could not be encoded.").with_context(format!("Cause: {reason}"))
}

impl<'t> Encoder<'t> {
    pub fn new(target: &'t Target, mode: Mode) -> Encoder<'t> {
        Encoder { target, mode }
    }

    fn encoding_of(&self, instruction: &MachineInstruction, index: usize) -> u16 {
        self.target.encoding(instruction.register_at(index))
    }

    fn number_of(&self, instruction: &MachineInstruction, index: usize) -> u8 {
        (self.encoding_of(instruction, index) & 7) as u8
    }

    fn set_rr2(&self, prefix: &mut PrefixHelper, instruction: &MachineInstruction, index: usize) {
        let encoding = self.encoding_of(instruction, index);
        prefix.set_r(encoding);
        prefix.set_r2(encoding);
    }

    fn set_bb2(&self, prefix: &mut PrefixHelper, instruction: &MachineInstruction, index: usize) {
        let register = instruction.register_at(index);
        let encoding = self.target.encoding(register);
        prefix.set_b(encoding);
        if prefix.kind <= PrefixKind::Rex2 || self.target.is_extended_apx(register) {
            prefix.set_b2(encoding);
        }
    }

    fn set_xx2(&self, prefix: &mut PrefixHelper, instruction: &MachineInstruction, index: usize) {
        let register = instruction.register_at(index);
        let encoding = self.target.encoding(register);
        prefix.set_x(encoding);
        if prefix.kind <= PrefixKind::Rex2 || self.target.is_extended_apx(register) {
            prefix.set_x2(encoding);
        }
    }

    fn set_x_shift(
        &self,
        prefix: &mut PrefixHelper,
        instruction: &MachineInstruction,
        index: usize,
        shift: u16,
    ) {
        let register = instruction.register_at(index);
        if shift != 3 && self.target.is_extended_apx(register) {
            return;
        }
        prefix.x = ((self.target.encoding(register) >> shift) & 1) as u8;
    }

    fn set_4v_v2(&self, prefix: &mut PrefixHelper, instruction: &MachineInstruction, index: usize) {
        let encoding = self.encoding_of(instruction, index);
        prefix.set_4v(encoding);
        prefix.set_v2(encoding);
    }

    fn set_v2_from_index(
        &self,
        prefix: &mut PrefixHelper,
        instruction: &MachineInstruction,
        index: usize,
        has_vex_4v: bool,
    ) {
        if has_vex_4v {
            return;
        }
        let register = instruction.register_at(index);
        if self.target.is_extended_apx(register) {
            return;
        }
        prefix.set_v2(self.target.encoding(register));
    }

    fn set_4v_immediate(
        &self,
        prefix: &mut PrefixHelper,
        instruction: &MachineInstruction,
        index: usize,
    ) {
        prefix.set_4v(!(instruction.immediate_at(index) as u16));
    }

    fn set_source_condition(
        &self,
        prefix: &mut PrefixHelper,
        instruction: &MachineInstruction,
        index: usize,
    ) {
        let value = instruction.immediate_at(index) as u8;
        prefix.evex_v2 = !(value >> 3) & 1;
        prefix.evex_aaa = value & 7;
    }

    fn uses_16_bit_memory(&self, instruction: &MachineInstruction, operand: usize) -> bool {
        let base = instruction.register_at(operand + ADDRESS_BASE);
        let index = instruction.register_at(operand + ADDRESS_INDEX);
        if self.mode == Mode::Bits16 && base == NO_REGISTER && index == NO_REGISTER {
            return true;
        }
        self.target.in_class("GR16", base) || self.target.in_class("GR16", index)
    }

    fn uses_32_bit_memory(&self, instruction: &MachineInstruction, operand: usize) -> bool {
        let base = instruction.register_at(operand + ADDRESS_BASE);
        let index = instruction.register_at(operand + ADDRESS_INDEX);
        if base == self.target.register("EIP") || index == self.target.register("EIZ") {
            return true;
        }
        self.target.in_class("GR32", base) || self.target.in_class("GR32", index)
    }

    pub fn needs_address_size_override(
        &self,
        instruction: &MachineInstruction,
        memory_operand: Option<usize>,
        flags: u64,
    ) -> bool {
        let address_size = flags & ADDRESS_SIZE_MASK;
        if (self.mode == Mode::Bits16 && address_size == ADDRESS_SIZE_32)
            || (self.mode == Mode::Bits32 && address_size == ADDRESS_SIZE_16)
            || (self.mode == Mode::Bits64 && address_size == ADDRESS_SIZE_32)
        {
            return true;
        }
        let esi = self.target.register("ESI");
        let si = self.target.register("SI");
        let edi = self.target.register("EDI");
        let di = self.target.register("DI");
        let thirty_two = self.mode == Mode::Bits32;
        match form_of(flags) {
            RAW_FORM_DESTINATION_SOURCE => {
                let source = instruction.register_at(1);
                return (!thirty_two && source == esi) || (thirty_two && source == si);
            }
            RAW_FORM_SOURCE => {
                let source = instruction.register_at(0);
                return (!thirty_two && source == esi) || (thirty_two && source == si);
            }
            RAW_FORM_DESTINATION => {
                let destination = instruction.register_at(0);
                return (!thirty_two && destination == edi) || (thirty_two && destination == di);
            }
            _ => {}
        }
        let Some(memory) = memory_operand else {
            return false;
        };
        match self.mode {
            Mode::Bits64 => self.uses_32_bit_memory(instruction, memory),
            Mode::Bits32 => self.uses_16_bit_memory(instruction, memory),
            Mode::Bits16 => !self.uses_16_bit_memory(instruction, memory),
        }
    }

    fn segment_override(&self, instruction: &MachineInstruction, index: usize, out: &mut Vec<u8>) {
        let register = instruction.register_at(index);
        if register == NO_REGISTER {
            return;
        }
        let byte = match self.target.registers[register as usize].record.as_str() {
            "CS" => 0x2e,
            "DS" => 0x3e,
            "ES" => 0x26,
            "FS" => 0x64,
            "GS" => 0x65,
            "SS" => 0x36,
            _ => return,
        };
        out.push(byte);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_immediate(
        &self,
        operand: &MachineOperand,
        kind: FixupKind,
        pc_relative: bool,
        start: usize,
        out: &mut Vec<u8>,
        fixups: &mut Vec<Fixup>,
        offset: i64,
    ) {
        let mut kind = kind;
        let size = match kind {
            FixupKind::Data1 => 1,
            FixupKind::Data2 => 2,
            FixupKind::Data8 => 8,
            _ => 4,
        };
        let mut offset = offset;
        let expression = match operand {
            MachineOperand::Immediate(value) => {
                if !(matches!(kind, FixupKind::Data1 | FixupKind::Data2 | FixupKind::Data4)
                    && pc_relative)
                {
                    let value = value.wrapping_add(offset) as u64;
                    for byte in 0..size {
                        out.push((value >> (8 * byte)) as u8);
                    }
                    return;
                }
                Expression::Constant(*value)
            }
            MachineOperand::Expression(expression) => expression.clone(),
            MachineOperand::Register(_) => Expression::Constant(0),
        };
        if matches!(
            kind,
            FixupKind::Data4 | FixupKind::Data8 | FixupKind::Signed4
        ) {
            let global = starts_with_global_offset_table(&expression);
            if global != GlobalOffsetTableKind::None {
                if size == 8 {
                    kind = FixupKind::Literal(29);
                } else {
                    kind = FixupKind::GlobalOffsetTable;
                }
                if global == GlobalOffsetTableKind::Normal {
                    offset = (out.len() - start) as i64;
                }
            } else if has_section_relative_reference(&expression) {
                kind = FixupKind::SectionRelative4;
            }
        }
        let expression = if offset != 0 {
            Expression::sum(expression, Expression::Constant(offset))
        } else {
            expression
        };
        fixups.push(Fixup {
            offset: out.len() - start,
            expression,
            kind,
            pc_relative,
        });
        out.extend(std::iter::repeat_n(0u8, size));
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_memory_modrm(
        &self,
        instruction: &MachineInstruction,
        operand: usize,
        register_field: u8,
        flags: u64,
        kind: PrefixKind,
        start: usize,
        out: &mut Vec<u8>,
        fixups: &mut Vec<Fixup>,
        force_sib: bool,
    ) -> Result<(), Diagnostic> {
        let displacement = instruction.operand(operand + ADDRESS_DISPLACEMENT).clone();
        let base = instruction.register_at(operand + ADDRESS_BASE);
        let scale = instruction.immediate_at(operand + ADDRESS_SCALE);
        let index = instruction.register_at(operand + ADDRESS_INDEX);
        let target = self.target;
        let record = target.instructions[instruction.opcode].record.as_str();
        if base == target.register("RIP") || base == target.register("EIP") {
            out.push(modrm_byte(0, register_field, 5));
            let is_symbol = matches!(
                displacement,
                MachineOperand::Expression(Expression::Symbol { .. })
            );
            let fixup = if !is_symbol {
                FixupKind::RipRelative4
            } else {
                match record {
                    "MOV64rm" => {
                        if kind == PrefixKind::Rex2 {
                            FixupKind::RipRelative4MoveLoadRex2
                        } else {
                            FixupKind::RipRelative4MoveLoad
                        }
                    }
                    "ADC32rm" | "ADD32rm" | "AND32rm" | "CMP32rm" | "MOV32rm" | "OR32rm"
                    | "SBB32rm" | "SUB32rm" | "TEST32mr" | "XOR32rm" | "CALL64m" | "JMP64m"
                    | "TAILJMPm64" | "TEST64mr" | "ADC64rm" | "ADD64rm" | "AND64rm" | "CMP64rm"
                    | "OR64rm" | "SBB64rm" | "SUB64rm" | "XOR64rm" | "LEA64r" => match kind {
                        PrefixKind::Rex2 => FixupKind::RipRelative4RelaxRex2,
                        PrefixKind::Rex => FixupKind::RipRelative4RelaxRex,
                        _ => FixupKind::RipRelative4Relax,
                    },
                    "ADD64rm_NF" | "ADD64rm_ND" | "ADD64mr_ND" | "ADD64mr_NF_ND"
                    | "ADD64rm_NF_ND" => FixupKind::RipRelative4RelaxEvex,
                    _ => FixupKind::RipRelative4,
                }
            };
            let immediate_size = if !displacement.is_immediate() && has_immediate(flags) {
                immediate_size(flags) as i64
            } else {
                0
            };
            self.emit_immediate(
                &displacement,
                fixup,
                true,
                start,
                out,
                fixups,
                -immediate_size,
            );
            return Ok(());
        }
        let base_number = if base != NO_REGISTER {
            (target.encoding(base) & 7) as u8
        } else {
            u8::MAX
        };
        let address_size_16 =
            self.mode == Mode::Bits32 && flags & ADDRESS_SIZE_MASK == ADDRESS_SIZE_16;
        if address_size_16 || self.uses_16_bit_memory(instruction, operand) {
            if base != NO_REGISTER {
                const REGISTER_16_TABLE: [u8; 8] = [0, 0, 0, 7, 0, 6, 4, 5];
                let mut field = REGISTER_16_TABLE[base_number as usize];
                if field == 0 {
                    return Err(encoding_failure("invalid 16-bit base register"));
                }
                if index != NO_REGISTER {
                    let index_16 = REGISTER_16_TABLE[(target.encoding(index) & 7) as usize];
                    if index_16 == 0 || (index_16 ^ field) & 2 == 0 || scale != 1 {
                        return Err(encoding_failure(
                            "invalid 16-bit base/index register combination",
                        ));
                    }
                    if index_16 & 2 != 0 {
                        field = (field & 1) | ((7 - index_16) << 1);
                    } else {
                        field = (index_16 & 1) | ((7 - field) << 1);
                    }
                }
                if let MachineOperand::Immediate(value) = displacement {
                    if (-128..=127).contains(&value) {
                        if value == 0 && field != 6 {
                            out.push(modrm_byte(0, register_field, field));
                            return Ok(());
                        }
                        out.push(modrm_byte(1, register_field, field));
                        self.emit_immediate(
                            &displacement,
                            FixupKind::Data1,
                            false,
                            start,
                            out,
                            fixups,
                            0,
                        );
                        return Ok(());
                    }
                }
                out.push(modrm_byte(2, register_field, field));
            } else {
                out.push(modrm_byte(0, register_field, 6));
            }
            self.emit_immediate(
                &displacement,
                FixupKind::Data2,
                false,
                start,
                out,
                fixups,
                0,
            );
            return Ok(());
        }
        let use_eight = instruction.flags & PREFIX_USE_DISPLACEMENT_8 != 0;
        let use_thirty_two = instruction.flags & PREFIX_USE_DISPLACEMENT_32 != 0;
        let allow_none = !use_eight && !use_thirty_two;
        let allow_eight = !use_thirty_two;
        let base_record = target.registers[base as usize].record.as_str();
        let needs_sib = index != NO_REGISTER
            || matches!(
                base_record,
                "ESP" | "RSP" | "R12" | "R12D" | "R20" | "R20D" | "R28" | "R28D"
            )
            || (self.mode == Mode::Bits64 && base == NO_REGISTER);
        if !force_sib && !needs_sib {
            if base == NO_REGISTER {
                out.push(modrm_byte(0, register_field, 5));
                self.emit_immediate(
                    &displacement,
                    FixupKind::Data4,
                    false,
                    start,
                    out,
                    fixups,
                    0,
                );
                return Ok(());
            }
            if base_number != 5 {
                if displacement == MachineOperand::Immediate(0) && allow_none {
                    out.push(modrm_byte(0, register_field, base_number));
                    return Ok(());
                }
                if let MachineOperand::Expression(Expression::Symbol {
                    specifier: Some(specifier),
                    ..
                }) = &displacement
                {
                    if specifier == "TLSCALL" {
                        if let MachineOperand::Expression(expression) = &displacement {
                            fixups.push(Fixup {
                                offset: 0,
                                expression: expression.clone(),
                                kind: FixupKind::None,
                                pc_relative: false,
                            });
                        }
                        out.push(modrm_byte(0, register_field, base_number));
                        return Ok(());
                    }
                }
            }
            if let MachineOperand::Immediate(value) = displacement {
                if allow_eight {
                    if let Some(offset) = displacement_fits_eight_bits(flags, value) {
                        out.push(modrm_byte(1, register_field, base_number));
                        self.emit_immediate(
                            &displacement,
                            FixupKind::Data1,
                            false,
                            start,
                            out,
                            fixups,
                            offset,
                        );
                        return Ok(());
                    }
                }
            }
            out.push(modrm_byte(2, register_field, base_number));
            let fixup = if record == "MOV32rm" {
                FixupKind::Signed4Relax
            } else {
                FixupKind::Signed4
            };
            self.emit_immediate(&displacement, fixup, false, start, out, fixups, 0);
            return Ok(());
        }
        let mut force_thirty_two = false;
        let mut force_eight = false;
        let mut offset = 0;
        let mut base_number = base_number;
        if base == NO_REGISTER {
            base_number = 5;
            out.push(modrm_byte(0, register_field, 4));
            force_thirty_two = true;
        } else if displacement == MachineOperand::Immediate(0) && allow_none && base_number != 5 {
            out.push(modrm_byte(0, register_field, 4));
        } else if let Some(found) = match displacement {
            MachineOperand::Immediate(value) if allow_eight => {
                displacement_fits_eight_bits(flags, value)
            }
            _ => None,
        } {
            offset = found;
            out.push(modrm_byte(1, register_field, 4));
            force_eight = true;
        } else {
            out.push(modrm_byte(2, register_field, 4));
            force_thirty_two = true;
        }
        let scale_field = match scale {
            1 => 0,
            2 => 1,
            4 => 2,
            8 => 3,
            _ => return Err(encoding_failure("invalid scale")),
        };
        let index_number = if index != NO_REGISTER {
            (target.encoding(index) & 7) as u8
        } else {
            4
        };
        out.push(modrm_byte(scale_field, index_number, base_number));
        if force_eight {
            self.emit_immediate(
                &displacement,
                FixupKind::Data1,
                false,
                start,
                out,
                fixups,
                offset,
            );
        } else if force_thirty_two {
            self.emit_immediate(
                &displacement,
                FixupKind::Signed4,
                false,
                start,
                out,
                fixups,
                0,
            );
        }
        Ok(())
    }

    fn emit_prefix_implementation(
        &self,
        instruction: &MachineInstruction,
        out: &mut Vec<u8>,
    ) -> Result<PrefixKind, Diagnostic> {
        let description = &self.target.instructions[instruction.opcode];
        let flags = description.flags;
        let memory = description.memory_operand_index();
        if let Some(memory) = memory {
            self.segment_override(instruction, memory + ADDRESS_SEGMENT, out);
        }
        if flags & REPEAT != 0 || instruction.flags & PREFIX_REPEAT != 0 {
            out.push(0xf3);
        }
        if instruction.flags & PREFIX_REPEAT_NOT_EQUAL != 0 {
            out.push(0xf2);
        }
        if self.needs_address_size_override(instruction, memory, flags)
            || instruction.flags & PREFIX_ADDRESS_SIZE != 0
        {
            out.push(0x67);
        }
        let ds = self.target.register("DS");
        match form_of(flags) {
            RAW_FORM_DESTINATION_SOURCE => {
                if instruction.register_at(2) != ds {
                    self.segment_override(instruction, 2, out);
                }
            }
            RAW_FORM_SOURCE => {
                if instruction.register_at(1) != ds {
                    self.segment_override(instruction, 1, out);
                }
            }
            RAW_FORM_MEMORY_OFFSET => self.segment_override(instruction, 1, out),
            _ => {}
        }
        if flags & ENCODING_MASK != 0 {
            self.emit_vex_prefix(memory, instruction, out)
        } else {
            self.emit_opcode_prefix(memory, instruction, out)
        }
    }

    fn emit_vex_prefix(
        &self,
        memory: Option<usize>,
        instruction: &MachineInstruction,
        out: &mut Vec<u8>,
    ) -> Result<PrefixKind, Diagnostic> {
        let description = &self.target.instructions[instruction.opcode];
        let flags = description.flags;
        for operand in instruction.operands.iter().skip(description.operand_bias()) {
            if let MachineOperand::Register(register) = operand {
                if self.target.is_high_byte(*register) {
                    return Err(encoding_failure(
                        "Cannot encode high byte register in VEX/EVEX-prefixed instruction",
                    ));
                }
            }
        }
        let mut prefix = PrefixHelper::new();
        prefix.kind = match flags & ENCODING_MASK {
            ENCODING_XOP => PrefixKind::Xop,
            ENCODING_VEX => {
                if instruction.flags & PREFIX_USE_VEX3 != 0 {
                    PrefixKind::Vex3
                } else {
                    PrefixKind::Vex2
                }
            }
            _ => PrefixKind::Evex,
        };
        prefix.w = (flags & REX_W != 0) as u8;
        prefix.evex_aaa |= ((flags & EVEX_NO_FLAGS != 0) as u8) << 2;
        let has_evex_k = flags & EVEX_K != 0;
        let has_vex_4v = flags & VEX_4V != 0;
        let new_data = has_new_data_destination(flags);
        let has_rounding = flags & EVEX_ROUNDING_CONTROL != 0;
        prefix.vex_5m = match flags & OPCODE_MAP_MASK {
            MAP_0F => 1,
            MAP_0F38 => 2,
            MAP_0F3A => 3,
            MAP_XOP_8 => 8,
            MAP_XOP_9 => 9,
            MAP_XOP_A => 10,
            MAP_4 => 4,
            MAP_5 => 5,
            MAP_6 => 6,
            MAP_7 => 7,
            _ => return Err(encoding_failure("Invalid prefix!")),
        };
        prefix.vex_l = (flags & VEX_L != 0) as u8;
        prefix.evex_l2 = (flags & EVEX_L2 != 0) as u8;
        prefix.vex_pp = match flags & OPERAND_PREFIX_MASK {
            PREFIX_66 => 1,
            PREFIX_F3 => 2,
            PREFIX_F2 => 3,
            _ => 0,
        };
        prefix.evex_z = (has_evex_k && flags & EVEX_Z != 0) as u8;
        prefix.evex_b = (flags & EVEX_B != 0) as u8;
        prefix.x2 = (flags & EVEX_U != 0) as u8;
        let mut encode_rounding = false;
        let mut rounding = 0u8;
        let mut current = description.operand_bias();
        let two_conditional = flags & TWO_CONDITIONAL_OPERANDS != 0;
        let memory_at = memory.unwrap_or(0);
        match form_of(flags) {
            MODRM_DESTINATION_MEMORY_VEX_OPERAND_3_CONDITION => {
                self.set_rr2(&mut prefix, instruction, current);
                current += 1;
                self.set_bb2(&mut prefix, instruction, memory_at + ADDRESS_BASE);
                self.set_xx2(&mut prefix, instruction, memory_at + ADDRESS_INDEX);
                current += ADDRESS_OPERAND_COUNT;
                self.set_4v_v2(&mut prefix, instruction, current);
            }
            MODRM_FIXED_FIRST | RAW_FORM => {}
            MODRM_DESTINATION_MEMORY_CONDITION
            | MODRM_DESTINATION_MEMORY_FORCED_SIB
            | MODRM_DESTINATION_MEMORY => {
                self.set_bb2(&mut prefix, instruction, memory_at + ADDRESS_BASE);
                self.set_xx2(&mut prefix, instruction, memory_at + ADDRESS_INDEX);
                self.set_v2_from_index(
                    &mut prefix,
                    instruction,
                    memory_at + ADDRESS_INDEX,
                    has_vex_4v,
                );
                if new_data {
                    self.set_4v_v2(&mut prefix, instruction, current);
                    current += 1;
                }
                current += ADDRESS_OPERAND_COUNT;
                if has_evex_k {
                    prefix.evex_aaa = self.encoding_of(instruction, current) as u8;
                    current += 1;
                }
                if !new_data && has_vex_4v {
                    self.set_4v_v2(&mut prefix, instruction, current);
                    current += 1;
                }
                self.set_rr2(&mut prefix, instruction, current);
                current += 1;
                if two_conditional {
                    self.set_4v_immediate(&mut prefix, instruction, current);
                    current += 1;
                    self.set_source_condition(&mut prefix, instruction, current);
                }
            }
            MODRM_SOURCE_MEMORY_CONDITION
            | MODRM_SOURCE_MEMORY_FORCED_SIB
            | MODRM_SOURCE_MEMORY => {
                if new_data {
                    self.set_4v_v2(&mut prefix, instruction, current);
                    current += 1;
                }
                self.set_rr2(&mut prefix, instruction, current);
                current += 1;
                if has_evex_k {
                    prefix.evex_aaa = self.encoding_of(instruction, current) as u8;
                    current += 1;
                }
                if !new_data && has_vex_4v {
                    self.set_4v_v2(&mut prefix, instruction, current);
                    current += 1;
                }
                self.set_bb2(&mut prefix, instruction, memory_at + ADDRESS_BASE);
                self.set_xx2(&mut prefix, instruction, memory_at + ADDRESS_INDEX);
                self.set_v2_from_index(
                    &mut prefix,
                    instruction,
                    memory_at + ADDRESS_INDEX,
                    has_vex_4v,
                );
                current += ADDRESS_OPERAND_COUNT;
                if two_conditional {
                    self.set_4v_immediate(&mut prefix, instruction, current);
                    current += 1;
                    self.set_source_condition(&mut prefix, instruction, current);
                }
            }
            MODRM_SOURCE_MEMORY_VEX_OPERAND_3 => {
                self.set_rr2(&mut prefix, instruction, current);
                current += 1;
                self.set_bb2(&mut prefix, instruction, memory_at + ADDRESS_BASE);
                self.set_xx2(&mut prefix, instruction, memory_at + ADDRESS_INDEX);
                self.set_4v_v2(&mut prefix, instruction, current + ADDRESS_OPERAND_COUNT);
            }
            MODRM_SOURCE_MEMORY_OPERAND_4 => {
                prefix.set_r(self.encoding_of(instruction, current));
                current += 1;
                prefix.set_4v(self.encoding_of(instruction, current));
                self.set_bb2(&mut prefix, instruction, memory_at + ADDRESS_BASE);
                self.set_xx2(&mut prefix, instruction, memory_at + ADDRESS_INDEX);
            }
            form if form == MODRM_ANY_MEMORY_CONDITION
                || (MODRM_MEMORY_0..=MODRM_MEMORY_7).contains(&form) =>
            {
                if has_vex_4v {
                    self.set_4v_v2(&mut prefix, instruction, current);
                    current += 1;
                }
                if has_evex_k {
                    prefix.evex_aaa = self.encoding_of(instruction, current) as u8;
                    current += 1;
                }
                self.set_bb2(&mut prefix, instruction, memory_at + ADDRESS_BASE);
                self.set_xx2(&mut prefix, instruction, memory_at + ADDRESS_INDEX);
                self.set_v2_from_index(
                    &mut prefix,
                    instruction,
                    memory_at + ADDRESS_INDEX,
                    has_vex_4v,
                );
                current += ADDRESS_OPERAND_COUNT + 1;
                if two_conditional {
                    self.set_4v_immediate(&mut prefix, instruction, current);
                    current += 1;
                    self.set_source_condition(&mut prefix, instruction, current);
                }
            }
            MODRM_SOURCE_REGISTER_CONDITION | MODRM_SOURCE_REGISTER => {
                if new_data {
                    self.set_4v_v2(&mut prefix, instruction, current);
                    current += 1;
                }
                self.set_rr2(&mut prefix, instruction, current);
                current += 1;
                if has_evex_k {
                    prefix.evex_aaa = self.encoding_of(instruction, current) as u8;
                    current += 1;
                }
                if !new_data && has_vex_4v {
                    self.set_4v_v2(&mut prefix, instruction, current);
                    current += 1;
                }
                self.set_bb2(&mut prefix, instruction, current);
                self.set_x_shift(&mut prefix, instruction, current, 4);
                current += 1;
                if two_conditional {
                    self.set_4v_immediate(&mut prefix, instruction, current);
                    current += 1;
                    self.set_source_condition(&mut prefix, instruction, current);
                    current += 1;
                }
                if flags & EVEX_B != 0 {
                    if has_rounding {
                        let last = description.operand_count() - 1;
                        rounding = instruction.immediate_at(last) as u8;
                    }
                    encode_rounding = true;
                }
                let _ = current;
            }
            MODRM_SOURCE_REGISTER_VEX_OPERAND_3 => {
                self.set_rr2(&mut prefix, instruction, current);
                current += 1;
                self.set_bb2(&mut prefix, instruction, current);
                current += 1;
                self.set_4v_v2(&mut prefix, instruction, current);
            }
            MODRM_SOURCE_REGISTER_OPERAND_4 => {
                prefix.set_r(self.encoding_of(instruction, current));
                current += 1;
                prefix.set_4v(self.encoding_of(instruction, current));
                current += 2;
                prefix.set_b(self.encoding_of(instruction, current));
                self.set_x_shift(&mut prefix, instruction, current, 4);
            }
            MODRM_DESTINATION_REGISTER_CONDITION | MODRM_DESTINATION_REGISTER => {
                if new_data {
                    self.set_4v_v2(&mut prefix, instruction, current);
                    current += 1;
                }
                self.set_bb2(&mut prefix, instruction, current);
                self.set_x_shift(&mut prefix, instruction, current, 4);
                current += 1;
                if has_evex_k {
                    prefix.evex_aaa = self.encoding_of(instruction, current) as u8;
                    current += 1;
                }
                if !new_data && has_vex_4v {
                    self.set_4v_v2(&mut prefix, instruction, current);
                    current += 1;
                }
                self.set_rr2(&mut prefix, instruction, current);
                current += 1;
                if two_conditional {
                    self.set_4v_immediate(&mut prefix, instruction, current);
                    current += 1;
                    self.set_source_condition(&mut prefix, instruction, current);
                }
                if flags & EVEX_B != 0 {
                    encode_rounding = true;
                }
            }
            MODRM_REGISTER_ZERO => {
                self.set_rr2(&mut prefix, instruction, current);
            }
            form if form == MODRM_ANY_REGISTER_CONDITION
                || (MODRM_REGISTER_0..=MODRM_REGISTER_7).contains(&form) =>
            {
                if has_vex_4v {
                    self.set_4v_v2(&mut prefix, instruction, current);
                    current += 1;
                }
                if has_evex_k {
                    prefix.evex_aaa = self.encoding_of(instruction, current) as u8;
                    current += 1;
                }
                self.set_bb2(&mut prefix, instruction, current);
                self.set_x_shift(&mut prefix, instruction, current, 4);
                current += 1;
                if two_conditional {
                    current += 1;
                    self.set_4v_immediate(&mut prefix, instruction, current);
                    current += 1;
                    self.set_source_condition(&mut prefix, instruction, current);
                }
            }
            _ => return Err(encoding_failure("Unexpected form in emitVEXOpcodePrefix!")),
        }
        if encode_rounding {
            prefix.vex_l = rounding & 1;
            prefix.evex_l2 = (rounding >> 1) & 1;
        }
        let kind = prefix.determine_optimal_kind();
        prefix.emit(out);
        Ok(kind)
    }

    fn emit_rex_prefix(
        &self,
        memory: Option<usize>,
        instruction: &MachineInstruction,
    ) -> Result<(PrefixKind, PrefixHelper), Diagnostic> {
        let mut prefix = PrefixHelper::new();
        if self.mode != Mode::Bits64 {
            return Ok((PrefixKind::None, prefix));
        }
        let description = &self.target.instructions[instruction.opcode];
        let flags = description.flags;
        prefix.w = (flags & REX_W != 0) as u8;
        let mut uses_high_byte = false;
        let mut current = if instruction.operands.is_empty() {
            0
        } else {
            description.operand_bias()
        };
        for operand in instruction.operands.iter().skip(current) {
            if let MachineOperand::Register(register) = operand {
                if self.target.is_high_byte(*register) {
                    uses_high_byte = true;
                }
                if self.target.is_low_byte_needing_rex(*register) {
                    prefix.kind = PrefixKind::Rex;
                }
            }
        }
        if instruction.flags & PREFIX_USE_REX != 0 {
            prefix.kind = PrefixKind::Rex;
        }
        if flags & EXPLICIT_PREFIX_MASK == EXPLICIT_REX2_PREFIX
            || instruction.flags & PREFIX_USE_REX2 != 0
        {
            prefix.kind = PrefixKind::Rex2;
        }
        let memory_at = memory.unwrap_or(0);
        match form_of(flags) {
            ADD_REGISTER_FORM => {
                self.set_bb2(&mut prefix, instruction, current);
            }
            MODRM_SOURCE_REGISTER | MODRM_SOURCE_REGISTER_CONDITION => {
                self.set_rr2(&mut prefix, instruction, current);
                current += 1;
                self.set_bb2(&mut prefix, instruction, current);
            }
            MODRM_SOURCE_MEMORY | MODRM_SOURCE_MEMORY_CONDITION => {
                self.set_rr2(&mut prefix, instruction, current);
                self.set_bb2(&mut prefix, instruction, memory_at + ADDRESS_BASE);
                self.set_xx2(&mut prefix, instruction, memory_at + ADDRESS_INDEX);
            }
            MODRM_DESTINATION_REGISTER => {
                self.set_bb2(&mut prefix, instruction, current);
                current += 1;
                self.set_rr2(&mut prefix, instruction, current);
            }
            MODRM_DESTINATION_MEMORY => {
                self.set_bb2(&mut prefix, instruction, memory_at + ADDRESS_BASE);
                self.set_xx2(&mut prefix, instruction, memory_at + ADDRESS_INDEX);
                current += ADDRESS_OPERAND_COUNT;
                self.set_rr2(&mut prefix, instruction, current);
            }
            form if form == MODRM_ANY_MEMORY_CONDITION
                || form == MODRM_ANY_MEMORY
                || (MODRM_MEMORY_0..=MODRM_MEMORY_7).contains(&form) =>
            {
                self.set_bb2(&mut prefix, instruction, memory_at + ADDRESS_BASE);
                self.set_xx2(&mut prefix, instruction, memory_at + ADDRESS_INDEX);
            }
            form if form == MODRM_ANY_REGISTER_CONDITION
                || form == MODRM_ANY_REGISTER
                || (MODRM_REGISTER_0..=MODRM_REGISTER_7).contains(&form) =>
            {
                self.set_bb2(&mut prefix, instruction, current);
            }
            _ => {}
        }
        prefix.m = (flags & OPCODE_MAP_MASK == MAP_0F) as u8;
        let kind = prefix.determine_optimal_kind();
        if kind != PrefixKind::None && uses_high_byte {
            return Err(encoding_failure(
                "Cannot encode high byte register in REX-prefixed instruction",
            ));
        }
        Ok((kind, prefix))
    }

    fn emit_opcode_prefix(
        &self,
        memory: Option<usize>,
        instruction: &MachineInstruction,
        out: &mut Vec<u8>,
    ) -> Result<PrefixKind, Diagnostic> {
        let flags = self.target.instructions[instruction.opcode].flags;
        let operand_size_needs_prefix = if self.mode == Mode::Bits16 {
            OPERAND_SIZE_32
        } else {
            OPERAND_SIZE_16
        };
        if flags & OPERAND_SIZE_MASK == operand_size_needs_prefix {
            out.push(0x66);
        }
        if flags & LOCK != 0 || instruction.flags & PREFIX_LOCK != 0 {
            out.push(0xf0);
        }
        if flags & NO_TRACK != 0 || instruction.flags & PREFIX_NO_TRACK != 0 {
            out.push(0x3e);
        }
        match flags & OPERAND_PREFIX_MASK {
            PREFIX_66 => out.push(0x66),
            PREFIX_F3 => out.push(0xf3),
            PREFIX_F2 => out.push(0xf2),
            _ => {}
        }
        let (kind, prefix) = self.emit_rex_prefix(memory, instruction)?;
        prefix.emit(out);
        match flags & OPCODE_MAP_MASK {
            MAP_0F => {
                if kind != PrefixKind::Rex2 {
                    out.push(0x0f);
                }
            }
            MAP_0F38 | MAP_0F3A | MAP_THREE_D_NOW => out.push(0x0f),
            _ => {}
        }
        match flags & OPCODE_MAP_MASK {
            MAP_0F38 => out.push(0x38),
            MAP_0F3A => out.push(0x3a),
            _ => {}
        }
        Ok(kind)
    }

    pub fn emit_prefix(
        &self,
        instruction: &MachineInstruction,
        out: &mut Vec<u8>,
    ) -> Result<(), Diagnostic> {
        let flags = self.target.instructions[instruction.opcode].flags;
        if form_of(flags) == PSEUDO {
            return Ok(());
        }
        self.emit_prefix_implementation(instruction, out)?;
        Ok(())
    }

    fn is_pc_relative_32_branch(&self, instruction: &MachineInstruction) -> bool {
        let description = &self.target.instructions[instruction.opcode];
        if !matches!(
            description.record.as_str(),
            "CALL64pcrel32" | "JMP_4" | "JCC_4"
        ) {
            return false;
        }
        if immediate_size(description.flags) != 4
            || immediate_is_signed(description.flags)
            || !immediate_is_pc_relative(description.flags)
        {
            return false;
        }
        matches!(
            instruction.operand(description.operand_bias()),
            MachineOperand::Expression(Expression::Symbol {
                specifier: None,
                ..
            })
        )
    }

    pub fn encode(&self, instruction: &MachineInstruction) -> Result<Encoded, Diagnostic> {
        let mut out = Vec::new();
        let mut fixups = Vec::new();
        let description = &self.target.instructions[instruction.opcode];
        let flags = description.flags;
        if form_of(flags) == PSEUDO {
            return Ok(Encoded::default());
        }
        if instruction.operands.len() < description.operand_count() {
            return Err(encoding_failure("too few operands"));
        }
        let mut operand_count = description.operand_count();
        let mut current = description.operand_bias();
        let start = 0;
        let kind = self.emit_prefix_implementation(instruction, &mut out)?;
        let has_vex_4v = flags & VEX_4V != 0;
        let has_register_immediate = flags & IMMEDIATE_MASK == IMMEDIATE_8_REGISTER;
        let has_evex_k = flags & EVEX_K != 0;
        let has_rounding = flags & EVEX_ROUNDING_CONTROL != 0;
        let mut register_immediate: u16 = 0;
        let mut opcode = base_opcode(flags);
        if flags & OPCODE_MAP_MASK == MAP_THREE_D_NOW {
            opcode = 0x0f;
        }
        let new_data = has_new_data_destination(flags);
        let two_conditional = flags & TWO_CONDITIONAL_OPERANDS != 0;
        let form = form_of(flags);
        let immediate_kind = immediate_fixup_kind(flags);
        match form {
            RAW_FORM_DESTINATION_SOURCE => {
                out.push(opcode);
                current += 3;
            }
            RAW_FORM_SOURCE => {
                out.push(opcode);
                current += 2;
            }
            RAW_FORM_DESTINATION => {
                out.push(opcode);
                current += 1;
            }
            PREFIX_BYTE => out.push(opcode),
            ADD_CONDITION_FORM | RAW_FORM => {
                let mut offset = 0u8;
                if form == ADD_CONDITION_FORM {
                    offset = instruction.immediate_at(operand_count - 1) as u8;
                    operand_count -= 1;
                }
                out.push(opcode.wrapping_add(offset));
                if self.mode == Mode::Bits64 && self.is_pc_relative_32_branch(instruction) {
                    let operand = instruction.operand(current).clone();
                    current += 1;
                    self.emit_immediate(
                        &operand,
                        FixupKind::Branch4PcRelative,
                        true,
                        start,
                        &mut out,
                        &mut fixups,
                        0,
                    );
                }
            }
            RAW_FORM_MEMORY_OFFSET => {
                out.push(opcode);
                let operand = instruction.operand(current).clone();
                current += 1;
                self.emit_immediate(
                    &operand,
                    immediate_kind,
                    immediate_is_pc_relative(flags),
                    start,
                    &mut out,
                    &mut fixups,
                    0,
                );
                current += 1;
            }
            RAW_FORM_IMMEDIATE_8 | RAW_FORM_IMMEDIATE_16 => {
                out.push(opcode);
                let operand = instruction.operand(current).clone();
                current += 1;
                self.emit_immediate(
                    &operand,
                    immediate_kind,
                    immediate_is_pc_relative(flags),
                    start,
                    &mut out,
                    &mut fixups,
                    0,
                );
                let operand = instruction.operand(current).clone();
                current += 1;
                let second = if form == RAW_FORM_IMMEDIATE_8 {
                    FixupKind::Data1
                } else {
                    FixupKind::Data2
                };
                self.emit_immediate(&operand, second, false, start, &mut out, &mut fixups, 0);
            }
            ADD_REGISTER_FORM => {
                out.push(opcode + self.number_of(instruction, current));
                current += 1;
            }
            MODRM_DESTINATION_REGISTER => {
                out.push(opcode);
                let mut source = current + 1;
                if has_evex_k {
                    source += 1;
                }
                if has_vex_4v {
                    source += 1;
                }
                if new_data {
                    current += 1;
                }
                out.push(modrm_byte(
                    3,
                    self.number_of(instruction, source),
                    self.number_of(instruction, current),
                ));
                current = source + 1;
            }
            MODRM_DESTINATION_REGISTER_CONDITION => {
                let first = current;
                let second = current + 1;
                let condition = instruction.immediate_at(current + 2) as u8;
                current += 3;
                out.push(opcode.wrapping_add(condition));
                out.push(modrm_byte(
                    3,
                    self.number_of(instruction, second),
                    self.number_of(instruction, first),
                ));
            }
            MODRM_DESTINATION_MEMORY_VEX_OPERAND_3_CONDITION => {
                let condition = instruction.immediate_at(8) as u8;
                out.push(opcode.wrapping_add(condition));
                let source = current + ADDRESS_OPERAND_COUNT;
                self.emit_memory_modrm(
                    instruction,
                    current + 1,
                    self.number_of(instruction, 0),
                    flags,
                    kind,
                    start,
                    &mut out,
                    &mut fixups,
                    false,
                )?;
                current = source + 3;
            }
            MODRM_DESTINATION_MEMORY_FORCED_SIB | MODRM_DESTINATION_MEMORY => {
                out.push(opcode);
                let mut source = current + ADDRESS_OPERAND_COUNT;
                if has_evex_k {
                    source += 1;
                }
                if has_vex_4v {
                    source += 1;
                }
                if new_data {
                    current += 1;
                }
                self.emit_memory_modrm(
                    instruction,
                    current,
                    self.number_of(instruction, source),
                    flags,
                    kind,
                    start,
                    &mut out,
                    &mut fixups,
                    form == MODRM_DESTINATION_MEMORY_FORCED_SIB,
                )?;
                current = source + 1;
            }
            MODRM_DESTINATION_MEMORY_CONDITION => {
                let memory = current;
                current = memory + ADDRESS_OPERAND_COUNT;
                let register = current;
                current += 1;
                let condition = instruction.immediate_at(current) as u8;
                current += 1;
                out.push(opcode.wrapping_add(condition));
                self.emit_memory_modrm(
                    instruction,
                    memory,
                    self.number_of(instruction, register),
                    flags,
                    kind,
                    start,
                    &mut out,
                    &mut fixups,
                    false,
                )?;
            }
            MODRM_SOURCE_REGISTER => {
                out.push(opcode);
                let mut source = current + 1;
                if has_evex_k {
                    source += 1;
                }
                if has_vex_4v {
                    source += 1;
                }
                if new_data {
                    current += 1;
                }
                out.push(modrm_byte(
                    3,
                    self.number_of(instruction, current),
                    self.number_of(instruction, source),
                ));
                current = source + 1;
                if has_register_immediate {
                    register_immediate = self.encoding_of(instruction, current);
                    current += 1;
                }
                if has_rounding {
                    operand_count -= 1;
                }
            }
            MODRM_SOURCE_REGISTER_VEX_OPERAND_3 => {
                out.push(opcode);
                let source = current + 1;
                out.push(modrm_byte(
                    3,
                    self.number_of(instruction, current),
                    self.number_of(instruction, source),
                ));
                current = source + 2;
            }
            MODRM_SOURCE_REGISTER_OPERAND_4 => {
                out.push(opcode);
                let mut source = current + 2;
                register_immediate = self.encoding_of(instruction, source);
                source += 1;
                out.push(modrm_byte(
                    3,
                    self.number_of(instruction, current),
                    self.number_of(instruction, source),
                ));
                current = source + 1;
            }
            MODRM_SOURCE_REGISTER_CONDITION => {
                if new_data {
                    current += 1;
                }
                let first = current;
                let second = current + 1;
                let condition = instruction.immediate_at(current + 2) as u8;
                current += 3;
                out.push(opcode.wrapping_add(condition));
                out.push(modrm_byte(
                    3,
                    self.number_of(instruction, first),
                    self.number_of(instruction, second),
                ));
            }
            MODRM_SOURCE_MEMORY_FORCED_SIB | MODRM_SOURCE_MEMORY => {
                let mut first_memory = current + 1;
                if new_data {
                    current += 1;
                }
                if has_evex_k {
                    first_memory += 1;
                }
                if has_vex_4v {
                    first_memory += 1;
                }
                out.push(opcode);
                self.emit_memory_modrm(
                    instruction,
                    first_memory,
                    self.number_of(instruction, current),
                    flags,
                    kind,
                    start,
                    &mut out,
                    &mut fixups,
                    form == MODRM_SOURCE_MEMORY_FORCED_SIB,
                )?;
                current = first_memory + ADDRESS_OPERAND_COUNT;
                if has_register_immediate {
                    register_immediate = self.encoding_of(instruction, current);
                    current += 1;
                }
            }
            MODRM_SOURCE_MEMORY_VEX_OPERAND_3 => {
                let first_memory = current + 1;
                out.push(opcode);
                self.emit_memory_modrm(
                    instruction,
                    first_memory,
                    self.number_of(instruction, current),
                    flags,
                    kind,
                    start,
                    &mut out,
                    &mut fixups,
                    false,
                )?;
                current = first_memory + ADDRESS_OPERAND_COUNT + 1;
            }
            MODRM_SOURCE_MEMORY_OPERAND_4 => {
                let mut first_memory = current + 2;
                register_immediate = self.encoding_of(instruction, first_memory);
                first_memory += 1;
                out.push(opcode);
                self.emit_memory_modrm(
                    instruction,
                    first_memory,
                    self.number_of(instruction, current),
                    flags,
                    kind,
                    start,
                    &mut out,
                    &mut fixups,
                    false,
                )?;
                current = first_memory + ADDRESS_OPERAND_COUNT;
            }
            MODRM_SOURCE_MEMORY_CONDITION => {
                if new_data {
                    current += 1;
                }
                let register = current;
                current += 1;
                let first_memory = current;
                current = first_memory + ADDRESS_OPERAND_COUNT;
                let condition = instruction.immediate_at(current) as u8;
                current += 1;
                out.push(opcode.wrapping_add(condition));
                self.emit_memory_modrm(
                    instruction,
                    first_memory,
                    self.number_of(instruction, register),
                    flags,
                    kind,
                    start,
                    &mut out,
                    &mut fixups,
                    false,
                )?;
            }
            MODRM_ANY_REGISTER_CONDITION => {
                let register = current;
                let condition = instruction.immediate_at(current + 1) as u8;
                current += 2;
                out.push(opcode.wrapping_add(condition));
                out.push(modrm_byte(3, 0, self.number_of(instruction, register)));
            }
            form if form == MODRM_ANY_REGISTER
                || (MODRM_REGISTER_0..=MODRM_REGISTER_7).contains(&form) =>
            {
                if has_vex_4v {
                    current += 1;
                }
                if has_evex_k {
                    current += 1;
                }
                out.push(opcode);
                let field = if form == MODRM_ANY_REGISTER {
                    0
                } else {
                    (form - MODRM_REGISTER_0) as u8
                };
                out.push(modrm_byte(3, field, self.number_of(instruction, current)));
                current += 1;
            }
            MODRM_REGISTER_ZERO => {
                out.push(opcode);
                out.push(modrm_byte(3, self.number_of(instruction, current), 0));
                current += 1;
            }
            MODRM_ANY_MEMORY_CONDITION => {
                let first_memory = current;
                current = first_memory + ADDRESS_OPERAND_COUNT;
                let condition = instruction.immediate_at(current) as u8;
                current += 1;
                out.push(opcode.wrapping_add(condition));
                self.emit_memory_modrm(
                    instruction,
                    first_memory,
                    0,
                    flags,
                    kind,
                    start,
                    &mut out,
                    &mut fixups,
                    false,
                )?;
            }
            form if form == MODRM_ANY_MEMORY
                || (MODRM_MEMORY_0..=MODRM_MEMORY_7).contains(&form) =>
            {
                if has_vex_4v {
                    current += 1;
                }
                if has_evex_k {
                    current += 1;
                }
                out.push(opcode);
                let field = if form == MODRM_ANY_MEMORY {
                    0
                } else {
                    (form - MODRM_MEMORY_0) as u8
                };
                self.emit_memory_modrm(
                    instruction,
                    current,
                    field,
                    flags,
                    kind,
                    start,
                    &mut out,
                    &mut fixups,
                    false,
                )?;
                current += ADDRESS_OPERAND_COUNT;
            }
            form if (MODRM_EXTENSION_0..=MODRM_EXTENSION_7).contains(&form) => {
                out.push(opcode);
                out.push(0xc0 + (((form - MODRM_EXTENSION_0) as u8) << 3));
            }
            form if (MODRM_FIXED_FIRST..=MODRM_FIXED_LAST).contains(&form) => {
                out.push(opcode);
                out.push(0xc0 + (form - MODRM_FIXED_FIRST) as u8);
            }
            _ => {
                return Err(encoding_failure(
                    "Unknown FormMask value in X86MCCodeEmitter!",
                ))
            }
        }
        if has_register_immediate {
            let mut value = (register_immediate as i64) << 4;
            if current != operand_count {
                value |= instruction.immediate_at(current) & 15;
            }
            self.emit_immediate(
                &MachineOperand::Immediate(value),
                FixupKind::Data1,
                false,
                start,
                &mut out,
                &mut fixups,
                0,
            );
        } else {
            let remaining = operand_count
                .saturating_sub(current)
                .saturating_sub(2 * two_conditional as usize);
            for _ in 0..remaining {
                let operand = instruction.operand(current).clone();
                current += 1;
                self.emit_immediate(
                    &operand,
                    immediate_kind,
                    immediate_is_pc_relative(flags),
                    start,
                    &mut out,
                    &mut fixups,
                    0,
                );
            }
        }
        if flags & OPCODE_MAP_MASK == MAP_THREE_D_NOW {
            out.push(base_opcode(flags));
        }
        if out.len() > 15 {
            return Err(encoding_failure(
                "instruction length exceeds the limit of 15",
            ));
        }
        Ok(Encoded { bytes: out, fixups })
    }
}

pub fn immediate_fixup_kind(flags: u64) -> FixupKind {
    let size = immediate_size(flags);
    if immediate_is_signed(flags) {
        return FixupKind::Signed4;
    }
    match size {
        1 => FixupKind::Data1,
        2 => FixupKind::Data2,
        8 => FixupKind::Data8,
        _ => FixupKind::Data4,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GlobalOffsetTableKind {
    None,
    Normal,
    SymbolDifference,
}

fn starts_with_global_offset_table(expression: &Expression) -> GlobalOffsetTableKind {
    let (left, right) = match expression {
        Expression::Binary { left, right, .. } => (left.as_ref(), Some(right.as_ref())),
        other => (other, None),
    };
    match left {
        Expression::Symbol { name, .. } if name == "_GLOBAL_OFFSET_TABLE_" => {
            if matches!(right, Some(Expression::Symbol { .. })) {
                GlobalOffsetTableKind::SymbolDifference
            } else {
                GlobalOffsetTableKind::Normal
            }
        }
        _ => GlobalOffsetTableKind::None,
    }
}

fn is_section_relative(expression: &Expression) -> bool {
    matches!(expression, Expression::Symbol { specifier: Some(specifier), .. } if specifier == "SECREL32")
}

fn has_section_relative_reference(expression: &Expression) -> bool {
    match expression {
        Expression::Binary { left, right, .. } => {
            is_section_relative(left) || is_section_relative(right)
        }
        other => is_section_relative(other),
    }
}
pub const COMPARE_SCALAR_LEGACY: &[&str] = &[
    "CMPPDrmi",
    "CMPPDrri",
    "CMPPSrmi",
    "CMPPSrri",
    "CMPSDrmi",
    "CMPSDrmi_Int",
    "CMPSDrri",
    "CMPSDrri_Int",
    "CMPSSrmi",
    "CMPSSrmi_Int",
    "CMPSSrri",
    "CMPSSrri_Int",
];

pub const COMPARE_VECTOR: &[&str] = &[
    "VCMPBF16Z128rmbi",
    "VCMPBF16Z128rmbik",
    "VCMPBF16Z128rmi",
    "VCMPBF16Z128rmik",
    "VCMPBF16Z128rri",
    "VCMPBF16Z128rrik",
    "VCMPBF16Z256rmbi",
    "VCMPBF16Z256rmbik",
    "VCMPBF16Z256rmi",
    "VCMPBF16Z256rmik",
    "VCMPBF16Z256rri",
    "VCMPBF16Z256rrik",
    "VCMPBF16Zrmbi",
    "VCMPBF16Zrmbik",
    "VCMPBF16Zrmi",
    "VCMPBF16Zrmik",
    "VCMPBF16Zrri",
    "VCMPBF16Zrrik",
    "VCMPPDYrmi",
    "VCMPPDYrri",
    "VCMPPDZ128rmbi",
    "VCMPPDZ128rmbik",
    "VCMPPDZ128rmi",
    "VCMPPDZ128rmik",
    "VCMPPDZ128rri",
    "VCMPPDZ128rrik",
    "VCMPPDZ256rmbi",
    "VCMPPDZ256rmbik",
    "VCMPPDZ256rmi",
    "VCMPPDZ256rmik",
    "VCMPPDZ256rri",
    "VCMPPDZ256rrik",
    "VCMPPDZrmbi",
    "VCMPPDZrmbik",
    "VCMPPDZrmi",
    "VCMPPDZrmik",
    "VCMPPDZrri",
    "VCMPPDZrrib",
    "VCMPPDZrribk",
    "VCMPPDZrrik",
    "VCMPPDrmi",
    "VCMPPDrri",
    "VCMPPHZ128rmbi",
    "VCMPPHZ128rmbik",
    "VCMPPHZ128rmi",
    "VCMPPHZ128rmik",
    "VCMPPHZ128rri",
    "VCMPPHZ128rrik",
    "VCMPPHZ256rmbi",
    "VCMPPHZ256rmbik",
    "VCMPPHZ256rmi",
    "VCMPPHZ256rmik",
    "VCMPPHZ256rri",
    "VCMPPHZ256rrik",
    "VCMPPHZrmbi",
    "VCMPPHZrmbik",
    "VCMPPHZrmi",
    "VCMPPHZrmik",
    "VCMPPHZrri",
    "VCMPPHZrrib",
    "VCMPPHZrribk",
    "VCMPPHZrrik",
    "VCMPPSYrmi",
    "VCMPPSYrri",
    "VCMPPSZ128rmbi",
    "VCMPPSZ128rmbik",
    "VCMPPSZ128rmi",
    "VCMPPSZ128rmik",
    "VCMPPSZ128rri",
    "VCMPPSZ128rrik",
    "VCMPPSZ256rmbi",
    "VCMPPSZ256rmbik",
    "VCMPPSZ256rmi",
    "VCMPPSZ256rmik",
    "VCMPPSZ256rri",
    "VCMPPSZ256rrik",
    "VCMPPSZrmbi",
    "VCMPPSZrmbik",
    "VCMPPSZrmi",
    "VCMPPSZrmik",
    "VCMPPSZrri",
    "VCMPPSZrrib",
    "VCMPPSZrribk",
    "VCMPPSZrrik",
    "VCMPPSrmi",
    "VCMPPSrri",
    "VCMPSDZrmi",
    "VCMPSDZrmi_Int",
    "VCMPSDZrmik_Int",
    "VCMPSDZrri",
    "VCMPSDZrri_Int",
    "VCMPSDZrrib_Int",
    "VCMPSDZrribk_Int",
    "VCMPSDZrrik_Int",
    "VCMPSDrmi",
    "VCMPSDrmi_Int",
    "VCMPSDrri",
    "VCMPSDrri_Int",
    "VCMPSHZrmi",
    "VCMPSHZrmi_Int",
    "VCMPSHZrmik_Int",
    "VCMPSHZrri",
    "VCMPSHZrri_Int",
    "VCMPSHZrrib_Int",
    "VCMPSHZrribk_Int",
    "VCMPSHZrrik_Int",
    "VCMPSSZrmi",
    "VCMPSSZrmi_Int",
    "VCMPSSZrmik_Int",
    "VCMPSSZrri",
    "VCMPSSZrri_Int",
    "VCMPSSZrrib_Int",
    "VCMPSSZrribk_Int",
    "VCMPSSZrrik_Int",
    "VCMPSSrmi",
    "VCMPSSrmi_Int",
    "VCMPSSrri",
    "VCMPSSrri_Int",
];

pub const COMPARE_XOP: &[&str] = &[
    "VPCOMBmi",
    "VPCOMBri",
    "VPCOMDmi",
    "VPCOMDri",
    "VPCOMQmi",
    "VPCOMQri",
    "VPCOMUBmi",
    "VPCOMUBri",
    "VPCOMUDmi",
    "VPCOMUDri",
    "VPCOMUQmi",
    "VPCOMUQri",
    "VPCOMUWmi",
    "VPCOMUWri",
    "VPCOMWmi",
    "VPCOMWri",
];

pub const COMPARE_INTEGER: &[&str] = &[
    "VPCMPBZ128rmi",
    "VPCMPBZ128rmik",
    "VPCMPBZ128rri",
    "VPCMPBZ128rrik",
    "VPCMPBZ256rmi",
    "VPCMPBZ256rmik",
    "VPCMPBZ256rri",
    "VPCMPBZ256rrik",
    "VPCMPBZrmi",
    "VPCMPBZrmik",
    "VPCMPBZrri",
    "VPCMPBZrrik",
    "VPCMPDZ128rmbi",
    "VPCMPDZ128rmbik",
    "VPCMPDZ128rmi",
    "VPCMPDZ128rmik",
    "VPCMPDZ128rri",
    "VPCMPDZ128rrik",
    "VPCMPDZ256rmbi",
    "VPCMPDZ256rmbik",
    "VPCMPDZ256rmi",
    "VPCMPDZ256rmik",
    "VPCMPDZ256rri",
    "VPCMPDZ256rrik",
    "VPCMPDZrmbi",
    "VPCMPDZrmbik",
    "VPCMPDZrmi",
    "VPCMPDZrmik",
    "VPCMPDZrri",
    "VPCMPDZrrik",
    "VPCMPQZ128rmbi",
    "VPCMPQZ128rmbik",
    "VPCMPQZ128rmi",
    "VPCMPQZ128rmik",
    "VPCMPQZ128rri",
    "VPCMPQZ128rrik",
    "VPCMPQZ256rmbi",
    "VPCMPQZ256rmbik",
    "VPCMPQZ256rmi",
    "VPCMPQZ256rmik",
    "VPCMPQZ256rri",
    "VPCMPQZ256rrik",
    "VPCMPQZrmbi",
    "VPCMPQZrmbik",
    "VPCMPQZrmi",
    "VPCMPQZrmik",
    "VPCMPQZrri",
    "VPCMPQZrrik",
    "VPCMPUBZ128rmi",
    "VPCMPUBZ128rmik",
    "VPCMPUBZ128rri",
    "VPCMPUBZ128rrik",
    "VPCMPUBZ256rmi",
    "VPCMPUBZ256rmik",
    "VPCMPUBZ256rri",
    "VPCMPUBZ256rrik",
    "VPCMPUBZrmi",
    "VPCMPUBZrmik",
    "VPCMPUBZrri",
    "VPCMPUBZrrik",
    "VPCMPUDZ128rmbi",
    "VPCMPUDZ128rmbik",
    "VPCMPUDZ128rmi",
    "VPCMPUDZ128rmik",
    "VPCMPUDZ128rri",
    "VPCMPUDZ128rrik",
    "VPCMPUDZ256rmbi",
    "VPCMPUDZ256rmbik",
    "VPCMPUDZ256rmi",
    "VPCMPUDZ256rmik",
    "VPCMPUDZ256rri",
    "VPCMPUDZ256rrik",
    "VPCMPUDZrmbi",
    "VPCMPUDZrmbik",
    "VPCMPUDZrmi",
    "VPCMPUDZrmik",
    "VPCMPUDZrri",
    "VPCMPUDZrrik",
    "VPCMPUQZ128rmbi",
    "VPCMPUQZ128rmbik",
    "VPCMPUQZ128rmi",
    "VPCMPUQZ128rmik",
    "VPCMPUQZ128rri",
    "VPCMPUQZ128rrik",
    "VPCMPUQZ256rmbi",
    "VPCMPUQZ256rmbik",
    "VPCMPUQZ256rmi",
    "VPCMPUQZ256rmik",
    "VPCMPUQZ256rri",
    "VPCMPUQZ256rrik",
    "VPCMPUQZrmbi",
    "VPCMPUQZrmbik",
    "VPCMPUQZrmi",
    "VPCMPUQZrmik",
    "VPCMPUQZrri",
    "VPCMPUQZrrik",
    "VPCMPUWZ128rmi",
    "VPCMPUWZ128rmik",
    "VPCMPUWZ128rri",
    "VPCMPUWZ128rrik",
    "VPCMPUWZ256rmi",
    "VPCMPUWZ256rmik",
    "VPCMPUWZ256rri",
    "VPCMPUWZ256rrik",
    "VPCMPUWZrmi",
    "VPCMPUWZrmik",
    "VPCMPUWZrri",
    "VPCMPUWZrrik",
    "VPCMPWZ128rmi",
    "VPCMPWZ128rmik",
    "VPCMPWZ128rri",
    "VPCMPWZ128rrik",
    "VPCMPWZ256rmi",
    "VPCMPWZ256rmik",
    "VPCMPWZ256rri",
    "VPCMPWZ256rrik",
    "VPCMPWZrmi",
    "VPCMPWZrmik",
    "VPCMPWZrri",
    "VPCMPWZrrik",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Syntax {
    Att,
    Intel,
}

impl Syntax {
    pub fn variant(self) -> usize {
        match self {
            Syntax::Att => 0,
            Syntax::Intel => 1,
        }
    }
}

pub fn flatten_variants(text: &str, variant: usize) -> String {
    let characters: Vec<char> = text.chars().collect();
    let mut result = String::new();
    let mut at = 0;
    loop {
        let mut start = at;
        while start < characters.len() {
            if characters[start] == '{'
                && (start == 0 || (characters[start - 1] != '$' && characters[start - 1] != '\\'))
            {
                break;
            }
            start += 1;
        }
        result.extend(&characters[at..start]);
        if start == characters.len() {
            break;
        }
        let begin = start + 1;
        let mut end = begin;
        let mut nested = 1;
        while end < characters.len() {
            if characters[end] == '}' && characters[end - 1] != '\\' {
                nested -= 1;
                if nested == 0 {
                    break;
                }
            } else if characters[end] == '{' {
                nested += 1;
            }
            end += 1;
        }
        let selection: String = characters[begin..end.min(characters.len())]
            .iter()
            .collect();
        let mut chosen = selection.as_str();
        for _ in 0..variant {
            chosen = chosen.split_once('|').map(|(_, rest)| rest).unwrap_or("");
        }
        result.push_str(chosen.split('|').next().unwrap_or(""));
        at = (end + 1).min(characters.len());
        if end >= characters.len() {
            break;
        }
    }
    result
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrintPiece {
    Literal(String),
    Operand {
        method: String,
        index: usize,
        pc_relative: bool,
    },
}

fn is_identifier_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn print_program(target: &Target, instruction: &Instruction, variant: usize) -> Vec<PrintPiece> {
    let text: Vec<char> = flatten_variants(&instruction.assembly, variant)
        .chars()
        .collect();
    let mut pieces = Vec::new();
    let mut literal = String::from("\t");
    let mut at = 0;
    while at < text.len() {
        let character = text[at];
        if character == '\\' && at + 1 < text.len() {
            let next = text[at + 1];
            match next {
                'n' => literal.push('\n'),
                't' => literal.push('\t'),
                other => literal.push(other),
            }
            at += 2;
            continue;
        }
        if character != '$' {
            literal.push(character);
            at += 1;
            continue;
        }
        if at + 1 < text.len() && text[at + 1] == '$' {
            literal.push('$');
            at += 2;
            continue;
        }
        let mut end = at + 1;
        let braces = end < text.len() && text[end] == '{';
        if braces {
            end += 1;
        }
        let name_start = end;
        while end < text.len() && is_identifier_character(text[end]) {
            end += 1;
        }
        let name: String = text[name_start..end].iter().collect();
        if braces {
            while end < text.len() && text[end] != '}' {
                end += 1;
            }
            end += 1;
        }
        if !literal.is_empty() {
            pieces.push(PrintPiece::Literal(std::mem::take(&mut literal)));
        }
        if let Some(group) = instruction.named_group(&name) {
            let group = &instruction.groups[group];
            let record = target.operand_records.get(&group.record);
            let method = record
                .map(|held| held.print_method.clone())
                .filter(|method| !method.is_empty())
                .unwrap_or_else(|| "printOperand".to_string());
            let pc_relative = record.is_some_and(|held| held.operand_type == "OPERAND_PCREL");
            pieces.push(PrintPiece::Operand {
                method,
                index: group.first,
                pc_relative,
            });
        }
        at = end;
    }
    if !literal.is_empty() {
        pieces.push(PrintPiece::Literal(literal));
    }
    pieces
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliasCondition {
    Ignore,
    Register(RegisterId),
    TiedRegister(usize),
    Immediate(i32),
    RegisterClass(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliasPiece {
    Literal(char),
    Operand {
        index: usize,
        method: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub struct AliasPattern {
    pub record: String,
    pub pieces: Vec<AliasPiece>,
    pub operand_count: usize,
    pub conditions: Vec<AliasCondition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResultOperand {
    Record { name: String, record: String },
    Immediate(i64),
    Register(RegisterId),
}

impl Target {
    pub fn register_class_of_operand(&self, record: &str) -> Option<String> {
        if self.register_classes.contains_key(record) {
            return Some(record.to_string());
        }
        let held = self.operand_records.get(record)?;
        if held.is_register_operand && self.register_classes.contains_key(&held.register_class) {
            return Some(held.register_class.clone());
        }
        let found = self.records.get(record)?;
        if found.is_a("RegClassByHwMode") || found.is_a("PointerLikeRegClass") {
            return Some(record.to_string());
        }
        None
    }

    pub fn is_register_class_like(&self, record: &str) -> bool {
        if self.register_classes.contains_key(record) {
            return true;
        }
        if let Some(held) = self.operand_records.get(record) {
            if held.is_register_operand {
                return true;
            }
        }
        self.records.get(record).is_some_and(|found| {
            found.is_a("RegClassByHwMode") || found.is_a("PointerLikeRegClass")
        })
    }

    pub fn operand_count_of_record(&self, record: &str) -> usize {
        match self.operand_records.get(record) {
            Some(held) if !held.sub_operands.is_empty() && !held.is_register_class => {
                held.sub_operands.len()
            }
            _ => 1,
        }
    }

    pub fn alias_result_operands(&self, alias: &InstructionAlias) -> Option<AliasResult> {
        let instruction = &self.instructions[alias.result];
        let mut operands = Vec::new();
        let mut indices = Vec::new();
        let mut argument = 0;
        for (group_index, group) in instruction.groups.iter().enumerate() {
            if group.count == 1 {
                if let Some(tied) = group.tied_to {
                    if instruction.groups[tied].record == group.record {
                        continue;
                    }
                }
            }
            let given = alias.arguments.get(argument)?;
            let sub_operands = self
                .operand_records
                .get(&group.record)
                .filter(|held| !held.is_register_class && !held.is_register_operand)
                .map(|held| held.sub_operands.clone())
                .unwrap_or_default();
            if !sub_operands.is_empty() {
                match given {
                    AliasArgument::Complex { arguments, .. } => {
                        for (sub_index, sub_argument) in arguments.iter().enumerate() {
                            let sub_record = sub_operands.get(sub_index)?;
                            operands.push(self.simple_result_operand(sub_argument, sub_record)?);
                            indices.push((group_index, Some(sub_index)));
                        }
                    }
                    AliasArgument::Operand { name, record } => {
                        operands.push(ResultOperand::Record {
                            name: name.clone(),
                            record: record.clone(),
                        });
                        indices.push((group_index, None));
                    }
                    _ => return None,
                }
            } else {
                operands.push(self.simple_result_operand(given, &group.record)?);
                indices.push((group_index, None));
            }
            argument += 1;
        }
        Some((operands, indices))
    }

    fn simple_result_operand(
        &self,
        argument: &AliasArgument,
        _slot: &str,
    ) -> Option<ResultOperand> {
        match argument {
            AliasArgument::Operand { name, record } => Some(ResultOperand::Record {
                name: name.clone(),
                record: record.clone(),
            }),
            AliasArgument::Immediate(value) => Some(ResultOperand::Immediate(*value)),
            AliasArgument::Register(register) => Some(ResultOperand::Register(*register)),
            AliasArgument::Complex { .. } => None,
        }
    }
}

fn unescape_alias(text: &str) -> String {
    let mut out = String::new();
    let characters: Vec<char> = text.chars().collect();
    let mut at = 0;
    while at < characters.len() {
        if characters[at] == '\\'
            && at + 1 < characters.len()
            && (characters[at + 1] == '{' || characters[at + 1] == '}')
        {
            out.push(characters[at + 1]);
            at += 2;
            continue;
        }
        out.push(characters[at]);
        at += 1;
    }
    out
}

fn count_spaces(text: &str) -> usize {
    text.chars()
        .filter(|character| *character == ' ' || *character == '\t')
        .count()
}

fn alias_patterns(target: &Target, variant: usize) -> BTreeMap<usize, Vec<AliasPattern>> {
    let mut grouped: BTreeMap<usize, Vec<&InstructionAlias>> = BTreeMap::new();
    for alias in &target.aliases {
        if alias.emit_priority < 1 {
            continue;
        }
        grouped.entry(alias.result).or_default().push(alias);
    }
    let mut result = BTreeMap::new();
    for (opcode, mut aliases) in grouped {
        aliases.sort_by(|left, right| {
            right
                .emit_priority
                .cmp(&left.emit_priority)
                .then(left.definition_order.cmp(&right.definition_order))
        });
        let instruction = &target.instructions[opcode];
        let mut patterns: Vec<AliasPattern> = Vec::new();
        for alias in aliases {
            let flat_instruction = flatten_variants(&instruction.assembly, variant);
            let flat_alias = unescape_alias(&flatten_variants(&alias.assembly, variant));
            if count_spaces(&flat_instruction) < count_spaces(&flat_alias) {
                continue;
            }
            let Some((operands, _)) = target.alias_result_operands(alias) else {
                continue;
            };
            let operand_count = instruction.operand_count();
            let mut conditions = Vec::new();
            let mut mapped: BTreeMap<String, (usize, Option<String>)> = BTreeMap::new();
            let mut operand_number = 0usize;
            for result_operand in &operands {
                while let Some(group_index) = instruction.operand_groups.get(operand_number) {
                    let group = &instruction.groups[*group_index];
                    let skipped = group.count == 1
                        && group
                            .tied_to
                            .is_some_and(|tied| instruction.groups[tied].record == group.record);
                    if !skipped {
                        break;
                    }
                    operand_number += 1;
                }
                while conditions.len() < operand_number {
                    conditions.push(AliasCondition::Ignore);
                }
                let mut count = 1;
                match result_operand {
                    ResultOperand::Record { name, record } => {
                        let mut method = None;
                        if let Some(held) = target.operand_records.get(record) {
                            if (held.is_register_operand || !held.is_register_class)
                                && !held.print_method.is_empty()
                                && held.print_method != "printOperand"
                            {
                                method = Some(held.print_method.clone());
                            }
                        }
                        if target.is_register_class_like(record) {
                            if let Some((index, _)) = mapped.get(name) {
                                conditions.push(AliasCondition::TiedRegister(*index));
                            } else {
                                mapped.insert(name.clone(), (operand_number, method));
                                let class = target
                                    .register_class_of_operand(record)
                                    .unwrap_or_else(|| record.clone());
                                conditions.push(AliasCondition::RegisterClass(class));
                            }
                        } else {
                            mapped.insert(name.clone(), (operand_number, method));
                            count = target.operand_count_of_record(record);
                        }
                    }
                    ResultOperand::Immediate(value) => {
                        conditions.push(AliasCondition::Immediate(*value as i32))
                    }
                    ResultOperand::Register(register) => {
                        conditions.push(AliasCondition::Register(*register))
                    }
                }
                operand_number += count;
            }
            let mut pieces = Vec::new();
            let characters: Vec<char> = flat_alias.chars().collect();
            let mut at = 0;
            let mut valid = true;
            while at < characters.len() {
                if characters[at] == '$' {
                    at += 1;
                    let name: String;
                    if at < characters.len() && characters[at] == '{' {
                        let start = at + 1;
                        let mut end = start;
                        while end < characters.len() && characters[end] != '}' {
                            end += 1;
                        }
                        name = characters[start..end].iter().collect();
                        at = (end + 1).min(characters.len());
                    } else {
                        let start = at;
                        let mut end = start;
                        while end < characters.len() && is_identifier_character(characters[end]) {
                            end += 1;
                        }
                        name = characters[start..end].iter().collect();
                        at = end;
                    }
                    match mapped.get(&name) {
                        Some((index, method)) => pieces.push(AliasPiece::Operand {
                            index: *index,
                            method: method.clone(),
                        }),
                        None => valid = false,
                    }
                } else {
                    pieces.push(AliasPiece::Literal(characters[at]));
                    at += 1;
                }
            }
            if !valid {
                continue;
            }
            patterns.push(AliasPattern {
                record: alias.record.clone(),
                pieces,
                operand_count,
                conditions,
            });
        }
        let unique: Vec<AliasPattern> = patterns
            .iter()
            .enumerate()
            .filter(|(index, pattern)| {
                !patterns.iter().enumerate().any(|(other_index, other)| {
                    other_index != *index
                        && other.operand_count == pattern.operand_count
                        && other.conditions == pattern.conditions
                })
            })
            .map(|(_, pattern)| pattern.clone())
            .collect();
        if !unique.is_empty() {
            result.insert(opcode, unique);
        }
    }
    result
}

pub type AliasResult = (Vec<ResultOperand>, Vec<(usize, Option<usize>)>);

pub struct PrintTables {
    pub programs: Vec<Vec<PrintPiece>>,
    pub aliases: BTreeMap<usize, Vec<AliasPattern>>,
}

pub fn print_tables(target: &Target, syntax: Syntax) -> &'static PrintTables {
    static ATT: OnceLock<PrintTables> = OnceLock::new();
    static INTEL: OnceLock<PrintTables> = OnceLock::new();
    let cell = match syntax {
        Syntax::Att => &ATT,
        Syntax::Intel => &INTEL,
    };
    cell.get_or_init(|| {
        let variant = syntax.variant();
        PrintTables {
            programs: target
                .instructions
                .iter()
                .map(|instruction| print_program(target, instruction, variant))
                .collect(),
            aliases: alias_patterns(target, variant),
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrintOptions {
    pub syntax: Syntax,
    pub mode: Mode,
    pub hexadecimal_immediates: bool,
    pub branch_as_address: bool,
}

pub struct Printer<'t> {
    pub target: &'t Target,
    pub options: PrintOptions,
    pub tables: &'static PrintTables,
}

fn hexadecimal(value: i64) -> String {
    if value < 0 {
        if value == i64::MIN {
            return "-0x8000000000000000".to_string();
        }
        return format!("-0x{:x}", -value);
    }
    format!("0x{value:x}")
}

pub fn is_acceptable_symbol_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || character == '_'
        || character == '.'
        || character == '@'
        || character == '$'
}

pub fn symbol_text(name: &str) -> String {
    if !name.is_empty() && name.chars().all(is_acceptable_symbol_character) {
        return name.to_string();
    }
    let mut out = String::from("\"");
    for character in name.chars() {
        match character {
            '\n' => out.push_str("\\n"),
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn binary_text(operator: BinaryOperator) -> &'static str {
    match operator {
        BinaryOperator::Add => "+",
        BinaryOperator::Subtract => "-",
        BinaryOperator::Multiply => "*",
        BinaryOperator::Divide => "/",
        BinaryOperator::Modulo => "%",
        BinaryOperator::And => "&",
        BinaryOperator::Or => "|",
        BinaryOperator::Xor => "^",
        BinaryOperator::ShiftLeft => "<<",
        BinaryOperator::ArithmeticShiftRight => ">>",
        BinaryOperator::LogicalShiftRight => ">>",
        BinaryOperator::LogicalAnd => "&&",
        BinaryOperator::LogicalOr => "||",
        BinaryOperator::Equal => "==",
        BinaryOperator::NotEqual => "!=",
        BinaryOperator::Less => "<",
        BinaryOperator::LessOrEqual => "<=",
        BinaryOperator::Greater => ">",
        BinaryOperator::GreaterOrEqual => ">=",
        BinaryOperator::OrNot => "!",
    }
}

pub fn expression_text(
    target: &Target,
    expression: &Expression,
    surrounding: i32,
    syntax: Syntax,
) -> String {
    const MAXIMUM_PRECEDENCE: i32 = 9;
    match expression {
        Expression::Constant(value) => value.to_string(),
        Expression::Symbol { name, specifier } => {
            let mut out = symbol_text(name);
            if let Some(specifier) = specifier {
                out.push('@');
                out.push_str(specifier);
            }
            out
        }
        Expression::Register(register) => {
            if syntax == Syntax::Att {
                format!("%{}", target.register_name(*register))
            } else {
                target.register_name(*register).to_string()
            }
        }
        Expression::Unary { operator, operand } => {
            let symbol = match operator {
                UnaryOperator::LogicalNot => "!",
                UnaryOperator::Negate => "-",
                UnaryOperator::Not => "~",
                UnaryOperator::Plus => "+",
            };
            format!(
                "{symbol}{}",
                expression_text(target, operand, MAXIMUM_PRECEDENCE, syntax)
            )
        }
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            let precedence = match operator {
                BinaryOperator::Add | BinaryOperator::Subtract => 1,
                _ => 0,
            };
            let parenthesis = precedence < surrounding;
            let sub = if precedence != 0 {
                precedence
            } else {
                MAXIMUM_PRECEDENCE
            };
            let mut out = String::new();
            if parenthesis {
                out.push('(');
            }
            out.push_str(&expression_text(target, left, sub, syntax));
            if *operator == BinaryOperator::Add {
                if let Expression::Constant(value) = right.as_ref() {
                    if *value < 0 {
                        out.push_str(&value.to_string());
                        if parenthesis {
                            out.push(')');
                        }
                        return out;
                    }
                }
            }
            out.push_str(binary_text(*operator));
            out.push_str(&expression_text(target, right, sub + 1, syntax));
            if parenthesis {
                out.push(')');
            }
            out
        }
    }
}

impl<'t> Printer<'t> {
    pub fn new(target: &'t Target, options: PrintOptions) -> Printer<'t> {
        Printer {
            target,
            options,
            tables: print_tables(target, options.syntax),
        }
    }

    fn att(&self) -> bool {
        self.options.syntax == Syntax::Att
    }

    fn format_immediate(&self, value: i64) -> String {
        if self.options.hexadecimal_immediates {
            hexadecimal(value)
        } else {
            value.to_string()
        }
    }

    fn expression(&self, expression: &Expression) -> String {
        expression_text(self.target, expression, 0, self.options.syntax)
    }

    fn register_text(&self, register: RegisterId) -> String {
        if self.att() {
            format!("%{}", self.target.register_name(register))
        } else {
            self.target.register_name(register).to_string()
        }
    }

    fn operand(&self, instruction: &MachineInstruction, index: usize, out: &mut String) {
        match instruction.operand(index) {
            MachineOperand::Register(register) => out.push_str(&self.register_text(*register)),
            MachineOperand::Immediate(value) => {
                if self.att() {
                    out.push('$');
                }
                out.push_str(&self.format_immediate(*value));
            }
            MachineOperand::Expression(expression) => {
                if self.att() {
                    out.push('$');
                } else {
                    out.push_str("offset ");
                }
                out.push_str(&self.expression(expression));
            }
        }
    }

    fn optional_segment(&self, instruction: &MachineInstruction, index: usize, out: &mut String) {
        if instruction.register_at(index) != NO_REGISTER {
            self.operand(instruction, index, out);
            out.push(':');
        }
    }

    fn memory_reference(&self, instruction: &MachineInstruction, operand: usize, out: &mut String) {
        let base = instruction.register_at(operand + ADDRESS_BASE);
        let index = instruction.register_at(operand + ADDRESS_INDEX);
        let scale = instruction.immediate_at(operand + ADDRESS_SCALE);
        let displacement = instruction.operand(operand + ADDRESS_DISPLACEMENT);
        if self.att() {
            self.optional_segment(instruction, operand + ADDRESS_SEGMENT, out);
            match displacement {
                MachineOperand::Immediate(value) => {
                    if *value != 0 || (index == NO_REGISTER && base == NO_REGISTER) {
                        out.push_str(&self.format_immediate(*value));
                    }
                }
                MachineOperand::Expression(expression) => {
                    let text = self.expression(expression);
                    if text.starts_with('$') {
                        out.push('(');
                        out.push_str(&text);
                        out.push(')');
                    } else {
                        out.push_str(&text);
                    }
                }
                MachineOperand::Register(_) => {}
            }
            if index != NO_REGISTER || base != NO_REGISTER {
                out.push('(');
                if base != NO_REGISTER {
                    self.operand(instruction, operand + ADDRESS_BASE, out);
                }
                if index != NO_REGISTER {
                    out.push(',');
                    self.operand(instruction, operand + ADDRESS_INDEX, out);
                    if scale != 1 {
                        out.push(',');
                        out.push_str(&scale.to_string());
                    }
                }
                out.push(')');
            }
            return;
        }
        self.optional_segment(instruction, operand + ADDRESS_SEGMENT, out);
        out.push('[');
        let mut need_plus = false;
        if base != NO_REGISTER {
            self.operand(instruction, operand + ADDRESS_BASE, out);
            need_plus = true;
        }
        if index != NO_REGISTER {
            if need_plus {
                out.push_str(" + ");
            }
            if scale != 1 || base == NO_REGISTER {
                out.push_str(&scale.to_string());
                out.push('*');
            }
            self.operand(instruction, operand + ADDRESS_INDEX, out);
            need_plus = true;
        }
        match displacement {
            MachineOperand::Immediate(value) => {
                let mut value = *value;
                if value != 0 || (index == NO_REGISTER && base == NO_REGISTER) {
                    if need_plus {
                        if value > 0 {
                            out.push_str(" + ");
                        } else {
                            out.push_str(" - ");
                            value = value.wrapping_neg();
                        }
                    }
                    out.push_str(&self.format_immediate(value));
                }
            }
            MachineOperand::Expression(expression) => {
                if need_plus {
                    out.push_str(" + ");
                }
                out.push_str(&self.expression(expression));
            }
            MachineOperand::Register(_) => {}
        }
        out.push(']');
    }

    fn memory_size_word(method: &str) -> &'static str {
        match method {
            "printbytemem" | "printSrcIdx8" | "printDstIdx8" | "printMemOffs8" => "byte ptr ",
            "printwordmem" | "printSrcIdx16" | "printDstIdx16" | "printMemOffs16" => "word ptr ",
            "printdwordmem" | "printSrcIdx32" | "printDstIdx32" | "printMemOffs32" => "dword ptr ",
            "printqwordmem" | "printSrcIdx64" | "printDstIdx64" | "printMemOffs64" => "qword ptr ",
            "printxmmwordmem" => "xmmword ptr ",
            "printymmwordmem" => "ymmword ptr ",
            "printzmmwordmem" => "zmmword ptr ",
            "printtbytemem" => "tbyte ptr ",
            _ => "",
        }
    }

    fn pc_relative(
        &self,
        instruction: &MachineInstruction,
        index: usize,
        address: u64,
        out: &mut String,
    ) {
        match instruction.operand(index) {
            MachineOperand::Immediate(value) => {
                if self.options.branch_as_address {
                    let mut target = address.wrapping_add(*value as u64);
                    if self.options.mode != Mode::Bits64 {
                        target &= 0xffff_ffff;
                    }
                    out.push_str(&format!("0x{target:x}"));
                } else {
                    out.push_str(&self.format_immediate(*value));
                }
            }
            MachineOperand::Expression(Expression::Constant(value)) => {
                out.push_str(&format!("0x{:x}", *value as u64));
            }
            MachineOperand::Expression(expression) => {
                let text = self.expression(expression);
                if self.att() && text.starts_with('$') {
                    out.push('(');
                    out.push_str(&text);
                    out.push(')');
                } else {
                    out.push_str(&text);
                }
            }
            MachineOperand::Register(_) => {}
        }
    }

    pub fn call_method(
        &self,
        method: &str,
        instruction: &MachineInstruction,
        index: usize,
        address: u64,
        out: &mut String,
    ) {
        let size_word = Printer::memory_size_word(method);
        match method {
            "printOperand" => self.operand(instruction, index, out),
            "printMemReference" | "printbytemem" | "printwordmem" | "printdwordmem"
            | "printqwordmem" | "printxmmwordmem" | "printymmwordmem" | "printzmmwordmem"
            | "printtbytemem" => {
                if !self.att() {
                    out.push_str(size_word);
                }
                self.memory_reference(instruction, index, out);
            }
            "printSrcIdx8" | "printSrcIdx16" | "printSrcIdx32" | "printSrcIdx64" => {
                if !self.att() {
                    out.push_str(size_word);
                }
                self.optional_segment(instruction, index + 1, out);
                out.push(if self.att() { '(' } else { '[' });
                self.operand(instruction, index, out);
                out.push(if self.att() { ')' } else { ']' });
            }
            "printDstIdx8" | "printDstIdx16" | "printDstIdx32" | "printDstIdx64" => {
                if self.att() {
                    out.push_str("%es:(");
                    self.operand(instruction, index, out);
                    out.push(')');
                } else {
                    out.push_str(size_word);
                    out.push_str("es:[");
                    self.operand(instruction, index, out);
                    out.push(']');
                }
            }
            "printMemOffs8" | "printMemOffs16" | "printMemOffs32" | "printMemOffs64"
            | "printMemOffset" => {
                if !self.att() {
                    out.push_str(size_word);
                }
                self.optional_segment(instruction, index + 1, out);
                if !self.att() {
                    out.push('[');
                }
                match instruction.operand(index) {
                    MachineOperand::Immediate(value) => {
                        out.push_str(&self.format_immediate(*value))
                    }
                    MachineOperand::Expression(expression) => {
                        let text = self.expression(expression);
                        if self.att() && text.starts_with('$') {
                            out.push('(');
                            out.push_str(&text);
                            out.push(')');
                        } else {
                            out.push_str(&text);
                        }
                    }
                    MachineOperand::Register(_) => {}
                }
                if !self.att() {
                    out.push(']');
                }
            }
            "printU8Imm" => match instruction.operand(index) {
                MachineOperand::Immediate(value) => {
                    if self.att() {
                        out.push('$');
                    }
                    out.push_str(&self.format_immediate(value & 0xff));
                }
                MachineOperand::Expression(expression) => {
                    if self.att() {
                        self.operand(instruction, index, out);
                    } else {
                        out.push_str(&self.expression(expression));
                    }
                }
                MachineOperand::Register(_) => self.operand(instruction, index, out),
            },
            "printSTiRegOperand" => {
                if self.target.registers[instruction.register_at(index) as usize].record == "ST0" {
                    out.push_str(if self.att() { "%st(0)" } else { "st(0)" });
                } else {
                    self.operand(instruction, index, out);
                }
            }
            "printPCRelImm" => self.pc_relative(instruction, index, address, out),
            "printCondCode" => {
                let record = &self.target.instructions[instruction.opcode].record;
                let flagged = record.starts_with("CCMP") || record.starts_with("CTEST");
                out.push_str(condition_code_text(
                    instruction.immediate_at(index),
                    flagged,
                ));
            }
            "printCondFlags" => {
                let value = instruction.immediate_at(index);
                let mut flags = Vec::new();
                if value & 8 != 0 {
                    flags.push("of");
                }
                if value & 4 != 0 {
                    flags.push("sf");
                }
                if value & 2 != 0 {
                    flags.push("zf");
                }
                if value & 1 != 0 {
                    flags.push("cf");
                }
                out.push_str("{dfv=");
                out.push_str(&flags.join(","));
                out.push('}');
            }
            "printRoundingControl" => {
                out.push_str(match instruction.immediate_at(index) {
                    0 => "{rn-sae}",
                    1 => "{rd-sae}",
                    2 => "{ru-sae}",
                    _ => "{rz-sae}",
                });
            }
            "printVKPair" => {
                let record = &self.target.registers[instruction.register_at(index) as usize].record;
                let first = record.split('_').next().unwrap_or("K0").to_string();
                let register = self.target.register(&first);
                out.push_str(&self.register_text(register));
            }
            _ => self.operand(instruction, index, out),
        }
    }

    fn instruction_flags(&self, instruction: &MachineInstruction, out: &mut String) {
        let description = &self.target.instructions[instruction.opcode];
        let flags = description.flags;
        let given = instruction.flags;
        if flags & LOCK != 0 || given & PREFIX_LOCK != 0 {
            out.push_str("\tlock\t");
        }
        if flags & NO_TRACK != 0 || given & PREFIX_NO_TRACK != 0 {
            out.push_str("\tnotrack\t");
        }
        if given & PREFIX_REPEAT_NOT_EQUAL != 0 {
            out.push_str("\trepne\t");
        } else if given & PREFIX_REPEAT != 0 {
            out.push_str("\trep\t");
        }
        if flags & EVEX_NO_FLAGS != 0 && !description.record.starts_with("CFCMOV") {
            out.push_str("\t{nf}");
        }
        if given & PREFIX_USE_VEX != 0 || flags & EXPLICIT_PREFIX_MASK == EXPLICIT_VEX_PREFIX {
            out.push_str("\t{vex}");
        } else if given & PREFIX_USE_VEX2 != 0 {
            out.push_str("\t{vex2}");
        } else if given & PREFIX_USE_VEX3 != 0 {
            out.push_str("\t{vex3}");
        } else if given & PREFIX_USE_EVEX != 0
            || flags & EXPLICIT_PREFIX_MASK == EXPLICIT_EVEX_PREFIX
        {
            out.push_str("\t{evex}");
        }
        if given & PREFIX_USE_DISPLACEMENT_8 != 0 {
            out.push_str("\t{disp8}");
        } else if given & PREFIX_USE_DISPLACEMENT_32 != 0 {
            out.push_str("\t{disp32}");
        }
        let memory = description.memory_operand_index();
        let encoder = Encoder::new(self.target, self.options.mode);
        if given & PREFIX_ADDRESS_SIZE != 0
            && !encoder.needs_address_size_override(instruction, memory, flags)
        {
            match self.options.mode {
                Mode::Bits16 | Mode::Bits64 => out.push_str("\taddr32\t"),
                Mode::Bits32 => out.push_str("\taddr16\t"),
            }
        }
    }

    fn alias(&self, instruction: &MachineInstruction, address: u64, out: &mut String) -> bool {
        let Some(patterns) = self.tables.aliases.get(&instruction.opcode) else {
            return false;
        };
        let mut chosen = None;
        for pattern in patterns {
            if instruction.operands.len() != pattern.operand_count {
                return false;
            }
            let mut index = 0;
            let matched = pattern.conditions.iter().all(|condition| {
                let operand = instruction.operand(index);
                index += 1;
                match condition {
                    AliasCondition::Ignore => true,
                    AliasCondition::Immediate(value) => {
                        matches!(operand, MachineOperand::Immediate(held) if *held == *value as i64)
                    }
                    AliasCondition::Register(register) => {
                        matches!(operand, MachineOperand::Register(held) if held == register)
                    }
                    AliasCondition::TiedRegister(other) => {
                        operand.is_register()
                            && operand.register() == instruction.register_at(*other)
                    }
                    AliasCondition::RegisterClass(class) => match operand {
                        MachineOperand::Register(held) => self
                            .target
                            .class(class)
                            .is_some_and(|found| found.contains(*held)),
                        _ => false,
                    },
                }
            });
            if matched {
                chosen = Some(pattern);
                break;
            }
        }
        let Some(pattern) = chosen else {
            return false;
        };
        let pieces = &pattern.pieces;
        let mut at = 0;
        out.push('\t');
        while at < pieces.len() {
            match &pieces[at] {
                AliasPiece::Literal(character) if *character != ' ' && *character != '\t' => {
                    out.push(*character);
                    at += 1;
                }
                _ => break,
            }
        }
        if at < pieces.len() {
            if matches!(
                pieces[at],
                AliasPiece::Literal(' ') | AliasPiece::Literal('\t')
            ) {
                out.push('\t');
                at += 1;
            }
            while at < pieces.len() {
                match &pieces[at] {
                    AliasPiece::Literal(character) => out.push(*character),
                    AliasPiece::Operand { index, method } => match method {
                        Some(method) => self.call_method(method, instruction, *index, address, out),
                        None => self.operand(instruction, *index, out),
                    },
                }
                at += 1;
            }
        }
        true
    }

    fn element_count(flags: u64, three_a_map_doubles: bool) -> u32 {
        let mut count = if flags & EVEX_L2 != 0 {
            if flags & REX_W != 0 {
                8
            } else {
                16
            }
        } else if flags & VEX_L != 0 {
            if flags & REX_W != 0 {
                4
            } else {
                8
            }
        } else if flags & REX_W != 0 {
            2
        } else {
            4
        };
        if three_a_map_doubles && flags & OPCODE_MAP_MASK == MAP_0F3A {
            count *= 2;
        }
        count
    }

    fn compare_mnemonic(&self, instruction: &MachineInstruction, vector: bool) -> String {
        let record = &self.target.instructions[instruction.opcode].record;
        let last = instruction.operands.len() - 1;
        let mut out = String::from(if vector { "vcmp" } else { "cmp" });
        out.push_str(sse_condition_text(instruction.immediate_at(last)));
        let rest = record.trim_start_matches('V').trim_start_matches("CMP");
        let suffix = ["BF16", "PD", "PS", "SD", "SS", "PH", "SH"]
            .iter()
            .find(|candidate| rest.starts_with(**candidate))
            .copied()
            .unwrap_or("");
        out.push_str(&suffix.to_ascii_lowercase());
        out.push('\t');
        out
    }

    fn compare_memory(&self, flags: u64, broadcast: bool) -> &'static str {
        if broadcast {
            if flags & OPCODE_MAP_MASK == MAP_0F3A {
                "printwordmem"
            } else if flags & REX_W != 0 {
                "printqwordmem"
            } else {
                "printdwordmem"
            }
        } else if flags & OPERAND_PREFIX_MASK == PREFIX_F3 {
            if flags & OPCODE_MAP_MASK == MAP_0F3A {
                "printwordmem"
            } else {
                "printdwordmem"
            }
        } else if flags & OPERAND_PREFIX_MASK == PREFIX_F2 && flags & OPCODE_MAP_MASK != MAP_0F3A {
            "printqwordmem"
        } else if flags & EVEX_L2 != 0 {
            "printzmmwordmem"
        } else if flags & VEX_L != 0 {
            "printymmwordmem"
        } else {
            "printxmmwordmem"
        }
    }

    fn vector_compare(&self, instruction: &MachineInstruction, out: &mut String) -> bool {
        let Some(MachineOperand::Immediate(value)) = instruction.operands.last() else {
            return false;
        };
        let value = *value;
        let description = &self.target.instructions[instruction.opcode];
        let flags = description.flags;
        let record = description.record.as_str();
        let memory_form = form_of(flags) == MODRM_SOURCE_MEMORY;
        let att = self.att();
        if COMPARE_SCALAR_LEGACY.contains(&record) {
            if !(0..=7).contains(&value) {
                return false;
            }
            out.push('\t');
            out.push_str(&self.compare_mnemonic(instruction, false));
            let method = if memory_form {
                match flags & OPERAND_PREFIX_MASK {
                    PREFIX_F3 => "printdwordmem",
                    PREFIX_F2 => "printqwordmem",
                    _ => "printxmmwordmem",
                }
            } else {
                "printOperand"
            };
            if att {
                self.call_method(method, instruction, 2, 0, out);
                out.push_str(", ");
                self.operand(instruction, 0, out);
            } else {
                self.operand(instruction, 0, out);
                out.push_str(", ");
                self.call_method(method, instruction, 2, 0, out);
            }
            return true;
        }
        if COMPARE_VECTOR.contains(&record) {
            if !(0..=31).contains(&value) {
                return false;
            }
            out.push('\t');
            out.push_str(&self.compare_mnemonic(instruction, true));
            let broadcast = flags & EVEX_B != 0;
            if att {
                let mut current: usize = if flags & EVEX_K != 0 { 3 } else { 2 };
                if memory_form {
                    self.call_method(
                        self.compare_memory(flags, broadcast),
                        instruction,
                        current,
                        0,
                        out,
                    );
                    current -= 1;
                    if broadcast {
                        out.push_str(&format!("{{1to{}}}", Printer::element_count(flags, true)));
                    }
                } else {
                    if broadcast {
                        out.push_str("{sae}, ");
                    }
                    self.operand(instruction, current, out);
                    current -= 1;
                }
                out.push_str(", ");
                self.operand(instruction, current, out);
                current -= 1;
                out.push_str(", ");
                self.operand(instruction, 0, out);
                if current > 0 {
                    out.push_str(" {");
                    self.operand(instruction, current, out);
                    out.push('}');
                }
            } else {
                let mut current = 0;
                self.operand(instruction, current, out);
                current += 1;
                if flags & EVEX_K != 0 {
                    out.push_str(" {");
                    self.operand(instruction, current, out);
                    current += 1;
                    out.push('}');
                }
                out.push_str(", ");
                self.operand(instruction, current, out);
                current += 1;
                out.push_str(", ");
                if memory_form {
                    self.call_method(
                        self.compare_memory(flags, broadcast),
                        instruction,
                        current,
                        0,
                        out,
                    );
                    if broadcast {
                        out.push_str(&format!("{{1to{}}}", Printer::element_count(flags, true)));
                    }
                } else {
                    self.operand(instruction, current, out);
                    if broadcast {
                        out.push_str(", {sae}");
                    }
                }
            }
            return true;
        }
        if COMPARE_XOP.contains(&record) {
            if !(0..=7).contains(&value) {
                return false;
            }
            out.push('\t');
            out.push_str("vpcom");
            out.push_str(match value {
                0 => "lt",
                1 => "le",
                2 => "gt",
                3 => "ge",
                4 => "eq",
                5 => "neq",
                6 => "false",
                _ => "true",
            });
            let rest = record.trim_start_matches("VPCOM");
            let suffix = rest.trim_end_matches("mi").trim_end_matches("ri");
            out.push_str(&suffix.to_ascii_lowercase());
            out.push('\t');
            let method = if memory_form {
                "printxmmwordmem"
            } else {
                "printOperand"
            };
            if att {
                self.call_method(method, instruction, 2, 0, out);
                out.push_str(", ");
                self.operand(instruction, 1, out);
                out.push_str(", ");
                self.operand(instruction, 0, out);
            } else {
                self.operand(instruction, 0, out);
                out.push_str(", ");
                self.operand(instruction, 1, out);
                out.push_str(", ");
                self.call_method(method, instruction, 2, 0, out);
            }
            return true;
        }
        if COMPARE_INTEGER.contains(&record) {
            if !((0..=2).contains(&value) || (4..=6).contains(&value)) {
                return false;
            }
            out.push('\t');
            out.push_str("vpcmp");
            out.push_str(sse_condition_text(value));
            let rest = record.trim_start_matches("VPCMP");
            let suffix: String = rest
                .chars()
                .take_while(|character| *character != 'Z')
                .collect();
            out.push_str(&suffix.to_ascii_lowercase());
            out.push('\t');
            let broadcast = flags & EVEX_B != 0;
            let memory_method = if broadcast {
                if flags & REX_W != 0 {
                    "printqwordmem"
                } else {
                    "printdwordmem"
                }
            } else if flags & EVEX_L2 != 0 {
                "printzmmwordmem"
            } else if flags & VEX_L != 0 {
                "printymmwordmem"
            } else {
                "printxmmwordmem"
            };
            if att {
                let mut current: usize = if flags & EVEX_K != 0 { 3 } else { 2 };
                if memory_form {
                    self.call_method(memory_method, instruction, current, 0, out);
                    if broadcast {
                        out.push_str(&format!("{{1to{}}}", Printer::element_count(flags, false)));
                    }
                } else {
                    self.operand(instruction, current, out);
                }
                current -= 1;
                out.push_str(", ");
                self.operand(instruction, current, out);
                current -= 1;
                out.push_str(", ");
                self.operand(instruction, 0, out);
                if current > 0 {
                    out.push_str(" {");
                    self.operand(instruction, current, out);
                    out.push('}');
                }
            } else {
                let mut current = 0;
                self.operand(instruction, current, out);
                current += 1;
                if flags & EVEX_K != 0 {
                    out.push_str(" {");
                    self.operand(instruction, current, out);
                    current += 1;
                    out.push('}');
                }
                out.push_str(", ");
                self.operand(instruction, current, out);
                current += 1;
                out.push_str(", ");
                if memory_form {
                    self.call_method(memory_method, instruction, current, 0, out);
                    if broadcast {
                        out.push_str(&format!("{{1to{}}}", Printer::element_count(flags, false)));
                    }
                } else {
                    self.operand(instruction, current, out);
                }
            }
            return true;
        }
        false
    }

    pub fn print(&self, instruction: &MachineInstruction, address: u64) -> String {
        let mut out = String::new();
        self.instruction_flags(instruction, &mut out);
        let record = self.target.instructions[instruction.opcode].record.as_str();
        if record == "CALLpcrel32" && self.options.mode == Mode::Bits64 && self.att() {
            out.push_str("\tcallq\t");
            self.pc_relative(instruction, 0, address, &mut out);
        } else if record == "DATA16_PREFIX" && self.options.mode == Mode::Bits16 {
            out.push_str("\tdata32");
        } else if !self.alias(instruction, address, &mut out)
            && !self.vector_compare(instruction, &mut out)
        {
            let program = &self.tables.programs[instruction.opcode];
            for piece in program {
                match piece {
                    PrintPiece::Literal(text) => out.push_str(text),
                    PrintPiece::Operand { method, index, .. } => {
                        self.call_method(method, instruction, *index, address, &mut out)
                    }
                }
            }
        }
        out
    }
}

pub fn condition_code_text(value: i64, flagged: bool) -> &'static str {
    match value {
        0 => "o",
        1 => "no",
        2 => "b",
        3 => "ae",
        4 => "e",
        5 => "ne",
        6 => "be",
        7 => "a",
        8 => "s",
        9 => "ns",
        10 => {
            if flagged {
                "t"
            } else {
                "p"
            }
        }
        11 => {
            if flagged {
                "f"
            } else {
                "np"
            }
        }
        12 => "l",
        13 => "ge",
        14 => "le",
        _ => "g",
    }
}

pub fn sse_condition_text(value: i64) -> &'static str {
    const NAMES: [&str; 32] = [
        "eq", "lt", "le", "unord", "neq", "nlt", "nle", "ord", "eq_uq", "nge", "ngt", "false",
        "neq_oq", "ge", "gt", "true", "eq_os", "lt_oq", "le_oq", "unord_s", "neq_us", "nlt_uq",
        "nle_uq", "ord_s", "eq_us", "nge_uq", "ngt_uq", "false_os", "neq_os", "ge_oq", "gt_oq",
        "true_us",
    ];
    NAMES.get(value as usize).copied().unwrap_or("eq")
}
