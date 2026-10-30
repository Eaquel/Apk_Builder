use super::*;

pub const TABLE: &[u8] = include_bytes!("Tables/Armv8.bin");

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
        Target::load_with(Isa::Armv8, records, "AArch64Inst")
    })
    .as_ref()
    .map_err(|problem| problem.clone())
}

pub fn matcher() -> &'static Matcher {
    static CELL: OnceLock<Matcher> = OnceLock::new();
    CELL.get_or_init(|| build_matcher(target().expect("the ARMv8 tables load")))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArmFixup {
    PcrelAdrImm21,
    PcrelAdrpImm21,
    AddImm12,
    LdStImm12Scale1,
    LdStImm12Scale2,
    LdStImm12Scale4,
    LdStImm12Scale8,
    LdStImm12Scale16,
    LdrPcrelImm19,
    Movw,
    PcrelBranch9,
    PcrelBranch14,
    PcrelBranch16,
    PcrelBranch19,
    PcrelBranch26,
    PcrelCall26,
}

impl ArmFixup {
    pub fn name(self) -> &'static str {
        match self {
            ArmFixup::PcrelAdrImm21 => "fixup_aarch64_pcrel_adr_imm21",
            ArmFixup::PcrelAdrpImm21 => "fixup_aarch64_pcrel_adrp_imm21",
            ArmFixup::AddImm12 => "fixup_aarch64_add_imm12",
            ArmFixup::LdStImm12Scale1 => "fixup_aarch64_ldst_imm12_scale1",
            ArmFixup::LdStImm12Scale2 => "fixup_aarch64_ldst_imm12_scale2",
            ArmFixup::LdStImm12Scale4 => "fixup_aarch64_ldst_imm12_scale4",
            ArmFixup::LdStImm12Scale8 => "fixup_aarch64_ldst_imm12_scale8",
            ArmFixup::LdStImm12Scale16 => "fixup_aarch64_ldst_imm12_scale16",
            ArmFixup::LdrPcrelImm19 => "fixup_aarch64_ldr_pcrel_imm19",
            ArmFixup::Movw => "fixup_aarch64_movw",
            ArmFixup::PcrelBranch9 => "fixup_aarch64_pcrel_branch9",
            ArmFixup::PcrelBranch14 => "fixup_aarch64_pcrel_branch14",
            ArmFixup::PcrelBranch16 => "fixup_aarch64_pcrel_branch16",
            ArmFixup::PcrelBranch19 => "fixup_aarch64_pcrel_branch19",
            ArmFixup::PcrelBranch26 => "fixup_aarch64_pcrel_branch26",
            ArmFixup::PcrelCall26 => "fixup_aarch64_pcrel_call26",
        }
    }

    fn named(name: &str) -> Option<ArmFixup> {
        let name = name.trim().trim_start_matches("AArch64::");
        [
            ArmFixup::PcrelAdrImm21,
            ArmFixup::PcrelAdrpImm21,
            ArmFixup::AddImm12,
            ArmFixup::LdStImm12Scale1,
            ArmFixup::LdStImm12Scale2,
            ArmFixup::LdStImm12Scale4,
            ArmFixup::LdStImm12Scale8,
            ArmFixup::LdStImm12Scale16,
            ArmFixup::LdrPcrelImm19,
            ArmFixup::Movw,
            ArmFixup::PcrelBranch9,
            ArmFixup::PcrelBranch14,
            ArmFixup::PcrelBranch16,
            ArmFixup::PcrelBranch19,
            ArmFixup::PcrelBranch26,
            ArmFixup::PcrelCall26,
        ]
        .into_iter()
        .find(|kind| kind.name() == name)
    }
}

