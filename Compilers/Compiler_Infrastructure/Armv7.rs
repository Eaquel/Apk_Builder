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
pub const MVE_MODE: u32 = 1 << 29;

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
    BfBranch,
    BfTarget,
    BflTarget,
    BfcTarget,
    BfcselElse,
    Wls,
    Le,
}

impl Fixup32 {
    pub fn is_half_word_relocation(self) -> bool {
        matches!(
            self,
            Fixup32::MovwLo16
                | Fixup32::MovtHi16
                | Fixup32::T2MovwLo16
                | Fixup32::T2MovtHi16
                | Fixup32::ThumbUpper815
                | Fixup32::ThumbUpper07
                | Fixup32::ThumbLower815
                | Fixup32::ThumbLower07
        )
    }

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
            Fixup32::BfBranch => "fixup_bf_branch",
            Fixup32::BfTarget => "fixup_bf_target",
            Fixup32::BflTarget => "fixup_bfl_target",
            Fixup32::BfcTarget => "fixup_bfc_target",
            Fixup32::BfcselElse => "fixup_bfcsel_else_target",
            Fixup32::Wls => "fixup_wls",
            Fixup32::Le => "fixup_le",
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

    pub fn aligned_down(self) -> bool {
        matches!(
            self,
            Fixup32::LdstPcrel12
                | Fixup32::T2LdstPcrel12
                | Fixup32::PcrelTenUnscaled
                | Fixup32::PcrelTen
                | Fixup32::T2PcrelTen
                | Fixup32::PcrelNine
                | Fixup32::T2PcrelNine
                | Fixup32::ThumbAdrPcrel10
                | Fixup32::AdrPcrel12
                | Fixup32::T2AdrPcrel12
                | Fixup32::ThumbBlx
                | Fixup32::ThumbCp
        )
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
                | Fixup32::BfcselElse
        )
    }
}

fn swap_half_words(value: u32) -> u32 {
    value.rotate_right(16)
}

pub fn long_call_features(features: &BTreeSet<String>) -> bool {
    features.contains("FeatureThumb2")
        || features.contains("HasV8MBaselineOps")
        || features.contains("HasV6MOps")
}

