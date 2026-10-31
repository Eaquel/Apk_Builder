use super::*;

pub const TABLE: &[u8] = include_bytes!("Tables/Armv7.bin");

pub fn table() -> Result<&'static crate::compilers::table_gen::Table, Diagnostic> {
    static CELL: OnceLock<Result<crate::compilers::table_gen::Table, Diagnostic>> = OnceLock::new();
    CELL.get_or_init(|| read_table(TABLE))
        .as_ref()
        .map_err(|problem| problem.clone())
}

pub fn target() -> Result<&'static Target, Diagnostic> {
    static CELL: OnceLock<Result<Target, Diagnostic>> = OnceLock::new();
    CELL.get_or_init(|| {
        let mut records = BTreeMap::new();
        for record in read_table(TABLE)?.records {
            records.insert(record.name.clone(), record);
        }
        Target::load_with(Isa::Armv7, records, "InstTemplate")
    })
    .as_ref()
    .map_err(|problem| problem.clone())
}

pub fn matcher() -> &'static Matcher {
    static CELL: OnceLock<Matcher> = OnceLock::new();
    CELL.get_or_init(|| build_matcher(target().expect("the ARMv7 tables load")))
}

pub const THUMB_MODE: u32 = 1 << 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fixup32 {
    LdstPcrel12,
    T2LdstPcrel12,
    PcrelTenUnscaled,
    PcrelTen,
    T2PcrelTen,
    PcrelNine,
    T2PcrelNine,
    LdstAbs12,
    ThumbAdrPcrel10,
    AdrPcrel12,
    T2AdrPcrel12,
    CondBranch,
    UncondBranch,
    T2CondBranch,
    T2UncondBranch,
    ThumbBr,
    UncondBl,
    CondBl,
    Blx,
    ThumbBl,
    ThumbBlx,
    ThumbCb,
    ThumbCp,
    ThumbBcc,
    MovtHi16,
    MovwLo16,
    T2MovtHi16,
    T2MovwLo16,
    ThumbUpper815,
    ThumbUpper07,
    ThumbLower815,
    ThumbLower07,
    ModImm,
    T2SoImm,
}

impl Fixup32 {
    pub fn name(self) -> &'static str {
        match self {
            Fixup32::LdstPcrel12 => "fixup_arm_ldst_pcrel_12",
            Fixup32::T2LdstPcrel12 => "fixup_t2_ldst_pcrel_12",
            Fixup32::PcrelTenUnscaled => "fixup_arm_pcrel_10_unscaled",
            Fixup32::PcrelTen => "fixup_arm_pcrel_10",
            Fixup32::T2PcrelTen => "fixup_t2_pcrel_10",
            Fixup32::PcrelNine => "fixup_arm_pcrel_9",
            Fixup32::T2PcrelNine => "fixup_t2_pcrel_9",
            Fixup32::LdstAbs12 => "fixup_arm_ldst_abs_12",
            Fixup32::ThumbAdrPcrel10 => "fixup_thumb_adr_pcrel_10",
            Fixup32::AdrPcrel12 => "fixup_arm_adr_pcrel_12",
            Fixup32::T2AdrPcrel12 => "fixup_t2_adr_pcrel_12",
            Fixup32::CondBranch => "fixup_arm_condbranch",
            Fixup32::UncondBranch => "fixup_arm_uncondbranch",
            Fixup32::T2CondBranch => "fixup_t2_condbranch",
            Fixup32::T2UncondBranch => "fixup_t2_uncondbranch",
            Fixup32::ThumbBr => "fixup_arm_thumb_br",
            Fixup32::UncondBl => "fixup_arm_uncondbl",
            Fixup32::CondBl => "fixup_arm_condbl",
            Fixup32::Blx => "fixup_arm_blx",
            Fixup32::ThumbBl => "fixup_arm_thumb_bl",
            Fixup32::ThumbBlx => "fixup_arm_thumb_blx",
            Fixup32::ThumbCb => "fixup_arm_thumb_cb",
            Fixup32::ThumbCp => "fixup_arm_thumb_cp",
            Fixup32::ThumbBcc => "fixup_arm_thumb_bcc",
            Fixup32::MovtHi16 => "fixup_arm_movt_hi16",
            Fixup32::MovwLo16 => "fixup_arm_movw_lo16",
            Fixup32::T2MovtHi16 => "fixup_t2_movt_hi16",
            Fixup32::T2MovwLo16 => "fixup_t2_movw_lo16",
            Fixup32::ThumbUpper815 => "fixup_arm_thumb_upper_8_15",
            Fixup32::ThumbUpper07 => "fixup_arm_thumb_upper_0_7",
            Fixup32::ThumbLower815 => "fixup_arm_thumb_lower_8_15",
            Fixup32::ThumbLower07 => "fixup_arm_thumb_lower_0_7",
            Fixup32::ModImm => "fixup_arm_mod_imm",
            Fixup32::T2SoImm => "fixup_t2_so_imm",
        }
    }

    pub fn bits(self) -> usize {
        match self {
            Fixup32::ThumbAdrPcrel10
            | Fixup32::ThumbCp
            | Fixup32::ThumbBcc
            | Fixup32::ThumbUpper815
            | Fixup32::ThumbUpper07
            | Fixup32::ThumbLower815
            | Fixup32::ThumbLower07 => 8,
            Fixup32::CondBranch
            | Fixup32::UncondBranch
            | Fixup32::UncondBl
            | Fixup32::CondBl
            | Fixup32::Blx => 24,
            Fixup32::ThumbBr | Fixup32::ThumbCb => 16,
            Fixup32::MovtHi16 | Fixup32::MovwLo16 | Fixup32::T2MovtHi16 | Fixup32::T2MovwLo16 => 20,
            Fixup32::ModImm => 12,
            Fixup32::T2SoImm => 26,
            _ => 32,
        }
    }

    pub fn pc_relative(self) -> bool {
        !matches!(
            self,
            Fixup32::LdstAbs12
                | Fixup32::MovtHi16
                | Fixup32::MovwLo16
                | Fixup32::T2MovtHi16
                | Fixup32::T2MovwLo16
                | Fixup32::ThumbUpper815
                | Fixup32::ThumbUpper07
                | Fixup32::ThumbLower815
                | Fixup32::ThumbLower07
                | Fixup32::ModImm
                | Fixup32::T2SoImm
        )
    }
}

fn swap_half_words(value: u32) -> u32 {
    value.rotate_right(16)
}

pub fn adjust_fixup_value(
    kind: Fixup32,
    value: u64,
    resolved: bool,
    thumb_two: bool,
) -> Result<u32, &'static str> {
    let range_error = "out of range pc-relative fixup value";
    let signed = |held: u64, bits: u32| -> bool {
        let shifted = (held as i64) << (64 - bits) >> (64 - bits);
        shifted == held as i64
    };
    Ok(match kind {
        Fixup32::MovtHi16 | Fixup32::MovwLo16 => {
            let mut held = value as u32;
            if kind == Fixup32::MovtHi16 && resolved {
                held >>= 16;
            }
            (((held & 0xf000) >> 12) << 16) | (held & 0xfff)
        }
        Fixup32::T2MovtHi16 | Fixup32::T2MovwLo16 => {
            let mut held = value as u32;
            if kind == Fixup32::T2MovtHi16 && resolved {
                held >>= 16;
            }
            let high4 = (held & 0xf000) >> 12;
            let i = (held & 0x800) >> 11;
            let mid3 = (held & 0x700) >> 8;
            let low8 = held & 0xff;
            swap_half_words((high4 << 16) | (i << 26) | (mid3 << 12) | low8)
        }
        Fixup32::ThumbUpper815 => {
            if resolved {
                (value as u32 & 0xff00_0000) >> 24
            } else {
                value as u32 & 0xff
            }
        }
        Fixup32::ThumbUpper07 => {
            if resolved {
                (value as u32 & 0x00ff_0000) >> 16
            } else {
                value as u32 & 0xff
            }
        }
        Fixup32::ThumbLower815 => {
            if resolved {
                (value as u32 & 0xff00) >> 8
            } else {
                value as u32 & 0xff
            }
        }
        Fixup32::ThumbLower07 => value as u32 & 0xff,
        Fixup32::LdstPcrel12 | Fixup32::T2LdstPcrel12 | Fixup32::LdstAbs12 => {
            let mut held = value as i64;
            if kind == Fixup32::LdstPcrel12 {
                held -= 4;
            }
            if kind != Fixup32::LdstAbs12 {
                held -= 4;
            }
            let add = held >= 0;
            let magnitude = held.unsigned_abs();
            if magnitude >= 4096 {
                return Err(range_error);
            }
            let result = magnitude as u32 | (u32::from(add) << 23);
            if kind == Fixup32::T2LdstPcrel12 {
                swap_half_words(result)
            } else {
                result
            }
        }
        Fixup32::AdrPcrel12 => {
            let mut held = (value as i64) - 8;
            let mut opcode = 4u32;
            if held < 0 {
                held = -held;
                opcode = 2;
            }
            match so_immediate(held as u32) {
                Some(encoded) => encoded | (opcode << 21),
                None => return Err(range_error),
            }
        }
        Fixup32::T2AdrPcrel12 => {
            let mut held = (value as i64) - 4;
            let mut opcode = 0u32;
            if held < 0 {
                held = -held;
                opcode = 5;
            }
            let held = held as u32;
            let mut out = opcode << 21;
            out |= (held & 0x800) << 15;
            out |= (held & 0x700) << 4;
            out |= held & 0xff;
            swap_half_words(out)
        }
        Fixup32::CondBranch
        | Fixup32::UncondBranch
        | Fixup32::UncondBl
        | Fixup32::CondBl
        | Fixup32::Blx => {
            let held = (value as i64) - 8;
            if !signed(held as u64, 26) {
                return Err("Relocation out of range");
            }
            if (kind == Fixup32::Blx && held % 2 != 0) || (kind != Fixup32::Blx && held % 4 != 0) {
                return Err("Relocation not aligned");
            }
            (0xff_ffff & (held >> 2)) as u32
        }
        Fixup32::T2UncondBranch => {
            let held = (value as i64) - 4;
            if !signed(held as u64, 25) {
                return Err("Relocation out of range");
            }
            let held = (held >> 1) as u32;
            let i = held & 0x80_0000 != 0;
            let mut j1 = held & 0x40_0000 != 0;
            let mut j2 = held & 0x20_0000 != 0;
            j1 ^= i;
            j2 ^= i;
            let mut out = 0u32;
            out |= u32::from(i) << 26;
            out |= u32::from(!j1) << 13;
            out |= u32::from(!j2) << 11;
            out |= (held & 0x1f_f800) << 5;
            out |= held & 0x7ff;
            swap_half_words(out)
        }
        Fixup32::T2CondBranch => {
            let held = (value as i64) - 4;
            if !signed(held as u64, 21) {
                return Err("Relocation out of range");
            }
            let held = (held >> 1) as u32;
            let mut out = 0u32;
            out |= (held & 0x8_0000) << 7;
            out |= (held & 0x4_0000) >> 7;
            out |= (held & 0x2_0000) >> 4;
            out |= (held & 0x1_f800) << 5;
            out |= held & 0x7ff;
            swap_half_words(out)
        }
        Fixup32::ThumbBl => {
            let held = (value as i64) - 4;
            if !signed(held as u64, 25) || (!thumb_two && !signed(held as u64, 23)) {
                return Err("Relocation out of range");
            }
            let offset = (held >> 1) as u32;
            let sign = (offset & 0x80_0000) >> 23;
            let i1 = (offset & 0x40_0000) >> 22;
            let j1 = (i1 ^ 1) ^ sign;
            let i2 = (offset & 0x20_0000) >> 21;
            let j2 = (i2 ^ 1) ^ sign;
            let imm10 = (offset & 0x1f_f800) >> 11;
            let imm11 = offset & 0x7ff;
            let first = (sign << 10) | imm10;
            let second = (j1 << 13) | (j2 << 11) | imm11;
            (second << 16) | first
        }
        Fixup32::ThumbBlx => {
            if !value.is_multiple_of(4) {
                return Err("misaligned ARM call destination");
            }
            let offset = ((value as i64) - 4) as u32 >> 2;
            let sign = (offset & 0x40_0000) >> 22;
            let i1 = (offset & 0x20_0000) >> 21;
            let j1 = (i1 ^ 1) ^ sign;
            let i2 = (offset & 0x10_0000) >> 20;
            let j2 = (i2 ^ 1) ^ sign;
            let imm10_high = (offset & 0xf_fc00) >> 10;
            let imm10_low = offset & 0x3ff;
            let first = (sign << 10) | imm10_high;
            let second = (j1 << 13) | (j2 << 11) | (imm10_low << 1);
            (second << 16) | first
        }
        Fixup32::ThumbAdrPcrel10 | Fixup32::ThumbCp => {
            if !thumb_two && resolved {
                let offset = (value as i64) - 4;
                if offset & 3 != 0 {
                    return Err("misaligned pc-relative fixup value");
                }
                if !(0..=1020).contains(&offset) {
                    return Err(range_error);
                }
            }
            (((value as i64) - 4) >> 2) as u32 & 0xff
        }
        Fixup32::ThumbCb => {
            if (value as i64) < 2 || value > 0x82 || value & 1 != 0 {
                return Err(range_error);
            }
            let binary = ((value - 4) >> 1) as u32;
            ((binary & 0x20) << 4) | ((binary & 0x1f) << 3)
        }
        Fixup32::ThumbBr => {
            if !thumb_two {
                let offset = (value as i64) - 4;
                if !(-2048..=2046).contains(&offset) {
                    return Err(range_error);
                }
            }
            ((((value as i64) - 4) >> 1) & 0x7ff) as u32
        }
        Fixup32::ThumbBcc => {
            if !thumb_two {
                let offset = (value as i64) - 4;
                if !(-256..=254).contains(&offset) {
                    return Err(range_error);
                }
            }
            ((((value as i64) - 4) >> 1) & 0xff) as u32
        }
        Fixup32::PcrelTenUnscaled => {
            let mut held = (value as i64) - 8;
            let add = held >= 0;
            if held < 0 {
                held = -held;
            }
            if held >= 256 {
                return Err(range_error);
            }
            let held = held as u32;
            ((held & 0xf) | ((held & 0xf0) << 4)) | (u32::from(add) << 23)
        }
        Fixup32::PcrelTen | Fixup32::T2PcrelTen => {
            let mut held = value as i64 - 4;
            if kind == Fixup32::PcrelTen {
                held -= 4;
            }
            let add = held >= 0;
            if held < 0 {
                held = -held;
            }
            held >>= 2;
            if held >= 256 {
                return Err(range_error);
            }
            let result = held as u32 | (u32::from(add) << 23);
            if kind == Fixup32::T2PcrelTen {
                swap_half_words(result)
            } else {
                result
            }
        }
        Fixup32::PcrelNine | Fixup32::T2PcrelNine => {
            let mut held = value as i64 - 4;
            if kind == Fixup32::PcrelNine {
                held -= 4;
            }
            let add = held >= 0;
            if held < 0 {
                held = -held;
            }
            if held & 1 != 0 {
                return Err("invalid value for this fixup");
            }
            held >>= 1;
            if held >= 256 {
                return Err(range_error);
            }
            let result = held as u32 | (u32::from(add) << 23);
            if kind == Fixup32::T2PcrelNine {
                swap_half_words(result)
            } else {
                result
            }
        }
        Fixup32::ModImm => match so_immediate(value as u32) {
            Some(encoded) if encoded >> 12 == 0 => encoded,
            _ => return Err("out of range immediate fixup value"),
        },
        Fixup32::T2SoImm => match thumb_so_immediate(value as u32) {
            Some(encoded) => {
                let mut result = 0u32;
                result |= (encoded & 0x800) << 15;
                result |= (encoded & 0x700) << 4;
                result |= encoded & 0xff;
                swap_half_words(result)
            }
            None => return Err("out of range immediate fixup value"),
        },
    })
}

pub fn elf_relocation(
    kind: Option<Fixup32>,
    size: usize,
    specifier: &str,
    pc_relative: bool,
) -> Result<u32, &'static str> {
    let specifier = specifier.to_ascii_lowercase();
    let spec = match specifier.as_str() {
        "lower16" | "upper16" | "lower0_7" | "lower8_15" | "upper0_7" | "upper8_15" => "",
        other => other,
    };
    if pc_relative {
        return Ok(match kind {
            None if size == 4 => match spec {
                "" => 3,
                "gottpoff" => 107,
                "got_prel" => 96,
                "prel31" => 42,
                _ => return Err("invalid fixup for 4-byte pc-relative data relocation"),
            },
            None => return Err("unsupported relocation type"),
            Some(Fixup32::Blx) | Some(Fixup32::UncondBl) => {
                if spec == "tlscall" {
                    91
                } else {
                    28
                }
            }
            Some(Fixup32::CondBl) | Some(Fixup32::CondBranch) | Some(Fixup32::UncondBranch) => 29,
            Some(Fixup32::T2CondBranch) => 51,
            Some(Fixup32::T2UncondBranch) => 30,
            Some(Fixup32::MovtHi16) => 46,
            Some(Fixup32::MovwLo16) => 45,
            Some(Fixup32::T2MovtHi16) => 50,
            Some(Fixup32::T2MovwLo16) => 49,
            Some(Fixup32::ThumbUpper815) => 135,
            Some(Fixup32::ThumbUpper07) => 134,
            Some(Fixup32::ThumbLower815) => 133,
            Some(Fixup32::ThumbLower07) => 132,
            Some(Fixup32::ThumbBr) => 102,
            Some(Fixup32::ThumbBcc) => 103,
            Some(Fixup32::ThumbBl) | Some(Fixup32::ThumbBlx) => {
                if spec == "tlscall" {
                    93
                } else {
                    10
                }
            }
            Some(Fixup32::LdstPcrel12) => 4,
            Some(Fixup32::PcrelTenUnscaled) => 64,
            Some(Fixup32::T2LdstPcrel12) => 54,
            Some(Fixup32::AdrPcrel12) => 58,
            Some(Fixup32::ThumbAdrPcrel10) => 11,
            Some(Fixup32::T2AdrPcrel12) => 53,
            Some(_) => return Err("unsupported relocation type"),
        });
    }
    Ok(match kind {
        None => match (size, spec) {
            (1, "") => 8,
            (2, "") => 5,
            (4, "") => 2,
            (4, "got") => 26,
            (4, "tlsgd") => 104,
            (4, "tpoff") => 108,
            (4, "gottpoff") => 107,
            (4, "gotoff") => 24,
            (4, "got_prel") => 96,
            (4, "target1") => 38,
            (4, "target2") => 41,
            (4, "prel31") => 42,
            (4, "sbrel") => 9,
            (4, "tlsldo") => 106,
            (4, "tlscall") => 91,
            (4, "tlsdesc") => 90,
            (4, "tlsldm") => 105,
            (4, "tlsdescseq") => 92,
            (1, _) => return Err("invalid fixup for 1-byte data relocation"),
            (2, _) => return Err("invalid fixup for 2-byte data relocation"),
            _ => return Err("invalid fixup for 4-byte data relocation"),
        },
        Some(Fixup32::CondBranch) | Some(Fixup32::UncondBranch) => 29,
        Some(Fixup32::MovtHi16) => match spec {
            "" => 44,
            "sbrel" => 79,
            _ => return Err("invalid fixup for ARM MOVT instruction"),
        },
        Some(Fixup32::MovwLo16) => match spec {
            "" => 43,
            "sbrel" => 78,
            _ => return Err("invalid fixup for ARM MOVW instruction"),
        },
        Some(Fixup32::T2MovtHi16) => match spec {
            "" => 48,
            "sbrel" => 82,
            _ => return Err("invalid fixup for Thumb MOVT instruction"),
        },
        Some(Fixup32::T2MovwLo16) => match spec {
            "" => 47,
            "sbrel" => 81,
            _ => return Err("invalid fixup for Thumb MOVW instruction"),
        },
        Some(Fixup32::ThumbUpper815) => 135,
        Some(Fixup32::ThumbUpper07) => 134,
        Some(Fixup32::ThumbLower815) => 133,
        Some(Fixup32::ThumbLower07) => 132,
        Some(_) => return Err("unsupported relocation type"),
    })
}

const CONDITION_AL: u32 = 14;

