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
        "isRegList" => matches!(operand, Operand::RegisterList(_)),
        "isRegListWithAPSR" => matches!(operand, Operand::RegisterList(_)),
        "isBitfield" => matches!(operand, Operand::Bitfield { .. }),
        "isShifterImm" => matches!(operand, Operand::ShifterImmediate { .. }),
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
            TokenKind::Hash | TokenKind::Dollar => {
                self.lex();
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
        let prefix = operands.len();
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

    fn arm32_record_index(&self, record: &str) -> Option<usize> {
        self.target.instruction_by_record.get(record).copied()
    }

    fn arm32_process(&self, instruction: &mut MachineInstruction) -> bool {
        let target = self.target;
        let record = target.instructions[instruction.opcode].record.as_str();
        let operand = |index: usize| instruction.operand(index).clone();
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
                while self.arm32_process(&mut instruction) {}
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
        "getARMBranchTargetOpValue" | "getARMBLTargetOpValue" | "getBranchTargetOpValue" => {
            OperandEncoder::BranchTarget
        }
        "getLdStmModeOpValue" => OperandEncoder::LoadStoreMultipleMode,
        "getPowerTwoOpValue" => OperandEncoder::PowerTwo,
        "getHiLoImmOpValue" => OperandEncoder::HighLowImmediate,
        _ => return Err(format!("operand encoder {text}")),
    })
}

struct EncodedVariable {
    first_operand: usize,
    encoder: OperandEncoder,
    bits: Vec<(u32, u32)>,
}

struct EncodingPlan {
    constant: u32,
    variables: Vec<EncodedVariable>,
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
    if bits.len() != 32 {
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

fn operand_value(
    target: &Target,
    instruction: &MachineInstruction,
    first: usize,
    encoder: &OperandEncoder,
) -> Result<u64, Diagnostic> {
    let register = |index: usize| -> u64 {
        u64::from(register_encoding(target, instruction.register_at(index)))
    };
    let immediate = |index: usize| instruction.immediate_at(index);
    let operand = instruction.operand(first);
    let needs_constant = |operand: &MachineOperand| -> Result<(), Diagnostic> {
        match operand {
            MachineOperand::Expression(_) => Err(unencodable("a relocatable expression operand")),
            _ => Ok(()),
        }
    };
    Ok(match encoder {
        OperandEncoder::Machine => match operand {
            MachineOperand::Register(register) => u64::from(register_encoding(target, *register)),
            MachineOperand::Immediate(value) => *value as u64,
            MachineOperand::Expression(_) => return Err(unencodable("an expression operand")),
        },
        OperandEncoder::CarryOut => u64::from(instruction.register_at(first) != NO_REGISTER),
        OperandEncoder::ModifiedImmediate => {
            needs_constant(operand)?;
            immediate(first) as u64
        }
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
            MachineOperand::Register(_) => {
                let second = instruction.operand(first + 1);
                if matches!(second, MachineOperand::Expression(_)) {
                    return Err(unencodable("a relocatable expression operand"));
                }
                let value = second.immediate();
                let (add, amount) = split_offset(value);
                let mut binary = u64::from(amount & 0xfff);
                if add {
                    binary |= 1 << 12;
                }
                binary | (register(first) << 13)
            }
            MachineOperand::Expression(_) => {
                return Err(unencodable("a relocatable expression operand"))
            }
            MachineOperand::Immediate(_) => unreachable!(),
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
            if matches!(operand, MachineOperand::Expression(_)) {
                return Err(unencodable("a relocatable expression operand"));
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
            MachineOperand::Immediate(value) => (*value >> 2) as u64,
            _ => return Err(unencodable("a relocatable branch target")),
        },
        OperandEncoder::LoadStoreMultipleMode => match immediate(first) {
            1 => 1,
            2 => 3,
            3 => 0,
            4 => 2,
            _ => return Err(unencodable("an unknown addressing sub-mode")),
        },
        OperandEncoder::PowerTwo => (immediate(first) as u64).trailing_zeros() as u64,
        OperandEncoder::HighLowImmediate => {
            needs_constant(operand)?;
            immediate(first) as u64
        }
    })
}

pub fn encode(
    target: &'static Target,
    instruction: &MachineInstruction,
) -> Result<Encoded, Diagnostic> {
    let description = &target.instructions[instruction.opcode];
    let plan = match &encoding_plans(target)[instruction.opcode] {
        Ok(plan) => plan,
        Err(cause) => {
            return Err(unencodable(&format!("{}: {cause}", description.record)));
        }
    };
    let mut value = plan.constant;
    for variable in &plan.variables {
        let encoded = operand_value(
            target,
            instruction,
            variable.first_operand,
            &variable.encoder,
        )?;
        for (source, destination) in &variable.bits {
            if (encoded >> source) & 1 == 1 {
                value |= 1 << destination;
            }
        }
    }
    Ok(Encoded {
        bytes: value.to_le_bytes().to_vec(),
        fixups: Vec::new(),
    })
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