#[derive(Clone, Debug)]
enum OperandEncoder {
    Machine,
    LdStUImm12(ArmFixup),
    AdrLabel,
    AddSubImm,
    CondBranchTarget,
    CondCompBranchTarget,
    PAuthPcRel,
    LoadLiteral,
    MemExtend,
    TestBranchTarget,
    BranchTarget,
    MoveWideImm,
    VecShifter,
    MoveVecShifter,
    FixedPointScale,
    VecShiftRight(i64),
    VecShiftLeft(i64),
    Imm8OptLsl,
    SveIncDecImm,
    RegisterMultiple { multiple: i64, minimum: i64 },
    ZeroK,
    PredicateCounterHigh,
    Zpr2Strided,
    Zpr4Strided,
    MatrixTileList,
    MatrixIndex(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PostEncoder {
    None,
    MoveZero,
    MultiplyHigh,
    LoadStoreExclusive { has_rs: bool, has_rt2: bool },
    OneOperandFloatCompare,
}

fn split_method(text: &str) -> (String, Vec<String>) {
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

fn operand_encoder(text: &str) -> Result<OperandEncoder, String> {
    let (name, arguments) = split_method(text);
    let number = |index: usize| -> i64 {
        arguments
            .get(index)
            .and_then(|argument| argument.parse::<i64>().ok())
            .unwrap_or(0)
    };
    Ok(match name.as_str() {
        "" | "getMachineOpValue" => OperandEncoder::Machine,
        "getLdStUImm12OpValue" => OperandEncoder::LdStUImm12(
            arguments
                .first()
                .and_then(|argument| ArmFixup::named(argument))
                .ok_or_else(|| format!("fixup of {text}"))?,
        ),
        "getAdrLabelOpValue" => OperandEncoder::AdrLabel,
        "getAddSubImmOpValue" => OperandEncoder::AddSubImm,
        "getCondBranchTargetOpValue" => OperandEncoder::CondBranchTarget,
        "getCondCompBranchTargetOpValue" => OperandEncoder::CondCompBranchTarget,
        "getPAuthPCRelOpValue" => OperandEncoder::PAuthPcRel,
        "getLoadLiteralOpValue" => OperandEncoder::LoadLiteral,
        "getMemExtendOpValue" => OperandEncoder::MemExtend,
        "getTestBranchTargetOpValue" => OperandEncoder::TestBranchTarget,
        "getBranchTargetOpValue" => OperandEncoder::BranchTarget,
        "getMoveWideImmOpValue" => OperandEncoder::MoveWideImm,
        "getVecShifterOpValue" => OperandEncoder::VecShifter,
        "getMoveVecShifterOpValue" => OperandEncoder::MoveVecShifter,
        "getFixedPointScaleOpValue" => OperandEncoder::FixedPointScale,
        "getVecShiftR64OpValue" => OperandEncoder::VecShiftRight(64),
        "getVecShiftR32OpValue" => OperandEncoder::VecShiftRight(32),
        "getVecShiftR16OpValue" => OperandEncoder::VecShiftRight(16),
        "getVecShiftR8OpValue" => OperandEncoder::VecShiftRight(8),
        "getVecShiftL64OpValue" => OperandEncoder::VecShiftLeft(64),
        "getVecShiftL32OpValue" => OperandEncoder::VecShiftLeft(32),
        "getVecShiftL16OpValue" => OperandEncoder::VecShiftLeft(16),
        "getVecShiftL8OpValue" => OperandEncoder::VecShiftLeft(8),
        "getImm8OptLsl" => OperandEncoder::Imm8OptLsl,
        "getSVEIncDecImm" => OperandEncoder::SveIncDecImm,
        "EncodeRegMul_MinMax" => OperandEncoder::RegisterMultiple {
            multiple: number(0),
            minimum: number(1),
        },
        "EncodeZK" => OperandEncoder::ZeroK,
        "EncodePNR_p8to15" => OperandEncoder::PredicateCounterHigh,
        "EncodeZPR2StridedRegisterClass" => OperandEncoder::Zpr2Strided,
        "EncodeZPR4StridedRegisterClass" => OperandEncoder::Zpr4Strided,
        "EncodeMatrixTileListRegisterClass" => OperandEncoder::MatrixTileList,
        "encodeMatrixIndexGPR32" => OperandEncoder::MatrixIndex(
            arguments
                .first()
                .map(|argument| argument.trim_start_matches("AArch64::").to_string())
                .unwrap_or_default(),
        ),
        _ => return Err(format!("operand encoder {text}")),
    })
}

fn post_encoder(text: &str) -> Result<PostEncoder, String> {
    let (name, arguments) = split_method(text);
    Ok(match name.as_str() {
        "" => PostEncoder::None,
        "fixMOVZ" => PostEncoder::MoveZero,
        "fixMulHigh" => PostEncoder::MultiplyHigh,
        "fixLoadStoreExclusive" => PostEncoder::LoadStoreExclusive {
            has_rs: arguments.first().is_some_and(|argument| argument != "0"),
            has_rt2: arguments.get(1).is_some_and(|argument| argument != "0"),
        },
        "fixOneOperandFPComparison" => PostEncoder::OneOperandFloatCompare,
        _ => return Err(format!("post encoder {text}")),
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
    let mut constant = 0u32;
    let mut variables: Vec<EncodedVariable> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for (position, bit) in bits.iter().enumerate().take(32) {
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

fn shift_value(operand: &MachineOperand) -> i64 {
    operand.immediate() & 0x3f
}

fn specifier_of(expression: &Expression) -> Option<&str> {
    match expression {
        Expression::Symbol { specifier, .. } => specifier.as_deref(),
        _ => None,
    }
}

fn operand_value(
    target: &Target,
    instruction: &MachineInstruction,
    first: usize,
    encoder: &OperandEncoder,
    fixups: &mut Vec<Fixup>,
) -> Result<u64, Diagnostic> {
    let operand = instruction.operand(first);
    let mut add = |expression: &Expression, kind: ArmFixup, pc_relative: bool| {
        fixups.push(Fixup {
            offset: 0,
            expression: expression.clone(),
            kind: FixupKind::Arm(kind),
            pc_relative,
        });
    };
    let register_encoding = |operand: &MachineOperand| -> i64 {
        target.registers[operand.register() as usize].encoding as i64
    };
    let immediate = |operand: &MachineOperand| -> Result<i64, Diagnostic> {
        match operand {
            MachineOperand::Immediate(value) => Ok(*value),
            _ => Err(unencodable("an immediate operand was expected")),
        }
    };
    let relocated = |kind: ArmFixup,
                     pc_relative: bool,
                     add: &mut dyn FnMut(&Expression, ArmFixup, bool)|
     -> Result<u64, Diagnostic> {
        match operand {
            MachineOperand::Immediate(value) => Ok(*value as u32 as u64),
            MachineOperand::Expression(expression) => {
                add(expression, kind, pc_relative);
                Ok(0)
            }
            MachineOperand::Register(_) => Err(unencodable("a label or immediate was expected")),
        }
    };
    let value: i64 = match encoder {
        OperandEncoder::Machine => match operand {
            MachineOperand::Register(_) => register_encoding(operand),
            MachineOperand::Immediate(value) => *value,
            MachineOperand::Expression(_) => {
                return Err(unencodable("an expression needs a relocation here"))
            }
        },
        OperandEncoder::LdStUImm12(kind) => return relocated(*kind, false, &mut add),
        OperandEncoder::AdrLabel => {
            let kind = if target.instructions[instruction.opcode].record == "ADR" {
                ArmFixup::PcrelAdrImm21
            } else {
                ArmFixup::PcrelAdrpImm21
            };
            return relocated(kind, true, &mut add);
        }
        OperandEncoder::AddSubImm => {
            let mut shift = shift_value(instruction.operand(first + 1));
            match operand {
                MachineOperand::Immediate(value) => {
                    *value | if shift == 0 { 0 } else { 1 << shift }
                }
                MachineOperand::Expression(expression) => {
                    add(expression, ArmFixup::AddImm12, false);
                    if matches!(
                        specifier_of(expression),
                        Some("tprel_hi12" | "dtprel_hi12" | "secrel_hi12")
                    ) {
                        shift = 12;
                    }
                    if shift == 0 {
                        0
                    } else {
                        1 << shift
                    }
                }
                MachineOperand::Register(_) => {
                    return Err(unencodable("an add/sub immediate was expected"))
                }
            }
        }
        OperandEncoder::CondBranchTarget => {
            return relocated(ArmFixup::PcrelBranch19, true, &mut add)
        }
        OperandEncoder::CondCompBranchTarget => {
            return relocated(ArmFixup::PcrelBranch9, true, &mut add)
        }
        OperandEncoder::PAuthPcRel => match operand {
            MachineOperand::Immediate(value) => -*value,
            MachineOperand::Expression(expression) => {
                add(expression, ArmFixup::PcrelBranch16, true);
                0
            }
            MachineOperand::Register(_) => return Err(unencodable("a label was expected")),
        },
        OperandEncoder::LoadLiteral => return relocated(ArmFixup::LdrPcrelImm19, true, &mut add),
        OperandEncoder::MemExtend => {
            (immediate(operand)? << 1) | immediate(instruction.operand(first + 1))?
        }
        OperandEncoder::TestBranchTarget => {
            return relocated(ArmFixup::PcrelBranch14, true, &mut add)
        }
        OperandEncoder::BranchTarget => {
            let kind = if target.instructions[instruction.opcode].record == "BL" {
                ArmFixup::PcrelCall26
            } else {
                ArmFixup::PcrelBranch26
            };
            return relocated(kind, true, &mut add);
        }
        OperandEncoder::MoveWideImm => return relocated(ArmFixup::Movw, false, &mut add),
        OperandEncoder::VecShifter => match immediate(operand)? {
            0 => 0,
            8 => 1,
            16 => 2,
            24 => 3,
            other => return Err(unencodable(&format!("vector shift amount {other}"))),
        },
        OperandEncoder::MoveVecShifter => match shift_value(operand) {
            8 => 0,
            16 => 1,
            other => return Err(unencodable(&format!("move shift amount {other}"))),
        },
        OperandEncoder::FixedPointScale => 64 - immediate(operand)?,
        OperandEncoder::VecShiftRight(width) => width - immediate(operand)?,
        OperandEncoder::VecShiftLeft(width) => immediate(operand)? - width,
        OperandEncoder::Imm8OptLsl => {
            let shift = shift_value(instruction.operand(first + 1));
            (immediate(operand)? & 0xff) | if shift == 0 { 0 } else { 1 << shift }
        }
        OperandEncoder::SveIncDecImm => immediate(operand)? - 1,
        OperandEncoder::RegisterMultiple { multiple, minimum } => {
            (register_encoding(operand) - minimum) / multiple
        }
        OperandEncoder::ZeroK => {
            let encoding = register_encoding(operand);
            if encoding > 27 {
                encoding - 24
            } else {
                encoding - 20
            }
        }
        OperandEncoder::PredicateCounterHigh => register_encoding(operand) - 8,
        OperandEncoder::Zpr2Strided => {
            let encoding = register_encoding(operand);
            ((encoding & 0x10) >> 1) | (encoding & 0x7)
        }
        OperandEncoder::Zpr4Strided => {
            let encoding = register_encoding(operand);
            ((encoding & 0x10) >> 2) | (encoding & 0x3)
        }
        OperandEncoder::MatrixTileList => immediate(operand)?,
        OperandEncoder::MatrixIndex(base) => {
            let base = target
                .register_by_record
                .get(base)
                .map(|register| target.registers[*register as usize].encoding as i64)
                .unwrap_or(0);
            register_encoding(operand) - base
        }
    };
    Ok(value as u32 as u64)
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
    let mut fixups = Vec::new();
    let mut value = plan.constant;
    for variable in &plan.variables {
        let encoded = operand_value(
            target,
            instruction,
            variable.first_operand,
            &variable.encoder,
            &mut fixups,
        )?;
        for (source, destination) in &variable.bits {
            if (encoded >> source) & 1 == 1 {
                value |= 1 << destination;
            }
        }
    }
    match plan.post {
        PostEncoder::None => {}
        PostEncoder::MoveZero => {
            if let MachineOperand::Expression(expression) = instruction.operand(1) {
                if matches!(
                    specifier_of(expression),
                    Some(
                        "dtprel_g2"
                            | "dtprel_g1"
                            | "dtprel_g0"
                            | "gottprel_g1"
                            | "tprel_g2"
                            | "tprel_g1"
                            | "tprel_g0"
                    )
                ) {
                    value &= !(1 << 30);
                }
            }
        }
        PostEncoder::MultiplyHigh => value |= 0x1f << 10,
        PostEncoder::LoadStoreExclusive { has_rs, has_rt2 } => {
            if !has_rs {
                value |= 0x001f_0000;
            }
            if !has_rt2 {
                value |= 0x0000_7c00;
            }
        }
        PostEncoder::OneOperandFloatCompare => value &= !(0x1f << 16),
    }
    Ok(Encoded {
        bytes: value.to_le_bytes().to_vec(),
        fixups,
    })
}

pub fn instruction(
    target: &'static Target,
    record: &str,
    operands: Vec<MachineOperand>,
) -> Result<MachineInstruction, Diagnostic> {
    let Some(index) = target.instruction_by_record.get(record) else {
        return Err(unencodable(&format!("no instruction named {record}")));
    };
    let mut built = MachineInstruction::new(*index);
    built.operands = operands;
    Ok(built)
}

pub fn register(target: &'static Target, name: &str) -> Result<MachineOperand, Diagnostic> {
    target
        .register_by_name
        .get(name)
        .map(|register| MachineOperand::Register(*register))
        .ok_or_else(|| unencodable(&format!("no register named {name}")))
}

pub fn coverage(target: &'static Target) -> Vec<(String, String)> {
    let mut missing = Vec::new();
    for (index, plan) in encoding_plans(target).iter().enumerate() {
        let description = &target.instructions[index];
        if description.pseudo || description.code_generation_only {
            continue;
        }
        if let Err(cause) = plan {
            missing.push((description.record.clone(), cause.clone()));
        }
    }
    missing
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegKind {
    Scalar,
    NeonVector,
    SveData,
    SvePredicateCounter,
    SvePredicate,
    Matrix,
    LookupTable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegEquality {
    Reg,
    SuperReg,
    SubReg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShiftExtendKind {
    Lsl,
    Lsr,
    Asr,
    Ror,
    Msl,
    Uxtb,
    Uxth,
    Uxtw,
    Uxtx,
    Sxtb,
    Sxth,
    Sxtw,
    Sxtx,
}

impl ShiftExtendKind {
    fn named(name: &str) -> Option<ShiftExtendKind> {
        Some(match name {
            "lsl" => ShiftExtendKind::Lsl,
            "lsr" => ShiftExtendKind::Lsr,
            "asr" => ShiftExtendKind::Asr,
            "ror" => ShiftExtendKind::Ror,
            "msl" => ShiftExtendKind::Msl,
            "uxtb" => ShiftExtendKind::Uxtb,
            "uxth" => ShiftExtendKind::Uxth,
            "uxtw" => ShiftExtendKind::Uxtw,
            "uxtx" => ShiftExtendKind::Uxtx,
            "sxtb" => ShiftExtendKind::Sxtb,
            "sxth" => ShiftExtendKind::Sxth,
            "sxtw" => ShiftExtendKind::Sxtw,
            "sxtx" => ShiftExtendKind::Sxtx,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            ShiftExtendKind::Lsl => "lsl",
            ShiftExtendKind::Lsr => "lsr",
            ShiftExtendKind::Asr => "asr",
            ShiftExtendKind::Ror => "ror",
            ShiftExtendKind::Msl => "msl",
            ShiftExtendKind::Uxtb => "uxtb",
            ShiftExtendKind::Uxth => "uxth",
            ShiftExtendKind::Uxtw => "uxtw",
            ShiftExtendKind::Uxtx => "uxtx",
            ShiftExtendKind::Sxtb => "sxtb",
            ShiftExtendKind::Sxth => "sxth",
            ShiftExtendKind::Sxtw => "sxtw",
            ShiftExtendKind::Sxtx => "sxtx",
        }
    }

    fn is_shift(self) -> bool {
        self <= ShiftExtendKind::Msl
    }

    fn shifter_immediate(self, amount: u32) -> i64 {
        ((self as i64) << 6) | i64::from(amount & 0x3f)
    }

    fn arith_extend_immediate(self, amount: u32) -> i64 {
        ((self as i64 - ShiftExtendKind::Uxtb as i64) << 3) | i64::from(amount & 7)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShiftExtendOperand {
    pub kind: ShiftExtendKind,
    pub amount: u32,
    pub explicit: bool,
}

impl ShiftExtendOperand {
    const NONE: ShiftExtendOperand = ShiftExtendOperand {
        kind: ShiftExtendKind::Lsl,
        amount: 0,
        explicit: false,
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArmOperand {
    Immediate(Expression),
    ShiftedImmediate(Expression, u32),
    ImmediateRange(u32, u32),
    ConditionCode(u32),
    Register {
        register: RegisterId,
        kind: RegKind,
        width: i32,
        equality: RegEquality,
        shift: ShiftExtendOperand,
    },
    VectorList {
        register: RegisterId,
        count: u32,
        stride: u32,
        elements: u32,
        width: u32,
        kind: RegKind,
    },
    VectorIndex(i64),
    SystemRegister {
        name: String,
        read: u32,
        write: u32,
        pstate: u32,
    },
    SystemCr(u32),
    Prefetch {
        name: String,
        value: u32,
    },
    ShiftExtend(ShiftExtendOperand),
    FloatImmediate {
        bits: u64,
        exact: bool,
    },
    Barrier {
        name: String,
        value: u32,
        no_xs: bool,
    },
    TIndexHint {
        name: String,
        value: u32,
    },
}

impl ArmOperand {
    fn immediate(&self) -> Option<&Expression> {
        match self {
            ArmOperand::Immediate(expression) => Some(expression),
            _ => None,
        }
    }

    fn constant(&self) -> Option<i64> {
        self.immediate().and_then(Expression::constant_value)
    }

    fn is_scalar_register(&self) -> bool {
        matches!(
            self,
            ArmOperand::Register {
                kind: RegKind::Scalar,
                ..
            }
        )
    }

    fn shift_extend(&self) -> Option<ShiftExtendOperand> {
        match self {
            ArmOperand::ShiftExtend(shift) => Some(*shift),
            ArmOperand::Register { shift, .. } => Some(*shift),
            _ => None,
        }
    }
}

pub fn condition_code(name: &str) -> Option<u32> {
    Some(match name.to_ascii_lowercase().as_str() {
        "eq" | "none" => 0,
        "ne" | "any" => 1,
        "cs" | "hs" | "nlast" => 2,
        "cc" | "lo" | "last" => 3,
        "mi" | "first" => 4,
        "pl" | "nfrst" => 5,
        "vs" => 6,
        "vc" => 7,
        "hi" | "pmore" => 8,
        "ls" | "plast" => 9,
        "ge" | "tcont" => 10,
        "lt" | "tstop" => 11,
        "gt" => 12,
        "le" => 13,
        "al" => 14,
        "nv" => 15,
        _ => return None,
    })
}

pub fn condition_name(code: u32) -> &'static str {
    const NAMES: [&str; 16] = [
        "eq", "ne", "hs", "lo", "mi", "pl", "vs", "vc", "hi", "ls", "ge", "lt", "gt", "le", "al",
        "nv",
    ];
    NAMES[(code & 15) as usize]
}

pub fn requirement_features(text: &str) -> Vec<String> {
    text.trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .split(',')
        .map(|piece| piece.trim().trim_start_matches("AArch64::").to_string())
        .filter(|piece| !piece.is_empty())
        .collect()
}

#[derive(Clone, Debug)]
pub struct SystemEntry {
    pub name: String,
    pub encoding: u32,
    pub readable: bool,
    pub writeable: bool,
    pub requires: Vec<String>,
}

#[derive(Default)]
pub struct SystemTables {
    pub registers: BTreeMap<String, SystemEntry>,
    pub barriers: BTreeMap<String, SystemEntry>,
    pub barrier_names: BTreeMap<u32, String>,
    pub barriers_no_xs: BTreeMap<String, (SystemEntry, u32)>,
    pub prefetches: BTreeMap<String, SystemEntry>,
    pub prefetch_names: BTreeMap<u32, String>,
    pub pstate_one: BTreeMap<String, SystemEntry>,
    pub pstate_fifteen: BTreeMap<String, SystemEntry>,
    pub pstate_one_encodings: BTreeSet<u32>,
    pub pstate_fifteen_encodings: BTreeSet<u32>,
    pub hints: BTreeMap<String, SystemEntry>,
    pub exact_floats: BTreeMap<u32, String>,
}

fn system_entry(record: &TableRecord) -> SystemEntry {
    SystemEntry {
        name: table_text(record, "Name"),
        encoding: table_integer(record, "Encoding") as u32,
        readable: record.field("Readable").is_none() || table_bit(record, "Readable"),
        writeable: record.field("Writeable").is_none() || table_bit(record, "Writeable"),
        requires: requirement_features(&table_text(record, "Requires")),
    }
}

pub fn system_tables(target: &'static Target) -> &'static SystemTables {
    static CELL: OnceLock<SystemTables> = OnceLock::new();
    CELL.get_or_init(|| {
        let mut tables = SystemTables::default();
        for record in target.records.values() {
            if record.name.starts_with("anonymous_") || record.name.is_empty() {
                let entry = || system_entry(record);
                let lower = |entry: &SystemEntry| entry.name.to_ascii_lowercase();
                if record.is_a("SysReg") {
                    let held = entry();
                    tables.registers.insert(lower(&held), held);
                } else if record.is_a("DBnXS") {
                    let held = entry();
                    let immediate = table_integer(record, "ImmValue") as u32;
                    tables
                        .barriers_no_xs
                        .insert(lower(&held), (held, immediate));
                } else if record.is_a("DB") {
                    let held = entry();
                    tables
                        .barrier_names
                        .insert(held.encoding, held.name.clone());
                    tables.barriers.insert(lower(&held), held);
                } else if record.is_a("PRFM") {
                    let held = entry();
                    tables
                        .prefetch_names
                        .insert(held.encoding, held.name.clone());
                    tables.prefetches.insert(lower(&held), held);
                } else if record.is_a("PStateImm0_1") {
                    let held = entry();
                    tables.pstate_one_encodings.insert(held.encoding);
                    tables.pstate_one.insert(lower(&held), held);
                } else if record.is_a("PStateImm0_15") {
                    let held = entry();
                    tables.pstate_fifteen_encodings.insert(held.encoding);
                    tables.pstate_fifteen.insert(lower(&held), held);
                } else if record.is_a("TIndex") {
                    let held = entry();
                    tables.hints.insert(lower(&held), held);
                } else if record.is_a("ExactFPImm") {
                    tables.exact_floats.insert(
                        table_integer(record, "Enum") as u32,
                        table_text(record, "Repr"),
                    );
                }
            }
        }
        tables
    })
}

pub fn feature_set(target: &Target, cpu: &str, attributes: &[String]) -> BTreeSet<String> {
    let mut features: BTreeSet<String> = BTreeSet::new();
    let by_name = |name: &str| -> Option<&TableRecord> {
        target
            .records
            .values()
            .find(|record| record.is_a("SubtargetFeature") && table_text(record, "Name") == name)
    };
    let wanted = if cpu.is_empty() { "generic" } else { cpu };
    if let Some(processor) = target
        .records
        .values()
        .find(|record| record.is_a("ProcessorModel") && table_text(record, "Name") == wanted)
    {
        for feature in table_record_list(processor, "Features") {
            features.insert(feature);
        }
    }
    for attribute in attributes {
        let (enable, name) = match attribute.strip_prefix('-') {
            Some(name) => (false, name),
            None => (true, attribute.trim_start_matches('+')),
        };
        if name == "all" {
            features.insert("FeatureAll".to_string());
            continue;
        }
        if let Some(record) = by_name(name) {
            if enable {
                features.insert(record.name.clone());
            } else {
                features.remove(&record.name);
            }
        }
    }
    let mut pending: Vec<String> = features.iter().cloned().collect();
    while let Some(name) = pending.pop() {
        if let Some(record) = target.records.get(&name) {
            for implied in table_record_list(record, "Implies") {
                if features.insert(implied.clone()) {
                    pending.push(implied);
                }
            }
        }
    }
    features
}

pub fn predicates_hold(
    target: &Target,
    predicates: &[String],
    features: &BTreeSet<String>,
) -> bool {
    let held: BTreeSet<&str> = features.iter().map(String::as_str).collect();
    predicates.iter().all(|predicate| {
        match target
            .records
            .get(predicate)
            .and_then(|record| record.field("AssemblerCondDag"))
        {
            Some(condition @ TableValue::Dag { .. }) => condition_holds(condition, &held),
            _ => true,
        }
    })
}

pub fn is_logical_immediate(value: u64, register_size: u32) -> bool {
    logical_immediate(value, register_size).is_some()
}

pub fn logical_immediate(value: u64, register_size: u32) -> Option<u64> {
    let mut immediate = value;
    if immediate == 0
        || immediate == u64::MAX
        || (register_size != 64
            && (immediate >> register_size != 0 || immediate == (u64::MAX >> (64 - register_size))))
    {
        return None;
    }
    let mut size = register_size;
    loop {
        size /= 2;
        let mask = (1u64 << size) - 1;
        if (immediate & mask) != ((immediate >> size) & mask) {
            size *= 2;
            break;
        }
        if size <= 2 {
            break;
        }
    }
    let mask = u64::MAX >> (64 - size);
    immediate &= mask;
    let shifted_mask = |value: u64| {
        value != 0 && ((value - 1) | value).wrapping_add(1) & ((value - 1) | value) == 0
    };
    let (rotation, ones);
    if shifted_mask(immediate) {
        rotation = immediate.trailing_zeros();
        ones = (immediate >> rotation).trailing_ones();
    } else {
        immediate |= !mask;
        if !shifted_mask(!immediate) {
            return None;
        }
        let leading = immediate.leading_ones();
        rotation = 64 - leading;
        ones = leading + immediate.trailing_ones() - (64 - size);
    }
    let immr = (size - rotation) & (size - 1);
    let mut nimms = (!(u64::from(size) - 1)) << 1;
    nimms |= u64::from(ones) - 1;
    let n = ((nimms >> 6) & 1) ^ 1;
    Some((n << 12) | (u64::from(immr) << 6) | (nimms & 0x3f))
}

pub fn float_immediate(bits: u64) -> Option<u32> {
    let sign = (bits >> 63) & 1;
    let exponent = (bits >> 52) & 0x7ff;
    let mantissa = bits & 0xf_ffff_ffff_ffff;
    if mantissa & 0xffff_ffff_ffff != 0 {
        return None;
    }
    if !(0x3fc..=0x403).contains(&exponent) {
        return None;
    }
    let fraction = mantissa >> 48;
    let b = ((exponent >> 10) & 1) ^ 1;
    Some(((sign << 7) | (b << 6) | ((exponent & 3) << 4) | fraction) as u32)
}

pub fn float_from_immediate(encoded: u32) -> f64 {
    let encoded = u64::from(encoded);
    let sign = (encoded >> 7) & 1;
    let b = (encoded >> 6) & 1;
    let exponent = ((b ^ 1) << 10) | (if b == 1 { 0xff << 2 } else { 0 }) | ((encoded >> 4) & 3);
    let fraction = encoded & 0xf;
    f64::from_bits((sign << 63) | (exponent << 52) | (fraction << 48))
}

fn is_movz_alias(value: u64, shift: u32, width: u32) -> bool {
    let value = if width == 32 {
        value & 0xffff_ffff
    } else {
        value
    };
    if value == 0 && shift != 0 {
        return false;
    }
    value & !(0xffffu64 << shift) == 0
}

fn is_movn_alias(value: u64, shift: u32, width: u32) -> bool {
    for movz_shift in [0u32, 16, 32, 48] {
        if value & !(0xffffu64 << movz_shift) == 0 {
            return false;
        }
    }
    let mut value = !value;
    if width == 32 {
        value &= 0xffff_ffff;
    }
    is_movz_alias(value, shift, width)
}

fn is_simd_type10(value: i64) -> bool {
    let value = value as u64;
    (0..8).all(|index| matches!((value >> (index * 8)) & 0xff, 0 | 0xff))
}

fn encode_simd_type10(value: i64) -> i64 {
    let value = value as u64;
    (0..8).fold(0i64, |encoded, index| {
        if (value >> (index * 8)) & 0xff == 0xff {
            encoded | (1 << index)
        } else {
            encoded
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Diag {
    Match,
    Near,
    No,
}

impl Diag {
    fn of(condition: bool) -> Diag {
        if condition {
            Diag::Match
        } else {
            Diag::No
        }
    }
}

fn classify_symbol(expression: &Expression) -> Option<(Option<String>, i64)> {
    match expression {
        Expression::Symbol { specifier, .. } => Some((specifier.clone(), 0)),
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            let sign = match operator {
                BinaryOperator::Add => 1,
                BinaryOperator::Subtract => -1,
                _ => return None,
            };
            match (left.as_ref(), right.as_ref()) {
                (Expression::Symbol { specifier, .. }, Expression::Constant(value)) => {
                    Some((specifier.clone(), sign * value))
                }
                (Expression::Constant(value), Expression::Symbol { specifier, .. })
                    if sign == 1 =>
                {
                    Some((specifier.clone(), *value))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

const ADD_SUB_SPECIFIERS: &[&str] = &[
    "lo12",
    "got_auth_lo12",
    "dtprel_hi12",
    "dtprel_lo12",
    "dtprel_lo12_nc",
    "tprel_hi12",
    "tprel_lo12",
    "tprel_lo12_nc",
    "tlsdesc_lo12",
    "tlsdesc_auth_lo12",
    "secrel_hi12",
    "secrel_lo12",
];

const UIMM12_SPECIFIERS: &[&str] = &[
    "lo12",
    "got_lo12",
    "got_auth_lo12",
    "dtprel_lo12",
    "dtprel_lo12_nc",
    "tprel_lo12",
    "tprel_lo12_nc",
    "gottprel_lo12",
    "tlsdesc_lo12",
    "tlsdesc_auth_lo12",
    "secrel_lo12",
    "secrel_hi12",
    "gotpage_lo15",
];

fn movw_symbol(operand: &ArmOperand, allowed: &[&str]) -> bool {
    let Some(expression) = operand.immediate() else {
        return false;
    };
    match classify_symbol(expression) {
        Some((Some(specifier), _)) => allowed.contains(&specifier.as_str()),
        _ => false,
    }
}

fn shifted_value(operand: &ArmOperand, width: u32) -> Option<(i64, u32)> {
    if let ArmOperand::ShiftedImmediate(expression, shift) = operand {
        if *shift == width {
            if let Some(value) = expression.constant_value() {
                return Some((value, width));
            }
        }
    }
    if let Some(value) = operand.constant() {
        if value != 0 && (((value >> width) as u64) << width) as i64 == value {
            return Some((value >> width, width));
        }
        return Some((value, 0));
    }
    None
}

fn template_arguments(text: &str) -> (String, Vec<String>) {
    split_method(text)
}

fn argument_type_bytes(arguments: &[String]) -> i64 {
    let text = arguments.first().map(String::as_str).unwrap_or("");
    for (name, bytes) in [("64", 8), ("32", 4), ("16", 2), ("8", 1)] {
        if text.contains(name) {
            return bytes;
        }
    }
    argument_number(arguments, 0).max(1)
}

fn argument_number(arguments: &[String], index: usize) -> i64 {
    arguments
        .get(index)
        .map(|text| {
            let text = text.trim();
            if let Some(hex) = text.strip_prefix("0x") {
                i64::from_str_radix(hex, 16).unwrap_or(0)
            } else {
                text.parse::<i64>().unwrap_or(0)
            }
        })
        .unwrap_or(0)
}

fn immediate_scaled(
    operand: &ArmOperand,
    bits: i64,
    scale: i64,
    signed: bool,
    range: bool,
) -> Diag {
    let is_range = matches!(operand, ArmOperand::ImmediateRange(..));
    let is_immediate = matches!(operand, ArmOperand::Immediate(_));
    if (!is_immediate && !is_range) || (is_immediate && range) || (is_range && !range) {
        return Diag::No;
    }
    let value = match operand {
        ArmOperand::ImmediateRange(first, _) => i64::from(*first),
        _ => match operand.constant() {
            Some(value) => value,
            None => return Diag::No,
        },
    };
    let (minimum, maximum) = if signed {
        let shift = bits - 1;
        ((1i64 << shift) * -scale, ((1i64 << shift) - 1) * scale)
    } else {
        (0, ((1i64 << bits) - 1) * scale)
    };
    if value >= minimum && value <= maximum && value % scale == 0 {
        Diag::Match
    } else {
        Diag::Near
    }
}

impl Target {
    fn in_register_class(&self, class: &str, register: RegisterId) -> bool {
        self.register_classes
            .get(class)
            .is_some_and(|held| held.contains(register))
    }

    fn named_register(&self, name: &str) -> RegisterId {
        self.register_by_record
            .get(name)
            .copied()
            .unwrap_or(NO_REGISTER)
    }

    fn arm_x_from_w(&self, register: RegisterId) -> RegisterId {
        let encoding = self.registers[register as usize].encoding;
        if encoding == 31 {
            return self.named_register("XZR");
        }
        self.named_register(&format!("X{encoding}"))
    }

    fn arm_w_from_x(&self, register: RegisterId) -> RegisterId {
        let record = self.registers[register as usize].record.as_str();
        match record {
            "XZR" => return self.named_register("WZR"),
            "SP" => return self.named_register("WSP"),
            "FP" => return self.named_register("W29"),
            "LR" => return self.named_register("W30"),
            _ => {}
        }
        let encoding = self.registers[register as usize].encoding;
        self.named_register(&format!("W{encoding}"))
    }
}

pub fn arm_predicate(target: &Target, name: &str, operand: &ArmOperand) -> Diag {
    let (method, arguments) = template_arguments(name);
    let register_of =
        |operand: &ArmOperand| -> Option<(RegisterId, RegKind, i32, ShiftExtendOperand)> {
            match operand {
                ArmOperand::Register {
                    register,
                    kind,
                    width,
                    shift,
                    ..
                } => Some((*register, *kind, *width, *shift)),
                _ => None,
            }
        };
    let class_name = |text: &str| -> String {
        text.trim()
            .trim_start_matches("AArch64::")
            .trim_end_matches("RegClassID")
            .to_string()
    };
    match method.as_str() {
        "isCondCode" => Diag::of(matches!(operand, ArmOperand::ConditionCode(_))),
        "isAddSubImm" => {
            let (expression, shift) = match operand {
                ArmOperand::ShiftedImmediate(expression, shift) => (expression, Some(*shift)),
                ArmOperand::Immediate(expression) => (expression, None),
                _ => return Diag::No,
            };
            if let Some(shift) = shift {
                if shift != 0 && shift != 12 {
                    return Diag::No;
                }
            }
            if let Some((specifier, _)) = classify_symbol(expression) {
                return Diag::of(
                    specifier
                        .as_deref()
                        .is_some_and(|specifier| ADD_SUB_SPECIFIERS.contains(&specifier)),
                );
            }
            match shifted_value(operand, 12) {
                Some((value, _)) => Diag::of((0..=0xfff).contains(&value)),
                None => Diag::Match,
            }
        }
        "isAddSubImmNeg" => {
            if !matches!(
                operand,
                ArmOperand::Immediate(_) | ArmOperand::ShiftedImmediate(..)
            ) {
                return Diag::No;
            }
            match shifted_value(operand, 12) {
                Some((value, _)) => Diag::of(value < 0 && -value <= 0xfff),
                None => Diag::No,
            }
        }
        "isUImm6" => Diag::of(
            operand
                .constant()
                .is_some_and(|value| (0..64).contains(&value)),
        ),
        "isSImm" => {
            let width = argument_number(&arguments, 0);
            Diag::of(immediate_scaled(operand, width, 1, true, false) == Diag::Match)
        }
        "isSImmScaled" => immediate_scaled(
            operand,
            argument_number(&arguments, 0),
            argument_number(&arguments, 1),
            true,
            false,
        ),
        "isUImmScaled" => {
            let bits = argument_number(&arguments, 0);
            let scale = argument_number(&arguments, 1);
            let offset = argument_number(&arguments, 2);
            let range = arguments.get(3).is_some_and(|text| text == "true");
            if range {
                if let ArmOperand::ImmediateRange(first, last) = operand {
                    if i64::from(*last) != i64::from(*first) + offset {
                        return Diag::No;
                    }
                }
            }
            immediate_scaled(operand, bits, scale, false, range)
        }
        "isSImm9OffsetFB" => {
            let width = argument_number(&arguments, 0);
            let signed = immediate_scaled(operand, 9, 1, true, false) == Diag::Match;
            let unsigned = unsigned_offset(operand, width / 8);
            Diag::of(signed && !unsigned)
        }
        "isUImm12Offset" => Diag::of(unsigned_offset(operand, argument_number(&arguments, 0))),
        "isImmInRange" => {
            let low = argument_number(&arguments, 0);
            let high = argument_number(&arguments, 1);
            Diag::of(
                operand
                    .constant()
                    .is_some_and(|value| value >= low && value <= high),
            )
        }
        "isHinteUImm16" => Diag::of(operand.constant().is_some_and(|value| {
            (0..=65535).contains(&value)
                && !((12319..=16383).contains(&value) && (value - 12319) % 32 == 0)
        })),
        "isLogicalImm" => {
            let bytes = argument_type_bytes(&arguments);
            let Some(value) = operand.constant() else {
                return Diag::No;
            };
            let bits = bytes * 8;
            let upper: u64 = if bits >= 64 { 0 } else { u64::MAX << bits };
            let value = value as u64;
            if value & upper != 0 && value & upper != upper {
                return Diag::No;
            }
            Diag::of(is_logical_immediate(value & !upper, bits as u32))
        }
        "isSIMDImmType10" => Diag::of(operand.constant().is_some_and(is_simd_type10)),
        "isBranchTarget" => {
            let bits = argument_number(&arguments, 0);
            let Some(expression) = operand.immediate() else {
                return Diag::No;
            };
            match expression.constant_value() {
                None => Diag::Match,
                Some(value) => {
                    if value & 3 != 0 {
                        return Diag::No;
                    }
                    Diag::of(
                        value >= -((1i64 << (bits - 1)) << 2)
                            && value <= (((1i64 << (bits - 1)) - 1) << 2),
                    )
                }
            }
        }
        "isPAuthPCRelLabel16Operand" => {
            let Some(expression) = operand.immediate() else {
                return Diag::No;
            };
            match expression.constant_value() {
                None => Diag::Match,
                Some(value) => Diag::of(value & 3 == 0 && value <= 0 && value > -(1 << 18)),
            }
        }
        "isMovWSymbolG3" => Diag::of(movw_symbol(operand, &["abs_g3", "prel_g3"])),
        "isMovWSymbolG2" => Diag::of(movw_symbol(
            operand,
            &[
                "abs_g2",
                "abs_g2_s",
                "abs_g2_nc",
                "prel_g2",
                "prel_g2_nc",
                "tprel_g2",
                "dtprel_g2",
            ],
        )),
        "isMovWSymbolG1" => Diag::of(movw_symbol(
            operand,
            &[
                "abs_g1",
                "abs_g1_s",
                "abs_g1_nc",
                "prel_g1",
                "prel_g1_nc",
                "gottprel_g1",
                "tprel_g1",
                "tprel_g1_nc",
                "dtprel_g1",
                "dtprel_g1_nc",
            ],
        )),
        "isMovWSymbolG0" => Diag::of(movw_symbol(
            operand,
            &[
                "abs_g0",
                "abs_g0_s",
                "abs_g0_nc",
                "prel_g0",
                "prel_g0_nc",
                "gottprel_g0_nc",
                "tprel_g0",
                "tprel_g0_nc",
                "dtprel_g0",
                "dtprel_g0_nc",
            ],
        )),
        "isMOVZMovAlias" => {
            let width = argument_number(&arguments, 0) as u32;
            let shift = argument_number(&arguments, 1) as u32;
            let Some(expression) = operand.immediate() else {
                return Diag::No;
            };
            match expression.constant_value() {
                Some(value) => Diag::of(is_movz_alias(value as u64, shift, width)),
                None => Diag::of(shift == 0),
            }
        }
        "isMOVNMovAlias" => {
            let width = argument_number(&arguments, 0) as u32;
            let shift = argument_number(&arguments, 1) as u32;
            Diag::of(
                operand
                    .constant()
                    .is_some_and(|value| is_movn_alias(value as u64, shift, width)),
            )
        }
        "isFPImm" => Diag::of(matches!(
            operand,
            ArmOperand::FloatImmediate { bits, .. } if float_immediate(*bits).is_some()
        )),
        "isBarrier" => Diag::of(matches!(operand, ArmOperand::Barrier { no_xs: false, .. })),
        "isBarriernXS" => Diag::of(matches!(operand, ArmOperand::Barrier { no_xs: true, .. })),
        "isSysReg" => Diag::of(matches!(operand, ArmOperand::SystemRegister { .. })),
        "isMRSSystemRegister" => Diag::of(
            matches!(operand, ArmOperand::SystemRegister { read, .. } if *read != u32::MAX),
        ),
        "isMSRSystemRegister" => Diag::of(
            matches!(operand, ArmOperand::SystemRegister { write, .. } if *write != u32::MAX),
        ),
        "isSystemPStateFieldWithImm0_1" => {
            let tables = system_tables_for();
            Diag::of(matches!(
                operand,
                ArmOperand::SystemRegister { pstate, .. } if tables.pstate_one_encodings.contains(pstate)
            ))
        }
        "isSystemPStateFieldWithImm0_15" => {
            let tables = system_tables_for();
            Diag::of(matches!(
                operand,
                ArmOperand::SystemRegister { pstate, .. } if tables.pstate_fifteen_encodings.contains(pstate)
            ))
        }
        "isReg" => Diag::of(register_of(operand).is_some()),
        "isScalarReg" => Diag::of(operand.is_scalar_register()),
        "isNeonVectorReg" => Diag::of(
            register_of(operand).is_some_and(|(_, kind, _, _)| kind == RegKind::NeonVector),
        ),
        "isNeonVectorRegLo" => {
            Diag::of(register_of(operand).is_some_and(|(register, kind, _, _)| {
                kind == RegKind::NeonVector
                    && (target.in_register_class("FPR128_lo", register)
                        || target.in_register_class("FPR64_lo", register))
            }))
        }
        "isNeonVectorReg0to7" => {
            Diag::of(register_of(operand).is_some_and(|(register, kind, _, _)| {
                kind == RegKind::NeonVector && target.in_register_class("FPR128_0to7", register)
            }))
        }
        "isGPR32as64" => Diag::of(register_of(operand).is_some_and(|(register, kind, _, _)| {
            kind == RegKind::Scalar && target.in_register_class("GPR64", register)
        })),
        "isGPR64as32" => Diag::of(register_of(operand).is_some_and(|(register, kind, _, _)| {
            kind == RegKind::Scalar && target.in_register_class("GPR32", register)
        })),
        "isGPR64x8" => Diag::of(register_of(operand).is_some_and(|(register, kind, _, _)| {
            kind == RegKind::Scalar && target.in_register_class("GPR64x8Class", register)
        })),
        "isWSeqPair" => Diag::of(register_of(operand).is_some_and(|(register, kind, _, _)| {
            kind == RegKind::Scalar && target.in_register_class("WSeqPairsClass", register)
        })),
        "isXSeqPair" => Diag::of(register_of(operand).is_some_and(|(register, kind, _, _)| {
            kind == RegKind::Scalar && target.in_register_class("XSeqPairsClass", register)
        })),
        "isSyspXzrPair" => Diag::of(register_of(operand).is_some_and(|(register, kind, _, _)| {
            kind == RegKind::Scalar
                && target.in_register_class("GPR64", register)
                && target.registers[register as usize].record == "XZR"
        })),
        "isGPR64" => {
            let class = class_name(arguments.first().map(String::as_str).unwrap_or(""));
            Diag::of(register_of(operand).is_some_and(|(register, kind, _, _)| {
                kind == RegKind::Scalar && target.in_register_class(&class, register)
            }))
        }
        "isGPR64WithShiftExtend" => {
            let class = class_name(arguments.first().map(String::as_str).unwrap_or(""));
            let width = argument_number(&arguments, 1);
            let Some((register, kind, _, shift)) = register_of(operand) else {
                return Diag::No;
            };
            if kind != RegKind::Scalar {
                return Diag::No;
            }
            let in_class = target.in_register_class(&class, register);
            if in_class
                && shift.kind == ShiftExtendKind::Lsl
                && i64::from(shift.amount) == i64::from((width / 8).max(1).ilog2())
            {
                Diag::Match
            } else {
                Diag::Near
            }
        }
        "isImplicitlyTypedVectorList" => {
            let kind = vector_kind(arguments.first().map(String::as_str).unwrap_or(""));
            let count = argument_number(&arguments, 1) as u32;
            let consecutive = arguments.get(2).is_some_and(|text| text == "true");
            Diag::of(matches!(
                operand,
                ArmOperand::VectorList { count: held, elements: 0, kind: held_kind, stride, .. }
                    if *held == count && *held_kind == kind && (!consecutive || *stride == 1)
            ))
        }
        "isTypedVectorList" => {
            let kind = vector_kind(arguments.first().map(String::as_str).unwrap_or(""));
            let count = argument_number(&arguments, 1) as u32;
            let elements = argument_number(&arguments, 2) as u32;
            let width = argument_number(&arguments, 3) as u32;
            let wanted_stride = if arguments.len() > 4 {
                argument_number(&arguments, 4) as u32
            } else {
                1
            };
            Diag::of(matches!(
                operand,
                ArmOperand::VectorList { count: held, kind: held_kind, width: held_width, stride, elements: held_elements, .. }
                    if *held == count && *held_kind == kind && *held_width == width
                        && *stride == wanted_stride && *held_elements == elements
            ))
        }
        "isVectorIndex" => {
            let low = argument_number(&arguments, 0);
            let high = argument_number(&arguments, 1);
            match operand {
                ArmOperand::VectorIndex(value) => {
                    if *value >= low && *value <= high {
                        Diag::Match
                    } else {
                        Diag::Near
                    }
                }
                _ => Diag::No,
            }
        }
        "isPrefetch" => Diag::of(matches!(operand, ArmOperand::Prefetch { .. })),
        "isTIndexHint" => Diag::of(matches!(operand, ArmOperand::TIndexHint { .. })),
        "isSysCR" => Diag::of(matches!(operand, ArmOperand::SystemCr(_))),
        "isShiftExtend" => Diag::of(matches!(operand, ArmOperand::ShiftExtend(_))),
        "isShifter" => Diag::of(shifter(operand).is_some()),
        "isArithmeticShifter" => {
            let width = argument_number(&arguments, 0) as u32;
            Diag::of(shifter(operand).is_some_and(|shift| {
                matches!(
                    shift.kind,
                    ShiftExtendKind::Lsl | ShiftExtendKind::Lsr | ShiftExtendKind::Asr
                ) && shift.amount < width
            }))
        }
        "isLogicalShifter" => {
            let width = argument_number(&arguments, 0) as u32;
            Diag::of(shifter(operand).is_some_and(|shift| {
                matches!(
                    shift.kind,
                    ShiftExtendKind::Lsl
                        | ShiftExtendKind::Lsr
                        | ShiftExtendKind::Asr
                        | ShiftExtendKind::Ror
                ) && shift.amount < width
            }))
        }
        "isMovImm32Shifter" => Diag::of(shifter(operand).is_some_and(|shift| {
            shift.kind == ShiftExtendKind::Lsl && matches!(shift.amount, 0 | 16)
        })),
        "isMovImm64Shifter" => Diag::of(shifter(operand).is_some_and(|shift| {
            shift.kind == ShiftExtendKind::Lsl && matches!(shift.amount, 0 | 16 | 32 | 48)
        })),
        "isLogicalVecShifter" => Diag::of(shifter(operand).is_some_and(|shift| {
            shift.kind == ShiftExtendKind::Lsl && matches!(shift.amount, 0 | 8 | 16 | 24)
        })),
        "isLogicalVecHalfWordShifter" => Diag::of(shifter(operand).is_some_and(|shift| {
            shift.kind == ShiftExtendKind::Lsl && matches!(shift.amount, 0 | 8)
        })),
        "isMoveVecShifter" => Diag::of(matches!(operand, ArmOperand::ShiftExtend(shift)
                if shift.kind == ShiftExtendKind::Msl && matches!(shift.amount, 8 | 16))),
        "isExtend" => Diag::of(extend(operand).is_some()),
        "isExtend64" => Diag::of(extend(operand).is_some_and(|shift| {
            matches!(
                shift.kind,
                ShiftExtendKind::Uxtb
                    | ShiftExtendKind::Sxtb
                    | ShiftExtendKind::Uxth
                    | ShiftExtendKind::Sxth
                    | ShiftExtendKind::Uxtw
                    | ShiftExtendKind::Sxtw
            )
        })),
        "isExtendLSL64" => Diag::of(extend(operand).is_some_and(|shift| {
            matches!(
                shift.kind,
                ShiftExtendKind::Uxtx | ShiftExtendKind::Sxtx | ShiftExtendKind::Lsl
            )
        })),
        "isLSLImm3Shift" => Diag::of(matches!(operand, ArmOperand::ShiftExtend(shift)
                if shift.kind == ShiftExtendKind::Lsl && shift.amount <= 7)),
        "isMemXExtend" => {
            let width = argument_number(&arguments, 0);
            let log = (width / 8).max(1).ilog2();
            Diag::of(extend(operand).is_some_and(|shift| {
                matches!(shift.kind, ShiftExtendKind::Lsl | ShiftExtendKind::Sxtx)
                    && (i64::from(shift.amount) == i64::from(log) || shift.amount == 0)
            }))
        }
        "isMemWExtend" => {
            let width = argument_number(&arguments, 0);
            let log = (width / 8).max(1).ilog2();
            Diag::of(extend(operand).is_some_and(|shift| {
                matches!(shift.kind, ShiftExtendKind::Uxtw | ShiftExtendKind::Sxtw)
                    && (i64::from(shift.amount) == i64::from(log) || shift.amount == 0)
            }))
        }
        "isAdrpLabel" => {
            let Some(expression) = operand.immediate() else {
                return Diag::No;
            };
            match expression.constant_value() {
                Some(value) => {
                    let minimum = -(4096i64 * (1 << 20));
                    let maximum = 4096i64 * ((1 << 20) - 1);
                    Diag::of(value % 4096 == 0 && value >= minimum && value <= maximum)
                }
                None => Diag::Match,
            }
        }
        "isAdrLabel" => {
            let Some(expression) = operand.immediate() else {
                return Diag::No;
            };
            match expression.constant_value() {
                Some(value) => Diag::of((-(1 << 20)..(1 << 20)).contains(&value)),
                None => Diag::Match,
            }
        }
        "isComplexRotation" => {
            let angle = argument_number(&arguments, 0);
            let remainder = argument_number(&arguments, 1);
            match operand.constant() {
                Some(value) => {
                    if value % angle == remainder && (0..=270).contains(&value) {
                        Diag::Match
                    } else {
                        Diag::Near
                    }
                }
                None => Diag::No,
            }
        }
        "isExactFPImm" => {
            let tables = system_tables_for();
            let ArmOperand::FloatImmediate { bits, exact } = operand else {
                return Diag::No;
            };
            if !exact {
                return Diag::Near;
            }
            let matches_enum = |index: i64| {
                tables
                    .exact_floats
                    .get(&(index as u32))
                    .and_then(|text| text.parse::<f64>().ok())
                    .is_some_and(|value| value.to_bits() == *bits)
            };
            if (0..arguments.len()).any(|index| matches_enum(argument_number(&arguments, index))) {
                Diag::Match
            } else {
                Diag::Near
            }
        }
        "isImmRange" => Diag::of(matches!(operand, ArmOperand::ImmediateRange(..))),
        "isImm" => Diag::of(matches!(operand, ArmOperand::Immediate(_))),
        "isSVEPattern"
        | "isSVEVecLenSpecifier"
        | "isSVECpyImm"
        | "isSVEAddSubImm"
        | "isSVEPreferredLogicalImm"
        | "isSVEDataVectorRegOfWidth"
        | "isSVEPredicateVectorRegOfWidth"
        | "isSVEPredicateAsCounterRegOfWidth"
        | "isSVEPredicateOrPredicateAsCounterRegOfWidth"
        | "isSVEDataVectorRegWithShiftExtend"
        | "isFPRasZPR"
        | "isMatrixRegOperand"
        | "isMatrixTileList"
        | "isTypedVectorListInRange"
        | "isTypedVectorListStrided"
        | "isSVCR" => Diag::No,
        _ => Diag::No,
    }
}

fn system_tables_for() -> &'static SystemTables {
    system_tables(target().expect("the ARMv8 tables load"))
}

fn unsigned_offset(operand: &ArmOperand, scale: i64) -> bool {
    let Some(expression) = operand.immediate() else {
        return false;
    };
    match expression.constant_value() {
        None => match classify_symbol(expression) {
            None => true,
            Some((Some(specifier), _)) => UIMM12_SPECIFIERS.contains(&specifier.as_str()),
            Some((None, _)) => false,
        },
        Some(value) => scale > 0 && value % scale == 0 && value >= 0 && value / scale < 0x1000,
    }
}

fn shifter(operand: &ArmOperand) -> Option<ShiftExtendOperand> {
    match operand {
        ArmOperand::ShiftExtend(shift) if shift.kind.is_shift() => Some(*shift),
        _ => None,
    }
}

fn extend(operand: &ArmOperand) -> Option<ShiftExtendOperand> {
    match operand {
        ArmOperand::ShiftExtend(shift) if shift.amount <= 4 => Some(*shift),
        _ => None,
    }
}

fn vector_kind(text: &str) -> RegKind {
    match text.trim().trim_start_matches("RegKind::") {
        "NeonVector" => RegKind::NeonVector,
        "SVEDataVector" => RegKind::SveData,
        "SVEPredicateVector" => RegKind::SvePredicate,
        "SVEPredicateAsCounter" => RegKind::SvePredicateCounter,
        "Matrix" => RegKind::Matrix,
        "LookupTable" => RegKind::LookupTable,
        _ => RegKind::Scalar,
    }
}

pub fn describe_operand(target: &Target, operand: &ArmOperand) -> String {
    match operand {
        ArmOperand::Immediate(expression) => {
            format!(
                "Imm:{}",
                expression_text(target, expression, 0, Syntax::Att)
            )
        }
        ArmOperand::Register { register, .. } => {
            format!("Reg:{}", target.register_name(*register))
        }
        other => format!("{other:?}"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Success,
    NoMatch,
    Failure,
}

fn custom_parsers() -> &'static BTreeMap<String, Vec<(usize, u32)>> {
    static CELL: OnceLock<BTreeMap<String, Vec<(usize, u32)>>> = OnceLock::new();
    CELL.get_or_init(|| {
        let matcher = matcher();
        let mut table: BTreeMap<String, Vec<(usize, u32)>> = BTreeMap::new();
        for entry in &matcher.matchables {
            let mut masks: Vec<(usize, u32)> = Vec::new();
            for (index, operand) in entry.operands.iter().enumerate() {
                if matcher.classes[operand.class].parser.is_empty() || index >= 31 {
                    continue;
                }
                match masks.iter_mut().find(|(class, _)| *class == operand.class) {
                    Some((_, mask)) => *mask |= 1 << index,
                    None => masks.push((operand.class, 1 << index)),
                }
            }
            masks.sort_by(|left, right| {
                matcher.classes[left.0]
                    .name
                    .cmp(&matcher.classes[right.0].name)
            });
            let list = table
                .entry(entry.mnemonic.to_ascii_lowercase())
                .or_default();
            for pair in masks {
                if !list.contains(&pair) {
                    list.push(pair);
                }
            }
        }
        table
    })
}

impl<'s> Assembler<'s> {
    fn arm_push(&mut self, operands: &mut Vec<ParsedOperand>, start: usize, operand: ArmOperand) {
        let end = self.lexer.token.start.max(start);
        operands.push(ParsedOperand {
            kind: ParsedKind::Arm(Box::new(operand)),
            start,
            end,
        });
    }

    fn arm_push_token(&mut self, operands: &mut Vec<ParsedOperand>, text: &str, start: usize) {
        operands.push(ParsedOperand::token(text, start));
    }

    fn arm_expression(&mut self) -> Result<Expression, ()> {
        let (expression, _) = self.parse_expression()?;
        if !matches!(expression, Expression::Constant(_)) {
            if let Some(value) = self.context.evaluate_absolute(&expression, None) {
                return Ok(Expression::Constant(value));
            }
        }
        Ok(expression)
    }

    fn arm_register_by_name(&self, name: &str, kind: RegKind) -> Option<RegisterId> {
        let lower = name.to_ascii_lowercase();
        let digits = |prefix: &str| -> Option<u32> {
            lower
                .strip_prefix(prefix)
                .filter(|rest| !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit()))
                .and_then(|rest| rest.parse::<u32>().ok())
        };
        if let Some(number) = digits("v") {
            if number < 32 {
                return (kind == RegKind::NeonVector)
                    .then(|| self.target.named_register(&format!("Q{number}")))
                    .filter(|register| *register != NO_REGISTER);
            }
        }
        let reserved = digits("z").is_some_and(|number| number < 32)
            || digits("p").is_some_and(|number| number < 16)
            || digits("pn").is_some_and(|number| number < 16)
            || lower == "za"
            || lower.starts_with("za")
            || lower == "zt0";
        if reserved {
            return None;
        }
        if kind == RegKind::Scalar {
            let alias = match lower.as_str() {
                "fp" => Some("FP"),
                "lr" => Some("LR"),
                "x31" => Some("XZR"),
                "w31" => Some("WZR"),
                _ => None,
            };
            if let Some(record) = alias {
                return Some(self.target.named_register(record));
            }
            if let Some(register) = self.target.register_by_name.get(lower.as_str()) {
                return Some(*register);
            }
        }
        match self.arm_aliases.get(&lower) {
            Some((held, register)) if *held == kind => Some(*register),
            _ => None,
        }
    }

    fn arm_try_parse_scalar_register(&mut self) -> Option<RegisterId> {
        if !self.is(TokenKind::Identifier) {
            return None;
        }
        let name = self.lexer.token.text.clone();
        let register = self.arm_register_by_name(&name, RegKind::Scalar)?;
        self.lex();
        Some(register)
    }

    fn arm_vector_kind_suffix(suffix: &str) -> Option<(u32, u32)> {
        Some(match suffix.to_ascii_lowercase().as_str() {
            "" => (0, 0),
            ".1d" => (1, 64),
            ".1q" => (1, 128),
            ".2h" => (2, 16),
            ".2b" => (2, 8),
            ".2s" => (2, 32),
            ".2d" => (2, 64),
            ".4b" => (4, 8),
            ".4h" => (4, 16),
            ".4s" => (4, 32),
            ".8b" => (8, 8),
            ".8h" => (8, 16),
            ".16b" => (16, 8),
            ".b" => (0, 8),
            ".h" => (0, 16),
            ".s" => (0, 32),
            ".d" => (0, 64),
            _ => return None,
        })
    }

    fn arm_try_parse_vector_register(&mut self) -> Result<Option<(RegisterId, String)>, ()> {
        if !self.is(TokenKind::Identifier) {
            return Ok(None);
        }
        let name = self.lexer.token.text.clone();
        let head_end = name.find('.').unwrap_or(name.len());
        let Some(register) = self.arm_register_by_name(&name[..head_end], RegKind::NeonVector)
        else {
            return Ok(None);
        };
        let kind = name[head_end..].to_string();
        if !kind.is_empty() && Self::arm_vector_kind_suffix(&kind).is_none() {
            self.token_error("invalid vector kind qualifier");
            return Err(());
        }
        self.lex();
        Ok(Some((register, kind)))
    }

    fn arm_try_parse_vector_index(&mut self, operands: &mut Vec<ParsedOperand>) -> Status {
        let start = self.lexer.token.start;
        if !self.parse_optional_token(TokenKind::LeftBracket) {
            return Status::NoMatch;
        }
        let Ok(expression) = self.arm_expression() else {
            return Status::NoMatch;
        };
        let Some(value) = expression.constant_value() else {
            self.token_error("immediate value expected for vector index");
            return Status::Failure;
        };
        if self.parse_token(TokenKind::RightBracket, "']' expected") {
            return Status::Failure;
        }
        self.arm_push(operands, start, ArmOperand::VectorIndex(value));
        Status::Success
    }

    fn arm_try_parse_neon_vector_register(&mut self, operands: &mut Vec<ParsedOperand>) -> bool {
        if !self.is(TokenKind::Identifier) {
            return true;
        }
        let start = self.lexer.token.start;
        let Ok(Some((register, kind))) = self.arm_try_parse_vector_register() else {
            return true;
        };
        let Some((_, width)) = Self::arm_vector_kind_suffix(&kind) else {
            return true;
        };
        self.arm_push(
            operands,
            start,
            ArmOperand::Register {
                register,
                kind: RegKind::NeonVector,
                width: width as i32,
                equality: RegEquality::Reg,
                shift: ShiftExtendOperand::NONE,
            },
        );
        if !kind.is_empty() {
            self.arm_push_token(operands, &kind, start);
        }
        self.arm_try_parse_vector_index(operands) == Status::Failure
    }

    fn arm_try_parse_gpr_operand(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        parse_shift_extend: bool,
        equality: RegEquality,
    ) -> Status {
        let start = self.lexer.token.start;
        let Some(register) = self.arm_try_parse_scalar_register() else {
            return Status::NoMatch;
        };
        let plain = ArmOperand::Register {
            register,
            kind: RegKind::Scalar,
            width: 0,
            equality,
            shift: ShiftExtendOperand::NONE,
        };
        if !parse_shift_extend || !self.is(TokenKind::Comma) {
            self.arm_push(operands, start, plain);
            return Status::Success;
        }
        self.lex();
        let mut extension = Vec::new();
        let status = self.arm_try_parse_optional_shift_extend(&mut extension);
        if status != Status::Success {
            return status;
        }
        let Some(ParsedOperand {
            kind: ParsedKind::Arm(held),
            ..
        }) = extension.pop()
        else {
            return Status::Failure;
        };
        let ArmOperand::ShiftExtend(shift) = *held else {
            return Status::Failure;
        };
        self.arm_push(
            operands,
            start,
            ArmOperand::Register {
                register,
                kind: RegKind::Scalar,
                width: 0,
                equality,
                shift,
            },
        );
        Status::Success
    }

    fn arm_try_parse_gpr64sp0_operand(&mut self, operands: &mut Vec<ParsedOperand>) -> Status {
        let start = self.lexer.token.start;
        let Some(register) = self.arm_try_parse_scalar_register() else {
            return Status::NoMatch;
        };
        let operand = ArmOperand::Register {
            register,
            kind: RegKind::Scalar,
            width: 0,
            equality: RegEquality::Reg,
            shift: ShiftExtendOperand::NONE,
        };
        if !self.parse_optional_token(TokenKind::Comma) {
            self.arm_push(operands, start, operand);
            return Status::Success;
        }
        self.parse_optional_token(TokenKind::Hash);
        if !self.is(TokenKind::Integer) {
            let location = self.location();
            self.error(location, "index must be absent or #0");
            return Status::Failure;
        }
        let location = self.location();
        match self.arm_expression() {
            Ok(Expression::Constant(0)) => {}
            _ => {
                self.error(location, "index must be absent or #0");
                return Status::Failure;
            }
        }
        self.arm_push(operands, start, operand);
        Status::Success
    }

    fn arm_try_parse_optional_shift_extend(&mut self, operands: &mut Vec<ParsedOperand>) -> Status {
        let lower = self.lexer.token.text.to_ascii_lowercase();
        let Some(kind) = ShiftExtendKind::named(&lower) else {
            return Status::NoMatch;
        };
        let start = self.lexer.token.start;
        self.lex();
        let hash = self.parse_optional_token(TokenKind::Hash);
        if !hash && !self.is(TokenKind::Integer) {
            if kind.is_shift() {
                self.token_error("expected #imm after shift specifier");
                return Status::Failure;
            }
            self.arm_push(
                operands,
                start,
                ArmOperand::ShiftExtend(ShiftExtendOperand {
                    kind,
                    amount: 0,
                    explicit: false,
                }),
            );
            return Status::Success;
        }
        let location = self.location();
        if !self.is(TokenKind::Integer)
            && !self.is(TokenKind::LeftParenthesis)
            && !self.is(TokenKind::Identifier)
        {
            self.error(location, "expected integer shift amount");
            return Status::Failure;
        }
        let Ok(expression) = self.arm_expression() else {
            return Status::Failure;
        };
        let Some(value) = expression.constant_value() else {
            self.error(location, "expected constant '#imm' after shift specifier");
            return Status::Failure;
        };
        self.arm_push(
            operands,
            start,
            ArmOperand::ShiftExtend(ShiftExtendOperand {
                kind,
                amount: value as u32,
                explicit: true,
            }),
        );
        Status::Success
    }

    fn arm_parse_symbolic_immediate(&mut self) -> Result<Expression, ()> {
        let mut specifier: Option<String> = None;
        if self.parse_optional_token(TokenKind::Colon) {
            if !self.is(TokenKind::Identifier) {
                self.token_error("expect relocation specifier in operand after ':'");
                return Err(());
            }
            let lower = self.lexer.token.text.to_ascii_lowercase();
            if !ARM_SPECIFIERS.contains(&lower.as_str()) {
                self.token_error("expect relocation specifier in operand after ':'");
                return Err(());
            }
            specifier = Some(lower);
            self.lex();
            if self.parse_token(TokenKind::Colon, "expect ':' after relocation specifier") {
                return Err(());
            }
        }
        let expression = self.arm_expression()?;
        match specifier {
            None => Ok(expression),
            Some(specifier) => match self.apply_specifier(&expression, &specifier) {
                Some(applied) => Ok(applied),
                None => {
                    self.token_error("expected relocatable expression");
                    Err(())
                }
            },
        }
    }

    fn arm_try_parse_imm_with_optional_shift(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
    ) -> Status {
        let start = self.lexer.token.start;
        if self.is(TokenKind::Hash) {
            self.lex();
        } else if !self.is(TokenKind::Integer) {
            return Status::NoMatch;
        }
        let Ok(immediate) = self.arm_parse_symbolic_immediate() else {
            return Status::Failure;
        };
        if !self.is(TokenKind::Comma) {
            self.arm_push(operands, start, ArmOperand::Immediate(immediate));
            return Status::Success;
        }
        self.lex();
        if !self.is(TokenKind::Identifier) || !self.lexer.token.text.eq_ignore_ascii_case("lsl") {
            let location = self.location();
            self.error(location, "only 'lsl #+N' valid after immediate");
            return Status::Failure;
        }
        self.lex();
        self.parse_optional_token(TokenKind::Hash);
        if !self.is(TokenKind::Integer) {
            let location = self.location();
            self.error(location, "only 'lsl #+N' valid after immediate");
            return Status::Failure;
        }
        let amount = self.lexer.token.integer();
        if amount < 0 {
            let location = self.location();
            self.error(location, "positive shift amount required");
            return Status::Failure;
        }
        self.lex();
        if amount == 0 {
            self.arm_push(operands, start, ArmOperand::Immediate(immediate));
            return Status::Success;
        }
        self.arm_push(
            operands,
            start,
            ArmOperand::ShiftedImmediate(immediate, amount as u32),
        );
        Status::Success
    }

    fn arm_try_parse_adr_label(&mut self, operands: &mut Vec<ParsedOperand>, page: bool) -> Status {
        let start = self.lexer.token.start;
        if !page && self.is(TokenKind::LeftBracket) {
            return Status::NoMatch;
        }
        if self.is(TokenKind::Hash) {
            self.lex();
        }
        let location = self.location();
        let Ok(mut expression) = self.arm_parse_symbolic_immediate() else {
            return Status::Failure;
        };
        if let Some((specifier, _)) = classify_symbol(&expression) {
            match specifier {
                None => {
                    let name = if page { "pg_hi21" } else { "abs" };
                    if let Some(applied) = self.apply_specifier(&expression, name) {
                        expression = applied;
                    }
                }
                Some(held) => {
                    let allowed: &[&str] = if page {
                        &[
                            "pg_hi21_nc",
                            "got",
                            "got_auth",
                            "gotpage_lo15",
                            "gottprel",
                            "tlsdesc",
                            "tlsdesc_auth",
                            "pg_hi21",
                        ]
                    } else {
                        &["got_auth"]
                    };
                    if !allowed.contains(&held.as_str()) {
                        self.error(
                            location,
                            if page {
                                "page or gotpage label reference expected"
                            } else {
                                "unexpected adr label"
                            },
                        );
                        return Status::Failure;
                    }
                }
            }
        }
        self.arm_push(operands, start, ArmOperand::Immediate(expression));
        Status::Success
    }

    fn arm_try_parse_fp_immediate(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        zero_as_literal: bool,
    ) -> Status {
        let start = self.lexer.token.start;
        let hash = self.parse_optional_token(TokenKind::Hash);
        let negative = self.parse_optional_token(TokenKind::Minus);
        if !self.is(TokenKind::Real) && !self.is(TokenKind::Integer) {
            if !hash {
                return Status::NoMatch;
            }
            self.token_error("invalid floating point immediate");
            return Status::Failure;
        }
        let text = self.lexer.token.text.clone();
        if self.is(TokenKind::Integer) && text.starts_with("0x") {
            let value = self.lexer.token.integer();
            if value > 255 || negative {
                self.token_error("encoded floating point value out of range");
                return Status::Failure;
            }
            let bits = float_from_immediate(value as u32).to_bits();
            self.arm_push(
                operands,
                start,
                ArmOperand::FloatImmediate { bits, exact: true },
            );
        } else {
            let Ok(mut value) = text.parse::<f64>() else {
                self.token_error("invalid floating point representation");
                return Status::Failure;
            };
            if negative {
                value = -value;
            }
            if zero_as_literal && value.to_bits() == 0 {
                self.arm_push_token(operands, "#0", start);
                self.arm_push_token(operands, ".0", start);
            } else {
                let exact = text
                    .parse::<f64>()
                    .map(|parsed| format!("{parsed}").parse::<f64>() == Ok(parsed))
                    .unwrap_or(false);
                self.arm_push(
                    operands,
                    start,
                    ArmOperand::FloatImmediate {
                        bits: value.to_bits(),
                        exact,
                    },
                );
            }
        }
        self.lex();
        Status::Success
    }

    fn arm_try_parse_barrier(&mut self, operands: &mut Vec<ParsedOperand>, no_xs: bool) -> Status {
        let tables = system_tables_for();
        let mnemonic = self.arm_mnemonic.clone();
        let start = self.lexer.token.start;
        if self.is(TokenKind::Hash) || self.is(TokenKind::Integer) {
            self.parse_optional_token(TokenKind::Hash);
            let integer_token = self.lexer.token.clone();
            let location = self.location();
            let Ok(expression) = self.arm_expression() else {
                return Status::Failure;
            };
            let Some(value) = expression.constant_value() else {
                self.error(location, "immediate value expected for barrier operand");
                return Status::Failure;
            };
            if no_xs {
                if ![16, 20, 24, 28].contains(&value) {
                    self.error(location, "barrier operand out of range");
                    return Status::Failure;
                }
                let found = tables
                    .barriers_no_xs
                    .values()
                    .find(|(_, immediate)| i64::from(*immediate) == value);
                let Some((entry, _)) = found else {
                    return Status::Failure;
                };
                self.arm_push(
                    operands,
                    start,
                    ArmOperand::Barrier {
                        name: entry.name.clone(),
                        value: entry.encoding,
                        no_xs: true,
                    },
                );
                return Status::Success;
            }
            if mnemonic == "dsb" && value > 15 {
                self.lexer.unlex(integer_token);
                return Status::NoMatch;
            }
            if !(0..=15).contains(&value) {
                self.error(location, "barrier operand out of range");
                return Status::Failure;
            }
            let name = tables
                .barrier_names
                .get(&(value as u32))
                .cloned()
                .unwrap_or_default();
            self.arm_push(
                operands,
                start,
                ArmOperand::Barrier {
                    name,
                    value: value as u32,
                    no_xs: false,
                },
            );
            return Status::Success;
        }
        if !self.is(TokenKind::Identifier) {
            self.token_error("invalid operand for instruction");
            return Status::Failure;
        }
        let text = self.lexer.token.text.clone();
        let lower = text.to_ascii_lowercase();
        if no_xs {
            let Some((entry, _)) = tables.barriers_no_xs.get(&lower) else {
                self.token_error("invalid barrier option name");
                return Status::Failure;
            };
            self.arm_push(
                operands,
                start,
                ArmOperand::Barrier {
                    name: text,
                    value: entry.encoding,
                    no_xs: true,
                },
            );
            self.lex();
            return Status::Success;
        }
        let entry = tables.barriers.get(&lower);
        if mnemonic == "isb" && entry.is_none_or(|entry| entry.encoding != 15) {
            self.token_error("'sy' or #imm operand expected");
            return Status::Failure;
        }
        let Some(entry) = entry else {
            if mnemonic == "dsb" {
                return Status::NoMatch;
            }
            self.token_error("invalid barrier option name");
            return Status::Failure;
        };
        self.arm_push(
            operands,
            start,
            ArmOperand::Barrier {
                name: text,
                value: entry.encoding,
                no_xs: false,
            },
        );
        self.lex();
        Status::Success
    }

    fn arm_try_parse_sys_reg(&mut self, operands: &mut Vec<ParsedOperand>) -> Status {
        if !self.is(TokenKind::Identifier) {
            return Status::NoMatch;
        }
        let tables = system_tables_for();
        let text = self.lexer.token.text.clone();
        let lower = text.to_ascii_lowercase();
        let start = self.lexer.token.start;
        let (mut read, mut write) = (u32::MAX, u32::MAX);
        match tables.registers.get(&lower) {
            Some(entry) if predicates_features(&entry.requires, &self.arm_features) => {
                if entry.readable {
                    read = entry.encoding;
                }
                if entry.writeable {
                    write = entry.encoding;
                }
            }
            _ => {
                if let Some(generic) = generic_system_register(&lower) {
                    read = generic;
                    write = generic;
                }
            }
        }
        let mut pstate = u32::MAX;
        match tables.pstate_fifteen.get(&lower) {
            Some(entry) if predicates_features(&entry.requires, &self.arm_features) => {
                pstate = entry.encoding;
            }
            Some(_) => {}
            None => {
                if let Some(entry) = tables.pstate_one.get(&lower) {
                    if predicates_features(&entry.requires, &self.arm_features) {
                        pstate = entry.encoding;
                    }
                }
            }
        }
        self.arm_push(
            operands,
            start,
            ArmOperand::SystemRegister {
                name: text,
                read,
                write,
                pstate,
            },
        );
        self.lex();
        Status::Success
    }

    fn arm_try_parse_sys_cr(&mut self, operands: &mut Vec<ParsedOperand>) -> Status {
        let location = self.location();
        let start = self.lexer.token.start;
        if !self.is(TokenKind::Identifier) {
            self.error(location, "Expected cN operand where 0 <= N <= 15");
            return Status::Failure;
        }
        let text = self.lexer.token.text.clone();
        let valid = text.starts_with(['c', 'C'])
            && text[1..].bytes().all(|byte| byte.is_ascii_digit())
            && text.len() > 1
            && text[1..].parse::<u32>().is_ok_and(|number| number <= 15);
        if !valid {
            self.error(location, "Expected cN operand where 0 <= N <= 15");
            return Status::Failure;
        }
        self.lex();
        self.arm_push(
            operands,
            start,
            ArmOperand::SystemCr(text[1..].parse::<u32>().unwrap_or(0)),
        );
        Status::Success
    }

    fn arm_try_parse_prefetch(&mut self, operands: &mut Vec<ParsedOperand>) -> Status {
        let tables = system_tables_for();
        let start = self.lexer.token.start;
        if self.is(TokenKind::Hash) || self.is(TokenKind::Integer) {
            self.parse_optional_token(TokenKind::Hash);
            let Ok(expression) = self.arm_expression() else {
                return Status::Failure;
            };
            let Some(value) = expression.constant_value() else {
                self.token_error("immediate value expected for prefetch operand");
                return Status::Failure;
            };
            if !(0..=31).contains(&value) {
                self.token_error("prefetch operand out of range, [0,31] expected");
                return Status::Failure;
            }
            let name = tables
                .prefetch_names
                .get(&(value as u32))
                .cloned()
                .unwrap_or_default();
            self.arm_push(
                operands,
                start,
                ArmOperand::Prefetch {
                    name,
                    value: value as u32,
                },
            );
            return Status::Success;
        }
        if !self.is(TokenKind::Identifier) {
            self.token_error("prefetch hint expected");
            return Status::Failure;
        }
        let text = self.lexer.token.text.clone();
        let Some(entry) = tables.prefetches.get(&text.to_ascii_lowercase()) else {
            self.token_error("prefetch hint expected");
            return Status::Failure;
        };
        self.arm_push(
            operands,
            start,
            ArmOperand::Prefetch {
                name: text,
                value: entry.encoding,
            },
        );
        self.lex();
        Status::Success
    }

    fn arm_try_parse_t_index_hint(&mut self, operands: &mut Vec<ParsedOperand>) -> Status {
        let tables = system_tables_for();
        let start = self.lexer.token.start;
        if !self.is(TokenKind::Identifier) {
            self.token_error("invalid operand for instruction");
            return Status::Failure;
        }
        let text = self.lexer.token.text.clone();
        let Some(entry) = tables.hints.get(&text.to_ascii_lowercase()) else {
            self.token_error("invalid operand for instruction");
            return Status::Failure;
        };
        self.arm_push(
            operands,
            start,
            ArmOperand::TIndexHint {
                name: text,
                value: entry.encoding,
            },
        );
        self.lex();
        Status::Success
    }

    fn arm_try_parse_syp_xzr_pair(&mut self, operands: &mut Vec<ParsedOperand>) -> Status {
        let start = self.lexer.token.start;
        let saved = self.lexer.token.clone();
        let Some(register) = self.arm_try_parse_scalar_register() else {
            return Status::NoMatch;
        };
        if self.target.registers[register as usize].record != "XZR" {
            self.lexer.unlex(saved);
            return Status::NoMatch;
        }
        if self.parse_comma() {
            return Status::Failure;
        }
        let Some(second) = self.arm_try_parse_scalar_register() else {
            self.token_error("expected register operand");
            return Status::Failure;
        };
        if self.target.registers[second as usize].record != "XZR" {
            self.token_error("xzr must be followed by xzr");
            return Status::Failure;
        }
        self.arm_push(
            operands,
            start,
            ArmOperand::Register {
                register: second,
                kind: RegKind::Scalar,
                width: 0,
                equality: RegEquality::Reg,
                shift: ShiftExtendOperand::NONE,
            },
        );
        Status::Success
    }

    fn arm_try_parse_gpr_seq_pair(&mut self, operands: &mut Vec<ParsedOperand>) -> Status {
        let start = self.lexer.token.start;
        let location = self.location();
        if !self.is(TokenKind::Identifier) {
            return Status::NoMatch;
        }
        let Some(first) = self.arm_try_parse_scalar_register() else {
            return Status::NoMatch;
        };
        let target = self.target;
        let wide = target.in_register_class("GPR64", first);
        let narrow = target.in_register_class("GPR32", first);
        if !wide && !narrow {
            self.error(
                location,
                "expected first even register of a consecutive same-size even/odd register pair",
            );
            return Status::Failure;
        }
        let first_encoding = u32::from(target.registers[first as usize].encoding);
        if first_encoding % 2 != 0 || first_encoding > 30 {
            self.error(
                location,
                "expected first even register of a consecutive same-size even/odd register pair",
            );
            return Status::Failure;
        }
        if !self.is(TokenKind::Comma) {
            self.token_error("expected comma");
            return Status::Failure;
        }
        self.lex();
        let second_location = self.location();
        let Some(second) = self.arm_try_parse_scalar_register() else {
            self.error(
                second_location,
                "expected second odd register of a consecutive same-size even/odd register pair",
            );
            return Status::Failure;
        };
        let same_size = if wide {
            target.in_register_class("GPR64", second)
        } else {
            target.in_register_class("GPR32", second)
        };
        let second_encoding = u32::from(target.registers[second as usize].encoding);
        if !same_size || second_encoding != first_encoding + 1 {
            self.error(
                second_location,
                "expected second odd register of a consecutive same-size even/odd register pair",
            );
            return Status::Failure;
        }
        let pair = if wide {
            format!("X{first_encoding}_X{second_encoding}")
        } else {
            format!("W{first_encoding}_W{second_encoding}")
        };
        let register = target.named_register(&pair);
        if register == NO_REGISTER {
            self.error(location, "expected a register pair");
            return Status::Failure;
        }
        self.arm_push(
            operands,
            start,
            ArmOperand::Register {
                register,
                kind: RegKind::Scalar,
                width: 0,
                equality: RegEquality::Reg,
                shift: ShiftExtendOperand::NONE,
            },
        );
        Status::Success
    }

    fn arm_try_parse_gpr64x8(&mut self, operands: &mut Vec<ParsedOperand>) -> Status {
        let start = self.lexer.token.start;
        let location = self.location();
        let Some(register) = self.arm_try_parse_scalar_register() else {
            return Status::NoMatch;
        };
        if !self.target.in_register_class("GPR64", register) {
            self.error(
                location,
                "expected an even-numbered x-register in the range [x0,x22]",
            );
            return Status::Failure;
        }
        let encoding = u32::from(self.target.registers[register as usize].encoding);
        if encoding % 2 != 0 || encoding > 22 {
            self.error(
                location,
                "expected an even-numbered x-register in the range [x0,x22]",
            );
            return Status::Failure;
        }
        let name = format!(
            "X{}_X{}_X{}_X{}_X{}_X{}_X{}_X{}",
            encoding,
            encoding + 1,
            encoding + 2,
            encoding + 3,
            encoding + 4,
            encoding + 5,
            encoding + 6,
            encoding + 7
        );
        let tuple = self.target.named_register(&name);
        let register = if tuple == NO_REGISTER {
            register
        } else {
            tuple
        };
        self.arm_push(
            operands,
            start,
            ArmOperand::Register {
                register,
                kind: RegKind::Scalar,
                width: 0,
                equality: RegEquality::Reg,
                shift: ShiftExtendOperand::NONE,
            },
        );
        Status::Success
    }

    fn arm_try_parse_adjusted_immediate(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        addend: i64,
    ) -> Status {
        let start = self.lexer.token.start;
        let hash = self.parse_optional_token(TokenKind::Hash);
        if !hash && !self.is(TokenKind::Integer) {
            return Status::NoMatch;
        }
        let Ok(expression) = self.arm_expression() else {
            return Status::Failure;
        };
        let Some(value) = expression.constant_value() else {
            self.token_error("immediate value expected");
            return Status::Failure;
        };
        self.arm_push(
            operands,
            start,
            ArmOperand::Immediate(Expression::Constant(value + addend)),
        );
        Status::Success
    }

    fn arm_try_parse_vector_list(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        expect_match: bool,
    ) -> Status {
        if !self.is(TokenKind::LeftBrace) {
            return Status::NoMatch;
        }
        let start = self.lexer.token.start;
        let brace = self.lexer.token.clone();
        self.lex();
        let location = self.location();
        let first = match self.arm_try_parse_vector_register() {
            Ok(Some(found)) => found,
            Ok(None) => {
                if expect_match {
                    self.error(location, "vector register expected");
                    return Status::Failure;
                }
                self.lexer.unlex(brace);
                return Status::NoMatch;
            }
            Err(()) => return Status::Failure,
        };
        let (first_register, kind) = first;
        let mut previous = first_register;
        let mut count = 1u32;
        let mut stride = 1u32;
        let encoding =
            |register: RegisterId| u32::from(self.target.registers[register as usize].encoding);
        if self.parse_optional_token(TokenKind::Minus) {
            let location = self.location();
            let next = match self.arm_try_parse_vector_register() {
                Ok(Some(found)) => found,
                Ok(None) => {
                    self.error(location, "vector register expected");
                    return Status::Failure;
                }
                Err(()) => return Status::Failure,
            };
            if next.1 != kind {
                self.error(location, "mismatched register size suffix");
                return Status::Failure;
            }
            let (previous_encoding, next_encoding) = (encoding(previous), encoding(next.0));
            let space = if previous_encoding < next_encoding {
                next_encoding - previous_encoding
            } else {
                32 - (previous_encoding - next_encoding)
            };
            if space == 0 || space > 3 {
                self.error(location, "invalid number of vectors");
                return Status::Failure;
            }
            count += space;
        } else {
            let mut calculated = false;
            while self.parse_optional_token(TokenKind::Comma) {
                let location = self.location();
                let next = match self.arm_try_parse_vector_register() {
                    Ok(Some(found)) => found,
                    Ok(None) => {
                        self.error(location, "vector register expected");
                        return Status::Failure;
                    }
                    Err(()) => return Status::Failure,
                };
                if next.1 != kind {
                    self.error(location, "mismatched register size suffix");
                    return Status::Failure;
                }
                let (previous_encoding, next_encoding) = (encoding(previous), encoding(next.0));
                if !calculated {
                    stride = if previous_encoding < next_encoding {
                        next_encoding - previous_encoding
                    } else {
                        32 - (previous_encoding - next_encoding)
                    };
                    calculated = true;
                }
                if stride == 0 || next_encoding != (previous_encoding + stride) % 32 {
                    self.error(location, "registers must have the same sequential stride");
                    return Status::Failure;
                }
                previous = next.0;
                count += 1;
            }
        }
        if self.parse_token(TokenKind::RightBrace, "'}' expected") {
            return Status::Failure;
        }
        if count > 4 {
            let location = SourceLocation {
                buffer: brace.buffer,
                position: brace.start,
            };
            self.error(location, "invalid number of vectors");
            return Status::Failure;
        }
        let (elements, width) = Self::arm_vector_kind_suffix(&kind).unwrap_or((0, 0));
        self.arm_push(
            operands,
            start,
            ArmOperand::VectorList {
                register: first_register,
                count,
                stride,
                elements,
                width,
                kind: RegKind::NeonVector,
            },
        );
        if self.is(TokenKind::LeftBracket)
            && self.arm_try_parse_vector_index(operands) == Status::Failure
        {
            return Status::Failure;
        }
        Status::Success
    }

    fn arm_custom_parse_class(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        class: usize,
    ) -> Status {
        let method = self.matcher.classes[class].parser.clone();
        let (name, arguments) = split_method(&method);
        let flag = |index: usize| arguments.get(index).is_some_and(|text| text == "true");
        match name.as_str() {
            "tryParseImmWithOptionalShift" => self.arm_try_parse_imm_with_optional_shift(operands),
            "tryParseAdrLabel" => self.arm_try_parse_adr_label(operands, false),
            "tryParseAdrpLabel" => self.arm_try_parse_adr_label(operands, true),
            "tryParseFPImm" => self.arm_try_parse_fp_immediate(operands, flag(0)),
            "tryParseBarrierOperand" => self.arm_try_parse_barrier(operands, false),
            "tryParseBarriernXSOperand" => self.arm_try_parse_barrier(operands, true),
            "tryParseSysReg" => self.arm_try_parse_sys_reg(operands),
            "tryParseSysCROperand" => self.arm_try_parse_sys_cr(operands),
            "tryParsePrefetch" => self.arm_try_parse_prefetch(operands),
            "tryParseTIndexHint" => self.arm_try_parse_t_index_hint(operands),
            "tryParseSyspXzrPair" => self.arm_try_parse_syp_xzr_pair(operands),
            "tryParseGPRSeqPair" => self.arm_try_parse_gpr_seq_pair(operands),
            "tryParseGPR64x8" => self.arm_try_parse_gpr64x8(operands),
            "tryParseGPR64sp0Operand" => self.arm_try_parse_gpr64sp0_operand(operands),
            "tryParseGPROperand" => {
                let equality = match arguments.get(1).map(String::as_str) {
                    Some("RegConstraintEqualityTy::EqualsSuperReg") => RegEquality::SuperReg,
                    Some("RegConstraintEqualityTy::EqualsSubReg") => RegEquality::SubReg,
                    _ => RegEquality::Reg,
                };
                self.arm_try_parse_gpr_operand(operands, flag(0), equality)
            }
            "tryParseVectorList" => {
                if arguments.first().map(String::as_str) == Some("RegKind::NeonVector") {
                    self.arm_try_parse_vector_list(operands, flag(1))
                } else {
                    Status::NoMatch
                }
            }
            "tryParseAdjImm0_63" => {
                let addend = argument_number(&arguments, 0);
                self.arm_try_parse_adjusted_immediate(operands, addend)
            }
            _ => Status::NoMatch,
        }
    }

    fn arm_match_operand_parser(&mut self, operands: &mut Vec<ParsedOperand>) -> Status {
        let mnemonic = self.arm_mnemonic.clone();
        let Some(entries) = custom_parsers().get(&mnemonic) else {
            return Status::NoMatch;
        };
        let next = operands.len().saturating_sub(1);
        for (class, mask) in entries {
            if next >= 31 || mask & (1 << next) == 0 {
                continue;
            }
            let status = self.arm_custom_parse_class(operands, *class);
            if status != Status::NoMatch {
                return status;
            }
        }
        Status::NoMatch
    }

    fn arm_parse_condition_code(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        invert: bool,
    ) -> bool {
        let start = self.lexer.token.start;
        let text = self.lexer.token.text.clone();
        let Some(mut code) = condition_code(&text) else {
            let mut message = "invalid condition code".to_string();
            if text.eq_ignore_ascii_case("nfirst") {
                message.push_str(", did you mean nfrst?");
            }
            return self.token_error(&message);
        };
        self.lex();
        if invert {
            if code == 14 || code == 15 {
                return self
                    .token_error("condition codes AL and NV are invalid for this instruction");
            }
            code ^= 1;
        }
        self.arm_push(operands, start, ArmOperand::ConditionCode(code));
        false
    }

    fn arm_parse_register(&mut self, operands: &mut Vec<ParsedOperand>) -> bool {
        if !self.arm_try_parse_neon_vector_register(operands) {
            return false;
        }
        self.arm_try_parse_gpr_operand(operands, false, RegEquality::Reg) != Status::Success
    }

    fn arm_parse_operand(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        is_condition: bool,
        invert: bool,
    ) -> bool {
        match self.arm_match_operand_parser(operands) {
            Status::Success => return false,
            Status::Failure => return true,
            Status::NoMatch => {}
        }
        let optional_shift_extend =
            |this: &mut Self, operands: &mut Vec<ParsedOperand>, saved: Token| -> bool {
                if this.parse_optional_token(TokenKind::Comma) {
                    match this.arm_try_parse_optional_shift_extend(operands) {
                        Status::NoMatch => this.lexer.unlex(saved),
                        Status::Failure => return true,
                        Status::Success => {}
                    }
                }
                false
            };
        match self.lexer.token.kind {
            TokenKind::LeftBracket => {
                let start = self.lexer.token.start;
                self.arm_push_token(operands, "[", start);
                self.lex();
                self.arm_parse_operand(operands, false, false)
            }
            TokenKind::LeftBrace => {
                if self.arm_try_parse_vector_list(operands, true) == Status::Success
                    && self.arm_post_vector_list()
                {
                    return false;
                }
                let start = self.lexer.token.start;
                self.arm_push_token(operands, "{", start);
                self.lex();
                self.arm_parse_operand(operands, false, false)
            }
            TokenKind::Identifier => {
                if is_condition {
                    return self.arm_parse_condition_code(operands, invert);
                }
                if !self.arm_parse_register(operands) {
                    let saved = self.lexer.token.clone();
                    if self.parse_optional_token(TokenKind::Comma) {
                        match self.arm_match_operand_parser(operands) {
                            Status::NoMatch => {}
                            other => return other == Status::Failure,
                        }
                        match self.arm_try_parse_optional_shift_extend(operands) {
                            Status::NoMatch => {}
                            other => return other == Status::Failure,
                        }
                        self.lexer.unlex(saved);
                    }
                    return false;
                }
                let keyword = matches!(
                    self.arm_mnemonic.as_str(),
                    "brb"
                        | "smstart"
                        | "smstop"
                        | "gcsb"
                        | "bti"
                        | "stshh"
                        | "psb"
                        | "tsb"
                        | "shuh"
                );
                if keyword {
                    let start = self.lexer.token.start;
                    let text = self.lexer.token.text.clone();
                    let known = [
                        "c", "csync", "j", "jc", "keep", "ph", "r", "sm", "strm", "za",
                    ];
                    let lower = text.to_ascii_lowercase();
                    let token = if known.contains(&lower.as_str()) {
                        lower
                    } else {
                        text
                    };
                    self.arm_push_token(operands, &token, start);
                    self.lex();
                    return false;
                }
                let start = self.lexer.token.start;
                let Ok(mut expression) = self.arm_expression() else {
                    return true;
                };
                let operator = if self.parse_optional_token(TokenKind::Plus) {
                    Some(BinaryOperator::Add)
                } else if self.parse_optional_token(TokenKind::Minus) {
                    Some(BinaryOperator::Subtract)
                } else {
                    None
                };
                if let Some(operator) = operator {
                    let Ok((term, _)) = self.parse_primary() else {
                        return true;
                    };
                    expression = Expression::Binary {
                        operator,
                        left: Box::new(expression),
                        right: Box::new(term),
                    };
                }
                self.arm_push(operands, start, ArmOperand::Immediate(expression));
                let saved = self.lexer.token.clone();
                optional_shift_extend(self, operands, saved)
            }
            TokenKind::Integer | TokenKind::Real | TokenKind::Hash => {
                let start = self.lexer.token.start;
                self.parse_optional_token(TokenKind::Hash);
                let mut negative = false;
                if self.is(TokenKind::Minus) {
                    negative = true;
                    if self
                        .lexer
                        .peek(1)
                        .first()
                        .is_some_and(|next| next.kind == TokenKind::Real)
                    {
                        self.lex();
                    }
                }
                if self.is(TokenKind::Real) {
                    let value = self.lexer.token.text.parse::<f64>().unwrap_or(1.0);
                    let mnemonic = self.arm_mnemonic.clone();
                    if !matches!(
                        mnemonic.as_str(),
                        "fcmp"
                            | "fcmpe"
                            | "fcmeq"
                            | "fcmge"
                            | "fcmgt"
                            | "fcmle"
                            | "fcmlt"
                            | "fcmne"
                    ) {
                        return self.token_error("unexpected floating point literal");
                    } else if value != 0.0 || negative {
                        return self.token_error("expected floating-point constant #0.0");
                    }
                    self.lex();
                    self.arm_push_token(operands, "#0", start);
                    self.arm_push_token(operands, ".0", start);
                    return false;
                }
                let Ok(immediate) = self.arm_parse_symbolic_immediate() else {
                    return true;
                };
                self.arm_push(operands, start, ArmOperand::Immediate(immediate));
                let saved = self.lexer.token.clone();
                optional_shift_extend(self, operands, saved)
            }
            TokenKind::Equal => self.token_error("unexpected token in operand"),
            _ => {
                let location = self.location();
                let start = self.lexer.token.start;
                let Ok(immediate) = self.arm_parse_symbolic_immediate() else {
                    return self.error(location, "invalid operand");
                };
                self.arm_push(operands, start, ArmOperand::Immediate(immediate));
                let saved = self.lexer.token.clone();
                optional_shift_extend(self, operands, saved)
            }
        }
    }

    fn arm_post_vector_list(&mut self) -> bool {
        true
    }

    fn arm_parse_instruction(
        &mut self,
        name: &str,
        name_start: usize,
    ) -> Result<Vec<ParsedOperand>, ()> {
        let mut name = name.to_ascii_lowercase();
        if let Some(condition) = name.strip_prefix('b') {
            if matches!(
                condition,
                "eq" | "ne"
                    | "hs"
                    | "cs"
                    | "lo"
                    | "cc"
                    | "mi"
                    | "pl"
                    | "vs"
                    | "vc"
                    | "hi"
                    | "ls"
                    | "ge"
                    | "lt"
                    | "gt"
                    | "le"
                    | "al"
                    | "nv"
            ) {
                name = format!("b.{condition}");
            }
        }
        let mut operands: Vec<ParsedOperand> = Vec::new();
        let head_end = name.find('.').unwrap_or(name.len());
        let head = name[..head_end].to_string();
        self.arm_push_token(&mut operands, &head, name_start);
        self.arm_mnemonic = head.clone();
        let mut next = name.find('.');
        let mut last = head.clone();
        if (head == "b" || head == "bc") && next.is_some() {
            let start = next.unwrap_or(0);
            let end = name[start + 1..]
                .find('.')
                .map(|position| position + start + 1);
            let piece = &name[start + 1..end.unwrap_or(name.len())];
            let suffix_start = name_start + start + 1;
            let Some(code) = condition_code(piece) else {
                let mut message = "invalid condition code".to_string();
                if piece == "nfirst" {
                    message.push_str(", did you mean nfrst?");
                }
                let location = self.statement_location(suffix_start);
                self.error(location, &message);
                return Err(());
            };
            self.arm_push_token(&mut operands, ".", suffix_start);
            operands.push(ParsedOperand {
                kind: ParsedKind::Arm(Box::new(ArmOperand::ConditionCode(code))),
                start: name_start,
                end: name_start,
            });
            next = end;
            last = piece.to_string();
        }
        while let Some(start) = next {
            let end = name[start + 1..]
                .find('.')
                .map(|position| position + start + 1);
            let piece = name[start..end.unwrap_or(name.len())].to_string();
            self.arm_push_token(&mut operands, &piece, name_start + start + 1);
            last = piece;
            next = end;
        }
        let head = last;
        let condition_fourth = matches!(
            head.as_str(),
            "ccmp" | "ccmn" | "fccmp" | "fccmpe" | "fcsel" | "csel" | "csinc" | "csinv" | "csneg"
        );
        let condition_second = matches!(head.as_str(), "cset" | "csetm");
        let condition_third = matches!(head.as_str(), "cinc" | "cinv" | "cneg");
        if !self.is(TokenKind::EndOfStatement) {
            let mut count = 1;
            loop {
                if self.arm_parse_operand(
                    &mut operands,
                    (count == 4 && condition_fourth)
                        || (count == 3 && condition_third)
                        || (count == 2 && condition_second),
                    condition_second || condition_third,
                ) {
                    return Err(());
                }
                if self.is(TokenKind::RightBracket) {
                    let start = self.lexer.token.start;
                    self.lex();
                    self.arm_push_token(&mut operands, "]", start);
                }
                if self.is(TokenKind::Exclamation) {
                    let start = self.lexer.token.start;
                    self.lex();
                    self.arm_push_token(&mut operands, "!", start);
                }
                if self.is(TokenKind::RightBrace) {
                    let start = self.lexer.token.start;
                    self.lex();
                    self.arm_push_token(&mut operands, "}", start);
                }
                count += 1;
                if !self.parse_optional_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        if self.parse_token(
            TokenKind::EndOfStatement,
            "unexpected token in argument list",
        ) {
            return Err(());
        }
        Ok(operands)
    }
}

const ARM_SPECIFIERS: &[&str] = &[
    "lo12",
    "abs_g3",
    "abs_g2",
    "abs_g2_s",
    "abs_g2_nc",
    "abs_g1",
    "abs_g1_s",
    "abs_g1_nc",
    "abs_g0",
    "abs_g0_s",
    "abs_g0_nc",
    "prel_g3",
    "prel_g2",
    "prel_g2_nc",
    "prel_g1",
    "prel_g1_nc",
    "prel_g0",
    "prel_g0_nc",
    "dtprel",
    "dtprel_g2",
    "dtprel_g1",
    "dtprel_g1_nc",
    "dtprel_g0",
    "dtprel_g0_nc",
    "dtprel_hi12",
    "dtprel_lo12",
    "dtprel_lo12_nc",
    "pg_hi21_nc",
    "tprel_g2",
    "tprel_g1",
    "tprel_g1_nc",
    "tprel_g0",
    "tprel_g0_nc",
    "tprel_hi12",
    "tprel_lo12",
    "tprel_lo12_nc",
    "tlsdesc_lo12",
    "tlsdesc_auth_lo12",
    "got",
    "gotpage_lo15",
    "got_lo12",
    "got_auth",
    "got_auth_lo12",
    "gottprel",
    "gottprel_lo12",
    "gottprel_g1",
    "gottprel_g0_nc",
    "tlsdesc",
    "tlsdesc_auth",
    "secrel_lo12",
    "secrel_hi12",
];

fn predicates_features(requires: &[String], features: &BTreeSet<String>) -> bool {
    requires
        .iter()
        .all(|feature| features.contains(feature) || feature.is_empty())
}

fn generic_system_register(name: &str) -> Option<u32> {
    let rest = name.strip_prefix('s')?;
    let pieces: Vec<&str> = rest.split('_').collect();
    if pieces.len() != 5 {
        return None;
    }
    let number = |text: &str, prefix: Option<char>| -> Option<u32> {
        let text = match prefix {
            Some(prefix) => text.strip_prefix(prefix)?,
            None => text,
        };
        text.parse::<u32>().ok()
    };
    let op0 = number(pieces[0], None)?;
    let op1 = number(pieces[1], None)?;
    let crn = number(pieces[2], Some('c'))?;
    let crm = number(pieces[3], Some('c'))?;
    let op2 = number(pieces[4], None)?;
    if !(2..=3).contains(&op0) || op1 > 7 || crn > 15 || crm > 15 || op2 > 7 {
        return None;
    }
    Some(((op0 & 1) << 14) | (op1 << 11) | (crn << 7) | (crm << 3) | op2)
}
pub const ARM_DIAGNOSTICS: &[(&str, &str)] = &[
    ("MissingFeature", "instruction requires a CPU feature not currently enabled"),
    ("InvalidOperand", "invalid operand for instruction"),
    ("InvalidSuffix", "invalid type suffix for instruction"),
    ("InvalidCondCode", "expected AArch64 condition code"),
    ("AddSubRegExtendSmall", "expected '[su]xt[bhw]' with optional integer in range [0, 4]"),
    ("AddSubRegExtendLarge", "expected 'sxtx' 'uxtx' or 'lsl' with optional integer in range [0, 4]"),
    ("AddSubSecondSource", "expected compatible register, symbol or integer in range [0, 4095]"),
    ("LogicalSecondSource", "expected compatible register or logical immediate"),
    ("InvalidMovImm32Shift", "expected 'lsl' with optional integer 0 or 16"),
    ("InvalidMovImm64Shift", "expected 'lsl' with optional integer 0, 16, 32 or 48"),
    ("AddSubRegShift32", "expected 'lsl', 'lsr' or 'asr' with optional integer in range [0, 31]"),
    ("AddSubRegShift64", "expected 'lsl', 'lsr' or 'asr' with optional integer in range [0, 63]"),
    ("InvalidFPImm", "expected compatible register or floating-point constant"),
    ("InvalidMemoryIndexedSImm6", "index must be an integer in range [-32, 31]."),
    ("InvalidMemoryIndexedSImm5", "index must be an integer in range [-16, 15]."),
    ("InvalidMemoryIndexed1SImm4", "index must be an integer in range [-8, 7]."),
    ("InvalidMemoryIndexed2SImm4", "index must be a multiple of 2 in range [-16, 14]."),
    ("InvalidMemoryIndexed3SImm4", "index must be a multiple of 3 in range [-24, 21]."),
    ("InvalidMemoryIndexed4SImm4", "index must be a multiple of 4 in range [-32, 28]."),
    ("InvalidMemoryIndexed16SImm4", "index must be a multiple of 16 in range [-128, 112]."),
    ("InvalidMemoryIndexed32SImm4", "index must be a multiple of 32 in range [-256, 224]."),
    ("InvalidMemoryIndexed1SImm6", "index must be an integer in range [-32, 31]."),
    ("InvalidMemoryIndexedSImm8", "index must be an integer in range [-128, 127]."),
    ("InvalidMemoryIndexedSImm9", "index must be an integer in range [-256, 255]."),
    ("InvalidMemoryIndexed16SImm9", "index must be a multiple of 16 in range [-4096, 4080]."),
    ("InvalidMemoryIndexed8SImm10", "index must be a multiple of 8 in range [-4096, 4088]."),
    ("InvalidMemoryIndexed4SImm7", "index must be a multiple of 4 in range [-256, 252]."),
    ("InvalidMemoryIndexed8SImm7", "index must be a multiple of 8 in range [-512, 504]."),
    ("InvalidMemoryIndexed16SImm7", "index must be a multiple of 16 in range [-1024, 1008]."),
    ("InvalidMemoryIndexed8UImm5", "index must be a multiple of 8 in range [0, 248]."),
    ("InvalidMemoryIndexed8UImm3", "index must be a multiple of 8 in range [0, 56]."),
    ("InvalidMemoryIndexed4UImm5", "index must be a multiple of 4 in range [0, 124]."),
    ("InvalidMemoryIndexed2UImm5", "index must be a multiple of 2 in range [0, 62]."),
    ("InvalidMemoryIndexed8UImm6", "index must be a multiple of 8 in range [0, 504]."),
    ("InvalidMemoryIndexed16UImm6", "index must be a multiple of 16 in range [0, 1008]."),
    ("InvalidMemoryIndexed4UImm6", "index must be a multiple of 4 in range [0, 252]."),
    ("InvalidMemoryIndexed2UImm6", "index must be a multiple of 2 in range [0, 126]."),
    ("InvalidMemoryIndexed1UImm6", "index must be in range [0, 63]."),
    ("InvalidMemoryWExtend8", "expected 'uxtw' or 'sxtw' with optional shift of #0"),
    ("InvalidMemoryWExtend16", "expected 'uxtw' or 'sxtw' with optional shift of #0 or #1"),
    ("InvalidMemoryWExtend32", "expected 'uxtw' or 'sxtw' with optional shift of #0 or #2"),
    ("InvalidMemoryWExtend64", "expected 'uxtw' or 'sxtw' with optional shift of #0 or #3"),
    ("InvalidMemoryWExtend128", "expected 'uxtw' or 'sxtw' with optional shift of #0 or #4"),
    ("InvalidMemoryXExtend8", "expected 'lsl' or 'sxtx' with optional shift of #0"),
    ("InvalidMemoryXExtend16", "expected 'lsl' or 'sxtx' with optional shift of #0 or #1"),
    ("InvalidMemoryXExtend32", "expected 'lsl' or 'sxtx' with optional shift of #0 or #2"),
    ("InvalidMemoryXExtend64", "expected 'lsl' or 'sxtx' with optional shift of #0 or #3"),
    ("InvalidMemoryXExtend128", "expected 'lsl' or 'sxtx' with optional shift of #0 or #4"),
    ("InvalidMemoryIndexed1", "index must be an integer in range [0, 4095]."),
    ("InvalidMemoryIndexed2", "index must be a multiple of 2 in range [0, 8190]."),
    ("InvalidMemoryIndexed4", "index must be a multiple of 4 in range [0, 16380]."),
    ("InvalidMemoryIndexed8", "index must be a multiple of 8 in range [0, 32760]."),
    ("InvalidMemoryIndexed16", "index must be a multiple of 16 in range [0, 65520]."),
    ("InvalidImm0_0", "immediate must be 0."),
    ("InvalidImm0_1", "immediate must be an integer in range [0, 1]."),
    ("InvalidImm0_3", "immediate must be an integer in range [0, 3]."),
    ("InvalidImm0_7", "immediate must be an integer in range [0, 7]."),
    ("InvalidImm0_15", "immediate must be an integer in range [0, 15]."),
    ("InvalidImm0_31", "immediate must be an integer in range [0, 31]."),
    ("InvalidImm0_63", "immediate must be an integer in range [0, 63]."),
    ("InvalidImm0_127", "immediate must be an integer in range [0, 127]."),
    ("InvalidImm0_255", "immediate must be an integer in range [0, 255]."),
    ("InvalidImm0_65535", "immediate must be an integer in range [0, 65535]."),
    ("InvalidHinteUImm16", "immediate must be an integer in range [0, 65535], excluding values in range [12319, 16383] where (value - 12319) is a multiple of 32."),
    ("InvalidImm1_8", "immediate must be an integer in range [1, 8]."),
    ("InvalidImm1_16", "immediate must be an integer in range [1, 16]."),
    ("InvalidImm1_32", "immediate must be an integer in range [1, 32]."),
    ("InvalidImm1_64", "immediate must be an integer in range [1, 64]."),
    ("InvalidImmM1_62", "immediate must be an integer in range [-1, 62]."),
    ("InvalidMemoryIndexedRange2UImm0", "vector select offset must be the immediate range 0:1."),
    ("InvalidMemoryIndexedRange2UImm1", "vector select offset must be an immediate range of the form <immf>:<imml>, where the first immediate is a multiple of 2 in the range [0, 2], and the second immediate is immf + 1."),
    ("InvalidMemoryIndexedRange2UImm2", "vector select offset must be an immediate range of the form <immf>:<imml>, where the first immediate is a multiple of 2 in the range [0, 6] or [0, 14] depending on the instruction, and the second immediate is immf + 1."),
    ("InvalidMemoryIndexedRange2UImm3", "vector select offset must be an immediate range of the form <immf>:<imml>, where the first immediate is a multiple of 2 in the range [0, 6] or [0, 14] depending on the instruction, and the second immediate is immf + 1."),
    ("InvalidMemoryIndexedRange4UImm0", "vector select offset must be the immediate range 0:3."),
    ("InvalidMemoryIndexedRange4UImm1", "vector select offset must be an immediate range of the form <immf>:<imml>, where the first immediate is a multiple of 4 in the range [0, 4] or [0, 12] depending on the instruction, and the second immediate is immf + 3."),
    ("InvalidMemoryIndexedRange4UImm2", "vector select offset must be an immediate range of the form <immf>:<imml>, where the first immediate is a multiple of 4 in the range [0, 4] or [0, 12] depending on the instruction, and the second immediate is immf + 3."),
    ("InvalidSVEAddSubImm8", "immediate must be an integer in range [0, 255] with a shift amount of 0"),
    ("InvalidSVEAddSubImm16", "immediate must be an integer in range [0, 255] or a multiple of 256 in range [256, 65280]"),
    ("InvalidSVEAddSubImm32", "immediate must be an integer in range [0, 255] or a multiple of 256 in range [256, 65280]"),
    ("InvalidSVEAddSubImm64", "immediate must be an integer in range [0, 255] or a multiple of 256 in range [256, 65280]"),
    ("InvalidSVECpyImm8", "immediate must be an integer in range [-128, 255] with a shift amount of 0"),
    ("InvalidSVECpyImm16", "immediate must be an integer in range [-128, 127] or a multiple of 256 in range [-32768, 65280]"),
    ("InvalidSVECpyImm32", "immediate must be an integer in range [-128, 127] or a multiple of 256 in range [-32768, 32512]"),
    ("InvalidSVECpyImm64", "immediate must be an integer in range [-128, 127] or a multiple of 256 in range [-32768, 32512]"),
    ("InvalidIndexRange0_0", "expected lane specifier '[0]'"),
    ("InvalidIndexRange1_1", "expected lane specifier '[1]'"),
    ("InvalidIndexRange0_15", "vector lane must be an integer in range [0, 15]."),
    ("InvalidIndexRange0_7", "vector lane must be an integer in range [0, 7]."),
    ("InvalidIndexRange0_3", "vector lane must be an integer in range [0, 3]."),
    ("InvalidIndexRange0_1", "vector lane must be an integer in range [0, 1]."),
    ("InvalidSVEIndexRange0_63", "vector lane must be an integer in range [0, 63]."),
    ("InvalidSVEIndexRange0_31", "vector lane must be an integer in range [0, 31]."),
    ("InvalidSVEIndexRange0_15", "vector lane must be an integer in range [0, 15]."),
    ("InvalidSVEIndexRange0_7", "vector lane must be an integer in range [0, 7]."),
    ("InvalidSVEIndexRange0_3", "vector lane must be an integer in range [0, 3]."),
    ("InvalidLabel", "expected label or encodable integer pc offset"),
    ("MRS", "expected readable system register"),
    ("MSR", "expected writable system register or pstate"),
    ("InvalidSVCR", "expected writable system register or pstate"),
    ("InvalidComplexRotationEven", "complex rotation must be 0, 90, 180 or 270."),
    ("InvalidComplexRotationOdd", "complex rotation must be 90 or 270."),
    ("InvalidGPR64shifted8", "register must be x0..x30 or xzr, without shift"),
    ("InvalidGPR64shifted16", "register must be x0..x30 or xzr, with required shift 'lsl #1'"),
    ("InvalidGPR64shifted32", "register must be x0..x30 or xzr, with required shift 'lsl #2'"),
    ("InvalidGPR64shifted64", "register must be x0..x30 or xzr, with required shift 'lsl #3'"),
    ("InvalidGPR64shifted128", "register must be x0..x30 or xzr, with required shift 'lsl #4'"),
    ("InvalidGPR64NoXZRshifted8", "register must be x0..x30 without shift"),
    ("InvalidGPR64NoXZRshifted16", "register must be x0..x30 with required shift 'lsl #1'"),
    ("InvalidGPR64NoXZRshifted32", "register must be x0..x30 with required shift 'lsl #2'"),
    ("InvalidGPR64NoXZRshifted64", "register must be x0..x30 with required shift 'lsl #3'"),
    ("InvalidGPR64NoXZRshifted128", "register must be x0..x30 with required shift 'lsl #4'"),
    ("InvalidZPR32UXTW8", "invalid shift/extend specified, expected 'z[0..31].s, (uxtw|sxtw)'"),
    ("InvalidZPR32SXTW8", "invalid shift/extend specified, expected 'z[0..31].s, (uxtw|sxtw)'"),
    ("InvalidZPR32UXTW16", "invalid shift/extend specified, expected 'z[0..31].s, (uxtw|sxtw) #1'"),
    ("InvalidZPR32SXTW16", "invalid shift/extend specified, expected 'z[0..31].s, (uxtw|sxtw) #1'"),
    ("InvalidZPR32UXTW32", "invalid shift/extend specified, expected 'z[0..31].s, (uxtw|sxtw) #2'"),
    ("InvalidZPR32SXTW32", "invalid shift/extend specified, expected 'z[0..31].s, (uxtw|sxtw) #2'"),
    ("InvalidZPR32UXTW64", "invalid shift/extend specified, expected 'z[0..31].s, (uxtw|sxtw) #3'"),
    ("InvalidZPR32SXTW64", "invalid shift/extend specified, expected 'z[0..31].s, (uxtw|sxtw) #3'"),
    ("InvalidZPR64UXTW8", "invalid shift/extend specified, expected 'z[0..31].d, (uxtw|sxtw)'"),
    ("InvalidZPR64SXTW8", "invalid shift/extend specified, expected 'z[0..31].d, (uxtw|sxtw)'"),
    ("InvalidZPR64UXTW16", "invalid shift/extend specified, expected 'z[0..31].d, (lsl|uxtw|sxtw) #1'"),
    ("InvalidZPR64SXTW16", "invalid shift/extend specified, expected 'z[0..31].d, (lsl|uxtw|sxtw) #1'"),
    ("InvalidZPR64UXTW32", "invalid shift/extend specified, expected 'z[0..31].d, (lsl|uxtw|sxtw) #2'"),
    ("InvalidZPR64SXTW32", "invalid shift/extend specified, expected 'z[0..31].d, (lsl|uxtw|sxtw) #2'"),
    ("InvalidZPR64UXTW64", "invalid shift/extend specified, expected 'z[0..31].d, (lsl|uxtw|sxtw) #3'"),
    ("InvalidZPR64SXTW64", "invalid shift/extend specified, expected 'z[0..31].d, (lsl|uxtw|sxtw) #3'"),
    ("InvalidZPR32LSL8", "invalid shift/extend specified, expected 'z[0..31].s'"),
    ("InvalidZPR32LSL16", "invalid shift/extend specified, expected 'z[0..31].s, lsl #1'"),
    ("InvalidZPR32LSL32", "invalid shift/extend specified, expected 'z[0..31].s, lsl #2'"),
    ("InvalidZPR32LSL64", "invalid shift/extend specified, expected 'z[0..31].s, lsl #3'"),
    ("InvalidZPR64LSL8", "invalid shift/extend specified, expected 'z[0..31].d'"),
    ("InvalidZPR64LSL16", "invalid shift/extend specified, expected 'z[0..31].d, lsl #1'"),
    ("InvalidZPR64LSL32", "invalid shift/extend specified, expected 'z[0..31].d, lsl #2'"),
    ("InvalidZPR64LSL64", "invalid shift/extend specified, expected 'z[0..31].d, lsl #3'"),
    ("InvalidZPR0", "expected register without element width suffix"),
    ("InvalidZPR8", "invalid element width"),
    ("InvalidZPR16", "invalid element width"),
    ("InvalidZPR32", "invalid element width"),
    ("InvalidZPR64", "invalid element width"),
    ("InvalidZPR128", "invalid element width"),
    ("InvalidZPR_3b8", "Invalid restricted vector register, expected z0.b..z7.b"),
    ("InvalidZPR_3b16", "Invalid restricted vector register, expected z0.h..z7.h"),
    ("InvalidZPR_3b32", "Invalid restricted vector register, expected z0.s..z7.s"),
    ("InvalidZPR_4b8", "Invalid restricted vector register, expected z0.b..z15.b"),
    ("InvalidZPR_4b16", "Invalid restricted vector register, expected z0.h..z15.h"),
    ("InvalidZPR_4b32", "Invalid restricted vector register, expected z0.s..z15.s"),
    ("InvalidZPR_4b64", "Invalid restricted vector register, expected z0.d..z15.d"),
    ("InvalidZPRMul2_Lo8", "Invalid restricted vector register, expected even register in z0.b..z14.b"),
    ("InvalidZPRMul2_Hi8", "Invalid restricted vector register, expected even register in z16.b..z30.b"),
    ("InvalidZPRMul2_Lo16", "Invalid restricted vector register, expected even register in z0.h..z14.h"),
    ("InvalidZPRMul2_Hi16", "Invalid restricted vector register, expected even register in z16.h..z30.h"),
    ("InvalidZPRMul2_Lo32", "Invalid restricted vector register, expected even register in z0.s..z14.s"),
    ("InvalidZPRMul2_Hi32", "Invalid restricted vector register, expected even register in z16.s..z30.s"),
    ("InvalidZPRMul2_Lo64", "Invalid restricted vector register, expected even register in z0.d..z14.d"),
    ("InvalidZPRMul2_Hi64", "Invalid restricted vector register, expected even register in z16.d..z30.d"),
    ("InvalidZPR_K0", "invalid restricted vector register, expected register in z20..z23 or z28..z31"),
    ("InvalidSVEPattern", "invalid predicate pattern"),
    ("InvalidSVEPPRorPNRAnyReg", "invalid predicate register."),
    ("InvalidSVEPPRorPNRBReg", "invalid predicate register."),
    ("InvalidSVEPredicateAnyReg", "invalid predicate register."),
    ("InvalidSVEPredicateBReg", "invalid predicate register."),
    ("InvalidSVEPredicateHReg", "invalid predicate register."),
    ("InvalidSVEPredicateSReg", "invalid predicate register."),
    ("InvalidSVEPredicateDReg", "invalid predicate register."),
    ("InvalidSVEPredicate3bAnyReg", "invalid restricted predicate register, expected p0..p7 (without element suffix)"),
    ("InvalidSVEPNPredicateB_p8to15Reg", "Invalid predicate register, expected PN in range pn8..pn15 with element suffix."),
    ("InvalidSVEPNPredicateH_p8to15Reg", "Invalid predicate register, expected PN in range pn8..pn15 with element suffix."),
    ("InvalidSVEPNPredicateS_p8to15Reg", "Invalid predicate register, expected PN in range pn8..pn15 with element suffix."),
    ("InvalidSVEPNPredicateD_p8to15Reg", "Invalid predicate register, expected PN in range pn8..pn15 with element suffix."),
    ("InvalidSVEPNPredicateAny_p8to15Reg", "invalid restricted predicate-as-counter register expected pn8..pn15"),
    ("InvalidSVEPNPredicateBReg", "Invalid predicate register, expected PN in range pn0..pn15 with element suffix."),
    ("InvalidSVEPNPredicateHReg", "Invalid predicate register, expected PN in range pn0..pn15 with element suffix."),
    ("InvalidSVEPNPredicateSReg", "Invalid predicate register, expected PN in range pn0..pn15 with element suffix."),
    ("InvalidSVEPNPredicateDReg", "Invalid predicate register, expected PN in range pn0..pn15 with element suffix."),
    ("InvalidSVEVecLenSpecifier", "Invalid vector length specifier, expected VLx2 or VLx4"),
    ("InvalidSVEPredicateListMul2x8", "Invalid vector list, expected list with 2 consecutive predicate registers, where the first vector is a multiple of 2 and with correct element type"),
    ("InvalidSVEPredicateListMul2x16", "Invalid vector list, expected list with 2 consecutive predicate registers, where the first vector is a multiple of 2 and with correct element type"),
    ("InvalidSVEPredicateListMul2x32", "Invalid vector list, expected list with 2 consecutive predicate registers, where the first vector is a multiple of 2 and with correct element type"),
    ("InvalidSVEPredicateListMul2x64", "Invalid vector list, expected list with 2 consecutive predicate registers, where the first vector is a multiple of 2 and with correct element type"),
    ("InvalidSVEExactFPImmOperandHalfOne", "Invalid floating point constant, expected 0.5 or 1.0."),
    ("InvalidSVEExactFPImmOperandHalfTwo", "Invalid floating point constant, expected 0.5 or 2.0."),
    ("InvalidSVEExactFPImmOperandZeroOne", "Invalid floating point constant, expected 0.0 or 1.0."),
    ("InvalidMatrixTileVectorH8", "invalid matrix operand, expected za0h.b or za0v.b"),
    ("InvalidMatrixTileVectorV8", "invalid matrix operand, expected za0h.b or za0v.b"),
    ("InvalidMatrixTileVectorH16", "invalid matrix operand, expected za[0-1]h.h or za[0-1]v.h"),
    ("InvalidMatrixTileVectorV16", "invalid matrix operand, expected za[0-1]h.h or za[0-1]v.h"),
    ("InvalidMatrixTileVectorH32", "invalid matrix operand, expected za[0-3]h.s or za[0-3]v.s"),
    ("InvalidMatrixTileVectorV32", "invalid matrix operand, expected za[0-3]h.s or za[0-3]v.s"),
    ("InvalidMatrixTileVectorH64", "invalid matrix operand, expected za[0-7]h.d or za[0-7]v.d"),
    ("InvalidMatrixTileVectorV64", "invalid matrix operand, expected za[0-7]h.d or za[0-7]v.d"),
    ("InvalidMatrixTileVectorH128", "invalid matrix operand, expected za[0-15]h.q or za[0-15]v.q"),
    ("InvalidMatrixTileVectorV128", "invalid matrix operand, expected za[0-15]h.q or za[0-15]v.q"),
    ("InvalidMatrixTile16", "invalid matrix operand, expected za[0-1].h"),
    ("InvalidMatrixTile32", "invalid matrix operand, expected za[0-3].s"),
    ("InvalidMatrixTile64", "invalid matrix operand, expected za[0-7].d"),
    ("InvalidMatrix", "invalid matrix operand, expected za"),
    ("InvalidMatrix8", "invalid matrix operand, expected suffix .b"),
    ("InvalidMatrix16", "invalid matrix operand, expected suffix .h"),
    ("InvalidMatrix32", "invalid matrix operand, expected suffix .s"),
    ("InvalidMatrix64", "invalid matrix operand, expected suffix .d"),
    ("InvalidMatrixIndexGPR32_12_15", "operand must be a register in range [w12, w15]"),
    ("InvalidMatrixIndexGPR32_8_11", "operand must be a register in range [w8, w11]"),
    ("InvalidSVEVectorList2x8Mul2", "Invalid vector list, expected list with 2 consecutive SVE vectors, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList2x16Mul2", "Invalid vector list, expected list with 2 consecutive SVE vectors, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList2x32Mul2", "Invalid vector list, expected list with 2 consecutive SVE vectors, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList2x64Mul2", "Invalid vector list, expected list with 2 consecutive SVE vectors, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList2x128Mul2", "Invalid vector list, expected list with 2 consecutive SVE vectors, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList2x8Mul2_Lo", "Invalid vector list, expected list with 2 consecutive SVE vectors in the range z0-z14, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList2x16Mul2_Lo", "Invalid vector list, expected list with 2 consecutive SVE vectors in the range z0-z14, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList2x32Mul2_Lo", "Invalid vector list, expected list with 2 consecutive SVE vectors in the range z0-z14, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList2x64Mul2_Lo", "Invalid vector list, expected list with 2 consecutive SVE vectors in the range z0-z14, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList2x8Mul2_Hi", "Invalid vector list, expected list with 2 consecutive SVE vectors in the range z16-z30, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList2x16Mul2_Hi", "Invalid vector list, expected list with 2 consecutive SVE vectors in the range z16-z30, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList2x32Mul2_Hi", "Invalid vector list, expected list with 2 consecutive SVE vectors in the range z16-z30, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList2x64Mul2_Hi", "Invalid vector list, expected list with 2 consecutive SVE vectors in the range z16-z30, where the first vector is a multiple of 2 and with matching element types"),
    ("InvalidSVEVectorList4x8Mul4", "Invalid vector list, expected list with 4 consecutive SVE vectors, where the first vector is a multiple of 4 and with matching element types"),
    ("InvalidSVEVectorList4x16Mul4", "Invalid vector list, expected list with 4 consecutive SVE vectors, where the first vector is a multiple of 4 and with matching element types"),
    ("InvalidSVEVectorList4x32Mul4", "Invalid vector list, expected list with 4 consecutive SVE vectors, where the first vector is a multiple of 4 and with matching element types"),
    ("InvalidSVEVectorList4x64Mul4", "Invalid vector list, expected list with 4 consecutive SVE vectors, where the first vector is a multiple of 4 and with matching element types"),
    ("InvalidSVEVectorList4x128Mul4", "Invalid vector list, expected list with 4 consecutive SVE vectors, where the first vector is a multiple of 4 and with matching element types"),
    ("InvalidSVEVectorList3x0_3b", "Invalid vector list, expected list with 3 consecutive SVE vectors starting at z0-z7"),
    ("InvalidLookupTable", "Invalid lookup table, expected zt0"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArmStatus {
    Success,
    InvalidOperand,
    MnemonicFail,
    MissingFeature,
    InvalidTied,
    Diagnostic(String),
}

pub struct ArmOutcome {
    pub status: ArmStatus,
    pub instruction: Option<MachineInstruction>,
    pub error_index: Option<usize>,
    pub missing: Vec<String>,
}

fn arm_of(operand: &ParsedOperand) -> Option<&ArmOperand> {
    match &operand.kind {
        ParsedKind::Arm(held) => Some(held.as_ref()),
        _ => None,
    }
}

fn immediate_expression(expression: &Expression) -> MachineOperand {
    match expression {
        Expression::Constant(value) => MachineOperand::Immediate(*value),
        other => MachineOperand::Expression(other.clone()),
    }
}

impl Target {
    pub fn arm_render(
        &self,
        method: &str,
        operand: &ParsedOperand,
        count: usize,
        into: &mut Vec<MachineOperand>,
    ) {
        let (name, arguments) = split_method(method);
        let Some(held) = arm_of(operand) else {
            for _ in 0..count {
                into.push(MachineOperand::Register(NO_REGISTER));
            }
            return;
        };
        let push = |into: &mut Vec<MachineOperand>, value: i64| {
            into.push(MachineOperand::Immediate(value))
        };
        let shift_of =
            |operand: &ArmOperand| operand.shift_extend().unwrap_or(ShiftExtendOperand::NONE);
        match (name.as_str(), held) {
            (
                "addRegOperands"
                | "addVectorReg128Operands"
                | "addVectorRegLoOperands"
                | "addVectorReg0to7Operands",
                ArmOperand::Register { register, .. },
            ) => into.push(MachineOperand::Register(*register)),
            ("addVectorReg64Operands", ArmOperand::Register { register, .. }) => {
                let encoding = self.registers[*register as usize].encoding;
                into.push(MachineOperand::Register(
                    self.named_register(&format!("D{encoding}")),
                ))
            }
            ("addGPR32as64Operands", ArmOperand::Register { register, .. }) => {
                into.push(MachineOperand::Register(self.arm_w_from_x(*register)))
            }
            ("addGPR64as32Operands", ArmOperand::Register { register, .. }) => {
                into.push(MachineOperand::Register(self.arm_x_from_w(*register)))
            }
            ("addSyspXzrPairOperand", ArmOperand::Register { .. }) => {
                into.push(MachineOperand::Register(self.named_register("XZR")))
            }
            ("addVectorListOperands", ArmOperand::VectorList { register, .. }) => {
                let tuple = argument_number(&arguments, 1) as usize;
                let encoding = self.registers[*register as usize].encoding;
                let kind = arguments.first().map(String::as_str).unwrap_or("");
                let record = if kind.ends_with("DReg") {
                    ["D", "D", "D_D", "D_D_D", "D_D_D_D"]
                } else {
                    ["Q", "Q", "Q_Q", "Q_Q_Q", "Q_Q_Q_Q"]
                }[tuple.min(4)];
                let letter = record.split('_').next().unwrap_or("Q");
                let name = (0..tuple.max(1))
                    .map(|offset| format!("{letter}{}", (u32::from(encoding) + offset as u32) % 32))
                    .collect::<Vec<_>>()
                    .join("_");
                let found = self.named_register(&name);
                into.push(MachineOperand::Register(if found == NO_REGISTER {
                    *register
                } else {
                    found
                }));
            }
            ("addImmOperands" | "addAdrLabelOperands", ArmOperand::Immediate(expression)) => {
                into.push(immediate_expression(expression))
            }
            ("addImmWithOptionalShiftOperands", _) => {
                let shift = argument_number(&arguments, 0) as u32;
                if let Some((value, applied)) = shifted_value(held, shift) {
                    push(into, value);
                    push(into, i64::from(applied));
                } else if let ArmOperand::ShiftedImmediate(expression, amount) = held {
                    into.push(immediate_expression(expression));
                    push(into, i64::from(*amount));
                } else if let ArmOperand::Immediate(expression) = held {
                    into.push(immediate_expression(expression));
                    push(into, 0);
                }
            }
            ("addImmNegWithOptionalShiftOperands", _) => {
                let shift = argument_number(&arguments, 0) as u32;
                if let Some((value, applied)) = shifted_value(held, shift) {
                    push(into, -value);
                    push(into, i64::from(applied));
                }
            }
            ("addCondCodeOperands", ArmOperand::ConditionCode(code)) => {
                push(into, i64::from(*code))
            }
            ("addAdrpLabelOperands", ArmOperand::Immediate(expression)) => {
                match expression.constant_value() {
                    Some(value) => push(into, value >> 12),
                    None => into.push(immediate_expression(expression)),
                }
            }
            ("addUImm12OffsetOperands", ArmOperand::Immediate(expression)) => {
                let scale = argument_number(&arguments, 0).max(1);
                match expression.constant_value() {
                    Some(value) => push(into, value / scale),
                    None => into.push(MachineOperand::Expression(expression.clone())),
                }
            }
            ("addUImm6Operands", _) => push(into, held.constant().unwrap_or(0)),
            ("addImmScaledOperands", _) => {
                let scale = argument_number(&arguments, 0).max(1);
                push(into, held.constant().unwrap_or(0) / scale)
            }
            ("addImmScaledRangeOperands", ArmOperand::ImmediateRange(first, _)) => {
                let scale = argument_number(&arguments, 0).max(1);
                push(into, i64::from(*first) / scale)
            }
            ("addLogicalImmOperands" | "addLogicalImmNotOperands", _) => {
                let bytes = argument_type_bytes(&arguments);
                let bits = (bytes * 8) as u32;
                let mut value = held.constant().unwrap_or(0) as u64;
                if name == "addLogicalImmNotOperands" {
                    value = !value;
                }
                if bits < 64 {
                    value &= (1u64 << bits) - 1;
                }
                push(into, logical_immediate(value, bits).unwrap_or(0) as i64)
            }
            ("addSIMDImmType10Operands", _) => {
                push(into, encode_simd_type10(held.constant().unwrap_or(0)))
            }
            (
                "addBranchTarget26Operands"
                | "addPAuthPCRelLabel16Operands"
                | "addPCRelLabel19Operands"
                | "addPCRelLabel9Operands"
                | "addBranchTarget14Operands",
                ArmOperand::Immediate(expression),
            ) => match expression.constant_value() {
                Some(value) => push(into, value >> 2),
                None => into.push(immediate_expression(expression)),
            },
            ("addFPImmOperands", ArmOperand::FloatImmediate { bits, .. }) => {
                push(into, i64::from(float_immediate(*bits).unwrap_or(0)))
            }
            ("addExactFPImmOperands", ArmOperand::FloatImmediate { bits, .. }) => {
                let second = argument_number(&arguments, 1) as u32;
                let tables = system_tables(self_ref(self));
                let matches_second = tables
                    .exact_floats
                    .get(&second)
                    .and_then(|text| text.parse::<f64>().ok())
                    .is_some_and(|value| value.to_bits() == *bits);
                push(into, i64::from(matches_second));
            }
            ("addBarrierOperands" | "addBarriernXSOperands", ArmOperand::Barrier { value, .. }) => {
                push(into, i64::from(*value))
            }
            ("addMRSSystemRegisterOperands", ArmOperand::SystemRegister { read, .. }) => {
                push(into, i64::from(*read))
            }
            ("addMSRSystemRegisterOperands", ArmOperand::SystemRegister { write, .. }) => {
                push(into, i64::from(*write))
            }
            (
                "addSystemPStateFieldWithImm0_1Operands"
                | "addSystemPStateFieldWithImm0_15Operands",
                ArmOperand::SystemRegister { pstate, .. },
            ) => push(into, i64::from(*pstate)),
            ("addSysCROperands", ArmOperand::SystemCr(value)) => push(into, i64::from(*value)),
            ("addPrefetchOperands", ArmOperand::Prefetch { value, .. }) => {
                push(into, i64::from(*value))
            }
            ("addTIndexHintOperands", ArmOperand::TIndexHint { value, .. }) => {
                push(into, i64::from(*value))
            }
            ("addShifterOperands", _) => {
                let shift = shift_of(held);
                push(into, shift.kind.shifter_immediate(shift.amount))
            }
            ("addLSLImm3ShifterOperands", _) => push(into, i64::from(shift_of(held).amount)),
            ("addExtendOperands" | "addExtend64Operands", _) => {
                let shift = shift_of(held);
                let mut kind = shift.kind;
                if kind == ShiftExtendKind::Lsl {
                    kind = if name == "addExtendOperands" {
                        ShiftExtendKind::Uxtw
                    } else {
                        ShiftExtendKind::Uxtx
                    };
                }
                push(into, kind.arith_extend_immediate(shift.amount))
            }
            ("addMemExtendOperands" | "addMemExtend8Operands", _) => {
                let shift = shift_of(held);
                let signed = matches!(shift.kind, ShiftExtendKind::Sxtw | ShiftExtendKind::Sxtx);
                push(into, i64::from(signed));
                if name == "addMemExtendOperands" {
                    push(into, i64::from(shift.amount != 0));
                } else {
                    push(into, i64::from(shift.explicit));
                }
            }
            ("addMOVZMovAliasOperands", ArmOperand::Immediate(expression)) => {
                let shift = argument_number(&arguments, 0);
                match expression.constant_value() {
                    Some(value) => push(into, ((value as u64 >> shift) & 0xffff) as i64),
                    None => into.push(immediate_expression(expression)),
                }
            }
            ("addMOVNMovAliasOperands", _) => {
                let shift = argument_number(&arguments, 0);
                let value = held.constant().unwrap_or(0) as u64;
                push(into, ((!value >> shift) & 0xffff) as i64)
            }
            ("addComplexRotationEvenOperands", _) => push(into, held.constant().unwrap_or(0) / 90),
            ("addComplexRotationOddOperands", _) => {
                push(into, (held.constant().unwrap_or(0) - 90) / 180)
            }
            ("addVectorIndexOperands", ArmOperand::VectorIndex(value)) => push(into, *value),
            _ => {
                for _ in 0..count {
                    into.push(MachineOperand::Register(NO_REGISTER));
                }
            }
        }
    }
}

fn self_ref(target: &Target) -> &'static Target {
    let _ = target;
    super::armv8::target().expect("the ARMv8 tables load")
}

impl<'s> Assembler<'s> {
    fn arm_class_predicate(&self, class: usize) -> String {
        let held = &self.matcher.classes[class];
        if held.predicate.is_empty() {
            format!("is{}", held.name)
        } else {
            held.predicate.clone()
        }
    }

    fn arm_validate_operand(&self, operand: &ParsedOperand, class: usize) -> ArmStatus {
        let matcher = self.matcher;
        let held = &matcher.classes[class];
        let diagnostic = |held: &MatchClass| {
            if held.diagnostic.is_empty() {
                ArmStatus::InvalidOperand
            } else {
                ArmStatus::Diagnostic(held.diagnostic.clone())
            }
        };
        match held.kind {
            ClassKind::Token => {
                if let ParsedKind::Token(text) = &operand.kind {
                    if let Some(found) = matcher.classes.iter().position(|candidate| {
                        candidate.kind == ClassKind::Token && candidate.value_name == *text
                    }) {
                        if matcher.is_subclass(found, class) {
                            return ArmStatus::Success;
                        }
                    }
                }
                if let Some(value) = held.value_name.strip_prefix('#') {
                    if let (Ok(expected), Some(actual)) = (
                        value.parse::<i64>(),
                        arm_of(operand).and_then(ArmOperand::constant),
                    ) {
                        if expected == actual {
                            return ArmStatus::Success;
                        }
                    }
                }
                ArmStatus::InvalidOperand
            }
            ClassKind::Register => match arm_of(operand) {
                Some(ArmOperand::Register { register, .. })
                    if held.registers.contains(register) =>
                {
                    ArmStatus::Success
                }
                _ => diagnostic(held),
            },
            ClassKind::User => {
                let Some(arm) = arm_of(operand) else {
                    return ArmStatus::InvalidOperand;
                };
                match arm_predicate(self.target, &self.arm_class_predicate(class), arm) {
                    Diag::Match => ArmStatus::Success,
                    Diag::Near => diagnostic(held),
                    Diag::No => ArmStatus::InvalidOperand,
                }
            }
        }
    }

    fn arm_equal_registers(&self, left: &ParsedOperand, right: &ParsedOperand) -> bool {
        match (arm_of(left), arm_of(right)) {
            (
                Some(ArmOperand::VectorList {
                    register: a,
                    count: ac,
                    stride: as_,
                    ..
                }),
                Some(ArmOperand::VectorList {
                    register: b,
                    count: bc,
                    stride: bs,
                    ..
                }),
            ) => a == b && ac == bc && as_ == bs,
            (
                Some(ArmOperand::Register {
                    register: a,
                    equality: ea,
                    ..
                }),
                Some(ArmOperand::Register {
                    register: b,
                    equality: eb,
                    ..
                }),
            ) => {
                if *ea == RegEquality::Reg && *eb == RegEquality::Reg {
                    return a == b;
                }
                if *ea == RegEquality::SuperReg {
                    return self.target.arm_x_from_w(*a) == *b;
                }
                if *ea == RegEquality::SubReg {
                    return self.target.arm_w_from_x(*a) == *b;
                }
                if *eb == RegEquality::SuperReg {
                    return self.target.arm_x_from_w(*b) == *a;
                }
                if *eb == RegEquality::SubReg {
                    return self.target.arm_w_from_x(*b) == *a;
                }
                false
            }
            _ => false,
        }
    }

    fn arm_match_instruction(&self, operands: &[ParsedOperand]) -> ArmOutcome {
        let matcher = self.matcher;
        let target = self.target;
        let failure = |status: ArmStatus, error_index: Option<usize>| ArmOutcome {
            status,
            instruction: None,
            error_index,
            missing: Vec::new(),
        };
        if operands.len() > matcher.maximum_operands + 1 {
            return failure(
                ArmStatus::InvalidOperand,
                Some(matcher.maximum_operands + 1),
            );
        }
        let mnemonic = operands
            .first()
            .map(|operand| operand.token_text().to_ascii_lowercase())
            .unwrap_or_default();
        let Some((start, end)) = matcher.ranges.get(&(0, mnemonic.clone())).copied() else {
            return failure(ArmStatus::MnemonicFail, None);
        };
        let mut had_match_other_than_features = false;
        let had_match_other_than_predicate = false;
        let mut result = ArmStatus::InvalidOperand;
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
            let has_features = predicates_hold(target, &predicates, &self.arm_features);
            let mut valid = true;
            let mut actual = 1;
            for formal in 0..matcher.maximum_operands {
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
                    None => ArmStatus::InvalidOperand,
                    Some(class) => self.arm_validate_operand(&operands[actual], class),
                };
                if diagnostic == ArmStatus::Success {
                    actual += 1;
                    continue;
                }
                if !had_match_other_than_predicate
                    && (at == start || error_index.is_none_or(|held| held <= actual))
                {
                    if has_features
                        && (error_index != Some(actual) || diagnostic != ArmStatus::InvalidOperand)
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
                let required: Vec<String> = predicates
                    .iter()
                    .filter(|predicate| {
                        !predicates_hold(
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
                        target.arm_render(
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
            for conversion in &entry.conversions {
                if let Conversion::Tied { first, second, .. } = conversion {
                    if first != second && *second != usize::MAX {
                        let (Some(left), Some(right)) =
                            (operands.get(first + 1), operands.get(second + 1))
                        else {
                            continue;
                        };
                        if !self.arm_equal_registers(left, right) {
                            return failure(ArmStatus::InvalidTied, Some(second + 1));
                        }
                    }
                }
            }
            return ArmOutcome {
                status: ArmStatus::Success,
                instruction: Some(instruction),
                error_index: None,
                missing: Vec::new(),
            };
        }
        let _ = had_match_other_than_predicate;
        if !had_match_other_than_features {
            return failure(result, error_index);
        }
        ArmOutcome {
            status: ArmStatus::MissingFeature,
            instruction: None,
            error_index: Some(0),
            missing: missing.unwrap_or_default(),
        }
    }

    fn arm_diagnostic_text(name: &str) -> &'static str {
        ARM_DIAGNOSTICS
            .iter()
            .find(|(held, _)| *held == name)
            .map(|(_, text)| *text)
            .unwrap_or("invalid operand for instruction")
    }

    fn arm_rewrite_aliases(&mut self, operands: &mut Vec<ParsedOperand>) -> bool {
        let target = self.target;
        let token = operands[0].token_text().to_string();
        let start0 = operands[0].start;
        let count = operands.len();
        let width_of = |operand: &ArmOperand| -> u64 {
            match operand {
                ArmOperand::Register { register, .. }
                    if target.in_register_class("GPR64all", *register) =>
                {
                    64
                }
                _ => 32,
            }
        };
        let constant_operand = |value: u64, start: usize| ParsedOperand {
            kind: ParsedKind::Arm(Box::new(ArmOperand::Immediate(Expression::Constant(
                value as i64,
            )))),
            start,
            end: start,
        };
        if count == 4 && token == "lsl" {
            if let (Some(second), Some(third)) = (arm_of(&operands[2]), arm_of(&operands[3])) {
                if second.is_scalar_register() {
                    if let Some(value) = third.constant() {
                        let value = value as u64;
                        let wide = target.in_register_class(
                            "GPR32all",
                            match second {
                                ArmOperand::Register { register, .. } => *register,
                                _ => NO_REGISTER,
                            },
                        );
                        let (third_value, fourth_value) = if wide {
                            (
                                (32u64.wrapping_sub(value)) & 0x1f,
                                31u64.wrapping_sub(value),
                            )
                        } else {
                            (
                                (64u64.wrapping_sub(value)) & 0x3f,
                                63u64.wrapping_sub(value),
                            )
                        };
                        let third_start = operands[3].start;
                        operands[0] = ParsedOperand::token("ubfm", start0);
                        operands.push(constant_operand(fourth_value, third_start));
                        operands[3] = constant_operand(third_value, third_start);
                    }
                }
            }
        } else if count == 4 && token == "bfc" {
            let first = arm_of(&operands[1]).cloned();
            let least = arm_of(&operands[2]).and_then(ArmOperand::constant);
            let width = arm_of(&operands[3]).and_then(ArmOperand::constant);
            if let (Some(first), Some(least), Some(width)) = (first, least, width) {
                if first.is_scalar_register() {
                    let register_width = width_of(&first);
                    let (least, width) = (least as u64, width as u64);
                    let least_start = operands[2].start;
                    let width_start = operands[3].start;
                    if least >= register_width {
                        let location = self.statement_location(least_start);
                        return self.error(location, "expected integer in range [0, 31]");
                    }
                    if width < 1 || width > register_width {
                        let location = self.statement_location(width_start);
                        return self.error(location, "expected integer in range [1, 32]");
                    }
                    let immr = if register_width == 32 {
                        (32 - least) & 0x1f
                    } else {
                        (64 - least) & 0x3f
                    };
                    let imms = width - 1;
                    if immr != 0 && imms >= immr {
                        let location = self.statement_location(width_start);
                        return self.error(location, "requested insert overflows register");
                    }
                    operands[0] = ParsedOperand::token("bfm", start0);
                    let zero =
                        target.named_register(if register_width == 32 { "WZR" } else { "XZR" });
                    operands[2] = ParsedOperand {
                        kind: ParsedKind::Arm(Box::new(ArmOperand::Register {
                            register: zero,
                            kind: RegKind::Scalar,
                            width: 0,
                            equality: RegEquality::Reg,
                            shift: ShiftExtendOperand::NONE,
                        })),
                        start: 0,
                        end: 0,
                    };
                    operands[3] = constant_operand(immr, least_start);
                    operands.push(constant_operand(imms, width_start));
                }
            }
        } else if count == 5
            && matches!(
                token.as_str(),
                "bfi" | "sbfiz" | "ubfiz" | "bfxil" | "sbfx" | "ubfx"
            )
        {
            let first = arm_of(&operands[1]).cloned();
            let third = arm_of(&operands[3]).and_then(ArmOperand::constant);
            let fourth = arm_of(&operands[4]).and_then(ArmOperand::constant);
            if let (Some(first), Some(third), Some(fourth)) = (first, third, fourth) {
                if first.is_scalar_register() {
                    let register_width = width_of(&first);
                    let (third, fourth) = (third as u64, fourth as u64);
                    let third_start = operands[3].start;
                    let fourth_start = operands[4].start;
                    if third >= register_width {
                        let location = self.statement_location(third_start);
                        return self.error(location, "expected integer in range [0, 31]");
                    }
                    if fourth < 1 || fourth > register_width {
                        let location = self.statement_location(fourth_start);
                        return self.error(location, "expected integer in range [1, 32]");
                    }
                    let insert = matches!(token.as_str(), "bfi" | "sbfiz" | "ubfiz");
                    if insert {
                        let new_third = if register_width == 32 {
                            (32 - third) & 0x1f
                        } else {
                            (64 - third) & 0x3f
                        };
                        let new_fourth = fourth - 1;
                        if new_third != 0 && new_fourth >= new_third {
                            let location = self.statement_location(fourth_start);
                            return self.error(location, "requested insert overflows register");
                        }
                        operands[3] = constant_operand(new_third, third_start);
                        operands[4] = constant_operand(new_fourth, fourth_start);
                    } else {
                        let new_fourth = third + fourth - 1;
                        if new_fourth >= register_width || new_fourth < third {
                            let location = self.statement_location(fourth_start);
                            return self.error(location, "requested extract overflows register");
                        }
                        operands[4] = constant_operand(new_fourth, fourth_start);
                    }
                    let replacement = match token.as_str() {
                        "bfi" | "bfxil" => "bfm",
                        "sbfiz" | "sbfx" => "sbfm",
                        _ => "ubfm",
                    };
                    operands[0] = ParsedOperand::token(replacement, start0);
                }
            }
        }
        let count = operands.len();
        let rewrite_register = |operands: &mut Vec<ParsedOperand>, index: usize, to_wide: bool| {
            if let Some(ArmOperand::Register { register, .. }) = arm_of(&operands[index]).cloned() {
                let converted = if to_wide {
                    target.arm_x_from_w(register)
                } else {
                    target.arm_w_from_x(register)
                };
                operands[index] = ParsedOperand {
                    kind: ParsedKind::Arm(Box::new(ArmOperand::Register {
                        register: converted,
                        kind: RegKind::Scalar,
                        width: 0,
                        equality: RegEquality::Reg,
                        shift: ShiftExtendOperand::NONE,
                    })),
                    start: operands[index].start,
                    end: operands[index].end,
                };
            }
        };
        let is_wide = |operand: &ParsedOperand| {
            matches!(arm_of(operand), Some(ArmOperand::Register { register, kind: RegKind::Scalar, .. })
                if target.in_register_class("GPR64all", *register))
        };
        if count == 3 && matches!(token.as_str(), "sxtw" | "uxtw") {
            rewrite_register(operands, 2, true);
        } else if count == 3 && matches!(token.as_str(), "sxtb" | "sxth") {
            if is_wide(&operands[1]) {
                rewrite_register(operands, 2, true);
            }
        } else if count == 3 && matches!(token.as_str(), "uxtb" | "uxth") && is_wide(&operands[1]) {
            rewrite_register(operands, 1, false);
        }
        false
    }

    fn arm_match_and_emit(
        &mut self,
        location: SourceLocation,
        operands: &mut Vec<ParsedOperand>,
    ) -> bool {
        if self.arm_rewrite_aliases(operands) {
            return true;
        }
        let outcome = self.arm_match_instruction(operands);
        match outcome.status {
            ArmStatus::Success => {
                let instruction = outcome
                    .instruction
                    .unwrap_or_else(|| MachineInstruction::new(0));
                let locations = std::mem::take(&mut self.expression_locations);
                self.streamer.note_expression_locations(&locations);
                self.emit_instruction(&instruction, location);
                false
            }
            ArmStatus::MissingFeature => {
                let mut text = "instruction requires:".to_string();
                for feature in &outcome.missing {
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
            ArmStatus::MnemonicFail => self.error(location, "unrecognized instruction mnemonic"),
            ArmStatus::InvalidTied => {
                let index = outcome.error_index.unwrap_or(0);
                let at = operands
                    .get(index)
                    .map(|operand| self.statement_location(operand.start))
                    .unwrap_or(location);
                let text = match operands.get(index).and_then(arm_of) {
                    Some(ArmOperand::VectorList { .. }) => {
                        "operand must match destination register list"
                    }
                    Some(ArmOperand::Register {
                        equality: RegEquality::SubReg,
                        ..
                    }) => "operand must be 64-bit form of destination register",
                    Some(ArmOperand::Register {
                        equality: RegEquality::SuperReg,
                        ..
                    }) => "operand must be 32-bit form of destination register",
                    _ => "operand must match destination register",
                };
                self.error(at, text)
            }
            ArmStatus::InvalidOperand | ArmStatus::Diagnostic(_) => {
                let index = outcome.error_index.unwrap_or(0);
                if index >= operands.len() {
                    return self.error(location, "too few operands for instruction");
                }
                let at = self.statement_location(operands[index].start);
                let suffix = matches!(&operands[index].kind, ParsedKind::Token(text) if index > 0 && text.starts_with('.'));
                let text = match &outcome.status {
                    ArmStatus::Diagnostic(name) if !suffix => Self::arm_diagnostic_text(name),
                    _ if suffix => "invalid type suffix for instruction",
                    _ => "invalid operand for instruction",
                };
                self.error(at, text)
            }
        }
    }

    pub(super) fn arm_parse_and_match(
        &mut self,
        identifier: &str,
        identifier_token: Token,
    ) -> bool {
        let Ok(mut operands) = self.arm_parse_instruction(identifier, identifier_token.start)
        else {
            return true;
        };
        if self.options.show_instruction_operands {
            let text: Vec<String> = operands
                .iter()
                .map(|operand| match &operand.kind {
                    ParsedKind::Token(text) => format!("Token:{text}"),
                    ParsedKind::Arm(held) => describe_operand(self.target, held),
                    _ => String::new(),
                })
                .collect();
            let location = identifier_token.location();
            self.message(
                MessageKind::Note,
                location,
                &format!("parsed instruction: [{}]", text.join(", ")),
                None,
            );
        }
        if self.has_pending_error() {
            return true;
        }
        self.arm_match_and_emit(identifier_token.location(), &mut operands)
    }
}