pub fn condition_code(text: &str) -> Option<u32> {
    Some(match text {
        "eq" => 0,
        "ne" => 1,
        "hs" | "cs" => 2,
        "lo" | "cc" => 3,
        "mi" => 4,
        "pl" => 5,
        "vs" => 6,
        "vc" => 7,
        "hi" => 8,
        "ls" => 9,
        "ge" => 10,
        "lt" => 11,
        "gt" => 12,
        "le" => 13,
        "al" => 14,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shift {
    None,
    Asr,
    Lsl,
    Lsr,
    Ror,
    Rrx,
    Uxtw,
}

impl Shift {
    fn code(self) -> u32 {
        match self {
            Shift::None => 0,
            Shift::Asr => 1,
            Shift::Lsl => 2,
            Shift::Lsr => 3,
            Shift::Ror => 4,
            Shift::Rrx => 5,
            Shift::Uxtw => 6,
        }
    }

    fn named(text: &str) -> Option<Shift> {
        Some(match text.to_ascii_lowercase().as_str() {
            "asl" | "lsl" => Shift::Lsl,
            "lsr" => Shift::Lsr,
            "asr" => Shift::Asr,
            "ror" => Shift::Ror,
            "rrx" => Shift::Rrx,
            "uxtw" => Shift::Uxtw,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Memory {
    pub base: RegisterId,
    pub offset_immediate: Option<Expression>,
    pub offset_register: RegisterId,
    pub shift: Shift,
    pub shift_immediate: u32,
    pub alignment: u32,
    pub negative: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operand {
    Condition(u32),
    CarrySet(RegisterId),
    Register(RegisterId),
    Immediate(Expression),
    ModifiedImmediate {
        bits: u32,
        rotation: u32,
    },
    ShiftedRegister {
        shift: Shift,
        source: RegisterId,
        amount_register: RegisterId,
        amount: u32,
    },
    ShiftedImmediate {
        shift: Shift,
        source: RegisterId,
        amount: u32,
    },
    Memory(Memory),
    PostIndexRegister {
        register: RegisterId,
        add: bool,
        shift: Shift,
        amount: u32,
    },
    RegisterList(Vec<RegisterId>),
    ConstantPool(Expression),
    Bitfield {
        lsb: u32,
        width: u32,
    },
    ShifterImmediate {
        arithmetic: bool,
        amount: u32,
    },
    RotateImmediate(u32),
    MemoryBarrier(u32),
    InstructionBarrier(u32),
    ItMask(u32),
}

impl Operand {
    fn constant(&self) -> Option<i64> {
        match self {
            Operand::Immediate(Expression::Constant(value)) => Some(*value),
            _ => None,
        }
    }
}

fn in_class(target: &Target, class: &str, register: RegisterId) -> bool {
    target
        .register_classes
        .get(class)
        .is_some_and(|held| held.contains(register))
}

fn split_arguments(text: &str) -> (String, Vec<String>) {
    match text.split_once('<') {
        Some((name, rest)) => (
            name.trim().to_string(),
            rest.trim_end_matches('>')
                .split(',')
                .map(|argument| argument.trim().to_string())
                .collect(),
        ),
        None => (text.trim().to_string(), Vec::new()),
    }
}

fn rotate_right(value: u32, amount: u32) -> u32 {
    value.rotate_right(amount)
}

fn so_immediate_rotate(value: u32) -> u32 {
    if value & !255 == 0 {
        return 0;
    }
    let trailing = value.trailing_zeros();
    let rotation = trailing & !1;
    if rotate_right(value, rotation) & !255 == 0 {
        return (32 - rotation) & 31;
    }
    if value & 63 != 0 {
        let second = (value & !63).trailing_zeros() & !1;
        if rotate_right(value, second) & !255 == 0 {
            return (32 - second) & 31;
        }
    }
    (32 - rotation) & 31
}

pub fn so_immediate(value: u32) -> Option<u32> {
    if value & !255 == 0 {
        return Some(value);
    }
    let rotation = so_immediate_rotate(value);
    if rotate_right(!255u32, rotation) & value != 0 {
        return None;
    }
    Some(value.rotate_left(rotation) | ((rotation >> 1) << 8))
}

fn thumb_so_immediate_splat(value: u32) -> Option<u32> {
    if value & 0xffff_ff00 == 0 {
        return Some(value);
    }
    let shifted = if value & 0xff == 0 { value >> 8 } else { value };
    let immediate = shifted & 0xff;
    let repeated = immediate | (immediate << 16);
    if shifted == repeated {
        return Some((if shifted == value { 1 } else { 2 } << 8) | immediate);
    }
    if shifted == (repeated | (repeated << 8)) {
        return Some((3 << 8) | immediate);
    }
    None
}

fn thumb_so_immediate_rotate(value: u32) -> Option<u32> {
    let rotation = value.leading_zeros();
    if rotation >= 24 {
        return None;
    }
    if 0xff00_0000u32.rotate_right(rotation) & value == value {
        return Some((value.rotate_right(24 - rotation) & 0x7f) | ((rotation + 8) << 7));
    }
    None
}

pub fn thumb_so_immediate(value: u32) -> Option<u32> {
    thumb_so_immediate_splat(value).or_else(|| thumb_so_immediate_rotate(value))
}

fn so_register_opcode(shift: Shift, amount: u32) -> i64 {
    i64::from(shift.code() | (amount << 3))
}

fn addressing_mode_2(add: bool, amount: u32, shift: Shift) -> i64 {
    i64::from(amount | (u32::from(!add) << 12) | (shift.code() << 13))
}

fn addressing_mode_3(add: bool, amount: u32) -> i64 {
    i64::from((u32::from(!add) << 8) | (amount & 0xff))
}

fn split_offset(value: i64) -> (bool, u32) {
    let narrow = value as i32;
    if narrow == i32::MIN {
        (false, 0)
    } else if narrow < 0 {
        (false, narrow.unsigned_abs())
    } else {
        (true, narrow as u32)
    }
}

pub fn predicate(target: &Target, name: &str, operand: &Operand, thumb: bool) -> bool {
    let _ = thumb;
    let (base, arguments) = split_arguments(name);
    let number = |index: usize| -> i64 {
        arguments
            .get(index)
            .and_then(|argument| argument.parse::<i64>().ok())
            .unwrap_or(0)
    };
    let constant = operand.constant();
    let is_immediate = matches!(operand, Operand::Immediate(_));
    let general = |register: RegisterId| in_class(target, "GPR", register);
    let memory = match operand {
        Operand::Memory(held)
            if general(held.base)
                && (held.offset_register == NO_REGISTER || general(held.offset_register)) =>
        {
            Some(held)
        }
        _ => None,
    };
    let minimum = i64::from(i32::MIN);
    let low = |register: RegisterId| in_class(target, "tGPR", register);
    match base.as_str() {
        "isCondCode" | "isITCondCode" => matches!(operand, Operand::Condition(_)),
        "isCCOut" => matches!(operand, Operand::CarrySet(_)),
        "isImm" => is_immediate,
        "isImmediate" => constant.is_some_and(|value| value >= number(0) && value <= number(1)),
        "isARMBranchTarget" => match operand {
            Operand::Immediate(Expression::Constant(value)) => value % 4 == 0,
            Operand::Immediate(_) => true,
            _ => false,
        },
        "isThumbBranchTarget" => match operand {
            Operand::Immediate(Expression::Constant(value)) => value % 2 == 0,
            Operand::Immediate(_) => true,
            _ => false,
        },
        "isModImm" => matches!(operand, Operand::ModifiedImmediate { .. }),
        "isModImmNot" => constant.is_some_and(|value| so_immediate(!(value as u32)).is_some()),
        "isModImmNeg" => constant.is_some_and(|value| {
            so_immediate(value as u32).is_none()
                && so_immediate((value as u32).wrapping_neg()).is_some()
        }),
        "isImm0_4095Neg" => constant.is_some_and(|value| {
            if (value >> 32) > 0 {
                return false;
            }
            let negated = (value as u32).wrapping_neg();
            negated > 0 && negated < 4096
        }),
        "isImm0_65535Expr" => match operand {
            Operand::Immediate(Expression::Constant(value)) => (0..65536).contains(value),
            Operand::Immediate(_) => true,
            _ => false,
        },
        "isImm0_255Expr" => match operand {
            Operand::Immediate(Expression::Constant(value)) => (0..256).contains(value),
            Operand::Immediate(_) => true,
            _ => false,
        },
        "isImm256_65535Expr" => match operand {
            Operand::Immediate(Expression::Constant(value)) => (256..65536).contains(value),
            Operand::Immediate(_) => true,
            _ => false,
        },
        "isAdrLabel" => match operand {
            Operand::Immediate(Expression::Constant(value)) => {
                so_immediate(*value as u32).is_some()
                    || so_immediate((*value as u32).wrapping_neg()).is_some()
            }
            Operand::Immediate(_) => true,
            _ => false,
        },
        "isRegShiftedReg" => matches!(
            operand,
            Operand::ShiftedRegister { source, amount_register, .. }
                if general(*source) && general(*amount_register)
        ),
        "isRegShiftedImm" => matches!(
            operand,
            Operand::ShiftedImmediate { source, .. } if general(*source)
        ),
        "isMemNoOffset" => memory.is_some_and(|held| {
            held.offset_register == NO_REGISTER
                && held.offset_immediate.is_none()
                && held.alignment == 0
        }),
        "isAddrMode2" => memory.is_some_and(|held| {
            if held.alignment != 0 {
                return false;
            }
            if held.offset_register != NO_REGISTER {
                return true;
            }
            match &held.offset_immediate {
                None => true,
                Some(Expression::Constant(value)) => *value > -4096 && *value < 4096,
                Some(_) => false,
            }
        }),
        "isAM2OffsetImm" => {
            constant.is_some_and(|value| value == minimum || (value > -4096 && value < 4096))
        }
        "isAddrMode3" => {
            if is_immediate && constant.is_none() {
                return true;
            }
            memory.is_some_and(|held| {
                if held.alignment != 0 || held.shift != Shift::None {
                    return false;
                }
                if held.offset_register != NO_REGISTER {
                    return true;
                }
                match &held.offset_immediate {
                    None => true,
                    Some(Expression::Constant(value)) => {
                        (*value > -256 && *value < 256) || *value == minimum
                    }
                    Some(_) => false,
                }
            })
        }
        "isAM3Offset" => match operand {
            Operand::PostIndexRegister {
                register, shift, ..
            } => general(*register) && *shift == Shift::None,
            _ => constant.is_some_and(|value| (value > -256 && value < 256) || value == minimum),
        },
        "isPostIdxReg" => matches!(
            operand,
            Operand::PostIndexRegister { register, shift, .. }
                if general(*register) && *shift == Shift::None
        ),
        "isPostIdxRegShifted" => matches!(
            operand,
            Operand::PostIndexRegister { register, .. } if general(*register)
        ),
        "isPostIdxImm8" => {
            constant.is_some_and(|value| (value > -256 && value < 256) || value == minimum)
        }
        "isMemImm12Offset" => {
            if is_immediate && constant.is_none() {
                return true;
            }
            memory.is_some_and(|held| {
                if held.offset_register != NO_REGISTER || held.alignment != 0 {
                    return false;
                }
                match &held.offset_immediate {
                    None => true,
                    Some(Expression::Constant(value)) => {
                        (*value > -4096 && *value < 4096) || *value == minimum
                    }
                    Some(_) => true,
                }
            })
        }
        "isMemUImm12Offset" => memory.is_some_and(|held| {
            if held.offset_register != NO_REGISTER || held.alignment != 0 {
                return false;
            }
            match &held.offset_immediate {
                None => true,
                Some(Expression::Constant(value)) => *value >= 0 && *value < 4096,
                Some(_) => false,
            }
        }),
        "isMemRegOffset" => {
            memory.is_some_and(|held| held.offset_register != NO_REGISTER && held.alignment == 0)
        }
        "isMemPCRelImm12" => memory.is_some_and(|held| {
            if held.offset_register != NO_REGISTER || held.alignment != 0 {
                return false;
            }
            if register_encoding(target, held.base) != 15 {
                return false;
            }
            match &held.offset_immediate {
                None => true,
                Some(Expression::Constant(value)) => {
                    (*value > -4096 && *value < 4096) || *value == minimum
                }
                Some(_) => false,
            }
        }),
        "isMemThumbRR" => memory.is_some_and(|held| {
            held.offset_register != NO_REGISTER
                && !held.negative
                && held.shift == Shift::None
                && held.alignment == 0
                && low(held.base)
                && low(held.offset_register)
        }),
        "isMemThumbRIs4" | "isMemThumbRIs2" | "isMemThumbRIs1" => memory.is_some_and(|held| {
            if held.offset_register != NO_REGISTER || !low(held.base) || held.alignment != 0 {
                return false;
            }
            let (limit, step) = match base.as_str() {
                "isMemThumbRIs4" => (124, 4),
                "isMemThumbRIs2" => (62, 2),
                _ => (31, 1),
            };
            match &held.offset_immediate {
                None => true,
                Some(Expression::Constant(value)) => {
                    *value >= 0 && *value <= limit && value % step == 0
                }
                Some(_) => false,
            }
        }),
        "isMemThumbSPI" => memory.is_some_and(|held| {
            if held.offset_register != NO_REGISTER
                || register_encoding(target, held.base) != 13
                || held.alignment != 0
            {
                return false;
            }
            match &held.offset_immediate {
                None => true,
                Some(Expression::Constant(value)) => {
                    *value >= 0 && *value <= 1020 && value % 4 == 0
                }
                Some(_) => false,
            }
        }),
        "isMemImm8s4Offset" | "isMemImm7s4Offset" => {
            if is_immediate && constant.is_none() {
                return true;
            }
            let (limit, class) = if base == "isMemImm8s4Offset" {
                (1020, "GPR")
            } else {
                (508, "GPRnopc")
            };
            memory.is_some_and(|held| {
                if held.offset_register != NO_REGISTER
                    || held.alignment != 0
                    || !in_class(target, class, held.base)
                {
                    return false;
                }
                match &held.offset_immediate {
                    None => true,
                    Some(Expression::Constant(value)) => {
                        (*value >= -limit && *value <= limit && value & 3 == 0) || *value == minimum
                    }
                    Some(_) => false,
                }
            })
        }
        "isMemImm0_1020s4Offset" => memory.is_some_and(|held| {
            if held.offset_register != NO_REGISTER || held.alignment != 0 {
                return false;
            }
            match &held.offset_immediate {
                None => true,
                Some(Expression::Constant(value)) => {
                    *value >= 0 && *value <= 1020 && value & 3 == 0
                }
                Some(_) => false,
            }
        }),
        "isMemImm8Offset" => memory.is_some_and(|held| {
            if held.offset_register != NO_REGISTER
                || held.alignment != 0
                || register_encoding(target, held.base) == 15
            {
                return false;
            }
            match &held.offset_immediate {
                None => true,
                Some(Expression::Constant(value)) => {
                    *value == minimum || (*value > -256 && *value < 256)
                }
                Some(_) => false,
            }
        }),
        "isMemPosImm8Offset" => memory.is_some_and(|held| {
            if held.offset_register != NO_REGISTER || held.alignment != 0 {
                return false;
            }
            match &held.offset_immediate {
                None => true,
                Some(Expression::Constant(value)) => *value >= 0 && *value < 256,
                Some(_) => false,
            }
        }),
        "isMemNegImm8Offset" => memory.is_some_and(|held| {
            if held.offset_register != NO_REGISTER
                || held.alignment != 0
                || register_encoding(target, held.base) == 15
            {
                return false;
            }
            match &held.offset_immediate {
                None => false,
                Some(Expression::Constant(value)) => {
                    *value == minimum || (*value > -256 && *value < 0)
                }
                Some(_) => false,
            }
        }),
        "isT2MemRegOffset" => memory.is_some_and(|held| {
            held.offset_register != NO_REGISTER
                && !held.negative
                && held.alignment == 0
                && register_encoding(target, held.base) != 15
                && (held.shift == Shift::None
                    || (held.shift == Shift::Lsl && held.shift_immediate <= 3))
        }),
        "isMemTBB" | "isMemTBH" => memory.is_some_and(|held| {
            held.offset_register != NO_REGISTER
                && !held.negative
                && held.alignment == 0
                && if base == "isMemTBB" {
                    held.shift == Shift::None
                } else {
                    held.shift == Shift::Lsl && held.shift_immediate == 1
                }
        }),
        "isMemNoOffsetT" | "isMemNoOffsetT2" | "isMemNoOffsetT2NoSp" => {
            memory.is_some_and(|held| {
                let class = match base.as_str() {
                    "isMemNoOffsetT" => "tGPR",
                    "isMemNoOffsetT2" => "GPRnopc",
                    _ => "rGPR",
                };
                in_class(target, class, held.base)
                    && held.offset_register == NO_REGISTER
                    && held.offset_immediate.is_none()
                    && held.alignment == 0
            })
        }
        "isThumbMemPC" => {
            let value = match operand {
                Operand::Immediate(Expression::Constant(value)) => Some(*value),
                Operand::Immediate(_) => return true,
                Operand::Memory(held)
                    if held.offset_register == NO_REGISTER
                        && register_encoding(target, held.base) == 15 =>
                {
                    match &held.offset_immediate {
                        Some(Expression::Constant(value)) => Some(*value),
                        _ => None,
                    }
                }
                _ => None,
            };
            value.is_some_and(|value| value % 4 == 0 && (0..=1020).contains(&value))
        }
        "isT2SOImm" => match operand {
            Operand::Immediate(Expression::Constant(value)) => {
                thumb_so_immediate(*value as u32).is_some()
            }
            Operand::Immediate(Expression::Symbol { specifier, .. }) => {
                !matches!(specifier.as_deref(), Some("upper16" | "lower16"))
            }
            Operand::Immediate(_) => true,
            _ => false,
        },
        "isT2SOImmNot" => constant.is_some_and(|value| {
            thumb_so_immediate(value as u32).is_none()
                && thumb_so_immediate(!(value as u32)).is_some()
        }),
        "isT2SOImmNeg" => constant.is_some_and(|value| {
            thumb_so_immediate(value as u32).is_none()
                && thumb_so_immediate((value as u32).wrapping_neg()).is_some()
        }),
        "isImmediateS4" => constant.is_some_and(|value| {
            (value & 3 == 0 && value >= number(0) && value <= number(1)) || value == minimum
        }),
        "isImmediateS2" => {
            constant.is_some_and(|value| value & 1 == 0 && value >= number(0) && value <= number(1))
        }
        "isImm8s4" => constant.is_some_and(|value| {
            (value & 3 == 0 && (-1020..=1020).contains(&value)) || value == minimum
        }),
        "isImm7s4" => constant.is_some_and(|value| {
            (value & 3 == 0 && (-508..=508).contains(&value)) || value == minimum
        }),
        "isImm7Shift0" | "isImm7" => constant.is_some_and(|value| (-127..=127).contains(&value)),
        "isImm7Shift1" => {
            constant.is_some_and(|value| value & 1 == 0 && (-255..=255).contains(&value))
        }
        "isImm7Shift2" => {
            constant.is_some_and(|value| value & 3 == 0 && (-511..=511).contains(&value))
        }
        "isImm0_1020s4" => {
            constant.is_some_and(|value| value & 3 == 0 && (0..=1020).contains(&value))
        }
        "isImm0_508s4" => {
            constant.is_some_and(|value| value & 3 == 0 && (0..=508).contains(&value))
        }
        "isImm0_508s4Neg" => constant.is_some_and(|value| {
            let negated = -value;
            negated & 3 == 0 && negated > 0 && negated <= 508
        }),
        "isImm1_16" => constant.is_some_and(|value| (1..=16).contains(&value)),
        "isImm1_32" => constant.is_some_and(|value| (1..=32).contains(&value)),
        "isImm8_255" => constant.is_some_and(|value| (8..=255).contains(&value)),
        "isImm24bit" => constant.is_some_and(|value| (0..=0xff_ffff).contains(&value)),
        "isImmThumbSR" => constant.is_some_and(|value| (1..=32).contains(&value)),
        "isPKHLSLImm" => constant.is_some_and(|value| (0..=31).contains(&value)),
        "isPKHASRImm" => constant.is_some_and(|value| (1..=32).contains(&value)),
        "isThumbModImmNeg1_7" => constant.is_some_and(|value| {
            let negated = -(value as i32);
            negated > 0 && negated < 8
        }),
        "isThumbModImmNeg8_255" => constant.is_some_and(|value| {
            let negated = -(value as i32);
            negated > 7 && negated < 256
        }),
        "isUnsignedOffset" => match operand {
            Operand::Immediate(Expression::Symbol { .. }) => true,
            Operand::Immediate(Expression::Constant(value)) => {
                let align = 1i64 << number(1);
                let maximum = align * ((1i64 << number(0)) - 1);
                value % align == 0 && *value >= 0 && *value <= maximum
            }
            _ => false,
        },
        "isSignedOffset" => match operand {
            Operand::Immediate(Expression::Symbol { .. }) => true,
            Operand::Immediate(Expression::Constant(value)) => {
                let align = 1i64 << number(1);
                let maximum = align * ((1i64 << (number(0) - 1)) - 1);
                let minimum_value = -align * (1i64 << (number(0) - 1));
                value % align == 0 && *value >= minimum_value && *value <= maximum
            }
            _ => false,
        },
        "isSetEndImm" => constant.is_some_and(|value| value == 0 || value == 1),
        "isRegList" => matches!(operand, Operand::RegisterList(_)),
        "isRegListWithAPSR" => matches!(operand, Operand::RegisterList(_)),
        "isBitfield" => matches!(operand, Operand::Bitfield { .. }),
        "isShifterImm" => matches!(operand, Operand::ShifterImmediate { .. }),
        "isITMask" => matches!(operand, Operand::ItMask(_)),
        "isMemBarrierOpt" => matches!(operand, Operand::MemoryBarrier(_)),
        "isInstSyncBarrierOpt" => matches!(operand, Operand::InstructionBarrier(_)),
        "isRotImm" => matches!(operand, Operand::RotateImmediate(_)),
        "isConstPoolAsmImm" | "isConstantPoolImm" => matches!(operand, Operand::ConstantPool(_)),
        _ => false,
    }
}

fn register_encoding(target: &Target, register: RegisterId) -> u32 {
    u32::from(target.registers[register as usize].encoding)
}

fn alias_register(text: &str) -> Option<&'static str> {
    Some(match text {
        "r13" => "sp",
        "r14" => "lr",
        "r15" => "pc",
        "ip" => "r12",
        "a1" => "r0",
        "a2" => "r1",
        "a3" => "r2",
        "a4" => "r3",
        "v1" => "r4",
        "v2" => "r5",
        "v3" => "r6",
        "v4" => "r7",
        "v5" => "r8",
        "v6" => "r9",
        "v7" => "r10",
        "v8" => "r11",
        "sb" => "r9",
        "sl" => "r10",
        "fp" => "r11",
        _ => return None,
    })
}

pub fn register_named(target: &Target, text: &str) -> Option<RegisterId> {
    let lower = text.to_ascii_lowercase();
    let name = alias_register(&lower).unwrap_or(&lower);
    target
        .register_by_name
        .get(name)
        .copied()
        .filter(|register| *register != NO_REGISTER)
}

struct Mnemonic {
    it_mask: Option<String>,
    base: String,
    condition: u32,
    carry: bool,
    interrupt_modification: u32,
}

const UNSPLIT: &[&str] = &[
    "teq", "vceq", "svc", "mls", "smmls", "vcls", "vmls", "vnmls", "vacge", "vcge", "vclt",
    "vacgt", "vaclt", "vacle", "hlt", "vcgt", "vcle", "smlal", "umaal", "umlal", "vabal", "vmlal",
    "vpadal", "vqdmlal", "fmuls", "vmaxnm", "vminnm", "vcvta", "vcvtn", "vcvtp", "vcvtm", "vrinta",
    "vrintn", "vrintp", "vrintm", "hvc", "vins", "vmovx", "bxns", "blxns", "vdot", "vmmla",
    "vudot", "vsdot", "vcmla", "vcadd", "vfmal", "vfmsl", "wls", "le", "dls", "csel", "csinc",
    "csinv", "csneg", "cinc", "cinv", "cneg", "cset", "csetm", "aut", "pac", "pacbti", "bti",
];

const KEEP_FLAG_SUFFIX: &[&str] = &[
    "cps", "mls", "mrs", "smmls", "vabs", "vcls", "vmls", "vmrs", "vnmls", "vqabs", "vrecps",
    "vrsqrts", "srs", "flds", "fmrs", "fsqrts", "fsubs", "fsts", "fcpys", "fdivs", "fmuls",
    "fcmps", "fcmpzs", "vfms", "vfnms", "fconsts", "bxns", "blxns", "vfmas", "vmlas",
];

const KEEP_CONDITION_SUFFIX: &[&str] = &[
    "adcs", "bics", "movs", "muls", "smlals", "smulls", "umlals", "umulls", "lsls", "sbcs", "rscs",
];

fn split_mnemonic(text: &str, thumb: bool) -> Mnemonic {
    let mut result = Mnemonic {
        it_mask: None,
        base: text.to_string(),
        condition: CONDITION_AL,
        carry: false,
        interrupt_modification: 0,
    };
    if (text == "movs" && thumb) || UNSPLIT.contains(&text) || text.starts_with("vsel") {
        return result;
    }
    let mut name = text.to_string();
    if !KEEP_CONDITION_SUFFIX.contains(&name.as_str()) && name.len() >= 2 {
        if let Some(code) = condition_code(&name[name.len() - 2..]) {
            name.truncate(name.len() - 2);
            result.condition = code;
        }
    }
    if name.ends_with('s')
        && !KEEP_FLAG_SUFFIX.contains(&name.as_str())
        && !(name == "movs" && thumb)
    {
        name.pop();
        result.carry = true;
    }
    if name.starts_with("cps") && name.len() >= 2 {
        let tail = &name[name.len() - 2..];
        let modification = match tail {
            "ie" => 2,
            "id" => 3,
            _ => 0,
        };
        if modification != 0 {
            name.truncate(name.len() - 2);
            result.interrupt_modification = modification;
        }
    }
    if let Some(mask) = name.strip_prefix("it") {
        result.it_mask = Some(mask.to_string());
        name = "it".to_string();
    }
    result.base = name;
    result
}

fn accepts_carry(mnemonic: &str, thumb: bool) -> bool {
    matches!(
        mnemonic,
        "and"
            | "lsl"
            | "lsr"
            | "rrx"
            | "ror"
            | "sub"
            | "add"
            | "adc"
            | "mul"
            | "bic"
            | "asr"
            | "orr"
            | "mvn"
            | "rsb"
            | "rsc"
            | "orn"
            | "sbc"
            | "eor"
            | "neg"
            | "vfm"
            | "vfnm"
    ) || (!thumb
        && matches!(
            mnemonic,
            "smull" | "mov" | "mla" | "smlal" | "umlal" | "umull"
        ))
}

fn accepts_condition(mnemonic: &str, thumb: bool) -> bool {
    if matches!(
        mnemonic,
        "bkpt"
            | "cbnz"
            | "setend"
            | "cps"
            | "it"
            | "cbz"
            | "trap"
            | "hlt"
            | "udf"
            | "vmaxnm"
            | "vminnm"
            | "vcvta"
            | "vcvtn"
            | "vcvtp"
            | "vcvtm"
            | "vrinta"
            | "vrintn"
            | "vrintp"
            | "vrintm"
            | "hvc"
            | "setpan"
            | "vmovx"
            | "vins"
            | "vudot"
            | "vsdot"
            | "vcmla"
            | "vcadd"
            | "vfmal"
            | "vfmsl"
            | "vfmat"
            | "vfmab"
            | "vdot"
            | "vmmla"
            | "sb"
            | "ssbb"
            | "pssbb"
            | "vsmmla"
            | "vummla"
            | "vusmmla"
            | "vusdot"
            | "vsudot"
            | "bfcsel"
            | "wls"
            | "dls"
            | "le"
            | "csel"
            | "csinc"
            | "csinv"
            | "csneg"
            | "cinc"
            | "cinv"
            | "cneg"
            | "cset"
            | "csetm"
            | "pac"
            | "pacbti"
            | "aut"
            | "bti"
    ) || mnemonic.starts_with("crc32")
        || mnemonic.starts_with("cps")
        || mnemonic.starts_with("vsel")
        || mnemonic.starts_with("aes")
        || mnemonic.starts_with("sha1")
        || mnemonic.starts_with("sha256")
        || mnemonic.starts_with("vpt")
        || mnemonic.starts_with("vpst")
    {
        return false;
    }
    if !thumb {
        return !matches!(
            mnemonic,
            "cdp2"
                | "clrex"
                | "mcr2"
                | "mcrr2"
                | "mrc2"
                | "mrrc2"
                | "dmb"
                | "dfb"
                | "dsb"
                | "isb"
                | "pld"
                | "pli"
                | "pldw"
                | "ldc2"
                | "ldc2l"
                | "stc2"
                | "stc2l"
                | "tsb"
        ) && !mnemonic.starts_with("rfe")
            && !mnemonic.starts_with("srs");
    }
    true
}

fn parsed(kind: Operand, start: usize, end: usize) -> ParsedOperand {
    ParsedOperand {
        kind: ParsedKind::Arm32(Box::new(kind)),
        start,
        end,
    }
}

fn operand_of(operand: &ParsedOperand) -> Option<&Operand> {
    match &operand.kind {
        ParsedKind::Arm32(held) => Some(held.as_ref()),
        _ => None,
    }
}

enum Outcome {
    Success(MachineInstruction),
    MnemonicFail,
    InvalidOperand(usize),
    MissingFeature(Vec<String>),
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Parse {
    Success,
    NoMatch,
    Failure,
}

impl<'s> Assembler<'s> {
    fn thumb_state(&self) -> bool {
        self.arm_thumb
    }

    fn arm32_push_token(&self, operands: &mut Vec<ParsedOperand>, text: &str, start: usize) {
        operands.push(ParsedOperand::token(text, start));
    }

    fn arm32_try_register(&mut self) -> Option<RegisterId> {
        if !self.is(TokenKind::Identifier) {
            return None;
        }
        let register = register_named(self.target, &self.lexer.token.text)?;
        self.lex();
        Some(register)
    }

    fn arm32_hash(&mut self) -> bool {
        if self.is(TokenKind::Hash) || self.is(TokenKind::Dollar) {
            self.lex();
            return true;
        }
        false
    }

    fn arm32_shift_amount(&mut self, shift: &mut Shift) -> Result<u32, ()> {
        if *shift == Shift::Rrx {
            return Ok(0);
        }
        let location = self.location();
        if !self.arm32_hash() {
            self.error(location, "'#' expected");
            return Err(());
        }
        let location = self.location();
        let (expression, _) = self.parse_expression()?;
        let Expression::Constant(amount) = expression else {
            self.error(location, "shift amount must be an immediate");
            return Err(());
        };
        let out_of_range = amount < 0
            || (matches!(*shift, Shift::Lsl | Shift::Ror) && amount > 31)
            || (matches!(*shift, Shift::Lsr | Shift::Asr) && amount > 32);
        if out_of_range {
            self.error(location, "immediate shift value out of range");
            return Err(());
        }
        if amount == 0 {
            *shift = Shift::Lsl;
        }
        Ok(if amount == 32 { 0 } else { amount as u32 })
    }

    fn arm32_memory_shift(&mut self) -> Result<(Shift, u32), ()> {
        let location = self.location();
        if !self.is(TokenKind::Identifier) {
            self.error(location, "illegal shift operator");
            return Err(());
        }
        let Some(mut shift) = Shift::named(&self.lexer.token.text) else {
            self.error(location, "illegal shift operator");
            return Err(());
        };
        self.lex();
        let amount = self.arm32_shift_amount(&mut shift)?;
        Ok((shift, amount))
    }

    fn arm32_parse_memory(&mut self, operands: &mut Vec<ParsedOperand>) -> bool {
        let start = self.lexer.token.start;
        self.lex();
        let location = self.location();
        let Some(base) = self.arm32_try_register() else {
            return self.error(location, "register expected");
        };
        let mut memory = Memory {
            base,
            offset_immediate: None,
            offset_register: NO_REGISTER,
            shift: Shift::None,
            shift_immediate: 0,
            alignment: 0,
            negative: false,
        };
        let location = self.location();
        if !self.is(TokenKind::Colon)
            && !self.is(TokenKind::Comma)
            && !self.is(TokenKind::RightBracket)
        {
            return self.error(location, "malformed memory operand");
        }
        let finish = |this: &mut Self, operands: &mut Vec<ParsedOperand>, memory: Memory| -> bool {
            let end = this.lexer.token.start;
            operands.push(parsed(Operand::Memory(memory), start, end));
            if this.is(TokenKind::Exclamation) {
                let at = this.lexer.token.start;
                this.lex();
                operands.push(ParsedOperand::token("!", at));
            }
            false
        };
        if self.is(TokenKind::RightBracket) {
            self.lex();
            return finish(self, operands, memory);
        }
        if self.is(TokenKind::Comma) {
            self.lex();
        }
        if self.is(TokenKind::Colon) {
            self.lex();
            let location = self.location();
            let Ok((expression, _)) = self.parse_expression() else {
                return true;
            };
            let Expression::Constant(bits) = expression else {
                return self.error(location, "constant expression expected");
            };
            memory.alignment = match bits {
                16 => 2,
                32 => 4,
                64 => 8,
                128 => 16,
                256 => 32,
                _ => {
                    return self.error(
                        location,
                        "alignment specifier must be 16, 32, 64, 128, or 256 bits",
                    )
                }
            };
            if !self.is(TokenKind::RightBracket) {
                return self.token_error("']' expected");
            }
            self.lex();
            return finish(self, operands, memory);
        }
        if self.is(TokenKind::Hash)
            || self.is(TokenKind::Dollar)
            || self.is(TokenKind::LeftParenthesis)
            || self.is(TokenKind::Integer)
        {
            if self.is(TokenKind::Hash) || self.is(TokenKind::Dollar) {
                self.lex();
            }
            let negative = self.is(TokenKind::Minus);
            let Ok((expression, _)) = self.parse_expression() else {
                return true;
            };
            let adjusted = match expression {
                Expression::Constant(0) if negative => Expression::Constant(i64::from(i32::MIN)),
                other => other,
            };
            memory.offset_immediate = Some(adjusted);
            if !self.is(TokenKind::RightBracket) {
                return self.token_error("']' expected");
            }
            self.lex();
            return finish(self, operands, memory);
        }
        if self.is(TokenKind::Minus) {
            memory.negative = true;
            self.lex();
        } else if self.is(TokenKind::Plus) {
            self.lex();
        }
        let location = self.location();
        let Some(offset) = self.arm32_try_register() else {
            return self.error(location, "register expected");
        };
        memory.offset_register = offset;
        if self.is(TokenKind::Comma) {
            self.lex();
            let Ok((shift, amount)) = self.arm32_memory_shift() else {
                return true;
            };
            memory.shift = shift;
            memory.shift_immediate = amount;
        }
        if !self.is(TokenKind::RightBracket) {
            return self.token_error("']' expected");
        }
        self.lex();
        finish(self, operands, memory)
    }

    fn arm32_parse_register_list(&mut self, operands: &mut Vec<ParsedOperand>) -> bool {
        let start = self.lexer.token.start;
        self.lex();
        let location = self.location();
        let Some(mut register) = self.arm32_try_register() else {
            return self.error(location, "register expected");
        };
        if !in_class(self.target, "GPR", register)
            && !in_class(self.target, "GPRwithAPSRnosp", register)
        {
            return self.error(location, "invalid register in register list");
        }
        let mut registers: Vec<RegisterId> = vec![register];
        while self.is(TokenKind::Comma) || self.is(TokenKind::Minus) {
            if self.is(TokenKind::Minus) {
                self.lex();
                let after = self.location();
                let Some(end) = self.arm32_try_register() else {
                    return self.error(after, "register expected");
                };
                if register == end {
                    continue;
                }
                let first = register_encoding(self.target, register);
                let last = register_encoding(self.target, end);
                if first > last {
                    return self.error(after, "bad range in register list");
                }
                for encoding in first + 1..=last {
                    let Some(next) = register_named(self.target, &format!("r{encoding}")) else {
                        return self.error(after, "invalid register in register list");
                    };
                    let next = if encoding == 13 {
                        register_named(self.target, "sp").unwrap_or(next)
                    } else if encoding == 14 {
                        register_named(self.target, "lr").unwrap_or(next)
                    } else if encoding == 15 {
                        register_named(self.target, "pc").unwrap_or(next)
                    } else {
                        next
                    };
                    if !registers.contains(&next) {
                        registers.push(next);
                    }
                    register = next;
                }
                continue;
            }
            self.lex();
            let location = self.location();
            let Some(next) = self.arm32_try_register() else {
                return self.error(location, "register expected");
            };
            if !in_class(self.target, "GPR", next)
                && !in_class(self.target, "GPRwithAPSRnosp", next)
            {
                return self.error(location, "invalid register in register list");
            }
            if register_encoding(self.target, next) < register_encoding(self.target, register) {
                self.message(
                    MessageKind::Warning,
                    location,
                    "register list not in ascending order",
                    None,
                );
            }
            if registers.contains(&next) {
                self.message(
                    MessageKind::Warning,
                    location,
                    "duplicated register in register list",
                    None,
                );
            } else {
                registers.push(next);
            }
            register = next;
        }
        if !self.is(TokenKind::RightBrace) {
            return self.token_error("'}' expected");
        }
        let end = self.lexer.token.end;
        self.lex();
        let target = self.target;
        registers.sort_by_key(|held| register_encoding(target, *held));
        operands.push(parsed(Operand::RegisterList(registers), start, end));
        if self.is(TokenKind::Caret) {
            let at = self.lexer.token.start;
            self.lex();
            operands.push(ParsedOperand::token("^", at));
        }
        false
    }

    fn arm32_parse_shifted_register(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        let start = self.lexer.token.start;
        if !self.is(TokenKind::Identifier) {
            return Parse::NoMatch;
        }
        let Some(mut shift) = Shift::named(&self.lexer.token.text) else {
            return Parse::NoMatch;
        };
        self.lex();
        let previous = operands.pop();
        let source = match previous.as_ref().and_then(operand_of) {
            Some(Operand::Register(register)) => *register,
            _ => {
                let at = previous.map(|held| held.start).unwrap_or(start);
                let location = self.statement_location(at);
                self.error(location, "shift must be of a register");
                return Parse::Failure;
            }
        };
        let mut amount_register = NO_REGISTER;
        let mut amount = 0;
        if shift == Shift::Rrx {
            amount_register = source;
        } else if self.is(TokenKind::Hash) || self.is(TokenKind::Dollar) {
            self.lex();
            let location = self.location();
            let Ok((expression, _)) = self.parse_expression() else {
                self.error(location, "invalid immediate shift value");
                return Parse::Failure;
            };
            let Expression::Constant(value) = expression else {
                self.error(location, "invalid immediate shift value");
                return Parse::Failure;
            };
            let out_of_range = value < 0
                || (matches!(shift, Shift::Lsl | Shift::Ror) && value > 31)
                || (matches!(shift, Shift::Lsr | Shift::Asr) && value > 32);
            if out_of_range {
                self.error(location, "immediate shift value out of range");
                return Parse::Failure;
            }
            if value == 0 {
                shift = Shift::Lsl;
            }
            amount = value as u32;
        } else if self.is(TokenKind::Identifier) {
            let location = self.location();
            match self.arm32_try_register() {
                Some(register) => amount_register = register,
                None => {
                    self.error(location, "expected immediate or register in shift operand");
                    return Parse::Failure;
                }
            }
        } else {
            let location = self.location();
            self.error(location, "expected immediate or register in shift operand");
            return Parse::Failure;
        }
        let end = self.lexer.token.start;
        if amount_register != NO_REGISTER && shift != Shift::Rrx {
            operands.push(parsed(
                Operand::ShiftedRegister {
                    shift,
                    source,
                    amount_register,
                    amount,
                },
                start,
                end,
            ));
        } else {
            operands.push(parsed(
                Operand::ShiftedImmediate {
                    shift,
                    source,
                    amount,
                },
                start,
                end,
            ));
        }
        Parse::Success
    }

    fn arm32_parse_post_index(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        shifted: bool,
    ) -> Parse {
        let start = self.lexer.token.start;
        let mut taken = false;
        let mut add = true;
        if self.is(TokenKind::Plus) {
            self.lex();
            taken = true;
        } else if self.is(TokenKind::Minus) {
            self.lex();
            add = false;
            taken = true;
        }
        let location = self.location();
        let Some(register) = self.arm32_try_register() else {
            if !taken {
                return Parse::NoMatch;
            }
            self.error(location, "register expected");
            return Parse::Failure;
        };
        let mut shift = Shift::None;
        let mut amount = 0;
        if shifted && self.is(TokenKind::Comma) {
            self.lex();
            let Ok((held, count)) = self.arm32_memory_shift() else {
                return Parse::Failure;
            };
            shift = held;
            amount = count;
        }
        let end = self.lexer.token.start;
        operands.push(parsed(
            Operand::PostIndexRegister {
                register,
                add,
                shift,
                amount,
            },
            start,
            end,
        ));
        Parse::Success
    }

    fn arm32_parse_modified_immediate(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        if self.is(TokenKind::Identifier) || self.is(TokenKind::Colon) {
            return Parse::NoMatch;
        }
        let start = self.lexer.token.start;
        if self.is(TokenKind::Hash) || self.is(TokenKind::Dollar) {
            self.lex();
            if self.is(TokenKind::Colon) {
                return Parse::NoMatch;
            }
        }
        let location = self.location();
        let Ok((first, _)) = self.parse_expression() else {
            self.error(location, "malformed expression");
            return Parse::Failure;
        };
        let end = self.lexer.token.start;
        let Expression::Constant(value) = first else {
            operands.push(parsed(Operand::Immediate(first), start, end));
            return Parse::Success;
        };
        if self.is(TokenKind::EndOfStatement) {
            match so_immediate(value as u32) {
                Some(encoded) => operands.push(parsed(
                    Operand::ModifiedImmediate {
                        bits: encoded & 0xff,
                        rotation: (encoded & 0xf00) >> 7,
                    },
                    start,
                    end,
                )),
                None => operands.push(parsed(
                    Operand::Immediate(Expression::Constant(value)),
                    start,
                    end,
                )),
            }
            return Parse::Success;
        }
        if !self.is(TokenKind::Comma) {
            self.error(
                location,
                "expected modified immediate operand: #[0, 255], #even[0-30]",
            );
            return Parse::Failure;
        }
        if value & !0xff != 0 {
            self.error(
                location,
                "immediate operand must a number in the range [0, 255]",
            );
            return Parse::Failure;
        }
        self.lex();
        let second_location = self.location();
        if self.is(TokenKind::Hash) || self.is(TokenKind::Dollar) {
            self.lex();
        }
        let Ok((second, _)) = self.parse_expression() else {
            self.error(second_location, "malformed expression");
            return Parse::Failure;
        };
        let Expression::Constant(rotation) = second else {
            self.error(second_location, "constant expression expected");
            return Parse::Failure;
        };
        if rotation & !0x1e != 0 {
            self.error(
                second_location,
                "immediate operand must an even number in the range [0, 30]",
            );
            return Parse::Failure;
        }
        let end = self.lexer.token.start;
        operands.push(parsed(
            Operand::ModifiedImmediate {
                bits: value as u32,
                rotation: rotation as u32,
            },
            start,
            end,
        ));
        Parse::Success
    }

    fn arm32_parse_am3_offset(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        let start = self.lexer.token.start;
        if self.is(TokenKind::Hash) || self.is(TokenKind::Dollar) {
            self.lex();
            let negative = self.is(TokenKind::Minus);
            let location = self.location();
            let Ok((expression, _)) = self.parse_expression() else {
                return Parse::Failure;
            };
            let Expression::Constant(value) = expression else {
                let at = self.statement_location(start);
                self.error(at, "constant expression expected");
                let _ = location;
                return Parse::Failure;
            };
            let value = if negative && value == 0 {
                i64::from(i32::MIN)
            } else {
                value
            };
            let end = self.lexer.token.start;
            operands.push(parsed(
                Operand::Immediate(Expression::Constant(value)),
                start,
                end,
            ));
            return Parse::Success;
        }
        self.arm32_parse_post_index(operands, false)
    }

    fn arm32_hash_constant(&mut self, message: &str) -> Result<i64, ()> {
        let location = self.location();
        if !self.arm32_hash() {
            self.error(location, "'#' expected");
            return Err(());
        }
        let location = self.location();
        let Ok((expression, _)) = self.parse_expression() else {
            return Err(());
        };
        match expression {
            Expression::Constant(value) => Ok(value),
            _ => {
                self.error(location, message);
                Err(())
            }
        }
    }

    fn arm32_parse_bitfield(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        if !self.is(TokenKind::Hash) && !self.is(TokenKind::Dollar) {
            return Parse::NoMatch;
        }
        let start = self.lexer.token.start;
        let location = self.location();
        let Ok(lsb) = self.arm32_hash_constant("'lsb' operand must be an immediate") else {
            return Parse::Failure;
        };
        if !(0..=31).contains(&lsb) {
            self.error(location, "'lsb' operand must be in the range [0,31]");
            return Parse::Failure;
        }
        if !self.is(TokenKind::Comma) {
            self.token_error("too few operands");
            return Parse::Failure;
        }
        self.lex();
        let Ok(width) = self.arm32_hash_constant("'width' operand must be an immediate") else {
            return Parse::Failure;
        };
        if width < 1 || width > 32 - lsb {
            self.error(location, "'width' operand must be in the range [1,32-lsb]");
            return Parse::Failure;
        }
        let end = self.lexer.token.start;
        operands.push(parsed(
            Operand::Bitfield {
                lsb: lsb as u32,
                width: width as u32,
            },
            start,
            end,
        ));
        Parse::Success
    }

    fn arm32_parse_shifter_immediate(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        if !self.is(TokenKind::Identifier) {
            return Parse::NoMatch;
        }
        let start = self.lexer.token.start;
        let arithmetic = match self.lexer.token.text.to_ascii_lowercase().as_str() {
            "lsl" => false,
            "asr" => true,
            _ => return Parse::NoMatch,
        };
        self.lex();
        let location = self.location();
        let Ok(mut value) = self.arm32_hash_constant("shift amount must be an immediate") else {
            return Parse::Failure;
        };
        if arithmetic {
            if !(1..=32).contains(&value) {
                self.error(location, "'asr' shift amount must be in range [1,32]");
                return Parse::Failure;
            }
            if self.thumb_state() && value == 32 {
                self.error(location, "'asr #32' shift amount not allowed in Thumb mode");
                return Parse::Failure;
            }
            if value == 32 {
                value = 0;
            }
        } else if !(0..=31).contains(&value) {
            self.error(location, "'lsr' shift amount must be in range [0,31]");
            return Parse::Failure;
        }
        let end = self.lexer.token.start;
        operands.push(parsed(
            Operand::ShifterImmediate {
                arithmetic,
                amount: value as u32,
            },
            start,
            end,
        ));
        Parse::Success
    }

    fn arm32_parse_rotate_immediate(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        if !self.is(TokenKind::Identifier) || !self.lexer.token.text.eq_ignore_ascii_case("ror") {
            return Parse::NoMatch;
        }
        let start = self.lexer.token.start;
        self.lex();
        let location = self.location();
        let Ok(value) = self.arm32_hash_constant("rotate amount must be an immediate") else {
            return Parse::Failure;
        };
        if !matches!(value, 0 | 8 | 16 | 24) {
            self.error(location, "'ror' rotate amount must be 8, 16, or 24");
            return Parse::Failure;
        }
        let end = self.lexer.token.start;
        operands.push(parsed(Operand::RotateImmediate(value as u32), start, end));
        Parse::Success
    }

    fn arm32_parse_set_end(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        let start = self.lexer.token.start;
        let location = self.location();
        if !self.is(TokenKind::Identifier) {
            self.error(location, "'be' or 'le' operand expected");
            return Parse::Failure;
        }
        let value = match self.lexer.token.text.to_ascii_lowercase().as_str() {
            "be" => 1,
            "le" => 0,
            _ => {
                self.lex();
                self.error(location, "'be' or 'le' operand expected");
                return Parse::Failure;
            }
        };
        self.lex();
        let end = self.lexer.token.start;
        operands.push(parsed(
            Operand::Immediate(Expression::Constant(value)),
            start,
            end,
        ));
        Parse::Success
    }

    fn arm32_parse_it_condition(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        let start = self.lexer.token.start;
        if !self.is(TokenKind::Identifier) {
            return Parse::NoMatch;
        }
        let Some(code) = condition_code(&self.lexer.token.text.to_ascii_lowercase()) else {
            return Parse::NoMatch;
        };
        self.lex();
        let end = self.lexer.token.start;
        operands.push(parsed(Operand::Condition(code), start, end));
        Parse::Success
    }

    fn arm32_parse_barrier(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        instruction: bool,
    ) -> Parse {
        let start = self.lexer.token.start;
        let option = if self.is(TokenKind::Identifier) {
            let text = self.lexer.token.text.to_ascii_lowercase();
            let value = if instruction {
                match text.as_str() {
                    "sy" => Some(15),
                    _ => None,
                }
            } else {
                match text.as_str() {
                    "sy" => Some(15),
                    "st" => Some(14),
                    "ld" => Some(13),
                    "sh" | "ish" => Some(11),
                    "shst" | "ishst" => Some(10),
                    "ishld" => Some(9),
                    "nsh" | "un" => Some(7),
                    "nshst" | "unst" => Some(6),
                    "nshld" => Some(5),
                    "osh" => Some(3),
                    "oshst" => Some(2),
                    "oshld" => Some(1),
                    _ => None,
                }
            };
            let Some(value) = value else {
                return Parse::NoMatch;
            };
            self.lex();
            value
        } else if self.is(TokenKind::Hash)
            || self.is(TokenKind::Dollar)
            || self.is(TokenKind::Integer)
        {
            if !self.is(TokenKind::Integer) {
                self.lex();
            }
            let location = self.location();
            let Ok((expression, _)) = self.parse_expression() else {
                self.error(location, "illegal expression");
                return Parse::Failure;
            };
            let Expression::Constant(value) = expression else {
                self.error(location, "constant expression expected");
                return Parse::Failure;
            };
            if value & !0xf != 0 {
                self.error(location, "immediate value out of range");
                return Parse::Failure;
            }
            value as u32
        } else {
            self.token_error("expected an immediate or barrier type");
            return Parse::Failure;
        };
        let end = self.lexer.token.start;
        operands.push(parsed(
            if instruction {
                Operand::InstructionBarrier(option)
            } else {
                Operand::MemoryBarrier(option)
            },
            start,
            end,
        ));
        Parse::Success
    }

    fn arm32_class_parsers(&self, operand_count: usize, mnemonic: &str) -> Vec<String> {
        let matcher = self.matcher;
        let mut found: Vec<String> = Vec::new();
        for entry in &matcher.matchables {
            if entry.mnemonic != mnemonic {
                continue;
            }
            let mut predicates = self.target.instructions[entry.result].predicates.clone();
            if let Some(alias) = entry.alias {
                predicates.extend(self.target.aliases[alias].predicates.clone());
            }
            if !armv8::predicates_hold(self.target, &predicates, &self.arm_features) {
                continue;
            }
            let mut index = 0;
            for formal in &entry.operands {
                let class = &matcher.classes[formal.class];
                if class.optional {
                    continue;
                }
                if index == operand_count {
                    if !class.parser.is_empty() && !found.contains(&class.name) {
                        found.push(class.name.clone());
                    }
                    break;
                }
                index += 1;
            }
        }
        found.sort();
        found
    }

    fn arm32_parse_operand(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        mnemonic: &str,
        prefix: usize,
    ) -> bool {
        let count = operands.len().saturating_sub(prefix);
        for class in self.arm32_class_parsers(count, mnemonic) {
            let status = match class.as_str() {
                "ModImm" => self.arm32_parse_modified_immediate(operands),
                "AM3Offset" => self.arm32_parse_am3_offset(operands),
                "ITCondCode" | "CondCodeNoAL" | "CondCodeNoALInv" => {
                    self.arm32_parse_it_condition(operands)
                }
                "MemBarrierOpt" => self.arm32_parse_barrier(operands, false),
                "InstSyncBarrierOpt" => self.arm32_parse_barrier(operands, true),
                "Bitfield" => self.arm32_parse_bitfield(operands),
                "ShifterImm" => self.arm32_parse_shifter_immediate(operands),
                "RotImm" => self.arm32_parse_rotate_immediate(operands),
                "SetEndImm" => self.arm32_parse_set_end(operands),
                "PostIdxReg" => self.arm32_parse_post_index(operands, false),
                "PostIdxRegShifted" => self.arm32_parse_post_index(operands, true),
                _ => Parse::NoMatch,
            };
            match status {
                Parse::Success => return false,
                Parse::Failure => return true,
                Parse::NoMatch => {}
            }
        }
        let start = self.lexer.token.start;
        match self.lexer.token.kind {
            TokenKind::Identifier => {
                let expect_label = mnemonic == "b" || mnemonic == "bl";
                if !expect_label {
                    if let Some(register) = self.arm32_try_register() {
                        let end = self.lexer.token.start;
                        operands.push(parsed(Operand::Register(register), start, end));
                        if self.is(TokenKind::Exclamation) {
                            let at = self.lexer.token.start;
                            self.lex();
                            operands.push(ParsedOperand::token("!", at));
                        }
                        return false;
                    }
                    match self.arm32_parse_shifted_register(operands) {
                        Parse::Success => return false,
                        Parse::Failure => return true,
                        Parse::NoMatch => {}
                    }
                }
                self.arm32_parse_expression_operand(operands)
            }
            TokenKind::LeftParenthesis
            | TokenKind::Integer
            | TokenKind::String
            | TokenKind::Dot
            | TokenKind::Minus
            | TokenKind::Plus => self.arm32_parse_expression_operand(operands),
            TokenKind::LeftBracket => self.arm32_parse_memory(operands),
            TokenKind::LeftBrace => self.arm32_parse_register_list(operands),
            TokenKind::Colon => {
                let Ok(expression) = self.arm32_parse_prefixed() else {
                    return true;
                };
                let end = self.lexer.token.start;
                operands.push(parsed(Operand::Immediate(expression), start, end));
                false
            }
            TokenKind::Hash | TokenKind::Dollar => {
                self.lex();
                if self.is(TokenKind::Colon) {
                    let Ok(expression) = self.arm32_parse_prefixed() else {
                        return true;
                    };
                    let end = self.lexer.token.start;
                    operands.push(parsed(Operand::Immediate(expression), start, end));
                    return false;
                }
                let negative = self.is(TokenKind::Minus);
                let Ok((expression, _)) = self.parse_expression() else {
                    return true;
                };
                let expression = match expression {
                    Expression::Constant(0) if negative => {
                        Expression::Constant(i64::from(i32::MIN))
                    }
                    other => other,
                };
                let end = self.lexer.token.start;
                operands.push(parsed(Operand::Immediate(expression), start, end));
                if self.is(TokenKind::Exclamation) {
                    let at = self.lexer.token.start;
                    self.lex();
                    operands.push(ParsedOperand::token("!", at));
                }
                false
            }
            TokenKind::Equal => {
                if mnemonic != "ldr" {
                    return self.token_error("unexpected token in operand");
                }
                self.lex();
                let Ok((expression, _)) = self.parse_expression() else {
                    return true;
                };
                let end = self.lexer.token.start;
                operands.push(parsed(Operand::ConstantPool(expression), start, end));
                false
            }
            _ => self.token_error("unexpected token in operand"),
        }
    }

    fn arm32_parse_prefixed(&mut self) -> Result<Expression, ()> {
        self.lex();
        if !self.is(TokenKind::Identifier) {
            self.token_error("expected prefix identifier in operand");
            return Err(());
        }
        let name = self.lexer.token.text.to_ascii_lowercase();
        if !matches!(
            name.as_str(),
            "lower16" | "upper16" | "lower0_7" | "lower8_15" | "upper0_7" | "upper8_15"
        ) {
            self.token_error(&format!("invalid prefix identifier '{name}' in operand"));
            return Err(());
        }
        self.lex();
        if self.parse_token(TokenKind::Colon, "unexpected token in operand") {
            return Err(());
        }
        let (expression, _) = self.parse_expression()?;
        if let Expression::Constant(value) = expression {
            let value = value as i32;
            let folded = match name.as_str() {
                "upper16" => (value & !0xffff) >> 16,
                "lower16" => value & 0xffff,
                "upper8_15" => (value & !0x00ff_ffff) >> 24,
                "upper0_7" => (value & 0x00ff_0000) >> 16,
                "lower8_15" => (value & 0xff00) >> 8,
                _ => value & 0xff,
            };
            return Ok(Expression::Constant(i64::from(folded)));
        }
        match self.apply_specifier(&expression, &name) {
            Some(applied) => Ok(applied),
            None => {
                self.token_error("expected relocatable expression");
                Err(())
            }
        }
    }

    fn arm32_parse_expression_operand(&mut self, operands: &mut Vec<ParsedOperand>) -> bool {
        let start = self.lexer.token.start;
        let Ok((expression, _)) = self.parse_expression() else {
            return true;
        };
        let end = self.lexer.token.start;
        operands.push(parsed(Operand::Immediate(expression), start, end));
        false
    }

    fn arm32_parse_instruction(
        &mut self,
        name: &str,
        name_start: usize,
    ) -> Result<(Vec<ParsedOperand>, String), ()> {
        let thumb = self.thumb_state();
        let lower = self
            .matcher
            .apply_mnemonic_alias(&name.to_ascii_lowercase(), 0, Mode::Bits32);
        let head_end = lower.find('.').unwrap_or(lower.len());
        let split = split_mnemonic(&lower[..head_end], thumb);
        let mnemonic = split.base.clone();
        let mut operands: Vec<ParsedOperand> = Vec::new();
        self.arm32_push_token(&mut operands, &mnemonic, name_start);
        if let Some(mask) = &split.it_mask {
            let location = self.statement_location(name_start + 2);
            if mask.len() > 3 {
                self.error(location, "too many conditions on IT instruction");
                return Err(());
            }
            let mut value = 8u32;
            for letter in mask.chars().rev() {
                if letter != 't' && letter != 'e' {
                    self.error(
                        location,
                        &format!("illegal IT block condition mask '{mask}'"),
                    );
                    return Err(());
                }
                value >>= 1;
                if letter == 'e' {
                    value |= 8;
                }
            }
            operands.push(parsed(
                Operand::ItMask(value),
                name_start + 2,
                name_start + 2,
            ));
        }
        let can_carry = accepts_carry(&mnemonic, thumb);
        let can_condition = accepts_condition(&mnemonic, thumb);
        let location = self.statement_location(name_start);
        if !can_carry && split.carry {
            self.error(
                location,
                &format!("instruction '{mnemonic}' can not set flags, but 's' suffix specified"),
            );
            return Err(());
        }
        if !can_condition && split.condition != CONDITION_AL {
            self.error(
                location,
                &format!(
                    "instruction '{mnemonic}' is not predicable, but condition code specified"
                ),
            );
            return Err(());
        }
        let after = name_start + mnemonic.len();
        if can_carry && split.carry {
            let carry = self
                .target
                .register_by_name
                .get("cpsr")
                .copied()
                .unwrap_or(NO_REGISTER);
            operands.push(parsed(Operand::CarrySet(carry), after, after));
        }
        if can_condition && split.condition != CONDITION_AL {
            operands.push(parsed(
                Operand::Condition(split.condition),
                after + usize::from(split.carry),
                after + usize::from(split.carry),
            ));
        }
        if split.interrupt_modification != 0 {
            operands.push(parsed(
                Operand::Immediate(Expression::Constant(i64::from(
                    split.interrupt_modification,
                ))),
                name_start,
                name_start,
            ));
        }
        let mut next = lower.find('.');
        while let Some(start) = next {
            let end = lower[start + 1..]
                .find('.')
                .map(|position| position + start + 1);
            let piece = &lower[start..end.unwrap_or(lower.len())];
            if piece == ".n" && !thumb {
                let at = self.statement_location(name_start + start);
                self.error(
                    at,
                    "instruction with .n (narrow) qualifier not allowed in arm mode",
                );
                return Err(());
            }
            if piece != ".n" && (thumb || piece != ".w") {
                self.arm_suffixes.insert(name_start + start);
                self.arm32_push_token(&mut operands, piece, name_start + start);
            }
            next = end;
        }
        let prefix = operands.len() - usize::from(split.it_mask.is_some());
        if !self.is(TokenKind::EndOfStatement) {
            if self.arm32_parse_operand(&mut operands, &mnemonic, prefix) {
                return Err(());
            }
            while self.parse_optional_token(TokenKind::Comma) {
                if self.arm32_parse_operand(&mut operands, &mnemonic, prefix) {
                    return Err(());
                }
            }
        }
        if !self.is(TokenKind::EndOfStatement) {
            self.token_error("unexpected token in argument list");
            return Err(());
        }
        if !thumb
            && mnemonic == "blx"
            && operands.len() == prefix + 1
            && matches!(operand_of(&operands[prefix]), Some(Operand::Immediate(_)))
        {
            operands.retain(|held| !matches!(operand_of(held), Some(Operand::Condition(_))));
        }
        Ok((operands, mnemonic))
    }

    fn arm32_render(
        &self,
        method: &str,
        operand: Option<&Operand>,
        count: usize,
        into: &mut Vec<MachineOperand>,
    ) {
        let (name, _) = split_arguments(method);
        let immediate = |into: &mut Vec<MachineOperand>, value: i64| {
            into.push(MachineOperand::Immediate(value))
        };
        let expression = |into: &mut Vec<MachineOperand>, held: &Expression| match held {
            Expression::Constant(value) => into.push(MachineOperand::Immediate(*value)),
            other => into.push(MachineOperand::Expression(other.clone())),
        };
        let nothing = MachineOperand::Register(NO_REGISTER);
        let Some(operand) = operand else {
            match name.as_str() {
                "addCondCodeOperands" => {
                    immediate(into, i64::from(CONDITION_AL));
                    into.push(nothing);
                }
                _ => {
                    for _ in 0..count {
                        into.push(nothing.clone());
                    }
                }
            }
            return;
        };
        match (name.as_str(), operand) {
            ("addCondCodeOperands", Operand::Condition(code)) => {
                immediate(into, i64::from(*code));
                let cpsr = self
                    .target
                    .register_by_name
                    .get("cpsr")
                    .copied()
                    .unwrap_or(NO_REGISTER);
                into.push(MachineOperand::Register(if *code == CONDITION_AL {
                    NO_REGISTER
                } else {
                    cpsr
                }));
            }
            ("addCCOutOperands", Operand::CarrySet(register)) => {
                into.push(MachineOperand::Register(*register))
            }
            ("addRegOperands", Operand::Register(register)) => {
                into.push(MachineOperand::Register(*register))
            }
            (
                "addImmOperands" | "addARMBranchTargetOperands" | "addThumbBranchTargetOperands",
                Operand::Immediate(held),
            ) => expression(into, held),
            ("addAdrLabelOperands", Operand::Immediate(held)) => expression(into, held),
            ("addModImmOperands", Operand::ModifiedImmediate { bits, rotation }) => {
                immediate(into, i64::from(bits | (rotation << 7)))
            }
            ("addModImmOperands", Operand::Immediate(held)) => expression(into, held),
            ("addModImmNotOperands", _) => {
                let value = operand.constant().unwrap_or(0) as u32;
                immediate(into, i64::from(so_immediate(!value).unwrap_or(0)))
            }
            ("addModImmNegOperands", _) => {
                let value = operand.constant().unwrap_or(0) as u32;
                immediate(
                    into,
                    i64::from(so_immediate(value.wrapping_neg()).unwrap_or(0)),
                )
            }
            ("addImm0_4095NegOperands", _) => {
                let value = operand.constant().unwrap_or(0) as u32;
                immediate(into, i64::from(value.wrapping_neg()))
            }
            (
                "addRegShiftedRegOperands",
                Operand::ShiftedRegister {
                    shift,
                    source,
                    amount_register,
                    amount,
                },
            ) => {
                into.push(MachineOperand::Register(*source));
                into.push(MachineOperand::Register(*amount_register));
                immediate(into, so_register_opcode(*shift, *amount));
            }
            (
                "addRegShiftedImmOperands",
                Operand::ShiftedImmediate {
                    shift,
                    source,
                    amount,
                },
            ) => {
                into.push(MachineOperand::Register(*source));
                let amount = if *amount == 32 { 0 } else { *amount };
                immediate(into, so_register_opcode(*shift, amount));
            }
            ("addMemNoOffsetOperands", Operand::Memory(memory)) => {
                into.push(MachineOperand::Register(memory.base))
            }
            ("addAddrMode2Operands", Operand::Memory(memory)) => {
                into.push(MachineOperand::Register(memory.base));
                into.push(MachineOperand::Register(memory.offset_register));
                if memory.offset_register == NO_REGISTER {
                    match &memory.offset_immediate {
                        None => immediate(into, 0),
                        Some(Expression::Constant(value)) => {
                            let (add, amount) = split_offset(*value);
                            immediate(into, addressing_mode_2(add, amount, Shift::None));
                        }
                        Some(other) => into.push(MachineOperand::Expression(other.clone())),
                    }
                } else {
                    immediate(
                        into,
                        addressing_mode_2(!memory.negative, memory.shift_immediate, memory.shift),
                    );
                }
            }
            ("addMemRegOffsetOperands", Operand::Memory(memory)) => {
                into.push(MachineOperand::Register(memory.base));
                into.push(MachineOperand::Register(memory.offset_register));
                immediate(
                    into,
                    addressing_mode_2(!memory.negative, memory.shift_immediate, memory.shift),
                );
            }
            ("addAM2OffsetImmOperands", _) => {
                let (add, amount) = split_offset(operand.constant().unwrap_or(0));
                into.push(MachineOperand::Register(NO_REGISTER));
                immediate(into, addressing_mode_2(add, amount, Shift::None));
            }
            ("addAddrMode3Operands", Operand::Memory(memory)) => {
                into.push(MachineOperand::Register(memory.base));
                into.push(MachineOperand::Register(memory.offset_register));
                if memory.offset_register == NO_REGISTER {
                    match &memory.offset_immediate {
                        None => immediate(into, 0),
                        Some(Expression::Constant(value)) => {
                            let (add, amount) = split_offset(*value);
                            immediate(into, addressing_mode_3(add, amount));
                        }
                        Some(other) => into.push(MachineOperand::Expression(other.clone())),
                    }
                } else {
                    immediate(into, addressing_mode_3(!memory.negative, 0));
                }
            }
            ("addAddrMode3Operands", Operand::Immediate(held)) => {
                expression(into, held);
                into.push(MachineOperand::Register(NO_REGISTER));
                immediate(into, 0);
            }
            ("addAM3OffsetOperands", Operand::PostIndexRegister { register, add, .. }) => {
                into.push(MachineOperand::Register(*register));
                immediate(into, addressing_mode_3(*add, 0));
            }
            ("addAM3OffsetOperands", _) => {
                let (add, amount) = split_offset(operand.constant().unwrap_or(0));
                into.push(MachineOperand::Register(NO_REGISTER));
                immediate(into, addressing_mode_3(add, amount));
            }
            (
                "addMemImm12OffsetOperands" | "addMemUImm12OffsetOperands",
                Operand::Immediate(held),
            ) => {
                expression(into, held);
                immediate(into, 0);
            }
            (
                "addMemImm12OffsetOperands"
                | "addMemUImm12OffsetOperands"
                | "addMemImmOffsetOperands",
                Operand::Memory(memory),
            ) => {
                into.push(MachineOperand::Register(memory.base));
                match &memory.offset_immediate {
                    None => immediate(into, 0),
                    Some(held) => expression(into, held),
                }
            }
            ("addMemPCRelImm12Operands", Operand::Memory(memory)) => {
                match &memory.offset_immediate {
                    None => immediate(into, 0),
                    Some(held) => expression(into, held),
                }
            }
            ("addPostIdxRegOperands", Operand::PostIndexRegister { register, add, .. }) => {
                into.push(MachineOperand::Register(*register));
                immediate(into, i64::from(*add));
            }
            (
                "addPostIdxRegShiftedOperands",
                Operand::PostIndexRegister {
                    register,
                    add,
                    shift,
                    amount,
                },
            ) => {
                into.push(MachineOperand::Register(*register));
                immediate(into, addressing_mode_2(*add, *amount, *shift));
            }
            ("addPostIdxImm8Operands", _) => {
                let value = operand.constant().unwrap_or(0);
                let (add, amount) = split_offset(value);
                immediate(into, i64::from(amount | (u32::from(add) << 8)));
            }
            ("addRegListOperands" | "addRegListWithAPSROperands", Operand::RegisterList(list)) => {
                for register in list {
                    into.push(MachineOperand::Register(*register));
                }
            }
            ("addConstPoolAsmImmOperands", Operand::ConstantPool(held)) => expression(into, held),
            ("addITMaskOperands", Operand::ItMask(mask)) => immediate(into, i64::from(*mask)),
            ("addITCondCodeOperands", Operand::Condition(code)) => {
                immediate(into, i64::from(*code))
            }
            (
                "addMemThumbRROperands" | "addMemTBBOperands" | "addMemTBHOperands",
                Operand::Memory(memory),
            ) => {
                into.push(MachineOperand::Register(memory.base));
                into.push(MachineOperand::Register(memory.offset_register));
            }
            (
                "addMemThumbRIs4Operands"
                | "addMemThumbRIs2Operands"
                | "addMemThumbSPIOperands"
                | "addMemImm0_1020s4OffsetOperands",
                Operand::Memory(memory),
            ) => {
                let divisor = match name.as_str() {
                    "addMemThumbRIs2Operands" => 2,
                    _ => 4,
                };
                into.push(MachineOperand::Register(memory.base));
                match &memory.offset_immediate {
                    None => immediate(into, 0),
                    Some(Expression::Constant(value)) => immediate(into, value / divisor),
                    Some(other) => into.push(MachineOperand::Expression(other.clone())),
                }
            }
            (
                "addMemThumbRIs1Operands"
                | "addMemImm8s4OffsetOperands"
                | "addMemImm7s4OffsetOperands"
                | "addMemImm8OffsetOperands"
                | "addMemPosImm8OffsetOperands"
                | "addMemNegImm8OffsetOperands",
                Operand::Memory(memory),
            ) => {
                into.push(MachineOperand::Register(memory.base));
                match &memory.offset_immediate {
                    None => immediate(into, 0),
                    Some(held) => expression(into, held),
                }
            }
            (
                "addMemImm8s4OffsetOperands" | "addMemImm7s4OffsetOperands",
                Operand::Immediate(held),
            ) => {
                expression(into, held);
                immediate(into, 0);
            }
            (
                "addMemNoOffsetT2Operands"
                | "addMemNoOffsetT2NoSpOperands"
                | "addMemNoOffsetTOperands",
                Operand::Memory(memory),
            ) => into.push(MachineOperand::Register(memory.base)),
            ("addT2MemRegOffsetOperands", Operand::Memory(memory)) => {
                into.push(MachineOperand::Register(memory.base));
                into.push(MachineOperand::Register(memory.offset_register));
                immediate(into, i64::from(memory.shift_immediate));
            }
            ("addT2SOImmNotOperands", _) => {
                immediate(into, i64::from(!(operand.constant().unwrap_or(0) as u32)))
            }
            ("addT2SOImmNegOperands", _) => immediate(
                into,
                i64::from((operand.constant().unwrap_or(0) as u32).wrapping_neg()),
            ),
            ("addUnsignedOffset_b8s2Operands", Operand::Immediate(held)) => match held {
                Expression::Constant(value) => immediate(into, value >> 2),
                other => into.push(MachineOperand::Expression(other.clone())),
            },
            ("addThumbMemPCOperands", Operand::Immediate(held)) => expression(into, held),
            ("addThumbMemPCOperands", Operand::Memory(memory)) => match &memory.offset_immediate {
                Some(held) => expression(into, held),
                None => immediate(into, 0),
            },
            ("addThumbModImmNeg8_255Operands" | "addThumbModImmNeg1_7Operands", _) => immediate(
                into,
                i64::from((operand.constant().unwrap_or(0) as u32).wrapping_neg()),
            ),
            (
                "addImm8s4Operands"
                | "addImm7s4Operands"
                | "addImm7Shift0Operands"
                | "addImm7Shift1Operands"
                | "addImm7Shift2Operands"
                | "addImm7Operands",
                _,
            ) => immediate(into, operand.constant().unwrap_or(0)),
            ("addImm0_508s4NegOperands", _) => {
                immediate(into, -operand.constant().unwrap_or(0) / 4)
            }
            ("addMemBarrierOptOperands", Operand::MemoryBarrier(value))
            | ("addInstSyncBarrierOptOperands", Operand::InstructionBarrier(value)) => {
                immediate(into, i64::from(*value))
            }
            ("addImm1_16Operands" | "addImm1_32Operands", _) => {
                immediate(into, operand.constant().unwrap_or(1) - 1)
            }
            ("addImmThumbSROperands" | "addPKHASRImmOperands", _) => {
                let value = operand.constant().unwrap_or(0);
                immediate(into, if value == 32 { 0 } else { value })
            }
            ("addFBits16Operands", _) => immediate(into, 16 - operand.constant().unwrap_or(0)),
            ("addFBits32Operands", _) => immediate(into, 32 - operand.constant().unwrap_or(0)),
            ("addImm0_508s4Operands" | "addImm0_1020s4Operands", _) => {
                immediate(into, operand.constant().unwrap_or(0) / 4)
            }
            ("addBitfieldOperands", Operand::Bitfield { lsb, width }) => {
                let mask = !(((0xffff_ffffu32 >> lsb) << (32 - width)) >> (32 - (lsb + width)));
                immediate(into, i64::from(mask))
            }
            ("addShifterImmOperands", Operand::ShifterImmediate { arithmetic, amount }) => {
                immediate(into, i64::from((u32::from(*arithmetic) << 5) | amount))
            }
            ("addRotImmOperands", Operand::RotateImmediate(amount)) => {
                immediate(into, i64::from(amount >> 3))
            }
            _ => {
                for _ in 0..count {
                    into.push(nothing.clone());
                }
            }
        }
    }

    fn arm32_class_predicate(&self, class: usize) -> String {
        let held = &self.matcher.classes[class];
        if held.predicate.is_empty() {
            format!("is{}", held.name)
        } else {
            held.predicate.clone()
        }
    }

    fn arm32_validate(&self, operand: &ParsedOperand, class: usize) -> bool {
        let matcher = self.matcher;
        let held = &matcher.classes[class];
        match held.kind {
            ClassKind::Token => {
                if let ParsedKind::Token(text) = &operand.kind {
                    if let Some(found) = matcher.classes.iter().position(|candidate| {
                        candidate.kind == ClassKind::Token
                            && candidate.value_name.eq_ignore_ascii_case(text)
                    }) {
                        return matcher.is_subclass(found, class);
                    }
                }
                false
            }
            ClassKind::Register => match operand_of(operand) {
                Some(Operand::Register(register)) => held.registers.contains(register),
                _ => false,
            },
            ClassKind::User => match operand_of(operand) {
                Some(held_operand) => predicate(
                    self.target,
                    &self.arm32_class_predicate(class),
                    held_operand,
                    self.thumb_state(),
                ),
                None => false,
            },
        }
    }

    fn arm32_match(&self, operands: &[ParsedOperand]) -> Outcome {
        let matcher = self.matcher;
        let target = self.target;
        if operands.len() > matcher.maximum_operands + 1 {
            return Outcome::InvalidOperand(matcher.maximum_operands + 1);
        }
        let mnemonic = operands
            .first()
            .map(|operand| operand.token_text().to_ascii_lowercase())
            .unwrap_or_default();
        let Some((start, end)) = matcher.ranges.get(&(0, mnemonic.clone())).copied() else {
            return Outcome::MnemonicFail;
        };
        let mut had_feature_failure = false;
        let mut error_index: Option<usize> = None;
        let mut missing: Option<Vec<String>> = None;
        for at in start..end {
            let entry = &matcher.matchables[at];
            if entry.mnemonic.to_ascii_lowercase() != mnemonic {
                continue;
            }
            let mut predicates = target.instructions[entry.result].predicates.clone();
            if let Some(alias) = entry.alias {
                predicates.extend(target.aliases[alias].predicates.clone());
            }
            if mnemonic == "nop"
                && target.instructions[entry.result].record == "tMOVr"
                && (self.arm32_thumb_two() || self.arm_features.contains("HasV6MOps"))
            {
                continue;
            }
            let has_features = armv8::predicates_hold(target, &predicates, &self.arm_features);
            let mut actual_of: Vec<Option<usize>> = Vec::new();
            let mut actual = 1;
            let mut valid = true;
            for formal in 0..entry.operands.len() {
                let class = entry.operands[formal].class;
                if actual < operands.len() && self.arm32_validate(&operands[actual], class) {
                    actual_of.push(Some(actual));
                    actual += 1;
                    continue;
                }
                if matcher.classes[class].optional {
                    actual_of.push(None);
                    continue;
                }
                valid = false;
                if error_index.is_none_or(|held| held <= actual) {
                    error_index = Some(actual);
                }
                break;
            }
            if valid && actual < operands.len() {
                valid = false;
                if error_index.is_none_or(|held| held <= actual) {
                    error_index = Some(actual);
                }
            }
            if !valid {
                continue;
            }
            if !has_features {
                had_feature_failure = true;
                let required: Vec<String> = predicates
                    .iter()
                    .filter(|predicate| {
                        !armv8::predicates_hold(
                            target,
                            std::slice::from_ref(predicate),
                            &self.arm_features,
                        )
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
                        let class = &matcher.classes[entry.operands[*operand].class];
                        let held =
                            actual_of[*operand].and_then(|index| operand_of(&operands[index]));
                        let method = match class.name.as_str() {
                            "CondCode" => "addCondCodeOperands".to_string(),
                            "CCOut" => "addCCOutOperands".to_string(),
                            name if class.render.is_empty() => format!("add{name}Operands"),
                            _ => class.render.clone(),
                        };
                        self.arm32_render(&method, held, *count, &mut instruction.operands);
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
            if !self.arm32_target_predicate(&instruction) {
                if error_index.is_none() {
                    error_index = Some(1);
                }
                continue;
            }
            return Outcome::Success(instruction);
        }
        if had_feature_failure {
            return Outcome::MissingFeature(missing.unwrap_or_default());
        }
        Outcome::InvalidOperand(error_index.unwrap_or(0))
    }

    fn arm32_feature(&self, name: &str) -> Option<String> {
        self.target
            .records
            .values()
            .find(|record| record.is_a("SubtargetFeature") && table_text(record, "Name") == name)
            .map(|record| record.name.clone())
    }

    fn arm32_set_features(&mut self, enable: &[&str], disable: &[&str]) {
        let target = self.target;
        for name in disable {
            if let Some(held) = self.arm32_feature(name) {
                armv8::clear_feature(target, &mut self.arm_features, &held);
            }
        }
        for name in enable {
            if let Some(held) = self.arm32_feature(name) {
                armv8::set_feature(target, &mut self.arm_features, &held);
            }
        }
        self.streamer.set_arm_features(&self.arm_features);
    }

    fn arm32_switch_mode(&mut self) {
        let thumb = self.arm_thumb;
        if thumb {
            self.arm32_set_features(&["thumb-mode"], &[]);
        } else {
            self.arm32_set_features(&[], &["thumb-mode"]);
        }
    }

    fn arm32_text_directive(&mut self, name: &str) -> Option<bool> {
        let text = self.parse_string_to_end_of_statement();
        let text = text.trim();
        let line = if text.is_empty() {
            format!("\t{name}")
        } else {
            format!("\t{name}\t{text}")
        };
        self.streamer.emit_arm_directive(&line);
        Some(false)
    }

    pub(super) fn arm32_directive(
        &mut self,
        identifier: &str,
        location: SourceLocation,
    ) -> Option<bool> {
        match identifier {
            ".syntax" => {
                let at = self.location();
                let mode = if self.is(TokenKind::Identifier) {
                    let text = self.lexer.token.text.clone();
                    self.lex();
                    text
                } else {
                    String::new()
                };
                if mode != "unified" && mode != "divided" {
                    return Some(self.error(at, "unrecognized syntax mode in .syntax directive"));
                }
                if self.parse_end_of_line() {
                    return Some(true);
                }
                self.streamer
                    .emit_arm_directive(&format!("\t.syntax\t{mode}"));
                Some(false)
            }
            ".arm" | ".thumb" => {
                if self.parse_end_of_line() {
                    return Some(true);
                }
                self.arm_thumb = identifier == ".thumb";
                self.arm32_switch_mode();
                self.streamer.emit_arm_directive(if self.arm_thumb {
                    "\t.code\t16"
                } else {
                    "\t.code\t32"
                });
                Some(false)
            }
            ".code" => {
                let at = self.location();
                let Ok((expression, _)) = self.parse_expression() else {
                    return Some(true);
                };
                let value = match expression {
                    Expression::Constant(16) => true,
                    Expression::Constant(32) => false,
                    _ => {
                        return Some(
                            self.error(at, "invalid operand to .code directive, expected 16 or 32"),
                        )
                    }
                };
                if self.parse_end_of_line() {
                    return Some(true);
                }
                self.arm_thumb = value;
                self.arm32_switch_mode();
                self.streamer
                    .emit_arm_directive(if value { "\t.code\t16" } else { "\t.code\t32" });
                Some(false)
            }
            ".thumb_func" | ".thumb_set" | ".fnstart" | ".fnend" | ".cantunwind"
            | ".handlerdata" | ".personality" | ".personalityindex" | ".save" | ".vsave"
            | ".setfp" | ".pad" | ".movsp" | ".unwind_raw" | ".eabi_attribute" | ".object_arch"
            | ".tlsdescseq" | ".even" => self.arm32_text_directive(identifier),
            ".hword" => Some(self.parse_directive_value(2)),
            ".word" => Some(self.parse_directive_value(4)),
            ".align" => Some(self.parse_directive_align(true, 1)),
            ".ltorg" | ".pool" => {
                if self.parse_end_of_line() {
                    return Some(true);
                }
                Some(false)
            }
            ".fpu" => {
                let at = self.location();
                let name = self.parse_string_to_end_of_statement().trim().to_string();
                let (enable, disable): (&[&str], &[&str]) = match name.as_str() {
                    "none" | "softvfp" => (
                        &[],
                        &["vfp2", "vfp3", "vfp4", "fp-armv8", "neon", "d32", "crypto"],
                    ),
                    "vfp" | "vfpv2" => (&["vfp2"], &["vfp3", "neon"]),
                    "vfpv3" | "vfpv3-fp16" => (&["vfp3", "d32"], &[]),
                    "vfpv3-d16" | "vfpv3-d16-fp16" => (&["vfp3"], &["d32"]),
                    "vfpv4" => (&["vfp4", "d32"], &[]),
                    "vfpv4-d16" => (&["vfp4"], &["d32"]),
                    "fpv4-sp-d16" | "fpv5-sp-d16" => (&["vfp4"], &["d32", "fp64"]),
                    "fpv5-d16" => (&["fp-armv8"], &["d32"]),
                    "fp-armv8" => (&["fp-armv8", "d32"], &[]),
                    "neon" | "neon-vfpv3" | "neon-fp16" => (&["neon", "vfp3", "d32"], &[]),
                    "neon-vfpv4" => (&["neon", "vfp4", "d32"], &[]),
                    "neon-fp-armv8" => (&["neon", "fp-armv8", "d32"], &[]),
                    "crypto-neon-fp-armv8" => (&["neon", "fp-armv8", "d32", "crypto"], &[]),
                    _ => return Some(self.error(at, "Unknown FPU name")),
                };
                self.arm32_set_features(enable, disable);
                self.streamer.emit_arm_directive(&format!("\t.fpu\t{name}"));
                Some(false)
            }
            ".cpu" => {
                let at = self.location();
                let name = self.parse_string_to_end_of_statement().trim().to_string();
                let target = self.target;
                let known = target.records.values().any(|record| {
                    record.is_a("ProcessorModel") && table_text(record, "Name") == name
                });
                if !known {
                    return Some(self.error(at, "unknown CPU name"));
                }
                self.arm_features = armv8::feature_set(target, &name, &[]);
                self.streamer.set_arm_features(&self.arm_features);
                self.streamer.emit_arm_directive(&format!("\t.cpu\t{name}"));
                Some(false)
            }
            ".arch" | ".arch_extension" => self.arm32_text_directive(identifier),
            _ => {
                let _ = location;
                None
            }
        }
    }

    fn arm32_track_it_block(&mut self, instruction: &MachineInstruction) {
        let record = self.target.instructions[instruction.opcode].record.as_str();
        if record == "ITasm" || record == "t2IT" {
            let mask = instruction.immediate_at(1) as u32;
            self.arm_it_remaining = 4 - mask.trailing_zeros().min(3);
            return;
        }
        self.arm_it_remaining = self.arm_it_remaining.saturating_sub(1);
    }

    fn arm32_thumb_two(&self) -> bool {
        self.arm_thumb && self.arm_features.contains("FeatureThumb2")
    }

    fn arm32_target_predicate(&self, instruction: &MachineInstruction) -> bool {
        let target = self.target;
        let description = &target.instructions[instruction.opcode];
        let record = target.records.get(&description.record);
        let flag_setting = record.is_some_and(|held| {
            matches!(
                held.field("thumbArithFlagSetting"),
                Some(TableValue::Bit(true))
            )
        });
        let thumb_two = self.arm32_thumb_two();
        let in_it = self.arm_it_remaining > 0;
        if flag_setting {
            let mut carry = false;
            for group in &description.groups {
                if group.record == "s_cc_out" {
                    carry = instruction.register_at(group.first) != NO_REGISTER;
                }
            }
            if self.arm_thumb && !thumb_two && !carry {
                return false;
            }
            if thumb_two && !carry && !in_it {
                return false;
            }
            if thumb_two && carry && in_it {
                return false;
            }
            if description.record == "tLSLri" && instruction.immediate_at(3) == 0 && in_it {
                return false;
            }
        }
        let version8 = self.arm_features.contains("HasV8Ops");
        let stack = target
            .register_by_name
            .get("sp")
            .copied()
            .unwrap_or(NO_REGISTER);
        let counter = target
            .register_by_name
            .get("pc")
            .copied()
            .unwrap_or(NO_REGISTER);
        for group in &description.groups {
            if group.record == "rGPR" && group.count == 1 {
                if let MachineOperand::Register(register) = instruction.operand(group.first) {
                    if *register == stack && !version8 {
                        return false;
                    }
                    if *register == counter {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn arm32_record_index(&self, record: &str) -> Option<usize> {
        self.target.instruction_by_record.get(record).copied()
    }

    fn arm32_process(&self, instruction: &mut MachineInstruction, wide: bool) -> bool {
        let target = self.target;
        let record = target.instructions[instruction.opcode].record.as_str();
        let operand = |index: usize| instruction.operand(index).clone();
        let in_it = self.arm_it_remaining > 0;
        let register = |name: &str| {
            target
                .register_by_name
                .get(name)
                .copied()
                .unwrap_or(NO_REGISTER)
        };
        let (cpsr, stack, counter, link) = (
            register("cpsr"),
            register("sp"),
            register("pc"),
            register("lr"),
        );
        let low = |held: RegisterId| in_class(target, "tGPR", held);
        let first_token = self.arm_mnemonic.clone();
        let operand_count = self.arm_operand_count;
        let writeback = self.arm_writeback;
        let shift_for = |prefix: &str| match prefix {
            "ASR" => Shift::Asr,
            "LSR" => Shift::Lsr,
            "LSL" => Shift::Lsl,
            _ => Shift::Ror,
        };
        let build = |name: &str, operands: Vec<MachineOperand>| -> Option<MachineInstruction> {
            let mut built = MachineInstruction::new(self.arm32_record_index(name)?);
            built.operands = operands;
            Some(built)
        };
        let replacement = match record {
            "ASRr" | "LSRr" | "LSLr" | "RORr" => {
                let shifter = so_register_opcode(shift_for(&record[..3]), 0);
                build(
                    "MOVsr",
                    vec![
                        operand(0),
                        operand(1),
                        operand(2),
                        MachineOperand::Immediate(shifter),
                        operand(3),
                        operand(4),
                        operand(5),
                    ],
                )
            }
            "ASRi" | "LSRi" | "LSLi" | "RORi" => {
                let shift = shift_for(&record[..3]);
                let mut amount = instruction.immediate_at(2) as u32;
                let name = if amount == 0 { "MOVr" } else { "MOVsi" };
                if amount == 32 && matches!(shift, Shift::Lsr | Shift::Asr) {
                    amount = 0;
                }
                let mut operands = vec![operand(0), operand(1)];
                if name == "MOVsi" {
                    operands.push(MachineOperand::Immediate(so_register_opcode(shift, amount)));
                }
                operands.extend([operand(3), operand(4), operand(5)]);
                build(name, operands)
            }
            "RRXi" => build(
                "MOVsi",
                vec![
                    operand(0),
                    operand(1),
                    MachineOperand::Immediate(so_register_opcode(Shift::Rrx, 0)),
                    operand(2),
                    operand(3),
                    operand(4),
                ],
            ),
            "MOVsi" => {
                let packed = instruction.immediate_at(2) as u32;
                if packed & 7 == Shift::Lsl.code() && packed >> 3 == 0 {
                    build(
                        "MOVr",
                        vec![operand(0), operand(1), operand(3), operand(4), operand(5)],
                    )
                } else {
                    None
                }
            }
            "t2ADDri12" | "t2SUBri12" | "t2ADDspImm12" | "t2SUBspImm12" => {
                let token = first_token.as_str();
                if (token != "add" && token != "sub")
                    || thumb_so_immediate(instruction.immediate_at(2) as u32).is_none()
                {
                    None
                } else {
                    let name = match record {
                        "t2ADDri12" => "t2ADDri",
                        "t2SUBri12" => "t2SUBri",
                        "t2ADDspImm12" => "t2ADDspImm",
                        _ => "t2SUBspImm",
                    };
                    let mut operands = instruction.operands.clone();
                    operands.push(MachineOperand::Register(NO_REGISTER));
                    build(name, operands)
                }
            }
            "tADDi8" | "tSUBi8" => {
                if instruction.operand(3).immediate() >= 0
                    && (instruction.immediate_at(3) as u32) < 8
                    && operand_count == 2
                {
                    let mut built = instruction.clone();
                    built.opcode = self
                        .arm32_record_index(if record == "tADDi8" {
                            "tADDi3"
                        } else {
                            "tSUBi3"
                        })
                        .unwrap_or(instruction.opcode);
                    *instruction = built;
                    return true;
                }
                None
            }
            "t2ADDri" | "t2SUBri" => {
                let low_destination = low(instruction.register_at(0));
                let carry = if in_it { NO_REGISTER } else { cpsr };
                let narrow = !wide
                    && instruction.register_at(0) == instruction.register_at(1)
                    && low_destination
                    && instruction.register_at(5) == carry
                    && matches!(instruction.operand(2), MachineOperand::Immediate(value) if (0..=255).contains(value));
                if narrow {
                    build(
                        if record == "t2ADDri" {
                            "tADDi8"
                        } else {
                            "tSUBi8"
                        },
                        vec![
                            operand(0),
                            operand(5),
                            operand(0),
                            operand(2),
                            operand(3),
                            operand(4),
                        ],
                    )
                } else {
                    None
                }
            }
            "t2ADDspImm" | "t2SUBspImm" => {
                let value = instruction.immediate_at(2) as u32;
                if instruction.register_at(5) != NO_REGISTER
                    || wide
                    || value & 3 != 0
                    || value > (127 << 2)
                {
                    None
                } else {
                    build(
                        if record == "t2ADDspImm" {
                            "tADDspi"
                        } else {
                            "tSUBspi"
                        },
                        vec![
                            MachineOperand::Register(stack),
                            MachineOperand::Register(stack),
                            MachineOperand::Immediate(i64::from(value / 4)),
                            operand(3),
                            operand(4),
                        ],
                    )
                }
            }
            "t2ADDrr" => {
                let destination = instruction.register_at(0);
                let mut transform = destination == instruction.register_at(1);
                let mut swap = false;
                if !transform && destination == instruction.register_at(2) {
                    transform = true;
                    swap = true;
                }
                if !transform || instruction.register_at(5) != NO_REGISTER || wide {
                    None
                } else {
                    build(
                        "tADDhirr",
                        vec![
                            operand(0),
                            operand(0),
                            operand(if swap { 1 } else { 2 }),
                            operand(3),
                            operand(4),
                        ],
                    )
                }
            }
            "tADDrSP" => {
                if instruction.register_at(0) != instruction.register_at(2) {
                    let mut operands = instruction.operands.clone();
                    operands.push(MachineOperand::Register(NO_REGISTER));
                    build("t2ADDrr", operands)
                } else {
                    None
                }
            }
            "tB" | "t2B" => {
                if instruction.immediate_at(1) != 14 && !in_it {
                    let mut built = instruction.clone();
                    built.opcode = self
                        .arm32_record_index(if record == "tB" { "tBcc" } else { "t2Bcc" })
                        .unwrap_or(instruction.opcode);
                    *instruction = built;
                    return true;
                }
                None
            }
            "t2Bcc" | "tBcc" => {
                if instruction.immediate_at(1) == 14 || (record == "t2Bcc" && in_it) {
                    let mut built = instruction.clone();
                    built.opcode = self
                        .arm32_record_index(if record == "tBcc" { "tB" } else { "t2B" })
                        .unwrap_or(instruction.opcode);
                    *instruction = built;
                    return true;
                }
                None
            }
            "t2MOVi" => {
                let carry = if in_it { NO_REGISTER } else { cpsr };
                if low(instruction.register_at(0))
                    && matches!(instruction.operand(1), MachineOperand::Immediate(value) if (0..=255).contains(value))
                    && instruction.register_at(4) == carry
                    && !wide
                {
                    build(
                        "tMOVi8",
                        vec![operand(0), operand(4), operand(1), operand(2), operand(3)],
                    )
                } else {
                    None
                }
            }
            "t2MOVr" => {
                if low(instruction.register_at(0))
                    && low(instruction.register_at(1))
                    && instruction.immediate_at(2) == 14
                    && instruction.register_at(4) == cpsr
                    && !wide
                {
                    build("tMOVSr", vec![operand(0), operand(1)])
                } else {
                    None
                }
            }
            "t2SXTH" | "t2SXTB" | "t2UXTH" | "t2UXTB" => {
                if low(instruction.register_at(0))
                    && low(instruction.register_at(1))
                    && instruction.immediate_at(2) == 0
                    && !wide
                {
                    build(
                        &format!("t{}", &record[2..]),
                        vec![operand(0), operand(1), operand(3), operand(4)],
                    )
                } else {
                    None
                }
            }
            "t2LSLri" | "t2LSRri" | "t2ASRri" => {
                let carry = if in_it { NO_REGISTER } else { cpsr };
                if low(instruction.register_at(0))
                    && low(instruction.register_at(1))
                    && instruction.register_at(5) == carry
                    && !wide
                {
                    build(
                        &format!("t{}", &record[2..]),
                        vec![
                            operand(0),
                            operand(5),
                            operand(1),
                            operand(2),
                            operand(3),
                            operand(4),
                        ],
                    )
                } else {
                    None
                }
            }
            "t2LSLrr" | "t2LSRrr" | "t2ASRrr" | "t2SBCrr" | "t2RORrr" | "t2BICrr" => {
                let carry = if in_it { NO_REGISTER } else { cpsr };
                if low(instruction.register_at(1))
                    && low(instruction.register_at(2))
                    && instruction.register_at(0) == instruction.register_at(1)
                    && instruction.register_at(5) == carry
                    && !wide
                {
                    let name = match record {
                        "t2LSLrr" => "tLSLrr",
                        "t2LSRrr" => "tLSRrr",
                        "t2ASRrr" => "tASRrr",
                        "t2SBCrr" => "tSBC",
                        "t2RORrr" => "tROR",
                        _ => "tBIC",
                    };
                    build(
                        name,
                        vec![
                            operand(0),
                            operand(5),
                            operand(1),
                            operand(2),
                            operand(3),
                            operand(4),
                        ],
                    )
                } else {
                    None
                }
            }
            "t2ANDrr" | "t2EORrr" | "t2ADCrr" | "t2ORRrr" => {
                let carry = if in_it { NO_REGISTER } else { cpsr };
                if low(instruction.register_at(1))
                    && low(instruction.register_at(2))
                    && (instruction.register_at(0) == instruction.register_at(1)
                        || instruction.register_at(0) == instruction.register_at(2))
                    && instruction.register_at(5) == carry
                    && !wide
                {
                    let name = match record {
                        "t2ADCrr" => "tADC",
                        "t2ANDrr" => "tAND",
                        "t2EORrr" => "tEOR",
                        _ => "tORR",
                    };
                    let (first, second) =
                        if instruction.register_at(0) == instruction.register_at(1) {
                            (operand(1), operand(2))
                        } else {
                            (operand(2), operand(1))
                        };
                    build(
                        name,
                        vec![
                            operand(0),
                            operand(5),
                            first,
                            second,
                            operand(3),
                            operand(4),
                        ],
                    )
                } else {
                    None
                }
            }
            "tLDMIA" => {
                let base = instruction.register_at(0);
                let list_contains_base = (3..instruction.operands.len())
                    .any(|index| instruction.register_at(index) == base);
                let non_low = (3..instruction.operands.len()).any(|index| {
                    !low(instruction.register_at(index)) && instruction.register_at(index) != base
                });
                if non_low
                    || (!list_contains_base && !writeback)
                    || (list_contains_base && writeback)
                {
                    let mut operands = instruction.operands.clone();
                    let name = if writeback {
                        operands.insert(0, MachineOperand::Register(base));
                        "t2LDMIA_UPD"
                    } else {
                        "t2LDMIA"
                    };
                    build(name, operands)
                } else {
                    None
                }
            }
            "tSTMIA_UPD" => {
                let base = instruction.register_at(0);
                let non_low = (4..instruction.operands.len()).any(|index| {
                    !low(instruction.register_at(index)) && instruction.register_at(index) != base
                });
                if non_low {
                    let mut built = instruction.clone();
                    built.opcode = self
                        .arm32_record_index("t2STMIA_UPD")
                        .unwrap_or(instruction.opcode);
                    *instruction = built;
                    return true;
                }
                None
            }
            "tPOP" | "tPUSH" => {
                let extra = if record == "tPOP" { counter } else { link };
                let non_low = (2..instruction.operands.len()).any(|index| {
                    let held = instruction.register_at(index);
                    !low(held) && held != extra
                });
                if non_low {
                    let mut operands = instruction.operands.clone();
                    operands.insert(0, MachineOperand::Register(stack));
                    operands.insert(0, MachineOperand::Register(stack));
                    build(
                        if record == "tPOP" {
                            "t2LDMIA_UPD"
                        } else {
                            "t2STMDB_UPD"
                        },
                        operands,
                    )
                } else {
                    None
                }
            }
            "ANDrsi" | "ORRrsi" | "EORrsi" | "BICrsi" | "SUBrsi" | "ADDrsi" => {
                let packed = instruction.immediate_at(3) as u32;
                let shift = packed & 7;
                if shift != Shift::Rrx.code() && packed >> 3 == 0 && !matches!(shift, 3 | 1) {
                    let name = format!("{}rr", &record[..3]);
                    build(
                        &name,
                        vec![
                            operand(0),
                            operand(1),
                            operand(2),
                            operand(4),
                            operand(5),
                            operand(6),
                        ],
                    )
                } else {
                    None
                }
            }
            _ => None,
        };
        match replacement {
            Some(built) => {
                *instruction = built;
                true
            }
            None => false,
        }
    }

    pub(super) fn arm32_parse_and_match(
        &mut self,
        identifier: &str,
        identifier_token: Token,
    ) -> bool {
        self.arm_suffixes.clear();
        let Ok((mut operands, _)) =
            self.arm32_parse_instruction(identifier, identifier_token.start)
        else {
            return true;
        };
        if self.has_pending_error() {
            return true;
        }
        let location = identifier_token.location();
        match self.arm32_match(&operands) {
            Outcome::Success(mut instruction) => {
                self.arm_mnemonic = operands
                    .first()
                    .map(|held| held.token_text().to_string())
                    .unwrap_or_default();
                self.arm_operand_count = operands
                    .iter()
                    .skip(1)
                    .filter(|held| {
                        operand_of(held).is_some_and(|op| {
                            !matches!(op, Operand::Condition(_) | Operand::CarrySet(_))
                        }) || held.token_text() == "!"
                    })
                    .count();
                self.arm_writeback = operands.iter().any(|held| held.token_text() == "!");
                let wide = operands.iter().any(|held| held.token_text() == ".w");
                while self.arm32_process(&mut instruction, wide) {}
                self.arm32_track_it_block(&instruction);
                let locations = std::mem::take(&mut self.expression_locations);
                self.streamer.note_expression_locations(&locations);
                self.emit_instruction(&instruction, location);
                false
            }
            Outcome::MnemonicFail => self.error(location, "unrecognized instruction mnemonic"),
            Outcome::MissingFeature(features) => {
                let mut text = "instruction requires:".to_string();
                for feature in features {
                    text.push(' ');
                    text.push_str(&feature);
                }
                self.error(location, &text)
            }
            Outcome::InvalidOperand(index) => {
                if index >= operands.len() {
                    let end = self.lexer.token.location();
                    return self.error_range(
                        location,
                        "too few operands for instruction",
                        (location, end),
                    );
                }
                let at = self.statement_location(operands.remove(index).start);
                self.error(at, "invalid operand for instruction")
            }
        }
    }
}

#[derive(Clone, Debug)]
enum OperandEncoder {
    Machine,
    CarryOut,
    ModifiedImmediate,
    ShiftedImmediate,
    ShiftedRegister,
    LoadStoreShiftedRegister,
    AddressImmediate12,
    AddressMode2Offset,
    AddressMode3,
    AddressMode3Offset,
    PostIndexRegister,
    RegisterList,
    BitfieldMask,
    BranchTarget,
    LoadStoreMultipleMode,
    PowerTwo,
    HighLowImmediate,
    ThumbBranch(Fixup32),
    ThumbLongBranch(Fixup32),
    ThumbWideBranch,
    ArmBranch,
    ArmLink,
    ArmExchange,
    AddressMode5,
    AddressMode5Half,
    ThumbShiftedImmediate,
    ThumbShiftedRegister,
    ThumbImmediateAddress { bits: u32, shift: u32 },
    ThumbImmediateOffset,
    ThumbScaledImmediate { bits: u32, shift: u32 },
    ThumbImmediate8s4,
    ThumbImmediate7s4,
    ThumbImmediate0To1020s4,
    ThumbRegisterAddress,
    ThumbRegisterRegister,
    ThumbFiveBitAddress,
    ThumbStackAddress,
    ThumbPcAddress,
    ThumbAdrLabel,
    ThumbWideAdrLabel,
    AdrLabel,
    ItMask,
}

fn operand_encoder(text: &str) -> Result<OperandEncoder, String> {
    let (name, _) = split_arguments(text);
    Ok(match name.as_str() {
        "" | "getMachineOpValue" => OperandEncoder::Machine,
        "getCCOutOpValue" => OperandEncoder::CarryOut,
        "getModImmOpValue" => OperandEncoder::ModifiedImmediate,
        "getSORegImmOpValue" => OperandEncoder::ShiftedImmediate,
        "getSORegRegOpValue" => OperandEncoder::ShiftedRegister,
        "getLdStSORegOpValue" => OperandEncoder::LoadStoreShiftedRegister,
        "getAddrModeImm12OpValue" => OperandEncoder::AddressImmediate12,
        "getAddrMode2OffsetOpValue" => OperandEncoder::AddressMode2Offset,
        "getAddrMode3OpValue" => OperandEncoder::AddressMode3,
        "getAddrMode3OffsetOpValue" => OperandEncoder::AddressMode3Offset,
        "getPostIdxRegOpValue" => OperandEncoder::PostIndexRegister,
        "getRegisterListOpValue" => OperandEncoder::RegisterList,
        "getBitfieldInvertedMaskOpValue" => OperandEncoder::BitfieldMask,
        "getARMBranchTargetOpValue" => OperandEncoder::ArmBranch,
        "getARMBLTargetOpValue" => OperandEncoder::ArmLink,
        "getARMBLXTargetOpValue" => OperandEncoder::ArmExchange,
        "getBranchTargetOpValue" => OperandEncoder::BranchTarget,
        "getAddrMode5OpValue" => OperandEncoder::AddressMode5,
        "getAddrMode5FP16OpValue" => OperandEncoder::AddressMode5Half,
        "getLdStmModeOpValue" => OperandEncoder::LoadStoreMultipleMode,
        "getPowerTwoOpValue" => OperandEncoder::PowerTwo,
        "getHiLoImmOpValue" => OperandEncoder::HighLowImmediate,
        "getThumbBRTargetOpValue" => OperandEncoder::ThumbBranch(Fixup32::ThumbBr),
        "getThumbBCCTargetOpValue" => OperandEncoder::ThumbBranch(Fixup32::ThumbBcc),
        "getThumbCBTargetOpValue" => OperandEncoder::ThumbBranch(Fixup32::ThumbCb),
        "getThumbBLTargetOpValue" => OperandEncoder::ThumbLongBranch(Fixup32::ThumbBl),
        "getThumbBLXTargetOpValue" => OperandEncoder::ThumbLongBranch(Fixup32::ThumbBlx),
        "getThumbBranchTargetOpValue" => OperandEncoder::ThumbWideBranch,
        "getT2SOImmOpValue" => OperandEncoder::ThumbShiftedImmediate,
        "getT2SORegOpValue" => OperandEncoder::ThumbShiftedRegister,
        "getT2AddrModeImmOpValue" => {
            let (_, arguments) = split_arguments(text);
            let number = |at: usize| {
                arguments
                    .get(at)
                    .and_then(|argument| argument.parse::<u32>().ok())
                    .unwrap_or(0)
            };
            OperandEncoder::ThumbImmediateAddress {
                bits: number(0),
                shift: number(1),
            }
        }
        "getT2ScaledImmOpValue" => {
            let (_, arguments) = split_arguments(text);
            let number = |at: usize| {
                arguments
                    .get(at)
                    .and_then(|argument| argument.parse::<u32>().ok())
                    .unwrap_or(0)
            };
            OperandEncoder::ThumbScaledImmediate {
                bits: number(0),
                shift: number(1),
            }
        }
        "getT2AddrModeImm8OffsetOpValue" => OperandEncoder::ThumbImmediateOffset,
        "getT2AddrModeImm8s4OpValue" => OperandEncoder::ThumbImmediate8s4,
        "getT2AddrModeImm7s4OpValue" => OperandEncoder::ThumbImmediate7s4,
        "getT2AddrModeImm0_1020s4OpValue" => OperandEncoder::ThumbImmediate0To1020s4,
        "getT2AddrModeSORegOpValue" => OperandEncoder::ThumbRegisterAddress,
        "getThumbAddrModeRegRegOpValue" => OperandEncoder::ThumbRegisterRegister,
        "getAddrModeISOpValue" => OperandEncoder::ThumbFiveBitAddress,
        "getAddrModeThumbSPOpValue" => OperandEncoder::ThumbStackAddress,
        "getAddrModePCOpValue" => OperandEncoder::ThumbPcAddress,
        "getThumbAdrLabelOpValue" => OperandEncoder::ThumbAdrLabel,
        "getT2AdrLabelOpValue" => OperandEncoder::ThumbWideAdrLabel,
        "getAdrLabelOpValue" => OperandEncoder::AdrLabel,
        "getITMaskOpValue" => OperandEncoder::ItMask,
        _ => return Err(format!("operand encoder {text}")),
    })
}

struct EncodedVariable {
    first_operand: usize,
    encoder: OperandEncoder,
    bits: Vec<(u32, u32)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PostEncoder {
    None,
    NeonData,
    NeonLoadStore,
    NeonDuplicate,
    NeonVersion8,
    Vfp,
}

fn post_encoder(text: &str) -> Result<PostEncoder, String> {
    Ok(match text {
        "" => PostEncoder::None,
        "NEONThumb2DataIPostEncoder" => PostEncoder::NeonData,
        "NEONThumb2LoadStorePostEncoder" => PostEncoder::NeonLoadStore,
        "NEONThumb2DupPostEncoder" => PostEncoder::NeonDuplicate,
        "NEONThumb2V8PostEncoder" => PostEncoder::NeonVersion8,
        "VFPThumb2PostEncoder" => PostEncoder::Vfp,
        _ => return Err(format!("post encoder {text}")),
    })
}

struct EncodingPlan {
    constant: u32,
    variables: Vec<EncodedVariable>,
    post: PostEncoder,
}

fn locate_variable(
    target: &Target,
    index: usize,
    variable: &str,
) -> Result<(usize, OperandEncoder), String> {
    let instruction = &target.instructions[index];
    if let Some(at) = instruction.named_group(variable) {
        let group = &instruction.groups[at];
        let operand = &target.records[&group.record];
        return Ok((
            group.first,
            operand_encoder(&table_text(operand, "EncoderMethod"))?,
        ));
    }
    for group in &instruction.groups {
        let Some(sub) = group.sub_names.iter().position(|name| name == variable) else {
            continue;
        };
        let operand = &target.records[&group.record];
        let own = table_text(operand, "EncoderMethod");
        let method = if sub == 0 && !own.is_empty() {
            own
        } else {
            match operand.field("MIOperandInfo") {
                Some(TableValue::Dag { arguments, .. }) => match arguments.get(sub) {
                    Some((TableValue::Record(name), _)) => {
                        table_text(&target.records[name], "EncoderMethod")
                    }
                    _ => String::new(),
                },
                _ => String::new(),
            }
        };
        return Ok((group.first + sub, operand_encoder(&method)?));
    }
    Err(format!("no operand named {variable}"))
}

fn encoding_plan(target: &Target, index: usize) -> Result<EncodingPlan, String> {
    let instruction = &target.instructions[index];
    let record = &target.records[&instruction.record];
    let Some(TableValue::Bits(bits)) = record.field("Inst") else {
        return Err("no Inst field".to_string());
    };
    if bits.len() != 32 && bits.len() != 16 {
        return Err(format!("a {}-bit encoding", bits.len()));
    }
    let mut constant = 0u32;
    let mut variables: Vec<EncodedVariable> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for (position, bit) in bits.iter().enumerate() {
        match bit {
            TableValue::Bit(true) => constant |= 1 << position,
            TableValue::Bit(false) | TableValue::Unset => {}
            TableValue::VariableBit { .. } | TableValue::Variable(_) => {
                let (variable, bit) = match bit {
                    TableValue::VariableBit { variable, bit } => (variable, *bit),
                    TableValue::Variable(variable) => (variable, 0),
                    _ => unreachable!(),
                };
                let at = match names.iter().position(|name| name == variable) {
                    Some(at) => at,
                    None => {
                        let (first_operand, encoder) = locate_variable(target, index, variable)?;
                        names.push(variable.clone());
                        variables.push(EncodedVariable {
                            first_operand,
                            encoder,
                            bits: Vec::new(),
                        });
                        names.len() - 1
                    }
                };
                variables[at].bits.push((bit, position as u32));
            }
            other => return Err(format!("Inst bit {other:?}")),
        }
    }
    Ok(EncodingPlan {
        constant,
        variables,
        post: post_encoder(&table_text(record, "PostEncoderMethod"))?,
    })
}

fn encoding_plans(target: &'static Target) -> &'static Vec<Result<EncodingPlan, String>> {
    static CELL: OnceLock<Vec<Result<EncodingPlan, String>>> = OnceLock::new();
    CELL.get_or_init(|| {
        (0..target.instructions.len())
            .map(|index| encoding_plan(target, index))
            .collect()
    })
}

fn unencodable(cause: &str) -> Diagnostic {
    fail(
        "EI501",
        "The generated machine code could not be assembled.",
    )
    .with_context(format!("Cause: {cause}"))
}

fn has_conditional_branch(instruction: &MachineInstruction) -> bool {
    let count = instruction.operands.len();
    (0..count.saturating_sub(1)).any(|index| {
        matches!(
            (instruction.operand(index), instruction.operand(index + 1)),
            (MachineOperand::Immediate(code), MachineOperand::Register(_)) if *code != 14
        ) && instruction.register_at(index + 1) != 0
    })
}

fn operand_value(
    target: &Target,
    instruction: &MachineInstruction,
    first: usize,
    encoder: &OperandEncoder,
    thumb: bool,
    fixups: &mut Vec<Fixup>,
) -> Result<u64, Diagnostic> {
    let mut add_fixup = |expression: &Expression, kind: Fixup32| {
        fixups.push(Fixup {
            offset: 0,
            expression: expression.clone(),
            kind: FixupKind::Arm32(kind),
            pc_relative: kind.pc_relative(),
        });
    };
    let conditional = has_conditional_branch(instruction);
    let register = |index: usize| -> u64 {
        u64::from(register_encoding(target, instruction.register_at(index)))
    };
    let immediate = |index: usize| instruction.immediate_at(index);
    let operand = instruction.operand(first);
    Ok(match encoder {
        OperandEncoder::Machine => match operand {
            MachineOperand::Register(register) => u64::from(register_encoding(target, *register)),
            MachineOperand::Immediate(value) => *value as u64,
            MachineOperand::Expression(_) => return Err(unencodable("an expression operand")),
        },
        OperandEncoder::CarryOut => u64::from(instruction.register_at(first) != NO_REGISTER),
        OperandEncoder::ModifiedImmediate => match operand {
            MachineOperand::Expression(expression) => {
                add_fixup(expression, Fixup32::ModImm);
                0
            }
            _ => immediate(first) as u64,
        },
        OperandEncoder::ShiftedImmediate => {
            let opcode = immediate(first + 1) as u32;
            let shift = opcode & 7;
            let amount = opcode >> 3;
            let mut binary = register(first);
            let bits = match shift {
                2 => 0u64,
                3 => 2,
                1 => 4,
                4 => 6,
                5 => {
                    return Ok(binary | 0x60);
                }
                _ => return Err(unencodable("an unknown shift")),
            };
            binary |= bits << 4;
            binary | (u64::from(amount) << 7)
        }
        OperandEncoder::ShiftedRegister => {
            let opcode = immediate(first + 2) as u32;
            let shift = opcode & 7;
            let mut binary = register(first);
            let has_register = instruction.register_at(first + 1) != NO_REGISTER;
            let bits = if has_register {
                match shift {
                    2 => 1u64,
                    3 => 3,
                    1 => 5,
                    4 => 7,
                    _ => return Err(unencodable("an unknown shift")),
                }
            } else {
                0
            };
            binary |= bits << 4;
            binary | (register(first + 1) << 8)
        }
        OperandEncoder::LoadStoreShiftedRegister => {
            let packed = immediate(first + 2) as u32;
            let amount = packed & 0xfff;
            let add = (packed >> 12) & 1 == 0;
            let shift = (packed >> 13) & 7;
            let bits = match shift {
                0 | 2 => 0u64,
                3 => 1,
                1 => 2,
                4 | 5 => 3,
                _ => return Err(unencodable("an unknown shift")),
            };
            let mut binary = register(first + 1);
            binary |= register(first) << 13;
            binary |= bits << 5;
            binary |= u64::from(amount) << 7;
            if add {
                binary |= 1 << 12;
            }
            binary
        }
        OperandEncoder::AddressImmediate12 => match operand {
            MachineOperand::Register(_) => match instruction.operand(first + 1) {
                MachineOperand::Expression(expression) => {
                    add_fixup(expression, Fixup32::LdstAbs12);
                    register(first) << 13
                }
                second => {
                    let (add, amount) = split_offset(second.immediate());
                    let mut binary = u64::from(amount & 0xfff);
                    if add {
                        binary |= 1 << 12;
                    }
                    binary | (register(first) << 13)
                }
            },
            MachineOperand::Expression(expression) => {
                add_fixup(
                    expression,
                    if thumb {
                        Fixup32::T2LdstPcrel12
                    } else {
                        Fixup32::LdstPcrel12
                    },
                );
                15 << 13
            }
            MachineOperand::Immediate(value) => {
                let (add, amount) = split_offset(*value);
                let mut binary = u64::from(amount & 0xfff);
                if add {
                    binary |= 1 << 12;
                }
                binary | (15 << 13)
            }
        },
        OperandEncoder::AddressMode2Offset => {
            let packed = immediate(first + 1) as u32;
            let add = (packed >> 12) & 1 == 0;
            let has_register = instruction.register_at(first) != NO_REGISTER;
            let mut binary = u64::from(packed & 0xfff);
            if has_register {
                let shift = (packed >> 13) & 7;
                let bits = match shift {
                    0 | 2 => 0u64,
                    3 => 1,
                    1 => 2,
                    4 | 5 => 3,
                    _ => return Err(unencodable("an unknown shift")),
                };
                binary <<= 7;
                binary |= bits << 5;
                binary |= register(first);
            }
            binary | (u64::from(add) << 12) | (u64::from(has_register) << 13)
        }
        OperandEncoder::AddressMode3 => {
            if let MachineOperand::Expression(expression) = operand {
                add_fixup(expression, Fixup32::PcrelTenUnscaled);
                return Ok((15 << 9) | (1 << 13));
            }
            let packed = immediate(first + 2) as u32;
            let add = (packed >> 8) & 1 == 0;
            let is_immediate = instruction.register_at(first + 1) == NO_REGISTER;
            let mut amount = u64::from(packed & 0xff);
            if !is_immediate {
                amount = register(first + 1);
            }
            (register(first) << 9)
                | amount
                | (u64::from(add) << 8)
                | (u64::from(is_immediate) << 13)
        }
        OperandEncoder::AddressMode3Offset => {
            let packed = immediate(first + 1) as u32;
            let add = (packed >> 8) & 1 == 0;
            let is_immediate = instruction.register_at(first) == NO_REGISTER;
            let mut amount = u64::from(packed & 0xff);
            if !is_immediate {
                amount = register(first);
            }
            amount | (u64::from(add) << 8) | (u64::from(is_immediate) << 9)
        }
        OperandEncoder::PostIndexRegister => {
            register(first) | (u64::from(immediate(first + 1) != 0) << 4)
        }
        OperandEncoder::RegisterList => {
            let mut binary = 0u64;
            for index in first..instruction.operands.len() {
                binary |= 1 << register(index);
            }
            binary
        }
        OperandEncoder::BitfieldMask => {
            let inverted = !(immediate(first) as u32);
            if inverted == 0 {
                return Err(unencodable("an illegal bitfield mask"));
            }
            let lsb = inverted.trailing_zeros();
            let msb = 31 - inverted.leading_zeros();
            u64::from(lsb | (msb << 5))
        }
        OperandEncoder::BranchTarget => match operand {
            MachineOperand::Immediate(value) => {
                if thumb {
                    *value as u64
                } else {
                    (*value >> 2) as u64
                }
            }
            MachineOperand::Expression(expression) => {
                if thumb {
                    add_fixup(expression, Fixup32::T2CondBranch);
                } else {
                    add_fixup(
                        expression,
                        if conditional {
                            Fixup32::CondBranch
                        } else {
                            Fixup32::UncondBranch
                        },
                    );
                }
                0
            }
            _ => return Err(unencodable("a branch target register")),
        },
        OperandEncoder::ArmBranch | OperandEncoder::ArmLink => match operand {
            MachineOperand::Immediate(value) => (*value >> 2) as u64,
            MachineOperand::Expression(expression) => {
                let link = matches!(encoder, OperandEncoder::ArmLink);
                add_fixup(
                    expression,
                    match (link, conditional) {
                        (false, true) => Fixup32::CondBranch,
                        (false, false) => Fixup32::UncondBranch,
                        (true, true) => Fixup32::CondBl,
                        (true, false) => Fixup32::UncondBl,
                    },
                );
                0
            }
            _ => return Err(unencodable("a branch target register")),
        },
        OperandEncoder::ArmExchange => match operand {
            MachineOperand::Immediate(value) => (*value >> 1) as u64,
            MachineOperand::Expression(expression) => {
                add_fixup(expression, Fixup32::Blx);
                0
            }
            _ => return Err(unencodable("a branch target register")),
        },
        OperandEncoder::ThumbBranch(kind) => match operand {
            MachineOperand::Immediate(value) => (*value >> 1) as u64,
            MachineOperand::Expression(expression) => {
                add_fixup(expression, *kind);
                0
            }
            _ => return Err(unencodable("a branch target register")),
        },
        OperandEncoder::ThumbLongBranch(kind) => match operand {
            MachineOperand::Immediate(value) => {
                let offset = (*value as i32) >> 1;
                let sign = (offset & 0x80_0000) >> 23;
                let mut first = ((offset & 0x40_0000) >> 22) as u32;
                let mut second = ((offset & 0x20_0000) >> 21) as u32;
                first = !first & 1;
                second = !second & 1;
                first ^= sign as u32;
                second ^= sign as u32;
                let mut result = offset & !0x60_0000;
                result |= (first as i32) << 22;
                result |= (second as i32) << 21;
                u64::from(result as u32)
            }
            MachineOperand::Expression(expression) => {
                add_fixup(expression, *kind);
                0
            }
            _ => return Err(unencodable("a branch target register")),
        },
        OperandEncoder::ThumbWideBranch => match operand {
            MachineOperand::Immediate(value) => {
                let mut held = (*value >> 1) as u32;
                let i = held & 0x80_0000 != 0;
                let j1 = held & 0x40_0000 != 0;
                let j2 = held & 0x20_0000 != 0;
                if i ^ j1 {
                    held &= !0x40_0000;
                } else {
                    held |= 0x40_0000;
                }
                if i ^ j2 {
                    held &= !0x20_0000;
                } else {
                    held |= 0x20_0000;
                }
                u64::from(held)
            }
            MachineOperand::Expression(expression) => {
                add_fixup(expression, Fixup32::T2UncondBranch);
                0
            }
            _ => return Err(unencodable("a branch target register")),
        },
        OperandEncoder::ThumbShiftedImmediate => match operand {
            MachineOperand::Expression(expression) => {
                add_fixup(expression, Fixup32::T2SoImm);
                0
            }
            _ => match thumb_so_immediate(immediate(first) as u32) {
                Some(encoded) => u64::from(encoded),
                None => return Err(unencodable("not a Thumb-2 modified immediate")),
            },
        },
        OperandEncoder::ThumbShiftedRegister => {
            let packed = immediate(first + 1) as u32;
            let shift = packed & 7;
            let mut binary = register(first);
            let bits = match shift {
                2 => 0u64,
                3 => 2,
                1 => 4,
                5 | 4 => 6,
                _ => return Err(unencodable("an unknown shift")),
            };
            binary |= bits << 4;
            if shift == 5 {
                return Ok(binary);
            }
            binary | (u64::from(packed >> 3) << 7)
        }
        OperandEncoder::ThumbImmediateAddress { bits, shift } => {
            let mut value = register(first) << (bits + 1);
            let mut held = immediate(first + 1) as i32;
            if held == i32::MIN {
                held = 0;
            } else if held < 0 {
                held = held.wrapping_abs();
            } else {
                value |= 1 << bits;
            }
            value | (((held >> shift) as u64) & ((1u64 << bits) - 1))
        }
        OperandEncoder::ThumbScaledImmediate { bits, shift } => {
            let held = immediate(first) as i32;
            let add = held >= 0;
            let magnitude = (held as i64).unsigned_abs() as u32 >> shift;
            let mut binary = u64::from(magnitude) & ((1u64 << bits) - 1);
            if add {
                binary |= 1 << bits;
            }
            binary
        }
        OperandEncoder::ThumbImmediateOffset => {
            let held = immediate(first) as u32;
            let mut binary = 0u64;
            let magnitude = if (held as i32) < 0 {
                held.wrapping_neg()
            } else {
                binary |= 256;
                held
            };
            binary | u64::from(magnitude & 255)
        }
        OperandEncoder::ThumbImmediate8s4 => {
            if let MachineOperand::Expression(expression) = operand {
                add_fixup(expression, Fixup32::T2PcrelTen);
                return Ok(15 << 9);
            }
            let (add, amount) = split_offset(immediate(first + 1));
            let mut binary = u64::from((amount >> 2) & 0xff);
            if add {
                binary |= 1 << 8;
            }
            binary | (register(first) << 9)
        }
        OperandEncoder::ThumbImmediate7s4 => {
            let (add, amount) = split_offset(immediate(first + 1));
            let mut binary = u64::from((amount >> 2) & 0xff);
            if add {
                binary |= 1 << 7;
            }
            binary | (register(first) << 8)
        }
        OperandEncoder::ThumbImmediate0To1020s4 => {
            (register(first) << 8) | (immediate(first + 1) as u64 & 0xff)
        }
        OperandEncoder::ThumbRegisterAddress => {
            let mut value = register(first);
            value <<= 4;
            value |= register(first + 1);
            value <<= 2;
            value | (immediate(first + 2) as u64 & 3)
        }
        OperandEncoder::ThumbRegisterRegister => (register(first + 1) << 3) | register(first),
        OperandEncoder::ThumbFiveBitAddress => {
            ((immediate(first + 1) as u64 & 0x1f) << 3) | register(first)
        }
        OperandEncoder::ThumbStackAddress => immediate(first + 1) as u64 & 0xff,
        OperandEncoder::ThumbPcAddress => match operand {
            MachineOperand::Immediate(value) => (*value >> 2) as u64,
            MachineOperand::Expression(expression) => {
                add_fixup(expression, Fixup32::ThumbCp);
                0
            }
            _ => return Err(unencodable("a relocatable branch target")),
        },
        OperandEncoder::ThumbAdrLabel => match operand {
            MachineOperand::Immediate(value) => *value as u64,
            MachineOperand::Expression(expression) => {
                add_fixup(expression, Fixup32::ThumbAdrPcrel10);
                0
            }
            _ => return Err(unencodable("a relocatable branch target")),
        },
        OperandEncoder::ThumbWideAdrLabel => match operand {
            MachineOperand::Immediate(value) => {
                let held = *value as i32;
                if held == i32::MIN {
                    0x1000
                } else if held < 0 {
                    u64::from(held.unsigned_abs() | 0x1000)
                } else {
                    held as u64
                }
            }
            MachineOperand::Expression(expression) => {
                add_fixup(expression, Fixup32::T2AdrPcrel12);
                0
            }
            _ => return Err(unencodable("a relocatable branch target")),
        },
        OperandEncoder::AdrLabel => match operand {
            MachineOperand::Expression(expression) => {
                add_fixup(expression, Fixup32::AdrPcrel12);
                0
            }
            MachineOperand::Immediate(value) => {
                let mut binary = 0x2000u64;
                let mut offset = *value;
                let mut found;
                if offset == i64::from(i32::MIN) {
                    binary = 0x1000;
                    found = Some(0);
                } else if offset < 0 {
                    binary = 0x1000;
                    offset = -offset;
                    found = so_immediate(offset as u32);
                    if found.is_none() {
                        binary = 0x2000;
                        offset = -offset;
                        found = so_immediate(offset as u32);
                    }
                } else {
                    found = so_immediate(offset as u32);
                    if found.is_none() {
                        binary = 0x1000;
                        offset = -offset;
                        found = so_immediate(offset as u32);
                    }
                }
                let Some(encoded) = found else {
                    return Err(unencodable("not a valid so_imm value"));
                };
                binary | u64::from(encoded)
            }
            _ => return Err(unencodable("a relocatable branch target")),
        },
        OperandEncoder::ItMask => {
            let mut mask = immediate(first) as u32;
            let condition = instruction.immediate_at(first.saturating_sub(1)) as u32;
            if condition & 1 != 0 {
                let low = mask & mask.wrapping_neg();
                let above = 0xf & (low.wrapping_neg() << 1);
                mask ^= above;
            }
            u64::from(mask)
        }
        OperandEncoder::LoadStoreMultipleMode => match immediate(first) {
            1 => 1,
            2 => 3,
            3 => 0,
            4 => 2,
            _ => return Err(unencodable("an unknown addressing sub-mode")),
        },
        OperandEncoder::PowerTwo => (immediate(first) as u64).trailing_zeros() as u64,
        OperandEncoder::HighLowImmediate => match operand {
            MachineOperand::Expression(expression) => {
                let specifier = match expression {
                    Expression::Symbol { specifier, .. } => specifier.clone().unwrap_or_default(),
                    _ => String::new(),
                };
                let kind = match (specifier.as_str(), thumb) {
                    ("upper16", false) => Fixup32::MovtHi16,
                    ("upper16", true) => Fixup32::T2MovtHi16,
                    ("lower16", false) => Fixup32::MovwLo16,
                    ("lower16", true) => Fixup32::T2MovwLo16,
                    ("upper8_15", true) => Fixup32::ThumbUpper815,
                    ("upper0_7", true) => Fixup32::ThumbUpper07,
                    ("lower8_15", true) => Fixup32::ThumbLower815,
                    ("lower0_7", true) => Fixup32::ThumbLower07,
                    _ => return Err(unencodable("an unsupported relocation specifier")),
                };
                add_fixup(expression, kind);
                0
            }
            _ => immediate(first) as u64,
        },
        OperandEncoder::AddressMode5 | OperandEncoder::AddressMode5Half => {
            let half = matches!(encoder, OperandEncoder::AddressMode5Half);
            match operand {
                MachineOperand::Expression(expression) => {
                    let kind = match (half, thumb) {
                        (false, false) => Fixup32::PcrelTen,
                        (false, true) => Fixup32::T2PcrelTen,
                        (true, false) => Fixup32::PcrelNine,
                        (true, true) => Fixup32::T2PcrelNine,
                    };
                    add_fixup(expression, kind);
                    15 << 9
                }
                _ => {
                    let packed = immediate(first + 1) as u32;
                    let add = (packed >> 8) & 1 == 0;
                    let mut binary = u64::from(packed & 0xff);
                    if add {
                        binary |= 1 << 8;
                    }
                    binary | (register(first) << 9)
                }
            }
        }
    })
}

pub fn encode(
    target: &'static Target,
    instruction: &MachineInstruction,
) -> Result<Encoded, Diagnostic> {
    let description = &target.instructions[instruction.opcode];
    let thumb = instruction.flags & THUMB_MODE != 0;
    let plan = match &encoding_plans(target)[instruction.opcode] {
        Ok(plan) => plan,
        Err(cause) => {
            return Err(unencodable(&format!("{}: {cause}", description.record)));
        }
    };
    let mut value = plan.constant;
    let mut fixups: Vec<Fixup> = Vec::new();
    for variable in &plan.variables {
        let encoded = operand_value(
            target,
            instruction,
            variable.first_operand,
            &variable.encoder,
            thumb,
            &mut fixups,
        )?;
        for (source, destination) in &variable.bits {
            if (encoded >> source) & 1 == 1 {
                value |= 1 << destination;
            }
        }
    }
    if thumb {
        value = thumb_post_encode(&plan.post, value);
    }
    let bytes = if description.size == 2 {
        (value as u16).to_le_bytes().to_vec()
    } else if thumb {
        let mut bytes = ((value >> 16) as u16).to_le_bytes().to_vec();
        bytes.extend_from_slice(&(value as u16).to_le_bytes());
        bytes
    } else {
        value.to_le_bytes().to_vec()
    };
    Ok(Encoded { bytes, fixups })
}

fn thumb_post_encode(post: &PostEncoder, value: u32) -> u32 {
    match post {
        PostEncoder::None => value,
        PostEncoder::NeonData => {
            let bit24 = value & 0x0100_0000;
            let bit28 = bit24 << 4;
            ((value & 0xefff_ffff) | bit28) | 0x0f00_0000
        }
        PostEncoder::NeonLoadStore => (value & 0xf0ff_ffff) | 0x0900_0000,
        PostEncoder::NeonDuplicate => (value & 0x00ff_ffff) | 0xee00_0000,
        PostEncoder::NeonVersion8 => value | 0x0c00_0000,
        PostEncoder::Vfp => (value & 0x0fff_ffff) | 0xe000_0000,
    }
}

fn architecture_name(text: &str) -> Option<&'static str> {
    let rest = text
        .strip_prefix("thumbeb")
        .or_else(|| text.strip_prefix("armeb"))
        .or_else(|| text.strip_prefix("thumb"))
        .or_else(|| text.strip_prefix("arm"))?;
    let rest = rest.strip_suffix("eb").unwrap_or(rest);
    Some(match rest {
        "v4" => "armv4",
        "v4t" => "armv4t",
        "v5" | "v5t" => "armv5t",
        "v5te" | "v5e" => "armv5te",
        "v5tej" | "v5ej" => "armv5tej",
        "v6" => "armv6",
        "v6j" => "armv6j",
        "v6k" => "armv6k",
        "v6z" | "v6kz" => "armv6kz",
        "v6t2" => "armv6t2",
        "v6m" => "armv6-m",
        "v6sm" => "armv6s-m",
        "v7" | "v7a" | "v7-a" => "armv7-a",
        "v7r" | "v7-r" => "armv7-r",
        "v7m" | "v7-m" => "armv7-m",
        "v7em" | "v7e-m" => "armv7e-m",
        "v7s" => "armv7s",
        "v7k" => "armv7k",
        "v8" | "v8a" | "v8-a" => "armv8-a",
        "v8.1a" => "armv8.1-a",
        "v8.2a" => "armv8.2-a",
        "v8.3a" => "armv8.3-a",
        "v8.4a" => "armv8.4-a",
        "v8.5a" => "armv8.5-a",
        "v8.6a" => "armv8.6-a",
        "v8.7a" => "armv8.7-a",
        "v8.8a" => "armv8.8-a",
        "v8.9a" => "armv8.9-a",
        "v9a" => "armv9-a",
        "v9.1a" => "armv9.1-a",
        "v9.2a" => "armv9.2-a",
        "v9.3a" => "armv9.3-a",
        "v8r" => "armv8-r",
        "v8m.base" => "armv8-m.base",
        "v8m.main" => "armv8-m.main",
        "v8.1m.main" => "armv8.1-m.main",
        _ => return None,
    })
}

pub fn initial_features(
    target: &Target,
    triple: &TargetTriple,
    cpu: &str,
    attributes: &[String],
) -> BTreeSet<String> {
    let mut combined: Vec<String> = Vec::new();
    let generic = cpu.is_empty() || cpu == "generic";
    if generic {
        if let Some(name) = architecture_name(&triple.architecture) {
            combined.push(format!("+{name}"));
        }
    }
    if triple.is_thumb() {
        if combined.is_empty() {
            combined.push("+thumb-mode".to_string());
            combined.push("+v4t".to_string());
        } else {
            combined.push("+thumb-mode".to_string());
        }
    }
    combined.extend(attributes.iter().cloned());
    armv8::feature_set(target, cpu, &combined)
}

fn print_tables() -> &'static PrintTables {
    static CELL: OnceLock<PrintTables> = OnceLock::new();
    CELL.get_or_init(|| {
        let target = target().expect("the ARMv7 tables load");
        PrintTables {
            programs: target
                .instructions
                .iter()
                .map(|instruction| print_program(target, instruction, 0))
                .collect(),
            aliases: alias_patterns(target, 0),
        }
    })
}

fn condition_name(code: u32) -> &'static str {
    const NAMES: [&str; 15] = [
        "eq", "ne", "hs", "lo", "mi", "pl", "vs", "vc", "hi", "ls", "ge", "lt", "gt", "le", "",
    ];
    NAMES.get(code as usize).copied().unwrap_or("")
}

fn shift_name(shift: u32) -> &'static str {
    match shift {
        1 => "asr",
        2 => "lsl",
        3 => "lsr",
        4 => "ror",
        5 => "rrx",
        6 => "uxtw",
        _ => "",
    }
}

fn translate_shift(amount: u32) -> u32 {
    if amount == 0 {
        32
    } else {
        amount
    }
}

fn print_hexadecimal(value: u64) -> String {
    if value == 0 {
        "0".to_string()
    } else {
        format!("{value:#x}")
    }
}

fn floating_text(value: f64) -> String {
    let text = format!("{value:.6e}");
    match text.split_once('e') {
        Some((mantissa, exponent)) => {
            let (sign, digits) = match exponent.strip_prefix('-') {
                Some(digits) => ('-', digits),
                None => ('+', exponent),
            };
            format!("{mantissa}e{sign}{digits:0>2}")
        }
        None => text,
    }
}

pub struct Printer<'t> {
    pub target: &'t Target,
    pub features: &'t BTreeSet<String>,
    pub hexadecimal: bool,
    pub markup: bool,
}

impl<'t> Printer<'t> {
    pub fn new(target: &'t Target, features: &'t BTreeSet<String>) -> Printer<'t> {
        Printer {
            target,
            features,
            hexadecimal: false,
            markup: false,
        }
    }

    fn register_name(&self, register: RegisterId) -> String {
        self.target.registers[register as usize]
            .assembly_name
            .clone()
    }

    fn register_text(&self, instruction: &MachineInstruction, index: usize) -> String {
        match instruction.operand(index) {
            MachineOperand::Register(register) => self.register_name(*register),
            _ => String::new(),
        }
    }

    fn immediate_text(&self, value: i64) -> String {
        if self.hexadecimal {
            if value < 0 {
                format!("-{}", print_hexadecimal(value.unsigned_abs()))
            } else {
                print_hexadecimal(value as u64)
            }
        } else {
            value.to_string()
        }
    }

    fn expression(&self, expression: &Expression) -> String {
        expression_text(self.target, expression, 0, Syntax::Att)
    }

    fn predicate(&self, instruction: &MachineInstruction, index: usize) -> String {
        let code = instruction.immediate_at(index) as u32;
        if code == 15 {
            "<und>".to_string()
        } else {
            condition_name(code).to_string()
        }
    }

    fn carry(&self, instruction: &MachineInstruction, index: usize) -> &'static str {
        if instruction.register_at(index) != NO_REGISTER {
            "s"
        } else {
            ""
        }
    }

    fn shift_suffix(&self, shift: u32, amount: u32) -> String {
        if shift == 0 || (shift == 2 && amount == 0) {
            return String::new();
        }
        let mut text = format!(", {}", shift_name(shift));
        if shift != 5 {
            text.push_str(&format!(" #{}", translate_shift(amount)));
        }
        text
    }

    fn register_list(&self, instruction: &MachineInstruction, first: usize) -> String {
        let names: Vec<String> = (first..instruction.operands.len())
            .map(|index| self.register_text(instruction, index))
            .collect();
        format!("{{{}}}", names.join(", "))
    }

    pub fn print(&self, instruction: &MachineInstruction) -> String {
        let text = self.print_plain(instruction);
        if self.markup {
            armv8::apply_markup(&text)
        } else {
            text
        }
    }

    fn special(&self, instruction: &MachineInstruction) -> Option<String> {
        let record = self.target.instructions[instruction.opcode].record.as_str();
        let stack = |index: usize| {
            self.target.register_by_name.get("sp").copied() == Some(instruction.register_at(index))
        };
        match record {
            "MOVsr" => {
                let shift = (instruction.immediate_at(3) as u32) & 7;
                Some(format!(
                    "\t{}{}{}\t{}, {}, {}",
                    shift_name(shift),
                    self.carry(instruction, 6),
                    self.predicate(instruction, 4),
                    self.register_text(instruction, 0),
                    self.register_text(instruction, 1),
                    self.register_text(instruction, 2)
                ))
            }
            "MOVsi" => {
                let packed = instruction.immediate_at(2) as u32;
                let shift = packed & 7;
                let mut text = format!(
                    "\t{}{}{}\t{}, {}",
                    shift_name(shift),
                    self.carry(instruction, 5),
                    self.predicate(instruction, 3),
                    self.register_text(instruction, 0),
                    self.register_text(instruction, 1)
                );
                if shift != 5 {
                    text.push_str(&format!(", #{}", translate_shift(packed >> 3)));
                }
                Some(text)
            }
            "STMDB_UPD" | "t2STMDB_UPD" if stack(0) && instruction.operands.len() > 5 => {
                Some(format!(
                    "\tpush{}{}\t{}",
                    self.predicate(instruction, 2),
                    if record == "t2STMDB_UPD" { ".w" } else { "" },
                    self.register_list(instruction, 4)
                ))
            }
            "STR_PRE_IMM" if stack(2) && instruction.immediate_at(3) == -4 => Some(format!(
                "\tpush{}\t{{{}}}",
                self.predicate(instruction, 4),
                self.register_text(instruction, 1)
            )),
            "LDMIA_UPD" | "t2LDMIA_UPD" if stack(0) && instruction.operands.len() > 5 => {
                Some(format!(
                    "\tpop{}{}\t{}",
                    self.predicate(instruction, 2),
                    if record == "t2LDMIA_UPD" { ".w" } else { "" },
                    self.register_list(instruction, 4)
                ))
            }
            "LDR_POST_IMM" if stack(2) && instruction.immediate_at(4) == 4 => Some(format!(
                "\tpop{}\t{{{}}}",
                self.predicate(instruction, 5),
                self.register_text(instruction, 0)
            )),
            "VSTMSDB_UPD" | "VSTMDDB_UPD" if stack(0) => Some(format!(
                "\tvpush{}\t{}",
                self.predicate(instruction, 2),
                self.register_list(instruction, 4)
            )),
            "VLDMSIA_UPD" | "VLDMDIA_UPD" if stack(0) => Some(format!(
                "\tvpop{}\t{}",
                self.predicate(instruction, 2),
                self.register_list(instruction, 4)
            )),
            "tLDMIA" => {
                let base = instruction.register_at(0);
                let writeback = !(3..instruction.operands.len())
                    .any(|index| instruction.register_at(index) == base);
                Some(format!(
                    "\tldm{}\t{}{}, {}",
                    self.predicate(instruction, 1),
                    self.register_name(base),
                    if writeback { "!" } else { "" },
                    self.register_list(instruction, 3)
                ))
            }
            "TSB" | "t2TSB" => Some("\ttsb\tcsync".to_string()),
            _ => None,
        }
    }

    fn print_plain(&self, instruction: &MachineInstruction) -> String {
        if let Some(text) = self.special(instruction) {
            return text;
        }
        let tables = print_tables();
        if let Some(text) = self.alias(tables, instruction) {
            return text;
        }
        let mut out = String::new();
        for piece in &tables.programs[instruction.opcode] {
            match piece {
                PrintPiece::Literal(text) => out.push_str(text),
                PrintPiece::Operand { method, index, .. } => {
                    self.call_method(method, instruction, *index, &mut out)
                }
            }
        }
        out
    }

    fn alias(&self, tables: &PrintTables, instruction: &MachineInstruction) -> Option<String> {
        let patterns = tables.aliases.get(&instruction.opcode)?;
        let mut chosen = None;
        for pattern in patterns {
            if instruction.operands.len() != pattern.operand_count {
                return None;
            }
            if let Some(record) = self.target.records.get(&pattern.record) {
                let predicates = table_record_list(record, "Predicates");
                if !armv8::predicates_hold(self.target, &predicates, self.features) {
                    continue;
                }
            }
            let mut index = 0;
            let matched = pattern.conditions.iter().all(|condition| {
                let operand = instruction.operand(index);
                index += 1;
                match condition {
                    AliasCondition::Ignore => true,
                    AliasCondition::InvertibleCondition => {
                        matches!(operand, MachineOperand::Immediate(held) if *held != 14 && *held != 15)
                    }
                    AliasCondition::Immediate(value) => {
                        matches!(operand, MachineOperand::Immediate(held) if *held == i64::from(*value))
                    }
                    AliasCondition::Register(register) => {
                        matches!(operand, MachineOperand::Register(held) if held == register)
                    }
                    AliasCondition::TiedRegister(other) => {
                        operand.is_register() && operand.register() == instruction.register_at(*other)
                    }
                    AliasCondition::RegisterClass(class) => match operand {
                        MachineOperand::Register(held) => in_class(self.target, class, *held),
                        _ => false,
                    },
                }
            });
            if matched {
                chosen = Some(pattern);
                break;
            }
        }
        let pattern = chosen?;
        let pieces = &pattern.pieces;
        let mut out = String::from("\t");
        let mut at = 0;
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
                        Some(method) => self.call_method(method, instruction, *index, &mut out),
                        None => self.call_method("printOperand", instruction, *index, &mut out),
                    },
                }
                at += 1;
            }
        }
        Some(out)
    }

    fn memory_barrier(&self, value: u32) -> String {
        let version8 = self.features.contains("HasV8Ops");
        let named = |name: &str, plain: bool| {
            if plain || version8 {
                name.to_string()
            } else {
                format!("#0x{value:x}")
            }
        };
        match value {
            1 => named("oshld", false),
            2 => "oshst".to_string(),
            3 => "osh".to_string(),
            5 => named("nshld", false),
            6 => "nshst".to_string(),
            7 => "nsh".to_string(),
            9 => named("ishld", false),
            10 => "ishst".to_string(),
            11 => "ish".to_string(),
            13 => named("ld", false),
            14 => "st".to_string(),
            15 => "sy".to_string(),
            other => format!("#0x{other:x}"),
        }
    }

    fn address_offset_sign(sub: bool) -> &'static str {
        if sub {
            "-"
        } else {
            ""
        }
    }

    fn call_method(
        &self,
        method: &str,
        instruction: &MachineInstruction,
        index: usize,
        out: &mut String,
    ) {
        let (name, arguments) = split_arguments(method);
        let operand = instruction.operand(index);
        let flag = |at: usize| arguments.get(at).is_some_and(|argument| argument == "true");
        let value = |at: usize| instruction.immediate_at(index + at);
        let register = |at: usize| self.register_text(instruction, index + at);
        let has_register = |at: usize| instruction.register_at(index + at) != NO_REGISTER;
        match name.as_str() {
            "" | "printOperand" => match operand {
                MachineOperand::Register(held) => out.push_str(&self.register_name(*held)),
                MachineOperand::Immediate(held) => {
                    out.push_str(&format!("#{}", self.immediate_text(*held)))
                }
                MachineOperand::Expression(expression) => match expression {
                    Expression::Binary { .. } => {
                        out.push('#');
                        out.push_str(&self.expression(expression));
                    }
                    other => out.push_str(&self.expression(other)),
                },
            },
            "printPredicateOperand" => out.push_str(&self.predicate(instruction, index)),
            "printMandatoryPredicateOperand" => {
                out.push_str(condition_name(value(0) as u32).max("al"))
            }
            "printMandatoryRestrictedPredicateOperand" => {
                if value(0) == 2 {
                    out.push_str("cs")
                } else {
                    out.push_str(condition_name(value(0) as u32).max("al"))
                }
            }
            "printMandatoryInvertedPredicateOperand" => {
                out.push_str(condition_name(value(0) as u32 ^ 1).max("al"))
            }
            "printSBitModifierOperand" => out.push_str(self.carry(instruction, index)),
            "printSORegRegOperand" => {
                out.push_str(&register(0));
                let shift = (value(2) as u32) & 7;
                out.push_str(&format!(", {}", shift_name(shift)));
                if shift != 5 {
                    out.push(' ');
                    out.push_str(&register(1));
                }
            }
            "printSORegImmOperand" | "printT2SOOperand" => {
                out.push_str(&register(0));
                let packed = value(1) as u32;
                out.push_str(&self.shift_suffix(packed & 7, packed >> 3));
            }
            "printAddrMode2Operand" | "printAddrMode3Operand" | "printAddrModeImm12Operand" => {
                if !matches!(operand, MachineOperand::Register(_)) {
                    self.call_method("printOperand", instruction, index, out);
                    return;
                }
                out.push('[');
                out.push_str(&register(0));
                match name.as_str() {
                    "printAddrMode2Operand" => {
                        let packed = value(2) as u32;
                        let sub = (packed >> 12) & 1 == 1;
                        let amount = packed & 0xfff;
                        if !has_register(1) {
                            if amount != 0 {
                                out.push_str(&format!(
                                    ", #{}{}",
                                    Self::address_offset_sign(sub),
                                    amount
                                ));
                            }
                        } else {
                            out.push_str(", ");
                            out.push_str(Self::address_offset_sign(sub));
                            out.push_str(&register(1));
                            out.push_str(&self.shift_suffix((packed >> 13) & 7, amount));
                        }
                    }
                    "printAddrMode3Operand" => {
                        let packed = value(2) as u32;
                        let sub = (packed >> 8) & 1 == 1;
                        if has_register(1) {
                            out.push_str(&format!(
                                ", {}{}",
                                Self::address_offset_sign(sub),
                                register(1)
                            ));
                        } else {
                            let amount = packed & 0xff;
                            if flag(0) || amount != 0 || sub {
                                out.push_str(&format!(
                                    ", #{}{}",
                                    Self::address_offset_sign(sub),
                                    amount
                                ));
                            }
                        }
                    }
                    _ => {
                        let offset = value(1) as i32;
                        if offset == i32::MIN {
                            if flag(0) {
                                out.push_str(", #0");
                            }
                        } else if offset < 0 {
                            out.push_str(&format!(
                                ", #-{}",
                                self.immediate_text(-i64::from(offset))
                            ));
                        } else if flag(0) || offset > 0 {
                            out.push_str(&format!(", #{}", self.immediate_text(i64::from(offset))));
                        }
                    }
                }
                out.push(']');
            }
            "printAddrMode2OffsetOperand" => {
                let packed = value(1) as u32;
                let sub = (packed >> 12) & 1 == 1;
                let amount = packed & 0xfff;
                if !has_register(0) {
                    out.push_str(&format!("#{}{}", Self::address_offset_sign(sub), amount));
                } else {
                    out.push_str(Self::address_offset_sign(sub));
                    out.push_str(&register(0));
                    out.push_str(&self.shift_suffix((packed >> 13) & 7, amount));
                }
            }
            "printAddrMode3OffsetOperand" => {
                let packed = value(1) as u32;
                let sub = (packed >> 8) & 1 == 1;
                if has_register(0) {
                    out.push_str(Self::address_offset_sign(sub));
                    out.push_str(&register(0));
                } else {
                    out.push_str(&format!(
                        "#{}{}",
                        Self::address_offset_sign(sub),
                        packed & 0xff
                    ));
                }
            }
            "printPostIdxImm8Operand" => {
                let packed = value(0) as u32;
                out.push_str(&format!(
                    "#{}{}",
                    if packed & 256 != 0 { "" } else { "-" },
                    packed & 0xff
                ));
            }
            "printPostIdxImm8s4Operand" => {
                let packed = value(0) as u32;
                out.push_str(&format!(
                    "#{}{}",
                    if packed & 256 != 0 { "" } else { "-" },
                    (packed & 0xff) << 2
                ));
            }
            "printPostIdxRegOperand" => {
                out.push_str(if value(1) != 0 { "" } else { "-" });
                out.push_str(&register(0));
            }
            "printLdStmModeOperand" => out.push_str(match value(0) {
                1 => "ia",
                2 => "ib",
                3 => "da",
                4 => "db",
                _ => "",
            }),
            "printAddrMode5Operand" | "printAddrMode5FP16Operand" => {
                if !matches!(operand, MachineOperand::Register(_)) {
                    self.call_method("printOperand", instruction, index, out);
                    return;
                }
                out.push('[');
                out.push_str(&register(0));
                let packed = value(1) as u32;
                let amount = packed & 0xff;
                let sub = (packed >> 8) & 1 == 1;
                let scale = if name == "printAddrMode5Operand" {
                    4
                } else {
                    2
                };
                if flag(0) || amount != 0 || sub {
                    out.push_str(&format!(
                        ", #{}{}",
                        Self::address_offset_sign(sub),
                        amount * scale
                    ));
                }
                out.push(']');
            }
            "printAddrMode6Operand" => {
                out.push('[');
                out.push_str(&register(0));
                if value(1) != 0 {
                    out.push_str(&format!(":{}", value(1) << 3));
                }
                out.push(']');
            }
            "printAddrMode7Operand" => out.push_str(&format!("[{}]", register(0))),
            "printAddrMode6OffsetOperand" => {
                if !has_register(0) {
                    out.push('!');
                } else {
                    out.push_str(&format!(", {}", register(0)));
                }
            }
            "printAddrModeTBB" => out.push_str(&format!("[{}, {}]", register(0), register(1))),
            "printAddrModeTBH" => {
                out.push_str(&format!("[{}, {}, lsl #1]", register(0), register(1)))
            }
            "printBitfieldInvMaskImmOperand" => {
                let inverted = !(value(0) as u32);
                let lsb = inverted.trailing_zeros();
                let width = (32 - inverted.leading_zeros()) - lsb;
                out.push_str(&format!("#{lsb}, #{width}"));
            }
            "printMemBOption" => out.push_str(&self.memory_barrier(value(0) as u32)),
            "printInstSyncBOption" => {
                let held = value(0);
                if held == 15 {
                    out.push_str("sy")
                } else {
                    out.push_str(&format!("#0x{held:x}"))
                }
            }
            "printTraceSyncBOption" => out.push_str("csync"),
            "printShiftImmOperand" => {
                let packed = value(0) as u32;
                let amount = packed & 0x1f;
                if packed & 32 != 0 {
                    out.push_str(&format!(", asr #{}", translate_shift(amount)));
                } else if amount != 0 {
                    out.push_str(&format!(", lsl #{amount}"));
                }
            }
            "printPKHLSLShiftImm" => {
                if value(0) != 0 {
                    out.push_str(&format!(", lsl #{}", value(0)));
                }
            }
            "printPKHASRShiftImm" => {
                let held = if value(0) == 0 { 32 } else { value(0) };
                out.push_str(&format!(", asr #{held}"));
            }
            "printRegisterList" => out.push_str(&self.register_list(instruction, index)),
            "printGPRPairOperand" => {
                if let MachineOperand::Register(held) = operand {
                    let members = &self.target.registers[*held as usize].sub_registers;
                    let names: Vec<String> = members
                        .iter()
                        .map(|member| self.register_name(*member))
                        .collect();
                    out.push_str(&names.join(", "));
                }
            }
            "printSetendOperand" => out.push_str(if value(0) != 0 { "be" } else { "le" }),
            "printCPSIMod" => out.push_str(match value(0) {
                2 => "ie",
                3 => "id",
                _ => "",
            }),
            "printCPSIFlag" => {
                let flags = value(0) as u32;
                for bit in (0..3).rev() {
                    if flags & (1 << bit) != 0 {
                        out.push_str(["f", "i", "a"][bit]);
                    }
                }
                if flags == 0 {
                    out.push_str("none");
                }
            }
            "printMSRMaskOperand" => {
                let held = value(0) as u32;
                let spsr = held >> 4;
                let mask = held & 0xf;
                if spsr == 0 && matches!(mask, 8 | 4 | 12) {
                    out.push_str("APSR_");
                    out.push_str(match mask {
                        4 => "g",
                        8 => "nzcvq",
                        _ => "nzcvqg",
                    });
                    return;
                }
                out.push_str(if spsr != 0 { "SPSR" } else { "CPSR" });
                if mask != 0 {
                    out.push('_');
                    for (bit, letter) in [(8, 'f'), (4, 's'), (2, 'x'), (1, 'c')] {
                        if mask & bit != 0 {
                            out.push(letter);
                        }
                    }
                }
            }
            "printNoHashImmediate" => out.push_str(&value(0).to_string()),
            "printPImmediate" => out.push_str(&format!("p{}", value(0))),
            "printCImmediate" => out.push_str(&format!("c{}", value(0))),
            "printCoprocOptionImm" => out.push_str(&format!("{{{}}}", value(0))),
            "printAdrLabelOperand" => match operand {
                MachineOperand::Expression(expression) => {
                    out.push_str(&self.expression(expression))
                }
                _ => {
                    let scale = arguments
                        .first()
                        .and_then(|argument| argument.parse::<u32>().ok())
                        .unwrap_or(0);
                    let offset = (value(0) as i32).wrapping_shl(scale);
                    if offset == i32::MIN {
                        out.push_str("#-0");
                    } else if offset < 0 {
                        out.push_str(&format!("#-{}", -i64::from(offset)));
                    } else {
                        out.push_str(&format!("#{offset}"));
                    }
                }
            },
            "printFPImmOperand" => {
                let held = armv8::float_from_immediate(value(0) as u32);
                out.push_str(&format!("#{}", floating_text(held)));
            }
            "printImmPlusOneOperand" => out.push_str(&format!("#{}", value(0) + 1)),
            "printRotImmOperand" => {
                if value(0) != 0 {
                    out.push_str(&format!(", ror #{}", 8 * value(0)));
                }
            }
            "printModImmOperand" => {
                if let MachineOperand::Expression(expression) = operand {
                    out.push_str(&self.expression(expression));
                    return;
                }
                let held = value(0) as u32;
                let bits = held & 0xff;
                let rotation = (held & 0xf00) >> 7;
                let record = self.target.instructions[instruction.opcode].record.as_str();
                let unsigned = match record {
                    "MOVi" => {
                        index > 0
                            && self.target.register_by_name.get("pc").copied()
                                == Some(instruction.register_at(index - 1))
                    }
                    "MSRi" => true,
                    _ => false,
                };
                let rotated = bits.rotate_right(rotation);
                if so_immediate(rotated) == Some(held) {
                    if unsigned {
                        out.push_str(&format!("#{rotated}"));
                    } else {
                        out.push_str(&format!("#{}", rotated as i32));
                    }
                } else {
                    out.push_str(&format!("#{bits}, #{rotation}"));
                }
            }
            "printFBits16" => out.push_str(&format!("#{}", 16 - value(0))),
            "printFBits32" => out.push_str(&format!("#{}", 32 - value(0))),
            "printVectorIndex" => out.push_str(&format!("[{}]", value(0))),
            "printThumbITMask" => {
                let mask = value(0) as u32;
                let trailing = mask.trailing_zeros();
                for position in (trailing + 1..=3).rev() {
                    out.push(if (mask >> position) & 1 == 1 {
                        'e'
                    } else {
                        't'
                    });
                }
            }
            "printThumbS4ImmOperand" => out.push_str(&format!("#{}", value(0) * 4)),
            "printThumbSRImm" => {
                out.push_str(&format!("#{}", if value(0) == 0 { 32 } else { value(0) }))
            }
            "printThumbAddrModeRROperand" => {
                if !matches!(operand, MachineOperand::Register(_)) {
                    self.call_method("printOperand", instruction, index, out);
                    return;
                }
                out.push('[');
                out.push_str(&register(0));
                if has_register(1) {
                    out.push_str(&format!(", {}", register(1)));
                }
                out.push(']');
            }
            "printThumbAddrModeImm5S1Operand"
            | "printThumbAddrModeImm5S2Operand"
            | "printThumbAddrModeImm5S4Operand"
            | "printThumbAddrModeSPOperand" => {
                if !matches!(operand, MachineOperand::Register(_)) {
                    self.call_method("printOperand", instruction, index, out);
                    return;
                }
                let scale = match name.as_str() {
                    "printThumbAddrModeImm5S1Operand" => 1,
                    "printThumbAddrModeImm5S2Operand" => 2,
                    _ => 4,
                };
                out.push('[');
                out.push_str(&register(0));
                if value(1) != 0 {
                    out.push_str(&format!(", #{}", value(1) * scale));
                }
                out.push(']');
            }
            "printThumbLdrLabelOperand" => match operand {
                MachineOperand::Expression(expression) => {
                    out.push_str(&self.expression(expression))
                }
                _ => {
                    let offset = value(0) as i32;
                    if offset < 0 && offset != i32::MIN {
                        out.push_str(&format!("[pc, #-{}]", -i64::from(offset)));
                    } else {
                        out.push_str(&format!(
                            "[pc, #{}]",
                            if offset == i32::MIN { 0 } else { offset }
                        ));
                    }
                }
            },
            "printT2AddrModeImm8Operand" | "printT2AddrModeImm8s4Operand" => {
                if !matches!(operand, MachineOperand::Register(_)) {
                    self.call_method("printOperand", instruction, index, out);
                    return;
                }
                out.push('[');
                out.push_str(&register(0));
                let offset = value(1) as i32;
                if offset == i32::MIN {
                    if flag(0) {
                        out.push_str(", #0");
                    }
                } else if offset < 0 {
                    out.push_str(&format!(", #-{}", -i64::from(offset)));
                } else if flag(0) || offset > 0 {
                    out.push_str(&format!(", #{offset}"));
                }
                out.push(']');
            }
            "printT2AddrModeImm8OffsetOperand" | "printT2AddrModeImm8s4OffsetOperand" => {
                let offset = value(0) as i32;
                if offset == i32::MIN {
                    out.push_str(", #-0");
                } else if offset < 0 {
                    out.push_str(&format!(", #-{}", -i64::from(offset)));
                } else {
                    out.push_str(&format!(", #{offset}"));
                }
            }
            "printT2AddrModeImm0_1020s4Operand" => {
                out.push('[');
                out.push_str(&register(0));
                if value(1) != 0 {
                    out.push_str(&format!(", #{}", value(1) * 4));
                }
                out.push(']');
            }
            "printT2AddrModeSoRegOperand" => {
                out.push_str(&format!("[{}, {}", register(0), register(1)));
                if value(2) != 0 {
                    out.push_str(&format!(", lsl #{}", value(2)));
                }
                out.push(']');
            }
            "printVectorListOne" => out.push_str(&format!("{{{}}}", register(0))),
            "printVectorListOneAllLanes" => out.push_str(&format!("{{{}[]}}", register(0))),
            "printVectorListTwo"
            | "printVectorListTwoSpaced"
            | "printVectorListTwoAllLanes"
            | "printVectorListTwoSpacedAllLanes" => {
                let spaced = name.contains("Spaced");
                let lanes = if name.contains("AllLanes") { "[]" } else { "" };
                let held = self.vector_members(instruction.register_at(index), 2, spaced);
                out.push_str(&format!("{{{}{lanes}, {}{lanes}}}", held[0], held[1]));
            }
            "printVectorListThree"
            | "printVectorListFour"
            | "printVectorListThreeSpaced"
            | "printVectorListFourSpaced"
            | "printVectorListThreeAllLanes"
            | "printVectorListFourAllLanes"
            | "printVectorListThreeSpacedAllLanes"
            | "printVectorListFourSpacedAllLanes" => {
                let spaced = name.contains("Spaced");
                let lanes = if name.contains("AllLanes") { "[]" } else { "" };
                let count = if name.contains("Three") { 3 } else { 4 };
                let first = instruction.register_at(index);
                let step = if spaced { 2 } else { 1 };
                let encoding = u32::from(self.target.registers[first as usize].encoding);
                let names: Vec<String> = (0..count)
                    .map(|at| format!("d{}{lanes}", encoding + at * step))
                    .collect();
                out.push_str(&format!("{{{}}}", names.join(", ")));
            }
            "printVPTPredicateOperand" => out.push_str(match value(0) {
                1 => "t",
                2 => "e",
                _ => "",
            }),
            "printComplexRotationOp" => {
                let angle = arguments
                    .first()
                    .and_then(|argument| argument.parse::<i64>().ok())
                    .unwrap_or(0);
                let remainder = arguments
                    .get(1)
                    .and_then(|argument| argument.parse::<i64>().ok())
                    .unwrap_or(0);
                out.push_str(&format!("#{}", value(0) * angle + remainder));
            }
            _ => {
                out.push_str(&format!("<{name}>"));
            }
        }
    }

    fn vector_members(&self, register: RegisterId, count: usize, spaced: bool) -> Vec<String> {
        let held = &self.target.registers[register as usize];
        if held.sub_registers.len() >= count && !spaced {
            let members: Vec<String> = held
                .sub_registers
                .iter()
                .take(count)
                .map(|member| self.register_name(*member))
                .collect();
            if members.len() == count {
                return members;
            }
        }
        let encoding = u32::from(held.encoding);
        let step = if spaced { 2 } else { 1 };
        (0..count as u32)
            .map(|at| format!("d{}", encoding + at * step))
            .collect()
    }
}
