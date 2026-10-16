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
    pub dwarf_numbers: Vec<i64>,
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
    pub is_commutable: bool,
    pub intel_mnemonic: String,
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
    fn tuple_names(&self, record: &TableRecord) -> Vec<(String, Vec<RegisterId>)> {
        let lists: Vec<Vec<RegisterId>> = table_list(record, "SubRegs")
            .iter()
            .map(|list| self.class_members(list, 1))
            .collect();
        let length = lists.iter().map(Vec::len).min().unwrap_or(0);
        let mut out = Vec::new();
        for index in 0..length {
            let members: Vec<RegisterId> = lists.iter().map(|list| list[index]).collect();
            let name = members
                .iter()
                .map(|member| self.registers[*member as usize].record.clone())
                .collect::<Vec<String>>()
                .join("_");
            out.push((name, members));
        }
        out
    }

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
                if let Some(record) = self.records.get(name) {
                    if record.is_a("RegisterTuples") {
                        return self
                            .tuple_names(record)
                            .iter()
                            .filter_map(|(tuple, _)| self.register_by_record.get(tuple).copied())
                            .collect();
                    }
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
                dwarf_numbers: Vec::new(),
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
                    dwarf_numbers: table_list(record, "DwarfNumbers")
                        .into_iter()
                        .map(|item| match item {
                            TableValue::Integer(value) => value,
                            _ => -1,
                        })
                        .collect(),
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
        let tuple_records: Vec<TableRecord> = target
            .records
            .values()
            .filter(|record| record.is_a("RegisterTuples"))
            .cloned()
            .collect();
        for record in &tuple_records {
            let names = table_text_list(record, "RegAsmNames");
            for (index, (name, members)) in target.tuple_names(record).into_iter().enumerate() {
                let first = members[0] as usize;
                let identifier = target.registers.len() as RegisterId;
                let encoding = target.registers[first].encoding;
                target.registers.push(Register {
                    record: name.clone(),
                    assembly_name: names.get(index).cloned().unwrap_or_default(),
                    encoding,
                    sub_registers: members.clone(),
                    super_registers: Vec::new(),
                    alternate_names: Vec::new(),
                    definition_order: record.definition_order,
                    dwarf_numbers: Vec::new(),
                });
                for member in &members {
                    target.registers[*member as usize]
                        .super_registers
                        .push(identifier);
                }
                target.register_by_record.insert(name, identifier);
            }
        }
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
                if operand == "variable_ops" {
                    continue;
                }
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
            is_commutable: table_bit(record, "isCommutable"),
            intel_mnemonic: intel_mnemonic_of(&table_text(record, "AsmString")),
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

pub fn intel_mnemonic_of(assembly: &str) -> String {
    let flattened = flatten_variants(assembly, 1);
    let mnemonic: String = flattened
        .chars()
        .take_while(|character| *character != '\t')
        .collect();
    mnemonic.replace("${cond}", "CC").to_ascii_uppercase()
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
    pub comments: std::cell::RefCell<Option<String>>,
    pub custom_comment: std::cell::Cell<bool>,
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
    character.is_ascii_alphanumeric() || character == '_' || character == '.' || character == '$'
}

pub const INTEL_RESERVED_WORDS: &[&str] = &[
    "byte", "word", "dword", "fword", "qword", "mmword", "tbyte", "oword", "xmmword", "ymmword",
    "zmmword", "offset", "flat", "near", "far", "short", "and", "eq", "ge", "gt", "le", "lt",
    "mod", "ne", "not", "or", "shl", "shr", "xor",
];

pub fn symbol_text_with(target: &Target, name: &str, syntax: Syntax) -> String {
    if syntax == Syntax::Intel {
        let lower = name.to_ascii_lowercase();
        if INTEL_RESERVED_WORDS.contains(&lower.as_str())
            || target.register_by_name.contains_key(&lower)
        {
            return quoted_symbol(name);
        }
    }
    symbol_text(name)
}

fn quoted_symbol(name: &str) -> String {
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
            let mut out = symbol_text_with(target, name, syntax);
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
            comments: std::cell::RefCell::new(None),
            custom_comment: std::cell::Cell::new(false),
        }
    }

    pub fn with_comments(self) -> Printer<'t> {
        *self.comments.borrow_mut() = Some(String::new());
        self
    }

    pub fn take_comments(&self) -> String {
        self.comments
            .borrow_mut()
            .as_mut()
            .map(std::mem::take)
            .unwrap_or_default()
    }

    fn immediate_comment(&self, value: i64) {
        if self.custom_comment.get() || (-256..=255).contains(&value) {
            return;
        }
        if let Some(comments) = self.comments.borrow_mut().as_mut() {
            if value == i64::from(value as i16) {
                comments.push_str(&format!("imm = 0x{:X}\n", value as u16));
            } else if value == i64::from(value as i32) {
                comments.push_str(&format!("imm = 0x{:X}\n", value as u32));
            } else {
                comments.push_str(&format!("imm = 0x{:X}\n", value as u64));
            }
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
                    self.immediate_comment(*value);
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
        let commented = self.comments.borrow().is_some();
        let comment = if commented {
            instruction_comment(self.target, instruction)
        } else {
            None
        };
        self.custom_comment.set(comment.is_some());
        if let (Some(text), Some(comments)) = (comment, self.comments.borrow_mut().as_mut()) {
            comments.push_str(&text);
        }
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

const TERNARY_LOGIC_FUNCTIONS: [&str; 256] = [
    "0",
    "~(a | b | c)",
    "c & ~(a | b)",
    "~(a | b)",
    "b & ~(a | c)",
    "~(a | c)",
    "~a & (b ^ c)",
    "~(a | (b & c))",
    "b & c & ~a",
    "~(a | (b ^ c))",
    "c & ~a",
    "~a & (c | ~b)",
    "b & ~a",
    "~a & (b | ~c)",
    "~a & (b | c)",
    "~a",
    "a & ~(b | c)",
    "~(b | c)",
    "~b & (a ^ c)",
    "~((a & c) | b)",
    "~c & (a ^ b)",
    "~((a & b) | c)",
    "a ^ ((a & b) | (b ^ c))",
    "(a & (b ^ c)) ^ ~(b & c)",
    "(a ^ b) & (a ^ c)",
    "~((a & b) | (b ^ c))",
    "a ^ ((a & b) | c)",
    "(a & c) ^ (c | ~b)",
    "a ^ ((a & c) | b)",
    "(a & b) ^ (b | ~c)",
    "a ^ (b | c)",
    "~(a & (b | c))",
    "a & c & ~b",
    "~(b | (a ^ c))",
    "c & ~b",
    "~b & (c | ~a)",
    "(a ^ b) & (b ^ c)",
    "~((a & b) | (a ^ c))",
    "b ^ ((a & b) | c)",
    "(b & c) ^ (c | ~a)",
    "c & (a ^ b)",
    "(a | b) ^ ((a & b) | ~c)",
    "c & ~(a & b)",
    "(c & (a ^ b)) | ~(a | b)",
    "(b | c) & (a ^ b)",
    "a ^ (b | ~c)",
    "(a & b) ^ (b | c)",
    "(c & ~b) | ~a",
    "a & ~b",
    "~b & (a | ~c)",
    "~b & (a | c)",
    "~b",
    "b ^ (a | (b & c))",
    "(a & b) ^ (a | ~c)",
    "b ^ (a | c)",
    "~(b & (a | c))",
    "(a | c) & (a ^ b)",
    "b ^ (a | ~c)",
    "(a & b) ^ (a | c)",
    "(c & ~a) | ~b",
    "a ^ b",
    "~(a | c) | (a ^ b)",
    "(c & ~a) | (a ^ b)",
    "~(a & b)",
    "a & b & ~c",
    "~(c | (a ^ b))",
    "(a ^ c) & (b ^ c)",
    "~((a & c) | (a ^ b))",
    "b & ~c",
    "~c & (b | ~a)",
    "c ^ ((a & c) | b)",
    "(b & c) ^ (b | ~a)",
    "b & (a ^ c)",
    "(a | c) ^ ((a & c) | ~b)",
    "(b | c) & (a ^ c)",
    "a ^ (c | ~b)",
    "b & ~(a & c)",
    "(b & (a ^ c)) | ~(a | c)",
    "(a & c) ^ (b | c)",
    "(b & ~c) | ~a",
    "a & ~c",
    "~c & (a | ~b)",
    "c ^ (a | (b & c))",
    "(a & c) ^ (a | ~b)",
    "~c & (a | b)",
    "~c",
    "c ^ (a | b)",
    "~(c & (a | b))",
    "(a | b) & (a ^ c)",
    "c ^ (a | ~b)",
    "a ^ c",
    "~(a | b) | (a ^ c)",
    "(a & c) ^ (a | b)",
    "(b & ~a) | ~c",
    "(b & ~a) | (a ^ c)",
    "~(a & c)",
    "a & (b ^ c)",
    "~(b ^ c) ^ (a | (b & c))",
    "(a | c) & (b ^ c)",
    "b ^ (c | ~a)",
    "(a | b) & (b ^ c)",
    "c ^ (b | ~a)",
    "b ^ c",
    "~(a | b) | (b ^ c)",
    "(a | b) & (c ^ (a & b))",
    "b ^ c ^ ~a",
    "c ^ (a & b)",
    "~(a | b) | (c ^ (a & b))",
    "b ^ (a & c)",
    "~(a | c) | (b ^ (a & c))",
    "(b & ~a) | (b ^ c)",
    "~a | (b ^ c)",
    "a & ~(b & c)",
    "(a & (b ^ c)) | ~(b | c)",
    "(b & c) ^ (a | c)",
    "(a & ~c) | ~b",
    "(b & c) ^ (a | b)",
    "(a & ~b) | ~c",
    "(a & ~b) | (b ^ c)",
    "~(b & c)",
    "a ^ (b & c)",
    "~(b | c) | (a ^ (b & c))",
    "(a & ~b) | (a ^ c)",
    "~b | (a ^ c)",
    "(a & ~c) | (a ^ b)",
    "~c | (a ^ b)",
    "(a ^ b) | (a ^ c)",
    "~(a & b & c)",
    "a & b & c",
    "~((a ^ b) | (a ^ c))",
    "c & ~(a ^ b)",
    "~(a ^ b) & (c | ~a)",
    "b & ~(a ^ c)",
    "~(a ^ c) & (b | ~a)",
    "(b | c) & (a ^ b ^ c)",
    "(b & c) ^ ~a",
    "b & c",
    "~(b ^ c) & (b | ~a)",
    "c & (b | ~a)",
    "~((b & c) ^ (a | b))",
    "b & (c | ~a)",
    "~((b & c) ^ (a | c))",
    "a ^ ((a ^ b) | (a ^ c))",
    "(b & c) | ~a",
    "a & ~(b ^ c)",
    "~(b ^ c) & (a | ~b)",
    "(a | c) & (a ^ b ^ c)",
    "(a & c) ^ ~b",
    "(a | b) & (a ^ b ^ c)",
    "(a & b) ^ ~c",
    "a ^ b ^ c",
    "~(a | b) | (a ^ b ^ c)",
    "~(b ^ c) & (a | b)",
    "~(b ^ c)",
    "c ^ (a & ~b)",
    "~((a | b) & (b ^ c))",
    "b ^ (a & ~c)",
    "~((a | c) & (b ^ c))",
    "(b & c) | (a ^ (b | c))",
    "~(a & (b ^ c))",
    "a & c",
    "~(a ^ c) & (a | ~b)",
    "c & (a | ~b)",
    "~((a & c) ^ (a | b))",
    "~(a ^ c) & (a | b)",
    "~(a ^ c)",
    "c ^ (b & ~a)",
    "~((a | b) & (a ^ c))",
    "c & (a | b)",
    "~c ^ (a | b)",
    "c",
    "c | ~(a | b)",
    "b ^ (a & (b ^ c))",
    "(b & c) | ~(a ^ c)",
    "(b & ~a) | c",
    "c | ~a",
    "a & (c | ~b)",
    "~((a & c) ^ (b | c))",
    "a ^ ((a ^ c) & (b ^ c))",
    "(a & c) | ~b",
    "a ^ (b & ~c)",
    "~((b | c) & (a ^ c))",
    "(a & c) | (a ^ b ^ c)",
    "~(b & (a ^ c))",
    "a ^ (b & (a ^ c))",
    "(a & c) | ~(b ^ c)",
    "(a & ~b) | c",
    "c | ~b",
    "(a & c) | (a ^ b)",
    "~((a ^ c) & (b ^ c))",
    "c | (a ^ b)",
    "c | ~(a & b)",
    "a & b",
    "~(a ^ b) & (a | ~c)",
    "~(a ^ b) & (a | c)",
    "~(a ^ b)",
    "b & (a | ~c)",
    "~((a & b) ^ (a | c))",
    "b ^ (c & ~a)",
    "~((a | c) & (a ^ b))",
    "b & (a | c)",
    "~b ^ (a | c)",
    "c ^ (a & (b ^ c))",
    "(b & c) | ~(a ^ b)",
    "b",
    "b | ~(a | c)",
    "(c & ~a) | b",
    "b | ~a",
    "a & (b | ~c)",
    "~((a & b) ^ (b | c))",
    "a ^ (c & ~b)",
    "~((b | c) & (a ^ b))",
    "a ^ ((a ^ b) & (b ^ c))",
    "(a & b) | ~c",
    "(a & b) | (a ^ b ^ c)",
    "~(c & (a ^ b))",
    "a ^ (c & (a ^ b))",
    "(a & b) | ~(b ^ c)",
    "(a & b) | (a ^ c)",
    "~((a ^ b) & (b ^ c))",
    "(a & ~c) | b",
    "b | ~c",
    "b | (a ^ c)",
    "b | ~(a & c)",
    "a & (b | c)",
    "~a ^ (b | c)",
    "c ^ (b & (a ^ c))",
    "(a & c) | ~(a ^ b)",
    "b ^ (c & (a ^ b))",
    "(a & b) | ~(a ^ c)",
    "(a & b) | (b ^ c)",
    "~((a ^ b) & (a ^ c))",
    "(a | b) & ((a & b) | c)",
    "(a & b) | (b ^ c ^ ~a)",
    "(a & b) | c",
    "c | ~(a ^ b)",
    "(a & c) | b",
    "b | ~(a ^ c)",
    "b | c",
    "~a | b | c",
    "a",
    "a | ~(b | c)",
    "a | (c & ~b)",
    "a | ~b",
    "a | (b & ~c)",
    "a | ~c",
    "a | (b ^ c)",
    "a | ~(b & c)",
    "a | (b & c)",
    "a | ~(b ^ c)",
    "a | c",
    "~b | a | c",
    "a | b",
    "~c | a | b",
    "a | b | c",
    "-1",
];

const SHUFFLE_UNDEFINED: i64 = -1;
const SHUFFLE_ZERO: i64 = -2;

fn decode_insert_ps(immediate: i64, source_is_memory: bool) -> Vec<i64> {
    let mut mask = vec![0, 1, 2, 3];
    let zero = immediate & 15;
    let destination = ((immediate >> 4) & 3) as usize;
    let source = if source_is_memory {
        0
    } else {
        (immediate >> 6) & 3
    };
    mask[destination] = 4 + source;
    for (lane, slot) in mask.iter_mut().enumerate() {
        if zero & (1 << lane) != 0 {
            *slot = SHUFFLE_ZERO;
        }
    }
    mask
}

fn decode_insert_element(count: i64, index: i64, length: i64) -> Vec<i64> {
    let mut mask: Vec<i64> = (0..count).collect();
    for offset in 0..length {
        mask[(index + offset) as usize] = count + offset;
    }
    mask
}

fn decode_move_high_low(count: i64) -> Vec<i64> {
    (count / 2..count)
        .map(|lane| count + lane)
        .chain(count / 2..count)
        .collect()
}

fn decode_move_low_high(count: i64) -> Vec<i64> {
    (0..count / 2)
        .chain((0..count / 2).map(|lane| count + lane))
        .collect()
}

fn decode_duplicate(count: i64, odd: i64) -> Vec<i64> {
    (0..count / 2)
        .flat_map(|pair| [2 * pair + odd, 2 * pair + odd])
        .collect()
}

fn decode_duplicate_double(count: i64) -> Vec<i64> {
    (0..count)
        .step_by(2)
        .flat_map(|lane| [lane, lane])
        .collect()
}

fn decode_shift_left_bytes(count: i64, immediate: i64) -> Vec<i64> {
    let mut mask = Vec::new();
    for lane in (0..count).step_by(16) {
        for index in 0..16 {
            mask.push(if index >= immediate {
                index - immediate + lane
            } else {
                SHUFFLE_ZERO
            });
        }
    }
    mask
}

fn decode_shift_right_bytes(count: i64, immediate: i64) -> Vec<i64> {
    let mut mask = Vec::new();
    for lane in (0..count).step_by(16) {
        for index in 0..16 {
            let base = index + immediate;
            mask.push(if base >= 16 {
                SHUFFLE_ZERO
            } else {
                base + lane
            });
        }
    }
    mask
}

fn decode_align_bytes(count: i64, immediate: i64) -> Vec<i64> {
    let mut mask = Vec::new();
    for lane in (0..count).step_by(16) {
        for index in 0..16 {
            let mut base = index + immediate;
            if base >= 16 {
                base += count - 16;
            }
            mask.push(base + lane);
        }
    }
    mask
}

fn decode_align_elements(count: i64, immediate: i64) -> Vec<i64> {
    let shift = immediate & (count - 1);
    (0..count).map(|index| index + shift).collect()
}

fn decode_shuffle_elements(count: i64, bits: i64, immediate: i64) -> Vec<i64> {
    let lanes = ((count * bits) / 128).max(1);
    let per_lane = count / lanes;
    let mut splat = ((immediate & 0xff) as u32).wrapping_mul(0x0101_0101);
    let mut mask = Vec::new();
    for lane in (0..count).step_by(per_lane as usize) {
        for _ in 0..per_lane {
            mask.push(i64::from(splat) % per_lane + lane);
            splat /= per_lane as u32;
        }
    }
    mask
}

fn decode_shuffle_half_words(count: i64, immediate: i64, high: bool) -> Vec<i64> {
    let mut mask = Vec::new();
    for lane in (0..count).step_by(8) {
        let mut selector = immediate;
        let mut shuffled = Vec::new();
        for _ in 0..4 {
            shuffled.push(selector & 3);
            selector >>= 2;
        }
        if high {
            mask.extend((0..4).map(|index| lane + index));
            mask.extend(shuffled.iter().map(|index| lane + 4 + index));
        } else {
            mask.extend(shuffled.iter().map(|index| lane + index));
            mask.extend((4..8).map(|index| lane + index));
        }
    }
    mask
}

fn decode_swap_halves(count: i64) -> Vec<i64> {
    let half = count / 2;
    (0..half).map(|index| index + half).chain(0..half).collect()
}

fn decode_shuffle_pairs(count: i64, bits: i64, immediate: i64) -> Vec<i64> {
    let per_lane = 128 / bits;
    let mut selector = immediate;
    let mut mask = Vec::new();
    for lane in (0..count).step_by(per_lane as usize) {
        for source in (0..count * 2).step_by(count as usize) {
            for _ in 0..per_lane / 2 {
                mask.push(selector % per_lane + source + lane);
                selector /= per_lane;
            }
        }
        if per_lane == 4 {
            selector = immediate;
        }
    }
    mask
}

fn decode_unpack(count: i64, bits: i64, high: bool) -> Vec<i64> {
    let lanes = ((count * bits) / 128).max(1);
    let per_lane = count / lanes;
    let mut mask = Vec::new();
    for lane in (0..count).step_by(per_lane as usize) {
        let start = if high { lane + per_lane / 2 } else { lane };
        for index in start..start + per_lane / 2 {
            mask.push(index);
            mask.push(index + count);
        }
    }
    mask
}

fn decode_subvector_broadcast(destination: i64, source: i64) -> Vec<i64> {
    (0..destination / source).flat_map(|_| 0..source).collect()
}

fn decode_shuffle_lanes(count: i64, bits: i64, immediate: i64) -> Vec<i64> {
    let per_lane = 128 / bits;
    let lanes = count / per_lane;
    let mut selector = immediate;
    let mut mask = Vec::new();
    for lane in (0..count).step_by(per_lane as usize) {
        let mut index = (selector % lanes) * per_lane;
        selector /= lanes;
        if lane >= count / 2 {
            index += count;
        }
        mask.extend((0..per_lane).map(|offset| index + offset));
    }
    mask
}

fn decode_permute_halves(count: i64, immediate: i64) -> Vec<i64> {
    let half = count / 2;
    let mut mask = Vec::new();
    for lane in 0..2 {
        let selector = immediate >> (lane * 4);
        let begin = (selector & 3) * half;
        for index in begin..begin + half {
            mask.push(if selector & 8 != 0 {
                SHUFFLE_ZERO
            } else {
                index
            });
        }
    }
    mask
}

fn decode_blend(count: i64, immediate: i64) -> Vec<i64> {
    (0..count)
        .map(|index| {
            if (immediate >> (index % 8)) & 1 != 0 {
                count + index
            } else {
                index
            }
        })
        .collect()
}

fn decode_permute(count: i64, immediate: i64) -> Vec<i64> {
    let mut mask = Vec::new();
    for lane in (0..count).step_by(4) {
        for index in 0..4 {
            mask.push(lane + ((immediate >> (2 * index)) & 3));
        }
    }
    mask
}

fn decode_zero_extend(source_bits: i64, destination_bits: i64, count: i64) -> Vec<i64> {
    let scale = destination_bits / source_bits;
    let mut mask = Vec::new();
    for index in 0..count {
        mask.push(index);
        mask.extend(std::iter::repeat_n(SHUFFLE_ZERO, (scale - 1) as usize));
    }
    mask
}

fn decode_scalar_move(count: i64) -> Vec<i64> {
    std::iter::once(count).chain(1..count).collect()
}

fn decode_bit_field(
    count: i64,
    element_bits: i64,
    length: i64,
    index: i64,
    insert: bool,
) -> Vec<i64> {
    let half = count / 2;
    let mut length = length & 0x3f;
    let mut index = index & 0x3f;
    if length % element_bits != 0 || index % element_bits != 0 {
        return Vec::new();
    }
    if length == 0 {
        length = 64;
    }
    if length + index > 64 {
        return vec![SHUFFLE_UNDEFINED; count as usize];
    }
    length /= element_bits;
    index /= element_bits;
    let mut mask = Vec::new();
    if insert {
        mask.extend(0..index);
        mask.extend((0..length).map(|offset| offset + count));
        mask.extend(index + length..half);
    } else {
        mask.extend((0..length).map(|offset| offset + index));
        mask.extend(std::iter::repeat_n(
            SHUFFLE_ZERO,
            (half - length).max(0) as usize,
        ));
    }
    mask.extend(std::iter::repeat_n(
        SHUFFLE_UNDEFINED,
        (count - half) as usize,
    ));
    mask
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FusedOrder {
    FourRegisters,
    FourMemoryLast,
    FourMemoryMiddle,
    First,
    Second,
    Third,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TernarySource {
    Register,
    Memory,
    Broadcast32,
    Broadcast64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShuffleFamily {
    Blend(i64),
    InsertPacked,
    MoveLowHigh,
    MoveHighLow,
    InsertElement(i64, i64, i64),
    DuplicateLow,
    DuplicateHigh,
    DuplicateDouble,
    ShiftLeftBytes,
    ShiftRightBytes,
    AlignBytes,
    AlignElements(i64),
    ShuffleElements(i64),
    ShuffleHighWords,
    ShuffleLowWords,
    ShuffleMultimediaWords,
    SwapHalves,
    Unpack(i64, bool),
    ShufflePairs(i64),
    ShuffleLanes(i64),
    PermuteInLane(i64),
    PermuteHalves,
    Permute,
    ScalarMove(i64),
    ZeroMoveLow,
    ExtractField,
    InsertField,
    SubvectorBroadcast(i64, i64),
    ZeroExtend(i64, i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommentRule {
    Fused {
        order: FusedOrder,
        register: bool,
        accumulate: &'static str,
        negate: bool,
    },
    Ternary(TernarySource),
    FloatClass(bool),
    Shuffle(ShuffleFamily, bool),
}

struct CommentRules {
    rules: BTreeMap<String, CommentRule>,
}

impl CommentRules {
    fn add(&mut self, names: Vec<String>, rule: CommentRule) {
        for name in names {
            self.rules.entry(name).or_insert(rule);
        }
    }

    fn shuffle(&mut self, names: Vec<String>, family: ShuffleFamily, register: bool) {
        self.add(names, CommentRule::Shuffle(family, register));
    }
}

fn names_avx(instruction: &str, suffix: &str, source: &str) -> Vec<String> {
    vec![format!("V{instruction}{suffix}{source}")]
}

fn names_masked(instruction: &str, suffix: &str, source: &str) -> Vec<String> {
    vec![format!("V{instruction}{suffix}{source}k")]
}

fn names_avx512(instruction: &str, suffix: &str, source: &str) -> Vec<String> {
    vec![
        format!("V{instruction}{suffix}{source}"),
        format!("V{instruction}{suffix}{source}k"),
        format!("V{instruction}{suffix}{source}kz"),
    ]
}

fn names_avx512_internal(instruction: &str, suffix: &str, source: &str) -> Vec<String> {
    vec![
        format!("V{instruction}{suffix}{source}_Int"),
        format!("V{instruction}{suffix}{source}k_Int"),
        format!("V{instruction}{suffix}{source}kz_Int"),
    ]
}

fn names_join(groups: Vec<Vec<String>>) -> Vec<String> {
    groups.into_iter().flatten().collect()
}

fn names_every_width(instruction: &str, source: &str) -> Vec<String> {
    names_join(vec![
        names_avx512(instruction, "Z", source),
        names_avx512(instruction, "Z256", source),
        names_avx512(instruction, "Z128", source),
        names_avx(instruction, "", source),
        names_avx(instruction, "Y", source),
        vec![format!("{instruction}{source}")],
    ])
}

fn names_permute_in_lane(instruction: &str, source: &str) -> Vec<String> {
    let source = format!("{source}i");
    names_join(vec![
        names_avx512(instruction, "Z", &source),
        names_avx512(instruction, "Z256", &source),
        names_avx512(instruction, "Z128", &source),
        names_avx(instruction, "", &source),
        names_avx(instruction, "Y", &source),
    ])
}

fn names_permute(instruction: &str, source: &str) -> Vec<String> {
    let source = format!("{source}i");
    names_join(vec![
        names_avx512(instruction, "Z", &source),
        names_avx512(instruction, "Z256", &source),
        names_avx(instruction, "Y", &source),
    ])
}

fn names_shuffle_lanes(instruction: &str, source: &str) -> Vec<String> {
    let source = format!("r{source}i");
    let float = format!("SHUFF{instruction}");
    let integer = format!("SHUFI{instruction}");
    names_join(vec![
        names_avx512(&float, "Z", &source),
        names_avx512(&integer, "Z", &source),
        names_avx512(&float, "Z256", &source),
        names_avx512(&integer, "Z256", &source),
    ])
}

fn names_fused_wide(instruction: &str, source: &str) -> Vec<String> {
    names_join(vec![
        names_avx512(instruction, "Z", source),
        names_avx512(instruction, "Z256", source),
        names_avx512(instruction, "Z128", source),
    ])
}

fn names_fused(instruction: &str, source: &str) -> Vec<String> {
    names_join(vec![
        names_fused_wide(instruction, source),
        names_avx(instruction, "", source),
        names_avx(instruction, "Y", source),
    ])
}

fn names_fused_packed(instruction: &str, register: bool) -> Vec<String> {
    let double = format!("{instruction}PD");
    let single = format!("{instruction}PS");
    if register {
        names_join(vec![names_fused(&double, "r"), names_fused(&single, "r")])
    } else {
        names_join(vec![
            names_fused(&double, "m"),
            names_fused(&single, "m"),
            names_fused_wide(&double, "mb"),
            names_fused_wide(&single, "mb"),
        ])
    }
}

fn names_fused_scalar(instruction: &str, register: bool) -> Vec<String> {
    let source = if register { "r" } else { "m" };
    let internal = format!("{source}_Int");
    let double = format!("{instruction}SD");
    let single = format!("{instruction}SS");
    names_join(vec![
        names_avx(&double, "", source),
        names_avx(&single, "", source),
        names_avx(&double, "", &internal),
        names_avx(&single, "", &internal),
        names_avx(&double, "Z", source),
        names_avx(&single, "Z", source),
        names_avx512_internal(&double, "Z", source),
        names_avx512_internal(&single, "Z", source),
    ])
}

fn names_fused_four(instruction: &str, source: &str, scalar: bool) -> Vec<String> {
    let double = format!("{instruction}PD");
    let single = format!("{instruction}PS");
    let mut names = names_join(vec![
        names_avx(&double, "4", source),
        names_avx(&double, "4Y", source),
        names_avx(&single, "4", source),
        names_avx(&single, "4Y", source),
    ]);
    if scalar {
        let internal = format!("{source}_Int");
        let double = format!("{instruction}SD4");
        let single = format!("{instruction}SS4");
        names.extend(names_join(vec![
            names_avx(&double, "", source),
            names_avx(&single, "", source),
            names_avx(&double, "", &internal),
            names_avx(&single, "", &internal),
        ]));
    }
    names
}

fn names_float_class_packed(instruction: &str, source: &str) -> Vec<String> {
    let source = format!("{source}i");
    names_join(vec![
        names_avx(instruction, "Z", &source),
        names_avx(instruction, "Z256", &source),
        names_avx(instruction, "Z128", &source),
        names_masked(instruction, "Z", &source),
    ])
}

fn names_float_class_scalar(instruction: &str, source: &str) -> Vec<String> {
    let source = format!("{source}i");
    names_join(vec![
        names_avx(instruction, "Z", &source),
        names_masked(instruction, "Z", &source),
    ])
}

fn names_plain(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

fn build_comment_rules() -> CommentRules {
    let mut rules = CommentRules {
        rules: BTreeMap::new(),
    };
    for (operation, accumulate, negate, scalar) in [
        ("FMADD", "+", false, true),
        ("FMSUB", "-", false, true),
        ("FNMADD", "+", true, true),
        ("FNMSUB", "-", true, true),
        ("FMADDSUB", "+/-", false, false),
        ("FMSUBADD", "-/+", false, false),
    ] {
        for (source, order) in [
            ("rr", FusedOrder::FourRegisters),
            ("rm", FusedOrder::FourMemoryLast),
            ("mr", FusedOrder::FourMemoryMiddle),
        ] {
            rules.add(
                names_fused_four(operation, source, scalar),
                CommentRule::Fused {
                    order,
                    register: true,
                    accumulate,
                    negate,
                },
            );
        }
    }
    for (operation, accumulate, negate, scalar) in [
        ("FMADD", "+", false, true),
        ("FMSUB", "-", false, true),
        ("FNMADD", "+", true, true),
        ("FNMSUB", "-", true, true),
        ("FMADDSUB", "+/-", false, false),
        ("FMSUBADD", "-/+", false, false),
    ] {
        for (number, order) in [
            ("132", FusedOrder::First),
            ("213", FusedOrder::Second),
            ("231", FusedOrder::Third),
        ] {
            let instruction = format!("{operation}{number}");
            for register in [true, false] {
                let rule = CommentRule::Fused {
                    order,
                    register,
                    accumulate,
                    negate,
                };
                rules.add(names_fused_packed(&instruction, register), rule);
                if scalar {
                    rules.add(names_fused_scalar(&instruction, register), rule);
                }
            }
        }
    }
    for instruction in ["PTERNLOGD", "PTERNLOGQ"] {
        for (source, kind) in [
            ("r", TernarySource::Register),
            ("m", TernarySource::Memory),
            (
                "mb",
                if instruction == "PTERNLOGD" {
                    TernarySource::Broadcast32
                } else {
                    TernarySource::Broadcast64
                },
            ),
        ] {
            let source = format!("r{source}i");
            rules.add(
                names_fused_wide(instruction, &source),
                CommentRule::Ternary(kind),
            );
        }
    }
    for instruction in ["FPCLASSBF16", "FPCLASSPH", "FPCLASSPS", "FPCLASSPD"] {
        rules.add(
            names_float_class_packed(instruction, "r"),
            CommentRule::FloatClass(true),
        );
        rules.add(
            names_join(vec![
                names_float_class_packed(instruction, "m"),
                names_float_class_packed(instruction, "mb"),
            ]),
            CommentRule::FloatClass(false),
        );
    }
    for instruction in ["FPCLASSSH", "FPCLASSSS", "FPCLASSSD"] {
        rules.add(
            names_float_class_scalar(instruction, "r"),
            CommentRule::FloatClass(true),
        );
        rules.add(
            names_float_class_scalar(instruction, "m"),
            CommentRule::FloatClass(false),
        );
    }
    for (base, bits, extra) in [
        ("BLENDPD", 64, false),
        ("BLENDPS", 32, false),
        ("PBLENDW", 16, false),
        ("PBLENDD", 32, true),
    ] {
        let avx = base;
        let mut register = if extra {
            Vec::new()
        } else {
            vec![format!("{base}rri")]
        };
        register.extend([format!("V{avx}rri"), format!("V{avx}Yrri")]);
        let mut memory = if extra {
            Vec::new()
        } else {
            vec![format!("{base}rmi")]
        };
        memory.extend([format!("V{avx}rmi"), format!("V{avx}Yrmi")]);
        rules.shuffle(register, ShuffleFamily::Blend(bits), true);
        rules.shuffle(memory, ShuffleFamily::Blend(bits), false);
    }
    rules.shuffle(
        names_plain(&["INSERTPSrri", "VINSERTPSrri", "VINSERTPSZrri"]),
        ShuffleFamily::InsertPacked,
        true,
    );
    rules.shuffle(
        names_plain(&["INSERTPSrmi", "VINSERTPSrmi", "VINSERTPSZrmi"]),
        ShuffleFamily::InsertPacked,
        false,
    );
    rules.shuffle(
        names_plain(&["MOVLHPSrr", "VMOVLHPSrr", "VMOVLHPSZrr"]),
        ShuffleFamily::MoveLowHigh,
        true,
    );
    rules.shuffle(
        names_plain(&["MOVHLPSrr", "VMOVHLPSrr", "VMOVHLPSZrr"]),
        ShuffleFamily::MoveHighLow,
        true,
    );
    for (name, family) in [
        ("MOVHPD", ShuffleFamily::InsertElement(2, 1, 1)),
        ("MOVHPS", ShuffleFamily::InsertElement(4, 2, 2)),
        ("MOVLPD", ShuffleFamily::InsertElement(2, 0, 1)),
        ("MOVLPS", ShuffleFamily::InsertElement(4, 0, 2)),
    ] {
        rules.shuffle(
            vec![
                format!("{name}rm"),
                format!("V{name}rm"),
                format!("V{name}Z128rm"),
            ],
            family,
            false,
        );
    }
    for (name, family) in [
        ("MOVSLDUP", ShuffleFamily::DuplicateLow),
        ("MOVSHDUP", ShuffleFamily::DuplicateHigh),
        ("MOVDDUP", ShuffleFamily::DuplicateDouble),
    ] {
        rules.shuffle(names_every_width(name, "rr"), family, true);
        rules.shuffle(names_every_width(name, "rm"), family, false);
    }
    for (name, family) in [
        ("PSLLDQ", ShuffleFamily::ShiftLeftBytes),
        ("PSRLDQ", ShuffleFamily::ShiftRightBytes),
    ] {
        rules.shuffle(
            vec![
                format!("{name}ri"),
                format!("V{name}ri"),
                format!("V{name}Yri"),
                format!("V{name}Z128ri"),
                format!("V{name}Z256ri"),
                format!("V{name}Zri"),
            ],
            family,
            true,
        );
        rules.shuffle(
            vec![
                format!("V{name}Z128mi"),
                format!("V{name}Z256mi"),
                format!("V{name}Zmi"),
            ],
            family,
            false,
        );
    }
    rules.shuffle(
        names_every_width("PALIGNR", "rri"),
        ShuffleFamily::AlignBytes,
        true,
    );
    rules.shuffle(
        names_every_width("PALIGNR", "rmi"),
        ShuffleFamily::AlignBytes,
        false,
    );
    for (name, bits) in [("ALIGNQ", 64), ("ALIGND", 32)] {
        rules.shuffle(
            names_fused_wide(name, "rri"),
            ShuffleFamily::AlignElements(bits),
            true,
        );
        rules.shuffle(
            names_fused_wide(name, "rmi"),
            ShuffleFamily::AlignElements(bits),
            false,
        );
    }
    for (name, family) in [
        ("PSHUFD", ShuffleFamily::ShuffleElements(32)),
        ("PSHUFHW", ShuffleFamily::ShuffleHighWords),
        ("PSHUFLW", ShuffleFamily::ShuffleLowWords),
    ] {
        rules.shuffle(names_every_width(name, "ri"), family, true);
        rules.shuffle(names_every_width(name, "mi"), family, false);
    }
    rules.shuffle(
        names_plain(&["MMX_PSHUFWri"]),
        ShuffleFamily::ShuffleMultimediaWords,
        true,
    );
    rules.shuffle(
        names_plain(&["MMX_PSHUFWmi"]),
        ShuffleFamily::ShuffleMultimediaWords,
        false,
    );
    rules.shuffle(names_plain(&["PSWAPDrr"]), ShuffleFamily::SwapHalves, true);
    rules.shuffle(names_plain(&["PSWAPDrm"]), ShuffleFamily::SwapHalves, false);
    for (name, bits, high, multimedia) in [
        ("PUNPCKHBW", 8, true, true),
        ("PUNPCKHWD", 16, true, true),
        ("PUNPCKHDQ", 32, true, true),
        ("PUNPCKHQDQ", 64, true, false),
        ("PUNPCKLBW", 8, false, true),
        ("PUNPCKLWD", 16, false, true),
        ("PUNPCKLDQ", 32, false, true),
        ("PUNPCKLQDQ", 64, false, false),
        ("UNPCKLPD", 64, false, false),
        ("UNPCKLPS", 32, false, false),
        ("UNPCKHPD", 64, true, false),
        ("UNPCKHPS", 32, true, false),
    ] {
        let mut register = names_every_width(name, "rr");
        let mut memory = names_every_width(name, "rm");
        if multimedia {
            register.push(format!("MMX_{name}rr"));
            memory.push(format!("MMX_{name}rm"));
        }
        rules.shuffle(register, ShuffleFamily::Unpack(bits, high), true);
        rules.shuffle(memory, ShuffleFamily::Unpack(bits, high), false);
    }
    for (name, bits) in [("SHUFPD", 64), ("SHUFPS", 32)] {
        rules.shuffle(
            names_every_width(name, "rri"),
            ShuffleFamily::ShufflePairs(bits),
            true,
        );
        rules.shuffle(
            names_every_width(name, "rmi"),
            ShuffleFamily::ShufflePairs(bits),
            false,
        );
    }
    for (name, bits) in [("64X2", 64), ("32X4", 32)] {
        rules.shuffle(
            names_shuffle_lanes(name, "r"),
            ShuffleFamily::ShuffleLanes(bits),
            true,
        );
        rules.shuffle(
            names_shuffle_lanes(name, "m"),
            ShuffleFamily::ShuffleLanes(bits),
            false,
        );
    }
    for (name, bits) in [("PERMILPS", 32), ("PERMILPD", 64)] {
        rules.shuffle(
            names_permute_in_lane(name, "r"),
            ShuffleFamily::PermuteInLane(bits),
            true,
        );
        rules.shuffle(
            names_permute_in_lane(name, "m"),
            ShuffleFamily::PermuteInLane(bits),
            false,
        );
    }
    rules.shuffle(
        names_plain(&["VPERM2F128rri", "VPERM2I128rri"]),
        ShuffleFamily::PermuteHalves,
        true,
    );
    rules.shuffle(
        names_plain(&["VPERM2F128rmi", "VPERM2I128rmi"]),
        ShuffleFamily::PermuteHalves,
        false,
    );
    for name in ["PERMPD", "PERMQ"] {
        rules.shuffle(names_permute(name, "r"), ShuffleFamily::Permute, true);
        rules.shuffle(names_permute(name, "m"), ShuffleFamily::Permute, false);
    }
    rules.shuffle(
        names_plain(&["MOVSDrr", "VMOVSDrr", "VMOVSDZrr"]),
        ShuffleFamily::ScalarMove(2),
        true,
    );
    rules.shuffle(
        names_plain(&["MOVSSrr", "VMOVSSrr", "VMOVSSZrr"]),
        ShuffleFamily::ScalarMove(4),
        true,
    );
    rules.shuffle(
        names_plain(&["VMOVSHZrr"]),
        ShuffleFamily::ScalarMove(8),
        true,
    );
    rules.shuffle(
        names_plain(&[
            "MOVPQI2QIrr",
            "MOVZPQILo2PQIrr",
            "VMOVPQI2QIrr",
            "VMOVPQI2QIZrr",
            "VMOVZPQILo2PQIrr",
            "VMOVZPQILo2PQIZrr",
        ]),
        ShuffleFamily::ZeroMoveLow,
        true,
    );
    rules.shuffle(names_plain(&["EXTRQI"]), ShuffleFamily::ExtractField, true);
    rules.shuffle(names_plain(&["INSERTQI"]), ShuffleFamily::InsertField, true);
    let mut broadcast = |names: Vec<String>, destination: i64, source: i64, register: bool| {
        rules.shuffle(
            names,
            ShuffleFamily::SubvectorBroadcast(destination, source),
            register,
        );
    };
    broadcast(
        names_join(vec![
            names_plain(&["VBROADCASTF128rm", "VBROADCASTI128rm"]),
            names_avx512("BROADCASTF64X2", "Z256", "rm"),
            names_avx512("BROADCASTI64X2", "Z256", "rm"),
        ]),
        4,
        2,
        false,
    );
    for (float, integer, suffix, destination, source) in [
        ("BROADCASTF64X2", "BROADCASTI64X2", "Z", 8, 2),
        ("BROADCASTF64X4", "BROADCASTI64X4", "Z", 8, 4),
        ("BROADCASTF32X4", "BROADCASTI32X4", "Z256", 8, 4),
        ("BROADCASTF32X4", "BROADCASTI32X4", "Z", 16, 4),
        ("BROADCASTF32X8", "BROADCASTI32X8", "Z", 16, 8),
    ] {
        broadcast(
            names_join(vec![
                names_avx512(float, suffix, "rm"),
                names_avx512(integer, suffix, "rm"),
            ]),
            destination,
            source,
            false,
        );
    }
    broadcast(names_avx512("BROADCASTI32X2", "Z128", "rr"), 4, 2, true);
    broadcast(names_avx512("BROADCASTI32X2", "Z128", "rm"), 4, 2, false);
    for (suffix, destination) in [("Z256", 8), ("Z", 16)] {
        for (source_form, register) in [("rr", true), ("rm", false)] {
            broadcast(
                names_join(vec![
                    names_avx512("BROADCASTF32X2", suffix, source_form),
                    names_avx512("BROADCASTI32X2", suffix, source_form),
                ]),
                destination,
                2,
                register,
            );
        }
    }
    for (name, source, destination) in [
        ("PMOVZXBW", 8, 16),
        ("PMOVZXBD", 8, 32),
        ("PMOVZXBQ", 8, 64),
        ("PMOVZXWD", 16, 32),
        ("PMOVZXWQ", 16, 64),
        ("PMOVZXDQ", 32, 64),
    ] {
        rules.shuffle(
            names_every_width(name, "rr"),
            ShuffleFamily::ZeroExtend(source, destination),
            true,
        );
    }
    rules
}

fn comment_rules() -> &'static CommentRules {
    static RULES: OnceLock<CommentRules> = OnceLock::new();
    RULES.get_or_init(build_comment_rules)
}

struct CommentOperands<'i> {
    target: &'i Target,
    instruction: &'i MachineInstruction,
}

impl<'i> CommentOperands<'i> {
    fn count(&self) -> usize {
        self.instruction.operands.len()
    }

    fn name(&self, index: usize) -> Option<String> {
        match self.instruction.operands.get(index) {
            Some(MachineOperand::Register(register)) => {
                Some(self.target.register_name(*register).to_string())
            }
            _ => None,
        }
    }

    fn counted_from_end(&self, distance: usize) -> Option<String> {
        self.count()
            .checked_sub(distance)
            .and_then(|index| self.name(index))
    }

    fn last_immediate(&self) -> Option<i64> {
        self.immediate(self.count().checked_sub(1)?)
    }

    fn immediate(&self, index: usize) -> Option<i64> {
        match self.instruction.operands.get(index) {
            Some(MachineOperand::Immediate(value)) => Some(*value),
            _ => None,
        }
    }

    fn elements(&self, bits: i64) -> i64 {
        let name = self.name(0).unwrap_or_default();
        let size = if name.starts_with("zmm") {
            512
        } else if name.starts_with("ymm") {
            256
        } else if name.starts_with("xmm") {
            128
        } else {
            64
        };
        size / bits
    }

    fn masking(&self) -> String {
        let description = &self.target.instructions[self.instruction.opcode];
        if description.flags & EVEX_K == 0 {
            return String::new();
        }
        let mut operand = description.definitions;
        if description
            .tied_to
            .get(operand)
            .copied()
            .flatten()
            .is_some()
        {
            operand += 1;
        }
        let mut text = format!(" {{%{}}}", self.name(operand).unwrap_or_default());
        if description.flags & EVEX_Z != 0 {
            text.push_str(" {z}");
        }
        text
    }
}

fn fused_comment(
    operands: &CommentOperands,
    order: FusedOrder,
    register: bool,
    accumulate: &str,
    negate: bool,
) -> String {
    let last = 1;
    let middle = if register { 2 } else { 6 };
    let (first, second, added) = match order {
        FusedOrder::FourRegisters => (
            operands.name(1),
            operands.name(2),
            operands.counted_from_end(last),
        ),
        FusedOrder::FourMemoryLast => (operands.name(1), operands.name(2), None),
        FusedOrder::FourMemoryMiddle => (operands.name(1), None, operands.counted_from_end(last)),
        FusedOrder::First => (
            operands.name(1),
            if register {
                operands.counted_from_end(last)
            } else {
                None
            },
            operands.counted_from_end(middle),
        ),
        FusedOrder::Second => (
            operands.counted_from_end(middle),
            operands.name(1),
            if register {
                operands.counted_from_end(last)
            } else {
                None
            },
        ),
        FusedOrder::Third => (
            operands.counted_from_end(middle),
            if register {
                operands.counted_from_end(last)
            } else {
                None
            },
            operands.name(1),
        ),
    };
    let memory = || "mem".to_string();
    format!(
        "{}{} = {}({} * {}) {} {}\n",
        operands.name(0).unwrap_or_default(),
        operands.masking(),
        if negate { "-" } else { "" },
        first.unwrap_or_else(memory),
        second.unwrap_or_else(memory),
        accumulate,
        added.unwrap_or_else(memory)
    )
}

fn ternary_comment(operands: &CommentOperands, source: TernarySource) -> Option<String> {
    let third = match source {
        TernarySource::Register => operands.counted_from_end(2).unwrap_or_default(),
        TernarySource::Memory => "mem".to_string(),
        TernarySource::Broadcast32 => "m32bcst".to_string(),
        TernarySource::Broadcast64 => "m64bcst".to_string(),
    };
    let second_distance = if source == TernarySource::Register {
        3
    } else {
        7
    };
    let names = [
        operands.name(1).unwrap_or_default(),
        operands
            .counted_from_end(second_distance)
            .unwrap_or_default(),
        third,
    ];
    let table = operands.last_immediate()? as u8;
    let mut text = format!(
        "{}{} = ",
        operands.name(0).unwrap_or_default(),
        operands.masking()
    );
    for character in TERNARY_LOGIC_FUNCTIONS[table as usize].chars() {
        match character {
            'a' | 'b' | 'c' => text.push_str(&names[(character as u8 - b'a') as usize]),
            other => text.push(other),
        }
    }
    text.push('\n');
    Some(text)
}

fn float_class_comment(operands: &CommentOperands, register: bool) -> Option<String> {
    let categories = operands.last_immediate()? as u8;
    let source = if register {
        operands.counted_from_end(2).unwrap_or_default()
    } else {
        "mem".to_string()
    };
    let mut text = format!(
        "{}{} = ",
        operands.name(0).unwrap_or_default(),
        operands.masking()
    );
    if categories == 0 {
        text.push_str("false");
    } else {
        let names = [
            "QuietNaN",
            "PositiveZero",
            "NegativeZero",
            "PositiveInfinity",
            "NegativeInfinity",
            "Subnormal",
            "Negative",
            "SignalingNaN",
        ];
        let mut first = true;
        for (bit, name) in names.iter().enumerate() {
            if categories & (1 << bit) != 0 {
                if !first {
                    text.push_str(" | ");
                }
                first = false;
                text.push_str(&format!("is{name}({source})"));
            }
        }
    }
    text.push('\n');
    Some(text)
}

fn shuffle_comment(
    operands: &CommentOperands,
    family: ShuffleFamily,
    register: bool,
) -> Option<String> {
    let mut destination = operands.name(0);
    let mut first: Option<String> = None;
    let mut second: Option<String> = None;
    let immediate = operands
        .last_immediate()
        .map(|value| i64::from(value as u32));
    let far = |near: usize, distant: usize| if register { near } else { distant };
    let mask: Vec<i64> = match family {
        ShuffleFamily::Blend(bits) => {
            if register {
                second = operands.name(2);
            }
            first = operands.name(1);
            immediate
                .map(|value| decode_blend(operands.elements(bits), value))
                .unwrap_or_default()
        }
        ShuffleFamily::InsertPacked => {
            if register {
                second = operands.name(2);
            }
            first = operands.name(1);
            immediate
                .map(|value| decode_insert_ps(value, !register))
                .unwrap_or_default()
        }
        ShuffleFamily::MoveLowHigh => {
            second = operands.name(2);
            first = operands.name(1);
            decode_move_low_high(2)
        }
        ShuffleFamily::MoveHighLow => {
            second = operands.name(2);
            first = operands.name(1);
            decode_move_high_low(2)
        }
        ShuffleFamily::InsertElement(count, index, length) => {
            first = operands.name(1);
            decode_insert_element(count, index, length)
        }
        ShuffleFamily::DuplicateLow
        | ShuffleFamily::DuplicateHigh
        | ShuffleFamily::DuplicateDouble => {
            if register {
                first = operands.counted_from_end(1);
            }
            match family {
                ShuffleFamily::DuplicateLow => decode_duplicate(operands.elements(32), 0),
                ShuffleFamily::DuplicateHigh => decode_duplicate(operands.elements(32), 1),
                _ => decode_duplicate_double(operands.elements(64)),
            }
        }
        ShuffleFamily::ShiftLeftBytes | ShuffleFamily::ShiftRightBytes => {
            if register {
                first = operands.name(1);
            }
            immediate
                .map(|value| {
                    if family == ShuffleFamily::ShiftLeftBytes {
                        decode_shift_left_bytes(operands.elements(8), value)
                    } else {
                        decode_shift_right_bytes(operands.elements(8), value)
                    }
                })
                .unwrap_or_default()
        }
        ShuffleFamily::AlignBytes | ShuffleFamily::AlignElements(_) => {
            if register {
                first = operands.counted_from_end(2);
            }
            second = operands.counted_from_end(far(3, 7));
            immediate
                .map(|value| match family {
                    ShuffleFamily::AlignElements(bits) => {
                        decode_align_elements(operands.elements(bits), value)
                    }
                    _ => decode_align_bytes(operands.elements(8), value),
                })
                .unwrap_or_default()
        }
        ShuffleFamily::ShuffleElements(bits) | ShuffleFamily::PermuteInLane(bits) => {
            if register {
                first = operands.counted_from_end(2);
            }
            immediate
                .map(|value| decode_shuffle_elements(operands.elements(bits), bits, value))
                .unwrap_or_default()
        }
        ShuffleFamily::ShuffleHighWords | ShuffleFamily::ShuffleLowWords => {
            if register {
                first = operands.counted_from_end(2);
            }
            immediate
                .map(|value| {
                    decode_shuffle_half_words(
                        operands.elements(16),
                        value,
                        family == ShuffleFamily::ShuffleHighWords,
                    )
                })
                .unwrap_or_default()
        }
        ShuffleFamily::ShuffleMultimediaWords => {
            if register {
                first = operands.name(1);
            }
            immediate
                .map(|value| decode_shuffle_elements(4, 16, value))
                .unwrap_or_default()
        }
        ShuffleFamily::SwapHalves => {
            if register {
                first = operands.name(1);
            }
            decode_swap_halves(2)
        }
        ShuffleFamily::Unpack(bits, high) => {
            if register {
                second = operands.counted_from_end(1);
            }
            first = operands.counted_from_end(far(2, 6));
            decode_unpack(operands.elements(bits), bits, high)
        }
        ShuffleFamily::ShufflePairs(bits) => {
            if register {
                second = operands.counted_from_end(2);
            }
            first = operands.counted_from_end(far(3, 7));
            immediate
                .map(|value| decode_shuffle_pairs(operands.elements(bits), bits, value))
                .unwrap_or_default()
        }
        ShuffleFamily::ShuffleLanes(bits) => {
            if register {
                second = operands.counted_from_end(2);
            }
            first = operands.counted_from_end(far(3, 7));
            decode_shuffle_lanes(operands.elements(bits), bits, immediate.unwrap_or(0))
        }
        ShuffleFamily::PermuteHalves => {
            if register {
                second = operands.name(2);
            }
            first = operands.name(1);
            immediate
                .map(|value| decode_permute_halves(4, value))
                .unwrap_or_default()
        }
        ShuffleFamily::Permute => {
            if register {
                first = operands.counted_from_end(2);
            }
            immediate
                .map(|value| decode_permute(operands.elements(64), value))
                .unwrap_or_default()
        }
        ShuffleFamily::ScalarMove(count) => {
            second = operands.name(2);
            first = operands.name(1);
            decode_scalar_move(count)
        }
        ShuffleFamily::ZeroMoveLow => {
            first = operands.name(1);
            std::iter::once(0)
                .chain(std::iter::repeat_n(SHUFFLE_ZERO, 1))
                .collect()
        }
        ShuffleFamily::ExtractField => {
            first = operands.name(1);
            match (operands.immediate(2), operands.immediate(3)) {
                (Some(length), Some(index)) => decode_bit_field(16, 8, length, index, false),
                _ => Vec::new(),
            }
        }
        ShuffleFamily::InsertField => {
            first = operands.name(1);
            second = operands.name(2);
            match (operands.immediate(3), operands.immediate(4)) {
                (Some(length), Some(index)) => decode_bit_field(16, 8, length, index, true),
                _ => Vec::new(),
            }
        }
        ShuffleFamily::SubvectorBroadcast(count, source) => {
            if register {
                first = operands.counted_from_end(1);
            }
            decode_subvector_broadcast(count, source)
        }
        ShuffleFamily::ZeroExtend(source, target) => {
            first = operands.counted_from_end(1);
            decode_zero_extend(source, target, operands.elements(target))
        }
    };
    let mut mask = mask;
    if mask.is_empty() {
        return None;
    }
    if destination.is_none() {
        destination = first.clone();
    }
    let mut text = match &destination {
        Some(name) => format!("{name}{}", operands.masking()),
        None => "mem".to_string(),
    };
    text.push_str(" = ");
    let size = mask.len() as i64;
    if first == second {
        for value in mask.iter_mut() {
            if *value >= size {
                *value -= size;
            }
        }
    }
    let mut index = 0;
    while index < mask.len() {
        if index != 0 {
            text.push(',');
        }
        if mask[index] == SHUFFLE_ZERO {
            text.push_str("zero");
            index += 1;
            continue;
        }
        let from_first = mask[index] < size;
        let source = if from_first { &first } else { &second };
        text.push_str(source.as_deref().unwrap_or("mem"));
        text.push('[');
        let mut leading = true;
        while index < mask.len()
            && mask[index] != SHUFFLE_ZERO
            && (mask[index] < size) == from_first
        {
            if !leading {
                text.push(',');
            }
            leading = false;
            if mask[index] == SHUFFLE_UNDEFINED {
                text.push('u');
            } else {
                text.push_str(&(mask[index] % size).to_string());
            }
            index += 1;
        }
        text.push(']');
    }
    text.push('\n');
    Some(text)
}

pub fn instruction_comment(target: &Target, instruction: &MachineInstruction) -> Option<String> {
    let record = target.instructions.get(instruction.opcode)?.record.as_str();
    let rule = *comment_rules().rules.get(record)?;
    let operands = CommentOperands {
        target,
        instruction,
    };
    match rule {
        CommentRule::Fused {
            order,
            register,
            accumulate,
            negate,
        } => Some(fused_comment(
            &operands, order, register, accumulate, negate,
        )),
        CommentRule::Ternary(source) => ternary_comment(&operands, source),
        CommentRule::FloatClass(register) => float_class_comment(&operands, register),
        CommentRule::Shuffle(family, register) => shuffle_comment(&operands, family, register),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassKind {
    Token,
    Register,
    User,
}

#[derive(Clone, Debug)]
pub struct MatchClass {
    pub kind: ClassKind,
    pub value_name: String,
    pub name: String,
    pub registers: BTreeSet<RegisterId>,
    pub super_classes: Vec<usize>,
    pub render: String,
}

#[derive(Clone, Debug)]
pub struct MatchOperand {
    pub token: String,
    pub class: usize,
    pub source: String,
    pub original: String,
    pub sub_index: Option<usize>,
    pub isolated: bool,
    pub singleton: Option<RegisterId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Conversion {
    Render {
        operand: usize,
        count: usize,
    },
    Tied {
        result: Option<usize>,
        first: usize,
        second: usize,
    },
    Immediate(i64),
    Register(RegisterId),
}

#[derive(Clone, Debug)]
pub struct Matchable {
    pub variant: usize,
    pub mnemonic: String,
    pub operands: Vec<MatchOperand>,
    pub result: usize,
    pub record: String,
    pub alias: Option<usize>,
    pub modes: [bool; 3],
    pub required_count: usize,
    pub conversions: Vec<Conversion>,
}

pub type MnemonicAliasTable = BTreeMap<String, Vec<(String, [bool; 3], bool)>>;

pub struct Matcher {
    pub classes: Vec<MatchClass>,
    pub matchables: Vec<Matchable>,
    pub ranges: BTreeMap<(usize, String), (usize, usize)>,
    pub maximum_operands: usize,
    pub mnemonic_aliases: Vec<MnemonicAliasTable>,
    pub feature_names: BTreeMap<String, String>,
}

fn tokenize_assembly(text: &str) -> Vec<(String, bool)> {
    let characters: Vec<char> = text.chars().collect();
    let tokenizing = ['[', ']', '*', '!'];
    let separators = [' ', '\t', ','];
    let mut tokens: Vec<(String, bool)> = Vec::new();
    let mut previous = 0;
    let mut in_token = false;
    let mut isolated = true;
    let mut index = 0;
    let slice = |from: usize, to: usize| -> String { characters[from..to].iter().collect() };
    while index < characters.len() {
        let character = characters[index];
        if tokenizing.contains(&character) {
            if in_token {
                tokens.push((slice(previous, index), isolated));
                in_token = false;
                isolated = false;
            }
            tokens.push((slice(index, index + 1), isolated));
            previous = index + 1;
            isolated = true;
            index += 1;
            continue;
        }
        if separators.contains(&character) {
            if in_token {
                tokens.push((slice(previous, index), isolated));
                in_token = false;
            }
            previous = index + 1;
            isolated = true;
            index += 1;
            continue;
        }
        match character {
            '\\' => {
                if in_token {
                    tokens.push((slice(previous, index), false));
                    in_token = false;
                    isolated = false;
                }
                index += 1;
                if index < characters.len() {
                    tokens.push((slice(index, index + 1), isolated));
                }
                previous = index + 1;
                isolated = false;
            }
            '$' => {
                if in_token {
                    tokens.push((slice(previous, index), isolated));
                    in_token = false;
                    isolated = false;
                }
                if index + 1 == characters.len() || characters[index + 1] != '{' {
                    previous = index;
                    in_token = true;
                    index += 1;
                    continue;
                }
                let end = (index..characters.len())
                    .find(|at| characters[*at] == '}')
                    .unwrap_or(characters.len() - 1);
                tokens.push((slice(index, end + 1), isolated));
                previous = end + 1;
                index = end;
                isolated = false;
            }
            _ => {
                in_token = true;
            }
        }
        index += 1;
    }
    if in_token && previous != characters.len() {
        tokens.push((slice(previous, characters.len()), isolated));
    }
    tokens
}

fn compare_register_classes(
    target: &Target,
    left: &RegisterClass,
    right: &RegisterClass,
) -> std::cmp::Ordering {
    let size = |class: &RegisterClass| {
        target
            .records
            .get(&class.record)
            .map(|record| {
                (
                    table_integer(record, "Size"),
                    table_integer(record, "Alignment"),
                )
            })
            .unwrap_or((0, 0))
    };
    size(left)
        .cmp(&size(right))
        .then(right.members.len().cmp(&left.members.len()))
        .then(left.record.cmp(&right.record))
}

impl Matcher {
    fn class_root(&self, class: usize) -> usize {
        let mut at = class;
        let mut guard = 0;
        while let Some(first) = self.classes[at].super_classes.first() {
            at = *first;
            guard += 1;
            if guard > 64 {
                break;
            }
        }
        at
    }

    fn class_depth(&self, class: usize) -> usize {
        let mut at = class;
        let mut depth = 0;
        while let Some(first) = self.classes[at].super_classes.first() {
            at = *first;
            depth += 1;
            if depth > 64 {
                break;
            }
        }
        depth
    }

    fn class_less(&self, left: usize, right: usize) -> bool {
        if left == right {
            return false;
        }
        let a = &self.classes[left];
        let b = &self.classes[right];
        match (a.kind, b.kind) {
            (ClassKind::Token, ClassKind::Token) => {}
            (ClassKind::Token, _) => return true,
            (ClassKind::Register, ClassKind::Token) => return false,
            (ClassKind::Register, ClassKind::User) => return true,
            (ClassKind::Register, ClassKind::Register) => {}
            (ClassKind::User, ClassKind::User) => {}
            (ClassKind::User, _) => return false,
        }
        match a.kind {
            ClassKind::Token | ClassKind::User => {
                let related =
                    a.kind == ClassKind::Token || self.class_root(left) == self.class_root(right);
                if related {
                    let (depth_a, depth_b) = (self.class_depth(left), self.class_depth(right));
                    if depth_a > depth_b {
                        return true;
                    }
                    if depth_a < depth_b {
                        return false;
                    }
                } else {
                    return self.classes[self.class_root(left)].value_name
                        < self.classes[self.class_root(right)].value_name;
                }
            }
            ClassKind::Register => {
                if a.registers.len() != b.registers.len() {
                    return a.registers.len() < b.registers.len();
                }
            }
        }
        a.value_name < b.value_name
    }

    pub fn is_subclass(&self, child: usize, parent: usize) -> bool {
        if child == parent {
            return true;
        }
        let mut stack: Vec<usize> = self.classes[child].super_classes.clone();
        let mut seen = BTreeSet::new();
        while let Some(at) = stack.pop() {
            if at == parent {
                return true;
            }
            if seen.insert(at) {
                stack.extend(self.classes[at].super_classes.iter().copied());
            }
        }
        false
    }

    fn matched_before(&self, left: &Matchable, right: &Matchable, target: &Target) -> bool {
        let lower_left = left.mnemonic.to_ascii_lowercase();
        let lower_right = right.mnemonic.to_ascii_lowercase();
        if lower_left != lower_right {
            return lower_left < lower_right;
        }
        if left.operands.len() != right.operands.len() {
            return left.operands.len() < right.operands.len();
        }
        for (a, b) in left.operands.iter().zip(right.operands.iter()) {
            if self.class_less(a.class, b.class) {
                return true;
            }
            if self.class_less(b.class, a.class) {
                return false;
            }
        }
        let first = &target.instructions[left.result];
        let second = &target.instructions[right.result];
        if first.has_position_order && second.has_position_order {
            return first.definition_order < second.definition_order;
        }
        if left.required_count != right.required_count {
            return left.required_count > right.required_count;
        }
        false
    }
}

struct MatcherBuilder<'t> {
    target: &'t Target,
    classes: Vec<MatchClass>,
    token_classes: BTreeMap<String, usize>,
    register_set_classes: BTreeMap<Vec<RegisterId>, usize>,
    user_classes: BTreeMap<String, usize>,
}

impl<'t> MatcherBuilder<'t> {
    fn token_class(&mut self, token: &str) -> usize {
        if let Some(found) = self.token_classes.get(token) {
            return *found;
        }
        let made = self.classes.len();
        self.classes.push(MatchClass {
            kind: ClassKind::Token,
            value_name: token.to_string(),
            name: token.to_string(),
            registers: BTreeSet::new(),
            super_classes: Vec::new(),
            render: String::new(),
        });
        self.token_classes.insert(token.to_string(), made);
        made
    }

    fn register_set_class(&mut self, registers: &BTreeSet<RegisterId>) -> usize {
        let key: Vec<RegisterId> = registers.iter().copied().collect();
        if let Some(found) = self.register_set_classes.get(&key) {
            return *found;
        }
        let mut named: Vec<&RegisterClass> = self
            .target
            .register_classes
            .values()
            .filter(|class| class.member_set == *registers)
            .collect();
        named.sort_by(|left, right| compare_register_classes(self.target, left, right));
        let mut value_name = named
            .iter()
            .map(|class| class.record.clone())
            .collect::<Vec<_>>()
            .join(",");
        if registers.len() == 1 {
            let only = *registers.iter().next().unwrap_or(&NO_REGISTER);
            let register_name = self.target.registers[only as usize].record.clone();
            if value_name.is_empty() {
                value_name = register_name;
            } else {
                value_name = format!("{value_name},{register_name}");
            }
        }
        let made = self.classes.len();
        self.classes.push(MatchClass {
            kind: ClassKind::Register,
            value_name,
            name: String::new(),
            registers: registers.clone(),
            super_classes: Vec::new(),
            render: "addRegOperands".to_string(),
        });
        self.register_set_classes.insert(key, made);
        made
    }

    fn user_class(&mut self, record: &str) -> Option<usize> {
        if let Some(found) = self.user_classes.get(record) {
            return Some(*found);
        }
        let held = self
            .target
            .assembly_operand_classes
            .iter()
            .find(|class| class.record == record)?
            .clone();
        let made = self.classes.len();
        self.classes.push(MatchClass {
            kind: ClassKind::User,
            value_name: held.record.clone(),
            name: held.name.clone(),
            registers: BTreeSet::new(),
            super_classes: Vec::new(),
            render: if held.render_method.is_empty() {
                format!("add{}Operands", held.name)
            } else {
                held.render_method.clone()
            },
        });
        self.user_classes.insert(record.to_string(), made);
        let mut supers = Vec::new();
        for parent in &held.super_classes {
            if let Some(found) = self.user_class(parent) {
                supers.push(found);
            }
        }
        self.classes[made].super_classes = supers;
        Some(made)
    }

    fn operand_class(&mut self, record: &str) -> Option<usize> {
        let target = self.target;
        if let Some(class) = target.register_classes.get(record) {
            let set = class.member_set.clone();
            return Some(self.register_set_class(&set));
        }
        let held = target.operand_records.get(record)?.clone();
        if held.is_register_operand {
            if !held.match_class.is_empty() {
                if let Some(found) = self.user_class(&held.match_class) {
                    return Some(found);
                }
            }
            let set = target
                .register_classes
                .get(&held.register_class)?
                .member_set
                .clone();
            return Some(self.register_set_class(&set));
        }
        self.user_class(&held.match_class)
    }
}

pub fn build_matcher(target: &Target) -> Matcher {
    let mut builder = MatcherBuilder {
        target,
        classes: Vec::new(),
        token_classes: BTreeMap::new(),
        register_set_classes: BTreeMap::new(),
        user_classes: BTreeMap::new(),
    };
    let variants = [(0usize, "att", "%"), (1usize, "intel", "")];
    let mut matchables: Vec<Matchable> = Vec::new();
    let is_mode_predicate = |predicate: &String| {
        matches!(
            target
                .records
                .get(predicate)
                .and_then(|record| record.field("AssemblerCondDag")),
            Some(TableValue::Dag { .. })
        )
    };
    for (variant, variant_name, prefix) in variants {
        let mut prepared: Vec<(Matchable, Vec<(String, bool)>)> = Vec::new();
        for (opcode, instruction) in target.instructions.iter().enumerate() {
            if instruction.code_generation_only {
                continue;
            }
            if !instruction.variant_name.is_empty() && instruction.variant_name != variant_name {
                continue;
            }
            let flat = flatten_variants(&instruction.assembly, variant);
            let mut tokens = tokenize_assembly(&flat);
            if tokens.is_empty() {
                continue;
            }
            let mnemonic = tokens.remove(0).0;
            if mnemonic.starts_with('$') {
                continue;
            }
            let mut seen = BTreeSet::new();
            let valid = tokens.iter().all(|(token, _)| {
                !(token.starts_with('$') && token.contains(':'))
                    && (!token.starts_with('$') || seen.insert(token.clone()))
            });
            if !valid || flat.contains('\n') {
                continue;
            }
            prepared.push((
                Matchable {
                    variant,
                    mnemonic,
                    operands: Vec::new(),
                    result: opcode,
                    record: instruction.record.clone(),
                    alias: None,
                    modes: instruction.modes,
                    required_count: instruction
                        .predicates
                        .iter()
                        .filter(|p| is_mode_predicate(p))
                        .count(),
                    conversions: Vec::new(),
                },
                tokens,
            ));
        }
        for (alias_index, alias) in target.aliases.iter().enumerate() {
            if !alias.variant_name.is_empty() && alias.variant_name != variant_name {
                continue;
            }
            let flat = flatten_variants(&alias.assembly, variant);
            let mut tokens = tokenize_assembly(&flat);
            if tokens.is_empty() {
                continue;
            }
            let mnemonic = tokens.remove(0).0;
            prepared.push((
                Matchable {
                    variant,
                    mnemonic,
                    operands: Vec::new(),
                    result: alias.result,
                    record: alias.record.clone(),
                    alias: Some(alias_index),
                    modes: alias.modes,
                    required_count: alias
                        .predicates
                        .iter()
                        .filter(|p| is_mode_predicate(p))
                        .count(),
                    conversions: Vec::new(),
                },
                tokens,
            ));
        }
        for (mut matchable, tokens) in prepared {
            let mut operands = Vec::new();
            for (token, isolated) in tokens {
                let mut singleton = None;
                if isolated {
                    let lowered = token.to_ascii_lowercase();
                    let name = if prefix.is_empty() {
                        Some(lowered.as_str())
                    } else {
                        token.strip_prefix(prefix)
                    };
                    if let Some(name) = name {
                        if let Some(register) = target.register_by_name.get(name) {
                            singleton = Some(*register);
                        }
                    }
                }
                operands.push(MatchOperand {
                    token,
                    class: usize::MAX,
                    source: String::new(),
                    original: String::new(),
                    sub_index: None,
                    isolated,
                    singleton,
                });
            }
            matchable.operands = operands;
            if !assign_classes(&mut builder, &mut matchable) {
                continue;
            }
            matchables.push(matchable);
        }
    }
    let mut matcher = Matcher {
        classes: builder.classes,
        matchables: Vec::new(),
        ranges: BTreeMap::new(),
        maximum_operands: 0,
        mnemonic_aliases: Vec::new(),
        feature_names: BTreeMap::new(),
    };
    let register_classes: Vec<usize> = (0..matcher.classes.len())
        .filter(|at| matcher.classes[*at].kind == ClassKind::Register)
        .collect();
    for first in &register_classes {
        for second in &register_classes {
            if first != second
                && matcher.classes[*first]
                    .registers
                    .is_subset(&matcher.classes[*second].registers)
            {
                matcher.classes[*first].super_classes.push(*second);
            }
        }
    }
    for (from, to) in &target.token_aliases {
        let from_class = matcher
            .classes
            .iter()
            .position(|class| class.kind == ClassKind::Token && class.value_name == *from);
        let to_class = matcher
            .classes
            .iter()
            .position(|class| class.kind == ClassKind::Token && class.value_name == *to);
        if let (Some(from_class), Some(to_class)) = (from_class, to_class) {
            matcher.classes[from_class].super_classes.push(to_class);
        }
    }
    let mut indices: Vec<usize> = (0..matchables.len()).collect();
    indices.sort_by(|left, right| {
        let a = &matchables[*left];
        let b = &matchables[*right];
        if matcher.matched_before(a, b, target) {
            std::cmp::Ordering::Less
        } else if matcher.matched_before(b, a, target) {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
    let mut sorted: Vec<Matchable> = indices
        .into_iter()
        .map(|at| matchables[at].clone())
        .collect();
    sorted.sort_by_key(|matchable| matchable.variant);
    for (at, matchable) in sorted.iter().enumerate() {
        matcher.maximum_operands = matcher.maximum_operands.max(matchable.operands.len());
        let key = (matchable.variant, matchable.mnemonic.to_ascii_lowercase());
        let entry = matcher.ranges.entry(key).or_insert((at, at));
        entry.1 = at + 1;
    }
    matcher.matchables = sorted;
    for variant_name in ["att", "intel"] {
        let mut aliases: BTreeMap<String, Vec<(String, [bool; 3], bool)>> = BTreeMap::new();
        for alias in &target.mnemonic_aliases {
            if alias.variant_name != variant_name && !alias.variant_name.is_empty() {
                continue;
            }
            aliases
                .entry(alias.from.to_ascii_lowercase())
                .or_default()
                .push((
                    alias.to.to_ascii_lowercase(),
                    alias.modes,
                    alias.has_predicates,
                ));
        }
        matcher.mnemonic_aliases.push(aliases);
    }
    for record in target.records.values() {
        let name = table_text(record, "PredicateName");
        if !name.is_empty() {
            matcher.feature_names.insert(record.name.clone(), name);
        }
    }
    matcher
}

fn assign_classes(builder: &mut MatcherBuilder, matchable: &mut Matchable) -> bool {
    let target = builder.target;
    let mut index = 0;
    while index < matchable.operands.len() {
        let operand = matchable.operands[index].clone();
        if let Some(register) = operand.singleton {
            let set: BTreeSet<RegisterId> = [register].into_iter().collect();
            matchable.operands[index].class = builder.register_set_class(&set);
            index += 1;
            continue;
        }
        let token = operand.token.clone();
        if !token.starts_with('$') || token[1..].starts_with(|c: char| c.is_ascii_digit()) {
            matchable.operands[index].class = builder.token_class(&token);
            index += 1;
            continue;
        }
        let name = if token.starts_with("${") {
            token[2..token.len() - 1].to_string()
        } else {
            token[1..].to_string()
        };
        match matchable.alias {
            None => {
                let instruction = &target.instructions[matchable.result];
                let Some(group_index) = instruction.named_group(&name) else {
                    return false;
                };
                let group = &instruction.groups[group_index];
                let Some(class) = builder.operand_class(&group.record) else {
                    return false;
                };
                let entry = &mut matchable.operands[index];
                entry.class = class;
                entry.original = name.clone();
                entry.source = name;
                if group.count == 1 {
                    if let Some(tied) = group.tied_to {
                        entry.source = instruction.groups[tied].name.clone();
                        entry.sub_index = Some(0);
                    }
                }
            }
            Some(alias_index) => {
                let alias = &target.aliases[alias_index];
                let Some((operands, indices)) = target.alias_result_operands(alias) else {
                    return false;
                };
                let mut found = false;
                for (result, (_, sub_index)) in operands.iter().zip(indices.iter()) {
                    if let ResultOperand::Record {
                        name: result_name,
                        record,
                    } = result
                    {
                        if *result_name == name {
                            let Some(class) = builder.operand_class(record) else {
                                return false;
                            };
                            let entry = &mut matchable.operands[index];
                            entry.class = class;
                            entry.sub_index = *sub_index;
                            entry.source = name.clone();
                            entry.original = name.clone();
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    return false;
                }
            }
        }
        index += 1;
    }
    match matchable.alias {
        None => build_instruction_conversions(target, matchable),
        Some(alias_index) => build_alias_conversions(target, alias_index, matchable),
    }
}

fn find_named(matchable: &Matchable, name: &str, after: Option<usize>) -> Option<usize> {
    let start = after.map(|at| at + 1).unwrap_or(0);
    (start..matchable.operands.len()).find(|at| matchable.operands[*at].source == name)
}

fn build_instruction_conversions(target: &Target, matchable: &mut Matchable) -> bool {
    let instruction = &target.instructions[matchable.result];
    let mut conversions: Vec<Conversion> = Vec::new();
    for group in &instruction.groups {
        if group.count == 1 {
            if let Some(tied) = group.tied_to {
                let tied_index = instruction.groups[tied].first;
                let original = matchable
                    .operands
                    .iter()
                    .position(|operand| operand.original == group.name);
                match (original, conversions.get(tied_index)) {
                    (Some(second), Some(Conversion::Render { operand, .. })) => {
                        let first = *operand;
                        conversions.push(Conversion::Tied {
                            result: Some(tied_index),
                            first,
                            second,
                        });
                    }
                    _ => conversions.push(Conversion::Tied {
                        result: Some(tied_index),
                        first: 0,
                        second: 0,
                    }),
                }
                continue;
            }
        }
        let source = if group.name.is_empty() {
            None
        } else {
            find_named(matchable, &group.name, None)
        };
        let Some(source) = source else {
            for _ in 0..group.count {
                conversions.push(Conversion::Immediate(0));
            }
            continue;
        };
        if matchable.operands[source].sub_index.is_none() {
            conversions.push(Conversion::Render {
                operand: source,
                count: group.count,
            });
            for _ in 1..group.count {
                conversions.push(Conversion::Immediate(i64::MIN));
            }
            continue;
        }
        for offset in 0..group.count {
            conversions.push(Conversion::Render {
                operand: source + offset,
                count: 1,
            });
        }
    }
    conversions.retain(|conversion| *conversion != Conversion::Immediate(i64::MIN));
    matchable.conversions = conversions;
    true
}

fn build_alias_conversions(target: &Target, alias_index: usize, matchable: &mut Matchable) -> bool {
    let alias = &target.aliases[alias_index];
    let instruction = &target.instructions[alias.result];
    let Some((operands, indices)) = target.alias_result_operands(alias) else {
        return false;
    };
    let mut conversions: Vec<Conversion> = Vec::new();
    let mut result_slots: Vec<usize> = Vec::new();
    let mut references: BTreeMap<String, Option<usize>> = BTreeMap::new();
    let mut alias_number = 0;
    let last = indices.len();
    for (group_index, group) in instruction.groups.iter().enumerate() {
        if group.count == 1 {
            if let Some(tied) = group.tied_to {
                let tied_slot = instruction.groups[tied].first;
                let mut first = 0;
                let mut second = 0;
                if let Some(Conversion::Render { operand, .. }) = result_slots
                    .get(tied_slot)
                    .and_then(|slot| conversions.get(*slot))
                {
                    first = *operand;
                    let name = matchable.operands[first].source.clone();
                    let entry = references.entry(name.clone()).or_insert(Some(first));
                    let found = find_named(matchable, &name, *entry);
                    *entry = found;
                    second = found.unwrap_or(first);
                }
                if group.record != instruction.groups[tied].record {
                    let first = match result_slots
                        .get(tied_slot)
                        .and_then(|slot| conversions.get(*slot))
                    {
                        Some(Conversion::Render { operand, .. }) => *operand,
                        _ => 0,
                    };
                    let (name, sub_index) =
                        match (&operands.get(alias_number), indices.get(alias_number)) {
                            (Some(ResultOperand::Record { name, .. }), Some((_, sub_index))) => {
                                (name.clone(), *sub_index)
                            }
                            _ => (String::new(), None),
                        };
                    let second = matchable
                        .operands
                        .iter()
                        .position(|operand| {
                            operand.source == name && operand.sub_index == sub_index
                        })
                        .unwrap_or(usize::MAX);
                    result_slots.push(conversions.len());
                    conversions.push(Conversion::Tied {
                        result: None,
                        first,
                        second,
                    });
                } else {
                    result_slots.push(conversions.len());
                    conversions.push(Conversion::Tied {
                        result: Some(tied_slot),
                        first,
                        second,
                    });
                    continue;
                }
            }
        }
        while alias_number < last && indices[alias_number].0 == group_index {
            let sub_index = indices[alias_number].1;
            match &operands[alias_number] {
                ResultOperand::Record { name, .. } => {
                    let Some(source) = matchable.operands.iter().position(|operand| {
                        operand.source == *name && operand.sub_index == sub_index
                    }) else {
                        return false;
                    };
                    references.entry(name.clone()).or_insert(Some(source));
                    let count = if sub_index.is_none() { group.count } else { 1 };
                    let slot_index = conversions.len();
                    for _ in 0..count {
                        result_slots.push(slot_index);
                    }
                    conversions.push(Conversion::Render {
                        operand: source,
                        count,
                    });
                }
                ResultOperand::Immediate(value) => {
                    result_slots.push(conversions.len());
                    conversions.push(Conversion::Immediate(*value));
                }
                ResultOperand::Register(register) => {
                    result_slots.push(conversions.len());
                    conversions.push(Conversion::Register(*register));
                }
            }
            alias_number += 1;
        }
    }
    matchable.conversions = conversions;
    true
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryOperand {
    pub segment: RegisterId,
    pub displacement: Expression,
    pub base: RegisterId,
    pub default_base: RegisterId,
    pub index: RegisterId,
    pub scale: u32,
    pub size: u32,
    pub mode_size: u32,
    pub frontend_size: u32,
    pub maybe_direct_branch: bool,
}

impl MemoryOperand {
    pub fn absolute(mode_size: u32, displacement: Expression) -> MemoryOperand {
        MemoryOperand {
            segment: NO_REGISTER,
            displacement,
            base: NO_REGISTER,
            default_base: NO_REGISTER,
            index: NO_REGISTER,
            scale: 1,
            size: 0,
            mode_size,
            frontend_size: 0,
            maybe_direct_branch: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParsedKind {
    Token(String),
    Register(RegisterId),
    DxRegister,
    Immediate(Expression),
    Memory(MemoryOperand),
    Prefix(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedOperand {
    pub kind: ParsedKind,
    pub start: usize,
    pub end: usize,
}

impl ParsedOperand {
    pub fn token(text: &str, start: usize) -> ParsedOperand {
        ParsedOperand {
            kind: ParsedKind::Token(text.to_string()),
            start,
            end: start + text.len(),
        }
    }

    pub fn is_token(&self) -> bool {
        matches!(self.kind, ParsedKind::Token(_))
    }

    pub fn token_text(&self) -> &str {
        match &self.kind {
            ParsedKind::Token(text) => text,
            _ => "",
        }
    }

    pub fn memory(&self) -> Option<&MemoryOperand> {
        match &self.kind {
            ParsedKind::Memory(memory) => Some(memory),
            _ => None,
        }
    }
}

pub fn is_immediate_sign_extended_16_from_8(value: i64) -> bool {
    (-128..=127).contains(&value)
        || ((0..=0xffff).contains(&value) && (-128..=127).contains(&(value as i16)))
}

pub fn is_immediate_sign_extended_32_from_8(value: i64) -> bool {
    (-128..=127).contains(&value)
        || ((0..=0xffff_ffff).contains(&value) && (-128..=127).contains(&(value as i32)))
}

impl Target {
    fn memory_index_in(
        &self,
        register: RegisterId,
        prefix: &str,
        lowest: u32,
        highest: u32,
    ) -> bool {
        register != NO_REGISTER
            && self
                .vector_number(register, prefix)
                .is_some_and(|number| (lowest..=highest).contains(&number))
    }

    pub fn operand_predicate(&self, name: &str, operand: &ParsedOperand) -> bool {
        let constant = |operand: &ParsedOperand| match &operand.kind {
            ParsedKind::Immediate(Expression::Constant(value)) => Some(Some(*value)),
            ParsedKind::Immediate(_) => Some(None),
            _ => None,
        };
        let memory = operand.memory();
        let sized = |bits: u32| memory.is_some_and(|held| held.size == 0 || held.size == bits);
        let is_gr = |class: &str, register: RegisterId| self.in_class(class, register);
        match name {
            "Imm" | "AVX512RC" => constant(operand).is_some(),
            "ImmSExti16i8" => match constant(operand) {
                Some(Some(value)) => is_immediate_sign_extended_16_from_8(value),
                Some(None) => true,
                None => false,
            },
            "ImmSExti32i8" => match constant(operand) {
                Some(Some(value)) => is_immediate_sign_extended_32_from_8(value),
                Some(None) => true,
                None => false,
            },
            "ImmSExti64i8" => match constant(operand) {
                Some(Some(value)) => (-128..=127).contains(&value),
                Some(None) => true,
                None => false,
            },
            "ImmSExti64i32" => match constant(operand) {
                Some(Some(value)) => i32::try_from(value).is_ok(),
                Some(None) => true,
                None => false,
            },
            "ImmUnsignedi4" => matches!(constant(operand), Some(Some(value)) if (0..16).contains(&value)),
            "ImmUnsignedi6" => matches!(constant(operand), Some(Some(value)) if (0..64).contains(&value)),
            "ImmUnsignedi8" => match constant(operand) {
                Some(Some(value)) => (-128..=255).contains(&value),
                Some(None) => true,
                None => false,
            },
            "Mem" => memory.is_some(),
            "Mem8" => sized(8),
            "Mem16" => sized(16),
            "Mem32" => sized(32),
            "Mem64" => sized(64),
            "Mem80" => sized(80),
            "Mem128" => sized(128),
            "Mem256" => sized(256),
            "Mem512" => sized(512),
            "SibMem" => memory.is_some_and(|held| {
                held.base != self.register("RIP") && held.base != self.register("EIP")
            }),
            "Mem32_RC128" => sized(32) && memory.is_some_and(|held| self.memory_index_in(held.index, "XMM", 0, 15)),
            "Mem64_RC128" => sized(64) && memory.is_some_and(|held| self.memory_index_in(held.index, "XMM", 0, 15)),
            "Mem32_RC256" => sized(32) && memory.is_some_and(|held| self.memory_index_in(held.index, "YMM", 0, 15)),
            "Mem64_RC256" => sized(64) && memory.is_some_and(|held| self.memory_index_in(held.index, "YMM", 0, 15)),
            "Mem32_RC128X" => sized(32) && memory.is_some_and(|held| self.memory_index_in(held.index, "XMM", 0, 31)),
            "Mem64_RC128X" => sized(64) && memory.is_some_and(|held| self.memory_index_in(held.index, "XMM", 0, 31)),
            "Mem32_RC256X" => sized(32) && memory.is_some_and(|held| self.memory_index_in(held.index, "YMM", 0, 31)),
            "Mem64_RC256X" => sized(64) && memory.is_some_and(|held| self.memory_index_in(held.index, "YMM", 0, 31)),
            "Mem32_RC512" => sized(32) && memory.is_some_and(|held| self.memory_index_in(held.index, "ZMM", 0, 31)),
            "Mem64_RC512" => sized(64) && memory.is_some_and(|held| self.memory_index_in(held.index, "ZMM", 0, 31)),
            "Mem512_GR16" => {
                sized(512)
                    && memory.is_some_and(|held| held.base == NO_REGISTER || is_gr("GR16", held.base))
            }
            "Mem512_GR32" => {
                sized(512)
                    && memory.is_some_and(|held| {
                        (held.base == NO_REGISTER
                            || is_gr("GR32", held.base)
                            || held.base == self.register("EIP"))
                            && (held.index == NO_REGISTER
                                || is_gr("GR32", held.index)
                                || held.index == self.register("EIZ"))
                    })
            }
            "Mem512_GR64" => {
                sized(512)
                    && memory.is_some_and(|held| {
                        (held.base == NO_REGISTER
                            || is_gr("GR64", held.base)
                            || held.base == self.register("RIP"))
                            && (held.index == NO_REGISTER
                                || is_gr("GR64", held.index)
                                || held.index == self.register("RIZ"))
                    })
            }
            "AbsMem" | "AbsMem8" | "AbsMemMode16" => {
                let absolute = memory.is_some_and(|held| {
                    held.segment == NO_REGISTER
                        && held.base == NO_REGISTER
                        && held.index == NO_REGISTER
                        && held.scale == 1
                        && held.maybe_direct_branch
                });
                match name {
                    "AbsMem" => absolute,
                    "AbsMemMode16" => absolute && memory.is_some_and(|held| held.mode_size == 16),
                    _ => {
                        absolute
                            && sized(8)
                            && memory.is_some_and(|held| match held.displacement {
                                Expression::Constant(value) => (-128..=127).contains(&value),
                                _ => true,
                            })
                    }
                }
            }
            "SrcIdx8" | "SrcIdx16" | "SrcIdx32" | "SrcIdx64" => {
                let bits: u32 = name[6..].parse().unwrap_or(8);
                sized(bits)
                    && memory.is_some_and(|held| {
                        held.index == NO_REGISTER
                            && held.scale == 1
                            && ["RSI", "ESI", "SI"]
                                .iter()
                                .any(|record| held.base == self.register(record))
                            && held.displacement == Expression::Constant(0)
                    })
            }
            "DstIdx8" | "DstIdx16" | "DstIdx32" | "DstIdx64" => {
                let bits: u32 = name[6..].parse().unwrap_or(8);
                sized(bits)
                    && memory.is_some_and(|held| {
                        held.index == NO_REGISTER
                            && held.scale == 1
                            && (held.segment == NO_REGISTER || held.segment == self.register("ES"))
                            && ["RDI", "EDI", "DI"]
                                .iter()
                                .any(|record| held.base == self.register(record))
                            && held.displacement == Expression::Constant(0)
                    })
            }
            "MemConstOffs64" => memory.is_some_and(|held| {
                held.base == NO_REGISTER
                    && held.index == NO_REGISTER
                    && held.scale == 1
                    && held.mode_size == 64
                    && matches!(held.displacement, Expression::Constant(value) if i32::try_from(value).is_err())
            }),
            _ if name.starts_with("MemOffs") => {
                let rest = &name[7..];
                let (mode, bits) = rest.split_once('_').unwrap_or(("0", "0"));
                let mode: u32 = mode.parse().unwrap_or(0);
                let bits: u32 = bits.parse().unwrap_or(0);
                memory.is_some_and(|held| {
                    held.base == NO_REGISTER
                        && held.index == NO_REGISTER
                        && held.scale == 1
                        && held.mode_size == mode
                        && (held.size == 0 || held.size == bits)
                })
            }
            "GR32orGR64" => match operand.kind {
                ParsedKind::Register(register) => is_gr("GR32", register) || is_gr("GR64", register),
                _ => false,
            },
            "GR16orGR32orGR64" => match operand.kind {
                ParsedKind::Register(register) => {
                    is_gr("GR16", register) || is_gr("GR32", register) || is_gr("GR64", register)
                }
                _ => false,
            },
            "VK1Pair" | "VK2Pair" | "VK4Pair" | "VK8Pair" | "VK16Pair" => match operand.kind {
                ParsedKind::Register(register) => is_gr(&name[..name.len() - 4], register),
                _ => false,
            },
            _ => false,
        }
    }

    pub fn render_operand(
        &self,
        method: &str,
        operand: &ParsedOperand,
        count: usize,
        into: &mut Vec<MachineOperand>,
    ) {
        let expression = |expression: &Expression| match expression {
            Expression::Constant(value) => MachineOperand::Immediate(*value),
            other => MachineOperand::Expression(other.clone()),
        };
        match (&operand.kind, method) {
            (ParsedKind::Register(register), "addGR32orGR64Operands") => {
                let mut register = *register;
                if self.in_class("GR64", register) {
                    register = self.sub_super_register(register, 32, false);
                }
                into.push(MachineOperand::Register(register));
            }
            (ParsedKind::Register(register), "addGR16orGR32orGR64Operands") => {
                let mut register = *register;
                if self.in_class("GR32", register) || self.in_class("GR64", register) {
                    register = self.sub_super_register(register, 16, false);
                }
                into.push(MachineOperand::Register(register));
            }
            (ParsedKind::Register(register), "addMaskPairOperands") => {
                let name = self.registers[*register as usize].record.as_str();
                let pair = match name {
                    "K0" | "K1" => "K0_K1",
                    "K2" | "K3" => "K2_K3",
                    "K4" | "K5" => "K4_K5",
                    "K6" | "K7" => "K6_K7",
                    _ => name,
                };
                into.push(MachineOperand::Register(self.register(pair)));
            }
            (ParsedKind::Register(register), _) => into.push(MachineOperand::Register(*register)),
            (ParsedKind::Immediate(value), _) => into.push(expression(value)),
            (ParsedKind::Memory(memory), "addMemOperands") => {
                let base = if memory.base != NO_REGISTER {
                    memory.base
                } else {
                    memory.default_base
                };
                into.push(MachineOperand::Register(base));
                into.push(MachineOperand::Immediate(memory.scale as i64));
                into.push(MachineOperand::Register(memory.index));
                into.push(expression(&memory.displacement));
                into.push(MachineOperand::Register(memory.segment));
            }
            (ParsedKind::Memory(memory), "addAbsMemOperands") => {
                into.push(expression(&memory.displacement))
            }
            (ParsedKind::Memory(memory), "addSrcIdxOperands") => {
                into.push(MachineOperand::Register(memory.base));
                into.push(MachineOperand::Register(memory.segment));
            }
            (ParsedKind::Memory(memory), "addDstIdxOperands") => {
                into.push(MachineOperand::Register(memory.base));
            }
            (ParsedKind::Memory(memory), "addMemOffsOperands") => {
                into.push(expression(&memory.displacement));
                into.push(MachineOperand::Register(memory.segment));
            }
            (ParsedKind::Memory(memory), _) => {
                into.push(expression(&memory.displacement));
                for _ in 1..count {
                    into.push(MachineOperand::Register(NO_REGISTER));
                }
            }
            _ => {
                for _ in 0..count {
                    into.push(MachineOperand::Register(NO_REGISTER));
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchStatus {
    Success,
    InvalidOperand,
    MnemonicFail,
    MissingFeature,
    Unsupported,
    InvalidTiedOperand,
    InvalidUnsigned4,
    InvalidUnsigned6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct MatchContext {
    pub forced_prefix: u32,
    pub forced_no_flags: bool,
    pub uses_extended_registers: bool,
}

pub struct MatchOutcome {
    pub status: MatchStatus,
    pub instruction: Option<MachineInstruction>,
    pub error_index: Option<usize>,
    pub missing_features: Vec<String>,
}

impl Matcher {
    fn validate_operand(
        &self,
        target: &Target,
        operand: &ParsedOperand,
        class: usize,
    ) -> MatchStatus {
        let held = &self.classes[class];
        match held.kind {
            ClassKind::Token => {
                if let ParsedKind::Token(text) = &operand.kind {
                    if let Some(found) = self.classes.iter().position(|candidate| {
                        candidate.kind == ClassKind::Token && candidate.value_name == *text
                    }) {
                        if self.is_subclass(found, class) {
                            return MatchStatus::Success;
                        }
                    }
                }
                MatchStatus::InvalidOperand
            }
            ClassKind::User => {
                if target.operand_predicate(&held.name, operand) {
                    MatchStatus::Success
                } else if held.name == "ImmUnsignedi4"
                    && matches!(operand.kind, ParsedKind::Immediate(_))
                {
                    MatchStatus::InvalidUnsigned4
                } else if held.name == "ImmUnsignedi6"
                    && matches!(operand.kind, ParsedKind::Immediate(_))
                {
                    MatchStatus::InvalidUnsigned6
                } else {
                    MatchStatus::InvalidOperand
                }
            }
            ClassKind::Register => match operand.kind {
                ParsedKind::Register(register) if held.registers.contains(&register) => {
                    MatchStatus::Success
                }
                _ => MatchStatus::InvalidOperand,
            },
        }
    }

    pub fn apply_mnemonic_alias(&self, mnemonic: &str, variant: usize, mode: Mode) -> String {
        let Some(table) = self.mnemonic_aliases.get(variant) else {
            return mnemonic.to_string();
        };
        let Some(candidates) = table.get(mnemonic) else {
            return mnemonic.to_string();
        };
        for (to, modes, has_predicates) in candidates {
            if *has_predicates && modes[mode.index()] {
                return to.clone();
            }
        }
        for (to, _, has_predicates) in candidates {
            if !*has_predicates {
                return to.clone();
            }
        }
        mnemonic.to_string()
    }

    pub fn match_instruction(
        &self,
        target: &Target,
        operands: &[ParsedOperand],
        variant: usize,
        mode: Mode,
        context: &MatchContext,
    ) -> MatchOutcome {
        let failure = |status: MatchStatus, error_index: Option<usize>| MatchOutcome {
            status,
            instruction: None,
            error_index,
            missing_features: Vec::new(),
        };
        if operands.len() > self.maximum_operands + 1 {
            return failure(MatchStatus::InvalidOperand, Some(self.maximum_operands + 1));
        }
        let raw = operands
            .first()
            .map(|operand| operand.token_text())
            .unwrap_or("");
        let mnemonic = self.apply_mnemonic_alias(raw, variant, mode);
        let Some((start, end)) = self.ranges.get(&(variant, mnemonic.clone())).copied() else {
            return failure(MatchStatus::MnemonicFail, None);
        };
        let mut had_match_other_than_features = false;
        let mut had_match_other_than_predicate = false;
        let mut result = MatchStatus::InvalidOperand;
        let mut error_index: Option<usize> = None;
        let mut missing: Option<Vec<String>> = None;
        for at in start..end {
            let entry = &self.matchables[at];
            if entry.mnemonic != mnemonic {
                continue;
            }
            let has_features = entry.modes[mode.index()];
            let mut valid = true;
            let mut actual = 1;
            for formal in 0..self.maximum_operands {
                let class = entry.operands.get(formal).map(|operand| operand.class);
                if actual >= operands.len() {
                    if class.is_none() {
                        break;
                    }
                    valid = false;
                    error_index = Some(actual);
                    break;
                }
                let diagnostic = match class {
                    None => MatchStatus::InvalidOperand,
                    Some(class) => self.validate_operand(target, &operands[actual], class),
                };
                if diagnostic == MatchStatus::Success {
                    actual += 1;
                    continue;
                }
                if !had_match_other_than_predicate
                    && (at == start || error_index.is_none_or(|held| held <= actual))
                {
                    if has_features
                        && (error_index != Some(actual)
                            || diagnostic != MatchStatus::InvalidOperand)
                    {
                        result = diagnostic;
                    }
                    error_index = Some(actual);
                }
                valid = false;
                break;
            }
            if !valid {
                continue;
            }
            if !has_features {
                had_match_other_than_features = true;
                let required: Vec<String> = target.instructions[entry.result]
                    .predicates
                    .iter()
                    .chain(
                        entry
                            .alias
                            .map(|alias| target.aliases[alias].predicates.clone())
                            .unwrap_or_default()
                            .iter(),
                    )
                    .filter(|predicate| self.feature_names.contains_key(*predicate))
                    .filter(|predicate| {
                        let modes = mode_validity(&target.records, std::slice::from_ref(predicate));
                        !modes[mode.index()]
                    })
                    .cloned()
                    .collect();
                if missing
                    .as_ref()
                    .is_none_or(|held| required.len() <= held.len())
                {
                    missing = Some(required);
                }
                continue;
            }
            let mut instruction = MachineInstruction::new(entry.result);
            for conversion in &entry.conversions {
                match conversion {
                    Conversion::Render { operand, count } => {
                        let class = &self.classes[entry.operands[*operand].class];
                        target.render_operand(
                            &class.render,
                            &operands[operand + 1],
                            *count,
                            &mut instruction.operands,
                        );
                    }
                    Conversion::Tied { result, .. } => {
                        if let Some(result) = result {
                            let copy = instruction.operand(*result).clone();
                            instruction.operands.push(copy);
                        }
                    }
                    Conversion::Immediate(value) => {
                        instruction.operands.push(MachineOperand::Immediate(*value))
                    }
                    Conversion::Register(register) => instruction
                        .operands
                        .push(MachineOperand::Register(*register)),
                }
            }
            let predicate = self.check_target_predicate(target, &instruction, mode, context);
            if predicate != MatchStatus::Success {
                result = predicate;
                had_match_other_than_predicate = true;
                continue;
            }
            for conversion in &entry.conversions {
                if let Conversion::Tied { first, second, .. } = conversion {
                    if first != second && *second != usize::MAX {
                        let left = operands.get(first + 1).map(|operand| &operand.kind);
                        let right = operands.get(second + 1).map(|operand| &operand.kind);
                        let equal = matches!((left, right), (Some(ParsedKind::Register(a)), Some(ParsedKind::Register(b))) if a == b);
                        if !equal {
                            return failure(MatchStatus::InvalidTiedOperand, Some(second + 1));
                        }
                    }
                }
            }
            return MatchOutcome {
                status: MatchStatus::Success,
                instruction: Some(instruction),
                error_index: None,
                missing_features: Vec::new(),
            };
        }
        if had_match_other_than_predicate || !had_match_other_than_features {
            return MatchOutcome {
                status: result,
                instruction: None,
                error_index,
                missing_features: Vec::new(),
            };
        }
        MatchOutcome {
            status: MatchStatus::MissingFeature,
            instruction: None,
            error_index: Some(0),
            missing_features: missing.unwrap_or_default(),
        }
    }

    fn check_target_predicate(
        &self,
        target: &Target,
        instruction: &MachineInstruction,
        mode: Mode,
        context: &MatchContext,
    ) -> MatchStatus {
        let description = &target.instructions[instruction.opcode];
        let flags = description.flags;
        let record = description.record.as_str();
        if context.uses_extended_registers
            && !target.can_use_apx_extended_register(instruction.opcode)
        {
            return MatchStatus::Unsupported;
        }
        if context.forced_no_flags == (flags & EVEX_NO_FLAGS == 0) && !record.starts_with("CFCMOV")
        {
            return MatchStatus::Unsupported;
        }
        let encoding = flags & ENCODING_MASK;
        let forced = context.forced_prefix;
        if forced & (PREFIX_USE_REX | PREFIX_USE_REX2) != 0 && encoding != 0 {
            return MatchStatus::Unsupported;
        }
        if forced & (PREFIX_USE_VEX | PREFIX_USE_VEX2 | PREFIX_USE_VEX3) != 0
            && encoding != ENCODING_VEX
        {
            return MatchStatus::Unsupported;
        }
        if forced & PREFIX_USE_EVEX != 0 {
            let mnemonic = description.intel_mnemonic.as_str();
            let compare_or_test = mnemonic == "CMP" || mnemonic == "TEST";
            if mode == Mode::Bits64 && encoding != ENCODING_EVEX && !compare_or_test {
                return MatchStatus::Unsupported;
            }
            if mode != Mode::Bits64 && encoding != ENCODING_EVEX {
                return MatchStatus::Unsupported;
            }
        }
        if flags & EXPLICIT_PREFIX_MASK == EXPLICIT_VEX_PREFIX
            && forced & (PREFIX_USE_VEX | PREFIX_USE_VEX2 | PREFIX_USE_VEX3) == 0
        {
            return MatchStatus::Unsupported;
        }
        MatchStatus::Success
    }
}

pub fn matcher(target: &Target) -> &'static Matcher {
    static MATCHER: OnceLock<Matcher> = OnceLock::new();
    MATCHER.get_or_init(|| build_matcher(target))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Error,
    Identifier,
    String,
    Integer,
    BigNumber,
    Real,
    EndOfStatement,
    EndOfFile,
    Colon,
    Space,
    Plus,
    Minus,
    Tilde,
    Slash,
    BackSlash,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Star,
    Dot,
    Comma,
    Dollar,
    Equal,
    EqualEqual,
    Pipe,
    PipePipe,
    Caret,
    Ampersand,
    AmpersandAmpersand,
    Exclamation,
    ExclamationEqual,
    Percent,
    Hash,
    HashDirective,
    Less,
    LessEqual,
    LessLess,
    LessGreater,
    Greater,
    GreaterEqual,
    GreaterGreater,
    At,
    MinusGreater,
    Question,
    Comment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub buffer: usize,
    pub start: usize,
    pub end: usize,
    pub value: u128,
}

impl Token {
    pub fn is(&self, kind: TokenKind) -> bool {
        self.kind == kind
    }

    pub fn integer(&self) -> i64 {
        self.value as u64 as i64
    }

    pub fn identifier(&self) -> String {
        if self.kind == TokenKind::String {
            string_contents(&self.text)
        } else {
            self.text.clone()
        }
    }

    pub fn location(&self) -> SourceLocation {
        SourceLocation {
            buffer: self.buffer,
            position: self.start,
        }
    }

    pub fn end_location(&self) -> SourceLocation {
        SourceLocation {
            buffer: self.buffer,
            position: self.end,
        }
    }
}

pub fn string_contents(text: &str) -> String {
    if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
        text[1..text.len() - 1].to_string()
    } else {
        text.to_string()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct SourceLocation {
    pub buffer: usize,
    pub position: usize,
}

#[derive(Clone, Debug)]
pub struct SourceBuffer {
    pub name: String,
    pub text: Vec<u8>,
    pub include_location: Option<SourceLocation>,
}

#[derive(Clone, Debug)]
struct LexerState {
    buffer: usize,
    position: usize,
    token_start: usize,
    at_start_of_line: bool,
    at_start_of_statement: bool,
}

pub struct Lexer {
    pub buffers: Vec<SourceBuffer>,
    state: LexerState,
    pending: Vec<Token>,
    pub token: Token,
    pub error: Option<(SourceLocation, String)>,
    skip_space: bool,
    peeking: bool,
    end_statement_at_end: bool,
    pub masm_integers: bool,
    consumed_end_of_statement: bool,
}

fn is_lexer_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.' | b'?' | b'@')
}

fn hexadecimal_value(byte: u8) -> u32 {
    match byte {
        b'0'..=b'9' => u32::from(byte - b'0'),
        b'a'..=b'f' => u32::from(byte - b'a') + 10,
        b'A'..=b'F' => u32::from(byte - b'A') + 10,
        _ => u32::MAX,
    }
}

fn parse_radix(text: &str, radix: u32) -> Option<u128> {
    let digits = if radix == 16 {
        text.strip_prefix("0x")
            .or_else(|| text.strip_prefix("0X"))
            .unwrap_or(text)
    } else {
        text
    };
    if digits.is_empty() {
        return None;
    }
    let mut value: u128 = 0;
    for byte in digits.bytes() {
        let digit = hexadecimal_value(byte);
        if digit >= radix {
            return None;
        }
        value = value
            .checked_mul(u128::from(radix))?
            .checked_add(u128::from(digit))?;
    }
    Some(value)
}

fn radix_name(radix: u32) -> &'static str {
    match radix {
        2 => "binary",
        8 => "octal",
        10 => "decimal",
        _ => "hexadecimal",
    }
}

impl Lexer {
    pub fn new(name: &str, text: &[u8]) -> Lexer {
        Lexer {
            buffers: vec![SourceBuffer {
                name: name.to_string(),
                text: text.to_vec(),
                include_location: None,
            }],
            state: LexerState {
                buffer: 0,
                position: 0,
                token_start: 0,
                at_start_of_line: true,
                at_start_of_statement: true,
            },
            pending: Vec::new(),
            token: Token {
                kind: TokenKind::Space,
                text: String::new(),
                buffer: 0,
                start: 0,
                end: 0,
                value: 0,
            },
            error: None,
            skip_space: true,
            peeking: false,
            end_statement_at_end: true,
            masm_integers: false,
            consumed_end_of_statement: false,
        }
    }

    pub fn add_buffer(
        &mut self,
        name: &str,
        text: Vec<u8>,
        include_location: Option<SourceLocation>,
    ) -> usize {
        self.buffers.push(SourceBuffer {
            name: name.to_string(),
            text,
            include_location,
        });
        self.buffers.len() - 1
    }

    pub fn jump_to(&mut self, location: SourceLocation) {
        self.state.buffer = location.buffer;
        self.state.position = location.position;
        self.state.token_start = location.position;
        self.pending.clear();
    }

    pub fn position(&self) -> SourceLocation {
        SourceLocation {
            buffer: self.state.buffer,
            position: self.state.position,
        }
    }

    fn byte(&self, at: usize) -> u8 {
        self.buffers[self.state.buffer]
            .text
            .get(at)
            .copied()
            .unwrap_or(0)
    }

    fn current(&self) -> u8 {
        self.byte(self.state.position)
    }

    fn at_end(&self) -> bool {
        self.state.position >= self.buffers[self.state.buffer].text.len()
    }

    fn next_character(&mut self) -> Option<u8> {
        if self.at_end() {
            return None;
        }
        let byte = self.current();
        self.state.position += 1;
        Some(byte)
    }

    fn make(&self, kind: TokenKind, start: usize, end: usize) -> Token {
        let text = &self.buffers[self.state.buffer].text;
        let end = end.min(text.len());
        let start = start.min(end);
        Token {
            kind,
            text: String::from_utf8_lossy(&text[start..end]).into_owned(),
            buffer: self.state.buffer,
            start,
            end,
            value: 0,
        }
    }

    fn make_value(&self, kind: TokenKind, start: usize, end: usize, value: u128) -> Token {
        let mut token = self.make(kind, start, end);
        token.value = value;
        token
    }

    fn error_token(&mut self, at: usize, message: &str) -> Token {
        if !self.peeking {
            self.error = Some((
                SourceLocation {
                    buffer: self.state.buffer,
                    position: at,
                },
                message.to_string(),
            ));
        }
        self.make(TokenKind::Error, at, self.state.position)
    }

    pub fn lex(&mut self) -> Token {
        self.consumed_end_of_statement = self.token.kind == TokenKind::EndOfStatement;
        let token = match self.pending.pop() {
            Some(token) => token,
            None => self.lex_token(),
        };
        self.token = token.clone();
        token
    }

    pub fn unlex(&mut self, token: Token) {
        let current = std::mem::replace(&mut self.token, token);
        self.pending.push(current);
    }

    pub fn peek(&mut self, count: usize) -> Vec<Token> {
        let mut result: Vec<Token> = Vec::new();
        let mut pending = self.pending.clone();
        while result.len() < count {
            match pending.pop() {
                Some(token) => {
                    let end = token.kind == TokenKind::EndOfFile;
                    result.push(token);
                    if end {
                        return result;
                    }
                }
                None => break,
            }
        }
        if result.len() < count {
            let saved = self.state.clone();
            let saved_error = self.error.clone();
            let saved_peeking = self.peeking;
            self.peeking = true;
            while result.len() < count {
                let token = self.lex_token();
                let end = token.kind == TokenKind::EndOfFile;
                result.push(token);
                if end {
                    break;
                }
            }
            self.state = saved;
            self.error = saved_error;
            self.peeking = saved_peeking;
        }
        result
    }

    pub fn rest_of_statement_start(&self) -> SourceLocation {
        SourceLocation {
            buffer: self.token.buffer,
            position: self.token.start,
        }
    }

    pub fn text_between(&self, start: SourceLocation, end: SourceLocation) -> String {
        if start.buffer != end.buffer {
            return String::new();
        }
        let text = &self.buffers[start.buffer].text;
        let end = end.position.min(text.len());
        let start = start.position.min(end);
        String::from_utf8_lossy(&text[start..end]).into_owned()
    }

    pub fn lex_until_end_of_statement(&mut self) -> String {
        self.pending.clear();
        self.state.position = self.token.start;
        let start = self.state.position;
        while !self.at_end() {
            let byte = self.current();
            if byte == b'#' || byte == b';' || byte == b'\n' || byte == b'\r' {
                break;
            }
            self.state.position += 1;
        }
        let text = self
            .make(TokenKind::String, start, self.state.position)
            .text;
        self.lex();
        text
    }

    pub fn lex_until_end_of_line(&mut self) -> String {
        self.pending.clear();
        self.state.position = self.token.start;
        let start = self.state.position;
        while !self.at_end() {
            let byte = self.current();
            if byte == b'\n' || byte == b'\r' {
                break;
            }
            self.state.position += 1;
        }
        let text = self
            .make(TokenKind::String, start, self.state.position)
            .text;
        self.lex();
        text
    }

    fn lex_float(&mut self, start: usize) -> Token {
        while self.current().is_ascii_digit() {
            self.state.position += 1;
        }
        if matches!(self.current(), b'-' | b'+') {
            let at = self.state.position;
            return self.error_token_from(start, at, "invalid sign in float literal");
        }
        if matches!(self.current(), b'e' | b'E') {
            self.state.position += 1;
            if matches!(self.current(), b'-' | b'+') {
                self.state.position += 1;
            }
            while self.current().is_ascii_digit() {
                self.state.position += 1;
            }
        }
        self.make(TokenKind::Real, start, self.state.position)
    }

    fn error_token_from(&mut self, start: usize, at: usize, message: &str) -> Token {
        let mut token = self.error_token(at, message);
        token.start = start.min(token.start);
        token
    }

    fn lex_hexadecimal_float(&mut self, start: usize, no_integer_digits: bool) -> Token {
        let mut no_fraction_digits = true;
        if self.current() == b'.' {
            self.state.position += 1;
            let fraction = self.state.position;
            while self.current().is_ascii_hexdigit() {
                self.state.position += 1;
            }
            no_fraction_digits = self.state.position == fraction;
        }
        if no_integer_digits && no_fraction_digits {
            return self.error_token(
                start,
                "invalid hexadecimal floating-point constant: expected at least one significand digit",
            );
        }
        if !matches!(self.current(), b'p' | b'P') {
            return self.error_token(
                start,
                "invalid hexadecimal floating-point constant: expected exponent part 'p'",
            );
        }
        self.state.position += 1;
        if matches!(self.current(), b'+' | b'-') {
            self.state.position += 1;
        }
        let exponent = self.state.position;
        while self.current().is_ascii_digit() {
            self.state.position += 1;
        }
        if self.state.position == exponent {
            return self.error_token(
                start,
                "invalid hexadecimal floating-point constant: expected at least one exponent digit",
            );
        }
        self.make(TokenKind::Real, start, self.state.position)
    }

    fn lex_identifier(&mut self, start: usize) -> Token {
        if self.byte(self.state.position - 1) == b'.' && self.current().is_ascii_digit() {
            while self.current().is_ascii_digit() {
                self.state.position += 1;
            }
            if !is_lexer_identifier_byte(self.current()) || matches!(self.current(), b'e' | b'E') {
                return self.lex_float(start);
            }
        }
        while is_lexer_identifier_byte(self.current()) {
            self.state.position += 1;
        }
        if self.state.position == start + 1 && self.byte(start) == b'.' {
            return self.make(TokenKind::Dot, start, start + 1);
        }
        self.make(TokenKind::Identifier, start, self.state.position)
    }

    fn lex_slash(&mut self, start: usize, old_start_of_statement: bool) -> Token {
        match self.current() {
            b'*' => {
                self.state.at_start_of_statement = false;
            }
            b'/' => {
                self.state.at_start_of_statement = old_start_of_statement;
                self.state.position += 1;
                return self.lex_line_comment(start);
            }
            _ => {
                self.state.at_start_of_statement = false;
                return self.make(TokenKind::Slash, start, start + 1);
            }
        }
        self.state.position += 1;
        while !self.at_end() {
            let byte = self.current();
            self.state.position += 1;
            if byte == b'*' && self.current() == b'/' {
                self.state.position += 1;
                return self.make(TokenKind::Comment, start, self.state.position);
            }
        }
        self.error_token(start, "unterminated comment")
    }

    fn lex_line_comment(&mut self, start: usize) -> Token {
        let mut character = self.next_character();
        while !matches!(character, Some(b'\n') | Some(b'\r') | None) {
            character = self.next_character();
        }
        if character == Some(b'\r') && !self.at_end() && self.current() == b'\n' {
            self.state.position += 1;
        }
        self.state.at_start_of_line = true;
        if self.state.at_start_of_statement {
            return self.make(TokenKind::EndOfStatement, start, self.state.position);
        }
        self.state.at_start_of_statement = true;
        let end = if character.is_some() {
            self.state.position - 1
        } else {
            self.state.position
        };
        self.make(TokenKind::EndOfStatement, start, end)
    }

    fn skip_integer_suffix(&mut self) {
        if matches!(self.current(), b'U' | b'u') {
            self.state.position += 1;
        }
        if matches!(self.current(), b'L' | b'l') {
            self.state.position += 1;
        }
        if matches!(self.current(), b'L' | b'l') {
            self.state.position += 1;
        }
    }

    fn integer_token(&self, start: usize, end: usize, value: u128) -> Token {
        let kind = if value >> 64 == 0 {
            TokenKind::Integer
        } else {
            TokenKind::BigNumber
        };
        self.make_value(kind, start, end, value)
    }

    fn lex_masm_digit(&mut self, start: usize) -> Option<Token> {
        let first = self.byte(self.state.position - 1);
        let mut first_non_binary = (!matches!(first, b'0' | b'1')).then_some(start);
        let mut first_non_decimal: Option<usize> = None;
        let resume = self.state.position;
        while self.current().is_ascii_hexdigit() {
            let byte = self.current();
            if !byte.is_ascii_digit() && first_non_decimal.is_none() {
                first_non_decimal = Some(self.state.position);
            }
            if !matches!(byte, b'0' | b'1') && first_non_binary.is_none() {
                first_non_binary = Some(self.state.position);
            }
            self.state.position += 1;
        }
        if self.current() == b'.' {
            self.state.position += 1;
            return Some(self.lex_float(start));
        }
        let last = self.state.position - 1;
        let radix = match self.current() {
            b'h' | b'H' => 16,
            b't' | b'T' => 10,
            b'o' | b'O' | b'q' | b'Q' => 8,
            b'y' | b'Y' => 2,
            _ if first_non_decimal == Some(last) && matches!(self.byte(last), b'd' | b'D') => 10,
            _ if first_non_binary == Some(last) && matches!(self.byte(last), b'b' | b'B') => 2,
            _ => 0,
        };
        if radix == 0 {
            self.state.position = resume;
            return None;
        }
        if matches!(
            self.current(),
            b'h' | b'H' | b't' | b'T' | b'o' | b'O' | b'q' | b'Q' | b'y' | b'Y'
        ) {
            self.state.position += 1;
        }
        let end = self.state.position;
        let text = self.make(TokenKind::Integer, start, end - 1).text;
        let Some(value) = parse_radix(&text, radix) else {
            return Some(self.error_token(start, &format!("invalid {} number", radix_name(radix))));
        };
        self.skip_integer_suffix();
        Some(self.integer_token(start, end, value))
    }

    fn lex_digit(&mut self, start: usize) -> Token {
        if self.masm_integers {
            if let Some(token) = self.lex_masm_digit(start) {
                return token;
            }
        }
        if self.byte(self.state.position - 1) != b'0' || self.current() == b'.' {
            while self.current().is_ascii_digit() {
                self.state.position += 1;
            }
            if matches!(self.current(), b'.' | b'e' | b'E') {
                if self.current() == b'.' {
                    self.state.position += 1;
                }
                return self.lex_float(start);
            }
            let end = self.state.position;
            let text = self.make(TokenKind::Integer, start, end).text;
            let Some(value) = parse_radix(&text, 10) else {
                return self.error_token(start, "invalid decimal number");
            };
            self.skip_integer_suffix();
            return self.integer_token(start, end, value);
        }
        if !self.masm_integers && matches!(self.current(), b'b' | b'B') {
            self.state.position += 1;
            if !self.current().is_ascii_digit() {
                self.state.position -= 1;
                return self.make_value(TokenKind::Integer, start, self.state.position, 0);
            }
            let digits = self.state.position;
            while matches!(self.current(), b'0' | b'1') {
                self.state.position += 1;
            }
            if self.state.position == digits {
                return self.error_token(start, "invalid binary number");
            }
            let end = self.state.position;
            let text = self.make(TokenKind::Integer, digits, end).text;
            let Some(value) = parse_radix(&text, 2) else {
                return self.error_token(start, "invalid binary number");
            };
            self.skip_integer_suffix();
            return self.integer_token(start, end, value);
        }
        if matches!(self.current(), b'x' | b'X') {
            self.state.position += 1;
            let digits = self.state.position;
            while self.current().is_ascii_hexdigit() {
                self.state.position += 1;
            }
            if matches!(self.current(), b'.' | b'p' | b'P') {
                let no_digits = digits == self.state.position;
                return self.lex_hexadecimal_float(start, no_digits);
            }
            if self.state.position == digits {
                let at = self.state.position - 2;
                return self.error_token(at, "invalid hexadecimal number");
            }
            let text = self
                .make(TokenKind::Integer, digits, self.state.position)
                .text;
            let Some(value) = parse_radix(&text, 16) else {
                return self.error_token(start, "invalid hexadecimal number");
            };
            if self.masm_integers && matches!(self.current(), b'h' | b'H') {
                self.state.position += 1;
            }
            let end = self.state.position;
            self.skip_integer_suffix();
            return self.integer_token(start, end, value);
        }
        let mut first_non_octal: Option<usize> = None;
        let mut look = self.state.position;
        while self.byte(look).is_ascii_digit() {
            look += 1;
        }
        let _ = &mut first_non_octal;
        self.state.position = look;
        let end = self.state.position;
        let text = self.make(TokenKind::Integer, start, end).text;
        let Some(value) = parse_radix(&text, 8) else {
            return self.error_token(start, &format!("invalid {} number", radix_name(8)));
        };
        self.skip_integer_suffix();
        self.integer_token(start, end, value)
    }

    fn lex_single_quote(&mut self, start: usize) -> Token {
        let mut character = self.next_character();
        if character == Some(b'\\') {
            character = self.next_character();
        }
        if character.is_none() {
            return self.error_token(start, "unterminated single quote");
        }
        let closing = self.next_character();
        if closing != Some(b'\'') {
            return self.error_token(start, "single quote way too long");
        }
        let text = &self.buffers[self.state.buffer].text;
        let value = if text.get(start + 1) == Some(&b'\\') {
            match text.get(start + 2).copied().unwrap_or(0) {
                b'\'' => b'\'',
                b't' => b'\t',
                b'n' => b'\n',
                b'b' => 8,
                b'f' => 12,
                b'r' => b'\r',
                other => other,
            }
        } else {
            text.get(start + 1).copied().unwrap_or(0)
        };
        self.make_value(
            TokenKind::Integer,
            start,
            self.state.position,
            u128::from(value as i8 as i64 as u64),
        )
    }

    fn lex_quote(&mut self, start: usize) -> Token {
        let mut character = self.next_character();
        while character != Some(b'"') {
            if character == Some(b'\\') {
                character = self.next_character();
            }
            if character.is_none() {
                return self.error_token(start, "unterminated string constant");
            }
            character = self.next_character();
        }
        self.make(TokenKind::String, start, self.state.position)
    }

    fn lex_token(&mut self) -> Token {
        let start = self.state.position;
        self.state.token_start = start;
        let character = self.next_character();
        if !self.peeking && character == Some(b'#') && self.state.at_start_of_statement {
            let saved_skip = self.skip_space;
            self.skip_space = true;
            let following = self.peek(2);
            self.skip_space = saved_skip;
            if self.state.at_start_of_line
                && following.len() == 2
                && following[0].kind == TokenKind::Integer
                && following[1].kind == TokenKind::String
            {
                self.state.position = start;
                let begin = start;
                while !self.at_end() && !matches!(self.current(), b'\n' | b'\r') {
                    self.state.position += 1;
                }
                let directive = self.make(TokenKind::HashDirective, begin, self.state.position);
                self.state.position = begin + 1;
                return directive;
            }
            return self.lex_line_comment(start);
        }
        if character == Some(b'#') {
            return self.lex_line_comment(start);
        }
        if character == Some(b';') {
            self.state.at_start_of_line = true;
            self.state.at_start_of_statement = true;
            return self.make(TokenKind::EndOfStatement, start, start + 1);
        }
        if character.is_none() && !self.state.at_start_of_statement && self.end_statement_at_end {
            self.state.at_start_of_line = true;
            self.state.at_start_of_statement = true;
            return self.make(TokenKind::EndOfStatement, start, start);
        }
        self.state.at_start_of_line = false;
        let old_start_of_statement = self.state.at_start_of_statement;
        self.state.at_start_of_statement = false;
        let Some(character) = character else {
            if self.end_statement_at_end {
                self.state.at_start_of_line = true;
                self.state.at_start_of_statement = true;
            }
            return self.make(TokenKind::EndOfFile, start, start);
        };
        let single = |lexer: &Lexer, kind: TokenKind| lexer.make(kind, start, start + 1);
        let double = |lexer: &mut Lexer, kind: TokenKind| {
            lexer.state.position += 1;
            lexer.make(kind, start, start + 2)
        };
        match character {
            0 | b' ' | b'\t' => {
                self.state.at_start_of_statement = old_start_of_statement;
                while matches!(self.current(), b' ' | b'\t') && !self.at_end() {
                    self.state.position += 1;
                }
                if self.skip_space {
                    self.lex_token()
                } else {
                    self.make(TokenKind::Space, start, self.state.position)
                }
            }
            b'\r' => {
                self.state.at_start_of_line = true;
                self.state.at_start_of_statement = true;
                if !self.at_end() && self.current() == b'\n' {
                    self.state.position += 1;
                }
                self.make(TokenKind::EndOfStatement, start, self.state.position)
            }
            b'\n' => {
                self.state.at_start_of_line = true;
                self.state.at_start_of_statement = true;
                self.make(TokenKind::EndOfStatement, start, start + 1)
            }
            b':' => single(self, TokenKind::Colon),
            b'+' => single(self, TokenKind::Plus),
            b'~' => single(self, TokenKind::Tilde),
            b'(' => single(self, TokenKind::LeftParenthesis),
            b')' => single(self, TokenKind::RightParenthesis),
            b'[' => single(self, TokenKind::LeftBracket),
            b']' => single(self, TokenKind::RightBracket),
            b'{' => single(self, TokenKind::LeftBrace),
            b'}' => single(self, TokenKind::RightBrace),
            b'*' => single(self, TokenKind::Star),
            b',' => single(self, TokenKind::Comma),
            b'$' => single(self, TokenKind::Dollar),
            b'@' => single(self, TokenKind::At),
            b'?' => single(self, TokenKind::Question),
            b'\\' => single(self, TokenKind::BackSlash),
            b'^' => single(self, TokenKind::Caret),
            b'=' => {
                if self.current() == b'=' {
                    double(self, TokenKind::EqualEqual)
                } else {
                    single(self, TokenKind::Equal)
                }
            }
            b'-' => {
                if self.current() == b'>' {
                    double(self, TokenKind::MinusGreater)
                } else {
                    single(self, TokenKind::Minus)
                }
            }
            b'|' => {
                if self.current() == b'|' {
                    double(self, TokenKind::PipePipe)
                } else {
                    single(self, TokenKind::Pipe)
                }
            }
            b'&' => {
                if self.current() == b'&' {
                    double(self, TokenKind::AmpersandAmpersand)
                } else {
                    single(self, TokenKind::Ampersand)
                }
            }
            b'!' => {
                if self.current() == b'=' {
                    double(self, TokenKind::ExclamationEqual)
                } else {
                    single(self, TokenKind::Exclamation)
                }
            }
            b'%' => single(self, TokenKind::Percent),
            b'/' => {
                self.state.at_start_of_statement = old_start_of_statement;
                self.lex_slash(start, old_start_of_statement)
            }
            b'\'' => self.lex_single_quote(start),
            b'"' => self.lex_quote(start),
            b'0'..=b'9' => self.lex_digit(start),
            b'<' => match self.current() {
                b'<' => double(self, TokenKind::LessLess),
                b'=' => double(self, TokenKind::LessEqual),
                b'>' => double(self, TokenKind::LessGreater),
                _ => single(self, TokenKind::Less),
            },
            b'>' => match self.current() {
                b'>' => double(self, TokenKind::GreaterGreater),
                b'=' => double(self, TokenKind::GreaterEqual),
                _ => single(self, TokenKind::Greater),
            },
            other => {
                if other.is_ascii_alphabetic() || other == b'_' || other == b'.' {
                    self.lex_identifier(start)
                } else {
                    self.error_token(start, "invalid character in input")
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectFormat {
    Elf,
    Coff,
    MachObject,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetTriple {
    pub text: String,
    pub architecture: String,
    pub vendor: String,
    pub system: String,
    pub environment: String,
    pub mode: Mode,
    pub format: ObjectFormat,
    pub x32: bool,
}

impl TargetTriple {
    pub fn parse(text: &str) -> TargetTriple {
        let pieces: Vec<&str> = text.split('-').collect();
        let architecture = pieces.first().copied().unwrap_or("").to_string();
        let vendor = pieces.get(1).copied().unwrap_or("").to_string();
        let system = pieces.get(2).copied().unwrap_or("").to_string();
        let environment = pieces
            .get(3..)
            .map(|rest| rest.join("-"))
            .unwrap_or_default();
        let sixty_four = matches!(architecture.as_str(), "x86_64" | "amd64" | "x86_64h")
            || architecture.starts_with("x86_64");
        let x32 = sixty_four && (environment.ends_with("x32") || environment == "gnux32");
        let mut mode = if sixty_four {
            Mode::Bits64
        } else {
            Mode::Bits32
        };
        if environment == "code16" {
            mode = Mode::Bits16;
        }
        let format = if vendor == "apple"
            || system.starts_with("darwin")
            || system.starts_with("macos")
            || system.starts_with("ios")
        {
            ObjectFormat::MachObject
        } else if system.starts_with("windows")
            || system.starts_with("win32")
            || system == "uefi"
            || system.starts_with("mingw")
            || system == "cygwin"
        {
            if environment.contains("elf") {
                ObjectFormat::Elf
            } else {
                ObjectFormat::Coff
            }
        } else {
            ObjectFormat::Elf
        };
        TargetTriple {
            text: text.to_string(),
            architecture,
            vendor,
            system,
            environment,
            mode,
            format,
            x32,
        }
    }

    pub fn is_64_bit(&self) -> bool {
        self.mode == Mode::Bits64 || self.architecture.starts_with("x86_64")
    }

    pub fn is_windows_msvc(&self) -> bool {
        self.format == ObjectFormat::Coff
            && (self.environment.is_empty() || self.environment == "msvc")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymbolBinding {
    Local,
    Global,
    Weak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymbolType {
    NoType,
    Object,
    Function,
    Section,
    File,
    Common,
    ThreadLocal,
    IndirectFunction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymbolVisibility {
    Default,
    Internal,
    Hidden,
    Protected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolAttribute {
    Global,
    Weak,
    Local,
    Hidden,
    Internal,
    Protected,
    TypeFunction,
    TypeIndirectFunction,
    TypeObject,
    ThreadLocal,
    TypeCommon,
    TypeNoType,
    TypeGnuUniqueObject,
    NoDeadStrip,
    Extern,
    WeakReference,
    LazyReference,
    PrivateExtern,
    Reference,
    WeakDefinition,
    WeakDefinitionAutoPrivate,
    SymbolResolver,
    Cold,
    Memtag,
    AlternateEntry,
    WeakAntiDependency,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub value: Option<Expression>,
    pub defined: bool,
    pub section: Option<usize>,
    pub fragment: Option<usize>,
    pub offset: u64,
    pub redefinable: bool,
    pub binding: Option<SymbolBinding>,
    pub symbol_type: Option<SymbolType>,
    pub visibility: SymbolVisibility,
    pub size: Option<Expression>,
    pub common: Option<(u64, u64)>,
    pub temporary: bool,
    pub used_in_relocation: bool,
    pub registered: bool,
    pub external: bool,
    pub weak_reference: bool,
    pub other: u8,
    pub definition: Option<SourceLocation>,
    pub version_name: Option<String>,
    pub coff_storage_class: Option<i64>,
    pub coff_type: Option<i64>,
    pub address_significant: bool,
}

impl Symbol {
    pub fn new(name: &str, temporary: bool) -> Symbol {
        Symbol {
            name: name.to_string(),
            value: None,
            defined: false,
            section: None,
            fragment: None,
            offset: 0,
            redefinable: false,
            binding: None,
            symbol_type: None,
            visibility: SymbolVisibility::Default,
            size: None,
            common: None,
            temporary,
            used_in_relocation: false,
            registered: false,
            external: false,
            weak_reference: false,
            other: 0,
            definition: None,
            version_name: None,
            coff_storage_class: None,
            coff_type: None,
            address_significant: false,
        }
    }

    pub fn is_variable(&self) -> bool {
        self.value.is_some()
    }

    pub fn is_undefined(&self) -> bool {
        !self.defined && self.value.is_none() && self.common.is_none()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub name: String,
    pub section_type: u32,
    pub flags: u64,
    pub entry_size: u64,
    pub group: Option<String>,
    pub comdat: bool,
    pub linked_symbol: Option<String>,
    pub unique: Option<u64>,
    pub characteristics: u32,
    pub coff_selection: u8,
    pub coff_associated: Option<String>,
    pub alignment: u64,
    pub has_instructions: bool,
    pub begin_symbol: Option<usize>,
    pub bundle_lock_depth: u32,
    pub bundle_align_to_end: bool,
    pub is_virtual: bool,
}

impl Section {
    pub fn new(name: &str, section_type: u32, flags: u64) -> Section {
        Section {
            name: name.to_string(),
            section_type,
            flags,
            entry_size: 0,
            group: None,
            comdat: false,
            linked_symbol: None,
            unique: None,
            characteristics: 0,
            coff_selection: 0,
            coff_associated: None,
            alignment: 1,
            has_instructions: false,
            begin_symbol: None,
            bundle_lock_depth: 0,
            bundle_align_to_end: false,
            is_virtual: section_type == ELF_SECTION_NO_BITS,
        }
    }
}

pub const ELF_SECTION_PROGRAM_BITS: u32 = 1;
pub const ELF_SECTION_SYMBOL_TABLE: u32 = 2;
pub const ELF_SECTION_STRING_TABLE: u32 = 3;
pub const ELF_SECTION_RELOCATION_ADDENDS: u32 = 4;
pub const ELF_SECTION_NOTE: u32 = 7;
pub const ELF_SECTION_NO_BITS: u32 = 8;
pub const ELF_SECTION_RELOCATION: u32 = 9;
pub const ELF_SECTION_INITIALIZER_ARRAY: u32 = 14;
pub const ELF_SECTION_FINALIZER_ARRAY: u32 = 15;
pub const ELF_SECTION_PREINITIALIZER_ARRAY: u32 = 16;
pub const ELF_SECTION_GROUP: u32 = 17;
pub const ELF_SECTION_X86_64_UNWIND: u32 = 0x7000_0001;
pub const ELF_SECTION_LLVM_ADDRESS_SIGNIFICANCE: u32 = 0x6fff_4c03;
pub const ELF_SECTION_LLVM_CALL_GRAPH_PROFILE: u32 = 0x6fff_4c09;

pub const ELF_FLAG_WRITE: u64 = 0x1;
pub const ELF_FLAG_ALLOCATE: u64 = 0x2;
pub const ELF_FLAG_EXECUTE: u64 = 0x4;
pub const ELF_FLAG_MERGE: u64 = 0x10;
pub const ELF_FLAG_STRINGS: u64 = 0x20;
pub const ELF_FLAG_INFORMATION_LINK: u64 = 0x40;
pub const ELF_FLAG_LINK_ORDER: u64 = 0x80;
pub const ELF_FLAG_GROUP: u64 = 0x200;
pub const ELF_FLAG_THREAD_LOCAL: u64 = 0x400;
pub const ELF_FLAG_COMPRESSED: u64 = 0x800;
pub const ELF_FLAG_GNU_RETAIN: u64 = 0x20_0000;
pub const ELF_FLAG_EXCLUDE: u64 = 0x8000_0000;
pub const ELF_FLAG_X86_64_LARGE: u64 = 0x1000_0000;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RelocatableValue {
    pub add: Option<usize>,
    pub subtract: Option<usize>,
    pub constant: i64,
    pub specifier: Option<String>,
}

impl RelocatableValue {
    pub fn constant(value: i64) -> RelocatableValue {
        RelocatableValue {
            constant: value,
            ..RelocatableValue::default()
        }
    }

    pub fn is_absolute(&self) -> bool {
        self.add.is_none() && self.subtract.is_none()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    Error,
    Warning,
    Note,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssemblyMessage {
    pub kind: MessageKind,
    pub location: Option<SourceLocation>,
    pub range: Option<(SourceLocation, SourceLocation)>,
    pub text: String,
    pub marker: Option<(SourceLocation, String, i64)>,
}

pub trait SymbolLayout {
    fn symbol_offset(&self, context: &AssemblyContext, symbol: usize) -> Option<(usize, i64)>;
    fn difference_resolved(
        &self,
        context: &AssemblyContext,
        left: usize,
        right: usize,
        in_set: bool,
    ) -> bool;
}

pub type SectionPosition = Option<(usize, i64)>;

pub type FileReader<'s> = &'s dyn Fn(&str) -> Option<Vec<u8>>;

pub struct AssemblyContext {
    pub triple: TargetTriple,
    pub symbols: Vec<Symbol>,
    pub symbol_index: BTreeMap<String, usize>,
    pub used_names: BTreeSet<String>,
    pub unique_counters: BTreeMap<String, u64>,
    pub directional_instances: BTreeMap<i64, u64>,
    pub directional_symbols: BTreeMap<(i64, u64), usize>,
    pub sections: Vec<Section>,
    pub section_index: BTreeMap<(String, String, u64), usize>,
    pub section_stack: Vec<(SectionPosition, SectionPosition)>,
    pub messages: Vec<AssemblyMessage>,
    pub had_error: bool,
    pub resolving: BTreeSet<usize>,
    pub line_marker: Option<(SourceLocation, String, i64)>,
}

impl AssemblyContext {
    pub fn new(triple: TargetTriple) -> AssemblyContext {
        AssemblyContext {
            triple,
            symbols: Vec::new(),
            symbol_index: BTreeMap::new(),
            used_names: BTreeSet::new(),
            unique_counters: BTreeMap::new(),
            directional_instances: BTreeMap::new(),
            directional_symbols: BTreeMap::new(),
            sections: Vec::new(),
            section_index: BTreeMap::new(),
            section_stack: vec![(None, None)],
            messages: Vec::new(),
            had_error: false,
            resolving: BTreeSet::new(),
            line_marker: None,
        }
    }

    pub fn report(&mut self, kind: MessageKind, location: Option<SourceLocation>, text: &str) {
        if kind == MessageKind::Error {
            self.had_error = true;
        }
        self.messages.push(AssemblyMessage {
            kind,
            location,
            range: None,
            text: text.to_string(),
            marker: self.line_marker.clone(),
        });
    }

    pub fn private_prefix(&self) -> &'static str {
        if (self.triple.format == ObjectFormat::Coff && !self.triple.is_64_bit())
            || self.triple.format == ObjectFormat::MachObject
        {
            "L"
        } else {
            ".L"
        }
    }

    pub fn lookup(&self, name: &str) -> Option<usize> {
        self.symbol_index.get(name).copied()
    }

    fn create_symbol(&mut self, name: &str, temporary: bool) -> usize {
        let index = self.symbols.len();
        self.symbols.push(Symbol::new(name, temporary));
        self.symbol_index.insert(name.to_string(), index);
        self.used_names.insert(name.to_string());
        index
    }

    pub fn symbol(&mut self, name: &str) -> usize {
        if let Some(index) = self.lookup(name) {
            return index;
        }
        let temporary = name.starts_with(self.private_prefix());
        if self.used_names.contains(name) && temporary {
            return self.renamable_symbol(name, false, true);
        }
        self.create_symbol(name, temporary)
    }

    pub fn parse_symbol(&mut self, name: &str) -> usize {
        if !name.contains('\\') {
            return self.symbol(name);
        }
        let characters: Vec<char> = name.chars().collect();
        let mut out = String::new();
        let mut at = 0;
        while at < characters.len() {
            let mut character = characters[at];
            if character == '\\'
                && at + 1 < characters.len()
                && matches!(characters[at + 1], '"' | '\\')
            {
                at += 1;
                character = characters[at];
            }
            out.push(character);
            at += 1;
        }
        self.symbol(&out)
    }

    pub fn renamable_symbol(
        &mut self,
        base: &str,
        always_add_suffix: bool,
        temporary: bool,
    ) -> usize {
        let mut name = base.to_string();
        let mut add = always_add_suffix;
        while add || self.used_names.contains(&name) {
            add = false;
            let counter = self.unique_counters.entry(base.to_string()).or_insert(0);
            name = format!("{base}{counter}");
            *counter += 1;
        }
        let index = self.symbols.len();
        self.symbols.push(Symbol::new(&name, temporary));
        self.used_names.insert(name.clone());
        self.symbol_index.entry(name).or_insert(index);
        index
    }

    pub fn temporary_symbol(&mut self) -> usize {
        let base = format!("{}tmp", self.private_prefix());
        self.renamable_symbol(&base, true, true)
    }

    pub fn clone_symbol(&mut self, index: usize) -> usize {
        let mut copy = self.symbols[index].clone();
        copy.value = None;
        copy.defined = false;
        copy.section = None;
        copy.fragment = None;
        let new_index = self.symbols.len();
        self.symbols.push(copy);
        let name = self.symbols[index].name.clone();
        self.symbol_index.insert(name, new_index);
        new_index
    }

    pub fn create_directional_symbol(&mut self, label: i64) -> usize {
        let instance = self.directional_instances.entry(label).or_insert(0);
        *instance += 1;
        let instance = *instance;
        self.directional_symbol(label, instance)
    }

    pub fn get_directional_symbol(&mut self, label: i64, before: bool) -> usize {
        let mut instance = *self.directional_instances.entry(label).or_insert(0);
        if !before {
            instance += 1;
        }
        self.directional_symbol(label, instance)
    }

    fn directional_symbol(&mut self, label: i64, instance: u64) -> usize {
        if let Some(found) = self.directional_symbols.get(&(label, instance)) {
            return *found;
        }
        let base = format!("{}tmp", self.private_prefix());
        let symbol = self.renamable_symbol(&base, true, true);
        self.directional_symbols.insert((label, instance), symbol);
        symbol
    }

    pub fn current_section(&self) -> Option<(usize, i64)> {
        self.section_stack.last().and_then(|entry| entry.0)
    }

    pub fn previous_section(&self) -> Option<(usize, i64)> {
        self.section_stack.last().and_then(|entry| entry.1)
    }

    pub fn elf_section(
        &mut self,
        name: &str,
        section_type: u32,
        flags: u64,
        entry_size: u64,
        group: &str,
        unique: Option<u64>,
    ) -> usize {
        let key = (
            name.to_string(),
            group.to_string(),
            unique.unwrap_or(u64::MAX),
        );
        if let Some(found) = self.section_index.get(&key) {
            return *found;
        }
        let mut section = Section::new(name, section_type, flags);
        section.entry_size = entry_size;
        if !group.is_empty() {
            section.group = Some(group.to_string());
        }
        section.unique = unique;
        let index = self.sections.len();
        self.sections.push(section);
        self.section_index.insert(key, index);
        index
    }

    pub fn find_section(&self, name: &str, group: &str, unique: Option<u64>) -> Option<usize> {
        self.section_index
            .get(&(
                name.to_string(),
                group.to_string(),
                unique.unwrap_or(u64::MAX),
            ))
            .copied()
    }

    pub fn is_in_section(&self, symbol: usize) -> bool {
        let held = &self.symbols[symbol];
        if held.defined {
            return held.section.is_some();
        }
        if let Some(value) = &held.value {
            return self.expression_in_section(value);
        }
        false
    }

    fn expression_in_section(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Symbol { name, .. } => self.lookup(name).is_some_and(|index| {
                let held = &self.symbols[index];
                held.defined && held.section.is_some()
            }),
            Expression::Unary { operand, .. } => self.expression_in_section(operand),
            Expression::Binary {
                operator,
                left,
                right,
            } => {
                let left_in = self.expression_in_section(left);
                let right_in = self.expression_in_section(right);
                if *operator == BinaryOperator::Subtract && left_in && right_in {
                    return false;
                }
                left_in || right_in
            }
            _ => false,
        }
    }

    pub fn evaluate_absolute(
        &mut self,
        expression: &Expression,
        layout: Option<&dyn SymbolLayout>,
    ) -> Option<i64> {
        self.evaluate_absolute_in_set(expression, layout, false)
    }

    pub fn evaluate_absolute_in_set(
        &mut self,
        expression: &Expression,
        layout: Option<&dyn SymbolLayout>,
        in_set: bool,
    ) -> Option<i64> {
        if let Expression::Constant(value) = expression {
            return Some(*value);
        }
        let value = self.evaluate_relocatable(expression, layout, in_set)?;
        if value.is_absolute() && value.specifier.is_none() {
            Some(value.constant)
        } else {
            None
        }
    }

    pub fn evaluate_relocatable(
        &mut self,
        expression: &Expression,
        layout: Option<&dyn SymbolLayout>,
        in_set: bool,
    ) -> Option<RelocatableValue> {
        match expression {
            Expression::Constant(value) => Some(RelocatableValue::constant(*value)),
            Expression::Register(_) => None,
            Expression::Symbol { name, specifier } => {
                let index = match self.lookup(name) {
                    Some(index) => index,
                    None => self.symbol(name),
                };
                if self.resolving.contains(&index) {
                    return None;
                }
                let symbol = &self.symbols[index];
                if let Some(value) = symbol.value.clone() {
                    if specifier.is_none() || layout.is_some() {
                        self.resolving.insert(index);
                        let inner = self.evaluate_relocatable(&value, layout, in_set);
                        self.resolving.remove(&index);
                        let inner = inner?;
                        let in_section = inner.add.is_some_and(|added| self.is_in_section(added));
                        if in_set
                            || !(inner.add.is_some() && inner.subtract.is_none() && in_section)
                        {
                            if let Some(specifier) = specifier {
                                if inner.is_absolute() {
                                    return Some(RelocatableValue {
                                        add: Some(index),
                                        subtract: None,
                                        constant: 0,
                                        specifier: Some(specifier.clone()),
                                    });
                                }
                                if inner.specifier.is_some()
                                    || inner.add.is_none()
                                    || inner.subtract.is_some()
                                    || inner.constant != 0
                                {
                                    return None;
                                }
                                let mut result = inner;
                                result.specifier = Some(specifier.clone());
                                return Some(result);
                            }
                            return Some(inner);
                        }
                    }
                }
                Some(RelocatableValue {
                    add: Some(index),
                    subtract: None,
                    constant: 0,
                    specifier: specifier.clone(),
                })
            }
            Expression::Unary { operator, operand } => {
                let value = self.evaluate_relocatable(operand, layout, in_set)?;
                match operator {
                    UnaryOperator::LogicalNot => {
                        if !value.is_absolute() {
                            return None;
                        }
                        Some(RelocatableValue::constant(i64::from(value.constant == 0)))
                    }
                    UnaryOperator::Negate => {
                        if value.add.is_some() && value.subtract.is_none() {
                            return None;
                        }
                        Some(RelocatableValue {
                            add: value.subtract,
                            subtract: value.add,
                            constant: (value.constant as u64).wrapping_neg() as i64,
                            specifier: None,
                        })
                    }
                    UnaryOperator::Not => {
                        if !value.is_absolute() {
                            return None;
                        }
                        Some(RelocatableValue::constant(!value.constant))
                    }
                    UnaryOperator::Plus => Some(value),
                }
            }
            Expression::Binary {
                operator,
                left,
                right,
            } => {
                let left_value = self.evaluate_relocatable(left, layout, in_set);
                let right_value = self.evaluate_relocatable(right, layout, in_set);
                let (Some(mut left_value), Some(mut right_value)) = (left_value, right_value)
                else {
                    if let (Expression::Register(a), Expression::Register(b)) =
                        (left.as_ref(), right.as_ref())
                    {
                        match operator {
                            BinaryOperator::Equal => {
                                return Some(RelocatableValue::constant(if a == b {
                                    -1
                                } else {
                                    0
                                }))
                            }
                            BinaryOperator::NotEqual => {
                                return Some(RelocatableValue::constant(if a == b {
                                    0
                                } else {
                                    -1
                                }))
                            }
                            _ => {}
                        }
                    }
                    return None;
                };
                let left_constant = left_value.constant;
                let right_constant = right_value.constant;
                if !left_value.is_absolute() || !right_value.is_absolute() {
                    match operator {
                        BinaryOperator::Add | BinaryOperator::Subtract => {
                            if *operator == BinaryOperator::Subtract {
                                std::mem::swap(&mut right_value.add, &mut right_value.subtract);
                                right_value.constant =
                                    (right_value.constant as u64).wrapping_neg() as i64;
                            }
                            if right_value.is_absolute() {
                                left_value.constant =
                                    left_value.constant.wrapping_add(right_value.constant);
                                return Some(left_value);
                            }
                            if left_value.is_absolute() {
                                right_value.constant =
                                    right_value.constant.wrapping_add(left_value.constant);
                                return Some(right_value);
                            }
                            if left_value.subtract.is_some() && left_value.specifier.is_some() {
                                return None;
                            }
                            if right_value.subtract.is_some() && right_value.specifier.is_some() {
                                return None;
                            }
                            return self.evaluate_symbolic_add(
                                layout,
                                in_set,
                                &left_value,
                                &right_value,
                            );
                        }
                        _ => return None,
                    }
                }
                let (a, b) = (left_constant, right_constant);
                let result = match operator {
                    BinaryOperator::ArithmeticShiftRight => a.wrapping_shr(b as u32),
                    BinaryOperator::Add => a.wrapping_add(b),
                    BinaryOperator::And => a & b,
                    BinaryOperator::Divide | BinaryOperator::Modulo => {
                        if b == 0 {
                            return None;
                        }
                        if *operator == BinaryOperator::Divide {
                            a.wrapping_div(b)
                        } else {
                            a.wrapping_rem(b)
                        }
                    }
                    BinaryOperator::Equal => i64::from(a == b),
                    BinaryOperator::Greater => i64::from(a > b),
                    BinaryOperator::GreaterOrEqual => i64::from(a >= b),
                    BinaryOperator::LogicalAnd => i64::from(a != 0 && b != 0),
                    BinaryOperator::LogicalOr => i64::from(a != 0 || b != 0),
                    BinaryOperator::LogicalShiftRight => ((a as u64).wrapping_shr(b as u32)) as i64,
                    BinaryOperator::Less => i64::from(a < b),
                    BinaryOperator::LessOrEqual => i64::from(a <= b),
                    BinaryOperator::Multiply => a.wrapping_mul(b),
                    BinaryOperator::NotEqual => i64::from(a != b),
                    BinaryOperator::Or => a | b,
                    BinaryOperator::OrNot => a | !b,
                    BinaryOperator::ShiftLeft => ((a as u64).wrapping_shl(b as u32)) as i64,
                    BinaryOperator::Subtract => a.wrapping_sub(b),
                    BinaryOperator::Xor => a ^ b,
                };
                let result = match operator {
                    BinaryOperator::Equal
                    | BinaryOperator::Greater
                    | BinaryOperator::GreaterOrEqual
                    | BinaryOperator::Less
                    | BinaryOperator::LessOrEqual
                    | BinaryOperator::NotEqual => {
                        if result != 0 {
                            -1
                        } else {
                            0
                        }
                    }
                    _ => result,
                };
                Some(RelocatableValue::constant(result))
            }
        }
    }

    fn evaluate_symbolic_add(
        &mut self,
        layout: Option<&dyn SymbolLayout>,
        in_set: bool,
        left: &RelocatableValue,
        right: &RelocatableValue,
    ) -> Option<RelocatableValue> {
        let mut left_add = left.add;
        let mut left_subtract = left.subtract;
        let mut right_add = right.add;
        let mut right_subtract = right.subtract;
        let mut constant = left.constant.wrapping_add(right.constant);
        if let Some(layout) = layout {
            if left.specifier.is_none() && right.specifier.is_none() {
                if left_add.is_some() && right_subtract.is_some() {
                    self.fold_difference(
                        layout,
                        in_set,
                        &mut left_add,
                        &mut right_subtract,
                        &mut constant,
                    );
                }
                if right_add.is_some() && left_subtract.is_some() {
                    self.fold_difference(
                        layout,
                        in_set,
                        &mut right_add,
                        &mut left_subtract,
                        &mut constant,
                    );
                }
            }
        }
        if (left_add.is_some() && right_add.is_some())
            || (left_subtract.is_some() && right_subtract.is_some())
        {
            return None;
        }
        Some(RelocatableValue {
            add: left_add.or(right_add),
            subtract: left_subtract.or(right_subtract),
            constant,
            specifier: left.specifier.clone().or_else(|| right.specifier.clone()),
        })
    }

    fn fold_difference(
        &mut self,
        layout: &dyn SymbolLayout,
        in_set: bool,
        add: &mut Option<usize>,
        subtract: &mut Option<usize>,
        constant: &mut i64,
    ) {
        let (Some(a), Some(b)) = (*add, *subtract) else {
            return;
        };
        if self.symbols[a].is_undefined() || self.symbols[b].is_undefined() {
            return;
        }
        if !layout.difference_resolved(self, a, b, in_set) {
            return;
        }
        let (Some((section_a, offset_a)), Some((section_b, offset_b))) =
            (layout.symbol_offset(self, a), layout.symbol_offset(self, b))
        else {
            return;
        };
        if section_a != section_b {
            return;
        }
        *constant = constant.wrapping_add(offset_a.wrapping_sub(offset_b));
        *add = None;
        *subtract = None;
    }
}

pub fn quoted_string(data: &[u8]) -> String {
    let mut out = String::from("\"");
    for &byte in data {
        if byte == b'"' || byte == b'\\' {
            out.push('\\');
            out.push(byte as char);
            continue;
        }
        if (0x20..0x7f).contains(&byte) {
            out.push(byte as char);
            continue;
        }
        match byte {
            8 => out.push_str("\\b"),
            12 => out.push_str("\\f"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            other => {
                out.push('\\');
                out.push((b'0' + ((other >> 6) & 7)) as char);
                out.push((b'0' + ((other >> 3) & 7)) as char);
                out.push((b'0' + (other & 7)) as char);
            }
        }
    }
    out.push('"');
    out
}

pub fn elf_section_name_text(name: &str) -> String {
    if name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '.')
    {
        return name.to_string();
    }
    let characters: Vec<char> = name.chars().collect();
    let mut out = String::from("\"");
    let mut at = 0;
    while at < characters.len() {
        let character = characters[at];
        if character == '"' {
            out.push_str("\\\"");
        } else if character != '\\' {
            out.push(character);
        } else if at + 1 == characters.len() {
            out.push_str("\\\\");
        } else {
            out.push(character);
            out.push(characters[at + 1]);
            at += 1;
        }
        at += 1;
    }
    out.push('"');
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputKind {
    Assembly,
    Object,
    Null,
}

#[derive(Clone, Debug)]
pub struct AssemblerOptions {
    pub triple: String,
    pub input_syntax: Syntax,
    pub output_syntax: Option<Syntax>,
    pub output: OutputKind,
    pub show_encoding: bool,
    pub show_instruction_operands: bool,
    pub hexadecimal_immediates: bool,
    pub relax_all: bool,
    pub relax_relocations: bool,
    pub align_branch_boundary: u64,
    pub align_branch: Vec<String>,
    pub pad_max_prefix_size: Option<u32>,
    pub branches_within_32_byte_boundaries: bool,
    pub definitions: Vec<(String, i64)>,
    pub cpu: String,
    pub features: Vec<String>,
    pub source_name: String,
    pub no_initial_text_section: bool,
    pub preserve_comments: bool,
    pub comment_column: usize,
    pub include_directories: Vec<String>,
    pub masm_integers: bool,
}

impl AssemblerOptions {
    pub fn new(triple: &str) -> AssemblerOptions {
        AssemblerOptions {
            triple: triple.to_string(),
            input_syntax: Syntax::Att,
            output_syntax: None,
            output: OutputKind::Assembly,
            show_encoding: false,
            show_instruction_operands: false,
            hexadecimal_immediates: false,
            relax_all: false,
            relax_relocations: true,
            align_branch_boundary: 0,
            align_branch: Vec::new(),
            pad_max_prefix_size: None,
            branches_within_32_byte_boundaries: false,
            definitions: Vec::new(),
            cpu: String::new(),
            features: Vec::new(),
            source_name: "<stdin>".to_string(),
            no_initial_text_section: false,
            preserve_comments: false,
            comment_column: 40,
            include_directories: Vec::new(),
            masm_integers: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConditionKind {
    None,
    If,
    ElseIf,
    Else,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConditionState {
    kind: ConditionKind,
    met: bool,
    ignore: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MacroParameter {
    name: String,
    value: Vec<Token>,
    required: bool,
    variadic: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MacroDefinition {
    name: String,
    body: String,
    parameters: Vec<MacroParameter>,
    count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MacroInstantiation {
    instantiation: SourceLocation,
    exit: SourceLocation,
    condition_depth: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AssignmentKind {
    Set,
    Equiv,
    Equal,
    ConditionalSet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CfiInstruction {
    Sections(bool, bool, bool),
    StartProcedure(bool),
    EndProcedure,
    DefineFrame(i64, i64),
    DefineFrameOffset(i64),
    AdjustFrameOffset(i64),
    DefineFrameRegister(i64),
    Offset(i64, i64),
    RelativeOffset(i64, i64),
    RememberState,
    RestoreState,
    SameValue(i64),
    Restore(i64),
    ReturnColumn(i64),
    SignalFrame,
    Undefined(i64),
    Register(i64, i64),
    WindowSave,
    ValueOffset(i64, i64),
    GnuArgumentsSize(i64),
}

pub trait Streamer {
    fn initialize_sections(&mut self, context: &mut AssemblyContext);
    fn switch_section(&mut self, context: &mut AssemblyContext, section: usize, subsection: i64);
    fn emit_label(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        location: Option<SourceLocation>,
    );
    fn emit_assignment(&mut self, context: &mut AssemblyContext, symbol: usize, value: &Expression);
    fn emit_conditional_assignment(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        value: &Expression,
    );
    fn emit_weak_reference(&mut self, context: &mut AssemblyContext, alias: usize, symbol: usize);
    fn emit_symbol_attribute(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        attribute: SymbolAttribute,
    ) -> bool;
    fn emit_symbol_version(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        name: &str,
        keep_original: bool,
    );
    fn emit_elf_size(&mut self, context: &mut AssemblyContext, symbol: usize, value: &Expression);
    fn emit_common_symbol(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        size: u64,
        alignment: u64,
        local: bool,
    );
    fn emit_bytes(&mut self, context: &mut AssemblyContext, data: &[u8]);
    fn emit_value(
        &mut self,
        context: &mut AssemblyContext,
        value: &Expression,
        size: u32,
        location: SourceLocation,
    );
    fn emit_leb128(&mut self, context: &mut AssemblyContext, value: &Expression, signed: bool);
    fn emit_fill_bytes(
        &mut self,
        context: &mut AssemblyContext,
        count: &Expression,
        value: u64,
        location: SourceLocation,
    );
    fn emit_fill(
        &mut self,
        context: &mut AssemblyContext,
        count: &Expression,
        size: i64,
        value: i64,
        location: SourceLocation,
    );
    fn emit_value_to_alignment(
        &mut self,
        context: &mut AssemblyContext,
        alignment: u64,
        value: i64,
        value_size: u32,
        maximum: u64,
    );
    fn emit_code_alignment(
        &mut self,
        context: &mut AssemblyContext,
        alignment: u64,
        maximum: u64,
        mode: Mode,
    );
    fn emit_value_to_offset(
        &mut self,
        context: &mut AssemblyContext,
        offset: &Expression,
        value: u8,
        location: SourceLocation,
    );
    fn emit_instruction(
        &mut self,
        context: &mut AssemblyContext,
        instruction: &MachineInstruction,
        mode: Mode,
        location: SourceLocation,
    );
    fn emit_nops(
        &mut self,
        context: &mut AssemblyContext,
        count: i64,
        control: i64,
        mode: Mode,
        location: SourceLocation,
    );
    fn emit_cfi(
        &mut self,
        context: &mut AssemblyContext,
        instruction: CfiInstruction,
        location: SourceLocation,
    );
    fn emit_cfi_escape(
        &mut self,
        context: &mut AssemblyContext,
        values: &[u8],
        location: SourceLocation,
    );
    fn emit_cfi_personality(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        encoding: i64,
        lsda: bool,
    );
    fn emit_cfi_label(
        &mut self,
        context: &mut AssemblyContext,
        name: &str,
        location: SourceLocation,
    );
    fn emit_bundle_align_mode(&mut self, context: &mut AssemblyContext, power: u32);
    fn emit_bundle_lock(
        &mut self,
        context: &mut AssemblyContext,
        align_to_end: bool,
        location: SourceLocation,
    );
    fn emit_bundle_unlock(&mut self, context: &mut AssemblyContext, location: SourceLocation);
    fn emit_relocation_directive(
        &mut self,
        context: &mut AssemblyContext,
        offset: &Expression,
        name: &str,
        value: Option<&Expression>,
        location: SourceLocation,
    );
    fn emit_identifier(&mut self, context: &mut AssemblyContext, text: &[u8]);
    fn emit_file(&mut self, context: &mut AssemblyContext, name: &[u8]);
    fn emit_raw_text(&mut self, context: &mut AssemblyContext, text: &str);
    fn emit_address_significance(&mut self, context: &mut AssemblyContext, symbol: Option<usize>);
    fn emit_mode_change(&mut self, context: &mut AssemblyContext, mode: Mode);
    fn emit_coff_directive(
        &mut self,
        context: &mut AssemblyContext,
        text: &str,
        symbol: Option<usize>,
        value: i64,
    );
    fn add_blank_line(&mut self);
    fn add_explicit_comment(&mut self, text: &str);
    fn finish(&mut self, context: &mut AssemblyContext);
    fn output(&mut self) -> Vec<u8>;
    fn current_fragment_valid(&self) -> bool {
        true
    }
}

pub struct AssemblyResult {
    pub output: Vec<u8>,
    pub messages: Vec<AssemblyMessage>,
    pub buffers: Vec<SourceBuffer>,
    pub failed: bool,
}

pub struct Assembler<'s> {
    pub target: &'static Target,
    pub matcher: &'static Matcher,
    pub options: AssemblerOptions,
    pub lexer: Lexer,
    pub context: AssemblyContext,
    pub streamer: Box<dyn Streamer + 's>,
    pub mode: Mode,
    pub dialect: Syntax,
    pending_errors: Vec<AssemblyMessage>,
    condition: ConditionState,
    conditions: Vec<ConditionState>,
    macros: BTreeMap<String, MacroDefinition>,
    macros_enabled: bool,
    alternate_macro_mode: bool,
    active_macros: Vec<MacroInstantiation>,
    macro_instantiations: u64,
    directional_labels: Vec<(SourceLocation, usize)>,
    code_16_gcc: bool,
    forced_data_prefix: Option<Mode>,
    statement_start: SourceLocation,
    cfi_start: Option<SourceLocation>,
    extended_register_seen: bool,
    pub file_reader: Option<FileReader<'s>>,
}

fn binary_precedence(kind: TokenKind) -> Option<(u32, BinaryOperator)> {
    Some(match kind {
        TokenKind::AmpersandAmpersand => (2, BinaryOperator::LogicalAnd),
        TokenKind::PipePipe => (1, BinaryOperator::LogicalOr),
        TokenKind::EqualEqual => (3, BinaryOperator::Equal),
        TokenKind::ExclamationEqual | TokenKind::LessGreater => (3, BinaryOperator::NotEqual),
        TokenKind::Less => (3, BinaryOperator::Less),
        TokenKind::LessEqual => (3, BinaryOperator::LessOrEqual),
        TokenKind::Greater => (3, BinaryOperator::Greater),
        TokenKind::GreaterEqual => (3, BinaryOperator::GreaterOrEqual),
        TokenKind::Plus => (4, BinaryOperator::Add),
        TokenKind::Minus => (4, BinaryOperator::Subtract),
        TokenKind::Pipe => (5, BinaryOperator::Or),
        TokenKind::Exclamation => (5, BinaryOperator::OrNot),
        TokenKind::Caret => (5, BinaryOperator::Xor),
        TokenKind::Ampersand => (5, BinaryOperator::And),
        TokenKind::Star => (6, BinaryOperator::Multiply),
        TokenKind::Slash => (6, BinaryOperator::Divide),
        TokenKind::Percent => (6, BinaryOperator::Modulo),
        TokenKind::LessLess => (6, BinaryOperator::ShiftLeft),
        TokenKind::GreaterGreater => (6, BinaryOperator::ArithmeticShiftRight),
        _ => return None,
    })
}

pub const SPECIFIER_NAMES: &[&str] = &[
    "ABS8",
    "DTPOFF",
    "DTPREL",
    "GOT",
    "GOTENT",
    "GOTNTPOFF",
    "GOTOFF",
    "GOTPCREL",
    "GOTPCREL_NORELAX",
    "GOTREL",
    "GOTTPOFF",
    "INDNTPOFF",
    "IMGREL",
    "NTPOFF",
    "PCREL",
    "PLT",
    "PLTOFF",
    "SECREL32",
    "SIZE",
    "tlscall",
    "tlsdesc",
    "TLSGD",
    "TLSLD",
    "TLSLDM",
    "TLVP",
    "TLVPPAGE",
    "TLVPPAGEOFF",
    "TPOFF",
];

pub fn specifier_for_name(name: &str) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    SPECIFIER_NAMES
        .iter()
        .find(|candidate| candidate.to_ascii_lowercase() == lower)
        .copied()
}

fn is_macro_argument_character(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.')
}

fn is_operator_token(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Tilde
            | TokenKind::Slash
            | TokenKind::Star
            | TokenKind::Dot
            | TokenKind::Equal
            | TokenKind::EqualEqual
            | TokenKind::Pipe
            | TokenKind::PipePipe
            | TokenKind::Caret
            | TokenKind::Ampersand
            | TokenKind::AmpersandAmpersand
            | TokenKind::Exclamation
            | TokenKind::ExclamationEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::LessLess
            | TokenKind::LessGreater
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::GreaterGreater
    )
}

fn has_section_prefix(name: &str, prefix: &str) -> bool {
    match name.strip_prefix(prefix) {
        Some(rest) => rest.is_empty() || rest.starts_with('.'),
        None => false,
    }
}

impl<'s> Assembler<'s> {
    pub fn new(
        options: AssemblerOptions,
        source: &[u8],
        streamer: Box<dyn Streamer + 's>,
    ) -> Result<Assembler<'s>, Diagnostic> {
        let target = target()?;
        let matcher = matcher(target);
        let triple = TargetTriple::parse(&options.triple);
        let mode = triple.mode;
        let dialect = options.input_syntax;
        let mut lexer = Lexer::new(&options.source_name, source);
        lexer.masm_integers = options.masm_integers;
        Ok(Assembler {
            target,
            matcher,
            options,
            lexer,
            context: AssemblyContext::new(triple),
            streamer,
            mode,
            dialect,
            pending_errors: Vec::new(),
            condition: ConditionState {
                kind: ConditionKind::None,
                met: false,
                ignore: false,
            },
            conditions: Vec::new(),
            macros: BTreeMap::new(),
            macros_enabled: true,
            alternate_macro_mode: false,
            active_macros: Vec::new(),
            macro_instantiations: 0,
            directional_labels: Vec::new(),
            code_16_gcc: false,
            forced_data_prefix: None,
            statement_start: SourceLocation::default(),
            cfi_start: None,
            extended_register_seen: false,
            file_reader: None,
        })
    }

    pub fn token(&self) -> &Token {
        &self.lexer.token
    }

    pub fn is(&self, kind: TokenKind) -> bool {
        self.lexer.token.kind == kind
    }

    pub fn lex(&mut self) -> Token {
        if self.lexer.token.kind == TokenKind::Error {
            if let Some((location, message)) = self.lexer.error.take() {
                self.error(location, &message);
            }
        }
        if self.lexer.token.kind == TokenKind::EndOfStatement && self.options.preserve_comments {
            let text = self.lexer.token.text.clone();
            if !text.is_empty() && !text.starts_with('\n') && !text.starts_with('\r') {
                self.streamer.add_explicit_comment(&text);
            }
        }
        let mut token = self.lexer.lex();
        while token.kind == TokenKind::Comment {
            if self.options.preserve_comments {
                self.streamer.add_explicit_comment(&token.text);
            }
            token = self.lexer.lex();
        }
        if token.kind == TokenKind::EndOfFile {
            if let Some(parent) = self.lexer.buffers[token.buffer].include_location {
                self.lexer.jump_to(parent);
                return self.lex();
            }
        }
        token
    }

    pub fn location(&self) -> SourceLocation {
        self.lexer.token.location()
    }

    fn message(
        &mut self,
        kind: MessageKind,
        location: SourceLocation,
        text: &str,
        range: Option<(SourceLocation, SourceLocation)>,
    ) {
        let message = AssemblyMessage {
            kind,
            location: Some(location),
            range,
            text: text.to_string(),
            marker: self.context.line_marker.clone(),
        };
        if kind == MessageKind::Error {
            self.context.had_error = true;
            self.pending_errors.push(message);
            self.push_macro_notes(true);
        } else {
            self.context.messages.push(message);
            self.push_macro_notes(false);
        }
    }

    fn push_macro_notes(&mut self, pending: bool) {
        for instantiation in self.active_macros.iter().rev() {
            let note = AssemblyMessage {
                kind: MessageKind::Note,
                location: Some(instantiation.instantiation),
                range: None,
                text: "while in macro instantiation".to_string(),
                marker: self.context.line_marker.clone(),
            };
            if pending {
                self.pending_errors.push(note);
            } else {
                self.context.messages.push(note);
            }
        }
    }

    pub fn error(&mut self, location: SourceLocation, text: &str) -> bool {
        self.message(MessageKind::Error, location, text, None);
        true
    }

    pub fn error_range(
        &mut self,
        location: SourceLocation,
        text: &str,
        range: (SourceLocation, SourceLocation),
    ) -> bool {
        self.message(MessageKind::Error, location, text, Some(range));
        true
    }

    pub fn warning(&mut self, location: SourceLocation, text: &str) -> bool {
        self.message(MessageKind::Warning, location, text, None);
        false
    }

    pub fn print_error(&mut self, location: SourceLocation, text: &str) {
        self.context.had_error = true;
        self.context.messages.push(AssemblyMessage {
            kind: MessageKind::Error,
            location: Some(location),
            range: None,
            text: text.to_string(),
            marker: self.context.line_marker.clone(),
        });
        self.push_macro_notes(false);
    }

    pub fn token_error(&mut self, text: &str) -> bool {
        let location = self.location();
        self.error(location, text)
    }

    fn has_pending_error(&self) -> bool {
        !self.pending_errors.is_empty()
    }

    fn print_pending_errors(&mut self) {
        let pending = std::mem::take(&mut self.pending_errors);
        self.context.messages.extend(pending);
    }

    fn check(&mut self, condition: bool, text: &str) -> bool {
        if condition {
            self.token_error(text)
        } else {
            false
        }
    }

    fn check_at(&mut self, condition: bool, location: SourceLocation, text: &str) -> bool {
        if condition {
            self.error(location, text)
        } else {
            false
        }
    }

    pub fn parse_optional_token(&mut self, kind: TokenKind) -> bool {
        if self.is(kind) {
            self.lex();
            true
        } else {
            false
        }
    }

    pub fn parse_token(&mut self, kind: TokenKind, text: &str) -> bool {
        if kind == TokenKind::EndOfStatement {
            return self.parse_end_of_line_with(text);
        }
        if !self.is(kind) {
            return self.token_error(text);
        }
        self.lex();
        false
    }

    pub fn parse_end_of_line(&mut self) -> bool {
        self.parse_end_of_line_with("expected newline")
    }

    fn parse_end_of_line_with(&mut self, text: &str) -> bool {
        if !self.is(TokenKind::EndOfStatement) {
            return self.token_error(text);
        }
        self.lex();
        false
    }

    pub fn parse_comma(&mut self) -> bool {
        self.parse_token(TokenKind::Comma, "expected comma")
    }

    fn eat_to_end_of_statement(&mut self) {
        while !self.is(TokenKind::EndOfStatement) && !self.is(TokenKind::EndOfFile) {
            self.lexer.lex();
        }
        if self.is(TokenKind::EndOfStatement) {
            self.lex();
        }
    }

    fn parse_string_to_end_of_statement(&mut self) -> String {
        let start = self.location();
        while !self.is(TokenKind::EndOfStatement) && !self.is(TokenKind::EndOfFile) {
            self.lexer.lex();
        }
        let end = self.location();
        self.lexer.text_between(start, end)
    }

    fn parse_string_to_comma(&mut self) -> String {
        let start = self.location();
        while !self.is(TokenKind::EndOfStatement)
            && !self.is(TokenKind::Comma)
            && !self.is(TokenKind::EndOfFile)
        {
            self.lexer.lex();
        }
        let end = self.location();
        self.lexer.text_between(start, end)
    }

    pub fn parse_identifier(&mut self) -> Option<String> {
        if self.is(TokenKind::Dollar) || self.is(TokenKind::At) {
            let prefix = self.lexer.token.clone();
            let saved_skip = self.lexer.skip_space;
            self.lexer.skip_space = false;
            let following = self.lexer.peek(1);
            self.lexer.skip_space = saved_skip;
            let next = following.first()?;
            if next.kind != TokenKind::Identifier && next.kind != TokenKind::Integer {
                return None;
            }
            if prefix.start + 1 != next.start || prefix.buffer != next.buffer {
                return None;
            }
            self.lexer.lex();
            let name = format!("{}{}", prefix.text, self.lexer.token.text);
            self.lex();
            return Some(name);
        }
        if !self.is(TokenKind::Identifier) && !self.is(TokenKind::String) {
            return None;
        }
        let name = self.lexer.token.identifier();
        self.lex();
        Some(name)
    }

    fn parse_symbol(&mut self) -> Option<usize> {
        let name = self.parse_identifier()?;
        Some(self.context.parse_symbol(&name))
    }

    fn parse_expression(&mut self) -> Result<(Expression, SourceLocation), ()> {
        let (mut expression, mut end) = self.parse_primary()?;
        self.parse_binary_right(1, &mut expression, &mut end)?;
        if self.is(TokenKind::At) {
            self.lex();
            if !self.is(TokenKind::Identifier) {
                self.token_error("unexpected symbol modifier following '@'");
                return Err(());
            }
            let name = self.lexer.token.text.clone();
            let Some(specifier) = specifier_for_name(&name) else {
                self.token_error(&format!("invalid variant '{name}'"));
                return Err(());
            };
            match self.apply_specifier(&expression, specifier) {
                Some(modified) => expression = modified,
                None => {
                    self.token_error(&format!("invalid modifier '{name}' (no symbols present)"));
                    return Err(());
                }
            }
            end = self.lexer.token.end_location();
            self.lex();
        }
        if !matches!(expression, Expression::Constant(_)) {
            if let Some(value) = self.context.evaluate_absolute(&expression, None) {
                expression = Expression::Constant(value);
            }
        }
        Ok((expression, end))
    }

    fn apply_specifier(&mut self, expression: &Expression, specifier: &str) -> Option<Expression> {
        match expression {
            Expression::Constant(_) | Expression::Register(_) => None,
            Expression::Symbol {
                name,
                specifier: existing,
            } => {
                if existing.is_some() {
                    let current = self.lexer.token.identifier();
                    self.token_error(&format!(
                        "invalid variant on expression '{current}' (already modified)"
                    ));
                    return Some(expression.clone());
                }
                Some(Expression::Symbol {
                    name: name.clone(),
                    specifier: Some(specifier.to_string()),
                })
            }
            Expression::Unary { operator, operand } => {
                let inner = self.apply_specifier(operand, specifier)?;
                Some(Expression::Unary {
                    operator: *operator,
                    operand: Box::new(inner),
                })
            }
            Expression::Binary {
                operator,
                left,
                right,
            } => {
                let new_left = self.apply_specifier(left, specifier);
                let new_right = self.apply_specifier(right, specifier);
                if new_left.is_none() && new_right.is_none() {
                    return None;
                }
                Some(Expression::Binary {
                    operator: *operator,
                    left: Box::new(new_left.unwrap_or_else(|| left.as_ref().clone())),
                    right: Box::new(new_right.unwrap_or_else(|| right.as_ref().clone())),
                })
            }
        }
    }

    fn parse_binary_right(
        &mut self,
        precedence: u32,
        expression: &mut Expression,
        end: &mut SourceLocation,
    ) -> Result<(), ()> {
        loop {
            let Some((token_precedence, operator)) = binary_precedence(self.lexer.token.kind)
            else {
                return Ok(());
            };
            if token_precedence < precedence {
                return Ok(());
            }
            self.lex();
            let (mut right, right_end) = self.parse_primary()?;
            *end = right_end;
            let next = binary_precedence(self.lexer.token.kind)
                .map(|(value, _)| value)
                .unwrap_or(0);
            if token_precedence < next {
                self.parse_binary_right(token_precedence + 1, &mut right, end)?;
            }
            let left = std::mem::replace(expression, Expression::Constant(0));
            *expression = Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    fn parse_primary(&mut self) -> Result<(Expression, SourceLocation), ()> {
        if self.is(TokenKind::Percent)
            || (self.dialect == Syntax::Intel
                && self.is(TokenKind::Identifier)
                && self
                    .target
                    .register_by_name
                    .contains_key(self.lexer.token.text.as_str()))
        {
            let start = self.location();
            let Some((register, end)) = self.parse_register_operand(start) else {
                return Err(());
            };
            return Ok((Expression::Register(register), end));
        }
        self.parse_generic_primary()
    }

    fn parse_generic_primary(&mut self) -> Result<(Expression, SourceLocation), ()> {
        let first_location = self.location();
        let first_kind = self.lexer.token.kind;
        match first_kind {
            TokenKind::Error => Err(()),
            TokenKind::Exclamation => {
                self.lex();
                let (operand, end) = self.parse_primary()?;
                Ok((
                    Expression::Unary {
                        operator: UnaryOperator::LogicalNot,
                        operand: Box::new(operand),
                    },
                    end,
                ))
            }
            TokenKind::Dollar
            | TokenKind::Star
            | TokenKind::At
            | TokenKind::String
            | TokenKind::Identifier => {
                let identifier_token = self.lexer.token.clone();
                let Some(identifier) = self.parse_identifier() else {
                    if self.is(TokenKind::Dollar) || self.is(TokenKind::Star) {
                        self.error(first_location, "invalid token in expression");
                        return Err(());
                    }
                    self.token_error("unknown token in expression");
                    return Err(());
                };
                let mut split: Option<(String, String)> = None;
                if first_kind == TokenKind::String {
                    if self.is(TokenKind::At) {
                        self.lex();
                        let at = self.location();
                        let Some(variant) = self.parse_identifier() else {
                            self.error(at, "expected symbol variant after '@'");
                            return Err(());
                        };
                        split = Some((identifier.clone(), variant));
                    }
                } else if let Some((before, after)) = identifier.split_once('@') {
                    split = Some((before.to_string(), after.to_string()));
                }
                let end = if first_kind == TokenKind::Identifier {
                    identifier_token.end_location()
                } else {
                    self.lexer.token.location()
                };
                let mut symbol_name = identifier.clone();
                if symbol_name.is_empty() {
                    let location = self.location();
                    self.error(location, "expected a symbol reference");
                    return Err(());
                }
                let mut specifier: Option<String> = None;
                if let Some((before, after)) = split {
                    if !after.is_empty() {
                        match specifier_for_name(&after) {
                            Some(found) => {
                                symbol_name = before;
                                specifier = Some(found.to_string());
                            }
                            None => {
                                if self.context.triple.format != ObjectFormat::Coff {
                                    let offset = identifier.find('@').map(|at| at + 1).unwrap_or(0);
                                    let location = SourceLocation {
                                        buffer: identifier_token.buffer,
                                        position: identifier_token.start + offset,
                                    };
                                    self.error(location, &format!("invalid variant '{after}'"));
                                    return Err(());
                                }
                            }
                        }
                    }
                }
                let symbol = self.context.parse_symbol(&symbol_name);
                let resolved_name = self.context.symbols[symbol].name.clone();
                if let Some(value) = self.context.symbols[symbol].value.clone() {
                    let inline = matches!(value, Expression::Constant(_) | Expression::Register(_));
                    if inline && specifier.is_none() {
                        return Ok((value, end));
                    }
                    if matches!(value, Expression::Register(_)) && specifier.is_some() {
                        self.error(end, "unexpected modifier on variable reference");
                        return Err(());
                    }
                }
                Ok((
                    Expression::Symbol {
                        name: resolved_name,
                        specifier,
                    },
                    end,
                ))
            }
            TokenKind::BigNumber => {
                self.token_error("literal value out of range for directive");
                Err(())
            }
            TokenKind::Integer => {
                let location = self.location();
                let value = self.lexer.token.integer();
                let mut expression = Expression::Constant(value);
                let mut end = self.lexer.token.end_location();
                self.lex();
                if self.is(TokenKind::Identifier) {
                    let text = self.lexer.token.text.clone();
                    let (mut base, mut specifier) = (text.clone(), None);
                    if let Some((before, after)) = text.split_once('@') {
                        match specifier_for_name(after) {
                            Some(found) => {
                                base = before.to_string();
                                specifier = Some(found.to_string());
                            }
                            None => {
                                self.token_error(&format!("invalid variant '{after}'"));
                                return Err(());
                            }
                        }
                    }
                    if base == "f" || base == "b" {
                        let symbol = self.context.get_directional_symbol(value, base == "b");
                        let name = self.context.symbols[symbol].name.clone();
                        expression = Expression::Symbol { name, specifier };
                        if base == "b" && self.context.symbols[symbol].is_undefined() {
                            self.error(location, "directional label undefined");
                            return Err(());
                        }
                        self.directional_labels.push((location, symbol));
                        end = self.lexer.token.end_location();
                        self.lex();
                    }
                }
                Ok((expression, end))
            }
            TokenKind::Real => {
                let text = self.lexer.token.text.clone();
                let value = text
                    .parse::<f64>()
                    .map(|parsed| parsed.to_bits() as i64)
                    .unwrap_or(0);
                let end = self.lexer.token.end_location();
                self.lex();
                Ok((Expression::Constant(value), end))
            }
            TokenKind::Dot => {
                let symbol = self.context.temporary_symbol();
                let location = self.location();
                self.streamer
                    .emit_label(&mut self.context, symbol, Some(location));
                let name = self.context.symbols[symbol].name.clone();
                let end = self.lexer.token.end_location();
                self.lex();
                Ok((Expression::symbol(&name), end))
            }
            TokenKind::LeftParenthesis => {
                self.lex();
                let (expression, _) = self.parse_expression()?;
                let end = self.lexer.token.end_location();
                if self.parse_token(TokenKind::RightParenthesis, "expected ')'") {
                    return Err(());
                }
                Ok((expression, end))
            }
            TokenKind::LeftBracket => {
                if self.context.triple.format != ObjectFormat::Elf {
                    self.token_error("brackets expression not supported on this target");
                    return Err(());
                }
                self.lex();
                let (expression, _) = self.parse_expression()?;
                let end = self.lexer.token.end_location();
                if self.parse_token(
                    TokenKind::RightBracket,
                    "expected ']' in brackets expression",
                ) {
                    return Err(());
                }
                Ok((expression, end))
            }
            TokenKind::Minus | TokenKind::Plus | TokenKind::Tilde => {
                self.lex();
                let (operand, end) = self.parse_primary()?;
                let operator = match first_kind {
                    TokenKind::Minus => UnaryOperator::Negate,
                    TokenKind::Plus => UnaryOperator::Plus,
                    _ => UnaryOperator::Not,
                };
                Ok((
                    Expression::Unary {
                        operator,
                        operand: Box::new(operand),
                    },
                    end,
                ))
            }
            _ => {
                self.token_error("unknown token in expression");
                Err(())
            }
        }
    }

    fn parse_absolute_expression(&mut self) -> Result<i64, ()> {
        let start = self.location();
        let (expression, _) = self.parse_expression()?;
        match self.context.evaluate_absolute(&expression, None) {
            Some(value) => Ok(value),
            None => {
                self.error(start, "expected absolute expression");
                Err(())
            }
        }
    }

    fn parse_many(&mut self, mut operation: impl FnMut(&mut Self) -> bool) -> bool {
        if self.parse_optional_token(TokenKind::EndOfStatement) {
            return false;
        }
        loop {
            if operation(self) {
                return true;
            }
            if self.parse_optional_token(TokenKind::EndOfStatement) {
                return false;
            }
            if self.parse_comma() {
                return true;
            }
        }
    }

    fn check_for_valid_section(&mut self) -> bool {
        if self.context.current_section().is_none() {
            self.streamer.initialize_sections(&mut self.context);
            return self.token_error("expected section directive before assembly directive");
        }
        false
    }

    pub fn run(&mut self) -> bool {
        if !self.options.no_initial_text_section {
            self.streamer.initialize_sections(&mut self.context);
        }
        let definitions = self.options.definitions.clone();
        for (name, value) in definitions {
            let symbol = self.context.symbol(&name);
            self.context.symbols[symbol].value = Some(Expression::Constant(value));
            self.context.symbols[symbol].redefinable = true;
        }
        self.lex();
        let starting = self.condition;
        while !self.is(TokenKind::EndOfFile) {
            let failed = self.parse_statement();
            if failed && !self.has_pending_error() && self.is(TokenKind::Error) {
                self.lex();
            }
            self.print_pending_errors();
            if failed && !self.just_consumed_end_of_line() {
                self.eat_to_end_of_statement();
            }
        }
        self.print_pending_errors();
        if self.condition != starting {
            let location = self.location();
            self.print_error(location, "unmatched .ifs or .elses");
        }
        let labels = self.directional_labels.clone();
        for (location, symbol) in labels {
            if self.context.symbols[symbol].is_undefined() {
                self.print_error(location, "directional label undefined");
            }
        }
        if !self.context.had_error {
            self.streamer.finish(&mut self.context);
        }
        self.context.had_error
    }

    fn just_consumed_end_of_line(&self) -> bool {
        self.lexer.consumed_end_of_statement
    }

    fn parse_statement(&mut self) -> bool {
        while self.is(TokenKind::Space) {
            self.lex();
        }
        if self.is(TokenKind::EndOfStatement) {
            let text = &self.lexer.token.text;
            if text.is_empty() || text.starts_with('\r') || text.starts_with('\n') {
                self.streamer.add_blank_line();
            }
            self.lex();
            return false;
        }
        let identifier_token = self.lexer.token.clone();
        let identifier_location = identifier_token.location();
        self.statement_start = identifier_location;
        let mut local_label: i64 = -1;
        let identifier: String;
        if self.is(TokenKind::HashDirective) {
            return self.parse_line_marker();
        }
        if self.is(TokenKind::Integer) {
            local_label = self.lexer.token.integer();
            if local_label < 0 {
                if !self.condition.ignore {
                    self.lex();
                    return self.error(
                        identifier_location,
                        "unexpected token at start of statement",
                    );
                }
                identifier = String::new();
            } else {
                identifier = self.lexer.token.text.clone();
                self.lex();
                if !self.is(TokenKind::Colon) && !self.condition.ignore {
                    self.lex();
                    return self.error(
                        identifier_location,
                        "unexpected token at start of statement",
                    );
                }
            }
        } else if self.is(TokenKind::Dot) {
            self.lex();
            identifier = ".".to_string();
        } else if self.is(TokenKind::LeftBrace) {
            self.lex();
            identifier = "{".to_string();
        } else {
            match self.parse_identifier() {
                Some(name) => identifier = name,
                None => {
                    if !self.condition.ignore {
                        self.lex();
                        return self.error(
                            identifier_location,
                            "unexpected token at start of statement",
                        );
                    }
                    identifier = String::new();
                }
            }
        }
        let lower = identifier.to_ascii_lowercase();
        match lower.as_str() {
            ".if" | ".ifne" | ".ifeq" | ".ifge" | ".ifgt" | ".ifle" | ".iflt" => {
                return self.parse_directive_if(&lower)
            }
            ".ifb" => return self.parse_directive_if_blank(true),
            ".ifnb" => return self.parse_directive_if_blank(false),
            ".ifc" => return self.parse_directive_if_compare(true),
            ".ifnc" => return self.parse_directive_if_compare(false),
            ".ifeqs" => return self.parse_directive_if_equal_strings(true),
            ".ifnes" => return self.parse_directive_if_equal_strings(false),
            ".ifdef" => return self.parse_directive_if_defined(true),
            ".ifndef" | ".ifnotdef" => return self.parse_directive_if_defined(false),
            ".elseif" => return self.parse_directive_else_if(identifier_location),
            ".else" => return self.parse_directive_else(identifier_location),
            ".endif" => return self.parse_directive_end_if(identifier_location),
            _ => {}
        }
        if self.condition.ignore {
            self.eat_to_end_of_statement();
            return false;
        }
        if self.is(TokenKind::Colon) && identifier != "{" {
            if self.check_for_valid_section() {
                return true;
            }
            self.lex();
            if identifier == "." {
                return self.error(
                    identifier_location,
                    "invalid use of pseudo-symbol '.' as a label",
                );
            }
            let symbol = if local_label == -1 {
                self.context.parse_symbol(&identifier)
            } else {
                self.context.create_directional_symbol(local_label)
            };
            if self.is(TokenKind::EndOfStatement) {
                self.lex();
            }
            return self.emit_label_checked(symbol, identifier_location);
        }
        if self.is(TokenKind::Equal) && identifier != "{" {
            self.lex();
            return self.parse_assignment(&identifier, AssignmentKind::Equal);
        }
        if self.macros_enabled {
            if let Some(definition) = self.macros.get(&identifier).cloned() {
                return self.handle_macro_entry(definition, identifier_location);
            }
        }
        if identifier.starts_with('.') && identifier != "." {
            return self.parse_directive(&identifier, &lower, identifier_location);
        }
        if self.check_for_valid_section() {
            return true;
        }
        self.parse_and_match_instruction(&identifier, identifier_token)
    }

    fn emit_label_checked(&mut self, symbol: usize, location: SourceLocation) -> bool {
        let held = &self.context.symbols[symbol];
        if held.redefinable && held.value.is_some() {
            let copy = self.context.clone_symbol(symbol);
            return self.emit_label_checked(copy, location);
        }
        let held = &self.context.symbols[symbol];
        if !held.is_undefined() || held.is_variable() {
            let name = held.name.clone();
            self.print_error(location, &format!("symbol '{name}' is already defined"));
            return false;
        }
        self.streamer
            .emit_label(&mut self.context, symbol, Some(location));
        false
    }

    fn parse_line_marker(&mut self) -> bool {
        let location = self.location();
        self.lex();
        let line = self.lexer.token.integer();
        self.lex();
        let name = string_contents(&self.lexer.token.text);
        self.lex();
        if self.active_macros.is_empty() {
            self.context.line_marker = Some((location, name, line));
        }
        false
    }

    fn parse_assignment(&mut self, name: &str, kind: AssignmentKind) -> bool {
        let location = self.location();
        let allow_redefinition = matches!(kind, AssignmentKind::Set | AssignmentKind::Equal);
        let Ok((value, _)) = self.parse_expression() else {
            return self.token_error("missing expression");
        };
        if self.parse_end_of_line() {
            return true;
        }
        if let Expression::Symbol {
            specifier: Some(_), ..
        } = &value
        {
            return self.error(
                location,
                "relocation specifier not permitted in symbol equating",
            );
        }
        let symbol = match self.context.lookup(name) {
            Some(found) => {
                let held = &self.context.symbols[found];
                if (held.is_variable() || held.defined)
                    && (!allow_redefinition || !held.redefinable)
                {
                    return self.error(location, &format!("redefinition of '{name}'"));
                }
                if held.redefinable {
                    self.context.clone_symbol(found)
                } else {
                    found
                }
            }
            None => {
                if name == "." {
                    self.streamer
                        .emit_value_to_offset(&mut self.context, &value, 0, location);
                    return false;
                }
                self.context.parse_symbol(name)
            }
        };
        self.context.symbols[symbol].redefinable = allow_redefinition;
        match kind {
            AssignmentKind::Equal | AssignmentKind::Set | AssignmentKind::Equiv => {
                self.streamer
                    .emit_assignment(&mut self.context, symbol, &value);
                if kind != AssignmentKind::Equal {
                    self.streamer.emit_symbol_attribute(
                        &mut self.context,
                        symbol,
                        SymbolAttribute::NoDeadStrip,
                    );
                }
            }
            AssignmentKind::ConditionalSet => {
                if !matches!(value, Expression::Symbol { .. }) {
                    return self.error(location, "expected identifier");
                }
                self.streamer
                    .emit_conditional_assignment(&mut self.context, symbol, &value);
            }
        }
        false
    }

    fn parse_escaped_string(&mut self) -> Result<Vec<u8>, ()> {
        if self.check(!self.is(TokenKind::String), "expected string") {
            return Err(());
        }
        let token = self.lexer.token.clone();
        let contents = string_contents(&token.text);
        let bytes = self.lexer.buffers[token.buffer].text
            [token.start + 1..token.end.saturating_sub(1).max(token.start + 1)]
            .to_vec();
        let _ = contents;
        let mut data: Vec<u8> = Vec::new();
        let mut at = 0;
        while at < bytes.len() {
            let byte = bytes[at];
            if byte != b'\\' {
                if byte == b'\n' || byte == b'\r' {
                    if byte == b'\n' && at > 0 && bytes[at - 1] == b'\r' {
                        at += 1;
                        continue;
                    }
                    let location = SourceLocation {
                        buffer: token.buffer,
                        position: token.start + 1 + at,
                    };
                    self.warning(location, "unterminated string; newline inserted");
                }
                data.push(byte);
                at += 1;
                continue;
            }
            at += 1;
            if at == bytes.len() {
                self.token_error("unexpected backslash at end of string");
                return Err(());
            }
            let escape = bytes[at];
            if escape == b'x' || escape == b'X' {
                if at + 1 >= bytes.len() || !bytes[at + 1].is_ascii_hexdigit() {
                    self.token_error("invalid hexadecimal escape sequence");
                    return Err(());
                }
                let mut value: u32 = 0;
                while at + 1 < bytes.len() && bytes[at + 1].is_ascii_hexdigit() {
                    at += 1;
                    value = value
                        .wrapping_mul(16)
                        .wrapping_add(hexadecimal_value(bytes[at]));
                }
                data.push((value & 0xff) as u8);
                at += 1;
                continue;
            }
            if (b'0'..=b'7').contains(&escape) {
                let mut value = u32::from(escape - b'0');
                if at + 1 < bytes.len() && (b'0'..=b'7').contains(&bytes[at + 1]) {
                    at += 1;
                    value = value * 8 + u32::from(bytes[at] - b'0');
                    if at + 1 < bytes.len() && (b'0'..=b'7').contains(&bytes[at + 1]) {
                        at += 1;
                        value = value * 8 + u32::from(bytes[at] - b'0');
                    }
                }
                if value > 255 {
                    self.token_error("invalid octal escape sequence (out of range)");
                    return Err(());
                }
                data.push(value as u8);
                at += 1;
                continue;
            }
            let translated = match escape {
                b'b' => 8,
                b'f' => 12,
                b'n' => b'\n',
                b'r' => b'\r',
                b't' => b'\t',
                b'"' => b'"',
                b'\\' => b'\\',
                _ => {
                    self.token_error("invalid escape sequence (unrecognized character)");
                    return Err(());
                }
            };
            data.push(translated);
            at += 1;
        }
        self.lex();
        Ok(data)
    }

    fn parse_directive_if(&mut self, kind: &str) -> bool {
        self.conditions.push(self.condition);
        self.condition.kind = ConditionKind::If;
        if self.condition.ignore {
            self.eat_to_end_of_statement();
            return false;
        }
        let Ok(mut value) = self.parse_absolute_expression() else {
            return true;
        };
        if self.parse_end_of_line() {
            return true;
        }
        value = match kind {
            ".ifeq" => i64::from(value == 0),
            ".ifge" => i64::from(value >= 0),
            ".ifgt" => i64::from(value > 0),
            ".ifle" => i64::from(value <= 0),
            ".iflt" => i64::from(value < 0),
            _ => value,
        };
        self.condition.met = value != 0;
        self.condition.ignore = !self.condition.met;
        false
    }

    fn parse_directive_if_blank(&mut self, expect_blank: bool) -> bool {
        self.conditions.push(self.condition);
        self.condition.kind = ConditionKind::If;
        if self.condition.ignore {
            self.eat_to_end_of_statement();
            return false;
        }
        let text = self.parse_string_to_end_of_statement();
        if self.parse_end_of_line() {
            return true;
        }
        self.condition.met = expect_blank == text.is_empty();
        self.condition.ignore = !self.condition.met;
        false
    }

    fn parse_directive_if_compare(&mut self, expect_equal: bool) -> bool {
        self.conditions.push(self.condition);
        self.condition.kind = ConditionKind::If;
        if self.condition.ignore {
            self.eat_to_end_of_statement();
            return false;
        }
        let first = self.parse_string_to_comma();
        if self.parse_comma() {
            return true;
        }
        let second = self.parse_string_to_end_of_statement();
        if self.parse_end_of_line() {
            return true;
        }
        self.condition.met = expect_equal == (first.trim() == second.trim());
        self.condition.ignore = !self.condition.met;
        false
    }

    fn parse_directive_if_equal_strings(&mut self, expect_equal: bool) -> bool {
        self.conditions.push(self.condition);
        self.condition.kind = ConditionKind::If;
        if self.condition.ignore {
            self.eat_to_end_of_statement();
            return false;
        }
        let name = if expect_equal { ".ifeqs" } else { ".ifnes" };
        if !self.is(TokenKind::String) {
            return self.token_error(&format!("expected string parameter for '{name}' directive"));
        }
        let first = string_contents(&self.lexer.token.text);
        self.lex();
        if !self.is(TokenKind::Comma) {
            return self.token_error(&format!(
                "expected comma after first string for '{name}' directive"
            ));
        }
        self.lex();
        if !self.is(TokenKind::String) {
            return self.token_error(&format!("expected string parameter for '{name}' directive"));
        }
        let second = string_contents(&self.lexer.token.text);
        self.lex();
        self.condition.met = expect_equal == (first == second);
        self.condition.ignore = !self.condition.met;
        false
    }

    fn parse_directive_if_defined(&mut self, expect_defined: bool) -> bool {
        self.conditions.push(self.condition);
        self.condition.kind = ConditionKind::If;
        if self.condition.ignore {
            self.eat_to_end_of_statement();
            return false;
        }
        let Some(name) = self.parse_identifier() else {
            return self.token_error("expected identifier after '.ifdef'");
        };
        if self.parse_end_of_line() {
            return true;
        }
        let defined = self
            .context
            .lookup(&name)
            .is_some_and(|symbol| !self.context.symbols[symbol].is_undefined());
        self.condition.met = if expect_defined { defined } else { !defined };
        self.condition.ignore = !self.condition.met;
        false
    }

    fn parse_directive_else_if(&mut self, location: SourceLocation) -> bool {
        if self.condition.kind != ConditionKind::If && self.condition.kind != ConditionKind::ElseIf
        {
            return self.error(
                location,
                "Encountered a .elseif that doesn't follow an .if or  an .elseif",
            );
        }
        self.condition.kind = ConditionKind::ElseIf;
        let last_ignore = self.conditions.last().is_some_and(|state| state.ignore);
        if last_ignore || self.condition.met {
            self.condition.ignore = true;
            self.eat_to_end_of_statement();
            return false;
        }
        let Ok(value) = self.parse_absolute_expression() else {
            return true;
        };
        if self.parse_end_of_line() {
            return true;
        }
        self.condition.met = value != 0;
        self.condition.ignore = !self.condition.met;
        false
    }

    fn parse_directive_else(&mut self, location: SourceLocation) -> bool {
        if self.parse_end_of_line() {
            return true;
        }
        if self.condition.kind != ConditionKind::If && self.condition.kind != ConditionKind::ElseIf
        {
            return self.error(
                location,
                "Encountered a .else that doesn't follow  an .if or an .elseif",
            );
        }
        self.condition.kind = ConditionKind::Else;
        let last_ignore = self.conditions.last().is_some_and(|state| state.ignore);
        self.condition.ignore = last_ignore || self.condition.met;
        false
    }

    fn parse_directive_end_if(&mut self, location: SourceLocation) -> bool {
        if self.parse_end_of_line() {
            return true;
        }
        if self.condition.kind == ConditionKind::None || self.conditions.is_empty() {
            return self.error(
                location,
                "Encountered a .endif that doesn't follow an .if or .else",
            );
        }
        if let Some(previous) = self.conditions.pop() {
            self.condition = previous;
        }
        false
    }
}

impl<'s> Assembler<'s> {
    fn parse_directive(&mut self, identifier: &str, lower: &str, location: SourceLocation) -> bool {
        if let Some(handled) = self.parse_target_directive(identifier, location) {
            return handled;
        }
        if self.context.triple.format == ObjectFormat::Elf {
            if let Some(handled) = self.parse_elf_directive(identifier, location) {
                return handled;
            }
        }
        if self.context.triple.format == ObjectFormat::Coff {
            if let Some(handled) = self.parse_coff_directive(identifier, location) {
                return handled;
            }
        }
        match lower {
            ".set" | ".equ" => self.parse_directive_set(AssignmentKind::Set),
            ".equiv" => self.parse_directive_set(AssignmentKind::Equiv),
            ".lto_set_conditional" => self.parse_directive_set(AssignmentKind::ConditionalSet),
            ".ascii" => self.parse_directive_ascii(false),
            ".asciz" | ".string" => self.parse_directive_ascii(true),
            ".byte" | ".dc.b" => self.parse_directive_value(1),
            ".dc" | ".dc.w" | ".short" | ".value" | ".2byte" | ".word" => {
                self.parse_directive_value(2)
            }
            ".long" | ".int" | ".4byte" | ".dc.l" => self.parse_directive_value(4),
            ".quad" | ".8byte" => self.parse_directive_value(8),
            ".dc.a" => {
                let size = if self.context.triple.is_64_bit() && !self.context.triple.x32 {
                    8
                } else {
                    4
                };
                self.parse_directive_value(size)
            }
            ".octa" => self.parse_directive_octa(),
            ".single" | ".float" | ".dc.s" => self.parse_directive_real(false),
            ".double" | ".dc.d" => self.parse_directive_real(true),
            ".align" => {
                let power = self.context.triple.format == ObjectFormat::MachObject;
                self.parse_directive_align(power, 1)
            }
            ".align32" => {
                let power = self.context.triple.format == ObjectFormat::MachObject;
                self.parse_directive_align(power, 4)
            }
            ".balign" => self.parse_directive_align(false, 1),
            ".balignw" => self.parse_directive_align(false, 2),
            ".balignl" => self.parse_directive_align(false, 4),
            ".p2align" => self.parse_directive_align(true, 1),
            ".p2alignw" => self.parse_directive_align(true, 2),
            ".p2alignl" => self.parse_directive_align(true, 4),
            ".org" => self.parse_directive_org(),
            ".fill" => self.parse_directive_fill(),
            ".zero" => self.parse_directive_zero(),
            ".extern" => {
                self.eat_to_end_of_statement();
                false
            }
            ".globl" | ".global" => self.parse_directive_symbol_attribute(SymbolAttribute::Global),
            ".lazy_reference" => {
                self.parse_directive_symbol_attribute(SymbolAttribute::LazyReference)
            }
            ".no_dead_strip" => self.parse_directive_symbol_attribute(SymbolAttribute::NoDeadStrip),
            ".symbol_resolver" => {
                self.parse_directive_symbol_attribute(SymbolAttribute::SymbolResolver)
            }
            ".private_extern" => {
                self.parse_directive_symbol_attribute(SymbolAttribute::PrivateExtern)
            }
            ".reference" => self.parse_directive_symbol_attribute(SymbolAttribute::Reference),
            ".weak_definition" => {
                self.parse_directive_symbol_attribute(SymbolAttribute::WeakDefinition)
            }
            ".weak_reference" => {
                self.parse_directive_symbol_attribute(SymbolAttribute::WeakReference)
            }
            ".weak_def_can_be_hidden" => {
                self.parse_directive_symbol_attribute(SymbolAttribute::WeakDefinitionAutoPrivate)
            }
            ".cold" => self.parse_directive_symbol_attribute(SymbolAttribute::Cold),
            ".memtag" => self.parse_directive_symbol_attribute(SymbolAttribute::Memtag),
            ".comm" | ".common" => self.parse_directive_common(false),
            ".lcomm" => self.parse_directive_common(true),
            ".abort" => {
                let text = self.parse_string_to_end_of_statement();
                if self.parse_end_of_line() {
                    return true;
                }
                if text.is_empty() {
                    self.error(location, ".abort detected. Assembly stopping")
                } else {
                    self.error(
                        location,
                        &format!(".abort '{text}' detected. Assembly stopping"),
                    )
                }
            }
            ".include" => self.parse_directive_include(),
            ".incbin" => self.parse_directive_include_binary(),
            ".code16" | ".code16gcc" => self.token_error(&format!(
                "{identifier} not currently supported for this target"
            )),
            ".rept" | ".rep" => self.parse_directive_repeat(location, identifier),
            ".irp" => self.parse_directive_iterate(location),
            ".irpc" => self.parse_directive_iterate_characters(location),
            ".endr" => {
                if self.active_macros.is_empty() {
                    return self.token_error("unmatched '.endr' directive");
                }
                self.handle_macro_exit();
                false
            }
            ".bundle_align_mode" => self.parse_directive_bundle_align_mode(),
            ".bundle_lock" => self.parse_directive_bundle_lock(),
            ".bundle_unlock" => {
                if self.check_for_valid_section() || self.parse_end_of_line() {
                    return true;
                }
                self.streamer
                    .emit_bundle_unlock(&mut self.context, location);
                false
            }
            ".sleb128" => self.parse_directive_leb128(true),
            ".uleb128" => self.parse_directive_leb128(false),
            ".space" | ".skip" => self.parse_directive_space(),
            ".file" => self.parse_directive_file(),
            ".line" | ".stabs" => {
                self.eat_to_end_of_statement();
                false
            }
            ".loc" | ".loc_label" => {
                let text = self.parse_string_to_end_of_statement();
                self.streamer
                    .emit_raw_text(&mut self.context, &format!("\t{lower}\t{}", text.trim()));
                self.parse_end_of_line()
            }
            ".cfi_sections" => self.parse_directive_cfi_sections(),
            ".cfi_startproc" => self.parse_directive_cfi_start_procedure(),
            ".cfi_endproc" => {
                self.cfi_start = None;
                if self.parse_end_of_line() {
                    return true;
                }
                self.streamer
                    .emit_cfi(&mut self.context, CfiInstruction::EndProcedure, location);
                false
            }
            ".cfi_def_cfa" => {
                self.parse_cfi_register_and_value(location, CfiInstruction::DefineFrame)
            }
            ".cfi_def_cfa_offset" => {
                self.parse_cfi_value(location, CfiInstruction::DefineFrameOffset)
            }
            ".cfi_adjust_cfa_offset" => {
                self.parse_cfi_value(location, CfiInstruction::AdjustFrameOffset)
            }
            ".cfi_def_cfa_register" => {
                self.parse_cfi_register(location, CfiInstruction::DefineFrameRegister)
            }
            ".cfi_offset" => self.parse_cfi_register_and_value(location, CfiInstruction::Offset),
            ".cfi_rel_offset" => {
                self.parse_cfi_register_and_value(location, CfiInstruction::RelativeOffset)
            }
            ".cfi_val_offset" => {
                self.parse_cfi_register_and_value(location, CfiInstruction::ValueOffset)
            }
            ".cfi_personality" => self.parse_directive_cfi_personality(false),
            ".cfi_lsda" => self.parse_directive_cfi_personality(true),
            ".cfi_remember_state" => self.parse_cfi_plain(location, CfiInstruction::RememberState),
            ".cfi_restore_state" => self.parse_cfi_plain(location, CfiInstruction::RestoreState),
            ".cfi_same_value" => self.parse_cfi_register(location, CfiInstruction::SameValue),
            ".cfi_restore" => self.parse_cfi_register(location, CfiInstruction::Restore),
            ".cfi_escape" => self.parse_directive_cfi_escape(location),
            ".cfi_return_column" => self.parse_cfi_register(location, CfiInstruction::ReturnColumn),
            ".cfi_signal_frame" => self.parse_cfi_plain(location, CfiInstruction::SignalFrame),
            ".cfi_undefined" => self.parse_cfi_register(location, CfiInstruction::Undefined),
            ".cfi_register" => {
                let Ok(first) = self.parse_register_or_number() else {
                    return true;
                };
                if self.parse_comma() {
                    return true;
                }
                let Ok(second) = self.parse_register_or_number() else {
                    return true;
                };
                if self.parse_end_of_line() {
                    return true;
                }
                self.streamer.emit_cfi(
                    &mut self.context,
                    CfiInstruction::Register(first, second),
                    location,
                );
                false
            }
            ".cfi_window_save" => self.parse_cfi_plain(location, CfiInstruction::WindowSave),
            ".cfi_label" => {
                let at = self.location();
                let Some(name) = self.parse_identifier() else {
                    return self.token_error("expected identifier");
                };
                if self.parse_end_of_line() {
                    return true;
                }
                self.streamer.emit_cfi_label(&mut self.context, &name, at);
                false
            }
            ".macros_on" | ".macros_off" => {
                if self.parse_end_of_line() {
                    return true;
                }
                self.macros_enabled = lower == ".macros_on";
                false
            }
            ".macro" => self.parse_directive_macro(location),
            ".altmacro" | ".noaltmacro" => {
                if self.parse_end_of_line() {
                    return true;
                }
                self.alternate_macro_mode = lower == ".altmacro";
                false
            }
            ".exitm" => {
                if self.parse_end_of_line() {
                    return true;
                }
                if self.active_macros.is_empty() {
                    return self.token_error(&format!(
                        "unexpected '{identifier}' in file, no current macro definition"
                    ));
                }
                let depth = self
                    .active_macros
                    .last()
                    .map(|active| active.condition_depth)
                    .unwrap_or(0);
                while self.conditions.len() != depth {
                    if let Some(previous) = self.conditions.pop() {
                        self.condition = previous;
                    }
                }
                self.handle_macro_exit();
                false
            }
            ".endm" | ".endmacro" => {
                if !self.is(TokenKind::EndOfStatement) {
                    return self
                        .token_error(&format!("unexpected token in '{identifier}' directive"));
                }
                if !self.active_macros.is_empty() {
                    self.handle_macro_exit();
                    return false;
                }
                self.token_error(&format!(
                    "unexpected '{identifier}' in file, no current macro definition"
                ))
            }
            ".purgem" => {
                let at = self.location();
                let Some(name) = self.parse_identifier() else {
                    return self.error(at, "expected identifier in '.purgem' directive");
                };
                if self.parse_end_of_line() {
                    return true;
                }
                if self.macros.remove(&name).is_none() {
                    return self.error(location, &format!("macro '{name}' is not defined"));
                }
                false
            }
            ".end" => {
                if self.parse_end_of_line() {
                    return true;
                }
                while !self.is(TokenKind::EndOfFile) {
                    self.lexer.lex();
                }
                false
            }
            ".err" => self.parse_directive_error(location, false),
            ".error" => self.parse_directive_error(location, true),
            ".warning" => self.parse_directive_warning(location),
            ".reloc" => self.parse_directive_relocation(),
            ".print" => {
                let token = self.lexer.token.clone();
                self.lex();
                if token.kind != TokenKind::String || !token.text.starts_with('"') {
                    return self.error(location, "expected double quoted string after .print");
                }
                self.parse_end_of_line()
            }
            ".addrsig" => {
                if self.parse_end_of_line() {
                    return true;
                }
                self.streamer
                    .emit_address_significance(&mut self.context, None);
                false
            }
            ".addrsig_sym" => {
                let Some(symbol) = self.parse_symbol() else {
                    return self.token_error("expected identifier");
                };
                if self.parse_end_of_line() {
                    return true;
                }
                self.streamer
                    .emit_address_significance(&mut self.context, Some(symbol));
                false
            }
            _ => self.error(location, "unknown directive"),
        }
    }

    fn parse_directive_set(&mut self, kind: AssignmentKind) -> bool {
        let Some(name) = self.parse_identifier() else {
            return self.token_error("expected identifier");
        };
        if self.parse_comma() {
            return true;
        }
        self.parse_assignment(&name, kind)
    }

    fn parse_directive_ascii(&mut self, zero_terminated: bool) -> bool {
        self.parse_many(|parser| {
            if parser.check_for_valid_section() {
                return true;
            }
            loop {
                let Ok(data) = parser.parse_escaped_string() else {
                    return true;
                };
                parser.streamer.emit_bytes(&mut parser.context, &data);
                if zero_terminated || !parser.is(TokenKind::String) {
                    break;
                }
            }
            if zero_terminated {
                parser.streamer.emit_bytes(&mut parser.context, &[0]);
            }
            false
        })
    }

    fn parse_directive_value(&mut self, size: u32) -> bool {
        self.parse_many(|parser| {
            let location = parser.location();
            if parser.check_for_valid_section() {
                return true;
            }
            let Ok((value, _)) = parser.parse_expression() else {
                return true;
            };
            if let Expression::Constant(constant) = value {
                let bits = 8 * size;
                let fits_unsigned = bits >= 64 || (constant as u64) < (1u64 << bits);
                let fits_signed = bits >= 64
                    || (constant >= -(1i64 << (bits - 1)) && constant < (1i64 << (bits - 1)));
                if !fits_unsigned && !fits_signed {
                    return parser.error(location, "out of range literal value");
                }
            }
            parser
                .streamer
                .emit_value(&mut parser.context, &value, size, location);
            false
        })
    }

    fn parse_directive_octa(&mut self) -> bool {
        self.parse_many(|parser| {
            if parser.check_for_valid_section() {
                return true;
            }
            if !parser.is(TokenKind::Integer) && !parser.is(TokenKind::BigNumber) {
                return parser.token_error("unknown token in expression");
            }
            let value = parser.lexer.token.value;
            parser.lex();
            let low = value as u64;
            let high = (value >> 64) as u64;
            let location = parser.location();
            parser.streamer.emit_value(
                &mut parser.context,
                &Expression::Constant(low as i64),
                8,
                location,
            );
            parser.streamer.emit_value(
                &mut parser.context,
                &Expression::Constant(high as i64),
                8,
                location,
            );
            false
        })
    }

    fn parse_real_value(&mut self, double: bool) -> Result<u64, ()> {
        let mut negative = false;
        if self.is(TokenKind::Minus) {
            self.lexer.lex();
            negative = true;
        } else if self.is(TokenKind::Plus) {
            self.lexer.lex();
        }
        if self.is(TokenKind::Error) {
            let message = self
                .lexer
                .error
                .clone()
                .map(|(_, text)| text)
                .unwrap_or_default();
            self.token_error(&message);
            return Err(());
        }
        if !self.is(TokenKind::Integer)
            && !self.is(TokenKind::Real)
            && !self.is(TokenKind::Identifier)
        {
            self.token_error("unexpected token in directive");
            return Err(());
        }
        let text = self.lexer.token.text.clone();
        let mut value: f64 = if self.is(TokenKind::Identifier) {
            let lower = text.to_ascii_lowercase();
            if lower == "infinity" || lower == "inf" {
                f64::INFINITY
            } else if lower == "nan" {
                f64::NAN
            } else {
                self.token_error("invalid floating point literal");
                return Err(());
            }
        } else {
            match parse_float_literal(&text) {
                Some(parsed) => parsed,
                None => {
                    self.token_error("invalid floating point literal");
                    return Err(());
                }
            }
        };
        if negative {
            value = -value;
        }
        self.lex();
        if double {
            Ok(value.to_bits())
        } else if value.is_nan() && !negative {
            Ok(u64::from(0x7fff_ffffu32))
        } else if value.is_nan() {
            Ok(u64::from(0xffff_ffffu32))
        } else {
            Ok(u64::from((value as f32).to_bits()))
        }
    }

    fn parse_directive_real(&mut self, double: bool) -> bool {
        self.parse_many(|parser| {
            if parser.check_for_valid_section() {
                return true;
            }
            let Ok(bits) = parser.parse_real_value(double) else {
                return true;
            };
            let location = parser.location();
            let size = if double { 8 } else { 4 };
            parser.streamer.emit_value(
                &mut parser.context,
                &Expression::Constant(bits as i64),
                size,
                location,
            );
            false
        })
    }

    fn parse_directive_zero(&mut self) -> bool {
        let location = self.location();
        if self.check_for_valid_section() {
            return true;
        }
        let Ok((count, _)) = self.parse_expression() else {
            return true;
        };
        let mut value = 0;
        if self.is(TokenKind::Comma) {
            self.lex();
            let Ok(parsed) = self.parse_absolute_expression() else {
                return true;
            };
            value = parsed;
        }
        if self.parse_end_of_line() {
            return true;
        }
        self.streamer
            .emit_fill_bytes(&mut self.context, &count, value as u64, location);
        false
    }

    fn parse_directive_fill(&mut self) -> bool {
        let location = self.location();
        if self.check_for_valid_section() {
            return true;
        }
        let Ok((count, _)) = self.parse_expression() else {
            return true;
        };
        let mut size = 1;
        let mut value = 0;
        let mut size_location = location;
        let mut value_location = location;
        if self.parse_optional_token(TokenKind::Comma) {
            size_location = self.location();
            let Ok(parsed) = self.parse_absolute_expression() else {
                return true;
            };
            size = parsed;
            if self.parse_optional_token(TokenKind::Comma) {
                value_location = self.location();
                let Ok(parsed) = self.parse_absolute_expression() else {
                    return true;
                };
                value = parsed;
            }
        }
        if self.parse_end_of_line() {
            return true;
        }
        if size < 0 {
            self.warning(
                size_location,
                "'.fill' directive with negative size has no effect",
            );
            return false;
        }
        if size > 8 {
            self.warning(
                size_location,
                "'.fill' directive with size greater than 8 has been truncated to 8",
            );
            size = 8;
        }
        if (value as u64) >> 32 != 0 && size > 4 {
            self.warning(
                value_location,
                "'.fill' directive pattern has been truncated to 32-bits",
            );
        }
        self.streamer
            .emit_fill(&mut self.context, &count, size, value, location);
        false
    }

    fn parse_directive_org(&mut self) -> bool {
        let location = self.location();
        if self.check_for_valid_section() {
            return true;
        }
        let Ok((offset, _)) = self.parse_expression() else {
            return true;
        };
        let mut value = 0;
        if self.parse_optional_token(TokenKind::Comma) {
            let Ok(parsed) = self.parse_absolute_expression() else {
                return true;
            };
            value = parsed;
        }
        if self.parse_end_of_line() {
            return true;
        }
        self.streamer
            .emit_value_to_offset(&mut self.context, &offset, value as u8, location);
        false
    }

    fn current_section_is_code(&self) -> bool {
        match self.context.current_section() {
            Some((section, _)) => {
                let held = &self.context.sections[section];
                if self.context.triple.format == ObjectFormat::Coff {
                    held.characteristics & 0x20 != 0
                } else {
                    held.flags & ELF_FLAG_EXECUTE != 0
                }
            }
            None => false,
        }
    }

    fn current_section_is_bss(&self) -> bool {
        match self.context.current_section() {
            Some((section, _)) => {
                let held = &self.context.sections[section];
                if self.context.triple.format == ObjectFormat::Coff {
                    held.characteristics & 0x80 != 0
                } else {
                    held.section_type == ELF_SECTION_NO_BITS
                }
            }
            None => false,
        }
    }

    fn parse_directive_align(&mut self, power_of_two: bool, value_size: u32) -> bool {
        let alignment_location = self.location();
        if self.check_for_valid_section() {
            return true;
        }
        if power_of_two && value_size == 1 && self.is(TokenKind::EndOfStatement) {
            self.warning(
                alignment_location,
                "p2align directive with no operand(s) is ignored",
            );
            return self.parse_end_of_line();
        }
        let Ok(mut alignment) = self.parse_absolute_expression() else {
            return true;
        };
        let mut has_fill = false;
        let mut fill: i64 = 0;
        let mut maximum: i64 = 0;
        let mut fill_location = alignment_location;
        let mut maximum_location: Option<SourceLocation> = None;
        if self.parse_optional_token(TokenKind::Comma) {
            if !self.is(TokenKind::Comma) {
                has_fill = true;
                fill_location = self.location();
                let Ok(parsed) = self.parse_absolute_expression() else {
                    return true;
                };
                fill = parsed;
            }
            if self.parse_optional_token(TokenKind::Comma) {
                maximum_location = Some(self.location());
                let Ok(parsed) = self.parse_absolute_expression() else {
                    return true;
                };
                maximum = parsed;
            }
        }
        if self.parse_end_of_line() {
            return true;
        }
        let mut result = false;
        if power_of_two {
            if alignment >= 32 {
                result |= self.error(alignment_location, "invalid alignment value");
                alignment = 31;
            }
            alignment = 1i64 << alignment;
        } else {
            if alignment == 0 {
                alignment = 1;
            } else if (alignment as u64).count_ones() != 1 {
                result |= self.error(alignment_location, "alignment must be a power of 2");
                alignment = 1i64 << (63 - (alignment as u64).leading_zeros());
            }
            if (alignment as u64) >> 32 != 0 {
                result |= self.error(alignment_location, "alignment must be smaller than 2**32");
                alignment = 1i64 << 31;
            }
        }
        if let Some(at) = maximum_location {
            if maximum < 1 {
                result |= self.error(at, "alignment directive can never be satisfied in this many bytes, ignoring maximum bytes expression");
                maximum = 0;
            }
            if maximum >= alignment {
                self.warning(
                    at,
                    "maximum bytes expression exceeds alignment and has no effect",
                );
                maximum = 0;
            }
        }
        if has_fill && fill != 0 && self.current_section_is_bss() {
            let name = self
                .context
                .current_section()
                .map(|(section, _)| self.context.sections[section].name.clone())
                .unwrap_or_default();
            result |= self.warning(
                fill_location,
                &format!("ignoring non-zero fill value in BSS section '{name}'"),
            );
            fill = 0;
        }
        if self.current_section_is_code() && !has_fill {
            let mode = self.mode;
            self.streamer.emit_code_alignment(
                &mut self.context,
                alignment as u64,
                maximum as u64,
                mode,
            );
        } else {
            self.streamer.emit_value_to_alignment(
                &mut self.context,
                alignment as u64,
                fill,
                value_size,
                maximum as u64,
            );
        }
        result
    }

    fn parse_directive_symbol_attribute(&mut self, attribute: SymbolAttribute) -> bool {
        self.parse_many(|parser| {
            let location = parser.location();
            let Some(name) = parser.parse_identifier() else {
                return parser.error(location, "expected identifier");
            };
            let symbol = parser.context.parse_symbol(&name);
            if parser.context.symbols[symbol].temporary && attribute != SymbolAttribute::Memtag {
                return parser.error(location, "non-local symbol required");
            }
            if !parser
                .streamer
                .emit_symbol_attribute(&mut parser.context, symbol, attribute)
            {
                return parser.error(location, "unable to emit symbol attribute");
            }
            false
        })
    }

    fn parse_directive_common(&mut self, local: bool) -> bool {
        if self.check_for_valid_section() {
            return true;
        }
        let location = self.location();
        let Some(symbol) = self.parse_symbol() else {
            return self.token_error("expected identifier in directive");
        };
        if self.parse_comma() {
            return true;
        }
        let size_location = self.location();
        let Ok(size) = self.parse_absolute_expression() else {
            return true;
        };
        let mut power: i64 = 0;
        if self.is(TokenKind::Comma) {
            self.lex();
            let power_location = self.location();
            let Ok(parsed) = self.parse_absolute_expression() else {
                return true;
            };
            power = parsed;
            let in_bytes = self.context.triple.format != ObjectFormat::MachObject;
            let local_in_bytes = self.context.triple.format == ObjectFormat::Elf;
            if (!local && in_bytes) || (local && local_in_bytes) {
                if (power as u64).count_ones() != 1 {
                    return self.error(power_location, "alignment must be a power of 2");
                }
                power = i64::from(63 - (power as u64).leading_zeros());
            }
        }
        if self.parse_end_of_line() {
            return true;
        }
        if size < 0 {
            return self.error(size_location, "size must be non-negative");
        }
        let held = &self.context.symbols[symbol];
        if !held.is_undefined() {
            return self.error(location, "invalid symbol redefinition");
        }
        self.streamer.emit_common_symbol(
            &mut self.context,
            symbol,
            size as u64,
            1u64 << power,
            local,
        );
        false
    }

    fn parse_directive_include(&mut self) -> bool {
        let location = self.location();
        if self.check(
            !self.is(TokenKind::String),
            "expected string in '.include' directive",
        ) {
            return true;
        }
        let Ok(name) = self.parse_escaped_string() else {
            return true;
        };
        if self.check(
            !self.is(TokenKind::EndOfStatement),
            "unexpected token in '.include' directive",
        ) {
            return true;
        }
        let name = String::from_utf8_lossy(&name).into_owned();
        let contents = self.read_file(&name);
        let Some(contents) = contents else {
            return self.error(location, &format!("Could not find include file '{name}'"));
        };
        let parent = self.lexer.token.end_location();
        let buffer = self.lexer.add_buffer(&name, contents, Some(parent));
        self.lexer.jump_to(SourceLocation {
            buffer,
            position: 0,
        });
        self.lex();
        false
    }

    fn read_file(&self, name: &str) -> Option<Vec<u8>> {
        let reader = self.file_reader?;
        if let Some(found) = reader(name) {
            return Some(found);
        }
        for directory in &self.options.include_directories {
            if let Some(found) = reader(&format!("{directory}/{name}")) {
                return Some(found);
            }
        }
        None
    }

    fn parse_directive_include_binary(&mut self) -> bool {
        let location = self.location();
        if self.check(
            !self.is(TokenKind::String),
            "expected string in '.incbin' directive",
        ) {
            return true;
        }
        let Ok(name) = self.parse_escaped_string() else {
            return true;
        };
        let mut skip: i64 = 0;
        let mut count: Option<(Expression, SourceLocation)> = None;
        let mut skip_location = location;
        if self.parse_optional_token(TokenKind::Comma) {
            if !self.is(TokenKind::Comma) {
                skip_location = self.location();
                let Ok(parsed) = self.parse_absolute_expression() else {
                    return true;
                };
                skip = parsed;
            }
            if self.parse_optional_token(TokenKind::Comma) {
                let at = self.location();
                let Ok((expression, _)) = self.parse_expression() else {
                    return true;
                };
                count = Some((expression, at));
            }
        }
        if self.parse_end_of_line() {
            return true;
        }
        if skip < 0 {
            return self.error(skip_location, "skip is negative");
        }
        let name = String::from_utf8_lossy(&name).into_owned();
        let Some(contents) = self.read_file(&name) else {
            return self.error(location, &format!("Could not find incbin file '{name}'"));
        };
        let mut bytes: Vec<u8> = contents
            .get(skip as usize..)
            .map(<[u8]>::to_vec)
            .unwrap_or_default();
        if let Some((expression, at)) = count {
            match self.context.evaluate_absolute(&expression, None) {
                Some(value) => {
                    if value < 0 {
                        self.warning(at, "negative count has no effect");
                        return false;
                    }
                    bytes.truncate(value as usize);
                }
                None => return self.error(at, "expected absolute expression"),
            }
        }
        self.streamer.emit_bytes(&mut self.context, &bytes);
        false
    }

    fn parse_directive_leb128(&mut self, signed: bool) -> bool {
        if self.check_for_valid_section() {
            return true;
        }
        self.parse_many(|parser| {
            let Ok((value, _)) = parser.parse_expression() else {
                return true;
            };
            parser
                .streamer
                .emit_leb128(&mut parser.context, &value, signed);
            false
        })
    }

    fn parse_directive_space(&mut self) -> bool {
        let location = self.location();
        if self.check_for_valid_section() {
            return true;
        }
        let Ok((count, _)) = self.parse_expression() else {
            return true;
        };
        let mut value = 0;
        if self.parse_optional_token(TokenKind::Comma) {
            let Ok(parsed) = self.parse_absolute_expression() else {
                return true;
            };
            value = parsed;
        }
        if self.parse_end_of_line() {
            return true;
        }
        self.streamer
            .emit_fill_bytes(&mut self.context, &count, value as u64, location);
        false
    }

    fn parse_directive_file(&mut self) -> bool {
        let mut number: i64 = -1;
        if self.is(TokenKind::Integer) {
            number = self.lexer.token.integer();
            self.lex();
            if number < 0 {
                return self.token_error("negative file number");
            }
        }
        let Ok(path) = self.parse_escaped_string() else {
            return true;
        };
        if number == -1 {
            let rest = if self.is(TokenKind::EndOfStatement) {
                String::new()
            } else {
                self.parse_string_to_end_of_statement()
            };
            let _ = rest;
            if self.parse_end_of_line() {
                return true;
            }
            self.streamer.emit_file(&mut self.context, &path);
            return false;
        }
        let mut text = format!("\t.file\t{number} {}", quoted_string(&path));
        if self.is(TokenKind::String) {
            let Ok(second) = self.parse_escaped_string() else {
                return true;
            };
            text = format!(
                "\t.file\t{number} {} {}",
                quoted_string(&path),
                quoted_string(&second)
            );
        }
        let rest = self.parse_string_to_end_of_statement();
        if !rest.trim().is_empty() {
            text.push(' ');
            text.push_str(rest.trim());
        }
        if self.parse_end_of_line() {
            return true;
        }
        self.streamer.emit_raw_text(&mut self.context, &text);
        false
    }

    fn parse_directive_cfi_sections(&mut self) -> bool {
        let mut exception = false;
        let mut debug = false;
        let mut frame = false;
        if !self.parse_optional_token(TokenKind::EndOfStatement) {
            loop {
                let Some(name) = self.parse_identifier() else {
                    return self.token_error("expected .eh_frame, .debug_frame, or .sframe");
                };
                match name.as_str() {
                    ".eh_frame" => exception = true,
                    ".debug_frame" => debug = true,
                    ".sframe" => frame = true,
                    _ => {}
                }
                if self.parse_optional_token(TokenKind::EndOfStatement) {
                    break;
                }
                if self.parse_comma() {
                    return true;
                }
            }
        }
        let location = self.location();
        self.streamer.emit_cfi(
            &mut self.context,
            CfiInstruction::Sections(exception, debug, frame),
            location,
        );
        false
    }

    fn parse_directive_cfi_start_procedure(&mut self) -> bool {
        self.cfi_start = Some(self.statement_start);
        let mut simple = false;
        if !self.parse_optional_token(TokenKind::EndOfStatement) {
            let name = self.parse_identifier();
            if name.as_deref() != Some("simple") {
                return self.token_error("unexpected token");
            }
            if self.parse_end_of_line() {
                return true;
            }
            simple = true;
        }
        let location = self.location();
        self.streamer.emit_cfi(
            &mut self.context,
            CfiInstruction::StartProcedure(simple),
            location,
        );
        false
    }

    fn parse_register_or_number(&mut self) -> Result<i64, ()> {
        if !self.is(TokenKind::Integer) {
            let start = self.location();
            let Some((register, _)) = self.parse_register_operand(start) else {
                return Err(());
            };
            return Ok(self.dwarf_number(register));
        }
        self.parse_absolute_expression()
    }

    pub fn dwarf_number(&self, register: RegisterId) -> i64 {
        let flavour = if self.context.triple.is_64_bit() {
            0
        } else if self.context.triple.format == ObjectFormat::MachObject {
            2
        } else {
            1
        };
        self.target.registers[register as usize]
            .dwarf_numbers
            .get(flavour)
            .copied()
            .unwrap_or(-1)
    }

    fn parse_cfi_plain(&mut self, location: SourceLocation, instruction: CfiInstruction) -> bool {
        if self.parse_end_of_line() {
            return true;
        }
        self.streamer
            .emit_cfi(&mut self.context, instruction, location);
        false
    }

    fn parse_cfi_value(
        &mut self,
        location: SourceLocation,
        make: fn(i64) -> CfiInstruction,
    ) -> bool {
        let Ok(value) = self.parse_absolute_expression() else {
            return true;
        };
        if self.parse_end_of_line() {
            return true;
        }
        self.streamer
            .emit_cfi(&mut self.context, make(value), location);
        false
    }

    fn parse_cfi_register(
        &mut self,
        location: SourceLocation,
        make: fn(i64) -> CfiInstruction,
    ) -> bool {
        let Ok(register) = self.parse_register_or_number() else {
            return true;
        };
        if self.parse_end_of_line() {
            return true;
        }
        self.streamer
            .emit_cfi(&mut self.context, make(register), location);
        false
    }

    fn parse_cfi_register_and_value(
        &mut self,
        location: SourceLocation,
        make: fn(i64, i64) -> CfiInstruction,
    ) -> bool {
        let Ok(register) = self.parse_register_or_number() else {
            return true;
        };
        if self.parse_comma() {
            return true;
        }
        let Ok(value) = self.parse_absolute_expression() else {
            return true;
        };
        if self.parse_end_of_line() {
            return true;
        }
        self.streamer
            .emit_cfi(&mut self.context, make(register, value), location);
        false
    }

    fn parse_directive_cfi_personality(&mut self, lsda: bool) -> bool {
        let Ok(encoding) = self.parse_absolute_expression() else {
            return true;
        };
        if encoding == 0xff {
            return false;
        }
        let format = encoding & 0xf;
        let application = encoding & 0x70;
        let valid = encoding & !0xff == 0
            && matches!(format, 0x0 | 0x2 | 0x3 | 0x4 | 0xa | 0xb | 0xc | 0x8)
            && matches!(application, 0x0 | 0x10);
        if self.check(!valid, "unsupported encoding.") || self.parse_comma() {
            return true;
        }
        let Some(symbol) = self.parse_symbol() else {
            return self.token_error("expected identifier in directive");
        };
        if self.parse_end_of_line() {
            return true;
        }
        self.streamer
            .emit_cfi_personality(&mut self.context, symbol, encoding, lsda);
        false
    }

    fn parse_directive_cfi_escape(&mut self, location: SourceLocation) -> bool {
        let mut values: Vec<u8> = Vec::new();
        let Ok(first) = self.parse_absolute_expression() else {
            return true;
        };
        values.push(first as u8);
        while self.is(TokenKind::Comma) {
            self.lex();
            let Ok(next) = self.parse_absolute_expression() else {
                return true;
            };
            values.push(next as u8);
        }
        self.streamer
            .emit_cfi_escape(&mut self.context, &values, location);
        false
    }

    fn parse_directive_error(&mut self, location: SourceLocation, with_message: bool) -> bool {
        if self.conditions.last().is_some_and(|state| state.ignore) {
            self.eat_to_end_of_statement();
            return false;
        }
        if !with_message {
            return self.error(location, ".err encountered");
        }
        let mut message = ".error directive invoked in source file".to_string();
        if !self.is(TokenKind::EndOfStatement) {
            if !self.is(TokenKind::String) {
                return self.token_error(".error argument must be a string");
            }
            message = string_contents(&self.lexer.token.text);
            self.lex();
        }
        self.error(location, &message)
    }

    fn parse_directive_warning(&mut self, location: SourceLocation) -> bool {
        if self.conditions.last().is_some_and(|state| state.ignore) {
            self.eat_to_end_of_statement();
            return false;
        }
        let mut message = ".warning directive invoked in source file".to_string();
        if !self.parse_optional_token(TokenKind::EndOfStatement) {
            if !self.is(TokenKind::String) {
                return self.token_error(".warning argument must be a string");
            }
            message = string_contents(&self.lexer.token.text);
            self.lex();
            if self.parse_end_of_line() {
                return true;
            }
        }
        self.warning(location, &message)
    }

    fn parse_directive_relocation(&mut self) -> bool {
        let Ok((offset, _)) = self.parse_expression() else {
            return true;
        };
        if self.parse_comma()
            || self.check(!self.is(TokenKind::Identifier), "expected relocation name")
        {
            return true;
        }
        let name_location = self.location();
        let name = self.lexer.token.identifier();
        self.lex();
        let mut value: Option<Expression> = None;
        if self.is(TokenKind::Comma) {
            self.lex();
            let at = self.location();
            let Ok((expression, _)) = self.parse_expression() else {
                return true;
            };
            if self
                .context
                .evaluate_relocatable(&expression, None, false)
                .is_none()
            {
                return self.error(at, "expression must be relocatable");
            }
            value = Some(expression);
        }
        if self.parse_end_of_line() {
            return true;
        }
        self.streamer.emit_relocation_directive(
            &mut self.context,
            &offset,
            &name,
            value.as_ref(),
            name_location,
        );
        false
    }

    fn parse_directive_bundle_align_mode(&mut self) -> bool {
        let location = self.location();
        if self.check_for_valid_section() {
            return true;
        }
        let Ok(power) = self.parse_absolute_expression() else {
            return true;
        };
        if self.parse_end_of_line() {
            return true;
        }
        if !(1..=30).contains(&power) {
            return self.error(
                location,
                "invalid bundle alignment size (expected between 1 and 30)",
            );
        }
        self.streamer
            .emit_bundle_align_mode(&mut self.context, power as u32);
        false
    }

    fn parse_directive_bundle_lock(&mut self) -> bool {
        if self.check_for_valid_section() {
            return true;
        }
        let mut align_to_end = false;
        let location = self.location();
        if !self.parse_optional_token(TokenKind::EndOfStatement) {
            let option = self.parse_identifier();
            if option.as_deref() != Some("align_to_end") {
                return self.error(location, "invalid option for `.bundle_lock`");
            }
            if self.parse_end_of_line() {
                return true;
            }
            align_to_end = true;
        }
        self.streamer
            .emit_bundle_lock(&mut self.context, align_to_end, location);
        false
    }
}

pub fn parse_float_literal(text: &str) -> Option<f64> {
    let lower = text.to_ascii_lowercase();
    if let Some(hexadecimal) = lower.strip_prefix("0x") {
        let (mantissa, exponent) = hexadecimal.split_once('p')?;
        let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        let mut value = 0f64;
        for byte in whole.bytes() {
            value = value * 16.0 + f64::from(hexadecimal_value(byte));
        }
        let mut scale = 1.0 / 16.0;
        for byte in fraction.bytes() {
            value += f64::from(hexadecimal_value(byte)) * scale;
            scale /= 16.0;
        }
        let exponent: i32 = exponent.parse().ok()?;
        return Some(value * 2f64.powi(exponent));
    }
    let trimmed = if lower.ends_with('.') {
        &lower[..lower.len() - 1]
    } else {
        lower.as_str()
    };
    trimmed.parse::<f64>().ok()
}

impl<'s> Assembler<'s> {
    fn switch_section(&mut self, section: usize, subsection: i64) {
        self.streamer
            .switch_section(&mut self.context, section, subsection);
    }

    fn parse_subsection_value(
        &mut self,
        expression: &Expression,
        location: SourceLocation,
    ) -> Option<i64> {
        match self.context.evaluate_absolute(expression, None) {
            Some(value) => {
                if !(0..=0x7fff_ffff).contains(&value) {
                    self.error(
                        location,
                        &format!("subsection number {value} is not within [0,2147483647]"),
                    );
                    return None;
                }
                Some(value)
            }
            None => {
                self.error(location, "cannot evaluate subsection number");
                None
            }
        }
    }

    fn parse_elf_directive(&mut self, identifier: &str, location: SourceLocation) -> Option<bool> {
        let result = match identifier {
            ".data" => self.parse_section_switch(
                ".data",
                ELF_SECTION_PROGRAM_BITS,
                ELF_FLAG_WRITE | ELF_FLAG_ALLOCATE,
            ),
            ".text" => self.parse_section_switch(
                ".text",
                ELF_SECTION_PROGRAM_BITS,
                ELF_FLAG_EXECUTE | ELF_FLAG_ALLOCATE,
            ),
            ".bss" => self.parse_section_switch(
                ".bss",
                ELF_SECTION_NO_BITS,
                ELF_FLAG_WRITE | ELF_FLAG_ALLOCATE,
            ),
            ".rodata" => {
                self.parse_section_switch(".rodata", ELF_SECTION_PROGRAM_BITS, ELF_FLAG_ALLOCATE)
            }
            ".tdata" => self.parse_section_switch(
                ".tdata",
                ELF_SECTION_PROGRAM_BITS,
                ELF_FLAG_ALLOCATE | ELF_FLAG_THREAD_LOCAL | ELF_FLAG_WRITE,
            ),
            ".tbss" => self.parse_section_switch(
                ".tbss",
                ELF_SECTION_NO_BITS,
                ELF_FLAG_ALLOCATE | ELF_FLAG_THREAD_LOCAL | ELF_FLAG_WRITE,
            ),
            ".section" => self.parse_section_arguments(false, location),
            ".pushsection" => {
                let top = self
                    .context
                    .section_stack
                    .last()
                    .copied()
                    .unwrap_or((None, None));
                self.context.section_stack.push(top);
                if self.parse_section_arguments(true, location) {
                    self.pop_section();
                    true
                } else {
                    false
                }
            }
            ".popsection" => {
                if !self.pop_section() {
                    self.token_error(".popsection without corresponding .pushsection")
                } else {
                    false
                }
            }
            ".previous" => match self.context.previous_section() {
                Some((section, subsection)) => {
                    self.switch_section(section, subsection);
                    false
                }
                None => self.token_error(".previous without corresponding .section"),
            },
            ".size" => self.parse_directive_size(),
            ".type" => self.parse_directive_type(),
            ".ident" => {
                if !self.is(TokenKind::String) {
                    return Some(self.token_error("expected string"));
                }
                let data = string_contents(&self.lexer.token.text).into_bytes();
                self.lex();
                if !self.is(TokenKind::EndOfStatement) {
                    return Some(self.token_error("expected end of directive"));
                }
                self.lex();
                self.streamer.emit_identifier(&mut self.context, &data);
                false
            }
            ".symver" => self.parse_directive_symbol_version(),
            ".weakref" => {
                let Some(alias) = self.parse_symbol() else {
                    return Some(self.token_error("expected identifier"));
                };
                if !self.is(TokenKind::Comma) {
                    return Some(self.token_error("expected a comma"));
                }
                self.lex();
                let Some(symbol) = self.parse_symbol() else {
                    return Some(self.token_error("expected identifier"));
                };
                self.streamer
                    .emit_weak_reference(&mut self.context, alias, symbol);
                false
            }
            ".weak" | ".local" | ".hidden" | ".internal" | ".protected" => {
                let attribute = match identifier {
                    ".weak" => SymbolAttribute::Weak,
                    ".local" => SymbolAttribute::Local,
                    ".hidden" => SymbolAttribute::Hidden,
                    ".internal" => SymbolAttribute::Internal,
                    _ => SymbolAttribute::Protected,
                };
                if !self.is(TokenKind::EndOfStatement) {
                    loop {
                        let Some(name) = self.parse_identifier() else {
                            return Some(self.token_error("expected identifier"));
                        };
                        let symbol = self.context.parse_symbol(&name);
                        self.streamer
                            .emit_symbol_attribute(&mut self.context, symbol, attribute);
                        if self.is(TokenKind::EndOfStatement) {
                            break;
                        }
                        if !self.is(TokenKind::Comma) {
                            return Some(self.token_error("expected comma"));
                        }
                        self.lex();
                    }
                }
                self.lex();
                false
            }
            ".subsection" => {
                let mut value = Expression::Constant(0);
                let at = self.location();
                if !self.is(TokenKind::EndOfStatement) {
                    let Ok((parsed, _)) = self.parse_expression() else {
                        return Some(true);
                    };
                    value = parsed;
                }
                if !self.is(TokenKind::EndOfStatement) {
                    return Some(self.token_error("expected end of directive"));
                }
                self.lex();
                let Some(subsection) = self.parse_subsection_value(&value, at) else {
                    return Some(true);
                };
                if let Some((section, _)) = self.context.current_section() {
                    self.switch_section(section, subsection);
                }
                false
            }
            ".version" => {
                if !self.is(TokenKind::String) {
                    return Some(self.token_error("expected string"));
                }
                let data = string_contents(&self.lexer.token.text).into_bytes();
                self.lex();
                let note = self
                    .context
                    .elf_section(".note", ELF_SECTION_NOTE, 0, 0, "", None);
                let top = self
                    .context
                    .section_stack
                    .last()
                    .copied()
                    .unwrap_or((None, None));
                self.context.section_stack.push(top);
                self.switch_section(note, 0);
                let at = self.location();
                for value in [data.len() as i64 + 1, 0, 1] {
                    self.streamer.emit_value(
                        &mut self.context,
                        &Expression::Constant(value),
                        4,
                        at,
                    );
                }
                self.streamer.emit_bytes(&mut self.context, &data);
                self.streamer
                    .emit_value(&mut self.context, &Expression::Constant(0), 1, at);
                self.streamer
                    .emit_value_to_alignment(&mut self.context, 4, 0, 1, 0);
                self.pop_section();
                false
            }
            _ => return None,
        };
        Some(result)
    }

    fn pop_section(&mut self) -> bool {
        if self.context.section_stack.len() <= 1 {
            return false;
        }
        self.context.section_stack.pop();
        if let Some((section, subsection)) = self.context.current_section() {
            self.streamer
                .switch_section(&mut self.context, section, subsection);
        }
        true
    }

    fn parse_section_switch(&mut self, name: &str, section_type: u32, flags: u64) -> bool {
        let mut subsection = 0;
        if !self.is(TokenKind::EndOfStatement) {
            let at = self.location();
            let Ok((value, _)) = self.parse_expression() else {
                return true;
            };
            match self.parse_subsection_value(&value, at) {
                Some(found) => subsection = found,
                None => return true,
            }
        }
        self.lex();
        let section = self
            .context
            .elf_section(name, section_type, flags, 0, "", None);
        self.switch_section(section, subsection);
        false
    }

    fn parse_section_name(&mut self) -> Option<String> {
        if self.is(TokenKind::String) {
            let name = self.lexer.token.identifier();
            self.lex();
            return Some(name);
        }
        let first = self.location();
        let mut size = 0usize;
        let mut name = String::new();
        while !self.has_pending_error() {
            let previous = self.location();
            if self.is(TokenKind::Comma) || self.is(TokenKind::EndOfStatement) {
                break;
            }
            let current = if self.is(TokenKind::String) {
                self.lexer.token.identifier().len() + 2
            } else {
                self.lexer.token.text.len()
            };
            self.lex();
            size += current;
            name = self.lexer.text_between(
                first,
                SourceLocation {
                    buffer: first.buffer,
                    position: first.position + size,
                },
            );
            let now = self.location();
            if previous.position + current != now.position || previous.buffer != now.buffer {
                break;
            }
        }
        if size == 0 {
            return None;
        }
        Some(name)
    }

    fn parse_section_arguments(&mut self, push: bool, location: SourceLocation) -> bool {
        let Some(name) = self.parse_section_name() else {
            return self.token_error("expected identifier");
        };
        let mut type_name = String::new();
        let mut size: i64 = 0;
        let mut group_name = String::new();
        let mut comdat = false;
        let mut flags: u64 = 0;
        let mut extra_flags: u64 = 0;
        let mut subsection: i64 = 0;
        let mut use_last_group = false;
        let mut linked: Option<String> = None;
        let mut unique: Option<u64> = None;
        if has_section_prefix(&name, ".rodata") || name == ".rodata1" {
            flags |= ELF_FLAG_ALLOCATE;
        } else if name == ".fini" || name == ".init" || has_section_prefix(&name, ".text") {
            flags |= ELF_FLAG_ALLOCATE | ELF_FLAG_EXECUTE;
        } else if has_section_prefix(&name, ".data")
            || name == ".data1"
            || has_section_prefix(&name, ".bss")
            || has_section_prefix(&name, ".init_array")
            || has_section_prefix(&name, ".fini_array")
            || has_section_prefix(&name, ".preinit_array")
        {
            flags |= ELF_FLAG_ALLOCATE | ELF_FLAG_WRITE;
        } else if has_section_prefix(&name, ".tdata") || has_section_prefix(&name, ".tbss") {
            flags |= ELF_FLAG_ALLOCATE | ELF_FLAG_WRITE | ELF_FLAG_THREAD_LOCAL;
        }
        let mut skip_to_end = false;
        if self.is(TokenKind::Comma) {
            self.lex();
            if push && !self.is(TokenKind::String) {
                let at = self.location();
                let Ok((value, _)) = self.parse_expression() else {
                    return true;
                };
                match self.parse_subsection_value(&value, at) {
                    Some(found) => subsection = found,
                    None => return true,
                }
                if !self.is(TokenKind::Comma) {
                    skip_to_end = true;
                } else {
                    self.lex();
                }
            }
            if !skip_to_end {
                if !self.is(TokenKind::String) {
                    if !self.is(TokenKind::Hash) {
                        return self.token_error("expected string");
                    }
                    let mut sun_flags = 0u64;
                    let mut valid = true;
                    while self.is(TokenKind::Hash) {
                        self.lex();
                        if !self.is(TokenKind::Identifier) {
                            valid = false;
                            break;
                        }
                        match self.lexer.token.text.as_str() {
                            "alloc" => sun_flags |= ELF_FLAG_ALLOCATE,
                            "execinstr" => sun_flags |= ELF_FLAG_EXECUTE,
                            "write" => sun_flags |= ELF_FLAG_WRITE,
                            "tls" => sun_flags |= ELF_FLAG_THREAD_LOCAL,
                            _ => {
                                valid = false;
                                break;
                            }
                        }
                        self.lex();
                        if !self.is(TokenKind::Comma) {
                            break;
                        }
                        self.lex();
                    }
                    if !valid {
                        return self.token_error("unknown flag");
                    }
                    extra_flags = sun_flags;
                } else {
                    let text = string_contents(&self.lexer.token.text);
                    self.lex();
                    match self.parse_section_flags(&text, &mut use_last_group) {
                        Some(found) => extra_flags = found,
                        None => return self.token_error("unknown flag"),
                    }
                }
                flags |= extra_flags;
                let mergeable = flags & ELF_FLAG_MERGE != 0;
                let group = flags & ELF_FLAG_GROUP != 0;
                if group && use_last_group {
                    return self.token_error("Section cannot specifiy a group name while also acting as a member of the last group");
                }
                if self.is(TokenKind::Comma) {
                    self.lex();
                    if !self.is(TokenKind::At)
                        && !self.is(TokenKind::Percent)
                        && !self.is(TokenKind::String)
                    {
                        return self.token_error("expected '@<type>', '%<type>' or \"<type>\"");
                    }
                    if !self.is(TokenKind::String) {
                        self.lex();
                    }
                    if self.is(TokenKind::Integer) {
                        type_name = self.lexer.token.text.clone();
                        self.lex();
                    } else {
                        match self.parse_identifier() {
                            Some(found) => type_name = found,
                            None => return self.token_error("expected identifier"),
                        }
                    }
                }
                if type_name.is_empty() {
                    if mergeable {
                        return self.token_error("Mergeable section must specify the type");
                    }
                    if group {
                        return self.token_error("Group section must specify the type");
                    }
                    if !self.is(TokenKind::EndOfStatement) {
                        return self.token_error("expected end of directive");
                    }
                }
                if mergeable || type_name == "llvm_cfi_jump_table" {
                    if !self.is(TokenKind::Comma) {
                        return self.token_error("expected the entry size");
                    }
                    self.lex();
                    let Ok(parsed) = self.parse_absolute_expression() else {
                        return true;
                    };
                    if parsed <= 0 {
                        return self.token_error("entry size must be positive");
                    }
                    size = parsed;
                }
                if flags & ELF_FLAG_LINK_ORDER != 0 {
                    if !self.is(TokenKind::Comma) {
                        return self.token_error("expected linked-to symbol");
                    }
                    self.lex();
                    let start = self.location();
                    match self.parse_identifier() {
                        Some(found) => {
                            let in_section = self
                                .context
                                .lookup(&found)
                                .is_some_and(|symbol| self.context.is_in_section(symbol));
                            if !in_section {
                                return self.error(
                                    start,
                                    &format!("linked-to symbol is not in a section: {found}"),
                                );
                            }
                            linked = Some(found);
                        }
                        None => {
                            if self.lexer.token.text == "0" {
                                self.lex();
                            } else {
                                return self.token_error("invalid linked-to symbol");
                            }
                        }
                    }
                }
                if group {
                    if !self.is(TokenKind::Comma) {
                        return self.token_error("expected group name");
                    }
                    self.lex();
                    if self.is(TokenKind::Integer) {
                        group_name = self.lexer.token.text.clone();
                        self.lex();
                    } else {
                        match self.parse_identifier() {
                            Some(found) => group_name = found,
                            None => return self.token_error("invalid group name"),
                        }
                    }
                    if self.is(TokenKind::Comma) {
                        self.lex();
                        let linkage = self.parse_identifier();
                        match linkage.as_deref() {
                            None => return self.token_error("invalid linkage"),
                            Some("comdat") => comdat = true,
                            Some(_) => return self.token_error("Linkage must be 'comdat'"),
                        }
                    }
                }
                if self.is(TokenKind::Comma) {
                    self.lex();
                    let unique_name = self.parse_identifier();
                    if unique_name.as_deref() != Some("unique") {
                        return self.token_error("expected 'unique'");
                    }
                    if !self.is(TokenKind::Comma) {
                        return self.token_error("expected commma");
                    }
                    self.lex();
                    let Ok(parsed) = self.parse_absolute_expression() else {
                        return true;
                    };
                    if parsed < 0 {
                        return self.token_error("unique id must be positive");
                    }
                    if parsed as u64 == u64::from(u32::MAX) {
                        return self.token_error("unique id is too large");
                    }
                    unique = Some(parsed as u64);
                }
            }
        }
        if !self.is(TokenKind::EndOfStatement) {
            return self.token_error("expected end of directive");
        }
        self.lex();
        let mut section_type = ELF_SECTION_PROGRAM_BITS;
        if type_name.is_empty() {
            if name.starts_with(".note") {
                section_type = ELF_SECTION_NOTE;
            } else if has_section_prefix(&name, ".init_array") {
                section_type = ELF_SECTION_INITIALIZER_ARRAY;
            } else if has_section_prefix(&name, ".bss") || has_section_prefix(&name, ".tbss") {
                section_type = ELF_SECTION_NO_BITS;
            } else if has_section_prefix(&name, ".fini_array") {
                section_type = ELF_SECTION_FINALIZER_ARRAY;
            } else if has_section_prefix(&name, ".preinit_array") {
                section_type = ELF_SECTION_PREINITIALIZER_ARRAY;
            }
        } else {
            section_type = match type_name.as_str() {
                "init_array" => ELF_SECTION_INITIALIZER_ARRAY,
                "fini_array" => ELF_SECTION_FINALIZER_ARRAY,
                "preinit_array" => ELF_SECTION_PREINITIALIZER_ARRAY,
                "nobits" => ELF_SECTION_NO_BITS,
                "progbits" => ELF_SECTION_PROGRAM_BITS,
                "note" => ELF_SECTION_NOTE,
                "unwind" => ELF_SECTION_X86_64_UNWIND,
                "llvm_odrtab" => 0x6fff_4c00,
                "llvm_linker_options" => 0x6fff_4c01,
                "llvm_call_graph_profile" => ELF_SECTION_LLVM_CALL_GRAPH_PROFILE,
                "llvm_dependent_libraries" => 0x6fff_4c04,
                "llvm_sympart" => 0x6fff_4c05,
                "llvm_bb_addr_map" => 0x6fff_4c0a,
                "llvm_offloading" => 0x6fff_4c0b,
                "llvm_lto" => 0x6fff_4c0c,
                "llvm_jt_sizes" => 0x6fff_4c0d,
                "llvm_cfi_jump_table" => 0x6fff_4c0e,
                other => match parse_integer_text(other) {
                    Some(value) => value as u32,
                    None => return self.token_error("unknown section type"),
                },
            };
        }
        if use_last_group {
            if let Some((current, _)) = self.context.current_section() {
                if let Some(group) = self.context.sections[current].group.clone() {
                    group_name = group;
                    comdat = self.context.sections[current].comdat;
                    flags |= ELF_FLAG_GROUP;
                }
            }
        }
        let existing = self.context.find_section(&name, &group_name, unique);
        let section = match existing {
            Some(found) => found,
            None => {
                let created = self.context.elf_section(
                    &name,
                    section_type,
                    flags,
                    size as u64,
                    &group_name,
                    unique,
                );
                self.context.sections[created].comdat = comdat;
                self.context.sections[created].linked_symbol = linked.clone();
                created
            }
        };
        self.switch_section(section, subsection);
        let held = self.context.sections[section].clone();
        let allow_mismatch = self.context.triple.is_64_bit()
            && name == ".eh_frame"
            && section_type == ELF_SECTION_PROGRAM_BITS;
        if !type_name.is_empty() && held.section_type != section_type && !allow_mismatch {
            self.error(
                location,
                &format!(
                    "changed section type for {name}, expected: 0x{:X}",
                    held.section_type
                ),
            );
        }
        if (extra_flags != 0 || size != 0 || !type_name.is_empty()) && held.flags != flags {
            self.error(
                location,
                &format!(
                    "changed section flags for {name}, expected: 0x{:X}",
                    held.flags
                ),
            );
        }
        if (extra_flags != 0 || size != 0 || !type_name.is_empty())
            && held.entry_size != size as u64
        {
            self.error(
                location,
                &format!(
                    "changed section entsize for {name}, expected: {}",
                    held.entry_size
                ),
            );
        }
        false
    }

    fn parse_section_flags(&self, text: &str, use_last_group: &mut bool) -> Option<u64> {
        if let Some(value) = parse_integer_text(text) {
            return Some(value as u64);
        }
        let mut flags = 0u64;
        for character in text.chars() {
            match character {
                'a' => flags |= ELF_FLAG_ALLOCATE,
                'e' => flags |= ELF_FLAG_EXCLUDE,
                'x' => flags |= ELF_FLAG_EXECUTE,
                'w' => flags |= ELF_FLAG_WRITE,
                'o' => flags |= ELF_FLAG_LINK_ORDER,
                'M' => flags |= ELF_FLAG_MERGE,
                'S' => flags |= ELF_FLAG_STRINGS,
                'T' => flags |= ELF_FLAG_THREAD_LOCAL,
                'G' => flags |= ELF_FLAG_GROUP,
                'l' => {
                    if !self.context.triple.is_64_bit() {
                        return None;
                    }
                    flags |= ELF_FLAG_X86_64_LARGE;
                }
                'R' => flags |= ELF_FLAG_GNU_RETAIN,
                '?' => *use_last_group = true,
                _ => return None,
            }
        }
        Some(flags)
    }

    fn parse_directive_size(&mut self) -> bool {
        let Some(symbol) = self.parse_symbol() else {
            return self.token_error("expected identifier");
        };
        if !self.is(TokenKind::Comma) {
            return self.token_error("expected comma");
        }
        self.lex();
        let Ok((value, _)) = self.parse_expression() else {
            return true;
        };
        if !self.is(TokenKind::EndOfStatement) {
            return self.token_error("unexpected token");
        }
        self.lex();
        self.streamer
            .emit_elf_size(&mut self.context, symbol, &value);
        false
    }

    fn parse_directive_type(&mut self) -> bool {
        let Some(symbol) = self.parse_symbol() else {
            return self.token_error("expected identifier");
        };
        if self.is(TokenKind::Comma) {
            self.lex();
        }
        if !self.is(TokenKind::Identifier)
            && !self.is(TokenKind::Hash)
            && !self.is(TokenKind::Percent)
            && !self.is(TokenKind::String)
            && !self.is(TokenKind::At)
        {
            return self.token_error(
                "expected STT_<TYPE_IN_UPPER_CASE>, '#<type>', '@<type>', '%<type>' or \"<type>\"",
            );
        }
        if !self.is(TokenKind::String) && !self.is(TokenKind::Identifier) {
            self.lex();
        }
        let type_location = self.location();
        let Some(type_name) = self.parse_identifier() else {
            return self.token_error("expected symbol type");
        };
        let attribute = match type_name.as_str() {
            "STT_FUNC" | "function" => SymbolAttribute::TypeFunction,
            "STT_OBJECT" | "object" => SymbolAttribute::TypeObject,
            "STT_TLS" | "tls_object" => SymbolAttribute::ThreadLocal,
            "STT_COMMON" | "common" => SymbolAttribute::TypeCommon,
            "STT_NOTYPE" | "notype" => SymbolAttribute::TypeNoType,
            "STT_GNU_IFUNC" | "gnu_indirect_function" => SymbolAttribute::TypeIndirectFunction,
            "gnu_unique_object" => SymbolAttribute::TypeGnuUniqueObject,
            _ => return self.error(type_location, "unsupported attribute"),
        };
        if !self.is(TokenKind::EndOfStatement) {
            return self.token_error("expected end of directive");
        }
        self.lex();
        self.streamer
            .emit_symbol_attribute(&mut self.context, symbol, attribute);
        false
    }

    fn parse_directive_symbol_version(&mut self) -> bool {
        let Some(symbol) = self.parse_symbol() else {
            return self.token_error("expected identifier");
        };
        if !self.is(TokenKind::Comma) {
            return self.token_error("expected a comma");
        }
        self.lex();
        let Some(name) = self.parse_identifier() else {
            return self.token_error("expected identifier");
        };
        if !name.contains('@') {
            return self.token_error("expected a '@' in the name");
        }
        let mut keep = !name.contains("@@@");
        if self.parse_optional_token(TokenKind::Comma) {
            let action = self.parse_identifier();
            if action.as_deref() != Some("remove") {
                return self.token_error("expected 'remove'");
            }
            keep = false;
        }
        self.parse_optional_token(TokenKind::EndOfStatement);
        self.streamer
            .emit_symbol_version(&mut self.context, symbol, &name, keep);
        false
    }

    fn parse_coff_directive(
        &mut self,
        identifier: &str,
        _location: SourceLocation,
    ) -> Option<bool> {
        let result = match identifier {
            ".text" | ".data" | ".bss" => {
                if !self.is(TokenKind::EndOfStatement) {
                    return Some(self.token_error("unexpected token in directive"));
                }
                self.lex();
                let (characteristics, name) = match identifier {
                    ".text" => (0x6000_0020u32, ".text"),
                    ".data" => (0xc000_0040u32, ".data"),
                    _ => (0xc000_0080u32, ".bss"),
                };
                let section = self.coff_section(name, characteristics, "", 0);
                self.switch_section(section, 0);
                false
            }
            ".section" => self.parse_coff_section(),
            ".def" | ".scl" | ".type" | ".endef" | ".secrel32" | ".secidx" | ".safeseh"
            | ".symidx" | ".linkonce" | ".rva" | ".weak" | ".seh_proc" | ".seh_endproc"
            | ".seh_endprologue" | ".seh_stackalloc" | ".seh_handler" | ".seh_handlerdata"
            | ".seh_endfunclet" | ".seh_startepilogue" | ".seh_endepilogue" | ".secnum"
            | ".secoffset" | ".seh_pushreg" | ".seh_setframe" | ".seh_savereg" | ".seh_savexmm"
            | ".seh_pushframe" => {
                let text = self.parse_string_to_end_of_statement();
                let line = if text.trim().is_empty() {
                    format!("\t{identifier}")
                } else {
                    format!("\t{identifier}\t{}", text.trim())
                };
                self.streamer
                    .emit_coff_directive(&mut self.context, &line, None, 0);
                self.parse_end_of_line()
            }
            _ => return None,
        };
        Some(result)
    }

    fn coff_section(
        &mut self,
        name: &str,
        characteristics: u32,
        comdat_symbol: &str,
        selection: u8,
    ) -> usize {
        let key = (name.to_string(), comdat_symbol.to_string(), u64::MAX);
        if let Some(found) = self.context.section_index.get(&key) {
            return *found;
        }
        let mut section = Section::new(name, 0, 0);
        section.characteristics = characteristics;
        section.coff_selection = selection;
        section.is_virtual = characteristics & 0x80 != 0;
        if !comdat_symbol.is_empty() {
            section.group = Some(comdat_symbol.to_string());
        }
        let index = self.context.sections.len();
        self.context.sections.push(section);
        self.context.section_index.insert(key, index);
        index
    }

    fn parse_coff_section(&mut self) -> bool {
        let Some(name) = self.parse_identifier() else {
            return self.token_error("expected identifier in directive");
        };
        let mut flags_text = String::new();
        if self.is(TokenKind::Comma) {
            self.lex();
            if !self.is(TokenKind::String) {
                return self.token_error("expected string in directive");
            }
            flags_text = string_contents(&self.lexer.token.text);
            self.lex();
        }
        let mut characteristics: u32 = if flags_text.is_empty() {
            if name.starts_with(".text") {
                0x6000_0020
            } else if name.starts_with(".bss") {
                0xc000_0080
            } else {
                0xc000_0040
            }
        } else {
            let mut value = 0u32;
            let mut readable = true;
            let mut writable = false;
            let mut code = false;
            let mut initialized = false;
            let mut uninitialized = false;
            let mut discardable = false;
            let mut shared = false;
            let mut remove = false;
            let mut no_load = false;
            let mut no_read = false;
            for character in flags_text.chars() {
                match character {
                    'a' => {}
                    'b' => {
                        uninitialized = true;
                        initialized = false;
                    }
                    'd' => {
                        initialized = true;
                        writable = true;
                    }
                    'D' => discardable = true,
                    'i' => remove = true,
                    'n' => no_load = true,
                    'r' => {
                        writable = false;
                        readable = true;
                        initialized = initialized || !uninitialized;
                    }
                    's' => shared = true,
                    'w' => {
                        writable = true;
                        initialized = initialized || !uninitialized;
                    }
                    'x' => {
                        code = true;
                        initialized = false;
                    }
                    'y' => no_read = true,
                    _ => return self.token_error("unknown flag"),
                }
            }
            if code {
                value |= 0x20 | 0x2000_0000;
            }
            if initialized {
                value |= 0x40;
            }
            if uninitialized {
                value |= 0x80;
            }
            if discardable {
                value |= 0x0200_0000;
            }
            if remove {
                value |= 0x800;
            }
            if no_load {
                value |= 0x2;
            }
            if shared {
                value |= 0x1000_0000;
            }
            if readable && !no_read {
                value |= 0x4000_0000;
            }
            if writable {
                value |= 0x8000_0000;
            }
            value
        };
        let mut comdat_symbol = String::new();
        let mut selection: u8 = 0;
        if self.is(TokenKind::Comma) {
            self.lex();
            let Some(kind) = self.parse_identifier() else {
                return self.token_error(
                    "expected comdat type such as 'discard' or 'largest' after protection bits",
                );
            };
            selection = match kind.as_str() {
                "one_only" => 1,
                "discard" => 2,
                "same_size" => 3,
                "same_contents" => 4,
                "associative" => 5,
                "largest" => 6,
                "newest" => 7,
                _ => return self.token_error("unrecognized COMDAT type"),
            };
            if !self.is(TokenKind::Comma) {
                return self.token_error("expected comma in directive");
            }
            self.lex();
            let Some(symbol) = self.parse_identifier() else {
                return self.token_error("expected identifier in directive");
            };
            comdat_symbol = symbol;
            characteristics |= 0x1000;
        }
        if !self.is(TokenKind::EndOfStatement) {
            return self.token_error("unexpected token in directive");
        }
        self.lex();
        let section = self.coff_section(&name, characteristics, &comdat_symbol, selection);
        self.switch_section(section, 0);
        false
    }

    fn parse_target_directive(
        &mut self,
        identifier: &str,
        location: SourceLocation,
    ) -> Option<bool> {
        if identifier.starts_with(".arch") {
            self.parse_string_to_end_of_statement();
            return Some(false);
        }
        if identifier.starts_with(".code") {
            self.code_16_gcc = false;
            let new_mode = match identifier {
                ".code16" => Mode::Bits16,
                ".code16gcc" => {
                    self.code_16_gcc = true;
                    Mode::Bits16
                }
                ".code32" => Mode::Bits32,
                ".code64" => Mode::Bits64,
                _ => {
                    self.error(location, &format!("unknown directive {identifier}"));
                    return Some(false);
                }
            };
            self.lex();
            if self.mode != new_mode {
                self.mode = new_mode;
                self.streamer.emit_mode_change(&mut self.context, new_mode);
            }
            return Some(false);
        }
        if identifier.starts_with(".att_syntax") {
            if !self.is(TokenKind::EndOfStatement) {
                if self.lexer.token.text == "prefix" {
                    self.lex();
                } else if self.lexer.token.text == "noprefix" {
                    return Some(self.error(location, "'.att_syntax noprefix' is not supported: registers must have a '%' prefix in .att_syntax"));
                }
            }
            self.dialect = Syntax::Att;
            return Some(false);
        }
        if identifier.starts_with(".intel_syntax") {
            self.dialect = Syntax::Intel;
            if !self.is(TokenKind::EndOfStatement) {
                if self.lexer.token.text == "noprefix" {
                    self.lex();
                } else if self.lexer.token.text == "prefix" {
                    return Some(self.error(location, "'.intel_syntax prefix' is not supported: registers must not have a '%' prefix in .intel_syntax"));
                }
            }
            return Some(false);
        }
        if identifier == ".nops" {
            let count_location = self.location();
            if self.check_for_valid_section() {
                return Some(true);
            }
            let Ok(count) = self.parse_absolute_expression() else {
                return Some(true);
            };
            let mut control = 0;
            let mut control_location = count_location;
            if self.parse_optional_token(TokenKind::Comma) {
                control_location = self.location();
                let Ok(parsed) = self.parse_absolute_expression() else {
                    return Some(true);
                };
                control = parsed;
            }
            if self.parse_end_of_line() {
                return Some(true);
            }
            if count <= 0 {
                self.error(count_location, "'.nops' directive with non-positive size");
                return Some(false);
            }
            if control < 0 {
                self.error(control_location, "'.nops' directive with negative NOP size");
                return Some(false);
            }
            let mode = self.mode;
            self.streamer
                .emit_nops(&mut self.context, count, control, mode, location);
            return Some(false);
        }
        if identifier == ".even" {
            if self.parse_end_of_line() {
                return Some(false);
            }
            if self.context.current_section().is_none() {
                self.streamer.initialize_sections(&mut self.context);
            }
            if self.current_section_is_code() {
                let mode = self.mode;
                self.streamer
                    .emit_code_alignment(&mut self.context, 2, 0, mode);
            } else {
                self.streamer
                    .emit_value_to_alignment(&mut self.context, 2, 0, 1, 0);
            }
            return Some(false);
        }
        None
    }

    fn parse_directive_macro(&mut self, location: SourceLocation) -> bool {
        let Some(name) = self.parse_identifier() else {
            return self.token_error("expected identifier in '.macro' directive");
        };
        if self.is(TokenKind::Comma) {
            self.lex();
        }
        let mut parameters: Vec<MacroParameter> = Vec::new();
        while !self.is(TokenKind::EndOfStatement) {
            if parameters
                .last()
                .is_some_and(|parameter| parameter.variadic)
            {
                let previous = parameters
                    .last()
                    .map(|parameter| parameter.name.clone())
                    .unwrap_or_default();
                let at = self.location();
                return self.error(
                    at,
                    &format!("vararg parameter '{previous}' should be the last parameter"),
                );
            }
            let Some(parameter_name) = self.parse_identifier() else {
                return self.token_error("expected identifier in '.macro' directive");
            };
            if parameters
                .iter()
                .any(|parameter| parameter.name == parameter_name)
            {
                return self.token_error(&format!(
                    "macro '{name}' has multiple parameters named '{parameter_name}'"
                ));
            }
            let mut parameter = MacroParameter {
                name: parameter_name.clone(),
                value: Vec::new(),
                required: false,
                variadic: false,
            };
            if self.is(TokenKind::Colon) {
                self.lex();
                let qualifier_location = self.location();
                let Some(qualifier) = self.parse_identifier() else {
                    return self.error(
                        qualifier_location,
                        &format!(
                            "missing parameter qualifier for '{parameter_name}' in macro '{name}'"
                        ),
                    );
                };
                match qualifier.as_str() {
                    "req" => parameter.required = true,
                    "vararg" => parameter.variadic = true,
                    _ => {
                        return self.error(qualifier_location, &format!("{qualifier} is not a valid parameter qualifier for '{parameter_name}' in macro '{name}'"));
                    }
                }
            }
            if self.is(TokenKind::Equal) {
                self.lex();
                let value_location = self.location();
                let Ok(value) = self.parse_macro_argument(false) else {
                    return true;
                };
                parameter.value = value;
                if parameter.required {
                    self.warning(value_location, &format!("pointless default value for required parameter '{parameter_name}' in macro '{name}'"));
                }
            }
            parameters.push(parameter);
            if self.is(TokenKind::Comma) {
                self.lex();
            }
        }
        self.lexer.lex();
        let start = self.lexer.token.clone();
        let end_token: Option<Token>;
        let mut depth = 0u32;
        loop {
            while self.is(TokenKind::Error) {
                self.lexer.lex();
            }
            if self.is(TokenKind::EndOfFile) {
                return self.error(location, "no matching '.endmacro' in definition");
            }
            if self.is(TokenKind::Identifier) {
                let text = self.lexer.token.text.clone();
                if text == ".endm" || text == ".endmacro" {
                    if depth == 0 {
                        let token = self.lexer.token.clone();
                        self.lexer.lex();
                        if !self.is(TokenKind::EndOfStatement) {
                            return self.token_error(&format!(
                                "unexpected token in '{}' directive",
                                token.text
                            ));
                        }
                        end_token = Some(token);
                        break;
                    }
                    depth -= 1;
                } else if text == ".macro" {
                    depth += 1;
                }
            }
            self.eat_to_end_of_statement();
        }
        if self.macros.contains_key(&name) {
            return self.error(location, &format!("macro '{name}' is already defined"));
        }
        let end = end_token
            .map(|token| token.location())
            .unwrap_or(start.location());
        let body = self.lexer.text_between(start.location(), end);
        self.check_for_bad_macro(location, &body, &parameters);
        self.macros.insert(
            name.clone(),
            MacroDefinition {
                name,
                body,
                parameters,
                count: 0,
            },
        );
        false
    }

    fn check_for_bad_macro(
        &mut self,
        location: SourceLocation,
        body: &str,
        parameters: &[MacroParameter],
    ) {
        if parameters.is_empty() {
            return;
        }
        let bytes = body.as_bytes();
        let mut named = false;
        let mut positional = false;
        let mut at = 0usize;
        while at < bytes.len() {
            let mut position = at;
            while position < bytes.len() {
                if bytes[position] == b'\\' && position + 1 < bytes.len() {
                    break;
                }
                if bytes[position] == b'$' && position + 1 < bytes.len() {
                    let next = bytes[position + 1];
                    if next == b'$' || next == b'n' || next.is_ascii_digit() {
                        break;
                    }
                }
                position += 1;
            }
            if position >= bytes.len() {
                break;
            }
            if bytes[position] == b'$' {
                if bytes[position + 1] != b'$' {
                    positional = true;
                }
                at = position + 2;
            } else {
                let mut end = position + 1;
                while end < bytes.len() && is_macro_argument_character(bytes[end]) {
                    end += 1;
                }
                let argument = &body[position + 1..end];
                if parameters
                    .iter()
                    .any(|parameter| parameter.name == argument)
                {
                    named = true;
                    at = position + 1 + argument.len();
                } else if bytes.get(position + 1) == Some(&b'(')
                    && bytes.get(position + 2) == Some(&b')')
                {
                    at = position + 3;
                } else {
                    at = end;
                }
            }
        }
        if !named && positional {
            self.warning(location, "macro defined with named parameters which are not used in macro body, possible positional parameter found in body which will have no effect");
        }
    }

    fn parse_macro_argument(&mut self, variadic: bool) -> Result<Vec<Token>, ()> {
        let mut argument: Vec<Token> = Vec::new();
        if variadic {
            if !self.is(TokenKind::EndOfStatement) {
                let start = self.location();
                let text = self.parse_string_to_end_of_statement();
                argument.push(Token {
                    kind: TokenKind::String,
                    text,
                    buffer: start.buffer,
                    start: start.position,
                    end: start.position,
                    value: 0,
                });
            }
            return Ok(argument);
        }
        let mut parenthesis = 0u32;
        self.lexer.skip_space = false;
        let result = loop {
            let mut space_eaten = false;
            if self.is(TokenKind::EndOfFile) || self.is(TokenKind::Equal) {
                self.token_error("unexpected token in macro instantiation");
                break Err(());
            }
            if parenthesis == 0 {
                if self.is(TokenKind::Comma) {
                    break Ok(());
                }
                if self.is(TokenKind::Space) {
                    self.lexer.lex();
                    space_eaten = true;
                }
                if is_operator_token(self.lexer.token.kind) {
                    argument.push(self.lexer.token.clone());
                    self.lexer.lex();
                    if self.is(TokenKind::Space) {
                        self.lexer.lex();
                    }
                    continue;
                }
                if space_eaten {
                    break Ok(());
                }
            }
            if self.is(TokenKind::EndOfStatement) {
                break Ok(());
            }
            if self.is(TokenKind::LeftParenthesis) {
                parenthesis += 1;
            } else if self.is(TokenKind::RightParenthesis) && parenthesis > 0 {
                parenthesis -= 1;
            }
            argument.push(self.lexer.token.clone());
            self.lexer.lex();
        };
        self.lexer.skip_space = true;
        if self.is(TokenKind::Space) {
            self.lexer.lex();
        }
        result?;
        if parenthesis != 0 {
            self.token_error("unbalanced parentheses in macro argument");
            return Err(());
        }
        Ok(argument)
    }

    fn parse_macro_arguments(
        &mut self,
        definition: Option<&MacroDefinition>,
    ) -> Result<Vec<Vec<Token>>, ()> {
        let count = definition.map(|held| held.parameters.len()).unwrap_or(0);
        let mut arguments: Vec<Vec<Token>> = vec![Vec::new(); count];
        let mut argument_locations: Vec<Option<SourceLocation>> = vec![None; count];
        let has_variadic = definition.is_some_and(|held| {
            held.parameters
                .last()
                .is_some_and(|parameter| parameter.variadic)
        });
        let mut named_found = false;
        let mut parameter = 0usize;
        loop {
            if count != 0 && parameter >= count {
                break;
            }
            let identifier_location = self.location();
            let mut formal_name = String::new();
            if self.is(TokenKind::Identifier)
                && self
                    .lexer
                    .peek(1)
                    .first()
                    .is_some_and(|token| token.kind == TokenKind::Equal)
            {
                match self.parse_identifier() {
                    Some(found) => formal_name = found,
                    None => {
                        self.error(
                            identifier_location,
                            "invalid argument identifier for formal argument",
                        );
                        return Err(());
                    }
                }
                if !self.is(TokenKind::Equal) {
                    self.token_error("expected '=' after formal parameter identifier");
                    return Err(());
                }
                self.lex();
                named_found = true;
            }
            let variadic = has_variadic && parameter == count.saturating_sub(1);
            if named_found && formal_name.is_empty() {
                self.error(
                    identifier_location,
                    "cannot mix positional and keyword arguments",
                );
                return Err(());
            }
            let string_location = self.location();
            let value: Vec<Token> = if self.alternate_macro_mode && self.is(TokenKind::Percent) {
                self.lex();
                let Ok((expression, end)) = self.parse_expression() else {
                    return Ok(arguments);
                };
                let Some(evaluated) = self.context.evaluate_absolute(&expression, None) else {
                    self.error(string_location, "expected absolute expression");
                    return Err(());
                };
                let text = self.lexer.text_between(string_location, end);
                vec![Token {
                    kind: TokenKind::Integer,
                    text,
                    buffer: string_location.buffer,
                    start: string_location.position,
                    end: end.position,
                    value: evaluated as u64 as u128,
                }]
            } else {
                self.parse_macro_argument(variadic)?
            };
            let mut index = parameter;
            if !formal_name.is_empty() {
                let Some(found) = definition.and_then(|held| {
                    held.parameters
                        .iter()
                        .position(|candidate| candidate.name == formal_name)
                }) else {
                    let macro_name = definition.map(|held| held.name.clone()).unwrap_or_default();
                    self.error(identifier_location, &format!("parameter named '{formal_name}' does not exist for macro '{macro_name}'"));
                    return Err(());
                };
                index = found;
            }
            if !value.is_empty() {
                if arguments.len() <= index {
                    arguments.resize(index + 1, Vec::new());
                }
                arguments[index] = value;
                if argument_locations.len() <= index {
                    argument_locations.resize(index + 1, None);
                }
                argument_locations[index] = Some(self.location());
            }
            if self.is(TokenKind::EndOfStatement) {
                let mut failure = false;
                if let Some(held) = definition {
                    for (position, formal) in held.parameters.iter().enumerate() {
                        if arguments[position].is_empty() {
                            if formal.required {
                                let at = argument_locations[position].unwrap_or(self.location());
                                self.error(
                                    at,
                                    &format!(
                                        "missing value for required parameter '{}' in macro '{}'",
                                        formal.name, held.name
                                    ),
                                );
                                failure = true;
                            }
                            if !formal.value.is_empty() {
                                arguments[position] = formal.value.clone();
                            }
                        }
                    }
                }
                if failure {
                    return Err(());
                }
                return Ok(arguments);
            }
            self.parse_optional_token(TokenKind::Comma);
            parameter += 1;
        }
        self.token_error("too many positional arguments");
        Err(())
    }

    fn expand_macro(
        &self,
        definition: &MacroDefinition,
        parameters: &[MacroParameter],
        arguments: &[Vec<Token>],
        enable_at: bool,
    ) -> String {
        let mut out = String::new();
        let count = parameters.len();
        let expand_argument = |out: &mut String, index: usize| {
            let has_variadic = parameters
                .last()
                .is_some_and(|parameter| parameter.variadic);
            let variadic_parameter = has_variadic && index == count - 1;
            if let Some(tokens) = arguments.get(index) {
                for token in tokens {
                    if self.alternate_macro_mode
                        && token.text.starts_with('%')
                        && token.kind == TokenKind::Integer
                    {
                        out.push_str(&token.integer().to_string());
                    } else if token.kind != TokenKind::String || variadic_parameter {
                        out.push_str(&token.text);
                    } else {
                        out.push_str(&string_contents(&token.text));
                    }
                }
            }
        };
        let body = definition.body.as_bytes();
        let mut at = 0usize;
        while at < body.len() {
            if body[at] == b'\\' && at + 1 < body.len() {
                if enable_at && body[at + 1] == b'@' {
                    out.push_str(&self.macro_instantiations.to_string());
                    at += 2;
                    continue;
                }
                if body[at + 1] == b'+' {
                    out.push_str(&definition.count.to_string());
                    at += 2;
                    continue;
                }
                if body[at + 1] == b'(' && at + 2 < body.len() && body[at + 2] == b')' {
                    at += 3;
                    continue;
                }
                at += 1;
                let start = at;
                while at < body.len() && is_macro_argument_character(body[at]) {
                    at += 1;
                }
                let argument = &definition.body[start..at];
                if self.alternate_macro_mode && at < body.len() && body[at] == b'&' {
                    at += 1;
                }
                match parameters
                    .iter()
                    .position(|parameter| parameter.name == argument)
                {
                    Some(index) => expand_argument(&mut out, index),
                    None => {
                        out.push('\\');
                        out.push_str(argument);
                    }
                }
                continue;
            }
            if !is_macro_argument_character(body[at]) {
                let character_length = definition.body[at..]
                    .chars()
                    .next()
                    .map(char::len_utf8)
                    .unwrap_or(1);
                out.push_str(&definition.body[at..at + character_length]);
                at += character_length;
                continue;
            }
            let start = at;
            at += 1;
            while at < body.len() && is_macro_argument_character(body[at]) {
                at += 1;
            }
            let token = &definition.body[start..at];
            if self.alternate_macro_mode {
                if let Some(index) = parameters
                    .iter()
                    .position(|parameter| parameter.name == token)
                {
                    expand_argument(&mut out, index);
                    if at < body.len() && body[at] == b'&' {
                        at += 1;
                    }
                    continue;
                }
            }
            out.push_str(token);
        }
        out
    }

    fn handle_macro_entry(
        &mut self,
        definition: MacroDefinition,
        location: SourceLocation,
    ) -> bool {
        if self.active_macros.len() == 20 {
            return self.token_error("macros cannot be nested more than 20 levels deep. Use -asm-macro-max-nesting-depth to increase this limit.");
        }
        let Ok(arguments) = self.parse_macro_arguments(Some(&definition)) else {
            return true;
        };
        if definition.parameters.len() != arguments.len() {
            let at = self.location();
            return self.error(at, "Wrong number of arguments");
        }
        let mut text = self.expand_macro(&definition, &definition.parameters, &arguments, true);
        if let Some(held) = self.macros.get_mut(&definition.name) {
            held.count += 1;
        }
        text.push_str(".endmacro\n");
        let exit = self.location();
        self.active_macros.push(MacroInstantiation {
            instantiation: location,
            exit,
            condition_depth: self.conditions.len(),
        });
        self.macro_instantiations += 1;
        let buffer = self
            .lexer
            .add_buffer("<instantiation>", text.into_bytes(), None);
        self.lexer.jump_to(SourceLocation {
            buffer,
            position: 0,
        });
        self.lexer.state.at_start_of_line = true;
        self.lexer.state.at_start_of_statement = true;
        self.lex();
        false
    }

    fn handle_macro_exit(&mut self) {
        let Some(active) = self.active_macros.pop() else {
            return;
        };
        self.lexer.jump_to(active.exit);
        self.lex();
        if self.is(TokenKind::EndOfStatement) {
            self.lex();
        }
    }

    fn parse_macro_like_body(&mut self, location: SourceLocation) -> Option<MacroDefinition> {
        let start = self.lexer.token.clone();
        let mut level = 0u32;
        let end: Token;
        loop {
            if self.is(TokenKind::EndOfFile) {
                self.print_error(location, "no matching '.endr' in definition");
                return None;
            }
            if self.is(TokenKind::Identifier) {
                let text = self.lexer.token.identifier();
                if matches!(text.as_str(), ".rep" | ".rept" | ".irp" | ".irpc") {
                    level += 1;
                } else if text == ".endr" {
                    if level == 0 {
                        end = self.lexer.token.clone();
                        self.lex();
                        if self.is(TokenKind::EndOfStatement) {
                            break;
                        }
                        let at = self.location();
                        self.print_error(at, "expected newline");
                        return None;
                    }
                    level -= 1;
                }
            }
            self.eat_to_end_of_statement();
        }
        let body = self.lexer.text_between(start.location(), end.location());
        Some(MacroDefinition {
            name: String::new(),
            body,
            parameters: Vec::new(),
            count: 0,
        })
    }

    fn instantiate_macro_like_body(&mut self, location: SourceLocation, mut text: String) {
        text.push_str(".endr\n");
        let exit = self.location();
        self.active_macros.push(MacroInstantiation {
            instantiation: location,
            exit,
            condition_depth: self.conditions.len(),
        });
        let buffer = self
            .lexer
            .add_buffer("<instantiation>", text.into_bytes(), None);
        self.lexer.jump_to(SourceLocation {
            buffer,
            position: 0,
        });
        self.lexer.state.at_start_of_line = true;
        self.lexer.state.at_start_of_statement = true;
        self.lex();
    }

    fn parse_directive_repeat(&mut self, location: SourceLocation, directive: &str) -> bool {
        let count_location = self.location();
        let Ok((expression, _)) = self.parse_expression() else {
            return true;
        };
        let Some(count) = self.context.evaluate_absolute(&expression, None) else {
            return self.error(
                count_location,
                &format!("unexpected token in '{directive}' directive"),
            );
        };
        if self.check_at(count < 0, count_location, "Count is negative") || self.parse_end_of_line()
        {
            return true;
        }
        let Some(mut definition) = self.parse_macro_like_body(location) else {
            return true;
        };
        let mut text = String::new();
        for _ in 0..count {
            text.push_str(&self.expand_macro(&definition, &[], &[], false));
            definition.count += 1;
        }
        self.instantiate_macro_like_body(location, text);
        false
    }

    fn parse_directive_iterate(&mut self, location: SourceLocation) -> bool {
        let Some(name) = self.parse_identifier() else {
            return self.token_error("expected identifier in '.irp' directive");
        };
        if self.parse_comma() {
            return true;
        }
        let Ok(arguments) = self.parse_macro_arguments(None) else {
            return true;
        };
        if self.parse_end_of_line() {
            return true;
        }
        let Some(mut definition) = self.parse_macro_like_body(location) else {
            return true;
        };
        let parameter = MacroParameter {
            name,
            value: Vec::new(),
            required: false,
            variadic: false,
        };
        let mut text = String::new();
        for argument in &arguments {
            text.push_str(&self.expand_macro(
                &definition,
                std::slice::from_ref(&parameter),
                std::slice::from_ref(argument),
                true,
            ));
            definition.count += 1;
        }
        self.instantiate_macro_like_body(location, text);
        false
    }

    fn parse_directive_iterate_characters(&mut self, location: SourceLocation) -> bool {
        let Some(name) = self.parse_identifier() else {
            return self.token_error("expected identifier in '.irpc' directive");
        };
        if self.parse_comma() {
            return true;
        }
        let Ok(arguments) = self.parse_macro_arguments(None) else {
            return true;
        };
        if arguments.len() != 1 || arguments[0].len() != 1 {
            return self.token_error("unexpected token in '.irpc' directive");
        }
        if self.parse_end_of_line() {
            return true;
        }
        let Some(mut definition) = self.parse_macro_like_body(location) else {
            return true;
        };
        let token = &arguments[0][0];
        let values = if token.kind == TokenKind::String {
            string_contents(&token.text)
        } else {
            token.text.clone()
        };
        let parameter = MacroParameter {
            name,
            value: Vec::new(),
            required: false,
            variadic: false,
        };
        let mut text = String::new();
        for character in values.chars() {
            let argument = vec![Token {
                kind: TokenKind::Identifier,
                text: character.to_string(),
                buffer: 0,
                start: 0,
                end: 0,
                value: 0,
            }];
            text.push_str(&self.expand_macro(
                &definition,
                std::slice::from_ref(&parameter),
                std::slice::from_ref(&argument),
                true,
            ));
            definition.count += 1;
        }
        self.instantiate_macro_like_body(location, text);
        false
    }
}

pub fn parse_integer_text(text: &str) -> Option<i64> {
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let value = if let Some(hexadecimal) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        u64::from_str_radix(hexadecimal, 16).ok()?
    } else if let Some(binary) = digits
        .strip_prefix("0b")
        .or_else(|| digits.strip_prefix("0B"))
    {
        u64::from_str_radix(binary, 2).ok()?
    } else if digits.len() > 1 && digits.starts_with('0') {
        u64::from_str_radix(&digits[1..], 8).ok()?
    } else {
        digits.parse::<u64>().ok()?
    };
    Some(if negative {
        (value as i64).wrapping_neg()
    } else {
        value as i64
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InfixToken {
    Or,
    Xor,
    And,
    LeftShift,
    RightShift,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Not,
    Negate,
    RightParenthesis,
    LeftParenthesis,
    Immediate,
    Register,
}

impl InfixToken {
    fn precedence(self) -> u8 {
        match self {
            InfixToken::Or | InfixToken::Immediate | InfixToken::Register => 0,
            InfixToken::Xor => 1,
            InfixToken::And => 2,
            InfixToken::LeftShift | InfixToken::RightShift => 4,
            InfixToken::Plus | InfixToken::Minus => 5,
            InfixToken::Multiply | InfixToken::Divide | InfixToken::Modulo => 6,
            InfixToken::Not => 7,
            InfixToken::Negate => 8,
            InfixToken::RightParenthesis => 9,
            InfixToken::LeftParenthesis => 10,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct InfixCalculator {
    operators: Vec<InfixToken>,
    postfix: Vec<(InfixToken, i64)>,
}

impl InfixCalculator {
    fn pop_operand(&mut self) -> i64 {
        match self.postfix.pop() {
            Some((InfixToken::Immediate | InfixToken::Register, value)) => value,
            _ => -1,
        }
    }

    fn push_operand(&mut self, token: InfixToken, value: i64) {
        self.postfix.push((token, value));
    }

    fn push_operator(&mut self, operator: InfixToken) {
        let Some(&top) = self.operators.last() else {
            self.operators.push(operator);
            return;
        };
        if operator.precedence() > top.precedence() || top == InfixToken::LeftParenthesis {
            self.operators.push(operator);
            return;
        }
        let mut parentheses = 0u32;
        while let Some(&top) = self.operators.last() {
            if !(top.precedence() >= operator.precedence() || parentheses != 0) {
                break;
            }
            if parentheses == 0 && top == InfixToken::LeftParenthesis {
                break;
            }
            self.operators.pop();
            match top {
                InfixToken::RightParenthesis => parentheses += 1,
                InfixToken::LeftParenthesis => parentheses -= 1,
                _ => self.postfix.push((top, 0)),
            }
        }
        self.operators.push(operator);
    }

    fn execute(&mut self) -> i64 {
        while let Some(top) = self.operators.pop() {
            if top != InfixToken::LeftParenthesis && top != InfixToken::RightParenthesis {
                self.postfix.push((top, 0));
            }
        }
        let mut stack: Vec<i64> = Vec::new();
        for &(token, value) in &self.postfix {
            match token {
                InfixToken::Immediate | InfixToken::Register => stack.push(value),
                InfixToken::Negate | InfixToken::Not => {
                    let operand = stack.pop().unwrap_or(0);
                    stack.push(if token == InfixToken::Negate {
                        operand.wrapping_neg()
                    } else {
                        !operand
                    });
                }
                _ => {
                    let right = stack.pop().unwrap_or(0);
                    let left = stack.pop().unwrap_or(0);
                    stack.push(match token {
                        InfixToken::Plus => left.wrapping_add(right),
                        InfixToken::Minus => left.wrapping_sub(right),
                        InfixToken::Multiply => left.wrapping_mul(right),
                        InfixToken::Divide => left.checked_div(right).unwrap_or(0),
                        InfixToken::Modulo => left.checked_rem(right).unwrap_or(0),
                        InfixToken::Or => left | right,
                        InfixToken::Xor => left ^ right,
                        InfixToken::And => left & right,
                        InfixToken::LeftShift => left.wrapping_shl(right as u32),
                        _ => left.wrapping_shr(right as u32),
                    });
                }
            }
        }
        stack.pop().unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IntelState {
    Initial,
    Or,
    Xor,
    And,
    LeftShift,
    RightShift,
    Plus,
    Minus,
    Offset,
    Not,
    Multiply,
    Divide,
    Modulo,
    LeftBracket,
    RightBracket,
    LeftParenthesis,
    RightParenthesis,
    Register,
    Integer,
    Error,
}

fn check_scale(scale: u32) -> Result<(), &'static str> {
    if matches!(scale, 1 | 2 | 4 | 8) {
        Ok(())
    } else {
        Err("scale factor in address must be 1, 2, 4 or 8")
    }
}

struct IntelExpression {
    state: IntelState,
    previous: IntelState,
    base: RegisterId,
    index: RegisterId,
    pending_register: RegisterId,
    scale: u32,
    pending_scale: Option<u32>,
    immediate: i64,
    symbol: Option<Expression>,
    calculator: InfixCalculator,
    bracket_count: i32,
    parenthesis_count: i32,
    left_parenthesis: Option<SourceLocation>,
    memory_expression: bool,
    bracket_used: bool,
    negative_term: Option<SourceLocation>,
    offset_operator: bool,
    attach_to_operand_index: bool,
    position_independent: bool,
}

impl IntelExpression {
    fn new() -> IntelExpression {
        IntelExpression {
            state: IntelState::Initial,
            previous: IntelState::Error,
            base: NO_REGISTER,
            index: NO_REGISTER,
            pending_register: NO_REGISTER,
            scale: 0,
            pending_scale: None,
            immediate: 0,
            symbol: None,
            calculator: InfixCalculator::default(),
            bracket_count: 0,
            parenthesis_count: 0,
            left_parenthesis: None,
            memory_expression: false,
            bracket_used: false,
            negative_term: None,
            offset_operator: false,
            attach_to_operand_index: false,
            position_independent: false,
        }
    }

    fn value(&mut self) -> i64 {
        self.immediate.wrapping_add(self.calculator.execute())
    }

    fn is_valid_end(&self) -> bool {
        matches!(
            self.state,
            IntelState::RightBracket
                | IntelState::RightParenthesis
                | IntelState::Integer
                | IntelState::Register
                | IntelState::Offset
        )
    }

    fn error_location(&self, default: SourceLocation) -> SourceLocation {
        self.negative_term.unwrap_or(default)
    }

    fn registers_used_up(&self) -> &'static str {
        if self.position_independent && self.attach_to_operand_index {
            "Don't use 2 or more regs for mem offset in PIC model!"
        } else {
            "BaseReg/IndexReg already set!"
        }
    }

    fn set_symbol(&mut self, value: Expression) -> Result<(), &'static str> {
        if self.symbol.is_some() {
            return Err("cannot use more than one symbol in memory operand");
        }
        self.symbol = Some(value);
        Ok(())
    }

    fn binary(&mut self, next: IntelState, operator: InfixToken) {
        let current = self.state;
        match self.state {
            IntelState::Integer | IntelState::RightParenthesis | IntelState::Register => {
                self.state = next;
                self.calculator.push_operator(operator);
            }
            _ => self.state = IntelState::Error,
        }
        self.previous = current;
    }

    fn settle_pending_register(&mut self, check_negative: bool) -> Result<(), &'static str> {
        if self.pending_register == NO_REGISTER {
            return Ok(());
        }
        if self.base == NO_REGISTER && self.pending_scale.is_none() {
            self.base = self.pending_register;
            self.pending_register = NO_REGISTER;
            return Ok(());
        }
        if self.index != NO_REGISTER {
            return Err(self.registers_used_up());
        }
        self.index = self.pending_register;
        self.pending_register = NO_REGISTER;
        if check_negative && self.negative_term.is_some() {
            return Err("Scale can't be negative");
        }
        if let Some(scale) = self.pending_scale {
            check_scale(scale)?;
        }
        self.scale = self.pending_scale.unwrap_or(0);
        Ok(())
    }

    fn on_plus(&mut self) -> Result<(), &'static str> {
        let current = self.state;
        match self.state {
            IntelState::Integer
            | IntelState::RightParenthesis
            | IntelState::Register
            | IntelState::Offset => {
                self.state = IntelState::Plus;
                self.calculator.push_operator(InfixToken::Plus);
                self.settle_pending_register(true)?;
            }
            _ => self.state = IntelState::Error,
        }
        self.negative_term = None;
        self.pending_scale = None;
        self.previous = current;
        Ok(())
    }

    fn on_minus(&mut self, location: SourceLocation) -> Result<(), &'static str> {
        let current = self.state;
        match self.state {
            IntelState::Minus | IntelState::Error => self.state = IntelState::Error,
            _ => {
                self.state = IntelState::Minus;
                self.negative_term = Some(location);
                if matches!(
                    current,
                    IntelState::Register
                        | IntelState::RightParenthesis
                        | IntelState::Integer
                        | IntelState::RightBracket
                        | IntelState::Offset
                ) {
                    self.calculator.push_operator(InfixToken::Minus);
                    self.settle_pending_register(false)?;
                } else if self.previous == IntelState::Register && current == IntelState::Multiply {
                    return Err("Scale can't be negative");
                } else {
                    self.calculator.push_operator(InfixToken::Negate);
                }
            }
        }
        self.pending_scale = None;
        self.previous = current;
        Ok(())
    }

    fn on_not(&mut self) {
        let current = self.state;
        match self.state {
            IntelState::Offset
            | IntelState::RightBracket
            | IntelState::RightParenthesis
            | IntelState::Register
            | IntelState::Integer
            | IntelState::Error => self.state = IntelState::Error,
            _ => {
                self.state = IntelState::Not;
                self.calculator.push_operator(InfixToken::Not);
            }
        }
        self.previous = current;
    }

    fn on_register(&mut self, register: RegisterId) -> Result<(), &'static str> {
        let current = self.state;
        match self.state {
            IntelState::Plus | IntelState::Minus | IntelState::LeftBracket => {
                self.state = IntelState::Register;
                self.pending_register = register;
                self.calculator.push_operand(InfixToken::Register, 0);
                if self.negative_term.is_some() {
                    return Err("Scale can't be negative");
                }
            }
            IntelState::LeftParenthesis | IntelState::Multiply => {
                if self.pending_register != NO_REGISTER {
                    return Err("Register can't be multiplied with register!");
                }
                self.state = IntelState::Register;
                self.pending_register = register;
                if self.pending_scale.is_some() {
                    if self.index != NO_REGISTER {
                        return Err(self.registers_used_up());
                    }
                    if self.negative_term.is_some() {
                        return Err("Scale can't be negative");
                    }
                    self.calculator.push_operand(InfixToken::Immediate, 0);
                } else {
                    self.calculator.push_operand(InfixToken::Register, 0);
                }
            }
            _ => self.state = IntelState::Error,
        }
        self.previous = current;
        Ok(())
    }

    fn on_identifier(&mut self, value: Expression) -> Result<(), &'static str> {
        if let Expression::Constant(constant) = value {
            return self.on_integer(constant);
        }
        self.previous = self.state;
        match self.state {
            IntelState::Plus
            | IntelState::Minus
            | IntelState::Not
            | IntelState::Initial
            | IntelState::LeftBracket
            | IntelState::LeftParenthesis => {
                self.set_symbol(value)?;
                self.pending_scale = Some(0);
                self.memory_expression = true;
                self.state = IntelState::Integer;
                self.calculator.push_operand(InfixToken::Immediate, 0);
            }
            _ => self.state = IntelState::Error,
        }
        Ok(())
    }

    fn on_integer(&mut self, value: i64) -> Result<(), &'static str> {
        let current = self.state;
        match self.state {
            IntelState::Divide
            | IntelState::Modulo
            | IntelState::Plus
            | IntelState::Minus
            | IntelState::Not
            | IntelState::Or
            | IntelState::Xor
            | IntelState::And
            | IntelState::LeftShift
            | IntelState::RightShift
            | IntelState::Multiply
            | IntelState::LeftParenthesis
            | IntelState::Initial
            | IntelState::LeftBracket => {
                if value == 0 && current == IntelState::Divide {
                    self.state = IntelState::Error;
                    return Err("division by zero in assembly expression");
                }
                if value == 0 && current == IntelState::Modulo {
                    self.state = IntelState::Error;
                    return Err("modulo by zero in assembly expression");
                }
                self.state = IntelState::Integer;
                let scale = match self.pending_scale {
                    Some(existing) => existing.wrapping_mul(value as u32),
                    None => value as u32,
                };
                self.pending_scale = Some(scale);
                if self.pending_register != NO_REGISTER && self.negative_term.is_some() {
                    return Err("Scale can't be negative");
                }
                if self.pending_register != NO_REGISTER {
                    check_scale(scale)?;
                }
                self.calculator.push_operand(InfixToken::Immediate, value);
            }
            _ => self.state = IntelState::Error,
        }
        self.previous = current;
        Ok(())
    }

    fn on_star(&mut self) {
        self.previous = self.state;
        match self.state {
            IntelState::Integer => {
                self.state = IntelState::Multiply;
                self.calculator.push_operator(InfixToken::Multiply);
            }
            IntelState::Register | IntelState::RightParenthesis => {
                if self.pending_register != NO_REGISTER && self.pending_scale.is_none() {
                    self.calculator.pop_operand();
                    self.calculator.push_operand(InfixToken::Immediate, 0);
                }
                self.state = IntelState::Multiply;
                self.calculator.push_operator(InfixToken::Multiply);
            }
            _ => self.state = IntelState::Error,
        }
    }

    fn on_divide(&mut self, next: IntelState, operator: InfixToken) {
        self.previous = self.state;
        match self.state {
            IntelState::Integer | IntelState::RightParenthesis => {
                self.state = next;
                self.calculator.push_operator(operator);
            }
            _ => self.state = IntelState::Error,
        }
    }

    fn on_left_bracket(&mut self) -> Result<(), ()> {
        if self.bracket_count != 0 {
            return Err(());
        }
        self.previous = self.state;
        match self.state {
            IntelState::RightBracket | IntelState::Integer | IntelState::RightParenthesis => {
                self.state = IntelState::Plus;
                self.calculator.push_operator(InfixToken::Plus);
            }
            IntelState::Initial => self.state = IntelState::LeftBracket,
            _ => self.state = IntelState::Error,
        }
        self.negative_term = None;
        self.pending_scale = None;
        self.memory_expression = true;
        self.bracket_used = true;
        self.bracket_count += 1;
        Ok(())
    }

    fn on_right_bracket(&mut self) -> Result<(), &'static str> {
        let current = self.state;
        match self.state {
            IntelState::Integer
            | IntelState::Offset
            | IntelState::Register
            | IntelState::RightParenthesis => {
                let count = self.bracket_count;
                self.bracket_count -= 1;
                if count != 1 {
                    return Err("unexpected bracket encountered");
                }
                self.state = IntelState::RightBracket;
                if self.pending_register != NO_REGISTER {
                    if self.base == NO_REGISTER && self.pending_scale.is_none() {
                        self.base = self.pending_register;
                        self.pending_register = NO_REGISTER;
                    } else if self.index == NO_REGISTER {
                        if self.negative_term.is_some() {
                            return Err("Scale can't be negative");
                        }
                        self.index = self.pending_register;
                        self.pending_register = NO_REGISTER;
                        if let Some(scale) = self.pending_scale {
                            check_scale(scale)?;
                        }
                        self.scale = self.pending_scale.unwrap_or(0);
                    } else {
                        return Err(self.registers_used_up());
                    }
                }
                self.negative_term = None;
            }
            _ => self.state = IntelState::Error,
        }
        self.pending_scale = None;
        self.previous = current;
        Ok(())
    }

    fn on_left_parenthesis(&mut self, location: SourceLocation) {
        let current = self.state;
        match self.state {
            IntelState::Offset
            | IntelState::RightBracket
            | IntelState::RightParenthesis
            | IntelState::Register
            | IntelState::Integer
            | IntelState::Error => self.state = IntelState::Error,
            _ => {
                self.parenthesis_count += 1;
                self.left_parenthesis = Some(location);
                self.state = IntelState::LeftParenthesis;
                self.calculator.push_operator(InfixToken::LeftParenthesis);
            }
        }
        self.previous = current;
    }

    fn on_right_parenthesis(&mut self) -> Result<(), &'static str> {
        let current = self.state;
        match self.state {
            IntelState::Integer
            | IntelState::Offset
            | IntelState::Register
            | IntelState::RightBracket
            | IntelState::RightParenthesis => {
                if self.parenthesis_count == 0 {
                    return Err("unmatched parenthesis");
                }
                self.parenthesis_count -= 1;
                self.state = IntelState::RightParenthesis;
                self.calculator.push_operator(InfixToken::RightParenthesis);
            }
            _ => self.state = IntelState::Error,
        }
        self.previous = current;
        Ok(())
    }

    fn on_offset(&mut self, value: Expression, text: &'static str) -> Result<(), &'static str> {
        self.previous = self.state;
        match self.state {
            IntelState::Plus | IntelState::Initial | IntelState::LeftBracket => {
                self.set_symbol(value)?;
                if text == "offset" {
                    self.offset_operator = true;
                }
                self.state = IntelState::Offset;
                self.calculator.push_operand(InfixToken::Immediate, 0);
                Ok(())
            }
            _ if text == "offset" => Err("unexpected offset operator expression"),
            _ => Err("unexpected imagerel operator expression"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InstructionState {
    pub forced_prefix: u32,
    pub forced_displacement: u32,
    pub uses_extended_registers: bool,
    pub forced_no_flags: bool,
}

const COMPARISON_PREDICATES: &[(&str, i64)] = &[
    ("eq", 0x00),
    ("eq_oq", 0x00),
    ("lt", 0x01),
    ("lt_os", 0x01),
    ("le", 0x02),
    ("le_os", 0x02),
    ("unord", 0x03),
    ("unord_q", 0x03),
    ("neq", 0x04),
    ("neq_uq", 0x04),
    ("nlt", 0x05),
    ("nlt_us", 0x05),
    ("nle", 0x06),
    ("nle_us", 0x06),
    ("ord", 0x07),
    ("ord_q", 0x07),
    ("eq_uq", 0x08),
    ("nge", 0x09),
    ("nge_us", 0x09),
    ("ngt", 0x0a),
    ("ngt_us", 0x0a),
    ("false", 0x0b),
    ("false_oq", 0x0b),
    ("neq_oq", 0x0c),
    ("ge", 0x0d),
    ("ge_os", 0x0d),
    ("gt", 0x0e),
    ("gt_os", 0x0e),
    ("true", 0x0f),
    ("true_uq", 0x0f),
    ("eq_os", 0x10),
    ("lt_oq", 0x11),
    ("le_oq", 0x12),
    ("unord_s", 0x13),
    ("neq_us", 0x14),
    ("nlt_uq", 0x15),
    ("nle_uq", 0x16),
    ("ord_s", 0x17),
    ("eq_us", 0x18),
    ("nge_uq", 0x19),
    ("ngt_uq", 0x1a),
    ("false_os", 0x1b),
    ("neq_os", 0x1c),
    ("ge_oq", 0x1d),
    ("gt_oq", 0x1e),
    ("true_us", 0x1f),
];

pub fn condition_code_from_text(text: &str) -> Option<i64> {
    Some(match text {
        "o" => 0,
        "no" => 1,
        "b" | "nae" => 2,
        "ae" | "nb" => 3,
        "e" | "z" => 4,
        "ne" | "nz" => 5,
        "be" | "na" => 6,
        "a" | "nbe" => 7,
        "s" => 8,
        "ns" => 9,
        "p" | "pe" => 10,
        "np" | "po" => 11,
        "l" | "nge" => 12,
        "ge" | "nl" => 13,
        "le" | "ng" => 14,
        "g" | "nle" => 15,
        _ => return None,
    })
}

impl<'s> Assembler<'s> {
    fn pointer_width(&self) -> u32 {
        self.mode.pointer_width()
    }

    pub fn match_register_name(&self, name: &str) -> Option<RegisterId> {
        let stripped = name.strip_prefix('%').unwrap_or(name);
        if let Some(found) = self.target.register_by_name.get(stripped) {
            return Some(*found);
        }
        let lower = stripped.to_ascii_lowercase();
        if let Some(found) = self.target.register_by_name.get(&lower) {
            return Some(*found);
        }
        None
    }

    fn register_by_name_checked(
        &mut self,
        name: &str,
        start: SourceLocation,
        end: SourceLocation,
        state: &mut InstructionState,
    ) -> Result<RegisterId, bool> {
        let stripped = name.strip_prefix('%').unwrap_or(name).to_string();
        let mut register = self.match_register_name(&stripped).unwrap_or(NO_REGISTER);
        if self.mode != Mode::Bits64 && register != NO_REGISTER {
            let record = self.target.registers[register as usize].record.clone();
            if record == "RIZ"
                || record == "RIP"
                || self.target.in_class("GR64", register)
                || self.target.is_low_byte_needing_rex(register)
                || self.target.is_extended(register)
            {
                self.error_range(
                    start,
                    &format!("register %{stripped} is only available in 64-bit mode"),
                    (start, end),
                );
                return Err(true);
            }
        }
        if register != NO_REGISTER && self.target.is_extended_apx(register) {
            state.uses_extended_registers = true;
        }
        if register == NO_REGISTER && stripped.starts_with("db") {
            let number = &stripped[2..];
            if let Ok(value) = number.parse::<u32>() {
                if value <= 15
                    && (number.len() == 1 || (number.len() == 2 && number.starts_with('1')))
                {
                    register = self.target.register(&format!("DR{value}"));
                }
            }
        }
        if register == NO_REGISTER {
            if self.dialect == Syntax::Intel {
                return Err(false);
            }
            self.error_range(start, "invalid register name", (start, end));
            return Err(true);
        }
        Ok(register)
    }

    pub fn parse_register_operand(
        &mut self,
        start: SourceLocation,
    ) -> Option<(RegisterId, SourceLocation)> {
        let mut state = InstructionState::default();
        let result = self.parse_register_with_state(start, &mut state);
        if state.uses_extended_registers {
            self.extended_register_seen = true;
        }
        result
    }

    fn parse_register_with_state(
        &mut self,
        start: SourceLocation,
        state: &mut InstructionState,
    ) -> Option<(RegisterId, SourceLocation)> {
        if self.dialect != Syntax::Intel && self.is(TokenKind::Percent) {
            self.lex();
        }
        let token = self.lexer.token.clone();
        let end = token.end_location();
        if token.kind != TokenKind::Identifier {
            if self.dialect != Syntax::Intel {
                self.error_range(start, "invalid register name", (start, end));
            }
            return None;
        }
        let register = match self.register_by_name_checked(&token.text, start, end, state) {
            Ok(register) => register,
            Err(_) => return None,
        };
        if self.target.registers[register as usize].record == "ST0" {
            self.lex();
            if !self.is(TokenKind::LeftParenthesis) {
                return Some((register, end));
            }
            self.lex();
            if !self.is(TokenKind::Integer) {
                self.token_error("expected stack index");
                return None;
            }
            let index = self.lexer.token.integer();
            if !(0..=7).contains(&index) {
                self.token_error("invalid stack index");
                return None;
            }
            let register = self.target.register(&format!("ST{index}"));
            self.lex();
            if !self.is(TokenKind::RightParenthesis) {
                self.token_error("expected ')'");
                return None;
            }
            let end = self.lexer.token.end_location();
            self.lex();
            return Some((register, end));
        }
        self.lex();
        Some((register, end))
    }

    fn operand_from(
        &self,
        kind: ParsedKind,
        start: SourceLocation,
        end: SourceLocation,
    ) -> ParsedOperand {
        ParsedOperand {
            kind,
            start: start.position,
            end: if end.buffer == start.buffer {
                end.position
            } else {
                start.position
            },
        }
    }

    fn statement_location(&self, position: usize) -> SourceLocation {
        SourceLocation {
            buffer: self.statement_start.buffer,
            position,
        }
    }

    fn parse_and_match_instruction(&mut self, identifier: &str, identifier_token: Token) -> bool {
        let name = identifier.to_ascii_lowercase();
        let mut state = InstructionState::default();
        self.extended_register_seen = false;
        let Ok(mut operands) =
            self.parse_instruction(&name, identifier_token.location(), &mut state)
        else {
            return true;
        };
        if self.extended_register_seen {
            state.uses_extended_registers = true;
        }
        if self.options.show_instruction_operands {
            let text = self.describe_operands(&operands);
            let location = identifier_token.location();
            self.message(
                MessageKind::Note,
                location,
                &format!("parsed instruction: [{text}]"),
                None,
            );
        }
        if self.has_pending_error() {
            return true;
        }
        self.match_and_emit(identifier_token.location(), &mut operands, &state)
    }

    fn describe_operands(&self, operands: &[ParsedOperand]) -> String {
        let mut pieces: Vec<String> = Vec::new();
        for operand in operands {
            pieces.push(match &operand.kind {
                ParsedKind::Token(text) => format!("Token:{text}"),
                ParsedKind::Register(register) => {
                    format!("Reg:{}", self.target.register_name(*register))
                }
                ParsedKind::DxRegister => "DXReg".to_string(),
                ParsedKind::Immediate(expression) => format!(
                    "Imm:{}",
                    expression_text(self.target, expression, 0, Syntax::Att)
                ),
                ParsedKind::Memory(memory) => format!("Memory: ModeSize={}", memory.mode_size),
                ParsedKind::Prefix(value) => format!("Prefix:{value}"),
            });
        }
        pieces.join(", ")
    }

    fn parse_instruction(
        &mut self,
        initial: &str,
        initial_location: SourceLocation,
        state: &mut InstructionState,
    ) -> Result<Vec<ParsedOperand>, ()> {
        let mut name = initial.to_string();
        let mut name_location = initial_location;
        while name == "{" {
            if !self.is(TokenKind::Identifier) {
                self.token_error("Unexpected token after '{'");
                return Err(());
            }
            let prefix = self.lexer.token.text.to_ascii_lowercase();
            self.lex();
            if !self.is(TokenKind::RightBrace) {
                self.token_error("Expected '}'");
                return Err(());
            }
            self.lex();
            match prefix.as_str() {
                "rex" => state.forced_prefix = PREFIX_USE_REX,
                "rex2" => state.forced_prefix = PREFIX_USE_REX2,
                "vex" => state.forced_prefix = PREFIX_USE_VEX,
                "vex2" => state.forced_prefix = PREFIX_USE_VEX2,
                "vex3" => state.forced_prefix = PREFIX_USE_VEX3,
                "evex" => state.forced_prefix = PREFIX_USE_EVEX,
                "disp8" => state.forced_displacement = 8,
                "disp32" => state.forced_displacement = 32,
                "nf" => state.forced_no_flags = true,
                _ => {
                    self.error(name_location, "unknown prefix");
                    return Err(());
                }
            }
            name_location = self.location();
            if self.is(TokenKind::LeftBrace) {
                self.lex();
                name = "{".to_string();
            } else {
                if !self.is(TokenKind::Identifier) {
                    self.token_error("Expected identifier");
                    return Err(());
                }
                name = self.lexer.token.text.clone();
                self.lex();
            }
        }
        if let Some(stripped) = name.strip_suffix(".d32") {
            state.forced_displacement = 32;
            name = stripped.to_string();
        } else if let Some(stripped) = name.strip_suffix(".d8") {
            state.forced_displacement = 8;
            name = stripped.to_string();
        }
        let mut patched = name.clone();
        if self.dialect == Syntax::Intel
            && (patched == "jmp"
                || patched == "jc"
                || patched == "jnc"
                || patched == "jcxz"
                || patched == "jecxz"
                || (patched.starts_with('j') && condition_code_from_text(&patched[1..]).is_some()))
            && self.lexer.token.text == "short"
        {
            self.lex();
        }
        if patched.starts_with("set")
            && patched.ends_with('b')
            && patched != "setzub"
            && patched != "setzunb"
            && patched != "setb"
            && patched != "setnb"
        {
            patched = patched[..name.len() - 1].to_string();
        }
        let mut comparison: Option<i64> = None;
        if (patched.starts_with("cmp") || patched.starts_with("vcmp"))
            && (patched.ends_with("ss")
                || patched.ends_with("sd")
                || patched.ends_with("sh")
                || patched.ends_with("ph")
                || patched.ends_with("bf16")
                || patched.ends_with("ps")
                || patched.ends_with("pd"))
        {
            let is_vector = patched.starts_with('v');
            let condition_index = if is_vector { 4 } else { 3 };
            let suffix_length = if patched.ends_with("bf16") { 5 } else { 2 };
            if patched.len() >= condition_index + suffix_length {
                let condition = &patched[condition_index..patched.len() - suffix_length];
                if let Some((_, code)) = COMPARISON_PREDICATES
                    .iter()
                    .find(|(text, _)| *text == condition)
                {
                    if (is_vector || *code < 8) && (is_vector || !patched.ends_with('h')) {
                        patched = if patched.ends_with("ss") {
                            if is_vector {
                                "vcmpss"
                            } else {
                                "cmpss"
                            }
                        } else if patched.ends_with("sd") {
                            if is_vector {
                                "vcmpsd"
                            } else {
                                "cmpsd"
                            }
                        } else if patched.ends_with("ps") {
                            if is_vector {
                                "vcmpps"
                            } else {
                                "cmpps"
                            }
                        } else if patched.ends_with("pd") {
                            if is_vector {
                                "vcmppd"
                            } else {
                                "cmppd"
                            }
                        } else if patched.ends_with("sh") {
                            "vcmpsh"
                        } else if patched.ends_with("ph") {
                            "vcmpph"
                        } else {
                            "vcmpbf16"
                        }
                        .to_string();
                        comparison = Some(*code);
                    }
                }
            }
        }
        if patched.starts_with("vpcmp")
            && matches!(patched.chars().last(), Some('b' | 'w' | 'd' | 'q'))
            && patched.len() >= 6
        {
            let bytes = patched.as_bytes();
            let suffix_size = if bytes[bytes.len() - 2] == b'u' { 2 } else { 1 };
            if patched.len() >= 5 + suffix_size {
                let condition = &patched[5..patched.len() - suffix_size];
                let code = match condition {
                    "eq" => Some(0),
                    "lt" => Some(1),
                    "le" => Some(2),
                    "neq" => Some(4),
                    "nlt" => Some(5),
                    "nle" => Some(6),
                    _ => None,
                };
                if let Some(code) = code {
                    if code != 0 || suffix_size == 2 {
                        let last = bytes[bytes.len() - 1] as char;
                        patched = if suffix_size == 2 {
                            format!("vpcmpu{last}")
                        } else {
                            format!("vpcmp{last}")
                        };
                        comparison = Some(code);
                    }
                }
            }
        }
        if patched.starts_with("vpcom")
            && matches!(patched.chars().last(), Some('b' | 'w' | 'd' | 'q'))
            && patched.len() >= 6
        {
            let bytes = patched.as_bytes();
            let suffix_size = if bytes[bytes.len() - 2] == b'u' { 2 } else { 1 };
            if patched.len() >= 5 + suffix_size {
                let condition = &patched[5..patched.len() - suffix_size];
                let code = match condition {
                    "lt" => Some(0),
                    "le" => Some(1),
                    "gt" => Some(2),
                    "ge" => Some(3),
                    "eq" => Some(4),
                    "neq" => Some(5),
                    "false" => Some(6),
                    "true" => Some(7),
                    _ => None,
                };
                if let Some(code) = code {
                    let last = bytes[bytes.len() - 1] as char;
                    patched = if suffix_size == 2 {
                        format!("vpcomu{last}")
                    } else {
                        format!("vpcom{last}")
                    };
                    comparison = Some(code);
                }
            }
        }
        let mut is_prefix = matches!(
            name.as_str(),
            "cs" | "ds"
                | "es"
                | "fs"
                | "gs"
                | "ss"
                | "rex64"
                | "data32"
                | "data16"
                | "addr32"
                | "addr16"
                | "xacquire"
                | "xrelease"
        ) || (self.dialect == Syntax::Intel
            && (name == "acquire" || name == "release"));
        let mut flags: u32 = 0;
        while matches!(
            name.to_ascii_lowercase().as_str(),
            "lock" | "rep" | "repe" | "repz" | "repne" | "repnz" | "notrack"
        ) {
            let prefix = match name.as_str() {
                "lock" => PREFIX_LOCK,
                "rep" | "repe" | "repz" => PREFIX_REPEAT,
                "repne" | "repnz" => PREFIX_REPEAT_NOT_EQUAL,
                "notrack" => PREFIX_NO_TRACK,
                _ => 0,
            };
            flags |= prefix;
            if self.is(TokenKind::EndOfStatement) {
                flags = 0;
                break;
            }
            name = self.lexer.token.text.clone();
            name_location = self.location();
            self.lex();
            while name.starts_with(';')
                || name.starts_with('\n')
                || name.starts_with('#')
                || name.starts_with('\t')
                || name.starts_with('/')
            {
                name = self.lexer.token.text.clone();
                name_location = self.location();
                self.lex();
            }
        }
        if flags != 0 {
            patched = name.clone();
        }
        if patched == "data16" && self.mode == Mode::Bits16 {
            self.error(name_location, "redundant data16 prefix");
            return Err(());
        }
        if patched == "data32" {
            if self.mode == Mode::Bits32 {
                self.error(name_location, "redundant data32 prefix");
                return Err(());
            }
            if self.mode == Mode::Bits64 {
                self.error(name_location, "'data32' is not supported in 64-bit mode");
                return Err(());
            }
            patched = "data16".to_string();
            if !self.is(TokenKind::EndOfStatement) {
                let mut next = self.lexer.token.text.clone();
                self.lexer.lex();
                if next == "callw" {
                    next = "calll".to_string();
                }
                if next == "ljmpw" {
                    next = "ljmpl".to_string();
                }
                name = next.clone();
                patched = next;
                self.forced_data_prefix = Some(Mode::Bits32);
                is_prefix = false;
            }
        }
        let name_position = name_location.position;
        let mut operands: Vec<ParsedOperand> = vec![ParsedOperand::token(&patched, name_position)];
        if let Some(code) = comparison {
            if self.dialect != Syntax::Intel {
                operands.push(ParsedOperand {
                    kind: ParsedKind::Immediate(Expression::Constant(code)),
                    start: name_position,
                    end: name_position,
                });
            }
        }
        if (name.starts_with("ccmp") || name.starts_with("ctest"))
            && self.parse_conditional_flags(&mut operands)
        {
            return Err(());
        }
        if !self.is(TokenKind::EndOfStatement) && !is_prefix {
            if self.is(TokenKind::Star) {
                let at = self.location();
                operands.push(ParsedOperand::token("*", at.position));
                self.lex();
            }
            loop {
                if self.parse_operand(&mut operands, &name, state) {
                    return Err(());
                }
                if self.handle_avx512_operand(&mut operands) {
                    return Err(());
                }
                if self.is(TokenKind::Comma) {
                    self.lex();
                } else {
                    break;
                }
            }
            if !self.is(TokenKind::EndOfStatement) {
                self.token_error("unexpected token in argument list");
                return Err(());
            }
        }
        if let Some(code) = comparison {
            if self.dialect == Syntax::Intel {
                operands.push(ParsedOperand {
                    kind: ParsedKind::Immediate(Expression::Constant(code)),
                    start: name_position,
                    end: name_position,
                });
            }
        }
        if self.is(TokenKind::EndOfStatement) || (is_prefix && self.is(TokenKind::Slash)) {
            self.lex();
        }
        let is_floating = matches!(name.as_str(), "fsub" | "fdiv" | "fsubr" | "fdivr");
        if is_floating && operands.len() == 1 {
            let replacement = match name.as_str() {
                "fsub" => "fsubp",
                "fdiv" => "fdivp",
                "fsubr" => "fsubrp",
                _ => "fdivrp",
            };
            operands[0] = ParsedOperand::token(replacement, name_position);
        }
        if (name == "mov" || name == "movw" || name == "movl") && operands.len() == 3 {
            if let (ParsedKind::Register(first), ParsedKind::Register(second)) =
                (&operands[1].kind, &operands[2].kind)
            {
                let (first, second) = (*first, *second);
                if self.target.in_class("SEGMENT_REG", second)
                    && (self.target.in_class("GR16", first) || self.target.in_class("GR32", first))
                {
                    let end = operands[1].end;
                    let sixteen = self.mode == Mode::Bits16;
                    if name != "mov" && name.as_bytes()[3] == if sixteen { b'l' } else { b'w' } {
                        operands[0] = ParsedOperand::token(
                            if sixteen { "movw" } else { "movl" },
                            name_position,
                        );
                    }
                    let register =
                        self.target
                            .sub_super_register(first, if sixteen { 16 } else { 32 }, false);
                    operands[1] = ParsedOperand {
                        kind: ParsedKind::Register(register),
                        start: end,
                        end,
                    };
                }
            }
        }
        if matches!(
            name.as_str(),
            "outb" | "outsb" | "outw" | "outsw" | "outl" | "outsl" | "out" | "outs"
        ) && operands.len() == 3
        {
            if let Some(last) = operands.last_mut() {
                if last.kind == ParsedKind::DxRegister {
                    last.kind = ParsedKind::Register(self.target.register("DX"));
                }
            }
        }
        if matches!(
            name.as_str(),
            "inb" | "insb" | "inw" | "insw" | "inl" | "insl" | "in" | "ins"
        ) && operands.len() == 3
            && operands[1].kind == ParsedKind::DxRegister
        {
            operands[1].kind = ParsedKind::Register(self.target.register("DX"));
        }
        let mut replacement: Option<Vec<ParsedOperand>> = None;
        let dx = || ParsedKind::Register(self.target.register("DX"));
        if matches!(name.as_str(), "insb" | "insw" | "insl" | "insd" | "ins")
            && (operands.len() == 1 || operands.len() == 3)
        {
            let source = self.operand_from(dx(), name_location, name_location);
            let destination = self.default_index_operand(name_location, false);
            replacement = Some(self.source_destination(source, destination));
        }
        if matches!(
            name.as_str(),
            "outsb" | "outsw" | "outsl" | "outsd" | "outs"
        ) && (operands.len() == 1 || operands.len() == 3)
        {
            let source = self.default_index_operand(name_location, true);
            let destination = self.operand_from(dx(), name_location, name_location);
            replacement = Some(self.source_destination(source, destination));
        }
        if matches!(
            name.as_str(),
            "lods" | "lodsb" | "lodsw" | "lodsl" | "lodsd" | "lodsq"
        ) && (operands.len() == 1 || operands.len() == 2)
        {
            replacement = Some(vec![self.default_index_operand(name_location, true)]);
        }
        if matches!(
            name.as_str(),
            "stos" | "stosb" | "stosw" | "stosl" | "stosd" | "stosq"
        ) && (operands.len() == 1 || operands.len() == 2)
        {
            replacement = Some(vec![self.default_index_operand(name_location, false)]);
        }
        if matches!(
            name.as_str(),
            "scas" | "scasb" | "scasw" | "scasl" | "scasd" | "scasq"
        ) && (operands.len() == 1 || operands.len() == 2)
        {
            replacement = Some(vec![self.default_index_operand(name_location, false)]);
        }
        if matches!(
            name.as_str(),
            "cmps" | "cmpsb" | "cmpsw" | "cmpsl" | "cmpsd" | "cmpsq"
        ) && (operands.len() == 1 || operands.len() == 3)
        {
            let source = self.default_index_operand(name_location, false);
            let destination = self.default_index_operand(name_location, true);
            replacement = Some(self.source_destination(source, destination));
        }
        if matches!(
            name.as_str(),
            "movs"
                | "movsb"
                | "movsw"
                | "movsl"
                | "movsd"
                | "movsq"
                | "smov"
                | "smovb"
                | "smovw"
                | "smovl"
                | "smovd"
                | "smovq"
        ) && (operands.len() == 1 || operands.len() == 3)
        {
            if name == "movsd" && operands.len() == 1 && self.dialect != Syntax::Intel {
                operands[0] = ParsedOperand::token("movsl", name_position);
            }
            let source = self.default_index_operand(name_location, true);
            let destination = self.default_index_operand(name_location, false);
            replacement = Some(self.source_destination(source, destination));
        }
        if let Some(finals) = replacement {
            if self.verify_and_adjust_operands(&mut operands, finals) {
                return Err(());
            }
        }
        if (name == "xlat" || name == "xlatb") && operands.len() == 2 {
            if let ParsedKind::Memory(memory) = &operands[1].kind {
                if memory.size == 0 || memory.size == 8 {
                    let at = self.statement_location(operands[1].start);
                    self.warning(at, "memory operand is only for determining the size, (R|E)BX will be used for the location");
                    operands.pop();
                    operands[0] = ParsedOperand::token("xlatb", name_position);
                }
            }
        }
        if flags != 0 {
            operands.push(ParsedOperand {
                kind: ParsedKind::Prefix(flags),
                start: name_position,
                end: name_position,
            });
        }
        Ok(operands)
    }

    fn source_destination(
        &self,
        source: ParsedOperand,
        destination: ParsedOperand,
    ) -> Vec<ParsedOperand> {
        if self.dialect == Syntax::Intel {
            vec![destination, source]
        } else {
            vec![source, destination]
        }
    }

    fn default_index_operand(&self, location: SourceLocation, source: bool) -> ParsedOperand {
        let parse_32 = self.mode == Mode::Bits32 || self.code_16_gcc;
        let base = match (self.mode, parse_32, source) {
            (Mode::Bits64, _, true) => "RSI",
            (Mode::Bits64, _, false) => "RDI",
            (_, true, true) => "ESI",
            (_, true, false) => "EDI",
            (_, false, true) => "SI",
            (_, false, false) => "DI",
        };
        let mut memory = MemoryOperand::absolute(self.pointer_width(), Expression::Constant(0));
        memory.base = self.target.register(base);
        self.operand_from(ParsedKind::Memory(memory), location, location)
    }

    fn verify_and_adjust_operands(
        &mut self,
        original: &mut Vec<ParsedOperand>,
        mut finals: Vec<ParsedOperand>,
    ) -> bool {
        if original.len() > 1 {
            if original.len() != finals.len() + 1 {
                return false;
            }
            let mut warnings: Vec<(usize, String)> = Vec::new();
            let mut class: Option<&'static str> = None;
            for position in 0..finals.len() {
                let original_operand = original[position + 1].clone();
                let final_operand = &mut finals[position];
                if let ParsedKind::Register(final_register) = final_operand.kind {
                    if original_operand.kind != ParsedKind::Register(final_register) {
                        return false;
                    }
                }
                if let ParsedKind::Memory(final_memory) = &mut final_operand.kind {
                    let ParsedKind::Memory(original_memory) = &original_operand.kind else {
                        return false;
                    };
                    let original_base = original_memory.base;
                    if let Some(held) = class {
                        if !self.target.in_class(held, original_base) {
                            let at = self.statement_location(original_operand.start);
                            return self
                                .error(at, "mismatching source and destination index registers");
                        }
                    }
                    class = if self.target.in_class("GR64", original_base) {
                        Some("GR64")
                    } else if self.target.in_class("GR32", original_base) {
                        Some("GR32")
                    } else if self.target.in_class("GR16", original_base) {
                        Some("GR16")
                    } else {
                        return false;
                    };
                    let final_record = self.target.registers[final_memory.base as usize]
                        .record
                        .clone();
                    let is_source = matches!(final_record.as_str(), "RSI" | "ESI" | "SI");
                    let size = match class {
                        Some("GR64") => 64,
                        Some("GR32") => 32,
                        _ => 16,
                    };
                    let name = match (size, is_source) {
                        (64, true) => "RSI",
                        (64, false) => "RDI",
                        (32, true) => "ESI",
                        (32, false) => "EDI",
                        (_, true) => "SI",
                        (_, false) => "DI",
                    };
                    let new_base = self.target.register(name);
                    if new_base != original_base {
                        let text = if is_source {
                            "ES:(R|E)SI"
                        } else {
                            "ES:(R|E)DI"
                        };
                        warnings.push((original_operand.start, format!("memory operand is only for determining the size, {text} will be used for the location")));
                    }
                    final_memory.size = original_memory.size;
                    final_memory.segment = original_memory.segment;
                    final_memory.base = new_base;
                }
            }
            for (position, text) in warnings {
                let at = self.statement_location(position);
                self.warning(at, &text);
            }
            for _ in 0..finals.len() {
                original.pop();
            }
        }
        original.extend(finals);
        false
    }

    fn parse_conditional_flags(&mut self, operands: &mut Vec<ParsedOperand>) -> bool {
        let start = self.location();
        if !self.is(TokenKind::LeftBrace) {
            return self.token_error("Expected { at this point");
        }
        self.lex();
        if !self.lexer.token.identifier().eq_ignore_ascii_case("dfv") {
            return self.token_error("Expected dfv at this point");
        }
        self.lex();
        if !self.is(TokenKind::Equal) {
            return self.token_error("Expected = at this point");
        }
        self.lex();
        if self.is(TokenKind::RightBrace) {
            let end = self.lexer.token.end_location();
            operands.push(self.operand_from(
                ParsedKind::Immediate(Expression::Constant(0)),
                start,
                end,
            ));
            self.lex();
            return false;
        }
        let mut flags = 0i64;
        for index in 0..4 {
            let value = match self.lexer.token.identifier().to_ascii_lowercase().as_str() {
                "of" => 8,
                "sf" => 4,
                "zf" => 2,
                "cf" => 1,
                _ => return self.token_error("Invalid conditional flags"),
            };
            if flags & value != 0 {
                return self.token_error("Duplicated conditional flag");
            }
            flags |= value;
            self.lex();
            if self.is(TokenKind::RightBrace) {
                let end = self.lexer.token.end_location();
                operands.push(self.operand_from(
                    ParsedKind::Immediate(Expression::Constant(flags)),
                    start,
                    end,
                ));
                self.lex();
                return false;
            } else if index == 3 {
                return self.token_error("Expected } at this point");
            } else if !self.is(TokenKind::Comma) {
                return self.token_error("Expected } or , at this point");
            }
            self.lex();
        }
        false
    }

    fn parse_operand(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        name: &str,
        state: &mut InstructionState,
    ) -> bool {
        if self.dialect == Syntax::Intel {
            return self.parse_intel_operand(operands, name, state);
        }
        self.parse_att_operand(operands, state)
    }

    fn parse_att_operand(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        state: &mut InstructionState,
    ) -> bool {
        match self.lexer.token.kind {
            TokenKind::Dollar => {
                let start = self.location();
                self.lex();
                let at = self.location();
                if self.is(TokenKind::Percent) {
                    return self.error(at, "expected immediate expression");
                }
                let Ok((value, end)) = self.parse_expression() else {
                    return true;
                };
                if matches!(value, Expression::Register(_)) {
                    return self.error(at, "expected immediate expression");
                }
                operands.push(self.operand_from(ParsedKind::Immediate(value), start, end));
                false
            }
            TokenKind::LeftBrace => {
                let start = self.location();
                self.parse_rounding_mode(start, operands)
            }
            _ => {
                let location = self.location();
                let mut end = location;
                let mut displacement: Option<Expression> = None;
                let mut segment = NO_REGISTER;
                if !self.is(TokenKind::LeftParenthesis) {
                    let Ok((expression, expression_end)) = self.parse_expression() else {
                        return true;
                    };
                    end = expression_end;
                    if let Expression::Register(register) = expression {
                        let record = self.target.registers[register as usize].record.clone();
                        if record == "EIZ" || record == "RIZ" {
                            return self.error_range(
                                location,
                                "%eiz and %riz can only be used as index registers",
                                (location, end),
                            );
                        }
                        if record == "RIP" {
                            return self.error_range(
                                location,
                                "%rip can only be used as a base register",
                                (location, end),
                            );
                        }
                        if !self.parse_optional_token(TokenKind::Colon) {
                            operands.push(self.operand_from(
                                ParsedKind::Register(register),
                                location,
                                end,
                            ));
                            return false;
                        }
                        if !self.target.in_class("SEGMENT_REG", register) {
                            return self.error(location, "invalid segment register");
                        }
                        segment = register;
                        if self.is(TokenKind::Star) {
                            let at = self.location();
                            operands.push(ParsedOperand::token("*", at.position));
                            self.lex();
                        }
                    } else {
                        displacement = Some(expression);
                    }
                }
                let _ = state;
                self.parse_memory_operand(segment, displacement, location, end, operands)
            }
        }
    }

    fn is_at_memory_operand(&mut self) -> bool {
        if !self.is(TokenKind::LeftParenthesis) {
            return false;
        }
        let tokens = self.lexer.peek(2);
        let Some(first) = tokens.first() else {
            return false;
        };
        let mut identifier = String::new();
        match first.kind {
            TokenKind::Percent | TokenKind::Comma => return true,
            TokenKind::At | TokenKind::Dollar => {
                if let Some(second) = tokens.get(1) {
                    if (second.kind == TokenKind::Identifier || second.kind == TokenKind::String)
                        && first.start + 1 == second.start
                    {
                        identifier = format!("{}{}", first.text, second.identifier());
                    }
                }
            }
            TokenKind::Identifier | TokenKind::String => identifier = first.identifier(),
            _ => return false,
        }
        if !identifier.is_empty() {
            let symbol = self.context.symbol(&identifier);
            if let Some(value) = &self.context.symbols[symbol].value {
                return matches!(value, Expression::Register(_));
            }
        }
        false
    }

    fn parse_memory_operand(
        &mut self,
        segment: RegisterId,
        displacement: Option<Expression>,
        start: SourceLocation,
        mut end: SourceLocation,
        operands: &mut Vec<ParsedOperand>,
    ) -> bool {
        let displacement = match displacement {
            Some(value) => value,
            None => {
                if !self.is_at_memory_operand() {
                    let at = self.location();
                    let Ok((value, value_end)) = self.parse_expression() else {
                        return true;
                    };
                    end = value_end;
                    if matches!(value, Expression::Register(_)) {
                        return self.error_range(
                            at,
                            "unexpected register in memory operand",
                            (at, end),
                        );
                    }
                    value
                } else {
                    Expression::Constant(0)
                }
            }
        };
        if !self.parse_optional_token(TokenKind::LeftParenthesis) {
            let mut memory = MemoryOperand::absolute(self.pointer_width(), displacement);
            memory.segment = segment;
            operands.push(self.operand_from(ParsedKind::Memory(memory), start, end));
            return false;
        }
        let mut base = NO_REGISTER;
        let mut index = NO_REGISTER;
        let mut scale: u32 = 1;
        let base_location = self.location();
        if !self.is(TokenKind::Comma) && !self.is(TokenKind::RightParenthesis) {
            let Ok((expression, expression_end)) = self.parse_expression() else {
                return true;
            };
            end = expression_end;
            let Expression::Register(register) = expression else {
                return self.error(base_location, "expected register here");
            };
            base = register;
            let record = self.target.registers[base as usize].record.as_str();
            if record == "EIZ" || record == "RIZ" {
                return self.error_range(
                    base_location,
                    "eiz and riz can only be used as index registers",
                    (base_location, end),
                );
            }
        }
        if self.parse_optional_token(TokenKind::Comma) && !self.is(TokenKind::RightParenthesis) {
            let at = self.location();
            let Ok((expression, _)) = self.parse_expression() else {
                return true;
            };
            match expression {
                Expression::Register(register) => {
                    index = register;
                    if self.target.registers[base as usize].record == "RIP" {
                        return self
                            .error(at, "%rip as base register can not have an index register");
                    }
                    if self.target.registers[index as usize].record == "RIP" {
                        return self.error(at, "%rip is not allowed as an index register");
                    }
                    if self.parse_optional_token(TokenKind::Comma)
                        && !self.is(TokenKind::RightParenthesis)
                    {
                        let scale_location = self.location();
                        let Ok(value) = self.parse_absolute_expression() else {
                            return self.error(scale_location, "expected scale expression");
                        };
                        scale = value as u32;
                        if self.target.in_class("GR16", base) && scale != 1 {
                            return self
                                .error(scale_location, "scale factor in 16-bit address must be 1");
                        }
                        if !matches!(scale, 1 | 2 | 4 | 8) {
                            return self.error(
                                scale_location,
                                "scale factor in address must be 1, 2, 4 or 8",
                            );
                        }
                    }
                }
                other => {
                    let Some(value) = self.context.evaluate_absolute(&other, None) else {
                        return self.error(at, "expected absolute expression");
                    };
                    if value != 1 {
                        self.warning(at, "scale factor without index register is ignored");
                    }
                    scale = 1;
                }
            }
        }
        end = self.lexer.token.end_location();
        if self.parse_token(
            TokenKind::RightParenthesis,
            "unexpected token in memory operand",
        ) {
            return true;
        }
        if self.target.registers[base as usize].record == "DX"
            && index == NO_REGISTER
            && scale == 1
            && segment == NO_REGISTER
            && displacement == Expression::Constant(0)
        {
            operands.push(self.operand_from(ParsedKind::DxRegister, base_location, base_location));
            return false;
        }
        if let Some(text) = self.check_base_index_scale(base, index, scale) {
            return self.error(base_location, text);
        }
        if self.check_displacement_overflow(base, index, &displacement, base_location) {
            return true;
        }
        let mut memory = MemoryOperand::absolute(self.pointer_width(), displacement);
        memory.segment = segment;
        memory.base = base;
        memory.index = index;
        memory.scale = scale;
        operands.push(self.operand_from(ParsedKind::Memory(memory), start, end));
        false
    }

    pub fn check_base_index_scale(
        &self,
        base: RegisterId,
        index: RegisterId,
        scale: u32,
    ) -> Option<&'static str> {
        let target = self.target;
        let record = |register: RegisterId| target.registers[register as usize].record.as_str();
        let gr16 = |register: RegisterId| target.in_class("GR16", register);
        let gr32 = |register: RegisterId| target.in_class("GR32", register);
        let gr64 = |register: RegisterId| target.in_class("GR64", register);
        let invalid = "invalid base+index expression";
        if base != NO_REGISTER
            && !(matches!(record(base), "RIP" | "EIP") || gr16(base) || gr32(base) || gr64(base))
        {
            return Some(invalid);
        }
        if index != NO_REGISTER
            && !(matches!(record(index), "EIZ" | "RIZ")
                || gr16(index)
                || gr32(index)
                || gr64(index)
                || target.in_class("VR128X", index)
                || target.in_class("VR256X", index)
                || target.in_class("VR512", index))
        {
            return Some(invalid);
        }
        if (matches!(record(base), "RIP" | "EIP") && index != NO_REGISTER)
            || matches!(record(index), "EIP" | "RIP" | "ESP" | "RSP")
        {
            return Some(invalid);
        }
        if gr16(base)
            && (self.mode == Mode::Bits64 || !matches!(record(base), "BX" | "BP" | "SI" | "DI"))
        {
            return Some("invalid 16-bit base register");
        }
        if base == NO_REGISTER && gr16(index) {
            return Some("16-bit memory operand may not include only index register");
        }
        if base != NO_REGISTER && index != NO_REGISTER {
            if gr64(base) && (gr16(index) || gr32(index) || record(index) == "EIZ") {
                return Some("base register is 64-bit, but index register is not");
            }
            if gr32(base) && (gr16(index) || gr64(index) || record(index) == "RIZ") {
                return Some("base register is 32-bit, but index register is not");
            }
            if gr16(base) {
                if gr32(index) || gr64(index) {
                    return Some("base register is 16-bit, but index register is not");
                }
                if !matches!(record(base), "BX" | "BP") || !matches!(record(index), "SI" | "DI") {
                    return Some("invalid 16-bit base/index register combination");
                }
            }
        }
        if self.mode != Mode::Bits64 && matches!(record(base), "RIP" | "EIP") {
            return Some("IP-relative addressing requires 64-bit mode");
        }
        if !matches!(scale, 1 | 2 | 4 | 8) {
            return Some("scale factor in address must be 1, 2, 4 or 8");
        }
        None
    }

    fn parse_rounding_mode(
        &mut self,
        start: SourceLocation,
        operands: &mut Vec<ParsedOperand>,
    ) -> bool {
        self.lex();
        let token = self.lexer.token.clone();
        if token.kind != TokenKind::Identifier {
            return self.error(token.location(), "Expected an identifier after {");
        }
        if token.text.starts_with('r') {
            let mode = match token.text.as_str() {
                "rn" => 0,
                "rd" => 1,
                "ru" => 2,
                "rz" => 3,
                _ => return self.error(token.location(), "Invalid rounding mode."),
            };
            self.lex();
            if !self.is(TokenKind::Minus) {
                return self.error(token.location(), "Expected - at this point");
            }
            self.lex();
            self.lex();
            if !self.is(TokenKind::RightBrace) {
                return self.error(token.location(), "Expected } at this point");
            }
            let end = token.end_location();
            self.lex();
            operands.push(self.operand_from(
                ParsedKind::Immediate(Expression::Constant(mode)),
                start,
                end,
            ));
            return false;
        }
        if token.text == "sae" {
            self.lex();
            if !self.is(TokenKind::RightBrace) {
                return self.error(token.location(), "Expected } at this point");
            }
            self.lex();
            operands.push(ParsedOperand::token("{sae}", start.position));
            return false;
        }
        self.error(token.location(), "unknown token in expression")
    }

    fn handle_avx512_operand(&mut self, operands: &mut Vec<ParsedOperand>) -> bool {
        if !self.is(TokenKind::LeftBrace) {
            return false;
        }
        let consumed = self.location();
        self.lex();
        if self.is(TokenKind::Integer) {
            if self.lexer.token.integer() != 1 {
                return self.token_error("Expected 1to<NUM> at this point");
            }
            let prefix = self.lexer.token.text.clone();
            self.lex();
            if !self.is(TokenKind::Identifier) {
                return self.token_error("Expected 1to<NUM> at this point");
            }
            let broadcast = format!("{prefix}{}", self.lexer.token.identifier());
            if !broadcast.starts_with("1to") {
                return self.token_error("Expected 1to<NUM> at this point");
            }
            let primitive = match broadcast.as_str() {
                "1to2" => "{1to2}",
                "1to4" => "{1to4}",
                "1to8" => "{1to8}",
                "1to16" => "{1to16}",
                "1to32" => "{1to32}",
                _ => return self.token_error("Invalid memory broadcast primitive."),
            };
            self.lex();
            if !self.is(TokenKind::RightBrace) {
                return self.token_error("Expected } at this point");
            }
            self.lex();
            operands.push(ParsedOperand::token(primitive, consumed.position));
            return false;
        }
        let mut zero: Option<ParsedOperand> = None;
        if self.parse_zeroing(&mut zero, consumed) {
            return true;
        }
        if zero.is_none() || self.is(TokenKind::LeftBrace) {
            let start = if zero.is_some() {
                let at = self.location();
                self.lex();
                at
            } else {
                consumed
            };
            let register_start = self.location();
            let parsed = self.parse_register_operand(register_start);
            match parsed {
                Some((register, _)) if self.target.in_class("VK1", register) => {
                    if self.target.registers[register as usize].record == "K0" {
                        return self
                            .error(register_start, "Register k0 can't be used as write mask");
                    }
                    if !self.is(TokenKind::RightBrace) {
                        return self.token_error("Expected } at this point");
                    }
                    operands.push(ParsedOperand::token("{", start.position));
                    operands.push(ParsedOperand {
                        kind: ParsedKind::Register(register),
                        start: start.position,
                        end: start.position,
                    });
                    let close = self.location();
                    self.lex();
                    operands.push(ParsedOperand::token("}", close.position));
                }
                _ => {
                    return self.token_error("Expected an op-mask register at this point");
                }
            }
            if self.is(TokenKind::LeftBrace) && zero.is_none() {
                let at = self.location();
                self.lex();
                if self.parse_zeroing(&mut zero, at) || zero.is_none() {
                    return self.token_error("Expected a {z} mark at this point");
                }
            }
        }
        if let Some(mark) = zero {
            operands.push(mark);
        }
        false
    }

    fn parse_zeroing(&mut self, zero: &mut Option<ParsedOperand>, start: SourceLocation) -> bool {
        if !(self.is(TokenKind::Identifier) && self.lexer.token.text == "z") {
            return false;
        }
        self.lex();
        if !self.is(TokenKind::RightBrace) {
            return self.token_error("Expected } at this point");
        }
        self.lex();
        *zero = Some(ParsedOperand::token("{z}", start.position));
        false
    }

    fn parse_intel_operand(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        name: &str,
        state: &mut InstructionState,
    ) -> bool {
        let Ok(sized) = self.parse_intel_memory_size() else {
            return true;
        };
        let size_given = sized.is_some();
        let size = sized.unwrap_or(0);
        let mut start = self.location();
        if self.is(TokenKind::LeftBrace) {
            return self.parse_rounding_mode(start, operands);
        }
        let mut segment = NO_REGISTER;
        if self.is(TokenKind::Identifier) {
            let Ok(found) = self.intel_register(state) else {
                return true;
            };
            if let Some((register, end)) = found {
                if self.target.registers[register as usize].record == "RIP" {
                    return self.error(start, "rip can only be used as a base register");
                }
                if !self.is(TokenKind::Colon) {
                    if size_given {
                        return self.error(
                            start,
                            "expected memory operand after 'ptr', found register operand instead",
                        );
                    }
                    operands.push(self.operand_from(ParsedKind::Register(register), start, end));
                    return false;
                }
                if !self.target.in_class("SEGMENT_REG", register) {
                    return self.error(start, "invalid segment register");
                }
                segment = register;
                self.lex();
                start = self.location();
            }
        }
        let mut expression = IntelExpression::new();
        let Ok(end) = self.parse_intel_expression(&mut expression, state) else {
            return true;
        };
        let value = expression.value();
        let constant = Expression::Constant(value);
        let displacement = match expression.symbol.clone() {
            Some(symbol) if value != 0 => Expression::sum(symbol, constant),
            Some(symbol) => symbol,
            None => constant,
        };
        if !expression.memory_expression && segment == NO_REGISTER {
            operands.push(self.operand_from(ParsedKind::Immediate(displacement), start, end));
            return false;
        }
        let mut base = expression.base;
        let mut index = expression.index;
        let rip = self.target.register("RIP");
        if index != NO_REGISTER && base == rip {
            base = NO_REGISTER;
        }
        let mut scale = expression.scale;
        let record = |register: RegisterId| self.target.registers[register as usize].record.clone();
        if scale == 0
            && !matches!(record(base).as_str(), "ESP" | "RSP")
            && matches!(record(index).as_str(), "ESP" | "RSP")
        {
            std::mem::swap(&mut base, &mut index);
        }
        let vector = |register: RegisterId| {
            ["VR128X", "VR256X", "VR512"]
                .iter()
                .any(|class| self.target.in_class(class, register))
        };
        if scale == 0 && !vector(index) && vector(base) {
            std::mem::swap(&mut base, &mut index);
        }
        if scale != 0 && self.target.in_class("GR16", index) {
            return self.error(start, "16-bit addresses cannot have a scale");
        }
        if scale == 0 {
            scale = 1;
        }
        if matches!(record(base).as_str(), "SI" | "DI")
            && matches!(record(index).as_str(), "BX" | "BP")
        {
            std::mem::swap(&mut base, &mut index);
        }
        if base != NO_REGISTER || index != NO_REGISTER {
            if let Some(text) = self.check_base_index_scale(base, index, scale) {
                return self.error(start, text);
            }
        }
        let branch = name.eq_ignore_ascii_case("jmp") || name.eq_ignore_ascii_case("call");
        let mut maybe_direct_branch = true;
        if branch {
            if !size_given && expression.offset_operator {
                return self.error(
                    start,
                    "`OFFSET` operator cannot be used in an unconditional branch",
                );
            }
            if size_given || expression.bracket_used {
                maybe_direct_branch = false;
            }
        }
        if self.check_displacement_overflow(base, index, &displacement, start) {
            return true;
        }
        let mut memory = MemoryOperand::absolute(self.pointer_width(), displacement);
        memory.segment = segment;
        memory.base = base;
        memory.index = index;
        memory.scale = scale;
        memory.size = size;
        memory.maybe_direct_branch = maybe_direct_branch;
        operands.push(self.operand_from(ParsedKind::Memory(memory), start, end));
        false
    }

    fn parse_intel_memory_size(&mut self) -> Result<Option<u32>, ()> {
        let text = self.lexer.token.text.clone();
        let size = match text.as_str() {
            "BYTE" | "byte" => 8,
            "WORD" | "word" => 16,
            "DWORD" | "dword" | "FLOAT" | "float" | "LONG" | "long" => 32,
            "FWORD" | "fword" => 48,
            "DOUBLE" | "double" | "QWORD" | "qword" | "MMWORD" | "mmword" => 64,
            "XWORD" | "xword" | "TBYTE" | "tbyte" => 80,
            "XMMWORD" | "xmmword" => 128,
            "YMMWORD" | "ymmword" => 256,
            "ZMMWORD" | "zmmword" => 512,
            _ => return Ok(None),
        };
        self.lex();
        if !matches!(self.lexer.token.text.as_str(), "PTR" | "ptr") {
            self.token_error("Expected 'PTR' or 'ptr' token!");
            return Err(());
        }
        self.lex();
        Ok(Some(size))
    }

    fn intel_register(
        &mut self,
        state: &mut InstructionState,
    ) -> Result<Option<(RegisterId, SourceLocation)>, ()> {
        let token = self.lexer.token.clone();
        if token.kind != TokenKind::Identifier {
            return Ok(None);
        }
        let start = token.location();
        let end = token.end_location();
        match self.register_by_name_checked(&token.text, start, end, state) {
            Ok(_) => match self.parse_register_with_state(start, state) {
                Some(found) => Ok(Some(found)),
                None => Err(()),
            },
            Err(_) => Ok(None),
        }
    }

    fn check_displacement_overflow(
        &mut self,
        base: RegisterId,
        index: RegisterId,
        displacement: &Expression,
        location: SourceLocation,
    ) -> bool {
        if base == NO_REGISTER && index == NO_REGISTER {
            return false;
        }
        let Expression::Constant(value) = *displacement else {
            return false;
        };
        let is_64 = self.target.in_class("GR64", base) || self.target.in_class("GR64", index);
        let is_16 = self.target.in_class("GR16", base);
        if is_64 {
            if i32::try_from(value).is_err() {
                return self.error(
                    location,
                    &format!("displacement {value} is not within [-2147483648, 2147483647]"),
                );
            }
        } else if !is_16 {
            if value.unsigned_abs() >> 32 != 0 {
                self.warning(
                    location,
                    &format!(
                        "displacement {value} shortened to 32-bit signed {}",
                        value as i32
                    ),
                );
            }
        } else if value.unsigned_abs() >> 16 != 0 {
            self.warning(
                location,
                &format!(
                    "displacement {value} shortened to 16-bit signed {}",
                    value as i16
                ),
            );
        }
        false
    }

    fn intel_failure<T>(&mut self, location: SourceLocation, text: &str) -> Result<T, ()> {
        self.error(location, text);
        Err(())
    }

    fn parse_intel_expression(
        &mut self,
        expression: &mut IntelExpression,
        state: &mut InstructionState,
    ) -> Result<SourceLocation, ()> {
        let mut end = self.location();
        let mut previous_kind = TokenKind::Error;
        loop {
            let token = self.lexer.token.clone();
            let kind = token.kind;
            let at = token.location();
            let mut advance = true;
            let mut done = false;
            match kind {
                TokenKind::Error => {
                    if let Some((location, message)) = self.lexer.error.take() {
                        self.error(location, &message);
                    }
                    return Err(());
                }
                TokenKind::Real => {
                    advance = false;
                    self.parse_intel_dot_operator(expression, &mut end)?;
                }
                TokenKind::String | TokenKind::At | TokenKind::Identifier => {
                    advance = false;
                    self.parse_intel_identifier(expression, state, &mut end)?;
                }
                TokenKind::Integer => {
                    advance = false;
                    let value = token.integer();
                    end = at;
                    self.lex();
                    let suffix = self.lexer.token.clone();
                    if suffix.kind == TokenKind::Identifier
                        && (suffix.text == "f" || suffix.text == "b")
                    {
                        let backward = suffix.text == "b";
                        let symbol = self.context.get_directional_symbol(value, backward);
                        if backward && self.context.symbols[symbol].is_undefined() {
                            return self.intel_failure(at, "invalid reference to undefined symbol");
                        }
                        self.directional_labels.push((at, symbol));
                        let name = self.context.symbols[symbol].name.clone();
                        if let Err(text) = expression.on_identifier(Expression::symbol(&name)) {
                            let location = expression.error_location(at);
                            return self.intel_failure(location, text);
                        }
                        end = suffix.location();
                        self.lex();
                    } else if let Err(text) = expression.on_integer(value) {
                        let location = expression.error_location(at);
                        return self.intel_failure(location, text);
                    }
                }
                TokenKind::Plus => {
                    if let Err(text) = expression.on_plus() {
                        return self.intel_failure(at, text);
                    }
                }
                TokenKind::Minus => {
                    if let Err(text) = expression.on_minus(at) {
                        let location = expression.error_location(at);
                        return self.intel_failure(location, text);
                    }
                }
                TokenKind::Tilde => expression.on_not(),
                TokenKind::Star => expression.on_star(),
                TokenKind::Slash => expression.on_divide(IntelState::Divide, InfixToken::Divide),
                TokenKind::Percent => expression.on_divide(IntelState::Modulo, InfixToken::Modulo),
                TokenKind::Pipe => expression.binary(IntelState::Or, InfixToken::Or),
                TokenKind::Caret => expression.binary(IntelState::Xor, InfixToken::Xor),
                TokenKind::Ampersand => expression.binary(IntelState::And, InfixToken::And),
                TokenKind::LessLess => {
                    expression.binary(IntelState::LeftShift, InfixToken::LeftShift)
                }
                TokenKind::GreaterGreater => {
                    expression.binary(IntelState::RightShift, InfixToken::RightShift)
                }
                TokenKind::LeftBracket => {
                    if expression.on_left_bracket().is_err() {
                        return self.intel_failure(at, "unexpected bracket encountered");
                    }
                    if previous_kind == TokenKind::RightBracket {
                        expression.attach_to_operand_index = true;
                    }
                }
                TokenKind::RightBracket => {
                    if let Err(text) = expression.on_right_bracket() {
                        let location = expression.error_location(at);
                        return self.intel_failure(location, text);
                    }
                }
                TokenKind::LeftParenthesis => expression.on_left_parenthesis(at),
                TokenKind::RightParenthesis => {
                    if let Err(text) = expression.on_right_parenthesis() {
                        let location = expression.error_location(at);
                        return self.intel_failure(location, text);
                    }
                }
                _ => {
                    if expression.is_valid_end() {
                        done = true;
                    } else {
                        return self.intel_failure(at, "unknown token in expression");
                    }
                }
            }
            if expression.state == IntelState::Error {
                let here = self.location();
                return self.intel_failure(here, "unknown token in expression");
            }
            if done {
                break;
            }
            if advance {
                end = self.location();
                self.lex();
            }
            previous_kind = kind;
        }
        if expression.parenthesis_count != 0 {
            let location = expression.left_parenthesis.unwrap_or(end);
            return self.intel_failure(location, "unmatched parenthesis");
        }
        Ok(end)
    }

    fn parse_intel_identifier(
        &mut self,
        expression: &mut IntelExpression,
        state: &mut InstructionState,
        end: &mut SourceLocation,
    ) -> Result<(), ()> {
        let token = self.lexer.token.clone();
        let at = token.location();
        if token.kind == TokenKind::Identifier {
            if let Some((register, register_end)) = self.intel_register(state)? {
                *end = register_end;
                if let Err(text) = expression.on_register(register) {
                    let location = expression.error_location(at);
                    return self.intel_failure(location, text);
                }
                return Ok(());
            }
            let text = token.text.as_str();
            let uniform = text == text.to_ascii_lowercase() || text == text.to_ascii_uppercase();
            if uniform {
                let lower = text.to_ascii_lowercase();
                let operator = match lower.as_str() {
                    "not" => Some((IntelState::Not, InfixToken::Not)),
                    "or" => Some((IntelState::Or, InfixToken::Or)),
                    "shl" => Some((IntelState::LeftShift, InfixToken::LeftShift)),
                    "shr" => Some((IntelState::RightShift, InfixToken::RightShift)),
                    "xor" => Some((IntelState::Xor, InfixToken::Xor)),
                    "and" => Some((IntelState::And, InfixToken::And)),
                    "mod" => Some((IntelState::Modulo, InfixToken::Modulo)),
                    _ => None,
                };
                if let Some((next, operator)) = operator {
                    match next {
                        IntelState::Not => expression.on_not(),
                        IntelState::Modulo => expression.on_divide(next, operator),
                        _ => expression.binary(next, operator),
                    }
                    *end = at;
                    self.lex();
                    return Ok(());
                }
                if lower == "offset" || lower == "imagerel" {
                    let kind: &'static str = if lower == "offset" {
                        "offset"
                    } else {
                        "imagerel"
                    };
                    let value = self.parse_intel_offset_operator(kind, end)?;
                    if let Err(text) = expression.on_offset(value, kind) {
                        return self.intel_failure(at, text);
                    }
                    return Ok(());
                }
            }
        }
        match self.parse_generic_primary() {
            Ok((value, value_end)) => {
                *end = value_end;
                if let Err(text) = expression.on_identifier(value) {
                    let location = expression.error_location(at);
                    return self.intel_failure(location, text);
                }
                Ok(())
            }
            Err(()) => self.intel_failure(at, "Unexpected identifier!"),
        }
    }

    fn parse_intel_offset_operator(
        &mut self,
        kind: &'static str,
        end: &mut SourceLocation,
    ) -> Result<Expression, ()> {
        self.lex();
        let start = self.location();
        if !self.is(TokenKind::Identifier) && !self.is(TokenKind::String) {
            self.error(start, "unexpected token!");
            return Err(());
        }
        let Ok((value, value_end)) = self.parse_generic_primary() else {
            self.error(start, "unexpected token!");
            return Err(());
        };
        *end = value_end;
        if kind == "offset" {
            return Ok(value);
        }
        match self.apply_specifier(&value, "IMGREL") {
            Some(modified) => Ok(modified),
            None => {
                self.error(start, "cannot apply 'imagerel' to this expression");
                Err(())
            }
        }
    }

    fn parse_intel_dot_operator(
        &mut self,
        expression: &mut IntelExpression,
        end: &mut SourceLocation,
    ) -> Result<(), ()> {
        let token = self.lexer.token.clone();
        let text = token.text.strip_prefix('.').unwrap_or(&token.text);
        let Ok(offset) = text.parse::<u64>() else {
            return self.intel_failure(token.location(), "Unexpected offset");
        };
        *end = token.end_location();
        self.lex();
        expression.immediate = expression.immediate.wrapping_add(offset as i64);
        Ok(())
    }
}

impl<'s> Assembler<'s> {
    fn is_vector_register(&self, operand: &ParsedOperand) -> bool {
        match operand.kind {
            ParsedKind::Register(register) => ["VR64", "VR128X", "VR256X", "VR512"]
                .iter()
                .any(|class| self.target.in_class(class, register)),
            _ => false,
        }
    }

    fn match_and_emit(
        &mut self,
        location: SourceLocation,
        operands: &mut Vec<ParsedOperand>,
        state: &InstructionState,
    ) -> bool {
        let replacement = match operands[0].token_text() {
            "finit" => Some("fninit"),
            "fsave" => Some("fnsave"),
            "fstcw" | "fstcww" => Some("fnstcw"),
            "fstenv" => Some("fnstenv"),
            "fstsw" | "fstsww" => Some("fnstsw"),
            "fclex" => Some("fnclex"),
            _ => None,
        };
        if let Some(replacement) = replacement {
            let wait = MachineInstruction::new(self.target.instruction("WAIT").unwrap_or(0));
            self.emit_instruction(&wait, location);
            let start = operands[0].start;
            operands[0] = ParsedOperand::token(replacement, start);
        }
        let mut prefixes = 0u32;
        if let Some(ParsedKind::Prefix(value)) = operands.last().map(|operand| operand.kind.clone())
        {
            prefixes = value;
            operands.pop();
        }
        prefixes |= state.forced_prefix;
        match state.forced_displacement {
            8 => prefixes |= PREFIX_USE_DISPLACEMENT_8,
            32 => prefixes |= PREFIX_USE_DISPLACEMENT_32,
            _ => {}
        }
        if self.dialect == Syntax::Intel {
            return self.match_and_emit_intel(location, operands, state, prefixes);
        }
        self.match_and_emit_att(location, operands, state, prefixes)
    }

    fn run_matcher(
        &mut self,
        operands: &[ParsedOperand],
        state: &InstructionState,
    ) -> MatchOutcome {
        let variant = if self.dialect == Syntax::Intel { 1 } else { 0 };
        self.run_matcher_in(operands, state, variant)
    }

    fn run_matcher_in(
        &mut self,
        operands: &[ParsedOperand],
        state: &InstructionState,
        variant: usize,
    ) -> MatchOutcome {
        let context = MatchContext {
            forced_prefix: state.forced_prefix,
            forced_no_flags: state.forced_no_flags,
            uses_extended_registers: state.uses_extended_registers,
        };
        let mode = if self.code_16_gcc {
            Mode::Bits32
        } else {
            self.mode
        };
        self.matcher
            .match_instruction(self.target, operands, variant, mode, &context)
    }

    fn missing_feature_error(&mut self, location: SourceLocation, features: &[String]) -> bool {
        let mut text = "instruction requires:".to_string();
        for feature in features {
            let name = self
                .matcher
                .feature_names
                .get(feature)
                .cloned()
                .unwrap_or_else(|| feature.clone());
            text.push(' ');
            text.push_str(&name);
        }
        self.error(location, &text)
    }

    fn finish_instruction(
        &mut self,
        location: SourceLocation,
        mut instruction: MachineInstruction,
        operands: &[ParsedOperand],
        state: &InstructionState,
        prefixes: u32,
    ) -> bool {
        instruction.flags |= prefixes;
        if self.validate_instruction(&instruction, operands, state, location) {
            return true;
        }
        while self.process_instruction(&mut instruction, state) {}
        self.emit_instruction(&instruction, location);
        false
    }

    fn match_and_emit_att(
        &mut self,
        location: SourceLocation,
        operands: &mut [ParsedOperand],
        state: &InstructionState,
        prefixes: u32,
    ) -> bool {
        let saved_mode = self.mode;
        if self.forced_data_prefix == Some(Mode::Bits32) {
            self.mode = Mode::Bits32;
        }
        let original = self.run_matcher(operands, state);
        if self.forced_data_prefix == Some(Mode::Bits32) {
            self.mode = Mode::Bits16;
            self.forced_data_prefix = None;
        } else {
            self.mode = saved_mode;
        }
        match original.status {
            MatchStatus::Success => {
                let instruction = original
                    .instruction
                    .clone()
                    .unwrap_or_else(|| MachineInstruction::new(0));
                return self.finish_instruction(location, instruction, operands, state, prefixes);
            }
            MatchStatus::InvalidUnsigned4 | MatchStatus::InvalidUnsigned6 => {
                let index = original.error_index.unwrap_or(0);
                let at = operands
                    .get(index)
                    .map(|operand| self.statement_location(operand.start))
                    .unwrap_or(location);
                let text = if original.status == MatchStatus::InvalidUnsigned4 {
                    "immediate must be an integer in range [0, 15]"
                } else {
                    "immediate must be an integer in range [0, 63]"
                };
                return self.error(at, text);
            }
            MatchStatus::MissingFeature => {
                return self.missing_feature_error(location, &original.missing_features);
            }
            _ => {}
        }
        let base = operands[0].token_text().to_string();
        if base.is_empty() {
            return self.error(location, "instruction must have size higher than 0");
        }
        let suffixes: &[char] = if !base.starts_with('f') {
            &['b', 'w', 'l', 'q']
        } else {
            &['s', 'l', 't', '\0']
        };
        let memory_sizes: [u32; 4] = if !base.starts_with('f') {
            [8, 16, 32, 64]
        } else {
            [32, 64, 80, 0]
        };
        let mut has_vector = false;
        let mut memory_index: Option<usize> = None;
        for (position, operand) in operands.iter().enumerate() {
            if self.is_vector_register(operand) {
                has_vector = true;
            } else if matches!(operand.kind, ParsedKind::Memory(_)) {
                memory_index = Some(position);
                break;
            }
        }
        let mut results: Vec<MatchStatus> = Vec::new();
        let mut successful: Option<MachineInstruction> = None;
        let mut missing_features: Vec<String> = Vec::new();
        let mut attempt = operands.to_vec();
        let start = operands[0].start;
        for (position, suffix) in suffixes.iter().enumerate() {
            let mut text = base.clone();
            if *suffix != '\0' {
                text.push(*suffix);
            } else {
                text.push('\0');
            }
            attempt[0] = ParsedOperand::token(&text, start);
            if let (Some(index), true) = (memory_index, has_vector) {
                if let ParsedKind::Memory(memory) = &mut attempt[index].kind {
                    memory.size = memory_sizes[position];
                }
            }
            let mut status = MatchStatus::MnemonicFail;
            if memory_index.is_some() || !has_vector {
                let outcome = self.run_matcher(&attempt, state);
                status = outcome.status;
                if status == MatchStatus::MissingFeature {
                    missing_features = outcome.missing_features.clone();
                }
                if status == MatchStatus::Success {
                    successful = outcome.instruction;
                }
            }
            results.push(status);
        }
        let success_count = results
            .iter()
            .filter(|status| **status == MatchStatus::Success)
            .count();
        if success_count == 1 {
            let instruction = successful.unwrap_or_else(|| MachineInstruction::new(0));
            return self.finish_instruction(location, instruction, &attempt, state, prefixes);
        }
        if success_count > 1 {
            let matched: Vec<char> = results
                .iter()
                .zip(suffixes.iter())
                .filter(|(status, _)| **status == MatchStatus::Success)
                .map(|(_, suffix)| *suffix)
                .collect();
            let mut text =
                "ambiguous instructions require an explicit suffix (could be ".to_string();
            for (position, suffix) in matched.iter().enumerate() {
                if position != 0 {
                    text.push_str(", ");
                }
                if position + 1 == matched.len() {
                    text.push_str("or ");
                }
                text.push_str(&format!("'{base}{suffix}'"));
            }
            text.push(')');
            return self.error(location, &text);
        }
        if results
            .iter()
            .all(|status| *status == MatchStatus::MnemonicFail)
        {
            if original.status == MatchStatus::MnemonicFail {
                let range_end = SourceLocation {
                    buffer: location.buffer,
                    position: operands[0].end,
                };
                return self.error_range(
                    location,
                    &format!("invalid instruction mnemonic '{base}'"),
                    (location, range_end),
                );
            }
            if original.status == MatchStatus::Unsupported {
                return self.error(location, "unsupported instruction");
            }
            if let Some(index) = original.error_index {
                if index >= operands.len() {
                    return self.error(location, "too few operands for instruction");
                }
                let operand = &operands[index];
                let at = self.statement_location(operand.start);
                let end = self.statement_location(operand.end);
                return self.error_range(at, "invalid operand for instruction", (at, end));
            }
            return self.error(location, "invalid operand for instruction");
        }
        if results
            .iter()
            .filter(|status| **status == MatchStatus::Unsupported)
            .count()
            == 1
        {
            return self.error(location, "unsupported instruction");
        }
        if results
            .iter()
            .filter(|status| **status == MatchStatus::MissingFeature)
            .count()
            == 1
        {
            return self.missing_feature_error(location, &missing_features);
        }
        if results
            .iter()
            .filter(|status| **status == MatchStatus::InvalidOperand)
            .count()
            == 1
        {
            return self.error(location, "invalid operand for instruction");
        }
        self.error(
            location,
            "unknown use of instruction mnemonic without a size suffix",
        )
    }

    fn match_and_emit_intel(
        &mut self,
        location: SourceLocation,
        operands: &mut [ParsedOperand],
        state: &InstructionState,
        prefixes: u32,
    ) -> bool {
        let saved_mode = self.mode;
        let forced = self.forced_data_prefix == Some(Mode::Bits32);
        if forced {
            self.mode = Mode::Bits32;
        }
        let mut working = operands.to_vec();
        let without_size = working
            .iter()
            .position(|operand| operand.memory().is_some_and(|memory| memory.size == 0));
        let mnemonic = working[0].token_text().to_string();
        if let Some(position) = without_size {
            if matches!(mnemonic.as_str(), "call" | "jmp" | "push" | "pop") {
                let width = self.pointer_width();
                if let ParsedKind::Memory(memory) = &mut working[position].kind {
                    memory.size = width;
                }
            }
        }
        let mut results: Vec<MatchStatus> = Vec::new();
        let mut instruction: Option<MachineInstruction> = None;
        let mut missing_features: Vec<String> = Vec::new();
        let mut error_index: Option<usize> = None;
        if mnemonic == "push" && working.len() == 2 {
            if let ParsedKind::Immediate(Expression::Constant(value)) = working[1].kind {
                let bits = self.pointer_width();
                let signed = bits >= 64 || {
                    let limit = 1i64 << (bits - 1);
                    (-limit..limit).contains(&value)
                };
                let unsigned = bits >= 64 || (value as u64) < (1u64 << bits);
                if signed || unsigned {
                    let suffix = match self.mode {
                        Mode::Bits64 => "q",
                        Mode::Bits32 => "l",
                        Mode::Bits16 => "w",
                    };
                    let mut attempt = working.clone();
                    attempt[0] =
                        ParsedOperand::token(&format!("{mnemonic}{suffix}"), working[0].start);
                    let outcome = self.run_matcher_in(&attempt, state, 0);
                    if outcome.status == MatchStatus::Success {
                        instruction = outcome.instruction;
                    }
                    results.push(outcome.status);
                }
            }
        }
        let still_without_size = without_size.filter(|position| {
            working[*position]
                .memory()
                .is_some_and(|memory| memory.size == 0)
        });
        if let Some(position) = still_without_size {
            for size in [8u32, 16, 32, 64, 80, 128, 256, 512] {
                if let ParsedKind::Memory(memory) = &mut working[position].kind {
                    memory.size = size;
                }
                let last_opcode = instruction.as_ref().map(|held| held.opcode);
                let outcome = self.run_matcher(&working, state);
                if outcome.status == MatchStatus::Success {
                    instruction = outcome.instruction.clone();
                }
                let opcode = instruction.as_ref().map(|held| held.opcode);
                if results.is_empty() || last_opcode != opcode {
                    results.push(outcome.status);
                }
                if results.last() == Some(&MatchStatus::MissingFeature) {
                    missing_features = outcome.missing_features.clone();
                }
            }
            if let ParsedKind::Memory(memory) = &mut working[position].kind {
                memory.size = 0;
            }
        }
        if results.is_empty() {
            let outcome = self.run_matcher(&working, state);
            if outcome.status == MatchStatus::Success {
                instruction = outcome.instruction.clone();
            }
            error_index = outcome.error_index;
            results.push(outcome.status);
            if outcome.status == MatchStatus::MissingFeature {
                missing_features = outcome.missing_features;
            }
        }
        if let Some(position) = without_size {
            if let ParsedKind::Memory(memory) = &mut working[position].kind {
                memory.size = 0;
            }
        }
        if forced {
            self.mode = Mode::Bits16;
            self.forced_data_prefix = None;
        } else {
            self.mode = saved_mode;
        }
        if results.last() == Some(&MatchStatus::MnemonicFail) {
            let range_end = SourceLocation {
                buffer: location.buffer,
                position: operands[0].end,
            };
            return self.error_range(
                location,
                &format!("invalid instruction mnemonic '{mnemonic}'"),
                (location, range_end),
            );
        }
        let count = |status: MatchStatus| results.iter().filter(|held| **held == status).count();
        let successes = count(MatchStatus::Success);
        if successes == 1 {
            let instruction = instruction.unwrap_or_else(|| MachineInstruction::new(0));
            return self.finish_instruction(location, instruction, &working, state, prefixes);
        }
        if successes > 1 {
            let position = without_size.unwrap_or(0);
            let at = self.statement_location(working[position].start);
            let end = self.statement_location(working[position].end);
            return self.error_range(
                at,
                &format!("ambiguous operand size for instruction '{mnemonic}'"),
                (at, end),
            );
        }
        if count(MatchStatus::Unsupported) == 1 {
            return self.error(location, "unsupported instruction");
        }
        if count(MatchStatus::MissingFeature) == 1 {
            return self.missing_feature_error(location, &missing_features);
        }
        if count(MatchStatus::InvalidOperand) == 1 {
            return self.error(location, "invalid operand for instruction");
        }
        for (status, text) in [
            (
                MatchStatus::InvalidUnsigned4,
                "immediate must be an integer in range [0, 15]",
            ),
            (
                MatchStatus::InvalidUnsigned6,
                "immediate must be an integer in range [0, 63]",
            ),
        ] {
            if count(status) == 1 {
                let at = error_index
                    .and_then(|index| working.get(index))
                    .map(|operand| self.statement_location(operand.start))
                    .unwrap_or(location);
                return self.error(at, text);
            }
        }
        self.error(location, "unknown instruction mnemonic")
    }

    fn validate_instruction(
        &mut self,
        instruction: &MachineInstruction,
        operands: &[ParsedOperand],
        state: &InstructionState,
        location: SourceLocation,
    ) -> bool {
        let target = self.target;
        let description = &target.instructions[instruction.opcode];
        let flags = description.flags;
        let mnemonic = description.intel_mnemonic.as_str();
        let at = operands
            .first()
            .map(|operand| self.statement_location(operand.start))
            .unwrap_or(location);
        if matches!(
            mnemonic,
            "VFCMADDCPH" | "VFCMADDCSH" | "VFMADDCPH" | "VFMADDCSH"
        ) {
            let destination = instruction.register_at(0);
            for index in 2..instruction.operands.len() {
                if instruction.operands[index].is_register()
                    && instruction.register_at(index) == destination
                {
                    return self.warning(
                        at,
                        "Destination register should be distinct from source registers",
                    );
                }
            }
        } else if matches!(
            mnemonic,
            "VFCMULCPH" | "VFCMULCSH" | "VFMULCPH" | "VFMULCSH"
        ) {
            let destination = instruction.register_at(0);
            let first = if flags & EVEX_K != 0 { 2 } else { 1 };
            for index in first..instruction.operands.len() {
                if instruction.operands[index].is_register()
                    && instruction.register_at(index) == destination
                {
                    return self.warning(
                        at,
                        "Destination register should be distinct from source registers",
                    );
                }
            }
        } else if matches!(
            mnemonic,
            "V4FMADDPS" | "V4FMADDSS" | "V4FNMADDPS" | "V4FNMADDSS" | "VP4DPWSSDS" | "VP4DPWSSD"
        ) {
            let source = instruction.register_at(instruction.operands.len() - 5 - 1);
            let number = target
                .vector_number(source, "ZMM")
                .unwrap_or(u32::from(target.encoding(source)));
            if !number.is_multiple_of(4) {
                let name = target.register_name(source).to_string();
                let prefix: String = name.chars().take(3).collect();
                let group_start = (number / 4) * 4;
                let group_end = group_start + 3;
                return self.warning(at, &format!("source register '{name}' implicitly denotes '{prefix}{group_start}' to '{prefix}{group_end}' source group"));
            }
        } else if matches!(
            mnemonic,
            "VGATHERDPD"
                | "VGATHERDPS"
                | "VGATHERQPD"
                | "VGATHERQPS"
                | "VPGATHERDD"
                | "VPGATHERDQ"
                | "VPGATHERQD"
                | "VPGATHERQQ"
        ) {
            let number = |register: RegisterId| -> u32 {
                ["XMM", "YMM", "ZMM"]
                    .iter()
                    .find_map(|prefix| target.vector_number(register, prefix))
                    .unwrap_or(u32::from(target.encoding(register)))
            };
            if flags & ENCODING_MASK == ENCODING_EVEX {
                let destination = number(instruction.register_at(0));
                let index = number(instruction.register_at(4 + ADDRESS_INDEX));
                if destination == index {
                    return self.warning(at, "index and destination registers should be distinct");
                }
            } else {
                let destination = number(instruction.register_at(0));
                let mask = number(instruction.register_at(1));
                let index = number(instruction.register_at(3 + ADDRESS_INDEX));
                if destination == mask || destination == index || mask == index {
                    return self.warning(
                        at,
                        "mask, index, and destination registers should be distinct",
                    );
                }
            }
        } else if matches!(
            mnemonic,
            "TCMMIMFP16PS"
                | "TCMMRLFP16PS"
                | "TDPBF16PS"
                | "TDPFP16PS"
                | "TDPBSSD"
                | "TDPBSUD"
                | "TDPBUSD"
                | "TDPBUUD"
        ) {
            let destination = instruction.register_at(0);
            let first = instruction.register_at(2);
            let second = instruction.register_at(3);
            if destination == first || destination == second || first == second {
                return self.error(at, "all tmm registers must be distinct");
            }
        }
        let encoding = flags & ENCODING_MASK;
        if encoding != ENCODING_VEX && encoding != ENCODING_XOP {
            let mut high: Option<RegisterId> = None;
            let mut uses_rex = flags & REX_W != 0;
            for operand in &instruction.operands {
                if let MachineOperand::Register(register) = operand {
                    if *register == NO_REGISTER {
                        continue;
                    }
                    if target.is_high_byte(*register) {
                        high = Some(*register);
                    }
                    if target.is_low_byte_needing_rex(*register) || target.is_extended(*register) {
                        uses_rex = true;
                    }
                }
            }
            if let Some(register) = high {
                if encoding == ENCODING_EVEX
                    || state.forced_prefix == PREFIX_USE_REX2
                    || state.forced_prefix == PREFIX_USE_REX
                    || uses_rex
                {
                    let name = target.register_name(register).to_string();
                    return self.error(at, &format!("can't encode '{name}' in an instruction requiring EVEX/REX2/REX prefix"));
                }
            }
        }
        let record = description.record.as_str();
        if record == "PREFETCHIT0" || record == "PREFETCHIT1" {
            let base = instruction.operand(ADDRESS_BASE);
            let is_rip = matches!(base, MachineOperand::Register(register) if target.registers[*register as usize].record == "RIP");
            if !is_rip {
                let text = if record == "PREFETCHIT0" {
                    "'prefetchit0'"
                } else {
                    "'prefetchit1'"
                };
                return self.warning(at, &format!("{text} only supports RIP-relative address"));
            }
        }
        false
    }

    fn process_instruction(
        &mut self,
        instruction: &mut MachineInstruction,
        state: &InstructionState,
    ) -> bool {
        let target = self.target;
        if state.forced_prefix != PREFIX_USE_VEX3 && self.optimize_vex3_to_vex2(instruction) {
            return true;
        }
        if self.optimize_shift_by_one(instruction) {
            return true;
        }
        let record = target.instructions[instruction.opcode].record.clone();
        match record.as_str() {
            "JMP_1" => {
                if state.forced_displacement == 32 {
                    let replacement = if self.mode == Mode::Bits16 {
                        "JMP_2"
                    } else {
                        "JMP_4"
                    };
                    instruction.opcode = target
                        .instruction(replacement)
                        .unwrap_or(instruction.opcode);
                    return true;
                }
                false
            }
            "JCC_1" => {
                if state.forced_displacement == 32 {
                    let replacement = if self.mode == Mode::Bits16 {
                        "JCC_2"
                    } else {
                        "JCC_4"
                    };
                    instruction.opcode = target
                        .instruction(replacement)
                        .unwrap_or(instruction.opcode);
                    return true;
                }
                false
            }
            "INT" => {
                if instruction.operand(0) != &MachineOperand::Immediate(3) {
                    return false;
                }
                instruction.operands.clear();
                instruction.opcode = target.instruction("INT3").unwrap_or(instruction.opcode);
                true
            }
            _ => {
                let conditional = if let Some(rest) = record.strip_prefix("CMP") {
                    Some(format!("CCMP{rest}"))
                } else {
                    record
                        .strip_prefix("TEST")
                        .map(|rest| format!("CTEST{rest}"))
                };
                let Some(conditional) = conditional else {
                    return false;
                };
                let allowed = matches!(
                    record.as_str(),
                    "CMP64rr"
                        | "CMP64mi32"
                        | "CMP64mi8"
                        | "CMP64mr"
                        | "CMP64ri32"
                        | "CMP64ri8"
                        | "CMP64rm"
                        | "CMP32rr"
                        | "CMP32mi"
                        | "CMP32mi8"
                        | "CMP32mr"
                        | "CMP32ri"
                        | "CMP32ri8"
                        | "CMP32rm"
                        | "CMP16rr"
                        | "CMP16mi"
                        | "CMP16mi8"
                        | "CMP16mr"
                        | "CMP16ri"
                        | "CMP16ri8"
                        | "CMP16rm"
                        | "CMP8rr"
                        | "CMP8mi"
                        | "CMP8mr"
                        | "CMP8ri"
                        | "CMP8rm"
                        | "TEST64rr"
                        | "TEST64mi32"
                        | "TEST64mr"
                        | "TEST64ri32"
                        | "TEST32rr"
                        | "TEST32mi"
                        | "TEST32mr"
                        | "TEST32ri"
                        | "TEST16rr"
                        | "TEST16mi"
                        | "TEST16mr"
                        | "TEST16ri"
                        | "TEST8rr"
                        | "TEST8mi"
                        | "TEST8mr"
                        | "TEST8ri"
                );
                if !allowed || state.forced_prefix != PREFIX_USE_EVEX {
                    return false;
                }
                let Some(opcode) = target.instruction(&conditional) else {
                    return false;
                };
                instruction.flags &= !PREFIX_USE_EVEX;
                instruction.opcode = opcode;
                instruction.operands.push(MachineOperand::Immediate(0));
                instruction.operands.push(MachineOperand::Immediate(10));
                true
            }
        }
    }

    fn optimize_vex3_to_vex2(&self, instruction: &mut MachineInstruction) -> bool {
        let target = self.target;
        let description = &target.instructions[instruction.opcode];
        let record = description.record.as_str();
        let flags = description.flags;
        let mut new_opcode: Option<usize> = None;
        let (first, second) = match record {
            "VCMPPDrri" | "VCMPPDYrri" | "VCMPPSrri" | "VCMPPSYrri" | "VCMPSDrri" | "VCMPSSrri" => {
                if !instruction.operand(3).is_immediate() {
                    return false;
                }
                match instruction.immediate_at(3) & 7 {
                    0 | 3 | 4 | 7 => (1, 2),
                    _ => return false,
                }
            }
            "VMOVZPQILo2PQIrr" => {
                new_opcode = target.instruction("VMOVPQI2QIrr");
                (0, 1)
            }
            "VMOVAPDrr" | "VMOVAPDYrr" | "VMOVAPSrr" | "VMOVAPSYrr" | "VMOVDQArr"
            | "VMOVDQAYrr" | "VMOVDQUrr" | "VMOVDQUYrr" | "VMOVUPDrr" | "VMOVUPDYrr"
            | "VMOVUPSrr" | "VMOVUPSYrr" => {
                new_opcode = target.instruction(&format!("{record}_REV"));
                (0, 1)
            }
            "VMOVSDrr" | "VMOVSSrr" => {
                new_opcode = target.instruction(&format!("{record}_REV"));
                (0, 2)
            }
            _ => {
                if !description.is_commutable
                    || flags & ENCODING_MASK != ENCODING_VEX
                    || flags & OPCODE_MAP_MASK != MAP_0F
                    || form_of(flags) != MODRM_SOURCE_REGISTER
                    || flags & REX_W != 0
                    || flags & VEX_4V == 0
                    || instruction.operands.len() != 3
                {
                    return false;
                }
                if record == "VMOVHLPSrr" || record == "VUNPCKHPDrr" {
                    return false;
                }
                (1, 2)
            }
        };
        if target.is_extended(instruction.register_at(first))
            || !target.is_extended(instruction.register_at(second))
        {
            return false;
        }
        if let Some(opcode) = new_opcode {
            instruction.opcode = opcode;
        } else {
            instruction.operands.swap(first, second);
        }
        true
    }

    fn optimize_shift_by_one(&self, instruction: &mut MachineInstruction) -> bool {
        let target = self.target;
        let record = target.instructions[instruction.opcode].record.as_str();
        let operations = ["RCR", "RCL", "ROR", "ROL", "SAR", "SHR", "SHL"];
        let Some(operation) = operations
            .iter()
            .find(|candidate| record.starts_with(**candidate))
        else {
            return false;
        };
        let rest = &record[operation.len()..];
        let digits: String = rest
            .chars()
            .take_while(|character| character.is_ascii_digit())
            .collect();
        if !matches!(digits.as_str(), "8" | "16" | "32" | "64") {
            return false;
        }
        let tail = &rest[digits.len()..];
        let form = match tail.chars().next() {
            Some(character @ ('r' | 'm')) => character,
            _ => return false,
        };
        let after = &tail[1..];
        let Some(suffix) = after.strip_prefix('i') else {
            return false;
        };
        let rotate_through_carry = *operation == "RCR" || *operation == "RCL";
        let allowed = if rotate_through_carry {
            matches!(suffix, "" | "_EVEX" | "_ND")
        } else {
            matches!(suffix, "" | "_EVEX" | "_NF" | "_ND" | "_NF_ND")
        };
        if !allowed {
            return false;
        }
        let replacement = format!("{operation}{digits}{form}1{suffix}");
        let Some(opcode) = target.instruction(&replacement) else {
            return false;
        };
        if instruction.operands.last() != Some(&MachineOperand::Immediate(1)) {
            return false;
        }
        instruction.opcode = opcode;
        instruction.operands.pop();
        true
    }

    fn emit_instruction(&mut self, instruction: &MachineInstruction, location: SourceLocation) {
        if let Some((section, _)) = self.context.current_section() {
            self.context.sections[section].has_instructions = true;
        }
        let mode = self.mode;
        self.streamer
            .emit_instruction(&mut self.context, instruction, mode, location);
    }
}

pub struct TextStreamer {
    pub target: &'static Target,
    pub syntax: Syntax,
    pub show_encoding: bool,
    pub hexadecimal_immediates: bool,
    pub comment_column: usize,
    output: String,
    column: usize,
    comments: String,
    explicit_comments: String,
    emitted_section_directive: bool,
    frame_open: bool,
    frames: usize,
}

impl TextStreamer {
    pub fn new(
        target: &'static Target,
        syntax: Syntax,
        show_encoding: bool,
        hexadecimal_immediates: bool,
        comment_column: usize,
    ) -> TextStreamer {
        TextStreamer {
            target,
            syntax,
            show_encoding,
            hexadecimal_immediates,
            comment_column,
            output: String::new(),
            column: 0,
            comments: String::new(),
            explicit_comments: String::new(),
            emitted_section_directive: false,
            frame_open: false,
            frames: 0,
        }
    }

    fn write(&mut self, text: &str) {
        for character in text.chars() {
            match character {
                '\n' => self.column = 0,
                '\t' => self.column = (self.column / 8 + 1) * 8,
                _ => self.column += 1,
            }
        }
        self.output.push_str(text);
    }

    fn pad_to_comment_column(&mut self) {
        let spaces = if self.column < self.comment_column {
            self.comment_column - self.column
        } else {
            1
        };
        let padding = " ".repeat(spaces);
        self.write(&padding);
    }

    fn end_of_line(&mut self) {
        if !self.explicit_comments.is_empty() {
            let comments = std::mem::take(&mut self.explicit_comments);
            self.write(&comments);
        }
        if self.comments.is_empty() {
            self.write("\n");
            return;
        }
        let comments = std::mem::take(&mut self.comments);
        let mut rest = comments.as_str();
        loop {
            self.pad_to_comment_column();
            let position = rest.find('\n').unwrap_or(rest.len());
            let line = format!("# {}\n", &rest[..position]);
            self.write(&line);
            rest = if position < rest.len() {
                &rest[position + 1..]
            } else {
                ""
            };
            if rest.is_empty() {
                break;
            }
        }
    }

    fn line(&mut self, text: &str) {
        self.write(text);
        self.end_of_line();
    }

    fn symbol(&self, context: &AssemblyContext, symbol: usize) -> String {
        symbol_text_with(self.target, &context.symbols[symbol].name, self.syntax)
    }

    fn expression(&self, expression: &Expression) -> String {
        expression_text(self.target, expression, 0, self.syntax)
    }

    fn section_text(&self, context: &AssemblyContext, section: usize, subsection: i64) -> String {
        let held = &context.sections[section];
        if context.triple.format == ObjectFormat::Coff {
            let mut out = String::new();
            if held.group.is_none() && matches!(held.name.as_str(), ".text" | ".data" | ".bss") {
                out.push('\t');
                out.push_str(&held.name);
                out.push('\n');
                return out;
            }
            out.push_str("\t.section\t");
            out.push_str(&held.name);
            out.push_str(",\"");
            let characteristics = held.characteristics;
            if characteristics & 0x40 != 0 {
                out.push('d');
            }
            if characteristics & 0x80 != 0 {
                out.push('b');
            }
            if characteristics & 0x2000_0000 != 0 {
                out.push('x');
            }
            if characteristics & 0x8000_0000 != 0 {
                out.push('w');
            } else if characteristics & 0x2000_0000 == 0 {
                out.push('r');
            }
            if characteristics & 0x0200_0000 != 0 {
                out.push('D');
            }
            if characteristics & 0x800 != 0 {
                out.push('i');
            }
            if characteristics & 0x2 != 0 {
                out.push('n');
            }
            out.push('"');
            if let Some(group) = &held.group {
                out.push(',');
                let selection = match held.coff_selection {
                    1 => "one_only",
                    2 => "discard",
                    3 => "same_size",
                    4 => "same_contents",
                    5 => "associative",
                    6 => "largest",
                    7 => "newest",
                    _ => "",
                };
                out.push_str(selection);
                out.push(',');
                out.push_str(group);
            }
            out.push('\n');
            return out;
        }
        if held.unique.is_none() && matches!(held.name.as_str(), ".text" | ".data" | ".bss") {
            let mut out = format!("\t{}", held.name);
            if subsection != 0 {
                out.push_str(&format!("\t{subsection}"));
            }
            out.push('\n');
            return out;
        }
        let mut out = format!("\t.section\t{},\"", elf_section_name_text(&held.name));
        let flags = held.flags;
        for (bit, letter) in [
            (ELF_FLAG_ALLOCATE, 'a'),
            (ELF_FLAG_EXCLUDE, 'e'),
            (ELF_FLAG_EXECUTE, 'x'),
            (ELF_FLAG_WRITE, 'w'),
            (ELF_FLAG_MERGE, 'M'),
            (ELF_FLAG_STRINGS, 'S'),
            (ELF_FLAG_THREAD_LOCAL, 'T'),
            (ELF_FLAG_LINK_ORDER, 'o'),
            (ELF_FLAG_GROUP, 'G'),
            (ELF_FLAG_GNU_RETAIN, 'R'),
        ] {
            if flags & bit != 0 {
                out.push(letter);
            }
        }
        if context.triple.is_64_bit() && flags & ELF_FLAG_X86_64_LARGE != 0 {
            out.push('l');
        }
        out.push_str("\",@");
        let type_name = match held.section_type {
            ELF_SECTION_INITIALIZER_ARRAY => "init_array".to_string(),
            ELF_SECTION_FINALIZER_ARRAY => "fini_array".to_string(),
            ELF_SECTION_PREINITIALIZER_ARRAY => "preinit_array".to_string(),
            ELF_SECTION_NO_BITS => "nobits".to_string(),
            ELF_SECTION_NOTE => "note".to_string(),
            ELF_SECTION_PROGRAM_BITS => "progbits".to_string(),
            ELF_SECTION_X86_64_UNWIND => "unwind".to_string(),
            0x6fff_4c00 => "llvm_odrtab".to_string(),
            0x6fff_4c01 => "llvm_linker_options".to_string(),
            ELF_SECTION_LLVM_CALL_GRAPH_PROFILE => "llvm_call_graph_profile".to_string(),
            0x6fff_4c04 => "llvm_dependent_libraries".to_string(),
            0x6fff_4c05 => "llvm_sympart".to_string(),
            0x6fff_4c0a => "llvm_bb_addr_map".to_string(),
            0x6fff_4c0b => "llvm_offloading".to_string(),
            0x6fff_4c0c => "llvm_lto".to_string(),
            0x6fff_4c0d => "llvm_jt_sizes".to_string(),
            0x6fff_4c0e => "llvm_cfi_jump_table".to_string(),
            other => format!("0x{other:X}"),
        };
        out.push_str(&type_name);
        if held.entry_size != 0 {
            out.push_str(&format!(",{}", held.entry_size));
        }
        if flags & ELF_FLAG_LINK_ORDER != 0 {
            out.push(',');
            match &held.linked_symbol {
                Some(name) => out.push_str(&elf_section_name_text(name)),
                None => out.push('0'),
            }
        }
        if flags & ELF_FLAG_GROUP != 0 {
            out.push(',');
            out.push_str(&elf_section_name_text(held.group.as_deref().unwrap_or("")));
            if held.comdat {
                out.push_str(",comdat");
            }
        }
        if let Some(unique) = held.unique {
            out.push_str(&format!(",unique,{unique}"));
        }
        out.push('\n');
        if subsection != 0 {
            out.push_str(&format!("\t.subsection\t{subsection}\n"));
        }
        out
    }

    fn encoding_comment(
        &mut self,
        context: &mut AssemblyContext,
        instruction: &MachineInstruction,
        mode: Mode,
        location: SourceLocation,
    ) -> bool {
        let encoder = Encoder::new(self.target, mode);
        let encoded = match encoder.encode(instruction) {
            Ok(encoded) => encoded,
            Err(diagnostic) => {
                let cause = diagnostic
                    .context
                    .iter()
                    .find_map(|line| line.strip_prefix("Cause: "))
                    .unwrap_or("the instruction could not be encoded")
                    .to_string();
                context.report(MessageKind::Error, Some(location), &cause);
                return false;
            }
        };
        let bytes = &encoded.bytes;
        let mut map = vec![0usize; bytes.len() * 8];
        for (index, fixup) in encoded.fixups.iter().enumerate() {
            let size = fixup.kind.size() * 8;
            for bit in 0..size {
                let at = fixup.offset * 8 + bit;
                if at < map.len() {
                    map[at] = index + 1;
                }
            }
        }
        let mut text = String::from("encoding: [");
        for (index, byte) in bytes.iter().enumerate() {
            if index != 0 {
                text.push(',');
            }
            let first = map[index * 8];
            let uniform = (1..8).all(|bit| map[index * 8 + bit] == first);
            if uniform {
                if first == 0 {
                    text.push_str(&format!("0x{byte:02x}"));
                } else if *byte != 0 {
                    text.push_str(&format!(
                        "0x{byte:02x}'{}'",
                        (b'A' + (first - 1) as u8) as char
                    ));
                } else {
                    text.push((b'A' + (first - 1) as u8) as char);
                }
            } else {
                text.push_str("0b");
                for bit in (0..8).rev() {
                    let entry = map[index * 8 + bit];
                    if entry != 0 {
                        text.push((b'A' + (entry - 1) as u8) as char);
                    } else {
                        text.push(if (byte >> bit) & 1 != 0 { '1' } else { '0' });
                    }
                }
            }
        }
        text.push_str("]\n");
        for (index, fixup) in encoded.fixups.iter().enumerate() {
            let value = self.expression(&fixup.expression);
            let kind = match fixup.kind {
                FixupKind::Literal(number) => format!("relocation type: {number}"),
                FixupKind::Data1 | FixupKind::Data2 | FixupKind::Data4 | FixupKind::Data8
                    if fixup.pc_relative =>
                {
                    format!("kind: FK_PCRel_{}", fixup.kind.size())
                }
                other => format!("kind: {}", other.name()),
            };
            let separator = ", ";
            text.push_str(&format!(
                "  fixup {} - offset: {}, value: {value}{separator}{kind}\n",
                (b'A' + index as u8) as char,
                fixup.offset
            ));
        }
        self.comments.push_str(&text);
        true
    }

    fn register_name_for_dwarf(&self, context: &AssemblyContext, number: i64) -> String {
        let flavour = if context.triple.is_64_bit() {
            0
        } else if context.triple.format == ObjectFormat::MachObject {
            2
        } else {
            1
        };
        for (index, register) in self.target.registers.iter().enumerate().skip(1) {
            if register.dwarf_numbers.get(flavour).copied() == Some(number) {
                let name = self.target.register_name(index as RegisterId);
                return if self.syntax == Syntax::Att {
                    format!("%{name}")
                } else {
                    name.to_string()
                };
            }
        }
        number.to_string()
    }

    fn require_frame(&mut self, context: &mut AssemblyContext, location: SourceLocation) -> bool {
        if !self.frame_open {
            context.report(
                MessageKind::Error,
                Some(location),
                "this directive must appear between .cfi_startproc and .cfi_endproc directives",
            );
            return false;
        }
        true
    }

    fn alignment_directive(
        &mut self,
        alignment: u64,
        value: Option<i64>,
        value_size: u32,
        maximum: u64,
    ) {
        let truncate = |value: i64| -> u64 {
            let size = value_size.min(8);
            if size >= 8 {
                value as u64
            } else {
                (value as u64) & ((1u64 << (size * 8)) - 1)
            }
        };
        if alignment.is_power_of_two() {
            let mut text = match value_size {
                2 => ".p2alignw ".to_string(),
                4 => ".p2alignl ".to_string(),
                _ => "\t.p2align\t".to_string(),
            };
            text.push_str(&alignment.trailing_zeros().to_string());
            if value.is_some() || maximum != 0 {
                match value {
                    Some(fill) => text.push_str(&format!(", 0x{:x}", truncate(fill))),
                    None => text.push_str(", "),
                }
                if maximum != 0 {
                    text.push_str(&format!(", {maximum}"));
                }
            }
            self.line(&text);
            return;
        }
        let mut text = match value_size {
            2 => ".balignw".to_string(),
            4 => ".balignl".to_string(),
            _ => ".balign".to_string(),
        };
        text.push_str(&format!(" {alignment}"));
        match value {
            Some(fill) => text.push_str(&format!(", {}", truncate(fill))),
            None => {
                if maximum != 0 {
                    text.push_str(", ");
                }
            }
        }
        if maximum != 0 {
            text.push_str(&format!(", {maximum}"));
        }
        self.line(&text);
    }

    fn define_label(&mut self, context: &mut AssemblyContext, symbol: usize) {
        let current = context.current_section().map(|(section, _)| section);
        let held = &mut context.symbols[symbol];
        held.defined = true;
        held.section = current;
    }
}

impl Streamer for TextStreamer {
    fn initialize_sections(&mut self, context: &mut AssemblyContext) {
        let section = if context.triple.format == ObjectFormat::Coff {
            let key = (".text".to_string(), String::new(), u64::MAX);
            match context.section_index.get(&key) {
                Some(found) => *found,
                None => {
                    let mut created = Section::new(".text", 0, 0);
                    created.characteristics = 0x6000_0020;
                    let index = context.sections.len();
                    context.sections.push(created);
                    context.section_index.insert(key, index);
                    index
                }
            }
        } else {
            context.elf_section(
                ".text",
                ELF_SECTION_PROGRAM_BITS,
                ELF_FLAG_EXECUTE | ELF_FLAG_ALLOCATE,
                0,
                "",
                None,
            )
        };
        self.switch_section(context, section, 0);
    }

    fn switch_section(&mut self, context: &mut AssemblyContext, section: usize, subsection: i64) {
        let current = context.current_section();
        if !self.emitted_section_directive || current != Some((section, subsection)) {
            self.emitted_section_directive = true;
            let text = self.section_text(context, section, subsection);
            self.write(&text);
        }
        if let Some(top) = context.section_stack.last_mut() {
            top.1 = top.0;
            top.0 = Some((section, subsection));
        }
    }

    fn emit_label(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        location: Option<SourceLocation>,
    ) {
        let held = &mut context.symbols[symbol];
        if held.redefinable {
            held.value = None;
            held.defined = false;
            held.redefinable = false;
        }
        if !held.is_undefined() || held.is_variable() {
            let name = held.name.clone();
            context.report(
                MessageKind::Error,
                location,
                &format!("symbol '{name}' is already defined"),
            );
            return;
        }
        self.define_label(context, symbol);
        let text = format!("{}:", self.symbol(context, symbol));
        self.line(&text);
    }

    fn emit_assignment(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        value: &Expression,
    ) {
        let text = format!(
            "{} = {}",
            self.symbol(context, symbol),
            self.expression(value)
        );
        self.line(&text);
        context.symbols[symbol].value = Some(value.clone());
    }

    fn emit_conditional_assignment(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        value: &Expression,
    ) {
        let text = format!(
            ".lto_set_conditional {}, {}",
            self.symbol(context, symbol),
            self.expression(value)
        );
        self.line(&text);
    }

    fn emit_weak_reference(&mut self, context: &mut AssemblyContext, alias: usize, symbol: usize) {
        let text = format!(
            ".weakref {}, {}",
            self.symbol(context, alias),
            self.symbol(context, symbol)
        );
        self.line(&text);
    }

    fn emit_symbol_attribute(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        attribute: SymbolAttribute,
    ) -> bool {
        let type_name = match attribute {
            SymbolAttribute::TypeFunction => Some("function"),
            SymbolAttribute::TypeIndirectFunction => Some("gnu_indirect_function"),
            SymbolAttribute::TypeObject => Some("object"),
            SymbolAttribute::ThreadLocal => Some("tls_object"),
            SymbolAttribute::TypeCommon => Some("common"),
            SymbolAttribute::TypeNoType => Some("notype"),
            SymbolAttribute::TypeGnuUniqueObject => Some("gnu_unique_object"),
            _ => None,
        };
        if let Some(type_name) = type_name {
            if context.triple.format != ObjectFormat::Elf {
                return false;
            }
            let text = format!("\t.type\t{},@{type_name}", self.symbol(context, symbol));
            self.line(&text);
            return true;
        }
        let directive = match attribute {
            SymbolAttribute::Global => "\t.globl\t",
            SymbolAttribute::Hidden => "\t.hidden\t",
            SymbolAttribute::Internal => "\t.internal\t",
            SymbolAttribute::LazyReference => "\t.lazy_reference\t",
            SymbolAttribute::Local => "\t.local\t",
            SymbolAttribute::NoDeadStrip => {
                if context.triple.format != ObjectFormat::MachObject {
                    return false;
                }
                "\t.no_dead_strip\t"
            }
            SymbolAttribute::SymbolResolver => "\t.symbol_resolver\t",
            SymbolAttribute::AlternateEntry => "\t.alt_entry\t",
            SymbolAttribute::PrivateExtern => "\t.private_extern\t",
            SymbolAttribute::Protected => "\t.protected\t",
            SymbolAttribute::Reference => "\t.reference\t",
            SymbolAttribute::Extern => "\t.extern\t",
            SymbolAttribute::Weak => "\t.weak\t",
            SymbolAttribute::WeakDefinition => "\t.weak_definition\t",
            SymbolAttribute::WeakReference => "\t.weak\t",
            SymbolAttribute::WeakDefinitionAutoPrivate => "\t.weak_def_can_be_hidden\t",
            SymbolAttribute::Cold => return false,
            SymbolAttribute::Memtag => "\t.memtag\t",
            SymbolAttribute::WeakAntiDependency => "\t.weak_anti_dep\t",
            _ => return false,
        };
        match attribute {
            SymbolAttribute::Global => {
                context.symbols[symbol].binding = Some(SymbolBinding::Global)
            }
            SymbolAttribute::Weak => context.symbols[symbol].binding = Some(SymbolBinding::Weak),
            SymbolAttribute::Local => context.symbols[symbol].binding = Some(SymbolBinding::Local),
            _ => {}
        }
        let text = format!("{directive}{}", self.symbol(context, symbol));
        self.line(&text);
        true
    }

    fn emit_symbol_version(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        name: &str,
        keep_original: bool,
    ) {
        let mut text = format!(".symver {}, {name}", self.symbol(context, symbol));
        if !keep_original && !name.contains("@@@") {
            text.push_str(", remove");
        }
        self.line(&text);
    }

    fn emit_elf_size(&mut self, context: &mut AssemblyContext, symbol: usize, value: &Expression) {
        let text = format!(
            "\t.size\t{}, {}",
            self.symbol(context, symbol),
            self.expression(value)
        );
        self.line(&text);
        context.symbols[symbol].size = Some(value.clone());
    }

    fn emit_common_symbol(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        size: u64,
        alignment: u64,
        local: bool,
    ) {
        let name = self.symbol(context, symbol);
        let text = if local {
            let mut text = format!("\t.lcomm\t{name},{size}");
            if alignment > 1 {
                if context.triple.format == ObjectFormat::Coff {
                    text.push_str(&format!(",{alignment}"));
                } else {
                    text.push_str(&format!(",{}", alignment.trailing_zeros()));
                }
            }
            text
        } else if context.triple.format == ObjectFormat::Coff
            || context.triple.format == ObjectFormat::MachObject
        {
            format!("\t.comm\t{name},{size},{}", alignment.trailing_zeros())
        } else {
            format!("\t.comm\t{name},{size},{alignment}")
        };
        context.symbols[symbol].common = Some((size, alignment));
        self.line(&text);
    }

    fn emit_bytes(&mut self, _context: &mut AssemblyContext, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        if data.len() != 1 {
            let (directive, body) = if data.last() == Some(&0) {
                ("\t.asciz\t", &data[..data.len() - 1])
            } else {
                ("\t.ascii\t", data)
            };
            let text = format!("{directive}{}", quoted_string(body));
            self.line(&text);
            return;
        }
        for byte in data {
            let text = format!("\t.byte\t{byte}");
            self.line(&text);
        }
    }

    fn emit_value(
        &mut self,
        _context: &mut AssemblyContext,
        value: &Expression,
        size: u32,
        _location: SourceLocation,
    ) {
        let directive = match size {
            1 => "\t.byte\t",
            2 => "\t.short\t",
            4 => "\t.long\t",
            8 => "\t.quad\t",
            _ => "",
        };
        if directive.is_empty() {
            return;
        }
        let text = format!("{directive}{}", self.expression(value));
        self.line(&text);
    }

    fn emit_leb128(&mut self, context: &mut AssemblyContext, value: &Expression, signed: bool) {
        if let Some(constant) = context.evaluate_absolute(value, None) {
            let mut bytes: Vec<u8> = Vec::new();
            if signed {
                let mut remaining = constant;
                loop {
                    let byte = (remaining & 0x7f) as u8;
                    remaining >>= 7;
                    let done = (remaining == 0 && byte & 0x40 == 0)
                        || (remaining == -1 && byte & 0x40 != 0);
                    bytes.push(if done { byte } else { byte | 0x80 });
                    if done {
                        break;
                    }
                }
            } else {
                let mut remaining = constant as u64;
                loop {
                    let byte = (remaining & 0x7f) as u8;
                    remaining >>= 7;
                    bytes.push(if remaining != 0 { byte | 0x80 } else { byte });
                    if remaining == 0 {
                        break;
                    }
                }
            }
            self.emit_bytes(context, &bytes);
            return;
        }
        let text = format!(
            "\t.{}leb128 {}",
            if signed { "s" } else { "u" },
            self.expression(value)
        );
        self.line(&text);
    }

    fn emit_fill_bytes(
        &mut self,
        context: &mut AssemblyContext,
        count: &Expression,
        value: u64,
        _location: SourceLocation,
    ) {
        if context.evaluate_absolute(count, None) == Some(0) {
            return;
        }
        let mut text = format!("\t.zero\t{}", self.expression(count));
        if value != 0 {
            text.push_str(&format!(",{}", value as i32));
        }
        self.line(&text);
    }

    fn emit_fill(
        &mut self,
        _context: &mut AssemblyContext,
        count: &Expression,
        size: i64,
        value: i64,
        _location: SourceLocation,
    ) {
        let text = format!(
            "\t.fill\t{}, {size}, 0x{:x}",
            self.expression(count),
            value as u64 & 0xffff_ffff
        );
        self.line(&text);
    }

    fn emit_value_to_alignment(
        &mut self,
        _context: &mut AssemblyContext,
        alignment: u64,
        value: i64,
        value_size: u32,
        maximum: u64,
    ) {
        self.alignment_directive(alignment, Some(value), value_size, maximum);
    }

    fn emit_code_alignment(
        &mut self,
        _context: &mut AssemblyContext,
        alignment: u64,
        maximum: u64,
        _mode: Mode,
    ) {
        self.alignment_directive(alignment, None, 1, maximum);
    }

    fn emit_value_to_offset(
        &mut self,
        _context: &mut AssemblyContext,
        offset: &Expression,
        value: u8,
        _location: SourceLocation,
    ) {
        let text = format!(".org {}, {value}", self.expression(offset));
        self.line(&text);
    }

    fn emit_instruction(
        &mut self,
        context: &mut AssemblyContext,
        instruction: &MachineInstruction,
        mode: Mode,
        location: SourceLocation,
    ) {
        if self.show_encoding && !self.encoding_comment(context, instruction, mode, location) {
            return;
        }
        let printer = Printer::new(
            self.target,
            PrintOptions {
                syntax: self.syntax,
                mode,
                hexadecimal_immediates: self.hexadecimal_immediates,
                branch_as_address: false,
            },
        );
        let printer = printer.with_comments();
        let text = printer.print(instruction, 0);
        self.comments.push_str(&printer.take_comments());
        self.write(&text);
        if !self.comments.is_empty() && !self.comments.ends_with('\n') {
            self.comments.push('\n');
        }
        self.end_of_line();
    }

    fn emit_nops(
        &mut self,
        _context: &mut AssemblyContext,
        _count: i64,
        _control: i64,
        _mode: Mode,
        _location: SourceLocation,
    ) {
    }

    fn emit_cfi(
        &mut self,
        context: &mut AssemblyContext,
        instruction: CfiInstruction,
        location: SourceLocation,
    ) {
        let text = match instruction {
            CfiInstruction::Sections(exception, debug, frame) => {
                let mut names: Vec<&str> = Vec::new();
                if exception {
                    names.push(".eh_frame");
                }
                if debug {
                    names.push(".debug_frame");
                }
                if frame {
                    names.push(".sframe");
                }
                format!("\t.cfi_sections {}", names.join(", "))
            }
            CfiInstruction::StartProcedure(simple) => {
                if self.frame_open {
                    context.report(
                        MessageKind::Error,
                        Some(location),
                        "starting new .cfi frame before finishing the previous one",
                    );
                    return;
                }
                self.frame_open = true;
                self.frames += 1;
                if simple {
                    "\t.cfi_startproc simple".to_string()
                } else {
                    "\t.cfi_startproc".to_string()
                }
            }
            CfiInstruction::EndProcedure => {
                if !self.require_frame(context, location) {
                    return;
                }
                self.frame_open = false;
                "\t.cfi_endproc".to_string()
            }
            other => {
                if !self.require_frame(context, location) {
                    return;
                }
                match other {
                    CfiInstruction::DefineFrame(register, offset) => format!(
                        "\t.cfi_def_cfa {}, {offset}",
                        self.register_name_for_dwarf(context, register)
                    ),
                    CfiInstruction::DefineFrameOffset(offset) => {
                        format!("\t.cfi_def_cfa_offset {offset}")
                    }
                    CfiInstruction::AdjustFrameOffset(offset) => {
                        format!("\t.cfi_adjust_cfa_offset {offset}")
                    }
                    CfiInstruction::DefineFrameRegister(register) => format!(
                        "\t.cfi_def_cfa_register {}",
                        self.register_name_for_dwarf(context, register)
                    ),
                    CfiInstruction::Offset(register, offset) => format!(
                        "\t.cfi_offset {}, {offset}",
                        self.register_name_for_dwarf(context, register)
                    ),
                    CfiInstruction::RelativeOffset(register, offset) => format!(
                        "\t.cfi_rel_offset {}, {offset}",
                        self.register_name_for_dwarf(context, register)
                    ),
                    CfiInstruction::RememberState => "\t.cfi_remember_state".to_string(),
                    CfiInstruction::RestoreState => "\t.cfi_restore_state".to_string(),
                    CfiInstruction::SameValue(register) => format!(
                        "\t.cfi_same_value {}",
                        self.register_name_for_dwarf(context, register)
                    ),
                    CfiInstruction::Restore(register) => format!(
                        "\t.cfi_restore {}",
                        self.register_name_for_dwarf(context, register)
                    ),
                    CfiInstruction::ReturnColumn(register) => format!(
                        "\t.cfi_return_column {}",
                        self.register_name_for_dwarf(context, register)
                    ),
                    CfiInstruction::SignalFrame => "\t.cfi_signal_frame".to_string(),
                    CfiInstruction::Undefined(register) => format!(
                        "\t.cfi_undefined {}",
                        self.register_name_for_dwarf(context, register)
                    ),
                    CfiInstruction::Register(first, second) => format!(
                        "\t.cfi_register {}, {}",
                        self.register_name_for_dwarf(context, first),
                        self.register_name_for_dwarf(context, second)
                    ),
                    CfiInstruction::WindowSave => "\t.cfi_window_save".to_string(),
                    CfiInstruction::ValueOffset(register, offset) => format!(
                        "\t.cfi_val_offset {}, {offset}",
                        self.register_name_for_dwarf(context, register)
                    ),
                    CfiInstruction::GnuArgumentsSize(size) => {
                        format!("\t.cfi_escape 0x2e, 0x{:02x}", size as u8)
                    }
                    _ => String::new(),
                }
            }
        };
        self.line(&text);
    }

    fn emit_cfi_escape(
        &mut self,
        context: &mut AssemblyContext,
        values: &[u8],
        location: SourceLocation,
    ) {
        if !self.require_frame(context, location) {
            return;
        }
        let pieces: Vec<String> = values
            .iter()
            .map(|value| format!("0x{value:02x}"))
            .collect();
        let text = format!("\t.cfi_escape {}", pieces.join(", "));
        self.line(&text);
    }

    fn emit_cfi_personality(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        encoding: i64,
        lsda: bool,
    ) {
        let name = self.symbol(context, symbol);
        let text = if lsda {
            format!("\t.cfi_lsda {encoding}, {name}")
        } else {
            format!("\t.cfi_personality {encoding}, {name}")
        };
        self.line(&text);
    }

    fn emit_cfi_label(
        &mut self,
        context: &mut AssemblyContext,
        name: &str,
        location: SourceLocation,
    ) {
        if !self.require_frame(context, location) {
            return;
        }
        let text = format!("\t.cfi_label {name}");
        self.line(&text);
    }

    fn emit_bundle_align_mode(&mut self, _context: &mut AssemblyContext, power: u32) {
        let text = format!("\t.bundle_align_mode {power}");
        self.line(&text);
    }

    fn emit_bundle_lock(
        &mut self,
        _context: &mut AssemblyContext,
        align_to_end: bool,
        _location: SourceLocation,
    ) {
        if align_to_end {
            self.line("\t.bundle_lock align_to_end");
        } else {
            self.line("\t.bundle_lock");
        }
    }

    fn emit_bundle_unlock(&mut self, _context: &mut AssemblyContext, _location: SourceLocation) {
        self.line("\t.bundle_unlock");
    }

    fn emit_relocation_directive(
        &mut self,
        _context: &mut AssemblyContext,
        offset: &Expression,
        name: &str,
        value: Option<&Expression>,
        _location: SourceLocation,
    ) {
        let mut text = format!("\t.reloc {}, {name}", self.expression(offset));
        if let Some(value) = value {
            text.push_str(&format!(", {}", self.expression(value)));
        }
        self.line(&text);
    }

    fn emit_identifier(&mut self, _context: &mut AssemblyContext, text: &[u8]) {
        let line = format!("\t.ident\t{}", quoted_string(text));
        self.line(&line);
    }

    fn emit_file(&mut self, _context: &mut AssemblyContext, name: &[u8]) {
        let line = format!("\t.file\t{}", quoted_string(name));
        self.line(&line);
    }

    fn emit_raw_text(&mut self, _context: &mut AssemblyContext, text: &str) {
        let trimmed = text.strip_suffix('\n').unwrap_or(text);
        self.line(trimmed);
    }

    fn emit_address_significance(&mut self, context: &mut AssemblyContext, symbol: Option<usize>) {
        match symbol {
            Some(symbol) => {
                let text = format!("\t.addrsig_sym {}", self.symbol(context, symbol));
                self.line(&text);
            }
            None => self.line("\t.addrsig"),
        }
    }

    fn emit_mode_change(&mut self, _context: &mut AssemblyContext, mode: Mode) {
        let text = match mode {
            Mode::Bits16 => "\t.code16\n",
            Mode::Bits32 => "\t.code32\n",
            Mode::Bits64 => "\t.code64\n",
        };
        self.write(text);
    }

    fn emit_coff_directive(
        &mut self,
        _context: &mut AssemblyContext,
        text: &str,
        _symbol: Option<usize>,
        _value: i64,
    ) {
        self.line(text);
    }

    fn add_blank_line(&mut self) {
        self.end_of_line();
    }

    fn add_explicit_comment(&mut self, text: &str) {
        if text == ";" {
            return;
        }
        if let Some(rest) = text.strip_prefix("//") {
            self.explicit_comments.push_str(&format!("\t#{rest}"));
        } else if text.starts_with("/*") {
            let body = &text[2..text.len().saturating_sub(2).max(2)];
            let lines: Vec<&str> = body.split(['\r', '\n']).collect();
            for (index, line) in lines.iter().enumerate() {
                self.explicit_comments.push_str(&format!("\t#{line}"));
                if index + 1 < lines.len() {
                    self.explicit_comments.push('\n');
                }
            }
        } else if text.starts_with('#') {
            self.explicit_comments.push('\t');
            self.explicit_comments
                .push_str(text.trim_end_matches(['\n', '\r']));
        }
        if text.ends_with('\n') {
            let comments = std::mem::take(&mut self.explicit_comments);
            self.write(&comments);
            self.write("\n");
        }
    }

    fn finish(&mut self, _context: &mut AssemblyContext) {}

    fn output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.output).into_bytes()
    }
}

pub struct NullStreamer {
    frame_open: bool,
}

impl NullStreamer {
    pub fn new() -> NullStreamer {
        NullStreamer { frame_open: false }
    }
}

impl Default for NullStreamer {
    fn default() -> Self {
        NullStreamer::new()
    }
}

impl Streamer for NullStreamer {
    fn initialize_sections(&mut self, context: &mut AssemblyContext) {
        let section = context.elf_section(
            ".text",
            ELF_SECTION_PROGRAM_BITS,
            ELF_FLAG_EXECUTE | ELF_FLAG_ALLOCATE,
            0,
            "",
            None,
        );
        self.switch_section(context, section, 0);
    }
    fn switch_section(&mut self, context: &mut AssemblyContext, section: usize, subsection: i64) {
        if let Some(top) = context.section_stack.last_mut() {
            top.1 = top.0;
            top.0 = Some((section, subsection));
        }
    }
    fn emit_label(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        location: Option<SourceLocation>,
    ) {
        let held = &mut context.symbols[symbol];
        if held.redefinable {
            held.value = None;
            held.defined = false;
            held.redefinable = false;
        }
        if !held.is_undefined() || held.is_variable() {
            let name = held.name.clone();
            context.report(
                MessageKind::Error,
                location,
                &format!("symbol '{name}' is already defined"),
            );
            return;
        }
        let current = context.current_section().map(|(section, _)| section);
        context.symbols[symbol].defined = true;
        context.symbols[symbol].section = current;
    }
    fn emit_assignment(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        value: &Expression,
    ) {
        context.symbols[symbol].value = Some(value.clone());
    }
    fn emit_conditional_assignment(
        &mut self,
        _context: &mut AssemblyContext,
        _symbol: usize,
        _value: &Expression,
    ) {
    }
    fn emit_weak_reference(
        &mut self,
        _context: &mut AssemblyContext,
        _alias: usize,
        _symbol: usize,
    ) {
    }
    fn emit_symbol_attribute(
        &mut self,
        _context: &mut AssemblyContext,
        _symbol: usize,
        _attribute: SymbolAttribute,
    ) -> bool {
        true
    }
    fn emit_symbol_version(
        &mut self,
        _context: &mut AssemblyContext,
        _symbol: usize,
        _name: &str,
        _keep_original: bool,
    ) {
    }
    fn emit_elf_size(
        &mut self,
        _context: &mut AssemblyContext,
        _symbol: usize,
        _value: &Expression,
    ) {
    }
    fn emit_common_symbol(
        &mut self,
        context: &mut AssemblyContext,
        symbol: usize,
        size: u64,
        alignment: u64,
        _local: bool,
    ) {
        context.symbols[symbol].common = Some((size, alignment));
    }
    fn emit_bytes(&mut self, _context: &mut AssemblyContext, _data: &[u8]) {}
    fn emit_value(
        &mut self,
        _context: &mut AssemblyContext,
        _value: &Expression,
        _size: u32,
        _location: SourceLocation,
    ) {
    }
    fn emit_leb128(&mut self, _context: &mut AssemblyContext, _value: &Expression, _signed: bool) {}
    fn emit_fill_bytes(
        &mut self,
        _context: &mut AssemblyContext,
        _count: &Expression,
        _value: u64,
        _location: SourceLocation,
    ) {
    }
    fn emit_fill(
        &mut self,
        _context: &mut AssemblyContext,
        _count: &Expression,
        _size: i64,
        _value: i64,
        _location: SourceLocation,
    ) {
    }
    fn emit_value_to_alignment(
        &mut self,
        _context: &mut AssemblyContext,
        _alignment: u64,
        _value: i64,
        _value_size: u32,
        _maximum: u64,
    ) {
    }
    fn emit_code_alignment(
        &mut self,
        _context: &mut AssemblyContext,
        _alignment: u64,
        _maximum: u64,
        _mode: Mode,
    ) {
    }
    fn emit_value_to_offset(
        &mut self,
        _context: &mut AssemblyContext,
        _offset: &Expression,
        _value: u8,
        _location: SourceLocation,
    ) {
    }
    fn emit_instruction(
        &mut self,
        _context: &mut AssemblyContext,
        _instruction: &MachineInstruction,
        _mode: Mode,
        _location: SourceLocation,
    ) {
    }
    fn emit_nops(
        &mut self,
        _context: &mut AssemblyContext,
        _count: i64,
        _control: i64,
        _mode: Mode,
        _location: SourceLocation,
    ) {
    }
    fn emit_cfi(
        &mut self,
        context: &mut AssemblyContext,
        instruction: CfiInstruction,
        location: SourceLocation,
    ) {
        match instruction {
            CfiInstruction::Sections(..) => {}
            CfiInstruction::StartProcedure(_) => {
                if self.frame_open {
                    context.report(
                        MessageKind::Error,
                        Some(location),
                        "starting new .cfi frame before finishing the previous one",
                    );
                    return;
                }
                self.frame_open = true;
            }
            CfiInstruction::EndProcedure => {
                if !self.frame_open {
                    context.report(MessageKind::Error, Some(location), "this directive must appear between .cfi_startproc and .cfi_endproc directives");
                }
                self.frame_open = false;
            }
            _ => {
                if !self.frame_open {
                    context.report(MessageKind::Error, Some(location), "this directive must appear between .cfi_startproc and .cfi_endproc directives");
                }
            }
        }
    }
    fn emit_cfi_escape(
        &mut self,
        _context: &mut AssemblyContext,
        _values: &[u8],
        _location: SourceLocation,
    ) {
    }
    fn emit_cfi_personality(
        &mut self,
        _context: &mut AssemblyContext,
        _symbol: usize,
        _encoding: i64,
        _lsda: bool,
    ) {
    }
    fn emit_cfi_label(
        &mut self,
        _context: &mut AssemblyContext,
        _name: &str,
        _location: SourceLocation,
    ) {
    }
    fn emit_bundle_align_mode(&mut self, _context: &mut AssemblyContext, _power: u32) {}
    fn emit_bundle_lock(
        &mut self,
        _context: &mut AssemblyContext,
        _align_to_end: bool,
        _location: SourceLocation,
    ) {
    }
    fn emit_bundle_unlock(&mut self, _context: &mut AssemblyContext, _location: SourceLocation) {}
    fn emit_relocation_directive(
        &mut self,
        _context: &mut AssemblyContext,
        _offset: &Expression,
        _name: &str,
        _value: Option<&Expression>,
        _location: SourceLocation,
    ) {
    }
    fn emit_identifier(&mut self, _context: &mut AssemblyContext, _text: &[u8]) {}
    fn emit_file(&mut self, _context: &mut AssemblyContext, _name: &[u8]) {}
    fn emit_raw_text(&mut self, _context: &mut AssemblyContext, _text: &str) {}
    fn emit_address_significance(
        &mut self,
        _context: &mut AssemblyContext,
        _symbol: Option<usize>,
    ) {
    }
    fn emit_mode_change(&mut self, _context: &mut AssemblyContext, _mode: Mode) {}
    fn emit_coff_directive(
        &mut self,
        _context: &mut AssemblyContext,
        _text: &str,
        _symbol: Option<usize>,
        _value: i64,
    ) {
    }
    fn add_blank_line(&mut self) {}
    fn add_explicit_comment(&mut self, _text: &str) {}
    fn finish(&mut self, _context: &mut AssemblyContext) {}
    fn output(&mut self) -> Vec<u8> {
        Vec::new()
    }
}

pub fn assemble_source(
    options: &AssemblerOptions,
    source: &[u8],
    file_reader: Option<FileReader>,
) -> Result<AssemblyResult, Diagnostic> {
    let target = target()?;
    let triple = TargetTriple::parse(&options.triple);
    let output_syntax = options.output_syntax.unwrap_or(options.input_syntax);
    let streamer: Box<dyn Streamer> = match options.output {
        OutputKind::Assembly => Box::new(TextStreamer::new(
            target,
            output_syntax,
            options.show_encoding,
            options.hexadecimal_immediates,
            options.comment_column,
        )),
        OutputKind::Null => Box::new(NullStreamer::new()),
        OutputKind::Object => Box::new(NullStreamer::new()),
    };
    let _ = triple;
    let mut assembler = Assembler::new(options.clone(), source, streamer)?;
    assembler.file_reader = file_reader;
    let failed = assembler.run();
    let output = assembler.streamer.output();
    Ok(AssemblyResult {
        output,
        messages: std::mem::take(&mut assembler.context.messages),
        buffers: std::mem::take(&mut assembler.lexer.buffers),
        failed,
    })
}

pub fn message_position(
    buffers: &[SourceBuffer],
    location: SourceLocation,
) -> (String, usize, usize, String) {
    let Some(buffer) = buffers.get(location.buffer) else {
        return (String::new(), 0, 0, String::new());
    };
    let text = &buffer.text;
    let position = location.position.min(text.len());
    let line_start = text[..position]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map(|at| at + 1)
        .unwrap_or(0);
    let line_end = text[position..]
        .iter()
        .position(|byte| *byte == b'\n' || *byte == b'\r')
        .map(|at| at + position)
        .unwrap_or(text.len());
    let line = text[..line_start]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        + 1;
    let column = position - line_start + 1;
    let source = String::from_utf8_lossy(&text[line_start..line_end]).into_owned();
    (buffer.name.clone(), line, column, source)
}

pub fn render_messages(result: &AssemblyResult) -> Vec<Diagnostic> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for message in &result.messages {
        let (code, sentence, severity) = match message.kind {
            MessageKind::Error => (
                "EI302",
                "The assembly source has an error.",
                crate::diag::Severity::Error,
            ),
            MessageKind::Warning => (
                "EI303",
                "The assembly source has a warning.",
                crate::diag::Severity::Warning,
            ),
            MessageKind::Note => (
                "EI304",
                "The assembly source has a note.",
                crate::diag::Severity::Info,
            ),
        };
        let mut diagnostic = fail(code, sentence);
        diagnostic.severity = severity;
        diagnostic = diagnostic.with_context(format!("Cause: {}", message.text));
        if let Some(location) = message.location {
            let (mut file, mut line, column, source) = message_position(&result.buffers, location);
            if let Some((marker_location, name, number)) = &message.marker {
                if marker_location.buffer == location.buffer {
                    let (_, marker_line, _, _) =
                        message_position(&result.buffers, *marker_location);
                    file = name.clone();
                    line = (number - 1 + line as i64 - marker_line as i64).max(0) as usize;
                }
            }
            diagnostic = diagnostic.with_location(crate::diag::Location {
                file,
                line: line as u32,
                column: column as u32,
            });
            let start_column = column - 1;
            let mut marker: Vec<u8> = vec![b' '; source.len() + 1];
            if let Some((range_start, range_end)) = message.range {
                if range_start.buffer == location.buffer && range_end.buffer == location.buffer {
                    let line_start = location.position - start_column;
                    let begin = range_start
                        .position
                        .saturating_sub(line_start)
                        .min(source.len());
                    let end = range_end
                        .position
                        .saturating_sub(line_start)
                        .min(source.len());
                    for item in marker.iter_mut().take(end).skip(begin) {
                        *item = b'~';
                    }
                }
            }
            let caret = start_column.min(source.len());
            marker[caret] = b'^';
            while marker.last() == Some(&b' ') {
                marker.pop();
            }
            let mut expanded_source = String::new();
            let mut expanded_marker = String::new();
            let source_bytes = source.as_bytes();
            let mut column_out = 0usize;
            for (index, byte) in source_bytes.iter().enumerate() {
                if *byte == b'\t' {
                    loop {
                        expanded_source.push(' ');
                        if index < marker.len() {
                            expanded_marker.push(marker[index] as char);
                        }
                        column_out += 1;
                        if column_out.is_multiple_of(8) {
                            break;
                        }
                    }
                } else {
                    expanded_source.push(*byte as char);
                    if index < marker.len() {
                        expanded_marker.push(marker[index] as char);
                    }
                    column_out += 1;
                }
            }
            if marker.len() > source_bytes.len() {
                for item in &marker[source_bytes.len()..] {
                    expanded_marker.push(*item as char);
                }
            }
            diagnostic = diagnostic.with_context(format!("Source: {expanded_source}"));
            diagnostic = diagnostic.with_context(format!("Marker: {expanded_marker}"));
        }
        diagnostics.push(diagnostic);
    }
    diagnostics
}