pub fn adjust_fixup_value(
    kind: Fixup32,
    value: u64,
    resolved: bool,
    thumb_two: bool,
    long_call: bool,
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
            if !signed(held as u64, 25) || (!long_call && !signed(held as u64, 23)) {
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
        Fixup32::BfBranch => {
            let offset = value as i64 - 4;
            if !(2..=30).contains(&offset) {
                return Err(range_error);
            }
            swap_half_words((((value - 4) >> 1) as u32 & 0xf) << 23)
        }
        Fixup32::BfTarget | Fixup32::BflTarget | Fixup32::BfcTarget => {
            let offset = value as i64 - 4;
            let (low, high, mask) = match kind {
                Fixup32::BfTarget => (-0x10000, 0xfffe, 0xf800u32),
                Fixup32::BflTarget => (-0x40000, 0x3fffe, 0x3f800),
                _ => (-0x1000, 0xffe, 0x800),
            };
            if offset < low || offset > high {
                return Err(range_error);
            }
            let shifted = (value.wrapping_sub(4) as i64 >> 1) as u32;
            let mut out = (shifted & 1) << 11;
            out |= shifted & 0x7fe;
            out |= (shifted & mask) << 5;
            swap_half_words(out)
        }
        Fixup32::BfcselElse => {
            if value != 2 && value != 4 {
                return Err("out of range label-relative fixup value");
            }
            swap_half_words(((value as u32 >> 2) & 1) << 17)
        }
        Fixup32::Wls | Fixup32::Le => {
            let offset = value as i64 - 4;
            let (low, high) = if kind == Fixup32::Wls {
                (0, 0xffe)
            } else {
                (-0xffe, 0)
            };
            if offset < low || offset > high {
                return Err(range_error);
            }
            let mut real = value.wrapping_sub(4) as i64;
            if kind == Fixup32::Le {
                real = -real;
            }
            let real = real as u64;
            let mut out = (((real >> 1) & 1) as u32) << 11;
            out |= ((real >> 1) & 0x7fe) as u32;
            swap_half_words(out)
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
    let specifier = specifier
        .split_once(':')
        .map_or(specifier.clone(), |(_, inner)| inner.to_string());
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
            Some(Fixup32::BfTarget) => 0x88,
            Some(Fixup32::BfcTarget) => 0x89,
            Some(Fixup32::BflTarget) => 0x8a,
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
            (4, "none") => 0,
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
            (8, _) => return Err("unsupported relocation type"),
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
    TraceSyncBarrier,
    ItMask(u32),
    VectorList {
        register: RegisterId,
        count: u32,
        spaced: bool,
        lane: VectorLane,
    },
    VectorIndex(u32),
    CoprocessorNumber(u32),
    CoprocessorRegister(u32),
    CoprocessorOption(u32),
    SystemMask(u32),
    BankedRegister(u32),
    ProcessorFlags(u32),
    VectorPredicate(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VectorLane {
    Whole,
    All,
    Index(u32),
}

impl Operand {
    fn constant(&self) -> Option<i64> {
        match self {
            Operand::Immediate(Expression::Constant(value)) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ItState {
    pub condition: u32,
    pub mask: u32,
    pub position: u32,
    pub explicit: bool,
}

impl ItState {
    pub const NONE: ItState = ItState {
        condition: 0,
        mask: 0,
        position: u32::MAX,
        explicit: false,
    };

    pub fn active(&self) -> bool {
        self.position != u32::MAX
    }

    pub fn explicit_block(&self) -> bool {
        self.active() && self.explicit
    }

    pub fn last(&self) -> bool {
        self.position == 4 - self.mask.trailing_zeros().min(4)
    }

    pub fn forward(&mut self) {
        if !self.active() {
            return;
        }
        let zeros = self.mask.trailing_zeros().min(4);
        self.position += 1;
        if self.position == 5 - zeros && self.explicit {
            self.position = u32::MAX;
        }
    }

    pub fn forward_vector(&mut self) {
        if !self.active() {
            return;
        }
        let zeros = self.mask.trailing_zeros().min(4);
        self.position += 1;
        if self.position == 5 - zeros {
            self.position = u32::MAX;
        }
    }

    pub fn current(&self) -> u32 {
        if (self.mask >> (5 - self.position.min(5))) & 1 == 1 {
            self.condition ^ 1
        } else {
            self.condition
        }
    }
}

const CONDITION_NAMES: [&str; 15] = [
    "eq", "ne", "hs", "lo", "mi", "pl", "vs", "vc", "hi", "ls", "ge", "lt", "gt", "le", "al",
];

#[derive(Clone, Copy)]
enum Pseudo {
    Operand(usize),
    None,
    Step(i64),
}

const VECTOR_PSEUDOS: &[(&str, &str, i64, &[Pseudo])] = &[
    (
        "VST1LNdWB_register_Asm_8",
        "VST1LNd8_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST1LNdWB_register_Asm_16",
        "VST1LNd16_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST1LNdWB_register_Asm_32",
        "VST1LNd32_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST2LNdWB_register_Asm_8",
        "VST2LNd8_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST2LNdWB_register_Asm_16",
        "VST2LNd16_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST2LNdWB_register_Asm_32",
        "VST2LNd32_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST2LNqWB_register_Asm_16",
        "VST2LNq16_UPD",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST2LNqWB_register_Asm_32",
        "VST2LNq32_UPD",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST3LNdWB_register_Asm_8",
        "VST3LNd8_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST3LNdWB_register_Asm_16",
        "VST3LNd16_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST3LNdWB_register_Asm_32",
        "VST3LNd32_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST3LNqWB_register_Asm_16",
        "VST3LNq16_UPD",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST3LNqWB_register_Asm_32",
        "VST3LNq32_UPD",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST4LNdWB_register_Asm_8",
        "VST4LNd8_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST4LNdWB_register_Asm_16",
        "VST4LNd16_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST4LNdWB_register_Asm_32",
        "VST4LNd32_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST4LNqWB_register_Asm_16",
        "VST4LNq16_UPD",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST4LNqWB_register_Asm_32",
        "VST4LNq32_UPD",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VST1LNdWB_fixed_Asm_8",
        "VST1LNd8_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST1LNdWB_fixed_Asm_16",
        "VST1LNd16_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST1LNdWB_fixed_Asm_32",
        "VST1LNd32_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST2LNdWB_fixed_Asm_8",
        "VST2LNd8_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST2LNdWB_fixed_Asm_16",
        "VST2LNd16_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST2LNdWB_fixed_Asm_32",
        "VST2LNd32_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST2LNqWB_fixed_Asm_16",
        "VST2LNq16_UPD",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST2LNqWB_fixed_Asm_32",
        "VST2LNq32_UPD",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3LNdWB_fixed_Asm_8",
        "VST3LNd8_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3LNdWB_fixed_Asm_16",
        "VST3LNd16_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3LNdWB_fixed_Asm_32",
        "VST3LNd32_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3LNqWB_fixed_Asm_16",
        "VST3LNq16_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3LNqWB_fixed_Asm_32",
        "VST3LNq32_UPD",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4LNdWB_fixed_Asm_8",
        "VST4LNd8_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4LNdWB_fixed_Asm_16",
        "VST4LNd16_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4LNdWB_fixed_Asm_32",
        "VST4LNd32_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4LNqWB_fixed_Asm_16",
        "VST4LNq16_UPD",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4LNqWB_fixed_Asm_32",
        "VST4LNq32_UPD",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST1LNdAsm_8",
        "VST1LNd8",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST1LNdAsm_16",
        "VST1LNd16",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST1LNdAsm_32",
        "VST1LNd32",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST2LNdAsm_8",
        "VST2LNd8",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST2LNdAsm_16",
        "VST2LNd16",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST2LNdAsm_32",
        "VST2LNd32",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST2LNqAsm_16",
        "VST2LNq16",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST2LNqAsm_32",
        "VST2LNq32",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3LNdAsm_8",
        "VST3LNd8",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3LNdAsm_16",
        "VST3LNd16",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3LNdAsm_32",
        "VST3LNd32",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3LNqAsm_16",
        "VST3LNq16",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3LNqAsm_32",
        "VST3LNq32",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4LNdAsm_8",
        "VST4LNd8",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4LNdAsm_16",
        "VST4LNd16",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4LNdAsm_32",
        "VST4LNd32",
        1,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4LNqAsm_16",
        "VST4LNq16",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4LNqAsm_32",
        "VST4LNq32",
        2,
        &[
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD1LNdWB_register_Asm_8",
        "VLD1LNd8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD1LNdWB_register_Asm_16",
        "VLD1LNd16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD1LNdWB_register_Asm_32",
        "VLD1LNd32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD2LNdWB_register_Asm_8",
        "VLD2LNd8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD2LNdWB_register_Asm_16",
        "VLD2LNd16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD2LNdWB_register_Asm_32",
        "VLD2LNd32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD2LNqWB_register_Asm_16",
        "VLD2LNq16_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD2LNqWB_register_Asm_32",
        "VLD2LNq32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD3LNdWB_register_Asm_8",
        "VLD3LNd8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD3LNdWB_register_Asm_16",
        "VLD3LNd16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD3LNdWB_register_Asm_32",
        "VLD3LNd32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD3LNqWB_register_Asm_16",
        "VLD3LNq16_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD3LNqWB_register_Asm_32",
        "VLD3LNq32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD4LNdWB_register_Asm_8",
        "VLD4LNd8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD4LNdWB_register_Asm_16",
        "VLD4LNd16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD4LNdWB_register_Asm_32",
        "VLD4LNd32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD4LNqWB_register_Asm_16",
        "VLD4LNq16_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD4LNqWB_register_Asm_32",
        "VLD4LNq32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(5),
            Pseudo::Operand(6),
        ],
    ),
    (
        "VLD1LNdWB_fixed_Asm_8",
        "VLD1LNd8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD1LNdWB_fixed_Asm_16",
        "VLD1LNd16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD1LNdWB_fixed_Asm_32",
        "VLD1LNd32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD2LNdWB_fixed_Asm_8",
        "VLD2LNd8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD2LNdWB_fixed_Asm_16",
        "VLD2LNd16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD2LNdWB_fixed_Asm_32",
        "VLD2LNd32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD2LNqWB_fixed_Asm_16",
        "VLD2LNq16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD2LNqWB_fixed_Asm_32",
        "VLD2LNq32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3LNdWB_fixed_Asm_8",
        "VLD3LNd8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3LNdWB_fixed_Asm_16",
        "VLD3LNd16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3LNdWB_fixed_Asm_32",
        "VLD3LNd32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3LNqWB_fixed_Asm_16",
        "VLD3LNq16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3LNqWB_fixed_Asm_32",
        "VLD3LNq32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4LNdWB_fixed_Asm_8",
        "VLD4LNd8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4LNdWB_fixed_Asm_16",
        "VLD4LNd16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4LNdWB_fixed_Asm_32",
        "VLD4LNd32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4LNqWB_fixed_Asm_16",
        "VLD4LNq16_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4LNqWB_fixed_Asm_32",
        "VLD4LNq32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD1LNdAsm_8",
        "VLD1LNd8",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD1LNdAsm_16",
        "VLD1LNd16",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD1LNdAsm_32",
        "VLD1LNd32",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD2LNdAsm_8",
        "VLD2LNd8",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD2LNdAsm_16",
        "VLD2LNd16",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD2LNdAsm_32",
        "VLD2LNd32",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD2LNqAsm_16",
        "VLD2LNq16",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD2LNqAsm_32",
        "VLD2LNq32",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3LNdAsm_8",
        "VLD3LNd8",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3LNdAsm_16",
        "VLD3LNd16",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3LNdAsm_32",
        "VLD3LNd32",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3LNqAsm_16",
        "VLD3LNq16",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3LNqAsm_32",
        "VLD3LNq32",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4LNdAsm_8",
        "VLD4LNd8",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4LNdAsm_16",
        "VLD4LNd16",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4LNdAsm_32",
        "VLD4LNd32",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4LNqAsm_16",
        "VLD4LNq16",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4LNqAsm_32",
        "VLD4LNq32",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3DUPdAsm_8",
        "VLD3DUPd8",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3DUPdAsm_16",
        "VLD3DUPd16",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3DUPdAsm_32",
        "VLD3DUPd32",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3DUPqAsm_8",
        "VLD3DUPq8",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3DUPqAsm_16",
        "VLD3DUPq16",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3DUPqAsm_32",
        "VLD3DUPq32",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3DUPdWB_fixed_Asm_8",
        "VLD3DUPd8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3DUPdWB_fixed_Asm_16",
        "VLD3DUPd16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3DUPdWB_fixed_Asm_32",
        "VLD3DUPd32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3DUPqWB_fixed_Asm_8",
        "VLD3DUPq8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3DUPqWB_fixed_Asm_16",
        "VLD3DUPq16_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3DUPqWB_fixed_Asm_32",
        "VLD3DUPq32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3DUPdWB_register_Asm_8",
        "VLD3DUPd8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3DUPdWB_register_Asm_16",
        "VLD3DUPd16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3DUPdWB_register_Asm_32",
        "VLD3DUPd32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3DUPqWB_register_Asm_8",
        "VLD3DUPq8_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3DUPqWB_register_Asm_16",
        "VLD3DUPq16_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3DUPqWB_register_Asm_32",
        "VLD3DUPq32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3dAsm_8",
        "VLD3d8",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3dAsm_16",
        "VLD3d16",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3dAsm_32",
        "VLD3d32",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3qAsm_8",
        "VLD3q8",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3qAsm_16",
        "VLD3q16",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3qAsm_32",
        "VLD3q32",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3dWB_fixed_Asm_8",
        "VLD3d8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3dWB_fixed_Asm_16",
        "VLD3d16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3dWB_fixed_Asm_32",
        "VLD3d32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3qWB_fixed_Asm_8",
        "VLD3q8_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3qWB_fixed_Asm_16",
        "VLD3q16_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3qWB_fixed_Asm_32",
        "VLD3q32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD3dWB_register_Asm_8",
        "VLD3d8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3dWB_register_Asm_16",
        "VLD3d16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3dWB_register_Asm_32",
        "VLD3d32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3qWB_register_Asm_8",
        "VLD3q8_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3qWB_register_Asm_16",
        "VLD3q16_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD3qWB_register_Asm_32",
        "VLD3q32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4DUPdAsm_8",
        "VLD4DUPd8",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4DUPdAsm_16",
        "VLD4DUPd16",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4DUPdAsm_32",
        "VLD4DUPd32",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4DUPqAsm_8",
        "VLD4DUPq8",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4DUPqAsm_16",
        "VLD4DUPq16",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4DUPqAsm_32",
        "VLD4DUPq32",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4DUPdWB_fixed_Asm_8",
        "VLD4DUPd8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4DUPdWB_fixed_Asm_16",
        "VLD4DUPd16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4DUPdWB_fixed_Asm_32",
        "VLD4DUPd32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4DUPqWB_fixed_Asm_8",
        "VLD4DUPq8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4DUPqWB_fixed_Asm_16",
        "VLD4DUPq16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4DUPqWB_fixed_Asm_32",
        "VLD4DUPq32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4DUPdWB_register_Asm_8",
        "VLD4DUPd8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4DUPdWB_register_Asm_16",
        "VLD4DUPd16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4DUPdWB_register_Asm_32",
        "VLD4DUPd32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4DUPqWB_register_Asm_8",
        "VLD4DUPq8_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4DUPqWB_register_Asm_16",
        "VLD4DUPq16_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4DUPqWB_register_Asm_32",
        "VLD4DUPq32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4dAsm_8",
        "VLD4d8",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4dAsm_16",
        "VLD4d16",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4dAsm_32",
        "VLD4d32",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4qAsm_8",
        "VLD4q8",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4qAsm_16",
        "VLD4q16",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4qAsm_32",
        "VLD4q32",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4dWB_fixed_Asm_8",
        "VLD4d8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4dWB_fixed_Asm_16",
        "VLD4d16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4dWB_fixed_Asm_32",
        "VLD4d32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4qWB_fixed_Asm_8",
        "VLD4q8_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4qWB_fixed_Asm_16",
        "VLD4q16_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4qWB_fixed_Asm_32",
        "VLD4q32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VLD4dWB_register_Asm_8",
        "VLD4d8_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4dWB_register_Asm_16",
        "VLD4d16_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4dWB_register_Asm_32",
        "VLD4d32_UPD",
        1,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4qWB_register_Asm_8",
        "VLD4q8_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4qWB_register_Asm_16",
        "VLD4q16_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VLD4qWB_register_Asm_32",
        "VLD4q32_UPD",
        2,
        &[
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3dAsm_8",
        "VST3d8",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST3dAsm_16",
        "VST3d16",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST3dAsm_32",
        "VST3d32",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST3qAsm_8",
        "VST3q8",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST3qAsm_16",
        "VST3q16",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST3qAsm_32",
        "VST3q32",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST3dWB_fixed_Asm_8",
        "VST3d8_UPD",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST3dWB_fixed_Asm_16",
        "VST3d16_UPD",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST3dWB_fixed_Asm_32",
        "VST3d32_UPD",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST3qWB_fixed_Asm_8",
        "VST3q8_UPD",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST3qWB_fixed_Asm_16",
        "VST3q16_UPD",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST3qWB_fixed_Asm_32",
        "VST3q32_UPD",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST3dWB_register_Asm_8",
        "VST3d8_UPD",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3dWB_register_Asm_16",
        "VST3d16_UPD",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3dWB_register_Asm_32",
        "VST3d32_UPD",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3qWB_register_Asm_8",
        "VST3q8_UPD",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3qWB_register_Asm_16",
        "VST3q16_UPD",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST3qWB_register_Asm_32",
        "VST3q32_UPD",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4dAsm_8",
        "VST4d8",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST4dAsm_16",
        "VST4d16",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST4dAsm_32",
        "VST4d32",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST4qAsm_8",
        "VST4q8",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST4qAsm_16",
        "VST4q16",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST4qAsm_32",
        "VST4q32",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST4dWB_fixed_Asm_8",
        "VST4d8_UPD",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST4dWB_fixed_Asm_16",
        "VST4d16_UPD",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST4dWB_fixed_Asm_32",
        "VST4d32_UPD",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST4qWB_fixed_Asm_8",
        "VST4q8_UPD",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST4qWB_fixed_Asm_16",
        "VST4q16_UPD",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST4qWB_fixed_Asm_32",
        "VST4q32_UPD",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::None,
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(3),
            Pseudo::Operand(4),
        ],
    ),
    (
        "VST4dWB_register_Asm_8",
        "VST4d8_UPD",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4dWB_register_Asm_16",
        "VST4d16_UPD",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4dWB_register_Asm_32",
        "VST4d32_UPD",
        1,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4qWB_register_Asm_8",
        "VST4q8_UPD",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4qWB_register_Asm_16",
        "VST4q16_UPD",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
    (
        "VST4qWB_register_Asm_32",
        "VST4q32_UPD",
        2,
        &[
            Pseudo::Operand(1),
            Pseudo::Operand(1),
            Pseudo::Operand(2),
            Pseudo::Operand(3),
            Pseudo::Operand(0),
            Pseudo::Step(1),
            Pseudo::Step(2),
            Pseudo::Step(3),
            Pseudo::Operand(4),
            Pseudo::Operand(5),
        ],
    ),
];

struct SystemRegister {
    name: String,
    m1_encoding: u32,
    m2m3_encoding: u32,
    encoding: u32,
    required: Vec<String>,
}

impl SystemRegister {
    fn available(&self, features: &BTreeSet<String>) -> bool {
        self.required.iter().all(|name| features.contains(name))
    }
}

struct BankedRegister {
    name: String,
    encoding: u32,
}

fn system_registers() -> &'static [SystemRegister] {
    static CELL: OnceLock<Vec<SystemRegister>> = OnceLock::new();
    CELL.get_or_init(|| {
        let target = target().expect("the ARMv7 tables load");
        target
            .records
            .values()
            .filter(|record| record.is_a("MClassSysReg"))
            .map(|record| {
                let required = match record.field("Requires") {
                    Some(TableValue::Code(text)) => text
                        .trim()
                        .trim_start_matches('{')
                        .trim_end_matches('}')
                        .split(',')
                        .filter_map(|item| {
                            let item = item.trim();
                            (!item.is_empty())
                                .then(|| item.rsplit("::").next().unwrap_or(item).to_string())
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                SystemRegister {
                    name: table_text(record, "Name"),
                    m1_encoding: table_integer(record, "M1Encoding12") as u32,
                    m2m3_encoding: table_integer(record, "M2M3Encoding8") as u32,
                    encoding: table_integer(record, "Encoding") as u32,
                    required,
                }
            })
            .collect()
    })
}

fn banked_registers() -> &'static [BankedRegister] {
    static CELL: OnceLock<Vec<BankedRegister>> = OnceLock::new();
    CELL.get_or_init(|| {
        let target = target().expect("the ARMv7 tables load");
        target
            .records
            .values()
            .filter(|record| record.is_a("BankedReg"))
            .map(|record| BankedRegister {
                name: table_text(record, "Name"),
                encoding: table_integer(record, "Encoding") as u32,
            })
            .collect()
    })
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
        "isITCondCodeNoAL" => matches!(operand, Operand::Condition(code) if *code != 14),
        "isITCondCodeRestrictedI" => {
            matches!(operand, Operand::Condition(code) if *code == 0 || *code == 1)
        }
        "isITCondCodeRestrictedS" => {
            matches!(operand, Operand::Condition(code) if (10..=13).contains(code))
        }
        "isITCondCodeRestrictedU" => {
            matches!(operand, Operand::Condition(code) if *code == 2 || *code == 8)
        }
        "isITCondCodeRestrictedFP" => {
            matches!(operand, Operand::Condition(code) if *code <= 1 || (10..=13).contains(code))
        }
        "isCCOut" => matches!(operand, Operand::CarrySet(_)),
        "isImm" => is_immediate,
        "isImmediate" => constant.is_some_and(|value| value >= number(0) && value <= number(1)),
        "isFBits16" => constant.is_some_and(|value| (0..=17).contains(&value)),
        "isFBits32" => constant.is_some_and(|value| (1..=33).contains(&value)),
        "isPowerTwoInRange" => constant.is_some_and(|value| {
            value >= number(0) && value <= number(1) && value > 0 && (value & (value - 1)) == 0
        }),
        "isMveSaturateOp" => constant.is_some_and(|value| value == 48 || value == 64),
        "isMVELongShift" => constant.is_some_and(|value| (1..=32).contains(&value)),
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
        "isMemImm7ShiftedOffset" => memory.is_some_and(|held| {
            let class = arguments
                .get(1)
                .map(|argument| {
                    argument
                        .trim_start_matches("ARM::")
                        .trim_end_matches("RegClassID")
                        .to_string()
                })
                .unwrap_or_default();
            if held.offset_register != NO_REGISTER
                || held.alignment != 0
                || !in_class(target, &class, held.base)
            {
                return false;
            }
            match &held.offset_immediate {
                None => true,
                Some(Expression::Constant(value)) => {
                    if *value == minimum {
                        return true;
                    }
                    let divisor = 1i64 << number(0);
                    value % divisor == 0 && (-127..=127).contains(&(value / divisor))
                }
                Some(_) => false,
            }
        }),
        "isMemRegRQOffset" => match operand {
            Operand::Memory(held) => {
                let shift = number(0);
                held.offset_immediate.is_none()
                    && held.alignment == 0
                    && in_class(target, "GPRnopc", held.base)
                    && (in_class(target, "MQPR", held.offset_register)
                        || in_class(target, "QPR", held.offset_register))
                    && if shift == 0 {
                        held.shift == Shift::None
                    } else {
                        held.shift == Shift::Uxtw && i64::from(held.shift_immediate) == shift
                    }
            }
            _ => false,
        },
        "isMemRegQOffset" => match operand {
            Operand::Memory(held) => {
                if held.offset_register != NO_REGISTER
                    || held.alignment != 0
                    || !(in_class(target, "MQPR", held.base) || in_class(target, "QPR", held.base))
                {
                    return false;
                }
                match &held.offset_immediate {
                    None => true,
                    Some(Expression::Constant(value)) => {
                        let shift = number(0);
                        let range = (1i64 << (7 + shift)) - 1;
                        value & ((1i64 << shift) - 1) == 0
                            && (*value == minimum || (*value > -range && *value < range))
                    }
                    Some(_) => false,
                }
            }
            _ => false,
        },
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
            Operand::Immediate(Expression::Symbol { specifier, .. }) => !specifier
                .as_deref()
                .is_some_and(|held| held.starts_with("upper16") || held.starts_with("lower16")),
            Operand::Immediate(Expression::Prefixed { prefix, .. }) => {
                !matches!(prefix.as_str(), "upper16" | "lower16")
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
        "isLEOffset" => match operand {
            Operand::Immediate(Expression::Symbol { .. }) => true,
            Operand::Immediate(Expression::Constant(value)) => {
                *value < 0 && *value >= -4094 && value & 1 == 0
            }
            _ => false,
        },
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
        "isRegList" | "isRegListWithAPSR" => match operand {
            Operand::RegisterList(list) => list.iter().all(|held| {
                in_class(target, "GPR", *held) || in_class(target, "GPRwithAPSRnosp", *held)
            }),
            _ => false,
        },
        "isDPRRegList" | "isDPR_VFP2RegList" | "isDPR_8RegList" => match operand {
            Operand::RegisterList(list) => {
                let class = match base.as_str() {
                    "isDPR_VFP2RegList" => "DPR_VFP2",
                    "isDPR_8RegList" => "DPR_8",
                    _ => "DPR",
                };
                list.iter().all(|held| in_class(target, class, *held))
            }
            _ => false,
        },
        "isFPSRegListWithVPR" | "isFPDRegListWithVPR" => match operand {
            Operand::RegisterList(list) => {
                let vpr = register_named(target, "vpr").unwrap_or(NO_REGISTER);
                let class = if base == "isFPSRegListWithVPR" {
                    "SPR"
                } else {
                    "DPR"
                };
                let body = match list.split_last() {
                    Some((last, body)) if *last == vpr => body,
                    _ => return false,
                };
                (base == "isFPSRegListWithVPR" && body.is_empty())
                    || (!body.is_empty()
                        && in_class(target, class, body[0])
                        && body.iter().all(|held| {
                            in_class(target, "SPR", *held) || in_class(target, "DPR", *held)
                        }))
            }
            _ => false,
        },
        "isSPRRegList" => match operand {
            Operand::RegisterList(list) => list.iter().all(|held| in_class(target, "SPR", *held)),
            _ => false,
        },
        "isVecListOneD" => match operand {
            Operand::Register(held) => in_class(target, "DPR", *held),
            Operand::VectorList {
                count,
                spaced: false,
                lane: VectorLane::Whole,
                ..
            } => *count == 1,
            _ => false,
        },
        "isVecListTwoMQ" | "isVecListFourMQ" => match operand {
            Operand::VectorList {
                register,
                count,
                spaced: false,
                lane: VectorLane::Whole,
            } => {
                *count == if base == "isVecListTwoMQ" { 2 } else { 4 }
                    && in_class(target, "MQPR", *register)
            }
            _ => false,
        },
        "isVecListDPair" => match operand {
            Operand::Register(held) => in_class(target, "QPR", *held),
            Operand::VectorList {
                register,
                spaced: false,
                lane: VectorLane::Whole,
                ..
            } => double_pair(target, *register, 1),
            _ => false,
        },
        "isVecListDPairSpaced" => match operand {
            Operand::VectorList {
                register,
                spaced: true,
                lane: VectorLane::Whole,
                ..
            } => double_pair(target, *register, 2),
            _ => false,
        },
        "isVecListDPairAllLanes" => match operand {
            Operand::VectorList {
                register,
                spaced: false,
                lane: VectorLane::All,
                ..
            } => double_pair(target, *register, 1),
            _ => false,
        },
        "isVecListThreeD"
        | "isVecListFourD"
        | "isVecListOneDAllLanes"
        | "isVecListThreeDAllLanes"
        | "isVecListFourDAllLanes"
        | "isVecListThreeQ"
        | "isVecListFourQ"
        | "isVecListDPairSpacedAllLanes"
        | "isVecListThreeQAllLanes"
        | "isVecListFourQAllLanes" => {
            let name = base.trim_start_matches("isVecList");
            let all_lanes = name.ends_with("AllLanes");
            let stem = name.trim_end_matches("AllLanes");
            let (wanted, spaced) = match stem {
                "OneD" => (1, false),
                "ThreeD" => (3, false),
                "FourD" => (4, false),
                "ThreeQ" => (3, true),
                "FourQ" => (4, true),
                _ => (2, true),
            };
            matches!(operand, Operand::VectorList { count, spaced: held_spaced, lane, .. }
                if *count == wanted
                    && *held_spaced == spaced
                    && *lane == if all_lanes { VectorLane::All } else { VectorLane::Whole })
        }
        name if name.starts_with("isVecList") && name.ends_with("Indexed") => {
            let stem = &name["isVecList".len()..name.len() - "Indexed".len()];
            let (count, spaced, limit) = match stem {
                "OneDByte" => (1, false, 7),
                "OneDHWord" => (1, false, 3),
                "OneDWord" => (1, false, 1),
                "TwoDByte" => (2, false, 7),
                "TwoDHWord" => (2, false, 3),
                "TwoQWord" => (2, true, 1),
                "TwoQHWord" => (2, true, 3),
                "TwoDWord" => (2, false, 1),
                "ThreeDByte" => (3, false, 7),
                "ThreeDHWord" => (3, false, 3),
                "ThreeQWord" => (3, true, 1),
                "ThreeQHWord" => (3, true, 3),
                "ThreeDWord" => (3, false, 1),
                "FourDByte" => (4, false, 7),
                "FourDHWord" => (4, false, 3),
                "FourQWord" => (4, true, 1),
                "FourQHWord" => (4, true, 3),
                _ => (4, false, 1),
            };
            matches!(operand, Operand::VectorList { count: held, spaced: held_spaced, lane: VectorLane::Index(index), .. }
                if *held == count && *held_spaced == spaced && *index <= limit)
        }
        "isComplexRotation" => constant
            .is_some_and(|value| (0..=270).contains(&value) && value % number(0) == number(1)),
        "isVectorIndexInRange" => {
            matches!(operand, Operand::VectorIndex(index) if i64::from(*index) < number(0))
        }
        "isMVEPairVectorIndex" => {
            matches!(operand, Operand::VectorIndex(index)
                if i64::from(*index) == number(0) || i64::from(*index) == number(1))
        }
        "isVectorIndex8" | "isVectorIndex16" | "isVectorIndex32" | "isVectorIndex64" => {
            let limit = match base.as_str() {
                "isVectorIndex8" => 8,
                "isVectorIndex16" => 4,
                "isVectorIndex32" => 2,
                _ => 1,
            };
            matches!(operand, Operand::VectorIndex(index) if *index < limit)
        }
        name if name.starts_with("isAlignedMemory") || name.starts_with("isDupAlignedMemory") => {
            let stem = name
                .trim_start_matches("isDup")
                .trim_start_matches("isAligned")
                .trim_start_matches("Memory");
            let stem = stem
                .trim_start_matches("Aligned")
                .trim_start_matches("Memory");
            memory.is_some_and(|held| {
                if held.offset_register != NO_REGISTER || held.offset_immediate.is_some() {
                    return false;
                }
                match stem {
                    "" => true,
                    "None" => held.alignment == 0,
                    other => {
                        let allowed: Vec<u32> = other
                            .split("or")
                            .filter_map(|piece| piece.parse::<u32>().ok())
                            .map(|bits| bits / 8)
                            .collect();
                        held.alignment == 0 || allowed.contains(&held.alignment)
                    }
                }
            })
        }
        "isAddrMode5" | "isAddrMode5FP16" => {
            let half = base == "isAddrMode5FP16";
            match operand {
                Operand::Immediate(Expression::Constant(_)) => false,
                Operand::Immediate(_) => true,
                _ => memory.is_some_and(|held| {
                    if held.offset_register != NO_REGISTER || held.alignment != 0 {
                        return false;
                    }
                    match &held.offset_immediate {
                        None => true,
                        Some(Expression::Constant(value)) => {
                            let limit = if half { 510 } else { 1020 };
                            let mask = if half { 1 } else { 3 };
                            (*value >= -limit && *value <= limit && value & mask == 0)
                                || *value == minimum
                        }
                        Some(_) => false,
                    }
                }),
            }
        }
        "isFPImm" => constant.is_some_and(|value| {
            u32::try_from(value).is_ok_and(|bits| floating_point_immediate(bits).is_some())
        }),
        "isNEONi8splat" => constant.is_some_and(|value| (0..256).contains(&value)),
        "isNEONi16splat" => {
            !neon_replicate(constant, 8, 2, false)
                && constant.is_some_and(|value| neon_i16_splat(value as u32))
        }
        "isNEONi16splatNot" => {
            constant.is_some_and(|value| neon_i16_splat(!(value as u32) & 0xffff))
        }
        "isNEONi32splat" => {
            !neon_replicate(constant, 8, 4, false)
                && constant.is_some_and(|value| neon_i32_splat(value as u32))
        }
        "isNEONi32splatNot" => constant.is_some_and(|value| neon_i32_splat(!(value as u32))),
        "isNEONi32vmov" => {
            !neon_replicate(constant, 8, 4, false) && constant.is_some_and(neon_valid_i32_vmov)
        }
        "isNEONi32vmovNeg" => constant.is_some_and(|value| neon_valid_i32_vmov(!value)),
        "isNEONi64splat" => constant.is_some_and(|value| {
            let mut rest = value as u64;
            (0..8).all(|_| {
                let byte = rest & 0xff;
                rest >>= 8;
                byte == 0 || byte == 0xff
            })
        }),
        "isNEONmovReplicate" | "isNEONinvReplicate" => {
            let from = number(0) as u32;
            let to = number(1) as u32;
            let inverse = base == "isNEONinvReplicate";
            if !inverse
                && to == 64
                && constant.is_some_and(|value| {
                    let mut rest = value as u64;
                    (0..8).all(|_| {
                        let byte = rest & 0xff;
                        rest >>= 8;
                        byte == 0 || byte == 0xff
                    })
                })
            {
                return false;
            }
            neon_replicate(constant, from, to / from.max(1), inverse)
        }
        "isBitfield" => matches!(operand, Operand::Bitfield { .. }),
        "isShifterImm" => matches!(operand, Operand::ShifterImmediate { .. }),
        "isITMask" => matches!(operand, Operand::ItMask(_)),
        "isMemBarrierOpt" => matches!(operand, Operand::MemoryBarrier(_)),
        "isInstSyncBarrierOpt" => matches!(operand, Operand::InstructionBarrier(_)),
        "isTraceSyncBarrierOpt" => matches!(operand, Operand::TraceSyncBarrier),
        "isRotImm" => matches!(operand, Operand::RotateImmediate(_)),
        "isCoprocNum" => matches!(operand, Operand::CoprocessorNumber(_)),
        "isCoprocReg" => matches!(operand, Operand::CoprocessorRegister(_)),
        "isCoprocOption" => matches!(operand, Operand::CoprocessorOption(_)),
        "isMSRMask" => matches!(operand, Operand::SystemMask(_)),
        "isBankedReg" => matches!(operand, Operand::BankedRegister(_)),
        "isProcIFlags" => matches!(operand, Operand::ProcessorFlags(_)),
        "isVPTPred" => matches!(operand, Operand::VectorPredicate(_)),
        "isConstPoolAsmImm" | "isConstantPoolImm" => matches!(operand, Operand::ConstantPool(_)),
        _ => false,
    }
}

fn floating_point_immediate(bits: u32) -> Option<u32> {
    let sign = (bits >> 31) & 1;
    let exponent = ((bits >> 23) & 0xff) as i32 - 127;
    let mantissa = bits & 0x7f_ffff;
    if mantissa & 0x7_ffff != 0 {
        return None;
    }
    let mantissa = mantissa >> 19;
    if mantissa & 0xf != mantissa || !(-3..=4).contains(&exponent) {
        return None;
    }
    let exponent = ((exponent + 3) & 7) ^ 4;
    Some((sign << 7) | ((exponent as u32) << 4) | mantissa)
}

fn neon_byte_splat(value: u32, size: u32) -> bool {
    let mut rest = value;
    let mut count = 0;
    for _ in 0..size {
        if rest & 0xff != 0 {
            count += 1;
        }
        rest >>= 8;
    }
    count == 1
}

fn neon_i16_splat(value: u32) -> bool {
    value <= 0xffff && (value == 0 || neon_byte_splat(value, 2))
}

fn encode_neon_i16_splat(value: u32) -> u32 {
    if value >= 0x100 {
        (value >> 8) | 0xa00
    } else {
        value | 0x800
    }
}

fn neon_i32_splat(value: u32) -> bool {
    value == 0 || neon_byte_splat(value, 4)
}

fn encode_neon_i32_splat(value: u32) -> u32 {
    if (0x100..=0xff00).contains(&value) {
        (value >> 8) | 0x200
    } else if value > 0xffff && value <= 0xff_0000 {
        (value >> 16) | 0x400
    } else if value > 0xff_ffff {
        (value >> 24) | 0x600
    } else {
        value
    }
}

fn neon_valid_i32_vmov(value: i64) -> bool {
    (value as u64) & 0xffff_ffff_ffff_ff00 == 0
        || (value as u64) & 0xffff_ffff_ffff_00ff == 0
        || (value as u64) & 0xffff_ffff_ff00_ffff == 0
        || (value as u64) & 0xffff_ffff_00ff_ffff == 0
        || (value as u64) & 0xffff_ffff_ffff_00ff == 0xff
        || (value as u64) & 0xffff_ffff_ff00_ffff == 0xffff
}

fn encode_neon_vmov_immediate(value: u32) -> u32 {
    if (256..=0xffff).contains(&value) {
        (value >> 8) | if value & 0xff != 0 { 0xc00 } else { 0x200 }
    } else if value > 0xffff && value <= 0xff_ffff {
        (value >> 16) | if value & 0xff != 0 { 0xd00 } else { 0x400 }
    } else if value > 0xff_ffff {
        (value >> 24) | 0x600
    } else {
        value
    }
}

fn neon_replicate(constant: Option<i64>, width: u32, elements: u32, inverse: bool) -> bool {
    let Some(mut value) = constant else {
        return false;
    };
    if value == 0 {
        return false;
    }
    if inverse {
        value = !value;
    }
    let mask = (1u64 << width) - 1;
    let element = (value as u64) & mask;
    if width == 16 && element & 0x00ff != 0 && element & 0xff00 != 0 {
        return false;
    }
    if width == 32 && !neon_valid_i32_vmov(element as i64) {
        return false;
    }
    let mut rest = value as u64;
    for _ in 1..elements {
        rest >>= width;
        if rest & mask != element {
            return false;
        }
    }
    true
}

fn double_pair(target: &Target, register: RegisterId, step: u32) -> bool {
    let held = &target.registers[register as usize];
    held.sub_registers.len() == 2
        && held
            .sub_registers
            .iter()
            .all(|member| in_class(target, "DPR", *member))
        && register_encoding(target, held.sub_registers[1])
            == register_encoding(target, held.sub_registers[0]) + step
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
    vpt_code: u32,
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

const VPT_PREDICABLE: &[&str] = &[
    "vabav",
    "vabd",
    "vabs",
    "vadc",
    "vadd",
    "vaddlv",
    "vaddv",
    "vand",
    "vbic",
    "vbrsr",
    "vcadd",
    "vcls",
    "vclz",
    "vcmla",
    "vcmp",
    "vcmul",
    "vctp",
    "vcvt",
    "vddup",
    "vdup",
    "vdwdup",
    "veor",
    "vfma",
    "vfmas",
    "vfms",
    "vhadd",
    "vhcadd",
    "vhsub",
    "vidup",
    "viwdup",
    "vldrb",
    "vldrd",
    "vldrw",
    "vmax",
    "vmaxa",
    "vmaxav",
    "vmaxnm",
    "vmaxnma",
    "vmaxnmav",
    "vmaxnmv",
    "vmaxv",
    "vmin",
    "vminav",
    "vminnm",
    "vminnmav",
    "vminnmv",
    "vminv",
    "vmla",
    "vmladav",
    "vmlaldav",
    "vmlalv",
    "vmlas",
    "vmlav",
    "vmlsdav",
    "vmlsldav",
    "vmovlb",
    "vmovlt",
    "vmovnb",
    "vmovnt",
    "vmul",
    "vmvn",
    "vneg",
    "vorn",
    "vorr",
    "vpnot",
    "vpsel",
    "vqabs",
    "vqadd",
    "vqdmladh",
    "vqdmlah",
    "vqdmlash",
    "vqdmlsdh",
    "vqdmulh",
    "vqdmull",
    "vqmovn",
    "vqmovun",
    "vqneg",
    "vqrdmladh",
    "vqrdmlah",
    "vqrdmlash",
    "vqrdmlsdh",
    "vqrdmulh",
    "vqrshl",
    "vqrshrn",
    "vqrshrun",
    "vqshl",
    "vqshrn",
    "vqshrun",
    "vqsub",
    "vrev16",
    "vrev32",
    "vrev64",
    "vrhadd",
    "vrmlaldavh",
    "vrmlalvh",
    "vrmlsldavh",
    "vrmulh",
    "vrshl",
    "vrshr",
    "vrshrn",
    "vsbc",
    "vshl",
    "vshlc",
    "vshll",
    "vshr",
    "vshrn",
    "vsli",
    "vsri",
    "vstrb",
    "vstrd",
    "vstrw",
    "vsub",
];

const VPT_SPLIT_EXCEPTIONS: &[&str] = &[
    "vmovlt",
    "vshllt",
    "vrshrnt",
    "vshrnt",
    "vqrshrunt",
    "vqshrunt",
    "vqrshrnt",
    "vqshrnt",
    "vmullt",
    "vqmovnt",
    "vqmovunt",
    "vmovnt",
    "vqdmullt",
    "vpnot",
    "vcvtt",
    "vcvt",
];

const MVE_NO_CONDITION: &[&str] = &[
    "vmine", "vshle", "vshlt", "vshllt", "vrshle", "vrshlt", "vmvne", "vorne", "vnege", "vnegt",
    "vmule", "vmult", "vrintne", "vcmult", "vcmule", "vpsele", "vpselt",
];

fn is_vpt_predicable(mnemonic: &str, extra: &str, mve: bool) -> bool {
    if !mve {
        return false;
    }
    if (mnemonic.starts_with("vldrh") && mnemonic != "vldrhi")
        || (mnemonic.starts_with("vmov") && !matches!(extra, ".f16" | ".32" | ".16" | ".8"))
        || (mnemonic.starts_with("vrint") && mnemonic != "vrintr")
        || (mnemonic.starts_with("vstrh") && mnemonic != "vstrhi")
    {
        return true;
    }
    let cde = mnemonic.strip_prefix("vcx").is_some_and(|rest| {
        let rest = rest.strip_suffix(['t', 'e']).unwrap_or(rest);
        matches!(rest, "1" | "1a" | "2" | "2a" | "3" | "3a")
    });
    cde || VPT_PREDICABLE
        .iter()
        .any(|prefix| mnemonic.starts_with(prefix))
}

fn split_mnemonic(text: &str, thumb: bool, mve: bool, extra: &str) -> Mnemonic {
    let mut result = Mnemonic {
        it_mask: None,
        base: text.to_string(),
        condition: CONDITION_AL,
        carry: false,
        interrupt_modification: 0,
        vpt_code: 0,
    };
    if (text == "movs" && thumb) || UNSPLIT.contains(&text) || text.starts_with("vsel") {
        return result;
    }
    let mut name = text.to_string();
    let mve_keep = mve && (MVE_NO_CONDITION.contains(&text) || text.starts_with("vq"));
    if !KEEP_CONDITION_SUFFIX.contains(&name.as_str()) && !mve_keep && name.len() >= 2 {
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
    if is_vpt_predicable(&name, extra, mve) && !VPT_SPLIT_EXCEPTIONS.contains(&name.as_str()) {
        let code = match name.chars().last() {
            Some('t') => 1,
            Some('e') => 2,
            _ => 0,
        };
        if code != 0 {
            name.pop();
            result.vpt_code = code;
        }
        result.base = name;
        return result;
    }
    if let Some(mask) = name.strip_prefix("it") {
        result.it_mask = Some(mask.to_string());
        name = "it".to_string();
    }
    if let Some(mask) = name.strip_prefix("vpst") {
        result.it_mask = Some(mask.to_string());
        name = "vpst".to_string();
    } else if let Some(mask) = name.strip_prefix("vpt") {
        result.it_mask = Some(mask.to_string());
        name = "vpt".to_string();
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

fn accepts_condition(mnemonic: &str, thumb: bool, full: &str) -> bool {
    if full.starts_with("vmull") && full.ends_with(".p64") {
        return false;
    }
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
    NearMisses(Vec<NearMiss>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PredicateError {
    NotITBlock,
    ITBlock,
    V6,
    Thumb2,
    V8,
    FlagSetting,
    TiedOperand,
    InvalidOperand,
}

#[derive(Clone, Debug)]
enum NearMiss {
    Operand {
        message: Option<String>,
        class: Option<usize>,
        index: usize,
    },
    TooFewOperands,
    Feature(Vec<String>),
    Predicate(PredicateError),
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
        self.arm32_try_register_bound(false)
    }

    fn arm32_try_register_bound(&mut self, allow_out_of_bound: bool) -> Option<RegisterId> {
        if !self.is(TokenKind::Identifier) {
            return None;
        }
        let register = register_named(self.target, &self.lexer.token.text).or_else(|| {
            self.arm_aliases
                .get(&self.lexer.token.text.to_ascii_lowercase())
                .map(|(_, held)| *held)
        })?;
        if !allow_out_of_bound
            && in_class(self.target, "DPR", register)
            && !in_class(self.target, "DPR_VFP2", register)
            && !self
                .arm32_feature("d32")
                .is_some_and(|held| self.arm_features.contains(&held))
        {
            return None;
        }
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
        self.arm32_parse_register_list_with(operands, false)
    }

    fn arm32_parse_register_list_with(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        allow_authentication: bool,
    ) -> bool {
        let start = self.lexer.token.start;
        let scrub = operands
            .first()
            .is_some_and(|held| held.token_text().eq_ignore_ascii_case("vscclrm"));
        let lazy = operands.first().is_some_and(|held| {
            held.token_text().eq_ignore_ascii_case("vlstm")
                || held.token_text().eq_ignore_ascii_case("vlldm")
        });
        let loose = scrub || lazy;
        self.lex();
        let location = self.location();
        let Some(mut register) = self.arm32_try_register_bound(loose) else {
            return self.error(location, "register expected");
        };
        let authentication = register_named(self.target, "ra_auth_code");
        if !allow_authentication && Some(register) == authentication {
            return self.error(location, "pseudo-register not allowed");
        }
        let mut register_location = location;
        let to_double = |target: &Target, held: RegisterId, odd: u32| -> Option<RegisterId> {
            in_class(target, "QPR", held)
                .then(|| {
                    register_named(
                        target,
                        &format!("d{}", register_encoding(target, held) * 2 + odd),
                    )
                })
                .flatten()
        };
        let mut leading: Option<RegisterId> = None;
        if let Some(low) = to_double(self.target, register, 0) {
            leading = to_double(self.target, register, 1);
            register = low;
        }
        let vpr = register_named(self.target, "vpr").unwrap_or(NO_REGISTER);
        let mut family = if in_class(self.target, "DPR", register) {
            "DPR"
        } else if in_class(self.target, "SPR", register) {
            "SPR"
        } else if register == vpr {
            "FPV"
        } else {
            "GPR"
        };
        let belongs = |target: &Target, family: &str, held: RegisterId| -> bool {
            if family == "GPR" {
                in_class(target, "GPR", held)
                    || in_class(target, "GPRwithAPSRnosp", held)
                    || Some(held) == authentication
            } else if family == "FPV" {
                in_class(target, "SPR", held) || in_class(target, "DPR", held) || held == vpr
            } else {
                in_class(target, family, held)
            }
        };
        if !belongs(self.target, family, register) {
            return self.error(location, "invalid register in register list");
        }
        let mut registers: Vec<RegisterId> = vec![register];
        if let Some(high) = leading {
            registers.push(high);
            register = high;
        }
        let mut adjust = false;
        let mut previous = register_encoding(self.target, register);
        while self.is(TokenKind::Comma) || self.is(TokenKind::Minus) {
            if self.is(TokenKind::Minus) {
                self.lex();
                let after = self.location();
                let Some(mut end) = self.arm32_try_register_bound(loose) else {
                    return self.error(after, "register expected");
                };
                if Some(register) == authentication {
                    return self.error(register_location, "pseudo-register not allowed");
                }
                if Some(end) == authentication {
                    return self.error(after, "pseudo-register not allowed");
                }
                if let Some(high) = to_double(self.target, end, 1) {
                    end = high;
                }
                if register == end {
                    continue;
                }
                if !belongs(self.target, family, register) || !belongs(self.target, family, end) {
                    return self.error(after, "invalid register in register list");
                }
                let first = register_encoding(self.target, register);
                let last = register_encoding(self.target, end);
                if first > last {
                    return self.error(after, "bad range in register list");
                }
                let double = in_class(self.target, "DPR", register);
                for encoding in first + 1..=last {
                    let name = match family {
                        "DPR" => format!("d{encoding}"),
                        "SPR" => format!("s{encoding}"),
                        "FPV" if double => format!("d{encoding}"),
                        "FPV" => format!("s{encoding}"),
                        _ => match encoding {
                            13 => "sp".to_string(),
                            14 => "lr".to_string(),
                            15 => "pc".to_string(),
                            _ => format!("r{encoding}"),
                        },
                    };
                    let Some(next) = register_named(self.target, &name) else {
                        return self.error(after, "invalid register in register list");
                    };
                    if registers.iter().any(|held| {
                        register_encoding(self.target, *held)
                            == register_encoding(self.target, next)
                    }) {
                        self.message(
                            MessageKind::Warning,
                            after,
                            &format!("duplicated register ({name}) in register list"),
                            None,
                        );
                    } else {
                        registers.push(next);
                    }
                    register = next;
                }
                previous = last + if adjust { 16 } else { 0 };
                continue;
            }
            self.lex();
            let location = self.location();
            let written = self.lexer.token.text.clone();
            let Some(mut next) = self.arm32_try_register_bound(loose) else {
                return self.error(location, "register expected");
            };
            if !allow_authentication && Some(next) == authentication {
                return self.error(location, "pseudo-register not allowed");
            }
            register_location = location;
            let mut trailing: Option<RegisterId> = None;
            if let Some(low) = to_double(self.target, next, 0) {
                trailing = to_double(self.target, next, 1);
                next = low;
            }
            if next == vpr && matches!(family, "SPR" | "DPR" | "FPV") {
                family = "FPV";
                if registers.contains(&next) {
                    self.message(
                        MessageKind::Warning,
                        location,
                        &format!("duplicated register ({written}) in register list"),
                        None,
                    );
                } else {
                    registers.push(next);
                }
                register = next;
                previous = register_encoding(self.target, next);
                continue;
            }
            if scrub
                && family == "SPR"
                && register_encoding(self.target, register) == 31
                && Some(next) == register_named(self.target, "d16")
            {
                adjust = true;
                family = "FPV";
            }
            if !belongs(self.target, family, next) {
                return self.error(location, "invalid register in register list");
            }
            let encoding = register_encoding(self.target, next) + if adjust { 16 } else { 0 };
            if encoding < previous {
                if family != "GPR" {
                    return self.error(location, "register list not in ascending order");
                }
                self.message(
                    MessageKind::Warning,
                    location,
                    "register list not in ascending order",
                    None,
                );
            }
            if family != "GPR" && encoding != previous + 1 {
                return self.error(location, "non-contiguous register range");
            }
            if registers.iter().any(|held| {
                register_encoding(self.target, *held)
                    + if adjust && in_class(self.target, "DPR", *held) {
                        16
                    } else {
                        0
                    }
                    == encoding
            }) {
                self.message(
                    MessageKind::Warning,
                    location,
                    &format!("duplicated register ({written}) in register list"),
                    None,
                );
            } else {
                registers.push(next);
            }
            register = next;
            previous = encoding;
            if let Some(high) = trailing {
                registers.push(high);
                register = high;
                previous = register_encoding(self.target, high);
            }
        }
        if !self.is(TokenKind::RightBrace) {
            return self.token_error("'}' expected");
        }
        let end = self.lexer.token.end;
        self.lex();
        let target = self.target;
        let key = |held: &RegisterId| {
            register_encoding(target, *held)
                + if adjust && in_class(target, "DPR", *held) {
                    16
                } else {
                    0
                }
        };
        registers.sort_by_key(key);
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
        let first = match first {
            Expression::Constant(_) => first,
            other => match self.context.evaluate_absolute(&other, None) {
                Some(value) => Expression::Constant(value),
                None => other,
            },
        };
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

    fn arm32_parse_pkh_immediate(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        arithmetic: bool,
    ) -> Parse {
        if !self.is(TokenKind::Identifier) {
            return Parse::NoMatch;
        }
        let start = self.lexer.token.start;
        let wanted = if arithmetic { "asr" } else { "lsl" };
        let written = self.lexer.token.text.to_ascii_lowercase();
        if Shift::named(&written).is_none() {
            return Parse::NoMatch;
        }
        if written != wanted {
            let location = self.location();
            self.error(location, &format!("{wanted} operand expected."));
            return Parse::Failure;
        }
        self.lex();
        if !self.is(TokenKind::Hash) && !self.is(TokenKind::Dollar) {
            return Parse::NoMatch;
        }
        self.lex();
        let location = self.location();
        let Ok((expression, _)) = self.parse_expression() else {
            self.error(location, "illegal expression");
            return Parse::Failure;
        };
        let Expression::Constant(value) = expression else {
            self.error(location, "constant expression expected");
            return Parse::Failure;
        };
        let (low, high) = if arithmetic { (1, 32) } else { (0, 31) };
        if value < low || value > high {
            self.error(location, "immediate value out of range");
            return Parse::Failure;
        }
        let end = self.lexer.token.start;
        operands.push(parsed(
            Operand::Immediate(Expression::Constant(value)),
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

    fn arm32_coprocessor_index(text: &str, prefix: char) -> Option<u32> {
        let text = text.to_ascii_lowercase();
        let rest = text.strip_prefix(prefix)?;
        let digits = rest.strip_prefix('r').unwrap_or(rest);
        if digits.is_empty()
            || digits.len() > 2
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
            || (digits.len() == 2 && !digits.starts_with('1'))
        {
            return None;
        }
        digits.parse::<u32>().ok().filter(|value| *value <= 15)
    }

    fn arm32_parse_coprocessor(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        prefix: char,
    ) -> Parse {
        if !self.is(TokenKind::Identifier) {
            return Parse::NoMatch;
        }
        let Some(number) = Self::arm32_coprocessor_index(&self.lexer.token.text, prefix) else {
            return Parse::NoMatch;
        };
        if prefix == 'p' {
            let version8 = self.arm_features.contains("HasV8Ops");
            let main_line = self.arm_features.contains("HasV8_1MMainlineOps");
            if (version8 && number & 0xe != 0xe)
                || (main_line && (number & 0xe == 0x8 || number & 0xe == 0xe))
            {
                return Parse::NoMatch;
            }
        }
        let start = self.lexer.token.start;
        let end = self.lexer.token.end;
        self.lex();
        let operand = if prefix == 'p' {
            Operand::CoprocessorNumber(number)
        } else {
            Operand::CoprocessorRegister(number)
        };
        operands.push(parsed(operand, start, end));
        Parse::Success
    }

    fn arm32_parse_system_mask(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        if self.is(TokenKind::Integer) {
            let value = self.lexer.token.value;
            if value > 255 {
                return Parse::NoMatch;
            }
            let start = self.lexer.token.start;
            let end = self.lexer.token.end;
            self.lex();
            operands.push(parsed(Operand::SystemMask(value as u32), start, end));
            return Parse::Success;
        }
        if !self.is(TokenKind::Identifier) {
            return Parse::NoMatch;
        }
        let text = self.lexer.token.text.clone();
        let start = self.lexer.token.start;
        let end = self.lexer.token.end;
        let m_class = self.arm_features.contains("FeatureMClass");
        let value = if m_class {
            let lower = text.to_ascii_lowercase();
            let Some(register) = system_registers()
                .iter()
                .find(|held| held.name == lower && held.available(&self.arm_features))
            else {
                return Parse::NoMatch;
            };
            register.encoding & 0xfff
        } else {
            let (special, flags) = match text.split_once('_') {
                Some((special, flags)) => (special.to_ascii_lowercase(), flags.to_string()),
                None => (text.to_ascii_lowercase(), String::new()),
            };
            let mut value = 0u32;
            if special == "apsr" {
                value = match flags.as_str() {
                    "nzcvq" => 8,
                    "g" => 4,
                    "nzcvqg" => 12,
                    "" => 8,
                    _ => return Parse::NoMatch,
                };
            } else if special == "cpsr" || special == "spsr" {
                let flags = if flags == "all" || flags.is_empty() {
                    "fc".to_string()
                } else {
                    flags
                };
                for letter in flags.chars() {
                    let flag = match letter {
                        'c' => 1,
                        'x' => 2,
                        's' => 4,
                        'f' => 8,
                        _ => return Parse::NoMatch,
                    };
                    if value & flag != 0 {
                        return Parse::NoMatch;
                    }
                    value |= flag;
                }
                if special == "spsr" {
                    value |= 16;
                }
            } else {
                return Parse::NoMatch;
            }
            value
        };
        self.lex();
        operands.push(parsed(Operand::SystemMask(value), start, end));
        Parse::Success
    }

    fn arm32_parse_processor_flags(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        if !self.is(TokenKind::Identifier) {
            return Parse::NoMatch;
        }
        let text = self.lexer.token.text.clone();
        let mut flags = 0u32;
        if text != "none" {
            for letter in text.chars() {
                let flag = match letter.to_ascii_lowercase() {
                    'a' => 4,
                    'i' => 2,
                    'f' => 1,
                    _ => return Parse::NoMatch,
                };
                if flags & flag != 0 {
                    return Parse::NoMatch;
                }
                flags |= flag;
            }
        }
        let start = self.lexer.token.start;
        let end = self.lexer.token.end;
        self.lex();
        operands.push(parsed(Operand::ProcessorFlags(flags), start, end));
        Parse::Success
    }

    fn arm32_parse_banked_register(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        if !self.is(TokenKind::Identifier) {
            return Parse::NoMatch;
        }
        let lower = self.lexer.token.text.to_ascii_lowercase();
        let Some(register) = banked_registers().iter().find(|held| held.name == lower) else {
            return Parse::NoMatch;
        };
        let value = register.encoding;
        let start = self.lexer.token.start;
        let end = self.lexer.token.end;
        self.lex();
        operands.push(parsed(Operand::BankedRegister(value), start, end));
        Parse::Success
    }

    fn arm32_parse_coprocessor_option(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        if !self.is(TokenKind::LeftBrace) {
            return Parse::NoMatch;
        }
        let start = self.lexer.token.start;
        self.lex();
        let location = self.location();
        let Ok((expression, _)) = self.parse_expression() else {
            self.error(location, "illegal expression");
            return Parse::Failure;
        };
        let value = match expression {
            Expression::Constant(value) if (0..=255).contains(&value) => value as u32,
            _ => {
                self.error(
                    location,
                    "coprocessor option must be an immediate in range [0, 255]",
                );
                return Parse::Failure;
            }
        };
        if !self.is(TokenKind::RightBrace) {
            return Parse::Failure;
        }
        let end = self.lexer.token.end;
        self.lex();
        operands.push(parsed(Operand::CoprocessorOption(value), start, end));
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

    fn arm32_parse_trace_barrier(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        let start = self.lexer.token.start;
        if !self.is(TokenKind::Identifier) || !self.lexer.token.text.eq_ignore_ascii_case("csync") {
            return Parse::NoMatch;
        }
        self.lex();
        let end = self.lexer.token.start;
        operands.push(parsed(Operand::TraceSyncBarrier, start, end));
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
            if !instruction
                && !self.arm_features.contains("HasV8Ops")
                && matches!(value, 9 | 1 | 5 | 13)
            {
                return Parse::NoMatch;
            }
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
                predicates = self.target.aliases[alias].predicates.clone();
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
                if class.kind == ClassKind::Token && class.value_name.starts_with('.') {
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
                "FPImm" => self.arm32_parse_fp_immediate(operands, mnemonic),
                name if name.starts_with("VecList") => self.arm32_parse_vector_list(operands),
                "AM3Offset" => self.arm32_parse_am3_offset(operands),
                "ITCondCode"
                | "CondCodeNoAL"
                | "CondCodeNoALInv"
                | "CondCodeRestrictedI"
                | "CondCodeRestrictedS"
                | "CondCodeRestrictedU"
                | "CondCodeRestrictedFP" => self.arm32_parse_it_condition(operands),
                "MemBarrierOpt" => self.arm32_parse_barrier(operands, false),
                "InstSyncBarrierOpt" => self.arm32_parse_barrier(operands, true),
                "TraceSyncBarrierOpt" => self.arm32_parse_trace_barrier(operands),
                "Bitfield" => self.arm32_parse_bitfield(operands),
                "ShifterImm" => self.arm32_parse_shifter_immediate(operands),
                "PKHLSLImm" => self.arm32_parse_pkh_immediate(operands, false),
                "PKHASRImm" => self.arm32_parse_pkh_immediate(operands, true),
                "RotImm" => self.arm32_parse_rotate_immediate(operands),
                "SetEndImm" => self.arm32_parse_set_end(operands),
                "CoprocNum" => self.arm32_parse_coprocessor(operands, 'p'),
                "CoprocReg" => self.arm32_parse_coprocessor(operands, 'c'),
                "CoprocOption" => self.arm32_parse_coprocessor_option(operands),
                "MSRMask" => self.arm32_parse_system_mask(operands),
                "BankedReg" => self.arm32_parse_banked_register(operands),
                "ProcIFlags" => self.arm32_parse_processor_flags(operands),
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
                        if self.is(TokenKind::LeftBracket) {
                            let index_start = self.lexer.token.start;
                            self.lex();
                            let Ok((expression, _)) = self.parse_expression() else {
                                return true;
                            };
                            let Expression::Constant(index) = expression else {
                                return self
                                    .token_error("immediate value expected for vector index");
                            };
                            if !self.is(TokenKind::RightBracket) {
                                return self.token_error("']' expected");
                            }
                            let index_end = self.lexer.token.end;
                            self.lex();
                            operands.push(parsed(
                                Operand::VectorIndex(index as u32),
                                index_start,
                                index_end,
                            ));
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

    fn arm32_vector_lane(&mut self) -> Result<VectorLane, ()> {
        if !self.is(TokenKind::LeftBracket) {
            return Ok(VectorLane::Whole);
        }
        self.lex();
        if self.is(TokenKind::RightBracket) {
            self.lex();
            return Ok(VectorLane::All);
        }
        if self.is(TokenKind::Hash) {
            self.lex();
        }
        let location = self.location();
        let Ok((expression, _)) = self.parse_expression() else {
            self.error(location, "illegal expression");
            return Err(());
        };
        let Expression::Constant(value) = expression else {
            self.error(location, "lane index must be empty or an integer");
            return Err(());
        };
        if !self.is(TokenKind::RightBracket) {
            self.token_error("']' expected");
            return Err(());
        }
        self.lex();
        if !(0..=7).contains(&value) {
            let at = self.location();
            self.error(at, "lane index out of range");
            return Err(());
        }
        Ok(VectorLane::Index(value as u32))
    }

    fn arm32_double_register(&self, number: u32) -> Option<RegisterId> {
        register_named(self.target, &format!("d{number}"))
    }

    fn arm32_double_of_quad(&self, register: RegisterId) -> u32 {
        let held = &self.target.registers[register as usize];
        held.sub_registers
            .first()
            .map_or(0, |first| register_encoding(self.target, *first))
    }

    fn arm32_register_pair(&self, first: u32, step: u32) -> Option<RegisterId> {
        let low = self.arm32_double_register(first)?;
        let high = self.arm32_double_register(first + step)?;
        self.target
            .registers
            .iter()
            .position(|held| held.sub_registers == [low, high])
            .map(|index| index as RegisterId)
    }

    fn arm32_parse_mve_vector_list(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        let start = self.lexer.token.start;
        self.lex();
        let mut location = self.location();
        let Some(first) = self.arm32_try_register() else {
            self.error(location, "register expected");
            return Parse::Failure;
        };
        let target = self.target;
        let q_register = |held: RegisterId| in_class(target, "MQPR", held);
        if !q_register(first) {
            let at = self.location();
            self.error(at, "vector register in range Q0-Q7 expected");
            return Parse::Failure;
        }
        let mut count = 1u32;
        let mut last = first;
        if self.arm32_vector_lane() != Ok(VectorLane::Whole) {
            return Parse::Failure;
        }
        while self.is(TokenKind::Comma) || self.is(TokenKind::Minus) {
            if self.is(TokenKind::Minus) {
                self.lex();
                let after = self.location();
                let Some(end) = self.arm32_try_register() else {
                    self.error(after, "register expected");
                    return Parse::Failure;
                };
                if last == end {
                    continue;
                }
                if !q_register(end) {
                    self.error(after, "invalid register in register list");
                    return Parse::Failure;
                }
                let (low, high) = (
                    register_encoding(target, last),
                    register_encoding(target, end),
                );
                if low > high {
                    self.error(after, "bad range in register list");
                    return Parse::Failure;
                }
                if self.arm32_vector_lane() != Ok(VectorLane::Whole) {
                    return Parse::Failure;
                }
                count += high - low;
                last = end;
                continue;
            }
            self.lex();
            location = self.location();
            let Some(next) = self.arm32_try_register() else {
                self.error(location, "register expected");
                return Parse::Failure;
            };
            if !q_register(next) {
                self.error(location, "vector register in range Q0-Q7 expected");
                return Parse::Failure;
            }
            if register_encoding(target, next) != register_encoding(target, last) + 1 {
                self.error(location, "non-contiguous register range");
                return Parse::Failure;
            }
            count += 1;
            last = next;
            if self.arm32_vector_lane() != Ok(VectorLane::Whole) {
                return Parse::Failure;
            }
        }
        if !self.is(TokenKind::RightBrace) {
            self.token_error("'}' expected");
            return Parse::Failure;
        }
        let end = self.lexer.token.end;
        self.lex();
        operands.push(parsed(
            Operand::VectorList {
                register: first,
                count,
                spaced: false,
                lane: VectorLane::Whole,
            },
            start,
            end,
        ));
        Parse::Success
    }

    fn arm32_parse_vector_list(&mut self, operands: &mut Vec<ParsedOperand>) -> Parse {
        if self.arm_features.contains("HasMVEIntegerOps") {
            if self.is(TokenKind::LeftBrace) {
                return self.arm32_parse_mve_vector_list(operands);
            }
            return Parse::NoMatch;
        }
        let start = self.lexer.token.start;
        if self.is(TokenKind::Identifier) {
            let Some(register) = self.arm32_try_register() else {
                return Parse::NoMatch;
            };
            let end = self.lexer.token.start;
            let quad = in_class(self.target, "QPR", register);
            let double = in_class(self.target, "DPR", register);
            if !quad && !double {
                operands.push(parsed(Operand::Register(register), start, end));
                return Parse::Success;
            }
            let first = if quad {
                self.arm32_double_of_quad(register)
            } else {
                register_encoding(self.target, register)
            };
            let Ok(lane) = self.arm32_vector_lane() else {
                return Parse::Failure;
            };
            let end = self.lexer.token.start;
            let held = if quad {
                self.arm32_double_register(first).unwrap_or(register)
            } else {
                register
            };
            let operand = match lane {
                VectorLane::Whole => Operand::Register(held),
                other => Operand::VectorList {
                    register: if quad && other == VectorLane::All {
                        self.arm32_register_pair(first, 1).unwrap_or(held)
                    } else {
                        held
                    },
                    count: if quad { 2 } else { 1 },
                    spaced: false,
                    lane: other,
                },
            };
            operands.push(parsed(operand, start, end));
            return Parse::Success;
        }
        if !self.is(TokenKind::LeftBrace) {
            return Parse::NoMatch;
        }
        self.lex();
        let location = self.location();
        let Some(register) = self.arm32_try_register() else {
            self.error(location, "register expected");
            return Parse::Failure;
        };
        let mut count = 1u32;
        let mut spacing = 0u32;
        let mut first_register = if in_class(self.target, "QPR", register) {
            self.arm32_double_of_quad(register)
        } else {
            register_encoding(self.target, register)
        };
        let first = first_register;
        let mut last = first_register;
        if in_class(self.target, "QPR", register) {
            spacing = 1;
            last = first_register + 1;
            count += 1;
        } else if !in_class(self.target, "DPR", register) {
            self.error(location, "invalid register in register list");
            return Parse::Failure;
        }
        let Ok(lane) = self.arm32_vector_lane() else {
            return Parse::Failure;
        };
        while self.is(TokenKind::Comma) || self.is(TokenKind::Minus) {
            if self.is(TokenKind::Minus) {
                if spacing == 0 {
                    spacing = 1;
                } else if spacing == 2 {
                    self.token_error("sequential registers in double spaced list");
                    return Parse::Failure;
                }
                self.lex();
                let after = self.location();
                let Some(end_register) = self.arm32_try_register() else {
                    self.error(after, "register expected");
                    return Parse::Failure;
                };
                let end_number = if in_class(self.target, "QPR", end_register) {
                    self.arm32_double_of_quad(end_register) + 1
                } else if in_class(self.target, "DPR", end_register) {
                    register_encoding(self.target, end_register)
                } else {
                    self.error(after, "invalid register in register list");
                    return Parse::Failure;
                };
                if last == end_number {
                    continue;
                }
                if last > end_number {
                    self.error(after, "bad range in register list");
                    return Parse::Failure;
                }
                let Ok(next_lane) = self.arm32_vector_lane() else {
                    return Parse::Failure;
                };
                if next_lane != lane {
                    self.error(after, "mismatched lane index in register list");
                    return Parse::Failure;
                }
                count += end_number - last;
                last = end_number;
                continue;
            }
            self.lex();
            let location = self.location();
            let old = last;
            let Some(next) = self.arm32_try_register() else {
                self.error(location, "register expected");
                return Parse::Failure;
            };
            if in_class(self.target, "QPR", next) {
                if spacing == 0 {
                    spacing = 1;
                } else if spacing == 2 {
                    self.error(
                        location,
                        "invalid register in double-spaced list (must be 'D' register')",
                    );
                    return Parse::Failure;
                }
                let number = self.arm32_double_of_quad(next);
                if number != old + 1 {
                    self.error(location, "non-contiguous register range");
                    return Parse::Failure;
                }
                last = number + 1;
                count += 2;
                let at = self.location();
                let Ok(next_lane) = self.arm32_vector_lane() else {
                    return Parse::Failure;
                };
                if next_lane != lane {
                    self.error(at, "mismatched lane index in register list");
                    return Parse::Failure;
                }
                continue;
            }
            if !in_class(self.target, "DPR", next) {
                self.error(location, "invalid register in register list");
                return Parse::Failure;
            }
            let number = register_encoding(self.target, next);
            if spacing == 0 {
                spacing = 1 + u32::from(number == old + 2);
            }
            if number != old + spacing {
                self.error(location, "non-contiguous register range");
                return Parse::Failure;
            }
            last = number;
            count += 1;
            let at = self.location();
            let Ok(next_lane) = self.arm32_vector_lane() else {
                return Parse::Failure;
            };
            if next_lane != lane {
                self.error(at, "mismatched lane index in register list");
                return Parse::Failure;
            }
        }
        if !self.is(TokenKind::RightBrace) {
            self.token_error("'}' expected");
            return Parse::Failure;
        }
        let end = self.lexer.token.end;
        self.lex();
        first_register = first;
        let Some(first_double) = self.arm32_double_register(first_register) else {
            self.error(location, "invalid register in register list");
            return Parse::Failure;
        };
        let held = match lane {
            VectorLane::Index(_) => first_double,
            _ if count == 2 => self
                .arm32_register_pair(first_register, if spacing == 2 { 2 } else { 1 })
                .unwrap_or(first_double),
            _ => first_double,
        };
        operands.push(parsed(
            Operand::VectorList {
                register: held,
                count,
                spaced: spacing == 2,
                lane,
            },
            start,
            end,
        ));
        Parse::Success
    }

    fn arm32_parse_fp_immediate(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        mnemonic: &str,
    ) -> Parse {
        if !self.is(TokenKind::Hash) && !self.is(TokenKind::Dollar) {
            return Parse::NoMatch;
        }
        let start = self.lexer.token.start;
        let floating = operands
            .iter()
            .any(|held| matches!(held.token_text(), ".f32" | ".f64" | ".f16"));
        let constant = mnemonic == "fconstd" || mnemonic == "fconsts";
        if !floating && !constant {
            return Parse::NoMatch;
        }
        self.lex();
        let negative = self.is(TokenKind::Minus);
        if negative {
            self.lex();
        }
        let location = self.location();
        let bits = if self.is(TokenKind::Real) && floating {
            let value = self.lexer.token.text.parse::<f32>().unwrap_or(0.0);
            let mut bits = u64::from(value.to_bits());
            bits ^= u64::from(negative) << 31;
            self.lex();
            bits as i64
        } else if self.is(TokenKind::Integer) && constant {
            let value = self.lexer.token.text.parse::<i64>().unwrap_or(0);
            self.lex();
            if !(0..=255).contains(&value) {
                self.error(location, "encoded floating point value out of range");
                return Parse::Failure;
            }
            let sign = (value >> 7) & 1;
            let exponent = (value >> 4) & 7;
            let mantissa = value & 15;
            let mut word = sign << 31;
            word |= i64::from(exponent & 4 == 0) << 30;
            word |= (if exponent & 4 != 0 { 0x1f } else { 0 }) << 25;
            word |= (exponent & 3) << 23;
            word |= mantissa << 19;
            word
        } else {
            self.error(location, "invalid floating point immediate");
            return Parse::Failure;
        };
        let end = self.lexer.token.start;
        operands.push(parsed(
            Operand::Immediate(Expression::Constant(bits)),
            start,
            end,
        ));
        Parse::Success
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
        if self.is(TokenKind::Hash) {
            self.lex();
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
        if let Expression::Symbol {
            name: symbol,
            specifier: Some(existing),
        } = &expression
        {
            return Ok(Expression::Symbol {
                name: symbol.clone(),
                specifier: Some(format!("{name}:{existing}")),
            });
        }
        if matches!(
            expression,
            Expression::Binary { .. } | Expression::Unary { .. }
        ) {
            return Ok(Expression::Prefixed {
                prefix: name,
                operand: Box::new(expression),
            });
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
        let mve = self.arm_features.contains("HasMVEIntegerOps");
        let extra = lower[head_end..]
            .split_once(' ')
            .map_or(&lower[head_end..], |(head, _)| head);
        let extra = extra
            .split('.')
            .nth(1)
            .map_or(String::new(), |piece| format!(".{piece}"));
        let split = split_mnemonic(&lower[..head_end], thumb, mve, &extra);
        let mnemonic = split.base.clone();
        let mut operands: Vec<ParsedOperand> = Vec::new();
        self.arm32_push_token(&mut operands, &mnemonic, name_start);
        if let Some(mask) = &split.it_mask {
            let width = mnemonic.len();
            let location = self.statement_location(name_start + width);
            if mask.len() > 3 {
                self.error(
                    location,
                    if mnemonic == "it" {
                        "too many conditions on IT instruction"
                    } else {
                        "too many conditions on VPT instruction"
                    },
                );
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
                name_start + width,
                name_start + width,
            ));
        }
        let can_vpt = is_vpt_predicable(&mnemonic, &extra, mve);
        let can_carry = accepts_carry(&mnemonic, thumb);
        let can_condition = accepts_condition(&mnemonic, thumb, &lower);
        let location = self.statement_location(name_start);
        if thumb
            && !self.arm_features.contains("FeatureThumb2")
            && split.condition != CONDITION_AL
            && mnemonic != "b"
        {
            self.error(location, "conditional execution not supported in Thumb1");
            return Err(());
        }
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
        if !can_vpt && split.vpt_code != 0 {
            self.error(
                location,
                &format!(
                    "instruction '{mnemonic}' is not VPT predicable, but VPT code T/E is specified"
                ),
            );
            return Err(());
        }
        if can_vpt
            && split.vpt_code != 0
            && !(mnemonic.starts_with("vcvt")
                && !matches!(mnemonic.as_str(), "vcvta" | "vcvtn" | "vcvtp" | "vcvtm"))
        {
            operands.push(parsed(
                Operand::VectorPredicate(split.vpt_code),
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
        let prefix = operands.len()
            - usize::from(split.it_mask.is_some() || split.interrupt_modification != 0);
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
        let mut mnemonic = mnemonic;
        if mve {
            let end = Self::arm32_mnemonic_operand_end(&operands);
            let omit = self.arm32_omit_vector_predicate(&mnemonic, &operands, end);
            let converting = mnemonic.starts_with("vcvt")
                && !matches!(mnemonic.as_str(), "vcvta" | "vcvtn" | "vcvtp" | "vcvtm");
            let less_than = 11;
            let not_equal = 1;
            let condition_at = operands
                .iter()
                .position(|held| matches!(operand_of(held), Some(Operand::Condition(_))));
            if !omit && mnemonic == "vmov" && split.condition == less_than && condition_at.is_some()
            {
                if let Some(at) = condition_at {
                    operands.remove(at);
                }
                let place = name_start + mnemonic.len() - 1 + usize::from(split.carry);
                operands.remove(0);
                operands.insert(0, parsed(Operand::VectorPredicate(0), place, place));
                operands.insert(0, ParsedOperand::token("vmovlt", name_start));
                mnemonic = "vmovlt".to_string();
            } else if !omit
                && mnemonic == "vcvt"
                && split.condition == not_equal
                && condition_at.is_some()
            {
                if let Some(at) = condition_at {
                    operands.remove(at);
                }
                let place = name_start + mnemonic.len() - 1 + usize::from(split.carry);
                operands.remove(0);
                operands.insert(0, parsed(Operand::VectorPredicate(2), place, place));
                operands.insert(0, ParsedOperand::token("vcvtn", name_start));
                mnemonic = "vcvtn".to_string();
            } else if !omit
                && mnemonic == "vmul"
                && split.condition == less_than
                && condition_at.is_some()
            {
                if let Some(at) = condition_at {
                    operands.remove(at);
                }
                operands.remove(0);
                operands.insert(0, ParsedOperand::token("vmullt", name_start));
                mnemonic = "vmullt".to_string();
            } else if converting {
                if !omit {
                    let at = name_start + mnemonic.len() + usize::from(split.carry);
                    operands.insert(1, parsed(Operand::VectorPredicate(split.vpt_code), at, at));
                }
            } else if can_vpt && omit {
                operands
                    .retain(|held| !matches!(operand_of(held), Some(Operand::VectorPredicate(_))));
            }
            if split.vpt_code != 0
                && !operands
                    .iter()
                    .any(|held| matches!(operand_of(held), Some(Operand::VectorPredicate(_))))
            {
                let joined = &lower[..mnemonic.len() + 1];
                mnemonic = joined.to_string();
                operands[0] = ParsedOperand::token(&mnemonic, name_start);
            }
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

    fn arm32_omit_vector_predicate(
        &self,
        mnemonic: &str,
        operands: &[ParsedOperand],
        end: usize,
    ) -> bool {
        if operands.len() <= end {
            return true;
        }
        if ["vld2", "vld4", "vst2", "vst4"]
            .iter()
            .any(|prefix| mnemonic.starts_with(prefix))
        {
            return true;
        }
        if mnemonic.starts_with("vctp") || mnemonic.starts_with("vpnot") {
            return false;
        }
        let target = self.target;
        let vector =
            |held: &ParsedOperand| matches!(operand_of(held), Some(Operand::VectorIndex(_)));
        let register_in = |held: &ParsedOperand, classes: &[&str]| match operand_of(held) {
            Some(Operand::Register(register)) => classes
                .iter()
                .any(|class| in_class(target, class, *register)),
            _ => false,
        };
        if mnemonic.starts_with("vmov")
            && !(mnemonic.starts_with("vmovl")
                || mnemonic.starts_with("vmovn")
                || mnemonic.starts_with("vmovx"))
        {
            return operands
                .iter()
                .any(|held| vector(held) || register_in(held, &["SPR", "DPR"]));
        }
        !operands
            .iter()
            .any(|held| vector(held) || register_in(held, &["MQPR", "QPR"]))
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
                "addVPTPredNOperands" | "addVPTPredROperands" => {
                    immediate(into, 0);
                    for _ in 1..count {
                        into.push(nothing.clone());
                    }
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
            ("addVPTPredNOperands" | "addVPTPredROperands", Operand::VectorPredicate(code)) => {
                immediate(into, i64::from(*code));
                into.push(if *code == 0 {
                    nothing.clone()
                } else {
                    MachineOperand::Register(
                        register_named(self.target, "p0").unwrap_or(NO_REGISTER),
                    )
                });
                into.push(nothing.clone());
                if count == 4 {
                    into.push(if *code == 0 {
                        nothing.clone()
                    } else {
                        match into.first() {
                            Some(MachineOperand::Register(held)) => MachineOperand::Register(*held),
                            _ => nothing.clone(),
                        }
                    });
                }
            }
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
                "addImmOperands"
                | "addPowerTwoOperands"
                | "addARMBranchTargetOperands"
                | "addThumbBranchTargetOperands",
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
            (
                "addMVEVecListOperands",
                Operand::VectorList {
                    register, count, ..
                },
            ) => {
                let first = *register;
                let pair = |low: RegisterId, high: RegisterId| {
                    self.target
                        .registers
                        .iter()
                        .position(|held| held.sub_registers == [low, high])
                        .map(|index| index as RegisterId)
                };
                let number = register_encoding(self.target, first);
                let at =
                    |offset: u32| register_named(self.target, &format!("q{}", number + offset));
                let joined = if *count == 2 {
                    at(1).and_then(|next| pair(first, next))
                } else {
                    let high = at(2).zip(at(3)).and_then(|(low, next)| pair(low, next));
                    at(1)
                        .and_then(|next| pair(first, next))
                        .zip(high)
                        .and_then(|(low, high)| pair(low, high))
                };
                into.push(MachineOperand::Register(joined.unwrap_or(NO_REGISTER)));
            }
            (
                "addMVEVectorIndexOperands" | "addMVEPairVectorIndexOperands",
                Operand::VectorIndex(index),
            ) => immediate(into, i64::from(*index)),
            ("addComplexRotationEvenOperands", Operand::Immediate(Expression::Constant(value))) => {
                immediate(into, *value / 90)
            }
            ("addComplexRotationOddOperands", Operand::Immediate(Expression::Constant(value))) => {
                immediate(into, (*value - 90) / 180)
            }
            ("addMemRegRQOffsetOperands", Operand::Memory(memory)) => {
                into.push(MachineOperand::Register(memory.base));
                into.push(MachineOperand::Register(memory.offset_register));
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
            (
                "addRegListOperands"
                | "addRegListWithAPSROperands"
                | "addDPRRegListOperands"
                | "addSPRRegListOperands"
                | "addFPSRegListWithVPROperands"
                | "addFPDRegListWithVPROperands"
                | "addDPR_VFP2RegListOperands"
                | "addDPR_8RegListOperands",
                Operand::RegisterList(list),
            ) => {
                for register in list {
                    into.push(MachineOperand::Register(*register));
                }
            }
            ("addConstPoolAsmImmOperands", Operand::ConstantPool(held)) => expression(into, held),
            ("addITMaskOperands", Operand::ItMask(mask)) => immediate(into, i64::from(*mask)),
            (
                "addCoprocNumOperands"
                | "addCoprocRegOperands"
                | "addCoprocOptionOperands"
                | "addMSRMaskOperands"
                | "addBankedRegOperands"
                | "addProcIFlagsOperands",
                Operand::CoprocessorNumber(value)
                | Operand::ProcessorFlags(value)
                | Operand::CoprocessorRegister(value)
                | Operand::CoprocessorOption(value)
                | Operand::SystemMask(value)
                | Operand::BankedRegister(value),
            ) => immediate(into, i64::from(*value)),
            ("addITCondCodeOperands", Operand::Condition(code)) => {
                immediate(into, i64::from(*code))
            }
            ("addMveSaturateOperands", _) => {
                immediate(into, i64::from(operand.constant().unwrap_or(64) == 48))
            }
            ("addITCondCodeInvOperands", Operand::Condition(code)) => {
                immediate(into, i64::from(*code ^ 1))
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
            ("addTraceSyncBarrierOptOperands", _) => immediate(into, 0),
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
            ("addVecListOperands", Operand::Register(held)) => {
                into.push(MachineOperand::Register(*held))
            }
            ("addVecListOperands", Operand::VectorList { register, .. }) => {
                into.push(MachineOperand::Register(*register))
            }
            ("addVecListIndexedOperands", Operand::VectorList { register, lane, .. }) => {
                into.push(MachineOperand::Register(*register));
                immediate(
                    into,
                    match lane {
                        VectorLane::Index(index) => i64::from(*index),
                        _ => 0,
                    },
                );
            }
            (
                "addVectorIndex8Operands"
                | "addVectorIndex16Operands"
                | "addVectorIndex32Operands"
                | "addVectorIndex64Operands",
                Operand::VectorIndex(index),
            ) => immediate(into, i64::from(*index)),
            (method_name, Operand::Memory(memory))
                if method_name.starts_with("addAlignedMemory")
                    || method_name.starts_with("addDupAlignedMemory") =>
            {
                into.push(MachineOperand::Register(memory.base));
                immediate(into, i64::from(memory.alignment));
            }
            ("addFPImmOperands", _) => immediate(
                into,
                i64::from(
                    floating_point_immediate(operand.constant().unwrap_or(0) as u32).unwrap_or(0),
                ),
            ),
            ("addNEONi8splatOperands", _) => {
                immediate(into, operand.constant().unwrap_or(0) | 0xe00)
            }
            ("addNEONi16splatOperands", _) => immediate(
                into,
                i64::from(encode_neon_i16_splat(operand.constant().unwrap_or(0) as u32)),
            ),
            ("addNEONi16splatNotOperands", _) => immediate(
                into,
                i64::from(encode_neon_i16_splat(
                    !(operand.constant().unwrap_or(0) as u32) & 0xffff,
                )),
            ),
            ("addNEONi32splatOperands", _) => immediate(
                into,
                i64::from(encode_neon_i32_splat(operand.constant().unwrap_or(0) as u32)),
            ),
            ("addNEONi32splatNotOperands", _) => immediate(
                into,
                i64::from(encode_neon_i32_splat(
                    !(operand.constant().unwrap_or(0) as u32),
                )),
            ),
            ("addNEONi32vmovOperands", _) => immediate(
                into,
                i64::from(encode_neon_vmov_immediate(
                    operand.constant().unwrap_or(0) as u32
                )),
            ),
            ("addNEONi32vmovNegOperands", _) => immediate(
                into,
                i64::from(encode_neon_vmov_immediate(
                    !(operand.constant().unwrap_or(0) as u32),
                )),
            ),
            ("addNEONvmovi8ReplicateOperands", _) => {
                immediate(into, (operand.constant().unwrap_or(0) & 0xff) | 0xe00)
            }
            ("addNEONinvi8ReplicateOperands", _) => {
                immediate(into, (!operand.constant().unwrap_or(0) & 0xff) | 0xe00)
            }
            ("addNEONvmovi16ReplicateOperands", _) => {
                let element = (operand.constant().unwrap_or(0) & 0xffff) as u32;
                immediate(
                    into,
                    i64::from(if element >= 256 {
                        (element >> 8) | 0x200
                    } else {
                        element
                    }),
                )
            }
            ("addNEONvmovi32ReplicateOperands", _) => immediate(
                into,
                i64::from(encode_neon_vmov_immediate(
                    (operand.constant().unwrap_or(0) & 0xffff_ffff) as u32,
                )),
            ),
            ("addNEONi64splatOperands", _) => {
                let mut value = operand.constant().unwrap_or(0) as u64;
                let mut bits = 0i64;
                for index in 0..8 {
                    bits |= ((value & 1) as i64) << index;
                    value >>= 8;
                }
                immediate(into, bits | 0x1e00)
            }
            ("addAddrMode5Operands" | "addAddrMode5FP16Operands", Operand::Immediate(held)) => {
                expression(into, held);
                immediate(into, 0);
            }
            ("addAddrMode5Operands" | "addAddrMode5FP16Operands", Operand::Memory(memory)) => {
                let divisor = if name == "addAddrMode5Operands" { 4 } else { 2 };
                let mut value = match &memory.offset_immediate {
                    Some(Expression::Constant(value)) => *value / divisor,
                    _ => 0,
                };
                let subtract = value < 0;
                if value == i64::from(i32::MIN) / divisor || value == i64::from(i32::MIN) {
                    value = 0;
                }
                if value < 0 {
                    value = -value;
                }
                into.push(MachineOperand::Register(memory.base));
                immediate(into, value | (i64::from(subtract) << 8));
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

    fn arm32_diagnostic_type(&self, class: usize) -> Option<String> {
        let held = &self.matcher.classes[class];
        if !held.diagnostic.is_empty() {
            Some(held.diagnostic.clone())
        } else if !held.diagnostic_string.is_empty() {
            Some(held.name.clone())
        } else {
            None
        }
    }

    fn arm32_diagnostic_message(&self, diagnostic: &str) -> Option<String> {
        let version8 = self.arm_features.contains("HasV8Ops");
        let thirty_two = self
            .arm32_feature("d32")
            .is_some_and(|held| self.arm_features.contains(&held));
        match diagnostic {
            "rGPR" => {
                return Some(
                    if version8 {
                        "operand must be a register in range [r0, r14]"
                    } else {
                        "operand must be a register in range [r0, r12] or r14"
                    }
                    .to_string(),
                )
            }
            "DPR" => {
                return Some(
                    if thirty_two {
                        "operand must be a register in range [d0, d31]"
                    } else {
                        "operand must be a register in range [d0, d15]"
                    }
                    .to_string(),
                )
            }
            "DPR_RegList" => {
                return Some(
                    if thirty_two {
                        "operand must be a list of registers in range [d0, d31]"
                    } else {
                        "operand must be a list of registers in range [d0, d15]"
                    }
                    .to_string(),
                )
            }
            _ => {}
        }
        self.matcher
            .classes
            .iter()
            .find(|held| {
                !held.diagnostic_string.is_empty()
                    && (held.diagnostic == diagnostic
                        || (held.diagnostic.is_empty() && held.name == diagnostic))
            })
            .map(|held| held.diagnostic_string.clone())
    }

    fn arm32_validate(&self, operand: &ParsedOperand, class: usize) -> Result<(), Option<String>> {
        let matcher = self.matcher;
        let held = &matcher.classes[class];
        let failure = || Err(self.arm32_diagnostic_type(class));
        match held.kind {
            ClassKind::Token => {
                if let ParsedKind::Token(text) = &operand.kind {
                    if let Some(found) = matcher.classes.iter().position(|candidate| {
                        candidate.kind == ClassKind::Token
                            && candidate.value_name.eq_ignore_ascii_case(text)
                    }) {
                        if matcher.is_subclass(found, class) {
                            return Ok(());
                        }
                    }
                }
                if let Some(Operand::Register(register)) = operand_of(operand) {
                    if register_named(self.target, &held.value_name.to_ascii_lowercase())
                        == Some(*register)
                    {
                        return Ok(());
                    }
                }
                if let Some(literal) = held.value_name.strip_prefix('#') {
                    if let (Some(Operand::Immediate(Expression::Constant(value))), Ok(wanted)) =
                        (operand_of(operand), literal.parse::<i64>())
                    {
                        if *value == wanted {
                            return Ok(());
                        }
                    }
                }
                Err(None)
            }
            ClassKind::Register => match operand_of(operand) {
                Some(Operand::Register(register)) if held.registers.contains(register) => Ok(()),
                Some(Operand::Register(register))
                    if held.diagnostic == "rGPR"
                        && self.arm_features.contains("HasV8Ops")
                        && self.target.register_by_name.get("sp") == Some(register) =>
                {
                    Ok(())
                }
                _ => failure(),
            },
            ClassKind::User => match operand_of(operand) {
                Some(Operand::Immediate(expression))
                    if held.name == "ModImm" && !matches!(expression, Expression::Constant(_)) =>
                {
                    Ok(())
                }
                Some(held_operand)
                    if predicate(
                        self.target,
                        &self.arm32_class_predicate(class),
                        held_operand,
                        self.thumb_state(),
                    ) =>
                {
                    Ok(())
                }
                _ => failure(),
            },
        }
    }

    fn arm32_match(&self, operands: &[ParsedOperand]) -> Outcome {
        let matcher = self.matcher;
        let target = self.target;
        let mnemonic = operands
            .first()
            .map(|operand| operand.token_text().to_ascii_lowercase())
            .unwrap_or_default();
        let Some((start, end)) = matcher.ranges.get(&(0, mnemonic.clone())).copied() else {
            return Outcome::MnemonicFail;
        };
        let mut misses: Vec<NearMiss> = Vec::new();
        let mut fallback: Vec<NearMiss> = Vec::new();
        for at in start..end {
            let entry = &matcher.matchables[at];
            if entry.mnemonic.to_ascii_lowercase() != mnemonic {
                continue;
            }
            let mut predicates = target.instructions[entry.result].predicates.clone();
            if let Some(alias) = entry.alias {
                predicates = target.aliases[alias].predicates.clone();
            }
            if mnemonic == "nop"
                && target.instructions[entry.result].record == "tMOVr"
                && (self.arm32_thumb_two() || self.arm_features.contains("HasV6MOps"))
            {
                continue;
            }
            let mut required: Vec<String> = predicates
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
            required.sort();
            required.dedup();
            let mut actual_of: Vec<Option<usize>> = Vec::new();
            let mut actual = 1;
            let mut operand_miss: Option<NearMiss> = None;
            let mut multiple = false;
            for formal in 0..entry.operands.len() {
                let class = entry.operands[formal].class;
                if actual >= operands.len() {
                    actual_of.push(None);
                    if matcher.classes[class].optional {
                        continue;
                    }
                    match operand_miss {
                        None => operand_miss = Some(NearMiss::TooFewOperands),
                        Some(NearMiss::TooFewOperands) => {}
                        Some(_) => {
                            multiple = true;
                            break;
                        }
                    }
                    continue;
                }
                match self.arm32_validate(&operands[actual], class) {
                    Ok(()) => {
                        actual_of.push(Some(actual));
                        actual += 1;
                    }
                    Err(diagnostic) => {
                        if diagnostic.is_none() && matcher.classes[class].optional {
                            actual_of.push(None);
                            continue;
                        }
                        if operand_miss.is_some() {
                            multiple = true;
                            break;
                        }
                        operand_miss = Some(NearMiss::Operand {
                            message: diagnostic
                                .and_then(|name| self.arm32_diagnostic_message(&name)),
                            class: Some(class),
                            index: actual,
                        });
                        actual_of.push(None);
                        actual += 1;
                    }
                }
            }
            if !multiple {
                while actual < operands.len() {
                    if operand_miss.is_some() {
                        multiple = true;
                        break;
                    }
                    operand_miss = Some(NearMiss::Operand {
                        message: None,
                        class: None,
                        index: actual,
                    });
                    actual += 1;
                }
            }
            if multiple {
                if let Some(miss) = operand_miss {
                    fallback.push(miss);
                }
                continue;
            }
            if let Some(miss) = operand_miss {
                if required.is_empty() {
                    misses.push(miss);
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
                            "VPTPredN" => "addVPTPredNOperands".to_string(),
                            "VPTPredR" => "addVPTPredROperands".to_string(),
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
            let late = self.arm32_target_predicate(&instruction);
            match (required.is_empty(), late) {
                (true, None) => return Outcome::Success(instruction),
                (false, None) => misses.push(NearMiss::Feature(required)),
                (true, Some(error)) => misses.push(NearMiss::Predicate(error)),
                (false, Some(_)) => {}
            }
        }
        if misses.is_empty() {
            misses = fallback;
        }
        Outcome::NearMisses(misses)
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
        self.streamer.set_arm_thumb(thumb);
        if thumb {
            self.arm32_set_features(&["thumb-mode"], &[]);
        } else {
            self.arm32_set_features(&[], &["thumb-mode"]);
        }
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
                let thumb = identifier == ".thumb";
                if thumb && !self.arm32_has_thumb() {
                    return Some(self.error(location, "target does not support Thumb mode"));
                }
                if !thumb && !self.arm32_has_arm() {
                    return Some(self.error(location, "target does not support ARM mode"));
                }
                self.arm_thumb = thumb;
                self.arm32_switch_mode();
                self.streamer.emit_arm_directive(if self.arm_thumb {
                    "\t.code\t16"
                } else {
                    "\t.code\t32"
                });
                if self.check_for_valid_section() {
                    return Some(true);
                }
                let mode = self.mode;
                self.streamer.emit_code_alignment(
                    &mut self.context,
                    if thumb { 2 } else { 4 },
                    0,
                    mode,
                );
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
            ".thumb_func" => {
                if self.parse_end_of_line() {
                    return Some(true);
                }
                if !self.arm_thumb {
                    self.arm_thumb = true;
                    self.arm32_switch_mode();
                }
                self.streamer.emit_arm_directive("\t.code\t16");
                self.arm_next_thumb = true;
                Some(false)
            }
            ".thumb_set" => Some(self.arm32_directive_thumb_set()),
            ".inst" | ".inst.n" | ".inst.w" => {
                let suffix = match identifier {
                    ".inst.n" => 'n',
                    ".inst.w" => 'w',
                    _ => '\0',
                };
                Some(self.arm32_directive_inst(location, suffix))
            }
            ".even" => {
                if self.parse_end_of_line() {
                    return Some(true);
                }
                if self.check_for_valid_section() {
                    return Some(true);
                }
                if self.current_section_is_code() {
                    let mode = self.mode;
                    self.streamer
                        .emit_code_alignment(&mut self.context, 2, 0, mode);
                } else {
                    self.streamer
                        .emit_value_to_alignment(&mut self.context, 2, 0, 1, 0);
                }
                Some(false)
            }
            ".unreq" => {
                if !self.is(TokenKind::Identifier) {
                    return Some(self.error(location, "unexpected input in .unreq directive."));
                }
                let name = self.lexer.token.text.to_ascii_lowercase();
                self.arm_aliases.remove(&name);
                self.lex();
                Some(self.parse_end_of_line())
            }
            ".fnstart" => Some(self.arm32_directive_fnstart(location)),
            ".fnend" => Some(self.arm32_directive_fnend(location)),
            ".cantunwind" => Some(self.arm32_directive_cantunwind(location)),
            ".handlerdata" => Some(self.arm32_directive_handlerdata(location)),
            ".personality" => Some(self.arm32_directive_personality(location)),
            ".personalityindex" => Some(self.arm32_directive_personality_index(location)),
            ".save" => Some(self.arm32_directive_register_save(location, false)),
            ".vsave" => Some(self.arm32_directive_register_save(location, true)),
            ".setfp" => Some(self.arm32_directive_setfp(location)),
            ".pad" => Some(self.arm32_directive_pad(location)),
            ".movsp" => Some(self.arm32_directive_movsp(location)),
            ".unwind_raw" => Some(self.arm32_directive_unwind_raw(location)),
            ".tlsdescseq" => {
                if !self.is(TokenKind::Identifier) {
                    return Some(
                        self.token_error("expected variable after '.tlsdescseq' directive"),
                    );
                }
                let name = self.lexer.token.text.clone();
                self.lex();
                if self.parse_end_of_line() {
                    return Some(true);
                }
                self.streamer
                    .emit_arm_directive(&format!("\t.tlsdescseq\t{name}"));
                let expression = Expression::Symbol {
                    name,
                    specifier: Some("tlsdescseq".to_string()),
                };
                self.streamer
                    .emit_arm_marker_fixup(&mut self.context, &expression, None, location);
                Some(false)
            }
            ".eabi_attribute" => Some(self.arm32_directive_eabi_attribute()),
            ".object_arch" => Some(self.arm32_directive_object_arch()),
            ".arch_extension" => Some(self.arm32_directive_arch_extension()),
            ".arch" => Some(self.arm32_directive_arch(location)),
            ".cpu" => Some(self.arm32_directive_cpu(location)),
            ".fpu" => Some(self.arm32_directive_fpu()),
            ".hword" => Some(self.parse_directive_value(2)),
            ".word" => Some(self.parse_directive_value(4)),
            ".align" => Some(self.parse_directive_align(true, 1)),
            ".ltorg" | ".pool" => {
                if self.parse_end_of_line() {
                    return Some(true);
                }
                if let Some((section, _)) = self.context.current_section() {
                    self.arm_pool_emit(section);
                }
                Some(false)
            }
            _ => {
                let _ = location;
                None
            }
        }
    }

    fn arm32_constant_pool(
        &mut self,
        instruction: &mut MachineInstruction,
        location: SourceLocation,
    ) -> bool {
        let target = self.target;
        let kind = match target.instructions[instruction.opcode].record.as_str() {
            "LDRConstPool" => 0,
            "tLDRConstPool" => 1,
            "t2LDRConstPool" => 2,
            _ => return false,
        };
        let value = instruction.operand(1).clone();
        let register = instruction.register_at(0);
        let stack = register_named(target, "sp").unwrap_or(NO_REGISTER);
        let counter = register_named(target, "pc").unwrap_or(NO_REGISTER);
        let condition = instruction.operand(2).clone();
        let condition_register = instruction.operand(3).clone();
        let feature = |name: &str| self.arm_features.contains(name);
        if let MachineOperand::Immediate(constant) = &value {
            if register != counter && register != stack {
                let constant = *constant;
                let mut chosen: Option<(&str, i64, bool)> = None;
                if kind == 0 {
                    if let Some(encoded) = so_immediate(constant as u32) {
                        chosen = Some(("MOVi", i64::from(encoded), true));
                    } else if let Some(encoded) = so_immediate(!(constant as u32)) {
                        chosen = Some(("MVNi", i64::from(encoded), true));
                    } else if feature("HasV6T2Ops") && (0..65536).contains(&constant) {
                        chosen = Some(("MOVi16", constant, false));
                    }
                } else {
                    let thumb_two = self.arm32_thumb_two();
                    if thumb_two && thumb_so_immediate(constant as u32).is_some() {
                        chosen = Some(("t2MOVi", constant, true));
                    } else if thumb_two && thumb_so_immediate(!(constant as u32)).is_some() {
                        chosen = Some(("t2MVNi", i64::from(!(constant as u32)), true));
                    } else if feature("HasV8MBaselineOps") && (0..65536).contains(&constant) {
                        chosen = Some(("t2MOVi16", constant, false));
                    }
                }
                if let Some((name, immediate, sets_flags)) = chosen {
                    if let Some(opcode) = self.arm32_record_index(name) {
                        let mut replaced = MachineInstruction::new(opcode);
                        replaced.operands = vec![
                            MachineOperand::Register(register),
                            MachineOperand::Immediate(immediate),
                            condition,
                            condition_register,
                        ];
                        if sets_flags {
                            replaced
                                .operands
                                .push(MachineOperand::Register(NO_REGISTER));
                        }
                        replaced.flags = instruction.flags;
                        *instruction = replaced;
                        return false;
                    }
                }
            }
        }
        let expression = match value {
            MachineOperand::Immediate(constant) => Expression::Constant(constant),
            MachineOperand::Expression(held) => held,
            MachineOperand::Register(_) => return false,
        };
        let label = self.arm_pool_add(expression, 4, location);
        let name = self.context.symbols[label].name.clone();
        let name_record = match kind {
            0 => "LDRi12",
            1 => "tLDRpci",
            _ => "t2LDRpci",
        };
        let Some(opcode) = self.arm32_record_index(name_record) else {
            return false;
        };
        let mut replaced = MachineInstruction::new(opcode);
        replaced.operands = vec![
            MachineOperand::Register(register),
            MachineOperand::Expression(Expression::symbol(&name)),
        ];
        if kind == 0 {
            replaced.operands.push(MachineOperand::Immediate(0));
        }
        replaced.operands.push(condition);
        replaced.operands.push(condition_register);
        replaced.flags = instruction.flags;
        *instruction = replaced;
        false
    }

    fn arm32_has_thumb(&self) -> bool {
        self.arm32_feature("v4t")
            .is_some_and(|held| self.arm_features.contains(&held))
    }

    fn arm32_has_arm(&self) -> bool {
        self.arm32_feature("noarm")
            .is_none_or(|held| !self.arm_features.contains(&held))
    }

    fn arm32_start_it_block(&mut self, instruction: &MachineInstruction) {
        let record = self.target.instructions[instruction.opcode].record.as_str();
        if record == "MVE_VPST" || record.starts_with("MVE_VPTv") {
            self.arm_vpt = ItState {
                condition: 0,
                mask: instruction.immediate_at(0) as u32,
                position: 0,
                explicit: true,
            };
        }
        if record == "ITasm" || record == "t2IT" {
            self.arm_it = ItState {
                condition: instruction.immediate_at(0) as u32,
                mask: instruction.immediate_at(1) as u32,
                position: 0,
                explicit: true,
            };
        }
    }

    fn arm32_thumb_two(&self) -> bool {
        self.arm_thumb && self.arm_features.contains("FeatureThumb2")
    }

    fn arm32_target_predicate(&self, instruction: &MachineInstruction) -> Option<PredicateError> {
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
        let thumb_one = self.arm_thumb && !thumb_two;
        let in_it = self.arm_it.active();
        let version8 = self.arm_features.contains("HasV8Ops");
        let register = |name: &str| {
            target
                .register_by_name
                .get(name)
                .copied()
                .unwrap_or(NO_REGISTER)
        };
        let (stack, counter) = (register("sp"), register("pc"));
        let register_at = |index: usize| match instruction.operand(index) {
            MachineOperand::Register(held) => *held,
            _ => NO_REGISTER,
        };
        let low = |held: RegisterId| in_class(target, "tGPR", held);
        if flag_setting {
            let mut carry = false;
            for group in &description.groups {
                if group.record == "s_cc_out" {
                    carry = instruction.register_at(group.first) != NO_REGISTER;
                }
            }
            if thumb_one && !carry {
                return Some(PredicateError::FlagSetting);
            }
            if thumb_two && !carry && !in_it {
                return Some(PredicateError::ITBlock);
            }
            if thumb_two && carry && in_it {
                return Some(PredicateError::NotITBlock);
            }
            if description.record == "tLSLri" && instruction.immediate_at(3) == 0 && in_it {
                return Some(PredicateError::NotITBlock);
            }
        } else if thumb_one {
            if description.record == "tADDhirr"
                && !self.arm_features.contains("HasV6MOps")
                && low(register_at(1))
                && low(register_at(2))
            {
                return Some(PredicateError::Thumb2);
            } else if description.record == "tMOVr"
                && !self.arm_features.contains("HasV6Ops")
                && low(register_at(0))
                && low(register_at(1))
            {
                return Some(PredicateError::V6);
            }
        }
        let cpsr = register("cpsr");
        if description.record == "t2MOVr" && !version8 {
            if register_at(0) == stack && register_at(1) == stack {
                return Some(PredicateError::V8);
            }
            if register_at(4) == cpsr && (register_at(0) == stack || register_at(1) == stack) {
                return Some(PredicateError::V8);
            }
        }
        match description.record.as_str() {
            "VMRS" | "VMSR" | "VMRS_FPCXTS" | "VMRS_FPCXTNS" | "VMSR_FPCXTS" | "VMSR_FPCXTNS"
            | "VMRS_FPSCR_NZCVQC" | "VMSR_FPSCR_NZCVQC" | "FMSTAT" | "VMRS_VPR" | "VMRS_P0"
            | "VMSR_VPR" | "VMSR_P0" => {
                if register_at(0) == stack && self.arm_thumb && !version8 {
                    return Some(PredicateError::InvalidOperand);
                }
            }
            "t2TBB" | "t2TBH" => {
                if !version8 && register_at(0) == stack {
                    return Some(PredicateError::V8);
                }
            }
            "tMUL" if register_at(0) != register_at(3) => {
                return Some(PredicateError::TiedOperand);
            }
            _ => {}
        }
        for group in &description.groups {
            let parts: Vec<String> = if group.count == 1 {
                vec![group.record.clone()]
            } else {
                target
                    .operand_records
                    .get(&group.record)
                    .map(|held| held.sub_operands.clone())
                    .unwrap_or_default()
            };
            for (offset, part) in parts.iter().enumerate() {
                if part != "rGPR" {
                    continue;
                }
                if let MachineOperand::Register(held) = instruction.operand(group.first + offset) {
                    if *held == stack && !version8 {
                        return Some(PredicateError::V8);
                    }
                    if *held == counter {
                        return Some(PredicateError::InvalidOperand);
                    }
                }
            }
        }
        None
    }

    fn arm32_record_index(&self, record: &str) -> Option<usize> {
        self.target.instruction_by_record.get(record).copied()
    }

    fn arm32_process(&self, instruction: &mut MachineInstruction, wide: bool) -> bool {
        let target = self.target;
        let record = target.instructions[instruction.opcode].record.as_str();
        let operand = |index: usize| instruction.operand(index).clone();
        let in_it = self.arm_it.active();
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
                    && operand_count == 3
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
            "t2MOVsi" | "t2MOVSsi" => {
                let flag_setting = record == "t2MOVSsi";
                let mut narrow = low(instruction.register_at(0))
                    && low(instruction.register_at(1))
                    && in_it == !flag_setting
                    && !wide;
                let packed = instruction.immediate_at(2) as u32;
                let shift = packed & 7;
                let mut amount = packed >> 3;
                let mut is_move = false;
                let name = if shift == 2 && amount == 0 {
                    is_move = true;
                    if in_it {
                        narrow = false;
                    }
                    if narrow {
                        "tMOVSr"
                    } else {
                        "t2MOVr"
                    }
                } else {
                    match shift {
                        1 => {
                            if narrow {
                                "tASRri"
                            } else {
                                "t2ASRri"
                            }
                        }
                        3 => {
                            if narrow {
                                "tLSRri"
                            } else {
                                "t2LSRri"
                            }
                        }
                        2 => {
                            if narrow {
                                "tLSLri"
                            } else {
                                "t2LSLri"
                            }
                        }
                        4 => {
                            narrow = false;
                            "t2RORri"
                        }
                        _ => {
                            narrow = false;
                            "t2RRX"
                        }
                    }
                };
                if amount == 32 {
                    amount = 0;
                }
                let status =
                    MachineOperand::Register(if flag_setting { cpsr } else { NO_REGISTER });
                let mut operands = vec![operand(0)];
                if narrow && !is_move {
                    operands.push(status.clone());
                }
                operands.push(operand(1));
                if name != "t2RRX" && !is_move {
                    operands.push(MachineOperand::Immediate(i64::from(amount)));
                }
                operands.push(operand(3));
                operands.push(operand(4));
                if !narrow {
                    operands.push(status);
                }
                build(name, operands)
            }
            "LDMIA_UPD" if first_token == "pop" && instruction.operands.len() == 5 => build(
                "LDR_POST_IMM",
                vec![
                    operand(4),
                    operand(0),
                    operand(1),
                    MachineOperand::Register(NO_REGISTER),
                    MachineOperand::Immediate(4),
                    operand(2),
                    operand(3),
                ],
            ),
            "STMDB_UPD" if first_token == "push" && instruction.operands.len() == 5 => build(
                "STR_PRE_IMM",
                vec![
                    operand(0),
                    operand(4),
                    operand(1),
                    MachineOperand::Immediate(-4),
                    operand(2),
                    operand(3),
                ],
            ),
            "t2LDMIA_UPD" if instruction.operands.len() == 5 => build(
                "t2LDR_POST",
                vec![
                    operand(4),
                    operand(0),
                    operand(1),
                    MachineOperand::Immediate(4),
                    operand(2),
                    operand(3),
                ],
            ),
            "t2STMDB_UPD" if instruction.operands.len() == 5 => build(
                "t2STR_PRE",
                vec![
                    operand(0),
                    operand(4),
                    operand(1),
                    MachineOperand::Immediate(-4),
                    operand(2),
                    operand(3),
                ],
            ),
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
            "LDRT_POST" | "LDRBT_POST" => build(
                if record == "LDRT_POST" {
                    "LDRT_POST_IMM"
                } else {
                    "LDRBT_POST_IMM"
                },
                vec![
                    operand(0),
                    operand(1),
                    operand(1),
                    MachineOperand::Register(NO_REGISTER),
                    MachineOperand::Immediate(0),
                    operand(2),
                    operand(3),
                ],
            ),
            "STRT_POST" | "STRBT_POST" => build(
                if record == "STRT_POST" {
                    "STRT_POST_IMM"
                } else {
                    "STRBT_POST_IMM"
                },
                vec![
                    operand(1),
                    operand(0),
                    operand(1),
                    MachineOperand::Register(NO_REGISTER),
                    MachineOperand::Immediate(0),
                    operand(2),
                    operand(3),
                ],
            ),
            "LDRSBTii" | "LDRHTii" | "LDRSHTii" => build(
                match record {
                    "LDRSBTii" => "LDRSBTi",
                    "LDRHTii" => "LDRHTi",
                    _ => "LDRSHTi",
                },
                vec![
                    operand(0),
                    operand(1),
                    operand(1),
                    MachineOperand::Immediate(256),
                    operand(2),
                ],
            ),
            "ADDri" => {
                let counter = register("pc");
                match (instruction.operand(2), instruction.register_at(5)) {
                    (MachineOperand::Immediate(encoded), NO_REGISTER)
                        if instruction.register_at(1) == counter =>
                    {
                        let encoded = *encoded as u32;
                        let value = (encoded & 0xff).rotate_right((encoded & 0xf00) >> 7);
                        build(
                            "ADR",
                            vec![
                                operand(0),
                                MachineOperand::Immediate(i64::from(value)),
                                operand(3),
                                operand(4),
                            ],
                        )
                    }
                    _ => None,
                }
            }
            "t2LDR_PRE_imm" | "t2LDR_POST_imm" | "t2LDRB_PRE_imm" | "t2LDRB_POST_imm"
            | "t2LDRH_PRE_imm" | "t2LDRH_POST_imm" | "t2LDRSB_PRE_imm" | "t2LDRSB_POST_imm"
            | "t2LDRSH_PRE_imm" | "t2LDRSH_POST_imm" => build(
                &record.replace("_imm", ""),
                vec![
                    operand(0),
                    operand(1),
                    operand(1),
                    operand(2),
                    operand(3),
                    operand(4),
                ],
            ),
            "t2STR_PRE_imm" | "t2STR_POST_imm" | "t2STRB_PRE_imm" | "t2STRB_POST_imm"
            | "t2STRH_PRE_imm" | "t2STRH_POST_imm" => build(
                &record.replace("_imm", ""),
                vec![
                    operand(1),
                    operand(0),
                    operand(1),
                    operand(2),
                    operand(3),
                    operand(4),
                ],
            ),
            "t2LDRB_OFFSET_imm" | "t2LDRH_OFFSET_imm" | "t2LDRSB_OFFSET_imm"
            | "t2LDRSH_OFFSET_imm" | "t2STRB_OFFSET_imm" | "t2STRH_OFFSET_imm" => build(
                &record.replace("_OFFSET_imm", "i8"),
                vec![operand(0), operand(1), operand(2), operand(3)],
            ),
            "t2LDRpcrel" => {
                let value = instruction.immediate_at(1);
                let name = if value > 0 && value <= 0xff && !wide {
                    "tLDRpci"
                } else {
                    "t2LDRpci"
                };
                build(name, instruction.operands.clone())
            }
            name if VECTOR_PSEUDOS.iter().any(|entry| entry.0 == name) => VECTOR_PSEUDOS
                .iter()
                .find(|entry| entry.0 == name)
                .and_then(|entry| {
                    let first = register_encoding(target, instruction.register_at(0)) as i64;
                    let mut operands = Vec::new();
                    for item in entry.3 {
                        match item {
                            Pseudo::Operand(index) => operands.push(operand(*index)),
                            Pseudo::None => operands.push(MachineOperand::Register(NO_REGISTER)),
                            Pseudo::Step(count) => {
                                let number = first + entry.2 * count;
                                let held = register_named(target, &format!("d{number}"))?;
                                operands.push(MachineOperand::Register(held));
                            }
                        }
                    }
                    build(entry.1, operands)
                }),
            "t2LDRBpcrel" | "t2LDRHpcrel" | "t2LDRSBpcrel" | "t2LDRSHpcrel" => build(
                &record.replace("pcrel", "pci"),
                instruction.operands.clone(),
            ),
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
        if self.is(TokenKind::Identifier) && self.lexer.token.text.eq_ignore_ascii_case(".req") {
            self.lex();
            let register_location = self.location();
            let Some(register) = self.arm32_try_register() else {
                return self.error(register_location, "register name expected");
            };
            if self.parse_end_of_line() {
                return true;
            }
            let lower = identifier.to_ascii_lowercase();
            match self.arm_aliases.get(&lower) {
                Some((_, existing)) if *existing != register => {
                    return self.error(
                        register_location,
                        &format!("redefinition of '{identifier}' does not match original."),
                    );
                }
                _ => {
                    self.arm_aliases
                        .insert(lower, (armv8::RegKind::Scalar, register));
                }
            }
            return false;
        }
        self.arm_suffixes.clear();
        let Ok((operands, _)) = self.arm32_parse_instruction(identifier, identifier_token.start)
        else {
            return true;
        };
        if self.has_pending_error() {
            return true;
        }
        if self.is(TokenKind::EndOfStatement) {
            self.lex();
        }
        let location = identifier_token.location();
        let mut operands = operands;
        if self.arm32_fix_pair_operands(&mut operands, location) {
            return true;
        }
        self.arm32_return_from_exception_form(&mut operands);
        self.arm32_two_operand_form(&mut operands);
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
                if !self.options.no_deprecated_warn {
                    if let Some(information) = self.arm32_deprecation(&instruction) {
                        self.warning(location, &information);
                    }
                }
                if self.arm32_constant_pool(&mut instruction, location) {
                    return true;
                }
                self.arm32_thumb_branches(&mut instruction);
                if self.arm32_validate_instruction(&instruction, &operands, location) {
                    self.arm_it.forward();
                    self.arm_vpt.forward_vector();
                    return true;
                }
                while self.arm32_process(&mut instruction, wide) {}
                self.arm32_start_it_block(&instruction);
                self.arm_it.forward();
                self.arm_vpt.forward_vector();
                if self.target.instructions[instruction.opcode].record == "ITasm" {
                    return false;
                }
                let locations = std::mem::take(&mut self.expression_locations);
                self.streamer.note_expression_locations(&locations);
                self.emit_instruction(&instruction, location);
                false
            }
            Outcome::MnemonicFail => {
                let base = operands
                    .first()
                    .map(|held| held.token_text().to_ascii_lowercase())
                    .unwrap_or_default();
                let suggestion = self.arm_mnemonic_suggestion(&base);
                self.error_range(
                    location,
                    &format!("invalid instruction{suggestion}"),
                    (location, self.statement_location(identifier_token.end)),
                )
            }
            Outcome::NearMisses(misses) => {
                self.arm32_report_near_misses(&misses, &operands, location)
            }
        }
    }

    fn arm32_deprecation(&self, instruction: &MachineInstruction) -> Option<String> {
        let record = self
            .target
            .records
            .get(&self.target.instructions[instruction.opcode].record)?;
        let feature = table_record_name(record, "DeprecatedFeatureMask");
        if !feature.is_empty() && self.arm_features.contains(&feature) {
            return Some("deprecated".to_string());
        }
        let immediate_is = |index: usize, wanted: i64| matches!(instruction.operand(index), MachineOperand::Immediate(held) if *held == wanted);
        let version7 = self.arm_features.contains("HasV7Ops");
        let reserved = |index: usize| immediate_is(index, 10) || immediate_is(index, 11);
        match table_text(record, "ComplexDeprecationPredicate").as_str() {
            "MCR" => {
                if version7 && immediate_is(0, 15) && immediate_is(1, 0) && immediate_is(3, 7) {
                    if immediate_is(5, 4) {
                        if immediate_is(4, 5) {
                            return Some("deprecated since v7, use 'isb'".to_string());
                        }
                        if immediate_is(4, 10) {
                            return Some("deprecated since v7, use 'dsb'".to_string());
                        }
                    }
                    if immediate_is(4, 10) && immediate_is(5, 5) {
                        return Some("deprecated since v7, use 'dmb'".to_string());
                    }
                }
                if version7 && reserved(0) {
                    return Some(
                        "since v7, cp10 and cp11 are reserved for advanced SIMD or floating point instructions"
                            .to_string(),
                    );
                }
                None
            }
            "MRC" => (version7 && reserved(1)).then(|| {
                "since v7, cp10 and cp11 are reserved for advanced SIMD or floating point instructions"
                    .to_string()
            }),
            "ARMStore" => {
                let counter = register_named(self.target, "pc");
                (4..instruction.operands.len())
                    .any(|index| Some(instruction.register_at(index)) == counter)
                    .then(|| "use of PC in the list is deprecated".to_string())
            }
            "ARMLoad" => {
                let counter = register_named(self.target, "pc");
                let link = register_named(self.target, "lr");
                let listed = |wanted: Option<RegisterId>| {
                    (4..instruction.operands.len())
                        .any(|index| Some(instruction.register_at(index)) == wanted)
                };
                (listed(counter) && listed(link))
                    .then(|| "use of LR and PC simultaneously in the list is deprecated".to_string())
            }
            _ => None,
        }
    }

    fn arm32_return_from_exception_form(&self, operands: &mut Vec<ParsedOperand>) {
        let end = Self::arm32_mnemonic_operand_end(operands);
        if !self.arm32_thumb_two() || operands.len() != end + 3 || operands[0].token_text() != "sub"
        {
            return;
        }
        let carry = operands[..end]
            .iter()
            .position(|held| matches!(operand_of(held), Some(Operand::CarrySet(_))));
        let Some(carry) = carry else {
            return;
        };
        let is_register = |held: &ParsedOperand, name: &str| {
            matches!(operand_of(held), Some(Operand::Register(register))
                if register_named(self.target, name) == Some(*register))
        };
        if is_register(&operands[end], "pc")
            && is_register(&operands[end + 1], "lr")
            && matches!(operand_of(&operands[end + 2]), Some(Operand::Immediate(_)))
        {
            let start = operands[0].start;
            operands[0] = ParsedOperand::token("subs", start);
            operands.remove(carry);
        }
    }

    fn arm32_thumb_branches(&self, instruction: &mut MachineInstruction) {
        let record = self.target.instructions[instruction.opcode].record.as_str();
        let (plain, conditional) = match record {
            "tB" | "tBcc" => ("tB", "tBcc"),
            "t2B" | "t2Bcc" => ("t2B", "t2Bcc"),
            _ => return,
        };
        let always = instruction.immediate_at(1) == 14;
        let wanted = if self.arm_it.active() || always {
            plain
        } else {
            conditional
        };
        if wanted != record {
            if let Some(index) = self.arm32_record_index(wanted) {
                instruction.opcode = index;
            }
        }
    }

    fn arm32_two_operand_form(&self, operands: &mut Vec<ParsedOperand>) {
        let end = Self::arm32_mnemonic_operand_end(operands);
        if operands[..end].iter().any(|held| held.token_text() == ".w") {
            return;
        }
        if operands.len() != end + 3 {
            return;
        }
        let register_of = |held: &ParsedOperand| match operand_of(held) {
            Some(Operand::Register(register)) => Some(*register),
            _ => None,
        };
        let (Some(first), Some(second)) =
            (register_of(&operands[end]), register_of(&operands[end + 1]))
        else {
            return;
        };
        let counter = register_named(self.target, "pc");
        let stack = register_named(self.target, "sp");
        let third = register_of(&operands[end + 2]);
        let third_is_immediate =
            matches!(operand_of(&operands[end + 2]), Some(Operand::Immediate(_)));
        let mnemonic = operands[0].token_text().to_ascii_lowercase();
        let carry = operands[..end]
            .iter()
            .any(|held| matches!(operand_of(held), Some(Operand::CarrySet(_))));
        if self.arm32_thumb_two() {
            if mnemonic != "add" {
                return;
            }
            let mut attempt = Some(first) == counter
                || Some(second) == counter
                || (third.is_some() && third == counter);
            if !attempt {
                let stacked = Some(first) == stack
                    || Some(second) == stack
                    || (third.is_some() && third == stack);
                let large = match operand_of(&operands[end + 2]) {
                    Some(Operand::Immediate(Expression::Constant(value))) => {
                        !((0..=508).contains(value) && value % 4 == 0)
                    }
                    Some(Operand::Immediate(_)) => true,
                    _ => false,
                };
                attempt = stacked
                    && !(Some(first) == stack
                        && Some(second) == stack
                        && third_is_immediate
                        && large);
            }
            if !attempt {
                return;
            }
        } else if !(self.arm_thumb && !self.arm_features.contains("FeatureThumb2")) {
            return;
        }
        if ![
            "add", "sub", "and", "eor", "lsl", "lsr", "asr", "adc", "sbc", "ror", "orr", "bic",
        ]
        .contains(&mnemonic.as_str())
        {
            return;
        }
        let mut transform = first == second;
        let mut swap = false;
        let mut last = end + 2;
        if !transform
            && third == Some(first)
            && ((mnemonic == "add" && Some(second) != stack)
                || matches!(mnemonic.as_str(), "and" | "eor" | "adc" | "orr"))
        {
            swap = true;
            last = end + 1;
            transform = true;
        }
        if transform {
            let last_is_register = register_of(&operands[last]).is_some();
            if ((mnemonic == "add" && carry) || mnemonic == "sub") && last_is_register {
                transform = false;
            }
            if (mnemonic == "add" || mnemonic == "sub")
                && matches!(
                    operand_of(&operands[last]),
                    Some(Operand::Immediate(Expression::Constant(value))) if (0..=7).contains(value)
                )
            {
                transform = false;
            }
        }
        if transform {
            if swap {
                operands.swap(end + 1, end + 2);
            }
            operands.remove(end);
        }
    }

    fn arm32_mnemonic_operand_end(operands: &[ParsedOperand]) -> usize {
        const DATA_TYPES: [&str; 22] = [
            ".8", ".16", ".32", ".64", ".i8", ".i16", ".i32", ".i64", ".u8", ".u16", ".u32",
            ".u64", ".s8", ".s16", ".s32", ".s64", ".p8", ".p16", ".f32", ".f64", ".f", ".d",
        ];
        let mut end = 1;
        if operands
            .first()
            .is_some_and(|held| held.token_text() == "cps")
        {
            if let Some(Operand::Immediate(Expression::Constant(value))) =
                operands.get(1).and_then(operand_of)
            {
                if *value == 2 || *value == 3 {
                    end += 1;
                }
            }
        }
        let mut right_hand_condition = false;
        while end < operands.len() {
            let held = &operands[end];
            let text = held.token_text();
            if matches!(operand_of(held), Some(Operand::ItMask(_)))
                || matches!(text, ".w" | ".bf16" | ".p64" | ".f16")
                || DATA_TYPES.contains(&text)
            {
                right_hand_condition = true;
                end += 1;
            } else if matches!(
                operand_of(held),
                Some(Operand::CarrySet(_) | Operand::VectorPredicate(_))
            ) || (matches!(operand_of(held), Some(Operand::Condition(_)))
                && !right_hand_condition)
            {
                end += 1;
            } else {
                break;
            }
        }
        end
    }

    fn arm32_signed_offset(operand: Option<&ParsedOperand>, width: u32, scale: u32) -> bool {
        match operand.and_then(operand_of) {
            Some(Operand::Immediate(Expression::Symbol { .. })) => true,
            Some(Operand::Immediate(Expression::Constant(value))) => {
                let align = 1i64 << scale;
                let maximum = align * ((1i64 << (width - 1)) - 1);
                let minimum = -align * (1i64 << (width - 1));
                value % align == 0 && *value >= minimum && *value <= maximum
            }
            _ => false,
        }
    }

    fn arm32_unsigned_offset(operand: Option<&ParsedOperand>, width: u32, scale: u32) -> bool {
        match operand.and_then(operand_of) {
            Some(Operand::Immediate(Expression::Symbol { .. })) => true,
            Some(Operand::Immediate(Expression::Constant(value))) => {
                let align = 1i64 << scale;
                let maximum = align * ((1i64 << width) - 1);
                value % align == 0 && *value >= 0 && *value <= maximum
            }
            _ => false,
        }
    }

    fn arm32_is_coprocessor_data_engine(&self, coprocessor: i64) -> bool {
        (0..8).contains(&coprocessor)
            && self
                .arm_features
                .contains(&format!("FeatureCoprocCDE{coprocessor}"))
    }

    fn arm32_validate_instruction(
        &mut self,
        instruction: &MachineInstruction,
        operands: &[ParsedOperand],
        location: SourceLocation,
    ) -> bool {
        let target = self.target;
        let description = &target.instructions[instruction.opcode];
        let record = description.record.as_str();
        let end = Self::arm32_mnemonic_operand_end(operands);
        let register = |name: &str| register_named(target, name).unwrap_or(NO_REGISTER);
        let (stack, counter, link) = (register("sp"), register("pc"), register("lr"));
        let encoding = |held: RegisterId| register_encoding(target, held);
        let version8 = self.arm_features.contains("HasV8Ops");
        let thumb_two = self.arm32_thumb_two();
        let predicate_group = description.groups.iter().find(|group| {
            target
                .records
                .get(&group.record)
                .is_some_and(|held| held.is_a("PredicateOp"))
        });
        let unpredicable = target
            .records
            .get(record)
            .is_some_and(|held| table_bit(held, "isUnpredicable"));
        let predicate_value = predicate_group.map(|group| instruction.immediate_at(group.first));
        let predicable_value = predicate_value.filter(|_| !unpredicable);
        let vector_group = description.groups.iter().find(|group| {
            target
                .operand_records
                .get(&group.record)
                .is_some_and(|held| held.operand_type.starts_with("OPERAND_VPRED"))
        });
        let breakpoint = matches!(record, "tBKPT" | "BKPT" | "tHLT" | "HLT");
        let positions: Vec<usize> = operands.iter().map(|held| held.start).collect();
        let ends: Vec<usize> = operands.iter().map(|held| held.end).collect();
        let statement = self.statement_start.buffer;
        let spot = move |index: usize| -> SourceLocation {
            SourceLocation {
                buffer: statement,
                position: positions.get(index).copied().unwrap_or(0),
            }
        };
        if self.arm_it.active() && !breakpoint {
            let Some(condition) = predicable_value else {
                return self.error(location, "instructions in IT block must be predicable");
            };
            let expected = self.arm_it.current();
            if condition as u32 != expected {
                let condition_location = operands
                    .iter()
                    .enumerate()
                    .skip(1)
                    .find(|(_, held)| matches!(operand_of(held), Some(Operand::Condition(_))))
                    .map_or(location, |(index, _)| spot(index));
                return self.error(
                    condition_location,
                    &format!(
                        "incorrect condition in IT block; got '{}', but expected '{}'",
                        CONDITION_NAMES
                            .get(condition as usize)
                            .copied()
                            .unwrap_or("al"),
                        CONDITION_NAMES
                            .get(expected as usize)
                            .copied()
                            .unwrap_or("al")
                    ),
                );
            }
        } else if thumb_two
            && predicate_value.is_some_and(|value| value != 14)
            && !matches!(record, "tBcc" | "t2Bcc" | "t2BFic" | "tB" | "t2B")
        {
            return self.error(location, "predicated instructions must be in IT block");
        } else if unpredicable && predicate_value.is_some_and(|value| value != 14) {
            return self.error(location, "instruction is not predicable");
        }
        if self.arm_it.explicit_block() && !self.arm_it.last() {
            let held = target.records.get(record);
            let terminator = held.is_some_and(|held| {
                table_bit(held, "isTerminator")
                    || (table_bit(held, "isCall") && record != "tSVC")
                    || table_bit(held, "isReturn")
                    || table_bit(held, "isBranch")
                    || table_bit(held, "isIndirectBranch")
            }) || description.groups.iter().any(|group| {
                group.output
                    && (0..group.count)
                        .any(|offset| instruction.register_at(group.first + offset) == counter)
            }) || (held.is_some_and(|held| table_bit(held, "variadicOpsAreDefs"))
                && (description
                    .groups
                    .iter()
                    .map(|group| group.first + group.count)
                    .max()
                    .unwrap_or(1)
                    - 1..instruction.operands.len())
                    .any(|index| instruction.register_at(index) == counter));
            if terminator {
                return self.error(
                    location,
                    "instruction must be outside of IT block or the last instruction in an IT block",
                );
            }
        }
        if self.arm_vpt.active() && !breakpoint {
            let Some(group) = vector_group else {
                return self.error(location, "instruction in VPT block must be predicable");
            };
            let bit = (self.arm_vpt.mask >> (5 - self.arm_vpt.position.min(5))) & 1;
            let expected = if bit == 1 { 2 } else { 1 };
            let found = instruction.immediate_at(group.first);
            if found != expected {
                let names = ["none", "t", "e"];
                let predicate_location = operands
                    .iter()
                    .enumerate()
                    .skip(1)
                    .find(|(_, held)| matches!(operand_of(held), Some(Operand::VectorPredicate(_))))
                    .map_or(location, |(index, _)| spot(index));
                return self.error(
                    predicate_location,
                    &format!(
                        "incorrect predication in VPT block; got '{}', but expected '{}'",
                        names[found.clamp(0, 2) as usize],
                        names[expected as usize]
                    ),
                );
            }
        } else if let Some(group) = vector_group {
            if instruction.immediate_at(group.first) != 0 {
                return self.error(location, "VPT predicated instructions must be in VPT block");
            }
        }
        let sequential = |first: usize, second: usize| {
            encoding(instruction.register_at(second))
                == encoding(instruction.register_at(first)) + 1
        };
        let immediate = |index: usize| instruction.immediate_at(index);
        let same_encoding = |first: usize, second: usize| {
            encoding(instruction.register_at(first)) == encoding(instruction.register_at(second))
        };
        let list_contains = |from: usize, held: RegisterId| {
            (from..instruction.operands.len()).any(|index| instruction.register_at(index) == held)
        };
        let register_list_index = || {
            operands
                .iter()
                .enumerate()
                .skip(end)
                .find(|(_, held)| matches!(operand_of(held), Some(Operand::RegisterList(_))))
                .map_or(0, |(index, _)| index)
        };
        let load_list = |from: usize, allow_stack: bool, me: &mut Self| -> bool {
            let contains_stack = list_contains(from, stack);
            let contains_link = list_contains(from, link);
            let contains_counter = list_contains(from, counter);
            if !allow_stack && contains_stack {
                return me.error(
                    spot(register_list_index()),
                    "SP may not be in the register list",
                );
            }
            if contains_counter && contains_link {
                return me.error(
                    spot(register_list_index()),
                    "PC and LR may not be in the register list simultaneously",
                );
            }
            false
        };
        let store_list = |from: usize, me: &mut Self| -> bool {
            let contains_stack = list_contains(from, stack);
            let contains_counter = list_contains(from, counter);
            if contains_stack && contains_counter {
                return me.error(
                    spot(register_list_index()),
                    "SP and PC may not be in the register list",
                );
            }
            if contains_stack {
                return me.error(
                    spot(register_list_index()),
                    "SP may not be in the register list",
                );
            }
            if contains_counter {
                return me.error(
                    spot(register_list_index()),
                    "PC may not be in the register list",
                );
            }
            false
        };
        let pair_check = |load: bool, arm_mode: bool, writeback: bool, me: &mut Self| -> bool {
            let first = if load || !writeback { 0 } else { 1 };
            let rt = encoding(instruction.register_at(first));
            let rt2 = encoding(instruction.register_at(first + 1));
            if arm_mode {
                if rt == 14 {
                    return me.error(spot(end), "Rt can't be R14");
                }
                if rt & 1 == 1 {
                    return me.error(spot(end), "Rt must be even-numbered");
                }
                if rt2 != rt + 1 {
                    return me.error(
                        spot(end),
                        if load {
                            "destination operands must be sequential"
                        } else {
                            "source operands must be sequential"
                        },
                    );
                }
            }
            if !arm_mode && load && rt2 == rt {
                return me.error(spot(end), "destination operands can't be identical");
            }
            if writeback {
                let base = encoding(instruction.register_at(3));
                if base == rt || base == rt2 {
                    return if load {
                        me.error(
                            spot(end),
                            "base register needs to be different from destination registers",
                        )
                    } else {
                        me.error(
                            spot(end + 2),
                            "source register and base register can't be identical",
                        )
                    };
                }
            }
            false
        };
        let low = |held: RegisterId| in_class(target, "tGPR", held);
        let offset_index = |first: usize| {
            if matches!(
                operands.get(first).and_then(operand_of),
                Some(Operand::Immediate(_))
            ) {
                first
            } else {
                first + 1
            }
        };
        match record {
            "VLLDM" | "VLLDM_T2" | "VLSTM" | "VLSTM_T2" => {
                if operands.len() == end + 2 {
                    if let Some(Operand::RegisterList(list)) =
                        operands.get(end + 1).and_then(operand_of)
                    {
                        let thirty_two = self
                            .arm32_feature("d32")
                            .is_some_and(|held| self.arm_features.contains(&held));
                        let at_end = SourceLocation {
                            buffer: statement,
                            position: ends[end + 1],
                        };
                        if list.len() == 32 && !self.arm_features.contains("HasV8_1MMainlineOps") {
                            return self.error(at_end, "T2 version requires v8.1-M.Main");
                        }
                        if thirty_two && list.len() != 32 {
                            return self.error(at_end, "operand must be exactly {d0-d31}");
                        }
                        if !thirty_two && list.len() != 16 && list.len() != 32 {
                            return self.error(
                                at_end,
                                "operand must be exactly {d0-d15} (T1) or {d0-d31} (T2)",
                            );
                        }
                    }
                }
                return false;
            }
            "t2IT" | "ITasm" => {
                if immediate(0) == 14 && (immediate(1) as u32).count_ones() != 1 {
                    return self.error(location, "unpredictable IT predicate sequence");
                }
            }
            "LDRD" if pair_check(true, true, false, self) => return true,
            "LDRD_PRE" | "LDRD_POST" if pair_check(true, true, true, self) => return true,
            "t2LDRDi8" if pair_check(true, false, false, self) => return true,
            "t2LDRD_PRE" | "t2LDRD_POST" if pair_check(true, false, true, self) => return true,
            "STRD" if pair_check(false, true, false, self) => return true,
            "STRD_PRE" | "STRD_POST" if pair_check(false, true, true, self) => return true,
            "t2STRD_PRE" | "t2STRD_POST" if pair_check(false, false, true, self) => return true,
            "t2BXJ" => {
                if instruction.register_at(0) == stack && !version8 {
                    return self.error(spot(end), "r13 (SP) is an unpredictable operand to BXJ");
                }
                return false;
            }
            "STR_PRE_IMM" | "STR_PRE_REG" | "t2STR_PRE" | "STR_POST_IMM" | "STR_POST_REG"
            | "t2STR_POST" | "STRH_PRE" | "t2STRH_PRE" | "STRH_POST" | "t2STRH_POST"
            | "STRB_PRE_IMM" | "STRB_PRE_REG" | "t2STRB_PRE" | "STRB_POST_IMM"
            | "STRB_POST_REG" | "t2STRB_POST" => {
                if same_encoding(1, 2) {
                    return self.error(
                        spot(end + 1),
                        "source register and base register can't be identical",
                    );
                }
                return false;
            }
            "t2LDR_PRE_imm" | "t2LDR_POST_imm" | "t2STR_PRE_imm" | "t2STR_POST_imm" => {
                if same_encoding(0, 1) {
                    return self.error(
                        spot(end),
                        "destination register and base register can't be identical",
                    );
                }
                if matches!(record, "t2LDR_POST_imm" | "t2STR_POST_imm") {
                    let value = immediate(2);
                    if !(-255..=255).contains(&value) {
                        return self.error(spot(end + 2), "operand must be in range [-255, 255]");
                    }
                }
                if matches!(record, "t2STR_PRE_imm" | "t2STR_POST_imm")
                    && instruction.register_at(0) == counter
                {
                    return self.error(spot(end), "operand must be a register in range [r0, r14]");
                }
                return false;
            }
            "t2LDRB_OFFSET_imm" | "t2LDRB_PRE_imm" | "t2LDRB_POST_imm" | "t2STRB_OFFSET_imm"
            | "t2STRB_PRE_imm" | "t2STRB_POST_imm" | "t2LDRH_OFFSET_imm" | "t2LDRH_PRE_imm"
            | "t2LDRH_POST_imm" | "t2STRH_OFFSET_imm" | "t2STRH_PRE_imm" | "t2STRH_POST_imm"
            | "t2LDRSB_OFFSET_imm" | "t2LDRSB_PRE_imm" | "t2LDRSB_POST_imm"
            | "t2LDRSH_OFFSET_imm" | "t2LDRSH_PRE_imm" | "t2LDRSH_POST_imm" => {
                let value = immediate(2);
                if record.contains("_POST_") || record.contains("_PRE_") {
                    if !(-255..=255).contains(&value) {
                        return self.error(spot(end + 2), "operand must be in range [-255, 255]");
                    }
                } else if !(-255..=0).contains(&value) {
                    return self.error(
                        spot(end + 2),
                        "operand must be in range [0, 255] with a negative sign",
                    );
                }
                if instruction.register_at(0) == counter {
                    let (place, text) = match record {
                        "t2LDRB_OFFSET_imm" | "t2LDRB_PRE_imm" | "t2LDRB_POST_imm"
                        | "t2STRB_OFFSET_imm" | "t2STRB_PRE_imm" | "t2STRB_POST_imm" => {
                            (end, "if operand is PC, should call the LDRB (literal)")
                        }
                        "t2LDRSB_OFFSET_imm" | "t2LDRSB_PRE_imm" | "t2LDRSB_POST_imm" => {
                            (end + 2, "if operand is PC, should call the LDRH (literal)")
                        }
                        _ => (end, "if operand is PC, should call the LDRH (literal)"),
                    };
                    return self.error(spot(place), text);
                }
                return false;
            }
            "LDR_PRE_IMM" | "LDR_PRE_REG" | "t2LDR_PRE" | "LDR_POST_IMM" | "LDR_POST_REG"
            | "t2LDR_POST" | "LDRH_PRE" | "t2LDRH_PRE" | "LDRH_POST" | "t2LDRH_POST"
            | "LDRSH_PRE" | "t2LDRSH_PRE" | "LDRSH_POST" | "t2LDRSH_POST" | "LDRB_PRE_IMM"
            | "LDRB_PRE_REG" | "t2LDRB_PRE" | "LDRB_POST_IMM" | "LDRB_POST_REG" | "t2LDRB_POST"
            | "LDRSB_PRE" | "t2LDRSB_PRE" | "LDRSB_POST" | "t2LDRSB_POST" => {
                if same_encoding(0, 2) {
                    return self.error(
                        spot(end),
                        "destination register and base register can't be identical",
                    );
                }
                return false;
            }
            "SBFX" | "t2SBFX" | "UBFX" | "t2UBFX" => {
                let lsb = immediate(2) as u32;
                let width_minus_one = immediate(3) as u32;
                if width_minus_one >= 32u32.saturating_sub(lsb) {
                    return self.error(spot(end + 2), "bitfield width must be in range [1,32-lsb]");
                }
                return false;
            }
            "tLDMIA" => {
                let base = instruction.register_at(0);
                let writeback_token = operands
                    .get(end + 1)
                    .is_some_and(|held| held.token_text() == "!");
                let contains_base = list_contains(3, base);
                let invalid_low = (3..instruction.operands.len())
                    .any(|index| !low(instruction.register_at(index)));
                if invalid_low && !thumb_two {
                    return self.error(
                        spot(register_list_index()),
                        "registers must be in range r0-r7",
                    );
                }
                if !contains_base && !writeback_token && !thumb_two {
                    return self.error(
                        spot(register_list_index()),
                        "writeback operator '!' expected",
                    );
                }
                if contains_base && writeback_token {
                    return self.error(
                        spot(end + 1),
                        "writeback operator '!' not allowed when base register in register list",
                    );
                }
                if load_list(3, false, self) {
                    return true;
                }
            }
            "LDMIA_UPD" | "LDMDB_UPD" | "LDMIB_UPD" | "LDMDA_UPD" => {
                if self.arm_features.contains("HasV7Ops")
                    && list_contains(3, instruction.register_at(0))
                {
                    return self.error(
                        spot(operands.len().saturating_sub(1)),
                        "writeback register not allowed in register list",
                    );
                }
            }
            "t2LDMIA" | "t2LDMDB" if load_list(3, false, self) => return true,
            "t2STMIA" | "t2STMDB" if store_list(3, self) => return true,
            "t2LDMIA_UPD" | "t2LDMDB_UPD" | "t2STMIA_UPD" | "t2STMDB_UPD" => {
                if list_contains(3, instruction.register_at(0)) {
                    return self.error(
                        spot(operands.len().saturating_sub(1)),
                        "writeback register not allowed in register list",
                    );
                }
                if matches!(record, "t2LDMIA_UPD" | "t2LDMDB_UPD") {
                    if load_list(3, false, self) {
                        return true;
                    }
                } else if store_list(3, self) {
                    return true;
                }
            }
            "sysLDMIA_UPD" | "sysLDMDA_UPD" | "sysLDMDB_UPD" | "sysLDMIB_UPD" => {
                if !list_contains(3, counter) {
                    return self.error(
                        spot(end + 1),
                        "writeback register only allowed on system LDM if PC in register-list",
                    );
                }
            }
            "sysSTMIA_UPD" | "sysSTMDA_UPD" | "sysSTMDB_UPD" | "sysSTMIB_UPD" => {
                return self.error(spot(end), "system STM cannot have writeback register");
            }
            "tPOP" => {
                let invalid = (2..instruction.operands.len()).any(|index| {
                    let held = instruction.register_at(index);
                    !low(held) && held != counter
                });
                if invalid && !thumb_two {
                    return self.error(spot(end), "registers must be in range r0-r7 or pc");
                }
                let m_class = self
                    .arm32_feature("mclass")
                    .is_some_and(|held| self.arm_features.contains(&held));
                if load_list(2, !m_class, self) {
                    return true;
                }
            }
            "tPUSH" => {
                let invalid = (2..instruction.operands.len()).any(|index| {
                    let held = instruction.register_at(index);
                    !low(held) && held != link
                });
                if invalid && !thumb_two {
                    return self.error(spot(end), "registers must be in range r0-r7 or lr");
                }
                if store_list(2, self) {
                    return true;
                }
            }
            "tSTMIA_UPD" => {
                let base = instruction.register_at(0);
                let invalid_low = (4..instruction.operands.len())
                    .any(|index| !low(instruction.register_at(index)));
                let contains_base = list_contains(4, base);
                if invalid_low && !thumb_two {
                    return self.error(spot(end + 2), "registers must be in range r0-r7");
                }
                if invalid_low && contains_base {
                    return self.error(
                        spot(end),
                        "writeback operator '!' not allowed when base register in register list",
                    );
                }
                if store_list(4, self) {
                    return true;
                }
            }
            "tADDrSP" => {
                if !thumb_two && instruction.register_at(0) != instruction.register_at(2) {
                    return self.error(
                        spot(end + 2),
                        "source register must be the same as destination",
                    );
                }
            }
            "t2ADDrr" | "t2ADDrs" | "t2SUBrr" | "t2SUBrs" => {
                if instruction.register_at(0) == stack && instruction.register_at(1) != stack {
                    return self.error(
                        spot(end + 1),
                        "source register must be sp if destination is sp",
                    );
                }
            }
            "tB" => {
                if !Self::arm32_signed_offset(operands.get(end), 11, 1) {
                    return self.error(spot(end), "branch target out of range");
                }
            }
            "t2B" => {
                let index = offset_index(end);
                let binary = matches!(
                    operands.get(index).and_then(operand_of),
                    Some(Operand::Immediate(Expression::Binary { .. }))
                );
                if !binary && !Self::arm32_signed_offset(operands.get(index), 24, 1) {
                    return self.error(spot(index), "branch target out of range");
                }
            }
            "tBcc" => {
                if !Self::arm32_signed_offset(operands.get(end), 8, 1) {
                    return self.error(spot(end), "branch target out of range");
                }
            }
            "t2Bcc" => {
                let index = offset_index(end);
                if !Self::arm32_signed_offset(operands.get(index), 20, 1) {
                    return self.error(spot(index), "branch target out of range");
                }
            }
            "tCBZ" | "tCBNZ" => {
                if !Self::arm32_unsigned_offset(operands.get(end + 1), 6, 1) {
                    return self.error(spot(end + 1), "branch target out of range");
                }
            }
            "MOVi16" | "MOVTi16" | "t2MOVi16" | "t2MOVTi16" => {
                let index = offset_index(end);
                if let Some(Operand::Immediate(expression)) =
                    operands.get(index).and_then(operand_of)
                {
                    let specified = matches!(
                        expression,
                        Expression::Symbol { specifier: Some(name), .. }
                            if name.starts_with("lower16") || name.starts_with("upper16")
                    ) || matches!(expression, Expression::Prefixed { .. });
                    if !matches!(expression, Expression::Constant(_)) && !specified {
                        return self.error(
                            spot(index),
                            "immediate expression for mov requires :lower16: or :upper16",
                        );
                    }
                }
            }
            "tADDi8" | "tMOVi8" => {
                let index = if record == "tMOVi8"
                    || matches!(
                        operands.get(end + 1).and_then(operand_of),
                        Some(Operand::Immediate(_))
                    ) {
                    end + 1
                } else {
                    end + 2
                };
                if let Some(Operand::Immediate(expression)) =
                    operands.get(index).and_then(operand_of)
                {
                    if !matches!(expression, Expression::Constant(_)) {
                        let allowed = matches!(
                            expression,
                            Expression::Symbol { specifier: Some(name), .. }
                                if matches!(
                                    name.as_str(),
                                    "lower0_7" | "lower8_15" | "upper0_7" | "upper8_15"
                                )
                        );
                        if !allowed {
                            return self.error(
                                spot(index),
                                if record == "tADDi8" {
                                    "Immediate expression for Thumb adds requires :lower0_7:, :lower8_15:, :upper0_7: or :upper8_15:"
                                } else {
                                    "Immediate expression for Thumb movs requires :lower0_7:, :lower8_15:, :upper0_7: or :upper8_15:"
                                },
                            );
                        }
                    }
                }
            }
            "HINT" | "t2HINT" => {
                let hint = immediate(0);
                let predicate = immediate(1);
                if hint == 0x10 && predicate != 14 && self.arm_features.contains("FeatureRAS") {
                    return self.error(
                        spot(1),
                        "instruction 'esb' is not predicable, but condition code specified",
                    );
                }
                if hint == 0x14 && predicate != 14 {
                    return self.error(
                        spot(1),
                        "instruction 'csdb' is not predicable, but condition code specified",
                    );
                }
            }
            "t2BFi" | "t2BFr" | "t2BFLi" | "t2BFLr" => {
                let zero = matches!(instruction.operand(0), MachineOperand::Immediate(0));
                if !Self::arm32_unsigned_offset(operands.get(end), 4, 1) || zero {
                    return self.error(
                        spot(end),
                        "branch location out of range or not a multiple of 2",
                    );
                }
                if record == "t2BFi" && !Self::arm32_signed_offset(operands.get(end + 1), 16, 1) {
                    return self.error(
                        spot(end),
                        "branch target out of range or not a multiple of 2",
                    );
                }
                if record == "t2BFLi" && !Self::arm32_signed_offset(operands.get(end + 1), 18, 1) {
                    return self.error(
                        spot(end),
                        "branch target out of range or not a multiple of 2",
                    );
                }
            }
            "t2BFic" => {
                let zero = matches!(instruction.operand(0), MachineOperand::Immediate(0));
                if !Self::arm32_unsigned_offset(operands.get(end), 4, 1) || zero {
                    return self.error(
                        spot(1),
                        "branch location out of range or not a multiple of 2",
                    );
                }
                if !Self::arm32_signed_offset(operands.get(end + 1), 16, 1) {
                    return self.error(
                        spot(end + 1),
                        "branch target out of range or not a multiple of 2",
                    );
                }
                if let (MachineOperand::Immediate(first), MachineOperand::Immediate(second)) =
                    (instruction.operand(0), instruction.operand(2))
                {
                    let difference = second - first;
                    if difference != 4 && difference != 2 {
                        return self.error(
                            spot(3),
                            "else branch target must be 2 or 4 greater than the branch location",
                        );
                    }
                }
            }
            "t2CLRM" => {
                for index in 2..instruction.operands.len() {
                    if let MachineOperand::Register(held) = instruction.operand(index) {
                        if !in_class(target, "GPRwithAPSRnosp", *held) {
                            return self.error(
                                spot(end),
                                "invalid register in register list. Valid registers are r0-r12, lr/r14 and APSR.",
                            );
                        }
                    }
                }
            }
            "DSB" | "t2DSB" => {
                if instruction.operands.len() >= 2 {
                    let option = immediate(0);
                    let predicate = immediate(1);
                    if option == 0 && predicate != 14 {
                        return self.error(
                            spot(1),
                            "instruction 'ssbb' is not predicable, but condition code specified",
                        );
                    }
                    if option == 4 && predicate != 14 {
                        return self.error(
                            spot(1),
                            "instruction 'pssbb' is not predicable, but condition code specified",
                        );
                    }
                }
            }
            "VMOVRRS" => {
                if !sequential(2, 3) {
                    return self.error(spot(end + 2), "source operands must be sequential");
                }
            }
            "VMOVSRR" => {
                if !sequential(0, 1) {
                    return self.error(spot(end), "destination operands must be sequential");
                }
            }
            "VLDMDIA" | "VSTMDIA" => {
                if let Some(Operand::RegisterList(list)) =
                    operands.get(end + 1).and_then(operand_of)
                {
                    if list.is_empty() || list.len() > 16 {
                        return self.error(
                            spot(end + 1),
                            "list of registers must be at least 1 and at most 16",
                        );
                    }
                }
            }
            "MVE_VMOV_rr_q" | "MVE_VMOV_q_rr" => {
                let offset = if record == "MVE_VMOV_rr_q" { 2 } else { 0 };
                let register_at = |at: usize| match operands.get(at).and_then(operand_of) {
                    Some(Operand::Register(held)) => *held,
                    _ => NO_REGISTER,
                };
                let index_at = |at: usize| match operands.get(at).and_then(operand_of) {
                    Some(Operand::VectorIndex(held)) => i64::from(*held),
                    _ => -1,
                };
                if register_at(end + offset) != register_at(end + offset + 2) {
                    return self.error(spot(end + offset), "Q-registers must be the same");
                }
                if index_at(end + offset + 1) != index_at(end + offset + 3) + 2 {
                    return self.error(
                        spot(end + offset + 1),
                        "Q-register indexes must be 2 and 0 or 3 and 1",
                    );
                }
            }
            "MVE_SQRSHR" | "MVE_UQRSHL" => {
                let register_of = |at: usize| match operands.get(at).and_then(operand_of) {
                    Some(Operand::Register(held)) => *held,
                    _ => NO_REGISTER,
                };
                if register_of(end) == register_of(end + 1) {
                    return self
                        .error(spot(end), "Rda register and Rm register can't be identical");
                }
            }
            "UMAAL" | "UMLAL" | "UMULL" | "t2UMAAL" | "t2UMLAL" | "t2UMULL" | "SMLAL"
            | "SMLALBB" | "SMLALBT" | "SMLALD" | "SMLALDX" | "SMLALTB" | "SMLALTT" | "SMLSLD"
            | "SMLSLDX" | "SMULL" | "t2SMLAL" | "t2SMLALBB" | "t2SMLALBT" | "t2SMLALD"
            | "t2SMLALDX" | "t2SMLALTB" | "t2SMLALTT" | "t2SMLSLD" | "t2SMLSLDX" | "t2SMULL" => {
                if instruction.register_at(0) == instruction.register_at(1) {
                    return self.error(
                        location,
                        "unpredictable instruction, RdHi and RdLo must be different",
                    );
                }
            }
            "VTOSHH" | "VTOUHH" | "VTOSLH" | "VTOULH" | "VTOSHS" | "VTOUHS" | "VTOSLS"
            | "VTOULS" | "VTOSHD" | "VTOUHD" | "VTOSLD" | "VTOULD" | "VSHTOH" | "VUHTOH"
            | "VSLTOH" | "VULTOH" | "VSHTOS" | "VUHTOS" | "VSLTOS" | "VULTOS" | "VSHTOD"
            | "VUHTOD" | "VSLTOD" | "VULTOD" => {
                let first = operands.get(end).and_then(operand_of);
                let second = operands.get(end + 1).and_then(operand_of);
                if let (Some(Operand::Register(left)), Some(Operand::Register(right))) =
                    (first, second)
                {
                    if left != right {
                        return self.error(
                            spot(end),
                            "source and destination registers must be the same",
                        );
                    }
                }
            }
            name if name.starts_with("CDE_") => {
                let coprocessor = immediate(1);
                if coprocessor < 8 && !self.arm32_is_coprocessor_data_engine(coprocessor) {
                    return self.error(spot(1), "coprocessor must be configured as CDE");
                } else if coprocessor >= 8 {
                    return self.error(spot(1), "coprocessor must be in the range [p0, p7]");
                }
            }
            "t2CDP" | "t2CDP2" | "t2MCR" | "t2MCR2" | "t2MCRR" | "t2MCRR2" | "t2MRC" | "t2MRC2"
            | "t2MRRC" | "t2MRRC2" => {
                let index = match record {
                    "t2MRRC" | "t2MRRC2" => 2,
                    "t2MRC" | "t2MRC2" => 1,
                    _ => 0,
                };
                if self.arm32_is_coprocessor_data_engine(immediate(index)) {
                    return self.error(spot(2), "coprocessor must be configured as GCP");
                }
            }
            name if (name.starts_with("t2LDC") || name.starts_with("t2STC"))
                && self.arm32_is_coprocessor_data_engine(immediate(0)) =>
            {
                return self.error(spot(2), "coprocessor must be configured as GCP");
            }
            _ => {}
        }
        false
    }

    fn arm32_general_pair(&self, low: RegisterId, high: RegisterId) -> Option<RegisterId> {
        self.target
            .registers
            .iter()
            .position(|held| held.sub_registers == [low, high] && !held.assembly_name.is_empty())
            .or_else(|| {
                self.target
                    .registers
                    .iter()
                    .position(|held| held.sub_registers == [low, high])
            })
            .map(|index| index as RegisterId)
    }

    fn arm32_fix_pair_operands(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        location: SourceLocation,
    ) -> bool {
        let Some(mnemonic) = operands.first().map(|held| held.token_text().to_string()) else {
            return false;
        };
        if matches!(
            mnemonic.as_str(),
            "cx1d" | "cx1da" | "cx2d" | "cx2da" | "cx3d" | "cx3da"
        ) && (0..6).any(|number| {
            self.arm_features
                .contains(&format!("FeatureCoprocCDE{number}"))
        }) {
            let end = Self::arm32_mnemonic_operand_end(operands);
            if operands.len() < end + 3 {
                return false;
            }
            let diagnostic = "operand must be an even-numbered register in the range [r0, r10]";
            let at = self.statement_location(operands[end + 1].start);
            let Some(Operand::Register(first)) = operands.get(end + 1).and_then(operand_of) else {
                return self.error(at, diagnostic);
            };
            let first = *first;
            let encoding = register_encoding(self.target, first);
            if !in_class(self.target, "GPR", first) || encoding > 10 || encoding & 1 == 1 {
                return self.error(at, diagnostic);
            }
            let next = register_named(self.target, &format!("r{}", encoding + 1));
            let second = match operands.get(end + 2).and_then(operand_of) {
                Some(Operand::Register(held)) if Some(*held) == next => *held,
                _ => {
                    let at = self.statement_location(operands[end + 2].start);
                    return self.error(at, "operand must be a consecutive register");
                }
            };
            if let Some(pair) = self.arm32_general_pair(first, second) {
                let (start, finish) = (operands[end + 1].start, operands[end + 2].end);
                operands[end + 1] = parsed(Operand::Register(pair), start, finish);
                operands.remove(end + 2);
            }
            return false;
        }
        if !matches!(
            mnemonic.as_str(),
            "ldrd" | "strd" | "ldrexd" | "strexd" | "ldaexd" | "stlexd"
        ) {
            return false;
        }
        let end = Self::arm32_mnemonic_operand_end(operands);
        let index = if matches!(mnemonic.as_str(), "strexd" | "stlexd") {
            end + 1
        } else {
            end
        };
        let target = self.target;
        let version8 = self.arm_features.contains("HasV8Ops");
        if operands.len() >= index + 2 {
            if let (Some(Operand::Register(first)), Some(Operand::Memory(memory))) = (
                operands.get(index).and_then(operand_of),
                operands.get(index + 1).and_then(operand_of),
            ) {
                let first = *first;
                let general = in_class(target, "GPR", first)
                    && (memory.base == NO_REGISTER || in_class(target, "GPR", memory.base))
                    && (memory.offset_register == NO_REGISTER
                        || in_class(target, "GPR", memory.offset_register));
                if general {
                    let encoding = register_encoding(target, first);
                    let stack = register_named(target, "sp").unwrap_or(NO_REGISTER);
                    let counter = register_named(target, "pc").unwrap_or(NO_REGISTER);
                    let next = register_named(target, &format!("r{}", encoding + 1));
                    let allowed = (self.arm_thumb || encoding & 1 == 0)
                        && first != counter
                        && next.is_some_and(|held| held != counter && (held != stack || version8));
                    if allowed {
                        let (start, finish) = (operands[index].start, operands[index].end);
                        operands.insert(
                            index + 1,
                            parsed(
                                Operand::Register(next.unwrap_or(NO_REGISTER)),
                                start,
                                finish,
                            ),
                        );
                    }
                }
            }
        }
        let load = matches!(mnemonic.as_str(), "ldrexd" | "ldaexd");
        if !self.arm_thumb
            && operands.len() > end + 1 + usize::from(!load)
            && matches!(mnemonic.as_str(), "ldrexd" | "strexd" | "ldaexd" | "stlexd")
        {
            let slot = if load { end } else { end + 1 };
            if let (Some(Operand::Register(first)), Some(Operand::Register(second))) = (
                operands.get(slot).and_then(operand_of),
                operands.get(slot + 1).and_then(operand_of),
            ) {
                let (first, second) = (*first, *second);
                if in_class(target, "GPR", first) {
                    let low = register_encoding(target, first);
                    let high = register_encoding(target, second);
                    if low + 1 != high {
                        let at = self.statement_location(operands[slot + 1].start);
                        return self.error(
                            at,
                            if load {
                                "destination operands must be sequential"
                            } else {
                                "source operands must be sequential"
                            },
                        );
                    }
                    if low & 1 == 1 {
                        let at = self.statement_location(operands[slot].start);
                        return self.error(
                            at,
                            if load {
                                "destination operands must start start at an even register"
                            } else {
                                "source operands must start start at an even register"
                            },
                        );
                    }
                    if let Some(pair) = self.arm32_general_pair(first, second) {
                        let (start, finish) = (operands[slot].start, operands[slot + 1].end);
                        operands[slot] = parsed(Operand::Register(pair), start, finish);
                        operands.remove(slot + 1);
                    }
                }
            }
        }
        let _ = location;
        false
    }

    fn arm32_near_miss_messages(
        &self,
        misses: &[NearMiss],
        operands: &[ParsedOperand],
        location: SourceLocation,
    ) -> Vec<(SourceLocation, String)> {
        let matcher = self.matcher;
        let thumb = self.arm_thumb;
        let has_arm = self
            .arm32_feature("noarm")
            .is_none_or(|held| !self.arm_features.contains(&held));
        let m_class = self
            .arm32_feature("mclass")
            .is_some_and(|held| self.arm_features.contains(&held));
        let mut messages: Vec<(SourceLocation, String)> = Vec::new();
        let mut operand_seen: Vec<(usize, Option<usize>)> = Vec::new();
        let mut feature_seen: Vec<&Vec<String>> = Vec::new();
        let mut reported_too_few = false;
        for miss in misses.iter().rev() {
            match miss {
                NearMiss::Operand {
                    message,
                    class,
                    index,
                } => {
                    let duplicate_class = message.as_ref().and(*class);
                    let duplicate = operand_seen.iter().any(|(held_index, held_class)| {
                        *held_index == *index
                            && match (duplicate_class, held_class) {
                                (Some(left), Some(right)) => matcher.is_subclass(left, *right),
                                (None, None) => true,
                                _ => false,
                            }
                    });
                    if duplicate {
                        continue;
                    }
                    operand_seen.push((*index, duplicate_class));
                    let text = match (message, class) {
                        (Some(text), _) => text.clone(),
                        (None, None) => "too many operands for instruction".to_string(),
                        (None, Some(_)) => "invalid operand for instruction".to_string(),
                    };
                    messages.push((self.statement_location(operands[*index].start), text));
                }
                NearMiss::Feature(features) => {
                    if feature_seen.contains(&features) {
                        continue;
                    }
                    feature_seen.push(features);
                    let has = |name: &str| features.iter().any(|held| held == name);
                    if has("IsARM") && !has_arm {
                        continue;
                    }
                    if thumb && has("IsARM") && features.len() > 1 {
                        continue;
                    }
                    if !thumb && has("IsThumb") && features.len() > 1 {
                        continue;
                    }
                    if !thumb
                        && has("IsThumb2")
                        && features
                            .iter()
                            .any(|held| held != "IsThumb2" && held != "IsThumb")
                    {
                        continue;
                    }
                    if m_class && has("HasNEON") {
                        continue;
                    }
                    let mut text = "instruction requires:".to_string();
                    for feature in features {
                        text.push(' ');
                        text.push_str(&self.arm32_feature_name(feature));
                    }
                    messages.push((location, text));
                }
                NearMiss::Predicate(error) => {
                    let (at, text) = match error {
                        PredicateError::NotITBlock => (
                            location,
                            "flag setting instruction only valid outside IT block",
                        ),
                        PredicateError::ITBlock => {
                            (location, "instruction only valid inside IT block")
                        }
                        PredicateError::V6 => {
                            (location, "instruction variant requires ARMv6 or later")
                        }
                        PredicateError::Thumb2 => (location, "instruction variant requires Thumb2"),
                        PredicateError::V8 => {
                            (location, "instruction variant requires ARMv8 or later")
                        }
                        PredicateError::FlagSetting => (
                            location,
                            "no flag-preserving variant of this instruction available",
                        ),
                        PredicateError::TiedOperand => {
                            let first = operands
                                .iter()
                                .enumerate()
                                .skip(1)
                                .find(|(_, held)| {
                                    !matches!(
                                        operand_of(held),
                                        Some(Operand::Condition(_) | Operand::CarrySet(_))
                                    )
                                })
                                .map_or(location, |(_, held)| self.statement_location(held.start));
                            (first, "destination register must match a source register")
                        }
                        PredicateError::InvalidOperand => {
                            (location, "invalid operand for instruction")
                        }
                    };
                    messages.push((at, text.to_string()));
                }
                NearMiss::TooFewOperands => {
                    if !reported_too_few {
                        reported_too_few = true;
                        let end = operands
                            .last()
                            .map_or(location, |held| self.statement_location(held.end));
                        messages.push((end, "too few operands for instruction".to_string()));
                    }
                }
            }
        }
        messages
    }

    fn arm32_feature_name(&self, feature: &str) -> String {
        match self.target.records.get(feature) {
            Some(record) if !table_text(record, "PredicateName").is_empty() => {
                table_text(record, "PredicateName")
            }
            Some(record) => match record.field("AssemblerCondDag") {
                Some(condition) => armv8::requirement_text(self.target, condition).0,
                None => feature.to_string(),
            },
            None => feature.to_string(),
        }
    }

    fn arm32_report_near_misses(
        &mut self,
        misses: &[NearMiss],
        operands: &[ParsedOperand],
        location: SourceLocation,
    ) -> bool {
        let messages = self.arm32_near_miss_messages(misses, operands, location);
        match messages.as_slice() {
            [] => self.error(location, "invalid instruction"),
            [(at, text)] => self.error(*at, text),
            _ => {
                let result = self.error(
                    location,
                    "invalid instruction, any one of the following would fix this:",
                );
                for (at, text) in &messages {
                    self.arm32_note(*at, text);
                }
                result
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
    VptMask,
    RestrictedCondition,
    MveRegisterQ,
    MveQImmediate(u32),
    MvePairIndex(i64),
    MveShiftImmediate,
    BranchFuture(bool, Fixup32),
    BranchFutureAfter,
    ShiftRight(u32),
    AddressMode6(u32),
    AddressMode6Offset,
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
        "getMVEPairVectorIndexOpValue" => {
            let (_, arguments) = split_arguments(text);
            OperandEncoder::MvePairIndex(
                arguments
                    .first()
                    .and_then(|argument| argument.parse::<i64>().ok())
                    .unwrap_or(0),
            )
        }
        "getMVEShiftImmOpValue" => OperandEncoder::MveShiftImmediate,
        "getMveAddrModeRQOpValue" => OperandEncoder::MveRegisterQ,
        "getMveAddrModeQOpValue" => {
            let (_, arguments) = split_arguments(text);
            OperandEncoder::MveQImmediate(
                arguments
                    .first()
                    .and_then(|argument| argument.parse::<u32>().ok())
                    .unwrap_or(0),
            )
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
        "getBFTargetOpValue" => {
            let (_, arguments) = split_arguments(text);
            let negative = arguments.first().is_some_and(|held| held == "true");
            let kind = match arguments.get(1).map(String::as_str) {
                Some("ARM::fixup_bf_branch") => Fixup32::BfBranch,
                Some("ARM::fixup_bf_target") => Fixup32::BfTarget,
                Some("ARM::fixup_bfl_target") => Fixup32::BflTarget,
                Some("ARM::fixup_bfc_target") => Fixup32::BfcTarget,
                Some("ARM::fixup_wls") => Fixup32::Wls,
                Some("ARM::fixup_le") => Fixup32::Le,
                _ => return Err(format!("the encoder {text}")),
            };
            OperandEncoder::BranchFuture(negative, kind)
        }
        "getBFAfterTargetOpValue" => OperandEncoder::BranchFutureAfter,
        "getVPTMaskOpValue" => OperandEncoder::VptMask,
        "getRestrictedCondCodeOpValue" => OperandEncoder::RestrictedCondition,
        "getAddrMode6AddressOpValue" => OperandEncoder::AddressMode6(0),
        "getAddrMode6OneLane32AddressOpValue" => OperandEncoder::AddressMode6(1),
        "getAddrMode6DupAddressOpValue" => OperandEncoder::AddressMode6(2),
        "getAddrMode6OffsetOpValue" => OperandEncoder::AddressMode6Offset,
        "getShiftRight8Imm" => OperandEncoder::ShiftRight(8),
        "getShiftRight16Imm" => OperandEncoder::ShiftRight(16),
        "getShiftRight32Imm" => OperandEncoder::ShiftRight(32),
        "getShiftRight64Imm" | "getNEONVcvtImm32OpValue" => OperandEncoder::ShiftRight(64),
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
    mve: bool,
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
            MachineOperand::Register(register) => {
                let number = u64::from(register_encoding(target, *register));
                if !mve
                    && (in_class(target, "QPR", *register) || in_class(target, "QQPR", *register))
                {
                    number * 2
                } else {
                    number
                }
            }
            MachineOperand::Immediate(value) => *value as u64,
            MachineOperand::Expression(_) => return Err(unencodable("an expression operand")),
        },
        OperandEncoder::CarryOut => u64::from(instruction.register_at(first) != NO_REGISTER),
        OperandEncoder::AddressMode6(kind) => {
            let alignment = immediate(first + 1);
            let align = match (kind, alignment) {
                (0, 2 | 4 | 8) | (2, 2 | 4 | 8) => 1,
                (0, 16) => 2,
                (0, 32) => 3,
                (2, 16) => 3,
                (1, 4) => 3,
                _ => 0,
            };
            register(first) | (align << 4)
        }
        OperandEncoder::AddressMode6Offset => {
            if instruction.register_at(first) == NO_REGISTER {
                0x0d
            } else {
                register(first)
            }
        }
        OperandEncoder::ShiftRight(width) => {
            u64::from(*width).wrapping_sub(immediate(first) as u64)
        }
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
            let leading = instruction.register_at(first);
            let singles = in_class(target, "SPR", leading);
            let doubles = in_class(target, "DPR", leading);
            let vpr = register_named(target, "vpr") == Some(leading);
            if singles || doubles || vpr {
                let mut count = (instruction.operands.len() - first) as u64 & 0xff;
                let binary = (register(first) & 0x1f) << 8;
                match target.instructions[instruction.opcode].record.as_str() {
                    "VSCCLRMD" => count -= 1,
                    "VSCCLRMS" => {
                        count = (first..instruction.operands.len())
                            .map(|index| {
                                let held = instruction.register_at(index);
                                if in_class(target, "SPR", held) {
                                    1
                                } else if in_class(target, "DPR", held) {
                                    2
                                } else {
                                    0
                                }
                            })
                            .sum();
                    }
                    _ => {}
                }
                binary | if singles { count } else { count * 2 }
            } else {
                let mut binary = 0u64;
                for index in first..instruction.operands.len() {
                    binary |= 1 << register(index);
                }
                binary
            }
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
        OperandEncoder::BranchFuture(negative, kind) => match operand {
            MachineOperand::Immediate(value) => {
                let half = *value >> 1;
                (if *negative { -half } else { half }) as u64
            }
            MachineOperand::Expression(expression) => {
                add_fixup(expression, *kind);
                0
            }
            _ => return Err(unencodable("a branch target register")),
        },
        OperandEncoder::BranchFutureAfter => match (operand, instruction.operand(0)) {
            (MachineOperand::Expression(after), MachineOperand::Expression(branch)) => {
                let difference = Expression::Binary {
                    operator: BinaryOperator::Subtract,
                    left: Box::new(after.clone()),
                    right: Box::new(branch.clone()),
                };
                add_fixup(&difference, Fixup32::BfcselElse);
                0
            }
            (MachineOperand::Immediate(after), MachineOperand::Immediate(branch)) => {
                u64::from(after - branch == 4)
            }
            _ => return Err(unencodable("an else branch target")),
        },
        OperandEncoder::MvePairIndex(start) => (immediate(first) - start) as u64,
        OperandEncoder::MveShiftImmediate => {
            let record = target.instructions[instruction.opcode].record.as_str();
            let size = if record.contains("8") { 8 } else { 16 };
            (size + immediate(first)) as u64
        }
        OperandEncoder::MveRegisterQ => (register(first) << 3) | register(first + 1),
        OperandEncoder::MveQImmediate(shift) => {
            let held = immediate(first + 1) as i32;
            let add = held >= 0;
            let mut magnitude = held >> shift;
            if !add {
                magnitude = (magnitude as u32).wrapping_neg() as i32;
            }
            let mut result = (magnitude & 0x7f) as u64;
            if add {
                result |= 0x80;
            }
            (register(first) << 8) | result
        }
        OperandEncoder::VptMask => {
            let value = immediate(first) as u32;
            let mut result = 0u32;
            let mut previous = 0u32;
            for bit in (0..4).rev() {
                let held = (value >> bit) & 1;
                if value & !(!0u32 << bit) == 0 {
                    result |= 1 << bit;
                    break;
                }
                if held != previous {
                    result |= 1 << bit;
                }
                previous = held;
            }
            u64::from(result)
        }
        OperandEncoder::RestrictedCondition => u64::from(match immediate(first) {
            2 | 0 => 0u32,
            8 | 1 => 1,
            10 => 4,
            11 => 5,
            12 => 6,
            13 => 7,
            _ => 0,
        }),
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
                let (expression, specifier) = match expression {
                    Expression::Prefixed { prefix, operand } => (operand.as_ref(), prefix.clone()),
                    Expression::Symbol { specifier, .. } => {
                        (expression, specifier.clone().unwrap_or_default())
                    }
                    _ => (expression, String::new()),
                };
                let specifier = specifier
                    .split_once(':')
                    .map_or(specifier.clone(), |(prefix, _)| prefix.to_string());
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
    let mve = instruction.flags & MVE_MODE != 0;
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
            mve,
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
            "VLLDM" | "VLLDM_T2" | "VLSTM" | "VLSTM_T2" => Some(format!(
                "\t{}\t{}, {{d0 - d{}}}",
                if record.starts_with("VLL") {
                    "vlldm"
                } else {
                    "vlstm"
                },
                self.register_text(instruction, 0),
                if record.ends_with("T2") { 31 } else { 15 }
            )),
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
            "printMveSaturateOp" => out.push_str(if value(0) == 1 { "#48" } else { "#64" }),
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
                if self.features.contains("FeatureMClass") {
                    let mut sysm = value(0) as u32;
                    let write = self.target.instructions[instruction.opcode].record == "t2MSR_M";
                    if write && self.features.contains("FeatureDSP") {
                        if let Some(found) = system_registers().iter().find(|held| {
                            held.m1_encoding == sysm && held.required == ["FeatureDSP"]
                        }) {
                            out.push_str(&found.name);
                            return;
                        }
                    }
                    sysm &= 0xff;
                    if write && self.features.contains("HasV7Ops") {
                        if let Some(found) = system_registers()
                            .iter()
                            .find(|held| held.m2m3_encoding == ((1 << 9) | sysm))
                        {
                            out.push_str(&found.name);
                            return;
                        }
                    }
                    match system_registers()
                        .iter()
                        .find(|held| held.m2m3_encoding == ((1 << 8) | sysm))
                    {
                        Some(found) => out.push_str(&found.name),
                        None => out.push_str(&sysm.to_string()),
                    }
                    return;
                }
                let held = value(0) as u32;
                let spsr = held >> 4;
                let mask = held & 0xf;
                if spsr == 0 && matches!(mask, 8 | 4 | 12) {
                    out.push_str("APSR_");
                    if mask & 8 != 0 {
                        out.push_str("nzcvq");
                    }
                    if mask & 4 != 0 {
                        out.push('g');
                    }
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
            "printBankedRegOperand" => {
                let encoding = value(0) as u32;
                if let Some(found) = banked_registers()
                    .iter()
                    .find(|held| held.encoding == encoding)
                {
                    if encoding & 0x20 != 0 {
                        out.push_str("SPSR");
                        out.push_str(&found.name[4..]);
                    } else {
                        out.push_str(&found.name);
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
            "printVMOVModImmOperand" => {
                let encoded = value(0) as u64;
                let command = (encoded >> 8) & 0x1f;
                let immediate = encoded & 0xff;
                let held: u64 = if command == 0xe {
                    immediate
                } else if command & 0xc == 0x8 || command & 0x8 == 0 {
                    immediate << (8 * ((command & 0x6) >> 1))
                } else if command & 0xe == 0xc {
                    let byte = 1 + (command & 1);
                    (immediate << (8 * byte)) | (0xffff >> (8 * (2 - byte)))
                } else {
                    let mut held = 0u64;
                    for byte in 0..8 {
                        if (encoded >> byte) & 1 == 1 {
                            held |= 0xff << (8 * byte);
                        }
                    }
                    held
                };
                out.push_str(&format!("#0x{held:x}"));
            }
            "printRotImmOperand" => {
                if value(0) != 0 {
                    out.push_str(&format!(", ror #{}", 8 * value(0)));
                }
            }
            "printModImmOperand" => {
                if let MachineOperand::Expression(_) = operand {
                    self.call_method("printOperand", instruction, index, out);
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
            "printThumbITMask" | "printVPTMask" => {
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
            "printMVEVectorListTwoQ" | "printMVEVectorListFourQ" | "printMVEVectorList" => {
                let names: Vec<String> = self
                    .leaf_registers(instruction.register_at(index))
                    .iter()
                    .map(|member| self.register_name(*member))
                    .collect();
                out.push_str(&format!("{{{}}}", names.join(", ")));
            }
            "printMveAddrModeRQOperand" => {
                out.push_str(&format!("[{}, {}", register(0), register(1)));
                let shift = arguments
                    .first()
                    .and_then(|argument| argument.parse::<u32>().ok())
                    .unwrap_or(0);
                if shift > 0 {
                    out.push_str(&format!(", uxtw #{shift}"));
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

    fn leaf_registers(&self, register: RegisterId) -> Vec<RegisterId> {
        let held = &self.target.registers[register as usize];
        if held.sub_registers.is_empty() || !held.assembly_name.is_empty() {
            return vec![register];
        }
        held.sub_registers
            .iter()
            .flat_map(|member| self.leaf_registers(*member))
            .collect()
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

pub struct FpuEntry {
    pub name: &'static str,
    pub version: u8,
    pub neon: u8,
    pub restriction: u8,
}

pub const FPUS: &[FpuEntry] = &[
    FpuEntry {
        name: "invalid",
        version: 0,
        neon: 0,
        restriction: 0,
    },
    FpuEntry {
        name: "none",
        version: 0,
        neon: 0,
        restriction: 0,
    },
    FpuEntry {
        name: "vfp",
        version: 1,
        neon: 0,
        restriction: 0,
    },
    FpuEntry {
        name: "vfpv2",
        version: 1,
        neon: 0,
        restriction: 0,
    },
    FpuEntry {
        name: "vfpv3",
        version: 2,
        neon: 0,
        restriction: 0,
    },
    FpuEntry {
        name: "vfpv3-fp16",
        version: 3,
        neon: 0,
        restriction: 0,
    },
    FpuEntry {
        name: "vfpv3-d16",
        version: 2,
        neon: 0,
        restriction: 1,
    },
    FpuEntry {
        name: "vfpv3-d16-fp16",
        version: 3,
        neon: 0,
        restriction: 1,
    },
    FpuEntry {
        name: "vfpv3xd",
        version: 2,
        neon: 0,
        restriction: 2,
    },
    FpuEntry {
        name: "vfpv3xd-fp16",
        version: 3,
        neon: 0,
        restriction: 2,
    },
    FpuEntry {
        name: "vfpv4",
        version: 4,
        neon: 0,
        restriction: 0,
    },
    FpuEntry {
        name: "vfpv4-d16",
        version: 4,
        neon: 0,
        restriction: 1,
    },
    FpuEntry {
        name: "fpv4-sp-d16",
        version: 4,
        neon: 0,
        restriction: 2,
    },
    FpuEntry {
        name: "fpv5-d16",
        version: 5,
        neon: 0,
        restriction: 1,
    },
    FpuEntry {
        name: "fpv5-sp-d16",
        version: 5,
        neon: 0,
        restriction: 2,
    },
    FpuEntry {
        name: "fp-armv8",
        version: 5,
        neon: 0,
        restriction: 0,
    },
    FpuEntry {
        name: "fp-armv8-fullfp16-d16",
        version: 6,
        neon: 0,
        restriction: 1,
    },
    FpuEntry {
        name: "fp-armv8-fullfp16-sp-d16",
        version: 6,
        neon: 0,
        restriction: 2,
    },
    FpuEntry {
        name: "neon",
        version: 2,
        neon: 1,
        restriction: 0,
    },
    FpuEntry {
        name: "neon-fp16",
        version: 3,
        neon: 1,
        restriction: 0,
    },
    FpuEntry {
        name: "neon-vfpv4",
        version: 4,
        neon: 1,
        restriction: 0,
    },
    FpuEntry {
        name: "neon-fp-armv8",
        version: 5,
        neon: 1,
        restriction: 0,
    },
    FpuEntry {
        name: "crypto-neon-fp-armv8",
        version: 5,
        neon: 2,
        restriction: 0,
    },
    FpuEntry {
        name: "softvfp",
        version: 0,
        neon: 0,
        restriction: 0,
    },
];

pub struct ArchEntry {
    pub name: &'static str,
    pub id: &'static str,
    pub cpu_attr: &'static str,
    pub feature: &'static str,
    pub attribute: u32,
    pub fpu: u32,
    pub extensions: u64,
}

pub const ARCHES: &[ArchEntry] = &[
    ArchEntry {
        name: "invalid",
        id: "INVALID",
        cpu_attr: "",
        feature: "+",
        attribute: 0,
        fpu: 1,
        extensions: 1,
    },
    ArchEntry {
        name: "armv4",
        id: "ARMV4",
        cpu_attr: "4",
        feature: "+v4",
        attribute: 1,
        fpu: 1,
        extensions: 1,
    },
    ArchEntry {
        name: "armv4t",
        id: "ARMV4T",
        cpu_attr: "4T",
        feature: "+v4t",
        attribute: 2,
        fpu: 1,
        extensions: 1,
    },
    ArchEntry {
        name: "armv5t",
        id: "ARMV5T",
        cpu_attr: "5T",
        feature: "+v5",
        attribute: 3,
        fpu: 1,
        extensions: 1,
    },
    ArchEntry {
        name: "armv5te",
        id: "ARMV5TE",
        cpu_attr: "5TE",
        feature: "+v5e",
        attribute: 4,
        fpu: 1,
        extensions: 1024,
    },
    ArchEntry {
        name: "armv5tej",
        id: "ARMV5TEJ",
        cpu_attr: "5TEJ",
        feature: "+v5e",
        attribute: 5,
        fpu: 1,
        extensions: 1024,
    },
    ArchEntry {
        name: "armv6",
        id: "ARMV6",
        cpu_attr: "6",
        feature: "+v6",
        attribute: 6,
        fpu: 3,
        extensions: 1024,
    },
    ArchEntry {
        name: "armv6k",
        id: "ARMV6K",
        cpu_attr: "6K",
        feature: "+v6k",
        attribute: 9,
        fpu: 3,
        extensions: 1024,
    },
    ArchEntry {
        name: "armv6t2",
        id: "ARMV6T2",
        cpu_attr: "6T2",
        feature: "+v6t2",
        attribute: 8,
        fpu: 1,
        extensions: 1024,
    },
    ArchEntry {
        name: "armv6kz",
        id: "ARMV6KZ",
        cpu_attr: "6KZ",
        feature: "+v6kz",
        attribute: 7,
        fpu: 3,
        extensions: 1280,
    },
    ArchEntry {
        name: "armv6-m",
        id: "ARMV6M",
        cpu_attr: "6-M",
        feature: "+v6m",
        attribute: 11,
        fpu: 1,
        extensions: 1,
    },
    ArchEntry {
        name: "armv7-a",
        id: "ARMV7A",
        cpu_attr: "7-A",
        feature: "+v7",
        attribute: 10,
        fpu: 18,
        extensions: 1024,
    },
    ArchEntry {
        name: "armv7ve",
        id: "ARMV7VE",
        cpu_attr: "7VE",
        feature: "+v7ve",
        attribute: 10,
        fpu: 18,
        extensions: 1904,
    },
    ArchEntry {
        name: "armv7-r",
        id: "ARMV7R",
        cpu_attr: "7-R",
        feature: "+v7r",
        attribute: 10,
        fpu: 1,
        extensions: 1040,
    },
    ArchEntry {
        name: "armv7-m",
        id: "ARMV7M",
        cpu_attr: "7-M",
        feature: "+v7m",
        attribute: 10,
        fpu: 1,
        extensions: 16,
    },
    ArchEntry {
        name: "armv7e-m",
        id: "ARMV7EM",
        cpu_attr: "7E-M",
        feature: "+v7em",
        attribute: 13,
        fpu: 1,
        extensions: 1040,
    },
    ArchEntry {
        name: "armv8-a",
        id: "ARMV8A",
        cpu_attr: "8-A",
        feature: "+v8a",
        attribute: 14,
        fpu: 22,
        extensions: 1906,
    },
    ArchEntry {
        name: "armv8.1-a",
        id: "ARMV8_1A",
        cpu_attr: "8.1-A",
        feature: "+v8.1a",
        attribute: 14,
        fpu: 22,
        extensions: 1906,
    },
    ArchEntry {
        name: "armv8.2-a",
        id: "ARMV8_2A",
        cpu_attr: "8.2-A",
        feature: "+v8.2a",
        attribute: 14,
        fpu: 22,
        extensions: 6002,
    },
    ArchEntry {
        name: "armv8.3-a",
        id: "ARMV8_3A",
        cpu_attr: "8.3-A",
        feature: "+v8.3a",
        attribute: 14,
        fpu: 22,
        extensions: 6002,
    },
    ArchEntry {
        name: "armv8.4-a",
        id: "ARMV8_4A",
        cpu_attr: "8.4-A",
        feature: "+v8.4a",
        attribute: 14,
        fpu: 22,
        extensions: 14194,
    },
    ArchEntry {
        name: "armv8.5-a",
        id: "ARMV8_5A",
        cpu_attr: "8.5-A",
        feature: "+v8.5a",
        attribute: 14,
        fpu: 22,
        extensions: 14194,
    },
    ArchEntry {
        name: "armv8.6-a",
        id: "ARMV8_6A",
        cpu_attr: "8.6-A",
        feature: "+v8.6a",
        attribute: 14,
        fpu: 22,
        extensions: 3159922,
    },
    ArchEntry {
        name: "armv8.7-a",
        id: "ARMV8_7A",
        cpu_attr: "8.7-A",
        feature: "+v8.7a",
        attribute: 14,
        fpu: 22,
        extensions: 3159922,
    },
    ArchEntry {
        name: "armv8.8-a",
        id: "ARMV8_8A",
        cpu_attr: "8.8-A",
        feature: "+v8.8a",
        attribute: 14,
        fpu: 22,
        extensions: 3209074,
    },
    ArchEntry {
        name: "armv8.9-a",
        id: "ARMV8_9A",
        cpu_attr: "8.9-A",
        feature: "+v8.9a",
        attribute: 14,
        fpu: 22,
        extensions: 3209074,
    },
    ArchEntry {
        name: "armv9-a",
        id: "ARMV9A",
        cpu_attr: "9-A",
        feature: "+v9a",
        attribute: 22,
        fpu: 21,
        extensions: 14194,
    },
    ArchEntry {
        name: "armv9.1-a",
        id: "ARMV9_1A",
        cpu_attr: "9.1-A",
        feature: "+v9.1a",
        attribute: 22,
        fpu: 21,
        extensions: 3159922,
    },
    ArchEntry {
        name: "armv9.2-a",
        id: "ARMV9_2A",
        cpu_attr: "9.2-A",
        feature: "+v9.2a",
        attribute: 22,
        fpu: 21,
        extensions: 3159922,
    },
    ArchEntry {
        name: "armv9.3-a",
        id: "ARMV9_3A",
        cpu_attr: "9.3-A",
        feature: "+v9.3a",
        attribute: 22,
        fpu: 22,
        extensions: 3159922,
    },
    ArchEntry {
        name: "armv9.4-a",
        id: "ARMV9_4A",
        cpu_attr: "9.4-A",
        feature: "+v9.4a",
        attribute: 22,
        fpu: 21,
        extensions: 3159922,
    },
    ArchEntry {
        name: "armv9.5-a",
        id: "ARMV9_5A",
        cpu_attr: "9.5-A",
        feature: "+v9.5a",
        attribute: 22,
        fpu: 21,
        extensions: 3159922,
    },
    ArchEntry {
        name: "armv9.6-a",
        id: "ARMV9_6A",
        cpu_attr: "9.6-A",
        feature: "+v9.6a",
        attribute: 22,
        fpu: 21,
        extensions: 3159922,
    },
    ArchEntry {
        name: "armv9.7-a",
        id: "ARMV9_7A",
        cpu_attr: "9.7-A",
        feature: "+v9.7a",
        attribute: 22,
        fpu: 21,
        extensions: 3159922,
    },
    ArchEntry {
        name: "armv8-r",
        id: "ARMV8R",
        cpu_attr: "8-R",
        feature: "+v8r",
        attribute: 15,
        fpu: 14,
        extensions: 1650,
    },
    ArchEntry {
        name: "armv8-m.base",
        id: "ARMV8MBaseline",
        cpu_attr: "8-M.Baseline",
        feature: "+v8m.base",
        attribute: 16,
        fpu: 1,
        extensions: 16,
    },
    ArchEntry {
        name: "armv8-m.main",
        id: "ARMV8MMainline",
        cpu_attr: "8-M.Mainline",
        feature: "+v8m.main",
        attribute: 17,
        fpu: 13,
        extensions: 16,
    },
    ArchEntry {
        name: "armv8.1-m.main",
        id: "ARMV8_1MMainline",
        cpu_attr: "8.1-M.Mainline",
        feature: "+v8.1m.main",
        attribute: 21,
        fpu: 17,
        extensions: 528400,
    },
    ArchEntry {
        name: "iwmmxt",
        id: "IWMMXT",
        cpu_attr: "iwmmxt",
        feature: "+",
        attribute: 4,
        fpu: 1,
        extensions: 1,
    },
    ArchEntry {
        name: "iwmmxt2",
        id: "IWMMXT2",
        cpu_attr: "iwmmxt2",
        feature: "+",
        attribute: 4,
        fpu: 1,
        extensions: 1,
    },
    ArchEntry {
        name: "xscale",
        id: "XSCALE",
        cpu_attr: "xscale",
        feature: "+v5e",
        attribute: 4,
        fpu: 1,
        extensions: 1,
    },
    ArchEntry {
        name: "armv7s",
        id: "ARMV7S",
        cpu_attr: "7-S",
        feature: "+v7s",
        attribute: 10,
        fpu: 20,
        extensions: 1024,
    },
    ArchEntry {
        name: "armv7k",
        id: "ARMV7K",
        cpu_attr: "7-K",
        feature: "+v7k",
        attribute: 10,
        fpu: 1,
        extensions: 1024,
    },
];

pub struct ExtensionEntry {
    pub name: &'static str,
    pub id: u64,
    pub feature: &'static str,
    pub negative: &'static str,
}

pub const EXTENSIONS: &[ExtensionEntry] = &[
    ExtensionEntry {
        name: "invalid",
        id: 0,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "none",
        id: 1,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "crc",
        id: 2,
        feature: "+crc",
        negative: "-crc",
    },
    ExtensionEntry {
        name: "crypto",
        id: 4,
        feature: "+crypto",
        negative: "-crypto",
    },
    ExtensionEntry {
        name: "sha2",
        id: 16384,
        feature: "+sha2",
        negative: "-sha2",
    },
    ExtensionEntry {
        name: "aes",
        id: 32768,
        feature: "+aes",
        negative: "-aes",
    },
    ExtensionEntry {
        name: "dotprod",
        id: 8192,
        feature: "+dotprod",
        negative: "-dotprod",
    },
    ExtensionEntry {
        name: "dsp",
        id: 1024,
        feature: "+dsp",
        negative: "-dsp",
    },
    ExtensionEntry {
        name: "fp",
        id: 8,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "fp.dp",
        id: 262144,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "mve",
        id: 2147484672,
        feature: "+mve",
        negative: "-mve",
    },
    ExtensionEntry {
        name: "mve.fp",
        id: 2147484680,
        feature: "+mve.fp",
        negative: "-mve.fp",
    },
    ExtensionEntry {
        name: "idiv",
        id: 48,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "mp",
        id: 64,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "simd",
        id: 128,
        feature: "+neon",
        negative: "-neon",
    },
    ExtensionEntry {
        name: "sec",
        id: 256,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "virt",
        id: 512,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "fp16",
        id: 2048,
        feature: "+fullfp16",
        negative: "-fullfp16",
    },
    ExtensionEntry {
        name: "ras",
        id: 4096,
        feature: "+ras",
        negative: "-ras",
    },
    ExtensionEntry {
        name: "os",
        id: 576460752303423488,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "iwmmxt",
        id: 1152921504606846976,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "iwmmxt2",
        id: 2305843009213693952,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "maverick",
        id: 4611686018427387904,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "xscale",
        id: 9223372036854775808,
        feature: "",
        negative: "",
    },
    ExtensionEntry {
        name: "fp16fml",
        id: 65536,
        feature: "+fp16fml",
        negative: "-fp16fml",
    },
    ExtensionEntry {
        name: "bf16",
        id: 1048576,
        feature: "+bf16",
        negative: "-bf16",
    },
    ExtensionEntry {
        name: "sb",
        id: 131072,
        feature: "+sb",
        negative: "-sb",
    },
    ExtensionEntry {
        name: "i8mm",
        id: 2097152,
        feature: "+i8mm",
        negative: "-i8mm",
    },
    ExtensionEntry {
        name: "lob",
        id: 524288,
        feature: "+lob",
        negative: "-lob",
    },
    ExtensionEntry {
        name: "cdecp0",
        id: 4194304,
        feature: "+cdecp0",
        negative: "-cdecp0",
    },
    ExtensionEntry {
        name: "cdecp1",
        id: 8388608,
        feature: "+cdecp1",
        negative: "-cdecp1",
    },
    ExtensionEntry {
        name: "cdecp2",
        id: 16777216,
        feature: "+cdecp2",
        negative: "-cdecp2",
    },
    ExtensionEntry {
        name: "cdecp3",
        id: 33554432,
        feature: "+cdecp3",
        negative: "-cdecp3",
    },
    ExtensionEntry {
        name: "cdecp4",
        id: 67108864,
        feature: "+cdecp4",
        negative: "-cdecp4",
    },
    ExtensionEntry {
        name: "cdecp5",
        id: 134217728,
        feature: "+cdecp5",
        negative: "-cdecp5",
    },
    ExtensionEntry {
        name: "cdecp6",
        id: 268435456,
        feature: "+cdecp6",
        negative: "-cdecp6",
    },
    ExtensionEntry {
        name: "cdecp7",
        id: 536870912,
        feature: "+cdecp7",
        negative: "-cdecp7",
    },
    ExtensionEntry {
        name: "pacbti",
        id: 1073741824,
        feature: "+pacbti",
        negative: "-pacbti",
    },
];

pub struct CpuEntry {
    pub name: &'static str,
    pub arch: &'static str,
    pub fpu: u32,
    pub default: bool,
    pub extensions: u64,
}

pub const CPUS: &[CpuEntry] = &[
    CpuEntry {
        name: "arm8",
        arch: "ARMV4",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm810",
        arch: "ARMV4",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "strongarm",
        arch: "ARMV4",
        fpu: 1,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "strongarm110",
        arch: "ARMV4",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "strongarm1100",
        arch: "ARMV4",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "strongarm1110",
        arch: "ARMV4",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm7tdmi",
        arch: "ARMV4T",
        fpu: 1,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "arm7tdmi-s",
        arch: "ARMV4T",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm710t",
        arch: "ARMV4T",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm720t",
        arch: "ARMV4T",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm9",
        arch: "ARMV4T",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm9tdmi",
        arch: "ARMV4T",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm920",
        arch: "ARMV4T",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm920t",
        arch: "ARMV4T",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm922t",
        arch: "ARMV4T",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm940t",
        arch: "ARMV4T",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "ep9312",
        arch: "ARMV4T",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm10tdmi",
        arch: "ARMV5T",
        fpu: 1,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "arm1020t",
        arch: "ARMV5T",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm9e",
        arch: "ARMV5TE",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm946e-s",
        arch: "ARMV5TE",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm966e-s",
        arch: "ARMV5TE",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm968e-s",
        arch: "ARMV5TE",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm10e",
        arch: "ARMV5TE",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm1020e",
        arch: "ARMV5TE",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm1022e",
        arch: "ARMV5TE",
        fpu: 1,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "arm926ej-s",
        arch: "ARMV5TEJ",
        fpu: 1,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "arm1136j-s",
        arch: "ARMV6",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm1136jf-s",
        arch: "ARMV6",
        fpu: 3,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "mpcore",
        arch: "ARMV6K",
        fpu: 3,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "mpcorenovfp",
        arch: "ARMV6K",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm1176jz-s",
        arch: "ARMV6KZ",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "arm1176jzf-s",
        arch: "ARMV6KZ",
        fpu: 3,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "arm1156t2-s",
        arch: "ARMV6T2",
        fpu: 1,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "arm1156t2f-s",
        arch: "ARMV6T2",
        fpu: 3,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "cortex-m0",
        arch: "ARMV6M",
        fpu: 1,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "cortex-m0plus",
        arch: "ARMV6M",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "cortex-m1",
        arch: "ARMV6M",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "sc000",
        arch: "ARMV6M",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "cortex-a5",
        arch: "ARMV7A",
        fpu: 20,
        default: false,
        extensions: 320,
    },
    CpuEntry {
        name: "cortex-a7",
        arch: "ARMV7A",
        fpu: 20,
        default: false,
        extensions: 880,
    },
    CpuEntry {
        name: "cortex-a8",
        arch: "ARMV7A",
        fpu: 18,
        default: false,
        extensions: 256,
    },
    CpuEntry {
        name: "cortex-a9",
        arch: "ARMV7A",
        fpu: 19,
        default: false,
        extensions: 320,
    },
    CpuEntry {
        name: "cortex-a12",
        arch: "ARMV7A",
        fpu: 20,
        default: false,
        extensions: 880,
    },
    CpuEntry {
        name: "cortex-a15",
        arch: "ARMV7A",
        fpu: 20,
        default: false,
        extensions: 880,
    },
    CpuEntry {
        name: "cortex-a17",
        arch: "ARMV7A",
        fpu: 20,
        default: false,
        extensions: 880,
    },
    CpuEntry {
        name: "krait",
        arch: "ARMV7A",
        fpu: 20,
        default: false,
        extensions: 48,
    },
    CpuEntry {
        name: "cortex-r4",
        arch: "ARMV7R",
        fpu: 1,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "cortex-r4f",
        arch: "ARMV7R",
        fpu: 6,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "cortex-r5",
        arch: "ARMV7R",
        fpu: 6,
        default: false,
        extensions: 96,
    },
    CpuEntry {
        name: "cortex-r7",
        arch: "ARMV7R",
        fpu: 7,
        default: false,
        extensions: 96,
    },
    CpuEntry {
        name: "cortex-r8",
        arch: "ARMV7R",
        fpu: 7,
        default: false,
        extensions: 96,
    },
    CpuEntry {
        name: "cortex-r52",
        arch: "ARMV8R",
        fpu: 21,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "cortex-r52plus",
        arch: "ARMV8R",
        fpu: 21,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "sc300",
        arch: "ARMV7M",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "cortex-m3",
        arch: "ARMV7M",
        fpu: 1,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "cortex-m4",
        arch: "ARMV7EM",
        fpu: 12,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "cortex-m7",
        arch: "ARMV7EM",
        fpu: 13,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "cortex-m23",
        arch: "ARMV8MBaseline",
        fpu: 1,
        default: false,
        extensions: 1,
    },
    CpuEntry {
        name: "cortex-m33",
        arch: "ARMV8MMainline",
        fpu: 14,
        default: false,
        extensions: 1024,
    },
    CpuEntry {
        name: "star-mc1",
        arch: "ARMV8MMainline",
        fpu: 14,
        default: false,
        extensions: 1024,
    },
    CpuEntry {
        name: "cortex-m35p",
        arch: "ARMV8MMainline",
        fpu: 14,
        default: false,
        extensions: 1024,
    },
    CpuEntry {
        name: "cortex-m55",
        arch: "ARMV8_1MMainline",
        fpu: 16,
        default: false,
        extensions: 2147486728,
    },
    CpuEntry {
        name: "cortex-m85",
        arch: "ARMV8_1MMainline",
        fpu: 16,
        default: false,
        extensions: 3221232648,
    },
    CpuEntry {
        name: "cortex-m52",
        arch: "ARMV8_1MMainline",
        fpu: 16,
        default: false,
        extensions: 3221232648,
    },
    CpuEntry {
        name: "star-mc3",
        arch: "ARMV8_1MMainline",
        fpu: 16,
        default: false,
        extensions: 3221232648,
    },
    CpuEntry {
        name: "cortex-a32",
        arch: "ARMV8A",
        fpu: 22,
        default: false,
        extensions: 2,
    },
    CpuEntry {
        name: "cortex-a35",
        arch: "ARMV8A",
        fpu: 22,
        default: false,
        extensions: 2,
    },
    CpuEntry {
        name: "cortex-a53",
        arch: "ARMV8A",
        fpu: 22,
        default: false,
        extensions: 2,
    },
    CpuEntry {
        name: "cortex-a55",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 10240,
    },
    CpuEntry {
        name: "cortex-a57",
        arch: "ARMV8A",
        fpu: 22,
        default: false,
        extensions: 2,
    },
    CpuEntry {
        name: "cortex-a72",
        arch: "ARMV8A",
        fpu: 22,
        default: false,
        extensions: 2,
    },
    CpuEntry {
        name: "cortex-a73",
        arch: "ARMV8A",
        fpu: 22,
        default: false,
        extensions: 2,
    },
    CpuEntry {
        name: "cortex-a75",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 10240,
    },
    CpuEntry {
        name: "cortex-a76",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 10240,
    },
    CpuEntry {
        name: "cortex-a76ae",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 10240,
    },
    CpuEntry {
        name: "cortex-a77",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 10240,
    },
    CpuEntry {
        name: "cortex-a78",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 10240,
    },
    CpuEntry {
        name: "cortex-a78ae",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 12288,
    },
    CpuEntry {
        name: "cortex-a78c",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 10240,
    },
    CpuEntry {
        name: "cortex-a510",
        arch: "ARMV9A",
        fpu: 21,
        default: false,
        extensions: 3350528,
    },
    CpuEntry {
        name: "cortex-a710",
        arch: "ARMV9A",
        fpu: 21,
        default: false,
        extensions: 3350528,
    },
    CpuEntry {
        name: "cortex-x1",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 10240,
    },
    CpuEntry {
        name: "cortex-x1c",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 10240,
    },
    CpuEntry {
        name: "neoverse-n1",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 10240,
    },
    CpuEntry {
        name: "neoverse-n2",
        arch: "ARMV9A",
        fpu: 21,
        default: false,
        extensions: 3354624,
    },
    CpuEntry {
        name: "neoverse-v1",
        arch: "ARMV8_4A",
        fpu: 22,
        default: false,
        extensions: 1062912,
    },
    CpuEntry {
        name: "cyclone",
        arch: "ARMV8A",
        fpu: 22,
        default: false,
        extensions: 2,
    },
    CpuEntry {
        name: "exynos-m3",
        arch: "ARMV8A",
        fpu: 22,
        default: false,
        extensions: 2,
    },
    CpuEntry {
        name: "exynos-m4",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 10240,
    },
    CpuEntry {
        name: "exynos-m5",
        arch: "ARMV8_2A",
        fpu: 22,
        default: false,
        extensions: 10240,
    },
    CpuEntry {
        name: "kryo",
        arch: "ARMV8A",
        fpu: 22,
        default: false,
        extensions: 2,
    },
    CpuEntry {
        name: "iwmmxt",
        arch: "IWMMXT",
        fpu: 1,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "xscale",
        arch: "XSCALE",
        fpu: 1,
        default: true,
        extensions: 1,
    },
    CpuEntry {
        name: "swift",
        arch: "ARMV7S",
        fpu: 20,
        default: true,
        extensions: 48,
    },
    CpuEntry {
        name: "invalid",
        arch: "INVALID",
        fpu: 0,
        default: true,
        extensions: 0,
    },
];

pub const ATTRIBUTE_TAGS: &[(&str, u32)] = &[
    ("Tag_File", 1),
    ("Tag_Section", 2),
    ("Tag_Symbol", 3),
    ("Tag_CPU_raw_name", 4),
    ("Tag_CPU_name", 5),
    ("Tag_CPU_arch", 6),
    ("Tag_CPU_arch_profile", 7),
    ("Tag_ARM_ISA_use", 8),
    ("Tag_THUMB_ISA_use", 9),
    ("Tag_FP_arch", 10),
    ("Tag_WMMX_arch", 11),
    ("Tag_Advanced_SIMD_arch", 12),
    ("Tag_MVE_arch", 48),
    ("Tag_PCS_config", 13),
    ("Tag_ABI_PCS_R9_use", 14),
    ("Tag_ABI_PCS_RW_data", 15),
    ("Tag_ABI_PCS_RO_data", 16),
    ("Tag_ABI_PCS_GOT_use", 17),
    ("Tag_ABI_PCS_wchar_t", 18),
    ("Tag_ABI_FP_rounding", 19),
    ("Tag_ABI_FP_denormal", 20),
    ("Tag_ABI_FP_exceptions", 21),
    ("Tag_ABI_FP_user_exceptions", 22),
    ("Tag_ABI_FP_number_model", 23),
    ("Tag_ABI_align_needed", 24),
    ("Tag_ABI_align_preserved", 25),
    ("Tag_ABI_enum_size", 26),
    ("Tag_ABI_HardFP_use", 27),
    ("Tag_ABI_VFP_args", 28),
    ("Tag_ABI_WMMX_args", 29),
    ("Tag_ABI_optimization_goals", 30),
    ("Tag_ABI_FP_optimization_goals", 31),
    ("Tag_compatibility", 32),
    ("Tag_CPU_unaligned_access", 34),
    ("Tag_FP_HP_extension", 36),
    ("Tag_ABI_FP_16bit_format", 38),
    ("Tag_MPextension_use", 42),
    ("Tag_DIV_use", 44),
    ("Tag_DSP_extension", 46),
    ("Tag_PAC_extension", 50),
    ("Tag_BTI_extension", 52),
    ("Tag_BTI_use", 74),
    ("Tag_PACRET_use", 76),
    ("Tag_nodefaults", 64),
    ("Tag_also_compatible_with", 65),
    ("Tag_T2EE_use", 66),
    ("Tag_conformance", 67),
    ("Tag_Virtualization_use", 68),
    ("Tag_VFP_arch", 10),
    ("Tag_VFP_HP_extension", 36),
    ("Tag_ABI_align8_needed", 24),
    ("Tag_ABI_align8_preserved", 25),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeItem {
    pub tag: u32,
    pub number: Option<u32>,
    pub text: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct AttributeState {
    pub items: Vec<AttributeItem>,
    pub architecture: Option<usize>,
    pub object_architecture: Option<usize>,
    pub fpu: Option<usize>,
}

impl AttributeState {
    fn set(&mut self, item: AttributeItem, overwrite: bool) {
        if let Some(held) = self.items.iter_mut().find(|held| held.tag == item.tag) {
            if overwrite {
                *held = item;
            }
            return;
        }
        self.items.push(item);
    }

    fn set_number(&mut self, tag: u32, value: u32, overwrite: bool) {
        self.set(
            AttributeItem {
                tag,
                number: Some(value),
                text: None,
            },
            overwrite,
        );
    }

    fn set_text(&mut self, tag: u32, value: &str, overwrite: bool) {
        self.set(
            AttributeItem {
                tag,
                number: None,
                text: Some(value.to_string()),
            },
            overwrite,
        );
    }

    fn architecture_defaults(&mut self) {
        let Some(index) = self.architecture else {
            return;
        };
        let entry = &ARCHES[index];
        self.set_text(5, entry.cpu_attr, false);
        let emitted = self.object_architecture.unwrap_or(index);
        self.set_number(6, ARCHES[emitted].attribute, false);
        let (profile, arm, thumb): (u32, Option<u32>, Option<u32>) = match entry.id {
            "ARMV4" => (0, Some(1), None),
            "ARMV4T" | "ARMV5T" | "XSCALE" | "ARMV5TE" | "ARMV5TEJ" | "ARMV6" => {
                (0, Some(1), Some(1))
            }
            "ARMV6T2" => (0, Some(1), Some(2)),
            "ARMV6K" | "ARMV6KZ" => (0, Some(1), Some(1)),
            "ARMV6M" => (0, None, Some(1)),
            "ARMV7A" => (0x41, Some(1), Some(2)),
            "ARMV7R" => (0x52, Some(1), Some(2)),
            "ARMV7EM" | "ARMV7M" => (0x4d, None, Some(2)),
            "ARMV8MBaseline" | "ARMV8MMainline" | "ARMV8_1MMainline" => (0x4d, None, Some(3)),
            "IWMMXT" | "IWMMXT2" => (0, Some(1), Some(1)),
            id if id.starts_with("ARMV8_") || id.starts_with("ARMV9") || id == "ARMV8A" => {
                (0x41, Some(1), Some(2))
            }
            _ => (0, None, None),
        };
        if profile != 0 {
            self.set_number(7, profile, false);
        }
        if let Some(value) = arm {
            self.set_number(8, value, false);
        }
        if let Some(value) = thumb {
            self.set_number(9, value, false);
        }
        match entry.id {
            "ARMV6K" | "ARMV6KZ" => self.set_number(68, 1, false),
            "IWMMXT" => self.set_number(11, 1, false),
            "IWMMXT2" => self.set_number(11, 2, false),
            id if id == "ARMV8A"
                || id.starts_with("ARMV8_") && !id.contains("MMainline")
                || id.starts_with("ARMV9") =>
            {
                self.set_number(42, 1, false);
                self.set_number(68, 3, false);
            }
            _ => {}
        }
    }

    fn fpu_defaults(&mut self) {
        let Some(index) = self.fpu else {
            return;
        };
        let name = FPUS[index].name;
        let (arch, simd, half): (Option<u32>, Option<u32>, bool) = match name {
            "vfp" | "vfpv2" => (Some(2), None, false),
            "vfpv3" => (Some(3), None, false),
            "vfpv3-fp16" => (Some(3), None, true),
            "vfpv3-d16" | "vfpv3xd" => (Some(4), None, false),
            "vfpv3-d16-fp16" | "vfpv3xd-fp16" => (Some(4), None, true),
            "vfpv4" => (Some(5), None, false),
            "fpv4-sp-d16" | "vfpv4-d16" => (Some(6), None, false),
            "fp-armv8" => (Some(7), None, false),
            "fpv5-sp-d16" | "fpv5-d16" | "fp-armv8-fullfp16-sp-d16" | "fp-armv8-fullfp16-d16" => {
                (Some(8), None, false)
            }
            "neon" => (Some(3), Some(1), false),
            "neon-fp16" => (Some(3), Some(1), true),
            "neon-vfpv4" => (Some(5), Some(2), false),
            "neon-fp-armv8" | "crypto-neon-fp-armv8" => (Some(7), None, false),
            _ => (None, None, false),
        };
        if let Some(value) = arch {
            self.set_number(10, value, false);
        }
        if let Some(value) = simd {
            self.set_number(12, value, false);
        }
        if half {
            self.set_number(36, 1, false);
        }
    }

    pub fn encode(&mut self) -> Vec<u8> {
        self.fpu_defaults();
        self.architecture_defaults();
        if self.items.is_empty() {
            return Vec::new();
        }
        let mut items = self.items.clone();
        items.sort_by_key(|item| (item.tag != 67, item.tag));
        let mut body: Vec<u8> = Vec::new();
        for item in &items {
            uleb(&mut body, u64::from(item.tag));
            if let Some(number) = item.number {
                uleb(&mut body, u64::from(number));
            }
            if let Some(text) = &item.text {
                body.extend_from_slice(text.as_bytes());
                body.push(0);
            }
        }
        let vendor = b"aeabi\0";
        let mut out: Vec<u8> = vec![0x41];
        let length = (4 + vendor.len() + 5 + body.len()) as u32;
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(vendor);
        out.push(1);
        out.extend_from_slice(&((5 + body.len()) as u32).to_le_bytes());
        out.extend_from_slice(&body);
        out
    }
}

fn uleb(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

fn canonical_architecture(text: &str) -> String {
    let mut rest = text;
    let mut offset = None;
    for (prefix, length) in [
        ("arm64_32", 8),
        ("arm64e", 6),
        ("arm64", 5),
        ("aarch64_32", 10),
        ("arm", 3),
        ("thumb", 5),
    ] {
        if text.starts_with(prefix) {
            offset = Some(length);
            break;
        }
    }
    if let Some(length) = offset {
        rest = &text[length..];
        if let Some(stripped) = rest.strip_prefix("eb") {
            rest = stripped;
        }
    } else if let Some(stripped) = rest.strip_suffix("eb") {
        rest = stripped;
    }
    if rest.is_empty() {
        return text.to_string();
    }
    if offset.is_some() {
        let bytes = rest.as_bytes();
        if bytes.len() >= 2 && (bytes[0] != b'v' || !bytes[1].is_ascii_digit()) {
            return String::new();
        }
        if rest.contains("eb") {
            return String::new();
        }
    }
    rest.to_string()
}

fn architecture_synonym(text: &str) -> &str {
    match text {
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

impl<'a> Assembler<'a> {
    pub(super) fn arm32_mark_thumb_function(&mut self, symbol: usize) {
        self.context.symbols[symbol].thumb_function = true;
        self.context.symbols[symbol].symbol_type = Some(SymbolType::Function);
        self.streamer.emit_arm_directive("\t.thumb_func");
    }

    fn arm32_directive_thumb_set(&mut self) -> bool {
        let Some(name) = self.parse_identifier() else {
            return self.token_error("expected identifier after '.thumb_set'");
        };
        if self.parse_token(TokenKind::Comma, "expected comma") {
            return true;
        }
        let Ok((expression, _)) = self.parse_expression() else {
            return self.token_error("missing expression");
        };
        if self.parse_end_of_line() {
            return true;
        }
        let symbol = self.context.symbol(&name);
        let defined_source = match &expression {
            Expression::Symbol { name: source, .. } => self
                .context
                .lookup(source)
                .is_some_and(|held| !self.context.symbols[held].is_undefined()),
            _ => true,
        };
        let plain = matches!(expression, Expression::Symbol { .. });
        let text = expression_text(self.target, &expression, 0, Syntax::Att);
        self.streamer
            .emit_arm_directive(&format!("\t.thumb_set\t{name}, {text}"));
        if plain && !defined_source {
            self.streamer
                .emit_assignment(&mut self.context, symbol, &expression);
            return false;
        }
        self.context.symbols[symbol].thumb_function = true;
        self.context.symbols[symbol].symbol_type = Some(SymbolType::Function);
        self.streamer
            .emit_assignment(&mut self.context, symbol, &expression);
        false
    }

    fn arm32_directive_inst(&mut self, location: SourceLocation, suffix: char) -> bool {
        if self.arm_thumb {
            if !matches!(suffix, '\0' | 'n' | 'w') {
                return true;
            }
        } else if suffix != '\0' {
            return self.error(location, "width suffixes are invalid in ARM mode");
        }
        if self.is(TokenKind::EndOfStatement) {
            return self.error(location, "expected expression following directive");
        }
        self.parse_many(|parser| {
            let Ok((expression, _)) = parser.parse_expression() else {
                return true;
            };
            let Some(value) = parser.context.evaluate_absolute(&expression, None) else {
                return parser.error(location, "expected constant expression");
            };
            let mut current = suffix;
            if parser.arm_thumb {
                match suffix {
                    'n' if value > 0xffff => {
                        return parser
                            .error(location, "inst.n operand is too big, use inst.w instead");
                    }
                    'w' if value > 0xffff_ffff => {
                        return parser.error(location, "inst.w operand is too big");
                    }
                    '\0' => {
                        if value < 0xe800 {
                            current = 'n';
                        } else if value >= 0xe800_0000 {
                            current = 'w';
                        } else {
                            return parser.error(
                                location,
                                "cannot determine Thumb instruction size, use inst.n/inst.w instead",
                            );
                        }
                    }
                    _ => {}
                }
            } else if value > 0xffff_ffff {
                return parser.error(location, "inst operand is too big");
            }
            parser
                .streamer
                .emit_arm32_inst(&mut parser.context, value, current, location);
            parser.arm_it.forward();
            false
        })
    }
}

pub fn parse_architecture(text: &str) -> Option<usize> {
    let canonical = canonical_architecture(text);
    let synonym = architecture_synonym(&canonical);
    if synonym.is_empty() {
        return None;
    }
    ARCHES
        .iter()
        .position(|entry| entry.name != "invalid" && entry.name.ends_with(synonym))
}

fn fpu_synonym(text: &str) -> &str {
    match text {
        "fpa" | "fpe2" | "fpe3" | "maverick" => "invalid",
        "vfp2" => "vfpv2",
        "vfp3" => "vfpv3",
        "vfp4" => "vfpv4",
        "vfp3-d16" => "vfpv3-d16",
        "vfp4-d16" => "vfpv4-d16",
        "fp4-sp-d16" | "vfpv4-sp-d16" => "fpv4-sp-d16",
        "fp4-dp-d16" | "fpv4-dp-d16" => "vfpv4-d16",
        "fp5-sp-d16" => "fpv5-sp-d16",
        "fp5-dp-d16" | "fpv5-dp-d16" => "fpv5-d16",
        "neon-vfpv3" => "neon",
        other => other,
    }
}

pub fn parse_fpu(text: &str) -> Option<usize> {
    let synonym = fpu_synonym(text);
    FPUS.iter()
        .position(|entry| entry.name == synonym && entry.name != "invalid")
}

fn fpu_features(index: usize) -> Vec<&'static str> {
    const FEATURES: &[(&str, &str, u8, u8)] = &[
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
    let entry = &FPUS[index];
    let mut found: Vec<&'static str> = Vec::new();
    for (plus, minus, version, restriction) in FEATURES {
        if entry.version >= *version && entry.restriction <= *restriction {
            found.push(plus);
        } else {
            found.push(minus);
        }
    }
    for (plus, minus, level) in [
        ("+neon", "-neon", 1),
        ("+sha2", "-sha2", 2),
        ("+aes", "-aes", 2),
    ] {
        if entry.neon >= level {
            found.push(plus);
        } else {
            found.push(minus);
        }
    }
    found
}

fn tag_name(tag: u32) -> Option<&'static str> {
    ATTRIBUTE_TAGS
        .iter()
        .find(|(_, value)| *value == tag)
        .map(|(name, _)| *name)
}

fn tag_from_name(name: &str) -> Option<u32> {
    let wanted = name.strip_prefix("Tag_").unwrap_or(name);
    ATTRIBUTE_TAGS
        .iter()
        .find(|(held, _)| held.strip_prefix("Tag_").unwrap_or(held) == wanted)
        .map(|(_, value)| *value)
}

struct ExtensionRequirement {
    kind: u64,
    check: &'static [&'static str],
    features: &'static [&'static str],
}

const EXTENSION_REQUIREMENTS: &[ExtensionRequirement] = &[
    ExtensionRequirement {
        kind: 1 << 1,
        check: &["HasV8"],
        features: &["crc"],
    },
    ExtensionRequirement {
        kind: 1 << 15,
        check: &["HasV8"],
        features: &["aes", "neon", "fp-armv8"],
    },
    ExtensionRequirement {
        kind: 1 << 14,
        check: &["HasV8"],
        features: &["sha2", "neon", "fp-armv8"],
    },
    ExtensionRequirement {
        kind: 1 << 2,
        check: &["HasV8"],
        features: &["crypto", "neon", "fp-armv8"],
    },
    ExtensionRequirement {
        kind: (1 << 10) | (1 << 31) | (1 << 3),
        check: &["HasV8_1MMainline"],
        features: &["mve.fp"],
    },
    ExtensionRequirement {
        kind: 1 << 3,
        check: &["HasV8"],
        features: &["vfp2sp", "fp-armv8"],
    },
    ExtensionRequirement {
        kind: (1 << 4) | (1 << 5),
        check: &["HasV7", "IsNotMClass"],
        features: &["hwdiv", "hwdiv-arm"],
    },
    ExtensionRequirement {
        kind: 1 << 6,
        check: &["HasV7", "IsNotMClass"],
        features: &["mp"],
    },
    ExtensionRequirement {
        kind: 1 << 7,
        check: &["HasV8"],
        features: &["neon", "vfp2sp", "fp-armv8"],
    },
    ExtensionRequirement {
        kind: 1 << 8,
        check: &["HasV6K"],
        features: &["trustzone"],
    },
    ExtensionRequirement {
        kind: 1 << 9,
        check: &["HasV7"],
        features: &["virtualization"],
    },
    ExtensionRequirement {
        kind: 1 << 11,
        check: &["HasV8_2a"],
        features: &["fp-armv8", "fullfp16"],
    },
    ExtensionRequirement {
        kind: 1 << 12,
        check: &["HasV8"],
        features: &["ras"],
    },
    ExtensionRequirement {
        kind: 1 << 19,
        check: &["HasV8_1MMainline"],
        features: &["lob"],
    },
    ExtensionRequirement {
        kind: 1 << 30,
        check: &["HasV8_1MMainline"],
        features: &["pacbti"],
    },
    ExtensionRequirement {
        kind: 1 << 59,
        check: &[],
        features: &[],
    },
    ExtensionRequirement {
        kind: 1 << 60,
        check: &[],
        features: &[],
    },
    ExtensionRequirement {
        kind: 1 << 61,
        check: &[],
        features: &[],
    },
    ExtensionRequirement {
        kind: 1 << 62,
        check: &[],
        features: &[],
    },
    ExtensionRequirement {
        kind: 1 << 63,
        check: &[],
        features: &[],
    },
];

impl<'a> Assembler<'a> {
    fn arm32_apply_features(&mut self, features: &[&str]) {
        for feature in features {
            if let Some(name) = feature.strip_prefix('+') {
                self.arm32_set_features(&[name], &[]);
            } else if let Some(name) = feature.strip_prefix('-') {
                self.arm32_set_features(&[], &[name]);
            }
        }
    }

    fn arm32_fix_mode(&mut self, was_thumb: bool, location: SourceLocation) {
        let thumb_now = self
            .arm32_feature("thumb-mode")
            .is_some_and(|held| self.arm_features.contains(&held));
        if was_thumb == thumb_now {
            return;
        }
        let supports_thumb = self
            .arm32_feature("v4t")
            .is_some_and(|held| self.arm_features.contains(&held));
        let supports_arm = self
            .arm32_feature("noarm")
            .is_none_or(|held| !self.arm_features.contains(&held));
        if (was_thumb && supports_thumb) || (!was_thumb && supports_arm) {
            self.arm_thumb = was_thumb;
            self.arm32_switch_mode();
            return;
        }
        self.arm_thumb = thumb_now;
        self.streamer.emit_arm_directive(if thumb_now {
            "\t.code\t16"
        } else {
            "\t.code\t32"
        });
        self.warning(
            location,
            &format!(
                "new target does not support {} mode, switching to {} mode",
                if was_thumb { "thumb" } else { "arm" },
                if !was_thumb { "thumb" } else { "arm" }
            ),
        );
    }

    fn arm32_directive_arch(&mut self, location: SourceLocation) -> bool {
        let text = self.parse_string_to_end_of_statement().trim().to_string();
        let Some(index) = parse_architecture(&text) else {
            return self.error(location, "Unknown arch name");
        };
        let was_thumb = self.arm_thumb;
        let name = ARCHES[index].name;
        self.arm_features = armv8::feature_set(self.target, "", &[format!("+{name}")]);
        self.streamer.set_arm_features(&self.arm_features);
        self.arm32_fix_mode(was_thumb, location);
        self.arm_attribute_state.architecture = Some(index);
        self.streamer
            .emit_arm_directive(&format!("\t.arch\t{name}"));
        false
    }

    fn arm32_directive_cpu(&mut self, location: SourceLocation) -> bool {
        let name = self.parse_string_to_end_of_statement().trim().to_string();
        self.arm_attribute_state.set_text(5, &name, true);
        self.streamer
            .emit_arm_directive(&format!("\t.cpu\t{}", name.to_ascii_lowercase()));
        let target = self.target;
        let known = name == "generic"
            || target.records.values().any(|record| {
                (record.is_a("ProcessorModel") || record.is_a("Processor"))
                    && table_text(record, "Name") == name
            });
        if !known {
            return self.error(location, "Unknown CPU name");
        }
        let was_thumb = self.arm_thumb;
        self.arm_features = armv8::feature_set(target, &name, &[]);
        self.streamer.set_arm_features(&self.arm_features);
        self.arm32_fix_mode(was_thumb, location);
        false
    }

    fn arm32_directive_fpu(&mut self) -> bool {
        let at = self.location();
        let name = self.parse_string_to_end_of_statement().trim().to_string();
        let Some(index) = parse_fpu(&name) else {
            return self.error(at, "Unknown FPU name");
        };
        self.arm32_apply_features(&fpu_features(index));
        self.arm_attribute_state.fpu = Some(index);
        self.streamer
            .emit_arm_directive(&format!("\t.fpu\t{}", FPUS[index].name));
        false
    }

    fn arm32_enable_extension(&mut self, name: &str, at: SourceLocation) -> bool {
        let (enable, bare) = match name.get(..2) {
            Some(prefix) if prefix.eq_ignore_ascii_case("no") => (false, &name[2..]),
            _ => (true, name),
        };
        let Some(entry) = EXTENSIONS.iter().find(|held| held.name == bare) else {
            return self.error(at, &format!("unknown architectural extension: {bare}"));
        };
        if entry.name == "invalid" {
            return self.error(at, &format!("unknown architectural extension: {bare}"));
        }
        let kind = entry.id;
        let Some(requirement) = EXTENSION_REQUIREMENTS.iter().find(|held| held.kind == kind) else {
            return self.error(at, &format!("unknown architectural extension: {bare}"));
        };
        if requirement.features.is_empty() {
            return self.error(at, &format!("unsupported architectural extension: {bare}"));
        }
        let check: Vec<String> = requirement
            .check
            .iter()
            .map(|held| held.to_string())
            .collect();
        if !armv8::predicates_hold(self.target, &check, &self.arm_features) {
            return self.error(
                at,
                &format!(
                    "architectural extension '{bare}' is not allowed for the current base architecture"
                ),
            );
        }
        if enable {
            self.arm32_set_features(requirement.features, &[]);
        } else {
            self.arm32_set_features(&[], requirement.features);
        }
        false
    }

    fn arm32_directive_arch_extension(&mut self) -> bool {
        if !self.is(TokenKind::Identifier) {
            let at = self.location();
            return self.error(at, "expected architecture extension name");
        }
        let name = self.lexer.token.text.clone();
        let at = self.location();
        self.lex();
        if self.parse_end_of_line() {
            return true;
        }
        if name == "nocrypto" {
            self.arm32_enable_extension("nosha2", at);
            self.arm32_enable_extension("noaes", at);
        }
        self.arm32_enable_extension(&name, at)
    }

    fn arm32_directive_object_arch(&mut self) -> bool {
        if !self.is(TokenKind::Identifier) {
            let at = self.location();
            return self.error(at, "unexpected token");
        }
        let name = self.lexer.token.text.clone();
        let at = self.location();
        self.lex();
        let Some(index) = parse_architecture(&name) else {
            return self.error(at, &format!("unknown architecture '{name}'"));
        };
        if !self.is(TokenKind::EndOfStatement) {
            return self.token_error("unexpected token");
        }
        self.lex();
        self.arm_attribute_state.object_architecture = Some(index);
        self.streamer
            .emit_arm_directive(&format!("\t.object_arch\t{}", ARCHES[index].name));
        false
    }

    fn arm32_directive_eabi_attribute(&mut self) -> bool {
        let tag_location = self.location();
        let tag: i64;
        if self.is(TokenKind::Identifier) {
            let name = self.lexer.token.text.clone();
            let Some(found) = tag_from_name(&name) else {
                return self.error(
                    tag_location,
                    &format!("attribute name not recognised: {name}"),
                );
            };
            tag = i64::from(found);
            self.lex();
        } else {
            let Ok((expression, _)) = self.parse_expression() else {
                return true;
            };
            let Expression::Constant(value) = expression else {
                return self.error(tag_location, "expected numeric constant");
            };
            tag = value;
        }
        if self.parse_token(TokenKind::Comma, "expected comma") {
            return true;
        }
        let (is_text, is_number) = if tag == 4 || tag == 5 {
            (true, false)
        } else if tag == 32 {
            (true, true)
        } else if tag < 32 || tag % 2 == 0 {
            (false, true)
        } else {
            (true, false)
        };
        let mut number = 0i64;
        if is_number {
            let at = self.location();
            let Ok((expression, _)) = self.parse_expression() else {
                return true;
            };
            let Expression::Constant(value) = expression else {
                return self.error(at, "expected numeric constant");
            };
            number = value;
        }
        if tag == 32 && self.parse_token(TokenKind::Comma, "expected comma") {
            return true;
        }
        let mut text = String::new();
        if is_text {
            if !self.is(TokenKind::String) {
                let at = self.location();
                return self.error(at, "bad string constant");
            }
            if tag == 65 {
                let Ok(bytes) = self.parse_escaped_string() else {
                    return true;
                };
                text = bytes.iter().map(|byte| char::from(*byte)).collect();
            } else {
                text = string_contents(&self.lexer.token.text);
                self.lex();
            }
        }
        if self.parse_end_of_line() {
            return true;
        }
        let tag = tag as u32;
        let comment = tag_name(tag)
            .map(|name| format!("\t@ {name}"))
            .unwrap_or_default();
        if is_text && is_number {
            self.arm_attribute_state.set(
                AttributeItem {
                    tag,
                    number: Some(number as u32),
                    text: Some(text.clone()),
                },
                true,
            );
            let mut line = format!("\t.eabi_attribute\t{tag}, {}", number as u32);
            if !text.is_empty() {
                line.push_str(&format!(", \"{text}\""));
            }
            line.push_str(&comment);
            self.streamer.emit_arm_directive(&line);
        } else if is_number {
            self.arm_attribute_state
                .set_number(tag, number as u32, true);
            self.streamer.emit_arm_directive(&format!(
                "\t.eabi_attribute\t{tag}, {}{comment}",
                number as u32
            ));
        } else {
            self.arm_attribute_state.set_text(tag, &text, true);
            if tag == 5 {
                self.streamer
                    .emit_arm_directive(&format!("\t.cpu\t{}", text.to_ascii_lowercase()));
            } else {
                let shown = if tag == 65 {
                    let mut escaped = String::new();
                    for character in text.chars() {
                        match character {
                            '\\' => escaped.push_str("\\\\"),
                            '"' => escaped.push_str("\\\""),
                            '\n' => escaped.push_str("\\n"),
                            '\t' => escaped.push_str("\\t"),
                            ' '..='~' => escaped.push(character),
                            other => escaped.push_str(&format!("\\{:03o}", other as u32 & 0xff)),
                        }
                    }
                    escaped
                } else {
                    text.clone()
                };
                self.streamer
                    .emit_arm_directive(&format!("\t.eabi_attribute\t{tag}, \"{shown}\"{comment}"));
            }
        }
        false
    }

    pub(super) fn arm32_finish_attributes(&mut self) {
        let bytes = self.arm_attribute_state.encode();
        self.streamer.emit_arm32_attributes(&bytes);
    }
}

#[derive(Clone, Debug, Default)]
pub struct UnwindOpcodes {
    operations: Vec<u8>,
    begins: Vec<usize>,
    has_personality: bool,
}

impl UnwindOpcodes {
    fn begin(&mut self) {
        if self.begins.is_empty() {
            self.begins.push(0);
        }
    }

    fn reset(&mut self) {
        self.operations.clear();
        self.begins.clear();
        self.begins.push(0);
        self.has_personality = false;
    }

    fn eight(&mut self, value: u32) {
        self.begin();
        self.operations.push((value & 0xff) as u8);
        let last = *self.begins.last().unwrap_or(&0);
        self.begins.push(last + 1);
    }

    fn sixteen(&mut self, value: u32) {
        self.begin();
        self.operations.push(((value >> 8) & 0xff) as u8);
        self.operations.push((value & 0xff) as u8);
        let last = *self.begins.last().unwrap_or(&0);
        self.begins.push(last + 2);
    }

    fn bytes(&mut self, bytes: &[u8]) {
        self.begin();
        self.operations.extend_from_slice(bytes);
        let last = *self.begins.last().unwrap_or(&0);
        self.begins.push(last + bytes.len());
    }

    fn register_save(&mut self, mask: u32) {
        let mut mask_value = mask;
        if mask_value == 0 {
            self.eight(0xb4);
            return;
        }
        if mask_value & (1 << 4) != 0 {
            let mut window = mask_value & 0xff0;
            let range = (window >> 5).trailing_ones();
            window &= !(0xffff_ffe0u32 << range);
            let unmasked = mask_value & 0xfff0 & !window;
            if unmasked == 0 {
                self.eight(0xa0 | range);
                mask_value &= 0x000f;
            } else if unmasked == 1 << 14 {
                self.eight(0xa8 | range);
                mask_value &= 0x000f;
            }
        }
        if mask_value & 0xfff0 != 0 {
            self.sixteen(0x8000 | (mask_value >> 4));
        }
        if mask_value & 0x000f != 0 {
            self.sixteen(0xb100 | (mask_value & 0x000f));
        }
    }

    fn vfp_register_save(&mut self, mask: u32) {
        for half in [mask & 0xffff_0000, mask & 0x0000_ffff] {
            let mut regs = half;
            while regs != 0 {
                let most = 32 - regs.leading_zeros();
                let length = (regs << (32 - most)).leading_ones();
                let least = most - length;
                let opcode = if least >= 16 { 0xc800 } else { 0xc900 };
                self.sixteen(opcode | ((least % 16) << 4) | (length - 1));
                regs &= !(u32::MAX << least);
            }
        }
    }

    fn set_stack_pointer(&mut self, register: u32) {
        self.eight(0x90 | register);
    }

    fn stack_offset(&mut self, offset: i64) {
        let mut offset = offset;
        if offset > 0x200 {
            let mut buffer = vec![0xb2u8];
            uleb(&mut buffer, ((offset - 0x204) >> 2) as u64);
            self.bytes(&buffer);
        } else if offset > 0 {
            if offset > 0x100 {
                self.eight(0x3f);
                offset -= 0x100;
            }
            self.eight(((offset - 4) >> 2) as u32 & 0xff);
        } else if offset < 0 {
            while offset < -0x100 {
                self.eight(0x40 | 0x3f);
                offset += 0x100;
            }
            self.eight(0x40 | ((((-offset) - 4) >> 2) as u32 & 0xff));
        }
    }

    fn finalize(&mut self, personality_index: &mut u32, result: &mut Vec<u8>) {
        self.begin();
        result.clear();
        let mut position = 3usize;
        let emit = |result: &mut Vec<u8>, position: &mut usize, byte: u8| {
            if *position < result.len() {
                result[*position] = byte;
            }
            *position = ((*position ^ 3) + 1) ^ 3;
        };
        let size_byte = |size: usize| -> u8 { (size.div_ceil(4) - 1) as u8 };
        if self.has_personality {
            *personality_index = 4;
            let rounded = (self.operations.len() + 1).div_ceil(4) * 4;
            result.resize(rounded, 0);
            emit(result, &mut position, size_byte(rounded));
        } else {
            if *personality_index == 4 {
                *personality_index = if self.operations.len() <= 3 { 0 } else { 1 };
            }
            if *personality_index == 0 {
                result.resize(4, 0);
                emit(result, &mut position, 0x80);
            } else {
                let rounded = (self.operations.len() + 2).div_ceil(4) * 4;
                result.resize(rounded, 0);
                emit(result, &mut position, 0x80 | *personality_index as u8);
                emit(result, &mut position, size_byte(rounded));
            }
        }
        for index in (1..self.begins.len()).rev() {
            for at in self.begins[index - 1]..self.begins[index] {
                emit(result, &mut position, self.operations[at]);
            }
        }
        while position < result.len() {
            emit(result, &mut position, 0xb0);
        }
        self.reset();
    }
}

#[derive(Clone, Debug)]
pub struct UnwindState {
    function_start: Option<usize>,
    function_section: Option<(usize, i64)>,
    exception_table: Option<usize>,
    personality: Option<String>,
    personality_index: u32,
    frame_register: Option<RegisterId>,
    frame_offset: i64,
    stack_offset: i64,
    pending_offset: i64,
    used_frame: bool,
    cannot_unwind: bool,
    opcodes: Vec<u8>,
    assembler: UnwindOpcodes,
    start_locations: Vec<SourceLocation>,
    cannot_unwind_locations: Vec<SourceLocation>,
    personality_locations: Vec<SourceLocation>,
    index_locations: Vec<SourceLocation>,
    handler_locations: Vec<SourceLocation>,
    context_frame_register: Option<RegisterId>,
}

impl Default for UnwindState {
    fn default() -> UnwindState {
        UnwindState {
            function_start: None,
            function_section: None,
            exception_table: None,
            personality: None,
            personality_index: 4,
            frame_register: None,
            frame_offset: 0,
            stack_offset: 0,
            pending_offset: 0,
            used_frame: false,
            cannot_unwind: false,
            opcodes: Vec::new(),
            assembler: UnwindOpcodes::default(),
            start_locations: Vec::new(),
            cannot_unwind_locations: Vec::new(),
            personality_locations: Vec::new(),
            index_locations: Vec::new(),
            handler_locations: Vec::new(),
            context_frame_register: None,
        }
    }
}

impl UnwindState {
    fn has_personality(&self) -> bool {
        !(self.personality_locations.is_empty() && self.index_locations.is_empty())
    }

    fn reset_context(&mut self) {
        self.start_locations.clear();
        self.cannot_unwind_locations.clear();
        self.personality_locations.clear();
        self.handler_locations.clear();
        self.index_locations.clear();
        self.context_frame_register = None;
    }

    fn reset_streamer(&mut self) {
        self.exception_table = None;
        self.function_start = None;
        self.personality = None;
        self.personality_index = 4;
        self.frame_register = None;
        self.frame_offset = 0;
        self.stack_offset = 0;
        self.pending_offset = 0;
        self.used_frame = false;
        self.cannot_unwind = false;
        self.opcodes.clear();
        self.assembler.reset();
    }
}

impl<'a> Assembler<'a> {
    fn arm32_note(&mut self, location: SourceLocation, text: &str) {
        let message = AssemblyMessage {
            kind: MessageKind::Note,
            location: Some(location),
            range: None,
            text: text.to_string(),
            marker: self.context.line_marker.clone(),
        };
        if self.pending_errors.is_empty() {
            self.context.messages.push(message);
        } else {
            self.pending_errors.push(message);
        }
    }

    fn arm32_note_start(&mut self) {
        for location in self.arm_unwind.start_locations.clone() {
            self.arm32_note(location, ".fnstart was specified here");
        }
    }

    fn arm32_note_cannot_unwind(&mut self) {
        for location in self.arm_unwind.cannot_unwind_locations.clone() {
            self.arm32_note(location, ".cantunwind was specified here");
        }
    }

    fn arm32_note_handler(&mut self) {
        for location in self.arm_unwind.handler_locations.clone() {
            self.arm32_note(location, ".handlerdata was specified here");
        }
    }

    fn arm32_note_personality(&mut self) {
        let mut names: Vec<(SourceLocation, &str)> = Vec::new();
        for location in &self.arm_unwind.personality_locations {
            names.push((*location, ".personality was specified here"));
        }
        for location in &self.arm_unwind.index_locations {
            names.push((*location, ".personalityindex was specified here"));
        }
        names.sort_by_key(|(location, _)| (location.buffer, location.position));
        for (location, text) in names {
            self.arm32_note(location, text);
        }
    }

    fn arm32_stack_register(&self) -> Option<RegisterId> {
        register_named(self.target, "sp")
    }

    fn arm32_emit_label_here(&mut self) -> usize {
        let symbol = self.context.temporary_symbol();
        self.streamer.emit_label(&mut self.context, symbol, None);
        symbol
    }

    fn arm32_switch_unwind_section(&mut self, prefix: &str, section_type: u32, flags: u64) {
        let Some((function_section, _)) = self.arm_unwind.function_section else {
            return;
        };
        let held = self.context.sections[function_section].clone();
        let mut name = prefix.to_string();
        if held.name != ".text" {
            name.push_str(&held.name);
        }
        let mut flags = flags;
        let group = held.group.clone().unwrap_or_default();
        if held.group.is_some() {
            flags |= ELF_FLAG_GROUP;
        }
        let linked = held
            .begin_symbol
            .map(|symbol| self.context.symbols[symbol].name.clone());
        let section = match self.context.find_section(&name, &group, held.unique) {
            Some(found) => found,
            None => {
                let created =
                    self.context
                        .elf_section(&name, section_type, flags, 0, &group, held.unique);
                self.context.sections[created].comdat = held.comdat;
                if section_type == 0x7000_0001 {
                    self.context.sections[created].linked_symbol = linked;
                    self.context.sections[created].linked_section = Some(function_section);
                }
                created
            }
        };
        self.streamer.switch_section(&mut self.context, section, 0);
        self.streamer
            .emit_value_to_alignment(&mut self.context, 4, 0, 1, 0);
    }

    fn arm32_prel31(&self, symbol: usize) -> Expression {
        Expression::Symbol {
            name: self.context.symbols[symbol].name.clone(),
            specifier: Some("prel31".to_string()),
        }
    }

    fn arm32_personality_fixup(&mut self, name: &str, location: SourceLocation) {
        let target = Expression::symbol(name);
        self.streamer
            .emit_arm_marker_fixup(&mut self.context, &target, Some(0), location);
    }

    fn arm32_flush_pending_offset(&mut self) {
        if self.arm_unwind.pending_offset != 0 {
            let offset = -self.arm_unwind.pending_offset;
            self.arm_unwind.assembler.stack_offset(offset);
            self.arm_unwind.pending_offset = 0;
        }
    }

    fn arm32_flush_unwind_opcodes(&mut self, no_handler_data: bool, location: SourceLocation) {
        if self.arm_unwind.used_frame {
            let last = self.arm_unwind.stack_offset - self.arm_unwind.pending_offset;
            let offset = last - self.arm_unwind.frame_offset;
            self.arm_unwind.assembler.stack_offset(offset);
            let register = self
                .arm_unwind
                .frame_register
                .map_or(13, |held| register_encoding(self.target, held));
            self.arm_unwind.assembler.set_stack_pointer(register);
        } else {
            self.arm32_flush_pending_offset();
        }
        let mut index = self.arm_unwind.personality_index;
        let mut opcodes = Vec::new();
        self.arm_unwind.assembler.finalize(&mut index, &mut opcodes);
        self.arm_unwind.personality_index = index;
        self.arm_unwind.opcodes = opcodes;
        if no_handler_data && self.arm_unwind.personality_index == 0 {
            return;
        }
        self.arm32_switch_unwind_section(".ARM.extab", 1, ELF_FLAG_ALLOCATE);
        let table = self.arm32_emit_label_here();
        self.arm_unwind.exception_table = Some(table);
        if let Some(name) = self.arm_unwind.personality.clone() {
            let symbol = self.context.symbol(&name);
            let expression = self.arm32_prel31(symbol);
            self.streamer
                .emit_value(&mut self.context, &expression, 4, location);
        }
        let opcodes = self.arm_unwind.opcodes.clone();
        for chunk in opcodes.chunks(4) {
            let mut word = 0i64;
            for (at, byte) in chunk.iter().enumerate() {
                word |= i64::from(*byte) << (8 * at);
            }
            self.streamer
                .emit_value(&mut self.context, &Expression::Constant(word), 4, location);
        }
        if no_handler_data && self.arm_unwind.personality.is_none() {
            self.streamer
                .emit_value(&mut self.context, &Expression::Constant(0), 4, location);
        }
    }

    fn arm32_unwind_text(&mut self, text: &str) {
        self.streamer.emit_arm_directive(text);
    }

    pub(super) fn arm32_directive_fnstart(&mut self, location: SourceLocation) -> bool {
        if self.parse_end_of_line() {
            return true;
        }
        if !self.arm_unwind.start_locations.is_empty() {
            self.error(location, ".fnstart starts before the end of previous one");
            self.arm32_note_start();
            return true;
        }
        if self.check_for_valid_section() {
            return true;
        }
        self.arm_unwind.reset_context();
        self.arm_unwind.reset_streamer();
        self.arm32_unwind_text("\t.fnstart");
        if self.streamer.emits_object() {
            let symbol = self.arm32_emit_label_here();
            self.arm_unwind.function_start = Some(symbol);
            self.arm_unwind.function_section = self.context.current_section();
        }
        self.arm_unwind.start_locations.push(location);
        false
    }

    pub(super) fn arm32_directive_fnend(&mut self, location: SourceLocation) -> bool {
        if self.parse_end_of_line() {
            return true;
        }
        if self.arm_unwind.start_locations.is_empty() {
            return self.error(location, ".fnstart must precede .fnend directive");
        }
        self.arm32_unwind_text("\t.fnend");
        let Some(start) = self.arm_unwind.function_start else {
            self.arm_unwind.reset_streamer();
            self.arm_unwind.reset_context();
            return false;
        };
        if self.arm_unwind.exception_table.is_none() && !self.arm_unwind.cannot_unwind {
            self.arm32_flush_unwind_opcodes(true, location);
        }
        self.arm32_switch_unwind_section(
            ".ARM.exidx",
            0x7000_0001,
            ELF_FLAG_ALLOCATE | ELF_FLAG_LINK_ORDER,
        );
        if self.arm_unwind.personality_index < 4 {
            let name = format!("__aeabi_unwind_cpp_pr{}", self.arm_unwind.personality_index);
            self.arm32_personality_fixup(&name, location);
        }
        let reference = self.arm32_prel31(start);
        self.streamer
            .emit_value(&mut self.context, &reference, 4, location);
        if self.arm_unwind.cannot_unwind {
            self.streamer
                .emit_value(&mut self.context, &Expression::Constant(1), 4, location);
        } else if let Some(table) = self.arm_unwind.exception_table {
            let reference = self.arm32_prel31(table);
            self.streamer
                .emit_value(&mut self.context, &reference, 4, location);
        } else {
            let opcodes = &self.arm_unwind.opcodes;
            let mut word = 0i64;
            for (at, byte) in opcodes.iter().take(4).enumerate() {
                word |= i64::from(*byte) << (8 * at);
            }
            self.streamer
                .emit_value(&mut self.context, &Expression::Constant(word), 4, location);
        }
        if let Some((section, subsection)) = self.arm_unwind.function_section {
            self.streamer
                .switch_section(&mut self.context, section, subsection);
        }
        self.arm_unwind.reset_streamer();
        self.arm_unwind.reset_context();
        false
    }

    pub(super) fn arm32_directive_cantunwind(&mut self, location: SourceLocation) -> bool {
        if self.parse_end_of_line() {
            return true;
        }
        self.arm_unwind.cannot_unwind_locations.push(location);
        if self.arm_unwind.start_locations.is_empty() {
            return self.error(location, ".fnstart must precede .cantunwind directive");
        }
        if !self.arm_unwind.handler_locations.is_empty() {
            self.error(
                location,
                ".cantunwind can't be used with .handlerdata directive",
            );
            self.arm32_note_handler();
            return true;
        }
        if self.arm_unwind.has_personality() {
            self.error(
                location,
                ".cantunwind can't be used with .personality directive",
            );
            self.arm32_note_personality();
            return true;
        }
        self.arm32_unwind_text("\t.cantunwind");
        self.arm_unwind.cannot_unwind = true;
        false
    }

    pub(super) fn arm32_directive_personality(&mut self, location: SourceLocation) -> bool {
        let existing = self.arm_unwind.has_personality();
        if !self.is(TokenKind::Identifier) {
            return self.error(location, "unexpected input in .personality directive.");
        }
        let name = self.lexer.token.text.clone();
        self.lex();
        if self.parse_end_of_line() {
            return true;
        }
        self.arm_unwind.personality_locations.push(location);
        if self.arm_unwind.start_locations.is_empty() {
            return self.error(location, ".fnstart must precede .personality directive");
        }
        if !self.arm_unwind.cannot_unwind_locations.is_empty() {
            self.error(
                location,
                ".personality can't be used with .cantunwind directive",
            );
            self.arm32_note_cannot_unwind();
            return true;
        }
        if !self.arm_unwind.handler_locations.is_empty() {
            self.error(location, ".personality must precede .handlerdata directive");
            self.arm32_note_handler();
            return true;
        }
        if existing {
            self.error(location, "multiple personality directives");
            self.arm32_note_personality();
            return true;
        }
        let symbol = self.context.symbol(&name);
        self.arm32_unwind_text(&format!("\t.personality {name}"));
        self.arm_unwind.personality = Some(self.context.symbols[symbol].name.clone());
        self.arm_unwind.assembler.begin();
        self.arm_unwind.assembler.has_personality = true;
        false
    }

    pub(super) fn arm32_directive_personality_index(&mut self, location: SourceLocation) -> bool {
        let existing = self.arm_unwind.has_personality();
        let index_location = self.location();
        let Ok((expression, _)) = self.parse_expression() else {
            return true;
        };
        if self.parse_end_of_line() {
            return true;
        }
        self.arm_unwind.index_locations.push(location);
        if self.arm_unwind.start_locations.is_empty() {
            return self.error(
                location,
                ".fnstart must precede .personalityindex directive",
            );
        }
        if !self.arm_unwind.cannot_unwind_locations.is_empty() {
            self.error(
                location,
                ".personalityindex cannot be used with .cantunwind",
            );
            self.arm32_note_cannot_unwind();
            return true;
        }
        if !self.arm_unwind.handler_locations.is_empty() {
            self.error(
                location,
                ".personalityindex must precede .handlerdata directive",
            );
            self.arm32_note_handler();
            return true;
        }
        if existing {
            self.error(location, "multiple personality directives");
            self.arm32_note_personality();
            return true;
        }
        let Expression::Constant(value) = expression else {
            return self.error(index_location, "index must be a constant number");
        };
        if !(0..4).contains(&value) {
            return self.error(
                index_location,
                "personality routine index should be in range [0-3]",
            );
        }
        self.arm32_unwind_text(&format!("\t.personalityindex {value}"));
        self.arm_unwind.personality_index = value as u32;
        false
    }

    pub(super) fn arm32_directive_handlerdata(&mut self, location: SourceLocation) -> bool {
        if self.parse_end_of_line() {
            return true;
        }
        self.arm_unwind.handler_locations.push(location);
        if self.arm_unwind.start_locations.is_empty() {
            return self.error(location, ".fnstart must precede .personality directive");
        }
        if !self.arm_unwind.cannot_unwind_locations.is_empty() {
            self.error(
                location,
                ".handlerdata can't be used with .cantunwind directive",
            );
            self.arm32_note_cannot_unwind();
            return true;
        }
        self.arm32_unwind_text("\t.handlerdata");
        if self.streamer.emits_object() {
            self.arm32_flush_unwind_opcodes(false, location);
        }
        false
    }

    pub(super) fn arm32_directive_setfp(&mut self, location: SourceLocation) -> bool {
        if self.arm_unwind.start_locations.is_empty() {
            return self.error(location, ".fnstart must precede .setfp directive");
        }
        if !self.arm_unwind.handler_locations.is_empty() {
            return self.error(location, ".setfp must precede .handlerdata directive");
        }
        let frame_location = self.location();
        let Some(frame) = self.arm32_try_register() else {
            return self.error(frame_location, "frame pointer register expected");
        };
        if self.parse_token(TokenKind::Comma, "expected comma") {
            return true;
        }
        let stack_location = self.location();
        let Some(stack) = self.arm32_try_register() else {
            return self.error(stack_location, "stack pointer register expected");
        };
        let sp = self.arm32_stack_register();
        let latest = self.arm_unwind.context_frame_register.or(sp);
        if Some(stack) != sp && Some(stack) != latest {
            return self.error(
                stack_location,
                "register should be either $sp or the latest fp register",
            );
        }
        self.arm_unwind.context_frame_register = Some(frame);
        let mut offset = 0i64;
        if self.parse_optional_token(TokenKind::Comma) {
            if !self.is(TokenKind::Hash) && !self.is(TokenKind::Dollar) {
                let at = self.location();
                return self.error(at, "'#' expected");
            }
            self.lex();
            let at = self.location();
            let Ok((expression, _)) = self.parse_expression() else {
                return self.error(at, "malformed setfp offset");
            };
            let Expression::Constant(value) = expression else {
                return self.error(at, "setfp offset must be an immediate");
            };
            offset = value;
        }
        if self.parse_end_of_line() {
            return true;
        }
        let frame_name = self.target.register_name(frame);
        let stack_name = self.target.register_name(stack);
        let mut line = format!("\t.setfp\t{frame_name}, {stack_name}");
        if offset != 0 {
            line.push_str(&format!(", #{offset}"));
        }
        self.arm32_unwind_text(&line);
        self.arm_unwind.used_frame = true;
        let previous = self.arm_unwind.frame_register.or(sp);
        self.arm_unwind.frame_register = Some(frame);
        if Some(stack) == sp {
            self.arm_unwind.frame_offset = self.arm_unwind.stack_offset + offset;
        } else {
            let _ = previous;
            self.arm_unwind.frame_offset += offset;
        }
        false
    }

    pub(super) fn arm32_directive_pad(&mut self, location: SourceLocation) -> bool {
        if self.arm_unwind.start_locations.is_empty() {
            return self.error(location, ".fnstart must precede .pad directive");
        }
        if !self.arm_unwind.handler_locations.is_empty() {
            return self.error(location, ".pad must precede .handlerdata directive");
        }
        if !self.is(TokenKind::Hash) && !self.is(TokenKind::Dollar) {
            let at = self.location();
            return self.error(at, "'#' expected");
        }
        self.lex();
        let at = self.location();
        let Ok((expression, _)) = self.parse_expression() else {
            return self.error(at, "malformed pad offset");
        };
        let Expression::Constant(value) = expression else {
            return self.error(at, "pad offset must be an immediate");
        };
        if self.parse_end_of_line() {
            return true;
        }
        self.arm32_unwind_text(&format!("\t.pad\t#{value}"));
        self.arm_unwind.stack_offset -= value;
        self.arm_unwind.pending_offset -= value;
        false
    }

    pub(super) fn arm32_directive_movsp(&mut self, location: SourceLocation) -> bool {
        if self.arm_unwind.start_locations.is_empty() {
            return self.error(location, ".fnstart must precede .movsp directives");
        }
        let sp = self.arm32_stack_register();
        let latest = self.arm_unwind.context_frame_register.or(sp);
        if latest != sp {
            return self.error(location, "unexpected .movsp directive");
        }
        let at = self.location();
        let Some(register) = self.arm32_try_register() else {
            return self.error(at, "register expected");
        };
        let pc = register_named(self.target, "pc");
        if Some(register) == sp || Some(register) == pc {
            return self.error(at, "sp and pc are not permitted in .movsp directive");
        }
        let mut offset = 0i64;
        if self.parse_optional_token(TokenKind::Comma) {
            if self.parse_token(TokenKind::Hash, "expected #constant") {
                return true;
            }
            let at = self.location();
            let Ok((expression, _)) = self.parse_expression() else {
                return self.error(at, "malformed offset expression");
            };
            let Expression::Constant(value) = expression else {
                return self.error(at, "offset must be an immediate");
            };
            offset = value;
        }
        if self.parse_end_of_line() {
            return true;
        }
        self.arm_unwind.context_frame_register = Some(register);
        let name = self.target.register_name(register);
        let mut line = format!("\t.movsp\t{name}");
        if offset != 0 {
            line.push_str(&format!(", #{offset}"));
        }
        self.arm32_unwind_text(&line);
        self.arm32_flush_pending_offset();
        self.arm_unwind.frame_register = Some(register);
        self.arm_unwind.frame_offset = self.arm_unwind.stack_offset + offset;
        let encoding = register_encoding(self.target, register);
        self.arm_unwind.assembler.set_stack_pointer(encoding);
        false
    }

    pub(super) fn arm32_directive_register_save(
        &mut self,
        location: SourceLocation,
        vector: bool,
    ) -> bool {
        if self.arm_unwind.start_locations.is_empty() {
            return self.error(location, ".fnstart must precede .save or .vsave directives");
        }
        if !self.arm_unwind.handler_locations.is_empty() {
            return self.error(
                location,
                ".save or .vsave must precede .handlerdata directive",
            );
        }
        let mut operands: Vec<ParsedOperand> = Vec::new();
        if !self.is(TokenKind::LeftBrace) {
            return self.token_error("register list expected");
        }
        if self.arm32_parse_register_list_with(&mut operands, true) {
            return true;
        }
        if self.parse_end_of_line() {
            return true;
        }
        let Some(Operand::RegisterList(list)) = operands.first().and_then(operand_of) else {
            return true;
        };
        let list = list.clone();
        let doubles = list
            .first()
            .is_some_and(|held| in_class(self.target, "DPR", *held));
        if !vector && doubles
            || !vector && list.iter().any(|held| in_class(self.target, "SPR", *held))
        {
            return self.error(location, ".save expects GPR registers");
        }
        if vector && !doubles {
            return self.error(location, ".vsave expects DPR registers");
        }
        let names: Vec<&str> = list
            .iter()
            .map(|held| self.target.register_name(*held))
            .collect();
        self.arm32_unwind_text(&format!(
            "\t.{}\t{{{}}}",
            if vector { "vsave" } else { "save" },
            names.join(", ")
        ));
        let authentication = register_named(self.target, "ra_auth_code");
        let mut index = list.len();
        while index > 0 {
            let mut mask = 0u32;
            let mut count = 0i64;
            while index > 0 && Some(list[index - 1]) != authentication {
                let encoding = register_encoding(self.target, list[index - 1]);
                if mask & (1 << encoding) == 0 {
                    mask |= 1 << encoding;
                    count += 1;
                }
                index -= 1;
            }
            if count > 0 {
                self.arm_unwind.stack_offset -= count * if vector { 8 } else { 4 };
                self.arm32_flush_pending_offset();
                if vector {
                    self.arm_unwind.assembler.vfp_register_save(mask);
                } else {
                    self.arm_unwind.assembler.register_save(mask);
                }
            } else if index > 0 {
                index -= 1;
                self.arm_unwind.stack_offset -= 4;
                self.arm32_flush_pending_offset();
                self.arm_unwind.assembler.register_save(0);
            }
        }
        false
    }

    pub(super) fn arm32_directive_unwind_raw(&mut self, location: SourceLocation) -> bool {
        let offset_location = self.location();
        if self.arm_unwind.start_locations.is_empty() {
            return self.error(location, ".fnstart must precede .unwind_raw directives");
        }
        let Ok((expression, _)) = self.parse_expression() else {
            return self.error(offset_location, "expected expression");
        };
        let Expression::Constant(offset) = expression else {
            return self.error(offset_location, "offset must be a constant");
        };
        if self.parse_token(TokenKind::Comma, "expected comma") {
            return true;
        }
        let first = self.location();
        if self.is(TokenKind::EndOfStatement) {
            return self.error(first, "expected opcode expression");
        }
        let mut opcodes: Vec<u8> = Vec::new();
        let failed = self.parse_many(|parser| {
            let at = parser.location();
            if parser.is(TokenKind::EndOfStatement) {
                return parser.error(at, "expected opcode expression");
            }
            let Ok((expression, _)) = parser.parse_expression() else {
                return parser.error(at, "expected opcode expression");
            };
            let Expression::Constant(value) = expression else {
                return parser.error(at, "opcode value must be a constant");
            };
            if value & !0xff != 0 {
                return parser.error(at, "invalid opcode");
            }
            opcodes.push(value as u8);
            false
        });
        if failed {
            return true;
        }
        let mut line = format!("\t.unwind_raw {offset}");
        for opcode in &opcodes {
            line.push_str(&format!(", 0x{opcode:x}"));
        }
        self.arm32_unwind_text(&line);
        self.arm32_flush_pending_offset();
        self.arm_unwind.stack_offset -= offset;
        self.arm_unwind.assembler.bytes(&opcodes);
        false
    }
}

const ARM_RELOCATIONS: &[(&str, u32)] = &[
    ("R_ARM_NONE", 0x00),
    ("R_ARM_PC24", 0x01),
    ("R_ARM_ABS32", 0x02),
    ("R_ARM_REL32", 0x03),
    ("R_ARM_LDR_PC_G0", 0x04),
    ("R_ARM_ABS16", 0x05),
    ("R_ARM_ABS12", 0x06),
    ("R_ARM_THM_ABS5", 0x07),
    ("R_ARM_ABS8", 0x08),
    ("R_ARM_SBREL32", 0x09),
    ("R_ARM_THM_CALL", 0x0a),
    ("R_ARM_THM_PC8", 0x0b),
    ("R_ARM_BREL_ADJ", 0x0c),
    ("R_ARM_TLS_DESC", 0x0d),
    ("R_ARM_THM_SWI8", 0x0e),
    ("R_ARM_XPC25", 0x0f),
    ("R_ARM_THM_XPC22", 0x10),
    ("R_ARM_TLS_DTPMOD32", 0x11),
    ("R_ARM_TLS_DTPOFF32", 0x12),
    ("R_ARM_TLS_TPOFF32", 0x13),
    ("R_ARM_COPY", 0x14),
    ("R_ARM_GLOB_DAT", 0x15),
    ("R_ARM_JUMP_SLOT", 0x16),
    ("R_ARM_RELATIVE", 0x17),
    ("R_ARM_GOTOFF32", 0x18),
    ("R_ARM_BASE_PREL", 0x19),
    ("R_ARM_GOT_BREL", 0x1a),
    ("R_ARM_PLT32", 0x1b),
    ("R_ARM_CALL", 0x1c),
    ("R_ARM_JUMP24", 0x1d),
    ("R_ARM_THM_JUMP24", 0x1e),
    ("R_ARM_BASE_ABS", 0x1f),
    ("R_ARM_ALU_PCREL_7_0", 0x20),
    ("R_ARM_ALU_PCREL_15_8", 0x21),
    ("R_ARM_ALU_PCREL_23_15", 0x22),
    ("R_ARM_LDR_SBREL_11_0_NC", 0x23),
    ("R_ARM_ALU_SBREL_19_12_NC", 0x24),
    ("R_ARM_ALU_SBREL_27_20_CK", 0x25),
    ("R_ARM_TARGET1", 0x26),
    ("R_ARM_SBREL31", 0x27),
    ("R_ARM_V4BX", 0x28),
    ("R_ARM_TARGET2", 0x29),
    ("R_ARM_PREL31", 0x2a),
    ("R_ARM_MOVW_ABS_NC", 0x2b),
    ("R_ARM_MOVT_ABS", 0x2c),
    ("R_ARM_MOVW_PREL_NC", 0x2d),
    ("R_ARM_MOVT_PREL", 0x2e),
    ("R_ARM_THM_MOVW_ABS_NC", 0x2f),
    ("R_ARM_THM_MOVT_ABS", 0x30),
    ("R_ARM_THM_MOVW_PREL_NC", 0x31),
    ("R_ARM_THM_MOVT_PREL", 0x32),
    ("R_ARM_THM_JUMP19", 0x33),
    ("R_ARM_THM_JUMP6", 0x34),
    ("R_ARM_THM_ALU_PREL_11_0", 0x35),
    ("R_ARM_THM_PC12", 0x36),
    ("R_ARM_ABS32_NOI", 0x37),
    ("R_ARM_REL32_NOI", 0x38),
    ("R_ARM_ALU_PC_G0_NC", 0x39),
    ("R_ARM_ALU_PC_G0", 0x3a),
    ("R_ARM_ALU_PC_G1_NC", 0x3b),
    ("R_ARM_ALU_PC_G1", 0x3c),
    ("R_ARM_ALU_PC_G2", 0x3d),
    ("R_ARM_LDR_PC_G1", 0x3e),
    ("R_ARM_LDR_PC_G2", 0x3f),
    ("R_ARM_LDRS_PC_G0", 0x40),
    ("R_ARM_LDRS_PC_G1", 0x41),
    ("R_ARM_LDRS_PC_G2", 0x42),
    ("R_ARM_LDC_PC_G0", 0x43),
    ("R_ARM_LDC_PC_G1", 0x44),
    ("R_ARM_LDC_PC_G2", 0x45),
    ("R_ARM_ALU_SB_G0_NC", 0x46),
    ("R_ARM_ALU_SB_G0", 0x47),
    ("R_ARM_ALU_SB_G1_NC", 0x48),
    ("R_ARM_ALU_SB_G1", 0x49),
    ("R_ARM_ALU_SB_G2", 0x4a),
    ("R_ARM_LDR_SB_G0", 0x4b),
    ("R_ARM_LDR_SB_G1", 0x4c),
    ("R_ARM_LDR_SB_G2", 0x4d),
    ("R_ARM_LDRS_SB_G0", 0x4e),
    ("R_ARM_LDRS_SB_G1", 0x4f),
    ("R_ARM_LDRS_SB_G2", 0x50),
    ("R_ARM_LDC_SB_G0", 0x51),
    ("R_ARM_LDC_SB_G1", 0x52),
    ("R_ARM_LDC_SB_G2", 0x53),
    ("R_ARM_MOVW_BREL_NC", 0x54),
    ("R_ARM_MOVT_BREL", 0x55),
    ("R_ARM_MOVW_BREL", 0x56),
    ("R_ARM_THM_MOVW_BREL_NC", 0x57),
    ("R_ARM_THM_MOVT_BREL", 0x58),
    ("R_ARM_THM_MOVW_BREL", 0x59),
    ("R_ARM_TLS_GOTDESC", 0x5a),
    ("R_ARM_TLS_CALL", 0x5b),
    ("R_ARM_TLS_DESCSEQ", 0x5c),
    ("R_ARM_THM_TLS_CALL", 0x5d),
    ("R_ARM_PLT32_ABS", 0x5e),
    ("R_ARM_GOT_ABS", 0x5f),
    ("R_ARM_GOT_PREL", 0x60),
    ("R_ARM_GOT_BREL12", 0x61),
    ("R_ARM_GOTOFF12", 0x62),
    ("R_ARM_GOTRELAX", 0x63),
    ("R_ARM_GNU_VTENTRY", 0x64),
    ("R_ARM_GNU_VTINHERIT", 0x65),
    ("R_ARM_THM_JUMP11", 0x66),
    ("R_ARM_THM_JUMP8", 0x67),
    ("R_ARM_TLS_GD32", 0x68),
    ("R_ARM_TLS_LDM32", 0x69),
    ("R_ARM_TLS_LDO32", 0x6a),
    ("R_ARM_TLS_IE32", 0x6b),
    ("R_ARM_TLS_LE32", 0x6c),
    ("R_ARM_TLS_LDO12", 0x6d),
    ("R_ARM_TLS_LE12", 0x6e),
    ("R_ARM_TLS_IE12GP", 0x6f),
    ("R_ARM_PRIVATE_0", 0x70),
    ("R_ARM_PRIVATE_1", 0x71),
    ("R_ARM_PRIVATE_2", 0x72),
    ("R_ARM_PRIVATE_3", 0x73),
    ("R_ARM_PRIVATE_4", 0x74),
    ("R_ARM_PRIVATE_5", 0x75),
    ("R_ARM_PRIVATE_6", 0x76),
    ("R_ARM_PRIVATE_7", 0x77),
    ("R_ARM_PRIVATE_8", 0x78),
    ("R_ARM_PRIVATE_9", 0x79),
    ("R_ARM_PRIVATE_10", 0x7a),
    ("R_ARM_PRIVATE_11", 0x7b),
    ("R_ARM_PRIVATE_12", 0x7c),
    ("R_ARM_PRIVATE_13", 0x7d),
    ("R_ARM_PRIVATE_14", 0x7e),
    ("R_ARM_PRIVATE_15", 0x7f),
    ("R_ARM_ME_TOO", 0x80),
    ("R_ARM_THM_TLS_DESCSEQ16", 0x81),
    ("R_ARM_THM_TLS_DESCSEQ32", 0x82),
    ("R_ARM_THM_ALU_ABS_G0_NC", 0x84),
    ("R_ARM_THM_ALU_ABS_G1_NC", 0x85),
    ("R_ARM_THM_ALU_ABS_G2_NC", 0x86),
    ("R_ARM_THM_ALU_ABS_G3", 0x87),
    ("R_ARM_THM_BF16", 0x88),
    ("R_ARM_THM_BF12", 0x89),
    ("R_ARM_THM_BF18", 0x8a),
    ("R_ARM_IRELATIVE", 0xa0),
    ("R_ARM_GOTFUNCDESC", 0xa1),
    ("R_ARM_GOTOFFFUNCDESC", 0xa2),
    ("R_ARM_FUNCDESC", 0xa3),
    ("R_ARM_FUNCDESC_VALUE", 0xa4),
    ("R_ARM_TLS_GD32_FDPIC", 0xa5),
    ("R_ARM_TLS_LDM32_FDPIC", 0xa6),
    ("R_ARM_TLS_IE32_FDPIC", 0xa7),
];

pub fn relocation_number(name: &str) -> Option<u32> {
    ARM_RELOCATIONS
        .iter()
        .find(|(held, _)| *held == name)
        .map(|(_, value)| *value)
        .or(match name {
            "BFD_RELOC_NONE" => Some(0),
            "BFD_RELOC_8" => Some(8),
            "BFD_RELOC_16" => Some(5),
            "BFD_RELOC_32" => Some(2),
            _ => None,
        })
}

pub fn variant_specifier(name: &str) -> Option<&'static str> {
    Some(match name.to_ascii_lowercase().as_str() {
        "none" => "none",
        "plt" => "plt",
        "got" => "got",
        "gotoff" => "gotoff",
        "got_prel" => "got_prel",
        "tlsgd" => "tlsgd",
        "tlsldm" => "tlsldm",
        "tlsldo" => "tlsldo",
        "gottpoff" => "gottpoff",
        "tpoff" => "tpoff",
        "target1" => "target1",
        "target2" => "target2",
        "prel31" => "prel31",
        "sbrel" => "sbrel",
        "tlsdesc" => "tlsdesc",
        "tlscall" => "tlscall",
        "secrel32" => "secrel32",
        _ => return None,
    })
}
